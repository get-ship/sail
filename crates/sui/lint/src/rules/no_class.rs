//! `sui-no-class` — `class=` and `class:` are banned at every policy.
//!
//! Light DOM has no boundary. A class written into a component's template is a
//! GLOBAL CSS target: the floor bans authoring CSS inside the component, and a
//! published class hook is how that CSS moves to a file the compiler never sees.
//! It is the same back door as an open `data-*` state prefix, wearing the name
//! of an anatomy hook.
//!
//! It also buys nothing the language doesn't already give you:
//!
//!   styling   sx=, and the framework skins for bare elements
//!   state     aria-* + the closed sx state prefixes (`hover:`, `selected:`, …)
//!   anatomy   the element's own semantics — role, aria-*, tag
//!   handles   bind:this — a ref, not a selector
//!
//! The compiler emits its own classes (`sui-root`, layout, sx atoms). Those are
//! the framework's, not the author's; this rule only sees what the source wrote.
//!
//! FLOOR — refuses everywhere. The historical "opt-in via `sail.config.json`
//! presence" was the SAME disease as the dead stylesheet permit: a check
//! that returned `Vec::new()` because a config file existed somewhere
//! above the source. With the permit fixed and the floor in place, every
//! pack's `sail.config.json` declares `stylesheets: []` and ships zero
//! `.css` files — the "declared stylesheet" the early-return justified
//! itself with was invented. `class=` is now `Error` on every path,
//! pack, route, and app; no config relaxes it.
//!
//! Natural severity `Error` (and on the floor). There is no
//! unconfigured-buffer fallback: an authored `class=` is a banned CSS
//! seam wherever the rule fires.

use std::path::Path;

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sail-no-class";

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    // FLOOR — refuses everywhere. The historical "opt-in via `sail.config.json`
    // presence" was removed: it walked up to the filesystem root, found a
    // config anywhere, and returned `Vec::new()` — the same dead layer as
    // the stylesheet permit. The pack's `sail.config.json` declares
    // `stylesheets: []` and ships zero `.css` files; there is no
    // declared sheet for a class to "pair with".
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
        // A `class` ATTRIBUTE or the `class:` directive, as an attribute
        // position: preceded by whitespace, followed by `=` or `:`.
        if i > 0
            && bytes[i - 1].is_ascii_whitespace()
            && source[i..].starts_with("class")
            && next_is_eq_or_colon(&bytes[i + 5..])
        {
            let (line, col) = offset_to_line_col(source, i);
            findings.push(Finding {
                file: path.to_path_buf(),
                line,
                col,
                rule: RULE_ID,
                severity: Severity::Error,
                message: "`class` is not the styling surface, and light DOM has no boundary — a \
                          class in a template is a global CSS target, which is how banned CSS \
                          moves to a file the compiler never sees."
                    .to_string(),
                suggest: Some(
                    "Style with sx=. For visual state use aria-* plus the state prefixes \
                     (hover:, selected:, expanded:, …). For a handle use bind:this. For anatomy \
                     — including what a test selects on — use the element's own semantics: its \
                     role, its aria-*, its tag."
                        .to_string(),
                ),
            });
        }
        i += 1;
        while i < bytes.len() && (bytes[i] & 0xC0) == 0x80 {
            i += 1;
        }
    }
    findings
}

/// After the attribute name: optional whitespace, then `=` (the attribute) or
/// `:` (the `class:name={…}` directive).
fn next_is_eq_or_colon(rest: &[u8]) -> bool {
    let mut k = 0;
    while k < rest.len() && rest[k].is_ascii_whitespace() {
        k += 1;
    }
    k < rest.len() && (rest[k] == b'=' || rest[k] == b':')
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

    fn run(src: &str) -> Vec<Finding> {
        check(Path::new("t.sui"), src)
    }

    #[test]
    fn static_class_is_flagged() {
        assert_eq!(run(r#"<div class="track"></div>"#).len(), 1);
    }

    #[test]
    fn the_class_directive_is_flagged() {
        assert_eq!(run(r#"<a class:active={current()}>x</a>"#).len(), 1);
    }

    #[test]
    fn a_class_inside_a_comment_is_prose() {
        assert!(run(r#"<!-- a bare <span class="sui-avatar"> --><div></div>"#).is_empty());
    }

    #[test]
    fn sx_and_semantics_are_the_way() {
        assert!(run(
            r#"<div sx="p-4 bg-surface-2" role="listbox" aria-label="x" bind:this={el}></div>"#
        )
        .is_empty());
    }

    #[test]
    fn a_word_ending_in_class_is_not_class() {
        assert!(run(r#"<div data-subclass="x"></div>"#).is_empty());
    }

    // The disease this commit cures: a `sail.config.json` ANYWHERE in the
    // ancestor chain used to short-circuit the rule. The new contract is
    // unconditional — `class=` is banned regardless of which directories
    // happen to have a config. These two tests pin both ends.

    #[test]
    fn a_class_in_a_sourceless_buffer_is_still_flagged() {
        // Old code walked `path.parent()` up to the filesystem root looking
        // for any `sail.config.json`; in a unit test, `t.sui` is a bare
        // path with no ancestors, and the old code returned `Vec::new()`
        // because nothing above it had a config. New behavior: it fires
        // anyway. Same test name as `static_class_is_flagged`, but
        // expressed in a way that fails if anyone re-introduces a
        // config-walk short-circuit.
        let findings = run(r#"<div class="track"></div>"#);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule, RULE_ID);
        assert!(matches!(findings[0].severity, Severity::Error));
    }

    #[test]
    fn severity_is_error_not_warn() {
        // The rule was Warn-by-nature and promoted to Error by the floor.
        // Both layers said the same thing, but the floor was the only
        // thing actually doing the work. Now the rule itself is Error —
        // the floor still includes it, but the rule does not depend on
        // the override to refuse.
        let findings = run(r#"<div class="x"></div>"#);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Error);
    }
}
