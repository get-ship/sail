//! Parser for the Style value language (chain + match / when / compound / bind).

use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap, HashSet};

use super::channel::is_animate_trigger;
use super::intent::is_known_when_key;
use super::ir::{PaintArg, PaintOp, Style, StyleDiagnostic};
use super::setters::validate_op;
use super::theme_env::{PlayPreset, ThemeStyleEnv, current_styling_policy};

/// Parse a Style expression. Flat `style.method(…)` chains are refused —
/// use `style({ pad(4).bg(surface), when: { hover: bg(muted) } })`.
pub fn parse(input: &str) -> (Style, Vec<StyleDiagnostic>) {
    parse_with(input)
}

fn parse_with(input: &str) -> (Style, Vec<StyleDiagnostic>) {
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
            message: residual_product_message(trimmed),
            source: trimmed.chars().take(80).collect(),
        });
        return (Style::Empty, diags);
    }
    // Offsets are into `trimmed`; `input_base` maps them back if needed. Motion
    // config capture uses the trimmed slice so quotes/units survive.
    let tokens = tokenize(trimmed);
    let mut p = Parser::new_full(trimmed, &tokens, false);
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
    // Interaction state cannot gate geometry.
    if let Err(d) = forbid_geometry_inside_interaction_when(&style) {
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
///
/// Does not resolve `Style::Ref` — callers that have a [`StyleEnv`] must use
/// [`collect_motion_attaches_env`]. Tag-position `style(named)` is a Ref; if
/// collection skips the env, the named style's `animate:` channel is dropped
/// and motion attaches never reach the runtime (dead loops / enters with full
/// paint classes still applied — the site red that sampling `translate` catches).
pub fn collect_motion_attaches(style: &Style) -> Vec<(String, String)> {
    collect_motion_attaches_env(style, &std::collections::HashMap::new())
}

/// Like [`collect_motion_attaches`], resolving scope Style consts via `env`.
///
/// CSS emit already expands `Style::Ref` through the same env; motion collection
/// must follow the same path or `style(name)` / multi-ref `style(a, b)` loses
/// every `MotionAttach` that lives only on the named declaration.
pub fn collect_motion_attaches_env(
    style: &Style,
    env: &std::collections::HashMap<String, Style>,
) -> Vec<(String, String)> {
    let mut out = Vec::new();
    collect_motion_attaches_inner(style, env, &mut out, &mut std::collections::HashSet::new());
    out
}

fn collect_motion_attaches_inner(
    style: &Style,
    env: &std::collections::HashMap<String, Style>,
    out: &mut Vec<(String, String)>,
    resolving: &mut std::collections::HashSet<String>,
) {
    match style {
        Style::Empty | Style::Set(_) => {}
        Style::MotionAttach { trigger, config } => {
            // `animate: { transition: play(…) }` is CSS emit, not a runtime
            // motion trigger. Do not hand it to the motion-attach compiler.
            if trigger != "transition" {
                out.push((trigger.clone(), config.clone()));
            }
        }
        Style::Ref(name) => {
            // Cycle guard mirrors emit_inner / style_is_textish_env.
            if !resolving.insert(name.clone()) {
                return;
            }
            if let Ok(s) =
                ThemeStyleEnv::from_registry().expand_style_ref(name, env, current_styling_policy())
            {
                collect_motion_attaches_inner(&s, env, out, resolving);
            }
            resolving.remove(name);
        }
        Style::Chain(items) | Style::List(items) => {
            for i in items {
                collect_motion_attaches_inner(i, env, out, resolving);
            }
        }
        Style::Match { arms, .. } => {
            for (_, body) in arms {
                collect_motion_attaches_inner(body, env, out, resolving);
            }
        }
        Style::Compound { body, .. } => collect_motion_attaches_inner(body, env, out, resolving),
        Style::When { entries } => {
            for (_, body) in entries {
                collect_motion_attaches_inner(body, env, out, resolving);
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
                            "lifecycle / ambient motion cannot appear inside when({{ {key}: … }}) — put `play(…)` on the head `animate:` map: style({{ …, animate: {{ enter: play(…) }} }})"
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

/// Interaction / pseudo `when` keys must not gate geometry ops.
/// Media/viewport `when(tablet|desktop|…)` may still gate geometry.
fn forbid_geometry_inside_interaction_when(style: &Style) -> Result<(), StyleDiagnostic> {
    match style {
        Style::When { entries } => {
            for (key, body) in entries {
                if super::capabilities::is_interaction_when_key(key)
                    && super::capabilities::style_contains_geometry(body)
                {
                    return Err(StyleDiagnostic::ParseError {
                        message: format!(
                            "geometry (pad/size/margin/grow/…) cannot appear inside \
                             interaction when({key}) — state can't gate layout. Put geometry on \
                             the rest chain or under media/viewport when(tablet|desktop|…)."
                        ),
                        source: format!("when({key})"),
                    });
                }
                forbid_geometry_inside_interaction_when(body)?;
            }
            Ok(())
        }
        Style::Chain(items) | Style::List(items) => {
            for i in items {
                forbid_geometry_inside_interaction_when(i)?;
            }
            Ok(())
        }
        Style::Match { arms, .. } => {
            for (_, body) in arms {
                forbid_geometry_inside_interaction_when(body)?;
            }
            Ok(())
        }
        Style::Compound { body, .. } => forbid_geometry_inside_interaction_when(body),
        _ => Ok(()),
    }
}

fn residual_product_message(src: &str) -> String {
    let base = "utility-string sx is not the Style product — use style({ pad(4).bg(surface), when:, match:, compound: })";
    let taught = crate::foreign_idioms::idioms_in_source(src);
    if taught.is_empty() {
        return base.into();
    }
    let pointers: Vec<String> = taught.iter().map(|i| i.teaching_line()).collect();
    format!("{base} — {}", pointers.join("; "))
}

fn teach_html_attr_eq(p: &mut Parser<'_>, name: &str, diags: &mut Vec<StyleDiagnostic>) -> bool {
    if p.peek() != Some(&Tok::Eq) {
        return false;
    }
    let habit = match name {
        "class" | "className" => format!("{name}="),
        "style" => "style=".into(),
        _ => return false,
    };
    p.bump(); // =
    if matches!(p.peek(), Some(Tok::Word(_))) {
        p.bump();
    }
    diags.push(StyleDiagnostic::UnknownVocabulary {
        name: habit.clone(),
        source: habit,
    });
    true
}

/// True if input is the killed utility-string product (atoms / prop: value soup without style./match/when).
pub fn looks_like_utility_string_product(input: &str) -> bool {
    let raw = input.trim();
    // Bare Style object form (scope `style name = {…}` / style() args).
    // Must not trip on `paint:` / `box:` colons — those are channel keys, not
    // utility-string soup (`hover:bg-muted`).
    if raw.starts_with('{') {
        return false;
    }
    // Compose form: `host, { … }` / `host, style({…})` (declaration RHS).
    if looks_like_style_compose_rhs(raw) {
        return false;
    }
    // Preprocess may pass `{expr}` (outer brace wrapper from quote_brace_attrs).
    let t = strip_one_outer_brace_layer(raw);
    if t.starts_with("style.")
        || t.starts_with("style(")
        || t.starts_with("match(")
        || t.starts_with("when(")
        || t.starts_with("compound(")
        || t.starts_with('[')
        || t.starts_with('{')
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
    // Space-separated utility soup or dashed atoms
    t.contains(':')
        || t.split_whitespace()
            .any(|w| w.contains('-') && !w.starts_with("style"))
}

/// `name, {…}` / `name, style(…)` / multi-arg compose heads for declaration RHS.
fn looks_like_style_compose_rhs(input: &str) -> bool {
    let t = input.trim();
    let Some((head, rest)) = t.split_once(',') else {
        return false;
    };
    let head = head.trim();
    if head.is_empty() || !head.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') {
        return false;
    }
    let rest = rest.trim_start();
    rest.starts_with('{')
        || rest.starts_with("style(")
        || rest.starts_with("style.")
        || rest
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b',' || b.is_ascii_whitespace())
}

fn strip_one_outer_brace_layer(t: &str) -> &str {
    let t = t.trim();
    if !(t.starts_with('{') && t.ends_with('}') && t.len() >= 2) {
        return t;
    }
    let inn = t[1..t.len() - 1].trim();
    // Strip when the outer pair wraps a whole Style expr (not a bare `{k: v}` map).
    if inn.starts_with("style.")
        || inn.starts_with("style(")
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

/// Detect retired `name.enter({…})` tails so they are not classified as
/// utility-string product. The parser still refuses them with a teaching
/// diagnostic pointing at `animate: { enter: play(…) }`.
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

/// Public for the s-attribute classifier (`rules::sx`): leftover
/// `row.enter({…})` is not a utility string — parse it and teach the play map.
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
            // Compound any-set wildcard (`tone: *`).
            '*' => {
                out.push(SpannedTok {
                    tok: Tok::Word("*"),
                    start: i,
                    end: i + 1,
                });
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
    /// When false, `style.method(…)` flat chains are refused (product parse).
    allow_flat: bool,
}

impl<'a> Parser<'a> {
    fn new_full(input: &'a str, tokens: &'a [SpannedTok<'a>], allow_flat: bool) -> Self {
        Self {
            input,
            tokens,
            pos: 0,
            allow_flat,
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
        Some(Tok::Word("style")) => parse_style_chain(p, diags),
        Some(Tok::Word(_)) => {
            // Style const ref. Motion tails (`name.enter({…})`) are retired.
            let name = match p.bump() {
                Some(Tok::Word(w)) => w.to_string(),
                _ => return Style::Empty,
            };
            if teach_html_attr_eq(p, &name, diags) {
                return Style::Empty;
            }
            if p.peek() == Some(&Tok::Dot) {
                let base = make_style_ref(name, diags);
                refuse_style_ref_tails(p, diags);
                base
            } else {
                make_style_ref(name, diags)
            }
        }
        _ => {
            diags.push(StyleDiagnostic::ParseError {
                message: "expected Style expression `style(partial, …)` or a Style const name"
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
    if p.peek() == Some(&Tok::LParen) {
        return parse_style_call(p, diags);
    }
    // Flat `style.method(…)` — product parse refuses; recovery consumes the chain.
    if !p.allow_flat {
        let source = p.remainder();
        let preview: String = source.chars().take(80).collect();
        diags.push(StyleDiagnostic::ParseError {
            message: "flat `style.method(…)` chains are retired — write concern calls: \
                      style({ size({ w: full }).pad(4).bg(surface), when: { hover: bg(muted) } })"
                .into(),
            source: if preview.is_empty() {
                "style.".into()
            } else {
                format!("style.{preview}")
            },
        });
        // Recovery: consume the flat chain so outer when/match/list can continue.
        let saved = p.allow_flat;
        p.allow_flat = true;
        let mut sink = Vec::new();
        let _ = flat_chain_style_chain_body(p, &mut sink);
        p.allow_flat = saved;
        return Style::Empty;
    }
    flat_chain_style_chain_body(p, diags)
}

/// Body of a flat `style.method(…).…` chain (caller already ate `style`).
fn flat_chain_style_chain_body(p: &mut Parser<'_>, diags: &mut Vec<StyleDiagnostic>) -> Style {
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
        // `.when(…)` — map form or cond form; may appear anywhere in the chain.
        if method == "when" {
            let entries = parse_when_args(p, diags);
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
        // Temporary rename pointer: box-sizing method was `box`, now `boxSizing`
        // so the channel key `box` is free.
        if method == "box" {
            let args = parse_paren_args(p, diags, method);
            let arg_text = format_arg_text(&args);
            diags.push(StyleDiagnostic::ParseError {
                message: "`style.box(…)` was renamed to `style.boxSizing(…)` for box-sizing".into(),
                source: format!("style.box({arg_text})"),
            });
            if p.pos == before {
                p.force_progress();
            }
            continue;
        }
        let args = parse_paren_args(p, diags, method);
        if p.pos == before {
            p.force_progress();
            continue;
        }
        let arg_text = format_arg_text(&args);
        let source = format!("style.{method}({arg_text})");
        push_paint_op(&mut ops, method, args, source, diags);
    }
    if ops.is_empty() {
        Style::Empty
    } else {
        Style::chain(ops)
    }
}

fn parse_style_call(p: &mut Parser<'_>, diags: &mut Vec<StyleDiagnostic>) -> Style {
    p.expect(Tok::LParen, diags);
    // Variadic composition: style(partial, partial, …). Empty style() → Empty.
    if p.peek() == Some(&Tok::RParen) {
        p.bump();
        return Style::Empty;
    }
    let mut parts = Vec::new();
    let mut iters = 0usize;
    loop {
        iters += 1;
        if iters > MAX_PARSE_ITERS {
            diags.push(StyleDiagnostic::ParseError {
                message: "style() too many arguments".into(),
                source: p.remainder(),
            });
            break;
        }
        parts.push(parse_style_arg(p, diags));
        if !p.eat(&Tok::Comma) {
            break;
        }
        // trailing comma
        if p.peek() == Some(&Tok::RParen) {
            break;
        }
    }
    p.expect(Tok::RParen, diags);

    // Residual product tails: `.when(…)` method chains — deleted product surface.
    // Variadic composition is ordered List (later args shadow), not Chain
    // (Chain = conjunction, duplicate slots clash). SPEC merge law: args L→R.
    let base = match parts.len() {
        0 => Style::Empty,
        1 => parts.swap_remove(0),
        _ => Style::List(parts),
    };
    let mut iters = 0usize;
    while p.eat(&Tok::Dot) {
        iters += 1;
        if iters > MAX_PARSE_ITERS {
            diags.push(StyleDiagnostic::ParseError {
                message: "style channel tail too long".into(),
                source: p.remainder(),
            });
            break;
        }
        let method = match p.bump() {
            Some(Tok::Word(w)) => w,
            _ => {
                diags.push(StyleDiagnostic::ParseError {
                    message: "unexpected `.` after `style(…)` — conditionality is object keys (`when:`, `match:`, `compound:`), not method chains".into(),
                    source: p.remainder(),
                });
                break;
            }
        };
        if method == "when" {
            diags.push(StyleDiagnostic::ParseError {
                message: "`.when(…)` method chains are retired — use object key `when: { hover: {…}, … }` inside `style({…})`".into(),
                source: ".when".into(),
            });
            let _ = parse_when_args(p, diags);
            continue;
        }
        if is_motion_chain_method(method) {
            diags.push(StyleDiagnostic::ParseError {
                message: format!(
                    "`.{method}({{…}})` after `style({{…}})` is retired — write \
                     `style({{ …, animate: {{ {method}: play(…) }} }})`"
                ),
                source: format!(".{method}"),
            });
            let _ = parse_motion_call(p, method, diags);
            continue;
        }
        diags.push(StyleDiagnostic::ParseError {
            message: format!(
                "method `.{method}` is not valid after `style(…)` — use object keys for conditionality"
            ),
            source: p.remainder(),
        });
        break;
    }
    base
}

/// One `style(…)` argument: StyleObj (top-level axes OK), StyleRef, or
/// residual free match/when/compound/list (refused).
fn parse_style_arg(p: &mut Parser<'_>, diags: &mut Vec<StyleDiagnostic>) -> Style {
    match p.peek() {
        Some(Tok::LBrace) => parse_style_object(p, diags, /* allow_axes */ true),
        Some(Tok::Word("style")) => parse_style_chain(p, diags),
        Some(Tok::Word("match")) | Some(Tok::Word("when")) | Some(Tok::Word("compound")) => {
            let name = match p.peek() {
                Some(Tok::Word(w)) => *w,
                _ => "?",
            };
            diags.push(StyleDiagnostic::ParseError {
                message: format!(
                    "free `{name}(…)` is retired — use object keys inside style({{ {name}: … }})"
                ),
                source: name.into(),
            });
            p.bump();
            skip_style_object_value(p, diags);
            Style::Empty
        }
        Some(Tok::LBrack) => {
            diags.push(StyleDiagnostic::ParseError {
                message: "`style([…])` list form is retired — use `style(partial, partial, …)` composition".into(),
                source: "[".into(),
            });
            let _ = parse_list(p, diags);
            Style::Empty
        }
        Some(Tok::Word(_)) => {
            let next = p.tokens.get(p.pos + 1).map(|t| &t.tok);
            if next == Some(&Tok::LParen) {
                // Inline sugar: `style(pad({ t: 2 }))` / `style(type(body-sm).fg(…))`.
                return parse_channel_chain(p, diags);
            }
            if next == Some(&Tok::Colon) {
                if let Some(Tok::Word(w)) = p.peek() {
                    if matches!(*w, "when" | "animate" | "match" | "compound") {
                        let key = *w;
                        diags.push(StyleDiagnostic::ParseError {
                            message: format!(
                                "`{key}:` needs the declared form — not a `style()` tag argument"
                            ),
                            source: format!("{key}:"),
                        });
                        p.bump();
                        p.expect(Tok::Colon, diags);
                        skip_style_object_value(p, diags);
                        return Style::Empty;
                    }
                }
            }
            let name = match p.bump() {
                Some(Tok::Word(w)) => w.to_string(),
                _ => return Style::Empty,
            };
            if teach_html_attr_eq(p, &name, diags) {
                return Style::Empty;
            }
            if p.peek() == Some(&Tok::Dot) {
                let base = make_style_ref(name, diags);
                refuse_style_ref_tails(p, diags);
                base
            } else {
                make_style_ref(name, diags)
            }
        }
        _ => {
            diags.push(StyleDiagnostic::ParseError {
                message: "expected style partial: `{…}` object or name ref".into(),
                source: p.remainder(),
            });
            Style::Empty
        }
    }
}

/// Style object: concern/setter calls (`size({ w: full }).bg(card)`), plus
/// keyed members (`when:`, `match:`, `animate:`). Channels are dissolved.
///
/// `allow_axes`: `match:` / `compound:` legal only at the top level of a
/// `style(…)` argument. Nested partials (when bodies, match arms, compound
/// row bodies) refuse axes — intersections live in compound, flat.
///
/// Merge law (structure, not source key order):
/// channels < when < match arms < compound rows (later rows win).
fn parse_style_object(
    p: &mut Parser<'_>,
    diags: &mut Vec<StyleDiagnostic>,
    allow_axes: bool,
) -> Style {
    p.expect(Tok::LBrace, diags);
    let mut animate_ch = Style::Empty;
    let mut chain_parts: Vec<Style> = Vec::new();
    let mut when_entries: Vec<(String, Style)> = Vec::new();
    // prop → arms (later match tables for same prop merge arms; same-key arms chain)
    let mut match_tables: BTreeMap<String, BTreeMap<String, Style>> = BTreeMap::new();
    let mut match_order: Vec<String> = Vec::new();
    let mut compounds: Vec<Style> = Vec::new();
    let mut iters = 0usize;
    while !p.at_end() && p.peek() != Some(&Tok::RBrace) {
        iters += 1;
        if iters > MAX_PARSE_ITERS {
            diags.push(StyleDiagnostic::ParseError {
                message: "style object too many entries".into(),
                source: p.remainder(),
            });
            break;
        }
        let before = p.pos;
        let next = p.tokens.get(p.pos + 1).map(|t| &t.tok);
        // Word + `:` is a keyed member. Word + `(` (or a dotted call) is a
        // flat chain segment — commas separate calls the same way dots do.
        let keyed = matches!(p.peek(), Some(Tok::Word(_))) && next == Some(&Tok::Colon);
        if !keyed {
            let segment = parse_channel_chain(p, diags);
            if !matches!(segment, Style::Empty) {
                chain_parts.push(segment);
            }
            p.eat(&Tok::Comma);
            if p.pos <= before {
                p.force_progress();
            }
            continue;
        }
        let key = match p.bump() {
            Some(Tok::Word(w)) => w,
            _ => {
                p.force_progress();
                break;
            }
        };
        p.expect(Tok::Colon, diags);
        match key {
            "box" | "paint" | "text" => {
                diags.push(StyleDiagnostic::ParseError {
                    message: format!(
                        "`{key}:` is not a style member — write concern calls: \
                         style({{ size({{ w: full }}).pad(4).bg(surface), when: {{ hover: bg(muted) }} }})"
                    ),
                    source: format!("{key}:"),
                });
                let _ = parse_channel_chain(p, diags);
            }
            "animate" => animate_ch = parse_animate_channel_chain(p, diags),
            "motion" => {
                diags.push(StyleDiagnostic::ParseError {
                    message: "style channel `motion` was renamed to `animate`".into(),
                    source: "motion".into(),
                });
                let _ = parse_animate_channel_chain(p, diags);
            }
            "when" => {
                let entries = parse_when_block_map(p, diags);
                when_entries.extend(entries);
            }
            "match" => {
                if !allow_axes {
                    diags.push(StyleDiagnostic::ParseError {
                        message: "`match:` is only legal at the top level of a style() object argument — use `compound:` for intersections".into(),
                        source: "match".into(),
                    });
                    // Recovery: skip a brace object.
                    let _ = parse_match_block_map(p, diags);
                } else {
                    let tables = parse_match_block_map(p, diags);
                    for (prop, arms) in tables {
                        if is_token_typed_match_axis(&prop) {
                            diags.push(StyleDiagnostic::BadScrutinee {
                                message: format!(
                                    "token-typed prop `{prop}` cannot be a match axis — bind `{prop}` in paint (e.g. `bg({prop})`)"
                                ),
                                source: prop.clone(),
                            });
                        }
                        if !match_tables.contains_key(&prop) {
                            match_order.push(prop.clone());
                        }
                        let slot = match_tables.entry(prop).or_default();
                        for (arm_key, body) in arms {
                            slot.entry(arm_key)
                                .and_modify(|existing| {
                                    *existing = Style::List(vec![existing.clone(), body.clone()]);
                                })
                                .or_insert(body);
                        }
                    }
                }
            }
            "compound" => {
                if !allow_axes {
                    diags.push(StyleDiagnostic::ParseError {
                        message: "`compound:` is only legal at the top level of a style() object argument".into(),
                        source: "compound".into(),
                    });
                    // Recovery skip array
                    if p.peek() == Some(&Tok::LBrack) {
                        let _ = parse_compound_block_array(p, diags);
                    } else {
                        p.force_progress();
                    }
                } else {
                    compounds.extend(parse_compound_block_array(p, diags));
                }
            }
            other => {
                if crate::foreign_idioms::find_foreign_idiom(other).is_some() {
                    diags.push(StyleDiagnostic::UnknownVocabulary {
                        name: other.to_string(),
                        source: other.to_string(),
                    });
                } else {
                    diags.push(StyleDiagnostic::ParseError {
                        message: format!(
                            "unknown style object key `{other}` — expected a concern call (`bg(…)`) or `animate`/`when`/`match`/`compound`"
                        ),
                        source: other.to_string(),
                    });
                }
                // Recovery: try to skip a value.
                skip_style_object_value(p, diags);
            }
        }
        p.eat(&Tok::Comma);
        if p.pos <= before {
            p.force_progress();
        }
    }
    p.expect(Tok::RBrace, diags);

    // Flat concern chain, then animate:/when:/match:/compound (SPEC merge law).
    let mut parts = Vec::new();
    if !chain_parts.is_empty() {
        parts.push(Style::chain(chain_parts));
    }
    parts.push(animate_ch);
    if !when_entries.is_empty() {
        parts.push(Style::When {
            entries: when_entries,
        });
    }
    for prop in match_order {
        if let Some(arms_map) = match_tables.remove(&prop) {
            let arms: Vec<(String, Style)> = arms_map.into_iter().collect();
            parts.push(Style::Match {
                scrutinee: prop,
                arms,
            });
        }
    }
    parts.extend(compounds);
    // Drop empty channel slots so single-channel objects stay `Set`/`Chain`.
    parts.retain(|p| !matches!(p, Style::Empty));
    // Ordered List: later compounds/when/match shadow earlier (SPEC merge law).
    // Chain would conjunction-clash dual rows that intentionally override.
    match parts.len() {
        0 => Style::Empty,
        1 => parts.swap_remove(0),
        _ => Style::List(parts),
    }
}

/// Token-typed props refused as match axes (bind in paint instead).
///
/// `tone` remains the product convention name for the Tone axis; any
/// Color-group typed prop binds in paint via group paths, not match expansion.
fn is_token_typed_match_axis(prop: &str) -> bool {
    prop == "tone"
}

/// `when: { hover: partial, … }` — values are nested partials (no axes).
fn parse_when_block_map(
    p: &mut Parser<'_>,
    diags: &mut Vec<StyleDiagnostic>,
) -> Vec<(String, Style)> {
    p.expect(Tok::LBrace, diags);
    let mut entries: Vec<(String, Style)> = Vec::new();
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
        if !is_known_when_key(&key) {
            diags.push(StyleDiagnostic::UnknownWhenKey {
                key: key.clone(),
                source: key.clone(),
            });
        }
        p.expect(Tok::Colon, diags);
        let body = parse_partial(p, diags, /* allow_axes */ false);
        // Duplicate when keys (from codemod fold of sequential when()) compose
        // as ordered List so later shadows earlier — same as composition args.
        if let Some((_, existing)) = entries.iter_mut().find(|(k, _)| k == &key) {
            *existing = Style::List(vec![existing.clone(), body]);
        } else {
            entries.push((key, body));
        }
        p.eat(&Tok::Comma);
        if p.pos <= before {
            p.force_progress();
        }
    }
    p.expect(Tok::RBrace, diags);
    entries
}

/// `match: { prop: { value: partial, … }, … }`
fn parse_match_block_map(
    p: &mut Parser<'_>,
    diags: &mut Vec<StyleDiagnostic>,
) -> Vec<(String, Vec<(String, Style)>)> {
    p.expect(Tok::LBrace, diags);
    let mut tables = Vec::new();
    let mut iters = 0usize;
    while !p.at_end() && p.peek() != Some(&Tok::RBrace) {
        iters += 1;
        if iters > MAX_PARSE_ITERS {
            diags.push(StyleDiagnostic::ParseError {
                message: "match block too many axes".into(),
                source: p.remainder(),
            });
            break;
        }
        let before = p.pos;
        let prop = match p.bump() {
            Some(Tok::Word(w)) => unquote(w).to_string(),
            _ => {
                p.force_progress();
                break;
            }
        };
        p.expect(Tok::Colon, diags);
        let arms = parse_match_arms_object(p, diags);
        tables.push((prop, arms));
        p.eat(&Tok::Comma);
        if p.pos <= before {
            p.force_progress();
        }
    }
    p.expect(Tok::RBrace, diags);
    tables
}

fn parse_match_arms_object(
    p: &mut Parser<'_>,
    diags: &mut Vec<StyleDiagnostic>,
) -> Vec<(String, Style)> {
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
        let body = parse_partial(p, diags, /* allow_axes */ false);
        arms.push((key, body));
        p.eat(&Tok::Comma);
        if p.pos <= before {
            p.force_progress();
        }
    }
    p.expect(Tok::RBrace, diags);
    arms
}

/// `compound: [ { prop: val|*, channels…, when… }, … ]`
fn parse_compound_block_array(p: &mut Parser<'_>, diags: &mut Vec<StyleDiagnostic>) -> Vec<Style> {
    p.expect(Tok::LBrack, diags);
    let mut rows = Vec::new();
    let mut iters = 0usize;
    while !p.at_end() && p.peek() != Some(&Tok::RBrack) {
        iters += 1;
        if iters > MAX_PARSE_ITERS {
            diags.push(StyleDiagnostic::ParseError {
                message: "compound array too many rows".into(),
                source: p.remainder(),
            });
            break;
        }
        let before = p.pos;
        rows.push(parse_compound_row(p, diags));
        p.eat(&Tok::Comma);
        if p.pos <= before {
            p.force_progress();
        }
    }
    p.expect(Tok::RBrack, diags);
    rows
}

/// One compound row object: prop predicates + partial style entries.
fn parse_compound_row(p: &mut Parser<'_>, diags: &mut Vec<StyleDiagnostic>) -> Style {
    p.expect(Tok::LBrace, diags);
    let mut when = BTreeMap::new();
    let mut animate_ch = Style::Empty;
    let mut when_entries: Vec<(String, Style)> = Vec::new();
    let mut extra: Vec<Style> = Vec::new();
    let mut iters = 0usize;
    while !p.at_end() && p.peek() != Some(&Tok::RBrace) {
        iters += 1;
        if iters > MAX_PARSE_ITERS {
            diags.push(StyleDiagnostic::ParseError {
                message: "compound row too many entries".into(),
                source: p.remainder(),
            });
            break;
        }
        let before = p.pos;
        let key = match p.bump() {
            Some(Tok::Word(w)) => unquote(w).to_string(),
            _ => {
                p.force_progress();
                break;
            }
        };
        // Concern-call tail: `{tone: muted, bg(primary).fg(foreground)}`.
        if p.peek() == Some(&Tok::LParen) {
            p.pos -= 1;
            extra.push(parse_channel_chain(p, diags));
            p.eat(&Tok::Comma);
            if p.pos <= before {
                p.force_progress();
            }
            continue;
        }
        p.expect(Tok::Colon, diags);
        match key.as_str() {
            "box" | "paint" | "text" => {
                diags.push(StyleDiagnostic::ParseError {
                    message: format!(
                        "`{key}:` is not legal in a compound row — write concern calls (`bg(muted)`), not channels"
                    ),
                    source: key.clone(),
                });
                let _ = parse_channel_chain(p, diags);
            }
            "animate" => animate_ch = parse_animate_channel_chain(p, diags),
            "when" => {
                when_entries.extend(parse_when_block_map(p, diags));
            }
            "match" | "compound" | "motion" => {
                diags.push(StyleDiagnostic::ParseError {
                    message: format!(
                        "`{key}:` is not legal inside a compound row — only prop predicates, concern calls, and `when:`"
                    ),
                    source: key.clone(),
                });
                skip_style_object_value(p, diags);
            }
            _ => {
                // Prop predicate: literal or `*`.
                let v = match p.bump() {
                    Some(Tok::Word(w)) => unquote(w).to_string(),
                    _ => {
                        diags.push(StyleDiagnostic::ParseError {
                            message: "compound predicate value must be a literal or `*`".into(),
                            source: p.remainder(),
                        });
                        String::new()
                    }
                };
                when.insert(key, v);
            }
        }
        p.eat(&Tok::Comma);
        if p.pos <= before {
            p.force_progress();
        }
    }
    p.expect(Tok::RBrace, diags);
    if when.is_empty() {
        diags.push(StyleDiagnostic::ParseError {
            message: "compound row requires ≥1 prop predicate".into(),
            source: "compound".into(),
        });
    }
    let mut body_parts = vec![animate_ch];
    body_parts.extend(extra);
    if !when_entries.is_empty() {
        body_parts.push(Style::When {
            entries: when_entries,
        });
    }
    Style::Compound {
        when,
        body: Box::new(Style::chain(body_parts)),
    }
}

/// Partial: StyleObj | StyleRef | style(…) | residual free forms.
fn parse_partial(p: &mut Parser<'_>, diags: &mut Vec<StyleDiagnostic>, allow_axes: bool) -> Style {
    match p.peek() {
        Some(Tok::LBrace) => parse_style_object(p, diags, allow_axes),
        Some(Tok::Word("style")) => parse_style_chain(p, diags),
        Some(Tok::Word(_)) => {
            let next = p.tokens.get(p.pos + 1).map(|t| &t.tok);
            if next == Some(&Tok::LParen) {
                // Bare chain `bg(primary)` as partial (when cond-form recovery).
                parse_channel_chain(p, diags)
            } else if next == Some(&Tok::Dot) {
                parse_style_expr(p, diags)
            } else {
                let name = match p.bump() {
                    Some(Tok::Word(w)) => w.to_string(),
                    _ => return Style::Empty,
                };
                make_style_ref(name, diags)
            }
        }
        _ => {
            diags.push(StyleDiagnostic::ParseError {
                message: "expected style partial (`{…}` or name)".into(),
                source: p.remainder(),
            });
            Style::Empty
        }
    }
}

fn skip_style_object_value(p: &mut Parser<'_>, diags: &mut Vec<StyleDiagnostic>) {
    match p.peek() {
        Some(Tok::LBrace) | Some(Tok::LBrack) | Some(Tok::LParen) => {
            let open = p.peek().cloned();
            let (open_t, close_t) = match open {
                Some(Tok::LBrace) => (Tok::LBrace, Tok::RBrace),
                Some(Tok::LBrack) => (Tok::LBrack, Tok::RBrack),
                Some(Tok::LParen) => (Tok::LParen, Tok::RParen),
                _ => return,
            };
            p.bump();
            let mut depth = 1i32;
            while !p.at_end() && depth > 0 {
                match p.bump() {
                    Some(t) if t == open_t => depth += 1,
                    Some(t) if t == close_t => depth -= 1,
                    Some(Tok::LBrace) | Some(Tok::LBrack) | Some(Tok::LParen) => depth += 1,
                    Some(Tok::RBrace) | Some(Tok::RBrack) | Some(Tok::RParen) => depth -= 1,
                    _ => {}
                }
            }
            let _ = diags;
        }
        Some(Tok::Word(_)) => {
            // chain or bare word
            let _ = parse_channel_chain(p, diags);
        }
        _ => {
            p.force_progress();
        }
    }
}

/// Channel object — alias retained for call sites that only want channels
/// (when bodies). Routes through the style object (axes refused).
#[allow(dead_code)]
fn parse_channel_object(p: &mut Parser<'_>, diags: &mut Vec<StyleDiagnostic>) -> Style {
    parse_style_object(p, diags, /* allow_axes */ false)
}

/// Method chain without `style.` head: `pad(4).gap(2).w(full)`.
fn parse_channel_chain(p: &mut Parser<'_>, diags: &mut Vec<StyleDiagnostic>) -> Style {
    // Empty body (trailing comma / omitted value) — leave Empty.
    if matches!(
        p.peek(),
        Some(Tok::Comma) | Some(Tok::RBrace) | Some(Tok::RParen) | None
    ) {
        return Style::Empty;
    }
    let mut ops = Vec::new();
    let mut iters = 0usize;
    let mut first = true;
    let mut prev_setter: Option<String> = None;
    loop {
        iters += 1;
        if iters > MAX_PARSE_ITERS {
            diags.push(StyleDiagnostic::ParseError {
                message: "channel chain too many setters".into(),
                source: p.remainder(),
            });
            break;
        }
        if !first && !p.eat(&Tok::Dot) {
            break;
        }
        first = false;
        let before = p.pos;
        let method = match p.bump() {
            Some(Tok::Word(w)) => w,
            _ => {
                diags.push(StyleDiagnostic::ParseError {
                    message: "expected channel setter name".into(),
                    source: p.remainder(),
                });
                break;
            }
        };
        if teach_html_attr_eq(p, method, diags) {
            if p.pos == before {
                p.force_progress();
            }
            continue;
        }
        // Member access at the use site does not resolve (`size(control-md).h`).
        if p.peek() != Some(&Tok::LParen) {
            if let Some(prev) = prev_setter.as_deref() {
                if crate::concerns::find_concern(prev).is_some() {
                    diags.push(StyleDiagnostic::ParseError {
                        message: format!(
                            "member access `{prev}.{method}` does not resolve at the use site — `{prev}(…)` is indivisible"
                        ),
                        source: format!("{prev}(…).{method}"),
                    });
                    break;
                }
            }
        }
        if method == "when" {
            diags.push(StyleDiagnostic::ParseError {
                message: "`.when` belongs on the `style({…})` head, not inside a channel chain"
                    .into(),
                source: p.remainder(),
            });
            break;
        }
        if method == "box" {
            let args = parse_paren_args(p, diags, method);
            let arg_text = format_arg_text(&args);
            diags.push(StyleDiagnostic::ParseError {
                message: "`box(…)` as a setter was renamed to `boxSizing(…)` — geometry is `size` / `pad` / `position`"
                    .into(),
                source: format!("box({arg_text})"),
            });
            if p.pos == before {
                p.force_progress();
            }
            continue;
        }
        let motion_object = if is_motion_chain_method(method) {
            if p.peek() == Some(&Tok::LParen) {
                let after_paren = p.tokens.get(p.pos + 1).map(|t| &t.tok);
                after_paren == Some(&Tok::LBrace) || !matches!(method, "scroll" | "press")
            } else {
                true
            }
        } else {
            false
        };
        if motion_object {
            diags.push(StyleDiagnostic::ParseError {
                message: "motion methods belong in `animate: { enter: play(…) }` — not in the concern chain".into(),
                source: method.to_string(),
            });
            let _ = parse_motion_call(p, method, diags);
            continue;
        }
        let args = parse_paren_args(p, diags, method);
        if p.pos == before {
            p.force_progress();
            continue;
        }
        let arg_text = format_arg_text(&args);
        let source = format!("{method}({arg_text})");
        prev_setter = Some(method.to_string());
        push_paint_op(&mut ops, method, args, source, diags);
    }
    if ops.is_empty() {
        Style::Empty
    } else {
        Style::chain(ops)
    }
}

/// `animate:` body — product form is the trigger map only:
/// `{ enter: play(fade), transition: play(colors, fast) }`.
///
/// The retired trigger-call chain (`enter({…}).exit({…})`) is refused with a
/// teaching diagnostic and consumed so the rest of the style object can parse.
fn parse_animate_channel_chain(p: &mut Parser<'_>, diags: &mut Vec<StyleDiagnostic>) -> Style {
    if matches!(
        p.peek(),
        Some(Tok::Comma) | Some(Tok::RBrace) | Some(Tok::RParen) | None
    ) {
        return Style::Empty;
    }
    if p.peek() == Some(&Tok::LBrace) {
        return parse_animate_trigger_map(p, diags);
    }
    diags.push(StyleDiagnostic::ParseError {
        message: "`animate:` is a keyed map of `play(…)` — write \
                  `animate: { enter: play(…) }` (keys: transition, enter, exit, reveal, loop, scroll)"
            .into(),
        source: p.remainder(),
    });
    recover_retired_animate_chain(p, diags);
    Style::Empty
}

/// Consume a retired `enter({…}).exit({…})` chain without producing IR.
fn recover_retired_animate_chain(p: &mut Parser<'_>, diags: &mut Vec<StyleDiagnostic>) {
    let mut iters = 0usize;
    let mut first = true;
    loop {
        iters += 1;
        if iters > MAX_PARSE_ITERS {
            break;
        }
        if !first && !p.eat(&Tok::Dot) {
            break;
        }
        first = false;
        let before = p.pos;
        let method = match p.bump() {
            Some(Tok::Word(w)) => w,
            _ => break,
        };
        if method == "when" {
            break;
        }
        if p.peek() == Some(&Tok::LParen) {
            let after_paren = p.tokens.get(p.pos + 1).map(|t| &t.tok);
            if after_paren == Some(&Tok::LBrace) {
                let _ = parse_motion_call(p, method, diags);
            } else {
                let _ = parse_paren_args(p, diags, method);
            }
        }
        if p.pos == before {
            p.force_progress();
        }
    }
}

/// `animate: { enter: play(…), exit: play(…), transition: play(…) }`.
///
/// Parses the map so a `play(…)` body does not abort the surrounding style
/// object. Play names resolve through [`ThemeStyleEnv::resolve_play`]; a
/// missing preset is unknown vocabulary.
fn parse_animate_trigger_map(p: &mut Parser<'_>, diags: &mut Vec<StyleDiagnostic>) -> Style {
    p.expect(Tok::LBrace, diags);
    let mut ops = Vec::new();
    let mut iters = 0usize;
    while !p.at_end() && p.peek() != Some(&Tok::RBrace) {
        iters += 1;
        if iters > MAX_PARSE_ITERS {
            diags.push(StyleDiagnostic::ParseError {
                message: "animate map too many triggers".into(),
                source: p.remainder(),
            });
            break;
        }
        let before = p.pos;
        let trigger = match p.bump() {
            Some(Tok::Word(w)) => w.to_string(),
            _ => {
                p.force_progress();
                break;
            }
        };
        p.expect(Tok::Colon, diags);
        if let Some(attach) = parse_animate_map_value(p, &trigger, diags) {
            ops.push(attach);
        }
        p.eat(&Tok::Comma);
        if p.pos <= before {
            p.force_progress();
        }
    }
    p.expect(Tok::RBrace, diags);
    if ops.is_empty() {
        Style::Empty
    } else {
        Style::chain(ops)
    }
}

fn is_animate_map_key(name: &str) -> bool {
    name == "transition" || is_animate_trigger(name)
}

fn parse_animate_map_value(
    p: &mut Parser<'_>,
    trigger: &str,
    diags: &mut Vec<StyleDiagnostic>,
) -> Option<Style> {
    if !is_animate_map_key(trigger) {
        diags.push(StyleDiagnostic::ParseError {
            message: format!(
                "`{trigger}` is not an `animate:` key — write `animate: {{ enter: play(…) }}` \
                 (keys: transition, enter, exit, reveal, loop, scroll)"
            ),
            source: trigger.to_string(),
        });
        skip_style_object_value(p, diags);
        return None;
    }
    match p.peek() {
        Some(Tok::Word("play")) => {
            p.bump();
            let args = parse_paren_args(p, diags, "play");
            resolve_play_args(&args, diags);
            Some(Style::MotionAttach {
                trigger: trigger.to_string(),
                config: expand_play_to_config(trigger, &args),
            })
        }
        Some(Tok::Word(w)) if is_motion_chain_method(w) => {
            let method = match p.bump() {
                Some(Tok::Word(w)) => w,
                _ => return None,
            };
            diags.push(StyleDiagnostic::ParseError {
                message: format!(
                    "`animate: {{ {trigger}: {method}({{…}}) }}` is retired — the concern \
                     inside is `play`: `animate: {{ {trigger}: play(…) }}`"
                ),
                source: method.to_string(),
            });
            let _ = parse_motion_call(p, method, diags);
            None
        }
        Some(Tok::LBrace) => {
            diags.push(StyleDiagnostic::ParseError {
                message: format!(
                    "`animate: {{ {trigger}: {{…}} }}` is retired — write \
                     `animate: {{ {trigger}: play(…) }}`"
                ),
                source: trigger.to_string(),
            });
            skip_style_object_value(p, diags);
            None
        }
        _ => {
            // Do not block the rest of the style object.
            skip_style_object_value(p, diags);
            None
        }
    }
}

/// Refuse `.enter({…})` / `.when(…)` tails on a Style const ref. Recovery
/// consumes the call so the surrounding expression can continue.
fn refuse_style_ref_tails(p: &mut Parser<'_>, diags: &mut Vec<StyleDiagnostic>) {
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
            diags.push(StyleDiagnostic::ParseError {
                message: "`.when(…)` is retired — use `style(ref, { when: {…} })` or `when:` inside a style object".into(),
                source: ".when".into(),
            });
            let _ = parse_when_args(p, diags);
        } else if is_motion_chain_method(method) {
            diags.push(StyleDiagnostic::ParseError {
                message: format!(
                    "`.{method}({{…}})` tails on a Style ref are retired — write \
                     `style(name, {{ animate: {{ {method}: play(…) }} }})`"
                ),
                source: format!(".{method}"),
            });
            let _ = parse_motion_call(p, method, diags);
        } else {
            diags.push(StyleDiagnostic::ParseError {
                message: "nothing may follow a Style ref — compose with `style(name, { … })`"
                    .into(),
                source: p.remainder(),
            });
            break;
        }
    }
}

/// Diagnose vocabulary-as-ref at parse. Unknown names stay `Style::Ref` —
/// locals are not in scope here; [`validate_style_refs`] / emit spend
/// [`ThemeStyleEnv::resolve_style_ref`] once an env is available.
fn make_style_ref(name: String, diags: &mut Vec<StyleDiagnostic>) -> Style {
    if matches!(
        ThemeStyleEnv::from_registry().resolve_vocab(&name),
        Some(crate::VocabTag::Concern | crate::VocabTag::Axis)
    ) {
        diags.push(StyleDiagnostic::NotAStyleRef {
            name: name.clone(),
            source: name.clone(),
        });
    }
    Style::Ref(name)
}

fn play_name_from_args(args: &[PaintArg]) -> Option<&str> {
    match args.first() {
        Some(PaintArg::Token(name)) if !name.is_empty() => Some(name.as_str()),
        Some(PaintArg::Slots(slots)) => match slots.get("preset").or_else(|| slots.get("play")) {
            Some(PaintArg::Token(name)) if !name.is_empty() => Some(name.as_str()),
            _ => None,
        },
        _ => None,
    }
}

fn resolve_play_args(args: &[PaintArg], diags: &mut Vec<StyleDiagnostic>) {
    let Some(name) = play_name_from_args(args) else {
        diags.push(StyleDiagnostic::UnknownVocabulary {
            name: "play".into(),
            source: format!("play({})", format_arg_text(args)),
        });
        return;
    };
    if ThemeStyleEnv::from_registry().resolve_play(name).is_none() {
        diags.push(StyleDiagnostic::UnknownVocabulary {
            name: name.to_string(),
            source: format!("play({name})"),
        });
    }
}

/// Resolve every `Style::Ref` through [`ThemeStyleEnv::resolve_style_ref`].
///
/// `env` is the file-local Style IR map. Open spends locals after theme;
/// Themed refuses locals (`ThemedLocalStyle`).
pub fn validate_style_refs(style: &Style, env: &HashMap<String, Style>) -> Vec<StyleDiagnostic> {
    let mut diags = Vec::new();
    validate_style_refs_inner(
        style,
        env,
        &ThemeStyleEnv::from_registry(),
        current_styling_policy(),
        &mut HashSet::new(),
        &mut diags,
    );
    diags
}

fn validate_style_refs_inner(
    style: &Style,
    env: &HashMap<String, Style>,
    theme: &ThemeStyleEnv,
    policy: super::theme_env::StylingPolicy,
    resolving: &mut HashSet<String>,
    diags: &mut Vec<StyleDiagnostic>,
) {
    match style {
        Style::Empty | Style::Set(_) | Style::MotionAttach { .. } => {}
        Style::Ref(name) => {
            if !resolving.insert(name.clone()) {
                return;
            }
            match theme.expand_style_ref(name, env, policy) {
                Ok(expanded) => {
                    validate_style_refs_inner(&expanded, env, theme, policy, resolving, diags)
                }
                Err(d) => diags.push(d),
            }
            resolving.remove(name);
        }
        Style::Chain(items) | Style::List(items) => {
            for i in items {
                validate_style_refs_inner(i, env, theme, policy, resolving, diags);
            }
        }
        Style::Match { arms, .. } => {
            for (_, body) in arms {
                validate_style_refs_inner(body, env, theme, policy, resolving, diags);
            }
        }
        Style::Compound { body, .. } => {
            validate_style_refs_inner(body, env, theme, policy, resolving, diags)
        }
        Style::When { entries } => {
            for (_, body) in entries {
                validate_style_refs_inner(body, env, theme, policy, resolving, diags);
            }
        }
    }
}

fn format_play_config(args: &[PaintArg]) -> String {
    if args.len() == 1 {
        match &args[0] {
            PaintArg::Slots(_) => args[0].display(),
            other => format!("{{ preset: {} }}", other.display()),
        }
    } else if args.len() >= 2 {
        format!(
            "{{ preset: {}, duration: {} }}",
            args[0].display(),
            args[1].display()
        )
    } else {
        "{}".into()
    }
}

/// Lower `play(…)` to the MotionAttach config the compiler already consumes.
///
/// - `transition` / CSS play prop-sets stay `{ preset, duration? }` for emit
/// - keyframe plays expand to `{ opacity: { from, to }, … }` so `_sail_m_*`
///   attach is derived from the same `play` path, not a second author object
fn expand_play_to_config(trigger: &str, args: &[PaintArg]) -> String {
    if trigger == "transition" {
        return format_play_config(args);
    }
    let Some(name) = play_name_from_args(args) else {
        return format_play_config(args);
    };
    let Some(play) = ThemeStyleEnv::from_registry().resolve_play(name) else {
        return format_play_config(args);
    };
    if play_is_css_transition(&play) {
        return format_play_config(args);
    }
    expand_keyframe_play(&play, trigger, args.get(1)).unwrap_or_else(|| format_play_config(args))
}

fn play_is_css_transition(play: &PlayPreset) -> bool {
    play.r#type.contains("play")
        || play.r#type.contains("transition")
        || play.value.contains_key("property")
        || play.value.contains_key("props")
}

fn expand_keyframe_play(
    play: &PlayPreset,
    trigger: &str,
    duration: Option<&PaintArg>,
) -> Option<String> {
    use serde_json::Value;
    let from = play.value.get("from")?.as_object()?;
    let to = play.value.get("to")?.as_object()?;
    let mut parts: Vec<String> = Vec::new();
    if from.contains_key("opacity") || to.contains_key("opacity") {
        if let (Some(f), Some(t)) = (
            json_num_or_str(from.get("opacity")),
            json_num_or_str(to.get("opacity")),
        ) {
            parts.push(format!("opacity: {{ from: {f}, to: {t} }}"));
        }
    }
    let from_tf = from.get("transform").and_then(Value::as_str).unwrap_or("");
    let to_tf = to.get("transform").and_then(Value::as_str).unwrap_or("");
    if !from_tf.is_empty() || !to_tf.is_empty() {
        for (prop, fn_name) in [
            ("scale", "scale"),
            ("translate-y", "translateY"),
            ("translate-x", "translateX"),
            ("rotate", "rotate"),
        ] {
            let f = css_fn_arg(from_tf, fn_name);
            let t = css_fn_arg(to_tf, fn_name);
            if f.is_some() || t.is_some() {
                let f = f.unwrap_or_else(|| "0".into());
                let t = t.unwrap_or_else(|| "0".into());
                parts.push(format!("{prop}: {{ from: {f}, to: {t} }}"));
            }
        }
    }
    if parts.is_empty() {
        return None;
    }
    if trigger == "loop" {
        parts.push("iterations: 'infinite'".into());
    }
    if let Some(PaintArg::Token(tok)) = duration {
        if !tok.is_empty() && tok.chars().all(|c| c.is_ascii_digit() || c == '.') {
            parts.push(format!("duration: {tok}"));
        }
    }
    Some(format!("{{ {} }}", parts.join(", ")))
}

fn json_num_or_str(v: Option<&serde_json::Value>) -> Option<String> {
    match v {
        Some(serde_json::Value::Number(n)) => Some(n.to_string()),
        Some(serde_json::Value::String(s)) => Some(s.clone()),
        _ => None,
    }
}

fn css_fn_arg(transform: &str, name: &str) -> Option<String> {
    let needle = format!("{name}(");
    let i = transform.find(&needle)?;
    let rest = &transform[i + needle.len()..];
    let end = rest.find(')')?;
    let raw = rest[..end].trim();
    if raw.ends_with('%') {
        Some(format!("'{raw}'"))
    } else if let Some(stripped) = raw.strip_suffix("px").or_else(|| raw.strip_suffix("deg")) {
        Some(stripped.to_string())
    } else {
        Some(raw.to_string())
    }
}

fn parse_paren_args(
    p: &mut Parser<'_>,
    diags: &mut Vec<StyleDiagnostic>,
    setter: &str,
) -> Vec<PaintArg> {
    p.expect(Tok::LParen, diags);
    let mut args = Vec::new();
    if p.peek() == Some(&Tok::RParen) {
        args.push(PaintArg::Token(String::new()));
    } else {
        args.push(parse_paint_arg(p, diags, setter));
        while p.eat(&Tok::Comma) {
            args.push(parse_paint_arg(p, diags, setter));
        }
    }
    p.expect(Tok::RParen, diags);
    args
}

fn format_arg_text(args: &[PaintArg]) -> String {
    args.iter()
        .map(PaintArg::display)
        .collect::<Vec<_>>()
        .join(", ")
}

fn push_paint_op(
    ops: &mut Vec<Style>,
    method: &str,
    args: Vec<PaintArg>,
    source: String,
    diags: &mut Vec<StyleDiagnostic>,
) {
    if crate::vocab::find_vocab(method).is_none()
        && crate::foreign_idioms::find_foreign_idiom(method).is_some()
    {
        diags.push(StyleDiagnostic::UnknownVocabulary {
            name: method.to_string(),
            source: source.clone(),
        });
        return;
    }
    if let Some(d) = crate::concerns::retired_diag(method, &source) {
        diags.push(d);
        return;
    }
    let (method, args) = crate::concerns::prepare_concern_call(method, args);
    if crate::vocab::find_vocab(&method).is_none() {
        diags.push(StyleDiagnostic::UnknownVocabulary {
            name: method.to_string(),
            source: source.clone(),
        });
    } else {
        let op = PaintOp {
            setter: method.clone(),
            args: args.clone(),
            source: source.clone(),
        };
        if let Err(d) = validate_op(&op) {
            diags.push(d);
        }
    }
    ops.push(Style::Set(PaintOp {
        setter: method,
        args,
        source,
    }));
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

fn parse_paint_arg(p: &mut Parser<'_>, diags: &mut Vec<StyleDiagnostic>, setter: &str) -> PaintArg {
    if p.peek() == Some(&Tok::LBrace) {
        return parse_slot_object(p, diags, setter);
    }
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
        // Member path: tone / tone.foreground / primary.foreground (root + optional .segments).
        Some(Tok::Word(w)) => {
            let root = w.to_string();
            let mut path = Vec::new();
            while p.peek() == Some(&Tok::Dot) {
                p.bump();
                match p.bump() {
                    Some(Tok::Word(seg)) => path.push(seg.to_string()),
                    _ => {
                        diags.push(StyleDiagnostic::ParseError {
                            message: "member path expects an identifier after `.`".into(),
                            source: p.remainder(),
                        });
                        break;
                    }
                }
            }
            normalize_paint_arg(root, path, p.remainder(), diags)
        }
        _ => {
            diags.push(StyleDiagnostic::ParseError {
                message: "expected token, member path, bind(name), or `{ slot: … }`".into(),
                source: p.remainder(),
            });
            PaintArg::Token(String::new())
        }
    }
}

/// Concern slot object `{ color: muted, width: hairline }`.
fn parse_slot_object(
    p: &mut Parser<'_>,
    diags: &mut Vec<StyleDiagnostic>,
    setter: &str,
) -> PaintArg {
    p.expect(Tok::LBrace, diags);
    let mut slots = BTreeMap::new();
    let mut iters = 0usize;
    while !p.at_end() && p.peek() != Some(&Tok::RBrace) {
        iters += 1;
        if iters > 32 {
            diags.push(StyleDiagnostic::ParseError {
                message: "concern slot object too many entries".into(),
                source: p.remainder(),
            });
            break;
        }
        let before = p.pos;
        let key = match p.bump() {
            Some(Tok::Word(w)) => unquote(w).to_string(),
            _ => {
                diags.push(StyleDiagnostic::ParseError {
                    message: "expected concern slot name".into(),
                    source: p.remainder(),
                });
                p.force_progress();
                break;
            }
        };
        p.expect(Tok::Colon, diags);
        let value = parse_paint_arg(p, diags, setter);
        match slots.entry(key.clone()) {
            std::collections::btree_map::Entry::Occupied(_) => {
                diags.push(StyleDiagnostic::DuplicateSlot {
                    setter: setter.to_string(),
                    slot: key.clone(),
                    source: format!("{key}: {}", value.display()),
                });
            }
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(value);
            }
        }
        p.eat(&Tok::Comma);
        if p.pos <= before {
            p.force_progress();
        }
    }
    p.expect(Tok::RBrace, diags);
    PaintArg::Slots(slots)
}

/// Resolve paint arg identity:
/// - bound group prop (`tone`, `tone.foreground`, `shade` with Tone type) → MemberPath
/// - L1 host surface (`chip`, `chip.foreground`) → MemberPath (host context only)
/// - static group path (`primary.foreground`) → Token(public name)
/// - bare non-group word → Token (global / scale name)
fn normalize_paint_arg(
    root: String,
    path: Vec<String>,
    source: String,
    diags: &mut Vec<StyleDiagnostic>,
) -> PaintArg {
    use super::component_surfaces::{current_surface_pack, resolve_surface_author};
    use super::token_groups::{
        bound_member_valid, looks_like_bound_group_root, resolve_static_member_path,
        static_token_group,
    };

    // L1 host surface graph — identity stays MemberPath; emit maps to public name.
    if current_surface_pack() == Some(root.as_str()) {
        if let Err(msg) = resolve_surface_author(&root, &path) {
            diags.push(StyleDiagnostic::BadToken {
                setter: String::new(),
                value: if path.is_empty() {
                    root.clone()
                } else {
                    format!("{root}.{}", path.join("."))
                },
                source: source.clone(),
            });
            let _ = msg;
        }
        return PaintArg::MemberPath { root, path };
    }

    // Static group stem first (`primary`, `primary.foreground`) → public Token.
    if static_token_group(&root).is_some() {
        if path.is_empty() {
            // Bare stem is also a color token — Token path below.
        } else {
            match resolve_static_member_path(&root, &path) {
                Ok(public) => return PaintArg::Token(public),
                Err(_) => {
                    let value = format!("{root}.{}", path.join("."));
                    diags.push(StyleDiagnostic::BadToken {
                        setter: String::new(),
                        value: value.clone(),
                        source: source.clone(),
                    });
                    return PaintArg::Token(value);
                }
            }
        }
    }

    // Bound group-typed prop path: bridge-var emit (`var(--tone)`). Type check
    // (color-group typed prop) is the compiler binding regime — parse keeps shape.
    if looks_like_bound_group_root(&root, &path) {
        if let Err(_msg) = bound_member_valid(&path) {
            // Non-empty invalid member (e.g. tone.hover when not universal).
            if !path.is_empty() {
                diags.push(StyleDiagnostic::BadToken {
                    setter: String::new(),
                    value: format!("{root}.{}", path.join(".")),
                    source: source.clone(),
                });
            }
        }
        return PaintArg::MemberPath { root, path };
    }

    // Bare tokens. Non-color scales may still use short names that collide
    // with emit labels (e.g. `z(popover)` → z-popover). No early reject here.
    if path.is_empty() {
        return PaintArg::Token(root);
    }

    let value = format!("{root}.{}", path.join("."));
    diags.push(StyleDiagnostic::BadToken {
        setter: String::new(),
        value: value.clone(),
        source,
    });
    PaintArg::Token(value)
}

/// Dual-accept when args:
/// - map form: `({ hover: bg(x), … })`
/// - cond form: `(hover, { bg(x) })` or `(hover, bg(x))`
fn parse_when_args(p: &mut Parser<'_>, diags: &mut Vec<StyleDiagnostic>) -> Vec<(String, Style)> {
    p.expect(Tok::LParen, diags);
    if p.peek() == Some(&Tok::LBrace) {
        let entries = parse_when_map_entries(p, diags);
        p.expect(Tok::RParen, diags);
        return entries;
    }
    let key = match p.bump() {
        Some(Tok::Word(w)) => unquote(w).to_string(),
        _ => {
            diags.push(StyleDiagnostic::ParseError {
                message: "expected when condition key or `{…}` map".into(),
                source: p.remainder(),
            });
            p.expect(Tok::RParen, diags);
            return Vec::new();
        }
    };
    if !is_known_when_key(&key) {
        diags.push(StyleDiagnostic::UnknownWhenKey {
            key: key.clone(),
            source: key.clone(),
        });
    }
    p.expect(Tok::Comma, diags);
    let body = parse_when_partial(p, diags);
    p.expect(Tok::RParen, diags);
    vec![(key, body)]
}

fn parse_when_map_entries(
    p: &mut Parser<'_>,
    diags: &mut Vec<StyleDiagnostic>,
) -> Vec<(String, Style)> {
    p.expect(Tok::LBrace, diags);
    let mut entries: Vec<(String, Style)> = Vec::new();
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
        if !is_known_when_key(&key) {
            diags.push(StyleDiagnostic::UnknownWhenKey {
                key: key.clone(),
                source: key.clone(),
            });
        }
        p.expect(Tok::Colon, diags);
        let body = parse_partial(p, diags, /* allow_axes */ false);
        entries.push((key, body));
        p.eat(&Tok::Comma);
        if p.pos <= before {
            p.force_progress();
        }
    }
    p.expect(Tok::RBrace, diags);
    entries
}

/// Partial body of cond-form when: channel object, full Style expr, or bare chain.
fn parse_when_partial(p: &mut Parser<'_>, diags: &mut Vec<StyleDiagnostic>) -> Style {
    // Cond-form when body — only used while recovering retired `.when(key, body)` tails.
    match p.peek() {
        Some(Tok::LBrace) => parse_style_object(p, diags, /* allow_axes */ false),
        Some(Tok::Word("style")) => parse_style_chain(p, diags),
        Some(Tok::Word(_)) => {
            let next = p.tokens.get(p.pos + 1).map(|t| &t.tok);
            if next == Some(&Tok::LParen) {
                parse_channel_chain(p, diags)
            } else {
                parse_partial(p, diags, false)
            }
        }
        _ => {
            diags.push(StyleDiagnostic::ParseError {
                message: "expected when partial (`{ box|paint|text|animate: … }` or name)".into(),
                source: p.remainder(),
            });
            Style::Empty
        }
    }
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

/// Parse a scope Style declaration RHS (`const name = <rhs>` or `style name = <rhs>`).
///
/// Accepts (SPEC-STYLE-DECL-1):
/// 1. **Object form** — bare `{ pad(4).bg(surface), when: … }` (no outer `style(`)
/// 2. **Call form** — `style({ … })` / `style(base, { … })`
/// 3. **Compose** — `name, { … }` / `name, style({ … })` (same merge as multi-arg `style`)
/// 4. **Bare ref** — another Style const name
pub fn parse_style_const_rhs(input: &str) -> (Style, Vec<StyleDiagnostic>) {
    parse_const_rhs(input)
}

/// Like [`parse_with`], but the body is one or more style *args* (compose / bare
/// object) rather than a single top-level `style(…)` / ref expression only.
fn parse_const_rhs(input: &str) -> (Style, Vec<StyleDiagnostic>) {
    let mut diags = Vec::new();
    let stripped = strip_comments(input);
    let trimmed = stripped.trim();
    if trimmed.is_empty() {
        return (Style::Empty, diags);
    }
    if looks_like_utility_string_product(trimmed) {
        diags.push(StyleDiagnostic::ResidualProduct {
            message: residual_product_message(trimmed),
            source: trimmed.chars().take(80).collect(),
        });
        return (Style::Empty, diags);
    }
    let tokens = tokenize(trimmed);
    let mut p = Parser::new_full(trimmed, &tokens, false);
    // Declaration RHS = style-arg composition (same merge law as `style(a, b, …)`).
    let mut parts = Vec::new();
    let mut iters = 0usize;
    loop {
        iters += 1;
        if iters > MAX_PARSE_ITERS {
            diags.push(StyleDiagnostic::ParseError {
                message: "style declaration RHS too many compose parts".into(),
                source: p.remainder(),
            });
            break;
        }
        parts.push(parse_style_arg(&mut p, &mut diags));
        if !p.eat(&Tok::Comma) {
            break;
        }
        if p.at_end() {
            break; // trailing comma
        }
    }
    let style = match parts.len() {
        0 => Style::Empty,
        1 => parts.swap_remove(0),
        _ => Style::List(parts),
    };
    if !p.at_end() {
        diags.push(StyleDiagnostic::ParseError {
            message: format!("unexpected token `{}`", p.peek_str()),
            source: p.remainder(),
        });
    }
    validate_style(&style, &mut diags);
    if let Err(d) = forbid_motion_inside_when(&style) {
        diags.push(d);
    }
    if let Err(d) = forbid_geometry_inside_interaction_when(&style) {
        diags.push(d);
    }
    (style, diags)
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

#[cfg(test)]
mod const_rhs_tests {
    use super::{Style, parse_style_const_rhs};

    #[test]
    fn bare_object_form() {
        let (style, diags) = parse_style_const_rhs("{ bg(primary) }");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        assert!(!matches!(style, Style::Empty));
    }

    #[test]
    fn call_form() {
        let (style, diags) = parse_style_const_rhs("style({ pad(2) })");
        assert!(diags.is_empty(), "{diags:?}");
        assert!(!matches!(style, Style::Empty));
    }

    #[test]
    fn compose_form() {
        let (style, diags) = parse_style_const_rhs("host, { pad(2) }");
        assert!(diags.is_empty(), "{diags:?}");
        match style {
            Style::List(parts) => assert_eq!(parts.len(), 2),
            other => panic!("expected List, got {other:?}"),
        }
    }
}

#[cfg(test)]
mod flat_chain_tests {
    use std::collections::HashMap;

    use super::{Style, parse};

    fn msgs(diags: &[crate::StyleDiagnostic]) -> Vec<String> {
        diags.iter().map(|d| d.message()).collect()
    }

    #[test]
    fn v2_author_names_are_unknown_vocabulary() {
        for src in [
            "style({ fill(surface) })",
            "style({ fill({ color: muted }) })",
            "style({ ink(foreground) })",
            "style({ place({ z: sticky }) })",
            "style({ space({ pad: 2 }) })",
            "style({ flow({ display: flex }) })",
            "style({ motion({ rotate: 90 }) })",
        ] {
            let (_, diags) = parse(src);
            assert!(
                diags
                    .iter()
                    .any(|d| { matches!(d, crate::StyleDiagnostic::UnknownVocabulary { .. }) }),
                "{src} must be unknown vocabulary: {:?}",
                msgs(&diags)
            );
        }
    }

    #[test]
    fn dissolved_channel_keys_are_not_style_members() {
        for key in ["box", "paint", "text"] {
            let src = format!("style({{ {key}: bg(muted) }})");
            let (_, diags) = parse(&src);
            assert!(
                diags
                    .iter()
                    .any(|d| d.message().contains("not a style member")),
                "{key}: {:?}",
                msgs(&diags)
            );
        }
    }

    #[test]
    fn compound_row_accepts_concern_call_tail() {
        let src =
            "style({ compound: [{ tone: muted, variant: solid, bg(muted).fg(foreground) }] })";
        let (style, diags) = parse(src);
        assert!(diags.is_empty(), "{:?}", msgs(&diags));
        assert!(!matches!(style, Style::Empty));
    }

    #[test]
    fn play_fade_resolves_and_attaches() {
        let (style, diags) = parse("style({ animate: { enter: play(fade) } })");
        assert!(
            !diags
                .iter()
                .any(|d| matches!(d, crate::StyleDiagnostic::UnknownVocabulary { .. })),
            "fade is a registry keyframe: {:?}",
            msgs(&diags)
        );
        let m = crate::parse::collect_motion_attaches(&style);
        assert_eq!(m.len(), 1, "{m:?}");
        assert_eq!(m[0].0, "enter");
        assert!(
            m[0].1.contains("opacity") && m[0].1.contains("from") && m[0].1.contains("to"),
            "play(fade) must expand to the motion config the compiler consumes: {:?}",
            m[0]
        );
    }

    #[test]
    fn animate_trigger_call_form_is_refused() {
        let (_, diags) = parse("style({ animate: enter({ opacity: 'fade' }) })");
        let joined = msgs(&diags).join("; ");
        assert!(
            joined.contains("animate: { enter: play(") || joined.contains("keyed map of `play"),
            "must teach the play map: {joined}"
        );
        assert!(
            crate::parse::collect_motion_attaches(
                &parse("style({ animate: enter({ opacity: 'fade' }) })").0
            )
            .is_empty()
                || diags.iter().any(|d| d.message().contains("play")),
            "retired trigger-call must not be a silent product: {joined}"
        );
    }

    #[test]
    fn style_ref_enter_tail_is_refused() {
        let (_, diags) = parse("row.enter({ opacity: 'fade' })");
        let joined = msgs(&diags).join("; ");
        assert!(
            joined.contains("animate:") && joined.contains("play"),
            "must teach the play map: {joined}"
        );
    }

    #[test]
    fn unknown_play_name_is_unknown_vocabulary() {
        let (_, diags) = parse("style({ animate: { enter: play(notARealPlayZzz) } })");
        assert!(
            diags.iter().any(|d| matches!(
                d,
                crate::StyleDiagnostic::UnknownVocabulary { name, .. } if name == "notARealPlayZzz"
            )),
            "unknown play must fail closed: {:?}",
            msgs(&diags)
        );
    }

    #[test]
    fn vocab_name_is_not_a_style_ref() {
        for vocab in ["border", "pad", "bg", "type"] {
            let (_, diags) = parse(&format!("style({vocab})"));
            assert!(
                diags.iter().any(|d| matches!(
                    d,
                    crate::StyleDiagnostic::NotAStyleRef { name, .. } if name == vocab
                )),
                "{vocab}: {:?}",
                msgs(&diags)
            );
        }
    }

    #[test]
    fn inline_when_arg_stays_illegal() {
        let (_, diags) = parse("style(pad({ t: 2 }), when: { hover: bg(muted) })");
        assert!(
            !diags.is_empty(),
            "inline when: as a style() arg must stay illegal"
        );
        let joined = msgs(&diags).join("; ");
        assert!(
            joined.contains("declared") || joined.contains("when:"),
            "must teach declared form: {joined}"
        );
    }

    #[test]
    fn open_local_ref_validates() {
        let mut env = HashMap::new();
        env.insert("foo".into(), parse("style({ bg(muted) })").0);
        let (style, diags) = parse("foo");
        assert!(diags.is_empty(), "{:?}", msgs(&diags));
        let refs = crate::parse::validate_style_refs(&style, &env);
        assert!(refs.is_empty(), "{refs:?}");
    }

    #[test]
    fn themed_local_ref_is_studio() {
        let mut env = HashMap::new();
        env.insert("foo".into(), parse("style({ bg(muted) })").0);
        let (style, _) = parse("foo");
        crate::with_styling_policy(crate::StylingPolicy::Themed, || {
            let refs = crate::parse::validate_style_refs(&style, &env);
            assert!(
                refs.iter().any(|d| matches!(
                    d,
                    crate::StyleDiagnostic::ThemedLocalStyle { name, .. } if name == "foo"
                )),
                "{refs:?}"
            );
        });
    }

    #[test]
    fn unknown_ref_with_env_is_unknown_vocabulary() {
        let env = HashMap::new();
        let (style, diags) = parse("notARealStyleZzz");
        assert!(
            diags.is_empty(),
            "parse keeps unknown refs: {:?}",
            msgs(&diags)
        );
        let refs = crate::parse::validate_style_refs(&style, &env);
        assert!(
            refs.iter().any(|d| matches!(
                d,
                crate::StyleDiagnostic::UnknownVocabulary { name, .. } if name == "notARealStyleZzz"
            )),
            "{refs:?}"
        );
    }

    #[test]
    fn padding_points_at_pad() {
        let (_, diags) = parse("style({ padding(4) })");
        let joined = msgs(&diags).join("; ");
        assert!(
            diags
                .iter()
                .any(|d| matches!(d, crate::StyleDiagnostic::UnknownVocabulary { .. })),
            "padding must be unknown vocabulary: {joined}"
        );
        assert!(
            joined.contains("pad"),
            "padding must point at pad: {joined}"
        );
    }

    #[test]
    fn font_size_points_at_type() {
        let (_, diags) = parse("style({ font-size(sm) })");
        let joined = msgs(&diags).join("; ");
        assert!(
            diags
                .iter()
                .any(|d| matches!(d, crate::StyleDiagnostic::UnknownVocabulary { .. })),
            "font-size must be unknown vocabulary: {joined}"
        );
        assert!(
            joined.contains("type"),
            "font-size must point at type: {joined}"
        );
    }

    #[test]
    fn class_eq_teaches_style() {
        let (_, diags) = parse(r#"style({ class="hero" })"#);
        let joined = msgs(&diags).join("; ");
        assert!(
            diags
                .iter()
                .any(|d| matches!(d, crate::StyleDiagnostic::UnknownVocabulary { .. })),
            "class= must be unknown vocabulary: {joined}"
        );
        assert!(
            joined.contains("style("),
            "class= must point at style(…): {joined}"
        );
    }

    #[test]
    fn tailwind_p4_points_at_pad() {
        let (_, diags) = parse("style({ p-4() })");
        let joined = msgs(&diags).join("; ");
        assert!(
            diags
                .iter()
                .any(|d| matches!(d, crate::StyleDiagnostic::UnknownVocabulary { .. })),
            "p-4 must be unknown vocabulary: {joined}"
        );
        assert!(joined.contains("pad"), "p-4 must point at pad: {joined}");
    }

    #[test]
    fn unknown_without_row_stays_generic() {
        let (_, diags) = parse("style({ notARealSetterZzz() })");
        let joined = msgs(&diags).join("; ");
        assert!(
            diags.iter().any(|d| matches!(
                d,
                crate::StyleDiagnostic::UnknownVocabulary { name, .. }
                    if name == "notARealSetterZzz"
            )),
            "made-up name must stay unknown vocabulary: {joined}"
        );
        assert!(
            !joined.contains("CSS") && !joined.contains("Tailwind") && !joined.contains("HTML"),
            "generic unknown must not invent a pointer: {joined}"
        );
    }
}
