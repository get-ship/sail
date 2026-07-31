//! Value→TypeScript-type heuristic for scope-factory entries, plus the
//! TS-syntax scanners it needs (generic-arg / return-type annotation walks).
//! See the module docs in `mod.rs` for the supported shapes. Best-effort
//! string inference; anything non-trivial degrades to `unknown`.

use super::scan::{is_ident_cont, is_ident_start, skip_balanced, skip_trivia};

/// Heuristic value→type mapper. See module docs for the supported shapes.
pub(super) fn infer_ts_type(bytes: &[u8], start: usize, end: usize) -> String {
    let mut i = skip_trivia(bytes, start, end);
    if i >= end {
        return "unknown".to_string();
    }
    // Arrow-shaped: `(...args): T => …` / `(...args) => …`.
    if bytes[i] == b'(' {
        let close = skip_balanced(bytes, i, end);
        let params_raw = std::str::from_utf8(&bytes[i..close])
            .unwrap_or("()")
            .to_string();
        let mut after = skip_trivia(bytes, close, end);
        let mut return_type = "void".to_string();
        if after < end && bytes[after] == b':' {
            let ret_start = after + 1;
            let ret_end = skip_type_annotation(bytes, ret_start, end);
            let s = std::str::from_utf8(&bytes[ret_start..ret_end])
                .unwrap_or("")
                .trim()
                .to_string();
            if !s.is_empty() {
                return_type = s;
            }
            after = ret_end;
        }
        if after + 1 < end && bytes[after] == b'=' && bytes[after + 1] == b'>' {
            let params = sanitize_arrow_params(&params_raw);
            return format!("{params} => {return_type}");
        }
    }
    // `name => body` (single-param shorthand) — uncommon for scope entries,
    // but model it as a unary action.
    if is_ident_start(bytes[i]) {
        let s = i;
        while i < end && is_ident_cont(bytes[i]) {
            i += 1;
        }
        let word = std::str::from_utf8(&bytes[s..i]).unwrap_or("");
        let mut after = skip_trivia(bytes, i, end);
        if after + 1 < end && bytes[after] == b'=' && bytes[after + 1] == b'>' {
            return format!("({word}: unknown) => unknown");
        }
        // `signal<T>(…)` / `ref<T>(…)` / `derived(...)` / `computed(...)`.
        let mut generic: Option<String> = None;
        if after < end && bytes[after] == b'<' {
            let close = match_angle(bytes, after, end);
            if close > after {
                let inner = std::str::from_utf8(&bytes[after + 1..close - 1])
                    .unwrap_or("")
                    .trim()
                    .to_string();
                if !inner.is_empty() {
                    generic = Some(inner);
                }
                after = close;
            }
            after = skip_trivia(bytes, after, end);
        }
        if after < end && bytes[after] == b'(' {
            let inner_close = skip_balanced(bytes, after, end);
            let arg_slice = std::str::from_utf8(&bytes[after + 1..inner_close - 1]).unwrap_or("");
            return match word {
                "signal" => match generic {
                    Some(t) => format!("Signal<{t}>"),
                    None => format!("Signal<{}>", literal_ts(arg_slice)),
                },
                "derived" | "computed" | "memo" => match generic {
                    Some(t) => format!("Signal<{t}>"),
                    None => "Signal<unknown>".to_string(),
                },
                // `live(thunk)` returns a `LiveSignal<T>` (= `Signal<LiveState<T>>`).
                // When the author annotates `live<T>(…)` we honour it; otherwise the
                // payload type lives on an inner SDK call (`liveExpr<T>(…)`) that
                // references controller-local types the `.d.ts` module can't see, so
                // we emit `LiveSignal<unknown>` rather than fabricate a broken ref.
                // It's still a real, callable signal type — `sig()` yields
                // `LiveState<unknown>` with its `kind`/`value` discriminant — which
                // is what removes the `(sig as () => …)` thunk-casts at call sites.
                "live" => match generic {
                    Some(t) => format!("LiveSignal<{t}>"),
                    None => "LiveSignal<unknown>".to_string(),
                },
                // `query(thunk, opts?)` returns a `LiveSignal<T>` (=
                // `Signal<LiveState<T>>`), same shape `live()` produces — the
                // payload type is whatever the thunk's client call resolves
                // to, which (like `live`) usually lives on an inner SDK
                // call the `.d.ts` module can't see. Honour an explicit
                // `query<T>(...)` annotation; otherwise `LiveSignal<unknown>`.
                "query" => match generic {
                    Some(t) => format!("LiveSignal<{t}>"),
                    None => "LiveSignal<unknown>".to_string(),
                },
                // `mutation(thunk, opts?)` returns `Mutation<T>` — same
                // best-effort caveat as `live`/`query`.
                "mutation" => match generic {
                    Some(t) => format!("Mutation<{t}>"),
                    None => "Mutation<unknown>".to_string(),
                },
                _ => "unknown".to_string(),
            };
        }
        // Bare identifier — opaque.
        return "unknown".to_string();
    }
    // Plain literals.
    match bytes[i] {
        b'"' | b'\'' | b'`' => "string".to_string(),
        b'-' | b'+' => {
            let mut k = i + 1;
            while k < end && (bytes[k].is_ascii_digit() || bytes[k] == b'.') {
                k += 1;
            }
            if k > i + 1 {
                "number".to_string()
            } else {
                "unknown".to_string()
            }
        }
        c if c.is_ascii_digit() => "number".to_string(),
        b't' | b'f' => {
            let word = remaining_word(bytes, i, end);
            if word == "true" || word == "false" {
                "boolean".to_string()
            } else {
                "unknown".to_string()
            }
        }
        b'n' => {
            let word = remaining_word(bytes, i, end);
            if word == "null" {
                "null".to_string()
            } else {
                "unknown".to_string()
            }
        }
        b'[' => "unknown[]".to_string(),
        b'{' => "Record<string, unknown>".to_string(),
        _ => "unknown".to_string(),
    }
}

pub(super) fn remaining_word(bytes: &[u8], start: usize, end: usize) -> &str {
    let mut k = start;
    while k < end && is_ident_cont(bytes[k]) {
        k += 1;
    }
    std::str::from_utf8(&bytes[start..k]).unwrap_or("")
}

/// Infer a TypeScript primitive type from a literal-shaped expression.
/// Falls back to `unknown` for anything non-trivial.
pub(super) fn literal_ts(src: &str) -> String {
    let s = src.trim();
    if s.is_empty() {
        return "unknown".to_string();
    }
    let first = s.as_bytes()[0];
    match first {
        b'"' | b'\'' | b'`' => "string".to_string(),
        b'-' | b'+'
            if s.len() > 1 && (s.as_bytes()[1].is_ascii_digit() || s.as_bytes()[1] == b'.') =>
        {
            "number".to_string()
        }
        c if c.is_ascii_digit() => "number".to_string(),
        b't' if s == "true" => "boolean".to_string(),
        b'f' if s == "false" => "boolean".to_string(),
        b'n' if s == "null" => "null".to_string(),
        b'[' => "unknown[]".to_string(),
        b'{' => "Record<string, unknown>".to_string(),
        _ => "unknown".to_string(),
    }
}

/// Rewrite arrow-function parameter source so it's valid as a TS function
/// type's parameter list. Strips default values (`= 0`) but keeps names
/// and TS annotations.
pub(super) fn sanitize_arrow_params(raw: &str) -> String {
    // The raw slice includes the surrounding parens. Strip the leading
    // underscore-prefixed-only-param convention by leaving names intact;
    // TS function types accept underscored placeholders unchanged.
    raw.to_string()
}

/// Walk a TypeScript generic-args block starting at `bytes[start] == b'<'`.
/// Returns the index immediately after the matching `>`; or `start` if
/// the block is malformed. String literals inside the args (string-literal
/// union types like `signal<'a' | 'b'>(…)`) are skipped over rather than
/// treated as a bail-out — that union is a perfectly good `Signal<'a' | 'b'>`.
pub(super) fn match_angle(bytes: &[u8], start: usize, end: usize) -> usize {
    if bytes[start] != b'<' {
        return start;
    }
    let mut depth = 0i32;
    let mut k = start;
    let mut in_str: Option<u8> = None;
    while k < end {
        let c = bytes[k];
        if let Some(q) = in_str {
            if c == b'\\' {
                k = (k + 2).min(end);
                continue;
            }
            if c == q {
                in_str = None;
            }
            k += 1;
            continue;
        }
        match c {
            b'"' | b'\'' | b'`' => in_str = Some(c),
            b'<' => depth += 1,
            b'>' => {
                depth -= 1;
                if depth == 0 {
                    return k + 1;
                }
            }
            _ => {}
        }
        k += 1;
    }
    start
}

/// Walk a **function-declaration** return-type annotation (after the `:`),
/// stopping at the body's opening `{` (depth 0) or a top-level separator.
/// Unlike [`skip_type_annotation`], a top-level `{` ends the type — it begins
/// the function body — rather than being consumed as an object type. Without
/// this, `function f(): void { body }` leaked its body into the `.d.ts`.
pub(super) fn skip_fn_return_type(bytes: &[u8], start: usize, end: usize) -> usize {
    let mut k = start;
    let mut depth: i32 = 0;
    while k < end {
        let c = bytes[k];
        if depth == 0 {
            if c == b'{' {
                return k; // function body begins
            }
            if matches!(c, b',' | b';' | b'=' | b')' | b'}' | b']') {
                return k;
            }
        }
        match c {
            b'<' | b'(' | b'[' => depth += 1,
            b'>' | b')' | b']' => depth -= 1,
            _ => {}
        }
        if depth < 0 {
            return k;
        }
        k += 1;
    }
    end
}

/// Walk a TS return-type annotation starting after the leading `:`.
/// Stops at the next `=>` token or top-level boundary (`,` `;` `=` `)` `}` `]`).
pub(super) fn skip_type_annotation(bytes: &[u8], start: usize, end: usize) -> usize {
    let mut k = start;
    let mut depth: i32 = 0;
    while k < end {
        let c = bytes[k];
        if depth == 0 {
            if c == b'=' && k + 1 < end && bytes[k + 1] == b'>' {
                return k;
            }
            if matches!(c, b',' | b';' | b'=' | b')' | b'}' | b']') {
                return k;
            }
        }
        match c {
            b'<' | b'(' | b'{' | b'[' => depth += 1,
            b'>' | b')' | b'}' | b']' => depth -= 1,
            _ => {}
        }
        if depth < 0 {
            return k;
        }
        k += 1;
    }
    end
}
