//! `sail-surface-not-card` — anti-smell #4 from `ship-design-system`.
//!
//! "Never reach for `var(--card)` when `var(--surface-1)` captures intent.
//!  Cards consume the surface layer so a theme that raises card depth (a
//!  gradient background, a tinted nested panel) only touches `--surface-1`
//!  / `--surface-2`."
//!
//! Cards are a layout concept; surfaces are the depth layer. A theme that
//! wants a warmer card, a tinted nested panel, or a gradient body surface
//! works through `--surface-1` / `--surface-2`. Components that read
//! `var(--card)` directly bake themselves against one palette slot and
//! stay frozen when a theme bends depth.
//!
//! What this rule flags:
//! - `var(--card)` (bare) used as a *value* in a color-bearing
//!   declaration.
//!
//! What it deliberately does NOT flag:
//! - `var(--card-foreground)` — the paired text-on-card token, a separate
//!   slot, not the surface.
//! - `var(--card-texture)` — the texture channel (a `background` value
//!   like `none` or a data-URI), not the surface color either.
//! - `--card: …;` declarations — themes defining the slot.
//! - `--card-foreground: …;` / `--card-texture: …;` declarations — same.
//! - `var(--card-…)` for any other modifier (`--card-edge`,
//!   `--card-shadow`, …) — the rule fires only on the bare surface slot.
//!
//! Severity: `Warn` (advisory). Cards *do* fall through to the surface
//! layer by default in `tokens.css`, so an existing `var(--card)` is
//! visually correct today; the rule points at the *intent* — write for
//! the layer, not the slot.
use std::path::Path;

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sail-surface-not-card";

const SEVERITY: Severity = Severity::Warn;

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    for block in scan_style_blocks(source) {
        for hit in scan_card_value_uses(&block.content) {
            let abs = block.content_offset + hit.offset;
            let (line, col) = byte_offset_to_line_col(source, abs);
            findings.push(Finding {
                file: path.to_path_buf(),
                line,
                col,
                rule: RULE_ID,
                severity: SEVERITY,
                message: format!(
                    "`var(--card)` used as the surface in `{}` — read \
                     `var(--surface-1)` (or `var(--surface-2)`) instead",
                    hit.property
                ),
                suggest: Some(SUGGEST.to_string()),
            });
        }
    }
    findings.sort_by(|a, b| a.line.cmp(&b.line).then_with(|| a.col.cmp(&b.col)));
    findings
}

/// Shared fix hint. Stays in front of the human author so the intent
/// (the surface layer, not the card slot) is the obvious next step.
const SUGGEST: &str = "Replace with `var(--surface-1)` — cards consume the \
                       surface layer so a theme that raises card depth (a \
                       gradient background, a tinted nested panel) only \
                       touches `--surface-1` / `--surface-2` (anti-smell #4).";

/// Properties whose value is the surface (a color-bearing background
/// channel). The rule fires only on declarations in this set.
///
/// We deliberately keep the set small: a `font-weight: var(--card)` is
/// nonsensical, so checking it would only produce noise. `background`,
/// `background-color`, and the texture channel `background-image` are
/// the three CSS properties that paint a card's surface.
const SURFACE_PROPERTIES: &[&str] = &[
    "background",
    "background-color",
    "background-image",
    "fill",
    "stroke",
    "box-shadow",
];

/// Position of one `var(--card)` value-use inside the style block.
struct CardHit<'a> {
    /// Byte offset of the value into the style block content.
    offset: usize,
    /// The property the finding lives on.
    property: &'a str,
}

/// Find every `var(--card)` value-use. Two layers of matching keep
/// false positives out:
///
/// 1. The declaration's *property* must be in `SURFACE_PROPERTIES` —
///    `font-weight: var(--card)` etc. is not the surface layer.
/// 2. The token name must be *exactly* `--card` — `--card-foreground`,
///    `--card-texture`, etc. have a different slot semantics and are
///    skipped via the `is_ident_byte` boundary check.
fn scan_card_value_uses(content: &str) -> Vec<CardHit<'_>> {
    let mut out = Vec::new();
    let bytes = content.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // Skip CSS comments.
        if bytes[i] == b'/' && bytes.get(i + 1) == Some(&b'*') {
            if let Some(end_rel) = find_subslice(&bytes[i + 2..], b"*/") {
                i += 2 + end_rel + 2;
                continue;
            }
            break;
        }
        if matches_at(bytes, i, b"var(") {
            // Walk the var() body to find the top-level token name. We
            // require whitespace or `--` immediately after `var(`, and
            // the name itself to be followed by `)` or whitespace (the
            // fallback separator) — anything else means a longer name
            // like `card-foreground` and we skip.
            let start = i + b"var(".len();
            let mut k = start;
            while k < bytes.len() && bytes[k].is_ascii_whitespace() {
                k += 1;
            }
            if bytes.get(k) == Some(&b'-') && bytes.get(k + 1) == Some(&b'-') {
                let name_start = k + 2;
                let mut n = name_start;
                while n < bytes.len() && is_ident_byte(bytes[n]) {
                    n += 1;
                }
                let name = &content[name_start..n];
                // Exact-name check: the next byte must close the var()
                // (whitespace, `,` for fallback, or `)`). A `-` here
                // means we're looking at `--card-foreground` or similar
                // — not the bare surface slot.
                let next = bytes.get(n).copied();
                let is_exact = matches!(next, Some(b')' | b',' | b' ' | b'\t' | b'\n' | b'\r'));
                if is_exact && name == "card" {
                    // Find the declaration this var() lives in. We walk
                    // *backwards* from the var() looking for the most
                    // recent `;` or `{`, then forward to `:`. The
                    // property name is the slice between the boundary
                    // and the colon, trimmed.
                    if let Some(prop) = enclosing_property(&content[..start]) {
                        if SURFACE_PROPERTIES.contains(&prop) {
                            out.push(CardHit {
                                offset: start,
                                property: prop,
                            });
                        }
                    }
                }
            }
            // Resume *inside* the var() so a nested var() — e.g. a
            // fallback — would be inspected on the next iteration.
            i = start;
            continue;
        }
        i += 1;
    }
    out
}

/// Given the bytes preceding a `var(...)` reference, find the property
/// name of the declaration the reference lives in. Returns `None` for
/// at-rule bodies (`@media`, `@supports`) where the property is in an
/// inner rule, not in this scope.
fn enclosing_property(before: &str) -> Option<&str> {
    // The reference lives inside `{ ... }` and after a `;` or opening
    // brace that introduced the current declaration. Find the last
    // boundary, then walk to the `:`.
    let bytes = before.as_bytes();
    let mut boundary = 0usize;
    let mut i = bytes.len();
    while i > 0 {
        let b = bytes[i - 1];
        if matches!(b, b';' | b'{' | b'}') {
            boundary = i;
            break;
        }
        i -= 1;
    }
    let mut j = boundary;
    while j < bytes.len() && bytes[j].is_ascii_whitespace() {
        j += 1;
    }
    let name_start = j;
    while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'-') {
        j += 1;
    }
    if j == name_start {
        return None;
    }
    let mut k = j;
    while k < bytes.len() && bytes[k].is_ascii_whitespace() {
        k += 1;
    }
    if k >= bytes.len() || bytes[k] != b':' {
        return None;
    }
    Some(before[name_start..j].trim())
}

// ---------------------------------------------------------------------
// Shared scanning primitives. Kept local to the rule (same convention
// as the sibling css_token / css_literal / layout_primitive rules) —
// each rule owns its position-mapping and the shape may diverge.
// ---------------------------------------------------------------------

struct StyleBlock<'a> {
    content_offset: usize,
    content: &'a str,
}

fn scan_style_blocks(source: &str) -> Vec<StyleBlock<'_>> {
    let mut out = Vec::new();
    let bytes = source.as_bytes();
    let mut i = 0;
    while let Some(rel) = find_subslice(&bytes[i..], b"<style") {
        let tag_start = i + rel;
        let after_tag = tag_start + b"<style".len();
        if !matches!(
            bytes.get(after_tag).copied(),
            Some(b' ' | b'\t' | b'\n' | b'\r' | b'>' | b'/')
        ) {
            i = after_tag;
            continue;
        }
        let Some(open_end_rel) = find_subslice(&bytes[after_tag..], b">") else {
            break;
        };
        if open_end_rel > 0 && bytes.get(after_tag + open_end_rel - 1) == Some(&b'/') {
            i = after_tag + open_end_rel + 1;
            continue;
        }
        let content_start = after_tag + open_end_rel + 1;
        let Some(close_rel) = find_subslice(&bytes[content_start..], b"</style>") else {
            break;
        };
        let content_end = content_start + close_rel;
        out.push(StyleBlock {
            content_offset: content_start,
            content: &source[content_start..content_end],
        });
        i = content_end + b"</style>".len();
    }
    out
}

fn matches_at(bytes: &[u8], i: usize, needle: &[u8]) -> bool {
    bytes.len() >= i + needle.len() && &bytes[i..i + needle.len()] == needle
}

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'_'
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    let last = haystack.len() - needle.len();
    (0..=last).find(|&i| &haystack[i..i + needle.len()] == needle)
}

fn byte_offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let bytes = source.as_bytes();
    let mut line = 1usize;
    let mut col = 1usize;
    let cap = offset.min(bytes.len());
    for &b in &bytes[..cap] {
        if b == b'\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn run(src: &str) -> Vec<Finding> {
        check(&PathBuf::from("/fake/foo.sui"), src)
    }

    // --- Bare --card fires ---

    #[test]
    fn var_card_in_background_fires() {
        let src = "<style>:host { background: var(--card); }</style>";
        let f = run(src);
        assert_eq!(f.len(), 1, "{f:?}");
        assert_eq!(f[0].rule, RULE_ID);
        assert_eq!(f[0].severity, Severity::Warn);
        assert!(f[0].message.contains("var(--card)"), "{}", f[0].message);
        assert!(f[0].message.contains("background"), "{}", f[0].message);
        assert!(f[0].suggest.as_deref().unwrap().contains("--surface-1"));
    }

    #[test]
    fn var_card_in_background_color_fires() {
        let src = "<style>.x { background-color: var(--card); }</style>";
        let f = run(src);
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(f[0].message.contains("background-color"), "{}", f[0].message);
    }

    #[test]
    fn var_card_in_fill_fires() {
        // SVG primitives use `fill: var(--card)` for a card surface.
        let src = "<style>rect { fill: var(--card); }</style>";
        let f = run(src);
        assert_eq!(f.len(), 1, "{f:?}");
    }

    // --- Companion tokens do NOT fire ---

    #[test]
    fn var_card_foreground_does_not_fire() {
        // `--card-foreground` is the paired text token, not the surface.
        let src = "<style>:host { color: var(--card-foreground); }</style>";
        let f = run(src);
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn var_card_texture_does_not_fire() {
        // `--card-texture` is a CSS background value (e.g. `none` or a
        // data-URI), not the surface slot. Even when used in
        // `background-image: var(--card-texture)` it is the texture
        // channel, not the color.
        let src = "<style>.x { background-image: var(--card-texture); }</style>";
        let f = run(src);
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn var_card_hyphenated_modifier_does_not_fire() {
        // A future `--card-elevation` or `--card-edge` is a different
        // slot; the rule is exact-name only.
        let src = "<style>.x { background: var(--card-elevation); }</style>";
        let f = run(src);
        assert!(f.is_empty(), "{f:?}");
    }

    // --- Definitions are not findings ---

    #[test]
    fn card_definition_does_not_fire() {
        let src = "<style>:root { --card: oklch(0.95 0 0); }</style>";
        let f = run(src);
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn card_foreground_definition_does_not_fire() {
        let src = "<style>:root { --card-foreground: oklch(0.2 0 0); }</style>";
        let f = run(src);
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn card_texture_definition_does_not_fire() {
        let src = "<style>:root { --card-texture: none; }</style>";
        let f = run(src);
        assert!(f.is_empty(), "{f:?}");
    }

    // --- Non-surface properties are skipped ---

    #[test]
    fn var_card_in_non_surface_property_does_not_fire() {
        // `color`, `border-color`, etc. read the *foreground* of the
        // card, not its surface — that is fine and is the
        // `var(--card-foreground)` slot, but `var(--card)` here is a
        // misuse the rule should NOT flag (different rule's job — the
        // own-slot misuse is a different shape, not the surface smell).
        let src = "<style>:host { color: var(--card); }</style>";
        let f = run(src);
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn var_card_in_border_color_does_not_fire() {
        let src = "<style>.x { border-color: var(--card); }</style>";
        let f = run(src);
        assert!(f.is_empty(), "{f:?}");
    }

    // --- Scoping / robustness ---

    #[test]
    fn no_style_block_no_findings() {
        let src = "<div style=\"background: var(--card)\"></div>";
        assert!(run(src).is_empty(), "{:?}", run(src));
    }

    #[test]
    fn commented_var_card_is_skipped() {
        let src = "<style>/* background: var(--card); */ :host { color: var(--foreground); }</style>";
        let f = run(src);
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn fallback_form_still_fires_on_outer_card() {
        // `var(--card, …)` — the var() is a fallback form, but the
        // surface smell is *using* `--card` as a value. Anti-smell #4
        // is independent of anti-smell #8; the same finding fires.
        let src = "<style>:host { background: var(--card, var(--surface-1)); }</style>";
        let f = run(src);
        // The sibling css_literals rule owns the *fallback* smell
        // (severity Warn, different rule id); this rule fires only on
        // the bare surface value. var(--card, …) has the name `card`
        // followed by `,`, so the exact-name check passes.
        assert!(
            f.iter().any(|x| x.rule == RULE_ID),
            "expected a surface-not-card finding: {f:?}"
        );
    }

    #[test]
    fn clean_surface_use_has_no_findings() {
        let src = r#"<style>
            :host {
                background: var(--surface-1);
                border: 1px solid var(--border);
                color: var(--foreground);
            }
        </style>"#;
        assert!(run(src).is_empty(), "{:?}", run(src));
    }

    #[test]
    fn position_points_at_var_call() {
        let src = "<style>\n  .x { background: var(--card); }\n</style>";
        let f = run(src);
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].line, 2);
        // "  .x { background: var(" is 23 chars; `var(` opens at col 24.
        // The reported offset is the start of the value, which is the
        // byte *after* `var(` — col 28 (the `--card`).
        assert!(
            f[0].col >= 24,
            "finding should point inside the var() call, got col {}",
            f[0].col
        );
    }

    #[test]
    fn multiple_uses_each_fire() {
        let src = "<style>.a { background: var(--card); } .b { background-color: var(--card); }</style>";
        let f = run(src);
        assert_eq!(f.len(), 2, "{f:?}");
    }
}
