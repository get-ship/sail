//! `sail-animate-token-only` — the `motion:` animation directive accepts
//! only the closed property/preset vocabulary that maps cleanly to the
//! design system.
//!
//! `motion:<trigger>={{ <prop>: '<preset>' }}` — the property must be one
//! of [`MOTION_PROPERTIES`] (the same closed set the parser enforces),
//! and the preset must be a name known to `animation-presets.json`.

use std::path::Path;

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sail-animate-token-only";

/// THE property vocabulary — one list, two readers. The compiler's parser imports
/// this rather than keeping its own copy, because it kept its own copy and the two
/// drifted the first time a key was added (`stagger` compiled fine, then failed the
/// lint). A comment claiming "the same closed set the parser enforces" is not a
/// mechanism; this is.
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

/// THE trigger vocabulary — what can drive a `motion:` directive. One list, read by
/// the parser (which rejects anything off it), by the runtime (which dispatches on
/// it), and by the generated agent skill (which teaches it).
///
/// Three kinds, and the difference matters:
///   INTERACTION  hover, press, focus, focus-visible — rest <-> active, event-driven
///   LIFECYCLE    enter, exit — the element joining or leaving the DOM. `exit` is the
///                one thing CSS cannot do alone: it needs the framework to HOLD the
///                element in the tree until the animation finishes.
///   TIMELINE     view, scroll — PROGRESS-driven, not time-driven. Reveal-on-scroll
///                and parallax, at zero JS runtime cost (the platform drives them off
///                the main thread — no listener, no rAF, no IntersectionObserver).
///   AMBIENT      loop — continuous: the baked animation alternates forever
///                (iterations: Infinity). The idle life of a page — a bouncing
///                specimen, a drifting card, a pulsing dot. Fully disabled
///                under prefers-reduced-motion.
pub const MOTION_TRIGGERS: &[&str] = &[
    "hover",
    "press",
    "focus",
    "focus-visible",
    "enter",
    "exit",
    "view",
    "scroll",
    "loop",
];

/// The preset catalog is `animation-presets.json` — the SAME file the compiler
/// bakes from. It used to be hand-copied into a `KNOWN_PRESETS` array here: a second
/// source of truth for a vocabulary that must not drift. A preset the compiler knows
/// and the lint does not is a build that passes one gate and fails the other for no
/// reason the author can see.
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
    // Walk by line so the byte index we hand to `find("motion:")`
    // is always on a char boundary — no multi-byte trap.
    for (line_no, line) in source.lines().enumerate() {
        // Skip lines that are entirely inside an HTML comment. This
        // is a coarse check; the precise comment-skip lives in the
        // compiler's pre-pass, not here.
        if line.trim_start().starts_with("<!--") && line.contains("-->") {
            continue;
        }
        let bytes = line.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            // Find next `motion:` in this line.
            let rest = &line[i..];
            let Some(colon_rel) = rest.find("motion:") else {
                break;
            };
            let attr_start = i + colon_rel;
            if attr_start > i && is_attr_name_continuation(bytes[attr_start - 1]) {
                i = attr_start + "motion:".len();
                continue;
            }
            let after_prefix = attr_start + "motion:".len();
            let Some(trigger_len) = scan_ident(&line[after_prefix..]) else {
                i = attr_start + "motion:".len();
                continue;
            };
            let trigger_end = after_prefix + trigger_len;
            let mut k = trigger_end;
            while k < bytes.len() && bytes[k].is_ascii_whitespace() {
                k += 1;
            }
            if k >= bytes.len() || bytes[k] != b'=' {
                i = k.max(trigger_end + 1);
                continue;
            }
            k += 1;
            while k < bytes.len() && bytes[k].is_ascii_whitespace() {
                k += 1;
            }
            if k >= bytes.len() {
                break;
            }
            // String form `motion:trigger="..."`.
            if bytes[k] == b'"' {
                let value_start = k + 1;
                let mut v = value_start;
                while v < bytes.len() {
                    match bytes[v] {
                        b'\\' if v + 1 < bytes.len() => v += 2,
                        b'"' => break,
                        _ => v += 1,
                    }
                }
                let value = &line[value_start..v.min(bytes.len())];
                validate_static_body(path, line_no + 1, attr_start, value, &mut findings);
                i = v + 1;
                continue;
            }
            // Brace-attr form `motion:trigger={{...}}`. The compiler
            // pre-pass leaves the literal `{{ ... }}` sequence in the
            // source. Strip the outer pair.
            if bytes[k] == b'{' {
                let mut depth: i32 = if k + 1 < bytes.len() && bytes[k + 1] == b'{'
                {
                    2
                } else {
                    1
                };
                let mut v = if depth == 2 { k + 2 } else { k + 1 };
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
                        _ => {}
                    }
                    v += 1;
                }
                let raw = &line[k..v.min(bytes.len())];
                let trimmed = strip_one_outer_brace_pair(raw);
                validate_static_body(
                    path,
                    line_no + 1,
                    attr_start,
                    trimmed,
                    &mut findings,
                );
                i = v;
                continue;
            }
            // Dynamic form `motion:hover={expr}` — skip silently.
            i = k + 1;
        }
    }
    findings
}

fn strip_one_outer_brace_pair(s: &str) -> &str {
    let s = s.trim();
    let s = s.strip_prefix('{').unwrap_or(s);
    let s = s.strip_suffix('}').unwrap_or(s);
    s.trim()
}

fn is_attr_name_continuation(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b':' || c == b'-' || c == b'_'
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
        if key == MOTION_STAGGER_KEY {
            continue; // a duration, not a preset — `stagger: '40ms'`
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
        }
        let val = val_raw.trim().trim_matches('\'').trim_matches('"');
        let presets = known_presets();
        if !presets.iter().any(|p| p == val) {
            findings.push(Finding {
                file: path.to_path_buf(),
                line,
                col: 1,
                rule: RULE_ID,
                severity: Severity::Error,
                message: format!(
                    "motion preset `{}` is not in the closed vocabulary. Known presets: {}",
                    val,
                    presets.join(", ")
                ),
                suggest: Some(
                    "Use a registered preset (spring:snappy, fade, …) or extend animation-presets.json."
                        .to_string(),
                ),
            });
        }
    }
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
        let f = findings(r#"<button motion:hover={{ scale: 'spring:snappy' }}>x</button>"#);
        assert!(f.is_empty(), "expected no findings, got {f:?}");
    }

    #[test]
    fn flags_unknown_property() {
        let f = findings(r#"<button motion:hover={{ foo: 'spring:snappy' }}>x</button>"#);
        assert_eq!(f.len(), 1, "expected 1 finding, got {f:?}");
        assert!(f[0].message.contains("foo"));
        assert!(f[0].message.contains("scale"));
    }

    #[test]
    fn flags_unknown_preset() {
        let f = findings(r#"<button motion:hover={{ scale: 'mystery' }}>x</button>"#);
        assert_eq!(f.len(), 1, "expected 1 finding, got {f:?}");
        assert!(f[0].message.contains("mystery"));
        assert!(f[0].message.contains("spring:snappy"));
    }

    #[test]
    fn flags_multiple_in_one_config() {
        let f = findings(
            r#"<button motion:hover={{ scale: 'mystery', opacity: 'also-bad' }}>x</button>"#,
        );
        assert_eq!(f.len(), 2, "expected 2 findings, got {f:?}");
    }

    #[test]
    fn ignores_dynamic_form() {
        let f = findings(r#"<button motion:hover={someExpr}>x</button>"#);
        assert!(f.is_empty(), "expected no findings for dynamic form, got {f:?}");
    }

    #[test]
    fn ignores_unrelated_motion_substrings() {
        let f = findings(r#"<button data-motion:foo="bar">x</button>"#);
        assert!(f.is_empty(), "expected no findings, got {f:?}");
    }

    #[test]
    fn skips_lines_that_are_pure_comment() {
        let f =
            findings("<!-- motion:hover={{ foo: 'mystery' }} -->\n<button>x</button>");
        assert!(f.is_empty(), "expected no findings, got {f:?}");
    }

    #[test]
    fn handles_utf8_source() {
        // Regression: source with a multi-byte char (`…` is 3 bytes)
        // would have panicked the rule on a byte-vs-char boundary.
        let f = findings(
            "<!-- Loading… -->\n<button motion:hover={{ foo: 'mystery' }}>x</button>",
        );
        assert_eq!(f.len(), 2, "expected 2 findings, got {f:?}");
    }
}
