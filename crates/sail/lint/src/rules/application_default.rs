//! `sail-application-default` — application restates the element default.
//!
//! Only when the spend **is** the tag default (`h1` → type heading). Extra
//! slots (`pad`) are not this error. Autofix omits the restated Style.

use std::path::Path;

use crate::report::{Finding, Severity};

use super::layout;
use super::style_invariants;

pub const RULE_ID: &str = "sail-application-default";

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    for tag in layout::scan_any_open_tags(source) {
        let Some(sx) = style_invariants::tag_style(&tag) else {
            continue;
        };
        let inner = style_invariants::authored_style_inner(sx);
        if inner.is_empty() || !inner.contains('(') {
            continue;
        }
        let style = style_invariants::parse_tag_style(sx);
        if !style_invariants::restates_tag_default(&tag.tag, &style) {
            continue;
        }
        let at = sx_spend_offset(source, &tag, sx).unwrap_or(tag.start);
        let (line, col) = layout::offset_to_line_col(source, at);
        findings.push(Finding {
            file: path.to_path_buf(),
            line,
            col,
            rule: RULE_ID,
            severity: Severity::Error,
            message: format!(
                "`{inner}` is the default for `<{}>` — omit it, or is this not an {}?",
                tag.tag, tag.tag
            ),
            suggest: Some("omit the restated Style".into()),
        });
    }
    findings
}

fn sx_spend_offset(source: &str, tag: &layout::OpenTag, sx: &str) -> Option<usize> {
    let open = source.get(tag.start..tag.end)?;
    let rel = open.find(sx)?;
    Some(tag.start + rel)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn run(src: &str) -> Vec<Finding> {
        check(Path::new("t.sail"), src)
    }

    #[test]
    fn flags_type_heading_on_h1() {
        let f = run("<h1 style({ type(heading) })>x</h1>");
        assert_eq!(f.len(), 1, "{f:?}");
        assert_eq!(f[0].rule, RULE_ID);
        assert_eq!(
            f[0].message,
            "`type(heading)` is the default for `<h1>` — omit it, or is this not an h1?"
        );
    }

    #[test]
    fn extra_pad_is_not_this_error() {
        let f = run("<h1 style({ type(heading), pad(4) })>x</h1>");
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn other_type_role_is_not_this_error() {
        let f = run("<h1 style({ type(body) })>x</h1>");
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn tag_without_default_is_not_this_error() {
        let f = run("<span style({ type(heading) })>x</span>");
        assert!(f.is_empty(), "{f:?}");
    }
}
