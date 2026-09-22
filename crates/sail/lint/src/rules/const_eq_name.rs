//! `sail-const-eq-name` — const restates an existing theme / tagDefault / preset name.
//!
//! Only an exact alias. Extra slots are not this error. Autofix deletes the alias.

use std::path::Path;

use crate::report::{Finding, Severity};

use super::layout;
use super::style_invariants;

pub const RULE_ID: &str = "sail-const-eq-name";

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    for decl in style_invariants::harvest_style_consts(source) {
        let style = style_invariants::parse_const_rhs(&decl.rhs);
        let Some(meaning) = style_invariants::exact_alias_meaning(&decl.name, &style) else {
            continue;
        };
        let (line, col) = layout::offset_to_line_col(source, decl.name_offset);
        findings.push(Finding {
            file: path.to_path_buf(),
            line,
            col,
            rule: RULE_ID,
            severity: Severity::Error,
            message: format!(
                "`{}` already names `{meaning}` — spend the name, don't redeclare it.",
                decl.name
            ),
            suggest: Some(format!("delete the alias `style {} =`", decl.name)),
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
    fn flags_heading_alias() {
        let f = run(r#"
<scope>
  style heading = { type(heading) }
</scope>
<h1>x</h1>
"#);
        assert_eq!(f.len(), 1, "{f:?}");
        assert_eq!(f[0].rule, RULE_ID);
        assert_eq!(
            f[0].message,
            "`heading` already names `type(heading)` — spend the name, don't redeclare it."
        );
    }

    #[test]
    fn extra_slots_are_not_this_error() {
        let f = run(r#"
<scope>
  style heading = { type(heading), pad(4) }
</scope>
<h1 style(heading)>x</h1>
"#);
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn quoted_trap_in_comment_is_not_this_error() {
        let f = run(r#"
<!--
  style heading = { type(heading) }
-->
<h1>x</h1>
"#);
        assert!(f.is_empty(), "{f:?}");
    }
}
