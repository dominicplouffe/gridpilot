//! [`Platform`] for macOS.

use crate::ax::AxElement;
use crate::ffi::{CGPoint, CGWarpMouseCursorPosition};
use gridpilot_core::geometry::{Point, Rect};
use gridpilot_core::keys::{Chord, Modifiers};
use gridpilot_core::platform::{DisplayId, DisplayInfo, Platform, PlatformResult, UiRequest, WindowId, WindowInfo};
use objc2::MainThreadMarker;
use objc2_app_kit::{NSApplicationActivationPolicy, NSRunningApplication, NSScreen, NSWorkspace};
use objc2_foundation::{NSNumber, NSString};
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Default)]
pub struct MacPlatform {
    apps: RefCell<HashMap<i32, AxElement>>,
    windows: RefCell<HashMap<WindowId, (i32, AxElement)>>,
    displays: RefCell<Vec<DisplayInfo>>,
    own_pid: i32,
}

impl MacPlatform {
    pub fn new() -> Self {
        Self { own_pid: std::process::id() as i32, ..Default::default() }
    }

    fn app(&self, pid: i32) -> Option<AxElement> {
        let mut apps = self.apps.borrow_mut();
        if let Some(a) = apps.get(&pid) {
            return Some(a.clone());
        }
        let a = AxElement::application(pid)?;
        apps.insert(pid, a.clone());
        Some(a)
    }

    fn window(&self, id: WindowId) -> PlatformResult<(i32, AxElement)> {
        self.windows.borrow().get(&id).cloned().ok_or_else(|| format!("unknown window {id}"))
    }
}

fn mtm() -> MainThreadMarker {
    MainThreadMarker::new().expect("GridPilot's platform layer runs on the main thread")
}

impl Platform for MacPlatform {
    fn displays(&self) -> PlatformResult<Vec<DisplayInfo>> {
        let screens = NSScreen::screens(mtm());
        let Some(primary) = screens.iter().next() else { return Err("no screens".into()) };
        // AppKit puts the origin at the bottom-left of the primary screen;
        // Accessibility (and GridPilot) at its top-left.
        let primary_h = primary.frame().size.height;
        let flip = |r: objc2_foundation::NSRect| {
            Rect::new(r.origin.x, primary_h - (r.origin.y + r.size.height), r.size.width, r.size.height)
        };
        let key = NSString::from_str("NSScreenNumber");
        let out: Vec<DisplayInfo> = screens
            .iter()
            .map(|s| {
                let number = s
                    .deviceDescription()
                    .objectForKey(&key)
                    .and_then(|n| n.downcast::<NSNumber>().ok())
                    .map(|n| n.unsignedIntValue())
                    .unwrap_or(0);
                DisplayInfo {
                    id: DisplayId(number),
                    name: s.localizedName().to_string(),
                    frame: flip(s.frame()),
                    visible: flip(s.visibleFrame()),
                    scale: s.backingScaleFactor(),
                }
            })
            .collect();
        *self.displays.borrow_mut() = out.clone();
        Ok(out)
    }

    fn windows(&self) -> PlatformResult<Vec<WindowInfo>> {
        let workspace = NSWorkspace::sharedWorkspace();
        let mut out = Vec::new();
        let mut cache = HashMap::new();
        for app in workspace.runningApplications().iter() {
            if app.activationPolicy() != NSApplicationActivationPolicy::Regular {
                continue;
            }
            let pid = app.processIdentifier();
            if pid == self.own_pid {
                continue;
            }
            let hidden = app.isHidden();
            let bundle = app
                .bundleIdentifier()
                .map(|b| b.to_string())
                .unwrap_or_else(|| app.localizedName().map(|n| n.to_string()).unwrap_or_else(|| format!("pid{pid}")));
            let Some(ax) = self.app(pid) else { continue };
            for w in ax.elements("AXWindows") {
                if w.string("AXRole").as_deref() != Some("AXWindow") {
                    continue;
                }
                let subrole = w.string("AXSubrole").unwrap_or_default();
                let is_dialog = match subrole.as_str() {
                    "AXStandardWindow" => false,
                    "AXDialog" | "AXSystemDialog" | "AXFloatingWindow" => true,
                    _ => continue,
                };
                // Fixed-size windows (About boxes, many settings panes)
                // can't fill a tile, so they always float.
                let is_dialog = is_dialog || !w.is_settable("AXSize");
                let (Some(id), Some(frame)) = (w.window_id(), w.frame()) else { continue };
                if frame.w < 50.0 || frame.h < 50.0 {
                    continue;
                }
                let id = WindowId(id as u64);
                out.push(WindowInfo {
                    id,
                    app: bundle.clone(),
                    title: w.string("AXTitle").unwrap_or_default(),
                    frame,
                    is_dialog,
                    is_minimized: hidden || w.bool("AXMinimized").unwrap_or(false),
                });
                cache.insert(id, (pid, w));
            }
        }
        *self.windows.borrow_mut() = cache;
        let live: Vec<i32> = self.windows.borrow().values().map(|(p, _)| *p).collect();
        self.apps.borrow_mut().retain(|pid, _| live.contains(pid));
        Ok(out)
    }

    fn focused_window(&self) -> PlatformResult<Option<WindowId>> {
        let Some(app) = NSWorkspace::sharedWorkspace().frontmostApplication() else { return Ok(None) };
        let Some(ax) = self.app(app.processIdentifier()) else { return Ok(None) };
        Ok(ax.element("AXFocusedWindow").and_then(|w| w.window_id()).map(|id| WindowId(id as u64)))
    }

    fn set_frame(&mut self, id: WindowId, frame: Rect) -> PlatformResult<()> {
        let (_, w) = self.window(id)?;
        w.set_frame(frame)
    }

    fn hide(&mut self, id: WindowId, on: &DisplayInfo) -> PlatformResult<()> {
        let (_, w) = self.window(id)?;
        let size = w.frame().map(|f| f.size()).unwrap_or_default();
        // Park the window so only a sliver stays on `on`, choosing the bottom
        // corner that has no neighbouring display (AeroSpace's approach).
        let displays = self.displays.borrow();
        let neighbour_right = displays.iter().any(|d| {
            d.id != on.id
                && d.frame.x >= on.frame.max_x() - 1.0
                && d.frame.y < on.frame.max_y()
                && d.frame.max_y() > on.frame.y
        });
        let x = if neighbour_right { on.frame.x - size.w + 1.0 } else { on.frame.max_x() - 1.0 };
        w.set_position(x, on.frame.max_y() - 1.0)
    }

    fn focus(&mut self, id: WindowId) -> PlatformResult<()> {
        let (pid, w) = self.window(id)?;
        if let Some(app) = NSRunningApplication::runningApplicationWithProcessIdentifier(pid) {
            if app.isHidden() {
                app.unhide();
            }
        }
        if let Some(ax) = self.app(pid) {
            let _ = ax.set_bool("AXFrontmost", true);
        }
        let _ = w.set_bool("AXMain", true);
        w.perform("AXRaise")
    }

    fn close(&mut self, id: WindowId) -> PlatformResult<()> {
        let (_, w) = self.window(id)?;
        w.element("AXCloseButton").ok_or("window has no close button")?.perform("AXPress")
    }

    fn minimize(&mut self, id: WindowId) -> PlatformResult<()> {
        let (_, w) = self.window(id)?;
        w.set_bool("AXMinimized", true)
    }

    fn warp_pointer(&mut self, to: Point) {
        unsafe { CGWarpMouseCursorPosition(CGPoint { x: to.x, y: to.y }) };
    }

    fn register_hotkeys(&mut self, chords: &[Chord]) -> PlatformResult<()> {
        // No hotkeys means "paused" (the settings page is recording).
        crate::capslock::set_passthrough(chords.is_empty());
        let failed = crate::hotkeys::register(chords);
        if failed.is_empty() {
            Ok(())
        } else {
            let names: Vec<String> = failed.iter().map(|c| c.to_string()).collect();
            Err(format!("could not register (taken by another app?): {}", names.join(", ")))
        }
    }

    fn set_capslock_modifiers(&mut self, mods: Option<Modifiers>) -> PlatformResult<()> {
        crate::capslock::configure(mods)
    }

    fn present(&mut self, request: UiRequest) {
        let mtm = mtm();
        match request {
            UiRequest::ShowGrid(o) => crate::overlay::show(mtm, o),
            UiRequest::HideGrid => crate::overlay::hide(),
            UiRequest::ShowKeys(keys) => crate::hud::show_keys(mtm, &keys),
            UiRequest::Notify { title, body } => crate::hud::flash(mtm, &title, &body),
            UiRequest::ThemeChanged(theme) => {
                crate::hud::set_theme(theme.clone());
                crate::overlay::set_theme(theme);
            }
        }
    }
}
