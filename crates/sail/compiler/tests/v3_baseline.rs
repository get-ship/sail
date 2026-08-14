//! Named snapshot of `sail-compiler framework-css` output.
//!
//! Checked-in under `tests/v3_baseline/` for later declaration-equal
//! parity. The default-on test is “snapshot present, `decl_count > 0`” —
//! not “equals live `element-skins.css`.” That file will drift when
//! `component.*` is cut; this pin must stay green.

use std::path::{Path, PathBuf};

fn baseline_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/v3_baseline")
}

/// Stable sorted unique `prop:value` pairs. Grouping at-rules are walked;
/// comments are stripped. Later suites can declaration-equal two sheets
/// with this helper — the default test does not compare live CSS.
fn declaration_pairs(css: &str) -> Vec<String> {
    let mut unique: Vec<String> = all_declarations(css)
        .into_iter()
        .map(|(p, v)| format!("{p}:{v}"))
        .collect();
    unique.sort();
    unique.dedup();
    unique
}

fn all_declarations(css: &str) -> Vec<(String, String)> {
    let stripped = strip_comments(css);
    let mut decls = Vec::new();
    walk_blocks(&stripped, &mut decls);
    decls
}

fn strip_comments(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    let mut in_str = None;
    while i < b.len() {
        let c = b[i];
        if let Some(q) = in_str {
            out.push(c);
            if c == b'\\' && i + 1 < b.len() {
                out.push(b[i + 1]);
                i += 2;
                continue;
            }
            if c == q {
                in_str = None;
            }
            i += 1;
            continue;
        }
        if c == b'"' || c == b'\'' {
            in_str = Some(c);
            out.push(c);
            i += 1;
            continue;
        }
        if c == b'/' && i + 1 < b.len() && b[i + 1] == b'*' {
            i += 2;
            while i + 1 < b.len() && !(b[i] == b'*' && b[i + 1] == b'/') {
                i += 1;
            }
            i = (i + 2).min(b.len());
            out.push(b' ');
            continue;
        }
        out.push(c);
        i += 1;
    }
    String::from_utf8(out).expect("skins.css is utf-8")
}

fn walk_blocks(chunk: &str, decls: &mut Vec<(String, String)>) {
    let b = chunk.as_bytes();
    let mut i = 0;
    let mut in_str = None;
    while i < b.len() {
        let c = b[i];
        if let Some(q) = in_str {
            if c == b'\\' && i + 1 < b.len() {
                i += 2;
                continue;
            }
            if c == q {
                in_str = None;
            }
            i += 1;
            continue;
        }
        if c == b'"' || c == b'\'' {
            in_str = Some(c);
            i += 1;
            continue;
        }
        if c == b'{' {
            let (inner, next) = brace_inner(chunk, i);
            if has_top_level_brace(inner) {
                walk_blocks(inner, decls);
            } else {
                parse_decl_list(inner, decls);
            }
            i = next;
            continue;
        }
        i += 1;
    }
}

/// Content between `{` at `open` and its matching `}`. Returns
/// `(inner, index_after_close)`.
fn brace_inner(s: &str, open: usize) -> (&str, usize) {
    let b = s.as_bytes();
    let mut depth = 1;
    let mut j = open + 1;
    let mut in_str = None;
    while j < b.len() && depth > 0 {
        let c = b[j];
        if let Some(q) = in_str {
            if c == b'\\' && j + 1 < b.len() {
                j += 2;
                continue;
            }
            if c == q {
                in_str = None;
            }
            j += 1;
            continue;
        }
        if c == b'"' || c == b'\'' {
            in_str = Some(c);
        } else if c == b'{' {
            depth += 1;
        } else if c == b'}' {
            depth -= 1;
        }
        j += 1;
    }
    let end = if depth == 0 { j - 1 } else { b.len() };
    (&s[open + 1..end], j)
}

fn has_top_level_brace(s: &str) -> bool {
    let b = s.as_bytes();
    let mut depth_paren = 0i32;
    let mut in_str = None;
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        if let Some(q) = in_str {
            if c == b'\\' && i + 1 < b.len() {
                i += 2;
                continue;
            }
            if c == q {
                in_str = None;
            }
            i += 1;
            continue;
        }
        if c == b'"' || c == b'\'' {
            in_str = Some(c);
        } else if c == b'(' {
            depth_paren += 1;
        } else if c == b')' {
            depth_paren = (depth_paren - 1).max(0);
        } else if c == b'{' && depth_paren == 0 {
            return true;
        }
        i += 1;
    }
    false
}

fn parse_decl_list(block: &str, decls: &mut Vec<(String, String)>) {
    for raw in split_top_level(block, b';') {
        let item = raw.trim();
        if item.is_empty() || item.starts_with('@') {
            continue;
        }
        let Some(colon) = first_top_level_colon(item) else {
            continue;
        };
        let prop = item[..colon].trim();
        let value = item[colon + 1..]
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        if prop.is_empty() || value.is_empty() {
            continue;
        }
        let first = prop.as_bytes()[0];
        if !(first.is_ascii_alphabetic() || first == b'-' || first == b'_') {
            continue;
        }
        decls.push((prop.to_string(), value));
    }
}

fn first_top_level_colon(item: &str) -> Option<usize> {
    let b = item.as_bytes();
    let mut in_str = None;
    let mut depth_paren = 0i32;
    for (i, &c) in b.iter().enumerate() {
        if let Some(q) = in_str {
            if c == b'\\' {
                continue;
            }
            if c == q {
                in_str = None;
            }
            continue;
        }
        if c == b'"' || c == b'\'' {
            in_str = Some(c);
        } else if c == b'(' {
            depth_paren += 1;
        } else if c == b')' {
            depth_paren = (depth_paren - 1).max(0);
        } else if c == b':' && depth_paren == 0 {
            return Some(i);
        }
    }
    None
}

fn split_top_level(s: &str, delim: u8) -> Vec<&str> {
    let b = s.as_bytes();
    let mut parts = Vec::new();
    let mut start = 0;
    let mut depth_brace = 0i32;
    let mut depth_paren = 0i32;
    let mut in_str = None;
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        if let Some(q) = in_str {
            if c == b'\\' && i + 1 < b.len() {
                i += 2;
                continue;
            }
            if c == q {
                in_str = None;
            }
            i += 1;
            continue;
        }
        if c == b'"' || c == b'\'' {
            in_str = Some(c);
        } else if c == b'{' {
            depth_brace += 1;
        } else if c == b'}' {
            depth_brace -= 1;
        } else if c == b'(' {
            depth_paren += 1;
        } else if c == b')' {
            depth_paren -= 1;
        } else if c == delim && depth_brace == 0 && depth_paren == 0 {
            parts.push(&s[start..i]);
            start = i + 1;
        }
        i += 1;
    }
    if start <= s.len() {
        parts.push(&s[start..]);
    }
    parts
}

fn read_meta(path: &Path) -> serde_json::Value {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| {
        panic!("read {}: {e}", path.display());
    });
    serde_json::from_str(&text).unwrap_or_else(|e| {
        panic!("{} is not JSON: {e}", path.display());
    })
}

#[test]
fn v3_skins_baseline_exists() {
    let dir = baseline_dir();
    let skins = dir.join("skins.css");
    let meta_path = dir.join("meta.json");
    let decls_path = dir.join("decls.txt");

    assert!(
        skins.is_file(),
        "named skins snapshot missing — capture with:\n  \
         cargo run -p sail-compiler --offline -- framework-css --out {}",
        skins.display()
    );
    assert!(meta_path.is_file(), "missing {}", meta_path.display());
    assert!(decls_path.is_file(), "missing {}", decls_path.display());

    let css = std::fs::read_to_string(&skins).expect("read skins.css");
    assert!(!css.trim().is_empty(), "skins.css is empty");

    let meta = read_meta(&meta_path);
    let decl_count = meta["decl_count"]
        .as_u64()
        .expect("meta.json.decl_count must be a number");
    assert!(decl_count > 0, "decl_count must be > 0, got {decl_count}");

    let git_sha = meta["git_sha"].as_str().unwrap_or("").trim().to_string();
    assert!(!git_sha.is_empty(), "meta.json.git_sha must be non-empty");

    let bytes = meta["bytes"]
        .as_u64()
        .expect("meta.json.bytes must be a number");
    assert_eq!(
        bytes,
        css.len() as u64,
        "meta.json.bytes must match skins.css length"
    );

    // Snapshot self-check only. Do not compare against live generated CSS.
    let parsed = all_declarations(&css);
    assert_eq!(
        decl_count,
        parsed.len() as u64,
        "meta.json.decl_count must match the extracted declaration count"
    );
    assert!(
        !declaration_pairs(&css).is_empty(),
        "declaration extract must be non-empty"
    );
}
