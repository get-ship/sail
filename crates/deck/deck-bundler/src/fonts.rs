//! Stage app-vendored fonts (SailTheme/FontAdd) into the build output.
//!
//! Layout under the project (written by ops FontAdd):
//! ```text
//! <project>/fonts/
//!   vendor-manifest.json
//!   fonts.css          # @font-face rules with url('./….woff2')
//!   <family>-….woff2
//! ```
//!
//! The bundler copies the directory to `<out>/fonts/` so a static host serves
//! the same path the CSS references. When `fonts.css` exists it is also
//! linked from `index.html` after the framework sheets (entry import is
//! preferred when the app already pulls CSS via JS; the copy keeps offline
//! specimen/static paths working without a second font pipeline).

use std::path::Path;

use anyhow::{Context, Result};

/// Project-relative fonts directory produced by SailTheme/FontAdd.
pub const APP_FONTS_DIR: &str = "fonts";

/// Copy `<project>/fonts/**` → `<out_dir>/fonts/**` when present.
pub fn stage_app_fonts(project: &Path, out_dir: &Path) -> Result<()> {
    let src = project.join(APP_FONTS_DIR);
    if !src.is_dir() {
        return Ok(());
    }
    let dest = out_dir.join(APP_FONTS_DIR);
    copy_dir_recursive(&src, &dest)
        .with_context(|| format!("copy {} → {}", src.display(), dest.display()))?;
    Ok(())
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<()> {
    std::fs::create_dir_all(dest)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let from = entry.path();
        let to = dest.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else if ty.is_file() {
            std::fs::copy(&from, &to).with_context(|| format!("copy file {}", from.display()))?;
        }
    }
    Ok(())
}

/// Formerly emitted `<link href="/{ns}/fonts/fonts.css">`. That path 404s in
/// dev (`serve_scaffold_asset` rejects nested segments). Fonts load via entry
/// `import './fonts/fonts.css'` instead. Kept as `None` so callers compile;
/// staged `.deck/fonts/**` is for offline render only.
pub fn app_fonts_css_href(_project: &Path, _url_namespace: &str) -> Option<String> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn stages_woff2_and_css() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path().join("app");
        let fonts = project.join("fonts");
        fs::create_dir_all(&fonts).unwrap();
        fs::write(fonts.join("fonts.css"), "@font-face { font-family: X; }\n").unwrap();
        fs::write(fonts.join("x.woff2"), b"woff").unwrap();
        let out = tmp.path().join("out");
        stage_app_fonts(&project, &out).unwrap();
        assert!(out.join("fonts/fonts.css").is_file());
        assert!(out.join("fonts/x.woff2").is_file());
        // Nested fonts href is intentionally not emitted (would 404 on the
        // single-segment asset route). Entry import stages the hashed CSS.
        assert_eq!(app_fonts_css_href(&project, "assets"), None);
    }
}
