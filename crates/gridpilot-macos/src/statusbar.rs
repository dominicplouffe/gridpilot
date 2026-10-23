//! The menu bar item.

use crate::app::post_command;
use objc2::rc::Retained;
use objc2::runtime::{AnyObject, NSObject};
use objc2::{define_class, msg_send, sel, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{NSImage, NSMenu, NSMenuItem, NSStatusBar, NSStatusItem, NSVariableStatusItemLength};
use objc2_foundation::{NSObjectProtocol, NSString};
use std::cell::RefCell;

/// (title, command). Empty title = separator.
const ITEMS: &[(&str, &str)] = &[
    ("Show Grid", "grid show"),
    ("Show Keys", "keys"),
    ("", ""),
    ("Layout: Floating (Divvy)", "layout floating"),
    ("Layout: Dwindle", "layout dwindle"),
    ("Layout: Scrolling", "layout scrolling"),
    ("Layout: Master", "layout master"),
    ("", ""),
    ("Next Theme", "theme next"),
    ("Reload Config", "reload"),
    ("Open Config", "exec open -t \"${GRIDPILOT_CONFIG:-$HOME/.config/gridpilot/gridpilot.toml}\""),
    ("", ""),
    ("Quit GridPilot", "quit"),
];

thread_local! {
    static ITEM: RefCell<Option<(Retained<NSStatusItem>, Retained<MenuTarget>)>> = const { RefCell::new(None) };
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "GPMenuTarget"]
    pub struct MenuTarget;

    unsafe impl NSObjectProtocol for MenuTarget {}

    impl MenuTarget {
        #[unsafe(method(runCommand:))]
        fn run_command(&self, sender: &AnyObject) {
            let tag: isize = unsafe { msg_send![sender, tag] };
            if let Some((_, cmd)) = ITEMS.get(tag as usize) {
                post_command(cmd.to_string());
            }
        }
    }
);

pub fn install(mtm: MainThreadMarker) {
    let target: Retained<MenuTarget> = unsafe { msg_send![MenuTarget::alloc(mtm), init] };
    let item = NSStatusBar::systemStatusBar().statusItemWithLength(NSVariableStatusItemLength);
    if let Some(button) = item.button(mtm) {
        let image = NSImage::imageWithSystemSymbolName_accessibilityDescription(
            &NSString::from_str("square.grid.3x3.topleft.filled"),
            Some(&NSString::from_str("GridPilot")),
        );
        match image {
            Some(img) => {
                img.setTemplate(true);
                button.setImage(Some(&img));
            }
            None => button.setTitle(&NSString::from_str("▦")),
        }
    }
    let menu = NSMenu::new(mtm);
    for (i, (title, _)) in ITEMS.iter().enumerate() {
        if title.is_empty() {
            menu.addItem(&NSMenuItem::separatorItem(mtm));
            continue;
        }
        let mi = unsafe {
            NSMenuItem::initWithTitle_action_keyEquivalent(
                NSMenuItem::alloc(mtm),
                &NSString::from_str(title),
                Some(sel!(runCommand:)),
                &NSString::from_str(""),
            )
        };
        unsafe { mi.setTarget(Some(&target)) };
        mi.setTag(i as isize);
        menu.addItem(&mi);
    }
    item.setMenu(Some(&menu));
    ITEM.with(|s| *s.borrow_mut() = Some((item, target)));
}
