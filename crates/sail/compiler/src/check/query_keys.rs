//! `query()` / `mutation()` / `invalidate()` resolution law — the same
//! "derive or refuse, no third path" muscle `sx_vars.rs` uses for `(--x)`.
//!
//! See `packages/sail/runtime/src/query.ts`'s module header for the full
//! author-facing contract this pass implements. Short version:
//!
//!   query(() => crm.deals.list({ stage: stage() }))
//!     → query(() => crm.deals.list({ stage: stage() }), undefined,
//!              () => ["crm", "deals", "list", { stage: stage() }])
//!
//!   mutation(() => crm.deals.update(draft()))
//!     → mutation(() => crm.deals.update(draft()), undefined, ["crm", "deals"])
//!
//!   mutation(() => crm.deals.update(draft()), { invalidates: [crm.deals.list] })
//!     → mutation(() => crm.deals.update(draft()), undefined, ["crm", "deals"],
//!                [["crm", "deals", "list"]])
//!
//!   invalidate(crm.deals.list) → __invalidate(["crm", "deals", "list"])
//!
//! Resolution law: the thunk's outer call expression must be a static
//! dot-chain (`root.seg.seg(...)`) rooted at a binding IMPORTED into this
//! module. No computed member access (`crm[method]()`), no non-imported
//! root, no bare thunk that doesn't call anything. A thunk this pass
//! cannot statically resolve is a compile error with teaching text — same
//! grammar `sx_vars.rs` uses for an unmatched `(--x)` read, minus the
//! did-you-mean when there's no sensible candidate list (e.g. "no call at
//! all" has nothing to suggest).
//!
//! This is a TEXTUAL pass, like every other `.ts`-shaped check in this
//! crate (`controller_parse`, `pure_derived`, `sx_vars`) — no JS/TS parser
//! dependency, just string/brace/comment-aware scanning.
//!
//! ## PULL vs PUSH routing is a RUNTIME decision, not an emit fork
//!
//! The 2026-07-18 ruling folds `live()`'s PUSH half into `query()` by
//! "type-directed routing": a client method returning `Promise<T>` is a
//! PULL, one returning `LiveSource<T>` is a PUSH. The natural reading is
//! that THIS pass would detect the return type and emit a pull-form vs a
//! push-form call. It does NOT — and cannot: this is a single-module
//! TEXTUAL pass with no type resolver and no cross-module access, while
//! the client method is defined in a DIFFERENT module (`import { crm }
//! from '../clients/crm'`). Resolving its declared return type would mean
//! building a cross-module TS type checker in Rust — far outside "wire
//! the two seams the cache already has".
//!
//! So routing is realized at the RUNTIME type-resolved seam instead (the
//! ruling's own sanctioned fallback: "If the return type is genuinely
//! unknowable at compile time, fall back to a runtime `isLiveSource`
//! branch"). This pass emits ONE unified lowering for every `query()` —
//! `query(thunk, opts, keyFn)` — whose derived KEY is identical whether
//! the method pulls or pushes (same member-path + args identity). At
//! runtime, `queryKeyed`/`nativeQuery` inspect the value the thunk
//! returns (`isLiveSource`) and route to the pull or push cache path. The
//! returned value's shape faithfully reflects the compile-time return
//! type, so the outcome matches the ruling; only the DECISION POINT moved
//! from emit-time to the runtime seam. Every corpus demo hits this path.
//! See `packages/sail/runtime/src/{query,native-cache,live-state}.ts`.

use std::collections::BTreeSet;

use super::js_scan::{is_ident_cont, is_ident_start, skip_balanced, skip_trivia};
use crate::diagnostics::CompileError;
use crate::emit::writer::js_string;

const CATEGORY: &str = "sail-query-key";

/// Rewrite every `query(...)` / `mutation(...)` / `invalidate(...)` call in
/// `source` (a controller module's full text, imports included) into its
/// compiler-lowered form. Returns the first refusal encountered, in source
/// order, on failure.
pub fn rewrite(source: &str) -> Result<String, CompileError> {
    let imports = collect_imports(source);
    // Only recognize `query(`/`mutation(`/`invalidate(` as OUR primitives
    // when that exact name is itself an imported binding — a local
    // `const query = signal('')` (a real fixture in the wild: a search-box
    // component naming its own signal `query`) is not this pass's business,
    // and `query()` reading that signal must be left untouched. This is the
    // same "any import IS a client, nothing else is" stance applied to the
    // primitive names themselves, not just their thunk roots.
    let calls: Vec<Call> = find_calls(source)
        .into_iter()
        .filter(|c| imports.contains(call_kind_ident(c.kind)))
        .collect();

    // Apply replacements back-to-front so earlier byte offsets stay valid.
    let mut out = source.to_string();
    let mut used_dunder_invalidate = false;
    for call in calls.into_iter().rev() {
        let replacement = match call.kind {
            CallKind::Query => lower_query(source, &call, &imports)?,
            CallKind::Mutation => lower_mutation(source, &call, &imports)?,
            CallKind::Invalidate => {
                used_dunder_invalidate = true;
                lower_invalidate(source, &call, &imports)?
            }
        };
        out.replace_range(call.name_start..call.close, &replacement);
    }
    // `__invalidate` is never authored — it's this rewrite's own lowering
    // target, so the emitted module must import it itself (the author's
    // `import { invalidate } from '@ship/sail-runtime'` names a different,
    // now-unused, binding). `runtime_ambient::RUNTIME_AMBIENT` also lists
    // it for the materialized-controller paths that run the ambient-import
    // wrap after this rewrite; this covers the path that doesn't (the dev
    // server's virtual controller module), so the output is self-contained
    // either way.
    if used_dunder_invalidate && !imports.contains("__invalidate") {
        out = format!("import {{ __invalidate }} from '@ship/sail-runtime'\n{out}");
    }
    Ok(out)
}

/// Validate only — same resolution law as [`rewrite`], but discards the
/// rewritten text. Used by `check::run`/`check::collect` so a bad
/// `query()`/`mutation()`/`invalidate()` usage is a build-time error even
/// on paths that never materialize the rewritten controller module.
pub fn check(source: &str) -> Result<(), CompileError> {
    rewrite(source).map(|_| ())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CallKind {
    Query,
    Mutation,
    Invalidate,
}

fn call_kind_ident(kind: CallKind) -> &'static str {
    match kind {
        CallKind::Query => "query",
        CallKind::Mutation => "mutation",
        CallKind::Invalidate => "invalidate",
    }
}

struct Call {
    kind: CallKind,
    /// Start of the call's name identifier (`query`/`mutation`/`invalidate`).
    name_start: usize,
    /// Offset of the opening `(`.
    open: usize,
    /// Offset just past the matching closing `)`.
    close: usize,
}

/// Find every top-level-callable `query(`/`mutation(`/`invalidate(` in
/// `source` — a bare identifier immediately followed by `(`, not itself a
/// property access (`x.query(...)` is a method call on `x`, not our
/// primitive, and is left untouched).
fn find_calls(source: &str) -> Vec<Call> {
    let bytes = source.as_bytes();
    let end = bytes.len();
    let mut out = Vec::new();
    let mut i = 0;
    let mut prev_significant: Option<u8> = None;
    while i < end {
        let c = bytes[i];
        match c {
            b'"' | b'\'' | b'`' => {
                i = skip_string(bytes, i, end, c);
                prev_significant = Some(b'"');
                continue;
            }
            b'/' if i + 1 < end && bytes[i + 1] == b'/' => {
                while i < end && bytes[i] != b'\n' {
                    i += 1;
                }
                continue;
            }
            b'/' if i + 1 < end && bytes[i + 1] == b'*' => {
                i += 2;
                while i + 1 < end && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                i = (i + 2).min(end);
                continue;
            }
            _ if is_ident_start(c) => {
                let s = i;
                while i < end && is_ident_cont(bytes[i]) {
                    i += 1;
                }
                let word = &source[s..i];
                let kind = match word {
                    "query" => Some(CallKind::Query),
                    "mutation" => Some(CallKind::Mutation),
                    "invalidate" => Some(CallKind::Invalidate),
                    _ => None,
                };
                if let Some(kind) = kind {
                    let is_method_call = prev_significant == Some(b'.');
                    let after = skip_trivia(bytes, i);
                    if !is_method_call && after < end && bytes[after] == b'(' {
                        let close = skip_balanced(bytes, after, end);
                        out.push(Call { kind, name_start: s, open: after, close });
                    }
                }
                // `word` is non-empty by construction (is_ident_start matched);
                // stay total anyway.
                if let Some(&last) = word.as_bytes().last() {
                    prev_significant = Some(last);
                }
                continue;
            }
            _ => {
                if !c.is_ascii_whitespace() {
                    prev_significant = Some(c);
                }
                i += 1;
            }
        }
    }
    out
}

/// Split the top-level comma-separated arguments of a call's inner slice
/// (`bytes[open+1..close-1]`, i.e. without the parens) into raw source
/// spans. Bracket/brace/paren aware; string/comment aware.
fn split_args(source: &str, inner_start: usize, inner_end: usize) -> Vec<(usize, usize)> {
    let bytes = source.as_bytes();
    let mut out = Vec::new();
    let mut i = skip_trivia(bytes, inner_start);
    let mut seg_start = i;
    while i < inner_end {
        let c = bytes[i];
        match c {
            b'"' | b'\'' | b'`' => {
                i = skip_string(bytes, i, inner_end, c);
                continue;
            }
            b'(' | b'{' | b'[' => {
                i = skip_balanced(bytes, i, inner_end);
                continue;
            }
            b',' => {
                out.push((seg_start, i));
                i += 1;
                seg_start = skip_trivia(bytes, i);
                continue;
            }
            _ => i += 1,
        }
    }
    if seg_start < inner_end || (out.is_empty() && inner_end > inner_start) {
        let e = trim_end(source, seg_start, inner_end);
        if e > seg_start {
            out.push((seg_start, e));
        }
    }
    out
}

fn trim_end(source: &str, start: usize, mut end: usize) -> usize {
    let bytes = source.as_bytes();
    while end > start && bytes[end - 1].is_ascii_whitespace() {
        end -= 1;
    }
    end
}

fn skip_string(bytes: &[u8], i: usize, end: usize, quote: u8) -> usize {
    let mut k = i + 1;
    while k < end {
        let c = bytes[k];
        if c == b'\\' {
            k = (k + 2).min(end);
            continue;
        }
        if c == quote {
            return k + 1;
        }
        if quote == b'`' && c == b'$' && k + 1 < end && bytes[k + 1] == b'{' {
            k = skip_balanced(bytes, k + 1, end);
            continue;
        }
        k += 1;
    }
    end
}

/// A statically-resolved member path: `root` is an imported local binding,
/// `segments` are the dot-access names after it (possibly empty).
struct MemberPath {
    root: String,
    segments: Vec<String>,
}

impl MemberPath {
    /// The path as a JS string-literal array: `["root", "seg", ...]`.
    fn as_js_array(&self) -> String {
        let mut parts: Vec<String> = vec![js_string(&self.root)];
        parts.extend(self.segments.iter().map(|s| js_string(s)));
        format!("[{}]", parts.join(", "))
    }

    /// The path minus its last segment (the auto-invalidate namespace for
    /// a mutation's mutated method). If there are no segments, this is
    /// just the root.
    fn namespace(&self) -> MemberPath {
        let mut segments = self.segments.clone();
        segments.pop();
        MemberPath { root: self.root.clone(), segments }
    }
}

enum ResolveErr {
    /// The expression isn't a dot-chain at all, or contains computed
    /// member access (`x[y]`) / optional chaining (`x?.y`).
    NotStatic,
    /// The root identifier isn't an imported binding.
    UnknownRoot(String),
}

/// Resolve a bare member-path expression (`crm.deals.list`, `crm`) into a
/// [`MemberPath`]. No call parens allowed here — this is for
/// `invalidate(ref)` and `invalidates: [...]` entries, which name a
/// method/namespace, they don't invoke it.
fn resolve_member_path(expr: &str, imports: &BTreeSet<String>) -> Result<MemberPath, ResolveErr> {
    let expr = expr.trim();
    if expr.is_empty() || expr.contains('[') || expr.contains('(') || expr.contains('?') {
        return Err(ResolveErr::NotStatic);
    }
    let mut parts = Vec::new();
    for part in expr.split('.') {
        let part = part.trim();
        if part.is_empty() || !is_plain_ident(part) {
            return Err(ResolveErr::NotStatic);
        }
        parts.push(part.to_string());
    }
    let root = parts.remove(0);
    if !imports.contains(&root) {
        return Err(ResolveErr::UnknownRoot(root));
    }
    Ok(MemberPath { root, segments: parts })
}

fn is_plain_ident(s: &str) -> bool {
    let bytes = s.as_bytes();
    if bytes.is_empty() || !is_ident_start(bytes[0]) {
        return false;
    }
    bytes[1..].iter().all(|b| is_ident_cont(*b))
}

enum ThunkErr {
    /// A computed member access (`crm[method]`) or optional chaining
    /// (`crm?.deals`) appears before any call — not a static dot-chain.
    Computed,
    /// No call expression at all (e.g. `() => crm.deals.list`, a bare
    /// reference with nothing invoked).
    NoCall,
}

/// Split the outer call in a thunk body (`root.seg.seg(args)`) into its
/// static callee path and its raw argument-list text. `body` is the
/// arrow's already-trimmed expression body (concise arrow only — see
/// [`arrow_body`]).
fn split_outer_call(body: &str) -> Result<(&str, &str), ThunkErr> {
    let bytes = body.as_bytes();
    let end = bytes.len();
    let mut i = 0;
    while i < end {
        match bytes[i] {
            b'(' => break,
            b'[' | b'?' => return Err(ThunkErr::Computed),
            b'"' | b'\'' | b'`' => return Err(ThunkErr::NoCall),
            _ => i += 1,
        }
    }
    if i >= end || i == 0 {
        return Err(ThunkErr::NoCall);
    }
    let close = skip_balanced(bytes, i, end);
    // Everything after the matching `)` must be trivia only — a single
    // trailing call, nothing chained after it.
    let rest = skip_trivia(bytes, close);
    if rest != end {
        return Err(ThunkErr::NoCall);
    }
    let callee = body[..i].trim();
    let args_inner = &body[i + 1..close.saturating_sub(1).max(i + 1)];
    Ok((callee, args_inner))
}

/// Extract the concise-expression body of an arrow function
/// (`(...) => EXPR` / `IDENT => EXPR`), rejecting a block body (`=> {
/// ... }`) — this pass only derives a key from a single expression, never
/// a multi-statement thunk.
fn arrow_body(expr: &str) -> Option<&str> {
    let bytes = expr.as_bytes();
    let end = bytes.len();
    let mut i = skip_trivia(bytes, 0);
    if i >= end {
        return None;
    }
    // async prefix
    if expr[i..].starts_with("async") {
        let after = i + 5;
        if after < end && !is_ident_cont(bytes[after]) {
            i = skip_trivia(bytes, after);
        }
    }
    if i < end && bytes[i] == b'(' {
        let close = skip_balanced(bytes, i, end);
        i = skip_trivia(bytes, close);
    } else if i < end && is_ident_start(bytes[i]) {
        let s = i;
        while i < end && is_ident_cont(bytes[i]) {
            i += 1;
        }
        let _ = &expr[s..i];
        i = skip_trivia(bytes, i);
    } else {
        return None;
    }
    if i + 1 >= end || bytes[i] != b'=' || bytes[i + 1] != b'>' {
        return None;
    }
    let body_start = skip_trivia(bytes, i + 2);
    if body_start < end && bytes[body_start] == b'{' {
        // Block body — not a concise expression thunk.
        return None;
    }
    let body_end = trim_end(expr, body_start, end);
    Some(&expr[body_start..body_end])
}

/// Resolve a `query()`/`mutation()` thunk (the first argument) into its
/// static callee path + raw call-argument text.
fn resolve_thunk<'a>(
    thunk_src: &'a str,
    imports: &BTreeSet<String>,
) -> Result<(MemberPath, Vec<&'a str>), CompileError> {
    let body = arrow_body(thunk_src).ok_or_else(refuse_not_a_thunk)?;
    let (callee, args_inner) = split_outer_call(body).map_err(|e| match e {
        ThunkErr::Computed => refuse_computed(),
        ThunkErr::NoCall => refuse_no_call(),
    })?;
    let path = resolve_member_path(callee, imports).map_err(|e| refuse_from(e, imports))?;
    let arg_spans = split_args(args_inner, 0, args_inner.len());
    let args: Vec<&str> = arg_spans
        .into_iter()
        .map(|(s, e)| args_inner[s..e].trim())
        .collect();
    Ok((path, args))
}

fn lower_query(source: &str, call: &Call, imports: &BTreeSet<String>) -> Result<String, CompileError> {
    let inner_start = call.open + 1;
    let inner_end = call.close.saturating_sub(1).max(inner_start);
    let args = split_args(source, inner_start, inner_end);
    let thunk_src = args
        .first()
        .map(|(s, e)| &source[*s..*e])
        .ok_or_else(refuse_no_call)?;
    let opts_src = args.get(1).map(|(s, e)| source[*s..*e].trim().to_string());

    let (path, tail_args) = resolve_thunk(thunk_src, imports)?;
    let mut key_parts: Vec<String> = vec![js_string(&path.root)];
    key_parts.extend(path.segments.iter().map(|s| js_string(s)));
    key_parts.extend(tail_args.iter().map(|s| s.to_string()));
    let key_fn = format!("() => [{}]", key_parts.join(", "));
    let opts_text = opts_src.unwrap_or_else(|| "undefined".to_string());

    Ok(format!("query({thunk_src}, {opts_text}, {key_fn})"))
}

fn lower_mutation(
    source: &str,
    call: &Call,
    imports: &BTreeSet<String>,
) -> Result<String, CompileError> {
    let inner_start = call.open + 1;
    let inner_end = call.close.saturating_sub(1).max(inner_start);
    let args = split_args(source, inner_start, inner_end);
    let thunk_src = args
        .first()
        .map(|(s, e)| &source[*s..*e])
        .ok_or_else(refuse_no_call)?;
    let opts_raw = args.get(1).map(|(s, e)| source[*s..*e].trim());

    let (path, _tail_args) = resolve_thunk(thunk_src, imports)?;
    let auto_invalidate = path.namespace().as_js_array();

    let (opts_text, invalidates_array) = match opts_raw {
        None => ("undefined".to_string(), None),
        Some(opts) => strip_invalidates(opts, imports)?,
    };

    let mut out = format!("mutation({thunk_src}, {opts_text}, {auto_invalidate}");
    if let Some(inv) = invalidates_array {
        out.push_str(", ");
        out.push_str(&inv);
    }
    out.push(')');
    Ok(out)
}

/// Given a `mutation()` opts object's raw source (`{ staleTime: 5000,
/// invalidates: [...] }`), strip the `invalidates` field and return
/// `(remaining_opts_text_or_undefined, resolved_invalidates_array_or_none)`.
/// A non-object opts value (e.g. a bare identifier) passes through
/// unchanged with no invalidates extraction — only a LITERAL `{...}` is
/// syntactically recognized (see module header: `invalidates` references
/// aren't runtime values, so the compiler must see the literal to erase
/// them).
fn strip_invalidates(
    opts: &str,
    imports: &BTreeSet<String>,
) -> Result<(String, Option<String>), CompileError> {
    let trimmed = opts.trim();
    if !trimmed.starts_with('{') || !trimmed.ends_with('}') {
        return Ok((trimmed.to_string(), None));
    }
    let inner = &trimmed[1..trimmed.len() - 1];
    let fields = split_args(inner, 0, inner.len());
    let mut kept: Vec<String> = Vec::new();
    let mut invalidates_array: Option<String> = None;
    for (s, e) in fields {
        let field = inner[s..e].trim();
        if let Some(rest) = field.strip_prefix("invalidates") {
            let rest = rest.trim_start();
            let Some(value) = rest.strip_prefix(':') else {
                // Shorthand `{ invalidates }` — not a literal array,
                // can't statically resolve.
                return Err(refuse_invalidates_not_literal());
            };
            let value = value.trim();
            if !value.starts_with('[') || !value.ends_with(']') {
                return Err(refuse_invalidates_not_literal());
            }
            let items_src = &value[1..value.len() - 1];
            let items = split_args(items_src, 0, items_src.len());
            let mut resolved: Vec<String> = Vec::new();
            for (is, ie) in items {
                let item = items_src[is..ie].trim();
                match resolve_member_path(item, imports) {
                    Ok(path) => resolved.push(path.as_js_array()),
                    Err(e) => return Err(refuse_from(e, imports)),
                }
            }
            invalidates_array = Some(format!("[{}]", resolved.join(", ")));
        } else if !field.is_empty() {
            kept.push(field.to_string());
        }
    }
    let opts_text = if kept.is_empty() {
        "undefined".to_string()
    } else {
        format!("{{ {} }}", kept.join(", "))
    };
    Ok((opts_text, invalidates_array))
}

fn lower_invalidate(
    source: &str,
    call: &Call,
    imports: &BTreeSet<String>,
) -> Result<String, CompileError> {
    let inner_start = call.open + 1;
    let inner_end = call.close.saturating_sub(1).max(inner_start);
    let args = split_args(source, inner_start, inner_end);
    let arg = args
        .first()
        .map(|(s, e)| source[*s..*e].trim())
        .ok_or_else(refuse_invalidate_no_arg)?;
    if arg.starts_with('"') || arg.starts_with('\'') || arg.starts_with('`') || arg.starts_with('[') {
        return Err(refuse_invalidate_literal());
    }
    let path = resolve_member_path(arg, imports).map_err(|e| refuse_from(e, imports))?;
    Ok(format!("__invalidate({})", path.as_js_array()))
}

// ---------------------------------------------------------------------
// Import table
// ---------------------------------------------------------------------

/// Collect every value-level local binding introduced by a top-level `import`
/// statement — the "client" table this pass resolves `query()`/`mutation()`/
/// `invalidate()` roots against (see module header: any import used as
/// `binding.prop(...)` inside one of these three forms IS a client, no
/// whitelist). `import type { ... }` (and `import { type X }` specifiers)
/// are erased at runtime and excluded.
fn collect_imports(source: &str) -> BTreeSet<String> {
    let bytes = source.as_bytes();
    let end = bytes.len();
    let mut out = BTreeSet::new();
    let mut i = 0;
    let mut depth: i32 = 0;
    while i < end {
        let c = bytes[i];
        match c {
            b'"' | b'\'' | b'`' => {
                i = skip_string(bytes, i, end, c);
                continue;
            }
            b'/' if i + 1 < end && bytes[i + 1] == b'/' => {
                while i < end && bytes[i] != b'\n' {
                    i += 1;
                }
                continue;
            }
            b'/' if i + 1 < end && bytes[i + 1] == b'*' => {
                i += 2;
                while i + 1 < end && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                i = (i + 2).min(end);
                continue;
            }
            b'(' | b'{' | b'[' => {
                depth += 1;
                i += 1;
                continue;
            }
            b')' | b'}' | b']' => {
                depth -= 1;
                i += 1;
                continue;
            }
            _ if depth == 0 && is_ident_start(c) => {
                let s = i;
                while i < end && is_ident_cont(bytes[i]) {
                    i += 1;
                }
                if &source[s..i] == "import" {
                    let (bindings, after) = parse_import_clause(source, i);
                    out.extend(bindings);
                    i = after;
                    continue;
                }
                continue;
            }
            _ => {
                i += 1;
            }
        }
    }
    out
}

/// Parse an `import ...` statement's binding clause starting just past the
/// `import` keyword. Returns the local names it introduces and the offset
/// to resume scanning from (just past the statement, best-effort — this
/// pass doesn't need exact statement boundaries, only to not re-enter the
/// clause it just consumed).
fn parse_import_clause(source: &str, start: usize) -> (Vec<String>, usize) {
    let bytes = source.as_bytes();
    let end = bytes.len();
    let mut i = skip_trivia(bytes, start);

    // `import type ...` — type-only import, no runtime binding.
    if source[i..].starts_with("type") {
        let after = i + 4;
        if after < end && !is_ident_cont(bytes[after]) {
            // Bail out to the statement end without collecting anything.
            let stop = find_stmt_end(bytes, i, end);
            return (Vec::new(), stop);
        }
    }

    let mut names = Vec::new();
    // Side-effect-only import: `import "./x"` — no clause at all.
    if i < end && (bytes[i] == b'"' || bytes[i] == b'\'') {
        let stop = skip_string(bytes, i, end, bytes[i]);
        return (names, stop);
    }

    loop {
        i = skip_trivia(bytes, i);
        if i >= end {
            break;
        }
        match bytes[i] {
            b'*' => {
                // `* as ns`
                i = skip_trivia(bytes, i + 1);
                if source[i..].starts_with("as") {
                    i = skip_trivia(bytes, i + 2);
                    let s = i;
                    while i < end && is_ident_cont(bytes[i]) {
                        i += 1;
                    }
                    names.push(source[s..i].to_string());
                }
            }
            b'{' => {
                let close = skip_balanced(bytes, i, end);
                let inner = &source[i + 1..close.saturating_sub(1).max(i + 1)];
                for spec in split_top_level_commas(inner) {
                    let spec = spec.trim();
                    if spec.is_empty() || spec.starts_with("type ") {
                        continue;
                    }
                    let local = spec.rsplit("as").next().unwrap_or(spec).trim();
                    if is_plain_ident(local) {
                        names.push(local.to_string());
                    }
                }
                i = close;
            }
            b'"' | b'\'' => {
                // `from "specifier"` — end of clause.
                i = skip_string(bytes, i, end, bytes[i]);
                break;
            }
            b',' => {
                i += 1;
            }
            _ if is_ident_start(bytes[i]) => {
                let s = i;
                while i < end && is_ident_cont(bytes[i]) {
                    i += 1;
                }
                let word = &source[s..i];
                if word == "from" {
                    // consume the specifier string
                    let j = skip_trivia(bytes, i);
                    if j < end && (bytes[j] == b'"' || bytes[j] == b'\'') {
                        i = skip_string(bytes, j, end, bytes[j]);
                    }
                    break;
                }
                names.push(word.to_string());
            }
            _ => {
                i += 1;
            }
        }
    }
    let stop = find_stmt_end(bytes, i, end);
    (names, stop)
}

fn find_stmt_end(bytes: &[u8], start: usize, end: usize) -> usize {
    let mut i = start;
    while i < end && bytes[i] != b';' && bytes[i] != b'\n' {
        i += 1;
    }
    (i + 1).min(end)
}

/// Split on top-level commas (bracket/paren/brace/string aware) — used for
/// named-import specifier lists (no nested brackets expected, but reuses
/// the same string-aware walk as [`split_args`] for safety).
fn split_top_level_commas(s: &str) -> Vec<String> {
    let spans = split_args(s, 0, s.len());
    spans.into_iter().map(|(a, b)| s[a..b].to_string()).collect()
}

// ---------------------------------------------------------------------
// Refusals
// ---------------------------------------------------------------------

fn refuse_not_a_thunk() -> CompileError {
    CompileError::new(
        "`query()`/`mutation()` must be called with a zero/one-arg arrow whose body is a \
         SINGLE call expression (e.g. `() => crm.deals.list(...)`) — the compiler derives the \
         cache key statically from that call. A block-bodied thunk (`() => { ... }`) has no \
         single expression to derive a key from.",
    )
    .with_category(CATEGORY)
}

fn refuse_computed() -> CompileError {
    CompileError::new(
        "the thunk passed to `query()`/`mutation()` uses computed member access \
         (`crm[method]()`) or optional chaining — the compiler derives the cache key from a \
         LITERAL dot-chain (`crm.deals.list(...)`), never a runtime-computed path.",
    )
    .with_category(CATEGORY)
}

fn refuse_no_call() -> CompileError {
    CompileError::new(
        "the thunk passed to `query()`/`mutation()` doesn't call anything — the compiler \
         derives the cache key from a static method call (e.g. `() => crm.deals.list(...)`), \
         so there must be exactly one, and it must be the whole expression.",
    )
    .with_category(CATEGORY)
}

fn refuse_invalidate_no_arg() -> CompileError {
    CompileError::new(
        "`invalidate(ref)` needs one argument — a method or namespace reference from an \
         imported client (e.g. `invalidate(crm.deals.list)`).",
    )
    .with_category(CATEGORY)
}

fn refuse_invalidate_literal() -> CompileError {
    CompileError::new(
        "`invalidate(ref)` takes a method/namespace REFERENCE, never a string or array \
         literal — cache keys are compiler-derived, never handwritten. Use \
         `invalidate(crm.deals.list)` (or `invalidate(crm.deals)` / `invalidate(crm)`), not \
         `invalidate('crm.deals.list')` or `invalidate(['crm', 'deals'])`.",
    )
    .with_category(CATEGORY)
}

fn refuse_invalidates_not_literal() -> CompileError {
    CompileError::new(
        "`invalidates:` must be a literal array of method references (e.g. `invalidates: \
         [crm.deals.list, billing.summary]`) — the compiler resolves each reference to a key \
         prefix at compile time, so it must see the array literal directly, not a variable or \
         spread.",
    )
    .with_category(CATEGORY)
}

fn refuse_from(err: ResolveErr, imports: &BTreeSet<String>) -> CompileError {
    match err {
        ResolveErr::NotStatic => CompileError::new(
            "this reference isn't a static dot-chain from an imported client — no computed \
             member access (`crm[method]`), no optional chaining (`crm?.deals`). The compiler \
             derives cache keys and invalidation prefixes from a literal `client.seg.seg` path.",
        )
        .with_category(CATEGORY),
        ResolveErr::UnknownRoot(root) => {
            let mut msg = format!(
                "`{root}` is not a client imported into this module. `query()`/`mutation()`/\
                 `invalidate()` derive identity from an IMPORTED binding — every root here \
                 must come from an ES `import`."
            );
            if let Some(s) = closest(&root, imports) {
                msg.push_str(&format!(" Did you mean `{s}`?"));
            }
            CompileError::new(msg).with_category(CATEGORY)
        }
    }
}

/// Closest import name by Levenshtein distance, capped so a wild guess
/// isn't offered as a "did you mean" — mirrors `sx_vars.rs`'s `closest`.
fn closest(name: &str, candidates: &BTreeSet<String>) -> Option<String> {
    let max_dist = (name.len() / 2).max(2);
    candidates
        .iter()
        .map(|c| (c, levenshtein(name, c)))
        .filter(|(_, d)| *d <= max_dist)
        .min_by_key(|(_, d)| *d)
        .map(|(c, _)| c.clone())
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0usize; b.len() + 1];
    for i in 1..=a.len() {
        cur[0] = i;
        for j in 1..=b.len() {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            cur[j] = (prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_lowers_the_derived_key_fn() {
        let src = r#"
import { query } from '@ship/sail-runtime'
import { crm } from '../clients/crm'
const deals = query(() => crm.deals.list({ stage: stage() }))
"#;
        let out = rewrite(src).unwrap();
        assert!(
            out.contains(
                r#"query(() => crm.deals.list({ stage: stage() }), undefined, () => ["crm", "deals", "list", { stage: stage() }])"#
            ),
            "{out}"
        );
    }

    #[test]
    fn query_lowering_is_return_type_agnostic_routing_is_runtime() {
        // The PULL-vs-PUSH decision is a RUNTIME property of the value the
        // thunk returns (`isLiveSource`), NOT an emit fork this textual,
        // single-module pass could make (it has no cross-module type
        // resolver). So a thunk calling a PUSH-shaped method (`.wire`,
        // returning `LiveSource<T>`) lowers to the SAME unified form — and
        // the SAME derived key identity — as a PULL-shaped one (`.list`,
        // returning `Promise<T>`): member-path + args, nothing about the
        // return type. See the module header, "PULL vs PUSH routing".
        let pull = r#"
import { query } from '@ship/sail-runtime'
import { crm } from '../clients/crm'
const deals = query(() => crm.deals.list())
"#;
        let push = r#"
import { query } from '@ship/sail-runtime'
import { feed } from '../clients/feed'
const ticks = query(() => feed.clock.wire())
"#;
        let pull_out = rewrite(pull).unwrap();
        let push_out = rewrite(push).unwrap();
        assert!(
            pull_out.contains(
                r#"query(() => crm.deals.list(), undefined, () => ["crm", "deals", "list"])"#
            ),
            "{pull_out}"
        );
        // Identical SHAPE of lowering (thunk, opts, key-fn) — the push
        // method is not marked, flagged, or emitted differently.
        assert!(
            push_out.contains(
                r#"query(() => feed.clock.wire(), undefined, () => ["feed", "clock", "wire"])"#
            ),
            "{push_out}"
        );
    }

    #[test]
    fn query_keeps_an_explicit_opts_arg() {
        let src = r#"
import { query } from '@ship/sail-runtime'
import { crm } from '../clients/crm'
const deals = query(() => crm.deals.list(), { staleTime: 5000 })
"#;
        let out = rewrite(src).unwrap();
        assert!(
            out.contains(r#"query(() => crm.deals.list(), { staleTime: 5000 }, () => ["crm", "deals", "list"])"#),
            "{out}"
        );
    }

    #[test]
    fn mutation_lowers_the_auto_invalidate_namespace() {
        let src = r#"
import { mutation } from '@ship/sail-runtime'
import { crm } from '../clients/crm'
const save = mutation(() => crm.deals.update(draft()))
"#;
        let out = rewrite(src).unwrap();
        assert!(
            out.contains(r#"mutation(() => crm.deals.update(draft()), undefined, ["crm", "deals"])"#),
            "{out}"
        );
    }

    #[test]
    fn mutation_lowers_explicit_invalidates_and_strips_the_option() {
        let src = r#"
import { mutation } from '@ship/sail-runtime'
import { crm } from '../clients/crm'
import { billing } from '../clients/billing'
const save = mutation(() => crm.deals.update(draft()), { invalidates: [crm.deals.list, billing.summary] })
"#;
        let out = rewrite(src).unwrap();
        assert!(
            out.contains(
                r#"mutation(() => crm.deals.update(draft()), undefined, ["crm", "deals"], [["crm", "deals", "list"], ["billing", "summary"]])"#
            ),
            "{out}"
        );
    }

    #[test]
    fn mutation_keeps_sibling_opts_fields_alongside_stripped_invalidates() {
        let src = r#"
import { mutation } from '@ship/sail-runtime'
import { crm } from '../clients/crm'
const save = mutation(() => crm.deals.update(draft()), { staleTime: 5000, invalidates: [crm.deals.list] })
"#;
        let out = rewrite(src).unwrap();
        assert!(
            out.contains(
                r#"mutation(() => crm.deals.update(draft()), { staleTime: 5000 }, ["crm", "deals"], [["crm", "deals", "list"]])"#
            ),
            "{out}"
        );
    }

    #[test]
    fn invalidate_lowers_a_method_reference() {
        let src = r#"
import { invalidate } from '@ship/sail-runtime'
import { crm } from '../clients/crm'
invalidate(crm.deals.list)
"#;
        let out = rewrite(src).unwrap();
        assert!(out.contains(r#"__invalidate(["crm", "deals", "list"])"#), "{out}");
    }

    #[test]
    fn invalidate_lowers_a_namespace_reference() {
        let src = r#"
import { invalidate } from '@ship/sail-runtime'
import { crm } from '../clients/crm'
invalidate(crm.deals)
"#;
        let out = rewrite(src).unwrap();
        assert!(out.contains(r#"__invalidate(["crm", "deals"])"#), "{out}");
    }

    #[test]
    fn invalidate_lowers_a_whole_client_reference() {
        let src = r#"
import { invalidate } from '@ship/sail-runtime'
import { crm } from '../clients/crm'
invalidate(crm)
"#;
        let out = rewrite(src).unwrap();
        assert!(out.contains(r#"__invalidate(["crm"])"#), "{out}");
    }

    #[test]
    fn a_computed_member_thunk_refuses() {
        let src = r#"
import { query } from '@ship/sail-runtime'
import { crm } from '../clients/crm'
const method = 'list'
const deals = query(() => crm.deals[method]({ stage: stage() }))
"#;
        let err = rewrite(src).unwrap_err();
        assert!(err.message.contains("computed member access"), "{}", err.message);
    }

    #[test]
    fn a_non_imported_root_refuses_with_did_you_mean() {
        let src = r#"
import { query } from '@ship/sail-runtime'
import { crm } from '../clients/crm'
const local = { deals: { list: () => Promise.resolve([]) } }
const deals = query(() => crn.deals.list())
"#;
        let err = rewrite(src).unwrap_err();
        assert!(err.message.contains("not a client imported"), "{}", err.message);
        assert!(err.message.contains("Did you mean `crm`?"), "{}", err.message);
    }

    #[test]
    fn a_thunk_that_calls_nothing_refuses() {
        let src = r#"
import { query } from '@ship/sail-runtime'
import { crm } from '../clients/crm'
const deals = query(() => crm.deals.list)
"#;
        let err = rewrite(src).unwrap_err();
        assert!(err.message.contains("doesn't call anything"), "{}", err.message);
    }

    #[test]
    fn invalidate_with_a_string_literal_refuses() {
        let src = r#"
import { invalidate } from '@ship/sail-runtime'
import { crm } from '../clients/crm'
invalidate('crm.deals.list')
"#;
        let err = rewrite(src).unwrap_err();
        assert!(err.message.contains("never a string or array literal"), "{}", err.message);
    }

    #[test]
    fn invalidate_with_an_array_literal_refuses() {
        let src = r#"
import { invalidate } from '@ship/sail-runtime'
import { crm } from '../clients/crm'
invalidate(['crm', 'deals', 'list'])
"#;
        let err = rewrite(src).unwrap_err();
        assert!(err.message.contains("never a string or array literal"), "{}", err.message);
    }

    #[test]
    fn a_thunk_closing_over_a_non_imported_local_refuses() {
        let src = r#"
import { query } from '@ship/sail-runtime'
const crm = { deals: { list: () => Promise.resolve([]) } }
const deals = query(() => crm.deals.list())
"#;
        let err = rewrite(src).unwrap_err();
        assert!(err.message.contains("not a client imported"), "{}", err.message);
    }

    #[test]
    fn a_local_signal_named_query_is_left_untouched() {
        // Real-world shape (sail-cmdk): `const query = signal('')` then
        // `query()` reads it. `query` isn't imported here, so this is not
        // our primitive — must not be rewritten or refused.
        let src = r#"
const query = signal('')
const q = query().trim()
"#;
        let out = rewrite(src).unwrap();
        assert_eq!(out, src, "an unimported local named `query` is not our primitive");
    }

    #[test]
    fn a_method_call_named_query_is_left_untouched() {
        let src = r#"
import { crm } from '../clients/crm'
const x = someObject.query(() => crm.deals.list())
"#;
        let out = rewrite(src).unwrap();
        assert_eq!(out, src, "a `.query(` method call on some other object is not our primitive");
    }
}
