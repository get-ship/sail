//! Parser for the Style value language (chain + match / when / compound / bind).

use std::borrow::Cow;
use std::collections::BTreeMap;

use super::ir::{PaintArg, PaintOp, Style, StyleDiagnostic};
use super::setters::{closest_unique_setter, find_setter, validate_op, when_target};

/// Parse a Style expression. Returns Style + diagnostics (fail-closed flags in diags).
pub fn parse(input: &str) -> (Style, Vec<StyleDiagnostic>) {
    let mut diags = Vec::new();
    // Authors reasonably write `//` / `/* */` comments inside long sx lists.
    // Strip them before tokenizing: the grammar has no division and no string
    // literals, so `/` can only start a comment. Without this, comment text
    // tokenizes into Words — silently injecting spurious Refs and truncating
    // list entries after the comment.
    let stripped = strip_comments(input);
    let trim_start = stripped.len() - stripped.trim_start().len();
    let trimmed = stripped.trim();
    if trimmed.is_empty() {
        return (Style::Empty, diags);
    }
    // Residual utility-string product → hard error (not architecture).
    if looks_like_utility_string_product(trimmed) {
        diags.push(StyleDiagnostic::ResidualProduct {
            message: "utility-string sx is not the Style product — use style.bg(token).… chains, match, when({…}), compound"
                .into(),
            source: trimmed.chars().take(80).collect(),
        });
        return (Style::Empty, diags);
    }
    // Offsets are into `trimmed`; `input_base` maps them back if needed. Motion
    // config capture uses the trimmed slice so quotes/units survive.
    let tokens = tokenize(trimmed);
    let mut p = Parser::new(trimmed, &tokens);
    let _ = trim_start; // kept for future span mapping into original input
    let style = parse_style_expr(&mut p, &mut diags);
    if !p.at_end() {
        diags.push(StyleDiagnostic::ParseError {
            message: format!("unexpected token `{}`", p.peek_str()),
            source: p.remainder(),
        });
    }
    // Validate all paint ops.
    validate_style(&style, &mut diags);
    // Lifecycle motion must not nest inside when() — clocks don't nest.
    if let Err(d) = forbid_motion_inside_when(&style) {
        diags.push(d);
    }
    (style, diags)
}

/// Motion chain methods that lower to Motion attach IR (not CSS paint).
/// Mirrors `MOTION_TRIGGERS` (hover/press/focus for spring bursts; enter/exit
/// lifecycle; loop/reveal/scroll ambient). `scroll` duals with paint
/// `style.scroll(auto|smooth)` — brace-object form is motion.
const MOTION_CHAIN_METHODS: &[&str] = &[
    "enter",
    "exit",
    "loop",
    "reveal",
    "scroll",
    "hover",
    "press",
    "focus",
    "focus-visible",
];

fn is_motion_chain_method(name: &str) -> bool {
    MOTION_CHAIN_METHODS.contains(&name)
}

/// Collect `MotionAttach` nodes from a Style tree (pre-order, source order).
/// Used by preprocess to inject compiler-private `_sail_m_*` attributes next
/// to the lowered class list.
pub fn collect_motion_attaches(style: &Style) -> Vec<(String, String)> {
    let mut out = Vec::new();
    collect_motion_attaches_inner(style, &mut out);
    out
}

fn collect_motion_attaches_inner(style: &Style, out: &mut Vec<(String, String)>) {
    match style {
        Style::Empty | Style::Set(_) | Style::Ref(_) => {}
        Style::MotionAttach { trigger, config } => {
            out.push((trigger.clone(), config.clone()));
        }
        Style::Chain(items) | Style::List(items) => {
            for i in items {
                collect_motion_attaches_inner(i, out);
            }
        }
        Style::Match { arms, .. } => {
            for (_, body) in arms {
                collect_motion_attaches_inner(body, out);
            }
        }
        Style::Compound { body, .. } => collect_motion_attaches_inner(body, out),
        Style::When { entries } => {
            for (_, body) in entries {
                collect_motion_attaches_inner(body, out);
            }
        }
    }
}

fn forbid_motion_inside_when(style: &Style) -> Result<(), StyleDiagnostic> {
    match style {
        Style::When { entries } => {
            for (key, body) in entries {
                if style_contains_motion(body) {
                    return Err(StyleDiagnostic::ParseError {
                        message: format!(
                            "`.enter` / `.exit` / `.loop` / `.reveal` / `.scroll` cannot appear inside when({{ {key}: … }}) — lifecycle and ambient motion are not pseudo-state. Put motion methods on the rest chain: style.…rest….enter({{…}}).when({{…}})"
                        ),
                        source: format!("when.{{ {key} }}"),
                    });
                }
            }
            Ok(())
        }
        Style::Chain(items) | Style::List(items) => {
            for i in items {
                forbid_motion_inside_when(i)?;
            }
            Ok(())
        }
        Style::Match { arms, .. } => {
            for (_, body) in arms {
                forbid_motion_inside_when(body)?;
            }
            Ok(())
        }
        Style::Compound { body, .. } => forbid_motion_inside_when(body),
        _ => Ok(()),
    }
}

fn style_contains_motion(style: &Style) -> bool {
    match style {
        Style::MotionAttach { .. } => true,
        Style::Chain(items) | Style::List(items) => items.iter().any(style_contains_motion),
        Style::Match { arms, .. } => arms.iter().any(|(_, b)| style_contains_motion(b)),
        Style::Compound { body, .. } => style_contains_motion(body),
        Style::When { entries } => entries.iter().any(|(_, b)| style_contains_motion(b)),
        _ => false,
    }
}

/// True if input is the killed utility-string product (atoms / prop: value soup without style./match/when).
pub fn looks_like_utility_string_product(input: &str) -> bool {
    // Preprocess may pass `{expr}` (outer brace wrapper from quote_brace_attrs).
    let t = strip_one_outer_brace_layer(input.trim());
    if t.starts_with("style.")
        || t.starts_with("match(")
        || t.starts_with("when(")
        || t.starts_with("compound(")
        || t.starts_with('[')
    {
        return false;
    }
    // Ident-only ref: `filled` — allowed as Style const ref
    if t.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') {
        return false;
    }
    // Style const + motion method: `cta.press({ scale: 'press' })`,
    // `wordmark.scroll({ scale: 'spring:deliberate' })`. Colons live inside the
    // config object; that is product, not utility-string soup.
    if is_const_ref_motion_chain(t) {
        return false;
    }
    // Space-separated utility soup or legacy atoms
    t.contains(':')
        || t.split_whitespace()
            .any(|w| w.contains('-') && !w.starts_with("style"))
}

fn strip_one_outer_brace_layer(t: &str) -> &str {
    let t = t.trim();
    if !(t.starts_with('{') && t.ends_with('}') && t.len() >= 2) {
        return t;
    }
    let inn = t[1..t.len() - 1].trim();
    // Strip when the outer pair wraps a whole Style expr (not a bare `{k: v}` map).
    if inn.starts_with("style.")
        || inn.starts_with("match(")
        || inn.starts_with("when(")
        || inn.starts_with("compound(")
        || inn.starts_with('[')
        || inn
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b.is_ascii_whitespace())
        || is_const_ref_motion_chain(inn)
    {
        return inn;
    }
    t
}

/// `name.enter|exit|loop|reveal|scroll|hover|press|focus…({…})` — motion attach
/// chained off a Style const ref (not `style.`).
fn is_const_ref_motion_chain(input: &str) -> bool {
    let Some((head, rest)) = input.split_once('.') else {
        return false;
    };
    if head.is_empty() || !head.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') {
        return false;
    }
    let method = rest.split('(').next().unwrap_or("");
    is_motion_chain_method(method)
}

/// Public for the sx-attribute classifier (`rules::sx`): a const-ref motion
/// chain (`row.enter({…}).exit({…})`) is a compile-time Style product, not a
/// runtime dynamic binding.
pub fn is_const_ref_motion_chain_pub(input: &str) -> bool {
    is_const_ref_motion_chain(input)
}

/// Remove `//` line and `/* */` block comments, replacing each with a single
/// space so words on either side can never merge. Unterminated block comments
/// consume to end of input (the truncated expression then fails loudly with a
/// normal parse diagnostic instead of silently dropping entries).
fn strip_comments(input: &str) -> Cow<'_, str> {
    let b = input.as_bytes();
    let mut out: Option<String> = None;
    let mut copied = 0; // input[..copied] already flushed into out
    let mut i = 0;
    while i + 1 < b.len() {
        if b[i] == b'/' && (b[i + 1] == b'/' || b[i + 1] == b'*') {
            let s = out.get_or_insert_with(|| String::with_capacity(input.len()));
            s.push_str(&input[copied..i]);
            s.push(' ');
            if b[i + 1] == b'/' {
                let mut j = i + 2;
                while j < b.len() && b[j] != b'\n' {
                    j += 1;
                }
                i = j;
            } else {
                let mut j = i + 2;
                while j + 1 < b.len() && !(b[j] == b'*' && b[j + 1] == b'/') {
                    j += 1;
                }
                i = if j + 1 < b.len() { j + 2 } else { b.len() };
            }
            copied = i;
        } else {
            i += 1;
        }
    }
    match out {
        Some(mut s) => {
            s.push_str(&input[copied..]);
            Cow::Owned(s)
        }
        None => Cow::Borrowed(input),
    }
}

// ─── tokenize ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
enum Tok<'a> {
    Word(&'a str),
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBrack,
    RBrack,
    Dot,
    Comma,
    Colon,
    Arrow, // =>
    Eq,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SpannedTok<'a> {
    tok: Tok<'a>,
    start: usize,
    end: usize,
}

fn tokenize(input: &str) -> Vec<SpannedTok<'_>> {
    let mut out = Vec::new();
    let b = input.as_bytes();
    let mut i = 0;
    while i < b.len() {
        let c = b[i] as char;
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        if c == '=' && i + 1 < b.len() && b[i + 1] == b'>' {
            out.push(SpannedTok {
                tok: Tok::Arrow,
                start: i,
                end: i + 2,
            });
            i += 2;
            continue;
        }
        // Quoted strings: keep as a single Word so motion-config brace depth
        // is not disturbed, and raw capture still sees quotes via spans.
        if c == '\'' || c == '"' {
            let q = b[i];
            let start = i;
            i += 1;
            while i < b.len() && b[i] != q {
                if b[i] == b'\\' && i + 1 < b.len() {
                    i += 1;
                }
                i += 1;
            }
            if i < b.len() {
                i += 1; // closing quote
            }
            out.push(SpannedTok {
                tok: Tok::Word(&input[start..i]),
                start,
                end: i,
            });
            continue;
        }
        let single = |tok: Tok<'static>, i: usize| SpannedTok {
            tok,
            start: i,
            end: i + 1,
        };
        match c {
            '(' => {
                out.push(single(Tok::LParen, i));
                i += 1;
                continue;
            }
            ')' => {
                out.push(single(Tok::RParen, i));
                i += 1;
                continue;
            }
            '{' => {
                out.push(single(Tok::LBrace, i));
                i += 1;
                continue;
            }
            '}' => {
                out.push(single(Tok::RBrace, i));
                i += 1;
                continue;
            }
            '[' => {
                out.push(single(Tok::LBrack, i));
                i += 1;
                continue;
            }
            ']' => {
                out.push(single(Tok::RBrack, i));
                i += 1;
                continue;
            }
            '.' => {
                out.push(single(Tok::Dot, i));
                i += 1;
                continue;
            }
            ',' => {
                out.push(single(Tok::Comma, i));
                i += 1;
                continue;
            }
            ':' => {
                out.push(single(Tok::Colon, i));
                i += 1;
                continue;
            }
            '=' => {
                out.push(single(Tok::Eq, i));
                i += 1;
                continue;
            }
            // Words: identifiers, tokens, and leading-minus tokens (`-px`).
            _ if c.is_ascii_alphanumeric() || c == '_' || c == '-' => {
                let start = i;
                let mut j = i + 1;
                while j < b.len() {
                    let ch = b[j] as char;
                    if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                        j += 1;
                    } else {
                        break;
                    }
                }
                // Bare `-` is not a word — leave it for the skip path.
                if &input[start..j] == "-" {
                    i = j;
                    continue;
                }
                out.push(SpannedTok {
                    tok: Tok::Word(&input[start..j]),
                    start,
                    end: j,
                });
                i = j;
                continue;
            }
            _ => {
                // skip unknown one char
                i += 1;
            }
        }
    }
    out
}

struct Parser<'a> {
    input: &'a str,
    tokens: &'a [SpannedTok<'a>],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str, tokens: &'a [SpannedTok<'a>]) -> Self {
        Self {
            input,
            tokens,
            pos: 0,
        }
    }
    fn at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }
    fn peek(&self) -> Option<&Tok<'a>> {
        self.tokens.get(self.pos).map(|t| &t.tok)
    }
    fn peek_str(&self) -> &str {
        match self.peek() {
            Some(Tok::Word(w)) => w,
            Some(Tok::LParen) => "(",
            Some(Tok::RParen) => ")",
            Some(Tok::LBrace) => "{",
            Some(Tok::RBrace) => "}",
            Some(Tok::LBrack) => "[",
            Some(Tok::RBrack) => "]",
            Some(Tok::Dot) => ".",
            Some(Tok::Comma) => ",",
            Some(Tok::Colon) => ":",
            Some(Tok::Arrow) => "=>",
            Some(Tok::Eq) => "=",
            None => "<eof>",
        }
    }
    fn bump(&mut self) -> Option<Tok<'a>> {
        let t = self.tokens.get(self.pos)?.tok.clone();
        self.pos += 1;
        Some(t)
    }
    fn eat(&mut self, t: &Tok<'static>) -> bool {
        if self.peek() == Some(t) {
            self.pos += 1;
            true
        } else {
            false
        }
    }
    fn expect(&mut self, t: Tok<'static>, diags: &mut Vec<StyleDiagnostic>) -> bool {
        if self.eat(&t) {
            true
        } else {
            diags.push(StyleDiagnostic::ParseError {
                message: format!("expected `{}`, found `{}`", tok_str(&t), self.peek_str()),
                source: self.remainder(),
            });
            false
        }
    }
    fn remainder(&self) -> String {
        self.tokens[self.pos..]
            .iter()
            .map(|s| tok_str_ref(&s.tok))
            .collect::<Vec<_>>()
            .join("")
    }

    fn force_progress(&mut self) {
        if !self.at_end() {
            self.pos += 1;
        }
    }

    /// Capture a balanced `{ … }` object as a raw source slice (quotes intact).
    fn capture_brace_object(&mut self, diags: &mut Vec<StyleDiagnostic>) -> Option<String> {
        if self.peek() != Some(&Tok::LBrace) {
            diags.push(StyleDiagnostic::ParseError {
                message: "expected `{…}` motion config object".into(),
                source: self.remainder(),
            });
            return None;
        }
        let start = self.tokens[self.pos].start;
        let mut depth = 0i32;
        while !self.at_end() {
            match self.peek() {
                Some(Tok::LBrace) => {
                    depth += 1;
                    self.pos += 1;
                }
                Some(Tok::RBrace) => {
                    depth -= 1;
                    self.pos += 1;
                    if depth == 0 {
                        let end = self.tokens[self.pos - 1].end;
                        return Some(self.input[start..end].to_string());
                    }
                }
                _ => {
                    self.pos += 1;
                }
            }
        }
        diags.push(StyleDiagnostic::ParseError {
            message: "unterminated `{…}` motion config object".into(),
            source: self.input[start..].chars().take(80).collect(),
        });
        None
    }
}

/// Cap map/list/chain iterations so a stuck parser cannot run unbounded.
const MAX_PARSE_ITERS: usize = 10_000;

fn tok_str<'a>(t: &'a Tok<'a>) -> &'a str {
    match t {
        Tok::Word(w) => w,
        Tok::LParen => "(",
        Tok::RParen => ")",
        Tok::LBrace => "{",
        Tok::RBrace => "}",
        Tok::LBrack => "[",
        Tok::RBrack => "]",
        Tok::Dot => ".",
        Tok::Comma => ",",
        Tok::Colon => ":",
        Tok::Arrow => "=>",
        Tok::Eq => "=",
    }
}
fn tok_str_ref<'a>(t: &'a Tok<'a>) -> &'a str {
    tok_str(t)
}

// ─── grammar ────────────────────────────────────────────────────────────────

fn parse_style_expr(p: &mut Parser<'_>, diags: &mut Vec<StyleDiagnostic>) -> Style {
    match p.peek() {
        Some(Tok::LBrack) => parse_list(p, diags),
        Some(Tok::Word("match")) => parse_match(p, diags),
        Some(Tok::Word("when")) => parse_when_call(p, diags),
        Some(Tok::Word("compound")) => parse_compound(p, diags),
        Some(Tok::Word("style")) => parse_style_chain(p, diags),
        Some(Tok::Word(_)) => {
            // Style const ref or start of something unexpected
            let name = match p.bump() {
                Some(Tok::Word(w)) => w.to_string(),
                _ => return Style::Empty,
            };
            // `filled.when({…})` / `filled.enter({…})`
            if p.peek() == Some(&Tok::Dot) {
                let mut base = Style::Ref(name);
                while p.eat(&Tok::Dot) {
                    let method = match p.bump() {
                        Some(Tok::Word(w)) => w,
                        _ => {
                            diags.push(StyleDiagnostic::ParseError {
                                message: "expected method after `.` on Style ref".into(),
                                source: p.remainder(),
                            });
                            break;
                        }
                    };
                    if method == "when" {
                        let entries = parse_when_map_body(p, diags);
                        base = Style::chain(vec![base, Style::When { entries }]);
                    } else if is_motion_chain_method(method) {
                        // `scroll` on a ref is motion only when the arg is `{…}`.
                        if method == "scroll" && p.peek() == Some(&Tok::LParen) {
                            // peek past `(` for brace — if not brace, not allowed on ref
                        }
                        if let Some(m) = parse_motion_call(p, method, diags) {
                            base = Style::chain(vec![base, m]);
                        }
                    } else {
                        diags.push(StyleDiagnostic::ParseError {
                            message: "only `.when({…})` / `.enter` / `.exit` / `.loop` / `.reveal` / `.scroll({…})` may follow a Style ref".into(),
                            source: p.remainder(),
                        });
                        break;
                    }
                }
                base
            } else {
                Style::Ref(name)
            }
        }
        _ => {
            diags.push(StyleDiagnostic::ParseError {
                message: "expected Style expression (style.…, match, when, compound, name, or […])"
                    .into(),
                source: p.remainder(),
            });
            Style::Empty
        }
    }
}

fn parse_list(p: &mut Parser<'_>, diags: &mut Vec<StyleDiagnostic>) -> Style {
    p.expect(Tok::LBrack, diags);
    let mut items = Vec::new();
    let mut iters = 0usize;
    while !p.at_end() && p.peek() != Some(&Tok::RBrack) {
        iters += 1;
        if iters > MAX_PARSE_ITERS {
            diags.push(StyleDiagnostic::ParseError {
                message: "style list too many entries".into(),
                source: p.remainder(),
            });
            break;
        }
        let before = p.pos;
        items.push(parse_style_expr(p, diags));
        if p.pos == before {
            // Malformed item — skip one token so we cannot spin forever.
            p.force_progress();
            continue;
        }
        if !p.eat(&Tok::Comma) {
            break;
        }
    }
    p.expect(Tok::RBrack, diags);
    Style::List(items)
}

fn parse_style_chain(p: &mut Parser<'_>, diags: &mut Vec<StyleDiagnostic>) -> Style {
    p.expect(Tok::Word("style"), diags);
    let mut ops = Vec::new();
    let mut iters = 0usize;
    loop {
        iters += 1;
        if iters > MAX_PARSE_ITERS {
            diags.push(StyleDiagnostic::ParseError {
                message: "style chain too many setters".into(),
                source: p.remainder(),
            });
            break;
        }
        if !p.eat(&Tok::Dot) {
            break;
        }
        let before = p.pos;
        let method = match p.bump() {
            Some(Tok::Word(w)) => w,
            _ => {
                diags.push(StyleDiagnostic::ParseError {
                    message: "expected setter name after `.`".into(),
                    source: p.remainder(),
                });
                break;
            }
        };
        // `.when({…})` — may appear anywhere in the chain (not only terminal).
        if method == "when" {
            let entries = parse_when_map_body(p, diags);
            ops.push(Style::When { entries });
            continue;
        }
        // Motion chain methods: `.enter({…})` etc. Config-object form only.
        // Dual paint/motion methods: only brace-object form is motion —
        //   `style.scroll(auto)` paint vs `style.scroll({ scale: '…' })` motion
        //   `style.press()` paint marker vs `style.press({ scale: 'press' })` motion
        // Pure motion methods (enter/exit/loop/reveal/hover/focus/focus-visible)
        // always take a config object.
        let motion_object = if is_motion_chain_method(method) {
            if p.peek() == Some(&Tok::LParen) {
                let after_paren = p.tokens.get(p.pos + 1).map(|t| &t.tok);
                if after_paren == Some(&Tok::LBrace) {
                    true
                } else if matches!(method, "scroll" | "press") {
                    // paint form
                    false
                } else {
                    // enter/exit/loop/reveal/hover/focus require a config object
                    true
                }
            } else {
                true
            }
        } else {
            false
        };
        if motion_object {
            if let Some(m) = parse_motion_call(p, method, diags) {
                ops.push(m);
            }
            if p.pos == before {
                p.force_progress();
            }
            continue;
        }
        p.expect(Tok::LParen, diags);
        // Empty-arg markers (style.group()) or comma-separated paint args
        // (`style.border(1, border)` — width + color in one call).
        let mut args = Vec::new();
        if p.peek() == Some(&Tok::RParen) {
            args.push(PaintArg::Token(String::new()));
        } else {
            args.push(parse_paint_arg(p, diags));
            while p.eat(&Tok::Comma) {
                args.push(parse_paint_arg(p, diags));
            }
        }
        p.expect(Tok::RParen, diags);
        if p.pos == before {
            p.force_progress();
            continue;
        }
        let arg_text = args
            .iter()
            .map(|a| match a {
                PaintArg::Token(t) => t.clone(),
                PaintArg::Bind(n) => format!("bind({n})"),
            })
            .collect::<Vec<_>>()
            .join(", ");
        let source = format!("style.{method}({arg_text})");
        if find_setter(method).is_none() {
            diags.push(StyleDiagnostic::UnknownSetter {
                setter: method.to_string(),
                source: source.clone(),
                suggestion: closest_unique_setter(method).map(|s| s.to_string()),
            });
        } else {
            let op = PaintOp {
                setter: method.to_string(),
                args: args.clone(),
                source: source.clone(),
            };
            if let Err(d) = validate_op(&op) {
                diags.push(d);
            }
        }
        ops.push(Style::Set(PaintOp {
            setter: method.to_string(),
            args,
            source,
        }));
    }
    if ops.is_empty() {
        Style::Empty
    } else {
        Style::chain(ops)
    }
}

/// Parse `( {…} )` after a motion method name has already been consumed.
fn parse_motion_call(
    p: &mut Parser<'_>,
    trigger: &str,
    diags: &mut Vec<StyleDiagnostic>,
) -> Option<Style> {
    p.expect(Tok::LParen, diags);
    let config = p.capture_brace_object(diags)?;
    p.expect(Tok::RParen, diags);
    Some(Style::MotionAttach {
        trigger: trigger.to_string(),
        config,
    })
}

fn parse_paint_arg(p: &mut Parser<'_>, diags: &mut Vec<StyleDiagnostic>) -> PaintArg {
    match p.bump() {
        Some(Tok::Word("bind")) => {
            p.expect(Tok::LParen, diags);
            let name = match p.bump() {
                Some(Tok::Word(w)) => w.to_string(),
                _ => {
                    diags.push(StyleDiagnostic::ParseError {
                        message: "bind(name) expects an identifier".into(),
                        source: p.remainder(),
                    });
                    String::new()
                }
            };
            p.expect(Tok::RParen, diags);
            PaintArg::Bind(name)
        }
        // Negatable token: style.ms(-px) tokenizes as Word("-px") after we allow leading '-'.
        Some(Tok::Word(w)) => PaintArg::Token(w.to_string()),
        _ => {
            diags.push(StyleDiagnostic::ParseError {
                message: "expected token or bind(name)".into(),
                source: p.remainder(),
            });
            PaintArg::Token(String::new())
        }
    }
}

fn parse_match(p: &mut Parser<'_>, diags: &mut Vec<StyleDiagnostic>) -> Style {
    // match(scrutinee, { arm: Style, … })
    p.bump();
    p.expect(Tok::LParen, diags);
    let scrutinee = parse_scrutinee(p, diags);
    p.expect(Tok::Comma, diags);
    p.expect(Tok::LBrace, diags);
    let mut arms = Vec::new();
    let mut iters = 0usize;
    while !p.at_end() && p.peek() != Some(&Tok::RBrace) {
        iters += 1;
        if iters > MAX_PARSE_ITERS {
            diags.push(StyleDiagnostic::ParseError {
                message: "match arms too many entries".into(),
                source: p.remainder(),
            });
            break;
        }
        let before = p.pos;
        let key = match p.bump() {
            Some(Tok::Word(w)) => unquote(w).to_string(),
            _ => {
                diags.push(StyleDiagnostic::ParseError {
                    message: "expected match arm key".into(),
                    source: p.remainder(),
                });
                p.force_progress();
                break;
            }
        };
        p.expect(Tok::Colon, diags);
        let body = parse_style_expr(p, diags);
        arms.push((key, body));
        p.eat(&Tok::Comma);
        if p.pos <= before {
            p.force_progress();
        }
    }
    p.expect(Tok::RBrace, diags);
    p.expect(Tok::RParen, diags);
    Style::Match { scrutinee, arms }
}

fn parse_scrutinee(p: &mut Parser<'_>, diags: &mut Vec<StyleDiagnostic>) -> String {
    // variant | props.variant | ($) => $.variant
    if p.peek() == Some(&Tok::LParen) {
        // static lambda: ( $ ) => $ . path
        p.bump();
        let _param = match p.bump() {
            Some(Tok::Word(w)) => w.to_string(),
            _ => {
                diags.push(StyleDiagnostic::BadScrutinee {
                    message: "match lambda must be static path: ($) => $.prop".into(),
                    source: p.remainder(),
                });
                return String::new();
            }
        };
        p.expect(Tok::RParen, diags);
        if !p.eat(&Tok::Arrow) {
            diags.push(StyleDiagnostic::BadScrutinee {
                message: "match lambda requires `=>`".into(),
                source: p.remainder(),
            });
            return String::new();
        }
        // $.prop or $ . prop
        let mut path = Vec::new();
        if let Some(Tok::Word(w)) = p.bump() {
            // might be $
            if w != "$" && w != _param {
                // allow bare prop after =>
                return w.to_string();
            }
        }
        while p.eat(&Tok::Dot) {
            if let Some(Tok::Word(seg)) = p.bump() {
                path.push(seg.to_string());
            }
        }
        if path.is_empty() {
            diags.push(StyleDiagnostic::BadScrutinee {
                message: "match lambda must select a prop path".into(),
                source: String::new(),
            });
            String::new()
        } else {
            path.join(".")
        }
    } else {
        // variant or props.variant
        let mut parts = Vec::new();
        if let Some(Tok::Word(w)) = p.bump() {
            parts.push(w.to_string());
        } else {
            diags.push(StyleDiagnostic::BadScrutinee {
                message: "match scrutinee must be a prop identifier".into(),
                source: p.remainder(),
            });
            return String::new();
        }
        while p.eat(&Tok::Dot) {
            if let Some(Tok::Word(w)) = p.bump() {
                parts.push(w.to_string());
            }
        }
        // props.variant → variant for attr emit
        if parts.len() > 1 && parts[0] == "props" {
            parts[1..].join(".")
        } else {
            parts.join(".")
        }
    }
}

fn parse_when_call(p: &mut Parser<'_>, diags: &mut Vec<StyleDiagnostic>) -> Style {
    p.bump(); // when
    let entries = parse_when_map_body(p, diags);
    Style::When { entries }
}

fn parse_when_map_body(
    p: &mut Parser<'_>,
    diags: &mut Vec<StyleDiagnostic>,
) -> Vec<(String, Style)> {
    p.expect(Tok::LParen, diags);
    p.expect(Tok::LBrace, diags);
    let mut entries = Vec::new();
    let mut iters = 0usize;
    while !p.at_end() && p.peek() != Some(&Tok::RBrace) {
        iters += 1;
        if iters > MAX_PARSE_ITERS {
            diags.push(StyleDiagnostic::ParseError {
                message: "when map too many entries".into(),
                source: p.remainder(),
            });
            break;
        }
        let before = p.pos;
        let key = match p.bump() {
            Some(Tok::Word(w)) => w.to_string(),
            _ => {
                p.force_progress();
                break;
            }
        };
        if when_target(&key).is_none() {
            diags.push(StyleDiagnostic::UnknownWhenKey {
                key: key.clone(),
                source: key.clone(),
            });
        }
        p.expect(Tok::Colon, diags);
        let body = parse_style_expr(p, diags);
        entries.push((key, body));
        p.eat(&Tok::Comma);
        if p.pos <= before {
            p.force_progress();
        }
    }
    p.expect(Tok::RBrace, diags);
    p.expect(Tok::RParen, diags);
    entries
}

/// Quoted literals arrive INSIDE the Word (the lexer keeps quotes so
/// motion-config brace depth is undisturbed) — condition/arm consumers need
/// the bare value, else selectors render `[offset=''8'']` and never match.
fn unquote(w: &str) -> &str {
    let b = w.as_bytes();
    if b.len() >= 2
        && ((b[0] == b'\'' && b[b.len() - 1] == b'\'') || (b[0] == b'"' && b[b.len() - 1] == b'"'))
    {
        &w[1..w.len() - 1]
    } else {
        w
    }
}

fn parse_compound(p: &mut Parser<'_>, diags: &mut Vec<StyleDiagnostic>) -> Style {
    p.bump(); // compound
    p.expect(Tok::LParen, diags);
    p.expect(Tok::LBrace, diags);
    let mut when = BTreeMap::new();
    let mut iters = 0usize;
    while !p.at_end() && p.peek() != Some(&Tok::RBrace) {
        iters += 1;
        if iters > MAX_PARSE_ITERS {
            diags.push(StyleDiagnostic::ParseError {
                message: "compound map too many entries".into(),
                source: p.remainder(),
            });
            break;
        }
        let before = p.pos;
        let k = match p.bump() {
            Some(Tok::Word(w)) => unquote(w).to_string(),
            _ => {
                p.force_progress();
                break;
            }
        };
        p.expect(Tok::Colon, diags);
        let v = match p.bump() {
            Some(Tok::Word(w)) => unquote(w).to_string(),
            _ => {
                diags.push(StyleDiagnostic::ParseError {
                    message: "compound condition value must be a literal".into(),
                    source: p.remainder(),
                });
                String::new()
            }
        };
        when.insert(k, v);
        p.eat(&Tok::Comma);
        if p.pos <= before {
            p.force_progress();
        }
    }
    p.expect(Tok::RBrace, diags);
    p.expect(Tok::Comma, diags);
    let body = parse_style_expr(p, diags);
    p.expect(Tok::RParen, diags);
    Style::Compound {
        when,
        body: Box::new(body),
    }
}

fn validate_style(style: &Style, diags: &mut Vec<StyleDiagnostic>) {
    match style {
        Style::Empty | Style::Ref(_) | Style::MotionAttach { .. } => {}
        Style::Set(op) => {
            if let Err(e) = validate_op(op) {
                diags.push(e);
            }
        }
        Style::Chain(items) | Style::List(items) => {
            for i in items {
                validate_style(i, diags);
            }
        }
        Style::Match { arms, .. } => {
            for (_, s) in arms {
                validate_style(s, diags);
            }
        }
        Style::Compound { body, .. } => validate_style(body, diags),
        Style::When { entries } => {
            for (_, s) in entries {
                validate_style(s, diags);
            }
        }
    }
}

/// Parse `const name = <style-expr>` (scope Style const line, without the `const` keyword optional).
pub fn parse_style_const_rhs(input: &str) -> (Style, Vec<StyleDiagnostic>) {
    parse(input)
}

#[cfg(test)]
mod utility_detect_tests {
    use super::looks_like_utility_string_product;
    #[test]
    fn const_press_is_not_utility() {
        assert!(!looks_like_utility_string_product(
            "cta.press({ scale: 'press' })"
        ));
    }
    #[test]
    fn atoms_are_utility() {
        assert!(looks_like_utility_string_product("p-4 bg-muted"));
    }
}
