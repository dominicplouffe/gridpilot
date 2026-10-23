//! Caps Lock as GridPilot's `mod` key, with nothing to install.
//!
//! 1. macOS's own `hidutil` remaps Caps Lock to F18 for this login session
//!    (other mappings are kept; the previous mapping is restored on quit).
//! 2. A keyboard event tap watches F18. While it is held, every other key
//!    press becomes a GridPilot hotkey with the `mod` modifiers added, and is
//!    swallowed so the focused app never sees it. A tap on its own does
//!    nothing.
//!
//! Holding the real modifiers (e.g. ⌃⌥⌘) keeps working through the normal
//! Carbon hotkeys.

use crate::ffi::*;
use crate::hidmap;
use crate::keycodes::key_for;
use gridpilot_core::keys::{Chord, Modifiers};
use gridpilot_core::PlatformEvent;
use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::process::Command;
use std::sync::Mutex;

/// kVK_F18: what Caps Lock sends once remapped.
const F18_KEYCODE: u16 = 0x4F;
const HIDUTIL: &str = "/usr/bin/hidutil";

/// The mapping in place before GridPilot changed it, for restoring. A Mutex
/// so the panic hook and signal handler can reach it.
static ORIGINAL: Mutex<Option<hidmap::Mapping>> = Mutex::new(None);

struct Tap {
    port: CFMachPortRef,
    source: CFRunLoopSourceRef,
}

thread_local! {
    static TAP: RefCell<Option<Tap>> = const { RefCell::new(None) };
    static MODS: Cell<Modifiers> = const { Cell::new(Modifiers::NONE) };
    static HELD: Cell<bool> = const { Cell::new(false) };
    /// Hotkeys are paused (the settings page is recording a shortcut): pass
    /// Caps+key through as mod+key so the page can see it.
    static PASSTHROUGH: Cell<bool> = const { Cell::new(false) };
    /// Keys pressed while Caps Lock was down, so their key-ups are swallowed
    /// too even if Caps Lock is released first.
    static SWALLOWED: RefCell<HashSet<u16>> = RefCell::new(HashSet::new());
}

/// Turn the feature on with `mods`, or off with `None`.
pub fn configure(mods: Option<Modifiers>) -> Result<(), String> {
    match mods {
        None => {
            remove_tap();
            restore();
            Ok(())
        }
        Some(m) => {
            MODS.set(m);
            // Tap first: without it a remapped Caps Lock would do nothing at all.
            install_tap()?;
            if let Err(e) = remap() {
                remove_tap();
                return Err(e);
            }
            match is_remapped() {
                Ok(true) => {
                    eprintln!("gridpilot: Caps Lock is now your mod key (hold it and press a key; quit GridPilot to get it back)");
                    Ok(())
                }
                Ok(false) => {
                    remove_tap();
                    Err("hidutil accepted the Caps Lock remap but it isn't active".into())
                }
                Err(e) => Err(e),
            }
        }
    }
}

fn hidutil(args: &[&str]) -> Result<String, String> {
    let out = Command::new(HIDUTIL).args(args).output().map_err(|e| format!("can't run {HIDUTIL}: {e}"))?;
    if !out.status.success() {
        return Err(format!("hidutil failed: {}", String::from_utf8_lossy(&out.stderr).trim()));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn current_mapping() -> Result<hidmap::Mapping, String> {
    Ok(hidmap::parse(&hidutil(&["property", "--get", "UserKeyMapping"])?))
}

fn set_mapping(m: &[(u64, u64)]) -> Result<(), String> {
    hidutil(&["property", "--set", &hidmap::to_json(m)]).map(|_| ())
}

fn remap() -> Result<(), String> {
    let current = current_mapping()?;
    {
        let mut orig = ORIGINAL.lock().unwrap_or_else(|p| p.into_inner());
        if orig.is_none() {
            // A crashed earlier run may have left our entry behind.
            *orig = Some(hidmap::without_capslock(&current));
        }
    }
    set_mapping(&hidmap::with_capslock(&current))
}

/// Put Caps Lock back the way it was. Safe to call more than once, and from
/// the panic hook or a signal handler.
pub fn restore() {
    let orig = ORIGINAL.lock().unwrap_or_else(|p| p.into_inner()).take();
    let target = match orig {
        Some(m) => m,
        None => match current_mapping() {
            Ok(m) if m.contains(&(hidmap::CAPS_LOCK, hidmap::F18)) => hidmap::without_capslock(&m),
            _ => return,
        },
    };
    let _ = set_mapping(&target);
}

/// While hotkeys are paused, let Caps+key through as mod+key.
pub fn set_passthrough(on: bool) {
    PASSTHROUGH.set(on);
}

/// Is Caps Lock currently remapped to F18?
pub fn is_remapped() -> Result<bool, String> {
    Ok(current_mapping()?.contains(&(hidmap::CAPS_LOCK, hidmap::F18)))
}

/// `gridpilot capslock off`: undo the remap even if GridPilot isn't running.
pub fn force_off() -> Result<(), String> {
    let m = current_mapping()?;
    set_mapping(&hidmap::without_capslock(&m))
}

fn install_tap() -> Result<(), String> {
    if TAP.with(|t| t.borrow().is_some()) {
        return Ok(());
    }
    let mask = (1u64 << kCGEventKeyDown) | (1u64 << kCGEventKeyUp);
    let port = unsafe {
        CGEventTapCreate(
            kCGSessionEventTap,
            kCGHeadInsertEventTap,
            kCGEventTapOptionDefault,
            mask,
            on_event,
            std::ptr::null_mut(),
        )
    };
    if port.is_null() {
        return Err("macOS refused the keyboard hook; check that GridPilot (or the terminal running it) is allowed \
                    under System Settings > Privacy & Security > Accessibility and Input Monitoring"
            .into());
    }
    let source = unsafe { CFMachPortCreateRunLoopSource(std::ptr::null(), port, 0) };
    unsafe {
        CFRunLoopAddSource(CFRunLoopGetMain(), source, kCFRunLoopCommonModes);
        CGEventTapEnable(port, true);
    }
    TAP.with(|t| *t.borrow_mut() = Some(Tap { port, source }));
    Ok(())
}

fn remove_tap() {
    if let Some(tap) = TAP.with(|t| t.borrow_mut().take()) {
        unsafe {
            CGEventTapEnable(tap.port, false);
            CFRunLoopRemoveSource(CFRunLoopGetMain(), tap.source, kCFRunLoopCommonModes);
            CFMachPortInvalidate(tap.port);
            CFRelease(tap.source);
            CFRelease(tap.port);
        }
    }
    HELD.set(false);
    SWALLOWED.with(|s| s.borrow_mut().clear());
}

fn event_mods(event: CGEventRef) -> Modifiers {
    let flags = unsafe { CGEventGetFlags(event) };
    let mut m = Modifiers::NONE;
    for (mask, modifier) in [
        (kCGEventFlagMaskShift, Modifiers::SHIFT),
        (kCGEventFlagMaskControl, Modifiers::CTRL),
        (kCGEventFlagMaskAlternate, Modifiers::ALT),
        (kCGEventFlagMaskCommand, Modifiers::CMD),
    ] {
        if flags & mask != 0 {
            m |= modifier;
        }
    }
    m
}

extern "C" fn on_event(
    _proxy: CGEventTapProxy,
    kind: u32,
    event: CGEventRef,
    _user: *mut std::ffi::c_void,
) -> CGEventRef {
    // macOS switches a slow tap off; switch it straight back on.
    if kind == kCGEventTapDisabledByTimeout || kind == kCGEventTapDisabledByUserInput {
        TAP.with(|t| {
            if let Some(tap) = t.borrow().as_ref() {
                unsafe { CGEventTapEnable(tap.port, true) };
            }
        });
        return event;
    }
    let code = unsafe { CGEventGetIntegerValueField(event, kCGKeyboardEventKeycode) } as u16;
    let swallow = std::ptr::null_mut();
    let down = kind == kCGEventKeyDown;
    let up = kind == kCGEventKeyUp;
    if code == F18_KEYCODE && (down || up) {
        HELD.set(down);
        return swallow;
    }
    if HELD.get() && PASSTHROUGH.get() && (down || up) {
        let mut flags = unsafe { CGEventGetFlags(event) };
        for (modifier, mask) in [
            (Modifiers::SHIFT, kCGEventFlagMaskShift),
            (Modifiers::CTRL, kCGEventFlagMaskControl),
            (Modifiers::ALT, kCGEventFlagMaskAlternate),
            (Modifiers::CMD, kCGEventFlagMaskCommand),
        ] {
            if MODS.get().contains(modifier) {
                flags |= mask;
            }
        }
        unsafe { CGEventSetFlags(event, flags) };
        return event;
    }
    if down && HELD.get() {
        SWALLOWED.with(|s| s.borrow_mut().insert(code));
        if let Some(key) = key_for(code) {
            let chord = Chord { mods: MODS.get() | event_mods(event), key };
            // Return to macOS at once; the engine runs on the next turn.
            crate::app::post_event(PlatformEvent::Hotkey(chord));
        }
        return swallow;
    }
    if up && SWALLOWED.with(|s| s.borrow_mut().remove(&code)) {
        return swallow;
    }
    event
}

/// Restore Caps Lock on Ctrl-C, `kill`, logout, or a panic.
pub fn install_safety_nets() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore();
        previous(info);
    }));
    extern "C" fn on_signal(_ctx: *mut std::ffi::c_void) {
        restore();
        std::process::exit(0);
    }
    for sig in [1, 2, 15] {
        // SIGHUP, SIGINT, SIGTERM: ignore the default action and handle the
        // signal on the main queue instead, where it is safe to run hidutil.
        unsafe {
            signal(sig, SIG_IGN);
            let src = dispatch_source_create(&_dispatch_source_type_signal as *const _, sig as usize, 0, main_queue());
            if !src.is_null() {
                dispatch_source_set_event_handler_f(src, on_signal);
                dispatch_resume(src);
            }
        }
    }
}
