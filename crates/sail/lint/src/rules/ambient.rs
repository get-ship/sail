//! Shared ambient-platform rule helpers (SPEC-DOMDEC-1).
//!
//! Path gate: app sources under `apps/*/src/**` and `apps/*/routes/**` only
//! (not test/qa/scripts/packages). Allowlist: `// sail-allow <rule-id>` on
//! the same or previous line.

use std::path::Path;

use crate::report::{Finding, Severity};

/// True when this path is in-scope for ambient Part-2 rules.
pub fn is_ambient_app_path(path: &Path) -> bool {
    let p = path.to_string_lossy().replace('\\', "/");
    if p.contains("/test/")
        || p.contains("/tests/")
        || p.contains("/qa/")
        || p.contains("/scripts/")
        || p.contains(".gen.")
        || p.contains(".test.")
        || p.contains(".spec.")
        || p.contains("/__tests__/")
        || p.contains("/node_modules/")
        || p.contains("/packages/")
    {
        return false;
    }
    let under_apps = p.contains("/apps/") || p.starts_with("apps/");
    if !under_apps {
        return false;
    }
    p.contains("/src/") || p.contains("/routes/")
}

/// Line (1-based) has `// sail-allow <rule>` or the previous non-empty line does.
pub fn is_sail_allowed(source: &str, line: usize, rule_id: &str) -> bool {
    let needle = format!("sail-allow {rule_id}");
    let lines: Vec<&str> = source.lines().collect();
    if line == 0 {
        return false;
    }
    let idx = line - 1;
    if idx < lines.len() && lines[idx].contains(&needle) {
        return true;
    }
    if idx > 0 {
        let prev = lines[idx - 1].trim();
        if prev.contains(&needle) {
            return true;
        }
    }
    false
}

pub fn byte_offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
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

pub fn finding(
    path: &Path,
    source: &str,
    offset: usize,
    rule: &'static str,
    message: impl Into<String>,
    suggest: impl Into<String>,
) -> Option<Finding> {
    let (line, col) = byte_offset_to_line_col(source, offset);
    if is_sail_allowed(source, line, rule) {
        return None;
    }
    Some(Finding {
        file: path.to_path_buf(),
        line,
        col,
        rule,
        severity: Severity::Error,
        message: message.into(),
        suggest: Some(suggest.into()),
    })
}

/// Empty when path is out of scope — callers return early.
pub fn gate(path: &Path) -> bool {
    is_ambient_app_path(path)
}
