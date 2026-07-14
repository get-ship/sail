//! `sui-no-undeclared-stylesheet` — every `.css` file a pack ships is
//! declared in `sui.config.json`.
//!
//! Why this matters: a `.css` file under a pack's `src/` is invisible to the
//! compiler — the framework sheet is keyed on `:host`, and orphan CSS
//! files that the loader never adopts live outside that contract. They
//! either re-define rules the compiler is already emitting (silent
//! duplication), or they smuggle in styles that authors cannot lint
//! (the css-token rule never sees them). Either way, the seam is the
//! `sui.config.json` declaration: every `.css` the pack owns is named
//! there, and the lint refuses anything else.
//!
//! Detection: walks a pack's `src/` directory, finds every `*.css` file,
//! reads the nearest `sui.config.json` walking up, and reports any file
//! not present in the config's `stylesheets` array. Tests pin both
//! legs of the contract — declared files pass, undeclared ones fail.
//!
//! Exempt: pack files outside `src/` (test fixtures, vitest setup), the
//! framework runtime, and pack-resident `<stem>.sui` host CSS (those
//! ARE compiler-known — they're typed through `element-skin.css`).

use std::path::{Path, PathBuf};

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sui-no-undeclared-stylesheet";

/// Walk a pack's src directory for `.css` files.
fn collect_css_files(src_dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(src_dir) else {
        return out;
    };
    for entry in rd.flatten() {
        let p = entry.path();
        if p.is_file()
            && p.extension().and_then(|e| e.to_str()) == Some("css")
        {
            out.push(p);
        } else if p.is_dir() {
            // Recurse one level — pack src trees are flat (controllers,
            // utilities, helpers), but we don't forbid subdirs.
            out.extend(collect_css_files(&p));
        }
    }
    out
}

/// Read the nearest `sui.config.json` walking UP from the css file's dir.
/// Returns `Some(declared_set)` if a config was read, `None` if no config
/// exists in any parent (so every file is reported). The JSON grammar
/// is small: `{ "stylesheets": ["name-or-relpath", …] }`.
fn read_declared_css(css_dir: &Path) -> Option<std::collections::HashSet<String>> {
    let mut dir = css_dir.to_path_buf();
    loop {
        let cfg = dir.join("sui.config.json");
        if let Ok(text) = std::fs::read_to_string(&cfg) {
            // Hand-rolled (the lint is WASM-safe-free; we never depend on
            // serde). The grammar accepts: an array under `stylesheets`
            // whose elements are quoted strings (with `-` for clarity).
            let mut set = std::collections::HashSet::new();
            // Find `"stylesheets"` and walk the array.
            if let Some(idx) = text.find("\"stylesheets\"") {
                if let Some(arr) = text[idx..]
                    .split('[')
                    .nth(1)
                    .and_then(|s| s.split(']').next())
                {
                    let mut buf = String::new();
                    let mut in_str = false;
                    for ch in arr.chars() {
                        match ch {
                            '"' => {
                                if in_str {
                                    if !buf.is_empty() {
                                        set.insert(buf.clone());
                                        buf.clear();
                                    }
                                    in_str = false;
                                } else {
                                    in_str = true;
                                }
                            }
                            _ => {
                                if in_str {
                                    buf.push(ch);
                                }
                            }
                        }
                    }
                }
            }
            return Some(set);
        }
        if !dir.pop() {
            return None;
        }
    }
}

fn byte_offset_to_line(source: &str, offset: usize) -> (usize, usize) {
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

pub fn check(path: &Path, _source: &str) -> Vec<Finding> {
    // We lint by DIRECTORY, but the dispatch entry gives a file path.
    // The convention: the lint is invoked once per pack src directory;
    // here we accept the css file's parent and report.
    if path.is_dir() {
        return check_dir(path);
    }
    Vec::new()
}

fn check_dir(src_dir: &Path) -> Vec<Finding> {
    let css_files = collect_css_files(src_dir);
    if css_files.is_empty() {
        return Vec::new();
    }
    let Some(declared) = read_declared_css(src_dir) else {
        // No sui.config.json anywhere — every css file is undeclared.
        return css_files
            .into_iter()
            .map(|p| Finding {
                file: p,
                line: 1,
                col: 1,
                rule: RULE_ID,
                severity: Severity::Error,
                message:
                    "no `sui.config.json` enumerates this `.css` file — every stylesheet a pack ships must be declared".to_string(),
                suggest: Some(
                    "add `sui.config.json` with `{ \"stylesheets\": [\"name.css\"] }` next to src/, OR move the CSS into the component's `<style>` block so the compiler knows about it".to_string()
                ),
            })
            .collect();
    };
    let mut findings = Vec::new();
    for css in css_files {
        let rel = css
            .strip_prefix(src_dir.parent().unwrap_or(src_dir))
            .unwrap_or(&css)
            .to_string_lossy()
            .trim_start_matches('/')
            .to_string();
        // The declaration can be either the bare file name (`sui-board.css`)
        // or a path (`src/sui-board.css`). Accept either.
        let bare_name = css
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_default();
        if !declared.contains(&rel) && !declared.contains(&bare_name) {
            findings.push(Finding {
                file: css,
                line: 1,
                col: 1,
                rule: RULE_ID,
                severity: Severity::Error,
                message: format!(
                    "`.css` file `{}` is not declared in `sui.config.json`",
                    rel
                ),
                suggest: Some(
                    "either add it to the `stylesheets` array in sui.config.json, \
                     or inline it into the component's <style> block (the compiler \
                     then knows about it and the css-token rule applies)"
                        .to_string(),
                ),
            });
        }
    }
    findings
}

// Silence the unused-warning for the helper — it's kept for the
// (future) per-file diagnostic path.
#[allow(dead_code)]
fn _linecol_helper(path: &Path, offset: usize) -> (usize, usize) {
    let Ok(text) = std::fs::read_to_string(path) else {
        return (0, 0);
    };
    byte_offset_to_line(&text, offset)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_pack(files: &[(&str, &str)], config: Option<&str>) -> tempfile::TempDir {
        let tmp = tempfile::tempdir().expect("tmpdir");
        for (rel, body) in files {
            let p = tmp.path().join(rel);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(&p, body).unwrap();
        }
        if let Some(c) = config {
            std::fs::write(tmp.path().join("sui.config.json"), c).unwrap();
        }
        tmp
    }

    #[test]
    fn declared_stylesheet_passes() {
        let tmp = write_pack(
            &[("src/widget.css", ".x { color: red; }")],
            Some(r#"{"stylesheets": ["widget.css"]}"#),
        );
        let findings = check_dir(&tmp.path().join("src"));
        assert!(
            findings.is_empty(),
            "declared css must not be flagged: {findings:?}"
        );
    }

    #[test]
    fn undeclared_stylesheet_reports() {
        let tmp = write_pack(
            &[("src/orphan.css", ".x { color: red; }")],
            Some(r#"{"stylesheets": ["other.css"]}"#),
        );
        let findings = check_dir(&tmp.path().join("src"));
        assert_eq!(findings.len(), 1, "undeclared css must be flagged: {findings:?}");
        assert_eq!(findings[0].rule, RULE_ID);
        assert!(findings[0].message.contains("orphan.css"));
    }

    #[test]
    fn no_config_at_all_reports_every_css() {
        let tmp = write_pack(
            &[("src/widget.css", ".x { color: red; }")],
            None,
        );
        let findings = check_dir(&tmp.path().join("src"));
        assert_eq!(
            findings.len(),
            1,
            "no sui.config.json → every css is undeclared"
        );
    }

    #[test]
    fn nested_dir_css_also_walks() {
        let tmp = write_pack(
            &[
                ("src/widget.css", ".x { color: red; }"),
                ("src/inner/extra.css", ".y { color: blue; }"),
            ],
            Some(
                r#"{"stylesheets": ["src/widget.css", "src/inner/extra.css"]}"#,
            ),
        );
        let findings = check_dir(&tmp.path().join("src"));
        assert!(findings.is_empty(), "declared nested css must pass: {findings:?}");
    }
}
