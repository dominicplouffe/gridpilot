//! A small safe wrapper over `AXUIElement`.

use crate::ffi::*;
use gridpilot_core::geometry::Rect;
use std::ffi::c_void;

#[derive(Clone)]
pub struct AxElement(CfOwned);

impl AxElement {
    pub fn application(pid: i32) -> Option<AxElement> {
        let el = CfOwned::from_create(unsafe { AXUIElementCreateApplication(pid) })?;
        // Hung apps must not freeze the window manager.
        unsafe { AXUIElementSetMessagingTimeout(el.as_ptr(), 0.25) };
        Some(AxElement(el))
    }

    fn raw(&self) -> AXUIElementRef {
        self.0.as_ptr()
    }

    pub fn attribute(&self, name: &str) -> Option<CfOwned> {
        let attr = cfstr(name);
        let mut out: CFTypeRef = std::ptr::null();
        let err = unsafe { AXUIElementCopyAttributeValue(self.raw(), attr.as_ptr(), &mut out) };
        if err != kAXErrorSuccess {
            return None;
        }
        CfOwned::from_create(out)
    }

    pub fn string(&self, name: &str) -> Option<String> {
        self.attribute(name).and_then(|v| string_from_cf(v.as_ptr()))
    }

    pub fn bool(&self, name: &str) -> Option<bool> {
        let v = self.attribute(name)?;
        unsafe {
            if CFGetTypeID(v.as_ptr()) != CFBooleanGetTypeID() {
                return None;
            }
            Some(CFBooleanGetValue(v.as_ptr()) != 0)
        }
    }

    pub fn element(&self, name: &str) -> Option<AxElement> {
        self.attribute(name).map(AxElement)
    }

    pub fn elements(&self, name: &str) -> Vec<AxElement> {
        let Some(arr) = self.attribute(name) else { return vec![] };
        unsafe {
            if CFGetTypeID(arr.as_ptr()) != CFArrayGetTypeID() {
                return vec![];
            }
            let n = CFArrayGetCount(arr.as_ptr());
            (0..n)
                .filter_map(|i| {
                    let v = CFArrayGetValueAtIndex(arr.as_ptr(), i);
                    // Borrowed from the array: retain to own it.
                    (!v.is_null()).then(|| AxElement(CfOwned(CFRetain(v))))
                })
                .collect()
        }
    }

    pub fn window_id(&self) -> Option<u32> {
        let mut id = 0u32;
        let err = unsafe { _AXUIElementGetWindow(self.raw(), &mut id) };
        (err == kAXErrorSuccess && id != 0).then_some(id)
    }

    pub fn frame(&self) -> Option<Rect> {
        let mut p = CGPoint::default();
        let mut s = CGSize::default();
        let pos = self.attribute("AXPosition")?;
        let size = self.attribute("AXSize")?;
        unsafe {
            if AXValueGetValue(pos.as_ptr(), kAXValueCGPointType, &mut p as *mut _ as *mut c_void) == 0
                || AXValueGetValue(size.as_ptr(), kAXValueCGSizeType, &mut s as *mut _ as *mut c_void) == 0
            {
                return None;
            }
        }
        Some(Rect::new(p.x, p.y, s.width, s.height))
    }

    fn set_value(&self, name: &str, value: &CfOwned) -> Result<(), String> {
        let attr = cfstr(name);
        let err = unsafe { AXUIElementSetAttributeValue(self.raw(), attr.as_ptr(), value.as_ptr()) };
        if err == kAXErrorSuccess {
            Ok(())
        } else {
            Err(format!("setting {name} failed (AXError {err})"))
        }
    }

    pub fn set_position(&self, x: f64, y: f64) -> Result<(), String> {
        let p = CGPoint { x, y };
        let v = CfOwned::from_create(unsafe { AXValueCreate(kAXValueCGPointType, &p as *const _ as *const c_void) })
            .ok_or("AXValueCreate failed")?;
        self.set_value("AXPosition", &v)
    }

    pub fn set_size(&self, width: f64, height: f64) -> Result<(), String> {
        let s = CGSize { width, height };
        let v = CfOwned::from_create(unsafe { AXValueCreate(kAXValueCGSizeType, &s as *const _ as *const c_void) })
            .ok_or("AXValueCreate failed")?;
        self.set_value("AXSize", &v)
    }

    /// Position, size, then position again: moving first lets the window grow
    /// into a new display, and the second move fixes up apps that clamp.
    pub fn set_frame(&self, r: Rect) -> Result<(), String> {
        self.set_position(r.x, r.y)?;
        self.set_size(r.w, r.h)?;
        self.set_position(r.x, r.y)
    }

    pub fn set_bool(&self, name: &str, value: bool) -> Result<(), String> {
        let v = unsafe { CfOwned(CFRetain(if value { kCFBooleanTrue } else { kCFBooleanFalse })) };
        self.set_value(name, &v)
    }

    pub fn perform(&self, action: &str) -> Result<(), String> {
        let a = cfstr(action);
        let err = unsafe { AXUIElementPerformAction(self.raw(), a.as_ptr()) };
        if err == kAXErrorSuccess {
            Ok(())
        } else {
            Err(format!("{action} failed (AXError {err})"))
        }
    }
}

/// Is GridPilot allowed to use Accessibility? With `prompt`, macOS shows its
/// "would like to control this computer" dialog if not.
pub fn is_trusted(prompt: bool) -> bool {
    unsafe {
        let key = cfstr("AXTrustedCheckOptionPrompt");
        let value = if prompt { kCFBooleanTrue } else { kCFBooleanFalse };
        let keys = [key.as_ptr()];
        let values = [value];
        let dict = CFDictionaryCreate(
            std::ptr::null(),
            keys.as_ptr(),
            values.as_ptr(),
            1,
            &kCFTypeDictionaryKeyCallBacks,
            &kCFTypeDictionaryValueCallBacks,
        );
        let ok = AXIsProcessTrustedWithOptions(dict) != 0;
        if !dict.is_null() {
            CFRelease(dict);
        }
        ok
    }
}
