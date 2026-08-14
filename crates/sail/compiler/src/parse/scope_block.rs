//! `<scope>` — the component's static surface. A fenced subset of TypeScript:
//! `type` / `export type` aliases, field declarations (`name: Type = value`)
//! with literal values, **Style declarations** (`style name = {…}`), and
//! **static imports** of types
//! + Style values only.
//! Nothing in the block executes — no runtime `const` computation, no
//! behavior imports. If it computes, it belongs in `<controller>`.
//!
//! Members (author grammar — also [`crate::scope_manifest::SCOPE_GRAMMAR`]):
//!   - `type X = …` / `export type X = …` — aliases, threaded into the
//!     compiled module so the controller can reuse them.
//!   - `import type { … } from '…'` / `import { type X } from '…'` — type
//!     imports only; threaded into the compiled module with aliases.
//!   - `import { name } from './peer.sail'` — Style-value (channel) import;
//!     names land in the Style env when the peer's Style consts resolve.
//!   - `props: { … }` — the component's typed inputs (field declarations).
//!     Residual CVA key→sx string maps type as the key union (no paint).
//!     Product paint is host `match` / Style.
//!   - `style name = <Style>` — first-class compile-time Style declaration
//!     (object / call / compose / ref).
//!   - Residual `compound: [ { when, sx } ]` is skipped (no paint).
//!
//! Bare `tone` / `size` defaults type as the shared stdlib unions and reflect
//! as ordinary props.
//!
//! Field grammar (props): `name = literal` (type inferred), `name: Type`
//! (parent must fill), `name: Type = literal`. Unions require an explicit
//! default.

use std::collections::HashMap;

use sail_style::{Style, StyleDiagnostic, StyleEnv};

use crate::diagnostics::{CompileError, Span};
use crate::ir::directive::{PropDecl, PropKind};

use super::preprocess::{parse_object_fields, unquote};

const RULE_ID: &str = "sail-scope-block";
/// Everything a `<scope>` block contributes to the Program.
#[derive(Debug)]
pub(crate) struct ScopeBlock {
    pub props: Vec<PropDecl>,
    /// Raw `type` / `export type` alias text, appended to the controller
    /// module (aliases hoist, so appending preserves controller spans).
    pub types: String,
    /// Scope Style values (`style name = …` → env for product `style(name)` Ref).
    pub style_env: StyleEnv,
}

const TONE_VOCAB: &str = "primary|info|success|warning|destructive|accent|muted";
const SIZE_VOCAB: &str = "xs|sm|md|lg";

const ALLOWED_HINT: &str = "`<scope>` holds `type` declarations, type/Style-value \
     `import`s, `props:`, Style `style name = {…}`, optional residual `compound:` — types, literals, \
     and Style only; logic goes in `<controller>`";

pub(crate) fn parse_scope_block(body: &str, span: Span) -> Result<ScopeBlock, CompileError> {
    parse_scope_block_at(body, span, None)
}

/// Like [`parse_scope_block`], with an optional directory for resolving
/// relative Style-value imports (peer `.sail` Style consts).
pub(crate) fn parse_scope_block_at(
    body: &str,
    span: Span,
    base_dir: Option<&std::path::Path>,
) -> Result<ScopeBlock, CompileError> {
    let bytes = body.as_bytes();
    let mut i = 0usize;
    let mut out = ScopeBlock {
        props: Vec::new(),
        types: String::new(),
        style_env: HashMap::new(),
    };

    while i < bytes.len() {
        i = skip_trivia(body, i, None);
        if i >= bytes.len() {
            break;
        }
        // Statement terminators between members are free.
        if bytes[i] == b';' || bytes[i] == b',' {
            i += 1;
            continue;
        }
        let (word, after) = read_word(body, i);
        match word {
            "export" | "type" => {
                let start = i;
                let end = consume_type_alias(body, if word == "export" { after } else { i })?;
                out.types.push_str(body[start..end].trim());
                out.types.push('\n');
                i = end;
            }
            "import" => {
                let (stmt, end, kind) = parse_scope_import(body, i, span)?;
                match kind {
                    ScopeImportKind::TypeOnly => {
                        out.types.push_str(stmt.trim());
                        out.types.push('\n');
                    }
                    ScopeImportKind::StyleValue { names, module } => {
                        // Keep the import text for TS surface / d.ts.
                        out.types.push_str(stmt.trim());
                        out.types.push('\n');
                        merge_style_value_imports(
                            &mut out.style_env,
                            &names,
                            &module,
                            base_dir,
                            span,
                        )?;
                    }
                }
                i = end;
            }
            "props" => {
                let obj_span = expect_member_object(body, after, "props", span)?;
                let props = parse_props_fields(&body[obj_span.0 + 1..obj_span.1], span)?;
                out.props.extend(props);
                i = obj_span.1 + 1;
            }
            "compound" => {
                // Paint from scope compound arrays is deleted (Style product is
                // host `compound({…}, style.…)`). Skip the array so remaining
                // stdlib files still parse as typed props until migrated.
                let arr_span = expect_member_array(body, after, "compound", span)?;
                i = arr_span.1 + 1;
            }
            "const" => {
                let (name, style, end) = parse_style_const(body, after, span)?;
                if out.style_env.contains_key(&name) {
                    return Err(CompileError::new(format!(
                        "Style `{name}` is declared twice in `<scope>`."
                    ))
                    .with_span(span)
                    .with_category(RULE_ID));
                }
                out.style_env.insert(name, style);
                i = end;
            }
            "style" => {
                // First-class Style decl: `style name = <rhs>` (not tag-position
                // `style(…)` — that lives on hosts, not as a scope statement).
                let (name, style, end) = parse_style_decl(body, after, span)?;
                if out.style_env.contains_key(&name) {
                    return Err(CompileError::new(format!(
                        "Style `{name}` is declared twice in `<scope>`."
                    ))
                    .with_span(span)
                    .with_category(RULE_ID));
                }
                out.style_env.insert(name, style);
                i = end;
            }
            "" => {
                return Err(CompileError::new(format!(
                    "unexpected `{}` in `<scope>` — {ALLOWED_HINT}.",
                    char_at(body, i)
                ))
                .with_span(span)
                .with_category(RULE_ID));
            }
            other => {
                // Name the closest wrong turn we can recognize.
                let hint = match other {
                    "let" | "var" => {
                        " Use `style name = {…}` for Style values, or a prop literal \
                         default — computation belongs in `<controller>`."
                    }
                    _ => "",
                };
                return Err(CompileError::new(format!(
                    "`{other}` is not a `<scope>` member — {ALLOWED_HINT}.{hint}"
                ))
                .with_span(span)
                .with_category(RULE_ID));
            }
        }
    }
    Ok(out)
}

enum ScopeImportKind {
    TypeOnly,
    StyleValue { names: Vec<String>, module: String },
}

/// Parse one `import …` statement in `<scope>`.
///
/// Allowed:
/// - `import type { X } from '…'` / `import type X from '…'`
/// - `import { type X, type Y } from '…'` (all specs type-only)
/// - `import { filled } from './peer.sail'` (Style-value / channel import)
///
/// Refused with teaching: default/namespace/side-effect imports, value
/// imports from packages, mixed type+value without `type`, runtime behavior.
fn parse_scope_import(
    s: &str,
    start: usize,
    span: Span,
) -> Result<(String, usize, ScopeImportKind), CompileError> {
    let bytes = s.as_bytes();
    let mut i = start;
    // consume `import`
    let (word, after) = read_word(s, i);
    if word != "import" {
        return Err(CompileError::new("expected `import`")
            .with_span(span)
            .with_category(RULE_ID));
    }
    i = skip_trivia(s, after, None);
    if i >= bytes.len() {
        return Err(CompileError::new("incomplete `import` in `<scope>`")
            .with_span(span)
            .with_category(RULE_ID));
    }

    // `import type …`
    let (next, after_next) = read_word(s, i);
    let whole_type = next == "type";
    if whole_type {
        i = skip_trivia(s, after_next, None);
    }

    // Side-effect: `import 'mod'`
    if !whole_type && (bytes[i] == b'\'' || bytes[i] == b'"') {
        return Err(CompileError::new(
            "side-effect `import` is not a `<scope>` member — scope may import \
             types (`import type { X } from '…'`) and Style values \
             (`import { filled } from './peer.sail'`) only; runtime behavior \
             belongs in `<controller>`.",
        )
        .with_span(span)
        .with_category(RULE_ID));
    }

    // Default import: `import Foo from` / `import type Foo from`
    if bytes[i] != b'{' && bytes[i] != b'*' {
        let (name, after_name) = read_word(s, i);
        if name.is_empty() {
            return Err(CompileError::new("malformed `import` in `<scope>`")
                .with_span(span)
                .with_category(RULE_ID));
        }
        i = skip_trivia(s, after_name, None);
        // `import type Foo from '…'` is type-only default — allow.
        if whole_type {
            let end = consume_import_tail(s, i, span)?;
            let stmt = s[start..end].trim().to_string();
            return Ok((stmt, end, ScopeImportKind::TypeOnly));
        }
        return Err(CompileError::new(
            "default `import` is not a `<scope>` member — use `import type { X } \
             from '…'` for types, or `import { name } from './peer.sail'` for \
             Style values. Runtime defaults belong in `<controller>`.",
        )
        .with_span(span)
        .with_category(RULE_ID));
    }

    // Namespace: `import * as ns from`
    if bytes[i] == b'*' {
        return Err(CompileError::new(
            "namespace `import * as` is not a `<scope>` member — import named \
             types or Style values only.",
        )
        .with_span(span)
        .with_category(RULE_ID));
    }

    // Named: `import { … } from '…'`
    if bytes[i] != b'{' {
        return Err(CompileError::new("malformed `import` in `<scope>`")
            .with_span(span)
            .with_category(RULE_ID));
    }
    let brace_end = skip_balanced_brace(s, i)?;
    let inner = s[i + 1..brace_end].trim();
    let specs = parse_import_specifiers(inner, span)?;
    i = skip_trivia(s, brace_end + 1, None);

    let end = consume_import_tail(s, i, span)?;
    let module = extract_import_module(s, start, end).unwrap_or_default();
    let stmt = s[start..end].trim().to_string();

    if whole_type || specs.iter().all(|sp| sp.is_type) {
        return Ok((stmt, end, ScopeImportKind::TypeOnly));
    }
    if specs.iter().any(|sp| sp.is_type) {
        return Err(CompileError::new(
            "mixed type/value `import` in `<scope>` — use `import type { … }` \
             for types, or a pure value import for Style consts; do not mix.",
        )
        .with_span(span)
        .with_category(RULE_ID));
    }

    // Value import → Style-value (channel) import only.
    if module_looks_like_package(&module) {
        return Err(CompileError::new(format!(
            "value import from `{module}` is not a `<scope>` member — scope \
             imports types and Style values only (relative `.sail` Style \
             consts). Runtime packages belong in `<controller>`."
        ))
        .with_span(span)
        .with_category(RULE_ID));
    }
    let names: Vec<String> = specs.into_iter().map(|sp| sp.local).collect();
    Ok((stmt, end, ScopeImportKind::StyleValue { names, module }))
}

struct ImportSpec {
    local: String,
    is_type: bool,
}

fn parse_import_specifiers(inner: &str, span: Span) -> Result<Vec<ImportSpec>, CompileError> {
    let mut out = Vec::new();
    for part in split_top_level_commas(inner) {
        let p = part.trim();
        if p.is_empty() {
            continue;
        }
        let mut rest = p;
        let mut is_type = false;
        if let Some(r) = rest.strip_prefix("type") {
            if r.starts_with(|c: char| c.is_whitespace()) || r.is_empty() {
                is_type = true;
                rest = r.trim_start();
            }
        }
        // `X as Y` → local Y; else local X
        let local = if let Some((left, right)) = rest.split_once(" as ") {
            let _ = left;
            right.trim()
        } else {
            rest.trim()
        };
        if local.is_empty() || !is_import_ident(local) {
            return Err(
                CompileError::new(format!("bad import specifier `{p}` in `<scope>`"))
                    .with_span(span)
                    .with_category(RULE_ID),
            );
        }
        out.push(ImportSpec {
            local: local.to_string(),
            is_type,
        });
    }
    if out.is_empty() {
        return Err(CompileError::new("empty `import { }` in `<scope>`")
            .with_span(span)
            .with_category(RULE_ID));
    }
    Ok(out)
}

fn is_import_ident(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' || c == '$' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

fn consume_import_tail(s: &str, mut i: usize, span: Span) -> Result<usize, CompileError> {
    let bytes = s.as_bytes();
    i = skip_trivia(s, i, None);
    let (from_word, after_from) = read_word(s, i);
    if from_word != "from" {
        return Err(CompileError::new("`import` in `<scope>` needs `from '…'`.")
            .with_span(span)
            .with_category(RULE_ID));
    }
    i = skip_trivia(s, after_from, None);
    if i >= bytes.len() || (bytes[i] != b'\'' && bytes[i] != b'"') {
        return Err(
            CompileError::new("`import … from` needs a string module specifier.")
                .with_span(span)
                .with_category(RULE_ID),
        );
    }
    let q = bytes[i];
    i += 1;
    while i < bytes.len() && bytes[i] != q {
        if bytes[i] == b'\\' {
            i += 2;
            continue;
        }
        i += 1;
    }
    if i >= bytes.len() {
        return Err(CompileError::new("unclosed import module string")
            .with_span(span)
            .with_category(RULE_ID));
    }
    i += 1; // closing quote
    i = skip_trivia(s, i, None);
    if i < bytes.len() && bytes[i] == b';' {
        i += 1;
    }
    Ok(i)
}

fn extract_import_module(s: &str, start: usize, end: usize) -> Option<String> {
    let slice = &s[start..end];
    let from = slice.rfind(" from ")?;
    let rest = slice[from + 6..].trim();
    let q = rest.chars().next()?;
    if q != '\'' && q != '"' {
        return None;
    }
    let inner = rest.trim_start_matches(q);
    let end_q = inner.find(q)?;
    Some(inner[..end_q].to_string())
}

fn module_looks_like_package(module: &str) -> bool {
    let m = module.trim();
    if m.is_empty() {
        return true;
    }
    // Relative / absolute / URL → not a package.
    if m.starts_with('.') || m.starts_with('/') || m.starts_with("file:") {
        return false;
    }
    true
}

fn skip_balanced_brace(s: &str, open: usize) -> Result<usize, CompileError> {
    let bytes = s.as_bytes();
    if bytes.get(open) != Some(&b'{') {
        return Err(CompileError::new("expected `{`").with_category(RULE_ID));
    }
    let mut i = open;
    let mut depth = 0i32;
    while i < bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Ok(i);
                }
            }
            b'\'' | b'"' | b'`' => {
                let q = bytes[i];
                i += 1;
                while i < bytes.len() && bytes[i] != q {
                    if bytes[i] == b'\\' {
                        i += 2;
                        continue;
                    }
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    Err(CompileError::new("unclosed `{` in import").with_category(RULE_ID))
}

fn merge_style_value_imports(
    env: &mut StyleEnv,
    names: &[String],
    module: &str,
    base_dir: Option<&std::path::Path>,
    span: Span,
) -> Result<(), CompileError> {
    // Resolve peer `.sail` Style consts into this env. Without a resolvable
    // peer, register empty Style placeholders so host `style(name)` still
    // type-checks as a known ref; paint lands when the peer is on disk.
    let peer_env = resolve_peer_style_env(module, base_dir);
    for name in names {
        if env.contains_key(name) {
            return Err(CompileError::new(format!(
                "Style const `{name}` is declared twice in `<scope>` (import + local)."
            ))
            .with_span(span)
            .with_category(RULE_ID));
        }
        if let Some(ref peer) = peer_env {
            if let Some(style) = peer.get(name) {
                env.insert(name.clone(), style.clone());
                continue;
            }
            return Err(CompileError::new(format!(
                "Style-value import `{name}` not found in `{module}` — export a \
                 `const {name} = style.….` from that `.sail` scope."
            ))
            .with_span(span)
            .with_category(RULE_ID));
        }
        // Peer not on disk (unit tests, missing path): keep the name as a
        // known Style ref so the import is legal; paint is empty until resolved.
        env.insert(name.clone(), Style::Empty);
    }
    Ok(())
}

fn resolve_peer_style_env(module: &str, base_dir: Option<&std::path::Path>) -> Option<StyleEnv> {
    let mut candidates = Vec::new();
    let m = module.trim();
    if m.is_empty() {
        return None;
    }
    if let Some(base) = base_dir {
        candidates.push(base.join(m));
        if !m.ends_with(".sail") {
            candidates.push(base.join(format!("{m}.sail")));
        }
    }
    candidates.push(std::path::PathBuf::from(m));
    if !m.ends_with(".sail") {
        candidates.push(std::path::PathBuf::from(format!("{m}.sail")));
    }
    for path in candidates {
        if let Ok(src) = std::fs::read_to_string(&path) {
            // Pull Style consts only (imports in the peer are not re-resolved).
            return Some(style_env_from_source_shallow(&src));
        }
    }
    None
}

/// Style consts from every `<scope>` body without re-entering import resolution.
fn style_env_from_source_shallow(source: &str) -> StyleEnv {
    let mut env = HashMap::new();
    harvest_style_consts_loose_all_scopes(source, &mut env);
    env
}

fn harvest_style_consts_loose_all_scopes(source: &str, env: &mut StyleEnv) {
    let mut rest = source;
    while let Some(open_rel) = rest.find("<scope") {
        let after_open_tag = &rest[open_rel..];
        let Some(gt_rel) = after_open_tag.find('>') else {
            break;
        };
        let tag_head = &after_open_tag[..gt_rel];
        if tag_head
            .as_bytes()
            .get(6)
            .is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'-')
        {
            rest = &after_open_tag[1..];
            continue;
        }
        let body_start = open_rel + gt_rel + 1;
        let after_body = &rest[body_start..];
        let Some(close_rel) = after_body.find("</scope>") else {
            break;
        };
        let body = &after_body[..close_rel];
        harvest_style_consts_loose(body, env);
        rest = &after_body[close_rel + "</scope>".len()..];
    }
}

/// Prop names whose type is a color-group union (type-keyed paint bind).
///
/// Scans every `<scope>` props block; used so Style parse recognizes
/// `bg(shade)` when `shade: Tone` (not only the product ambient `tone`).
pub(crate) fn color_group_typed_prop_names(source: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = source;
    while let Some(open_rel) = rest.find("<scope") {
        let after_open_tag = &rest[open_rel..];
        let Some(gt_rel) = after_open_tag.find('>') else {
            break;
        };
        let tag_head = &after_open_tag[..gt_rel];
        if tag_head
            .as_bytes()
            .get(6)
            .is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'-')
        {
            rest = &after_open_tag[1..];
            continue;
        }
        let body_start = open_rel + gt_rel + 1;
        let after_body = &rest[body_start..];
        let Some(close_rel) = after_body.find("</scope>") else {
            break;
        };
        let body = &after_body[..close_rel];
        if let Ok(block) = parse_scope_block(body, Span::new(0, body.len())) {
            for p in &block.props {
                if let crate::ir::PropKind::Union(members) = &p.kind {
                    if sail_lint::token_groups::is_color_group_union(members) {
                        let name = crate::emit::util::camel_of(&p.name);
                        if !out.contains(&name) {
                            out.push(name);
                        }
                    }
                }
            }
        }
        rest = &after_body[close_rel + "</scope>".len()..];
    }
    out
}

/// Collect Style consts from every `<scope>…</scope>` body in an SFC source.
/// Used by preprocess before `sx=` lowering so `Ref` names resolve.
pub(crate) fn style_env_from_source(source: &str) -> StyleEnv {
    let mut env = HashMap::new();
    let mut rest = source;
    while let Some(open_rel) = rest.find("<scope") {
        let after_open_tag = &rest[open_rel..];
        let Some(gt_rel) = after_open_tag.find('>') else {
            break;
        };
        // Reject `<scope…>` that is not the tag (e.g. longer name) — tag ends at >.
        let tag_head = &after_open_tag[..gt_rel];
        if tag_head
            .as_bytes()
            .get(6)
            .is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'-')
        {
            rest = &after_open_tag[1..];
            continue;
        }
        let body_start = open_rel + gt_rel + 1;
        let after_body = &rest[body_start..];
        let Some(close_rel) = after_body.find("</scope>") else {
            break;
        };
        let body = &after_body[..close_rel];
        if let Ok(block) = parse_scope_block(body, Span::new(0, body.len())) {
            for (k, v) in block.style_env {
                env.insert(k, v);
            }
        } else {
            // Partial recovery: still harvest well-formed consts when props fail.
            harvest_style_consts_loose(body, &mut env);
        }
        rest = &after_body[close_rel + "</scope>".len()..];
    }
    env
}

/// `const name = <Style expr>` — Style const binding (no runtime computation).
fn parse_style_const(
    s: &str,
    after_const: usize,
    span: Span,
) -> Result<(String, Style, usize), CompileError> {
    parse_named_style_binding(s, after_const, span, StyleBindingKw::Const)
}

/// `style name = <Style rhs>` — first-class Style declaration.
fn parse_style_decl(
    s: &str,
    after_style: usize,
    span: Span,
) -> Result<(String, Style, usize), CompileError> {
    parse_named_style_binding(s, after_style, span, StyleBindingKw::Style)
}

enum StyleBindingKw {
    Const,
    Style,
}

fn parse_named_style_binding(
    s: &str,
    after_kw: usize,
    span: Span,
    kw: StyleBindingKw,
) -> Result<(String, Style, usize), CompileError> {
    let bytes = s.as_bytes();
    let i = skip_trivia(s, after_kw, None);
    let (name, after_name) = read_word(s, i);
    let form_hint = match kw {
        StyleBindingKw::Const => "const filled = style({ bg(primary) })",
        StyleBindingKw::Style => "style filled = { bg(primary) }",
    };
    if name.is_empty() {
        let msg = match kw {
            StyleBindingKw::Const => format!(
                "`const` in `<scope>` needs a name — write `{form_hint}` \
                 (or preferred `style name = {{…}}`)."
            ),
            StyleBindingKw::Style => {
                // `style(` at statement position is not a declaration.
                if i < bytes.len() && bytes[i] == b'(' {
                    "`style(…)` is tag-position product syntax, not a `<scope>` \
                     declaration — write `style name = { … }` (or compose \
                     `style quiet = host, { … }`)."
                        .to_string()
                } else {
                    format!("`style` in `<scope>` needs a name — write `{form_hint}`.")
                }
            }
        };
        return Err(CompileError::new(msg)
            .with_span(span)
            .with_category(RULE_ID));
    }
    let mut j = skip_trivia(s, after_name, None);
    if j >= bytes.len() || bytes[j] != b'=' {
        return Err(CompileError::new(format!(
            "`{} {name}` in `<scope>` needs `=` — write `{} {name} = {{ … }}` \
             (or `style(…)` call form).",
            match kw {
                StyleBindingKw::Const => "const",
                StyleBindingKw::Style => "style",
            },
            match kw {
                StyleBindingKw::Const => "const",
                StyleBindingKw::Style => "style",
            },
        ))
        .with_span(span)
        .with_category(RULE_ID));
    }
    if j + 1 < bytes.len() && bytes[j + 1] == b'>' {
        return Err(CompileError::new(format!(
            "`{} {name}` in `<scope>` is a Style value, not an arrow function — \
             write `style {name} = {{ … }}`.",
            match kw {
                StyleBindingKw::Const => "const",
                StyleBindingKw::Style => "style",
            },
        ))
        .with_span(span)
        .with_category(RULE_ID));
    }
    j = skip_trivia(s, j + 1, None);
    let start = j;
    let end = scan_style_expr_end(s, start);
    let rhs = s[start..end].trim();
    if rhs.is_empty() {
        return Err(CompileError::new(format!(
            "`{} {name}` in `<scope>` has no Style RHS — write `style {name} = {{ … }}`.",
            match kw {
                StyleBindingKw::Const => "const",
                StyleBindingKw::Style => "style",
            },
        ))
        .with_span(span)
        .with_category(RULE_ID));
    }
    if !looks_like_style_const_rhs(rhs) {
        return Err(CompileError::new(format!(
            "`{} {name}` in `<scope>` must be a Style expression \
             (`{{…}}`, `style(…)`, compose `base, {{…}}`, or another Style name) — \
             computation belongs in `<controller>`.",
            match kw {
                StyleBindingKw::Const => "const",
                StyleBindingKw::Style => "style",
            },
        ))
        .with_span(span)
        .with_category(RULE_ID));
    }
    let (style, diags) = sail_style::parse_style_const_rhs(rhs);
    if let Some(msg) = style_const_diag_message(&diags) {
        return Err(CompileError::new(format!("Style `{name}`: {msg}"))
            .with_span(span)
            .with_category(RULE_ID));
    }
    let mut after = skip_trivia(s, end, None);
    if after < bytes.len() && bytes[after] == b';' {
        after += 1;
    }
    Ok((name.to_string(), style, after))
}

fn looks_like_style_const_rhs(rhs: &str) -> bool {
    let t = rhs.trim();
    if t.starts_with('{') || t.starts_with("style(") || t.starts_with("style.") {
        // Live Style RHS: object form, style({…}) / style(ref, {…}), compose head.
        // Free match(/when(/compound(/list forms are deleted (STYLE-BINDING-1).
        return true;
    }
    // Bare Style ref or compose head: `host` / `host, { … }` / `host, style({…})`.
    let head = t.split(',').next().unwrap_or("").trim();
    if head.is_empty() {
        return false;
    }
    let mut chars = head.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {
            head.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
        }
        _ => false,
    }
}

fn style_const_diag_message(diags: &[StyleDiagnostic]) -> Option<String> {
    // Any Style diagnostic on a const RHS is a hard compile error (fail closed).
    diags.first().map(|d| d.message())
}

/// End of a Style RHS: balanced `{}`/`()`/`[]` (strings/comments skipped).
/// Compose may continue after `,` across newlines; a closed group ends the
/// RHS unless the next non-trivia token is `,`.
fn scan_style_expr_end(s: &str, start: usize) -> usize {
    let bytes = s.as_bytes();
    let mut i = start;
    let mut depth = 0i32;
    while i < bytes.len() {
        let j = skip_strings_and_comments(s, i);
        if j != i {
            i = j;
            continue;
        }
        match bytes[i] {
            b'{' | b'(' | b'[' => depth += 1,
            b'}' | b')' | b']' => {
                if depth == 0 {
                    // Stray closer — stop before it so the outer loop diagnoses.
                    return i;
                }
                depth -= 1;
                if depth == 0 {
                    // Group closed. End here unless compose continues with `,`.
                    let k = skip_trivia(s, i + 1, None);
                    if k >= bytes.len() || bytes[k] != b',' {
                        return i + 1;
                    }
                }
            }
            b';' if depth == 0 => return i,
            b'\n' if depth == 0 => {
                let before = s[start..i].trim_end();
                if !before.is_empty() && !before.ends_with(',') {
                    return i;
                }
            }
            _ => {}
        }
        i += 1;
    }
    bytes.len()
}

/// Best-effort Style harvest when full scope parse fails (props errors, etc.).
fn harvest_style_consts_loose(body: &str, env: &mut StyleEnv) {
    let bytes = body.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        i = skip_trivia(body, i, None);
        if i >= bytes.len() {
            break;
        }
        if body[i..].starts_with("const") {
            let after_kw = i + 5;
            if after_kw < bytes.len()
                && (bytes[after_kw].is_ascii_alphanumeric() || bytes[after_kw] == b'_')
            {
                i += 1;
                continue;
            }
            if let Ok((name, style, end)) =
                parse_style_const(body, after_kw, Span::new(0, body.len()))
            {
                env.entry(name).or_insert(style);
                i = end;
                continue;
            }
        }
        if body[i..].starts_with("style") {
            let after_kw = i + 5;
            // Word boundary: `style` not `stylesheet` / `styleFoo`.
            if after_kw < bytes.len()
                && (bytes[after_kw].is_ascii_alphanumeric() || bytes[after_kw] == b'_')
            {
                i += 1;
                continue;
            }
            // Skip tag-position-looking `style(` at statement start.
            let after_triv = skip_trivia(body, after_kw, None);
            if after_triv < bytes.len() && bytes[after_triv] == b'(' {
                i = after_kw;
                continue;
            }
            if let Ok((name, style, end)) =
                parse_style_decl(body, after_kw, Span::new(0, body.len()))
            {
                env.entry(name).or_insert(style);
                i = end;
                continue;
            }
        }
        // Skip one token-ish span so we progress.
        let mut n = i + 1;
        while n < bytes.len() && (bytes[n] & 0xC0) == 0x80 {
            n += 1;
        }
        i = n;
    }
}

// ── statement pieces ─────────────────────────────────────────────────────────

/// The char starting at byte `i`, for diagnostics. Char-boundary-safe: a
/// multibyte char (`…`) must land in the message whole, never as a
/// `i..i+1` slice that panics mid-char.
fn char_at(s: &str, i: usize) -> char {
    s[i..].chars().next().unwrap_or('\u{FFFD}')
}

fn read_word(s: &str, i: usize) -> (&str, usize) {
    let bytes = s.as_bytes();
    let mut j = i;
    while j < bytes.len()
        && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_' || bytes[j] == b'$')
    {
        j += 1;
    }
    (&s[i..j], j)
}

/// Consume a `type X = …` alias starting at `i` (the `type` keyword or just
/// after `export`). Ends at the first `;` or newline at depth 0 after the `=`.
fn consume_type_alias(s: &str, i: usize) -> Result<usize, CompileError> {
    let j = skip_trivia(s, i, None);
    let (kw, after_kw) = read_word(s, j);
    if kw != "type" {
        return Err(CompileError::new(format!(
            "`export {kw}` is not a `<scope>` member — only `export type` is exportable here; {ALLOWED_HINT}."
        )).with_category(RULE_ID));
    }
    let bytes = s.as_bytes();
    let mut k = after_kw;
    let mut depth = 0i32;
    let mut seen_eq = false;
    while k < bytes.len() {
        k = skip_strings_and_comments(s, k);
        if k >= bytes.len() {
            break;
        }
        match bytes[k] {
            b'{' | b'(' | b'[' | b'<' => depth += 1,
            b'}' | b')' | b']' | b'>' => depth -= 1,
            b'=' if depth == 0 && k + 1 < bytes.len() && bytes[k + 1] != b'>' => seen_eq = true,
            b';' if depth == 0 && seen_eq => return Ok(k + 1),
            b'\n' if depth <= 0 && seen_eq => {
                // A newline ends the alias only when the RHS has content.
                let rhs_done = s[..k]
                    .trim_end()
                    .ends_with(|c: char| c != '=' && c != '|' && c != '&');
                if rhs_done {
                    return Ok(k);
                }
            }
            _ => {}
        }
        k += 1;
    }
    if !seen_eq {
        return Err(CompileError::new(
            "`type` declaration in `<scope>` has no `=` — write `type Name = …`.".to_string(),
        )
        .with_category(RULE_ID));
    }
    Ok(bytes.len())
}

/// After a member keyword, expect `: { … }`; return (open, close) brace offsets.
fn expect_member_object(
    s: &str,
    after_kw: usize,
    member: &str,
    span: Span,
) -> Result<(usize, usize), CompileError> {
    let bytes = s.as_bytes();
    let mut i = skip_trivia(s, after_kw, None);
    if i >= bytes.len() || bytes[i] != b':' {
        return Err(CompileError::new(format!(
            "`{member}` in `<scope>` is a member — write `{member}: {{ … }}`."
        ))
        .with_span(span)
        .with_category(RULE_ID));
    }
    i = skip_trivia(s, i + 1, None);
    if i >= bytes.len() || bytes[i] != b'{' {
        return Err(CompileError::new(format!(
            "`{member}:` expects an object body — write `{member}: {{ … }}`."
        ))
        .with_span(span)
        .with_category(RULE_ID));
    }
    let close = find_balanced(s, i)?;
    Ok((i, close))
}

/// `compound: [ … ]` — array body. Returns `(open_bracket, close_bracket)`.
fn expect_member_array(
    s: &str,
    after_kw: usize,
    member: &str,
    span: Span,
) -> Result<(usize, usize), CompileError> {
    let bytes = s.as_bytes();
    let mut i = skip_trivia(s, after_kw, None);
    if i >= bytes.len() || bytes[i] != b':' {
        return Err(CompileError::new(format!(
            "`{member}` in `<scope>` is a member — write `{member}: [ … ]`."
        ))
        .with_span(span)
        .with_category(RULE_ID));
    }
    i = skip_trivia(s, i + 1, None);
    if i >= bytes.len() || bytes[i] != b'[' {
        return Err(CompileError::new(format!(
            "`{member}:` expects an array body — write `{member}: [ {{ when: {{…}}, sx: '…' }}, … ]`."
        ))
        .with_span(span).with_category(RULE_ID));
    }
    let close = find_balanced_bracket(s, i)?;
    Ok((i, close))
}

/// Offset of the `]` matching the `[` at `open`.
fn find_balanced_bracket(s: &str, open: usize) -> Result<usize, CompileError> {
    let bytes = s.as_bytes();
    let mut depth = 0i32;
    let mut i = open;
    while i < bytes.len() {
        let j = skip_strings_and_comments(s, i);
        if j != i {
            i = j;
            continue;
        }
        match bytes[i] {
            b'[' | b'{' | b'(' => depth += 1,
            b']' => {
                depth -= 1;
                if depth == 0 {
                    return Ok(i);
                }
            }
            b'}' | b')' => depth -= 1,
            _ => {}
        }
        i += 1;
    }
    Err(CompileError::new("unclosed `[` in `<scope>` member.").with_category(RULE_ID))
}

// ── props fields ─────────────────────────────────────────────────────────────

/// Parse `props:` fields. Returns declared props.
fn parse_props_fields(body: &str, span: Span) -> Result<Vec<PropDecl>, CompileError> {
    let bytes = body.as_bytes();
    let mut out = Vec::new();
    let mut i = 0usize;
    loop {
        let mut doc: Option<String> = None;
        i = skip_trivia(body, i, Some(&mut doc));
        if i >= bytes.len() {
            break;
        }
        if bytes[i] == b',' || bytes[i] == b';' {
            i += 1;
            continue;
        }
        // Quoted names carry kebab-case attribute names ('line-numbers').
        let (name, after) = if bytes[i] == b'\'' || bytes[i] == b'"' {
            let end = skip_strings_and_comments(body, i);
            (body[i + 1..end - 1].to_string(), end)
        } else {
            let (w, a) = read_word(body, i);
            (w.to_string(), a)
        };
        let name = name.as_str();
        if name.is_empty() {
            return Err(CompileError::new(format!(
                "unexpected `{}` in `props:` — fields are `name: Type = literal`, \
                 or a union `name: 'a' | 'b' = 'a'`.",
                char_at(body, i)
            ))
            .with_span(span)
            .with_category(RULE_ID));
        }
        i = skip_trivia(body, after, None);

        // Optional `: Type` — up to a top-level `=` (not `=>`) or terminator.
        let mut annotation: Option<String> = None;
        if i < bytes.len() && bytes[i] == b':' {
            let start = skip_trivia(body, i + 1, None);
            let end = scan_annotation_end(body, start);
            let text = body[start..end].trim();
            if text.is_empty() {
                return Err(CompileError::new(format!(
                    "prop `{name}` has `:` but no type — write `{name}: Type` or `{name} = literal`."
                ))
                .with_span(span).with_category(RULE_ID));
            }
            annotation = Some(text.to_string());
            i = end;
        }

        // Optional `= literal` — up to a terminator at depth 0.
        let mut value: Option<String> = None;
        i = skip_trivia(body, i, None);
        if i < bytes.len() && bytes[i] == b'=' && (i + 1 >= bytes.len() || bytes[i + 1] != b'>') {
            let start = skip_trivia(body, i + 1, None);
            let end = scan_value_end(body, start);
            let text = body[start..end].trim();
            if text.is_empty() {
                return Err(CompileError::new(format!(
                    "prop `{name}` has `=` but no value — give a literal default or drop the `=`."
                ))
                .with_span(span)
                .with_category(RULE_ID));
            }
            // `<literal> as <Type>` — TypeScript's widening idiom for an empty
            // array/object literal whose type can't be inferred (`[] as
            // Array<{ id: string }>`). The literal IS the value; the cast
            // supplies the annotation when one wasn't already written.
            let (literal_text, cast_ty) = split_as_cast(text);
            if !is_literal(literal_text) {
                return Err(CompileError::new(format!(
                    "prop `{name}` default `{text}` is not a literal — `<scope>` holds types and \
                     literal values only; computed values belong in `<controller>`."
                ))
                .with_span(span)
                .with_category(RULE_ID));
            }
            if annotation.is_none()
                && let Some(ty) = cast_ty
            {
                annotation = Some(ty.to_string());
            }
            value = Some(literal_text.to_string());
            i = end;
        }

        out.push(classify_field(
            name,
            annotation.as_deref(),
            value.as_deref(),
            doc,
            span,
        )?);
    }
    Ok(out)
}

/// Map one parsed field onto a `PropDecl` — the kind/default/reflect rules.
fn classify_field(
    name: &str,
    annotation: Option<&str>,
    value: Option<&str>,
    doc: Option<String>,
    span: Span,
) -> Result<PropDecl, CompileError> {
    let decl = |kind: PropKind, default: Option<String>, reflect: bool| PropDecl {
        name: name.to_string(),
        kind,
        default,
        reflect,
        bindable: false,
        description: doc.clone(),
        span: Span::new(0, 0),
    };

    if let Some(ann) = annotation {
        // Union of string literals — the ONLY annotation shape that is not
        // Identity. Default is optional; when present it must be a member (no first-member magic).
        if let Some(members) = union_of_string_literals(ann) {
            let default = match value {
                Some(v) => {
                    let d = unquote(v);
                    if !members.contains(&d) {
                        return Err(CompileError::new(format!(
                            "prop `{name}` default {v} is not a member of its union `{ann}`."
                        ))
                        .with_span(span)
                        .with_category(RULE_ID));
                    }
                    Some(d)
                }
                // Optional union: omit attribute = rest path (no first-member magic).
                None => None,
            };
            return Ok(decl(PropKind::Union(members), default, true));
        }
        return Ok(match ann {
            "string" => decl(
                PropKind::String,
                Some(value.map(unquote).unwrap_or_default()),
                false,
            ),
            "number" => decl(
                PropKind::Number,
                Some(value.unwrap_or("0").to_string()),
                false,
            ),
            "boolean" => decl(
                PropKind::Bool,
                Some(value.unwrap_or("false").to_string()),
                true,
            ),
            // Arbitrary authored TypeScript: a plain reactive property, never
            // an attribute. The literal default (when given) is carried verbatim.
            _ => decl(
                PropKind::Identity(ann.to_string()),
                value.map(str::to_string),
                false,
            ),
        });
    }

    // No annotation: the kind comes from the literal.
    let Some(v) = value else {
        return Err(CompileError::new(format!(
            "prop `{name}` declares neither a type nor a default — write \
             `{name}: Type`, `{name} = literal`, or `{name}: Type = literal`."
        ))
        .with_span(span)
        .with_category(RULE_ID));
    };
    // Shared stdlib tone/size convention: a bare vocabulary default types as
    // the full shared union (ordinary reflected props / CVA axes).
    if name == "tone" || name == "size" {
        let vocab = if name == "tone" {
            TONE_VOCAB
        } else {
            SIZE_VOCAB
        };
        let d = unquote(v);
        if !vocab.split('|').any(|m| m == d) {
            return Err(CompileError::new(format!(
                "`{name}` uses the shared {name} vocabulary — its default must be one of \
                 `{vocab}` (got {v}). For a custom union, annotate it: \
                 `{name}: 'a' | 'b' = 'a'`."
            ))
            .with_span(span)
            .with_category(RULE_ID));
        }
        let members = vocab.split('|').map(str::to_string).collect();
        return Ok(decl(PropKind::Union(members), Some(d), true));
    }
    if v == "true" || v == "false" {
        return Ok(decl(PropKind::Bool, Some(v.to_string()), true));
    }
    if v.parse::<f64>().is_ok() {
        return Ok(decl(PropKind::Number, Some(v.to_string()), false));
    }
    if v.starts_with('\'') || v.starts_with('"') {
        return Ok(decl(PropKind::String, Some(unquote(v)), false));
    }
    // Object/array literal without an annotation: the type IS the contract —
    // demand it rather than inventing one.
    Err(CompileError::new(format!(
        "prop `{name}` has an object/array default but no type — annotate it: \
         `{name}: T = {v}`."
    ))
    .with_span(span)
    .with_category(RULE_ID))
}

/// `'a' | 'b' | 'c'` → the members; anything else → None.
fn union_of_string_literals(ann: &str) -> Option<Vec<String>> {
    if !ann.contains('|') {
        return None;
    }
    let mut members = Vec::new();
    for part in split_top_level_pipe(ann) {
        let p = part.trim();
        let quoted = (p.starts_with('\'') && p.ends_with('\'') && p.len() >= 2)
            || (p.starts_with('"') && p.ends_with('"') && p.len() >= 2);
        if !quoted {
            return None;
        }
        members.push(unquote(p));
    }
    (members.len() >= 2).then_some(members)
}

fn split_top_level_pipe(s: &str) -> Vec<&str> {
    let bytes = s.as_bytes();
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut start = 0usize;
    let mut i = 0usize;
    while i < bytes.len() {
        let j = skip_strings_and_comments(s, i);
        if j != i {
            i = j;
            continue;
        }
        match bytes[i] {
            b'{' | b'(' | b'[' | b'<' => depth += 1,
            b'}' | b')' | b']' => depth -= 1,
            // An arrow's `>` (`=>`) closes nothing — mirrors the guard in
            // `scan_annotation_end`. Without it, a method-typed field (e.g.
            // `Array<{ onClick: () => void }>`) desyncs the depth counter.
            b'>' if i > 0 && bytes[i - 1] != b'=' => depth -= 1,
            b'|' if depth == 0 => {
                parts.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    parts.push(&s[start..]);
    parts
}

/// Split `<literal> as <Type>` at the top-level ` as ` keyword — TypeScript's
/// widening idiom for an empty array/object literal whose type can't be
/// inferred (`[] as Array<{ id: string }>`). Returns `(literal, None)`
/// unchanged when there's no top-level ` as `.
fn split_as_cast(s: &str) -> (&str, Option<&str>) {
    let bytes = s.as_bytes();
    let mut depth = 0i32;
    let mut i = 0usize;
    while i < bytes.len() {
        let j = skip_strings_and_comments(s, i);
        if j != i {
            i = j;
            continue;
        }
        match bytes[i] {
            b'{' | b'(' | b'[' => depth += 1,
            b'}' | b')' | b']' => depth -= 1,
            b'a' if depth == 0
                && i > 0
                && bytes[i - 1].is_ascii_whitespace()
                && i + 2 < bytes.len()
                && bytes[i + 1] == b's'
                && bytes[i + 2].is_ascii_whitespace() =>
            {
                let lit = s[..i].trim_end();
                let ty = s[i + 2..].trim();
                if !lit.is_empty() && !ty.is_empty() {
                    return (lit, Some(ty));
                }
            }
            _ => {}
        }
        i += 1;
    }
    (s, None)
}

// ── literal validation ───────────────────────────────────────────────────────

/// A value in `<scope>` must be data: a string/number/boolean literal, or an
/// array/object of the same. Identifiers, calls, template strings with `${}`,
/// and arrows are code — refused.
pub(crate) fn is_literal(v: &str) -> bool {
    let v = v.trim();
    if v.is_empty() {
        return false;
    }
    if v == "true" || v == "false" || v == "null" {
        return true;
    }
    if v.parse::<f64>().is_ok() {
        return true;
    }
    if (v.starts_with('\'') && v.ends_with('\'') && v.len() >= 2)
        || (v.starts_with('"') && v.ends_with('"') && v.len() >= 2)
    {
        // A quoted literal, as long as the quotes are the ends of ONE string.
        return scan_one_string(v);
    }
    if v.starts_with('[') && v.ends_with(']') {
        let inner = &v[1..v.len() - 1];
        return split_top_level_commas(inner).iter().all(|e| {
            let e = e.trim();
            e.is_empty() || is_literal(e)
        });
    }
    if v.starts_with('{') && v.ends_with('}') {
        return parse_object_fields(&v[1..v.len() - 1])
            .iter()
            .all(|(_, fv)| is_literal(fv));
    }
    false
}

/// True iff `v` is exactly one string literal (no `'a' + b()` smuggling).
fn scan_one_string(v: &str) -> bool {
    let bytes = v.as_bytes();
    let quote = bytes[0];
    let mut i = 1;
    while i < bytes.len() - 1 {
        if bytes[i] == b'\\' {
            i += 2;
            continue;
        }
        if bytes[i] == quote {
            return false; // closed early — trailing code after the literal
        }
        i += 1;
    }
    true
}

fn split_top_level_commas(s: &str) -> Vec<&str> {
    let bytes = s.as_bytes();
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut start = 0usize;
    let mut i = 0usize;
    while i < bytes.len() {
        let j = skip_strings_and_comments(s, i);
        if j != i {
            i = j;
            continue;
        }
        match bytes[i] {
            b'{' | b'(' | b'[' => depth += 1,
            b'}' | b')' | b']' => depth -= 1,
            b',' if depth == 0 => {
                parts.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    parts.push(&s[start..]);
    parts
}

// ── scanning helpers ─────────────────────────────────────────────────────────

/// Skip whitespace + comments from `i`; when `doc` is given, capture the LAST
/// `/** … */` block's inner text (a field's description, like today's CEM docs).
fn skip_trivia(s: &str, mut i: usize, mut doc: Option<&mut Option<String>>) -> usize {
    let bytes = s.as_bytes();
    loop {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            let start = i;
            let is_doc = i + 2 < bytes.len() && bytes[i + 2] == b'*';
            i += 2;
            while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i = (i + 2).min(bytes.len());
            if is_doc && let Some(d) = doc.as_deref_mut() {
                let inner = s[start..i]
                    .trim_start_matches("/**")
                    .trim_end_matches("*/")
                    .lines()
                    .map(|l| l.trim().trim_start_matches('*').trim())
                    .collect::<Vec<_>>()
                    .join(" ")
                    .trim()
                    .to_string();
                *d = Some(inner);
            }
            continue;
        }
        return i;
    }
}

/// Step over a string literal or comment starting at `i`; returns the offset
/// just past it, or `i` unchanged when none starts here.
fn skip_strings_and_comments(s: &str, i: usize) -> usize {
    let bytes = s.as_bytes();
    if i >= bytes.len() {
        return i;
    }
    match bytes[i] {
        b'\'' | b'"' | b'`' => {
            let q = bytes[i];
            let mut j = i + 1;
            while j < bytes.len() {
                if bytes[j] == b'\\' {
                    j += 2;
                    continue;
                }
                if bytes[j] == q {
                    return j + 1;
                }
                j += 1;
            }
            bytes.len()
        }
        b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'/' => {
            let mut j = i;
            while j < bytes.len() && bytes[j] != b'\n' {
                j += 1;
            }
            j
        }
        b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'*' => {
            let mut j = i + 2;
            while j + 1 < bytes.len() && !(bytes[j] == b'*' && bytes[j + 1] == b'/') {
                j += 1;
            }
            (j + 2).min(bytes.len())
        }
        _ => i,
    }
}

/// End of a type annotation: the first top-level `=` (not `=>`) or field
/// terminator (`,`/`;`, or a newline that ends a complete type).
fn scan_annotation_end(s: &str, start: usize) -> usize {
    let bytes = s.as_bytes();
    let mut depth = 0i32;
    let mut i = start;
    while i < bytes.len() {
        let j = skip_strings_and_comments(s, i);
        if j != i {
            i = j;
            continue;
        }
        match bytes[i] {
            b'{' | b'(' | b'[' | b'<' => depth += 1,
            b'}' | b')' | b']' => depth -= 1,
            b'>' if i > 0 && bytes[i - 1] != b'=' => depth -= 1,
            b'=' if depth == 0 && (i + 1 >= bytes.len() || bytes[i + 1] != b'>') => return i,
            b',' | b';' if depth == 0 => return i,
            b'\n' if depth == 0 => {
                // A newline ends the annotation unless the type continues
                // (`|`, `&`, `=>` dangling at either edge).
                let before = s[start..i].trim_end();
                let after = s[i..].trim_start();
                let open = before.ends_with(['|', '&', '=', '<', ','])
                    || before.ends_with("=>")
                    || after.starts_with(['|', '&', '=']);
                if !before.is_empty() && !open {
                    return i;
                }
            }
            _ => {}
        }
        i += 1;
    }
    bytes.len()
}

/// End of a literal value: the first `,`/`;`/newline at depth 0. Trailing
/// comments are trivia, not value — the returned end excludes them (an
/// inline `/* what this is */` after a default must not leak into the
/// literal).
fn scan_value_end(s: &str, start: usize) -> usize {
    let bytes = s.as_bytes();
    let mut depth = 0i32;
    let mut i = start;
    let mut last_content_end = start;
    while i < bytes.len() {
        if bytes[i] == b'\'' || bytes[i] == b'"' || bytes[i] == b'`' {
            i = skip_strings_and_comments(s, i);
            last_content_end = i;
            continue;
        }
        let j = skip_strings_and_comments(s, i);
        if j != i {
            // A comment: step over it WITHOUT extending the content end.
            i = j;
            continue;
        }
        match bytes[i] {
            b'{' | b'(' | b'[' => depth += 1,
            b'}' | b')' | b']' => depth -= 1,
            b',' | b';' | b'\n' if depth == 0 => return last_content_end,
            _ => {}
        }
        if !bytes[i].is_ascii_whitespace() {
            last_content_end = i + 1;
        }
        i += 1;
    }
    last_content_end
}

/// Offset of the `}` matching the `{` at `open`.
fn find_balanced(s: &str, open: usize) -> Result<usize, CompileError> {
    let bytes = s.as_bytes();
    let mut depth = 0i32;
    let mut i = open;
    while i < bytes.len() {
        let j = skip_strings_and_comments(s, i);
        if j != i {
            i = j;
            continue;
        }
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Ok(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    Err(CompileError::new("unclosed `{` in `<scope>`.".to_string()).with_category(RULE_ID))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block(body: &str) -> ScopeBlock {
        parse_scope_block(body, Span::new(0, body.len())).expect("parses")
    }
    fn err(body: &str) -> CompileError {
        parse_scope_block(body, Span::new(0, body.len())).expect_err("must refuse")
    }
    fn prop<'a>(b: &'a ScopeBlock, n: &str) -> &'a PropDecl {
        b.props
            .iter()
            .find(|p| p.name == n)
            .unwrap_or_else(|| panic!("no prop {n}"))
    }

    #[test]
    fn literal_defaults_infer_their_kinds() {
        let b = block("props: {\n  label = 'Loading'\n  centered = false\n  count = 3\n}");
        assert_eq!(prop(&b, "label").kind, PropKind::String);
        assert_eq!(prop(&b, "label").default.as_deref(), Some("Loading"));
        assert_eq!(prop(&b, "centered").kind, PropKind::Bool);
        assert!(prop(&b, "centered").reflect);
        assert_eq!(prop(&b, "count").kind, PropKind::Number);
        assert_eq!(prop(&b, "count").default.as_deref(), Some("3"));
    }

    #[test]
    fn union_annotation_with_explicit_default() {
        let b = block("props: {\n  kind: 'a' | 'b' = 'b'\n}");
        assert_eq!(
            prop(&b, "kind").kind,
            PropKind::Union(vec!["a".into(), "b".into()])
        );
        assert_eq!(prop(&b, "kind").default.as_deref(), Some("b"));
        assert!(prop(&b, "kind").reflect);
    }

    #[test]
    fn union_without_default_is_optional() {
        let b = block("props: {\n  kind: 'a' | 'b'\n}");
        assert_eq!(
            prop(&b, "kind").kind,
            PropKind::Union(vec!["a".into(), "b".into()])
        );
        assert_eq!(prop(&b, "kind").default, None);
        assert!(prop(&b, "kind").reflect);
    }

    #[test]
    fn identity_types_carry_verbatim_ts() {
        let b = block(
            "props: {\n  config: { a: 'x' | 'y' }\n  columns: Column[]\n  data: DataPoint\n  onSelect: (id: string) => void\n}",
        );
        assert_eq!(
            prop(&b, "config").kind,
            PropKind::Identity("{ a: 'x' | 'y' }".into())
        );
        assert_eq!(
            prop(&b, "columns").kind,
            PropKind::Identity("Column[]".into())
        );
        assert_eq!(
            prop(&b, "data").kind,
            PropKind::Identity("DataPoint".into())
        );
        assert_eq!(
            prop(&b, "onSelect").kind,
            PropKind::Identity("(id: string) => void".into())
        );
        assert!(prop(&b, "data").default.is_none());
    }

    #[test]
    fn identity_with_annotated_literal_default() {
        let b = block("props: {\n  center: Point = { x: 0, y: 0 }\n}");
        assert_eq!(prop(&b, "center").kind, PropKind::Identity("Point".into()));
        assert_eq!(
            prop(&b, "center").default.as_deref(),
            Some("{ x: 0, y: 0 }")
        );
    }

    #[test]
    fn array_of_object_type_default_is_identity_not_union() {
        // `Array<{...}>` with a method-typed field: the exact shape a
        // false-union classifier would trip on (an object type argument
        // containing its own `=>` arrow). Must land as Identity, never
        // PropKind::Union, and the annotation must be captured whole.
        let b = block("props: {\n  items: Array<{ id: string; onClick: () => void }> = []\n}");
        assert_eq!(
            prop(&b, "items").kind,
            PropKind::Identity("Array<{ id: string; onClick: () => void }>".into())
        );
        assert_eq!(prop(&b, "items").default.as_deref(), Some("[]"));
    }

    #[test]
    fn as_cast_widens_an_empty_array_default_and_infers_the_annotation() {
        // TypeScript's widening idiom: `[] as Array<{...}>` gives an empty
        // array literal a type when it can't be inferred. No explicit
        // `: Type` — the cast supplies the annotation. This must land as
        // Identity (verbatim TS), never PropKind::Union, with the literal
        // `[]` as the default (the cast itself is not the value).
        let b = block("props: {\n  items = [] as Array<{ id: string; onClick: () => void }>\n}");
        assert_eq!(
            prop(&b, "items").kind,
            PropKind::Identity("Array<{ id: string; onClick: () => void }>".into())
        );
        assert_eq!(prop(&b, "items").default.as_deref(), Some("[]"));
    }

    #[test]
    fn as_cast_with_explicit_annotation_strips_the_redundant_cast() {
        let b = block("props: {\n  items: Array<{ id: string }> = [] as Array<{ id: string }>\n}");
        assert_eq!(
            prop(&b, "items").kind,
            PropKind::Identity("Array<{ id: string }>".into())
        );
        assert_eq!(prop(&b, "items").default.as_deref(), Some("[]"));
    }

    #[test]
    fn as_cast_to_a_string_union_still_classifies_as_union() {
        // The cast target can itself be a union of string literals — this
        // must still land as PropKind::Union, not Identity, proving the
        // `as`-cast path and the union-detection path compose correctly.
        let b = block("props: {\n  kind = 'a' as 'a' | 'b'\n}");
        assert_eq!(
            prop(&b, "kind").kind,
            PropKind::Union(vec!["a".into(), "b".into()])
        );
        assert_eq!(prop(&b, "kind").default.as_deref(), Some("a"));
    }

    #[test]
    fn tone_and_size_bare_defaults_expand_shared_unions() {
        let b = block("props: {\n  tone = 'primary'\n  size = 'md'\n}");
        match &prop(&b, "tone").kind {
            PropKind::Union(v) => {
                assert_eq!(v.len(), 7);
                assert!(!v.iter().any(|x| x == "default"));
            }
            k => panic!("tone must be the shared union, got {k:?}"),
        }
        assert_eq!(prop(&b, "tone").default.as_deref(), Some("primary"));
        assert_eq!(prop(&b, "size").default.as_deref(), Some("md"));
    }

    #[test]
    fn tone_default_outside_vocab_is_refused() {
        let e = err("props: {\n  tone = 'sparkly'\n}");
        assert!(
            e.message.contains("vocabulary") || e.message.contains("shared tone"),
            "{}",
            e.message
        );
    }

    #[test]
    fn object_default_without_annotation_demands_the_type() {
        let e = err("props: {\n  data = { x: 1 }\n}");
        assert!(e.message.contains("annotate"), "{}", e.message);
    }

    #[test]
    fn non_literal_values_are_refused() {
        let e = err("props: {\n  now = Date.now()\n}");
        assert!(e.message.contains("not a literal"), "{}", e.message);
    }

    #[test]
    fn executable_statements_are_refused_with_teaching() {
        let e = err("const MAX = 50");
        assert!(
            e.message.contains("`<controller>`") || e.message.contains("Style expression"),
            "{}",
            e.message
        );
        let e = err("import { signal } from '@ship/sail-runtime'");
        assert!(
            e.message.contains("not a `<scope>` member") || e.message.contains("Style values"),
            "{}",
            e.message
        );
        let e = err("props()");
        assert!(e.message.contains("props: { … }"), "{}", e.message);
    }

    #[test]
    fn type_import_is_accepted() {
        let b = block("import type { Row } from './types'\nprops: { open = false }");
        assert!(
            b.types.contains("import type { Row } from './types'"),
            "types={}",
            b.types
        );
        assert_eq!(prop(&b, "open").default.as_deref(), Some("false"));
    }

    #[test]
    fn style_value_import_is_accepted() {
        let b = block("import { filled } from './styles.sail'\nprops: { label = '' }");
        assert!(
            b.style_env.contains_key("filled"),
            "must register Style name"
        );
        assert!(
            b.types.contains("import { filled } from './styles.sail'"),
            "types={}",
            b.types
        );
    }

    #[test]
    fn package_value_import_is_refused() {
        let e = err("import { signal } from '@ship/sail-runtime'");
        assert!(
            e.message.contains("Style values") || e.message.contains("not a `<scope>`"),
            "{}",
            e.message
        );
    }

    #[test]
    fn side_effect_import_is_refused() {
        let e = err("import './polyfill'");
        assert!(e.message.contains("side-effect"), "{}", e.message);
    }

    #[test]
    fn style_const_is_a_scope_member() {
        let b = block(
            "const filled = style({ bg(primary).fg(primary-foreground) })\nprops: {\n  label = ''\n}",
        );
        assert!(
            b.style_env.contains_key("filled"),
            "const must land in style_env"
        );
        assert_eq!(prop(&b, "label").default.as_deref(), Some(""));
    }

    #[test]
    fn style_decl_is_a_scope_member() {
        let b = block(
            "style filled = { bg(primary).fg(primary-foreground) }\nprops: {\n  label = ''\n}",
        );
        assert!(
            b.style_env.contains_key("filled"),
            "style name = must land in style_env"
        );
        assert_eq!(prop(&b, "label").default.as_deref(), Some(""));
    }

    #[test]
    fn style_decl_call_form_is_accepted() {
        let b = block("style host = style({ bg(primary) })\nprops: { label = '' }");
        assert!(b.style_env.contains_key("host"));
    }

    #[test]
    fn style_decl_compose_merges() {
        let b = block("style a = { pad(1) }\nstyle b = a, { pad(2) }\nprops: { label = '' }");
        assert!(b.style_env.contains_key("a"));
        assert!(b.style_env.contains_key("b"));
        // Compose is ordered List (same as style(a, {…})).
        match b.style_env.get("b") {
            Some(sail_style::Style::List(parts)) => {
                assert_eq!(parts.len(), 2, "compose should be List of base + partial");
            }
            other => panic!("expected List compose, got {other:?}"),
        }
    }

    #[test]
    fn style_const_match_rhs_is_accepted() {
        let b = block(
            "const soft = style({ bg(muted) })\nconst paint = style({ match: { variant: { soft: soft, solid: style({ bg(primary) }) } } })",
        );
        assert!(b.style_env.contains_key("soft"));
        assert!(b.style_env.contains_key("paint"));
    }

    #[test]
    fn style_decl_and_const_style_binding() {
        let b = block(
            "style host = { bg(primary) }\nconst legacy = style({ bg(muted) })\nprops: { label = '' }",
        );
        assert!(b.style_env.contains_key("host"));
        assert!(b.style_env.contains_key("legacy"));
    }

    #[test]
    fn button_vsoft_rhs_diags() {
        let rhs = "{bg(hover-bg).border({ color: border }).fg(button), when: {hover: filter({ opacity: 90 })}}";
        let (_, diags) = sail_style::parse_style_const_rhs(rhs);
        assert!(
            diags.is_empty(),
            "vSoft RHS: {:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn button_stdlib_harvests_variant_styles() {
        let src = include_str!("../../stdlib/button.sail");
        let env = style_env_from_source(src);
        for name in [
            "base",
            "vSolid",
            "vSoft",
            "vOutline",
            "vGhost",
            "vGhostTone",
        ] {
            assert!(
                env.contains_key(name),
                "missing `{name}` in {:?}",
                env.keys().collect::<Vec<_>>()
            );
        }
        let expr = "style(base, {match: { variant: { ghost: style(vGhost, vGhostTone) } }})";
        let mut css = String::new();
        let mut seen = std::collections::HashSet::new();
        sail_style::lower_style_source(expr, &env, &mut css, &mut seen).unwrap_or_else(|diags| {
            panic!(
                "button attach failed: {:?}",
                diags.iter().map(|d| d.message()).collect::<Vec<_>>()
            )
        });
    }

    #[test]
    fn style_call_as_scope_statement_is_refused() {
        let e = err("style({ bg(primary) })\nprops: { label = '' }");
        assert!(
            e.message.contains("tag-position") || e.message.contains("declaration"),
            "{}",
            e.message
        );
    }

    #[test]
    fn type_aliases_are_captured_for_the_module() {
        let b = block(
            "export type Point = { x: number; y: number }\ntype Kind = 'a' | 'b'\nprops: {\n  center: Point = { x: 0, y: 0 }\n}",
        );
        assert!(b.types.contains("export type Point"));
        assert!(b.types.contains("type Kind"));
    }

    #[test]
    fn cva_map_annotation_is_identity_not_reflected_union() {
        // CVA key→sx tables are no longer special-cased. They parse as ordinary
        // authored TypeScript object-type annotations (Identity), not reflected unions.
        let b = block(
            "props: {\n  variant: {\n    solid: 'bg-tone text-tone-contrast',\n    soft: 'bg-tone-bg text-tone-fg hover:bg-muted',\n  }\n  tone: 'default' | 'primary' = 'default'\n}",
        );
        assert!(
            matches!(prop(&b, "variant").kind, PropKind::Identity(_)),
            "CVA map annotation must be treated as Identity, not a reflected union"
        );
        assert_eq!(
            prop(&b, "tone").kind,
            PropKind::Union(vec!["default".into(), "primary".into()])
        );
        assert_eq!(prop(&b, "tone").default.as_deref(), Some("default"));
    }

    #[test]
    fn doc_comments_reach_prop_descriptions() {
        let b = block("props: {\n  /** The visible label. */\n  label = ''\n}");
        assert_eq!(
            prop(&b, "label").description.as_deref(),
            Some("The visible label.")
        );
    }

    #[test]
    fn quoted_kebab_names_are_fields() {
        let b = block("props: {\n  'line-numbers' = true\n}");
        assert_eq!(prop(&b, "line-numbers").kind, PropKind::Bool);
    }

    #[test]
    fn empty_scope_is_valid() {
        let b = block("");
        assert!(b.props.is_empty() && b.types.is_empty() && b.style_env.is_empty());
    }

    #[test]
    fn style_decl_nested_when_hover_multiline() {
        let b = block(
            r#"
style host = {
  pad(2),
  when: {
    hover: bg(primary)
  }
}
style next = {
  bg(muted)
}
props: { label = '' }
"#,
        );
        assert!(
            b.style_env.contains_key("host"),
            "host: {:?}",
            b.style_env.keys()
        );
        assert!(
            b.style_env.contains_key("next"),
            "next: {:?}",
            b.style_env.keys()
        );
        assert_eq!(b.style_env.len(), 2);
    }

    #[test]
    fn style_decl_string_with_braces() {
        // Quoted match-arm key keeps `}` inside a string so depth stays honest.
        let b = block(
            r#"
style a = {
  match: {
    kind: {
      "has } brace": { pad(1) }
    }
  }
}
style b = { pad(1) }
"#,
        );
        assert!(b.style_env.contains_key("a"));
        assert!(b.style_env.contains_key("b"));
    }

    #[test]
    fn style_decl_comments_with_braces() {
        let b = block(
            r#"
style a = {
  // closing } here is comment
  /* } { */
  bg(primary)
}
style b = { pad(1) }
"#,
        );
        assert!(b.style_env.contains_key("a"));
        assert!(b.style_env.contains_key("b"));
    }

    #[test]
    fn style_decl_compose_multiline_after_comma() {
        let b = block(
            r#"
style a = { pad(1) }
style b = a,
{ pad(2) }
style c = a,
style({ pad(3) })
"#,
        );
        assert!(b.style_env.contains_key("a"));
        assert!(b.style_env.contains_key("b"));
        assert!(b.style_env.contains_key("c"));
        match b.style_env.get("b") {
            Some(sail_style::Style::List(parts)) => {
                assert_eq!(parts.len(), 2, "a, {{…}} compose");
            }
            other => panic!("expected List compose for b, got {other:?}"),
        }
        match b.style_env.get("c") {
            Some(sail_style::Style::List(parts)) => {
                assert_eq!(parts.len(), 2, "a, style({{…}}) compose");
            }
            other => panic!("expected List compose for c, got {other:?}"),
        }
    }

    #[test]
    fn style_decl_back_to_back_objects() {
        let b = block(
            r#"
style a = { pad(1) }
style b = { pad(2) }
style c = { when: { hover: bg(primary) } }
"#,
        );
        assert_eq!(b.style_env.len(), 3);
        for name in ["a", "b", "c"] {
            assert!(b.style_env.contains_key(name), "missing {name}");
        }
    }

    #[test]
    fn style_decl_trailing_comment_before_next() {
        let b = block(
            r#"
style a = { pad(1) } // keep
style b = { pad(2) }
"#,
        );
        assert!(b.style_env.contains_key("a"));
        assert!(b.style_env.contains_key("b"));
    }

    #[test]
    fn unknown_word_still_refused() {
        let e = err("filled = { pad(1) }");
        assert!(
            e.message.contains("`filled` is not a `<scope>` member"),
            "{}",
            e.message
        );
    }

    /// LSP path (`check_all_with_tier`) must not emit scope-member noise on
    /// the multi-line Style shapes that used to re-enter mid-declaration.
    #[test]
    fn check_all_with_tier_clean_on_multiline_style_scope() {
        let src = r#"
<scope>
  style host = {
    pad(2),
    when: {
      hover: bg(primary)
    }
  }
  style quiet = host,
  {
    pad(1)
  }
  style labeled = {
    match: {
      kind: {
        "has } brace": { pad(1) }
      }
    }
  }
  // } in comment
  style noted = {
    /* } { */
    bg(muted)
  }
</scope>
<div style(host)></div>
"#;
        let diags = crate::check_all_with_tier(src, None, &[], None, crate::DiagnosticTier::OnType);
        let scope_noise: Vec<_> = diags
            .iter()
            .filter(|d| d.message.contains("is not a `<scope>` member"))
            .map(|d| d.message.as_str())
            .collect();
        assert!(
            scope_noise.is_empty(),
            "false scope-member diags: {scope_noise:?}\nall={diags:?}"
        );
    }
}
