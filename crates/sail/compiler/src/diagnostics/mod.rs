//! Span tracking and structured compile errors.
//!
//! Every IR node carries a `Span` pointing back at the user's source.
//! Errors capture the span, an optional file path, and a one-sentence
//! message so the CLI can render a useful error report.
//!
//! DiagSpec registry skeleton (SPEC-LINTS-1 §4 Phase 1): see [`registry`].

use std::path::PathBuf;

pub mod registry;

pub use registry::{DiagSpec, Jurisdiction, lookup, registry};

/// A byte range into the original source. Inclusive start, exclusive end.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn point(at: usize) -> Self {
        Self { start: at, end: at }
    }
}

/// Translate a byte offset to (line, column), both 1-based.
pub fn line_col(source: &str, offset: usize) -> (usize, usize) {
    let clamped = offset.min(source.len());
    let mut line = 1usize;
    let mut col = 1usize;
    for (i, ch) in source.char_indices() {
        if i >= clamped {
            return (line, col);
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

/// A single compile-time error.
#[derive(Debug, Clone, thiserror::Error)]
#[error("{message}")]
pub struct CompileError {
    pub message: String,
    pub span: Option<Span>,
    pub source_path: Option<PathBuf>,
    /// Optional category, e.g. a check category or the offending expression text.
    pub category: Option<String>,
}

impl CompileError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            span: None,
            source_path: None,
            category: None,
        }
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    pub fn with_path(mut self, path: PathBuf) -> Self {
        self.source_path = Some(path);
        self
    }

    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    /// Human-readable rendering. Used by the CLI and snapshot tests.
    pub fn render(&self, source: &str) -> String {
        let mut out = String::new();
        if let Some(p) = &self.source_path {
            out.push_str(&format!("{}: ", p.display()));
        }
        if let Some(s) = self.span {
            let (line, col) = line_col(source, s.start);
            out.push_str(&format!("line {line} col {col}: "));
        }
        if let Some(cat) = &self.category {
            out.push_str(&format!("[{cat}] "));
        }
        out.push_str(&self.message);
        out
    }
}

/// Severity of a [`Diagnostic`]. `Error` blocks the build; `Warning`
/// surfaces in LSP / agent feedback without failing compilation;
/// `Fix` is an auto-repair diagnostic emitted by the compose pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagSeverity {
    Error,
    Warning,
    Fix,
}

/// SPEC-TOOLING-1 §WI6 — when a diagnostic stream is computed.
///
/// Doctrine:
/// - **OnType** — parse + vocabulary + semantic checks (LSP `didOpen` /
///   `didChange`). Never runs the taffy layout / compose pass.
/// - **OnSave** — full stream including taffy layout + compose (LSP
///   `didSave`, `sail check`, MCP `sail_check`, compile gate).
///
/// Render stays **on-demand** outside this enum (MCP `sail_render` / CLI) —
/// never part of the push-diagnostic pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DiagnosticTier {
    /// Parse + vocabulary + semantic checks. No taffy.
    OnType,
    /// Full diagnostics including taffy layout + compose.
    #[default]
    OnSave,
}

impl DiagnosticTier {
    /// Whether this tier runs the taffy layout (and taffy-backed compose) pass.
    pub const fn includes_layout(self) -> bool {
        matches!(self, Self::OnSave)
    }
}

/// One diagnostic with a resolved **1-based** line/col range. This is the
/// shared currency of [`crate::check_all`] — it merges compiler check-pass
/// errors ([`CompileError`]) and `sail-lint` findings into one stream so an
/// LSP server or MCP tool reports every issue at once, not just the first.
///
/// Positions are 1-based (matching the rest of the compiler + CLI output).
/// The LSP transport subtracts 1 to produce 0-based protocol positions.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Diagnostic {
    pub line: usize,
    pub col: usize,
    pub end_line: usize,
    pub end_col: usize,
    pub severity: DiagSeverity,
    pub message: String,
    /// Lint rule id (`sail-for-key`) or check category, when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule: Option<String>,
}

impl Diagnostic {
    /// Build from a compiler [`CompileError`], resolving its byte span to a
    /// 1-based line/col range against `source`. Span-less errors point at 1:1.
    pub fn from_compile_error(err: &CompileError, source: &str) -> Self {
        let (line, col, end_line, end_col) = match err.span {
            Some(s) => {
                let (l, c) = line_col(source, s.start);
                let (el, ec) = line_col(source, s.end);
                (l, c, el, ec)
            }
            None => (1, 1, 1, 1),
        };
        Self {
            line,
            col,
            end_line,
            end_col,
            severity: DiagSeverity::Error,
            message: err.message.clone(),
            rule: err.category.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_col_counts_newlines() {
        let src = "abc\ndef\nghi";
        assert_eq!(line_col(src, 0), (1, 1));
        assert_eq!(line_col(src, 4), (2, 1));
        assert_eq!(line_col(src, 8), (3, 1));
    }

    #[test]
    fn line_col_clamps() {
        let src = "abc";
        assert_eq!(line_col(src, 99), (1, 4));
    }

    #[test]
    fn render_includes_path_and_position() {
        let err = CompileError::new("oops")
            .with_span(Span::new(4, 5))
            .with_path(PathBuf::from("foo.set"))
            .with_category("expr");
        let rendered = err.render("abc\ndef");
        assert!(rendered.contains("foo.set"));
        assert!(rendered.contains("line 2"));
        assert!(rendered.contains("[expr]"));
        assert!(rendered.contains("oops"));
    }

    #[test]
    fn diagnostic_tier_layout_gate() {
        assert!(!DiagnosticTier::OnType.includes_layout());
        assert!(DiagnosticTier::OnSave.includes_layout());
        assert!(DiagnosticTier::default().includes_layout());
    }
}
