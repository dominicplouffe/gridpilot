//! The grid overlay: one small panel per display, Divvy-style. The panel only
//! draws the engine's [`GridOverlay`] state and turns input into `grid ...`
//! commands; selection logic lives in `gridpilot_core::grid::GridSession`.

use crate::app::post_command;
use crate::keycodes::key_for;
use gridpilot_core::grid::Cell;
use gridpilot_core::keys::{Chord, Key, Modifiers};
use gridpilot_core::platform::{DisplayId, GridOverlay};
use gridpilot_core::theme::{Color, Theme};
use objc2::rc::Retained;
use objc2::{define_class, msg_send, DefinedClass, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSBackingStoreType, NSBezierPath, NSColor, NSEvent, NSEventModifierFlags, NSFont, NSPanel, NSScreen, NSTextField,
    NSView, NSWindowCollectionBehavior, NSWindowStyleMask,
};
use objc2_foundation::{NSNumber, NSObjectProtocol, NSPoint, NSRect, NSSize, NSString};
use std::cell::{Cell as StdCell, RefCell};

const GRID_W: f64 = 440.0;
const PAD: f64 = 18.0;
const FOOTER: f64 = 44.0;

type Panels = Vec<(DisplayId, Retained<GridPanel>, Retained<GridView>)>;

thread_local! {
    static PANELS: RefCell<Panels> = const { RefCell::new(Vec::new()) };
    static STATE: RefCell<Option<GridOverlay>> = const { RefCell::new(None) };
    static THEME: RefCell<Option<Theme>> = const { RefCell::new(None) };
    static OPTION_DOWN: StdCell<bool> = const { StdCell::new(false) };
}

pub fn set_theme(theme: Theme) {
    THEME.with(|t| *t.borrow_mut() = Some(theme));
}

fn theme() -> Theme {
    STATE
        .with(|s| s.borrow().as_ref().map(|o| o.theme.clone()))
        .or_else(|| THEME.with(|t| t.borrow().clone()))
        .unwrap_or_default()
}

pub(crate) fn ns_color(c: Color) -> Retained<NSColor> {
    let [r, g, b, a] = c.to_f64();
    NSColor::colorWithSRGBRed_green_blue_alpha(r, g, b, a)
}

pub struct GridViewIvars {
    display: StdCell<u32>,
}

define_class!(
    #[unsafe(super(NSPanel))]
    #[thread_kind = MainThreadOnly]
    #[name = "GPGridPanel"]
    pub struct GridPanel;

    unsafe impl NSObjectProtocol for GridPanel {}

    impl GridPanel {
        // Borderless panels refuse key status by default; we need arrows,
        // return and escape.
        #[unsafe(method(canBecomeKeyWindow))]
        fn can_become_key(&self) -> bool {
            true
        }
    }
);

define_class!(
    #[unsafe(super(NSView))]
    #[thread_kind = MainThreadOnly]
    #[name = "GPGridView"]
    #[ivars = GridViewIvars]
    pub struct GridView;

    unsafe impl NSObjectProtocol for GridView {}

    impl GridView {
        #[unsafe(method(isFlipped))]
        fn is_flipped(&self) -> bool {
            true
        }

        #[unsafe(method(acceptsFirstResponder))]
        fn accepts_first_responder(&self) -> bool {
            true
        }

        #[unsafe(method(acceptsFirstMouse:))]
        fn accepts_first_mouse(&self, _event: Option<&NSEvent>) -> bool {
            true
        }

        #[unsafe(method(drawRect:))]
        fn draw_rect(&self, _dirty: NSRect) {
            self.draw();
        }

        #[unsafe(method(mouseDown:))]
        fn mouse_down(&self, event: &NSEvent) {
            self.mouse("press", event);
        }

        #[unsafe(method(mouseDragged:))]
        fn mouse_dragged(&self, event: &NSEvent) {
            self.mouse("drag", event);
        }

        #[unsafe(method(mouseUp:))]
        fn mouse_up(&self, event: &NSEvent) {
            self.mouse("release", event);
        }

        #[unsafe(method(keyDown:))]
        fn key_down(&self, event: &NSEvent) {
            self.key(event);
        }

        #[unsafe(method(flagsChanged:))]
        fn flags_changed(&self, event: &NSEvent) {
            let down = event.modifierFlags().contains(NSEventModifierFlags::Option);
            if down && !OPTION_DOWN.get() {
                post_command("grid subdivide".into());
            }
            OPTION_DOWN.set(down);
        }
    }
);

impl GridView {
    fn new(mtm: MainThreadMarker, frame: NSRect, display: DisplayId) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(GridViewIvars { display: StdCell::new(display.0) });
        unsafe { msg_send![super(this), initWithFrame: frame] }
    }

    fn display(&self) -> DisplayId {
        DisplayId(self.ivars().display.get())
    }

    /// The rectangle cells are drawn in, in view coordinates (flipped).
    fn grid_rect(&self) -> NSRect {
        let b = self.bounds();
        NSRect::new(NSPoint::new(PAD, PAD), NSSize::new(b.size.width - 2.0 * PAD, b.size.height - 2.0 * PAD - FOOTER))
    }

    fn cell_at(&self, event: &NSEvent) -> Option<Cell> {
        let grid = STATE.with(|s| s.borrow().as_ref().map(|o| o.session.grid()))?;
        let p = self.convertPoint_fromView(event.locationInWindow(), None);
        let r = self.grid_rect();
        let fx = ((p.x - r.origin.x) / r.size.width).clamp(0.0, 0.9999);
        let fy = ((p.y - r.origin.y) / r.size.height).clamp(0.0, 0.9999);
        Some(Cell { col: (fx * grid.cols as f64) as u16, row: (fy * grid.rows as f64) as u16 })
    }

    fn mouse(&self, what: &str, event: &NSEvent) {
        if let Some(c) = self.cell_at(event) {
            post_command(format!("grid {what} {} {} --display {}", c.col, c.row, self.display()));
        }
    }

    fn key(&self, event: &NSEvent) {
        let flags = event.modifierFlags();
        let shift = flags.contains(NSEventModifierFlags::Shift);
        let Some(key) = key_for(event.keyCode()) else { return };
        let d = self.display();
        let cmd = match key {
            Key::Escape => "grid cancel".to_string(),
            Key::Return | Key::Space => format!("grid commit --display {d}"),
            Key::Left | Key::Right | Key::Up | Key::Down => {
                let dir = match key {
                    Key::Left => "left",
                    Key::Right => "right",
                    Key::Up => "up",
                    _ => "down",
                };
                format!("grid key {dir}{}", if shift { " --extend" } else { "" })
            }
            Key::Char('a') if !flags.contains(NSEventModifierFlags::Command) => "grid all".into(),
            Key::Equal => if shift { "grid size 0 1" } else { "grid size 1 0" }.into(),
            Key::Minus => if shift { "grid size 0 -1" } else { "grid size -1 0" }.into(),
            other => {
                let mut mods = Modifiers::NONE;
                for (f, m) in [
                    (NSEventModifierFlags::Shift, Modifiers::SHIFT),
                    (NSEventModifierFlags::Control, Modifiers::CTRL),
                    (NSEventModifierFlags::Option, Modifiers::ALT),
                    (NSEventModifierFlags::Command, Modifiers::CMD),
                ] {
                    if flags.contains(f) {
                        mods |= m;
                    }
                }
                format!("grid local {}", Chord { mods, key: other })
            }
        };
        post_command(cmd);
    }

    fn draw(&self) {
        let t = theme();
        let Some(o) = STATE.with(|s| s.borrow().clone()) else { return };
        let bounds = self.bounds();
        let is_target = o.display == self.display();

        let bg = NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(
            bounds,
            t.corner_radius + 4.0,
            t.corner_radius + 4.0,
        );
        ns_color(t.surface.with_alpha(0xf2)).setFill();
        bg.fill();
        if is_target {
            ns_color(t.accent).setStroke();
            bg.setLineWidth(t.border_width);
            bg.stroke();
        }

        let grid = o.session.grid();
        let r = self.grid_rect();
        let gap = if grid.cols > 10 { 2.0 } else { 4.0 };
        let cw = (r.size.width - gap * (grid.cols as f64 - 1.0)) / grid.cols as f64;
        let ch = (r.size.height - gap * (grid.rows as f64 - 1.0)) / grid.rows as f64;
        let sel = o.session.selection();
        let radius = (t.corner_radius / 2.0).min(cw / 3.0);
        for row in 0..grid.rows {
            for col in 0..grid.cols {
                let x = r.origin.x + col as f64 * (cw + gap);
                let y = r.origin.y + row as f64 * (ch + gap);
                let inside = col >= sel.col && col < sel.col + sel.cols && row >= sel.row && row < sel.row + sel.rows;
                let c = if inside { t.accent.with_alpha(if is_target { 0xe6 } else { 0x80 }) } else { t.line };
                ns_color(c).setFill();
                NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(
                    NSRect::new(NSPoint::new(x, y), NSSize::new(cw, ch)),
                    radius,
                    radius,
                )
                .fill();
            }
        }
        // Outline the whole selection with the second accent.
        let sx = r.origin.x + sel.col as f64 * (cw + gap);
        let sy = r.origin.y + sel.row as f64 * (ch + gap);
        let sw = sel.cols as f64 * cw + (sel.cols as f64 - 1.0) * gap;
        let sh = sel.rows as f64 * ch + (sel.rows as f64 - 1.0) * gap;
        let outline = NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(
            NSRect::new(NSPoint::new(sx - 1.5, sy - 1.5), NSSize::new(sw + 3.0, sh + 3.0)),
            radius + 1.5,
            radius + 1.5,
        );
        outline.setLineWidth(1.5);
        ns_color(t.accent_alt).setStroke();
        outline.stroke();
    }
}

fn hint_text(o: &GridOverlay) -> String {
    let g = o.session.grid();
    format!(
        "GridPilot  {}{}   drag to place · ←↑→↓ move · ⇧ extend · ⌥ subdivide · ± size · ⏎ apply · esc",
        g,
        if o.session.is_subdivided() { " (subdivided)" } else { "" }
    )
}

fn screen_number(s: &NSScreen) -> u32 {
    s.deviceDescription()
        .objectForKey(&NSString::from_str("NSScreenNumber"))
        .and_then(|n| n.downcast::<NSNumber>().ok())
        .map(|n| n.unsignedIntValue())
        .unwrap_or(0)
}

fn build(mtm: MainThreadMarker) {
    let t = theme();
    let mut panels = Vec::new();
    for screen in NSScreen::screens(mtm).iter() {
        let vis = screen.visibleFrame();
        let aspect = (vis.size.height / vis.size.width).clamp(0.3, 2.0);
        let w = GRID_W + 2.0 * PAD;
        let h = GRID_W * aspect + 2.0 * PAD + FOOTER;
        let origin =
            NSPoint::new(vis.origin.x + (vis.size.width - w) / 2.0, vis.origin.y + (vis.size.height - h) / 2.0);
        let rect = NSRect::new(origin, NSSize::new(w, h));
        let panel: Retained<GridPanel> = unsafe {
            msg_send![
                GridPanel::alloc(mtm),
                initWithContentRect: rect,
                styleMask: NSWindowStyleMask::Borderless | NSWindowStyleMask::NonactivatingPanel,
                backing: NSBackingStoreType::Buffered,
                defer: false
            ]
        };
        unsafe { panel.setReleasedWhenClosed(false) };
        panel.setOpaque(false);
        panel.setBackgroundColor(Some(&NSColor::clearColor()));
        panel.setHasShadow(true);
        panel.setLevel(objc2_app_kit::NSStatusWindowLevel);
        panel.setFloatingPanel(true);
        panel.setCollectionBehavior(
            NSWindowCollectionBehavior::CanJoinAllSpaces
                | NSWindowCollectionBehavior::Transient
                | NSWindowCollectionBehavior::FullScreenAuxiliary,
        );

        let display = DisplayId(screen_number(&screen));
        let view = GridView::new(mtm, NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(w, h)), display);
        let label = NSTextField::labelWithString(&NSString::from_str(""), mtm);
        label.setFrame(NSRect::new(NSPoint::new(PAD, h - FOOTER + 4.0), NSSize::new(w - 2.0 * PAD, FOOTER - 12.0)));
        label.setTextColor(Some(&ns_color(t.muted)));
        label.setFont(Some(&NSFont::systemFontOfSize(11.0)));
        view.addSubview(&label);
        panel.setContentView(Some(&view));
        panels.push((display, panel, view));
    }
    PANELS.with(|p| *p.borrow_mut() = panels);
}

pub fn show(mtm: MainThreadMarker, overlay: GridOverlay) {
    let text = hint_text(&overlay);
    let target = overlay.display;
    STATE.with(|s| *s.borrow_mut() = Some(overlay));
    let screens = NSScreen::screens(mtm).len();
    if PANELS.with(|p| p.borrow().len() != screens) {
        build(mtm);
    }
    PANELS.with(|p| {
        for (display, panel, view) in p.borrow().iter() {
            if let Some(sub) = view.subviews().iter().next() {
                if let Ok(label) = sub.downcast::<NSTextField>() {
                    label.setStringValue(&NSString::from_str(&text));
                }
            }
            view.setNeedsDisplay(true);
            if *display == target {
                panel.makeKeyAndOrderFront(None);
                panel.makeFirstResponder(Some(view));
            } else {
                panel.orderFront(None);
            }
        }
    });
}

pub fn hide() {
    STATE.with(|s| *s.borrow_mut() = None);
    OPTION_DOWN.set(false);
    PANELS.with(|p| {
        for (_, panel, _) in p.borrow().iter() {
            panel.orderOut(None);
        }
    });
}
