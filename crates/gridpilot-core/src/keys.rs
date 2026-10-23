//! Platform-neutral key chords such as `mod+shift+left`.
//!
//! Backends translate [`Key`] into their own key codes; the core only deals
//! with names. Modifier aliases (`mod`, `grid`, `hyper`, ...) are expanded from
//! the config so the whole keymap can be moved to another modifier by editing
//! one line, the way Omarchy's SUPER bindings map onto GridPilot's `mod`.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub struct Modifiers(u8);

impl Modifiers {
    pub const NONE: Modifiers = Modifiers(0);
    pub const SHIFT: Modifiers = Modifiers(1);
    pub const CTRL: Modifiers = Modifiers(1 << 1);
    /// Option on macOS, Alt elsewhere.
    pub const ALT: Modifiers = Modifiers(1 << 2);
    /// Command on macOS, the Windows/Super key elsewhere.
    pub const CMD: Modifiers = Modifiers(1 << 3);

    pub fn contains(self, other: Modifiers) -> bool {
        self.0 & other.0 == other.0
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub fn bits(self) -> u8 {
        self.0
    }

    fn parse_one(name: &str) -> Option<Modifiers> {
        Some(match name {
            "shift" => Modifiers::SHIFT,
            "ctrl" | "control" => Modifiers::CTRL,
            "alt" | "opt" | "option" => Modifiers::ALT,
            "cmd" | "command" | "super" | "win" | "meta" => Modifiers::CMD,
            _ => return None,
        })
    }
}

impl std::ops::BitOr for Modifiers {
    type Output = Modifiers;
    fn bitor(self, rhs: Self) -> Self {
        Modifiers(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for Modifiers {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl fmt::Display for Modifiers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();
        for (m, n) in [(Self::CTRL, "ctrl"), (Self::ALT, "alt"), (Self::SHIFT, "shift"), (Self::CMD, "cmd")] {
            if self.contains(m) {
                parts.push(n);
            }
        }
        f.write_str(&parts.join("+"))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Key {
    /// `a`..`z`, stored lowercase.
    Char(char),
    /// `0`..`9` on the main row.
    Digit(u8),
    /// `f1`..`f20`.
    F(u8),
    Left,
    Right,
    Up,
    Down,
    Return,
    Tab,
    Space,
    Escape,
    Backspace,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,
    Grave,
    Minus,
    Equal,
    LeftBracket,
    RightBracket,
    Backslash,
    Semicolon,
    Quote,
    Comma,
    Period,
    Slash,
}

impl Key {
    pub fn parse(name: &str) -> Option<Key> {
        let n = name.to_ascii_lowercase();
        if n.chars().count() == 1 {
            let c = n.chars().next()?;
            if c.is_ascii_lowercase() {
                return Some(Key::Char(c));
            }
            if let Some(d) = c.to_digit(10) {
                return Some(Key::Digit(d as u8));
            }
        }
        if let Some(num) = n.strip_prefix('f') {
            if let Ok(v) = num.parse::<u8>() {
                if (1..=20).contains(&v) {
                    return Some(Key::F(v));
                }
            }
        }
        Some(match n.as_str() {
            "left" => Key::Left,
            "right" => Key::Right,
            "up" => Key::Up,
            "down" => Key::Down,
            "return" | "enter" => Key::Return,
            "tab" => Key::Tab,
            "space" => Key::Space,
            "escape" | "esc" => Key::Escape,
            "backspace" => Key::Backspace,
            "delete" | "del" => Key::Delete,
            "home" => Key::Home,
            "end" => Key::End,
            "pageup" => Key::PageUp,
            "pagedown" => Key::PageDown,
            "grave" | "backtick" | "`" => Key::Grave,
            "minus" | "-" => Key::Minus,
            "equal" | "=" | "plus" => Key::Equal,
            "leftbracket" | "[" => Key::LeftBracket,
            "rightbracket" | "]" => Key::RightBracket,
            "backslash" | "\\" => Key::Backslash,
            "semicolon" | ";" => Key::Semicolon,
            "quote" | "'" => Key::Quote,
            "comma" | "," => Key::Comma,
            "period" | "." => Key::Period,
            "slash" | "/" => Key::Slash,
            _ => return None,
        })
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Key::Char(c) => write!(f, "{c}"),
            Key::Digit(d) => write!(f, "{d}"),
            Key::F(n) => write!(f, "f{n}"),
            other => {
                let s = format!("{other:?}").to_ascii_lowercase();
                f.write_str(&s)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Chord {
    pub mods: Modifiers,
    pub key: Key,
}

impl Chord {
    /// Parse `mod+shift+left`, expanding `aliases` (e.g. `mod = "ctrl+alt"`).
    pub fn parse(spec: &str, aliases: &BTreeMap<String, String>) -> Result<Chord, String> {
        let mut mods = Modifiers::NONE;
        let mut key = None;
        let parts: Vec<String> = spec.split('+').map(|p| p.trim().to_ascii_lowercase()).collect();
        if parts.iter().any(|p| p.is_empty()) {
            return Err(format!("`{spec}` has an empty key name"));
        }
        let last = parts.len() - 1;
        for (i, part) in parts.iter().enumerate() {
            if aliases.contains_key(part) {
                mods |= expand_alias(part, aliases, 0)?;
            } else if let Some(m) = Modifiers::parse_one(part) {
                mods |= m;
            } else if i == last {
                key = Some(Key::parse(part).ok_or_else(|| format!("unknown key `{part}` in `{spec}`"))?);
            } else {
                return Err(format!("unknown modifier `{part}` in `{spec}`"));
            }
        }
        let key = key.ok_or_else(|| format!("`{spec}` has no key, only modifiers"))?;
        Ok(Chord { mods, key })
    }
}

/// Modifiers an alias stands for. Aliases may name other aliases
/// (`mod = "hyper"`, `hyper = "ctrl+alt+cmd"`).
fn expand_alias(name: &str, aliases: &BTreeMap<String, String>, depth: u8) -> Result<Modifiers, String> {
    if depth > 8 {
        return Err(format!("alias `{name}` refers to itself"));
    }
    let expansion = aliases.get(name).ok_or_else(|| format!("unknown modifier `{name}`"))?;
    let mut mods = Modifiers::NONE;
    for m in expansion.split('+').map(|m| m.trim().to_ascii_lowercase()) {
        mods |= match Modifiers::parse_one(&m) {
            Some(x) => x,
            None if aliases.contains_key(&m) => expand_alias(&m, aliases, depth + 1)?,
            None => return Err(format!("alias `{name}` expands to non-modifier `{m}`")),
        };
    }
    Ok(mods)
}

impl fmt::Display for Chord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.mods.is_empty() {
            write!(f, "{}", self.key)
        } else {
            write!(f, "{}+{}", self.mods, self.key)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_aliases() {
        let mut aliases = BTreeMap::new();
        aliases.insert("mod".to_string(), "ctrl+alt".to_string());
        let c = Chord::parse("mod+Shift+Left", &aliases).unwrap();
        assert_eq!(c.mods, Modifiers::CTRL | Modifiers::ALT | Modifiers::SHIFT);
        assert_eq!(c.key, Key::Left);
        assert_eq!(c.to_string(), "ctrl+alt+shift+left");
    }

    #[test]
    fn aliases_can_refer_to_aliases() {
        let mut aliases = BTreeMap::new();
        aliases.insert("mod".to_string(), "hyper".to_string());
        aliases.insert("hyper".to_string(), "ctrl+alt+cmd".to_string());
        let c = Chord::parse("mod+shift+left", &aliases).unwrap();
        assert_eq!(c.mods, Modifiers::CTRL | Modifiers::ALT | Modifiers::CMD | Modifiers::SHIFT);
        aliases.insert("hyper".to_string(), "mod".to_string());
        assert!(Chord::parse("mod+a", &aliases).is_err(), "cycles are rejected");
    }

    #[test]
    fn rejects_garbage() {
        let a = BTreeMap::new();
        assert!(Chord::parse("ctrl+alt", &a).is_err());
        assert!(Chord::parse("hyperx+a", &a).is_err());
        assert!(Chord::parse("ctrl++a", &a).is_err());
        assert_eq!(Chord::parse("cmd+1", &a).unwrap().key, Key::Digit(1));
        assert_eq!(Chord::parse("f12", &a).unwrap().key, Key::F(12));
    }
}
