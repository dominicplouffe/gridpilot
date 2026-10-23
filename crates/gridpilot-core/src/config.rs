//! `~/.config/gridpilot/gridpilot.toml`.
//!
//! Every section has defaults, so an empty file is a valid config. The
//! annotated reference config lives at `config/gridpilot.toml` in the repo and
//! is what `gridpilot init` writes out.

use crate::geometry::Insets;
use crate::grid::{Grid, GridSelection, Margins};
use crate::layout::LayoutOptions;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const REFERENCE_CONFIG: &str = include_str!("../../../config/gridpilot.toml");

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub general: General,
    /// Modifier aliases usable in any chord, e.g. `mod = "ctrl+alt"`.
    pub modifiers: BTreeMap<String, String>,
    pub tiling: Tiling,
    pub grid: GridConfig,
    pub workspaces: Workspaces,
    pub theme: ThemeConfig,
    pub features: Features,
    /// Chord -> command. Overrides feature defaults; `"none"` unbinds.
    pub bindings: BTreeMap<String, String>,
    #[serde(rename = "rule")]
    pub rules: Vec<Rule>,
}

impl Default for Config {
    fn default() -> Self {
        let mut modifiers = BTreeMap::new();
        // Omarchy's SUPER. Cmd is left alone because macOS apps own it.
        modifiers.insert("mod".into(), "ctrl+alt".into());
        // Divvy-style placement shortcuts.
        modifiers.insert("grid".into(), "ctrl+cmd".into());
        modifiers.insert("hyper".into(), "ctrl+alt+shift+cmd".into());
        Self {
            general: General::default(),
            modifiers,
            tiling: Tiling::default(),
            grid: GridConfig::default(),
            workspaces: Workspaces::default(),
            theme: ThemeConfig::default(),
            features: Features::default(),
            bindings: BTreeMap::new(),
            rules: Vec::new(),
        }
    }
}

impl Config {
    pub fn parse(text: &str) -> Result<Config, String> {
        toml::from_str(text).map_err(|e| e.to_string())
    }

    pub fn load(path: &Path) -> Result<Config, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        Config::parse(&text).map_err(|e| format!("{}: {e}", path.display()))
    }

    /// `$GRIDPILOT_CONFIG`, else `$XDG_CONFIG_HOME/gridpilot/gridpilot.toml`,
    /// else `~/.config/gridpilot/gridpilot.toml`.
    pub fn default_path() -> PathBuf {
        if let Some(p) = std::env::var_os("GRIDPILOT_CONFIG") {
            return PathBuf::from(p);
        }
        config_dir().join("gridpilot.toml")
    }

    pub fn layout_options(&self) -> LayoutOptions {
        LayoutOptions {
            gaps_inner: self.tiling.gaps_inner,
            gaps_outer: self.tiling.gaps_outer,
            master_ratio: self.tiling.master_ratio,
            column_width: self.tiling.column_width,
        }
    }

    pub fn grid_margins(&self) -> Margins {
        Margins { screen: Insets::uniform(self.grid.screen_margin), window: self.grid.window_margin }
    }
}

pub fn config_dir() -> PathBuf {
    if let Some(x) = std::env::var_os("XDG_CONFIG_HOME") {
        return PathBuf::from(x).join("gridpilot");
    }
    let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
    home.join(".config").join("gridpilot")
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct General {
    /// Load the key bindings each feature ships with.
    pub default_bindings: bool,
    /// Milliseconds between polls of the window list, for backends without events.
    pub poll_interval_ms: u64,
    /// Move the mouse pointer to the centre of a window focused by keyboard.
    pub mouse_follows_focus: bool,
}

impl Default for General {
    fn default() -> Self {
        Self { default_bindings: true, poll_interval_ms: 250, mouse_follows_focus: false }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Tiling {
    /// Layout of new workspaces. `floating` gives a pure Divvy experience;
    /// `dwindle` gives Omarchy/Hyprland auto-tiling.
    pub default_layout: String,
    /// Order used by `layout next` / `layout prev` (Omarchy `SUPER + L`).
    pub cycle: Vec<String>,
    pub gaps_inner: f64,
    pub gaps_outer: f64,
    pub master_ratio: f64,
    pub column_width: f64,
    /// Size used by `pop` (float, pin and centre a window).
    pub pop_size: [f64; 2],
    /// Pixel steps for `resize` bindings: small, normal, large.
    pub resize_steps: [f64; 3],
}

impl Default for Tiling {
    fn default() -> Self {
        Self {
            default_layout: "floating".into(),
            cycle: vec!["dwindle".into(), "scrolling".into(), "master".into(), "floating".into()],
            gaps_inner: 10.0,
            gaps_outer: 10.0,
            master_ratio: 0.55,
            column_width: 0.49,
            pop_size: [1300.0, 900.0],
            resize_steps: [25.0, 100.0, 300.0],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GridConfig {
    #[serde(with = "as_string")]
    pub size: Grid,
    pub screen_margin: f64,
    pub window_margin: f64,
    /// Pressing the same shortcut again moves the window to the next display.
    pub cycle_displays: bool,
    /// Keep the overlay open after applying a selection.
    pub keep_open: bool,
    /// Applying a grid selection to a tiled window floats it.
    pub float_on_apply: bool,
    #[serde(rename = "shortcut")]
    pub shortcuts: Vec<GridShortcut>,
}

impl Default for GridConfig {
    fn default() -> Self {
        let s = |name: &str, keys: &str, cells: &str| GridShortcut {
            name: name.into(),
            keys: Some(keys.into()),
            cells: cells.parse().expect("built-in shortcut"),
            global: true,
        };
        Self {
            size: Grid::new(6, 4),
            screen_margin: 10.0,
            window_margin: 10.0,
            cycle_displays: true,
            keep_open: false,
            float_on_apply: true,
            shortcuts: vec![
                s("Left half", "grid+left", "0,0 3x4 / 6x4"),
                s("Right half", "grid+right", "3,0 3x4 / 6x4"),
                s("Top half", "grid+up", "0,0 6x2 / 6x4"),
                s("Bottom half", "grid+down", "0,2 6x2 / 6x4"),
                s("Maximize", "grid+return", "0,0 6x4 / 6x4"),
                s("Center", "grid+c", "1,0 4x4 / 6x4"),
                s("Left third", "grid+1", "0,0 2x4 / 6x4"),
                s("Middle third", "grid+2", "2,0 2x4 / 6x4"),
                s("Right third", "grid+3", "4,0 2x4 / 6x4"),
                s("Left two thirds", "grid+4", "0,0 4x4 / 6x4"),
                s("Right two thirds", "grid+5", "2,0 4x4 / 6x4"),
                s("Top left", "grid+u", "0,0 3x2 / 6x4"),
                s("Top right", "grid+i", "3,0 3x2 / 6x4"),
                s("Bottom left", "grid+j", "0,2 3x2 / 6x4"),
                s("Bottom right", "grid+k", "3,2 3x2 / 6x4"),
            ],
        }
    }
}

/// A named grid selection with an optional key, Divvy's "shortcut".
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GridShortcut {
    pub name: String,
    #[serde(default)]
    pub keys: Option<String>,
    #[serde(with = "as_string")]
    pub cells: GridSelection,
    /// Global shortcuts work anywhere; local ones only while the overlay is open.
    #[serde(default = "yes")]
    pub global: bool,
}

fn yes() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Workspaces {
    /// Workspaces `1..=count` always exist (Omarchy has 10).
    pub count: u8,
    /// Pin a workspace to a display by index: `{ "1" = 0, "6" = 1 }`.
    pub display: BTreeMap<String, usize>,
    /// Per-workspace layout overrides: `{ "3" = "scrolling" }`.
    pub layout: BTreeMap<String, String>,
}

impl Default for Workspaces {
    fn default() -> Self {
        Self { count: 10, display: BTreeMap::new(), layout: BTreeMap::new() }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ThemeConfig {
    pub name: String,
    /// Draw a coloured border around the focused window.
    pub focus_border: bool,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self { name: "cockpit".into(), focus_border: true }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Features {
    /// Feature ids to leave out, e.g. `["groups", "scrolling"]`.
    pub disabled: Vec<String>,
}

/// Hyprland-style window rule. All given matchers must match.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Rule {
    /// Exact app id (bundle id on macOS) or executable name.
    pub app: Option<String>,
    /// Case-insensitive substring of the title.
    pub title: Option<String>,
    pub float: Option<bool>,
    pub workspace: Option<String>,
    /// Grid selection to place a floating window on when it opens.
    pub grid: Option<String>,
    /// Ignore the window entirely.
    pub ignore: bool,
}

impl Rule {
    pub fn matches(&self, app: &str, title: &str) -> bool {
        let app_ok = self.app.as_deref().is_none_or(|a| a.eq_ignore_ascii_case(app));
        let title_ok = self.title.as_deref().is_none_or(|t| title.to_lowercase().contains(&t.to_lowercase()));
        (self.app.is_some() || self.title.is_some()) && app_ok && title_ok
    }
}

mod as_string {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::fmt::Display;
    use std::str::FromStr;

    pub fn serialize<T: Display, S: Serializer>(v: &T, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(v)
    }

    pub fn deserialize<'de, T, D>(d: D) -> Result<T, D::Error>
    where
        T: FromStr<Err = String>,
        D: Deserializer<'de>,
    {
        let s = String::deserialize(d)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_config_is_default() {
        assert_eq!(Config::parse("").unwrap(), Config::default());
    }

    #[test]
    fn reference_config_parses() {
        let c = Config::parse(REFERENCE_CONFIG).expect("config/gridpilot.toml must stay valid");
        assert!(!c.grid.shortcuts.is_empty());
    }

    #[test]
    fn unknown_keys_are_errors() {
        assert!(Config::parse("[grid]\nsizee = \"6x4\"").is_err());
    }

    #[test]
    fn rules_match() {
        let r = Rule { app: Some("com.apple.finder".into()), title: Some("info".into()), ..Default::default() };
        assert!(r.matches("com.apple.Finder", "Get Info"));
        assert!(!r.matches("com.apple.Finder", "Downloads"));
        assert!(!Rule::default().matches("x", "y"));
    }
}
