//! `sui-no-raw-container` — raw `<div>` / `<span>` are banned in `.sui` templates.
//!
//! They are pure hand-rolled containers whose only job is structure — which
//! belongs to the layout keywords: `<column>` (vertical), `<row>`
//! (horizontal), `<grid>`, plus `<sui-card>` (surface).
//!
//! A **`strict`-tier** rule, NOT a floor rule: raw `<div>`/`<span>` are fine
//! under `full` (primitives compose structure from them). Natural severity `Warn`;
//! `check::policy::Policy::FULL` *silences* it (divs are legitimate there),
//! and `Policy::STRICT` promotes it to a hard `Error` — so app authors
//! physically cannot hand-roll layout with a token-styled `<div>` (the failure
//! the recipe-card bake-off surfaced) while primitive authors keep their boxes.
//!
//! Deliberately SCOPED to `div`/`span`. Content/semantic HTML stays legal —
//! `<p>`, `<ul>`/`<ol>`/`<li>`, `<h1>`–`<h6>`, `<a>`, `<img>`, `<strong>`,
//! `<em>`, `<figure>`, `<details>`, … — because the design system does not (and
//! should not) own a primitive for every content element. Only the two generic
//! "unstyled box" tags are closed.
//!
//! Skips HTML comments. A package sets `sail.config.json` `{ "preset": "strict" }`
//! once its files compose primitives and no longer hand-roll raw boxes.

use std::path::Path;

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sail-no-raw-container";

fn suggestion(tag: &str) -> &'static str {
    match tag {
        "div" => "use <column> (vertical), <row> (horizontal), <grid>, or <sui-card> (surface) — style it with sx",
        "span" => "use <row> for inline grouping, or a semantic inline tag (<strong>/<em>/<a>) styled with sx",
        _ => "compose a sui-* primitive",
    }
}

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
            for tag in ["div", "span"] {
                if rest.starts_with(tag) {
                    // must be a real tag open: next char ends the name
                    let after = rest.as_bytes().get(tag.len()).copied();
                    if matches!(after, Some(b) if b == b'>' || b == b'/' || b.is_ascii_whitespace()) {
                        let (line, col) = offset_to_line_col(source, i);
                        findings.push(Finding {
                            file: path.to_path_buf(),
                            line,
                            col,
                            rule: RULE_ID,
                            severity: Severity::Warn,
                            message: format!(
                                "raw <{tag}> is not allowed — structure belongs to the layout primitives; {}",
                                suggestion(tag)
                            ),
                            suggest: Some(format!(
                                "Content tags (<p>, <ul>/<li>, <h1>-<h6>, <a>, <img>, <strong>…) are fine — only <div>/<span> are closed.{}",
                                crate::config::grant_hint(path, "raw_containers")
                            )),
                        });
                    }
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
    fn bans_div_and_span() {
        assert_eq!(run(r#"<div sx="p-4">x</div>"#), 1);
        assert_eq!(run(r#"<span>hi</span>"#), 1);
    }

    #[test]
    fn allows_content_and_primitives() {
        assert_eq!(run(r#"<column gap="3"><h2>Ingredients</h2><ul><li>x</li></ul></column>"#), 0);
        assert_eq!(run(r#"<sui-card><p>hi</p><a href="/x">link</a></sui-card>"#), 0);
    }

    #[test]
    fn skips_comments_and_wordlike() {
        assert_eq!(run(r#"<!-- <div> --> <divider>ok</divider>"#), 0);
    }

    #[test]
    fn multibyte_source_does_not_panic() {
        assert_eq!(run(r#"<p>café — ☕</p><div>x</div>"#), 1);
        assert_eq!(run(r#"<sui-card>café ☕ retry…</sui-card>"#), 0);
    }
}
