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
//! Natural severity `Warn` (an unconfigured buffer keeps compiling and the LSP
//! shows the hint); every policy promotes it to `Error`.

use std::path::Path;

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sui-no-class";

/// True iff a `sui.config.json` is anywhere up the parent chain of `path`.
/// When present, the pack has opted INTO author-managed CSS through a declared
/// stylesheet (the `sui-no-undeclared-stylesheet` rule enforces the seam),
/// so `class=` and `class:` are LEGAL here — they pair with the declared
/// sheet. Without that opt-in the ban holds.
fn has_sui_config(path: &Path) -> bool {
    let mut dir = path.parent().unwrap_or(path).to_path_buf();
    loop {
        if dir.join("sui.config.json").is_file() {
            return true;
        }
        if !dir.pop() {
            return false;
        }
    }
}

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    // A pack that ships a `sui.config.json` declares its OWN stylesheets;
    // classes are then author-managed and pair with the declared sheet
    // (the undeclared-stylesheet rule keeps that seam honest).
    if has_sui_config(path) {
        return Vec::new();
    }
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
                severity: Severity::Warn,
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
            r#"<div sx="p:4 bg:surface-2" role="listbox" aria-label="x" bind:this={el}></div>"#
        )
        .is_empty());
    }

    #[test]
    fn a_word_ending_in_class_is_not_class() {
        assert!(run(r#"<div data-subclass="x"></div>"#).is_empty());
    }
}
