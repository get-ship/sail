//! `sail-primary-for-status` — anti-smell #7 from `ship-design-system`.
//!
//! "Never read a theme's `--primary` directly for a status-like intent.
//!  Use the tone ramps. `--primary` is a brand slot, not a status signal."
//!
//! `--primary` is the *brand* colour. It does not communicate success,
//! failure, or warning. Components that tint a `destructive` / `warning`
//! / `success` / `info` / `accent` selector with `var(--primary)` bake
//! themselves against a slot that says "this is the company colour",
//! defeating the entire status vocabulary. The fix is the matching tone
//! ramp (`--tone-X-bg` / `-border` / `-fg` / `-ring`).
//!
//! What this rule flags:
//! - A rule whose selector mentions a *status* tone (`[tone=…]`,
//!   `[invalid]`) AND whose body references `var(--primary)`.
//!
//! What it deliberately does NOT flag:
//! - `:host([tone='primary'])` — the brand tone; `--primary` is correct
//!   here. A brand-tinted control is a brand surface, not a status
//!   signal.
//! - `:host(:not([tone]))` / `:host([tone='default'])` — the *default*
//!   block. The default is supposed to look like the brand surface.
//! - `var(--primary-foreground)` — the paired text-on-primary token.
//! - `--primary: …;` declarations — themes defining the slot.
//! - `var(--primary-…)` for any other modifier (`--primary-hover`,
//!   `--primary-active`, …) — paired tokens; the rule fires only on
//!   the bare brand slot.
//! - The `accent` *modifier* on a non-status selector (e.g. a
//!   class-based highlight using `--primary` for the highlight color)
//!   is out of scope; the rule is *status-tone-list-driven*, not "any
//!   use of `--primary` is bad".
//!
//! Severity: `Warn` (advisory). Today the brand slot happens to read
//! correctly in most themes, so the finding is intent correction, not
//! a correctness gate.
use std::path::Path;

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sail-primary-for-status";

const SEVERITY: Severity = Severity::Warn;

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    for block in scan_style_blocks(source) {
        for rule in scan_rule_blocks(&block.content) {
            // A list of selectors can mix status and non-status
            // (`:host([tone='destructive']), :host([invalid])`).
            // Evaluate each comma-separated piece independently — if
            // ANY piece is a status-tone selector, the whole rule is
            // a status block.
            let status_pieces: Vec<&str> = rule
                .selector
                .split(',')
                .map(str::trim)
                .filter(|p| is_status_tone_selector(p))
                .collect();
            if status_pieces.is_empty() {
                continue;
            }
            // The rule body must reference `var(--primary)` exactly
            // (not `--primary-foreground`, not `--primary-…`).
            if let Some(hit_offset) = find_primary_ref(&rule.body) {
                let abs = block.content_offset + rule.body_offset + hit_offset;
                let (line, col) = byte_offset_to_line_col(source, abs);
                findings.push(Finding {
                    file: path.to_path_buf(),
                    line,
                    col,
                    rule: RULE_ID,
                    severity: SEVERITY,
                    message: format!(
                        "`var(--primary)` inside a status-tone block \
                         ({}) — use the matching tone ramp instead",
                        status_pieces.join(", ")
                    ),
                    suggest: Some(SUGGEST.to_string()),
                });
            }
        }
    }
    findings.sort_by(|a, b| a.line.cmp(&b.line).then_with(|| a.col.cmp(&b.col)));
    findings
}

/// Shared fix hint. Names the ramp slots so the human author sees
/// "use the matching ramp" rather than re-deriving the rule.
const SUGGEST: &str = "Use the matching tone ramp \
                       (`--tone-X-bg`, `--tone-X-border`, `--tone-X-fg`, \
                       `--tone-X-ring`) instead of `--primary` for status \
                       intent. `--primary` is a brand slot \
                       (anti-smell #7).";

/// Tone names that communicate a *status* (not the brand). A selector
/// mentioning any of these is a status block; using `--primary` there
/// is the smell.
///
/// Deliberately does NOT include `primary` (brand), `default` (the
/// default-tone block, which is supposed to read like the brand), or
/// `muted` (a non-status tint). The non-status tones map to brand or
/// neutral layers and `--primary` there is fine.
const STATUS_TONES: &[&str] = &[
    "destructive",
    "warning",
    "success",
    "info",
    "accent",
];

/// True iff `piece` is a selector for a *status* tone. Matches:
/// - `:host([tone='destructive'])` (and siblings)
/// - `:host([tone="destructive"])` (double-quoted form, just in case)
/// - `:host([invalid])` (a status signal even without an explicit tone)
///
/// Brand tone, default tone, no-tone default, and any other modifier
/// return `false`.
fn is_status_tone_selector(piece: &str) -> bool {
    if piece.is_empty() {
        return false;
    }
    // [invalid] is a status signal independent of the tone attribute.
    if piece.contains("[invalid]") {
        return true;
    }
    // Walk `[tone=…]` (single or double quoted) and report the
    // enclosed value.
    for tone in STATUS_TONES {
        let pat_single = format!("[tone='{tone}']");
        let pat_double = format!("[tone=\"{tone}\"]");
        if piece.contains(&pat_single) || piece.contains(&pat_double) {
            return true;
        }
    }
    false
}

/// Find the byte offset (relative to `body`) of a `var(--primary)`
/// reference, OR `None`. Excludes `--primary-foreground`,
/// `--primary-…` etc. by exact-name matching. Skips CSS comments.
fn find_primary_ref(body: &str) -> Option<usize> {
    let bytes = body.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'/' && bytes.get(i + 1) == Some(&b'*') {
            if let Some(end_rel) = find_subslice(&bytes[i + 2..], b"*/") {
                i += 2 + end_rel + 2;
                continue;
            }
            return None;
        }
        if matches_at(bytes, i, b"var(") {
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
                let name = &body[name_start..n];
                let next = bytes.get(n).copied();
                let is_exact = matches!(next, Some(b')' | b',' | b' ' | b'\t' | b'\n' | b'\r'));
                if is_exact && name == "primary" {
                    return Some(i);
                }
            }
            // Resume inside the var() so a nested var() is found on
            // the next iteration (the inner var() may itself be
            // primary).
            i = start;
            continue;
        }
        i += 1;
    }
    None
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

/// An innermost declaration block: `selector { body }` with no nested
/// `{`. The selector is the slice from the end of the previous `;`/
/// `}` (or the start of the block) up to the matching `{`.
struct RuleBlock<'a> {
    /// Byte offset of `body` into the style-block content.
    body_offset: usize,
    /// The selector slice (trimmed). Stored as a string because the
    /// rule needs to inspect the selector's text (to detect status
    /// tones), not just its offset.
    selector: &'a str,
    body: &'a str,
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

/// Find every innermost `{...}` (a declaration block — body contains
/// no nested brace). Skips at-rule wrappers (`@media { ... }`) by
/// descending into them so the inner rule blocks are seen. Skips CSS
/// comments so a commented-out example rule does not fire.
fn scan_rule_blocks(content: &str) -> Vec<RuleBlock<'_>> {
    let mut out = Vec::new();
    let bytes = content.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // Skip CSS comments. Without this, a commented-out
        // `:host([tone='destructive']) { background: var(--primary); }`
        // gets parsed as a real rule and fires a finding.
        if bytes[i] == b'/' && bytes.get(i + 1) == Some(&b'*') {
            if let Some(end_rel) = find_subslice(&bytes[i + 2..], b"*/") {
                i += 2 + end_rel + 2;
                continue;
            }
            return out;
        }
        if bytes[i] == b'{' {
            // Walk to the matching `}`, tracking nested braces.
            let mut depth = 0i32;
            let mut j = i;
            let mut nested = false;
            while j < bytes.len() {
                match bytes[j] {
                    b'{' => {
                        depth += 1;
                        if depth > 1 {
                            nested = true;
                        }
                    }
                    b'}' => {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    _ => {}
                }
                j += 1;
            }
            if nested {
                i += 1;
            } else {
                if j < bytes.len() {
                    // The selector is the slice from the previous
                    // boundary (`;`, `}`, or start of block) up to
                    // the matching `{`.
                    let selector_start = previous_selector_boundary(content, i);
                    out.push(RuleBlock {
                        body_offset: i + 1,
                        selector: content[selector_start..i].trim(),
                        body: &content[i + 1..j],
                    });
                }
                i = j + 1;
            }
            continue;
        }
        i += 1;
    }
    out
}

/// Index of the byte *after* the previous selector boundary (`;`,
/// `}`, or 0) preceding `open_brace`. The selector slice for the rule
/// opened by `open_brace` starts at this index and ends at
/// `open_brace` (exclusive).
fn previous_selector_boundary(content: &str, open_brace: usize) -> usize {
    let bytes = content.as_bytes();
    let mut k = open_brace;
    while k > 0 {
        let b = bytes[k - 1];
        if matches!(b, b';' | b'}') {
            return k;
        }
        k -= 1;
    }
    0
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

    // --- Status tones fire ---

    #[test]
    fn destructive_with_primary_fires() {
        let src = "<style>:host([tone='destructive']) { background: var(--primary); }</style>";
        let f = run(src);
        assert_eq!(f.len(), 1, "{f:?}");
        assert_eq!(f[0].rule, RULE_ID);
        assert_eq!(f[0].severity, Severity::Warn);
        assert!(f[0].message.contains("var(--primary)"), "{}", f[0].message);
        assert!(
            f[0].message.contains("destructive"),
            "{}",
            f[0].message
        );
        assert!(f[0].suggest.as_deref().unwrap().contains("tone-X"));
    }

    #[test]
    fn warning_with_primary_fires() {
        let src = "<style>:host([tone='warning']) { border-color: var(--primary); }</style>";
        let f = run(src);
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(f[0].message.contains("warning"), "{}", f[0].message);
    }

    #[test]
    fn success_with_primary_fires() {
        let src = "<style>.x[tone='success'] { fill: var(--primary); }</style>";
        let f = run(src);
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(f[0].message.contains("success"), "{}", f[0].message);
    }

    #[test]
    fn info_with_primary_fires() {
        let src = "<style>:host([tone='info']) { color: var(--primary); }</style>";
        let f = run(src);
        assert_eq!(f.len(), 1, "{f:?}");
    }

    #[test]
    fn accent_with_primary_fires() {
        let src = "<style>:host([tone='accent']) { background: var(--primary); }</style>";
        let f = run(src);
        assert_eq!(f.len(), 1, "{f:?}");
    }

    #[test]
    fn invalid_with_primary_fires() {
        // [invalid] is a status signal even without an explicit tone.
        let src = "<style>:host([invalid]) { background: var(--primary); }</style>";
        let f = run(src);
        assert_eq!(f.len(), 1, "{f:?}");
    }

    #[test]
    fn list_selector_with_status_piece_fires() {
        // The list mixes a status piece with another; the rule fires
        // because the body is reachable from the status selector.
        let src = "<style>:host([tone='destructive']), :host([invalid]) { border-color: var(--primary); }</style>";
        let f = run(src);
        assert_eq!(f.len(), 1, "{f:?}");
    }

    // --- Brand / default / non-status tones do NOT fire ---

    #[test]
    fn primary_tone_with_primary_does_not_fire() {
        // Brand tone: --primary is the correct token here.
        let src = "<style>:host([tone='primary']) { background: var(--primary); }</style>";
        assert!(run(src).is_empty(), "{:?}", run(src));
    }

    #[test]
    fn no_tone_default_with_primary_does_not_fire() {
        // Default block: --primary is the correct default.
        let src = "<style>:host(:not([tone])) { background: var(--primary); }</style>";
        assert!(run(src).is_empty(), "{:?}", run(src));
    }

    #[test]
    fn default_tone_with_primary_does_not_fire() {
        // `[tone='default']` — the default-tone selector.
        let src = "<style>:host([tone='default']) { background: var(--primary); }</style>";
        assert!(run(src).is_empty(), "{:?}", run(src));
    }

    #[test]
    fn muted_tone_with_primary_does_not_fire() {
        // `muted` is a non-status tint; --primary there is fine.
        let src = "<style>:host([tone='muted']) { background: var(--primary); }</style>";
        assert!(run(src).is_empty(), "{:?}", run(src));
    }

    // --- Paired / non-bare primary tokens do NOT fire ---

    #[test]
    fn primary_foreground_does_not_fire() {
        // `--primary-foreground` is the paired text token; reading
        // it from a status block is the correct way to put primary-
        // coloured text on a status surface.
        let src = "<style>:host([tone='destructive']) { color: var(--primary-foreground); }</style>";
        assert!(run(src).is_empty(), "{:?}", run(src));
    }

    #[test]
    fn primary_modifier_does_not_fire() {
        // Any `--primary-…` paired token (e.g. a future
        // `--primary-hover`) is a different slot.
        let src = "<style>:host([tone='success']) { background: var(--primary-hover); }</style>";
        assert!(run(src).is_empty(), "{:?}", run(src));
    }

    // --- Status tone block with NO primary use does NOT fire ---

    #[test]
    fn status_block_using_destructive_does_not_fire() {
        // Status block, but it uses the right token. No finding.
        let src = "<style>:host([tone='destructive']) { background: var(--destructive); }</style>";
        assert!(run(src).is_empty(), "{:?}", run(src));
    }

    #[test]
    fn status_block_using_tone_ramp_does_not_fire() {
        // Status block using a tone ramp — correct.
        let src = "<style>:host([tone='warning']) { background: var(--tone-warning-bg); }</style>";
        assert!(run(src).is_empty(), "{:?}", run(src));
    }

    // --- Scoping / robustness ---

    #[test]
    fn no_style_block_no_findings() {
        let src = "<div>:host([tone='destructive']) { background: var(--primary); }</div>";
        assert!(run(src).is_empty(), "{:?}", run(src));
    }

    #[test]
    fn commented_status_block_is_skipped() {
        // The status-tone comment is just text; the only real
        // declaration is the comment. No body, no finding.
        let src = "<style>/* :host([tone='destructive']) { background: var(--primary); } */</style>";
        assert!(run(src).is_empty(), "{:?}", run(src));
    }

    #[test]
    fn media_wrapper_status_block_fires() {
        // @media wraps the real status block; the rule descends.
        let src = "<style>@media (min-width: 600px) { :host([tone='destructive']) { background: var(--primary); } }</style>";
        let f = run(src);
        assert_eq!(f.len(), 1, "{f:?}");
    }

    #[test]
    fn nested_at_rule_status_block_fires() {
        // @supports is an at-rule wrapper too; the rule descends.
        let src = "<style>@supports (color: oklch(0 0 0)) { :host([tone='info']) { fill: var(--primary); } }</style>";
        let f = run(src);
        assert_eq!(f.len(), 1, "{f:?}");
    }

    #[test]
    fn clean_brand_block_has_no_findings() {
        let src = r#"<style>
            :host {
                color: var(--foreground);
                background: var(--surface-1);
                border: 1px solid var(--border);
            }
            :host([tone='primary']) { background: var(--primary); }
            :host([tone='destructive']) { background: var(--tone-destructive-bg); }
        </style>"#;
        assert!(run(src).is_empty(), "{:?}", run(src));
    }

    #[test]
    fn position_points_at_var_call() {
        let src = "<style>\n  :host([tone='destructive']) { background: var(--primary); }\n</style>";
        let f = run(src);
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].line, 2);
    }

    #[test]
    fn multiple_status_blocks_each_fire() {
        let src = "<style>:host([tone='destructive']) { background: var(--primary); } :host([tone='warning']) { border-color: var(--primary); }</style>";
        let f = run(src);
        assert_eq!(f.len(), 2, "{f:?}");
    }
}
