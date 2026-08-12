//! In-file `<partial>`: extract declarations, expand invocations at preprocess
//! time (SPEC-PARTIAL-1). Runtime never sees partials — the lowered tree is
//! identical to hand-written markup after param substitution + slot projection.
//!
//! Expansion is textual and by-reference: a param read N times re-evaluates the
//! invocation expression N times. Cycles are refused. Tag-position `style(…)`
//! on an invocation is banned (partial is not a host node).

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;

use crate::diagnostics::{CompileError, Span};
use crate::emit::util::camel_of;

/// One top-level `<partial kebab-name props: TypeName>…</partial>` declaration.
#[derive(Debug, Clone)]
pub struct PartialDecl {
    pub name: String,
    pub props_type: String,
    pub body: String,
    /// Byte span of the full declaration in the source this was extracted from.
    pub span: Span,
    /// Span of the open-tag name token (for collision / header errors).
    pub name_span: Span,
}

/// Extract top-level partial declarations (does not mutate source).
pub fn extract_partials(source: &str) -> Result<Vec<PartialDecl>, CompileError> {
    let mut decls = Vec::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    for hit in find_top_level_partial_decls(source)? {
        if !seen.insert(hit.name.clone()) {
            return Err(CompileError::new(format!(
                "duplicate partial `{}` — each name may be declared once per file",
                hit.name
            ))
            .with_span(hit.name_span)
            .with_category("sail-partial-duplicate"));
        }
        decls.push(hit);
    }
    // Nested declarations (inside elements / other partials) are errors.
    refuse_nested_partial_decls(source, &decls)?;
    Ok(decls)
}

/// Expand every partial: strip declarations, inline invocations (with cycle
/// stack), substitute params textually, project children at `<slot/>`.
pub fn expand_partials(source: &str) -> Result<String, CompileError> {
    let decls = extract_partials(source)?;
    if decls.is_empty() {
        // Still refuse nested decls and style-on-invocation of unknown tags? No
        // decls → nothing to expand. Nested bare `<partial` is still illegal.
        refuse_any_partial_tag_if_misplaced(source)?;
        return Ok(source.to_string());
    }
    let by_name: HashMap<String, PartialDecl> =
        decls.iter().map(|d| (d.name.clone(), d.clone())).collect();

    // Remove declarations first (replace with whitespace to keep spans stable
    // for diagnostics that still reference author offsets in residual checks).
    let mut s = source.to_string();
    // Remove from end so earlier offsets stay valid while blanking.
    let mut spans: Vec<(usize, usize)> = decls.iter().map(|d| (d.span.start, d.span.end)).collect();
    spans.sort_by_key(|(a, _)| *a);
    spans.reverse();
    for (start, end) in spans {
        s = blank_range(&s, start, end);
    }

    expand_invocations_in(&s, &by_name, &mut Vec::new())
}

/// Check partials against controller leakage, props type fields, and tag
/// collision with `src/<tag>/<tag>.sail`. Called from check with the author
/// source (before or after expand — uses its own extract).
pub fn check_partials(
    source: &str,
    controller_names: &BTreeSet<String>,
    scope_const_names: &BTreeSet<String>,
    source_path: Option<&Path>,
) -> Vec<CompileError> {
    let mut errs = Vec::new();
    let decls = match extract_partials(source) {
        Ok(d) => d,
        Err(e) => return vec![e],
    };
    if decls.is_empty() {
        return errs;
    }

    // Type field tables from `<scope>` type aliases.
    let type_fields = parse_scope_type_fields(source);

    for decl in &decls {
        // Leaked controller idents in body (params + scope consts OK).
        let params = type_fields
            .get(decl.props_type.as_str())
            .map(|f| f.iter().map(|x| x.name.clone()).collect::<BTreeSet<_>>())
            .unwrap_or_default();
        for ident in free_idents_in_template(&decl.body) {
            if params.contains(&ident) || scope_const_names.contains(&ident) {
                continue;
            }
            if is_template_builtin(&ident) {
                continue;
            }
            if controller_names.contains(&ident) {
                errs.push(
                    CompileError::new(format!(
                        "partial `{}` body reads controller binding `{ident}` — \
                         partials are pure functions of their params and scope \
                         constants; pass `{ident}` as a param instead",
                        decl.name
                    ))
                    .with_span(decl.span)
                    .with_category("sail-partial-leaked-ident"),
                );
            }
        }

        // Tag collision: partial name also a component file.
        if let Some(path) = source_path
            && let Some(comp) = find_component_path(path, &decl.name)
        {
            errs.push(
                CompileError::new(format!(
                    "partial `{}` collides with component file `{}` — rename the \
                     partial or the component; no shadowing either direction",
                    decl.name,
                    comp.display()
                ))
                .with_span(decl.name_span)
                .with_category("sail-partial-tag-collision"),
            );
        }
    }

    // Invocation-site checks (missing/excess attrs, style ban) against the
    // source that still has invocations + decls (author source).
    if let Err(e) = walk_invocations(source, &decls, &type_fields, |inv, decl, fields| {
        check_invocation(inv, decl, fields)
    }) {
        errs.push(e);
    }

    errs
}

// ── extraction ─────────────────────────────────────────────────────────────

struct OpenTag {
    /// Absolute index of `<`.
    start: usize,
    /// Absolute index just past `>`.
    end: usize,
    tag: String,
    /// Raw attribute region between tag name and `>`.
    attrs_raw: String,
    self_closing: bool,
}

fn find_top_level_partial_decls(source: &str) -> Result<Vec<PartialDecl>, CompileError> {
    let mut out = Vec::new();
    let bytes = source.as_bytes();
    let mut i = 0usize;
    let mut depth: i32 = 0;
    while i < bytes.len() {
        if bytes[i..].starts_with(b"<!--") {
            i = bytes[i..]
                .windows(3)
                .position(|w| w == b"-->")
                .map(|e| i + e + 3)
                .unwrap_or(bytes.len());
            continue;
        }
        if bytes[i] != b'<' {
            i = next_char_boundary(bytes, i + 1);
            continue;
        }
        if i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            // Close tag — decrease depth for non-skipped.
            let name = read_tag_name(source, i + 2);
            let end = skip_to_gt(source, i + 2 + name.len()).unwrap_or(bytes.len());
            if !is_document_block(&name) && depth > 0 {
                depth -= 1;
            }
            i = end;
            continue;
        }
        if i + 1 >= bytes.len() || !bytes[i + 1].is_ascii_alphabetic() {
            i = next_char_boundary(bytes, i + 1);
            continue;
        }
        let Some(open) = parse_open_tag(source, i) else {
            i += 1;
            continue;
        };
        if open.tag == "partial" {
            if depth != 0 {
                return Err(CompileError::new(
                    "`<partial>` may only be declared at the top level of a file \
                     (sibling of `<scope>` / `<controller>`), not inside an element \
                     or another partial",
                )
                .with_span(Span::new(open.start, open.end))
                .with_category("sail-partial-nested-decl"));
            }
            let header = parse_partial_header(&open)?;
            let (body, block_end) = if open.self_closing {
                (String::new(), open.end)
            } else {
                let close = matching_close(source, open.end, "partial").ok_or_else(|| {
                    CompileError::new("unclosed `<partial>`")
                        .with_span(Span::new(open.start, open.end))
                })?;
                let body = source[open.end..close - "</partial>".len()].to_string();
                (body, close)
            };
            out.push(PartialDecl {
                name: header.name,
                props_type: header.props_type,
                body,
                span: Span::new(open.start, block_end),
                name_span: header.name_span,
            });
            i = block_end;
            continue;
        }
        if is_document_block(&open.tag) {
            // Skip whole block body without affecting depth.
            if open.self_closing {
                i = open.end;
            } else if let Some(end) = matching_close(source, open.end, &open.tag) {
                i = end;
            } else {
                i = open.end;
            }
            continue;
        }
        if open.self_closing {
            i = open.end;
            continue;
        }
        depth += 1;
        i = open.end;
    }
    Ok(out)
}

fn refuse_nested_partial_decls(source: &str, top: &[PartialDecl]) -> Result<(), CompileError> {
    let top_spans: Vec<_> = top.iter().map(|d| d.span).collect();
    let bytes = source.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i..].starts_with(b"<!--") {
            i = bytes[i..]
                .windows(3)
                .position(|w| w == b"-->")
                .map(|e| i + e + 3)
                .unwrap_or(bytes.len());
            continue;
        }
        if bytes[i..].starts_with(b"<partial") {
            let boundary = bytes
                .get(i + "<partial".len())
                .is_some_and(|&b| b.is_ascii_whitespace() || b == b'/' || b == b'>');
            if boundary && !top_spans.iter().any(|s| s.start == i) {
                let in_any = top_spans.iter().any(|s| i > s.start && i < s.end);
                if in_any {
                    return Err(CompileError::new(
                        "`<partial>` may only be declared at the top level — \
                         not inside another partial's body",
                    )
                    .with_span(Span::point(i))
                    .with_category("sail-partial-nested-decl"));
                }
            }
        }
        i = next_char_boundary(bytes, i + 1);
    }
    Ok(())
}

fn next_char_boundary(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() && (bytes[i] & 0xC0) == 0x80 {
        i += 1;
    }
    i
}

fn refuse_any_partial_tag_if_misplaced(source: &str) -> Result<(), CompileError> {
    // When there are zero top-level decls, any `<partial` is misplaced.
    if let Some(at) = find_outside_comments(source, "<partial") {
        return Err(CompileError::new(
            "`<partial>` may only be declared at the top level of a file \
             (sibling of `<scope>` / `<controller>`)",
        )
        .with_span(Span::point(at))
        .with_category("sail-partial-nested-decl"));
    }
    Ok(())
}

struct PartialHeader {
    name: String,
    props_type: String,
    name_span: Span,
}

/// Header shape: `<partial kebab-name props: TypeName …>`
fn parse_partial_header(open: &OpenTag) -> Result<PartialHeader, CompileError> {
    let raw = open.attrs_raw.trim();
    // Expected: `kebab-name props: TypeName` with optional trailing junk refused.
    let bytes = raw.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    let name_start = i;
    while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'-') {
        i += 1;
    }
    if name_start == i {
        return Err(CompileError::new(
            "`<partial>` requires a kebab-case name: `<partial list-tile props: Props>`",
        )
        .with_span(Span::new(open.start, open.end))
        .with_category("sail-partial-header"));
    }
    let name = raw[name_start..i].to_string();
    if !name.contains('-') {
        return Err(CompileError::new(format!(
            "partial name `{name}` must be kebab-case with a hyphen \
             (custom-element namespace), e.g. `list-tile`"
        ))
        .with_span(Span::new(open.start, open.end))
        .with_category("sail-partial-header"));
    }
    // Absolute name span: open.start + 1 + "partial".len() + whitespace in attrs…
    // attrs_raw is the region after the tag name; name is at the start of attrs_raw.
    let attrs_abs = open.start + 1 + "partial".len();
    // Skip whitespace between tag and attrs in original — use attrs position from parse.
    let name_span = Span::new(
        // Approximate: search name token after `<partial`
        find_substr_from(open, &name).unwrap_or(open.start),
        find_substr_from(open, &name).unwrap_or(open.start) + name.len(),
    );
    let _ = (attrs_abs, name_start); // kept for clarity

    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    let rest = &raw[i..];
    let props_type = if let Some(after) = rest.strip_prefix("props:") {
        let t = after.trim();
        let ty: String = t
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '$')
            .collect();
        if ty.is_empty() {
            return Err(CompileError::new(
                "`props:` on `<partial>` needs a type name defined in `<scope>` \
                 (e.g. `props: ListTileProps`)",
            )
            .with_span(Span::new(open.start, open.end))
            .with_category("sail-partial-header"));
        }
        let after_ty = t[ty.len()..].trim();
        if !after_ty.is_empty() {
            return Err(CompileError::new(format!(
                "unexpected tokens after partial props type: `{after_ty}`"
            ))
            .with_span(Span::new(open.start, open.end))
            .with_category("sail-partial-header"));
        }
        ty
    } else {
        return Err(CompileError::new(
            "`<partial>` requires `props: TypeName` — params are a plain scope type, \
             e.g. `<partial list-tile props: ListTileProps>`",
        )
        .with_span(Span::new(open.start, open.end))
        .with_category("sail-partial-header"));
    };

    Ok(PartialHeader {
        name,
        props_type,
        name_span,
    })
}

fn find_substr_from(open: &OpenTag, name: &str) -> Option<usize> {
    // Search within open tag for the name token as a whole word.
    let slice = &open.attrs_raw;
    if let Some(rel) = slice.find(name) {
        // attrs_raw is not absolute — recover by finding open tag text.
        // We don't have source here; name_span is best-effort for errors.
        // Use open.start + offset of name inside reconstructed open.
        let tag_head = format!("<partial");
        let _ = tag_head;
        // Caller only uses this for diagnostics; point at open tag if unsure.
        Some(open.start + 1 + "partial".len() + 1 + rel)
    } else {
        None
    }
}

// ── expansion ──────────────────────────────────────────────────────────────

fn expand_invocations_in(
    source: &str,
    decls: &HashMap<String, PartialDecl>,
    stack: &mut Vec<String>,
) -> Result<String, CompileError> {
    let bytes = source.as_bytes();
    let mut out = String::with_capacity(source.len());
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i..].starts_with(b"<!--") {
            let end = bytes[i..]
                .windows(3)
                .position(|w| w == b"-->")
                .map(|e| i + e + 3)
                .unwrap_or(bytes.len());
            out.push_str(&source[i..end]);
            i = end;
            continue;
        }
        // Pass document blocks through untouched (no partial expand inside).
        if let Some(tag) = ["style", "scope", "controller", "script"]
            .into_iter()
            .find(|t| bytes[i..].starts_with(format!("<{t}").as_bytes()))
        {
            let end = if let Some(gt) = skip_to_gt(source, i) {
                if source[gt.saturating_sub(2)..gt].contains('/') {
                    gt // self-close unlikely
                } else {
                    matching_close(source, gt, tag).unwrap_or(gt)
                }
            } else {
                bytes.len()
            };
            out.push_str(&source[i..end]);
            i = end;
            continue;
        }
        if bytes[i] == b'<' && i + 1 < bytes.len() && bytes[i + 1].is_ascii_alphabetic() {
            let Some(open) = parse_open_tag(source, i) else {
                let n = next_char_boundary(bytes, i + 1);
                out.push_str(&source[i..n]);
                i = n;
                continue;
            };
            if let Some(decl) = decls.get(&open.tag) {
                // Ban tag-position style(...) on the invocation open tag.
                if open.attrs_raw.contains("style(") {
                    return Err(CompileError::new(format!(
                        "tag-position `style(…)` is not allowed on partial `<{}>` — \
                         a partial is not a node (it may expand to multiple roots). \
                         Paint nodes inside the partial body, wrap the invocation in \
                         a real element, or declare a `SailStyleChain` param and pass \
                         style explicitly",
                        decl.name
                    ))
                    .with_span(Span::new(open.start, open.end))
                    .with_category("sail-partial-style-on-invocation"));
                }
                if stack.iter().any(|n| n == &decl.name) {
                    let chain = stack
                        .iter()
                        .chain(std::iter::once(&decl.name))
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(" → ");
                    return Err(CompileError::new(format!(
                        "partials do not recurse; a recursive structure is a component \
                         file (cycle: {chain})"
                    ))
                    .with_span(Span::new(open.start, open.end))
                    .with_category("sail-partial-cycle"));
                }
                let (children, inv_end) = if open.self_closing {
                    (String::new(), open.end)
                } else {
                    let close = matching_close(source, open.end, &open.tag).ok_or_else(|| {
                        CompileError::new(format!("unclosed partial invocation `<{}>`", open.tag))
                            .with_span(Span::new(open.start, open.end))
                    })?;
                    let close_tag = format!("</{}>", open.tag);
                    let body = source[open.end..close - close_tag.len()].to_string();
                    (body, close)
                };
                let attrs = parse_attr_list(&open.attrs_raw);
                let param_map = build_param_map(&attrs);
                let mut body = decl.body.clone();
                // Slot projection
                let has_slot = body_has_default_slot(&body);
                if !children.trim().is_empty() && !has_slot {
                    return Err(CompileError::new(format!(
                        "partial `{}` was invoked with children but its body has no \
                         `<slot/>` — children would be dropped; add a default `<slot/>` \
                         or remove the children",
                        decl.name
                    ))
                    .with_span(Span::new(open.start, open.end))
                    .with_category("sail-partial-slot"));
                }
                if has_slot {
                    body = project_default_slot(&body, &children);
                }
                body = substitute_params(&body, &param_map);
                stack.push(decl.name.clone());
                let expanded = expand_invocations_in(&body, decls, stack)?;
                stack.pop();
                out.push_str(&expanded);
                i = inv_end;
                continue;
            }
            // Not a partial — copy open tag and continue (children scanned next).
            out.push_str(&source[i..open.end]);
            i = open.end;
            continue;
        }
        // UTF-8 safe: copy full char.
        let n = next_char_boundary(bytes, i + 1);
        out.push_str(&source[i..n]);
        i = n;
    }
    Ok(out)
}

// ── invocation checks (author source) ──────────────────────────────────────

struct Invocation<'a> {
    open: OpenTag,
    #[allow(dead_code)]
    children: &'a str,
    attrs: Vec<(String, String)>,
}

fn walk_invocations(
    source: &str,
    decls: &[PartialDecl],
    type_fields: &HashMap<String, Vec<TypeField>>,
    mut f: impl FnMut(&Invocation<'_>, &PartialDecl, Option<&[TypeField]>) -> Result<(), CompileError>,
) -> Result<(), CompileError> {
    let by_name: HashMap<&str, &PartialDecl> = decls.iter().map(|d| (d.name.as_str(), d)).collect();
    let bytes = source.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i..].starts_with(b"<!--") {
            i = bytes[i..]
                .windows(3)
                .position(|w| w == b"-->")
                .map(|e| i + e + 3)
                .unwrap_or(bytes.len());
            continue;
        }
        if bytes[i] == b'<' && i + 1 < bytes.len() && bytes[i + 1].is_ascii_alphabetic() {
            let Some(open) = parse_open_tag(source, i) else {
                i = next_char_boundary(bytes, i + 1);
                continue;
            };
            // Skip declaration opens.
            if open.tag == "partial" {
                i = if open.self_closing {
                    open.end
                } else {
                    matching_close(source, open.end, "partial").unwrap_or(open.end)
                };
                continue;
            }
            if let Some(decl) = by_name.get(open.tag.as_str()) {
                if open.attrs_raw.contains("style(") {
                    return Err(CompileError::new(format!(
                        "tag-position `style(…)` is not allowed on partial `<{}>` — \
                         a partial is not a node (it may expand to multiple roots). \
                         Paint nodes inside the partial body, wrap the invocation in \
                         a real element, or declare a `SailStyleChain` param and pass \
                         style explicitly",
                        decl.name
                    ))
                    .with_span(Span::new(open.start, open.end))
                    .with_category("sail-partial-style-on-invocation"));
                }
                let (children, inv_end) = if open.self_closing {
                    ("", open.end)
                } else {
                    let close = matching_close(source, open.end, &open.tag).unwrap_or(open.end);
                    let c =
                        &source[open.end..close.saturating_sub(format!("</{}>", open.tag).len())];
                    (c, close)
                };
                let attrs = parse_attr_list(&open.attrs_raw);
                let fields = type_fields
                    .get(decl.props_type.as_str())
                    .map(|v| v.as_slice());
                f(
                    &Invocation {
                        open,
                        children,
                        attrs,
                    },
                    decl,
                    fields,
                )?;
                i = inv_end;
                continue;
            }
            i = open.end;
            continue;
        }
        i = next_char_boundary(bytes, i + 1);
    }
    Ok(())
}

fn check_invocation(
    inv: &Invocation<'_>,
    decl: &PartialDecl,
    fields: Option<&[TypeField]>,
) -> Result<(), CompileError> {
    let Some(fields) = fields else {
        // Unknown type — still allow (tsgo/typecheck may catch); no field table.
        return Ok(());
    };
    let provided: BTreeSet<String> = inv
        .attrs
        .iter()
        .map(|(n, _)| camel_of(n))
        .filter(|n| n != "key") // key is framework, not a param
        .collect();
    for f in fields {
        if !f.optional && !provided.contains(&f.name) {
            return Err(CompileError::new(format!(
                "partial `{}` missing required param `{}` (from type `{}`)",
                decl.name, f.name, decl.props_type
            ))
            .with_span(Span::new(inv.open.start, inv.open.end))
            .with_category("sail-partial-missing-attr"));
        }
    }
    let known: BTreeSet<&str> = fields.iter().map(|f| f.name.as_str()).collect();
    for p in &provided {
        if !known.contains(p.as_str()) {
            return Err(CompileError::new(format!(
                "partial `{}` has unknown param `{}` (not in type `{}`)",
                decl.name, p, decl.props_type
            ))
            .with_span(Span::new(inv.open.start, inv.open.end))
            .with_category("sail-partial-excess-attr"));
        }
    }
    Ok(())
}

// ── type fields from scope ─────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct TypeField {
    pub name: String,
    pub optional: bool,
    /// Raw type text (for hover).
    pub ty: String,
}

/// Parse `type Name = { field: T; opt?: U }` aliases from `<scope>` bodies.
pub fn parse_scope_type_fields(source: &str) -> HashMap<String, Vec<TypeField>> {
    let mut out = HashMap::new();
    for body in scope_bodies(source) {
        harvest_type_aliases(&body, &mut out);
    }
    out
}

fn scope_bodies(source: &str) -> Vec<String> {
    let mut bodies = Vec::new();
    let mut i = 0usize;
    while let Some(rel) = source[i..].find("<scope") {
        let at = i + rel;
        let Some(gt) = skip_to_gt(source, at) else {
            break;
        };
        let Some(end) = matching_close(source, gt, "scope") else {
            break;
        };
        let body = &source[gt..end - "</scope>".len()];
        bodies.push(body.to_string());
        i = end;
    }
    bodies
}

fn harvest_type_aliases(body: &str, out: &mut HashMap<String, Vec<TypeField>>) {
    let bytes = body.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        // type Name = { … }  or export type Name = { … }
        if bytes[i..].starts_with(b"export") {
            i += 6;
            continue;
        }
        if bytes[i..].starts_with(b"type")
            && bytes.get(i + 4).is_some_and(|b| b.is_ascii_whitespace())
        {
            let after = i + 4;
            let mut j = after;
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            let name_start = j;
            while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') {
                j += 1;
            }
            if name_start == j {
                i = next_char_boundary(bytes, i + 1);
                continue;
            }
            let name = body[name_start..j].to_string();
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            if j >= bytes.len() || bytes[j] != b'=' {
                i = j;
                continue;
            }
            j += 1;
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b'{' {
                if let Some(close) = find_matching_brace(body, j) {
                    let inner = &body[j + 1..close];
                    out.insert(name, parse_object_type_fields(inner));
                    i = close + 1;
                    continue;
                }
            }
            i = j;
            continue;
        }
        i = next_char_boundary(bytes, i + 1);
    }
}

fn parse_object_type_fields(inner: &str) -> Vec<TypeField> {
    let mut fields = Vec::new();
    // Split on `;` and newlines, crude but good enough for plain object types.
    for part in inner.split(|c| c == ';' || c == '\n') {
        let part = part.trim();
        if part.is_empty() || part.starts_with("//") || part.starts_with('*') {
            continue;
        }
        // name?: Type  or  name: Type
        let bytes = part.as_bytes();
        let mut i = 0usize;
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        let ns = i;
        while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
            i += 1;
        }
        if ns == i {
            continue;
        }
        let name = part[ns..i].to_string();
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        let optional = if i < bytes.len() && bytes[i] == b'?' {
            i += 1;
            true
        } else {
            false
        };
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() || bytes[i] != b':' {
            continue;
        }
        i += 1;
        let ty = part[i..].trim().to_string();
        if ty.is_empty() {
            continue;
        }
        fields.push(TypeField { name, optional, ty });
    }
    fields
}

// ── param substitution ─────────────────────────────────────────────────────

fn build_param_map(attrs: &[(String, String)]) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for (name, value) in attrs {
        if name == "key" {
            continue;
        }
        let camel = camel_of(name);
        map.insert(camel, attr_value_to_expr(value));
    }
    map
}

/// Invocation attr value → expression text substituted for bare param reads.
/// After `quote_brace_attrs`, dynamic attrs look like `"{expr}"` (quoted).
fn attr_value_to_expr(value: &str) -> String {
    let v = value.trim();
    if v.is_empty() {
        return "true".to_string();
    }
    // Bare `{expr}` (pre-quote or self-close path).
    if v.starts_with('{') && v.ends_with('}') && v.len() >= 2 {
        return v[1..v.len() - 1].trim().to_string();
    }
    // Quoted value — strip quotes; body may still be `{expr}`.
    let inner = if (v.starts_with('"') && v.ends_with('"') && v.len() >= 2)
        || (v.starts_with('\'') && v.ends_with('\'') && v.len() >= 2)
    {
        v[1..v.len() - 1]
            .replace("&quot;", "\"")
            .replace("&#39;", "'")
    } else {
        v.to_string()
    };
    let t = inner.trim();
    if t.starts_with('{') && t.ends_with('}') && t.len() >= 2 {
        return t[1..t.len() - 1].trim().to_string();
    }
    // Static string
    format!("\"{}\"", t.replace('\\', "\\\\").replace('"', "\\\""))
}

/// Replace bare param identifiers **inside brace expressions only**
/// (`{title}`, `token={token}`, `disabled={!disabled}`). Attribute names
/// like `token-row` must not be touched (SPEC: param *reads*, not markup).
fn substitute_params(body: &str, params: &BTreeMap<String, String>) -> String {
    if params.is_empty() {
        return body.to_string();
    }
    let bytes = body.as_bytes();
    let mut out = String::with_capacity(body.len());
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i..].starts_with(b"<!--") {
            let end = bytes[i..]
                .windows(3)
                .position(|w| w == b"-->")
                .map(|e| i + e + 3)
                .unwrap_or(bytes.len());
            out.push_str(&body[i..end]);
            i = end;
            continue;
        }
        if bytes[i] == b'{' {
            if let Some(close) = find_matching_brace(body, i) {
                let expr = body[i + 1..close].trim();
                // Whole-expression param: `{title}` / `{token}`.
                // Static string args expand to a quoted attr value (no braces)
                // so `token={token}` + `token="bg"` → `token="bg"`, not
                // `token={"bg"}` (which quote_brace_attrs mangles).
                if is_ident(expr) {
                    if let Some(val) = params.get(expr) {
                        if is_js_string_literal(val) {
                            out.push_str(val);
                        } else {
                            out.push('{');
                            out.push_str(val);
                            out.push('}');
                        }
                        i = close + 1;
                        continue;
                    }
                }
                let rewritten = substitute_idents_in_expr(expr, params);
                out.push('{');
                out.push_str(&rewritten);
                out.push('}');
                i = close + 1;
                continue;
            }
        }
        let n = next_char_boundary(bytes, i + 1);
        out.push_str(&body[i..n]);
        i = n;
    }
    out
}

fn substitute_idents_in_expr(expr: &str, params: &BTreeMap<String, String>) -> String {
    let bytes = expr.as_bytes();
    let mut out = String::with_capacity(expr.len());
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'"' || bytes[i] == b'\'' {
            let q = bytes[i];
            let start = i;
            i += 1;
            while i < bytes.len() {
                if bytes[i] == q && bytes.get(i.wrapping_sub(1)) != Some(&b'\\') {
                    i += 1;
                    break;
                }
                i = next_char_boundary(bytes, i + 1);
            }
            out.push_str(&expr[start..i]);
            continue;
        }
        if is_ident_start(bytes[i]) {
            let start = i;
            i += 1;
            while i < bytes.len() && is_ident_cont(bytes[i]) {
                i += 1;
            }
            let ident = &expr[start..i];
            let prev_non_ws = out
                .as_bytes()
                .iter()
                .rev()
                .find(|b| !b.is_ascii_whitespace());
            if prev_non_ws == Some(&b'.') {
                out.push_str(ident);
                continue;
            }
            if let Some(repl) = params.get(ident) {
                out.push_str(repl);
            } else {
                out.push_str(ident);
            }
            continue;
        }
        let n = next_char_boundary(bytes, i + 1);
        out.push_str(&expr[i..n]);
        i = n;
    }
    out
}

// ── slots ──────────────────────────────────────────────────────────────────

fn body_has_default_slot(body: &str) -> bool {
    // Default slot: `<slot/>`, `<slot>`, `<slot >`, not `name=`
    let bytes = body.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i..].starts_with(b"<slot") {
            let after = i + 5;
            let boundary = bytes
                .get(after)
                .is_some_and(|b| b.is_ascii_whitespace() || *b == b'>' || *b == b'/');
            if boundary {
                // Read open tag
                if let Some(gt) = skip_to_gt(body, i) {
                    let open = &body[i..gt];
                    if !open.contains("name=") && !open.contains("name =") {
                        return true;
                    }
                }
            }
        }
        i = next_char_boundary(bytes, i + 1);
    }
    false
}

fn project_default_slot(body: &str, children: &str) -> String {
    let bytes = body.as_bytes();
    let mut out = String::with_capacity(body.len() + children.len());
    let mut i = 0usize;
    let mut replaced = false;
    while i < bytes.len() {
        if bytes[i..].starts_with(b"<slot") {
            let after = i + 5;
            let boundary = bytes
                .get(after)
                .is_some_and(|b| b.is_ascii_whitespace() || *b == b'>' || *b == b'/');
            if boundary {
                if let Some(open) = parse_open_tag(body, i) {
                    let is_default =
                        !open.attrs_raw.contains("name=") && !open.attrs_raw.contains("name =");
                    if is_default && !replaced {
                        let end = if open.self_closing {
                            open.end
                        } else {
                            matching_close(body, open.end, "slot").unwrap_or(open.end)
                        };
                        if children.trim().is_empty() {
                            if !open.self_closing {
                                let fallback = &body[open.end..end.saturating_sub("</slot>".len())];
                                out.push_str(fallback);
                            }
                        } else {
                            out.push_str(children);
                        }
                        replaced = true;
                        i = end;
                        continue;
                    }
                }
            }
        }
        let n = next_char_boundary(bytes, i + 1);
        out.push_str(&body[i..n]);
        i = n;
    }
    out
}

// ── free idents / leakage ──────────────────────────────────────────────────

fn free_idents_in_template(body: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    // Collect idents inside `{…}` interpolations and attr values.
    let bytes = body.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i..].starts_with(b"<!--") {
            i = bytes[i..]
                .windows(3)
                .position(|w| w == b"-->")
                .map(|e| i + e + 3)
                .unwrap_or(bytes.len());
            continue;
        }
        if bytes[i] == b'{' {
            if let Some(end) = find_matching_brace(body, i) {
                let expr = &body[i + 1..end];
                for id in crate::parse::expr::idents::collect_idents(expr, false) {
                    out.insert(id);
                }
                i = end + 1;
                continue;
            }
        }
        i = next_char_boundary(bytes, i + 1);
    }
    out
}

fn is_template_builtin(name: &str) -> bool {
    matches!(
        name,
        "true"
            | "false"
            | "null"
            | "undefined"
            | "NaN"
            | "Infinity"
            | "style"
            | "match"
            | "when"
            | "Math"
            | "String"
            | "Number"
            | "Boolean"
            | "Array"
            | "Object"
            | "JSON"
            | "console"
            | "gap"
            | "pad"
            | "padX"
            | "padY"
            | "bg"
            | "fg"
            | "radius"
            | "border"
            | "borderWidth"
            | "fontSize"
            | "fontWeight"
            | "fontFamily"
            | "opacity"
            | "overflow"
            | "display"
            | "grow"
            | "shrink"
            | "minW"
            | "minH"
            | "maxW"
            | "maxH"
            | "w"
            | "h"
            | "size"
            | "truncate"
            | "position"
            | "cursor"
            | "shadow"
            | "leading"
            | "tracking"
            | "textCase"
            | "textAlign"
            | "nums"
            | "full"
            | "box"
            | "paint"
            | "text"
            | "animate"
    )
}

// ── component collision ────────────────────────────────────────────────────

fn find_component_path(source_path: &Path, tag: &str) -> Option<std::path::PathBuf> {
    for anc in source_path.ancestors() {
        if anc.file_name().and_then(|s| s.to_str()) == Some("src") {
            let candidate = anc.join(tag).join(format!("{tag}.sail"));
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

// ── scope const names (style name = / const name = style) ──────────────────

/// Harvest compile-time scope constant names (Style decls) from author source.
pub fn scope_const_names(source: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for body in scope_bodies(source) {
        harvest_style_names(&body, &mut names);
    }
    names
}

fn harvest_style_names(body: &str, names: &mut BTreeSet<String>) {
    let bytes = body.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        // `style name =` or `const name =` — byte compares only (UTF-8 safe).
        if bytes[i..].starts_with(b"style")
            && bytes.get(i + 5).is_some_and(|b| b.is_ascii_whitespace())
        {
            let mut j = i + 5;
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            let ns = j;
            while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') {
                j += 1;
            }
            if ns < j {
                let name = &body[ns..j];
                if name != "style" {
                    let mut k = j;
                    while k < bytes.len() && bytes[k].is_ascii_whitespace() {
                        k += 1;
                    }
                    if k < bytes.len() && bytes[k] == b'=' {
                        names.insert(name.to_string());
                    }
                }
            }
            i = j;
            continue;
        }
        if bytes[i..].starts_with(b"const")
            && bytes.get(i + 5).is_some_and(|b| b.is_ascii_whitespace())
        {
            let mut j = i + 5;
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            let ns = j;
            while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') {
                j += 1;
            }
            if ns < j {
                names.insert(body[ns..j].to_string());
            }
            i = j.max(i + 1);
            continue;
        }
        i = next_char_boundary(bytes, i + 1);
    }
}

// ── low-level scanners ─────────────────────────────────────────────────────

fn is_document_block(tag: &str) -> bool {
    matches!(tag, "scope" | "controller" | "style" | "script" | "partial")
}

fn parse_open_tag(source: &str, at: usize) -> Option<OpenTag> {
    let bytes = source.as_bytes();
    if at >= bytes.len() || bytes[at] != b'<' {
        return None;
    }
    if at + 1 >= bytes.len() || !bytes[at + 1].is_ascii_alphabetic() {
        return None;
    }
    let tag = read_tag_name(source, at + 1);
    if tag.is_empty() {
        return None;
    }
    let after_tag = at + 1 + tag.len();
    let mut k = after_tag;
    let mut quote: Option<u8> = None;
    let mut paren = 0i32;
    let mut brace = 0i32;
    while k < bytes.len() {
        let b = bytes[k];
        match quote {
            Some(q) => {
                if b == q && bytes.get(k.wrapping_sub(1)) != Some(&b'\\') {
                    quote = None;
                }
            }
            None => match b {
                b'"' | b'\'' => quote = Some(b),
                b'(' => paren += 1,
                b')' => paren -= 1,
                b'{' => brace += 1,
                b'}' => brace -= 1,
                b'>' if paren == 0 && brace == 0 => {
                    let self_closing = k > at && bytes[k - 1] == b'/';
                    let attrs_end = if self_closing { k - 1 } else { k };
                    let attrs_raw = source[after_tag..attrs_end].to_string();
                    return Some(OpenTag {
                        start: at,
                        end: k + 1,
                        tag,
                        attrs_raw,
                        self_closing,
                    });
                }
                _ => {}
            },
        }
        k += 1;
    }
    None
}

fn read_tag_name(source: &str, from: usize) -> String {
    let bytes = source.as_bytes();
    let mut i = from;
    while i < bytes.len()
        && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'-' || bytes[i] == b'_')
    {
        i += 1;
    }
    source[from..i].to_string()
}

fn skip_to_gt(source: &str, from: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut k = from;
    let mut quote: Option<u8> = None;
    let mut paren = 0i32;
    let mut brace = 0i32;
    while k < bytes.len() {
        let b = bytes[k];
        match quote {
            Some(q) => {
                if b == q {
                    quote = None;
                }
            }
            None => match b {
                b'"' | b'\'' => quote = Some(b),
                b'(' => paren += 1,
                b')' => paren -= 1,
                b'{' => brace += 1,
                b'}' => brace -= 1,
                b'>' if paren == 0 && brace == 0 => return Some(k + 1),
                _ => {}
            },
        }
        k += 1;
    }
    None
}

fn matching_close(source: &str, body_start: usize, tag: &str) -> Option<usize> {
    let bytes = source.as_bytes();
    let open_prefix = format!("<{tag}");
    let close = format!("</{tag}>");
    let mut depth = 1usize;
    let mut scan = body_start;
    while scan < bytes.len() {
        if bytes[scan..].starts_with(close.as_bytes()) {
            depth -= 1;
            scan += close.len();
            if depth == 0 {
                return Some(scan);
            }
        } else if bytes[scan..].starts_with(open_prefix.as_bytes())
            && bytes
                .get(scan + open_prefix.len())
                .is_some_and(|&b| b.is_ascii_whitespace() || b == b'/' || b == b'>')
        {
            depth += 1;
            scan += open_prefix.len();
        } else {
            scan += 1;
        }
    }
    None
}

fn parse_attr_list(raw: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let bytes = raw.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        // Skip style(…) call form if present (banned elsewhere)
        if raw[i..].starts_with("style(") {
            let mut depth = 0i32;
            while i < bytes.len() {
                if bytes[i] == b'(' {
                    depth += 1;
                } else if bytes[i] == b')' {
                    depth -= 1;
                    i += 1;
                    if depth == 0 {
                        break;
                    }
                    continue;
                }
                i += 1;
            }
            continue;
        }
        let ns = i;
        while i < bytes.len()
            && (bytes[i].is_ascii_alphanumeric()
                || bytes[i] == b'-'
                || bytes[i] == b'_'
                || bytes[i] == b':'
                || bytes[i] == b'|')
        {
            i += 1;
        }
        if ns == i {
            i += 1;
            continue;
        }
        let name = raw[ns..i].to_string();
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i < bytes.len() && bytes[i] == b'=' {
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if i < bytes.len() && bytes[i] == b'{' {
                if let Some(end) = find_matching_brace(raw, i) {
                    out.push((name, raw[i..=end].to_string()));
                    i = end + 1;
                    continue;
                }
            }
            if i < bytes.len() && (bytes[i] == b'"' || bytes[i] == b'\'') {
                let q = bytes[i];
                let vs = i;
                i += 1;
                while i < bytes.len() && bytes[i] != q {
                    i += 1;
                }
                if i < bytes.len() {
                    i += 1;
                }
                out.push((name, raw[vs..i].to_string()));
                continue;
            }
            // bare value
            let vs = i;
            while i < bytes.len() && !bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            out.push((name, raw[vs..i].to_string()));
        } else {
            // boolean attr
            out.push((name, String::new()));
        }
    }
    out
}

fn find_matching_brace(s: &str, open: usize) -> Option<usize> {
    let bytes = s.as_bytes();
    if open >= bytes.len() || bytes[open] != b'{' {
        return None;
    }
    let mut depth = 0i32;
    let mut quote: Option<u8> = None;
    let mut i = open;
    while i < bytes.len() {
        let b = bytes[i];
        match quote {
            Some(q) => {
                if b == q && bytes.get(i.wrapping_sub(1)) != Some(&b'\\') {
                    quote = None;
                }
            }
            None => match b {
                b'"' | b'\'' => quote = Some(b),
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(i);
                    }
                }
                _ => {}
            },
        }
        i += 1;
    }
    None
}

fn blank_range(source: &str, start: usize, end: usize) -> String {
    let mut out = String::with_capacity(source.len());
    out.push_str(&source[..start]);
    for b in source[start..end].bytes() {
        out.push(if b == b'\n' { '\n' } else { ' ' });
    }
    out.push_str(&source[end..]);
    out
}

fn find_outside_comments(source: &str, needle: &str) -> Option<usize> {
    let bytes = source.as_bytes();
    let n = needle.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i..].starts_with(b"<!--") {
            i = bytes[i..]
                .windows(3)
                .position(|w| w == b"-->")
                .map(|e| i + e + 3)
                .unwrap_or(bytes.len());
            continue;
        }
        if bytes[i..].starts_with(n) {
            return Some(i);
        }
        i = next_char_boundary(bytes, i + 1);
    }
    None
}

fn is_ident_start(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'_' || b == b'$'
}

fn is_ident_cont(b: u8) -> bool {
    is_ident_start(b) || b.is_ascii_digit()
}

fn is_ident(s: &str) -> bool {
    let b = s.as_bytes();
    if b.is_empty() || !is_ident_start(b[0]) {
        return false;
    }
    b[1..].iter().all(|&c| is_ident_cont(c))
}

fn is_js_string_literal(s: &str) -> bool {
    let t = s.trim();
    (t.starts_with('"') && t.ends_with('"') && t.len() >= 2)
        || (t.starts_with('\'') && t.ends_with('\'') && t.len() >= 2)
}

// ── public helpers for LSP ─────────────────────────────────────────────────

/// File-local partial names for completion.
pub fn partial_names(source: &str) -> Vec<String> {
    extract_partials(source)
        .map(|d| d.into_iter().map(|p| p.name).collect())
        .unwrap_or_default()
}

/// Hover info for a partial name or its props type fields.
pub fn partial_hover(source: &str, name: &str) -> Option<String> {
    let decls = extract_partials(source).ok()?;
    let decl = decls.iter().find(|d| d.name == name)?;
    let fields = parse_scope_type_fields(source);
    let field_list = fields.get(&decl.props_type).map(|fs| {
        fs.iter()
            .map(|f| {
                if f.optional {
                    format!("  {}?: {}", f.name, f.ty)
                } else {
                    format!("  {}: {}", f.name, f.ty)
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    });
    Some(match field_list {
        Some(list) if !list.is_empty() => format!(
            "**partial** `{}` · props: `{}`\n```ts\n{{\n{list}\n}}\n```",
            decl.name, decl.props_type
        ),
        _ => format!("**partial** `{}` · props: `{}`", decl.name, decl.props_type),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_simple_param_and_slot() {
        let src = r#"
<scope>
type TileProps = { title: string }
</scope>
<partial list-tile props: TileProps>
  <row><text>{title}</text><slot/></row>
</partial>
<list-tile title={"Hello"}><span>kid</span></list-tile>
"#;
        let out = expand_partials(src).expect("expand");
        assert!(
            out.contains("{ \"Hello\" }")
                || out.contains("{\"Hello\"}")
                || out.contains("\"Hello\""),
            "got: {out}"
        );
        assert!(out.contains("<span>kid</span>"), "slot projected: {out}");
        assert!(!out.contains("<partial"), "decl removed: {out}");
        assert!(!out.contains("<list-tile"), "invoc expanded: {out}");
    }

    #[test]
    fn cycle_errors() {
        let src = r#"
<scope>
type A = { x: string }
type B = { y: string }
</scope>
<partial part-a props: A><part-b y={x}/></partial>
<partial part-b props: B><part-a x={y}/></partial>
<part-a x={"z"}/>
"#;
        let err = expand_partials(src).expect_err("cycle");
        assert!(
            err.message.contains("do not recurse") || err.message.contains("cycle"),
            "{}",
            err.message
        );
    }

    #[test]
    fn style_on_invocation_banned() {
        let src = r#"
<scope>
type T = { title: string }
</scope>
<partial list-tile props: T><text>{title}</text></partial>
<list-tile title={"x"} style({ paint: bg(primary) })/>
"#;
        let err = expand_partials(src).expect_err("style ban");
        assert!(
            err.message.contains("style") && err.message.contains("not a node"),
            "{}",
            err.message
        );
    }

    #[test]
    fn children_without_slot_error() {
        let src = r#"
<scope>
type T = { title: string }
</scope>
<partial list-tile props: T><text>{title}</text></partial>
<list-tile title={"x"}><span>nope</span></list-tile>
"#;
        let err = expand_partials(src).expect_err("slot");
        assert!(err.message.contains("slot"), "{}", err.message);
    }

    #[test]
    fn style_const_readable_in_body() {
        let src = r#"
<scope>
type T = { title: string }
style card = { box: pad(2) }
</scope>
<partial list-tile props: T>
  <row style(card)><text>{title}</text></row>
</partial>
<list-tile title={"Hi"}/>
"#;
        let out = expand_partials(src).expect("expand");
        assert!(out.contains("style(card)"), "style const kept: {out}");
        assert!(
            out.contains("\"Hi\"") || out.contains("Hi"),
            "param sub: {out}"
        );
    }
}
