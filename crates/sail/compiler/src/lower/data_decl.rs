//! Lowering for `<data name=… source=… live />` — the template's
//! sanctioned bridge to the daemon (see
//! `docs/ship-sail/references/docs/data-subscriptions.md`).
//!
//! `source` is a `ship.*`/`<client>.*` SDK call expression, evaluated with
//! `ship`/`route`/declared clients already in scope as plain locals (the
//! setup body destructures them before any `<data>` seed runs). A `live`
//! declaration derives a cache key from the same "static dot-chain, tail
//! args are tracked key segments" rule `check::query_keys` uses for the
//! controller-side `query()`/`mutation()` primitives — `<data>` sources are
//! a template-level instance of the identical shape, just rooted at `ship`
//! (or a `<client>` alias) instead of an imported binding.

use crate::diagnostics::CompileError;
use crate::emit::writer::js_string;
use crate::ir::{AttrInstr, ElementInstr};
use crate::scan::{is_ident_cont, is_ident_start, skip_balanced, skip_trivia};

#[derive(Debug, Clone)]
pub struct DataDecl {
    pub name: String,
    pub source: String,
    pub live: bool,
    /// `["root", "seg", ...tail-arg-exprs]` as a JS array-literal body —
    /// the `queryKeyed` key-fn's return expression. `None` for a
    /// non-`live` `<data>` (one-shot read, no cache entry, no key).
    pub key_js: Option<String>,
}

/// Extract the `DataDecl` a `<data>` element declares. `el.tag` is assumed
/// already lowercase-matched to `"data"` by the caller.
pub fn extract(el: &ElementInstr) -> Result<DataDecl, CompileError> {
    let mut name: Option<String> = None;
    let mut source: Option<String> = None;
    let mut live = false;
    for a in &el.attrs {
        if let AttrInstr::Static { name: n, value, .. } = a {
            match n.as_str() {
                "name" => name = Some(value.clone()),
                "source" => source = Some(value.clone()),
                "live" => live = true,
                _ => {}
            }
        }
    }
    let name = name.ok_or_else(|| {
        CompileError::new("<data> requires a `name` attribute (e.g. `<data name=\"agents\" source=\"ship.agents.summaries()\" />`)")
            .with_span(el.span)
    })?;
    let source = source.ok_or_else(|| {
        CompileError::new("<data> requires a `source` attribute — a `ship.*` SDK call expression")
            .with_span(el.span)
    })?;
    let key_js = if live {
        Some(derive_key(&source).ok_or_else(|| refuse_not_static(&source, el))?)
    } else {
        None
    };
    Ok(DataDecl { name, source, live, key_js })
}

fn refuse_not_static(source: &str, el: &ElementInstr) -> CompileError {
    CompileError::new(format!(
        "<data source=\"{source}\" live /> isn't a static call the compiler can derive a \
         cache key from — `source` must be a plain dot-chain call rooted at an identifier \
         (`ship.foo.bar(…)`, `myClient.foo.bar(…)`), with no computed member access \
         (`ship[x]`) or optional chaining (`ship?.foo`)."
    ))
    .with_span(el.span)
}

/// Derive a `queryKeyed` key-fn body (`[root, seg, ..., tailArgs]`, as a JS
/// array-literal source string) from a static `root.seg.seg(args)` call
/// expression. `None` when `source` isn't a static dot-chain call.
fn derive_key(source: &str) -> Option<String> {
    let bytes = source.as_bytes();
    let end = bytes.len();
    let mut i = skip_trivia(bytes, 0, end);
    if i >= end || !is_ident_start(bytes[i]) {
        return None;
    }
    let mut segments: Vec<String> = Vec::new();
    let seg_start = i;
    while i < end && is_ident_cont(bytes[i]) {
        i += 1;
    }
    segments.push(source[seg_start..i].to_string());
    loop {
        let after = skip_trivia(bytes, i, end);
        if after < end && bytes[after] == b'.' {
            let name_start = skip_trivia(bytes, after + 1, end);
            if name_start >= end || !is_ident_start(bytes[name_start]) {
                return None;
            }
            let mut k = name_start;
            while k < end && is_ident_cont(bytes[k]) {
                k += 1;
            }
            segments.push(source[name_start..k].to_string());
            i = k;
            continue;
        }
        i = after;
        break;
    }
    if i >= end || bytes[i] != b'(' {
        return None;
    }
    let close = skip_balanced(bytes, i, end);
    // Nothing but trivia may follow the call — a single trailing
    // invocation, no chaining after it.
    if skip_trivia(bytes, close, end) != end {
        return None;
    }
    let args_inner = source[i + 1..close.saturating_sub(1).max(i + 1)].trim();
    let mut parts: Vec<String> = segments.iter().map(|s| js_string(s)).collect();
    if !args_inner.is_empty() {
        for arg in split_top_level_commas(args_inner) {
            let arg = arg.trim();
            if !arg.is_empty() {
                parts.push(arg.to_string());
            }
        }
    }
    Some(format!("[{}]", parts.join(", ")))
}

/// Split `s` on top-level commas (depth-aware — commas inside `(...)`,
/// `{...}`, `[...]`, or a string are not separators).
fn split_top_level_commas(s: &str) -> Vec<String> {
    let bytes = s.as_bytes();
    let end = bytes.len();
    let mut out = Vec::new();
    let mut start = 0;
    let mut i = 0;
    while i < end {
        match bytes[i] {
            b'(' | b'{' | b'[' => i = skip_balanced(bytes, i, end),
            b'"' | b'\'' | b'`' => {
                let quote = bytes[i];
                i += 1;
                while i < end && bytes[i] != quote {
                    i += if bytes[i] == b'\\' { 2 } else { 1 };
                }
                i += 1;
            }
            b',' => {
                out.push(s[start..i].to_string());
                i += 1;
                start = i;
            }
            _ => i += 1,
        }
    }
    if start < end {
        out.push(s[start..end].to_string());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_args_call() {
        assert_eq!(
            derive_key("ship.agents.summaries()").as_deref(),
            Some(r#"["ship", "agents", "summaries"]"#)
        );
    }

    #[test]
    fn one_arg_call() {
        assert_eq!(
            derive_key("ship.agents.detail({id: route.params.id()})").as_deref(),
            Some(r#"["ship", "agents", "detail", {id: route.params.id()}]"#)
        );
    }

    #[test]
    fn computed_member_is_not_static() {
        assert!(derive_key("ship[x].agents()").is_none());
    }

    #[test]
    fn no_call_is_not_static() {
        assert!(derive_key("ship.agents").is_none());
    }

    #[test]
    fn chained_call_after_call_is_not_static() {
        assert!(derive_key("ship.agents.summaries().filter()").is_none());
    }
}
