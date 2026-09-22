//! `Parser` — recursive-descent walk over the preprocessed source.

use crate::diagnostics::{CompileError, Span};

use super::{Attr, Comment, Element, Node, RAW_TEXT_TAGS, Text, decode_entities};

const RULE_ID: &str = "sail-html-parse";
pub(super) struct Parser<'a> {
    src: &'a str,
    bytes: &'a [u8],
    i: usize,
}

impl<'a> Parser<'a> {
    pub(super) fn new(src: &'a str) -> Self {
        Self {
            src,
            bytes: src.as_bytes(),
            i: 0,
        }
    }

    pub(super) fn parse_children(
        &mut self,
        parent_tag: Option<&str>,
    ) -> Result<Vec<Node>, CompileError> {
        let mut out = Vec::new();
        while self.i < self.bytes.len() {
            if self.peek2() == Some((b'<', b'/')) {
                let close = self.peek_close_tag()?;
                if let Some(parent) = parent_tag
                    && close.eq_ignore_ascii_case(parent)
                {
                    return Ok(out);
                }
                if parent_tag.is_none() {
                    self.skip_until(b'>');
                    continue;
                }
                return Err(CompileError::new(format!(
                    "unexpected </{close}> while inside <{}>",
                    parent_tag.unwrap_or("?")
                ))
                .with_span(Span::point(self.i))
                .with_category(RULE_ID));
            }
            if self.starts_with("<!--") {
                out.push(Node::Comment(self.parse_comment()?));
                continue;
            }
            if self.starts_with("<!") {
                self.skip_until(b'>');
                continue;
            }
            if self.bytes[self.i] == b'<' && self.is_tag_start_at(self.i + 1) {
                out.push(Node::Element(self.parse_element()?));
                continue;
            }
            // Text run. Inside an unclosed `{…}` arm/expr, tags are text so
            // match/if arms can carry subtrees: `{ok as x => <p>{x}</p>}`.
            // Outside braces, stop at the next real tag start (stray `<` that
            // is not a tag — e.g. `x < y` in an expression already handled
            // via brace depth — is still consumed as text so we don't spin).
            let start = self.i;
            if self.bytes[self.i] == b'<' {
                self.i += 1;
            }
            let mut brace_depth = 0i32;
            let mut str_quote: Option<u8> = None;
            while self.i < self.bytes.len() {
                let c = self.bytes[self.i];
                if let Some(q) = str_quote {
                    if c == b'\\' {
                        self.i = (self.i + 2).min(self.bytes.len());
                        continue;
                    }
                    if c == q {
                        str_quote = None;
                    }
                    self.i += 1;
                    continue;
                }
                // Quote-awareness only applies inside `{…}` expression arms so
                // JS string literals can hide tags. Outside braces, apostrophes
                // in prose (`Couldn't`, `it's`) are ordinary text — treating
                // them as string delimiters swallows the closing tag.
                if brace_depth > 0 && (c == b'"' || c == b'\'' || c == b'`') {
                    str_quote = Some(c);
                    self.i += 1;
                    continue;
                }
                if c == b'{' {
                    brace_depth += 1;
                    self.i += 1;
                    continue;
                }
                if c == b'}' {
                    if brace_depth > 0 {
                        brace_depth -= 1;
                    }
                    self.i += 1;
                    continue;
                }
                if c == b'<' && brace_depth == 0 {
                    break;
                }
                self.i += 1;
            }
            // Don't decode HTML entities here — the text chunk may contain
            // `{...}` expression markers whose bodies are JS source, not
            // HTML content. Decoding `&deg;` to `°` inside an expression
            // body produces multi-byte chars that downstream byte-walkers
            // (lex.rs, expr_subset.rs, expr_rewrite.rs) historically can't
            // handle. The template tokenizer decodes Text-token bodies
            // explicitly after expression splitting; expressions stay raw.
            let data = self.src[start..self.i].to_string();
            out.push(Node::Text(Text {
                data,
                span: Span::new(start, self.i),
            }));
        }
        if let Some(parent) = parent_tag {
            return Err(CompileError::new(format!("unterminated <{parent}>"))
                .with_span(Span::point(self.bytes.len()))
                .with_category(RULE_ID));
        }
        Ok(out)
    }

    fn parse_comment(&mut self) -> Result<Comment, CompileError> {
        let start = self.i;
        self.i += 4; // `<!--`
        let body_start = self.i;
        while self.i + 3 <= self.bytes.len() {
            if &self.bytes[self.i..self.i + 3] == b"-->" {
                let body = self.src[body_start..self.i].to_string();
                self.i += 3;
                return Ok(Comment {
                    data: body,
                    span: Span::new(start, self.i),
                });
            }
            self.i += 1;
        }
        Err(CompileError::new("unterminated <!-- comment")
            .with_span(Span::point(start))
            .with_category(RULE_ID))
    }

    fn parse_element(&mut self) -> Result<Element, CompileError> {
        let start = self.i;
        self.i += 1;
        let tag_start = self.i;
        while self.i < self.bytes.len() && is_tag_name_byte(self.bytes[self.i]) {
            self.i += 1;
        }
        if self.i == tag_start {
            return Err(CompileError::new("expected tag name")
                .with_span(Span::point(start))
                .with_category(RULE_ID));
        }
        let tag = self.src[tag_start..self.i].to_ascii_lowercase();
        let attributes = self.parse_attrs()?;
        let self_closed = self.consume_tag_end()?;
        if self_closed || crate::l1_manifest::is_author_void(&tag) {
            if !self_closed {
                self.consume_optional_close(&tag);
            }
            return Ok(Element {
                tag,
                attributes,
                children: Vec::new(),
                span: Span::new(start, self.i),
            });
        }
        let children = if RAW_TEXT_TAGS.contains(&tag.as_str()) {
            let raw = self.consume_raw_text(&tag);
            if raw.data.is_empty() {
                Vec::new()
            } else {
                vec![Node::Text(raw)]
            }
        } else {
            self.parse_children(Some(&tag))?
        };
        if self.peek2() != Some((b'<', b'/')) {
            return Err(CompileError::new(format!("missing </{tag}>"))
                .with_span(Span::point(self.i))
                .with_category(RULE_ID));
        }
        let close_at = self.i;
        self.i += 2;
        let close_name_start = self.i;
        while self.i < self.bytes.len() && is_tag_name_byte(self.bytes[self.i]) {
            self.i += 1;
        }
        let close_name = self.src[close_name_start..self.i].to_ascii_lowercase();
        if close_name != tag {
            return Err(CompileError::new(format!(
                "mismatched close tag </{close_name}> for <{tag}>"
            ))
            .with_span(Span::point(close_at))
            .with_category(RULE_ID));
        }
        self.skip_ws();
        if self.i >= self.bytes.len() || self.bytes[self.i] != b'>' {
            return Err(CompileError::new(format!("malformed </{tag}>"))
                .with_span(Span::point(close_at))
                .with_category(RULE_ID));
        }
        self.i += 1;
        Ok(Element {
            tag,
            attributes,
            children,
            span: Span::new(start, self.i),
        })
    }

    fn parse_attrs(&mut self) -> Result<Vec<Attr>, CompileError> {
        let mut out = Vec::new();
        loop {
            self.skip_ws();
            if self.i >= self.bytes.len() {
                break;
            }
            let c = self.bytes[self.i];
            if c == b'>' || c == b'/' {
                break;
            }
            // Brace-primary attribute: `<if {open()}>`, `<match {status()}>`,
            // `<for {items() as item}>`. Stored as name `@primary` with the
            // raw `{…}` value (including braces) so control-flow can resolve it
            // without a `cond=` / `on=` / `of=` name.
            if c == b'{' {
                let start = self.i;
                let (raw, vs) = self.read_brace_group()?;
                out.push(Attr {
                    name: PRIMARY_BRACE_ATTR.to_string(),
                    value: raw,
                    span: Span::new(start, self.i),
                    value_span: vs,
                });
                continue;
            }
            // Tag-position Style call: `style(…)` (+ `.when` chains). Stored as
            // name `style` with the full call text so author/fmt paths keep it.
            if self.src[self.i..].starts_with("style") {
                let after = self.i + 5;
                let boundary = after >= self.bytes.len()
                    || (!self.bytes[after].is_ascii_alphanumeric()
                        && self.bytes[after] != b'_'
                        && self.bytes[after] != b'$'
                        && self.bytes[after] != b'-'
                        && self.bytes[after] != b':');
                if boundary {
                    let mut k = after;
                    while k < self.bytes.len() && self.bytes[k].is_ascii_whitespace() {
                        k += 1;
                    }
                    if k < self.bytes.len() && self.bytes[k] == b'(' {
                        let start = self.i;
                        // consume balanced style expr from "style"
                        if let Some(end) = consume_style_call(self.src, start) {
                            let raw = self.src[start..end].to_string();
                            self.i = end;
                            out.push(Attr {
                                name: "style".to_string(),
                                value: raw,
                                span: Span::new(start, end),
                                value_span: Span::new(start, end),
                            });
                            continue;
                        }
                    }
                }
            }
            let name_start = self.i;
            while self.i < self.bytes.len() && is_attr_name_byte(self.bytes[self.i]) {
                self.i += 1;
            }
            if self.i == name_start {
                return Err(CompileError::new(format!(
                    "unexpected character `{}` while parsing attributes",
                    c as char
                ))
                .with_span(Span::point(self.i))
                .with_category(RULE_ID));
            }
            // Lowercase mirrors DOMParser's behavior — EXCEPT `use:<action>`,
            // whose suffix is a case-sensitive JS identifier (the action/hook in
            // scope, e.g. `use:clickOutside`). Preserve the suffix's case so it
            // resolves; the `use:` prefix normalizes to lowercase.
            let raw = &self.src[name_start..self.i];
            // Compare the `use:` prefix on BYTES, not a `str` slice: an
            // attribute name may contain non-ASCII (is_attr_name_byte admits
            // any non-delimiter byte), so `raw[..4]` could slice mid-char and
            // panic ("byte index 4 is not a char boundary"). The byte compare
            // is boundary-safe, and reaching `&raw[4..]` implies bytes 0..4 are
            // the ASCII "use:" prefix — a valid char boundary at 4.
            let name = if raw
                .as_bytes()
                .get(..4)
                .is_some_and(|p| p.eq_ignore_ascii_case(b"use:"))
            {
                format!("use:{}", &raw[4..])
            } else {
                raw.to_ascii_lowercase()
            };
            self.skip_ws();
            if self.i >= self.bytes.len() || self.bytes[self.i] != b'=' {
                let span = Span::new(name_start, self.i);
                out.push(Attr {
                    name,
                    value: String::new(),
                    span,
                    value_span: Span::new(self.i, self.i),
                });
                continue;
            }
            self.i += 1;
            self.skip_ws();
            let (value, vs) = if self.peek_byte() == Some(b'"') {
                self.read_quoted(b'"')?
            } else if self.peek_byte() == Some(b'\'') {
                self.read_quoted(b'\'')?
            } else {
                self.read_unquoted()
            };
            // Attribute values are kept raw when they're an expression
            // (`name="{expr}"`) — the body is JS source. Plain literal
            // values are decoded as HTML so `href="https://x?a=1&amp;b=2"`
            // still produces a valid URL.
            let trimmed = value.trim();
            let decoded = if trimmed.starts_with('{') && trimmed.ends_with('}') {
                value
            } else {
                decode_entities(&value)
            };
            out.push(Attr {
                name,
                value: decoded,
                span: Span::new(name_start, self.i),
                value_span: vs,
            });
        }
        Ok(out)
    }

    fn consume_tag_end(&mut self) -> Result<bool, CompileError> {
        self.skip_ws();
        let mut self_closed = false;
        if self.peek_byte() == Some(b'/') {
            self_closed = true;
            self.i += 1;
            self.skip_ws();
        }
        if self.i >= self.bytes.len() || self.bytes[self.i] != b'>' {
            return Err(CompileError::new("expected `>`")
                .with_span(Span::point(self.i))
                .with_category(RULE_ID));
        }
        self.i += 1;
        Ok(self_closed)
    }

    /// Old corpus wrote `<separator></separator>`. Author-void tags do not
    /// take children; a matching closer is ignored so those files still parse.
    fn consume_optional_close(&mut self, tag: &str) {
        self.skip_ws();
        if self.peek2() != Some((b'<', b'/')) {
            return;
        }
        let save = self.i;
        self.i += 2;
        let name_start = self.i;
        while self.i < self.bytes.len() && is_tag_name_byte(self.bytes[self.i]) {
            self.i += 1;
        }
        let name = self.src[name_start..self.i].to_ascii_lowercase();
        self.skip_ws();
        if name == tag && self.i < self.bytes.len() && self.bytes[self.i] == b'>' {
            self.i += 1;
            return;
        }
        self.i = save;
    }

    fn consume_raw_text(&mut self, tag: &str) -> Text {
        let needle_lower = format!("</{tag}");
        let start = self.i;
        while self.i < self.bytes.len() {
            if self.bytes[self.i] == b'<' && self.starts_with_ci(&needle_lower) {
                break;
            }
            self.i += 1;
        }
        Text {
            data: self.src[start..self.i].to_string(),
            span: Span::new(start, self.i),
        }
    }

    fn read_quoted(&mut self, q: u8) -> Result<(String, Span), CompileError> {
        let qstart = self.i;
        self.i += 1;
        let value_start = self.i;
        while self.i < self.bytes.len() && self.bytes[self.i] != q {
            self.i += 1;
        }
        if self.i >= self.bytes.len() {
            return Err(CompileError::new("unterminated attribute value")
                .with_span(Span::point(qstart))
                .with_category(RULE_ID));
        }
        let v = self.src[value_start..self.i].to_string();
        let vs = Span::new(value_start, self.i);
        self.i += 1;
        Ok((v, vs))
    }

    fn read_unquoted(&mut self) -> (String, Span) {
        let start = self.i;
        while self.i < self.bytes.len() {
            let c = self.bytes[self.i];
            if c.is_ascii_whitespace() || c == b'>' || c == b'/' {
                break;
            }
            self.i += 1;
        }
        (
            self.src[start..self.i].to_string(),
            Span::new(start, self.i),
        )
    }

    /// Read a balanced `{…}` group (outer braces included in the returned
    /// string). Nesting is counted; string/template contents are **not**
    /// string-aware (control-flow predicates rarely need `{` inside quotes).
    fn read_brace_group(&mut self) -> Result<(String, Span), CompileError> {
        debug_assert_eq!(self.peek_byte(), Some(b'{'));
        let start = self.i;
        self.i += 1;
        let mut depth = 1usize;
        while self.i < self.bytes.len() {
            match self.bytes[self.i] {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        self.i += 1;
                        let end = self.i;
                        return Ok((self.src[start..end].to_string(), Span::new(start, end)));
                    }
                }
                _ => {}
            }
            self.i += 1;
        }
        Err(CompileError::new("unterminated `{…}` attribute expression")
            .with_span(Span::point(start))
            .with_category(RULE_ID))
    }

    fn skip_ws(&mut self) {
        while self.i < self.bytes.len() && self.bytes[self.i].is_ascii_whitespace() {
            self.i += 1;
        }
    }
    fn skip_until(&mut self, b: u8) {
        while self.i < self.bytes.len() && self.bytes[self.i] != b {
            self.i += 1;
        }
        if self.i < self.bytes.len() {
            self.i += 1;
        }
    }
    fn peek_byte(&self) -> Option<u8> {
        self.bytes.get(self.i).copied()
    }
    fn peek2(&self) -> Option<(u8, u8)> {
        if self.i + 1 < self.bytes.len() {
            Some((self.bytes[self.i], self.bytes[self.i + 1]))
        } else {
            None
        }
    }
    fn peek_close_tag(&self) -> Result<String, CompileError> {
        let mut j = self.i + 2;
        let start = j;
        while j < self.bytes.len() && is_tag_name_byte(self.bytes[j]) {
            j += 1;
        }
        if j == start {
            return Err(CompileError::new("malformed close tag")
                .with_span(Span::point(self.i))
                .with_category(RULE_ID));
        }
        Ok(self.src[start..j].to_ascii_lowercase())
    }
    fn starts_with(&self, s: &str) -> bool {
        self.src[self.i..].starts_with(s)
    }
    fn starts_with_ci(&self, s: &str) -> bool {
        let rest = &self.src[self.i..];
        rest.len() >= s.len()
            && rest.is_char_boundary(s.len())
            && rest[..s.len()].eq_ignore_ascii_case(s)
    }
    fn is_tag_start_at(&self, idx: usize) -> bool {
        idx < self.bytes.len() && self.bytes[idx].is_ascii_alphabetic()
    }
}

fn is_tag_name_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b':'
}
fn is_attr_name_byte(b: u8) -> bool {
    // `{` / `}` are brace-primary attrs (see `parse_attrs`), not name chars.
    !b.is_ascii_whitespace()
        && b != b'='
        && b != b'>'
        && b != b'/'
        && b != b'"'
        && b != b'\''
        && b != b'{'
        && b != b'}'
}

/// Synthetic attribute name for brace-primary expressions on open tags
/// (`<if {expr}>`). Public so control-flow lowering can resolve it.
pub const PRIMARY_BRACE_ATTR: &str = "@primary";

#[cfg(test)]
mod tests {
    use super::super::parse_fragment;

    #[test]
    fn raw_text_with_multibyte_after_lt_does_not_panic() {
        // A `<` inside a <style>/<script> body followed by a multi-byte
        // UTF-8 char (here U+2500 BOX DRAWINGS LIGHT HORIZONTAL, 3 bytes)
        // used to panic in `starts_with_ci`: it sliced
        // `rest[..needle.len()]` without checking that the byte index
        // landed on a char boundary. Surfaced by libfuzzer in
        // crates/sail/compiler/fuzz/ — minimized input was `<style><-──`.
        let _ = parse_fragment("<style><-──");
        let _ = parse_fragment("<script>x<─</script>");
        let _ = parse_fragment("<style>a<bc</style>");
    }

    #[test]
    fn attr_name_with_multibyte_at_prefix_boundary_does_not_panic() {
        // An attribute name whose 4th byte lands inside a multi-byte UTF-8
        // char used to panic when the `use:`-prefix check did `raw[..4]` on
        // the str slice: `<a abc—>` — name `abc—` is 3 ASCII bytes + `—`
        // (U+2014, 3 bytes), so byte index 4 is mid-char. Found by the
        // `bindings_build` fuzz target. Must yield a diagnostic or parse,
        // never panic.
        let _ = parse_fragment("<a abc—>x</a>");
        let _ = parse_fragment("<a ───>x</a>");
        // The legitimate `use:` path must still work (ASCII prefix).
        let _ = parse_fragment(r#"<div use:clickOutside={h}></div>"#);
        // A non-ASCII char immediately after a valid `use:` prefix.
        let _ = parse_fragment("<div use:—>x</div>");
    }

    #[test]
    fn prose_apostrophe_does_not_swallow_closing_tag() {
        // Outside `{…}`, English apostrophes must not enter string mode.
        let nodes = parse_fragment("<p>Couldn't load preview</p>").expect("parse");
        match &nodes[0] {
            super::super::Node::Element(e) => {
                assert_eq!(e.tag, "p");
                assert!(
                    e.children.iter().any(
                        |n| matches!(n, super::super::Node::Text(t) if t.data.contains("Couldn"))
                    ),
                    "expected prose text, got {:?}",
                    e.children
                );
            }
            _ => panic!("expected p element"),
        }
        let nodes = parse_fragment(r#"<strong slot="title">Couldn't load preview</strong>"#)
            .expect("parse strong");
        match &nodes[0] {
            super::super::Node::Element(e) => assert_eq!(e.tag, "strong"),
            _ => panic!("expected strong"),
        }
    }

    #[test]
    fn arm_brace_depth_keeps_nested_tags_as_text() {
        let nodes =
            parse_fragment("<when {tablet}><p>w</p>{else => <if {x()}><span>n</span></if>}</when>")
                .expect("parse");
        let when = match &nodes[0] {
            super::super::Node::Element(e) => e,
            _ => panic!("expected element"),
        };
        let kinds: Vec<&str> = when
            .children
            .iter()
            .map(|n| match n {
                super::super::Node::Element(e) => e.tag.as_str(),
                super::super::Node::Text(t) => {
                    eprintln!("TEXT {:?}", t.data);
                    "text"
                }
                _ => "other",
            })
            .collect();
        eprintln!("kinds={kinds:?}");
        assert!(kinds.iter().any(|k| *k == "text"), "expected text arm");
        assert!(
            !kinds.iter().any(|k| *k == "if"),
            "if must stay inside arm text, not sibling element: {kinds:?}"
        );
    }
}

/// Consume `style(…)` (+ `.method(` tails) starting at `start` in `src`.
fn consume_style_call(src: &str, start: usize) -> Option<usize> {
    let s = &src[start..];
    if !s.starts_with("style") {
        return None;
    }
    let bytes = s.as_bytes();
    let mut i = 0;
    let mut paren = 0i32;
    let mut brack = 0i32;
    let mut brace = 0i32;
    let mut saw = false;
    while i < bytes.len() {
        match bytes[i] {
            b'"' | b'\'' => {
                let q = bytes[i];
                i += 1;
                while i < bytes.len() && bytes[i] != q {
                    if bytes[i] == b'\\' && i + 1 < bytes.len() {
                        i += 1;
                    }
                    i += 1;
                }
                if i < bytes.len() {
                    i += 1;
                }
                continue;
            }
            b'(' => {
                paren += 1;
                saw = true;
            }
            b')' => {
                paren -= 1;
                if paren < 0 {
                    return None;
                }
            }
            b'[' => {
                brack += 1;
                saw = true;
            }
            b']' => {
                brack -= 1;
                if brack < 0 {
                    return None;
                }
            }
            b'{' => {
                brace += 1;
                saw = true;
            }
            b'}' => {
                brace -= 1;
                if brace < 0 {
                    return None;
                }
            }
            _ => {}
        }
        i += 1;
        if saw && paren == 0 && brack == 0 && brace == 0 {
            let mut j = i;
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b'.' {
                let after = j + 1;
                let mut k = after;
                while k < bytes.len()
                    && (bytes[k].is_ascii_alphanumeric() || bytes[k] == b'_' || bytes[k] == b'-')
                {
                    k += 1;
                }
                if k > after {
                    let mut m = k;
                    while m < bytes.len() && bytes[m].is_ascii_whitespace() {
                        m += 1;
                    }
                    if m < bytes.len() && bytes[m] == b'(' {
                        i = m;
                        continue;
                    }
                }
            }
            return Some(start + i);
        }
    }
    None
}
