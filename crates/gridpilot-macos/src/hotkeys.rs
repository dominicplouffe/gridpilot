//! Global hot keys through Carbon's `RegisterEventHotKey`. Unlike an event
//! tap this needs no Input Monitoring permission, and the key press is
//! swallowed so the focused app never sees it.

use crate::ffi::*;
use crate::keycodes::code_for;
use gridpilot_core::keys::{Chord, Modifiers};
use std::cell::RefCell;
use std::ffi::c_void;

const SIGNATURE: u32 = four_cc(b"GPLT");

type Handler = Box<dyn Fn(Chord)>;

thread_local! {
    static REGISTERED: RefCell<Vec<(EventHotKeyRef, Chord)>> = const { RefCell::new(Vec::new()) };
    static HANDLER: RefCell<Option<Handler>> = const { RefCell::new(None) };
    static INSTALLED: RefCell<bool> = const { RefCell::new(false) };
}

fn carbon_mods(m: Modifiers) -> u32 {
    let mut out = 0;
    if m.contains(Modifiers::CMD) {
        out |= cmdKey;
    }
    if m.contains(Modifiers::SHIFT) {
        out |= shiftKey;
    }
    if m.contains(Modifiers::ALT) {
        out |= optionKey;
    }
    if m.contains(Modifiers::CTRL) {
        out |= controlKey;
    }
    out
}

extern "C" fn on_hotkey(_next: EventHandlerCallRef, event: EventRef, _user: *mut c_void) -> OSStatus {
    let mut id = EventHotKeyID::default();
    let status = unsafe {
        GetEventParameter(
            event,
            kEventParamDirectObject,
            typeEventHotKeyID,
            std::ptr::null_mut(),
            std::mem::size_of::<EventHotKeyID>(),
            std::ptr::null_mut(),
            &mut id as *mut _ as *mut c_void,
        )
    };
    if status != 0 || id.signature != SIGNATURE {
        return status;
    }
    let chord = REGISTERED.with(|r| r.borrow().get(id.id as usize).map(|(_, c)| *c));
    if let Some(chord) = chord {
        HANDLER.with(|h| {
            if let Some(h) = h.borrow().as_ref() {
                h(chord);
            }
        });
    }
    0
}

/// Set the callback for presses. Must be called on the main thread.
pub fn set_handler(f: impl Fn(Chord) + 'static) {
    HANDLER.with(|h| *h.borrow_mut() = Some(Box::new(f)));
    INSTALLED.with(|i| {
        if *i.borrow() {
            return;
        }
        let spec = EventTypeSpec { event_class: kEventClassKeyboard, event_kind: kEventHotKeyPressed };
        let mut handler_ref: EventHandlerRef = std::ptr::null_mut();
        unsafe {
            InstallEventHandler(
                GetApplicationEventTarget(),
                on_hotkey,
                1,
                &spec,
                std::ptr::null_mut(),
                &mut handler_ref,
            );
        }
        *i.borrow_mut() = true;
    });
}

/// Replace all registered hot keys. Returns chords that could not be
/// registered (already taken by another app, or no key code).
pub fn register(chords: &[Chord]) -> Vec<Chord> {
    let mut failed = Vec::new();
    REGISTERED.with(|r| {
        let mut r = r.borrow_mut();
        for (h, _) in r.drain(..) {
            unsafe { UnregisterEventHotKey(h) };
        }
        for chord in chords {
            let Some(code) = code_for(chord.key) else {
                failed.push(*chord);
                continue;
            };
            let id = EventHotKeyID { signature: SIGNATURE, id: r.len() as u32 };
            let mut out: EventHotKeyRef = std::ptr::null_mut();
            let status = unsafe {
                RegisterEventHotKey(code as u32, carbon_mods(chord.mods), id, GetApplicationEventTarget(), 0, &mut out)
            };
            if status == 0 && !out.is_null() {
                r.push((out, *chord));
            } else {
                failed.push(*chord);
            }
        }
    });
    failed
}
