//! `sail fmt` — pretty-print `.sail` source from the compiler's real CST.
//!
//! Template regions reprint from `parse::html` nodes (not tree-sitter).
//! `<scope>` / `<controller>` bodies are handed to pinned oxfmt when
//! available (node_modules/.bin → PATH); missing oxfmt degrades with a
//! warning and keeps the body (indent-normalized only).
//!
//! No user format knobs — style is scaffold-pinned. Same entry points are
//! used by CLI, LSP format-on-save, and future codemods.

mod oxfmt;
mod pretty;
mod util;

use crate::diagnostics::CompileError;
use crate::parse::{html, preprocess};

pub use oxfmt::{OxfmtStatus, discover_oxfmt, format_ts_region};

/// Scaffold-pinned format options. Intentionally not user-configurable.
#[derive(Debug, Clone)]
pub struct FormatOpts {
    /// Indent width in spaces (pinned: 2).
    pub indent: usize,
    /// Soft wrap for open tags / attrs (pinned: 100).
    pub max_width: usize,
    /// When false, skip oxfmt even if discovered (tests).
    pub use_oxfmt: bool,
}

impl Default for FormatOpts {
    fn default() -> Self {
        Self {
            indent: 2,
            max_width: 100,
            use_oxfmt: true,
        }
    }
}

/// Result of formatting one source string.
#[derive(Debug, Clone)]
pub struct FormatResult {
    pub source: String,
    /// True when oxfmt was requested but not found (template still formatted).
    pub oxfmt_missing: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum FormatError {
    #[error("{0}")]
    Parse(#[from] CompileError),
}

/// Pretty-print a `.sail` source string (library / LSP / codemod entry).
pub fn format_source(source: &str) -> Result<FormatResult, FormatError> {
    format_source_with(source, &FormatOpts::default())
}

/// Pretty-print with explicit options (tests pin `use_oxfmt: false`).
pub fn format_source_with(source: &str, opts: &FormatOpts) -> Result<FormatResult, FormatError> {
    let prepared = preprocess::prepare_fmt(source);
    let nodes = html::parse_fragment(&prepared)?;
    let mut oxfmt_missing = false;
    let oxfmt_bin = if opts.use_oxfmt {
        match discover_oxfmt(None) {
            OxfmtStatus::Found(p) => Some(p),
            OxfmtStatus::Missing => {
                oxfmt_missing = true;
                None
            }
        }
    } else {
        None
    };
    let printed = pretty::print_document(&nodes, opts, oxfmt_bin.as_deref());
    Ok(FormatResult {
        source: printed,
        oxfmt_missing,
    })
}

/// Whether `source` is already in canonical form (`fmt ∘ id`).
pub fn is_formatted(source: &str, opts: &FormatOpts) -> Result<bool, FormatError> {
    let out = format_source_with(source, opts)?;
    Ok(out.source == source)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts_no_oxfmt() -> FormatOpts {
        FormatOpts {
            use_oxfmt: false,
            ..Default::default()
        }
    }

    #[test]
    fn template_indent_idempotent() {
        let mangled = "<column gap=\"4\">\n<p>hi</p>\n  <row>\n<button type=\"button\">go</button>\n<button type=\"button\">more</button></row></column>\n";
        let once = format_source_with(mangled, &opts_no_oxfmt())
            .expect("fmt")
            .source;
        let twice = format_source_with(&once, &opts_no_oxfmt())
            .expect("fmt2")
            .source;
        assert_eq!(once, twice, "fmt ∘ fmt must be identity:\n{once}");
        assert!(once.contains("<column gap=\"4\">"), "{once}");
        assert!(once.contains("  <p>hi</p>"), "{once}");
        // Two buttons force multi-line row body under indent rules.
        assert!(
            once.contains("<button type=\"button\">go</button>"),
            "{once}"
        );
        assert!(
            once.contains("<button type=\"button\">more</button>"),
            "{once}"
        );
    }

    #[test]
    fn primary_brace_control_round_trips() {
        let src = "<if {loading()}>\n  <span>…</span>\n</if>\n";
        let out = format_source_with(src, &opts_no_oxfmt())
            .expect("fmt")
            .source;
        assert!(
            out.contains("<if {loading()}>"),
            "primary brace must not become @primary=\n{out}"
        );
        let twice = format_source_with(&out, &opts_no_oxfmt())
            .expect("fmt2")
            .source;
        assert_eq!(out, twice);
    }

    #[test]
    fn expr_attr_prefers_unquoted_brace() {
        let src = r#"<button disabled="{a || b}">x</button>"#;
        let out = format_source_with(src, &opts_no_oxfmt())
            .expect("fmt")
            .source;
        assert!(
            out.contains("disabled={a || b}"),
            "canonical form uses unquoted brace attrs:\n{out}"
        );
    }

    #[test]
    fn comment_and_style_statement_preserved() {
        let src = "<!-- head -->\n<div>\nstyle({ box: pad(4) })\n<p>x</p>\n</div>\n";
        let out = format_source_with(src, &opts_no_oxfmt())
            .expect("fmt")
            .source;
        assert!(out.contains("<!-- head -->"), "{out}");
        assert!(out.contains("style({ box: pad(4) })"), "{out}");
        let twice = format_source_with(&out, &opts_no_oxfmt())
            .expect("fmt2")
            .source;
        assert_eq!(out, twice);
    }

    #[test]
    fn controller_region_kept_without_oxfmt() {
        let src = "<controller>\nexport default controller(() => ({ x: signal(0) }))\n</controller>\n\n<p>{x()}</p>\n";
        let out = format_source_with(src, &opts_no_oxfmt())
            .expect("fmt")
            .source;
        assert!(out.contains("<controller>"), "{out}");
        assert!(out.contains("export default controller"), "{out}");
        assert!(out.contains("</controller>"), "{out}");
        assert!(out.contains("<p>{x()}</p>"), "{out}");
        let twice = format_source_with(&out, &opts_no_oxfmt())
            .expect("fmt2")
            .source;
        assert_eq!(out, twice);
    }

    #[test]
    fn void_and_empty_elements() {
        let src = "<div><br/><img src=\"a.png\"/><span></span></div>\n";
        let out = format_source_with(src, &opts_no_oxfmt())
            .expect("fmt")
            .source;
        assert!(out.contains("<br>"), "void as bare tag:\n{out}");
        assert!(out.contains("<img src=\"a.png\">"), "{out}");
        assert!(out.contains("<span></span>"), "{out}");
    }
}
