//! `sui-no-style-block` — raw `<style>` blocks are not the floor styling surface.
//!
//! The styling floor is: **one way to do CSS — the design-token vocabulary**,
//! authored as `sx` token utilities (`sx="p-4 bg-surface-2 rounded-lg"`).
//! Motion, tone/state variants, and responsive breakpoints are becoming
//! first-class *language elements* rather than hand-authored CSS. A raw
//! `<style>` block is the escape hatch the floor exists to remove.
//!
//! Severity is **Warn** by nature so the rule read directly (the LSP, `sui_lint`
//! CLI) surfaces the hint without hard-failing an unrelated tool. The compiler,
//! though, promotes it to a hard `Error` UNCONDITIONALLY: this is a floor rule
//! (`check::policy::FLOOR_RULES`), and `check::lint_overrides::enforce_floor`
//! runs on every compile independent of `sui.config.json` or `lint_overrides`.
//! Raw `<style>` is banned at the floor, primitives included; there is no
//! capability key, preset, or host wiring under which it compiles.
//!
//! IMPORTANT: this rule scans the **raw** source (pre-preprocess), so the
//! synthetic `<style>` that `sx` lowering injects is invisible to it — only
//! author-written blocks are flagged. Skips HTML comments.
//!
//! Scoped to the `<style>` element only. The inline `style=` attribute is a
//! separate raw-CSS surface handled by its own rules; this rule owns the block.

use std::path::Path;

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sui-no-style-block";

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let bytes = source.as_bytes();
    let mut findings = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if source[i..].starts_with("<!--") {
            match source[i..].find("-->") {
                Some(end) => {
                    i += end + 3;
                    continue;
                }
                None => break,
            }
        }
        if bytes[i] == b'<' {
            let rest = &source[i + 1..];
            if rest.starts_with("style") {
                // must be a real tag open: the char after the name ends it.
                let after = rest.as_bytes().get("style".len()).copied();
                if matches!(after, Some(b) if b == b'>' || b == b'/' || b.is_ascii_whitespace()) {
                    let (line, col) = offset_to_line_col(source, i);
                    findings.push(Finding {
                        file: path.to_path_buf(),
                        line,
                        col,
                        rule: RULE_ID,
                        severity: Severity::Warn,
                        message:
                            "raw <style> is not the floor styling surface — the floor is one way \
                             to do CSS: sx token utilities (sx=\"p:4 bg:surface-2\"). \
                             Motion/variants/responsive are becoming language elements, not raw \
                             CSS. This is the opinionated floor; primitives use it too."
                                .to_string(),
                        suggest: Some(
                            "Text elements (<h1>-<h6>, <p>) are styled from the type scale \
                             automatically — write them plain. Everything else: sx= for surface \
                             tokens, <sui-stack>/<sui-cluster> for structure. A gap here is a \
                             missing language element to add, not a reason to hand-author CSS."
                                .to_string(),
                        ),
                    });
                }
            }
        }
        // Advance one full UTF-8 char so `source[i..]` stays on a boundary — a
        // bare `i += 1` panics mid-codepoint on any multi-byte char.
        i += 1;
        while i < bytes.len() && (bytes[i] & 0xC0) == 0x80 {
            i += 1;
        }
    }
    findings
}

fn offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 1usize;
    let mut col = 1usize;
    for (idx, b) in source.bytes().enumerate() {
        if idx == offset {
            break;
        }
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

    fn run(src: &str) -> usize {
        check(Path::new("t.sui"), src).len()
    }

    #[test]
    fn flags_style_block() {
        assert_eq!(run(r#"<sui-card><style>:host{color:red}</style></sui-card>"#), 1);
        assert_eq!(run("<style>\n  .x { padding: 4px }\n</style>"), 1);
    }

    #[test]
    fn ignores_style_attr_and_lookalikes() {
        // `style=` attribute is a different surface; `<styled-thing>` is not <style>.
        assert_eq!(run(r#"<sui-card style="x">y</sui-card>"#), 0);
        assert_eq!(run(r#"<styled-thing>y</styled-thing>"#), 0);
    }

    #[test]
    fn is_warn_not_error() {
        let f = check(Path::new("t.sui"), "<style>x</style>");
        assert_eq!(f.len(), 1);
        assert!(matches!(f[0].severity, Severity::Warn));
    }

    #[test]
    fn skips_comments() {
        assert_eq!(run(r#"<!-- <style>x</style> --> <sui-card>y</sui-card>"#), 0);
    }

    #[test]
    fn multibyte_source_does_not_panic() {
        assert_eq!(run(r#"<p>café — ☕</p><style>x</style>"#), 1);
        assert_eq!(run(r#"<sui-card>café ☕ retry…</sui-card>"#), 0);
    }
}
