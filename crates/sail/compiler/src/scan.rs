//! Shared byte-scan primitives for the compiler's textual passes.
//!
//! Several passes (parse, render, check, emit, lower) walk raw `.ts`/expr
//! source byte-by-byte. The lexical leaves — "is this byte an identifier
//! char?" — were re-defined in 8 places; this is their one home. They are
//! pure, stable character-class predicates with no pass-specific knowledge.
//!
//! The bounded, string- and comment-aware `skip_trivia` / `skip_balanced` are
//! the canonical scanners. Bounded `(bytes, i, end)` is the general form;
//! unbounded callers pass `end = bytes.len()` (js_scan keeps a thin wrapper).

/// A JavaScript identifier start byte: ASCII letter, `_`, or `$`.
#[inline]
pub(crate) fn is_ident_start(c: u8) -> bool {
    c == b'_' || c == b'$' || c.is_ascii_alphabetic()
}

/// A JavaScript identifier continuation byte: an [`is_ident_start`] byte or a digit.
#[inline]
pub(crate) fn is_ident_cont(c: u8) -> bool {
    is_ident_start(c) || c.is_ascii_digit()
}

/// Advance past whitespace, `//` line comments, and `/* … */` block comments
/// within `[i, end)`. Idempotent at non-trivia bytes.
pub(crate) fn skip_trivia(bytes: &[u8], mut i: usize, end: usize) -> usize {
    loop {
        while i < end && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i + 1 < end && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            while i < end && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if i + 1 < end && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < end && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i = (i + 2).min(end);
            continue;
        }
        break;
    }
    i
}

/// Can a `/` at `i` open a REGEX LITERAL rather than a division?
///
/// JavaScript's one genuinely context-dependent token. Every byte-walking pass
/// in this compiler used to answer "division, always" — so
/// `v.match(/cubic-bezier\(([^)]+)\)/)` fed its `(`/`)`/`[`/`]` to the
/// bracket-depth counters as real brackets, the counts went negative, and the
/// controller's `return {...}` extraction silently produced an EMPTY scope. The
/// template then blamed its first binding ("Identifier 'rail' is not declared
/// in scope. Available: ctx.") — a phantom error pointing away from the cause.
///
/// The standard lexical heuristic: a regex may start wherever an *operand* may
/// start. It may not directly follow something that ends an operand — an
/// identifier, a literal, or a closing `)`/`]`/`}`. The keyword exceptions
/// (`return /re/`, `typeof /re/`, …) are identifiers that are NOT operands.
/// Lookback skips whitespace only; a `/` that ends a `*/` comment is read as
/// regex-permitting, which is the safe direction (a division right after a
/// block comment is not a shape this compiler's inputs take).
pub(crate) fn regex_allowed_at(bytes: &[u8], i: usize) -> bool {
    let mut k = i;
    while k > 0 && bytes[k - 1].is_ascii_whitespace() {
        k -= 1;
    }
    if k == 0 {
        return true;
    }
    let prev = bytes[k - 1];
    if is_ident_cont(prev) {
        // An identifier or numeric literal ends an operand — UNLESS it is one
        // of the keywords a regex may legally follow.
        let mut s = k;
        while s > 0 && is_ident_cont(bytes[s - 1]) {
            s -= 1;
        }
        return matches!(
            std::str::from_utf8(&bytes[s..k]).unwrap_or(""),
            "return"
                | "typeof"
                | "instanceof"
                | "in"
                | "of"
                | "case"
                | "do"
                | "else"
                | "yield"
                | "await"
                | "delete"
                | "void"
                | "new"
                | "throw"
        );
    }
    // `)`/`]` close an operand; `}` is ambiguous (block vs object literal) and
    // is read as operand-closing, the conservative direction — mis-reading an
    // object literal's `}` would break far more than it fixes. Quote bytes
    // close a string operand.
    !matches!(prev, b')' | b']' | b'}' | b'"' | b'\'' | b'`')
}

/// At a `/` that [`regex_allowed_at`] blesses, walk the regex literal and
/// return the offset just past its flags. Returns `None` when the literal
/// doesn't terminate before a newline or `end` — an unterminated regex is far
/// more likely a division we misjudged, so the caller keeps its old behavior.
///
/// Handles `\` escapes and `[...]` character classes, inside which `/` is a
/// literal slash (`/[a/b]/`) — the reason a naive "scan to the next `/`" is
/// not enough.
pub(crate) fn skip_regex(bytes: &[u8], i: usize, end: usize) -> Option<usize> {
    if i >= end || bytes[i] != b'/' {
        return None;
    }
    // `//` and `/*` are ALWAYS comments — an empty regex is spelled `/(?:)/`.
    // Without this guard every line comment scans as a zero-width regex whose
    // "flags" eat the comment's first word.
    if i + 1 < end && matches!(bytes[i + 1], b'/' | b'*') {
        return None;
    }
    let mut k = i + 1;
    let mut in_class = false;
    while k < end {
        match bytes[k] {
            b'\\' => k += 2,
            b'\n' => return None,
            b'[' => {
                in_class = true;
                k += 1;
            }
            b']' if in_class => {
                in_class = false;
                k += 1;
            }
            b'/' if !in_class => {
                k += 1;
                while k < end && is_ident_cont(bytes[k]) {
                    k += 1;
                }
                return Some(k);
            }
            _ => k += 1,
        }
    }
    None
}

/// If a regex literal starts at `i`, return the offset past it; else `None`.
/// The one call the byte-walking passes make — it bundles the "may a regex
/// start here?" lookback with the literal scan so no caller can forget half.
pub(crate) fn try_skip_regex(bytes: &[u8], i: usize, end: usize) -> Option<usize> {
    if i >= end || bytes[i] != b'/' || !regex_allowed_at(bytes, i) {
        return None;
    }
    skip_regex(bytes, i, end)
}

/// At `'`, `"`, or `` ` ``, return the offset just past the closing quote.
///
/// Template interpolations `${…}` are scanned as JS (nested strings, regex,
/// comments, nested templates). A backtick inside `${}` must not terminate
/// the outer template — otherwise `</controller>` after that inner backtick
/// is read as the SFC close.
pub(crate) fn skip_string_or_template(bytes: &[u8], i: usize, end: usize) -> usize {
    if i >= end {
        return i;
    }
    match bytes[i] {
        b'\'' | b'"' => skip_quoted(bytes, i, end, bytes[i]),
        b'`' => skip_template(bytes, i, end),
        _ => i,
    }
}

fn skip_quoted(bytes: &[u8], i: usize, end: usize, q: u8) -> usize {
    let mut k = i + 1;
    while k < end {
        let c = bytes[k];
        if c == b'\\' {
            k = (k + 2).min(end);
            continue;
        }
        if c == q {
            return k + 1;
        }
        k += 1;
    }
    end
}

fn skip_template(bytes: &[u8], i: usize, end: usize) -> usize {
    let mut k = i + 1;
    while k < end {
        let c = bytes[k];
        if c == b'\\' {
            k = (k + 2).min(end);
            continue;
        }
        if c == b'`' {
            return k + 1;
        }
        if c == b'$' && k + 1 < end && bytes[k + 1] == b'{' {
            k = skip_template_interpolation(bytes, k + 2, end);
            continue;
        }
        k += 1;
    }
    end
}

fn skip_template_interpolation(bytes: &[u8], i: usize, end: usize) -> usize {
    let mut k = i;
    let mut depth = 0i32;
    while k < end {
        match bytes[k] {
            b'\'' | b'"' | b'`' => k = skip_string_or_template(bytes, k, end),
            b'/' if k + 1 < end && bytes[k + 1] == b'/' => {
                while k < end && bytes[k] != b'\n' {
                    k += 1;
                }
            }
            b'/' if k + 1 < end && bytes[k + 1] == b'*' => {
                k += 2;
                while k + 1 < end && !(bytes[k] == b'*' && bytes[k + 1] == b'/') {
                    k += 1;
                }
                k = (k + 2).min(end);
            }
            b'/' => k = try_skip_regex(bytes, k, end).unwrap_or(k + 1),
            b'{' => {
                depth += 1;
                k += 1;
            }
            b'}' => {
                if depth == 0 {
                    return k + 1;
                }
                depth -= 1;
                k += 1;
            }
            _ => k += 1,
        }
    }
    end
}

/// At a `(`, `{`, or `[`, return the offset just past the matching closer,
/// within `[i, end)`. String/template-literal contents and `//`+`/* */`
/// comments are skipped. Returns `i + 1` when `bytes[i]` isn't an opener so
/// callers can defensively step forward.
pub(crate) fn skip_balanced(bytes: &[u8], i: usize, end: usize) -> usize {
    if i >= end {
        return end;
    }
    let (open, close) = match bytes[i] {
        b'(' => (b'(', b')'),
        b'{' => (b'{', b'}'),
        b'[' => (b'[', b']'),
        _ => return i + 1,
    };
    let mut depth = 0i32;
    let mut k = i;
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
            b'"' | b'\'' | b'`' => {
                in_str = Some(c);
                k += 1;
            }
            c if c == open => {
                depth += 1;
                k += 1;
            }
            c if c == close => {
                depth -= 1;
                k += 1;
                if depth == 0 {
                    return k;
                }
            }
            b'/' if k + 1 < end && bytes[k + 1] == b'/' => {
                while k < end && bytes[k] != b'\n' {
                    k += 1;
                }
            }
            b'/' if k + 1 < end && bytes[k + 1] == b'*' => {
                k += 2;
                while k + 1 < end && !(bytes[k] == b'*' && bytes[k + 1] == b'/') {
                    k += 1;
                }
                k = (k + 2).min(end);
            }
            // A regex literal's brackets are TEXT, not structure — counting
            // them corrupts `depth` and walks past the real closer.
            b'/' => k = try_skip_regex(bytes, k, end).unwrap_or(k + 1),
            _ => k += 1,
        }
    }
    end
}

#[cfg(test)]
mod tests {
    use super::*;

    fn past(src: &str) -> Option<usize> {
        let b = src.as_bytes();
        let i = src.find('/').unwrap();
        try_skip_regex(b, i, b.len())
    }

    #[test]
    fn recognises_a_regex_where_an_operand_may_start() {
        // The literal that started this: brackets and parens inside a regex.
        assert_eq!(past(r"v.match(/cubic-bezier\(([^)]+)\)/)"), Some(33));
        assert_eq!(past("= /ab+c/gi"), Some(10));
        assert_eq!(past("return /x/"), Some(10));
        // `/` inside a character class is not the terminator.
        assert_eq!(past("(/[a/b]/)"), Some(8));
    }

    #[test]
    fn leaves_division_alone() {
        assert_eq!(past("a / b"), None);
        assert_eq!(past("total() / 2"), None);
        assert_eq!(past("arr[0] / n"), None);
        assert_eq!(past("'s' / 2"), None);
    }

    #[test]
    fn comments_are_never_regexes() {
        assert_eq!(past("= // note\n"), None);
        assert_eq!(past("= /* note */ 1"), None);
    }

    #[test]
    fn an_unterminated_slash_is_not_claimed() {
        // A lone `/` at line end is far likelier a division we misjudged than
        // a regex — leave the caller's old behavior in place.
        assert_eq!(past("= /oops\nnext"), None);
    }

    #[test]
    fn skip_balanced_walks_over_a_regex_literal() {
        // Without regex awareness the `)` inside `[^)]` closed this call early.
        let src = r"(v.match(/cubic-bezier\(([^)]+)\)/)) rest";
        let b = src.as_bytes();
        assert_eq!(
            skip_balanced(b, 0, b.len()),
            src.find(") rest").unwrap() + 1
        );
    }

    #[test]
    fn skip_string_walks_escaped_quotes() {
        let src = r#"'it\'s' rest"#;
        let b = src.as_bytes();
        assert_eq!(
            skip_string_or_template(b, 0, b.len()),
            src.find(" rest").unwrap()
        );
    }

    #[test]
    fn skip_template_does_not_close_on_inner_backtick() {
        // `${'`'}` must not terminate the outer template.
        let src = "`foo ${'`'} bar` rest";
        let b = src.as_bytes();
        assert_eq!(
            skip_string_or_template(b, 0, b.len()),
            src.find(" rest").unwrap()
        );
    }
}
