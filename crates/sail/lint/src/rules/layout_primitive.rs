//! `sail-layout-primitive` — a `<style>` rule that hand-rolls a flex layout the
//! design system already provides as a primitive should use that primitive.
//!
//! The every-layout keywords exist (`<column>`, `<row>`,
//! `<sail-grid>`) but are under-adopted: most components re-derive
//! `display:flex; flex-direction:column; gap:…` by hand, which is where layout
//! drift and inconsistency live. This rule is the adoption driver — it flags the
//! two unambiguous shapes and teaches the primitive in the suggest text.
//!
//! Deliberately conservative to stay low-noise:
//! - **Stack**: `display:flex` + `flex-direction:column` + a `gap` → `<column>`.
//! - **Cluster**: `display:flex` + `flex-wrap:wrap` + a `gap`, row direction →
//!   `<row>`.
//! - **Skips** any block with `justify-content` — the primitives model the
//!   cross-axis (`align`) and the gap, not main-axis distribution, so a block
//!   that distributes is not a clean swap and is left alone.
//!
//! Severity `Warn` (advisory): this is a "prefer" rule, not a correctness gate.

use std::path::Path;

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sail-layout-primitive";

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    for block in scan_style_blocks(source) {
        for rule in scan_rule_blocks(block.content) {
            // Exempt the custom element's own `:host` rule. `:host` IS the
            // host element — there is no `<column>` to swap it with; the
            // host's own `display: inline-flex` + `flex-direction: column` +
            // `gap` is the host's root layout, not a hand-rolled sub-stack.
            // Both bare `:host` and functional `:host([attr=…])` count.
            if block_starts_with_host(&block.content[..rule.body_offset - 1]) {
                continue;
            }
            let mut display = None;
            let mut direction = None;
            let mut wrap = None;
            let mut has_gap = false;
            let mut has_justify = false;
            let mut display_off = rule.body_offset;

            for decl in scan_declarations(rule.body) {
                match decl.property {
                    "display" => {
                        display = Some(decl.value.trim());
                        display_off = rule.body_offset + decl.prop_offset;
                    }
                    "flex-direction" => direction = Some(decl.value.trim()),
                    "flex-flow" => {
                        // shorthand: `column wrap` etc.
                        let v = decl.value.trim();
                        if v.contains("column") {
                            direction = Some("column");
                        }
                        if v.contains("wrap") {
                            wrap = Some("wrap");
                        }
                    }
                    "flex-wrap" => wrap = Some(decl.value.trim()),
                    "gap" | "row-gap" | "column-gap" => has_gap = true,
                    "justify-content" => has_justify = true,
                    _ => {}
                }
            }

            let is_flex = matches!(display, Some("flex") | Some("inline-flex"));
            if !is_flex || !has_gap || has_justify {
                continue;
            }

            let (line, col) = byte_offset_to_line_col(source, block.content_offset + display_off);
            if direction == Some("column") {
                findings.push(mk(
                    path,
                    line,
                    col,
                    "this rule hand-rolls a vertical flex stack (`display:flex` + \
                     `flex-direction:column` + `gap`) — use the `<column>` \
                     layout primitive instead",
                    "Replace the element with `<column gap=N>` (gap 1..8 → \
                     var(--space-N)); use the `align` attr for align-items. Drop \
                     the hand-rolled flex rule.",
                ));
            } else if wrap == Some("wrap") {
                findings.push(mk(
                    path,
                    line,
                    col,
                    "this rule hand-rolls a horizontal wrapping flex row \
                     (`display:flex` + `flex-wrap:wrap` + `gap`) — use the \
                     `<row>` layout keyword instead",
                    "Replace the element with `<row gap=N>` (gap 1..8 → \
                     var(--space-N)). Drop the hand-rolled flex rule.",
                ));
            }
        }
    }
    findings
}

fn mk(path: &Path, line: usize, col: usize, message: &str, suggest: &str) -> Finding {
    Finding {
        file: path.to_path_buf(),
        line,
        col,
        rule: RULE_ID,
        severity: Severity::Warn,
        message: message.to_string(),
        suggest: Some(suggest.to_string()),
    }
}


/// True when the rule whose body starts at position `body_offset` (i.e. the
/// rule whose selector ends at `body_offset - 1`, the `{`) is a `:host`
/// rule — bare `:host` or functional `:host(...)`. The host's own layout
/// is the host element itself, not a hand-rolled sub-stack; flagging it
/// as a `<column>` swap is meaningless.
///
/// We work from `content[..body_offset]` — the slice that contains the
/// selector. The selector is everything from the start of the content (or
/// after a previous `;` / `}`) up to (but not including) the `{` at
/// `body_offset`. The slice passed in is the prefix; we walk back to the
/// most recent rule boundary and check whether the trimmed selector begins
/// with `:host`.
fn block_starts_with_host(prefix: &str) -> bool {
    let bytes = prefix.as_bytes();
    let mut k = bytes.len();
    while k > 0 {
        let prev = bytes[k - 1];
        if prev == b';' || prev == b'}' {
            break;
        }
        k -= 1;
    }
    let selector = prefix[k..].trim();
    selector == ":host" || selector.starts_with(":host(")
}
struct StyleBlock<'a> {
    content_offset: usize,
    content: &'a str,
}

/// An innermost declaration block: `selector { body }` with no nested `{`.
struct RuleBlock<'a> {
    /// Byte offset of `body` into the style-block content.
    body_offset: usize,
    body: &'a str,
}

struct Decl<'a> {
    property: &'a str,
    /// Byte offset of the property name into the rule body.
    prop_offset: usize,
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

/// Find every innermost `{...}` (a declaration block — body contains no `{`).
/// Skips at-rule wrappers (`@media { ... }`) by only emitting blocks whose body
/// has no nested brace.
fn scan_rule_blocks(content: &str) -> Vec<RuleBlock<'_>> {
    let mut out = Vec::new();
    let bytes = content.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'{' {
            // Is this a leaf declaration block (no nested `{`), or an at-rule
            // wrapper (`@media { ... }`)? Peek for a nested brace before the
            // matching close.
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
                // Descend into the wrapper so its inner rule blocks are seen.
                i += 1;
            } else {
                if j < bytes.len() {
                    out.push(RuleBlock {
                        body_offset: i + 1,
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

fn scan_declarations(content: &str) -> Vec<Decl<'_>> {
    let mut out = Vec::new();
    let bytes = content.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        while i < bytes.len() && (bytes[i].is_ascii_whitespace() || matches!(bytes[i], b';')) {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
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
            while i < bytes.len() && bytes[i] != b';' {
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
                b';' if depth <= 0 => break,
                _ => {}
            }
            v += 1;
        }
        out.push(Decl {
            property: name,
            prop_offset: name_start,
            value: content[value_start..v].trim_end(),
        });
        i = v;
    }
    out
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

    #[test]
    fn vertical_stack_warns() {
        let f = run("<style>.x { display: flex; flex-direction: column; gap: var(--space-3); }</style>");
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(f[0].message.contains("column"));
    }

    #[test]
    fn wrapping_cluster_warns() {
        let f = run("<style>.x { display: flex; flex-wrap: wrap; gap: var(--space-2); }</style>");
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(f[0].message.contains("row"));
    }

    #[test]
    fn flex_with_justify_is_left_alone() {
        // distributes on the main axis — not a clean primitive swap.
        let f = run("<style>.x { display: flex; flex-direction: column; gap: var(--space-3); justify-content: space-between; }</style>");
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn plain_flex_without_gap_passes() {
        let f = run("<style>.x { display: flex; align-items: center; }</style>");
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn row_flex_with_gap_no_wrap_passes() {
        // a non-wrapping row with a gap isn't unambiguously a cluster.
        let f = run("<style>.x { display: flex; gap: var(--space-2); }</style>");
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn flex_flow_shorthand_column_warns() {
        let f = run("<style>.x { display: flex; flex-flow: column nowrap; gap: var(--space-2); }</style>");
        assert_eq!(f.len(), 1);
        assert!(f[0].message.contains("column"));
    }

    #[test]
    fn raw_px_gap_still_recognised_as_stack() {
        // gap value doesn't matter for the shape (space_literal handles the px).
        let f = run("<style>.x { display: flex; flex-direction: column; gap: 12px; }</style>");
        assert_eq!(f.len(), 1);
    }

    #[test]
    fn media_wrapper_not_treated_as_a_decl_block() {
        // The @media wrapper has a nested `{` so it's skipped; the inner rule
        // is the one evaluated.
        let f = run("<style>@media (min-width: 600px) { .x { display: flex; flex-direction: column; gap: var(--space-3); } }</style>");
        assert_eq!(f.len(), 1, "inner rule should warn: {f:?}");
    }

    #[test]
    fn non_flex_grid_block_passes() {
        let f = run("<style>.x { display: grid; gap: var(--space-3); }</style>");
        assert!(f.is_empty(), "grid is out of scope for v1: {f:?}");
    }
    #[test]
    fn host_bare_selector_passes() {
        // The custom element's own `:host` is the host itself — not a
        // hand-rolled sub-stack. Pattern from `sail-input.sail:85`.
        let f = run("<style>:host { display: inline-flex; flex-direction: column; gap: var(--space-1); }</style>");
        assert!(f.is_empty(), "host should be exempt: {f:?}");
    }

    #[test]
    fn host_functional_selector_passes() {
        // `:host([tone='primary'])` is still a host-context rule.
        let f = run("<style>:host([tone='primary']) { display: flex; flex-direction: column; gap: var(--space-2); }</style>");
        assert!(f.is_empty(), "host([attr]) should be exempt: {f:?}");
    }

    #[test]
    fn non_host_after_host_still_warns() {
        // The :host rule is exempt, but a sibling `.wrap` rule with the
        // same hand-rolled flex shape is NOT a host rule — should warn.
        let f = run("<style>:host { display: inline-flex; flex-direction: column; gap: var(--space-1); } .wrap { display: flex; flex-direction: column; gap: var(--space-2); }</style>");
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(f[0].message.contains("column"));
    }

    #[test]
    fn host_with_space_then_other_selector_warns() {
        // `:host .x` — the descendant is the hand-rolled stack, not the
        // host. The selector starts with `:host` but is not a host rule.
        let f = run("<style>:host .x { display: flex; flex-direction: column; gap: var(--space-2); }</style>");
        assert_eq!(f.len(), 1, "{f:?}");
    }
}
