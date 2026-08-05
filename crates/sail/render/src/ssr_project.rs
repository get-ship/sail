//! Static SSR projection: controller signal initials → first-paint template.
//!
//! Smallest honest slice for SailRender: evaluate S-tier `signal(literal)`
//! defaults into text interpolations and simple `<if>` / `<for>` so takumi
//! does not paint empty controller shells. Full setup eval stays in deck-ssr.

use std::collections::HashMap;
use std::path::PathBuf;

use sail_compiler::check::controller_parse::{self, ParsedController, ScopeKind};

/// A JSON-ish value recovered from a signal initializer.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

impl Value {
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            Value::Number(n) => Some(*n != 0.0),
            Value::String(s) => Some(!s.is_empty()),
            Value::Null => Some(false),
            Value::Array(a) => Some(!a.is_empty()),
            Value::Object(o) => Some(!o.is_empty()),
        }
    }

    pub fn length(&self) -> Option<f64> {
        match self {
            Value::Array(a) => Some(a.len() as f64),
            Value::String(s) => Some(s.len() as f64),
            _ => None,
        }
    }

    pub fn get_prop(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Object(o) => o.get(key),
            _ => None,
        }
    }

    /// Display for text interpolation (no surrounding quotes).
    pub fn display_text(&self) -> String {
        match self {
            Value::Null => String::new(),
            Value::Bool(b) => b.to_string(),
            Value::Number(n) => {
                if n.fract() == 0.0 && n.abs() < 1e15 {
                    format!("{}", *n as i64)
                } else {
                    n.to_string()
                }
            }
            Value::String(s) => s.clone(),
            Value::Array(_) | Value::Object(_) => String::new(),
        }
    }
}

/// Result of projecting controller initials into the template.
#[derive(Debug)]
pub struct Projected {
    /// Template source with controller stripped and known state inlined.
    pub source: String,
    /// Human-readable notes (tests / sidecars): which bindings were projected.
    pub projected: Vec<String>,
    /// Expressions we could not project (caller may still strip them).
    pub unresolved: Vec<String>,
    /// True when a controller block or projectable binding was present.
    pub had_controller: bool,
}

/// Project S-tier controller initials into `source`.
///
/// Always safe to call. Strips an inline `<controller>` when present, collects
/// S-tier signal initials into a scope (empty when none), expands/drops
/// control flow against that scope, then substitutes text exprs. Unevaluable
/// `<if>` / `<for>` are **removed** (not left as dual-branch static markup).
pub fn project(source: &str) -> Projected {
    let (stripped, controller_body) =
        sail_compiler::parse::preprocess::extract_controller_block(source);
    let had_controller = controller_body.is_some();
    let scope = match controller_body {
        Some(body) => match controller_parse::parse(PathBuf::from("_inline.controller.ts"), body)
        {
            Some(c) => collect_scope(&c),
            None => HashMap::new(),
        },
        None => HashMap::new(),
    };

    let mut projected = Vec::new();
    let mut unresolved = Vec::new();
    // Always walk control flow — even with an empty scope — so unknown
    // conditions drop both branches instead of painting them as static siblings.
    let mut out = expand_control_flow(&stripped, &scope, &mut projected, &mut unresolved);
    out = substitute_text_exprs(&out, &scope, &mut projected, &mut unresolved);

    Projected {
        source: out,
        projected,
        unresolved,
        had_controller,
    }
}

fn collect_scope(ctrl: &ParsedController) -> HashMap<String, Value> {
    let mut scope = HashMap::new();
    for m in &ctrl.returned {
        if m.kind != ScopeKind::Signal {
            continue;
        }
        let rhs = ctrl.slice(m.value_span).trim();
        let Some(init) = signal_init_src(rhs) else {
            continue;
        };
        if let Some(v) = parse_js_value(init.trim()) {
            scope.insert(m.name.clone(), v);
        }
    }
    scope
}

/// From `signal(0)` / `signal<T>([…])` return the init source slice.
fn signal_init_src(rhs: &str) -> Option<&str> {
    let t = rhs.trim();
    let rest = t.strip_prefix("signal")?;
    let mut i = 0;
    let bytes = rest.as_bytes();
    // Optional generics: `<…>`
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    if i < bytes.len() && bytes[i] == b'<' {
        let mut depth = 0i32;
        while i < bytes.len() {
            match bytes[i] {
                b'<' => depth += 1,
                b'>' => {
                    depth -= 1;
                    if depth == 0 {
                        i += 1;
                        break;
                    }
                }
                _ => {}
            }
            i += 1;
        }
    }
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    if i >= bytes.len() || bytes[i] != b'(' {
        return None;
    }
    let open = i;
    let close = skip_balanced(bytes, open)?;
    // init is between open+1 and close
    Some(rest[open + 1..close].trim())
}

// ── JS literal parser (subset) ──────────────────────────────────────────────

fn parse_js_value(s: &str) -> Option<Value> {
    let s = s.trim();
    if s.is_empty() {
        return Some(Value::Null);
    }
    if s == "null" || s == "undefined" {
        return Some(Value::Null);
    }
    if s == "true" {
        return Some(Value::Bool(true));
    }
    if s == "false" {
        return Some(Value::Bool(false));
    }
    if let Ok(n) = s.parse::<f64>() {
        // reject bare identifiers that parse as hex etc. — only pure numeric forms
        if s.chars()
            .all(|c| c.is_ascii_digit() || matches!(c, '.' | '-' | '+' | 'e' | 'E'))
        {
            return Some(Value::Number(n));
        }
    }
    if (s.starts_with('"') && s.ends_with('"'))
        || (s.starts_with('\'') && s.ends_with('\''))
        || (s.starts_with('`') && s.ends_with('`') && !s.contains("${"))
    {
        return Some(Value::String(unquote(s)));
    }
    if s.starts_with('[') && s.ends_with(']') {
        return parse_array(s);
    }
    if s.starts_with('{') && s.ends_with('}') {
        return parse_object(s);
    }
    None
}

fn unquote(s: &str) -> String {
    let inner = &s[1..s.len() - 1];
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some('\\') => out.push('\\'),
                Some('\'') => out.push('\''),
                Some('"') => out.push('"'),
                Some('`') => out.push('`'),
                Some(other) => out.push(other),
                None => {}
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn parse_array(s: &str) -> Option<Value> {
    let inner = s[1..s.len() - 1].trim();
    if inner.is_empty() {
        return Some(Value::Array(Vec::new()));
    }
    let parts = split_top_level(inner, b',')?;
    let mut items = Vec::with_capacity(parts.len());
    for p in parts {
        let p = p.trim();
        if p.is_empty() {
            // Trailing / double commas — ignore empty slots.
            continue;
        }
        items.push(parse_js_value(p)?);
    }
    Some(Value::Array(items))
}

fn parse_object(s: &str) -> Option<Value> {
    let inner = s[1..s.len() - 1].trim();
    if inner.is_empty() {
        return Some(Value::Object(HashMap::new()));
    }
    let parts = split_top_level(inner, b',')?;
    let mut map = HashMap::new();
    for p in parts {
        let p = p.trim();
        if p.is_empty() {
            continue;
        }
        let (key, val_src) = split_object_entry(p)?;
        map.insert(key, parse_js_value(val_src.trim())?);
    }
    Some(Value::Object(map))
}

fn split_object_entry(p: &str) -> Option<(String, &str)> {
    let bytes = p.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    if i >= bytes.len() {
        return None;
    }
    // "key" | 'key' | bareIdent
    let key = if bytes[i] == b'"' || bytes[i] == b'\'' {
        let q = bytes[i];
        i += 1;
        let start = i;
        while i < bytes.len() && bytes[i] != q {
            if bytes[i] == b'\\' && i + 1 < bytes.len() {
                i += 2;
            } else {
                i += 1;
            }
        }
        if i >= bytes.len() {
            return None;
        }
        let k = p[start..i].to_string();
        i += 1;
        k
    } else if is_ident_start(bytes[i]) {
        let start = i;
        i += 1;
        while i < bytes.len() && is_ident_cont(bytes[i]) {
            i += 1;
        }
        p[start..i].to_string()
    } else {
        return None;
    };
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    if i >= bytes.len() || bytes[i] != b':' {
        return None;
    }
    i += 1;
    Some((key, &p[i..]))
}

fn split_top_level(s: &str, sep: u8) -> Option<Vec<&str>> {
    let bytes = s.as_bytes();
    let mut out = Vec::new();
    let mut start = 0usize;
    let mut i = 0usize;
    let mut depth_paren = 0i32;
    let mut depth_brack = 0i32;
    let mut depth_brace = 0i32;
    let mut quote: Option<u8> = None;
    while i < bytes.len() {
        let b = bytes[i];
        if let Some(q) = quote {
            if b == b'\\' && i + 1 < bytes.len() {
                i += 2;
                continue;
            }
            if b == q {
                quote = None;
            }
            i += 1;
            continue;
        }
        match b {
            b'"' | b'\'' | b'`' => {
                quote = Some(b);
                i += 1;
            }
            b'(' => {
                depth_paren += 1;
                i += 1;
            }
            b')' => {
                depth_paren -= 1;
                i += 1;
            }
            b'[' => {
                depth_brack += 1;
                i += 1;
            }
            b']' => {
                depth_brack -= 1;
                i += 1;
            }
            b'{' => {
                depth_brace += 1;
                i += 1;
            }
            b'}' => {
                depth_brace -= 1;
                i += 1;
            }
            c if c == sep && depth_paren == 0 && depth_brack == 0 && depth_brace == 0 => {
                out.push(&s[start..i]);
                i += 1;
                start = i;
            }
            _ => i += 1,
        }
    }
    if depth_paren != 0 || depth_brack != 0 || depth_brace != 0 || quote.is_some() {
        return None;
    }
    out.push(&s[start..]);
    Some(out)
}

fn skip_balanced(bytes: &[u8], open: usize) -> Option<usize> {
    let open_ch = *bytes.get(open)?;
    let close_ch = match open_ch {
        b'(' => b')',
        b'[' => b']',
        b'{' => b'}',
        _ => return None,
    };
    let mut depth = 0i32;
    let mut i = open;
    let mut quote: Option<u8> = None;
    while i < bytes.len() {
        let b = bytes[i];
        if let Some(q) = quote {
            if b == b'\\' && i + 1 < bytes.len() {
                i += 2;
                continue;
            }
            if b == q {
                quote = None;
            }
            i += 1;
            continue;
        }
        match b {
            b'"' | b'\'' | b'`' => {
                quote = Some(b);
                i += 1;
            }
            c if c == open_ch => {
                depth += 1;
                i += 1;
            }
            c if c == close_ch => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
                i += 1;
            }
            _ => i += 1,
        }
    }
    None
}

fn is_ident_start(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'_' || b == b'$'
}
fn is_ident_cont(b: u8) -> bool {
    is_ident_start(b) || b.is_ascii_digit()
}

// ── Expression eval against scope ───────────────────────────────────────────

/// Evaluate a small subset of renderable template expressions against scope.
fn eval_expr(expr: &str, scope: &HashMap<String, Value>) -> Option<Value> {
    let e = expr.trim();
    if e.is_empty() {
        return None;
    }

    // Unary !
    if let Some(rest) = e.strip_prefix('!') {
        let v = eval_expr(rest.trim(), scope)?;
        return Some(Value::Bool(!v.as_bool()?));
    }

    // Comparisons: a.length > 0, a() > 0, etc.
    for op in [">=", "<=", "===", "!==", "==", "!=", ">", "<"] {
        if let Some((l, r)) = split_once_top(e, op) {
            let lv = eval_expr(l.trim(), scope)?;
            let rv = eval_expr(r.trim(), scope)?;
            return cmp_values(&lv, &rv, op);
        }
    }

    // Logical && / ||
    if let Some((l, r)) = split_once_top(e, "&&") {
        let lv = eval_expr(l.trim(), scope)?;
        if !lv.as_bool()? {
            return Some(Value::Bool(false));
        }
        return eval_expr(r.trim(), scope);
    }
    if let Some((l, r)) = split_once_top(e, "||") {
        let lv = eval_expr(l.trim(), scope)?;
        if lv.as_bool()? {
            return Some(lv);
        }
        return eval_expr(r.trim(), scope);
    }

    // member: foo().bar / foo.bar / foo().length / a.b.c
    if let Some(dot) = find_member_dot(e) {
        let base = &e[..dot];
        let prop = e[dot + 1..].trim();
        let bv = eval_expr(base, scope)?;
        if prop == "length" {
            return bv.length().map(Value::Number);
        }
        // further .prop chain on the remainder
        let mut cur = bv;
        for p in prop.split('.') {
            if p == "length" {
                return cur.length().map(Value::Number);
            }
            cur = cur.get_prop(p)?.clone();
        }
        return Some(cur);
    }

    // call: name()
    if let Some(name) = e.strip_suffix("()") {
        let name = name.trim();
        if is_simple_ident(name) {
            return scope.get(name).cloned();
        }
    }

    // bare ident
    if is_simple_ident(e) {
        return scope.get(e).cloned();
    }

    // literal
    parse_js_value(e)
}

fn is_simple_ident(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' || c == '$' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

fn find_member_dot(e: &str) -> Option<usize> {
    let bytes = e.as_bytes();
    let mut i = 0;
    let mut depth = 0i32;
    let mut quote: Option<u8> = None;
    let mut last_dot = None;
    while i < bytes.len() {
        let b = bytes[i];
        if let Some(q) = quote {
            if b == b'\\' {
                i += 2;
                continue;
            }
            if b == q {
                quote = None;
            }
            i += 1;
            continue;
        }
        match b {
            b'"' | b'\'' | b'`' => {
                quote = Some(b);
                i += 1;
            }
            b'(' | b'[' | b'{' => {
                depth += 1;
                i += 1;
            }
            b')' | b']' | b'}' => {
                depth -= 1;
                i += 1;
            }
            b'.' if depth == 0 => {
                last_dot = Some(i);
                i += 1;
            }
            _ => i += 1,
        }
    }
    last_dot
}

fn split_once_top<'a>(s: &'a str, op: &str) -> Option<(&'a str, &'a str)> {
    let bytes = s.as_bytes();
    let opb = op.as_bytes();
    let mut i = 0;
    let mut depth = 0i32;
    let mut quote: Option<u8> = None;
    while i + opb.len() <= bytes.len() {
        let b = bytes[i];
        if let Some(q) = quote {
            if b == b'\\' {
                i += 2;
                continue;
            }
            if b == q {
                quote = None;
            }
            i += 1;
            continue;
        }
        match b {
            b'"' | b'\'' | b'`' => {
                quote = Some(b);
                i += 1;
            }
            b'(' | b'[' | b'{' => {
                depth += 1;
                i += 1;
            }
            b')' | b']' | b'}' => {
                depth -= 1;
                i += 1;
            }
            _ if depth == 0 && bytes[i..].starts_with(opb) => {
                return Some((&s[..i], &s[i + opb.len()..]));
            }
            _ => i += 1,
        }
    }
    None
}

fn cmp_values(l: &Value, r: &Value, op: &str) -> Option<Value> {
    let ln = match l {
        Value::Number(n) => *n,
        Value::Bool(b) => {
            if *b {
                1.0
            } else {
                0.0
            }
        }
        Value::String(s) => {
            if let Value::String(rs) = r {
                let ord = s.cmp(rs);
                let ok = match op {
                    "==" | "===" => ord.is_eq(),
                    "!=" | "!==" => ord.is_ne(),
                    ">" => ord.is_gt(),
                    ">=" => ord.is_ge(),
                    "<" => ord.is_lt(),
                    "<=" => ord.is_le(),
                    _ => return None,
                };
                return Some(Value::Bool(ok));
            }
            return None;
        }
        _ => return None,
    };
    let rn = match r {
        Value::Number(n) => *n,
        Value::Bool(b) => {
            if *b {
                1.0
            } else {
                0.0
            }
        }
        _ => return None,
    };
    let ok = match op {
        ">" => ln > rn,
        ">=" => ln >= rn,
        "<" => ln < rn,
        "<=" => ln <= rn,
        "==" | "===" => (ln - rn).abs() < f64::EPSILON,
        "!=" | "!==" => (ln - rn).abs() >= f64::EPSILON,
        _ => return None,
    };
    Some(Value::Bool(ok))
}

// ── Control-flow expansion ──────────────────────────────────────────────────

fn expand_control_flow(
    src: &str,
    scope: &HashMap<String, Value>,
    projected: &mut Vec<String>,
    unresolved: &mut Vec<String>,
) -> String {
    let mut out = String::with_capacity(src.len());
    let mut i = 0;
    let bytes = src.as_bytes();
    while i < bytes.len() {
        if bytes[i] == b'<' {
            if let Some((end, replacement)) =
                try_expand_control_at(src, i, scope, projected, unresolved)
            {
                out.push_str(&replacement);
                i = end;
                continue;
            }
        }
        // copy one char
        let len = utf8_len(bytes[i]);
        out.push_str(&src[i..i + len]);
        i += len;
    }
    out
}

fn try_expand_control_at(
    src: &str,
    start: usize,
    scope: &HashMap<String, Value>,
    projected: &mut Vec<String>,
    unresolved: &mut Vec<String>,
) -> Option<(usize, String)> {
    let rest = &src[start..];
    if rest.starts_with("<if")
        && rest
            .as_bytes()
            .get(3)
            .map(|b| b.is_ascii_whitespace() || *b == b'{' || *b == b'>')
            .unwrap_or(false)
    {
        return expand_if(src, start, scope, projected, unresolved);
    }
    if rest.starts_with("<for")
        && rest
            .as_bytes()
            .get(4)
            .map(|b| b.is_ascii_whitespace() || *b == b'{')
            .unwrap_or(false)
    {
        return expand_for(src, start, scope, projected, unresolved);
    }
    if rest.starts_with("<when")
        && rest
            .as_bytes()
            .get(5)
            .map(|b| b.is_ascii_whitespace() || *b == b'{' || *b == b'>')
            .unwrap_or(false)
    {
        return expand_when(src, start, scope, projected, unresolved);
    }
    None
}

/// Structural `<when>`: paint the default/`else` branch only (mobile-first).
/// Viewport is unknown at SSR/takumi; client hydration swaps on matchMedia.
fn expand_when(
    src: &str,
    start: usize,
    scope: &HashMap<String, Value>,
    projected: &mut Vec<String>,
    unresolved: &mut Vec<String>,
) -> Option<(usize, String)> {
    let (open_end, attrs) = parse_open_tag(src, start)?;
    if tag_name(&attrs).as_deref() != Some("when") {
        return None;
    }
    let close = find_matching_close(src, open_end, "when")?;
    let body = &src[open_end..close];
    let end = close + "</when>".len();

    let token = attrs
        .get("__primary")
        .cloned()
        .or_else(|| attrs.get("bp").cloned())
        .or_else(|| attrs.get("breakpoint").cloned())
        .map(|s| strip_attr_braces(&s).to_string())
        .unwrap_or_default();
    projected.push(format!("when {{{token}}} → else (ssr default)"));
    let (_then_body, else_body) = split_if_branches(body);
    let expanded = expand_control_flow(else_body, scope, projected, unresolved);
    Some((end, expanded))
}

fn expand_if(
    src: &str,
    start: usize,
    scope: &HashMap<String, Value>,
    projected: &mut Vec<String>,
    unresolved: &mut Vec<String>,
) -> Option<(usize, String)> {
    let (open_end, attrs) = parse_open_tag(src, start)?;
    if tag_name(&attrs).as_deref() != Some("if") {
        return None;
    }
    let close = find_matching_close(src, open_end, "if")?;
    let body = &src[open_end..close];
    let end = close + "</if>".len();

    let cond_src = attrs
        .get("cond")
        .cloned()
        .or_else(|| attrs.get("__primary").cloned())?;
    let cond_src = strip_attr_braces(&cond_src);

    let Some(cond_val) = eval_expr(&cond_src, scope) else {
        unresolved.push(format!("if cond={{{cond_src}}}"));
        // Paint-honest: drop *both* branches. Leaving the raw <if> would
        // rasterize then + else as sibling static markup (double content).
        // Dynamics sidecar still records the unresolved cond.
        return Some((end, String::new()));
    };
    let truthy = cond_val.as_bool().unwrap_or(false);
    projected.push(format!("if cond={{{cond_src}}} → {truthy}"));

    let (then_body, else_body) = split_if_branches(body);
    let chosen = if truthy { then_body } else { else_body };
    let expanded = expand_control_flow(chosen, scope, projected, unresolved);
    Some((end, expanded))
}

fn expand_for(
    src: &str,
    start: usize,
    scope: &HashMap<String, Value>,
    projected: &mut Vec<String>,
    unresolved: &mut Vec<String>,
) -> Option<(usize, String)> {
    let (open_end, attrs) = parse_open_tag(src, start)?;
    if tag_name(&attrs).as_deref() != Some("for") {
        return None;
    }
    let close = find_matching_close(src, open_end, "for")?;
    let body = &src[open_end..close];
    let end = close + "</for>".len();

    let of_src = attrs
        .get("of")
        .cloned()
        .or_else(|| attrs.get("__primary").cloned())?;
    let of_src = strip_attr_braces(&of_src);
    let as_name = attrs
        .get("as")
        .map(|s| s.trim().trim_matches(|c| c == '"' || c == '\'').to_string())
        .filter(|s| !s.is_empty())?;

    let Some(Value::Array(items)) = eval_expr(&of_src, scope) else {
        unresolved.push(format!("for of={{{of_src}}}"));
        // Same honesty as unknown <if>: do not paint the template body once
        // with empty loop bindings (looks like a half-empty list shell).
        return Some((end, String::new()));
    };
    projected.push(format!(
        "for as={as_name} of={{{of_src}}} × {}",
        items.len()
    ));

    let mut out = String::new();
    for item in items {
        // Skip empty slots from trailing commas in array literals.
        if matches!(item, Value::Null) {
            continue;
        }
        let mut local = scope.clone();
        local.insert(as_name.clone(), item);
        // Expand nested control flow, then project text under *local* bindings
        // (item.prop must resolve here — outer substitute has no loop vars).
        let nested = expand_control_flow(body, &local, projected, unresolved);
        out.push_str(&substitute_text_exprs(
            &nested,
            &local,
            projected,
            unresolved,
        ));
    }
    Some((end, out))
}

fn split_if_branches(body: &str) -> (&str, &str) {
    // Find a top-level <else>…</else> or bare <else>… at the end.
    // Also handle <else-if> by treating first false path chain simply: only plain <else>.
    let bytes = body.as_bytes();
    let mut i = 0;
    let mut depth = 0i32; // nested <if> depth inside body
    while i < bytes.len() {
        if bytes[i] != b'<' {
            i += utf8_len(bytes[i]);
            continue;
        }
        let rest = &body[i..];
        if rest.starts_with("<if")
            && rest
                .as_bytes()
                .get(3)
                .map(|b| b.is_ascii_whitespace() || *b == b'{' || *b == b'>')
                .unwrap_or(false)
        {
            depth += 1;
            i += 3;
            continue;
        }
        if rest.starts_with("</if>") {
            depth -= 1;
            i += 5;
            continue;
        }
        if depth == 0 && rest.starts_with("<else") {
            // <else> or <else-if
            let after_else = &rest[5..];
            if after_else.starts_with('>') || after_else.starts_with(|c: char| c.is_whitespace()) {
                if after_else.starts_with("-if") {
                    // else-if: not fully evaluated in this slice — treat as else boundary
                    // only when cond known; for simplicity take remainder as else and stop.
                    let then = &body[..i];
                    let else_part = &body[i..];
                    // strip outer <else-if …>…</else-if> / <else>… if present
                    return (then, strip_else_wrapper(else_part));
                }
                // plain <else>
                let then = &body[..i];
                let else_part = &body[i..];
                return (then, strip_else_wrapper(else_part));
            }
        }
        i += 1;
    }
    (body, "")
}

fn strip_else_wrapper(s: &str) -> &str {
    let s = s.trim_start();
    if let Some(rest) = s.strip_prefix("<else>") {
        if let Some(idx) = rest.rfind("</else>") {
            return &rest[..idx];
        }
        return rest;
    }
    if s.starts_with("<else-if") {
        // leave content after first open tag through matching close if possible
        if let Some((open_end, _)) = parse_open_tag(s, 0) {
            if let Some(close) = find_matching_close(s, open_end, "else-if") {
                return &s[open_end..close];
            }
            return &s[open_end..];
        }
    }
    if let Some(rest) = s.strip_prefix("<else") {
        if let Some(gt) = rest.find('>') {
            let after = &rest[gt + 1..];
            if let Some(idx) = after.rfind("</else>") {
                return &after[..idx];
            }
            return after;
        }
    }
    s
}

fn tag_name(attrs: &HashMap<String, String>) -> Option<String> {
    attrs.get("__tag").cloned()
}

fn strip_attr_braces(s: &str) -> String {
    let t = s.trim();
    // preprocess may produce cond="{show()}" or cond={show()}
    let t = t.trim_matches('"').trim_matches('\'');
    let t = t.trim();
    if t.starts_with('{') && t.ends_with('}') && t.len() >= 2 {
        t[1..t.len() - 1].trim().to_string()
    } else {
        t.to_string()
    }
}

/// Parse an open tag starting at `start` (`<…>`). Returns end index after `>`
/// and a map of attributes; `__tag` is the tag name; brace-primary stored as `__primary`.
fn parse_open_tag(src: &str, start: usize) -> Option<(usize, HashMap<String, String>)> {
    let bytes = src.as_bytes();
    if bytes.get(start) != Some(&b'<') {
        return None;
    }
    let mut i = start + 1;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    let name_start = i;
    while i < bytes.len() && (is_ident_cont(bytes[i]) || bytes[i] == b'-') {
        i += 1;
    }
    if i == name_start {
        return None;
    }
    let tag = src[name_start..i].to_string();
    let mut attrs = HashMap::new();
    attrs.insert("__tag".into(), tag);

    loop {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            return None;
        }
        if bytes[i] == b'>' {
            return Some((i + 1, attrs));
        }
        if bytes[i] == b'/' && bytes.get(i + 1) == Some(&b'>') {
            return Some((i + 2, attrs));
        }
        // Brace-primary: `{expr}`
        if bytes[i] == b'{' {
            let close = skip_balanced(bytes, i)?;
            let expr = src[i + 1..close].trim().to_string();
            attrs.insert("__primary".into(), format!("{{{expr}}}"));
            i = close + 1;
            continue;
        }
        // name=value
        if !is_ident_start(bytes[i]) {
            // skip unknown junk carefully
            if bytes[i] == b'>' {
                return Some((i + 1, attrs));
            }
            i += 1;
            continue;
        }
        let ns = i;
        while i < bytes.len() && (is_ident_cont(bytes[i]) || bytes[i] == b'-' || bytes[i] == b':')
        {
            i += 1;
        }
        let name = src[ns..i].to_string();
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i < bytes.len() && bytes[i] == b'=' {
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if i >= bytes.len() {
                return None;
            }
            let val = if bytes[i] == b'"' || bytes[i] == b'\'' {
                let q = bytes[i];
                i += 1;
                let vs = i;
                while i < bytes.len() && bytes[i] != q {
                    if bytes[i] == b'\\' {
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                let v = src[vs..i].to_string();
                if i < bytes.len() {
                    i += 1;
                }
                v
            } else if bytes[i] == b'{' {
                let close = skip_balanced(bytes, i)?;
                let v = src[i..=close].to_string();
                i = close + 1;
                v
            } else {
                let vs = i;
                while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'>' {
                    i += 1;
                }
                src[vs..i].to_string()
            };
            attrs.insert(name, val);
        } else {
            // boolean attr
            attrs.insert(name, "true".into());
        }
    }
}

fn find_matching_close(src: &str, body_start: usize, tag: &str) -> Option<usize> {
    let open_pat = format!("<{tag}");
    let close_pat = format!("</{tag}>");
    let bytes = src.as_bytes();
    let mut i = body_start;
    let mut depth = 1i32;
    while i < bytes.len() {
        if src[i..].starts_with(&close_pat) {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
            i += close_pat.len();
            continue;
        }
        if src[i..].starts_with(&open_pat) {
            let after = i + open_pat.len();
            let boundary = src
                .as_bytes()
                .get(after)
                .map(|b| b.is_ascii_whitespace() || *b == b'{' || *b == b'>' || *b == b'/')
                .unwrap_or(false);
            if boundary {
                depth += 1;
            }
        }
        i += utf8_len(bytes[i]);
    }
    None
}

// ── Text interpolation ─────────────────────────────────────────────────────

fn substitute_text_exprs(
    src: &str,
    scope: &HashMap<String, Value>,
    projected: &mut Vec<String>,
    unresolved: &mut Vec<String>,
) -> String {
    let mut out = String::with_capacity(src.len());
    let bytes = src.as_bytes();
    let mut i = 0;
    let mut in_tag = false;
    let mut in_quote: Option<u8> = None;
    while i < bytes.len() {
        let b = bytes[i];
        if in_tag {
            let len = utf8_len(b);
            out.push_str(&src[i..i + len]);
            if let Some(q) = in_quote {
                if b == q {
                    in_quote = None;
                }
            } else if b == b'"' || b == b'\'' {
                in_quote = Some(b);
            } else if b == b'>' {
                in_tag = false;
            }
            i += len;
            continue;
        }
        if b == b'<' {
            in_tag = true;
            out.push('<');
            i += 1;
            continue;
        }
        if b == b'{' {
            let mut j = i + 1;
            let mut depth = 1i32;
            while j < bytes.len() && depth > 0 {
                match bytes[j] {
                    b'{' => depth += 1,
                    b'}' => depth -= 1,
                    b'<' => break,
                    _ => {}
                }
                j += 1;
            }
            if depth == 0 && j > i + 1 {
                let expr = src[i + 1..j - 1].trim();
                if looks_like_expr(expr) {
                    if let Some(v) = eval_expr(expr, scope) {
                        let text = v.display_text();
                        // Only treat as resolved when we produced text or a non-complex value.
                        match &v {
                            Value::Array(_) | Value::Object(_) => {
                                unresolved.push(expr.to_string());
                                // leave empty (same as pre-projection strip)
                            }
                            _ => {
                                projected.push(format!("{{{expr}}} → {text:?}"));
                                out.push_str(&html_escape_text(&text));
                            }
                        }
                    } else {
                        unresolved.push(expr.to_string());
                        // leave empty for residual strip honesty
                    }
                    i = j;
                    continue;
                }
            }
        }
        let len = utf8_len(b);
        out.push_str(&src[i..i + len]);
        i += len;
    }
    out
}

fn looks_like_expr(s: &str) -> bool {
    let t = s.trim();
    if t.is_empty() {
        return false;
    }
    let first = t.chars().next().unwrap();
    first.is_ascii_alphabetic() || first == '_' || first == '(' || first == '[' || first == '!'
}

fn html_escape_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(c),
        }
    }
    out
}

fn utf8_len(b: u8) -> usize {
    match b {
        0x00..=0x7F => 1,
        0xC0..=0xDF => 2,
        0xE0..=0xEF => 3,
        _ => 4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projects_string_signal_into_heading() {
        let src = r#"
<controller>
import { controller, signal } from '@ship/sail-runtime'
export default controller(() => {
  const title = signal("Hello Board")
  return { title }
})
</controller>
<h1>{title()}</h1>
"#;
        let p = project(src);
        assert!(p.had_controller);
        assert!(
            p.source.contains("Hello Board"),
            "projected source: {}",
            p.source
        );
        assert!(!p.source.contains("{title()}"));
        assert!(!p.source.contains("<controller"));
    }

    #[test]
    fn projects_for_over_object_array() {
        let src = r#"
<controller>
import { controller, signal } from '@ship/sail-runtime'
export default controller(() => {
  const items = signal([
    { id: 'a', label: 'Alpha' },
    { id: 'b', label: 'Bravo' },
  ])
  return { items }
})
</controller>
<ul>
  <for {items()} as="item" key={item.id}>
    <li>{item.label}</li>
  </for>
</ul>
"#;
        let p = project(src);
        assert!(p.source.contains("Alpha"), "{}", p.source);
        assert!(p.source.contains("Bravo"), "{}", p.source);
        assert!(!p.source.contains("<for"), "{}", p.source);
    }

    #[test]
    fn projects_if_truthy_branch() {
        let src = r#"
<controller>
import { controller, signal } from '@ship/sail-runtime'
export default controller(() => {
  const show = signal(true)
  const hide = signal(false)
  return { show, hide }
})
</controller>
<div>
  <if {show()}><p>visible</p></if>
  <if {hide()}><p>nope</p><else><p>fallback</p></else></if>
</div>
"#;
        let p = project(src);
        assert!(p.source.contains("visible"), "{}", p.source);
        assert!(!p.source.contains("nope"), "{}", p.source);
        assert!(p.source.contains("fallback"), "{}", p.source);
    }

    #[test]
    fn leaves_unknown_expr_empty_and_unresolved() {
        let src = r#"
<controller>
import { controller, signal } from '@ship/sail-runtime'
export default controller(() => {
  const ticks = signal(0)
  return { ticks }
})
</controller>
<span>{ticks()}</span><span>{window.innerWidth}</span>
"#;
        let p = project(src);
        assert!(p.source.contains(">0<") || p.source.contains(">0</"), "{}", p.source);
        assert!(
            p.unresolved.iter().any(|u| u.contains("window")),
            "{:?}",
            p.unresolved
        );
    }

    #[test]
    fn unknown_if_drops_both_branches() {
        let src = r#"
<div>
  <if {window.x}><p>yes-branch</p><else><p>no-branch</p></else></if>
  <p>after</p>
</div>
"#;
        let p = project(src);
        assert!(
            !p.source.contains("yes-branch") && !p.source.contains("no-branch"),
            "unevaluable if must not paint either branch; got:\n{}",
            p.source
        );
        assert!(p.source.contains("after"), "{}", p.source);
        assert!(
            p.unresolved.iter().any(|u| u.contains("window.x")),
            "{:?}",
            p.unresolved
        );
    }

    #[test]
    fn unknown_for_drops_template_body() {
        let src = r#"
<ul>
  <for {window.list} as="item" key={item}>
    <li>row-shell</li>
  </for>
</ul>
"#;
        let p = project(src);
        assert!(
            !p.source.contains("row-shell"),
            "unevaluable for must not paint body; got:\n{}",
            p.source
        );
        assert!(
            p.unresolved.iter().any(|u| u.contains("window.list")),
            "{:?}",
            p.unresolved
        );
    }

    #[test]
    fn no_controller_is_identity() {
        let src = "<div>static</div>";
        let p = project(src);
        assert!(!p.had_controller);
        assert!(p.source.contains("static"));
    }
}
