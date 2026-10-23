//! Transient notifications ("Workspace 3", "Layout: dwindle") and the
//! Omarchy-style key cheat sheet.

use crate::app::after;
use crate::overlay::ns_color;
use gridpilot_core::platform::KeyHelp;
use gridpilot_core::theme::Theme;
use objc2::rc::Retained;
use objc2::{MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSApplication, NSBackingStoreType, NSColor, NSFont, NSFontWeightRegular, NSPanel, NSScreen, NSTextAlignment,
    NSTextField, NSTextView, NSWindowCollectionBehavior, NSWindowStyleMask,
};
use objc2_foundation::{NSPoint, NSRect, NSSize, NSString};
use std::cell::{Cell, RefCell};

thread_local! {
    static FLASH: RefCell<Option<(Retained<NSPanel>, Retained<NSTextField>)>> = const { RefCell::new(None) };
    static FLASH_GEN: Cell<u64> = const { Cell::new(0) };
    static KEYS: RefCell<Option<Retained<NSPanel>>> = const { RefCell::new(None) };
    static THEME: RefCell<Theme> = RefCell::new(Theme::default());
}

pub fn set_theme(theme: Theme) {
    THEME.with(|t| *t.borrow_mut() = theme);
}

fn main_visible(mtm: MainThreadMarker) -> NSRect {
    NSScreen::mainScreen(mtm)
        .map(|s| s.visibleFrame())
        .unwrap_or(NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(1440.0, 900.0)))
}

/// Show `title · body` briefly near the bottom of the main screen.
pub fn flash(mtm: MainThreadMarker, title: &str, body: &str) {
    let theme = THEME.with(|t| t.borrow().clone());
    let vis = main_visible(mtm);
    let (w, h) = (320.0, 44.0);
    let rect =
        NSRect::new(NSPoint::new(vis.origin.x + (vis.size.width - w) / 2.0, vis.origin.y + 80.0), NSSize::new(w, h));
    FLASH.with(|f| {
        let mut f = f.borrow_mut();
        if f.is_none() {
            let panel = NSPanel::initWithContentRect_styleMask_backing_defer(
                NSPanel::alloc(mtm),
                rect,
                NSWindowStyleMask::Borderless | NSWindowStyleMask::NonactivatingPanel,
                NSBackingStoreType::Buffered,
                false,
            );
            unsafe { panel.setReleasedWhenClosed(false) };
            panel.setIgnoresMouseEvents(true);
            panel.setLevel(objc2_app_kit::NSStatusWindowLevel);
            panel.setHasShadow(true);
            panel.setCollectionBehavior(
                NSWindowCollectionBehavior::CanJoinAllSpaces | NSWindowCollectionBehavior::Transient,
            );
            let label = NSTextField::labelWithString(&NSString::from_str(""), mtm);
            label.setFrame(NSRect::new(NSPoint::new(0.0, 12.0), NSSize::new(w, 20.0)));
            label.setAlignment(NSTextAlignment::Center);
            label.setFont(Some(&NSFont::boldSystemFontOfSize(14.0)));
            panel.contentView().expect("panel has a content view").addSubview(&label);
            *f = Some((panel, label));
        }
        let (panel, label) = f.as_ref().expect("just created");
        panel.setBackgroundColor(Some(&ns_color(theme.surface.with_alpha(0xee))));
        label.setTextColor(Some(&ns_color(theme.accent)));
        label.setStringValue(&NSString::from_str(&format!("{title}  ·  {body}")));
        panel.setFrame_display(rect, true);
        panel.orderFrontRegardless();
    });
    let gen = FLASH_GEN.get() + 1;
    FLASH_GEN.set(gen);
    after(900, move || {
        if FLASH_GEN.get() == gen {
            FLASH.with(|f| {
                if let Some((p, _)) = f.borrow().as_ref() {
                    p.orderOut(None);
                }
            });
        }
    });
}

/// Toggle a window listing every binding, grouped by feature.
pub fn show_keys(mtm: MainThreadMarker, keys: &[KeyHelp]) {
    let open = KEYS.with(|k| k.borrow().as_ref().is_some_and(|p| p.isVisible()));
    if open {
        KEYS.with(|k| k.borrow().as_ref().map(|p| p.orderOut(None)));
        return;
    }
    let mut sorted: Vec<&KeyHelp> = keys.iter().collect();
    sorted.sort_by(|a, b| a.feature.cmp(&b.feature).then(a.description.cmp(&b.description)));
    let mut text = String::new();
    let mut last = "";
    for k in sorted {
        if k.feature != last {
            text.push_str(&format!("\n{}\n", k.feature.to_uppercase()));
            last = &k.feature;
        }
        text.push_str(&format!("  {:<30} {}\n", k.chord, k.description));
    }

    let theme = THEME.with(|t| t.borrow().clone());
    let vis = main_visible(mtm);
    let (w, h) = (640.0, (vis.size.height * 0.8).min(820.0));
    let rect = NSRect::new(
        NSPoint::new(vis.origin.x + (vis.size.width - w) / 2.0, vis.origin.y + (vis.size.height - h) / 2.0),
        NSSize::new(w, h),
    );
    let panel = NSPanel::initWithContentRect_styleMask_backing_defer(
        NSPanel::alloc(mtm),
        rect,
        NSWindowStyleMask::Titled | NSWindowStyleMask::Closable | NSWindowStyleMask::Resizable,
        NSBackingStoreType::Buffered,
        false,
    );
    unsafe { panel.setReleasedWhenClosed(false) };
    panel.setTitle(&NSString::from_str("GridPilot keys"));
    panel.setLevel(objc2_app_kit::NSFloatingWindowLevel);
    let scroll = NSTextView::scrollableTextView(mtm);
    if let Some(doc) = scroll.documentView() {
        if let Ok(tv) = doc.downcast::<NSTextView>() {
            tv.setEditable(false);
            tv.setString(&NSString::from_str(&text));
            tv.setFont(Some(&NSFont::monospacedSystemFontOfSize_weight(12.0, unsafe { NSFontWeightRegular })));
            tv.setBackgroundColor(&ns_color(theme.surface));
            tv.setTextColor(Some(&ns_color(theme.text)));
        }
    }
    panel.setContentView(Some(&scroll));
    panel.setBackgroundColor(Some(&NSColor::windowBackgroundColor()));
    panel.makeKeyAndOrderFront(None);
    #[allow(deprecated)]
    NSApplication::sharedApplication(mtm).activateIgnoringOtherApps(true);
    KEYS.with(|k| *k.borrow_mut() = Some(panel));
}
