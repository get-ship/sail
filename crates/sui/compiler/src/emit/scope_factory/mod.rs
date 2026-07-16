//! Extract `(name, ts-type)` pairs from a paired controller's
//! `setup: () => ({...})` factory so the d.ts emitter can surface them
//! as additional `Scope` members.
//!
//! Two paths are handled:
//!
//!  * Expression-body factories — `setup: () => ({ foo: signal(0) })`.
//!    The returned object literal is the entry source.
//!  * Block-body factories — `setup: () => { const foo = signal(0); return { foo } }`.
//!    The last `return {...}` is the entry source; shorthand entries
//!    resolve by locating a top-level `const NAME = …` / `let NAME = …`
//!    / `function NAME …` binding within the body and re-running the
//!    type heuristic against its RHS.
//!
//! Inferred types are best-effort string heuristics:
//!  * `signal<T>(…)` → `Signal<T>`; `signal(literal)` → `Signal<typeof literal>`
//!    (e.g. `signal(false)` → `Signal<boolean>`). The generic may itself be a
//!    nested generic — `signal<Record<K, V>>(…)` — whose internal commas are
//!    not entry separators (see `next_top_separator`/`skip_angle`).
//!  * `ref<T>(…)` → `Signal<T | null>`; `ref(null)` → `Signal<HTMLElement | null>`
//!    (`Ref<T>` is itself an alias for `Signal<T | null>`, so this is exact).
//!  * `derived(() => …)` / `computed` / `memo` → `Signal<unknown>` (the body's
//!    return type is genuinely uninferrable from a byte scan; annotate
//!    `derived<T>(…)` for a tighter type).
//!  * `live(() => …)` → `LiveSignal<unknown>` (= `Signal<LiveState<unknown>>`),
//!    or `LiveSignal<T>` when annotated `live<T>(…)`. The payload type usually
//!    sits on an inner SDK call (`liveExpr<T>(…)`) referencing controller-local
//!    types the `.d.ts` module can't import, so we don't fabricate it.
//!  * `(...): T => …` / `(...) => {…}` → `(...args) => T | void`
//!  * primitive literals (`42`, `'x'`, `true`) → primitive type
//!  * anything else → `unknown`
//!
//! Authors who want a tighter type than the heuristic can always annotate
//! the call (`signal<MyType>(...)`). Untyped values fall back to `unknown`,
//! which still typechecks against `Signal<unknown>` / direct member use.

use crate::check::controller_parse::ParsedController;
use crate::diagnostics::Span;

// Three concerns: this module owns scope-entry extraction (locate the return
// object, walk its entries, resolve shorthand bindings). `infer` is the
// value→TS-type heuristic; `scan` is the generic byte-scan primitives.
mod infer;
mod scan;
use infer::{infer_ts_type, skip_fn_return_type, skip_type_annotation};
use scan::{
    advance_past_entry, is_ident_cont, is_ident_start, next_top_separator, skip_balanced,
    skip_trivia,
};

/// `(name, ts_type)` pairs for every entry surfaced by the controller's
/// `setup: () => ({...})` factory. Returns an empty vec when no `scope`
/// section is present or the factory body doesn't model a return object.
pub fn extract(controller: &ParsedController) -> Vec<(String, String)> {
    let Some(section) = controller.section("setup") else {
        return Vec::new();
    };
    let bytes = controller.source.as_bytes();
    let body_span = section.value_span;
    let Some(obj_span) = locate_return_object(bytes, body_span) else {
        return Vec::new();
    };
    // Types declared in the controller .ts (`interface Pin`, `type Foo`) can't
    // resolve in the generated .sui.d.ts — see `downgrade_unresolvable`.
    let locals = local_type_names(&controller.source);
    let mut out: Vec<(String, String)> = Vec::new();
    walk_entries(bytes, obj_span, |name, value_start, value_end| {
        let ty = if is_shorthand(bytes, name, value_start, value_end) {
            resolve_local_binding(bytes, name, body_span.start, obj_span.start)
                .unwrap_or_else(|| "unknown".to_string())
        } else {
            infer_ts_type(bytes, value_start, value_end)
        };
        out.push((name.to_string(), downgrade_unresolvable(&ty, &locals)));
    });
    out
}

/// Names declared locally in the controller source (`interface X`, `type X`,
/// `enum X`, `class X`). These are visible to the controller's own type
/// inference but NOT to the generated `.sui.d.ts`.
fn local_type_names(source: &str) -> std::collections::HashSet<String> {
    let mut set = std::collections::HashSet::new();
    for kw in ["interface ", "type ", "enum ", "class "] {
        let mut from = 0usize;
        while let Some(rel) = source[from..].find(kw) {
            let after = from + rel + kw.len();
            let name: String = source[after..]
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                set.insert(name);
            }
            from = after;
        }
    }
    set
}

/// Downgrade a scope-factory field type whose generic references a
/// controller-local type (e.g. `Signal<Pin[]>` where `Pin` is an `interface`
/// in the controller) to `unknown`.
///
/// Why: the generated `.sui.d.ts` only imports the signal wrappers, so a
/// dangling `Pin` degrades to `any` — and `any & <SC-inferred>` absorbs to
/// `any`, muddying the controller's `MutableState`. `Signal<unknown>` instead
/// resolves, and `unknown & Signal<Pin[]>` collapses to the precise `SC` type
/// the controller's scope factory already carries. Precision lives in `SC`
/// inference; the `.d.ts` stays self-contained. DOM/lib globals
/// (`HTMLButtonElement`, `Record`, …) resolve fine and keep their generic.
fn downgrade_unresolvable(ty: &str, locals: &std::collections::HashSet<String>) -> String {
    let references_local = locals.iter().any(|n| word_appears(ty, n));
    if !references_local {
        return ty.to_string();
    }
    if ty.starts_with("Signal<LiveState<") {
        "Signal<LiveState<unknown>>".to_string()
    } else if ty.starts_with("LiveSignal<") {
        "LiveSignal<unknown>".to_string()
    } else if ty.starts_with("Signal<") {
        "Signal<unknown>".to_string()
    } else {
        "unknown".to_string()
    }
}

/// `word` occurs in `ty` bounded by non-identifier chars (so `Pin` matches in
/// `Signal<Pin[]>` but not inside `Pinned`).
fn word_appears(ty: &str, word: &str) -> bool {
    let mut from = 0;
    while let Some(rel) = ty[from..].find(word) {
        let start = from + rel;
        let end = start + word.len();
        let before_ok = start == 0
            || !ty.as_bytes()[start - 1].is_ascii_alphanumeric() && ty.as_bytes()[start - 1] != b'_';
        let after_ok = end >= ty.len()
            || !ty.as_bytes()[end].is_ascii_alphanumeric() && ty.as_bytes()[end] != b'_';
        if before_ok && after_ok {
            return true;
        }
        from = start + 1;
    }
    false
}

/// Find the byte span of the object literal returned by the factory.
/// Handles both `=> ({…})` and `=> { … return {…} }` shapes.
fn locate_return_object(bytes: &[u8], span: Span) -> Option<Span> {
    let start = span.start;
    let end = span.end.min(bytes.len());
    match find_factory_body_brace(bytes, start, end) {
        Some(BodyShape::Expression(k)) => {
            // `=> ({...})` — `k` points at the result object's `{`.
            let close = skip_balanced(bytes, k, end);
            Some(Span::new(k + 1, close.saturating_sub(1)))
        }
        Some(BodyShape::Block(k)) => {
            // `=> { ... return {…} }` — walk the block body looking
            // for the last top-level `return {…}`. The walker is
            // string/template/comment-aware and tracks brace + paren
            // depth so doc snippets (`"return { count }"`), template
            // literals, `//` line comments, and `/* */` block
            // comments cannot inject keys; `return` inside a nested
            // function body or helper closure is also ignored.
            find_last_top_level_return_object(bytes, k, end).map(|s| {
                let e = skip_balanced(bytes, s, end);
                Span::new(s + 1, e.saturating_sub(1))
            })
        }
        None => None,
    }
}

/// Shape of the scope factory's body, once we've located the
/// opening brace. `Expression` covers `=> ({...})`; `Block` covers
/// `=> { ... return {…} }`. Both carry the offset of the `{` that
/// opens the result.
enum BodyShape {
    Expression(usize),
    Block(usize),
}

/// Skip past the arrow's parameter list (and any return-type
/// annotation) and return the offset of the brace that opens the
/// factory body. Distinguishes expression-body (`=> ({...})`) from
/// block-body (`=> { ... }`).
fn find_factory_body_brace(bytes: &[u8], start: usize, end: usize) -> Option<BodyShape> {
    let mut k = skip_trivia(bytes, start, end);
    if k >= end {
        return None;
    }
    if bytes[k] == b'(' {
        k = skip_balanced(bytes, k, end);
    } else if is_ident_start(bytes[k]) {
        while k < end && is_ident_cont(bytes[k]) {
            k += 1;
        }
    } else {
        return None;
    }
    k = skip_trivia(bytes, k, end);
    // Optional return-type annotation: `(): T =>`.
    if k < end && bytes[k] == b':' {
        k = skip_type_annotation(bytes, k + 1, end);
        k = skip_trivia(bytes, k, end);
    }
    if k + 1 >= end || bytes[k] != b'=' || bytes[k + 1] != b'>' {
        return None;
    }
    k = skip_trivia(bytes, k + 2, end);
    // `=> ({...})` — the wrapping `(` distinguishes expression-body.
    if k < end && bytes[k] == b'(' {
        k = skip_trivia(bytes, k + 1, end);
        if k < end && bytes[k] == b'{' {
            return Some(BodyShape::Expression(k));
        }
        return None;
    }
    if k < end && bytes[k] == b'{' {
        Some(BodyShape::Block(k))
    } else {
        None
    }
}

/// Walk `bytes[start..end]` (which spans the factory's block body,
/// starting at its opening `{`) looking for the last top-level
/// `return {` outside of strings, template literals, and comments.
/// Returns the offset of the `{`.
///
/// Pre-fix: the walker was a naïve byte scan that matched any
/// `return` substring. So a doc snippet like `"return { count }"`
/// (or a comment `// return { count }`) would inject a fake `count`
/// key into the extracted set when it appeared AFTER the real
/// `return { name }` — or (worse) overwrite the real key set when it
/// was the only match. That's the bug.
///
/// Depth tracking ensures `return` inside a nested function body or
/// helper closure is ignored — those aren't scope-factory returns.
/// `target_depth == 1` because we start AT the factory's opening
/// `{` (depth 0); the first `{` we walk over is the factory body's
/// own opening, taking us to depth 1.
fn find_last_top_level_return_object(bytes: &[u8], start: usize, end: usize) -> Option<usize> {
    debug_assert!(start < end && bytes[start] == b'{');
    let return_kw = b"return";
    let target_depth: i32 = 1;
    let mut i = start;
    let mut last: Option<usize> = None;
    let mut brace_depth: i32 = 0;
    let mut paren_depth: i32 = 0;
    while i < end {
        let c = bytes[i];
        // ── string / template literals ────────────────────────────────
        if c == b'"' || c == b'\'' || c == b'`' {
            i = skip_string(bytes, i, end, c);
            continue;
        }
        // ── comments ───────────────────────────────────────────────────
        if c == b'/' && i + 1 < end && bytes[i + 1] == b'/' {
            while i < end && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if c == b'/' && i + 1 < end && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < end && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i = (i + 2).min(end);
            continue;
        }
        // ── bracket depth tracking ─────────────────────────────────────
        if c == b'{' {
            brace_depth += 1;
            i += 1;
            continue;
        }
        if c == b'}' {
            brace_depth -= 1;
            i += 1;
            if brace_depth < target_depth {
                // Stepped out of the factory body — any `return`
                // beyond this belongs to the outer scope, not the
                // factory.
                break;
            }
            continue;
        }
        if c == b'(' {
            paren_depth += 1;
            i += 1;
            continue;
        }
        if c == b')' {
            paren_depth -= 1;
            i += 1;
            continue;
        }
        // ── `return` keyword (only at factory-body depth) ──────────────
        if brace_depth == target_depth
            && paren_depth == 0
            && i + return_kw.len() <= end
            && bytes[i..].starts_with(return_kw)
        {
            let prev_ok = i == start || !is_ident_cont(bytes[i - 1]);
            let next = i + return_kw.len();
            let post_ok = next >= end || !is_ident_cont(bytes[next]);
            if prev_ok && post_ok {
                let after = skip_trivia(bytes, next, end);
                if after < end && bytes[after] == b'{' {
                    last = Some(after);
                }
            }
        }
        i += 1;
    }
    last
}

/// Advance past a string / template literal starting at `i` (which
/// must point at the opening quote). Handles `\\` escapes and `${…}`
/// template substitutions (each substitution opens a real JS
/// expression, so we re-enter the string-state machine after its
/// matching `}`). Returns the offset just past the closing quote.
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

/// Walk a `{ key: value, … }` slice (start/end excluding the braces) and
/// invoke `f` per entry.
fn walk_entries<F>(bytes: &[u8], span: Span, mut f: F)
where
    F: FnMut(&str, usize, usize),
{
    let start = span.start;
    let end = span.end.min(bytes.len());
    let mut i = start;
    while i < end {
        i = skip_trivia(bytes, i, end);
        if i >= end {
            break;
        }
        if bytes[i] == b',' {
            i += 1;
            continue;
        }
        let key_start = i;
        let key;
        if bytes[i] == b'"' || bytes[i] == b'\'' {
            let q = bytes[i];
            i += 1;
            let s = i;
            while i < end && bytes[i] != q {
                if bytes[i] == b'\\' {
                    i = (i + 2).min(end);
                    continue;
                }
                i += 1;
            }
            key = std::str::from_utf8(&bytes[s..i])
                .unwrap_or("")
                .to_string();
            if i < end {
                i += 1;
            }
        } else if is_ident_start(bytes[i]) {
            while i < end && is_ident_cont(bytes[i]) {
                i += 1;
            }
            key = std::str::from_utf8(&bytes[key_start..i])
                .unwrap_or("")
                .to_string();
        } else {
            // Spread / computed key / something we don't model.
            i = advance_past_entry(bytes, i, end);
            continue;
        }
        let after_key = i;
        i = skip_trivia(bytes, i, end);
        // Shorthand: `{ key }` — value span IS the key.
        if i >= end || bytes[i] == b',' || bytes[i] == b'}' {
            f(&key, key_start, after_key);
            if i < end && bytes[i] == b',' {
                i += 1;
            }
            continue;
        }
        if bytes[i] != b':' {
            // Method form (`foo() {…}`) or shape we don't recognise — skip.
            i = advance_past_entry(bytes, i, end);
            continue;
        }
        i += 1;
        i = skip_trivia(bytes, i, end);
        let value_start = i;
        let value_end = next_top_separator(bytes, i, end);
        f(&key, value_start, value_end);
        i = value_end;
        if i < end && bytes[i] == b',' {
            i += 1;
        }
    }
}

/// True when a `key: value` entry's value-span is just the key itself
/// (e.g. `{ foo }` — a shorthand binding).
fn is_shorthand(bytes: &[u8], name: &str, start: usize, end: usize) -> bool {
    let s = skip_trivia(bytes, start, end);
    if s + name.len() > end {
        return false;
    }
    if &bytes[s..s + name.len()] != name.as_bytes() {
        return false;
    }
    let after = skip_trivia(bytes, s + name.len(), end);
    after >= end
}

/// Walk the factory body for a top-level `const NAME = …` / `let NAME = …`
/// / `function NAME …` declaration and infer its TS type. Returns `None`
/// if no such binding exists.
fn resolve_local_binding(
    bytes: &[u8],
    name: &str,
    start: usize,
    end: usize,
) -> Option<String> {
    let nb = name.as_bytes();
    let mut i = start;
    while i < end {
        // `function NAME(...)` / `async function NAME(...)`.
        if bytes[i..end.min(i + 8)].starts_with(b"function") {
            let prev_ok = i == start || !is_ident_cont(bytes[i - 1]);
            let after_kw = i + 8;
            if prev_ok && after_kw < end && !is_ident_cont(bytes[after_kw]) {
                let j_start = skip_trivia(bytes, after_kw, end);
                if j_start + nb.len() <= end
                    && &bytes[j_start..j_start + nb.len()] == nb
                    && (j_start + nb.len() == end || !is_ident_cont(bytes[j_start + nb.len()]))
                {
                    let j = skip_trivia(bytes, j_start + nb.len(), end);
                    if j < end && bytes[j] == b'(' {
                        let pclose = skip_balanced(bytes, j, end);
                        let after_params = skip_trivia(bytes, pclose, end);
                        let ret = if after_params < end && bytes[after_params] == b':' {
                            let ret_start = after_params + 1;
                            // A function-declaration return type ends at the
                            // body's `{` — NOT `skip_type_annotation`, which
                            // treats that brace as an object type and swallows
                            // the whole body into the `.d.ts` (invalid TS).
                            let ret_end = skip_fn_return_type(bytes, ret_start, end);
                            let ts = std::str::from_utf8(&bytes[ret_start..ret_end])
                                .unwrap_or("")
                                .trim()
                                .to_string();
                            if ts.is_empty() { "void".to_string() } else { ts }
                        } else {
                            "void".to_string()
                        };
                        let params = std::str::from_utf8(&bytes[j..pclose])
                            .unwrap_or("()")
                            .to_string();
                        return Some(format!("{params} => {ret}"));
                    }
                }
            }
        }
        // `const NAME = …` / `let NAME = …`.
        let kw_len = if bytes[i..end.min(i + 5)].starts_with(b"const") {
            5
        } else if bytes[i..end.min(i + 3)].starts_with(b"let") {
            3
        } else {
            0
        };
        if kw_len > 0 {
            let prev_ok = i == start || !is_ident_cont(bytes[i - 1]);
            let after_kw = i + kw_len;
            let post_ok = after_kw < end && !is_ident_cont(bytes[after_kw]);
            if prev_ok && post_ok {
                let j = skip_trivia(bytes, after_kw, end);
                if j + nb.len() <= end
                    && &bytes[j..j + nb.len()] == nb
                    && (j + nb.len() == end || !is_ident_cont(bytes[j + nb.len()]))
                {
                    let mut k = skip_trivia(bytes, j + nb.len(), end);
                    let mut explicit_type: Option<String> = None;
                    if k < end && bytes[k] == b':' {
                        let ts_start = k + 1;
                        let ts_end = walk_to_equals(bytes, ts_start, end);
                        let s = std::str::from_utf8(&bytes[ts_start..ts_end])
                            .unwrap_or("")
                            .trim()
                            .to_string();
                        if !s.is_empty() {
                            explicit_type = Some(s);
                        }
                        k = ts_end;
                    }
                    if k < end && bytes[k] == b'=' {
                        if let Some(t) = explicit_type {
                            return Some(t);
                        }
                        let rhs_start = skip_trivia(bytes, k + 1, end);
                        let rhs_end = next_top_separator(bytes, rhs_start, end);
                        return Some(infer_ts_type(bytes, rhs_start, rhs_end));
                    }
                }
            }
        }
        i += 1;
    }
    None
}

/// Walk a binding annotation `: TYPE = …` up to the top-level `=`.
fn walk_to_equals(bytes: &[u8], start: usize, end: usize) -> usize {
    let mut k = start;
    let mut depth: i32 = 0;
    while k < end {
        let c = bytes[k];
        if depth == 0 && c == b'=' {
            // Don't trip on `=>` inside the type.
            if k + 1 < end && bytes[k + 1] == b'>' {
                k += 2;
                continue;
            }
            return k;
        }
        match c {
            b'<' | b'(' | b'{' | b'[' => depth += 1,
            b'>' | b')' | b'}' | b']' => depth -= 1,
            _ => {}
        }
        k += 1;
    }
    end
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::controller_parse::parse;
    use std::path::PathBuf;

    fn ctrl(src: &str) -> ParsedController {
        parse(PathBuf::from("x.controller.ts"), src.to_string()).expect("parsed")
    }
    fn names_types(v: &[(String, String)]) -> Vec<(&str, &str)> {
        v.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect()
    }


    #[test]
    fn typed_signal_generic_uses_generic_arg() {
        let c = ctrl(
            "export default controller({\n  \
             setup: () => ({\n    \
               err: signal<Error | null>(null),\n  \
             }),\n});",
        );
        let v = extract(&c);
        assert_eq!(names_types(&v), vec![("err", "Signal<Error | null>")]);
    }

    #[test]
    fn untyped_signal_infers_from_literal() {
        let c = ctrl(
            "export default controller({\n  \
             setup: () => ({\n    \
               open: signal(false),\n    \
               count: signal(0),\n    \
               name: signal(''),\n  \
             }),\n});",
        );
        let v = extract(&c);
        let map: Vec<(&str, &str)> = names_types(&v);
        assert!(map.contains(&("open", "Signal<boolean>")), "{map:?}");
        assert!(map.contains(&("count", "Signal<number>")), "{map:?}");
        assert!(map.contains(&("name", "Signal<string>")), "{map:?}");
    }

    #[test]
    fn ref_with_generic() {
        let c = ctrl(
            "export default controller({\n  \
             setup: () => ({\n    \
               btn: ref<HTMLButtonElement>(null),\n  \
             }),\n});",
        );
        let v = extract(&c);
        assert_eq!(names_types(&v), vec![("btn", "Signal<HTMLButtonElement | null>")]);
    }

    #[test]
    fn arrow_function_returns_void() {
        let c = ctrl(
            "export default controller({\n  \
             setup: () => ({\n    \
               reset: (): void => {},\n  \
             }),\n});",
        );
        let v = extract(&c);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].0, "reset");
        assert!(v[0].1.ends_with("=> void"), "got {}", v[0].1);
    }

    #[test]
    fn block_body_resolves_shorthand_to_const_rhs() {
        let c = ctrl(
            "export default controller({\n  \
               setup: (_b: unknown, host?: Element) => {\n    \
                 const caughtError = signal<Error | null>(null)\n    \
                 const reset = (): void => { caughtError.set(null) }\n    \
                 return { caughtError, reset }\n  \
               },\n});",
        );
        let v = extract(&c);
        let map: Vec<(&str, &str)> = names_types(&v);
        assert!(map.contains(&("caughtError", "Signal<Error | null>")), "{map:?}");
        let reset = v.iter().find(|(n, _)| n == "reset").expect("reset present");
        assert!(reset.1.ends_with("=> void"), "got {}", reset.1);
    }

    #[test]
    fn block_body_function_decl_does_not_leak_body() {
        // Regression: `function NAME(): void { body }` referenced via shorthand
        // leaked its entire body into the .d.ts return type (invalid TS).
        let c = ctrl(
            "export default controller({\n  \
               setup: (_, host) => {\n    \
                 function endSession(id: string): void {\n      \
                   void fetch(`/x/${id}`)\n      \
                   if (true) { return }\n    \
                 }\n    \
                 function reset(): void { state.clear() }\n    \
                 return { endSession, reset }\n  \
               },\n});",
        );
        let v = extract(&c);
        let es = v.iter().find(|(n, _)| n == "endSession").expect("endSession present");
        assert_eq!(es.1, "(id: string) => void", "body leaked: {}", es.1);
        let reset = v.iter().find(|(n, _)| n == "reset").expect("reset present");
        assert_eq!(reset.1, "() => void", "body leaked: {}", reset.1);
    }

    #[test]
    fn empty_factory_returns_no_entries() {
        let c = ctrl("export default controller({ setup: () => ({}) });");
        assert!(extract(&c).is_empty());
    }

    #[test]
    fn missing_scope_section_returns_no_entries() {
        let c = ctrl("export default controller({ on: { click: () => {} } });");
        assert!(extract(&c).is_empty());
    }

    #[test]
    fn live_without_generic_is_live_signal_unknown() {
        let c = ctrl(
            "export default controller({\n  \
             setup: () => ({\n    \
               rail: live(() => liveExpr(`ship.skills.summaries()`)),\n  \
             }),\n});",
        );
        let v = extract(&c);
        assert_eq!(names_types(&v), vec![("rail", "LiveSignal<unknown>")]);
    }

    #[test]
    fn live_with_explicit_generic_uses_it() {
        let c = ctrl(
            "export default controller({\n  \
             setup: () => ({\n    \
               agents: live<AgentList>(() => ship.agents.summaries()),\n  \
             }),\n});",
        );
        let v = extract(&c);
        assert_eq!(names_types(&v), vec![("agents", "LiveSignal<AgentList>")]);
    }

    #[test]
    fn controller_local_type_downgrades_to_unknown() {
        // `Pin` is declared in the controller, so the generated .sui.d.ts
        // can't resolve it — emit Signal<unknown> (the controller's SC
        // inference carries the precise Signal<Pin[]>). DOM/lib globals and
        // un-declared (imported) names are left untouched.
        let c = ctrl(
            "interface Pin { id: string }\n\
             export default controller({\n  \
             setup: () => ({\n    \
               pins: signal<Pin[]>([]),\n    \
               btn: ref<HTMLButtonElement>(null),\n  \
             }),\n});",
        );
        let v = extract(&c);
        assert_eq!(
            names_types(&v),
            vec![("pins", "Signal<unknown>"), ("btn", "Signal<HTMLButtonElement | null>")],
        );
    }

    #[test]
    fn live_inner_call_generic_does_not_leak_into_type() {
        // The payload generic on the inner `liveExpr<RailValue>(…)` references a
        // controller-local type the `.d.ts` can't import — must NOT be emitted as
        // the LiveSignal arg (that's a broken ref). Stays `LiveSignal<unknown>`.
        let c = ctrl(
            "export default controller({\n  \
             setup: () => ({\n    \
               rail: live(() => liveExpr<RailValue>(`ship.skills.summaries()`)),\n  \
             }),\n});",
        );
        let v = extract(&c);
        assert_eq!(names_types(&v), vec![("rail", "LiveSignal<unknown>")]);
    }

    #[test]
    fn signal_string_literal_union_generic_is_inferred() {
        // Regression: `match_angle` bailed on the first string literal in the
        // type args, so `signal<'a' | 'b'>(…)` degraded to bare `unknown`.
        let c = ctrl(
            "export default controller({\n  \
             setup: () => ({\n    \
               tab: signal<'schema' | 'rows' | 'jsonc'>('schema'),\n  \
             }),\n});",
        );
        let v = extract(&c);
        assert_eq!(
            names_types(&v),
            vec![("tab", "Signal<'schema' | 'rows' | 'jsonc'>")]
        );
    }

    #[test]
    fn nested_generic_signal_is_not_truncated_at_inner_comma() {
        // Regression: `signal<Record<K, V>>(…)` truncated the value span at the
        // first top-level comma (inside the generic), degrading the whole entry
        // to bare `unknown`. The angle-aware separator keeps the full type.
        let c = ctrl(
            "export default controller({\n  \
             setup: () => ({\n    \
               open: signal<Record<Section, boolean>>({ a: true }),\n    \
               q: signal(''),\n  \
             }),\n});",
        );
        let v = extract(&c);
        let map = names_types(&v);
        assert!(
            map.contains(&("open", "Signal<Record<Section, boolean>>")),
            "{map:?}"
        );
        // The trailing entry must still be discovered (proves we resumed
        // scanning past the generic, not swallowed the rest of the object).
        assert!(map.contains(&("q", "Signal<string>")), "{map:?}");
    }

    #[test]
    fn unknown_call_falls_back_to_unknown() {
        let c = ctrl(
            "export default controller({\n  \
             setup: () => ({\n    \
               weird: someUnknownThing(1, 2),\n  \
             }),\n});",
        );
        let v = extract(&c);
        assert_eq!(names_types(&v), vec![("weird", "unknown")]);
    }
    #[test]
    fn string_return_object_does_not_inject_keys() {
        // The snippet string appears AFTER the real return — so the
        // buggy scanner, which picks the LAST `return {…}` it sees,
        // finds the snippet's `{ count }` and treats it as the scope
        // keys. (Doc snippets are commonly appended.)
        let c = ctrl(
            "export default controller({\n  \
               setup: () => {\n    \
                 const name = signal('ada')\n    \
                 return { name }\n    \
                 const snippet = \"return { count }\"\n  \
               },\n});",
        );
        let v = extract(&c);
        let names: Vec<&str> = v.iter().map(|(n, _)| n.as_str()).collect();
        assert!(
            !names.contains(&"count"),
            "string literal leaked into scope keys: {names:?}"
        );
        assert!(
            names.contains(&"name"),
            "real return key missing: {names:?}"
        );
        assert_eq!(v.len(), 1, "expected exactly one key, got {names:?}");
    }

    #[test]
    fn template_literal_return_object_does_not_inject_keys() {
        // Same shape, but with a backtick template literal. The
        // naïve scanner doesn't even skip template substitutions
        // (`${...}`); a string-aware tokenizer must.
        let c = ctrl(
            "export default controller({\n  \
               setup: () => {\n    \
                 const name = signal('ada')\n    \
                 return { name }\n    \
                 const snippet = `return { count }`\n  \
               },\n});",
        );
        let v = extract(&c);
        let names: Vec<&str> = v.iter().map(|(n, _)| n.as_str()).collect();
        assert!(
            !names.contains(&"count"),
            "template literal leaked into scope keys: {names:?}"
        );
        assert!(
            names.contains(&"name"),
            "real return key missing: {names:?}"
        );
        assert_eq!(v.len(), 1, "expected exactly one key, got {names:?}");
    }

    #[test]
    fn comment_return_object_does_not_inject_keys() {
        // Comment text containing `return { ghost }` must not
        // contribute a `ghost` key. (Existing behaviour for code in
        // `//` comments is already correct because the `prev_ok`
        // boundary check sees the `/`; this test pins it.)
        let c = ctrl(
            "export default controller({\n  \
               setup: () => {\n    \
                 // return { ghost }\n    \
                 const name = signal('ada')\n    \
                 return { name }\n  \
               },\n});",
        );
        let v = extract(&c);
        let names: Vec<&str> = v.iter().map(|(n, _)| n.as_str()).collect();
        assert!(
            !names.contains(&"ghost"),
            "comment leaked into scope keys: {names:?}"
        );
        assert!(
            names.contains(&"name"),
            "real return key missing: {names:?}"
        );
        assert_eq!(v.len(), 1, "expected exactly one key, got {names:?}");
    }

    #[test]
    fn block_body_with_complex_scope_extracts_top_level_keys() {
        // Legitimately complex scope: helpers, comments, nested
        // objects. Top-level keys must be exactly the three in the
        // return object — helpers live in `const` / `function`
        // declarations, not in the return shape.
        let c = ctrl(
            "export default controller({\n  \
               setup: (_b: unknown, host?: Element) => {\n    \
                 // helper\n    \
                 function endSession(id: string): void {\n      \
                   void fetch(`/x/${id}`)\n    \
                 }\n    \
                 const caughtError = signal<Error | null>(null)\n    \
                 const reset = (): void => { caughtError.set(null) }\n    \
                 return { caughtError, reset, endSession }\n  \
               },\n});",
        );
        let v = extract(&c);
        let names: Vec<&str> = v.iter().map(|(n, _)| n.as_str()).collect();
        assert!(
            names.contains(&"caughtError"),
            "missing caughtError: {names:?}"
        );
        assert!(names.contains(&"reset"), "missing reset: {names:?}");
        assert!(
            names.contains(&"endSession"),
            "missing endSession: {names:?}"
        );
        // The function body of `endSession` mentions `id` but `id` is
        // NOT a top-level return key.
        assert!(
            !names.contains(&"id"),
            "function param leaked into scope keys: {names:?}"
        );
        assert_eq!(v.len(), 3, "expected exactly 3 keys, got {names:?}");
    }

    #[test]
    fn block_body_with_string_mentioning_signal_does_not_inject_keys() {
        // Doc snippet inside a string contains `signal(...)` — must
        // not inject anything into the keys.
        let c = ctrl(
            "export default controller({\n  \
               setup: () => {\n    \
                 const name = signal('ada')\n    \
                 const doc = \"use signal(count) to track\" \n    \
                 return { name }\n  \
               },\n});",
        );
        let v = extract(&c);
        let names: Vec<&str> = v.iter().map(|(n, _)| n.as_str()).collect();
        assert!(!names.contains(&"count"), "string leaked: {names:?}");
        assert!(
            names.contains(&"name"),
            "real return key missing: {names:?}"
        );
    }
}
