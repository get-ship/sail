//! `sail-paired-controller` — a `.sail` file's controller is its `<controller>`
//! block. A sibling `<stem>.controller.ts` is a compile error.
//!
//! One spelling. A template whose meaning depends on whether a sibling file
//! exists on disk cannot be read (by a human or an agent) without stat'ing the
//! filesystem, and it loses its scope silently when the sibling is renamed or
//! deleted. Reuse belongs in a composition module the `<controller>` imports.

use std::path::Path;

use crate::diagnostics::CompileError;

pub const RULE_ID: &str = "sail-paired-controller";

/// The sibling controller path for a `.sail` source, by the retired convention:
/// `page.sail` → `controller.ts`, `<stem>.sail` → `<stem>.controller.ts`.
pub fn sibling_path(source_path: &Path) -> Option<std::path::PathBuf> {
    let stem = source_path.file_stem()?.to_str()?;
    let dir = source_path.parent().unwrap_or(Path::new("."));
    Some(if stem == "page" {
        dir.join("controller.ts")
    } else {
        dir.join(format!("{stem}.controller.ts"))
    })
}

pub fn check(source: &str, source_path: Option<&Path>) -> Result<(), CompileError> {
    let Some(path) = source_path else {
        return Ok(());
    };
    let Some(sibling) = sibling_path(path) else {
        return Ok(());
    };
    if !sibling.is_file() {
        return Ok(());
    }
    let has_inline = crate::parse::preprocess::extract_controller_block(source).1.is_some();
    let detail = if has_inline {
        "this source already declares a `<controller>` block, so the sibling is dead code"
    } else {
        "move its body into a `<controller>` block in this file"
    };
    Err(CompileError::new(format!(
        "`{}` — a paired controller file is not the language: a `.sail` file's controller is its \
         `<controller>` block. Delete the sibling and {detail}. Reusable logic belongs in a \
         composition module the block imports (`import {{ createTable }} from './engine'`).",
        sibling.display()
    ))
    .with_category(RULE_ID.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sibling_file_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let sail = dir.path().join("sail-x.sail");
        std::fs::write(&sail, "<span></span>").unwrap();
        std::fs::write(dir.path().join("sail-x.controller.ts"), "export default {}").unwrap();
        let err = check("<span></span>", Some(&sail)).unwrap_err();
        assert_eq!(err.category.as_deref(), Some(RULE_ID));
    }

    #[test]
    fn page_convention_is_covered() {
        let dir = tempfile::tempdir().unwrap();
        let sail = dir.path().join("page.sail");
        std::fs::write(&sail, "<span></span>").unwrap();
        std::fs::write(dir.path().join("controller.ts"), "export default {}").unwrap();
        assert!(check("<span></span>", Some(&sail)).is_err());
    }

    #[test]
    fn inline_block_alone_passes() {
        let dir = tempfile::tempdir().unwrap();
        let sail = dir.path().join("sail-x.sail");
        let src = "<controller>export default controller((ctx) => ({}))</controller>\n<span></span>";
        std::fs::write(&sail, src).unwrap();
        check(src, Some(&sail)).unwrap();
    }

    #[test]
    fn no_sibling_no_error() {
        let dir = tempfile::tempdir().unwrap();
        let sail = dir.path().join("sail-x.sail");
        std::fs::write(&sail, "<span></span>").unwrap();
        check("<span></span>", Some(&sail)).unwrap();
    }
}
