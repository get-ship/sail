//! Indent-aware string writer. Keeps the JS emitter readable.

#[derive(Debug, Default)]
pub struct Writer {
    buf: String,
    indent: usize,
}

impl Writer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_str(&mut self, s: &str) {
        self.buf.push_str(s);
    }

    pub fn nl(&mut self) {
        self.buf.push('\n');
    }

    pub fn indent(&mut self) {
        for _ in 0..self.indent {
            self.buf.push_str("  ");
        }
    }

    pub fn line(&mut self, s: &str) {
        self.indent();
        self.buf.push_str(s);
        self.buf.push('\n');
    }

    pub fn enter(&mut self) {
        self.indent += 1;
    }

    pub fn leave(&mut self) {
        self.indent = self.indent.saturating_sub(1);
    }

    pub fn into_string(self) -> String {
        self.buf
    }

    /// Current buffer byte-length. Used by control-flow emitters to
    /// snapshot the boundary-hoist buffer before walking a nested
    /// region, then split off the new content via [`split_off_from`]
    /// so it can be relocated (e.g. inlined inside a for-iteration's
    /// IIFE so nested boundaries close over the iteration's `$scope`).
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    /// Split the buffer at `at` and return the suffix; the writer
    /// keeps the prefix. Caller is responsible for keeping `at` on a
    /// UTF-8 char boundary (this is true when `at` came from a prior
    /// `len()` call against the same buffer).
    pub fn split_off_from(&mut self, at: usize) -> String {
        self.buf.split_off(at)
    }
}

/// Quote a string as a JS double-quoted literal.
pub fn js_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
