//! `sail-dead-const` — a Style const is never spent.
//!
//! Autofix deletes the unused `style name =`. Exact aliases of an existing
//! name are `sail-const-eq-name`, not this rule.

use std::path::Path;

use crate::report::{Finding, Severity};

use super::layout;
use super::style_invariants;

pub const RULE_ID: &str = "sail-dead-const";

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let decls = style_invariants::harvest_style_consts(source);
    let spent = style_invariants::spent_style_names(source, &decls);
    let mut findings = Vec::new();
    for decl in decls {
        if spent.contains(&decl.name) {
            continue;
        }
        let style = style_invariants::parse_const_rhs(&decl.rhs);
        if style_invariants::exact_alias_meaning(&decl.name, &style).is_some() {
            continue;
        }
        let (line, col) = layout::offset_to_line_col(source, decl.name_offset);
        findings.push(Finding {
            file: path.to_path_buf(),
            line,
            col,
            rule: RULE_ID,
            severity: Severity::Error,
            message: format!(
                "`{}` is never spent — delete it, or apply `style({})`.",
                decl.name, decl.name
            ),
            suggest: Some(format!("delete unused style const `{}`", decl.name)),
        });
    }
    findings
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn run(src: &str) -> Vec<Finding> {
        check(Path::new("t.sail"), src)
    }

    #[test]
    fn flags_unused_style_const() {
        let f = run(r#"
<scope>
  style unused = { bg(muted) }
</scope>
<column>x</column>
"#);
        assert_eq!(f.len(), 1, "{f:?}");
        assert_eq!(f[0].rule, RULE_ID);
        assert_eq!(
            f[0].message,
            "`unused` is never spent — delete it, or apply `style(unused)`."
        );
    }

    #[test]
    fn spent_const_is_not_this_error() {
        let f = run(r#"
<scope>
  style unused = { bg(muted) }
</scope>
<column style(unused)>x</column>
"#);
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn heading_alias_is_not_this_error() {
        let f = run(r#"
<scope>
  style heading = { type(heading) }
</scope>
<h1>x</h1>
"#);
        assert!(f.is_empty(), "{f:?}");
    }
}
