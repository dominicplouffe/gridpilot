//! Colour themes for everything GridPilot draws: the grid overlay, focus
//! borders, group tab bars and the key cheat sheet.
//!
//! Like Omarchy, a theme is one small file of named colours and every surface
//! reads from it, so switching theme (`theme <name>`) restyles all of them at
//! once. User themes live in `~/.config/gridpilot/themes/<name>.toml`.

use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub fn with_alpha(self, a: u8) -> Self {
        Self { a, ..self }
    }

    /// Components as 0..1 floats, the form most drawing APIs want.
    pub fn to_f64(self) -> [f64; 4] {
        [self.r as f64 / 255.0, self.g as f64 / 255.0, self.b as f64 / 255.0, self.a as f64 / 255.0]
    }
}

impl FromStr for Color {
    type Err = String;

    /// `#rrggbb` or `#rrggbbaa`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let hex = s.trim().trim_start_matches('#');
        let byte = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).map_err(|_| format!("bad colour `{s}`"));
        match hex.len() {
            6 => Ok(Color { r: byte(0)?, g: byte(2)?, b: byte(4)?, a: 255 }),
            8 => Ok(Color { r: byte(0)?, g: byte(2)?, b: byte(4)?, a: byte(6)? }),
            _ => Err(format!("colour `{s}` must be #rrggbb or #rrggbbaa")),
        }
    }
}

impl TryFrom<String> for Color {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.parse()
    }
}

impl From<Color> for String {
    fn from(c: Color) -> String {
        if c.a == 255 {
            format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b)
        } else {
            format!("#{:02x}{:02x}{:02x}{:02x}", c.r, c.g, c.b, c.a)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    /// Overlay backdrop.
    pub background: Color,
    /// Grid panel and cheat sheet cards.
    pub surface: Color,
    /// Grid lines and inactive borders.
    pub line: Color,
    /// The selection and the active border. GridPilot's Beacon amber.
    pub accent: Color,
    /// Second stop of the active-border gradient and hover state.
    pub accent_alt: Color,
    pub text: Color,
    pub muted: Color,
    pub danger: Color,
    #[serde(default = "default_border")]
    pub border_width: f64,
    #[serde(default = "default_radius")]
    pub corner_radius: f64,
}

fn default_border() -> f64 {
    2.0
}
fn default_radius() -> f64 {
    10.0
}

fn hex(s: &str) -> Color {
    s.parse().expect("built-in colours are valid")
}

impl Theme {
    #[allow(clippy::too_many_arguments)]
    fn make(
        name: &str,
        bg: &str,
        surface: &str,
        line: &str,
        accent: &str,
        alt: &str,
        text: &str,
        muted: &str,
        danger: &str,
    ) -> Theme {
        Theme {
            name: name.into(),
            background: hex(bg),
            surface: hex(surface),
            line: hex(line),
            accent: hex(accent),
            accent_alt: hex(alt),
            text: hex(text),
            muted: hex(muted),
            danger: hex(danger),
            border_width: default_border(),
            corner_radius: default_radius(),
        }
    }

    /// The GridPilot brand palette, see docs/BRAND.md.
    pub fn cockpit() -> Theme {
        Theme::make("cockpit", "#0b1120d9", "#141c2f", "#26324d", "#ffb020", "#3ec6ff", "#e7ecf5", "#8893ab", "#f2545b")
    }

    pub fn daylight() -> Theme {
        Theme::make(
            "daylight",
            "#f5f7fbd9",
            "#ffffff",
            "#d5dbe7",
            "#c97a00",
            "#0284c7",
            "#0b1120",
            "#5a6478",
            "#d62f3a",
        )
    }

    pub fn builtins() -> Vec<Theme> {
        vec![
            Theme::cockpit(),
            Theme::daylight(),
            Theme::make(
                "tokyo-night",
                "#1a1b26d9",
                "#24283b",
                "#414868",
                "#7aa2f7",
                "#bb9af7",
                "#c0caf5",
                "#565f89",
                "#f7768e",
            ),
            Theme::make(
                "catppuccin",
                "#1e1e2ed9",
                "#313244",
                "#45475a",
                "#f5c2e7",
                "#89b4fa",
                "#cdd6f4",
                "#7f849c",
                "#f38ba8",
            ),
            Theme::make(
                "gruvbox",
                "#282828d9",
                "#3c3836",
                "#504945",
                "#fabd2f",
                "#83a598",
                "#ebdbb2",
                "#928374",
                "#fb4934",
            ),
            Theme::make(
                "nord",
                "#2e3440d9",
                "#3b4252",
                "#4c566a",
                "#88c0d0",
                "#81a1c1",
                "#eceff4",
                "#7b88a1",
                "#bf616a",
            ),
        ]
    }

    pub fn builtin(name: &str) -> Option<Theme> {
        Theme::builtins().into_iter().find(|t| t.name == name)
    }
}

impl Default for Theme {
    fn default() -> Self {
        Theme::cockpit()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colour_round_trip() {
        let c: Color = "#ffb020".parse().unwrap();
        assert_eq!(c, Color::rgb(0xff, 0xb0, 0x20));
        assert_eq!(String::from(c.with_alpha(0x80)), "#ffb02080");
        assert!("#fff".parse::<Color>().is_err());
    }

    #[test]
    fn theme_from_toml() {
        let t: Theme = toml::from_str(
            r##"
            name = "mine"
            background = "#000000cc"
            surface = "#111111"
            line = "#222222"
            accent = "#ff00ff"
            accent_alt = "#00ffff"
            text = "#ffffff"
            muted = "#999999"
            danger = "#ff0000"
            "##,
        )
        .unwrap();
        assert_eq!(t.border_width, 2.0);
        assert_eq!(t.background.a, 0xcc);
    }
}
