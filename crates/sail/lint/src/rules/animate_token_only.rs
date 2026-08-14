//! `sail-animate-token-only` — Style motion configs
//! (`style({ animate: hover({ … }) })` / `animate: enter({ … })`) accept only the
//! closed property/preset vocabulary that maps cleanly to the design system.
//!
//! `animate: <trigger>({ <prop>: '<preset>' })` — the property must be one of
//! [`MOTION_PROPERTIES`] (the same closed set the parser enforces), and the
//! preset must be a name known to `animation-presets.json`.

use std::path::Path;

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sail-animate-token-only";

/// THE property vocabulary — one list, two readers. The compiler's parser imports
/// this rather than keeping its own copy.
pub const MOTION_PROPERTIES: &[&str] = &[
    "scale",
    "scale-x",
    "scale-y",
    "opacity",
    "translate-y",
    "translate-x",
    "rotate",
];

/// `stagger` is RESERVED, not a property: it describes the relationship BETWEEN
/// sibling elements (the delay step), not anything about any one of them.
pub const MOTION_STAGGER_KEY: &str = "stagger";

/// THE timing vocabulary — the config-level (and per-channel) keys that are not
/// animated properties. Same one-list-two-readers contract as the properties above.
///
/// These take INLINE VALUES. Style motion is the animation library's surface: a
/// duration is a number of milliseconds and an easing is a curve, not a global
/// design token. `sx` paint stays closed because colour and spacing are theming
/// integrity; a 3600ms decorative loop never was.
pub const MOTION_TIMING_KEYS: &[&str] = &["duration", "delay", "easing", "iterations", "direction"];

/// The keys legal inside a per-channel inline object:
/// `translate-x: { from: 0, to: '120px' }`.
pub const MOTION_CHANNEL_KEYS: &[&str] = &["from", "to", "preset"];

/// THE trigger vocabulary — what can drive Style motion
/// (`animate: <trigger>({…})`). One list, read by the parser, the runtime, and
/// the generated agent skill.
///
/// Three kinds, and the difference matters:
///   INTERACTION  hover, press, focus, focus-visible — rest <-> active, event-driven
///   LIFECYCLE    enter, exit — the element joining or leaving the DOM. `exit` is the
///                one thing CSS cannot do alone: it needs the framework to HOLD the
///                element in the tree until the animation finishes.
///   TIMELINE     reveal, scroll — PROGRESS-driven, not time-driven.
///   AMBIENT      loop — continuous: the baked animation alternates forever
///                (iterations: Infinity). Fully disabled under prefers-reduced-motion.
pub const MOTION_TRIGGERS: &[&str] = &[
    "hover",
    "press",
    "focus",
    "focus-visible",
    "enter",
    "exit",
    "reveal",
    "scroll",
    "loop",
];

const PRESETS_JSON: &str = include_str!("../../animation-presets.json");

fn known_presets() -> Vec<String> {
    let v: serde_json::Value = match serde_json::from_str(PRESETS_JSON) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    match v.get("presets").and_then(|p| p.as_object()) {
        Some(o) => o.keys().cloned().collect(),
        None => Vec::new(),
    }
}

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (line_no, line) in source.lines().enumerate() {
        if line.trim_start().starts_with("<!--") && line.contains("-->") {
            continue;
        }
        let bytes = line.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            let Some(method_start) = next_ident_start(line, i) else {
                break;
            };
            let Some(method_len) = scan_ident(&line[method_start..]) else {
                i = method_start + 1;
                continue;
            };
            let method = &line[method_start..method_start + method_len];
            if method_start > 0 && is_attr_name_continuation(bytes[method_start - 1]) {
                i = method_start + method_len;
                continue;
            }
            if !MOTION_TRIGGERS.contains(&method) {
                i = method_start + method_len;
                continue;
            }
            let mut k = method_start + method_len;
            while k < bytes.len() && bytes[k].is_ascii_whitespace() {
                k += 1;
            }
            if k >= bytes.len() || bytes[k] != b'(' {
                i = k.max(method_start + method_len + 1);
                continue;
            }
            k += 1;
            while k < bytes.len() && bytes[k].is_ascii_whitespace() {
                k += 1;
            }
            if k >= bytes.len() || bytes[k] != b'{' {
                // Dual paint/motion methods (`press()`, `scroll(auto)`) — skip.
                i = k.max(method_start + method_len + 1);
                continue;
            }
            let mut depth: i32 = 1;
            let body_start = k;
            let mut v = k + 1;
            while v < bytes.len() {
                match bytes[v] {
                    b'{' => depth += 1,
                    b'}' => {
                        depth -= 1;
                        if depth == 0 {
                            v += 1;
                            break;
                        }
                    }
                    b'"' | b'\'' => {
                        let q = bytes[v];
                        v += 1;
                        while v < bytes.len() {
                            if bytes[v] == b'\\' && v + 1 < bytes.len() {
                                v += 2;
                                continue;
                            }
                            if bytes[v] == q {
                                v += 1;
                                break;
                            }
                            v += 1;
                        }
                        continue;
                    }
                    _ => {}
                }
                v += 1;
            }
            let raw = &line[body_start..v.min(bytes.len())];
            validate_static_body(path, line_no + 1, method_start, raw, &mut findings);
            i = v;
        }
    }
    findings
}

fn is_attr_name_continuation(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b':' || c == b'-' || c == b'_'
}

fn next_ident_start(line: &str, from: usize) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut i = from;
    while i < bytes.len() {
        let c = bytes[i];
        if c.is_ascii_alphabetic() || c == b'_' {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn scan_ident(rest: &str) -> Option<usize> {
    let bytes = rest.as_bytes();
    if bytes.is_empty() {
        return None;
    }
    let first = bytes[0];
    if !(first.is_ascii_alphabetic() || first == b'_') {
        return None;
    }
    let mut n = 1;
    while n < bytes.len() {
        let c = bytes[n];
        if !(c.is_ascii_alphanumeric() || c == b'_' || c == b'-') {
            break;
        }
        n += 1;
    }
    Some(n)
}

fn validate_static_body(
    path: &Path,
    line: usize,
    _attr_offset: usize,
    body: &str,
    findings: &mut Vec<Finding>,
) {
    let s = body.trim();
    if !s.starts_with('{') || !s.ends_with('}') {
        return;
    }
    let inner = s[1..s.len() - 1].trim();
    if inner.is_empty() {
        return;
    }
    for pair in split_top_level_commas(inner) {
        let pair = pair.trim();
        let Some((key_raw, val_raw)) = pair.split_once(':') else {
            continue;
        };
        let key = key_raw.trim();
        if key == MOTION_STAGGER_KEY || MOTION_TIMING_KEYS.contains(&key) {
            continue;
        }
        if !MOTION_PROPERTIES.contains(&key) {
            findings.push(Finding {
                file: path.to_path_buf(),
                line,
                col: 1,
                rule: RULE_ID,
                severity: Severity::Error,
                message: format!(
                    "motion property `{}` is not supported. Supported properties: {}",
                    key,
                    MOTION_PROPERTIES.join(", ")
                ),
                suggest: Some(
                    "Pick a supported property, or use the primitive's variant prop instead."
                        .to_string(),
                ),
            });
            continue;
        }
        let val_trimmed = val_raw.trim();
        let preset_name = if val_trimmed.starts_with('{') {
            match inline_object_preset(val_trimmed) {
                Some(name) => name,
                None => continue,
            }
        } else if let Some(name) = quoted_literal(val_trimmed) {
            name
        } else {
            continue;
        };
        let presets = known_presets();
        if !presets.iter().any(|p| p == &preset_name) {
            findings.push(Finding {
                file: path.to_path_buf(),
                line,
                col: 1,
                rule: RULE_ID,
                severity: Severity::Error,
                message: format!(
                    "motion preset `{}` is not in the closed vocabulary. Known presets: {}",
                    preset_name,
                    presets.join(", ")
                ),
                suggest: Some(
                    "Use a registered preset, write the values inline \
                     (`{ from: 0, to: 1 }`), or extend animation-presets.json."
                        .to_string(),
                ),
            });
        }
    }
}

fn quoted_literal(s: &str) -> Option<String> {
    let b = s.as_bytes();
    let q = *b.first()?;
    if q != b'"' && q != b'\'' {
        return None;
    }
    if b.len() < 2 || *b.last()? != q {
        return None;
    }
    Some(s[1..s.len() - 1].to_string())
}

fn inline_object_preset(s: &str) -> Option<String> {
    let inner = s.strip_prefix('{')?.strip_suffix('}')?;
    for pair in split_top_level_commas(inner) {
        let Some((k, v)) = pair.split_once(':') else {
            continue;
        };
        if k.trim() == "preset" {
            return quoted_literal(v.trim());
        }
    }
    None
}

fn split_top_level_commas(s: &str) -> Vec<String> {
    let bytes = s.as_bytes();
    let mut out = Vec::new();
    let mut start = 0;
    let mut depth = 0i32;
    let mut in_str: Option<u8> = None;
    for i in 0..bytes.len() {
        let c = bytes[i];
        if let Some(q) = in_str {
            if c == b'\\' && i + 1 < bytes.len() {
                continue;
            }
            if c == q {
                in_str = None;
            }
            continue;
        }
        match c {
            b'"' | b'\'' => in_str = Some(c),
            b'{' | b'[' | b'(' => depth += 1,
            b'}' | b']' | b')' => depth -= 1,
            b',' if depth == 0 => {
                out.push(s[start..i].to_string());
                start = i + 1;
            }
            _ => {}
        }
    }
    out.push(s[start..].to_string());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn findings(src: &str) -> Vec<Finding> {
        check(Path::new("t.sail"), src)
    }

    #[test]
    fn accepts_static_hover_with_known_preset_and_prop() {
        let f =
            findings(r#"<button style({ animate: hover({ scale: 'spring:snappy' }) })>x</button>"#);
        assert!(f.is_empty(), "expected no findings, got {f:?}");
    }

    #[test]
    fn flags_unknown_property() {
        let f =
            findings(r#"<button style({ animate: hover({ foo: 'spring:snappy' }) })>x</button>"#);
        assert_eq!(f.len(), 1, "expected 1 finding, got {f:?}");
        assert!(f[0].message.contains("foo"));
        assert!(f[0].message.contains("scale"));
    }

    #[test]
    fn flags_unknown_preset() {
        let f = findings(r#"<button style({ animate: hover({ scale: 'mystery' }) })>x</button>"#);
        assert_eq!(f.len(), 1, "expected 1 finding, got {f:?}");
        assert!(f[0].message.contains("mystery"));
        assert!(f[0].message.contains("spring:snappy"));
    }

    #[test]
    fn flags_multiple_in_one_config() {
        let f = findings(
            r#"<button style({ animate: hover({ scale: 'mystery', opacity: 'also-bad' }) })>x</button>"#,
        );
        assert_eq!(f.len(), 2, "expected 2 findings, got {f:?}");
    }

    #[test]
    fn ignores_paint_press_without_config_object() {
        let f = findings(r#"<button style({ press().bg(primary) })>x</button>"#);
        assert!(
            f.is_empty(),
            "expected no findings for paint press(), got {f:?}"
        );
    }

    #[test]
    fn ignores_unrelated_motion_substrings() {
        let f = findings(r#"<button data-motion:foo="bar">x</button>"#);
        assert!(f.is_empty(), "expected no findings, got {f:?}");
    }

    #[test]
    fn skips_lines_that_are_pure_comment() {
        let f =
            findings("<!-- style({ animate: hover({ foo: 'mystery' }) }) -->\n<button>x</button>");
        assert!(f.is_empty(), "expected no findings, got {f:?}");
    }

    #[test]
    fn handles_utf8_source() {
        let f = findings(
            "<!-- Loading… -->\n<button style({ animate: hover({ foo: 'mystery' }) })>x</button>",
        );
        assert_eq!(f.len(), 1, "expected 1 finding, got {f:?}");
        assert!(f[0].message.contains("`foo` is not supported"), "got {f:?}");
    }

    #[test]
    fn animate_enter_known_preset_passes() {
        let f = findings(r#"<button style({ animate: enter({ opacity: 'fade' }) })>x</button>"#);
        assert!(f.is_empty(), "expected no findings, got {f:?}");
    }
}
