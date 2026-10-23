//! Hand-written bindings for the few C APIs GridPilot needs: Core
//! Foundation basics, the Accessibility API, Carbon hot keys and
//! libdispatch. Kept tiny and in one place on purpose.

#![allow(non_upper_case_globals, non_camel_case_types, dead_code)]

use std::ffi::c_void;

pub type CFTypeRef = *const c_void;
pub type CFStringRef = *const c_void;
pub type CFArrayRef = *const c_void;
pub type CFDictionaryRef = *const c_void;
pub type CFBooleanRef = *const c_void;
pub type CFAllocatorRef = *const c_void;
pub type CFIndex = isize;
pub type CFTypeID = usize;
pub type Boolean = u8;

pub type AXUIElementRef = *const c_void;
pub type AXValueRef = *const c_void;
pub type AXError = i32;
pub const kAXErrorSuccess: AXError = 0;
pub const kAXValueCGPointType: u32 = 1;
pub const kAXValueCGSizeType: u32 = 2;

pub type pid_t = i32;
pub type OSStatus = i32;

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct CGPoint {
    pub x: f64,
    pub y: f64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct CGSize {
    pub width: f64,
    pub height: f64,
}

pub const kCFStringEncodingUTF8: u32 = 0x0800_0100;

#[repr(C)]
pub struct CFDictionaryKeyCallBacks {
    _private: [u8; 0],
}
#[repr(C)]
pub struct CFDictionaryValueCallBacks {
    _private: [u8; 0],
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    pub static kCFBooleanTrue: CFBooleanRef;
    pub static kCFBooleanFalse: CFBooleanRef;
    pub static kCFTypeDictionaryKeyCallBacks: CFDictionaryKeyCallBacks;
    pub static kCFTypeDictionaryValueCallBacks: CFDictionaryValueCallBacks;

    pub fn CFRelease(cf: CFTypeRef);
    pub fn CFRetain(cf: CFTypeRef) -> CFTypeRef;
    pub fn CFGetTypeID(cf: CFTypeRef) -> CFTypeID;
    pub fn CFStringGetTypeID() -> CFTypeID;
    pub fn CFBooleanGetTypeID() -> CFTypeID;
    pub fn CFArrayGetTypeID() -> CFTypeID;
    pub fn CFStringCreateWithBytes(
        alloc: CFAllocatorRef,
        bytes: *const u8,
        len: CFIndex,
        encoding: u32,
        is_external: Boolean,
    ) -> CFStringRef;
    pub fn CFStringGetLength(s: CFStringRef) -> CFIndex;
    pub fn CFStringGetMaximumSizeForEncoding(len: CFIndex, encoding: u32) -> CFIndex;
    pub fn CFStringGetCString(s: CFStringRef, buf: *mut u8, size: CFIndex, encoding: u32) -> Boolean;
    pub fn CFArrayGetCount(a: CFArrayRef) -> CFIndex;
    pub fn CFArrayGetValueAtIndex(a: CFArrayRef, i: CFIndex) -> CFTypeRef;
    pub fn CFBooleanGetValue(b: CFBooleanRef) -> Boolean;
    pub fn CFDictionaryCreate(
        alloc: CFAllocatorRef,
        keys: *const CFTypeRef,
        values: *const CFTypeRef,
        n: CFIndex,
        key_cb: *const CFDictionaryKeyCallBacks,
        value_cb: *const CFDictionaryValueCallBacks,
    ) -> CFDictionaryRef;
}

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    pub fn AXIsProcessTrustedWithOptions(options: CFDictionaryRef) -> Boolean;
    pub fn AXUIElementCreateApplication(pid: pid_t) -> AXUIElementRef;
    pub fn AXUIElementCopyAttributeValue(el: AXUIElementRef, attr: CFStringRef, value: *mut CFTypeRef) -> AXError;
    pub fn AXUIElementSetAttributeValue(el: AXUIElementRef, attr: CFStringRef, value: CFTypeRef) -> AXError;
    pub fn AXUIElementIsAttributeSettable(el: AXUIElementRef, attr: CFStringRef, settable: *mut Boolean) -> AXError;
    pub fn AXUIElementPerformAction(el: AXUIElementRef, action: CFStringRef) -> AXError;
    pub fn AXUIElementSetMessagingTimeout(el: AXUIElementRef, seconds: f32) -> AXError;
    pub fn AXValueCreate(kind: u32, value: *const c_void) -> AXValueRef;
    pub fn AXValueGetValue(v: AXValueRef, kind: u32, out: *mut c_void) -> Boolean;
    /// Private but stable since 10.x; used by yabai, AeroSpace, Hammerspoon.
    pub fn _AXUIElementGetWindow(el: AXUIElementRef, id: *mut u32) -> AXError;
    pub fn CGWarpMouseCursorPosition(p: CGPoint) -> i32;
}

// ----- Quartz event taps -------------------------------------------------

pub type CFMachPortRef = *mut c_void;
pub type CFRunLoopRef = *mut c_void;
pub type CFRunLoopSourceRef = *mut c_void;
pub type CGEventRef = *mut c_void;
pub type CGEventTapProxy = *mut c_void;
pub type CGEventTapCallBack = extern "C" fn(CGEventTapProxy, u32, CGEventRef, *mut c_void) -> CGEventRef;

pub const kCGSessionEventTap: u32 = 1;
pub const kCGHeadInsertEventTap: u32 = 0;
pub const kCGEventTapOptionDefault: u32 = 0;
pub const kCGEventKeyDown: u32 = 10;
pub const kCGEventKeyUp: u32 = 11;
pub const kCGEventTapDisabledByTimeout: u32 = 0xFFFF_FFFE;
pub const kCGEventTapDisabledByUserInput: u32 = 0xFFFF_FFFF;
pub const kCGKeyboardEventKeycode: u32 = 9;
pub const kCGEventFlagMaskShift: u64 = 0x0002_0000;
pub const kCGEventFlagMaskControl: u64 = 0x0004_0000;
pub const kCGEventFlagMaskAlternate: u64 = 0x0008_0000;
pub const kCGEventFlagMaskCommand: u64 = 0x0010_0000;

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    pub fn CGEventTapCreate(
        tap: u32,
        place: u32,
        options: u32,
        events_of_interest: u64,
        callback: CGEventTapCallBack,
        user_info: *mut c_void,
    ) -> CFMachPortRef;
    pub fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);
    pub fn CGEventGetIntegerValueField(event: CGEventRef, field: u32) -> i64;
    pub fn CGEventGetFlags(event: CGEventRef) -> u64;
    pub fn CGEventSetFlags(event: CGEventRef, flags: u64);
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    pub static kCFRunLoopCommonModes: CFStringRef;
    pub fn CFMachPortCreateRunLoopSource(
        alloc: CFAllocatorRef,
        port: CFMachPortRef,
        order: CFIndex,
    ) -> CFRunLoopSourceRef;
    pub fn CFMachPortInvalidate(port: CFMachPortRef);
    pub fn CFRunLoopGetMain() -> CFRunLoopRef;
    pub fn CFRunLoopAddSource(rl: CFRunLoopRef, source: CFRunLoopSourceRef, mode: CFStringRef);
    pub fn CFRunLoopRemoveSource(rl: CFRunLoopRef, source: CFRunLoopSourceRef, mode: CFStringRef);
}

// ----- signals via libdispatch -------------------------------------------

extern "C" {
    pub static _dispatch_source_type_signal: c_void;
    pub fn dispatch_source_create(
        kind: *const c_void,
        handle: usize,
        mask: usize,
        queue: *const dispatch_queue_s,
    ) -> *mut c_void;
    pub fn dispatch_source_set_event_handler_f(source: *mut c_void, handler: extern "C" fn(*mut c_void));
    pub fn dispatch_resume(object: *mut c_void);
    pub fn signal(sig: i32, handler: usize) -> usize;
}
pub const SIG_IGN: usize = 1;

// ----- Carbon hot keys --------------------------------------------------

pub type EventTargetRef = *mut c_void;
pub type EventHandlerRef = *mut c_void;
pub type EventHandlerCallRef = *mut c_void;
pub type EventRef = *mut c_void;
pub type EventHotKeyRef = *mut c_void;
pub type EventHandlerUPP = extern "C" fn(EventHandlerCallRef, EventRef, *mut c_void) -> OSStatus;

#[repr(C)]
pub struct EventTypeSpec {
    pub event_class: u32,
    pub event_kind: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct EventHotKeyID {
    pub signature: u32,
    pub id: u32,
}

pub const fn four_cc(s: &[u8; 4]) -> u32 {
    ((s[0] as u32) << 24) | ((s[1] as u32) << 16) | ((s[2] as u32) << 8) | s[3] as u32
}

pub const kEventClassKeyboard: u32 = four_cc(b"keyb");
pub const kEventHotKeyPressed: u32 = 5;
pub const kEventParamDirectObject: u32 = four_cc(b"----");
pub const typeEventHotKeyID: u32 = four_cc(b"hkid");
pub const cmdKey: u32 = 1 << 8;
pub const shiftKey: u32 = 1 << 9;
pub const optionKey: u32 = 1 << 11;
pub const controlKey: u32 = 1 << 12;

#[link(name = "Carbon", kind = "framework")]
extern "C" {
    pub fn GetApplicationEventTarget() -> EventTargetRef;
    pub fn InstallEventHandler(
        target: EventTargetRef,
        handler: EventHandlerUPP,
        num_types: u32,
        list: *const EventTypeSpec,
        user_data: *mut c_void,
        out: *mut EventHandlerRef,
    ) -> OSStatus;
    pub fn RegisterEventHotKey(
        key_code: u32,
        modifiers: u32,
        id: EventHotKeyID,
        target: EventTargetRef,
        options: u32,
        out: *mut EventHotKeyRef,
    ) -> OSStatus;
    pub fn UnregisterEventHotKey(r: EventHotKeyRef) -> OSStatus;
    pub fn GetEventParameter(
        event: EventRef,
        name: u32,
        desired_type: u32,
        actual_type: *mut u32,
        size: usize,
        actual_size: *mut usize,
        data: *mut c_void,
    ) -> OSStatus;
}

// ----- libdispatch ------------------------------------------------------

#[repr(C)]
pub struct dispatch_queue_s {
    _private: [u8; 0],
}
pub type dispatch_time_t = u64;
pub const DISPATCH_TIME_NOW: dispatch_time_t = 0;

extern "C" {
    pub static _dispatch_main_q: dispatch_queue_s;
    pub fn dispatch_async_f(queue: *const dispatch_queue_s, ctx: *mut c_void, work: extern "C" fn(*mut c_void));
    pub fn dispatch_after_f(
        when: dispatch_time_t,
        queue: *const dispatch_queue_s,
        ctx: *mut c_void,
        work: extern "C" fn(*mut c_void),
    );
    pub fn dispatch_time(when: dispatch_time_t, delta_ns: i64) -> dispatch_time_t;
}

pub fn main_queue() -> *const dispatch_queue_s {
    unsafe { &_dispatch_main_q }
}

// ----- tiny RAII helpers --------------------------------------------------

/// An owned Core Foundation reference, released on drop.
pub struct CfOwned(pub CFTypeRef);

impl CfOwned {
    /// Takes ownership of a +1 reference (from a Create/Copy function).
    pub fn from_create(r: CFTypeRef) -> Option<CfOwned> {
        (!r.is_null()).then_some(CfOwned(r))
    }

    pub fn as_ptr(&self) -> CFTypeRef {
        self.0
    }
}

impl Clone for CfOwned {
    fn clone(&self) -> Self {
        unsafe { CfOwned(CFRetain(self.0)) }
    }
}

impl Drop for CfOwned {
    fn drop(&mut self) {
        unsafe { CFRelease(self.0) }
    }
}

pub fn cfstr(s: &str) -> CfOwned {
    unsafe {
        let r = CFStringCreateWithBytes(std::ptr::null(), s.as_ptr(), s.len() as CFIndex, kCFStringEncodingUTF8, 0);
        CfOwned::from_create(r).expect("CFString allocation")
    }
}

/// Copy a CFString into a Rust string. Returns `None` for other types.
pub fn string_from_cf(r: CFTypeRef) -> Option<String> {
    unsafe {
        if r.is_null() || CFGetTypeID(r) != CFStringGetTypeID() {
            return None;
        }
        let len = CFStringGetLength(r);
        let cap = CFStringGetMaximumSizeForEncoding(len, kCFStringEncodingUTF8) + 1;
        let mut buf = vec![0u8; cap as usize];
        if CFStringGetCString(r, buf.as_mut_ptr(), cap, kCFStringEncodingUTF8) == 0 {
            return None;
        }
        let end = buf.iter().position(|b| *b == 0).unwrap_or(buf.len());
        buf.truncate(end);
        String::from_utf8(buf).ok()
    }
}

// The pointers above are only touched on the main thread; CfOwned itself is
// just a refcounted handle, which Core Foundation allows across threads.
unsafe impl Send for CfOwned {}
