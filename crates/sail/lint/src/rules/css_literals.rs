//! `sail-css-literals` — design-system *anti-smells* inside a `.sail`
//! `<style>` block. These are the token-layer bypasses the primitives
//! audit flagged 28 times and that were, until this rule, REVIEW-only.
//!
//! What this rule flags (each maps to a `ship-design-system` anti-smell):
//! - **Inline `color-mix(...)`** (anti-smell #2). A `color-mix(in oklch,
//!   var(--success) 14%, transparent)` formula hand-rolled in a
//!   component should be a pre-computed ramp token (`var(--hover-bg)`).
//!   The formula belongs in `tokens.css` once, not scattered across
//!   primitives where the percentages drift.
//! - **`oklch(...)` / `oklab(...)` color literals** (anti-smells #1, #3).
//!   Ship is OKLCH-native, but a raw `oklch(0.7 0.1 200)` *literal* in a
//!   component bypasses the palette. (The `in oklch` / `in oklab`
//!   *color-space keyword* inside `color-mix` is NOT a function call and
//!   is not matched here — the surrounding `color-mix(` is the finding.)
//! - **Hex colors in themed properties** (anti-smell #3). `#3b82f6` in a
//!   `color`/`background`/`border-color`/… declaration ignores the
//!   active theme entirely.
//! - **`var(--token, <fallback>)` fallback forms** (anti-smell #8).
//!   Tokens are guaranteed loaded (tokens.css is imported at the document
//!   root); a fallback creates a parallel, stale system that masks
//!   token-load-order bugs. The *presence of any fallback* is the smell —
//!   whether the fallback is a literal, another token, or `0`.
//!
//! Relationship to the sibling `sail-css-literal` (singular) rule:
//! `css_literal` is the narrow, ERROR-severity rule that rejects raw
//! color literals (hex / rgb / hsl / named) inside color-bearing
//! declarations and inside `var(--x, RAW)` color fallbacks — it
//! deliberately *passes* inline `color-mix(...)` (it blanks the inner
//! `var()`s) and does not touch token-valued fallbacks. This rule
//! (`css_literals`, plural) covers exactly that complementary surface:
//! the `color-mix` formula itself, standalone `oklch`/`oklab` literals,
//! and the *fallback form regardless of fallback value*. Overlap is
//! avoided by design — the hex-in-themed-property check here is the one
//! intentional intersection, kept so this rule is a complete statement
//! of the audit's four anti-smells; the two findings carry different
//! rule IDs and severities, so a reviewer sees both framings.
//!
//! ## Severity: WARNING, not Error — and why
//!
//! At introduction there are ~28 existing violations across the
//! primitive library (see `.artifacts/sail-arc/primitives-audit.md`):
//! the inline-`color-mix` cluster, `sail-color-picker`'s hex palette,
//! `sail-icon`'s `oklab` color-space, and the `var(--token, fallback)`
//! patterns in `sail-toast`/`sail-card`/`sail-tab-panel`/`sail-accordion`/
//! `sail-alert`. Promoting this rule to `Error` today would fail every
//! `.sail` compile that touches those primitives.
//!
//! **This rule FLIPS TO `Error` once Tier B clears the existing
//! violations** (the tone-ramp sweep + fallback strip described in
//! the audit's "Top priorities"). The pattern mirrors `sail-css-literal`,
//! which started at `Warn` during its own Warn window and is now
//! `Error`. To flip: change every `Severity::Warn` below to
//! `Severity::Error` and delete this paragraph.

use std::path::Path;

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sail-css-literals";

/// Severity for every finding this rule emits. `Warn` during the
/// Warn window; flip to `Error` once Tier B clears the ~28
/// existing violations (see the module header).
const SEVERITY: Severity = Severity::Warn;

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    for block in scan_style_blocks(source) {
        findings.extend(scan_block(path, source, &block));
    }
    findings.sort_by(|a, b| a.line.cmp(&b.line).then_with(|| a.col.cmp(&b.col)));
    findings
}

/// Scan one `<style>` block for all four anti-smells. Offsets reported
/// are absolute byte offsets into the original `source`.
fn scan_block(path: &Path, source: &str, block: &StyleBlock) -> Vec<Finding> {
    let mut findings = Vec::new();
    let content = block.content;

    // 1. Inline color-mix(...) and 2. standalone oklch/oklab literals.
    //    Both are detected by a single function-call scan so a name
    //    isn't matched twice (and so `color-mix` swallows nothing it
    //    shouldn't — the inner color-space keyword is bare, not a call).
    for hit in scan_color_functions(content) {
        let abs = block.content_offset + hit.offset;
        let (line, col) = byte_offset_to_line_col(source, abs);
        let (message, suggest) = match hit.kind {
            FnKind::ColorMix => (
                "inline `color-mix(...)` — read a pre-computed ramp token instead".to_string(),
                "Replace the hand-rolled mix with the matching ramp token \
                 (e.g. `var(--tone-{tone}-bg)` / `var(--tone-{tone}-border)`). \
                 The mix formula lives in tokens.css once; scattering it across \
                 primitives lets the percentages drift (anti-smell #2)."
                    .to_string(),
            ),
            FnKind::OklchLiteral => (
                format!(
                    "raw `{}(...)` color literal — use a design-system token (`var(--…)`)",
                    hit.name
                ),
                "Ship is OKLCH-native, but a raw color literal bypasses the \
                 palette and ignores the active theme. Use the matching token \
                 (`--primary`, `--foreground`, `--border`, a `--tone-*` ramp, …) \
                 from packages/sail/tokens/src/tokens.css (anti-smells #1, #3)."
                    .to_string(),
            ),
        };
        findings.push(Finding {
            file: path.to_path_buf(),
            line,
            col,
            rule: RULE_ID,
            severity: SEVERITY,
            message,
            suggest: Some(suggest),
        });
    }

    // 3. Hex colors in themed (color-bearing) declarations.
    for hit in scan_themed_hex(content) {
        let abs = block.content_offset + hit.offset;
        let (line, col) = byte_offset_to_line_col(source, abs);
        findings.push(Finding {
            file: path.to_path_buf(),
            line,
            col,
            rule: RULE_ID,
            severity: SEVERITY,
            message: format!(
                "hex color `{}` in `{}` — use a design-system token (`var(--…)`)",
                hit.text, hit.property
            ),
            suggest: Some(
                "Hardcoded hex ignores the active theme. Pick the closest token \
                 from packages/sail/tokens/src/tokens.css (anti-smell #3)."
                    .to_string(),
            ),
        });
    }

    // 4. var(--token, <fallback>) fallback forms (any fallback).
    for hit in scan_var_fallbacks(content) {
        let abs = block.content_offset + hit.offset;
        let (line, col) = byte_offset_to_line_col(source, abs);
        findings.push(Finding {
            file: path.to_path_buf(),
            line,
            col,
            rule: RULE_ID,
            severity: SEVERITY,
            message: format!(
                "`var(--{}, …)` fallback — tokens are guaranteed loaded; drop the fallback",
                hit.name
            ),
            suggest: Some(
                "tokens.css is imported at the document root, so every `var(--…)` \
                 resolves. A fallback creates a parallel, stale system that masks \
                 token-load-order bugs. Write `var(--token)` with no fallback \
                 (anti-smell #8)."
                    .to_string(),
            ),
        });
    }

    findings
}

// ---------------------------------------------------------------------
// Anti-smell 1+2: color-mix() and oklch()/oklab() literals.
// ---------------------------------------------------------------------

enum FnKind {
    ColorMix,
    OklchLiteral,
}

struct FnHit<'a> {
    offset: usize,
    name: &'a str,
    kind: FnKind,
}

/// Scan for `color-mix(` and standalone `oklch(` / `oklab(` calls.
/// Matching is at a word boundary and requires the `(` immediately
/// after the name (modulo nothing — CSS does not allow whitespace
/// between a function name and its `(`). This is what keeps the
/// color-space keyword `in oklab` (no paren) from being mistaken for an
/// `oklab(` literal.
fn scan_color_functions(content: &str) -> Vec<FnHit<'_>> {
    let mut out = Vec::new();
    let bytes = content.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // Skip CSS comments so commented-out examples don't fire.
        if bytes[i] == b'/' && bytes.get(i + 1) == Some(&b'*') {
            if let Some(end_rel) = find_subslice(&bytes[i + 2..], b"*/") {
                i += 2 + end_rel + 2;
                continue;
            }
            break;
        }
        let at_word_boundary = i == 0 || !is_ident_byte(bytes[i - 1]);
        if at_word_boundary && is_ident_byte(bytes[i]) {
            let name_start = i;
            while i < bytes.len() && is_ident_byte(bytes[i]) {
                i += 1;
            }
            let name = &content[name_start..i];
            // Function call iff the very next byte is `(`.
            if bytes.get(i) == Some(&b'(') {
                let kind = match name {
                    "color-mix" => Some(FnKind::ColorMix),
                    "oklch" | "oklab" => Some(FnKind::OklchLiteral),
                    _ => None,
                };
                if let Some(kind) = kind {
                    out.push(FnHit {
                        offset: name_start,
                        name,
                        kind,
                    });
                }
            }
            continue;
        }
        i += 1;
    }
    out
}

// ---------------------------------------------------------------------
// Anti-smell 3: hex colors in themed properties.
// ---------------------------------------------------------------------

struct HexHit<'a> {
    offset: usize,
    text: &'a str,
    property: &'a str,
}

/// Properties whose value is themed (color-bearing). A hex literal in
/// any of these ignores the theme. Mirrors `css_literal`'s set so the
/// two rules agree on what "themed" means.
const THEMED_PROPERTIES: &[&str] = &[
    "color",
    "background",
    "background-color",
    "border-color",
    "border-top-color",
    "border-right-color",
    "border-bottom-color",
    "border-left-color",
    "outline-color",
    "fill",
    "stroke",
    "border",
    "border-top",
    "border-right",
    "border-bottom",
    "border-left",
    "box-shadow",
    "text-shadow",
];

/// Find hex literals (`#abc`, `#aabbcc`, `#abcd`, `#aabbccdd`) inside
/// themed declarations. We scan the value verbatim — a hex inside a
/// `var(--x, #fff)` fallback is also a fallback smell (anti-smell #4),
/// but a hex in a themed property is reported here regardless of where
/// it sits, since the audit calls hex-in-themed-property its own smell.
fn scan_themed_hex(content: &str) -> Vec<HexHit<'_>> {
    let mut out = Vec::new();
    for decl in scan_declarations(content) {
        if !THEMED_PROPERTIES.contains(&decl.property) {
            continue;
        }
        let value = decl.value;
        let bytes = value.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'#' {
                let mut j = i + 1;
                while j < bytes.len() && bytes[j].is_ascii_hexdigit() {
                    j += 1;
                }
                let len = j - i - 1;
                if matches!(len, 3 | 4 | 6 | 8) {
                    out.push(HexHit {
                        offset: decl.value_offset + i,
                        text: &value[i..j],
                        property: decl.property,
                    });
                }
                i = j.max(i + 1);
                continue;
            }
            i += 1;
        }
    }
    out
}

// ---------------------------------------------------------------------
// Anti-smell 4: var(--token, <fallback>) fallback forms.
// ---------------------------------------------------------------------

struct FallbackHit<'a> {
    /// Offset of the `var(` start.
    offset: usize,
    name: &'a str,
}

/// Find every `var(--name, <anything>)` reference — the presence of a
/// fallback (a top-level comma inside the `var(...)`) is the smell.
/// Nested `var(--inner)` inside the fallback is scanned too: the outer
/// scan resumes after the closing paren, but the inner var's own
/// fallback (if any) is found on the next iteration because we walk the
/// whole content, not balanced groups exclusively.
fn scan_var_fallbacks(content: &str) -> Vec<FallbackHit<'_>> {
    let mut out = Vec::new();
    let bytes = content.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // Skip comments.
        if bytes[i] == b'/' && bytes.get(i + 1) == Some(&b'*') {
            if let Some(end_rel) = find_subslice(&bytes[i + 2..], b"*/") {
                i += 2 + end_rel + 2;
                continue;
            }
            break;
        }
        if matches_at(bytes, i, b"var(") {
            let start = i;
            // Locate the first top-level comma (a fallback) by walking to
            // the matching close paren. We don't need the close-paren
            // position itself — scanning resumes just inside `var(` below
            // so nested var()s in the fallback are also caught.
            let mut depth = 0i32;
            let mut comma_at: Option<usize> = None;
            let mut j = i;
            while j < bytes.len() {
                match bytes[j] {
                    b'(' => depth += 1,
                    b')' => {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    b',' if depth == 1 && comma_at.is_none() => comma_at = Some(j),
                    _ => {}
                }
                j += 1;
            }
            // Extract the token name: skip `var(` + whitespace, expect `--`.
            let mut k = start + b"var(".len();
            while k < bytes.len() && bytes[k].is_ascii_whitespace() {
                k += 1;
            }
            if comma_at.is_some() && bytes.get(k) == Some(&b'-') && bytes.get(k + 1) == Some(&b'-')
            {
                let name_start = k + 2;
                let mut n = name_start;
                while n < bytes.len() && is_ident_byte(bytes[n]) {
                    n += 1;
                }
                out.push(FallbackHit {
                    offset: start,
                    name: &content[name_start..n],
                });
            }
            // Resume scanning *inside* the var() (right after `var(`) so a
            // nested `var(--inner, …)` in the fallback is also caught.
            i = start + b"var(".len();
            continue;
        }
        i += 1;
    }
    out
}

// ---------------------------------------------------------------------
// Shared scanning primitives. Kept local to the rule (same convention
// as the sibling css_token / css_literal rules) — each rule owns its
// position-mapping and the shapes may diverge.
// ---------------------------------------------------------------------

struct StyleBlock<'a> {
    content_offset: usize,
    content: &'a str,
}

struct Decl<'a> {
    property: &'a str,
    /// Byte offset of the value into the style block content.
    value_offset: usize,
    value: &'a str,
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

/// Walk every `property: value` declaration in a style block. Skips CSS
/// comments and balances parens so a `;` inside `var(...)` / `color-mix(...)`
/// doesn't terminate the value early.
fn scan_declarations(content: &str) -> Vec<Decl<'_>> {
    let mut out = Vec::new();
    let bytes = content.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        while i < bytes.len()
            && (bytes[i].is_ascii_whitespace() || matches!(bytes[i], b';' | b'{' | b'}'))
        {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        if bytes[i] == b'/' && bytes.get(i + 1) == Some(&b'*') {
            if let Some(end_rel) = find_subslice(&bytes[i + 2..], b"*/") {
                i += 2 + end_rel + 2;
                continue;
            }
            break;
        }
        let name_start = i;
        while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'-') {
            i += 1;
        }
        if i == name_start {
            i += 1;
            continue;
        }
        let name = &content[name_start..i];
        let mut k = i;
        while k < bytes.len() && bytes[k].is_ascii_whitespace() {
            k += 1;
        }
        if k >= bytes.len() || bytes[k] != b':' {
            while i < bytes.len() && !matches!(bytes[i], b';' | b'{' | b'}') {
                i += 1;
            }
            continue;
        }
        let mut v = k + 1;
        while v < bytes.len() && bytes[v].is_ascii_whitespace() {
            v += 1;
        }
        let value_start = v;
        let mut depth = 0i32;
        while v < bytes.len() {
            match bytes[v] {
                b'(' => depth += 1,
                b')' => depth -= 1,
                b';' | b'}' if depth <= 0 => break,
                _ => {}
            }
            v += 1;
        }
        out.push(Decl {
            property: name,
            value_offset: value_start,
            value: content[value_start..v].trim_end(),
        });
        i = v;
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
        check(&PathBuf::from("/fake/foo.sail"), src)
    }

    // --- Anti-smell 1: inline color-mix() ---

    #[test]
    fn inline_color_mix_reports() {
        let src = "<style>:host { background: color-mix(in oklch, var(--success) 14%, transparent); }</style>";
        let f = run(src);
        assert_eq!(f.len(), 1, "{f:?}");
        assert_eq!(f[0].rule, RULE_ID);
        assert_eq!(f[0].severity, Severity::Warn);
        assert!(f[0].message.contains("color-mix"), "{}", f[0].message);
    }

    #[test]
    fn color_mix_in_oklab_does_not_double_report_the_keyword() {
        // sail-icon ships `color-mix(in oklab, …)`. The `in oklab` keyword
        // is NOT a function call, so only the `color-mix(` fires — one
        // finding, not two.
        let src =
            "<style>:host { fill: color-mix(in oklab, var(--primary) 50%, transparent); }</style>";
        let f = run(src);
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(f[0].message.contains("color-mix"));
    }

    // --- Anti-smell 2: oklch()/oklab() literals ---

    #[test]
    fn oklch_literal_reports() {
        let src = "<style>:host { color: oklch(0.7 0.1 200); }</style>";
        let f = run(src);
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(f[0].message.contains("oklch"), "{}", f[0].message);
    }

    #[test]
    fn oklab_literal_reports() {
        let src = "<style>:host { color: oklab(0.7 0.1 0.05); }</style>";
        let f = run(src);
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(f[0].message.contains("oklab"));
    }

    // --- Anti-smell 3: hex in themed properties ---

    #[test]
    fn hex_in_color_property_reports() {
        let src = "<style>:host { color: #3b82f6; }</style>";
        let f = run(src);
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(f[0].message.contains("#3b82f6"), "{}", f[0].message);
        assert!(f[0].message.contains("color"));
    }

    #[test]
    fn hex_three_digit_reports() {
        let src = "<style>:host { background: #fff; }</style>";
        assert_eq!(run(src).len(), 1);
    }

    #[test]
    fn hex_in_non_themed_property_is_ignored() {
        // A `#` outside a color-bearing property (e.g. a content
        // string-ish or grid-template name) is not our concern here.
        let src = "<style>:host { grid-template-columns: [#col] 1fr; }</style>";
        assert!(run(src).is_empty(), "{:?}", run(src));
    }

    // --- Anti-smell 4: var(--token, fallback) ---

    #[test]
    fn var_with_literal_fallback_reports() {
        let src = "<style>:host { color: var(--info, #3b82f6); }</style>";
        let f = run(src);
        // One fallback finding + one themed-hex finding (the literal in a
        // color property is also a hex-in-themed smell).
        assert!(
            f.iter().any(|x| x.message.contains("fallback")),
            "expected a fallback finding: {f:?}"
        );
        assert!(
            f.iter().any(|x| x.message.contains("var(--info, …)")),
            "{f:?}"
        );
    }

    #[test]
    fn var_with_token_fallback_reports() {
        // The fallback being *another token* is still the smell — the
        // fallback form itself is forbidden (anti-smell #8).
        let src = "<style>:host { color: var(--card, var(--background)); }</style>";
        let f = run(src);
        // Outer var(--card, …) is a fallback; inner var(--background) has
        // no fallback. Exactly one finding.
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(f[0].message.contains("var(--card, …)"), "{}", f[0].message);
    }

    #[test]
    fn var_with_px_fallback_reports() {
        let src = "<style>:host { font-size: var(--text-base, 14px); }</style>";
        let f = run(src);
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(f[0].message.contains("fallback"));
        assert!(f[0].message.contains("var(--text-base, …)"));
    }

    #[test]
    fn var_without_fallback_passes() {
        let src = "<style>:host { color: var(--foreground); }</style>";
        assert!(run(src).is_empty(), "{:?}", run(src));
    }

    #[test]
    fn nested_fallback_inside_fallback_both_report() {
        // var(--a, var(--b, 1px)) — both --a and --b carry a fallback.
        let src = "<style>:host { width: var(--a, var(--b, 1px)); }</style>";
        let f = run(src);
        assert_eq!(f.len(), 2, "{f:?}");
        assert!(f.iter().any(|x| x.message.contains("var(--a, …)")));
        assert!(f.iter().any(|x| x.message.contains("var(--b, …)")));
    }

    // --- Clean case ---

    #[test]
    fn clean_tokenized_style_has_no_findings() {
        let src = r#"<style>
            :host {
                color: var(--foreground);
                background: var(--hover-bg);
                border: 1px solid var(--border);
                border-radius: var(--radius-md);
            }
        </style>"#;
        assert!(run(src).is_empty(), "{:?}", run(src));
    }

    // --- Scoping / robustness ---

    #[test]
    fn no_style_block_no_findings() {
        let src = "<div>color-mix(in oklch, a, b) var(--x, 1px) #fff</div>";
        assert!(run(src).is_empty(), "{:?}", run(src));
    }

    #[test]
    fn commented_out_smell_is_skipped() {
        let src = "<style>/* background: color-mix(in oklch, var(--a), transparent); */ :host { color: var(--foreground); }</style>";
        assert!(run(src).is_empty(), "{:?}", run(src));
    }

    #[test]
    fn position_points_at_the_construct() {
        let src = "<style>\n  :host { color: oklch(0.7 0.1 200); }\n</style>";
        let f = run(src);
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].line, 2);
        // `  :host { color: ` is 17 chars; the `o` of `oklch` is col 18.
        assert_eq!(f[0].col, 18);
    }

    #[test]
    fn multiple_smells_reported_sorted_by_position() {
        let src = "<style>:host { color: #fff; background: color-mix(in oklch, var(--a), transparent); border-color: var(--x, red); }</style>";
        let f = run(src);
        // #fff (themed hex), color-mix, var fallback, and the `red` in the
        // fallback is themed-hex? No — `red` is a named color, not hex;
        // this rule's hex check is hex-only. So: hex + color-mix + fallback.
        assert!(f.len() >= 3, "{f:?}");
        // Sorted ascending by (line, col).
        for w in f.windows(2) {
            assert!(
                (w[0].line, w[0].col) <= (w[1].line, w[1].col),
                "not sorted: {f:?}"
            );
        }
    }
}
