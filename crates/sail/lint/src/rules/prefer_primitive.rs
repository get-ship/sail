//! `sail-prefer-primitive` — a component `<style>` block that lays out with
//! bespoke `display: grid` / `display: flex` should compose a layout
//! primitive instead.
//!
//! The language ships the layout keywords (`<grid>`, `<column>`, `<row>`)
//! precisely so that layout is declared once,
//! consistently, in one place — not re-derived by hand in every component's
//! `<style>`. Hand-rolled `display: grid` / `display: flex` is where layout
//! drift lives. This rule flags those two declarations and points at the
//! primitives.
//!
//! Scope and exemptions:
//! - **Primitives package is exempt.** Components under
//!   `packages/components/sail/primitives/**` are the layout primitives (and
//!   their peers); they legitimately implement flex/grid internally — there is
//!   no lower-level primitive to compose. Warning there would be circular.
//! - This is a **sibling** to `sail-layout-primitive`, not a replacement.
//!   `sail-layout-primitive` recognizes two *specific* clean-swap flex shapes
//!   (vertical stack, wrapping cluster) and teaches the exact primitive +
//!   attrs. This rule is the broad net: *any* `display: grid` / `display:
//!   flex` in a consumer component is ad-hoc layout worth a second look.
//!
//! **Severity: Warning** — advisory, never a build failure. It flags a
//! discipline preference, not a correctness bug; existing components that
//! hand-roll layout must keep building.

use std::path::{Component, Path};

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sail-prefer-primitive";

/// The ad-hoc-layout message. Points at all three layout keywords and the skill.
const MESSAGE: &str =
    "ad-hoc layout — prefer the layout keywords <grid>/<column>/<row> (see the sail skill)";

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    // The primitives package IS the layout substrate — flex/grid is legitimate
    // there. Exempt it wholesale so we don't warn on the very components the
    // message tells everyone else to compose.
    if is_primitives_pack(path) {
        return Vec::new();
    }

    let mut findings = Vec::new();
    for block in scan_style_blocks(source) {
        for rule in scan_rule_blocks(block.content) {
            for decl in scan_declarations(rule.body) {
                if decl.property != "display" {
                    continue;
                }
                // Match exactly `grid` / `flex` (the two layout displays the
                // primitives replace). `block`, `none`, `inline`, `contents`,
                // etc. are not layout containers and are left alone.
                let value = decl.value.trim();
                if value != "grid" && value != "flex" {
                    continue;
                }
                let off = block.content_offset + rule.body_offset + decl.prop_offset;
                let (line, col) = byte_offset_to_line_col(source, off);
                findings.push(Finding {
                    file: path.to_path_buf(),
                    line,
                    col,
                    rule: RULE_ID,
                    severity: Severity::Warn,
                    message: MESSAGE.to_string(),
                    suggest: Some(
                        "Compose a layout keyword instead of hand-rolling the container: \
                         `<grid>` for two-dimensional / content-aware grids, `<column>` \
                         for a vertical stack, `<row>` for a horizontal row (it wraps by \
                         default). Drop the `display: grid|flex` rule."
                            .to_string(),
                    ),
                });
            }
        }
    }
    findings
}

/// True when `path` lives under `packages/components/sail/primitives/`.
///
/// Matches the consecutive path-component window
/// `packages / components / sail / primitives` anywhere in the path, so it
/// works for absolute (`/home/.../packages/components/sail/primitives/...`) and
/// repo-relative (`packages/components/sail/primitives/...`) forms alike.
fn is_primitives_pack(path: &Path) -> bool {
    const WINDOW: [&str; 4] = ["packages", "components", "sail", "primitives"];
    let parts: Vec<&str> = path
        .components()
        .filter_map(|c| match c {
            Component::Normal(s) => s.to_str(),
            _ => None,
        })
        .collect();
    parts.windows(WINDOW.len()).any(|w| w == WINDOW)
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
/// Descends into at-rule wrappers (`@media { ... }`) so their inner rules are
/// still seen; a `display: grid` nested inside `@media` still counts.
fn scan_rule_blocks(content: &str) -> Vec<RuleBlock<'_>> {
    let mut out = Vec::new();
    let bytes = content.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'{' {
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

    /// A consumer component path (NOT under the primitives package).
    fn consumer() -> PathBuf {
        PathBuf::from("packages/components/sail/charts/src/sail-gauge/sail-gauge.sail")
    }

    /// A primitives-pack component path.
    fn primitive() -> PathBuf {
        PathBuf::from("packages/components/sail/primitives/src/sail-card/sail-card.sail")
    }

    // (a) A consumer component with `display: grid` in its <style> warns.
    #[test]
    fn consumer_display_grid_warns() {
        let src = "<style>.wrap { display: grid; gap: var(--space-3); }</style><div class=\"wrap\"></div>";
        let f = check(&consumer(), src);
        assert_eq!(f.len(), 1, "{f:?}");
        assert_eq!(f[0].rule, RULE_ID);
        assert_eq!(f[0].severity, Severity::Warn);
        assert_eq!(
            f[0].message,
            "ad-hoc layout — prefer the layout keywords <grid>/<column>/<row> (see the sail skill)"
        );
    }

    // (a′) `display: flex` in a consumer likewise warns.
    #[test]
    fn consumer_display_flex_warns() {
        let src = "<style>.row { display: flex; }</style><div class=\"row\"></div>";
        let f = check(&consumer(), src);
        assert_eq!(f.len(), 1, "{f:?}");
        assert_eq!(f[0].rule, RULE_ID);
        assert_eq!(f[0].severity, Severity::Warn);
    }

    // (b) The SAME source under the primitives package does NOT warn.
    #[test]
    fn primitives_pack_is_exempt() {
        let src = "<style>.wrap { display: grid; gap: var(--space-3); }</style><div class=\"wrap\"></div>";
        let f = check(&primitive(), src);
        assert!(f.is_empty(), "primitives package must be exempt: {f:?}");
    }

    // (b′) Exemption holds for an absolute path form too.
    #[test]
    fn primitives_pack_absolute_path_is_exempt() {
        let p = PathBuf::from(
            "/tmp/repo/packages/components/sail/primitives/src/sail-stack/sail-stack.sail",
        );
        let src = "<style>:host { display: flex; }</style><slot></slot>";
        let f = check(&p, src);
        assert!(
            f.is_empty(),
            "absolute primitives path must be exempt: {f:?}"
        );
    }

    // (c) A component that composes <sail-grid> (no ad-hoc layout CSS) is silent.
    #[test]
    fn composing_sail_grid_emits_nothing() {
        let src = "<style>.title { color: var(--foreground); }</style>\
                   <sail-grid gap=\"3\"><div>a</div><div>b</div></sail-grid>";
        let f = check(&consumer(), src);
        assert!(
            f.is_empty(),
            "composing a primitive is the goal — no warning: {f:?}"
        );
    }

    // A non-layout display (block/none) is never flagged.
    #[test]
    fn non_layout_display_passes() {
        let src = "<style>:host { display: block; } .x { display: none; }</style>";
        let f = check(&consumer(), src);
        assert!(f.is_empty(), "block/none are not layout containers: {f:?}");
    }

    // The finding points at the offending `display` declaration.
    #[test]
    fn finding_points_at_display_declaration() {
        let src = "<div></div>\n<style>\n  .wrap { display: grid; }\n</style>";
        let f = check(&consumer(), src);
        assert_eq!(f.len(), 1, "{f:?}");
        assert_eq!(f[0].line, 3, "display is on line 3");
        // col points at the `d` of `display`.
        let line = src.lines().nth(2).unwrap();
        let expected_col = line.find("display").unwrap() + 1;
        assert_eq!(f[0].col, expected_col, "col should land on `display`");
    }

    // A grid nested inside an @media wrapper is still flagged.
    #[test]
    fn display_grid_inside_media_warns() {
        let src = "<style>@media (min-width: 600px) { .wrap { display: grid; } }</style>";
        let f = check(&consumer(), src);
        assert_eq!(f.len(), 1, "inner rule should warn: {f:?}");
    }
}
