//! An in-memory [`Platform`] for tests, demos and the `gridpilot simulate`
//! command. It records every call so tests can assert on them.

use crate::geometry::Rect;
use crate::keys::{Chord, Modifiers};
use crate::platform::{DisplayId, DisplayInfo, Platform, PlatformResult, UiRequest, WindowId, WindowInfo};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

#[derive(Debug, Default)]
pub struct MockState {
    pub displays: Vec<DisplayInfo>,
    /// Front-most first.
    pub windows: Vec<WindowInfo>,
    pub hidden: BTreeMap<WindowId, bool>,
    pub focused: Option<WindowId>,
    pub hotkeys: Vec<Chord>,
    pub ui: Vec<UiRequest>,
    pub spawned: Vec<String>,
    pub closed: Vec<WindowId>,
    /// Number of `set_frame` calls, to check GridPilot leaves windows alone.
    pub set_frame_calls: usize,
    /// Windows whose `set_frame` fails, like a fixed-size macOS window.
    pub unmovable: Vec<WindowId>,
    /// Chords `register_hotkeys` reports as taken by another app.
    pub taken_hotkeys: Vec<Chord>,
    /// What Caps Lock was last set to press.
    pub capslock: Option<Modifiers>,
}

impl MockState {
    pub fn frame(&self, id: WindowId) -> Rect {
        self.windows.iter().find(|w| w.id == id).map(|w| w.frame).expect("mock window exists")
    }

    pub fn is_hidden(&self, id: WindowId) -> bool {
        self.hidden.get(&id).copied().unwrap_or(false)
    }
}

/// Cheap to clone: clones share state, so a test keeps one handle while the
/// engine owns another.
#[derive(Debug, Clone, Default)]
pub struct MockPlatform(pub Rc<RefCell<MockState>>);

impl MockPlatform {
    /// One 1920x1080 display with a 40px menu bar, or two side by side.
    pub fn with_displays(n: usize) -> Self {
        let p = MockPlatform::default();
        for i in 0..n {
            let x = 1920.0 * i as f64;
            p.0.borrow_mut().displays.push(DisplayInfo {
                id: DisplayId(i as u32 + 1),
                name: format!("Display {}", i + 1),
                frame: Rect::new(x, 0.0, 1920.0, 1080.0),
                visible: Rect::new(x, 40.0, 1920.0, 1040.0),
                scale: 2.0,
            });
        }
        p
    }

    pub fn open(&self, id: u64, app: &str, frame: Rect) -> WindowId {
        let id = WindowId(id);
        let mut s = self.0.borrow_mut();
        s.windows.insert(
            0,
            WindowInfo {
                id,
                app: app.into(),
                title: format!("{app} {}", id.0),
                frame,
                is_dialog: false,
                is_minimized: false,
            },
        );
        s.focused = Some(id);
        id
    }

    pub fn close_window(&self, id: WindowId) {
        let mut s = self.0.borrow_mut();
        s.windows.retain(|w| w.id != id);
        if s.focused == Some(id) {
            s.focused = s.windows.first().map(|w| w.id);
        }
    }

    pub fn state(&self) -> std::cell::Ref<'_, MockState> {
        self.0.borrow()
    }
}

impl Platform for MockPlatform {
    fn displays(&self) -> PlatformResult<Vec<DisplayInfo>> {
        Ok(self.0.borrow().displays.clone())
    }

    fn windows(&self) -> PlatformResult<Vec<WindowInfo>> {
        Ok(self.0.borrow().windows.clone())
    }

    fn focused_window(&self) -> PlatformResult<Option<WindowId>> {
        Ok(self.0.borrow().focused)
    }

    fn set_frame(&mut self, id: WindowId, frame: Rect) -> PlatformResult<()> {
        let mut s = self.0.borrow_mut();
        s.set_frame_calls += 1;
        if s.unmovable.contains(&id) {
            return Err("setting AXSize failed (AXError -25200)".into());
        }
        s.hidden.insert(id, false);
        let w = s.windows.iter_mut().find(|w| w.id == id).ok_or("no such window")?;
        w.frame = frame;
        Ok(())
    }

    fn hide(&mut self, id: WindowId, on: &DisplayInfo) -> PlatformResult<()> {
        let mut s = self.0.borrow_mut();
        s.hidden.insert(id, true);
        let w = s.windows.iter_mut().find(|w| w.id == id).ok_or("no such window")?;
        w.frame = Rect::new(on.frame.max_x() - 1.0, on.frame.max_y() - 1.0, w.frame.w, w.frame.h);
        Ok(())
    }

    fn focus(&mut self, id: WindowId) -> PlatformResult<()> {
        self.0.borrow_mut().focused = Some(id);
        Ok(())
    }

    fn close(&mut self, id: WindowId) -> PlatformResult<()> {
        self.0.borrow_mut().closed.push(id);
        Ok(())
    }

    fn register_hotkeys(&mut self, chords: &[Chord]) -> PlatformResult<()> {
        let mut s = self.0.borrow_mut();
        s.hotkeys = chords.iter().filter(|c| !s.taken_hotkeys.contains(c)).copied().collect();
        let taken: Vec<String> = chords.iter().filter(|c| s.taken_hotkeys.contains(c)).map(|c| c.to_string()).collect();
        if taken.is_empty() {
            Ok(())
        } else {
            Err(format!("could not register (taken by another app?): {}", taken.join(", ")))
        }
    }

    fn set_capslock_modifiers(&mut self, mods: Option<Modifiers>) -> PlatformResult<()> {
        self.0.borrow_mut().capslock = mods;
        Ok(())
    }

    fn present(&mut self, request: UiRequest) {
        self.0.borrow_mut().ui.push(request);
    }

    fn spawn(&mut self, command: &str) -> PlatformResult<()> {
        self.0.borrow_mut().spawned.push(command.to_string());
        Ok(())
    }
}
