//! `sui-no-undeclared-stylesheet` — every `.css` file a pack ships is
//! declared in `sail.config.json`.
//!
//! Why this matters: a `.css` file under a pack's `src/` is invisible to the
//! compiler — the framework sheet is keyed on `:host`, and orphan CSS
//! files that the loader never adopts live outside that contract. They
//! either re-define rules the compiler is already emitting (silent
//! duplication), or they smuggle in styles that authors cannot lint
//! (the css-token rule never sees them). Either way, the seam is the
//! `sail.config.json` declaration: every `.css` the pack owns is named
//! there, and the lint refuses anything else.
//!
//! Detection: walks a pack's `src/` directory, finds every `*.css` file,
//! reads the nearest `sail.config.json` walking up, and reports any file
//! not present in the config's `stylesheets` array. Tests pin both
//! legs of the contract — declared files pass, undeclared ones fail.
//!
//! Exempt: pack files outside `src/` (test fixtures, vitest setup), the
//! framework runtime, and pack-resident `<stem>.sui` host CSS (those
//! ARE compiler-known — they're typed through `element-skin.css`).

use std::path::{Path, PathBuf};

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sail-no-undeclared-stylesheet";

/// Walk a pack's src directory for `.css` files.
fn collect_css_files(src_dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(src_dir) else {
        return out;
    };
    for entry in rd.flatten() {
        let p = entry.path();
        // file_type() does not follow symlinks, so a symlinked `node_modules`
        // dir reads as a symlink and is skipped — the walk stays on pack
        // sources and never enters the isolated-linker dependency graph.
        let Ok(ft) = entry.file_type() else { continue };
        if ft.is_file()
            && p.extension().and_then(|e| e.to_str()) == Some("css")
        {
            out.push(p);
        } else if ft.is_dir() {
            if matches!(
                p.file_name().and_then(|n| n.to_str()),
                Some("node_modules" | "dist" | "target" | ".git" | ".deck" | ".sail-out")
            ) {
                continue;
            }
            out.extend(collect_css_files(&p));
        }
    }
    out
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
    if path.is_dir() {
        return check_dir(path);
    }
    // The dispatch hands us the `.sui` being compiled, never a directory — so an
    // is_dir() guard meant this rule COULD NOT FIRE. It was written, registered,
    // and dead: a stylesheet could be declared in the config and enforced by
    // nothing. Walk up to the package that owns this source (the nearest
    // `sail.config.json`) and lint THAT.
    match path.parent().and_then(crate::config::owning_dir) {
        Some(dir) => check_dir(&dir),
        None => Vec::new(),
    }
}

fn check_dir(src_dir: &Path) -> Vec<Finding> {
    let css_files = collect_css_files(src_dir);
    if css_files.is_empty() {
        return Vec::new();
    }
    let Some(declared) = crate::config::resolve(src_dir).map(|(_, c)| c.declared_stylesheets()) else {
        // No sail.config.json anywhere — every css file is undeclared.
        return css_files
            .into_iter()
            .map(|p| Finding {
                file: p,
                line: 1,
                col: 1,
                rule: RULE_ID,
                severity: Severity::Error,
                message:
                    "no `sail.config.json` enumerates this `.css` file — every stylesheet a pack ships must be declared".to_string(),
                suggest: Some(
                    "add `sail.config.json` with `{ \"stylesheets\": [\"name.css\"] }` next to src/, OR move the CSS into the component's `<style>` block so the compiler knows about it".to_string()
                ),
            })
            .collect();
    };
    let mut findings = Vec::new();
    for css in css_files {
        // PACK-RELATIVE rel — `sail.config.json` and its `stylesheets[]` both
        // live at the package root (the directory we walked up to). A CSS
        // file at `<pack>/src/sui-board/sui-board.css` must strip against
        // `<pack>`, not `<pack>/..`, so the rel is `src/sui-board/sui-board.css`
        // exactly as it appears in the config. The previous form used
        // `src_dir.parent()` here, which produced e.g. `canvas/src/...` — a
        // path the config never wrote, so the rule fired against every css
        // in the only pack with declared sheets (canvas), and the lint
        // surfaced real orphans in packs with `stylesheets: []` only by
        // accident (the declared set was empty so anything was undeclared).
        let rel = css
            .strip_prefix(src_dir)
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
                    "`.css` file `{}` is not declared in `sail.config.json`",
                    rel
                ),
                suggest: Some(
                    "either add it to the `stylesheets` array in sail.config.json, \
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
            std::fs::write(tmp.path().join("sail.config.json"), c).unwrap();
        }
        tmp
    }

    // The contract this module pins: rel is PACK-RELATIVE — strip_prefix on
    // the package root (the directory holding `sail.config.json`), which is
    // how the wiring in `check()` hands `check_dir` its `src_dir`. The
    // earlier form stripped against `src_dir.parent()`; that produced
    // `<packs-root>/<pack>/src/...` rels the config never wrote, so any
    // pack with declared sheets was uniformly flagged (the rule's most
    // dangerous failure mode — passing nothing to a declared sheet looks
    // like enforcement).
    #[test]
    fn declared_stylesheet_passes() {
        let tmp = write_pack(
            &[("src/widget.css", ".x { color: red; }")],
            Some(r#"{"stylesheets": ["src/widget.css"]}"#),
        );
        let findings = check_dir(tmp.path());
        assert!(
            findings.is_empty(),
            "declared css must not be flagged: {findings:?}"
        );
    }

    #[test]
    fn undeclared_stylesheet_reports() {
        let tmp = write_pack(
            &[("src/orphan.css", ".x { color: red; }")],
            Some(r#"{"stylesheets": ["src/other.css"]}"#),
        );
        let findings = check_dir(tmp.path());
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
        let findings = check_dir(tmp.path());
        assert_eq!(
            findings.len(),
            1,
            "no sail.config.json → every css is undeclared"
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
        let findings = check_dir(tmp.path());
        assert!(findings.is_empty(), "declared nested css must pass: {findings:?}");
    }

    // The two ends of the contract on canvas's real layout: a pack with
    // both declared sheets in src/ nested dirs still passes; an undeclared
    // sheet next to a declared one fails. This is the test that was always
    // going to catch the off-by-one-strip bug — it just needed to be
    // written with pack-relative declarations and a pack-root src_dir.
    #[test]
    fn both_declared_sheets_pass() {
        let tmp = write_pack(
            &[
                ("src/sui-board/sui-board.css", ".x { color: red; }"),
                ("src/sui-draw/sui-draw.css", ".y { color: blue; }"),
            ],
            Some(
                r#"{"stylesheets": [
                    "src/sui-board/sui-board.css",
                    "src/sui-draw/sui-draw.css"
                ]}"#,
            ),
        );
        let findings = check_dir(tmp.path());
        assert!(
            findings.is_empty(),
            "both declared sheets must pass: {findings:?}"
        );
    }

    #[test]
    fn undeclared_sheet_among_declared_fails() {
        let tmp = write_pack(
            &[
                ("src/sui-board/sui-board.css", ".x { color: red; }"),
                ("src/sui-draw/sui-draw.css", ".y { color: blue; }"),
                ("src/sui-orphan/sneaky.css", ".z { color: green; }"),
            ],
            Some(
                r#"{"stylesheets": [
                    "src/sui-board/sui-board.css",
                    "src/sui-draw/sui-draw.css"
                ]}"#,
            ),
        );
        let findings = check_dir(tmp.path());
        assert_eq!(
            findings.len(),
            1,
            "exactly one undeclared sheet must be flagged: {findings:?}"
        );
        assert!(findings[0].message.contains("sneaky.css"));
        assert!(findings[0].message.contains("src/sui-orphan/sneaky.css"));
    }

    #[test]
    fn bare_name_declaration_still_recognised() {
        // The historical shorthand: declaring just `name.css` accepts the
        // sheet under any nested directory. Still supported after the
        // pack-relative rel switch.
        let tmp = write_pack(
            &[("src/sui-board/sui-board.css", ".x { color: red; }")],
            Some(r#"{"stylesheets": ["sui-board.css"]}"#),
        );
        let findings = check_dir(tmp.path());
        assert!(
            findings.is_empty(),
            "bare-name declaration must still be honoured: {findings:?}"
        );
    }
}
