//! `sail-virtualizer` — `use:virtualRow` / `use:virtualSpacer` must wire CSS.
//!
//! Runtime actions write geometry to custom properties; **sx must read them**
//! or every absolute row stacks at y=0 (the studio explorer disaster) and the
//! spacer contributes no scroll height.
//!
//! | Action              | Required sx bridge                                      |
//! |---------------------|---------------------------------------------------------|
//! | `use:virtualRow`    | `translateY(bind(rowY))` or `translate-y-(--row-y)`     |
//! | `use:virtualSpacer` | `h(bind(rowsH))` / `height-(--rows-h)` / `h-(--rows-h)` |
//!
//! Reference: `packages/sail/runtime/src/actions/virtual-row.ts`,
//! `packages/components/sail/primitives/src/sail-tree/sail-tree.sail`.
//!
//! **Severity: Error.** Missing wiring is a silent functional bug, not taste.

use std::path::Path;

use crate::report::{Finding, Severity};
use crate::rules::layout::{self, OpenTag};

pub const RULE_ID: &str = "sail-virtualizer";

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    for t in layout::scan_any_open_tags(source) {
        if let Some(f) = check_tag(path, source, &t) {
            findings.push(f);
        }
    }
    findings
}

fn check_tag(path: &Path, source: &str, t: &OpenTag) -> Option<Finding> {
    let has_row = t
        .attrs
        .iter()
        .any(|(n, _)| n.eq_ignore_ascii_case("use:virtualRow"));
    let has_spacer = t
        .attrs
        .iter()
        .any(|(n, _)| n.eq_ignore_ascii_case("use:virtualSpacer"));
    if !has_row && !has_spacer {
        return None;
    }

    let sx = t
        .attrs
        .iter()
        .find(|(n, _)| n == "style")
        .map(|(_, v)| v.as_str())
        .unwrap_or("");

    let (line, col) = layout::offset_to_line_col(source, t.start);

    if has_row && !has_row_translate(sx) {
        return Some(Finding {
            file: path.to_path_buf(),
            line,
            col,
            rule: RULE_ID,
            severity: Severity::Error,
            message: format!(
                "<{}> has use:virtualRow but no translateY(bind(rowY)) (or \
                 translate-y-(--row-y)) on the same node — virtualRow writes \
                 --row-y; without the sx bridge every absolute row stacks at y=0.",
                t.tag
            ),
            suggest: Some(
                "Add sx translate for the row offset, e.g. \
                 `style={style.position(absolute).left(0).w(full).translateY(bind(rowY))}` \
                 (see sail-tree / virtual-row.ts)."
                    .to_string(),
            ),
        });
    }

    if has_spacer && !has_spacer_height(sx) {
        return Some(Finding {
            file: path.to_path_buf(),
            line,
            col,
            rule: RULE_ID,
            severity: Severity::Error,
            message: format!(
                "<{}> has use:virtualSpacer but no height binding for --rows-h \
                 on the same node — virtualSpacer writes --rows-h; without \
                 h(bind(rowsH)) / h-(--rows-h) the list has no scroll extent.",
                t.tag
            ),
            suggest: Some(
                "Add height from the spacer var, e.g. \
                 `style={style.h(bind(rowsH)).position(relative).w(full)}` \
                 (see sail-tree / virtual-row.ts)."
                    .to_string(),
            ),
        });
    }

    None
}

/// Accept Style API and token forms that read `--row-y`.
///
/// **Loose same-node accept is intentional:** after exact forms, any
/// `translateY` / `translate-y` co-occurring with `rowY` / `row-y` / `--row-y`
/// on the same `sx` string passes. The rule is a same-node wiring check (action
/// + sx on one tag), not a full Style-AST proof — reducing false negatives on
/// real primitives matters more than rejecting exotic dead-code substrings.
fn has_row_translate(sx: &str) -> bool {
    let s = sx;
    // style.translateY(bind(rowY)) / translateY(bind(rowY))
    if contains_ci(s, "translateY(bind(rowY))") || contains_ci(s, "translateY(bind(row-y))") {
        return true;
    }
    // Token atoms: translate-y-(--row-y) / translate-y-(--rowy) after sanitize
    if contains_ci(s, "translate-y-(--row-y)") || contains_ci(s, "translate-y-(--rowy)") {
        return true;
    }
    // Lowered class fingerprint sometimes appears in tests/docs
    if s.contains("translateYbindrowY") || s.contains("translateybindrowy") {
        return true;
    }
    // Loose same-node accept (see module note above).
    let has_translate = contains_ci(s, "translateY") || contains_ci(s, "translate-y");
    let has_rowy = contains_ci(s, "rowY") || contains_ci(s, "row-y") || s.contains("--row-y");
    has_translate && has_rowy
}

/// Accept Style API and token forms that read `--rows-h`.
///
/// **Loose same-node accept is intentional** (mirrors [`has_row_translate`]):
/// height setter + `rowsH` / `--rows-h` on the same `sx` is enough.
fn has_spacer_height(sx: &str) -> bool {
    let s = sx;
    if contains_ci(s, "h(bind(rowsH))")
        || contains_ci(s, "h(bind(rows-h))")
        || contains_ci(s, "height(bind(rowsH))")
    {
        return true;
    }
    if contains_ci(s, "h-(--rows-h)")
        || contains_ci(s, "height-(--rows-h)")
        || contains_ci(s, "h-(--rowsh)")
    {
        return true;
    }
    // Loose same-node accept (see note above).
    let has_h = s.contains("h(bind")
        || contains_ci(s, "height(bind")
        || s.contains("h-(")
        || s.contains("height-(");
    let has_var = contains_ci(s, "rowsH") || contains_ci(s, "rows-h") || s.contains("--rows-h");
    has_h && has_var
}

fn contains_ci(hay: &str, needle: &str) -> bool {
    hay.to_ascii_lowercase()
        .contains(&needle.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn lint(src: &str) -> Vec<Finding> {
        check(Path::new("tree.sail"), src)
    }

    #[test]
    fn virtual_row_without_translate_fails() {
        let src = r#"
            <div use:virtualRow={w.start} style={style.position(absolute).left(0).w(full)}>
              row
            </div>
        "#;
        let f = lint(src);
        assert_eq!(f.len(), 1, "{f:?}");
        assert_eq!(f[0].rule, RULE_ID);
        assert_eq!(f[0].severity, Severity::Error);
        assert!(f[0].message.contains("virtualRow"), "{}", f[0].message);
    }

    #[test]
    fn virtual_row_with_translate_passes() {
        let src = r#"
            <row use:virtualRow={w.start} style={style.position(absolute).left(0).w(full).translateY(bind(rowY))}>
              row
            </row>
        "#;
        assert!(lint(src).is_empty(), "{:?}", lint(src));
    }

    #[test]
    fn virtual_spacer_without_height_fails() {
        let src = r#"
            <div use:virtualSpacer={totalSize()} style={style.position(relative).w(full)}>
            </div>
        "#;
        let f = lint(src);
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(f[0].message.contains("virtualSpacer"), "{}", f[0].message);
    }

    #[test]
    fn virtual_spacer_with_rows_h_passes() {
        let src = r#"
            <div use:virtualSpacer={totalSize()} style={style.h(bind(rowsH)).position(relative).w(full)}>
            </div>
        "#;
        assert!(lint(src).is_empty(), "{:?}", lint(src));
    }

    #[test]
    fn token_form_translate_accepted() {
        let src = r#"
            <grid use:virtualRow={w.start} style="translate-y-(--row-y) absolute left-0 w-full">
            </grid>
        "#;
        assert!(lint(src).is_empty(), "{:?}", lint(src));
    }

    /// Golden: shipped sail-tree must stay wired (explorer disaster regression).
    #[test]
    fn sail_tree_primitive_has_zero_virtualizer_findings() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../packages/components/sail/primitives/src/sail-tree/sail-tree.sail");
        assert!(
            path.is_file(),
            "sail-tree.sail golden missing at {} — monorepo path from \
             crates/sail/lint must resolve (virtualizer wiring regression gate)",
            path.display()
        );
        let src = std::fs::read_to_string(&path).expect("read sail-tree.sail");
        let f = check(&path, &src);
        let virt: Vec<_> = f.iter().filter(|x| x.rule == RULE_ID).collect();
        assert!(
            virt.is_empty(),
            "sail-tree must not trip sail-virtualizer: {virt:?}"
        );
    }
}
