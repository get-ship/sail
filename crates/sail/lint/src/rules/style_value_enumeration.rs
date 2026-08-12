//! `sail-style-value-enumeration` — prop value enumeration smell in Style.
//!
//! Fires when ≥3 compound rows in one style object differ **only** in a value
//! fed to a setter (same condition keys family, same setter, varying arg)
//! and the varying arg correlates with a prop. Message teaches `bind(name)`.

use std::collections::{BTreeSet, HashMap};
use std::path::Path;

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sail-style-value-enumeration";

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (offset, block) in find_compound_arrays(source) {
        if let Some((prop, setter, n)) = detect_enumeration(block) {
            if n >= 3 {
                let (line, col) = offset_to_line_col(source, offset);
                findings.push(Finding {
                    file: path.to_path_buf(),
                    line,
                    col,
                    rule: RULE_ID,
                    severity: Severity::Warn,
                    message: format!(
                        "style value enumeration: {n} compound rows vary only `{prop}` → \
                         `{setter}(…)` — prefer `bind({prop})` on a single row so any value is \
                         legal without an enumerated matrix."
                    ),
                    suggest: Some(format!(
                        "Collapse the matrix into one row using `{setter}(bind({prop}))` \
                         (declare `{prop}` as a number prop when the setter is a length)."
                    )),
                });
            }
        }
    }
    findings
}

fn find_compound_arrays(source: &str) -> Vec<(usize, &str)> {
    let mut out = Vec::new();
    let mut i = 0;
    let bytes = source.as_bytes();
    while i < bytes.len() {
        // Advance on UTF-8 char boundaries only.
        if !source.is_char_boundary(i) {
            i += 1;
            continue;
        }
        if source[i..].starts_with("compound:") {
            let after = i + "compound:".len();
            let rest = source[after..].trim_start();
            let pad = source[after..].len() - rest.len();
            let start = after + pad;
            if rest.starts_with('[') {
                if let Some(end) = matching_bracket(source, start, b'[', b']') {
                    out.push((start, &source[start..=end]));
                    i = end + 1;
                    continue;
                }
            }
        }
        let mut n = i + 1;
        while n < bytes.len() && !source.is_char_boundary(n) {
            n += 1;
        }
        i = n;
    }
    out
}

fn matching_bracket(s: &str, open: usize, open_ch: u8, close_ch: u8) -> Option<usize> {
    let bytes = s.as_bytes();
    if open >= bytes.len() || bytes[open] != open_ch {
        return None;
    }
    let mut depth = 0i32;
    let mut i = open;
    let mut quote: Option<u8> = None;
    while i < bytes.len() {
        let b = bytes[i];
        if let Some(q) = quote {
            if b == q {
                quote = None;
            }
            i += 1;
            continue;
        }
        match b {
            b'"' | b'\'' => quote = Some(b),
            b if b == open_ch => depth += 1,
            b if b == close_ch => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

fn detect_enumeration(block: &str) -> Option<(String, String, usize)> {
    let mut rows: Vec<(HashMap<String, String>, Vec<(String, String)>)> = Vec::new();
    let mut i = 0;
    let bytes = block.as_bytes();
    while i < bytes.len() {
        if bytes[i] == b'{' {
            if let Some(end) = matching_bracket(block, i, b'{', b'}') {
                rows.push(parse_row(&block[i..=end]));
                i = end + 1;
                continue;
            }
        }
        i += 1;
    }
    if rows.len() < 3 {
        return None;
    }

    let mut prop_values: HashMap<String, Vec<(usize, String)>> = HashMap::new();
    for (i, (conds, _)) in rows.iter().enumerate() {
        for (k, v) in conds {
            prop_values
                .entry(k.clone())
                .or_default()
                .push((i, v.clone()));
        }
    }

    for (prop, vals) in &prop_values {
        let distinct: BTreeSet<&str> = vals.iter().map(|(_, v)| v.as_str()).collect();
        if distinct.len() < 3 {
            continue;
        }
        let idxs: Vec<usize> = vals.iter().map(|(i, _)| *i).collect();
        let mut setter_args: HashMap<String, BTreeSet<String>> = HashMap::new();
        for &i in &idxs {
            for (s, a) in &rows[i].1 {
                setter_args.entry(s.clone()).or_default().insert(a.clone());
            }
        }
        for (setter, args) in setter_args {
            if args.len() >= 3 {
                return Some((prop.clone(), setter, args.len()));
            }
        }
    }
    None
}

fn parse_row(row: &str) -> (HashMap<String, String>, Vec<(String, String)>) {
    let mut conds = HashMap::new();
    let mut setters = Vec::new();
    scan_setters(row, &mut setters);
    let re_flat = row.trim_matches(|c| c == '{' || c == '}');
    for part in split_top_level(re_flat) {
        let part = part.trim();
        if let Some((k, v)) = part.split_once(':') {
            let k = k.trim();
            let v = v.trim().trim_matches(|c| c == '\'' || c == '"');
            if k.chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
                && !v.contains('(')
                && !v.starts_with('{')
                && !matches!(
                    k,
                    "box" | "paint" | "text" | "animate" | "when" | "match" | "compound"
                )
            {
                conds.insert(k.to_string(), v.to_string());
            }
        }
    }
    (conds, setters)
}

fn scan_setters(s: &str, out: &mut Vec<(String, String)>) {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_alphabetic() {
            let start = i;
            i += 1;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            let name = &s[start..i];
            let mut j = i;
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b'(' {
                if let Some(end) = matching_bracket(s, j, b'(', b')') {
                    let arg = s[j + 1..end].trim();
                    if !matches!(
                        name,
                        "style" | "match" | "when" | "compound" | "bind" | "if" | "for"
                    ) {
                        out.push((name.to_string(), arg.to_string()));
                    }
                    i = end + 1;
                    continue;
                }
            }
        }
        i += 1;
    }
}

fn split_top_level(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let bytes = s.as_bytes();
    let mut start = 0;
    let mut depth = 0i32;
    let mut quote: Option<u8> = None;
    for (i, &b) in bytes.iter().enumerate() {
        if let Some(q) = quote {
            if b == q {
                quote = None;
            }
            continue;
        }
        match b {
            b'"' | b'\'' => quote = Some(b),
            b'(' | b'{' | b'[' => depth += 1,
            b')' | b'}' | b']' => depth -= 1,
            b',' if depth == 0 => {
                out.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    if start < s.len() {
        out.push(&s[start..]);
    }
    out
}

fn offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 1usize;
    let mut col = 1usize;
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

    #[test]
    fn offset_matrix_warns() {
        let src = r#"
<div style({ compound: [
  { offset: 8, placement: rail, sticky: true, box: top(8) },
  { offset: 10, placement: rail, sticky: true, box: top(10) },
  { offset: 12, placement: rail, sticky: true, box: top(12) },
  { offset: 16, placement: rail, sticky: true, box: top(16) }
] })></div>
"#;
        let f = check(Path::new("x.sail"), src);
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(f[0].message.contains("bind(offset)"), "{:?}", f[0].message);
        assert_eq!(f[0].rule, RULE_ID);
    }

    #[test]
    fn single_row_ok() {
        let src = r#"
<div style({ compound: [
  { sticky: true, box: top(bind(offset)) }
] })></div>
"#;
        assert!(check(Path::new("x.sail"), src).is_empty());
    }
}
