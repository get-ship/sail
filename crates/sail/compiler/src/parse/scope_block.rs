//! `<scope>` — the component's static surface. A fenced subset of TypeScript:
//! `type` / `export type` aliases plus field declarations (`name: Type = value`),
//! where every value is a LITERAL. Nothing in the block executes — no imports,
//! no `const`, no calls. If it computes, it belongs in `<controller>`.
//!
//! Members:
//!   - `type X = …` / `export type X = …` — aliases, threaded into the
//!     compiled module so the controller can reuse them.
//!   - `props: { … }` — the component's typed inputs (field declarations).
//!     **CVA-in-props (the only prop→paint matrix form):** an axis whose
//!     value is a map of key → sx string (`variant: { solid: 'bg-tone', … }`)
//!     types as the key union (first key = default), reflects as an
//!     attribute, and lowers through the **same sx atom pipeline** as host
//!     `sx=`. Host `sx=` remains base paint.
//!   - `compound: [ { when: { … }, sx: '…' }, … ]` — combination rules over
//!     prop axes (sibling of `props`).
//!   - `variants: { … }` — **legacy** parallel surface (same lowerer). Prefer
//!     CVA-in-props; do not grow this as a second stylesheet language.
//!
//! Field grammar (props): `name = literal` (type inferred), `name: Type`
//! (parent must fill), `name: Type = literal`, or CVA map `name: { k: 'sx…' }`.
//! Unions require an explicit default — there is no first-member magic in
//! ordinary props (CVA maps use first key as default). `tone` / `size` are
//! the framework channels: declared with a bare vocabulary default they type
//! as the full channel union.

use crate::diagnostics::{CompileError, Span};
use crate::ir::directive::{PropDecl, PropKind};

use super::preprocess::{parse_object_fields, unquote, variants_object_to_css};

/// Everything a `<scope>` block contributes to the Program.
#[derive(Debug)]
pub(crate) struct ScopeBlock {
    pub props: Vec<PropDecl>,
    /// CSS lowered from the `variants:` table (empty when absent).
    pub variants_css: String,
    /// Raw `type` / `export type` alias text, appended to the controller
    /// module (aliases hoist, so appending preserves controller spans).
    pub types: String,
}

const TONE_VOCAB: &str = "default|primary|info|success|warning|destructive|accent|muted";
const SIZE_VOCAB: &str = "xs|sm|md|lg";

const ALLOWED_HINT: &str = "`<scope>` holds `type` declarations, `props:` (incl. CVA key→sx maps), \
     optional `compound:`, and legacy `variants:` — types and literal values only; \
     logic goes in `<controller>`";

pub(crate) fn parse_scope_block(body: &str, span: Span) -> Result<ScopeBlock, CompileError> {
    let bytes = body.as_bytes();
    let mut i = 0usize;
    let mut out = ScopeBlock { props: Vec::new(), variants_css: String::new(), types: String::new() };

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
            "props" => {
                let obj_span = expect_member_object(body, after, "props", span)?;
                let (props, cva_css) =
                    parse_props_fields(&body[obj_span.0 + 1..obj_span.1], span)?;
                out.props.extend(props);
                out.variants_css.push_str(&cva_css);
                i = obj_span.1 + 1;
            }
            "compound" => {
                // Sibling of `props`: combination rules over axes.
                let arr_span = expect_member_array(body, after, "compound", span)?;
                let inner = &body[arr_span.0..arr_span.1 + 1]; // include brackets
                let css = variants_object_to_css(&format!("{{ compound: {inner} }}"));
                out.variants_css.push_str(&css);
                i = arr_span.1 + 1;
            }
            "variants" => {
                // Legacy parallel surface — same lowerer as CVA-in-props.
                let obj_span = expect_member_object(body, after, "variants", span)?;
                let inner = &body[obj_span.0 + 1..obj_span.1];
                let (axis_props, normalized) = parse_variants_table(inner, span)?;
                out.props.extend(axis_props);
                out.variants_css.push_str(&variants_object_to_css(&normalized));
                i = obj_span.1 + 1;
            }
            "" => {
                return Err(CompileError::new(format!(
                    "unexpected `{}` in `<scope>` — {ALLOWED_HINT}.",
                    char_at(body, i)
                ))
                .with_span(span));
            }
            other => {
                // Name the closest wrong turn we can recognize.
                let hint = match other {
                    "import" => " Type imports are not supported here yet — inline the type.",
                    "const" | "let" | "var" => {
                        " A constant is either a literal (declare it as a field default) \
                         or a computation (it belongs in `<controller>`)."
                    }
                    _ => "",
                };
                return Err(CompileError::new(format!(
                    "`{other}` is not a `<scope>` member — {ALLOWED_HINT}.{hint}"
                ))
                .with_span(span));
            }
        }
    }
    Ok(out)
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
    while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_' || bytes[j] == b'$') {
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
        )));
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
                let rhs_done = s[..k].trim_end().ends_with(|c: char| c != '=' && c != '|' && c != '&');
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
        ));
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
        .with_span(span));
    }
    i = skip_trivia(s, i + 1, None);
    if i >= bytes.len() || bytes[i] != b'{' {
        return Err(CompileError::new(format!(
            "`{member}:` expects an object body — write `{member}: {{ … }}`."
        ))
        .with_span(span));
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
        .with_span(span));
    }
    i = skip_trivia(s, i + 1, None);
    if i >= bytes.len() || bytes[i] != b'[' {
        return Err(CompileError::new(format!(
            "`{member}:` expects an array body — write `{member}: [ {{ when: {{…}}, sx: '…' }}, … ]`."
        ))
        .with_span(span));
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
    Err(CompileError::new("unclosed `[` in `<scope>` member."))
}

// ── props fields ─────────────────────────────────────────────────────────────

/// Parse `props:` fields. Returns declared props plus CSS lowered from any
/// CVA-in-props axes (key → sx string maps) through the same pipeline as
/// host `sx=` / legacy `variants:`.
fn parse_props_fields(body: &str, span: Span) -> Result<(Vec<PropDecl>, String), CompileError> {
    let bytes = body.as_bytes();
    let mut out = Vec::new();
    let mut cva_axes: Vec<String> = Vec::new();
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
                 or a CVA map `name: {{ key: 'sx…', … }}`.",
                char_at(body, i)
            ))
            .with_span(span));
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
                .with_span(span));
            }
            annotation = Some(text.to_string());
            i = end;
        }

        // CVA-in-props: `name: { key: 'sx…', … }` — object of string-literal
        // sx lists. Keys become the union type (first = default); values lower
        // via the same sx atom pipeline as host `sx=`. No separate `= default`.
        if let Some(ann) = annotation.as_deref() {
            if is_sx_cva_table(ann) {
                let keys: Vec<String> = parse_object_fields(
                    ann.trim().trim_start_matches('{').trim_end_matches('}'),
                )
                .into_iter()
                .map(|(k, _)| k)
                .collect();
                if keys.is_empty() {
                    return Err(CompileError::new(format!(
                        "CVA prop `{name}` has no values — write `{name}: {{ key: 'sx…', … }}`."
                    ))
                    .with_span(span));
                }
                // Reject a following `= default` — first key is the default.
                i = skip_trivia(body, i, None);
                if i < bytes.len()
                    && bytes[i] == b'='
                    && (i + 1 >= bytes.len() || bytes[i + 1] != b'>')
                {
                    return Err(CompileError::new(format!(
                        "CVA prop `{name}` takes its default from the first map key — \
                         drop the `= …` (first key is `{}`).",
                        keys[0]
                    ))
                    .with_span(span));
                }
                out.push(PropDecl {
                    name: name.to_string(),
                    kind: PropKind::Union(keys.clone()),
                    default: keys.first().cloned(),
                    reflect: true,
                    bindable: false,
                    description: doc,
                    span: Span::new(0, 0),
                });
                cva_axes.push(format!("{name}: {ann}"));
                continue;
            }
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
                .with_span(span));
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
                .with_span(span));
            }
            if annotation.is_none() && let Some(ty) = cast_ty {
                annotation = Some(ty.to_string());
            }
            value = Some(literal_text.to_string());
            i = end;
        }

        out.push(classify_field(name, annotation.as_deref(), value.as_deref(), doc, span)?);
    }
    let cva_css = if cva_axes.is_empty() {
        String::new()
    } else {
        variants_object_to_css(&format!("{{ {} }}", cva_axes.join(", ")))
    };
    Ok((out, cva_css))
}

/// True when `ann` is a CVA key→sx table: a `{ … }` object whose every value
/// is a **single** string literal (sx list). Distinguishes from TS object
/// types like `{ id: string }` or `{ a: 'x' | 'y' }` (union types end with a
/// quote but are not one literal).
fn is_sx_cva_table(ann: &str) -> bool {
    let t = ann.trim();
    if !(t.starts_with('{') && t.ends_with('}')) {
        return false;
    }
    let fields = parse_object_fields(&t[1..t.len() - 1]);
    if fields.is_empty() {
        return false;
    }
    fields.iter().all(|(_, v)| is_single_string_literal(v.trim()))
}

/// A complete `'…'` / `"…"` literal and nothing else (so `'x' | 'y'` is false).
fn is_single_string_literal(v: &str) -> bool {
    let bytes = v.as_bytes();
    if bytes.len() < 2 {
        return false;
    }
    let q = bytes[0];
    if q != b'\'' && q != b'"' {
        return false;
    }
    let mut i = 1usize;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i = i.saturating_add(2);
            continue;
        }
        if bytes[i] == q {
            return i + 1 == bytes.len();
        }
        i += 1;
    }
    false
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
        // Identity. Default must be explicit and a member (no first-member magic).
        if let Some(members) = union_of_string_literals(ann) {
            let default = match value {
                Some(v) => {
                    let d = unquote(v);
                    if !members.contains(&d) {
                        return Err(CompileError::new(format!(
                            "prop `{name}` default {v} is not a member of its union `{ann}`."
                        ))
                        .with_span(span));
                    }
                    d
                }
                None => {
                    return Err(CompileError::new(format!(
                        "union prop `{name}` needs an explicit default — write \
                         `{name}: {ann} = '{first}'`.",
                        first = members.first().map(String::as_str).unwrap_or_default()
                    ))
                    .with_span(span));
                }
            };
            return Ok(decl(PropKind::Union(members), Some(default), true));
        }
        return Ok(match ann {
            "string" => decl(PropKind::String, Some(value.map(unquote).unwrap_or_default()), false),
            "number" => decl(PropKind::Number, Some(value.unwrap_or("0").to_string()), false),
            "boolean" => decl(PropKind::Bool, Some(value.unwrap_or("false").to_string()), true),
            // Arbitrary authored TypeScript: a plain reactive property, never
            // an attribute. The literal default (when given) is carried verbatim.
            _ => decl(PropKind::Identity(ann.to_string()), value.map(str::to_string), false),
        });
    }

    // No annotation: the kind comes from the literal.
    let Some(v) = value else {
        return Err(CompileError::new(format!(
            "prop `{name}` declares neither a type nor a default — write \
             `{name}: Type`, `{name} = literal`, or `{name}: Type = literal`."
        ))
        .with_span(span));
    };
    // tone/size are the framework channels: a bare vocabulary default types
    // as the full channel union.
    if name == "tone" || name == "size" {
        let vocab = if name == "tone" { TONE_VOCAB } else { SIZE_VOCAB };
        let d = unquote(v);
        if !vocab.split('|').any(|m| m == d) {
            return Err(CompileError::new(format!(
                "`{name}` is the framework {name} channel — its default must be one of \
                 `{vocab}` (got {v}). For a custom union, annotate it: \
                 `{name}: 'a' | 'b' = 'a'`."
            ))
            .with_span(span));
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
    .with_span(span))
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

// ── variants ─────────────────────────────────────────────────────────────────

/// Each axis in the CVA table becomes a reflected union prop. The first key is
/// the default — the table is ordered data, its order IS the declaration.
/// `compound:` rows contribute CSS only, never a prop.
///
/// Members are newline- or comma-separated (field style, like the rest of the
/// block). Returns the derived props plus a comma-normalized copy of the table
/// for the CVA CSS core, which expects object-literal syntax.
fn parse_variants_table(
    table_inner: &str,
    span: Span,
) -> Result<(Vec<PropDecl>, String), CompileError> {
    let bytes = table_inner.as_bytes();
    let mut props = Vec::new();
    let mut normalized: Vec<String> = Vec::new();
    let mut i = 0usize;
    while i < bytes.len() {
        i = skip_trivia(table_inner, i, None);
        if i >= bytes.len() {
            break;
        }
        if bytes[i] == b',' || bytes[i] == b';' {
            i += 1;
            continue;
        }
        let (axis, after) = read_word(table_inner, i);
        if axis.is_empty() {
            return Err(CompileError::new(format!(
                "unexpected `{}` in `variants:` — axes are `name: {{ value: 'sx…', … }}`.",
                char_at(table_inner, i)
            ))
            .with_span(span));
        }
        let axis = axis.to_string();
        let mut j = skip_trivia(table_inner, after, None);
        if j >= bytes.len() || bytes[j] != b':' {
            return Err(CompileError::new(format!(
                "variants axis `{axis}` needs a table — `{axis}: {{ value: 'sx…', … }}`."
            ))
            .with_span(span));
        }
        j = skip_trivia(table_inner, j + 1, None);
        let end = scan_value_end(table_inner, j);
        let val = table_inner[j..end].trim();
        if axis == "compound" {
            normalized.push(format!("compound: {val}"));
            i = end;
            continue;
        }
        if !val.starts_with('{') {
            return Err(CompileError::new(format!(
                "variants axis `{axis}` must be a table — `{axis}: {{ value: 'sx…', … }}`."
            ))
            .with_span(span));
        }
        let keys: Vec<String> =
            parse_object_fields(val.trim_start_matches('{').trim_end_matches('}'))
                .into_iter()
                .map(|(k, _)| k)
                .collect();
        if keys.is_empty() {
            return Err(CompileError::new(format!(
                "variants axis `{axis}` has no values — a variant axis needs at least one row."
            ))
            .with_span(span));
        }
        props.push(PropDecl {
            name: axis.clone(),
            kind: PropKind::Union(keys.clone()),
            default: keys.first().cloned(),
            reflect: true,
            bindable: false,
            description: None,
            span: Span::new(0, 0),
        });
        normalized.push(format!("{axis}: {val}"));
        i = end;
    }
    Ok((props, format!("{{ {} }}", normalized.join(", "))))
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
        return parse_object_fields(&v[1..v.len() - 1]).iter().all(|(_, fv)| is_literal(fv));
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
    Err(CompileError::new("unclosed `{` in `<scope>`.".to_string()))
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
        b.props.iter().find(|p| p.name == n).unwrap_or_else(|| panic!("no prop {n}"))
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
        assert_eq!(prop(&b, "kind").kind, PropKind::Union(vec!["a".into(), "b".into()]));
        assert_eq!(prop(&b, "kind").default.as_deref(), Some("b"));
        assert!(prop(&b, "kind").reflect);
    }

    #[test]
    fn union_without_default_is_refused_with_the_fix() {
        let e = err("props: {\n  kind: 'a' | 'b'\n}");
        assert!(e.message.contains("explicit default"), "{}", e.message);
        assert!(e.message.contains("= 'a'"), "names the fix: {}", e.message);
    }

    #[test]
    fn identity_types_carry_verbatim_ts() {
        let b = block(
            "props: {\n  config: { a: 'x' | 'y' }\n  columns: Column[]\n  data: DataPoint\n  onSelect: (id: string) => void\n}",
        );
        assert_eq!(prop(&b, "config").kind, PropKind::Identity("{ a: 'x' | 'y' }".into()));
        assert_eq!(prop(&b, "columns").kind, PropKind::Identity("Column[]".into()));
        assert_eq!(prop(&b, "data").kind, PropKind::Identity("DataPoint".into()));
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
        assert_eq!(prop(&b, "center").default.as_deref(), Some("{ x: 0, y: 0 }"));
    }

    #[test]
    fn array_of_object_type_default_is_identity_not_union() {
        // `Array<{...}>` with a method-typed field: the exact shape a
        // false-union classifier would trip on (an object type argument
        // containing its own `=>` arrow). Must land as Identity, never
        // PropKind::Union, and the annotation must be captured whole.
        let b = block(
            "props: {\n  items: Array<{ id: string; onClick: () => void }> = []\n}",
        );
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
        let b = block(
            "props: {\n  items = [] as Array<{ id: string; onClick: () => void }>\n}",
        );
        assert_eq!(
            prop(&b, "items").kind,
            PropKind::Identity("Array<{ id: string; onClick: () => void }>".into())
        );
        assert_eq!(prop(&b, "items").default.as_deref(), Some("[]"));
    }

    #[test]
    fn as_cast_with_explicit_annotation_strips_the_redundant_cast() {
        let b = block(
            "props: {\n  items: Array<{ id: string }> = [] as Array<{ id: string }>\n}",
        );
        assert_eq!(prop(&b, "items").kind, PropKind::Identity("Array<{ id: string }>".into()));
        assert_eq!(prop(&b, "items").default.as_deref(), Some("[]"));
    }

    #[test]
    fn as_cast_to_a_string_union_still_classifies_as_union() {
        // The cast target can itself be a union of string literals — this
        // must still land as PropKind::Union, not Identity, proving the
        // `as`-cast path and the union-detection path compose correctly.
        let b = block("props: {\n  kind = 'a' as 'a' | 'b'\n}");
        assert_eq!(prop(&b, "kind").kind, PropKind::Union(vec!["a".into(), "b".into()]));
        assert_eq!(prop(&b, "kind").default.as_deref(), Some("a"));
    }

    #[test]
    fn tone_and_size_are_framework_channels() {
        let b = block("props: {\n  tone = 'primary'\n  size = 'md'\n}");
        match &prop(&b, "tone").kind {
            PropKind::Union(v) => assert_eq!(v.len(), 8),
            k => panic!("tone must be the channel union, got {k:?}"),
        }
        assert_eq!(prop(&b, "tone").default.as_deref(), Some("primary"));
        assert_eq!(prop(&b, "size").default.as_deref(), Some("md"));
    }

    #[test]
    fn channel_default_outside_vocab_is_refused() {
        let e = err("props: {\n  tone = 'sparkly'\n}");
        assert!(e.message.contains("channel"), "{}", e.message);
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
        assert!(e.message.contains("`<controller>`"), "{}", e.message);
        let e = err("import { x } from './y'");
        assert!(e.message.contains("not supported here"), "{}", e.message);
        let e = err("props()");
        assert!(e.message.contains("props: { … }"), "{}", e.message);
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
    fn variants_axes_become_reflected_union_props_and_css() {
        let b = block(
            "variants: {\n  variant: { solid: 'bg-muted', outline: 'border-border' }\n  size: { sm: 'text-xs', lg: 'text-base' }\n  compound: [\n    { when: { variant: 'solid' }, sx: 'shadow-xs' },\n  ]\n}",
        );
        assert_eq!(
            prop(&b, "variant").kind,
            PropKind::Union(vec!["solid".into(), "outline".into()])
        );
        assert_eq!(prop(&b, "variant").default.as_deref(), Some("solid"));
        assert_eq!(prop(&b, "size").default.as_deref(), Some("sm"));
        // compound contributes CSS only, never a prop
        assert!(b.props.iter().all(|p| p.name != "compound"));
        assert!(!b.variants_css.is_empty(), "axes must lower to CSS");
    }

    #[test]
    fn cva_in_props_axes_lower_like_variants() {
        // The only matrix form: prop value is key → sx string; type = key union;
        // first key = default; same sx pipeline as host sx=.
        let b = block(
            "props: {\n  variant: {\n    solid: 'bg-tone text-tone-contrast',\n    soft: 'bg-tone-bg text-tone-fg hover:bg-muted',\n  }\n  size: { sm: 'text-xs px-2', md: 'text-sm px-2_5' }\n  tone: 'default' | 'primary' = 'default'\n}",
        );
        assert_eq!(
            prop(&b, "variant").kind,
            PropKind::Union(vec!["solid".into(), "soft".into()])
        );
        assert_eq!(prop(&b, "variant").default.as_deref(), Some("solid"));
        assert!(prop(&b, "variant").reflect, "CVA axes reflect as attributes");
        assert_eq!(prop(&b, "size").default.as_deref(), Some("sm"));
        assert_eq!(
            prop(&b, "tone").kind,
            PropKind::Union(vec!["default".into(), "primary".into()])
        );
        assert!(
            b.variants_css.contains("variant='solid'")
                && b.variants_css.contains("background-color:var(--tone)")
                && b.variants_css.contains("variant='soft'")
                && b.variants_css.contains("size='sm'"),
            "CVA-in-props must lower through sx atoms to attr-keyed CSS: {}",
            b.variants_css
        );
        // hover state prefix on a map value
        assert!(
            b.variants_css.contains(":hover") || b.variants_css.contains("hover"),
            "state prefixes on map values must lower: {}",
            b.variants_css
        );
    }

    #[test]
    fn doc_comments_reach_prop_descriptions() {
        let b = block("props: {\n  /** The visible label. */\n  label = ''\n}");
        assert_eq!(prop(&b, "label").description.as_deref(), Some("The visible label."));
    }

    #[test]
    fn quoted_kebab_names_are_fields() {
        let b = block("props: {\n  'line-numbers' = true\n}");
        assert_eq!(prop(&b, "line-numbers").kind, PropKind::Bool);
    }

    #[test]
    fn empty_scope_is_valid() {
        let b = block("");
        assert!(b.props.is_empty() && b.types.is_empty() && b.variants_css.is_empty());
    }
}
