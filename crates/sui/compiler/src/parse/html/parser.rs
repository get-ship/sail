//! `Parser` — recursive-descent walk over the preprocessed source.

use crate::diagnostics::{CompileError, Span};

use super::{decode_entities, Attr, Comment, Element, Node, Text, RAW_TEXT_TAGS, VOID_TAGS};

pub(super) struct Parser<'a> {
    src: &'a str,
    bytes: &'a [u8],
    i: usize,
}

impl<'a> Parser<'a> {
    pub(super) fn new(src: &'a str) -> Self {
        Self { src, bytes: src.as_bytes(), i: 0 }
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
                    && close.eq_ignore_ascii_case(parent) {
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
                .with_span(Span::point(self.i)));
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
            // Text run — read up to the next `<`. A stray `<` that wasn't
            // recognized as a tag start (e.g. the JS less-than operator inside
            // a `{#if x < y}` template expression) must be consumed as text;
            // otherwise the loop body below never advances `self.i` and we
            // spin forever pushing empty text nodes.
            let start = self.i;
            if self.bytes[self.i] == b'<' {
                self.i += 1;
            }
            while self.i < self.bytes.len() && self.bytes[self.i] != b'<' {
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
            out.push(Node::Text(Text { data, span: Span::new(start, self.i) }));
        }
        if let Some(parent) = parent_tag {
            return Err(CompileError::new(format!("unterminated <{parent}>"))
                .with_span(Span::point(self.bytes.len())));
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
                return Ok(Comment { data: body, span: Span::new(start, self.i) });
            }
            self.i += 1;
        }
        Err(CompileError::new("unterminated <!-- comment").with_span(Span::point(start)))
    }

    fn parse_element(&mut self) -> Result<Element, CompileError> {
        let start = self.i;
        self.i += 1;
        let tag_start = self.i;
        while self.i < self.bytes.len() && is_tag_name_byte(self.bytes[self.i]) {
            self.i += 1;
        }
        if self.i == tag_start {
            return Err(CompileError::new("expected tag name").with_span(Span::point(start)));
        }
        let tag = self.src[tag_start..self.i].to_ascii_lowercase();
        let attributes = self.parse_attrs()?;
        let self_closed = self.consume_tag_end()?;
        if self_closed || VOID_TAGS.contains(&tag.as_str()) {
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
                .with_span(Span::point(self.i)));
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
            .with_span(Span::point(close_at)));
        }
        self.skip_ws();
        if self.i >= self.bytes.len() || self.bytes[self.i] != b'>' {
            return Err(CompileError::new(format!("malformed </{tag}>"))
                .with_span(Span::point(close_at)));
        }
        self.i += 1;
        Ok(Element { tag, attributes, children, span: Span::new(start, self.i) })
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
            let name_start = self.i;
            while self.i < self.bytes.len() && is_attr_name_byte(self.bytes[self.i]) {
                self.i += 1;
            }
            if self.i == name_start {
                return Err(CompileError::new(format!(
                    "unexpected character `{}` while parsing attributes",
                    c as char
                ))
                .with_span(Span::point(self.i)));
            }
            // Lowercase mirrors DOMParser's behavior.
            let name = self.src[name_start..self.i].to_ascii_lowercase();
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
            return Err(CompileError::new("expected `>`").with_span(Span::point(self.i)));
        }
        self.i += 1;
        Ok(self_closed)
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
        Text { data: self.src[start..self.i].to_string(), span: Span::new(start, self.i) }
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
                .with_span(Span::point(qstart)));
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
        (self.src[start..self.i].to_string(), Span::new(start, self.i))
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
    fn peek_byte(&self) -> Option<u8> { self.bytes.get(self.i).copied() }
    fn peek2(&self) -> Option<(u8, u8)> {
        if self.i + 1 < self.bytes.len() {
            Some((self.bytes[self.i], self.bytes[self.i + 1]))
        } else { None }
    }
    fn peek_close_tag(&self) -> Result<String, CompileError> {
        let mut j = self.i + 2;
        let start = j;
        while j < self.bytes.len() && is_tag_name_byte(self.bytes[j]) {
            j += 1;
        }
        if j == start {
            return Err(CompileError::new("malformed close tag").with_span(Span::point(self.i)));
        }
        Ok(self.src[start..j].to_ascii_lowercase())
    }
    fn starts_with(&self, s: &str) -> bool { self.src[self.i..].starts_with(s) }
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
    !b.is_ascii_whitespace() && b != b'=' && b != b'>' && b != b'/' && b != b'"' && b != b'\''
}

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
        // crates/sui/compiler/fuzz/ — minimized input was `<style><-──`.
        let _ = parse_fragment("<style><-──");
        let _ = parse_fragment("<script>x<─</script>");
        let _ = parse_fragment("<style>a<bc</style>");
    }
}
