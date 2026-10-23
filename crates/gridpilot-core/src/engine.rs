//! The engine: owns the [`World`], runs commands, reacts to platform events
//! and pushes the resulting window frames to the [`Platform`].
//!
//! Features use the helper methods here (`show_workspace`, `set_floating`,
//! `move_window_to_workspace`, ...) instead of poking at the world directly,
//! so invariants such as "a workspace's layout and tiles agree" hold in one
//! place.

use crate::command::{parse_line, CmdResult, CommandRegistry};
use crate::config::{Config, Rule};
use crate::feature::{DefaultBinding, EngineEvent, Feature, Hook, Registrar};
use crate::geometry::{nearest_in_direction, Direction, Rect};
use crate::grid::GridSelection;
use crate::keys::Chord;
use crate::layout::{rebuild, LayoutOptions, LayoutRegistry};
use crate::platform::{DisplayId, KeyHelp, Platform, PlatformEvent, UiRequest, WindowId, WindowInfo};
use crate::theme::Theme;
use crate::world::{workspace_order, ManagedWindow, Placement, Workspace, World, SCRATCHPAD};
use std::any::{Any, TypeId};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub struct Binding {
    pub chord: Chord,
    pub command: String,
    pub description: String,
    pub feature: String,
}

pub struct Engine {
    pub world: World,
    pub config: Config,
    pub theme: Theme,
    pub config_path: Option<PathBuf>,
    platform: Box<dyn Platform>,
    features: Vec<Box<dyn Feature>>,
    installed: Vec<(&'static str, &'static str)>,
    commands: CommandRegistry,
    layouts: LayoutRegistry,
    hooks: Vec<(String, Hook)>,
    defaults: Vec<DefaultBinding>,
    keymap: BTreeMap<Chord, Binding>,
    applied: HashMap<WindowId, Placement>,
    /// Last platform error per window, so each one is reported once.
    failures: HashMap<WindowId, String>,
    extensions: HashMap<TypeId, Box<dyn Any>>,
    pending: Vec<EngineEvent>,
    focus_dirty: bool,
    pub warnings: Vec<String>,
    pub quit_requested: bool,
}

impl Engine {
    pub fn new(platform: Box<dyn Platform>, config: Config, features: Vec<Box<dyn Feature>>) -> Engine {
        let theme = Theme::builtin(&config.theme.name).unwrap_or_default();
        let mut e = Engine {
            world: World::default(),
            config,
            theme,
            config_path: None,
            platform,
            features,
            installed: Vec::new(),
            commands: CommandRegistry::default(),
            layouts: LayoutRegistry::default(),
            hooks: Vec::new(),
            defaults: Vec::new(),
            keymap: BTreeMap::new(),
            applied: HashMap::new(),
            failures: HashMap::new(),
            extensions: HashMap::new(),
            pending: Vec::new(),
            focus_dirty: false,
            warnings: Vec::new(),
            quit_requested: false,
        };
        e.install_features();
        e
    }

    fn install_features(&mut self) {
        self.commands = CommandRegistry::default();
        self.layouts = LayoutRegistry::default();
        self.hooks.clear();
        self.defaults.clear();
        self.installed.clear();
        for f in &self.features {
            if self.config.features.disabled.iter().any(|d| d == f.id()) {
                continue;
            }
            let mut r = Registrar {
                feature: f.id(),
                config: &self.config,
                commands: &mut self.commands,
                bindings: &mut self.defaults,
                layouts: &mut self.layouts,
                hooks: &mut self.hooks,
            };
            f.install(&mut r);
            self.installed.push((f.id(), f.summary()));
        }
        self.build_keymap();
    }

    fn build_keymap(&mut self) {
        let aliases = &self.config.modifiers;
        let mut map: BTreeMap<Chord, Binding> = BTreeMap::new();
        let mut warnings = Vec::new();
        for b in &self.defaults {
            if !self.config.general.default_bindings && !b.from_config {
                continue;
            }
            match Chord::parse(&b.chord, aliases) {
                Ok(chord) => {
                    if let Some(prev) = map.get(&chord) {
                        warnings.push(format!(
                            "{chord} is bound by both `{}` ({}) and `{}` ({}); keeping the latter",
                            prev.command, prev.feature, b.command, b.feature
                        ));
                    }
                    map.insert(
                        chord,
                        Binding {
                            chord,
                            command: b.command.clone(),
                            description: b.description.clone(),
                            feature: b.feature.clone(),
                        },
                    );
                }
                Err(e) => warnings.push(format!("binding `{}`: {e}", b.chord)),
            }
        }
        for (spec, cmd) in &self.config.bindings {
            match Chord::parse(spec, aliases) {
                Ok(chord) if cmd.trim() == "none" => {
                    map.remove(&chord);
                }
                Ok(chord) => {
                    map.insert(
                        chord,
                        Binding { chord, command: cmd.clone(), description: cmd.clone(), feature: "config".into() },
                    );
                }
                Err(e) => warnings.push(format!("[bindings] `{spec}`: {e}")),
            }
        }
        self.keymap = map;
        self.warnings.extend(warnings);
    }

    /// Read displays and windows from the platform and register hotkeys.
    pub fn start(&mut self) -> Result<(), String> {
        self.sync_displays()?;
        let chords: Vec<Chord> = self.keymap.keys().copied().collect();
        self.platform.register_hotkeys(&chords)?;
        self.apply_capslock();
        self.refresh()?;
        let theme = self.theme.clone();
        self.platform.present(UiRequest::ThemeChanged(theme));
        Ok(())
    }

    // ----- accessors -----------------------------------------------------

    pub fn platform(&mut self) -> &mut dyn Platform {
        self.platform.as_mut()
    }

    pub fn commands(&self) -> &CommandRegistry {
        &self.commands
    }

    pub fn layouts(&self) -> &LayoutRegistry {
        &self.layouts
    }

    pub fn keymap(&self) -> &BTreeMap<Chord, Binding> {
        &self.keymap
    }

    /// Bindings as features declared them, before config overrides.
    pub fn default_bindings(&self) -> &[DefaultBinding] {
        &self.defaults
    }

    /// The modifiers Caps Lock should press: those of the `mod` alias.
    pub fn capslock_modifiers(&self) -> Option<crate::keys::Modifiers> {
        if !self.config.general.capslock_mod {
            return None;
        }
        Chord::parse("mod+a", &self.config.modifiers).ok().map(|c| c.mods).filter(|m| !m.is_empty())
    }

    fn apply_capslock(&mut self) {
        let mods = self.capslock_modifiers();
        if let Err(e) = self.platform.set_capslock_modifiers(mods) {
            // Leave Caps Lock alone; mod still works by holding its keys.
            let _ = self.platform.set_capslock_modifiers(None);
            self.warn(format!("Caps Lock can't act as mod: {e}"));
        }
    }

    /// Turn global hotkeys off (e.g. while the settings page records a
    /// shortcut, so GridPilot doesn't swallow the keys) or back on.
    pub fn set_hotkeys_enabled(&mut self, on: bool) -> Result<(), String> {
        let chords: Vec<Chord> = if on { self.keymap.keys().copied().collect() } else { Vec::new() };
        self.platform.register_hotkeys(&chords)
    }

    pub fn installed_features(&self) -> &[(&'static str, &'static str)] {
        &self.installed
    }

    pub fn layout_options(&self) -> LayoutOptions {
        self.config.layout_options()
    }

    /// Per-feature state, created on first use.
    pub fn ext<T: Any + Default>(&mut self) -> &mut T {
        self.extensions
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(T::default()))
            .downcast_mut::<T>()
            .expect("extension type matches its TypeId")
    }

    pub fn emit(&mut self, event: EngineEvent) {
        self.pending.push(event);
    }

    pub fn present(&mut self, ui: UiRequest) {
        self.platform.present(ui);
    }

    pub fn key_help(&self) -> Vec<KeyHelp> {
        self.keymap
            .values()
            .map(|b| KeyHelp {
                chord: b.chord.to_string(),
                command: b.command.clone(),
                description: b.description.clone(),
                feature: b.feature.clone(),
            })
            .collect()
    }

    // ----- commands ------------------------------------------------------

    /// Run a command line and apply the result.
    pub fn dispatch(&mut self, line: &str) -> CmdResult {
        let out = self.run(line);
        self.flush();
        out
    }

    /// Run without flushing; for commands composed of other commands.
    pub fn run(&mut self, line: &str) -> CmdResult {
        let mut output = Vec::new();
        for args in parse_line(line)? {
            let spec = self
                .commands
                .get(&args.name)
                .ok_or_else(|| format!("unknown command `{}` (try `gridpilot commands`)", args.name))?;
            let handler = spec.handler.clone();
            if let Some(text) = handler(self, &args)? {
                output.push(text);
            }
        }
        Ok((!output.is_empty()).then(|| output.join("\n")))
    }

    pub fn handle(&mut self, event: PlatformEvent) {
        let result = match event {
            PlatformEvent::Hotkey(chord) => match self.keymap.get(&chord).map(|b| b.command.clone()) {
                Some(cmd) => self.run(&cmd).map(|_| ()),
                None => Ok(()),
            },
            PlatformEvent::WindowCreated(info) => {
                if !self.world.windows.contains_key(&info.id) {
                    self.add_window(info);
                }
                Ok(())
            }
            PlatformEvent::WindowDestroyed(id) => {
                self.remove_window(id);
                Ok(())
            }
            PlatformEvent::WindowFocused(id) => {
                self.note_os_focus(id);
                Ok(())
            }
            PlatformEvent::WindowMoved { id, frame } => {
                self.note_user_move(id, frame);
                Ok(())
            }
            PlatformEvent::DisplaysChanged => self.sync_displays(),
        };
        if let Err(e) = result {
            self.warn(e);
        }
        self.flush();
    }

    pub fn warn(&mut self, message: impl Into<String>) {
        let m = message.into();
        eprintln!("gridpilot: {m}");
        self.warnings.push(m);
        if self.warnings.len() > 100 {
            self.warnings.remove(0);
        }
    }

    // ----- syncing with the platform ------------------------------------

    pub fn sync_displays(&mut self) -> Result<(), String> {
        let displays = self.platform.displays()?;
        if displays.is_empty() || displays == self.world.displays {
            return Ok(());
        }
        let old: BTreeMap<DisplayId, Rect> = self.world.displays.iter().map(|d| (d.id, d.visible)).collect();
        self.world.displays = displays;
        let primary = self.world.primary_display().expect("at least one display");
        let live: BTreeSet<DisplayId> = self.world.displays.iter().map(|d| d.id).collect();

        // Workspaces of unplugged displays move to the primary one; floating
        // windows keep their relative position.
        let orphans: Vec<String> =
            self.world.workspaces.values().filter(|ws| !live.contains(&ws.display)).map(|ws| ws.name.clone()).collect();
        let to = self.world.display(primary).map(|d| d.visible).unwrap_or_default();
        for name in orphans {
            let ws = self.world.workspaces.get_mut(&name).expect("listed above");
            let from = old.get(&ws.display).copied().unwrap_or(to);
            ws.display = primary;
            let ids: Vec<WindowId> = ws.windows().collect();
            for id in ids {
                if let Some(w) = self.world.windows.get_mut(&id) {
                    w.float_frame = w.float_frame.reproject(&from, &to);
                }
            }
        }
        self.world.active.retain(|d, _| live.contains(d));
        self.world.previous.retain(|d, _| live.contains(d));
        if self.world.scratchpad_on.is_some_and(|d| !live.contains(&d)) {
            self.world.scratchpad_on = None;
        }
        let ids: Vec<DisplayId> = self.world.displays.iter().map(|d| d.id).collect();
        for d in ids {
            if !self.world.active.contains_key(&d) {
                let name = self.free_workspace_for(d);
                self.ensure_workspace(&name, d);
                self.world.active.insert(d, name);
            }
        }
        self.applied.clear();
        Ok(())
    }

    /// A workspace to show on a display that has none: one already assigned
    /// to it, else the lowest number not visible anywhere.
    fn free_workspace_for(&self, display: DisplayId) -> String {
        let visible: BTreeSet<&String> = self.world.active.values().collect();
        let mut own: Vec<&Workspace> = self
            .world
            .workspaces
            .values()
            .filter(|ws| ws.display == display && ws.name != SCRATCHPAD && !visible.contains(&ws.name))
            .collect();
        own.sort_by(|a, b| workspace_order(&a.name, &b.name));
        if let Some(ws) = own.first() {
            return ws.name.clone();
        }
        (1..)
            .map(|n: u32| n.to_string())
            .find(|n| {
                !visible.contains(n)
                    && self.world.workspaces.get(n).is_none_or(|ws| ws.display == display || ws.is_empty())
            })
            .expect("unbounded")
    }

    /// Poll windows and focus from the platform.
    pub fn refresh(&mut self) -> Result<(), String> {
        self.sync_displays()?;
        let windows = self.platform.windows()?;
        let mut seen = BTreeSet::new();
        for info in windows {
            if info.is_minimized {
                continue;
            }
            seen.insert(info.id);
            match self.world.windows.get_mut(&info.id) {
                Some(w) => {
                    w.info.title = info.title.clone();
                    // A window we failed to move isn't where we put it; that
                    // is not the user dragging it.
                    let expected =
                        self.applied.get(&info.id).copied().filter(|_| !self.failures.contains_key(&info.id));
                    if let Some(Placement::Show(r)) = expected {
                        // Apps with a minimum size keep their own size inside
                        // a tile; only a changed position means the user moved it.
                        let tiled = w.tile.is_some() && !w.floating;
                        let moved = (info.frame.x - r.x).abs() > 2.0 || (info.frame.y - r.y).abs() > 2.0;
                        if moved || (!tiled && !info.frame.approx_eq(&r, 2.0)) {
                            self.note_user_move(info.id, info.frame);
                        }
                    }
                }
                None => self.add_window(info),
            }
        }
        let gone: Vec<WindowId> = self.world.windows.keys().filter(|id| !seen.contains(id)).copied().collect();
        for id in gone {
            self.remove_window(id);
        }
        if let Some(id) = self.platform.focused_window()? {
            if Some(id) != self.world.focused {
                self.note_os_focus(id);
            }
        }
        self.flush();
        Ok(())
    }

    fn rule_for(&self, info: &WindowInfo) -> Option<Rule> {
        self.config.rules.iter().find(|r| r.matches(&info.app, &info.title)).cloned()
    }

    pub fn add_window(&mut self, info: WindowInfo) {
        let rule = self.rule_for(&info).unwrap_or_default();
        if rule.ignore {
            return;
        }
        let Some(display) = self.world.display_for_frame(&info.frame) else { return };
        let ws_name = rule
            .workspace
            .clone()
            .or_else(|| self.world.active_workspace(display).map(str::to_string))
            .unwrap_or_else(|| "1".into());
        self.ensure_workspace(&ws_name, display);
        let ws_display = self.world.workspaces[&ws_name].display;
        let mut float_frame = info.frame;
        let mut floating = rule.float.unwrap_or(false);
        if let Some(sel) = rule.grid.as_deref().and_then(|g| g.parse::<GridSelection>().ok()) {
            if let Some(d) = self.world.display(ws_display) {
                float_frame = sel.frame(d.visible, &self.config.grid_margins());
                floating = true;
            }
        }
        let id = info.id;
        // The window is already where it is: only move it if the plan says
        // otherwise, so floating (Divvy) workspaces never touch new windows.
        self.applied.insert(id, Placement::Show(info.frame));
        self.world.windows.insert(
            id,
            ManagedWindow {
                info,
                workspace: ws_name.clone(),
                tile: None,
                floating,
                float_frame,
                pinned: false,
                fullscreen: None,
                tags: BTreeSet::new(),
            },
        );
        // Floating layouts keep windows where they opened; a window given a
        // grid rule floats so tiling layouts don't take it over.
        self.world.attach(id, &ws_name, floating);
        self.emit(EngineEvent::WindowAdded(id));
    }

    pub fn remove_window(&mut self, id: WindowId) {
        if !self.world.windows.contains_key(&id) {
            return;
        }
        let ws = self.world.windows[&id].workspace.clone();
        self.world.detach(id);
        self.world.windows.remove(&id);
        self.applied.remove(&id);
        self.failures.remove(&id);
        if self.world.focused == Some(id) {
            self.world.focused = None;
            self.focus_fallback(&ws);
        }
        self.emit(EngineEvent::WindowRemoved(id));
    }

    fn note_os_focus(&mut self, id: WindowId) {
        let Some(w) = self.world.windows.get(&id) else { return };
        let ws = w.workspace.clone();
        // Cmd-Tab or a Dock click on a window parked on a hidden workspace:
        // bring the workspace forward, as AeroSpace does.
        if ws != SCRATCHPAD && !self.world.is_visible_workspace(&ws) && !w.pinned {
            self.show_workspace(&ws);
        }
        self.world.focused = Some(id);
        self.note_focus(id);
    }

    fn note_focus(&mut self, id: WindowId) {
        let Some(w) = self.world.windows.get(&id) else { return };
        let ws_name = w.workspace.clone();
        let tile = w.tile;
        if let Some(d) = self.world.workspaces.get(&ws_name).map(|ws| ws.display) {
            if ws_name != SCRATCHPAD {
                self.world.focused_display = Some(d);
            }
        }
        let area = self.workspace_area(&ws_name);
        let opts = self.layout_options();
        if let Some(ws) = self.world.workspaces.get_mut(&ws_name) {
            ws.last_focused = Some(id);
            if let Some(t) = tile {
                if let Some(tile) = ws.tiles.get_mut(&t) {
                    if let Some(i) = tile.windows.iter().position(|x| *x == id) {
                        tile.active = i;
                    }
                }
                ws.layout.on_focus(t, area, &opts);
            }
        }
        self.emit(EngineEvent::FocusChanged(Some(id)));
    }

    fn note_user_move(&mut self, id: WindowId, frame: Rect) {
        let Some(w) = self.world.windows.get(&id) else { return };
        let ws_name = w.workspace.clone();
        let manages = self.world.workspaces.get(&ws_name).is_some_and(|ws| ws.layout.manages_geometry());
        let free = w.floating || !manages || w.pinned;
        let from_display = self.world.workspaces.get(&ws_name).map(|ws| ws.display);
        let to_display = self.world.display_for_frame(&frame);
        if free {
            if let Some(w) = self.world.windows.get_mut(&id) {
                w.float_frame = frame;
                w.fullscreen = None;
            }
            self.applied.insert(id, Placement::Show(frame));
        } else {
            // A tiled window dragged by hand snaps back on the next flush.
            self.applied.remove(&id);
        }
        if ws_name != SCRATCHPAD && to_display.is_some() && to_display != from_display {
            if let Some(target) = to_display.and_then(|d| self.world.active_workspace(d)).map(str::to_string) {
                self.move_window_to_workspace(id, &target, false);
            }
        }
    }

    // ----- helpers for features -----------------------------------------

    pub fn focused(&self) -> Option<WindowId> {
        self.world.focused.filter(|id| self.world.windows.contains_key(id))
    }

    pub fn require_focused(&self) -> Result<WindowId, String> {
        self.focused().ok_or_else(|| "no focused window".to_string())
    }

    pub fn current_display(&self) -> Result<DisplayId, String> {
        self.world.current_display().ok_or_else(|| "no displays".to_string())
    }

    /// Usable area of the display a workspace is on, inside the outer gaps.
    pub fn workspace_area(&self, ws: &str) -> Rect {
        self.world
            .workspaces
            .get(ws)
            .and_then(|w| self.world.display(w.display))
            .map(|d| d.visible.inset_by(self.config.tiling.gaps_outer))
            .unwrap_or_default()
    }

    /// The frame a window currently has, according to what we applied.
    pub fn frame_of(&self, id: WindowId) -> Option<Rect> {
        match self.applied.get(&id) {
            Some(Placement::Show(r)) => Some(*r),
            _ => self.world.windows.get(&id).map(|w| w.info.frame),
        }
    }

    /// Frames of the windows currently shown, for directional navigation.
    pub fn visible_frames(&self) -> Vec<(WindowId, Rect)> {
        self.world
            .plan(&self.layout_options())
            .into_iter()
            .filter_map(|(id, p)| match p {
                Placement::Show(r) => Some((id, r)),
                Placement::Hide(_) => None,
            })
            .collect()
    }

    pub fn focus_window(&mut self, id: WindowId) {
        if !self.world.windows.contains_key(&id) {
            return;
        }
        self.world.focused = Some(id);
        self.focus_dirty = true;
        self.note_focus(id);
    }

    /// The window in `dir` from the focused one, across displays.
    pub fn neighbour(&self, dir: Direction) -> Option<WindowId> {
        let me = self.focused()?;
        let frames = self.visible_frames();
        let from = frames.iter().find(|(id, _)| *id == me).map(|(_, r)| *r)?;
        nearest_in_direction(&from, dir, frames.iter().filter(|(id, _)| *id != me).map(|(id, r)| (*id, r)))
    }

    pub fn ensure_workspace(&mut self, name: &str, display: DisplayId) {
        if self.world.workspaces.contains_key(name) {
            return;
        }
        let display = self
            .config
            .workspaces
            .display
            .get(name)
            .and_then(|i| self.world.displays.get(*i))
            .map(|d| d.id)
            .unwrap_or(display);
        let layout_name = self.config.workspaces.layout.get(name).unwrap_or(&self.config.tiling.default_layout).clone();
        let layout = self
            .layouts
            .create(&layout_name)
            .or_else(|| self.layouts.create("floating"))
            .expect("floating layout is always registered");
        self.world.workspaces.insert(
            name.to_string(),
            Workspace {
                name: name.to_string(),
                display,
                layout,
                tiles: BTreeMap::new(),
                floating: Vec::new(),
                last_focused: None,
            },
        );
    }

    /// Focus some window of `ws` after its focused window went away.
    pub fn focus_fallback(&mut self, ws: &str) {
        let pick = self.world.workspaces.get(ws).and_then(|w| w.last_focused.or_else(|| w.windows().next()));
        match pick {
            Some(id) => self.focus_window(id),
            None => {
                self.world.focused = None;
                self.emit(EngineEvent::FocusChanged(None));
            }
        }
    }

    /// Show workspace `name` on its display (creating it on the current
    /// display first if needed) and focus it.
    pub fn show_workspace(&mut self, name: &str) {
        let Ok(current) = self.current_display() else { return };
        self.ensure_workspace(name, current);
        let display = self.world.workspaces[name].display;
        self.world.scratchpad_on = None;
        let prev = self.world.active.insert(display, name.to_string());
        if let Some(prev) = prev.filter(|p| p != name) {
            self.world.previous.insert(display, prev);
        }
        self.world.focused_display = Some(display);
        self.focus_fallback(name);
        self.emit(EngineEvent::WorkspaceShown { display, name: name.to_string() });
    }

    pub fn move_window_to_workspace(&mut self, id: WindowId, name: &str, follow: bool) {
        let Some(w) = self.world.windows.get(&id) else { return };
        let from = w.workspace.clone();
        if from == name {
            return;
        }
        let floating = w.floating || name == SCRATCHPAD;
        let current = self.current_display().unwrap_or(DisplayId(0));
        self.ensure_workspace(name, current);
        let from_area =
            self.world.workspaces.get(&from).and_then(|ws| self.world.display(ws.display)).map(|d| d.visible);
        let to_area = self.world.display(self.world.workspaces[name].display).map(|d| d.visible);
        self.world.detach(id);
        if let (Some(a), Some(b), Some(w)) = (from_area, to_area, self.world.windows.get_mut(&id)) {
            if a != b {
                w.float_frame = w.float_frame.reproject(&a, &b);
            }
            w.fullscreen = None;
        }
        self.world.attach(id, name, floating);
        if follow {
            if name == SCRATCHPAD {
                self.world.scratchpad_on = Some(current);
            } else {
                self.show_workspace(name);
            }
            self.focus_window(id);
        } else if self.world.focused == Some(id) {
            self.world.focused = None;
            self.focus_fallback(&from);
        }
    }

    pub fn set_floating(&mut self, id: WindowId, floating: bool) {
        let Some(w) = self.world.windows.get(&id) else { return };
        if w.floating == floating {
            return;
        }
        let ws = w.workspace.clone();
        let current_frame = self.frame_of(id);
        self.world.detach(id);
        if let Some(w) = self.world.windows.get_mut(&id) {
            w.fullscreen = None;
            if floating {
                if let Some(f) = current_frame {
                    // Don't jump: float where the tile was, a bit inset.
                    w.float_frame = f.inset_by(24.0);
                }
            }
        }
        self.world.attach(id, &ws, floating);
        if self.world.focused == Some(id) {
            self.note_focus(id);
        }
    }

    /// Replace the layout of a workspace, keeping its tiles in order.
    pub fn set_layout(&mut self, ws_name: &str, layout: &str) -> Result<(), String> {
        let new = self.layouts.create(layout).ok_or_else(|| {
            format!("unknown layout `{layout}` (have: {})", self.layouts.names().collect::<Vec<_>>().join(", "))
        })?;
        let area = self.workspace_area(ws_name);
        // Leaving a tiling layout: freeze each window where it is now so a
        // floating workspace starts from the current arrangement.
        if !new.manages_geometry() {
            let ids: Vec<WindowId> = self
                .world
                .workspaces
                .get(ws_name)
                .map(|ws| ws.tiles.values().flat_map(|t| t.windows.clone()).collect())
                .unwrap_or_default();
            for id in ids {
                if let Some(Placement::Show(r)) = self.applied.get(&id).copied() {
                    if let Some(w) = self.world.windows.get_mut(&id) {
                        w.float_frame = r;
                    }
                }
            }
        }
        let ws = self.world.workspaces.get_mut(ws_name).ok_or_else(|| format!("no workspace `{ws_name}`"))?;
        ws.layout = rebuild(ws.layout.as_ref(), new, area);
        self.emit(EngineEvent::LayoutChanged { workspace: ws_name.to_string(), layout: layout.to_string() });
        Ok(())
    }

    pub fn reload(&mut self) -> Result<(), String> {
        let path = self.config_path.clone().unwrap_or_else(Config::default_path);
        let config = if path.exists() { Config::load(&path)? } else { Config::default() };
        self.apply_config(config)
    }

    pub fn apply_config(&mut self, config: Config) -> Result<(), String> {
        self.config = config;
        self.theme = crate::features::system::load_theme(&self.config.theme.name).unwrap_or_default();
        self.install_features();
        let chords: Vec<Chord> = self.keymap.keys().copied().collect();
        self.platform.register_hotkeys(&chords)?;
        self.apply_capslock();
        let theme = self.theme.clone();
        self.platform.present(UiRequest::ThemeChanged(theme));
        self.applied.clear();
        self.emit(EngineEvent::ConfigReloaded);
        Ok(())
    }

    /// Push planned frames to the platform and run event hooks.
    pub fn flush(&mut self) {
        for _ in 0..8 {
            let events = std::mem::take(&mut self.pending);
            if events.is_empty() {
                break;
            }
            let hooks: Vec<Hook> = self.hooks.iter().map(|(_, h)| h.clone()).collect();
            for e in &events {
                for h in &hooks {
                    h(self, e);
                }
            }
        }
        self.collect_empty_workspaces();

        let plan = self.world.plan(&self.layout_options());
        let mut errors = Vec::new();
        for (id, p) in plan {
            if self.applied.get(&id) == Some(&p) {
                continue;
            }
            let r = match p {
                Placement::Show(r) => self.platform.set_frame(id, r),
                Placement::Hide(d) => match self.world.display(d).cloned() {
                    Some(info) => self.platform.hide(id, &info),
                    None => Ok(()),
                },
            };
            // Record the attempt even when it failed, so an unmovable window
            // isn't retried (and reported) on every poll; it is tried again
            // when its planned placement changes.
            self.applied.insert(id, p);
            match r {
                Ok(()) => {
                    self.failures.remove(&id);
                }
                Err(e) => {
                    if self.failures.get(&id) != Some(&e) {
                        errors.push(format!("window {id}: {e}"));
                        self.failures.insert(id, e);
                    }
                }
            }
        }
        if self.focus_dirty {
            self.focus_dirty = false;
            if let Some(id) = self.focused() {
                if let Err(e) = self.platform.focus(id) {
                    errors.push(format!("focus {id}: {e}"));
                }
                if self.config.general.mouse_follows_focus {
                    if let Some(Placement::Show(r)) = self.applied.get(&id).copied() {
                        self.platform.warp_pointer(r.center());
                    }
                }
            }
        }
        for e in errors {
            self.warn(e);
        }
    }

    /// Empty, hidden, unconfigured workspaces are dropped so their number can
    /// be reused on whichever display wants it next.
    fn collect_empty_workspaces(&mut self) {
        let visible: BTreeSet<String> = self.world.active.values().cloned().collect();
        let pinned = &self.config.workspaces;
        self.world.workspaces.retain(|name, ws| {
            !ws.is_empty()
                || visible.contains(name)
                || name == SCRATCHPAD
                || pinned.display.contains_key(name)
                || pinned.layout.contains_key(name)
        });
    }
}
