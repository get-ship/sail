//! `sail-prop-value` — static enum attribute values on registry elements
//! must be one of the values the element's stdlib `<scope>` props
//! declaration allows.
//!
//! Catches the silent-typo failure mode: `tone="danger"` on a primitive
//! whose tone enum is `default|primary|...|destructive`. Today the
//! attribute renders, no exception fires, and the styling silently
//! falls back to the default — exactly the slop the design system was
//! supposed to prevent.
//!
//! Severity is `Error`: `sail-compiler::check::css_token` promotes
//! every error-severity finding to a build failure. The rule was
//! introduced alongside a one-shot migration that cleared the prior
//! drift (~100 violations dominated by `sail-icon#size`); afterward
//! Error became safe to enable without breaking compile.
//!
//! Scope and limits:
//! - Only static string-literal attribute values are validated.
//!   Dynamic bindings (`tone={…}`), boolean-style attributes (no `=`),
//!   and attribute-binding syntaxes (`attr:tone=…`, `bind:value=…`)
//!   are skipped.
//! - Only elements registered in `primitives.json` are validated.
//!   Unknown tags (native HTML, user-authored web components, or new
//!   stdlib elements not yet regenerated into the registry) emit no
//!   findings.
//! - Only props whose registry entry has a `values` set are validated.
//!   Bool/string/number-typed props are not type-checked here.
//!
//! Suggestions use Levenshtein with a small cap so common typos
//! (`danger` → `destructive`) are reported even when the distance is
//! not tiny — destructive vs danger is distance 6, but the warm-misuse
//! case is the prime motivator for this rule, so we accept up to half
//! the candidate's length.

use std::path::Path;

use crate::prop_registry::registry;
use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sail-prop-value";

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let reg = registry();
    let mut findings = Vec::new();
    for tag in scan_sui_tags(source) {
        let Some(props) = reg.get(tag.name) else {
            continue;
        };
        for attr in tag.attributes {
            let Some(rule) = props.get(attr.name) else {
                continue;
            };
            let Some(allowed) = rule.values.as_ref() else {
                continue;
            };
            let value = match attr.value {
                AttrValue::Literal(v) => v,
                // Dynamic, bound, or flag attributes are unverifiable
                // by this rule.
                AttrValue::Dynamic | AttrValue::Flag => continue,
            };
            if allowed.iter().any(|v| v == value) {
                continue;
            }
            let suggestion = nearest_value(value, allowed);
            let mut message = format!(
                "unknown value `{value}` for `{tag}#{attr}`; allowed: {allowed_list}",
                tag = tag.name,
                attr = attr.name,
                allowed_list = allowed.join(", "),
            );
            if let Some(s) = &suggestion {
                message.push_str(&format!("; did you mean `{s}`?"));
            }
            let (line, col) = byte_offset_to_line_col(source, attr.value_offset);
            findings.push(Finding {
                file: path.to_path_buf(),
                line,
                col,
                rule: RULE_ID,
                severity: Severity::Error,
                message,
                suggest: Some(format!(
                    "Replace with one of: {}. If a real new variant is \
                     required, add it to the prop's union in the stdlib \
                     `<scope>` declaration, then regenerate the registry \
                     with `cargo run -p sail-compiler -- prop-registry`.",
                    allowed.join(", ")
                )),
            });
        }
    }
    findings
}

/// One opening tag and its parsed attributes.
struct SuiTag<'a> {
    name: &'a str,
    attributes: Vec<Attribute<'a>>,
}

struct Attribute<'a> {
    name: &'a str,
    value: AttrValue<'a>,
    /// Byte offset of the *value start* (after the opening quote, if
    /// any) into the source. Reported as the finding's column so the
    /// caret points at the bad word.
    value_offset: usize,
}

enum AttrValue<'a> {
    Literal(&'a str),
    /// `attr={expression}` or `bind:attr=…` or `attr:foo=…` — anything
    /// that isn't a static string the rule can validate.
    Dynamic,
    /// Bare attribute with no `=`. Used as boolean flag.
    Flag,
}

/// Walk every element opening tag and parse its attributes. HTML
/// comments (`<!-- ... -->`) are skipped — primitives frequently embed
/// example tags in their leading docblock and we must not treat those
/// as real call sites.
fn scan_sui_tags(source: &str) -> Vec<SuiTag<'_>> {
    let mut out = Vec::new();
    let bytes = source.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // Skip past HTML comments entirely.
        if bytes[i] == b'<' && bytes.get(i + 1..i + 4) == Some(b"!--") {
            if let Some(end_rel) = find_subslice(&bytes[i + 4..], b"-->") {
                i += 4 + end_rel + 3;
                continue;
            }
            // Unterminated comment — bail rather than scan to EOF.
            break;
        }
        // Any element opening tag. The registry lookup is the filter:
        // stdlib tags are bare (`navbar`, `avatar`), and a tag the registry
        // doesn't know is skipped by the caller — so scanning every tag
        // costs one BTreeMap miss for `<p>` and friends.
        if !(bytes[i] == b'<' && bytes.get(i + 1).is_some_and(|b| b.is_ascii_lowercase())) {
            i += 1;
            continue;
        }
        // Read tag name (until whitespace, `>`, or `/`).
        let name_start = i + 1;
        let mut j = name_start;
        while j < bytes.len() && is_tag_name_byte(bytes[j]) {
            j += 1;
        }
        if j == name_start {
            i += 1;
            continue;
        }
        let name = &source[name_start..j];
        // Find tag end.
        let Some(end_rel) = find_byte(&bytes[j..], b'>') else {
            break;
        };
        let tag_end = j + end_rel;
        let inner_start = j;
        let inner = &source[inner_start..tag_end];
        let attributes = parse_open_tag_attributes(inner_start, inner);
        out.push(SuiTag { name, attributes });
        i = tag_end + 1;
    }
    out
}

/// Parse attributes from the interior of an opening tag (between the
/// tag name and the closing `>`). Returns offsets relative to the
/// outer `source` so findings can point at the original column.
fn parse_open_tag_attributes(inner_offset: usize, inner: &str) -> Vec<Attribute<'_>> {
    let mut out = Vec::new();
    let bytes = inner.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        if bytes[i] == b'/' {
            break;
        }
        let name_start = i;
        while i < bytes.len() && is_attr_name_byte(bytes[i]) {
            i += 1;
        }
        if i == name_start {
            i += 1;
            continue;
        }
        let name = &inner[name_start..i];
        // Bound-style names (`bind:value`, `on:click`, `attr:foo`,
        // `class:bar`, `style:baz`) are framework-binding sugar.
        // Skip — this rule only validates plain HTML attribute values.
        let skip_name = name.contains(':');
        if i >= bytes.len() || bytes[i] != b'=' {
            if !skip_name {
                out.push(Attribute {
                    name,
                    value: AttrValue::Flag,
                    value_offset: inner_offset + name_start,
                });
            }
            continue;
        }
        i += 1; // consume '='
        // Brace-bound: `attr={…}`.
        if i < bytes.len() && bytes[i] == b'{' {
            let depth_start = i;
            let mut depth = 0usize;
            while i < bytes.len() {
                match bytes[i] {
                    b'{' => depth += 1,
                    b'}' => {
                        depth -= 1;
                        if depth == 0 {
                            i += 1;
                            break;
                        }
                    }
                    _ => {}
                }
                i += 1;
            }
            if !skip_name {
                out.push(Attribute {
                    name,
                    value: AttrValue::Dynamic,
                    value_offset: inner_offset + depth_start,
                });
            }
            continue;
        }
        // Quoted literal.
        if i < bytes.len() && (bytes[i] == b'"' || bytes[i] == b'\'') {
            let quote = bytes[i];
            i += 1;
            let v_start = i;
            while i < bytes.len() && bytes[i] != quote {
                i += 1;
            }
            let v = &inner[v_start..i];
            if i < bytes.len() {
                i += 1;
            }
            if !skip_name {
                // Mustache-interpolated literals (`tone="{x()}"`) are
                // dynamic — only purely static strings are validated.
                if v.contains('{') {
                    out.push(Attribute {
                        name,
                        value: AttrValue::Dynamic,
                        value_offset: inner_offset + v_start,
                    });
                } else {
                    out.push(Attribute {
                        name,
                        value: AttrValue::Literal(v),
                        value_offset: inner_offset + v_start,
                    });
                }
            }
            continue;
        }
        // Unquoted literal.
        let v_start = i;
        while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'/' {
            i += 1;
        }
        let v = &inner[v_start..i];
        if !skip_name {
            out.push(Attribute {
                name,
                value: AttrValue::Literal(v),
                value_offset: inner_offset + v_start,
            });
        }
    }
    out
}

fn is_tag_name_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-'
}

fn is_attr_name_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b':'
}

fn find_byte(haystack: &[u8], needle: u8) -> Option<usize> {
    haystack.iter().position(|&b| b == needle)
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    let last = haystack.len() - needle.len();
    (0..=last).find(|&i| &haystack[i..i + needle.len()] == needle)
}

/// Distance-bounded nearest-match for the typo suggestion. Cap is the
/// larger of 3 or half the candidate's length, so warm-misuse cases
/// like `danger` → `destructive` (distance 6, target length 11) still
/// suggest correctly.
fn nearest_value(name: &str, candidates: &[String]) -> Option<String> {
    let mut best: Option<(usize, String)> = None;
    for c in candidates {
        let cap = std::cmp::max(3, c.len() / 2 + 1);
        let d = levenshtein(name, c, cap + 1);
        if d <= cap && best.as_ref().is_none_or(|(b, _)| d < *b) {
            best = Some((d, c.clone()));
        }
    }
    best.map(|(_, s)| s)
}

fn levenshtein(a: &str, b: &str, cap: usize) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.is_empty() {
        return b.len().min(cap);
    }
    if b.is_empty() {
        return a.len().min(cap);
    }
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut curr: Vec<usize> = vec![0; b.len() + 1];
    for i in 1..=a.len() {
        curr[0] = i;
        let mut row_min = curr[0];
        for j in 1..=b.len() {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            curr[j] = (prev[j] + 1)
                .min(curr[j - 1] + 1)
                .min(prev[j - 1] + cost);
            if curr[j] < row_min {
                row_min = curr[j];
            }
        }
        if row_min > cap {
            return cap;
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[b.len()].min(cap)
}

fn byte_offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let bytes = source.as_bytes();
    let mut line = 1usize;
    let mut col = 1usize;
    let cap = offset.min(bytes.len());
    let mut i = 0;
    while i < cap {
        if bytes[i] == b'\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
        i += 1;
    }
    (line, col)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fake_path() -> PathBuf {
        PathBuf::from("/fake/view.sail")
    }

    fn run(src: &str) -> Vec<Finding> {
        check(&fake_path(), src)
    }

    fn rule_ids(f: &[Finding]) -> Vec<&'static str> {
        f.iter().map(|x| x.rule).collect()
    }

    #[test]
    fn valid_button_value_passes() {
        let src = r#"<badge pill tone="primary" size="sm">Go</badge>"#;
        assert!(run(src).is_empty(), "{:?}", run(src));
    }

    #[test]
    fn unknown_tone_value_reports_with_suggestion() {
        // The motivating case: tone="danger" silently fell through to
        // the default before this rule existed.
        let src = r#"<badge tone="danger">Sign out</badge>"#;
        let f = run(src);
        assert_eq!(rule_ids(&f), vec![RULE_ID]);
        assert!(f[0].message.contains("`danger`"), "{}", f[0].message);
        assert!(
            f[0].message.contains("destructive"),
            "expected `destructive` suggestion in message, got: {}",
            f[0].message
        );
    }

    #[test]
    fn dynamic_binding_is_skipped() {
        let src = r#"<badge tone={accent()}>X</badge>"#;
        assert!(run(src).is_empty());
    }

    #[test]
    fn bound_attr_namespace_is_skipped() {
        let src = r#"<badge bind:tone="anything" attr:tone="danger">X</badge>"#;
        assert!(run(src).is_empty());
    }

    #[test]
    fn unknown_primitive_is_skipped() {
        // Not in the registry — user-authored components are out of scope.
        let src = r#"<sail-not-a-real-primitive tone="banana"></sail-not-a-real-primitive>"#;
        assert!(run(src).is_empty());
    }

    #[test]
    fn unknown_prop_is_skipped() {
        // tone is registered, but `flavor` is not — leave it alone.
        let src = r#"<badge flavor="grape" tone="primary">X</badge>"#;
        assert!(run(src).is_empty());
    }

    #[test]
    fn position_points_at_value_start() {
        let src = "<badge tone=\"danger\">X</badge>";
        let f = run(src);
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].line, 1);
        // 1-based col of the first char of `danger`.
        // `<badge tone="` = 13 chars; `d` starts at col 14.
        assert_eq!(f[0].col, 14);
    }

    #[test]
    fn mustache_inside_literal_is_dynamic() {
        // `tone="{x()}"` — value is dynamic, do not validate.
        let src = r#"<badge tone="{accentToken()}">X</badge>"#;
        assert!(run(src).is_empty());
    }

    #[test]
    fn comment_block_is_skipped() {
        // Primitives often quote example tags in a leading docblock —
        // those are not real call sites.
        let src = r#"<!-- <badge tone="danger"> -->
<badge tone="primary">X</badge>"#;
        assert!(run(src).is_empty(), "{:?}", run(src));
    }

    #[test]
    fn multiple_findings_on_one_tag() {
        let src = r#"<badge variant="grape" tone="danger" size="huge">X</badge>"#;
        let f = run(src);
        assert_eq!(f.len(), 3, "{f:?}");
        assert!(f.iter().all(|x| x.rule == RULE_ID));
    }
}
