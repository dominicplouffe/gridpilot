//! End-to-end behaviour of the engine with the built-in features, driven
//! through the same command lines a key binding or the CLI would send.

use gridpilot_core::config::Config;
use gridpilot_core::features::builtin;
use gridpilot_core::geometry::Rect;
use gridpilot_core::keys::Chord;
use gridpilot_core::testing::MockPlatform;
use gridpilot_core::{Engine, PlatformEvent, UiRequest, WindowId};

fn engine_with(displays: usize, config: Config) -> (Engine, MockPlatform) {
    let p = MockPlatform::with_displays(displays);
    let mut e = Engine::new(Box::new(p.clone()), config, builtin());
    e.start().unwrap();
    (e, p)
}

fn tiling() -> Config {
    let mut c = Config::default();
    c.tiling.default_layout = "dwindle".into();
    c
}

fn open(e: &mut Engine, p: &MockPlatform, id: u64) -> WindowId {
    let w = p.open(id, "app", Rect::new(100.0, 100.0, 800.0, 600.0));
    e.refresh().unwrap();
    w
}

#[test]
fn no_binding_conflicts_in_defaults() {
    let (e, _) = engine_with(1, Config::default());
    assert!(e.warnings.is_empty(), "default bindings conflict: {:#?}", e.warnings);
    assert!(e.keymap().len() > 80, "expected the full Omarchy-style keymap, got {}", e.keymap().len());
}

#[test]
fn dwindle_tiles_new_windows() {
    let (mut e, p) = engine_with(1, tiling());
    let a = open(&mut e, &p, 1);
    assert_eq!(p.state().frame(a), Rect::new(10.0, 50.0, 1900.0, 1020.0));
    let b = open(&mut e, &p, 2);
    assert_eq!(p.state().frame(a), Rect::new(10.0, 50.0, 945.0, 1020.0));
    assert_eq!(p.state().frame(b), Rect::new(965.0, 50.0, 945.0, 1020.0));

    e.dispatch("focus left").unwrap();
    assert_eq!(e.focused(), Some(a));
    e.dispatch("swap right").unwrap();
    assert_eq!(p.state().frame(a).x, 965.0);

    p.close_window(a);
    e.refresh().unwrap();
    assert_eq!(p.state().frame(b), Rect::new(10.0, 50.0, 1900.0, 1020.0));
}

#[test]
fn grid_shortcut_places_and_cycles_displays() {
    let (mut e, p) = engine_with(2, Config::default());
    let w = open(&mut e, &p, 1);
    e.dispatch("grid shortcut Left half").unwrap();
    // 1920 wide, 10px screen margin, 10px window margin, 6 columns.
    assert_eq!(p.state().frame(w), Rect::new(10.0, 50.0, 945.0, 1020.0));
    // Pressing it again sends the window to the next display.
    e.dispatch("grid shortcut Left half").unwrap();
    assert_eq!(p.state().frame(w), Rect::new(1930.0, 50.0, 945.0, 1020.0));
    assert_eq!(e.world.windows[&w].workspace, "2");
}

#[test]
fn grid_shortcut_hotkey_goes_through_keymap() {
    let (mut e, p) = engine_with(1, Config::default());
    let w = open(&mut e, &p, 1);
    let chord = Chord::parse("ctrl+cmd+right", &Default::default()).unwrap();
    assert!(p.state().hotkeys.contains(&chord));
    e.handle(PlatformEvent::Hotkey(chord));
    assert_eq!(p.state().frame(w).x, 965.0);
}

#[test]
fn grid_overlay_drag_applies_selection() {
    let (mut e, p) = engine_with(1, Config::default());
    let w = open(&mut e, &p, 1);
    e.dispatch("grid show").unwrap();
    assert!(matches!(p.state().ui.last(), Some(UiRequest::ShowGrid(_))));
    e.dispatch("grid press 0 0").unwrap();
    e.dispatch("grid drag 1 1").unwrap();
    e.dispatch("grid release 1 1").unwrap();
    assert!(matches!(p.state().ui.last(), Some(UiRequest::HideGrid)));
    let f = p.state().frame(w);
    assert_eq!((f.x, f.y), (10.0, 50.0));
    // two of six columns, two of four rows
    assert_eq!(f.w, ((1900.0 - 50.0) / 6.0 * 2.0 + 10.0_f64).round());
}

#[test]
fn grid_apply_floats_a_tiled_window() {
    let (mut e, p) = engine_with(1, tiling());
    let a = open(&mut e, &p, 1);
    let b = open(&mut e, &p, 2);
    e.dispatch("grid apply 4,3 2x1 / 6x4").unwrap();
    assert!(e.world.windows[&b].floating);
    // a reclaims the whole screen once b floats
    assert_eq!(p.state().frame(a).w, 1900.0);
}

#[test]
fn workspaces_hide_and_restore_windows() {
    let (mut e, p) = engine_with(1, tiling());
    let a = open(&mut e, &p, 1);
    e.dispatch("move-to-workspace 3 --silent").unwrap();
    assert!(p.state().is_hidden(a));
    assert_eq!(e.world.current_workspace().as_deref(), Some("1"));

    let b = open(&mut e, &p, 2);
    e.dispatch("workspace 3").unwrap();
    assert!(!p.state().is_hidden(a));
    assert!(p.state().is_hidden(b));
    assert_eq!(e.focused(), Some(a));

    e.dispatch("workspace previous").unwrap();
    assert_eq!(e.world.current_workspace().as_deref(), Some("1"));
    assert_eq!(e.focused(), Some(b));
}

#[test]
fn os_focus_on_hidden_window_switches_workspace() {
    let (mut e, p) = engine_with(1, tiling());
    let a = open(&mut e, &p, 1);
    e.dispatch("move-to-workspace 4 --silent").unwrap();
    e.handle(PlatformEvent::WindowFocused(a));
    assert_eq!(e.world.current_workspace().as_deref(), Some("4"));
}

#[test]
fn scratchpad_round_trip() {
    let (mut e, p) = engine_with(1, tiling());
    let a = open(&mut e, &p, 1);
    let _b = open(&mut e, &p, 2);
    e.dispatch("focus left").unwrap();
    e.dispatch("scratchpad move").unwrap();
    assert!(p.state().is_hidden(a));
    e.dispatch("scratchpad toggle").unwrap();
    assert!(!p.state().is_hidden(a));
    assert_eq!(e.focused(), Some(a));
    e.dispatch("scratchpad toggle").unwrap();
    assert!(p.state().is_hidden(a));
}

#[test]
fn layout_cycle_and_floating_freeze() {
    let (mut e, p) = engine_with(1, tiling());
    let a = open(&mut e, &p, 1);
    let _b = open(&mut e, &p, 2);
    let tiled = p.state().frame(a);
    assert_eq!(e.dispatch("layout floating").unwrap().as_deref(), Some("floating"));
    assert_eq!(p.state().frame(a), tiled, "switching to floating keeps windows in place");
    e.dispatch("layout next").unwrap();
    assert_eq!(e.world.workspaces["1"].layout.name(), "dwindle");
}

#[test]
fn groups_stack_windows_in_one_tile() {
    let (mut e, p) = engine_with(1, tiling());
    let a = open(&mut e, &p, 1);
    let b = open(&mut e, &p, 2);
    e.dispatch("group join left").unwrap();
    assert!(p.state().is_hidden(a));
    assert_eq!(p.state().frame(b).w, 1900.0);
    e.dispatch("group next").unwrap();
    assert_eq!(e.focused(), Some(a));
    assert!(p.state().is_hidden(b));
    e.dispatch("group toggle").unwrap();
    assert!(!p.state().is_hidden(b));
}

#[test]
fn pop_floats_pins_and_centres() {
    let (mut e, p) = engine_with(1, tiling());
    let a = open(&mut e, &p, 1);
    e.dispatch("pop").unwrap();
    let f = p.state().frame(a);
    assert_eq!((f.w, f.h), (1300.0, 900.0));
    e.dispatch("workspace 2").unwrap();
    assert!(!p.state().is_hidden(a), "pinned windows follow you across workspaces");
    e.dispatch("workspace 1; pop").unwrap();
    assert!(!e.world.windows[&a].floating);
}

#[test]
fn rules_and_bindings_from_config() {
    let config = Config::parse(
        r#"
        [tiling]
        default_layout = "dwindle"
        [bindings]
        "mod+m" = "none"
        "mod+e" = "exec open -a Mail"
        [[rule]]
        app = "calc"
        float = true
        [[rule]]
        app = "music"
        workspace = "9"
        "#,
    )
    .unwrap();
    let (mut e, p) = engine_with(1, config);
    let c = p.open(1, "calc", Rect::new(0.0, 0.0, 300.0, 400.0));
    let m = p.open(2, "music", Rect::new(0.0, 0.0, 300.0, 400.0));
    e.refresh().unwrap();
    assert!(e.world.windows[&c].floating);
    assert_eq!(e.world.windows[&m].workspace, "9");

    let aliases = &e.config.modifiers.clone();
    assert!(!e.keymap().contains_key(&Chord::parse("mod+m", aliases).unwrap()));
    e.handle(PlatformEvent::Hotkey(Chord::parse("mod+e", aliases).unwrap()));
    assert_eq!(p.state().spawned, vec!["open -a Mail".to_string()]);
}

#[test]
fn disabled_features_drop_commands_and_keys() {
    let config = Config::parse("[features]\ndisabled = [\"groups\"]").unwrap();
    let (mut e, _) = engine_with(1, config);
    assert!(e.dispatch("group toggle").is_err());
    assert!(e.key_help().iter().all(|k| k.feature != "groups"));
}

#[test]
fn unplugging_a_display_moves_its_workspace() {
    let (mut e, p) = engine_with(2, Config::default());
    let w = p.open(1, "app", Rect::new(2000.0, 100.0, 800.0, 600.0));
    e.refresh().unwrap();
    assert_eq!(e.world.windows[&w].workspace, "2");
    p.0.borrow_mut().displays.truncate(1);
    e.handle(PlatformEvent::DisplaysChanged);
    let f = p.state().frame(w);
    assert!(p.state().is_hidden(w) || f.x < 1920.0);
    e.dispatch("workspace 2").unwrap();
    assert!(p.state().frame(w).x < 1920.0, "floating window reprojected onto the remaining display");
}

#[test]
fn app_enforced_size_does_not_cause_a_fight_but_a_drag_snaps_back() {
    let (mut e, p) = engine_with(1, tiling());
    let a = open(&mut e, &p, 1);
    let planned = p.state().frame(a);
    // The app refuses to be that small and keeps a larger height.
    p.0.borrow_mut().windows[0].frame.h = planned.h + 50.0;
    e.refresh().unwrap();
    assert_eq!(p.state().frame(a).h, planned.h + 50.0, "no re-apply for a size the app insists on");
    // The user drags the tiled window away: it snaps back.
    p.0.borrow_mut().windows[0].frame.x = 400.0;
    e.refresh().unwrap();
    assert_eq!(p.state().frame(a), planned);
}

#[test]
fn new_windows_on_a_floating_workspace_are_left_alone() {
    let (mut e, p) = engine_with(1, Config::default());
    open(&mut e, &p, 1);
    open(&mut e, &p, 2);
    e.refresh().unwrap();
    assert_eq!(p.state().set_frame_calls, 0);
}

#[test]
fn a_window_that_cannot_be_moved_is_reported_once_not_every_poll() {
    let (mut e, p) = engine_with(1, tiling());
    let a = WindowId(1);
    p.0.borrow_mut().unmovable.push(a);
    open(&mut e, &p, 1);
    for _ in 0..10 {
        e.refresh().unwrap();
    }
    let failures = e.warnings.iter().filter(|w| w.contains("window 1:")).count();
    assert_eq!(failures, 1, "{:#?}", e.warnings);
    assert_eq!(p.state().set_frame_calls, 1, "not retried while the plan is unchanged");
}

#[test]
fn hotkeys_can_be_paused_for_recording() {
    let (mut e, p) = engine_with(1, Config::default());
    let all = p.state().hotkeys.len();
    assert!(all > 50);
    e.dispatch("hotkeys off").unwrap();
    assert!(p.state().hotkeys.is_empty());
    e.dispatch("hotkeys on").unwrap();
    assert_eq!(p.state().hotkeys.len(), all);
}

#[test]
fn capslock_presses_the_mod_modifiers() {
    use gridpilot_core::keys::Modifiers;
    let (mut e, p) = engine_with(1, Config::default());
    assert_eq!(p.state().capslock, Some(Modifiers::CTRL | Modifiers::ALT | Modifiers::CMD));
    let mut c = Config::default();
    c.modifiers.insert("mod".into(), "ctrl+alt".into());
    e.apply_config(c).unwrap();
    assert_eq!(p.state().capslock, Some(Modifiers::CTRL | Modifiers::ALT));
    let mut c = Config::default();
    c.general.capslock_mod = false;
    e.apply_config(c).unwrap();
    assert_eq!(p.state().capslock, None);
}
