//! `animation-presets.json` lookup. Hand-rolled parser (the compiler
//! crate is WASM-safe; `serde_json` is not a workspace dep). The JSON
//! grammar we accept is the subset needed by
//! `crates/sui/compiler/animation-presets.json`: objects with string
//! keys, string / number / bool / null values, arrays. Comments
//! beginning with `$` (the file uses `$comment` / `$doc`) are
//! tolerated on any line; everything else is strict.
//!
//! One-shot parse via [`load`]; the result is `Clone` and small
//! enough to carry around per-emit.

use std::collections::BTreeMap;

const PRESETS_JSON: &str = include_str!("../../animation-presets.json");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresetKind {
    Spring,
    Tween,
}

#[derive(Debug, Clone)]
pub struct PresetSpec {
    pub kind: PresetKind,
    pub stiffness: Option<f64>,
    pub damping: Option<f64>,
    pub mass: Option<f64>,
    pub rest_speed: Option<f64>,
    pub rest_delta: Option<f64>,
    pub from: Option<f64>,
    pub to: Option<f64>,
    pub duration_token: Option<String>,
    pub easing: Option<String>,
    pub iterations: Option<String>,
    pub direction: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Resolver {
    pub duration_tokens: BTreeMap<String, String>,
    pub easing_tokens: BTreeMap<String, String>,
    pub property_aliases: BTreeMap<String, f64>,
}

#[derive(Debug, Clone)]
pub struct PresetDb {
    pub presets: BTreeMap<String, PresetSpec>,
    pub resolver: Resolver,
}

pub fn load() -> Result<PresetDb, String> {
    let bytes = PRESETS_JSON.as_bytes();
    let mut p = Parser { bytes, pos: 0 };
    p.skip_ws_and_comments();
    let v = p.parse_value()?;
    p.skip_ws_and_comments();
    if p.pos != bytes.len() {
        return Err(format!("trailing content at byte {}", p.pos));
    }
    parse_db(v).ok_or_else(|| "animation-presets.json: missing top-level object".to_string())
}

impl PresetDb {
    pub fn preset(&self, name: &str) -> Option<&PresetSpec> {
        self.presets.get(name)
    }
    pub fn known_preset_names(&self) -> Vec<String> {
        self.presets.keys().cloned().collect()
    }
    /// Resolve a `duration-fast` style token to its CSS identifier.
    /// Falls back to the input verbatim when unknown.
    pub fn resolve_duration_token(&self, token: &str) -> String {
        let bare = token.strip_prefix("duration-").unwrap_or(token);
        self.resolver
            .duration_tokens
            .get(bare)
            .cloned()
            .unwrap_or_else(|| token.to_string())
    }
    /// Resolve a scale/translate alias (`sm`, `md`, `lg`, `xl`, `xs`)
    /// to its numeric value: `0.96`, `1.02`, etc.
    pub fn alias_value(&self, alias: &str) -> Option<f64> {
        self.resolver.property_aliases.get(alias).copied()
    }
}

fn parse_db(v: Value) -> Option<PresetDb> {
    let Value::Object(top) = v else { return None };
    let mut presets = BTreeMap::new();
    if let Some(Value::Object(m)) = top.get("presets") {
        for (name, val) in m {
            let Value::Object(fields) = val else { continue };
            let kind_str = match fields.get("kind") {
                Some(Value::String(s)) => s.as_str(),
                _ => continue,
            };
            let kind = match kind_str {
                "spring" => PresetKind::Spring,
                "tween" => PresetKind::Tween,
                _ => continue,
            };
            let p = PresetSpec {
                kind,
                stiffness: num(fields.get("stiffness")),
                damping: num(fields.get("damping")),
                mass: num(fields.get("mass")),
                rest_speed: num(fields.get("rest_speed")),
                rest_delta: num(fields.get("rest_delta")),
                from: num(fields.get("from")),
                to: num(fields.get("to")),
                duration_token: str(fields.get("duration_token")),
                easing: str(fields.get("easing")),
                iterations: str(fields.get("iterations")),
                direction: str(fields.get("direction")),
            };
            presets.insert(name.clone(), p);
        }
    }
    let mut resolver = Resolver::default();
    if let Some(Value::Object(m)) = top.get("duration_tokens") {
        for (k, v) in m {
            if let Value::String(s) = v {
                resolver.duration_tokens.insert(k.clone(), s.clone());
            }
        }
    }
    if let Some(Value::Object(m)) = top.get("easing_tokens") {
        for (k, v) in m {
            if let Value::String(s) = v {
                resolver.easing_tokens.insert(k.clone(), s.clone());
            }
        }
    }
    if let Some(Value::Object(m)) = top.get("property_aliases") {
        for (k, v) in m {
            if let Value::Number(n) = v {
                resolver.property_aliases.insert(k.clone(), *n);
            }
        }
    }
    Some(PresetDb { presets, resolver })
}

fn num(v: Option<&Value>) -> Option<f64> {
    match v? {
        Value::Number(n) => Some(*n),
        Value::String(s) => s.parse().ok(),
        _ => None,
    }
}

fn str(v: Option<&Value>) -> Option<String> {
    match v? {
        Value::String(s) => Some(s.clone()),
        _ => None,
    }
}

// ---------- minimal JSON ----------

#[derive(Debug, Clone)]
enum Value {
    Object(BTreeMap<String, Value>),
    Array(Vec<Value>),
    String(String),
    Number(f64),
    Bool(bool),
    Null,
}

struct Parser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn skip_ws_and_comments(&mut self) {
        loop {
            while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_whitespace() {
                self.pos += 1;
            }
            if self.pos < self.bytes.len() && self.bytes[self.pos] == b'$' {
                while self.pos < self.bytes.len() && self.bytes[self.pos] != b'\n' {
                    self.pos += 1;
                }
                continue;
            }
            break;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn eat(&mut self, b: u8) -> Result<(), String> {
        if self.peek() == Some(b) {
            self.pos += 1;
            Ok(())
        } else {
            Err(format!(
                "expected `{}` at byte {}, got {:?}",
                b as char,
                self.pos,
                self.peek().map(|c| c as char)
            ))
        }
    }

    fn parse_value(&mut self) -> Result<Value, String> {
        self.skip_ws_and_comments();
        let c = self.peek().ok_or_else(|| "unexpected EOF".to_string())?;
        match c {
            b'{' => self.parse_object(),
            b'[' => self.parse_array(),
            b'"' => self.parse_string().map(Value::String),
            b't' | b'f' => self.parse_bool(),
            b'n' => self.parse_null(),
            b'-' | b'0'..=b'9' => self.parse_number(),
            other => Err(format!("unexpected `{}` at byte {}", other as char, self.pos)),
        }
    }

    fn parse_object(&mut self) -> Result<Value, String> {
        self.eat(b'{')?;
        let mut m = BTreeMap::new();
        self.skip_ws_and_comments();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(Value::Object(m));
        }
        loop {
            self.skip_ws_and_comments();
            let key = self.parse_string()?;
            self.skip_ws_and_comments();
            self.eat(b':')?;
            let val = self.parse_value()?;
            m.insert(key, val);
            self.skip_ws_and_comments();
            match self.peek() {
                Some(b',') => {
                    self.pos += 1;
                }
                Some(b'}') => {
                    self.pos += 1;
                    return Ok(Value::Object(m));
                }
                other => {
                    return Err(format!(
                        "expected `,` or `}}` at byte {}, got {:?}",
                        self.pos,
                        other.map(|c| c as char)
                    ))
                }
            }
        }
    }

    fn parse_array(&mut self) -> Result<Value, String> {
        self.eat(b'[')?;
        let mut v = Vec::new();
        self.skip_ws_and_comments();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Ok(Value::Array(v));
        }
        loop {
            v.push(self.parse_value()?);
            self.skip_ws_and_comments();
            match self.peek() {
                Some(b',') => {
                    self.pos += 1;
                }
                Some(b']') => {
                    self.pos += 1;
                    return Ok(Value::Array(v));
                }
                other => {
                    return Err(format!(
                        "expected `,` or `]` at byte {}, got {:?}",
                        self.pos,
                        other.map(|c| c as char)
                    ))
                }
            }
        }
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.eat(b'"')?;
        let mut out = String::new();
        while let Some(&b) = self.bytes.get(self.pos) {
            match b {
                b'"' => {
                    self.pos += 1;
                    return Ok(out);
                }
                b'\\' => {
                    self.pos += 1;
                    match self.bytes.get(self.pos).copied() {
                        Some(b'"') => {
                            out.push('"');
                            self.pos += 1;
                        }
                        Some(b'\\') => {
                            out.push('\\');
                            self.pos += 1;
                        }
                        Some(b'/') => {
                            out.push('/');
                            self.pos += 1;
                        }
                        Some(b'n') => {
                            out.push('\n');
                            self.pos += 1;
                        }
                        Some(b'r') => {
                            out.push('\r');
                            self.pos += 1;
                        }
                        Some(b't') => {
                            out.push('\t');
                            self.pos += 1;
                        }
                        Some(b'u') => {
                            self.pos += 1;
                            let hex = std::str::from_utf8(
                                self.bytes.get(self.pos..self.pos + 4).ok_or("bad \\u escape")?,
                            )
                            .map_err(|e| format!("invalid utf-8: {e}"))?;
                            let cp = u32::from_str_radix(hex, 16)
                                .map_err(|e| format!("bad hex `{hex}`: {e}"))?;
                            let c = std::char::from_u32(cp).ok_or("bad unicode codepoint")?;
                            out.push(c);
                            self.pos += 4;
                        }
                        other => return Err(format!("bad escape `\\{}`", other.map(|b| b as char).unwrap_or('?'))),
                    }
                }
                _ => out.push(b as char),
            }
            self.pos += 1;
        }
        Err("unterminated string".to_string())
    }

    fn parse_number(&mut self) -> Result<Value, String> {
        let start = self.pos;
        if self.peek() == Some(b'-') {
            self.pos += 1;
        }
        while let Some(&b) = self.bytes.get(self.pos) {
            if b.is_ascii_digit()
                || b == b'.'
                || b == b'e'
                || b == b'E'
                || b == b'+'
                || b == b'-'
            {
                self.pos += 1;
            } else {
                break;
            }
        }
        let s = std::str::from_utf8(&self.bytes[start..self.pos])
            .map_err(|e| format!("invalid utf-8: {e}"))?;
        let n: f64 = s.parse().map_err(|_| format!("invalid number `{s}`"))?;
        Ok(Value::Number(n))
    }

    fn parse_bool(&mut self) -> Result<Value, String> {
        if self.bytes[self.pos..].starts_with(b"true") {
            self.pos += 4;
            Ok(Value::Bool(true))
        } else if self.bytes[self.pos..].starts_with(b"false") {
            self.pos += 5;
            Ok(Value::Bool(false))
        } else {
            Err("expected boolean".to_string())
        }
    }

    fn parse_null(&mut self) -> Result<Value, String> {
        if self.bytes[self.pos..].starts_with(b"null") {
            self.pos += 4;
            Ok(Value::Null)
        } else {
            Err("expected null".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_embedded_presets() {
        let db = load().expect("parse embedded JSON");
        let snappy = db.preset("spring:snappy").expect("spring:snappy");
        assert_eq!(snappy.kind, PresetKind::Spring);
        assert_eq!(snappy.stiffness, Some(380.0));
        assert_eq!(snappy.damping, Some(30.0));
        let fade = db.preset("fade").expect("fade");
        assert_eq!(fade.kind, PresetKind::Tween);
        assert_eq!(fade.duration_token.as_deref(), Some("duration-fast"));
        assert_eq!(
            fade.easing.as_deref(),
            Some("cubic-bezier(0.2, 0, 0, 1)")
        );
        assert_eq!(db.alias_value("sm"), Some(1.02));
        assert_eq!(db.alias_value("xs"), Some(0.96));
    }
}
