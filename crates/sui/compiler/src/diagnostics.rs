//! Span tracking and structured compile errors.
//!
//! Every IR node carries a `Span` pointing back at the user's source.
//! Errors capture the span, an optional file path, and a one-sentence
//! message so the CLI can render a useful error report.

use std::path::PathBuf;

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
    /// Optional category, e.g. `"<data>"` or the offending expression text.
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
/// surfaces in LSP / agent feedback without failing compilation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagSeverity {
    Error,
    Warning,
}

/// One diagnostic with a resolved **1-based** line/col range. This is the
/// shared currency of [`crate::check_all`] — it merges compiler check-pass
/// errors ([`CompileError`]) and `sui-lint` findings into one stream so an
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
    /// Lint rule id (`sui-for-key`) or check category (`<data>`), when known.
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
}
