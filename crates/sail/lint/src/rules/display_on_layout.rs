//! `sail-display-on-layout` — do not put `style.display(…)` on layout keywords.
//!
//! Layout bases (`.sail-col` / `.sail-row` / `.sail-grid` / …) own
//! `display:flex|grid`. A sibling atomic class like `s-displaynone` loses the
//! cascade fight in product CSS (same specificity, base often later or equal),
//! so `display(none)` on `<column>` / `<row>` / … is a silent production
//! footgun. Authors must use a plain host element for show/hide and nest the
//! layout keyword inside.
//!
//! Flags:
//! - direct `style={style.display(…)}` and mid-chain `.display(` / `when({…})`
//! - **simple Style const refs** whose defining expression includes `display(`:
//!   `const hide = style.display(none)` then `<column style={hide}>`
//!
//! **Known gaps (not yet flagged):** multi-hop refs (`const a = hide`), list
//! spreads, `match`/`compound` wrappers that only nest a display-bearing const,
//! and post-lower class inspection (`s-displaynone` on a layout base). Prefer
//! plain host + nest until those land.

use std::collections::HashSet;
use std::path::Path;

use crate::report::{Finding, Severity};
#[cfg(test)]
use crate::rules::layout::TAGS;
use crate::rules::layout::{self};

pub const RULE_ID: &str = "sail-display-on-layout";

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    // Scope consts with display(…) — used when style={name} is a bare ref.
    let display_consts = display_bearing_const_names(source);

    // Tag scanning is SHARED with the layout rule (`layout::scan_open_tags`):
    // one scanner, one attr parser, one stray-punctuation guard — this rule
    // only interprets the `sx` attr. Controller/scope spans are excluded so
    // TS noise never reads as an open tag.
    let excluded = excluded_spans(source);
    let mut findings = Vec::new();
    for t in layout::scan_open_tags(source) {
        if excluded.iter().any(|(s, e)| t.start >= *s && t.start < *e) {
            continue;
        }
        let Some((_, sx)) = t.attrs.iter().find(|(n, _)| n == "style") else {
            continue;
        };
        let via_inline = sx_contains_display(sx);
        let via_const = sx_bare_const_ref(sx)
            .map(|name| display_consts.contains(name.as_str()))
            .unwrap_or(false);
        if !via_inline && !via_const {
            continue;
        }
        let tag = t.tag.as_str();
        let (line, col) = layout::offset_to_line_col(source, t.start);
        findings.push(Finding {
            file: path.to_path_buf(),
            line,
            col,
            rule: RULE_ID,
            severity: Severity::Error,
            message: format!(
                "`style.display(…)` on <{tag}> is rejected — layout bases \
                 (.{base}) own `display:flex|grid` and beat \
                 `.s-displaynone` / other display atoms. Use a plain host \
                 element for show/hide and nest <{tag}> inside.",
                base = layout_base_class(tag),
            ),
            suggest: Some(format!(
                "Replace `<{tag} style({{ box: display(none).… }})>` (or a Style \
                 const that sets display) with \
                 `<div style({{ box: display(none) }})><{tag} …>…</{tag}></div>` \
                 (or hide the host with when/match)."
            )),
        });
    }
    findings
}

/// `<controller>…</controller>` / `<scope>…</scope>` spans — excluded from tag
/// scanning so TS noise never false-positives as an open tag.
fn excluded_spans(source: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    for (open, close) in [("<controller", "</controller>"), ("<scope", "</scope>")] {
        let mut from = 0;
        while let Some(s) = source[from..].find(open) {
            let start = from + s;
            match source[start..].find(close) {
                Some(e) => {
                    let end = start + e + close.len();
                    out.push((start, end));
                    from = end;
                }
                None => break,
            }
        }
    }
    out
}

/// Names of `const name = …` whose initializer text includes `display(`.
fn display_bearing_const_names(source: &str) -> HashSet<String> {
    let mut out = HashSet::new();
    let bytes = source.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // Never slice mid UTF-8 char (skill examples use `…`, etc.).
        if (bytes[i] & 0xC0) == 0x80 {
            i += 1;
            continue;
        }
        if source[i..].starts_with("<!--") {
            match source[i..].find("-->") {
                Some(end) => {
                    i += end + 3;
                    continue;
                }
                None => break,
            }
        }
        if source[i..].starts_with("<controller") {
            if let Some(end) = source[i..].find("</controller>") {
                i += end + "</controller>".len();
                continue;
            }
        }
        // `const name =`
        if source[i..].starts_with("const ") && (i == 0 || !bytes[i - 1].is_ascii_alphanumeric()) {
            let after = i + "const ".len();
            let rest = &source[after..];
            let name_end = rest
                .find(|c: char| !c.is_ascii_alphanumeric() && c != '_' && c != '$')
                .unwrap_or(rest.len());
            if name_end > 0 {
                let name = &rest[..name_end];
                let mut j = after + name_end;
                while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                    j += 1;
                }
                if bytes.get(j) == Some(&b'=') {
                    j += 1;
                    // Initializer: until newline (simple product style) or `</scope>`.
                    let init_start = j;
                    while j < bytes.len() && bytes[j] != b'\n' {
                        if (bytes[j] & 0xC0) != 0x80 && source[j..].starts_with("</scope>") {
                            break;
                        }
                        j += 1;
                    }
                    let init = &source[init_start..j];
                    if sx_contains_display(init) {
                        out.insert(name.to_string());
                    }
                }
            }
        }
        i += 1;
        while i < bytes.len() && (bytes[i] & 0xC0) == 0x80 {
            i += 1;
        }
    }
    out
}

/// `style={hide}` / `style={ hide }` → `Some("hide")`. Chains/calls → None.
fn sx_bare_const_ref(sx: &str) -> Option<String> {
    let mut s = sx.trim();
    if let Some(inner) = s.strip_prefix('{').and_then(|t| t.strip_suffix('}')) {
        s = inner.trim();
    }
    if s.is_empty() {
        return None;
    }
    // Reject anything that is not a single identifier.
    if !s
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
    {
        return None;
    }
    if s.chars().next()?.is_ascii_digit() {
        return None;
    }
    Some(s.to_string())
}

fn layout_base_class(tag: &str) -> &'static str {
    match tag {
        "column" => "sail-col",
        "row" => "sail-row",
        "grid" => "sail-grid",
        "center" => "sail-center",
        "stack" => "sail-stackz",
        _ => "sail-*",
    }
}

/// True when an `sx=` expression (or Style chain) applies `display(…)`.
/// UTF-8-safe: advances by char indices so multi-byte text (e.g. `…` in a
/// trailing comment) never panics on a mid-character slice.
fn sx_contains_display(sx: &str) -> bool {
    // Mid-chain or leading: `.display(` / `style.display(`
    let bytes = sx.as_bytes();
    for (i, _) in sx.char_indices() {
        if sx[i..].starts_with("display") {
            let after = i + "display".len();
            // word boundary before (start, or non-ident)
            let before_ok =
                i == 0 || (!bytes[i - 1].is_ascii_alphanumeric() && bytes[i - 1] != b'_');
            if before_ok {
                // skip whitespace then require '('
                let mut j = after;
                while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                    j += 1;
                }
                if j < bytes.len() && bytes[j] == b'(' {
                    return true;
                }
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(src: &str) -> Vec<Finding> {
        check(Path::new("t.sail"), src)
    }

    #[test]
    fn flags_display_none_on_column() {
        let f = run(r#"<column style={style.display(none).pad(4)}>x</column>"#);
        assert_eq!(f.len(), 1, "{f:?}");
        assert_eq!(f[0].rule, RULE_ID);
        assert!(f[0].message.contains("sail-col"), "{}", f[0].message);
        assert!(f[0].message.contains("display"), "{}", f[0].message);
    }

    #[test]
    fn flags_when_display_on_row() {
        let f = run(r#"<row style={style.pad(2).when({ tablet: style.display(none) })}>x</row>"#);
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(
            f[0].message.contains("<row>") || f[0].message.contains("row"),
            "{}",
            f[0].message
        );
    }

    #[test]
    fn allows_display_on_plain_div() {
        let f = run(r#"<div style={style.display(none)}><column>x</column></div>"#);
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn allows_layout_without_display() {
        let f = run(r#"<column style={style.pad(4).bg(surface-2)}>x</column>"#);
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn allows_display_const_on_plain_div() {
        // Const may set display; footgun is the layout host class only.
        let f = run(r#"
            <scope>
              const rail = style.display(none).pad(4)
            </scope>
            <div style={rail}>x</div>
            "#);
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn flags_display_const_ref_on_column() {
        let f = run(r#"
            <scope>
              const hide = style.display(none)
            </scope>
            <column style={hide}>x</column>
            "#);
        assert_eq!(f.len(), 1, "{f:?}");
        assert_eq!(f[0].rule, RULE_ID);
    }

    #[test]
    fn allows_non_display_const_on_column() {
        let f = run(r#"
            <scope>
              const paint = style.pad(4).bg(surface-2)
            </scope>
            <column style={paint}>x</column>
            "#);
        assert!(f.is_empty(), "{f:?}");
    }

    /// Regression: multi-byte UTF-8 in a non-display const initializer must not
    /// panic `sx_contains_display` / MCP `sail_check` (ellipsis mid-byte slice).
    #[test]
    fn utf8_ellipsis_in_const_initializer_does_not_panic() {
        let f = run(r#"
            <scope>
              const paint = style.pad(4) // …
            </scope>
            <column style={paint}>x</column>
            "#);
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn utf8_ellipsis_in_sx_attr_does_not_panic() {
        // Comment-like multi-byte in an sx expression without display(.
        let f = run(r#"<column style={style.pad(4) /* … */}>x</column>"#);
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn flags_all_layout_keywords() {
        for tag in TAGS {
            let src = format!(r#"<{tag} style={{style.display(flex)}}>x</{tag}>"#);
            let f = run(&src);
            assert_eq!(f.len(), 1, "expected flag for <{tag}>: {f:?}");
        }
    }

    /// Regression: a self-closing layout tag leaves a trailing `/` in the
    /// attrs slice; `attr_value` must step past it instead of spinning on an
    /// empty attr name (hung `sail check` / compile past the 60s wall cap).
    #[test]
    fn self_closing_layout_tag_does_not_hang() {
        for tag in TAGS {
            let f = run(&format!("<{tag} style={{style.pad(2)}} />"));
            assert!(f.is_empty(), "<{tag}/>: {f:?}");
        }
        let f = run(r#"<column style={style.display(none)} />"#);
        assert_eq!(
            f.len(),
            1,
            "self-closing with display must still flag: {f:?}"
        );
    }
}
