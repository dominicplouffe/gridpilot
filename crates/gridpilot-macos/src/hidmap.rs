//! Reading and editing the session key mapping that macOS's built-in
//! `hidutil` manages (`hidutil property --get/--set UserKeyMapping`).
//!
//! GridPilot adds one entry, Caps Lock → F18, and removes it again, leaving
//! any other mapping (e.g. one made in System Settings → Keyboard → Modifier
//! Keys) untouched.

/// HID usage of Caps Lock (page 7, usage 0x39).
pub const CAPS_LOCK: u64 = 0x7_0000_0039;
/// HID usage of F18 (page 7, usage 0x6D), a key no app uses.
pub const F18: u64 = 0x7_0000_006D;

/// (source, destination) pairs.
pub type Mapping = Vec<(u64, u64)>;

/// Parse `hidutil property --get UserKeyMapping` output, e.g.
///
/// ```text
/// (
///         {
///         HIDKeyboardModifierMappingDst = 30064771181;
///         HIDKeyboardModifierMappingSrc = 30064771129;
///     }
/// )
/// ```
///
/// or `(null)` when there is none.
pub fn parse(text: &str) -> Mapping {
    let mut out = Vec::new();
    for block in text.split('{').skip(1) {
        let block = block.split('}').next().unwrap_or("");
        let field = |name: &str| -> Option<u64> {
            let at = block.find(name)? + name.len();
            let rest = block[at..].trim_start().strip_prefix('=')?.trim_start();
            let digits: String = rest.chars().take_while(|c| c.is_ascii_hexdigit() || *c == 'x').collect();
            match digits.strip_prefix("0x") {
                Some(hex) => u64::from_str_radix(hex, 16).ok(),
                None => digits.parse().ok(),
            }
        };
        if let (Some(src), Some(dst)) = (field("HIDKeyboardModifierMappingSrc"), field("HIDKeyboardModifierMappingDst"))
        {
            out.push((src, dst));
        }
    }
    out
}

/// `existing` with Caps Lock sending F18 (replacing any other Caps Lock entry).
pub fn with_capslock(existing: &[(u64, u64)]) -> Mapping {
    let mut m: Mapping = existing.iter().copied().filter(|(src, _)| *src != CAPS_LOCK).collect();
    m.push((CAPS_LOCK, F18));
    m
}

/// `existing` without GridPilot's entry.
pub fn without_capslock(existing: &[(u64, u64)]) -> Mapping {
    existing.iter().copied().filter(|pair| *pair != (CAPS_LOCK, F18)).collect()
}

/// The argument for `hidutil property --set`.
pub fn to_json(m: &[(u64, u64)]) -> String {
    let items: Vec<String> = m
        .iter()
        .map(|(src, dst)| {
            format!("{{\"HIDKeyboardModifierMappingSrc\":{src},\"HIDKeyboardModifierMappingDst\":{dst}}}")
        })
        .collect();
    format!("{{\"UserKeyMapping\":[{}]}}", items.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "(\n        {\n        HIDKeyboardModifierMappingDst = 30064771300;\n        HIDKeyboardModifierMappingSrc = 30064771129;\n    },\n        {\n        HIDKeyboardModifierMappingDst = 30064771298;\n        HIDKeyboardModifierMappingSrc = 30064771299;\n    }\n)\n";

    #[test]
    fn parses_hidutil_output() {
        assert_eq!(parse(SAMPLE), vec![(CAPS_LOCK, 30064771300), (30064771299, 30064771298)]);
        assert!(parse("(null)\n").is_empty());
        assert_eq!(
            parse("({HIDKeyboardModifierMappingSrc = 0x700000039; HIDKeyboardModifierMappingDst = 0x70000006D;})"),
            vec![(CAPS_LOCK, F18)]
        );
    }

    #[test]
    fn adds_and_removes_only_our_entry() {
        let user = parse(SAMPLE);
        let ours = with_capslock(&user);
        // The user's own Caps Lock remap (to Control) is replaced while we run...
        assert_eq!(ours, vec![(30064771299, 30064771298), (CAPS_LOCK, F18)]);
        // ...and the other one is kept.
        assert_eq!(without_capslock(&ours), vec![(30064771299, 30064771298)]);
        assert_eq!(with_capslock(&ours), ours, "idempotent");
    }

    #[test]
    fn writes_hidutil_json() {
        assert_eq!(
            to_json(&[(CAPS_LOCK, F18)]),
            "{\"UserKeyMapping\":[{\"HIDKeyboardModifierMappingSrc\":30064771129,\"HIDKeyboardModifierMappingDst\":30064771181}]}"
        );
        assert_eq!(to_json(&[]), "{\"UserKeyMapping\":[]}");
    }
}
