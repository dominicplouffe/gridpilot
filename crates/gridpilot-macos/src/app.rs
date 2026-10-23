//! The main-thread run loop that owns the engine.
//!
//! Everything that wants the engine (hot keys, the overlay, the menu, the
//! IPC socket thread, the poll timer) enqueues an [`Input`] and calls
//! [`pump`]. Inputs are handled one at a time on the main thread, so the
//! engine never needs locking and re-entrant calls simply queue up.

use crate::platform::MacPlatform;
use crate::{ax, ffi, hotkeys, statusbar};
use gridpilot_core::command::CmdResult;
use gridpilot_core::features::builtin;
use gridpilot_core::{Config, Engine, PlatformEvent};
use objc2::MainThreadMarker;
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::ffi::c_void;
use std::path::PathBuf;

pub type Reply = Box<dyn FnOnce(CmdResult) + Send>;

enum Input {
    Event(PlatformEvent),
    Command(String),
    Remote(String, Reply),
    Poll,
}

thread_local! {
    static ENGINE: RefCell<Option<Engine>> = const { RefCell::new(None) };
    static QUEUE: RefCell<VecDeque<Input>> = const { RefCell::new(VecDeque::new()) };
}

fn enqueue(input: Input) {
    QUEUE.with(|q| q.borrow_mut().push_back(input));
}

/// Run queued inputs until the queue is empty or the engine is busy.
fn pump() {
    loop {
        let handled = ENGINE.with(|cell| {
            let Ok(mut slot) = cell.try_borrow_mut() else { return false };
            let Some(engine) = slot.as_mut() else { return false };
            let Some(input) = QUEUE.with(|q| q.borrow_mut().pop_front()) else { return false };
            match input {
                Input::Event(ev) => engine.handle(ev),
                Input::Command(line) => {
                    if let Err(e) = engine.dispatch(&line) {
                        engine.warn(format!("`{line}`: {e}"));
                    }
                }
                Input::Remote(line, reply) => reply(engine.dispatch(&line)),
                Input::Poll => {
                    if let Err(e) = engine.refresh() {
                        engine.warn(e);
                    }
                }
            }
            if engine.quit_requested {
                let mtm = MainThreadMarker::new().expect("main thread");
                NSApplication::sharedApplication(mtm).terminate(None);
            }
            true
        });
        if !handled {
            break;
        }
    }
}

/// Queue a command from UI code. It runs on the next turn of the run loop,
/// never re-entrantly inside the caller.
pub fn post_command(line: String) {
    enqueue(Input::Command(line));
    after(0, pump);
}

/// Run `f` on the main thread after `ms` milliseconds.
pub fn after(ms: u64, f: impl FnOnce() + 'static) {
    // libdispatch hands the pointer back on the main queue, i.e. this thread.
    let boxed: Box<Box<dyn FnOnce()>> = Box::new(Box::new(f));
    extern "C" fn trampoline(ctx: *mut c_void) {
        let f = unsafe { Box::from_raw(ctx as *mut Box<dyn FnOnce()>) };
        f();
    }
    unsafe {
        let when = ffi::dispatch_time(ffi::DISPATCH_TIME_NOW, (ms as i64) * 1_000_000);
        ffi::dispatch_after_f(when, ffi::main_queue(), Box::into_raw(boxed) as *mut c_void, trampoline);
    }
}

/// Hands commands from other threads (the IPC socket) to the main thread.
#[derive(Clone, Copy)]
pub struct MainThreadPoster;

impl MainThreadPoster {
    pub fn post(&self, line: String, reply: Reply) {
        struct Job(String, Reply);
        extern "C" fn trampoline(ctx: *mut c_void) {
            let job = unsafe { Box::from_raw(ctx as *mut Job) };
            enqueue(Input::Remote(job.0, job.1));
            pump();
        }
        let job = Box::new(Job(line, reply));
        unsafe { ffi::dispatch_async_f(ffi::main_queue(), Box::into_raw(job) as *mut c_void, trampoline) };
    }
}

fn schedule_poll(ms: u64) {
    after(ms, move || {
        enqueue(Input::Poll);
        pump();
        schedule_poll(ms);
    });
}

/// Start GridPilot and block in the AppKit run loop.
pub fn run(
    config: Config,
    config_path: Option<PathBuf>,
    on_ready: impl FnOnce(MainThreadPoster),
) -> Result<(), String> {
    let mtm = MainThreadMarker::new().ok_or("GridPilot must be started on the main thread")?;
    let app = NSApplication::sharedApplication(mtm);
    // No Dock icon, no app menu: a menu bar utility like Divvy.
    app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);

    if !ax::is_trusted(true) {
        eprintln!(
            "gridpilot: grant Accessibility access in System Settings > Privacy & Security > Accessibility, \
             then GridPilot will pick up your windows automatically."
        );
    }

    let poll = config.general.poll_interval_ms.max(50);
    let mut engine = Engine::new(Box::new(MacPlatform::new()), config, builtin());
    engine.config_path = config_path;
    for w in engine.warnings.clone() {
        eprintln!("gridpilot: {w}");
    }
    hotkeys::set_handler(|chord| {
        enqueue(Input::Event(PlatformEvent::Hotkey(chord)));
        pump();
    });
    if let Err(e) = engine.start() {
        engine.warn(e);
    }
    ENGINE.with(|e| *e.borrow_mut() = Some(engine));

    statusbar::install(mtm);
    schedule_poll(poll);
    on_ready(MainThreadPoster);
    app.run();
    Ok(())
}
