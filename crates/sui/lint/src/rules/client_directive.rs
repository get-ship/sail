//! `sui-client-directive` — templates that consume the `ship` client should
//! declare it with a top-level `<client name="ship" />` directive (#265).
//!
//! The directive is the keyed-injection contract: a view DECLARES the
//! clients it consumes, the host supplies a keyed map at mount, and the
//! framework wires only what's declared. Ambient `ship` (no declaration)
//! remains the grandfathered compatibility default so the ~100-view
//! migration can land as a sweep, not a flag-day — this rule is the
//! affordance-first nudge (same posture as #166).
//!
//! Detection is textual: a `ship.`-rooted reference anywhere outside an
//! HTML comment, in a file with no `<client name="ship"` declaration.
//! One finding per file (at the first occurrence) — the fix is a single
//! declaration line, not a per-call-site edit.
//!
//! **Severity: Warn.** Promote to Error once the first-party sweep lands
//! and ambient `ship` is retired from the compiler's BUILTINS.

use std::path::Path;

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sui-client-directive";

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    if declares_ship_client(source) {
        return Vec::new();
    }
    let Some(offset) = first_ship_root_usage(source) else {
        return Vec::new();
    };
    let (line, col) = byte_offset_to_line_col(source, offset);
    vec![Finding {
        file: path.to_path_buf(),
        line,
        col,
        rule: RULE_ID,
        severity: Severity::Warn,
        message: "template uses the ambient `ship` client without declaring it — \
                  add a top-level `<client name=\"ship\" />` directive. Declared \
                  clients are the keyed-injection contract: the host supplies the \
                  map at mount and the framework wires only what's declared. \
                  Ambient `ship` is a grandfathered compatibility default."
            .to_string(),
        suggest: Some(
            "Add `<client name=\"ship\" />` beside the template's <data>/<prop> \
             directives."
                .to_string(),
        ),
    }]
}

/// `<client name="ship" />` (either quote style) anywhere in the source.
fn declares_ship_client(source: &str) -> bool {
    let mut rest = source;
    while let Some(pos) = rest.find("<client") {
        let after = &rest[pos + "<client".len()..];
        let head = &after[..after.find('>').unwrap_or(after.len())];
        if head.contains("name=\"ship\"") || head.contains("name='ship'") {
            return true;
        }
        rest = after;
    }
    false
}

/// Byte offset of the first `ship.` reference used as an expression root
/// (preceded by a non-identifier, non-`.` char), skipping HTML comments.
fn first_ship_root_usage(source: &str) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut i = 0;
    while i < source.len() {
        if source[i..].starts_with("<!--") {
            match source[i + 4..].find("-->") {
                Some(rel) => {
                    i += 4 + rel + 3;
                    continue;
                }
                None => return None,
            }
        }
        if source[i..].starts_with("ship.") {
            let prev_ok = i == 0 || {
                let p = bytes[i - 1];
                !(p.is_ascii_alphanumeric() || p == b'_' || p == b'$' || p == b'.')
            };
            // `ship.` must be followed by an identifier start — `ship. `
            // in prose is not a reference.
            let next = bytes.get(i + 5).copied();
            let next_ok = matches!(next, Some(c) if c.is_ascii_alphabetic() || c == b'_' || c == b'$');
            if prev_ok && next_ok {
                return Some(i);
            }
        }
        // Advance one char (UTF-8 safe).
        i += source[i..].chars().next().map(char::len_utf8).unwrap_or(1);
    }
    None
}

fn byte_offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    for (i, ch) in source.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
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

    fn run(src: &str) -> Vec<Finding> {
        check(&PathBuf::from("page.sui"), src)
    }

    #[test]
    fn undeclared_ship_usage_warns_once() {
        let f = run(
            "<data name=\"o\" source=\"ship.git.overview()\" live />\n\
             <button on:click={() => ship.git.stage({ path: 'x' })}>stage</button>",
        );
        assert_eq!(f.len(), 1, "one finding per file, got {f:?}");
        assert_eq!(f[0].rule, RULE_ID);
        assert_eq!(f[0].severity, Severity::Warn);
        assert_eq!((f[0].line, f[0].col), (1, 24));
    }

    #[test]
    fn declared_ship_is_clean() {
        let f = run(
            "<client name=\"ship\" />\n\
             <data name=\"o\" source=\"ship.git.overview()\" live />\n<div>{o()}</div>",
        );
        assert!(f.is_empty(), "declared ship must not warn: {f:?}");
    }

    #[test]
    fn non_ship_clients_and_prose_do_not_warn() {
        // A different declared client, a comment mention, and a word that
        // merely ends in "ship." are all clean.
        let f = run(
            "<client name=\"api\" />\n\
             <!-- ship.git.overview() is quoted prose -->\n\
             <data name=\"t\" source=\"api.demo.clock()\" live />\n\
             <p>Update your membership. Renew today.</p>",
        );
        assert!(f.is_empty(), "got {f:?}");
    }

    #[test]
    fn single_quoted_declaration_counts() {
        let f = run("<client name='ship' />\n<div>{ship.agents.summaries()}</div>");
        assert!(f.is_empty(), "got {f:?}");
    }
}
