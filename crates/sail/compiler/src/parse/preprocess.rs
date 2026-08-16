//! Source preprocessor — the single, Rust-only path. (There is no JS twin:
//! the `@ship/sail-dom` preprocess was removed when every directive was
//! made to round-trip through this crate; deck's bundler and the Node test
//! harness both compile in-process through `sail_compiler`.)
//!
//! Transforms, in order, before the HTML parser sees the source:
//!  1. Quote unquoted brace attributes — `disabled={x || y}` becomes
//!     `disabled="{x || y}"` with `"` inside the expression escaped to `&quot;`.
//!  2. Expand non-void self-closing tags — `<time … />` becomes `<time …></time>`.
//!  3. Lift tag-position `style(…)` calls → atomic `class`es on the open tag.
//!  4. Lower the layout keywords (`<column>`/`<row>`/…) → `<div>` + atomic classes.
//! Steps 3–4 emit deduped CSS that `preprocess` injects as one `<style>`.

use crate::diagnostics::{CompileError, Span};

/// Framework blocks that are not element hosts for Style attachment.
const SKIP_BODY_TAGS: &[&str] = &["scope", "controller", "script", "style", "partial"];

pub fn preprocess(source: &str) -> Result<String, CompileError> {
    Ok(preprocess_with_style(source)?.0)
}

/// Full preprocess plus Style-resolved type tokens per lifted host.
/// Layout measure uses the tokens, not the emitted `s-fontSize*` classes.
pub fn preprocess_with_types(
    source: &str,
) -> Result<(String, Vec<sail_layout::TypeHost>), CompileError> {
    let (html, hosts) = preprocess_with_style(source)?;
    Ok((html, hosts.types))
}

/// Full preprocess plus Style-resolved type and box tokens per lifted host.
pub fn preprocess_with_style(
    source: &str,
) -> Result<(String, sail_layout::StyleHosts), CompileError> {
    preprocess_impl_full(source, true)
}

/// Light normalize only: brace-attr quoting + self-close expansion.
/// No Style / layout / stdlib lowering — author tags and near-author spans
/// for SailInspect tree identity (TOOL-2).
pub fn preprocess_author(source: &str) -> Result<String, CompileError> {
    let s = expand_self_close(&quote_brace_attrs(source));
    Ok(lower_prop_shorthand(&s))
}

/// Normalize only enough for the real HTML CST parse used by `fmt`.
/// Quotes unquoted brace attrs and expands non-void self-closers — no
/// Style/layout/prop desugar (those are compile semantics, not format).
pub fn prepare_fmt(source: &str) -> String {
    expand_self_close(&quote_brace_attrs(source))
}

/// The full pipeline with the stdlib-element pass switchable — element
/// sources compile WITHOUT it (no element-in-element; no recursion when the
/// registry compiles the stdlib with this same crate).
pub(crate) fn preprocess_impl(source: &str, expand_elements: bool) -> Result<String, CompileError> {
    Ok(preprocess_impl_full(source, expand_elements)?.0)
}

pub(crate) fn preprocess_impl_full(
    source: &str,
    expand_elements: bool,
) -> Result<(String, sail_layout::StyleHosts), CompileError> {
    // SPEC-PARTIAL-1: expand in-file `<partial>` BEFORE quote_brace_attrs so
    // bare param reads inside attr values (`token={token}`) are still
    // identifiers — quoting would bury them in `"{token}"` and block
    // substitution. After expand, the tree is ordinary markup.
    let s = super::partials::expand_partials(source)?;
    // Normalize, then lift tag-position `style(…)` → class (before stdlib so
    // use-site paint is class attrs when native hosts stamp). Layout keywords
    // lower after paint classes exist.
    let s = expand_self_close(&quote_brace_attrs(&s));
    let s = lower_prop_shorthand(&s);
    // Type-keyed color-group props so `bg(tone)` / `bg(shade)` parse as bridge paths.
    let group_props = super::scope_block::color_group_typed_prop_names(&s);
    // Scope Style consts (`const filled = style(…)`) resolve Refs in host Style.
    let (s, style_css, hosts) = spinnaker::with_bound_group_props(&group_props, || {
        let style_env = super::scope_block::style_env_from_source(&s);
        lift_tag_style(&s, &style_env)
    })?;
    refuse_body_style_statements(&s)?;
    // L1 stdlib elements: splice structure; skins live in framework sheet.
    let (s, elements_css) = if expand_elements {
        lower_stdlib_elements(&s)?
    } else {
        (s, String::new())
    };
    // Host display override before lower_layout rewrites row/column.
    let host_display_css = derive_host_display(&s);
    let s = merge_duplicate_class_attrs(&s);
    // Participation is Style only (`style({ grow(1).size({ w: full }) })`).
    // Residual bare attr lower deleted (CHANNELS Phase V DEEMBALM) — lint
    // `sail-participation-style` refuses bare grow/shrink/w/min-w/max-w.
    let (s, layout_css) = lower_layout(&s);
    // ORDER: skin classes FIRST, author classes AFTER (use-site wins).
    let css = dedup_css_rules(&format!(
        "{elements_css}{host_display_css}{style_css}{layout_css}"
    ));
    let html = if css.is_empty() {
        s
    } else {
        format!("<style>{css}</style>\n{s}")
    };
    Ok((html, hosts))
}

/// Find every tag-position Style product expression in raw `.sail` source,
/// with the byte offset of the expression start. Used by `(--x)` / `bind(name)`
/// resolution (`check::binding`) against author source.
pub(crate) fn find_sx_attr_values(source: &str) -> Vec<(usize, &str)> {
    let mut out = Vec::new();
    for_each_tag_style(source, |abs_start, expr| {
        out.push((abs_start, expr));
    });
    out
}

/// Walk open tags; invoke `f(expr_abs_start, expr)` for each tag-position
/// `style(…)` (+ `.when` chains).
fn for_each_tag_style<'a>(source: &'a str, mut f: impl FnMut(usize, &'a str)) {
    let bytes = source.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if source[i..].starts_with("<!--") {
            match source[i..].find("-->") {
                Some(end) => {
                    i += end + 3;
                    continue;
                }
                None => break,
            }
        }
        if source[i..].starts_with("<style") {
            if let Some(rel) = source[i..].find("</style>") {
                i += rel + "</style>".len();
                continue;
            }
            break;
        }
        if bytes[i] == b'<' && i + 1 < bytes.len() {
            if bytes[i + 1] == b'/' {
                i += 2;
                continue;
            }
            if !bytes[i + 1].is_ascii_alphabetic() {
                i += 1;
                continue;
            }
            let tag_start = i + 1;
            let mut tag_end = tag_start;
            while tag_end < bytes.len()
                && (bytes[tag_end].is_ascii_alphanumeric()
                    || bytes[tag_end] == b'-'
                    || bytes[tag_end] == b'_'
                    || bytes[tag_end] == b':')
            {
                tag_end += 1;
            }
            let tag = &source[tag_start..tag_end];
            let mut k = tag_end;
            let mut quote: Option<u8> = None;
            while k < bytes.len() {
                match (quote, bytes[k]) {
                    (Some(q), b) if b == q => quote = None,
                    (None, b'"') | (None, b'\'') => quote = Some(bytes[k]),
                    (None, b'>') => break,
                    _ => {}
                }
                k += 1;
            }
            if k >= bytes.len() {
                break;
            }
            let gt_pos = k;
            let body_start = k + 1;
            if SKIP_BODY_TAGS.contains(&tag) {
                if let Some(end) = matching_close(source, body_start, tag) {
                    i = end;
                    continue;
                }
                i = body_start;
                continue;
            }
            if let Some((lo, _hi, expr)) = find_tag_style_call(source, tag_end, gt_pos) {
                f(lo, expr);
            }
            i = body_start;
            continue;
        }
        let mut n = i + 1;
        while n < bytes.len() && (bytes[n] & 0xC0) == 0x80 {
            n += 1;
        }
        i = n;
    }
}

fn is_void_tag(tag: &str) -> bool {
    crate::l1_manifest::is_author_void(tag)
}

/// Lift tag-position `style(…)` (+ `.when` chains) onto the open tag as
/// `class="s-…"` (+ motion attrs), stripping the call from the open tag.
fn lift_tag_style(
    source: &str,
    env: &spinnaker::StyleEnv,
) -> Result<(String, String, sail_layout::StyleHosts), CompileError> {
    struct Lift {
        gt_pos: usize,
        call_lo: usize,
        call_hi: usize,
        classes: String,
        motions: Vec<(String, String)>,
    }
    let bytes = source.as_bytes();
    let mut lifts: Vec<Lift> = Vec::new();
    let mut css = String::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut hosts = sail_layout::StyleHosts::default();
    let mut i = 0;
    while i < bytes.len() {
        if source[i..].starts_with("<!--") {
            match source[i..].find("-->") {
                Some(end) => {
                    i += end + 3;
                    continue;
                }
                None => break,
            }
        }
        if source[i..].starts_with("<style") {
            if let Some(rel) = source[i..].find("</style>") {
                i += rel + "</style>".len();
                continue;
            }
            break;
        }
        if bytes[i] == b'<' && i + 1 < bytes.len() {
            if bytes[i + 1] == b'/' {
                i += 2;
                continue;
            }
            if !bytes[i + 1].is_ascii_alphabetic() {
                i += 1;
                continue;
            }
            let tag_start = i + 1;
            let mut tag_end = tag_start;
            while tag_end < bytes.len()
                && (bytes[tag_end].is_ascii_alphanumeric()
                    || bytes[tag_end] == b'-'
                    || bytes[tag_end] == b'_'
                    || bytes[tag_end] == b':')
            {
                tag_end += 1;
            }
            let tag = &source[tag_start..tag_end];
            let mut k = tag_end;
            let mut quote: Option<u8> = None;
            while k < bytes.len() {
                match (quote, bytes[k]) {
                    (Some(q), b) if b == q => quote = None,
                    (None, b'"') | (None, b'\'') => quote = Some(bytes[k]),
                    (None, b'>') => break,
                    _ => {}
                }
                k += 1;
            }
            if k >= bytes.len() {
                break;
            }
            let gt_pos = k;
            let body_start = k + 1;
            if SKIP_BODY_TAGS.contains(&tag) {
                if let Some(end) = matching_close(source, body_start, tag) {
                    i = end;
                    continue;
                }
                i = body_start;
                continue;
            }
            if let Some(bad) = find_bad_tag_style_token(source, tag_end, gt_pos) {
                return Err(CompileError::new(
                    "style in tag position must be a call: style({ … })  \
                     or  style(name) / style(partial, …)  — not bare `style`, `style=`, or `style={…}`",
                )
                .with_span(Span::new(bad, bad + 5))
                .with_category("spinnaker"));
            }
            if let Some((call_lo, call_hi, expr)) = find_tag_style_call(source, tag_end, gt_pos) {
                refuse_text_on_non_text_host(tag, expr, env, call_lo, call_hi)?;
                let (classes, motions) = lower_sx_value(expr, env, &mut css, &mut seen)
                    .map_err(|diags| style_error(diags, call_lo, call_hi))?;
                let val = strip_outer_braces(expr.trim());
                let (style_ir, _) = spinnaker::parse(val);
                let resolved_type = spinnaker::resolve_type(&style_ir, env);
                if let Some(host) = sail_layout::TypeHost::from_resolved(&classes, &resolved_type) {
                    hosts.types.push(host);
                }
                let resolved_box = sail_layout::resolve_box(&style_ir, env);
                if let Some(host) = sail_layout::BoxHost::from_resolved(&classes, &resolved_box) {
                    hosts.boxes.push(host);
                }
                lifts.push(Lift {
                    gt_pos,
                    call_lo,
                    call_hi,
                    classes,
                    motions,
                });
            }
            i = body_start;
            continue;
        }
        let mut n = i + 1;
        while n < bytes.len() && (bytes[n] & 0xC0) == 0x80 {
            n += 1;
        }
        i = n;
    }

    if lifts.is_empty() {
        return Ok((source.to_string(), css, hosts));
    }

    let mut out = source.to_string();
    for lift in lifts.into_iter().rev() {
        let mut lo = lift.call_lo;
        let mut hi = lift.call_hi;
        // Keep one space when the call sits before `/>` or `>`. Stripping
        // it glues the previous attr to the slash (`multiple/>`) and the
        // HTML parser treats `/` as self-close mid-tag.
        let rest = out[hi..].trim_start();
        let keep_sep = rest.starts_with('/') || rest.starts_with('>');
        while lo > 0 && out.as_bytes()[lo - 1] == b' ' {
            lo -= 1;
        }
        if keep_sep && lo < lift.call_lo {
            lo += 1;
        }
        let rest = out[hi..].trim_start();
        if rest.starts_with('>') || rest.starts_with('/') {
            while hi < out.len() && out.as_bytes()[hi].is_ascii_whitespace() {
                hi += 1;
            }
        }
        out.replace_range(lo..hi, "");
        let inject_at = if lift.gt_pos >= hi {
            lift.gt_pos - (hi - lo)
        } else if lift.gt_pos >= lo {
            lo
        } else {
            lift.gt_pos
        };
        let mut inject = String::new();
        inject.push_str(" class=\"");
        inject.push_str(&lift.classes);
        inject.push('"');
        push_motion_attrs(&mut inject, &lift.motions);
        out.insert_str(inject_at, &inject);
    }
    Ok((out, css, hosts))
}

/// Body-nested Style product is not a statement — generic unknown-statement error.
fn refuse_body_style_statements(source: &str) -> Result<(), CompileError> {
    let bytes = source.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if source[i..].starts_with("<!--") {
            match source[i..].find("-->") {
                Some(end) => {
                    i += end + 3;
                    continue;
                }
                None => break,
            }
        }
        if bytes[i] == b'<' && i + 1 < bytes.len() {
            if bytes[i + 1] == b'/' {
                i += 2;
                continue;
            }
            if !bytes[i + 1].is_ascii_alphabetic() {
                i += 1;
                continue;
            }
            let tag_start = i + 1;
            let mut tag_end = tag_start;
            while tag_end < bytes.len()
                && (bytes[tag_end].is_ascii_alphanumeric()
                    || bytes[tag_end] == b'-'
                    || bytes[tag_end] == b'_'
                    || bytes[tag_end] == b':')
            {
                tag_end += 1;
            }
            let tag = &source[tag_start..tag_end];
            let mut k = tag_end;
            let mut quote: Option<u8> = None;
            while k < bytes.len() {
                match (quote, bytes[k]) {
                    (Some(q), b) if b == q => quote = None,
                    (None, b'"') | (None, b'\'') => quote = Some(bytes[k]),
                    (None, b'>') => break,
                    _ => {}
                }
                k += 1;
            }
            if k >= bytes.len() {
                break;
            }
            let self_closing = k > 0 && bytes[k - 1] == b'/';
            let body_start = k + 1;
            if self_closing || is_void_tag(tag) || SKIP_BODY_TAGS.contains(&tag) {
                if SKIP_BODY_TAGS.contains(&tag) && !self_closing {
                    if let Some(end) = matching_close(source, body_start, tag) {
                        i = end;
                        continue;
                    }
                }
                i = body_start;
                continue;
            }
            let body = &source[body_start..];
            if let Some((rel_start, rel_end, _expr)) = try_style_statement(body) {
                let abs_lo = body_start + rel_start;
                let abs_hi = body_start + rel_end;
                return Err(CompileError::new(
                    "unknown statement — Style attaches as style({…}) on the open tag, \
                     not as a body statement",
                )
                .with_span(Span::new(abs_lo, abs_hi))
                .with_category("spinnaker"));
            }
            i = body_start;
            continue;
        }
        let mut n = i + 1;
        while n < bytes.len() && (bytes[n] & 0xC0) == 0x80 {
            n += 1;
        }
        i = n;
    }
    Ok(())
}

/// Locate `style(…)` (+ chains) in an open-tag attribute region `[tag_end, gt_pos)`.
fn find_tag_style_call(
    source: &str,
    tag_end: usize,
    gt_pos: usize,
) -> Option<(usize, usize, &str)> {
    let bytes = source.as_bytes();
    let mut j = tag_end;
    let mut quote: Option<u8> = None;
    while j < gt_pos {
        match (quote, bytes[j]) {
            (Some(q), b) if b == q => {
                quote = None;
                j += 1;
            }
            (None, b'"') | (None, b'\'') => {
                quote = Some(bytes[j]);
                j += 1;
            }
            (Some(_), _) => j += 1,
            (None, _) => {
                if is_style_word_at(bytes, j) {
                    let after = j + 5;
                    let mut k = after;
                    while k < gt_pos && bytes[k].is_ascii_whitespace() {
                        k += 1;
                    }
                    if k < gt_pos && bytes[k] == b'(' {
                        let rest = &source[j..gt_pos];
                        let end = consume_style_expr(rest)?;
                        let expr = rest[..end].trim_end();
                        if expr.is_empty() {
                            return None;
                        }
                        return Some((j, j + end, expr));
                    }
                }
                j += 1;
            }
        }
    }
    None
}

/// `style` token in open-tag region that is NOT a valid `style(` call.
fn find_bad_tag_style_token(source: &str, tag_end: usize, gt_pos: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut j = tag_end;
    let mut quote: Option<u8> = None;
    while j < gt_pos {
        match (quote, bytes[j]) {
            (Some(q), b) if b == q => {
                quote = None;
                j += 1;
            }
            (None, b'"') | (None, b'\'') => {
                quote = Some(bytes[j]);
                j += 1;
            }
            (Some(_), _) => j += 1,
            (None, _) => {
                if is_style_word_at(bytes, j) {
                    let after = j + 5;
                    let mut k = after;
                    while k < gt_pos && bytes[k].is_ascii_whitespace() {
                        k += 1;
                    }
                    if k < gt_pos && bytes[k] == b'(' {
                        if let Some(end) = consume_style_expr(&source[j..gt_pos]) {
                            j += end;
                            continue;
                        }
                    }
                    return Some(j);
                }
                j += 1;
            }
        }
    }
    None
}

fn is_style_word_at(bytes: &[u8], j: usize) -> bool {
    if j + 5 > bytes.len() || &bytes[j..j + 5] != b"style" {
        return false;
    }
    if j > 0 {
        let p = bytes[j - 1];
        if p.is_ascii_alphanumeric() || p == b'_' || p == b'$' || p == b'-' || p == b':' {
            return false;
        }
    }
    if j + 5 < bytes.len() {
        let n = bytes[j + 5];
        // `style:color` is a colon-prefix directive, not the style keyword.
        if n.is_ascii_alphanumeric() || n == b'_' || n == b'$' || n == b'-' || n == b':' {
            return false;
        }
    }
    true
}

/// First non-ws body content is a Style product statement?
fn try_style_statement(body: &str) -> Option<(usize, usize, &str)> {
    let bytes = body.as_bytes();
    let mut start = 0;
    while start < bytes.len() && bytes[start].is_ascii_whitespace() {
        start += 1;
    }
    if start >= bytes.len() {
        return None;
    }
    if bytes[start] == b'<' || bytes[start] == b'{' {
        return None;
    }
    let rest = &body[start..];
    if !looks_like_style_product(rest) {
        return None;
    }
    let end = consume_style_expr(rest)?;
    let expr = rest[..end].trim_end();
    if expr.is_empty() {
        return None;
    }
    Some((start, start + end, expr))
}

fn looks_like_style_product(s: &str) -> bool {
    let t = s.trim_start();
    t.starts_with("style.")
        || t.starts_with("style(")
        || t.starts_with("match(")
        || t.starts_with("when(")
        || t.starts_with("compound(")
        || t.starts_with('[')
        || is_style_const_method_call(t)
}

fn is_style_const_method_call(s: &str) -> bool {
    let Some((head, rest)) = s.split_once('.') else {
        return false;
    };
    if head.is_empty() || !head.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') {
        return false;
    }
    let r = rest.trim_start().as_bytes();
    let mut i = 0;
    while i < r.len() && (r[i].is_ascii_alphanumeric() || r[i] == b'_' || r[i] == b'-') {
        i += 1;
    }
    if i == 0 {
        return false;
    }
    let mut j = i;
    while j < r.len() && r[j].is_ascii_whitespace() {
        j += 1;
    }
    j < r.len() && r[j] == b'('
}

/// Consume a balanced Style product expression; keep `.method(` chain tails.
fn consume_style_expr(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut i = 0;
    let mut paren = 0i32;
    let mut brack = 0i32;
    let mut brace = 0i32;
    let mut saw_group = false;

    while i < bytes.len() {
        match bytes[i] {
            b'"' | b'\'' => {
                let q = bytes[i];
                i += 1;
                while i < bytes.len() && bytes[i] != q {
                    if bytes[i] == b'\\' && i + 1 < bytes.len() {
                        i += 1;
                    }
                    i += 1;
                }
                if i < bytes.len() {
                    i += 1;
                }
                continue;
            }
            b'(' => {
                paren += 1;
                saw_group = true;
            }
            b')' => {
                paren -= 1;
                if paren < 0 {
                    return None;
                }
            }
            b'[' => {
                brack += 1;
                saw_group = true;
            }
            b']' => {
                brack -= 1;
                if brack < 0 {
                    return None;
                }
            }
            b'{' => {
                brace += 1;
                saw_group = true;
            }
            b'}' => {
                brace -= 1;
                if brace < 0 {
                    return None;
                }
            }
            _ => {}
        }
        i += 1;
        if saw_group && paren == 0 && brack == 0 && brace == 0 {
            let mut j = i;
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b'.' {
                let after_dot = j + 1;
                let mut k = after_dot;
                while k < bytes.len()
                    && (bytes[k].is_ascii_alphanumeric() || bytes[k] == b'_' || bytes[k] == b'-')
                {
                    k += 1;
                }
                if k > after_dot {
                    let mut m = k;
                    while m < bytes.len() && bytes[m].is_ascii_whitespace() {
                        m += 1;
                    }
                    if m < bytes.len() && bytes[m] == b'(' {
                        i = m;
                        continue;
                    }
                }
            }
            return Some(i);
        }
    }
    if saw_group && paren == 0 && brack == 0 && brace == 0 {
        Some(i)
    } else {
        None
    }
}

fn style_error(diags: Vec<spinnaker::StyleDiagnostic>, start: usize, end: usize) -> CompileError {
    let message = diags
        .into_iter()
        .map(|d| d.message())
        .collect::<Vec<_>>()
        .join("; ");
    CompileError::new(format!("Style error: {message}"))
        .with_span(Span::new(start, end))
        .with_category("spinnaker")
}
fn lower_sx_value(
    val: &str,
    env: &spinnaker::StyleEnv,
    css: &mut String,
    seen: &mut std::collections::HashSet<String>,
) -> Result<(String, Vec<(String, String)>), Vec<spinnaker::StyleDiagnostic>> {
    // `quote_brace_attrs` rewrites `sx={…}` → `sx="{…}"` (braces preserved inside).
    let val = strip_outer_braces(val.trim());
    // Sole product path: Style value language (chain + match/when/compound/bind).
    // Motion chain methods (`.enter` / `.exit` / …) return alongside classes.
    spinnaker_css::lower_style_source_with_motion(val, env, css, seen)
}

/// Textish setters may only attach to text-bearing hosts.
///
/// Scope consts are not host-checked at definition (no tag). Attach-time check
/// inspects the **inline** Style IR only — `style(constRef)` is not expanded
/// here (const-split / ref host-check is a follow-up rewrite). Direct
/// `style({ type({ … }) })` / when bodies with textish setters on non-text
/// hosts fail. Host law is compiler-owned (HTML + L1 registry).
fn refuse_text_on_non_text_host(
    tag: &str,
    expr: &str,
    _env: &spinnaker::StyleEnv,
    abs_lo: usize,
    abs_hi: usize,
) -> Result<(), CompileError> {
    if crate::text_host::is_text_host(tag) {
        return Ok(());
    }
    let val = strip_outer_braces(expr.trim());
    let (style, _) = spinnaker::parse(val);
    if !spinnaker::style_is_textish(&style) {
        return Ok(());
    }
    Err(CompileError::new(crate::text_host::text_host_error(tag))
        .with_span(Span::new(abs_lo, abs_hi))
        .with_category("spinnaker"))
}

/// Inject compiler-private `_sail_m_<trigger>="{{…}}"` attrs derived from
/// `animate: { <trigger>: play(…) }`. Not an author channel.
fn push_motion_attrs(out: &mut String, motions: &[(String, String)]) {
    for (trigger, config) in motions {
        // config includes braces: `{ opacity: 'fade' }`
        // attribute value: `{{ opacity: 'fade' }}` (outer pair is the expr wrapper)
        let value = format!("{{{config}}}");
        let escaped = value.replace('"', "&quot;");
        out.push(' ');
        out.push_str("_sail_m_");
        out.push_str(trigger);
        out.push_str("=\"");
        out.push_str(&escaped);
        out.push('"');
    }
}

fn strip_outer_braces(val: &str) -> &str {
    let t = val.trim();
    if t.starts_with('{') && t.ends_with('}') && t.len() >= 2 {
        // Only strip one layer when balanced as a whole expression wrapper.
        let inner = &t[1..t.len() - 1];
        // If outer braces wrap the whole Style expr (from sx={…}), strip them.
        // Keep map literals that aren't the whole expression — Style lists start
        // with `[` or `style`/`match`/`when`/`compound`/ident, not bare `{k:`.
        let inn = inner.trim();
        if inn.starts_with("style.")
            || inn.starts_with("style(")
            || inn.starts_with("match(")
            || inn.starts_with("when(")
            || inn.starts_with("compound(")
            || inn.starts_with('[')
            || inn
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b.is_ascii_whitespace())
            // Style const + motion: `cta.press({ scale: 'press' })`
            || is_ident_dot_call(inn)
        {
            return inn;
        }
    }
    t
}

/// `name.method(…)` — Style const ref with a motion/paint method call.
fn is_ident_dot_call(s: &str) -> bool {
    let Some((head, rest)) = s.split_once('.') else {
        return false;
    };
    !head.is_empty()
        && head.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
        && rest.contains('(')
}

/// Split `hover:before:bg-muted` → prefixes `["hover","before"]`, atom `bg-muted`.
/// Consumes a leading chain of closed prefixes; the remainder is the utility.
#[allow(dead_code)] // kept for diagnostics / non-IR callers
fn split_sx_prefixes(pair: &str) -> (Vec<String>, String) {
    let parts: Vec<&str> = pair.split(':').collect();
    if parts.len() < 2 {
        return (Vec::new(), pair.to_string());
    }
    let mut i = 0usize;
    while i < parts.len() - 1 {
        if sail_lint::rules::style_rule::is_prefix(parts[i]) {
            i += 1;
        } else {
            break;
        }
    }
    if i == 0 {
        return (Vec::new(), pair.to_string());
    }
    let prefixes = parts[..i].iter().map(|s| (*s).to_string()).collect();
    let atom = parts[i..].join(":");
    (prefixes, atom)
}

/// After the `sx` name: optional ws, `=`, optional ws, then a `"`.
/// L1 STDLIB ELEMENT LOWERING — splice each `<tag …>children</tag>` use of a
/// registry element into its compiled structure snippet:
///   - use-site attributes merge onto the snippet root (use-site wins on a
///     name collision; `class` values concatenate),
///   - children replace the snippet's default `<slot></slot>` (named-slot
///     compile-time projection is slice 3; today's stdlib elements use at
///     most the default slot),
///   - the element's skin CSS joins the injected style ONCE per tag.
fn lower_stdlib_elements(source: &str) -> Result<(String, String), CompileError> {
    let registry = crate::stdlib::elements();
    if registry.is_empty() {
        return Ok((source.to_string(), String::new()));
    }
    let mut out = String::with_capacity(source.len());
    let mut css = String::new();
    let mut css_done: std::collections::HashSet<&str> = std::collections::HashSet::new();
    // STDLIB ELEMENT SKINS ARE NOT INLINED. They ship in the framework stylesheet
    // (`element-skins.css`, generated by `sail-compiler framework-css` — the compiler's
    // half of `sail.css`), which the app links once and the browser caches.
    //
    // They used to be spliced into every component's emitted `<style>`, which meant
    // 28.7 KB of CSS embedded in the JAVASCRIPT of a single page using twelve L1
    // components — and duplicated again in the next page's chunk. The comment that
    // used to sit here said so: "interim delivery … moves to the framework sheet with
    // deck style-delivery." It never moved, so it kept charging every component.
    //
    // A stylesheet belongs in a stylesheet. Same call as the contracts sheet, and the
    // same one link (`sail.css`) delivers both. LAYOUT css is different and stays
    // inline: the preprocessor generates only the rules a file actually uses, so it is
    // not in the framework sheet and has nothing to be deduplicated against.
    lower_into(source, registry, &mut out, &mut css, &mut css_done)?;
    Ok((out, css))
}

/// The recursive core of stdlib lowering. Factored out so a nested element
/// (a `<card>` inside a `<card>`) gets ITS children lowered too: the old
/// single pass used `find("</tag>")` — the FIRST close — which paired the
/// inner card's close with the outer card, corrupting the tree. Now the
/// matching close is found depth-aware (elements CAN self-nest), and the
/// children are lowered recursively before they splice into the slot. `css`
/// / `css_done` are threaded so each tag's skin still joins exactly once,
/// no matter how deeply it nests.
#[allow(clippy::only_used_in_recursion)]
fn lower_into(
    source: &str,
    registry: &'static [crate::stdlib::StdlibElement],
    out: &mut String,
    css: &mut String,
    css_done: &mut std::collections::HashSet<&'static str>,
) -> Result<(), CompileError> {
    let bytes = source.as_bytes();
    let mut i = 0;
    'outer: while i < bytes.len() {
        if source[i..].starts_with("<!--") {
            let end = source[i..]
                .find("-->")
                .map(|e| i + e + 3)
                .unwrap_or(bytes.len());
            out.push_str(&source[i..end]);
            i = end;
            continue;
        }
        // <style> blocks pass through UNTOUCHED — CSS text (selectors,
        // /* comments */ mentioning element names) must never be spliced
        // (the sonnet-agent find: a CSS comment naming <label> swallowed
        // the rest of the file).
        if source[i..].starts_with("<style") {
            let end = source[i..]
                .find("</style>")
                .map(|e| i + e + "</style>".len())
                .unwrap_or(bytes.len());
            out.push_str(&source[i..end]);
            i = end;
            continue;
        }
        if bytes[i] == b'<' && i + 1 < bytes.len() && bytes[i + 1].is_ascii_alphabetic() {
            for el in registry {
                let tag = el.tag;
                // NATIVE-KIND elements (root tag == element tag, e.g. label
                // → <label>) are skin + manifest only — the tag already IS
                // the element; splicing would rewrite every real HTML use.
                // But the element's HOST paint (root class + the atomic sx
                // classes its Style lowered to, the same string a spliced
                // root carries) reaches real tags only if the use site
                // carries those classes — nothing else ever stamps them.
                // Stamp them onto the open tag here, merging any author
                // `class`; use-site props stay ordinary attributes, so the
                // skin's `[attr=value]` selectors keep matching.
                if el.out.native {
                    if !source[i + 1..].starts_with(tag) {
                        continue;
                    }
                    let after = i + 1 + tag.len();
                    let boundary = bytes
                        .get(after)
                        .is_some_and(|&b| b.is_ascii_whitespace() || b == b'>');
                    if !boundary {
                        continue;
                    }
                    // Open-tag end, quote-aware.
                    let mut k = after;
                    let mut quote: Option<u8> = None;
                    while k < bytes.len() {
                        match (quote, bytes[k]) {
                            (Some(q), b) if b == q => quote = None,
                            (None, b'"') | (None, b'\'') => quote = Some(bytes[k]),
                            (None, b'>') => break,
                            _ => {}
                        }
                        k += 1;
                    }
                    if k >= bytes.len() {
                        break;
                    }
                    let open_tag = &source[i..=k];
                    // `[anatomy]` marks an element INTERNAL (a part rendered
                    // elsewhere whose paint already rides its own classes) —
                    // host paint on top would apply twice. Copy the whole
                    // subtree through untouched.
                    if open_tag_attr(open_tag, "anatomy").is_some() {
                        let end = if crate::emit::element::is_void_element(tag) {
                            k + 1
                        } else {
                            matching_close(source, k + 1, tag).unwrap_or(k + 1)
                        };
                        out.push_str(&source[i..end]);
                        i = end;
                        continue 'outer;
                    }
                    // Freeform `<input type=…>` products that have own tags are
                    // not Sail API — hard error with redirect (no dual path).
                    if tag == "input" {
                        if let Some(ty) = open_tag_attr(open_tag, "type") {
                            let ty = ty.trim_matches(['"', '\'']);
                            if let Some(instead) = crate::authoring_bans::banned_input_type(ty) {
                                return Err(CompileError::new(format!(
                                    "`<input type=\"{ty}\">` is not part of the Sail API. Use {instead} instead."
                                ))
                                .with_span(Span::new(i, k + 1)));
                            }
                        }
                    }
                    out.push_str(&stamp_native_host(
                        open_tag,
                        &el.out.host_classes,
                        &el.out.default_attrs,
                        &el.out.bound_props,
                    ));
                    // Same bookkeeping as the spliced path: the tag was seen.
                    css_done.insert(tag);
                    // Inner pack (button's `<row cluster>`): wrap use-site
                    // children. Host stays stamped — not spliced. Label-style
                    // extras stay off the use site (`inner_pack` is None).
                    if let Some(pack_open) = el.out.inner_pack.as_deref() {
                        if !crate::emit::element::is_void_element(tag) {
                            if let Some(end) = matching_close(source, k + 1, tag) {
                                let close = format!("</{tag}>");
                                let child_end = end.saturating_sub(close.len());
                                let children = if child_end > k + 1 {
                                    &source[k + 1..child_end]
                                } else {
                                    ""
                                };
                                let mut lowered = String::new();
                                lower_into(children, registry, &mut lowered, css, css_done)?;
                                out.push_str(pack_open);
                                out.push_str(&lowered);
                                if let Some(pack_tag) = pack_open_tag_name(pack_open) {
                                    out.push_str("</");
                                    out.push_str(pack_tag);
                                    out.push('>');
                                }
                                i = child_end;
                                continue 'outer;
                            }
                        }
                    }
                    i = k + 1;
                    continue 'outer;
                }
                if source[i + 1..].starts_with(tag) {
                    let after = i + 1 + tag.len();
                    let boundary = bytes
                        .get(after)
                        .is_some_and(|&b| b.is_ascii_whitespace() || b == b'>');
                    if !boundary {
                        continue;
                    }
                    // Open-tag end, quote-aware.
                    let mut k = after;
                    let mut quote: Option<u8> = None;
                    while k < bytes.len() {
                        match (quote, bytes[k]) {
                            (Some(q), b) if b == q => quote = None,
                            (None, b'"') | (None, b'\'') => quote = Some(bytes[k]),
                            (None, b'>') => break,
                            _ => {}
                        }
                        k += 1;
                    }
                    if k >= bytes.len() {
                        break;
                    }
                    let use_attrs = source[after..k]
                        .trim()
                        .trim_end_matches('/')
                        .trim()
                        .to_string();
                    let self_closed = source[i..k].trim_end().ends_with('/');
                    // Author-void L1 (`<avatar />`, `<separator>`) has no
                    // children. expand_self_close leaves those tags unpaired,
                    // so splice must not wait for `</tag>`.
                    if self_closed || crate::l1_manifest::is_author_void(tag) {
                        let mut end = k + 1;
                        if !self_closed {
                            let rest = source[end..].trim_start();
                            let closer = format!("</{tag}>");
                            if rest.starts_with(&closer) {
                                let ws = source[end..].len() - rest.len();
                                end += ws + closer.len();
                            }
                        }
                        css_done.insert(tag);
                        out.push_str(&splice_element(&el.out, &use_attrs, ""));
                        i = end;
                        continue 'outer;
                    }
                    // Matching close, DEPTH-AWARE: skip the closes of nested
                    // same-tag opens (expand_self_close already paired the
                    // self-closing forms, so only `<tag …>` opens count).
                    let open_prefix = format!("<{tag}");
                    let close = format!("</{tag}>");
                    let mut depth = 1usize;
                    let mut scan = k + 1;
                    let close_start = loop {
                        if scan >= bytes.len() {
                            break None;
                        }
                        // Byte-level compares — `scan` advances by one byte in
                        // the fallthrough, so it can land mid-multibyte-char;
                        // slicing `source[scan..]` there would panic (the ⌘/—/°
                        // in real pages). `bytes` slicing is boundary-free.
                        if bytes[scan..].starts_with(close.as_bytes()) {
                            depth -= 1;
                            if depth == 0 {
                                break Some(scan);
                            }
                            scan += close.len();
                        } else if bytes[scan..].starts_with(open_prefix.as_bytes())
                            && bytes
                                .get(scan + open_prefix.len())
                                .is_some_and(|&b| b.is_ascii_whitespace() || b == b'>')
                        {
                            depth += 1;
                            scan += open_prefix.len();
                        } else {
                            scan += 1;
                        }
                    };
                    let Some(close_start) = close_start else {
                        break;
                    };
                    let children = &source[k + 1..close_start];
                    let end = close_start + close.len();

                    // The skin is NOT joined — it lives in the framework stylesheet
                    // (see `lower_stdlib_elements`). `css_done` is still marked so the
                    // bookkeeping stays honest for anything else keyed on it.
                    css_done.insert(tag);
                    // Lower nested elements in the children, then splice.
                    let mut lowered = String::new();
                    lower_into(children, registry, &mut lowered, css, css_done)?;
                    out.push_str(&splice_element(&el.out, &use_attrs, &lowered));
                    i = end;
                    continue 'outer;
                }
            }
        }
        let next = bytes[i + 1..]
            .iter()
            .position(|&b| b == b'<')
            .map(|n| i + 1 + n)
            .unwrap_or(bytes.len());
        out.push_str(&source[i..next]);
        i = next;
    }
    Ok(())
}

fn pack_open_tag_name(open: &str) -> Option<&str> {
    let rest = open.strip_prefix('<')?;
    let name = rest
        .split(|c: char| c.is_ascii_whitespace() || c == '>' || c == '/')
        .next()
        .unwrap_or("");
    if name.is_empty() { None } else { Some(name) }
}

/// Byte offset just past the close tag matching the `<tag …>` open whose
/// open tag ends at `from` — depth-aware over nested same-tag opens, byte
/// compares so a multibyte char is never sliced mid-codepoint (mirrors the
/// close scan in `lower_into`). None when the source runs out first.
fn matching_close(source: &str, from: usize, tag: &str) -> Option<usize> {
    let bytes = source.as_bytes();
    let open_prefix = format!("<{tag}");
    let close = format!("</{tag}>");
    let mut depth = 1usize;
    let mut scan = from;
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
                .is_some_and(|&b| b.is_ascii_whitespace() || b == b'>')
        {
            depth += 1;
            scan += open_prefix.len();
        } else {
            scan += 1;
        }
    }
    None
}

/// Sentinel in `active` when a use-site attr is a dynamic expression (`{…}`).
/// Freeform dual-gate CSS still needs every arm for that axis — the attr value
/// is only known at runtime.
const FREEFORM_DYNAMIC: &str = "\0dyn";

/// `head` is the class segment after the axis name (before the first `-` prop).
/// Nested match/compound emit `checkboxchecked`, `checkboxsizemd`, etc.
fn freeform_axis_value_matches(head: &str, want: &str) -> bool {
    let head_l = head.to_ascii_lowercase();
    let want_l = want.to_ascii_lowercase();
    if head_l == want_l {
        return true;
    }
    if !head_l.starts_with(&want_l) {
        return false;
    }
    let rem = &head_l[want_l.len()..];
    rem.is_empty()
        || rem.starts_with("checked")
        || rem.starts_with("disabled")
        || rem.starts_with("indeterminate")
        || rem.starts_with("invalid")
        || rem.starts_with("size")
        || rem.starts_with("variant")
        || rem.starts_with("tone")
}

/// Author already names a width channel on `<select>` — participation
/// `w` / `min-w` / `max-w`, or Style `size({ w / min-w / max-w })`.
fn author_sets_select_width(attrs: &[(String, String)]) -> bool {
    for (name, value) in attrs {
        match name.as_str() {
            "w" | "min-w" | "max-w" => return true,
            "class" => {
                let v = value.trim_matches(['"', '\'']);
                for c in v.split_whitespace() {
                    let cl = c.to_ascii_lowercase();
                    if cl.starts_with("s-w")
                        || cl.starts_with("s-minw")
                        || cl.starts_with("s-maxw")
                        || cl.starts_with("s-sizew")
                        || cl.starts_with("s-sizeminw")
                        || cl.starts_with("s-sizemaxw")
                    {
                        return true;
                    }
                }
            }
            _ => {}
        }
    }
    false
}

/// Whether a freeform host class is needed for the active attr set.
///
/// Host Style lowers *every* `match` / `compound` arm into classes. Freeform
/// dual-gate CSS only needs the arms for attrs actually on the tag (+ defaults).
/// Keeps base/state classes; drops inactive variant/size/tone/type match arms.
/// (Style match/compound — not CVA string maps.)
///
/// Class names compound: `s-typecheckboxchecked-bgprimary`,
/// `s-typecheckboxsizemd-w…`. The axis value is a *prefix* of the head segment,
/// not the whole head (`checkbox` not `checkboxchecked`).
fn freeform_class_active(class: &str, active: &std::collections::HashMap<String, String>) -> bool {
    // Layout / identity tokens always stay.
    if !class.starts_with("s-") {
        return true;
    }
    let rest = &class[3..];
    // State / pseudo arms (hover, focus, disabled, first/last, group, …).
    const STATE: &[&str] = &[
        "hover-",
        "active-",
        "disabled-",
        "focus-",
        "pressed-",
        "group",
        "after-",
        "open-",
        "first-",
        "last-",
        "current-",
        "forcedcolors",
        "empty-",
        "motionreduce",
        "interactivetrue",
        "selected",
        "invalid",
        "centered",
        // :checked / :indeterminate ride dual-gate pseudo on freeform inputs
        "checked",
        "indeterminate",
    ];
    if STATE.iter().any(|p| rest.starts_with(p)) {
        return true;
    }
    // Attr-gated match arms: s-variant{value}-*, s-size{value}-*, s-type{value}-*, …
    // Nested match emits typecheckboxsizemd / typecheckboxchecked as one head —
    // active value must be a case-insensitive *prefix* of that head.
    const AXES: &[&str] = &[
        "variant",
        "size",
        "type",
        "side",
        "placement",
        "orientation",
        "padding",
        "backdrop",
        "align",
        "justify",
    ];
    for axis in AXES {
        if let Some(after) = rest.strip_prefix(axis) {
            let head = after.split('-').next().unwrap_or("");
            if head.is_empty() {
                return true;
            }
            let want = active.get(*axis).map(String::as_str).unwrap_or("");
            if want == FREEFORM_DYNAMIC {
                return true;
            }
            if want.is_empty() {
                return false;
            }
            // Exact or compound: "checkbox" matches "checkbox", "checkboxchecked",
            // "checkboxsizemd" — remainder must be a known compound suffix.
            return freeform_axis_value_matches(head, want);
        }
    }
    // Tone / compound: s-tone{t}-* or s-tone{t}variant{v}-*
    if let Some(after) = rest.strip_prefix("tone") {
        let head = after.split('-').next().unwrap_or("");
        if head.is_empty() {
            return true;
        }
        let want_t = active.get("tone").map(String::as_str).unwrap_or("");
        let want_v = active.get("variant").map(String::as_str).unwrap_or("");
        if want_t == FREEFORM_DYNAMIC || want_v == FREEFORM_DYNAMIC {
            return true;
        }
        if let Some(idx) = head.find("variant") {
            let tone = &head[..idx];
            let variant = &head[idx + "variant".len()..];
            return tone.eq_ignore_ascii_case(want_t) && variant.eq_ignore_ascii_case(want_v);
        }
        return !want_t.is_empty()
            && head
                .to_ascii_lowercase()
                .starts_with(&want_t.to_ascii_lowercase());
    }
    // Unconditional base paint (display, pad, radius, borderWidth, …).
    true
}

/// Stamp a native-kind element's host classes (`sail-<name>` + the atomic
/// sx classes its host Style lowered to — `ElementOutput::host_classes`,
/// the same string `serialize_static` bakes into a spliced root) onto a
/// use-site open tag: `<label size="sm">` → `<label class="sail-label s-…"
/// data-sail-host="label" size="sm">`. Rules:
///   - an author `class` merges — host classes first, author classes after,
///     deduped by token, so re-scanning an already-stamped tag is a NO-OP
///     (idempotent: the second stamp produces the identical string);
///   - a dynamic `class={…}` becomes a concat expression — a dynamic class
///     binding REPLACES the attribute at runtime, so the stamped classes
///     must ride inside the expression to survive;
///   - `data-sail-host` is always set to the L1 name (compiler identity IR);
///   - every other attribute (use-site props like `size=`/`required`) is
///     preserved untouched — the skin's `[attr=value]` selectors match on
///     them.
///
/// **Match-arm thinning:** host Style lowers *every* match/compound arm into
/// classes (e.g. all variants × tones). Freeform dual-gate CSS only needs the
/// arms for attrs actually on the tag (+ defaults). Stamping the full soup is
/// ~4–12KB per control. We keep base/state classes + arms for active attrs.
/// (Style match/compound — not CVA string maps.)
///
/// **`<a>` dual-mode:** bare (no `variant`) is a text link painted by
/// sail-base `a:not([variant])` — only `sail-a`, no button-link match arms.
fn stamp_native_host(
    open_tag: &str,
    host_classes: &str,
    default_attrs: &[(String, String)],
    bound_props: &[(String, crate::check::binding::StyleBindingUnit)],
) -> String {
    let self_closing = open_tag.ends_with("/>");
    let inner = &open_tag[..open_tag.len() - if self_closing { 2 } else { 1 }];
    let name_end = inner.find(char::is_whitespace).unwrap_or(inner.len());
    let tag_name = &inner[1..name_end];

    // Bare text links: identity only — no button-link class dump.
    let bare_link = tag_name == "a" && open_tag_attr(open_tag, "variant").is_none();
    let host_classes = if bare_link { "sail-a" } else { host_classes };
    // Defaults for bare `<a>` already skip variant/tone (default_prop_attrs).
    let default_attrs: &[(String, String)] = if bare_link { &[] } else { default_attrs };
    let bridge_list: Vec<String> = if bare_link {
        Vec::new()
    } else {
        bound_props
            .iter()
            .filter(|(_, u)| *u == crate::check::binding::StyleBindingUnit::TokenGroup)
            .map(|(n, _)| kebab_of(n))
            .collect()
    };

    if host_classes.trim().is_empty() && default_attrs.is_empty() && bridge_list.is_empty() {
        return open_tag.to_string();
    }
    let host_name = host_classes
        .split_whitespace()
        .next()
        .and_then(|c| c.strip_prefix("sail-"))
        .unwrap_or(tag_name);
    let attrs = parse_attr_list(&inner[name_end..]);

    // Active variance attrs = prop defaults, overridden by use-site.
    // Dynamic `{…}` bindings mark the axis so every match arm stays (CSS dual-gates
    // on the live attr; we can't pick one arm at stamp time).
    let mut active: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for (n, v) in default_attrs {
        if !v.is_empty() {
            active.insert(n.clone(), v.clone());
        }
    }
    for (name, value) in &attrs {
        let v = value.trim_matches(['"', '\'']);
        if v.starts_with('{') {
            active.insert((*name).to_string(), FREEFORM_DYNAMIC.to_string());
        } else if !v.is_empty() {
            active.insert((*name).to_string(), v.to_string());
        } else if value.is_empty() {
            // boolean presence attrs
            active.insert((*name).to_string(), "true".to_string());
        }
    }

    // Select base ships `w(full)` for form columns; when the author already
    // sets width (participation or Style), drop the default so fixed toolbar
    // widths are not dual-classed with `s-wfull`.
    let drop_select_wfull = tag_name == "select" && author_sets_select_width(&attrs);

    let mut seen: std::collections::HashSet<&str> = std::collections::HashSet::new();
    let mut merged: Vec<&str> = Vec::new();
    for c in host_classes.split_whitespace() {
        if drop_select_wfull && c == "s-wfull" {
            continue;
        }
        if !bare_link && !freeform_class_active(c, &active) {
            continue;
        }
        if seen.insert(c) {
            merged.push(c);
        }
    }
    let mut static_classes: Vec<&str> = Vec::new();
    let mut dynamic_class: Option<&str> = None;
    let mut rest_attrs: Vec<(&str, &str)> = Vec::new();
    let mut present: std::collections::HashSet<String> = std::collections::HashSet::new();
    for (name, value) in &attrs {
        if name == "class" {
            let v = value.trim_matches(['"', '\'']);
            let t = v.trim();
            if t.starts_with('{') && t.ends_with('}') && t.len() > 1 {
                dynamic_class = Some(v);
            } else {
                static_classes.push(v);
            }
        } else if name != "data-sail-host" {
            present.insert((*name).to_string());
            rest_attrs.push((name, value));
        }
    }
    for v in static_classes {
        for c in v.split_whitespace() {
            if seen.insert(c) {
                merged.push(c);
            }
        }
    }
    let merged = merged.join(" ");
    let mut outb = format!("<{tag_name}");
    if !merged.is_empty() {
        match dynamic_class {
            Some(expr) => {
                let t = expr.trim();
                let body = t[1..t.len() - 1].trim();
                outb.push_str(&format!(" class=\"{{\"{merged} \" + ({body})}}\""));
            }
            None => outb.push_str(&format!(" class=\"{merged}\"")),
        }
    } else if let Some(expr) = dynamic_class {
        outb.push_str(&format!(" class=\"{expr}\""));
    }
    outb.push_str(&format!(" data-sail-host=\"{host_name}\""));
    if !bridge_list.is_empty() {
        let list = bridge_list.join(" ");
        outb.push_str(&format!(" data-sail-bridge=\"{list}\""));
    }
    // Static first-paint for TokenGroup props present as literal attrs.
    let mut bind_style = String::new();
    for (prop, unit) in bound_props {
        if *unit != crate::check::binding::StyleBindingUnit::TokenGroup {
            continue;
        }
        let raw = rest_attrs
            .iter()
            .find(|(n, _)| *n == prop.as_str() || camel_eq(n, prop))
            .map(|(_, v)| v.trim_matches(['"', '\'']).to_string())
            .or_else(|| {
                default_attrs
                    .iter()
                    .find(|(n, _)| n == prop || camel_eq(n, prop))
                    .map(|(_, v)| v.clone())
            });
        let Some(val) = raw.filter(|v| !v.is_empty() && !v.starts_with('{')) else {
            continue;
        };
        let css = kebab_of(prop);
        let Some(members) = sail_lint::token_groups::group_value_member_publics(&val) else {
            continue;
        };
        for (member, public) in members {
            if !bind_style.is_empty() {
                bind_style.push(' ');
            }
            let bridge = if member.is_empty() {
                css.clone()
            } else {
                format!("{css}-{member}")
            };
            bind_style.push_str(&format!("--{bridge}: var(--{public});"));
        }
    }
    // Password product: stamp reveal when type=password and reveal is not off.
    // Native freeform does not inherit use: from element structure — inject here.
    let stamp_password_reveal = if tag_name == "input" {
        let ty = rest_attrs
            .iter()
            .find(|(n, _)| *n == "type")
            .map(|(_, v)| v.trim_matches(['"', '\'']))
            .or_else(|| {
                default_attrs
                    .iter()
                    .find(|(n, _)| n == "type")
                    .map(|(_, v)| v.as_str())
            })
            .unwrap_or("text");
        let reveal_explicit_false = rest_attrs.iter().any(|(n, v)| {
            *n == "reveal" && matches!(v.trim_matches(['"', '\'']), "false" | "0" | "off")
        });
        ty == "password"
            && !reveal_explicit_false
            && !rest_attrs.iter().any(|(n, _)| *n == "use:passwordReveal")
    } else {
        false
    };
    let mut wrote_style = false;
    for (n, v) in rest_attrs {
        outb.push(' ');
        outb.push_str(n);
        if n == "style" && !bind_style.is_empty() {
            let prev = v.trim_matches(['"', '\'']);
            if !prev.starts_with('{') {
                outb.push_str(&format!("=\"{prev} {bind_style}\""));
                wrote_style = true;
                continue;
            }
        }
        if !v.is_empty() {
            outb.push('=');
            outb.push_str(v);
        }
    }
    if !bind_style.is_empty() && !wrote_style {
        outb.push_str(&format!(" style=\"{bind_style}\""));
    }
    for (n, v) in default_attrs {
        if present.contains(n.as_str()) {
            continue;
        }
        outb.push(' ');
        outb.push_str(n);
        if !v.is_empty() {
            outb.push_str("=\"");
            outb.push_str(v);
            outb.push('"');
        }
    }
    if stamp_password_reveal {
        outb.push_str(" use:passwordReveal");
    }
    if self_closing {
        outb.push('/');
    }
    outb.push('>');
    outb
}

/// Merge use-site attrs onto the snippet root and place children.
fn splice_element(
    el: &crate::emit::element::ElementOutput,
    use_attrs: &str,
    children: &str,
) -> String {
    let snippet = &el.structure_html;
    let Some(root_end) = snippet.find('>') else {
        return snippet.clone();
    };
    let root_open = &snippet[..root_end]; // "<span class=\"sail-x …\" …"
    let rest = &snippet[root_end..]; // ">…</span>"

    let mut snippet_attrs = parse_attr_list(
        &root_open[root_open
            .find(char::is_whitespace)
            .map(|p| p + 1)
            .unwrap_or(root_open.len())..],
    );
    let use_list = parse_attr_list(use_attrs);
    for (name, value) in use_list {
        if name == "class" {
            if let Some(slot) = snippet_attrs.iter_mut().find(|(n, _)| n == "class") {
                let merged = format!("{} {}", slot.1.trim_matches('"'), value.trim_matches('"'));
                slot.1 = format!("\"{merged}\"");
                continue;
            }
        }
        // Use-site wins on collision.
        snippet_attrs.retain(|(n, _)| *n != name);
        snippet_attrs.push((name, value));
    }
    // Fill prop defaults the use site omitted (CVA keys on attributes).
    let present: std::collections::HashSet<String> =
        snippet_attrs.iter().map(|(n, _)| n.clone()).collect();
    for (n, v) in &el.default_attrs {
        if present.contains(n) {
            continue;
        }
        if v.is_empty() {
            snippet_attrs.push((n.clone(), String::new()));
        } else {
            snippet_attrs.push((n.clone(), format!("\"{v}\"")));
        }
    }
    let tag_name: String = root_open[1..]
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
        .collect();
    // Progressive enhancement: static attr values for bound props become
    // host custom properties so non-JS renderers see the real inset (not
    // only the @property initial-value floor). TokenGroup props also stamp
    // `data-sail-bridge` so runtime bindAttr can reflect dynamic tone values.
    let mut bind_style = String::new();
    let mut bridge_props: Vec<String> = Vec::new();
    if !el.bound_props.is_empty() {
        for (prop, unit) in &el.bound_props {
            if *unit == crate::check::binding::StyleBindingUnit::TokenGroup {
                let kb = kebab_of(prop);
                if !bridge_props.contains(&kb) {
                    bridge_props.push(kb);
                }
            }
            let raw = snippet_attrs
                .iter()
                .find(|(n, _)| n == prop || camel_eq(n, prop))
                .map(|(_, v)| v.trim_matches(['"', '\'']).to_string())
                .or_else(|| {
                    el.default_attrs
                        .iter()
                        .find(|(n, _)| n == prop || camel_eq(n, prop))
                        .map(|(_, v)| v.clone())
                });
            let Some(val) = raw.filter(|v| !v.is_empty() && !v.starts_with('{')) else {
                continue;
            };
            let css = kebab_of(prop);
            match unit {
                // Color-group typed: static first-paint inlines var *references*
                // (`--tone: var(--primary); --tone-foreground: var(--primary-foreground)`).
                crate::check::binding::StyleBindingUnit::TokenGroup => {
                    let Some(members) = sail_lint::token_groups::group_value_member_publics(&val)
                    else {
                        continue;
                    };
                    for (member, public) in members {
                        if !bind_style.is_empty() {
                            bind_style.push(' ');
                        }
                        let bridge = if member.is_empty() {
                            css.clone()
                        } else {
                            format!("{css}-{member}")
                        };
                        bind_style.push_str(&format!("--{bridge}: var(--{public});"));
                    }
                }
                crate::check::binding::StyleBindingUnit::LengthPx
                    if val
                        .chars()
                        .all(|c| c.is_ascii_digit() || c == '.' || c == '-') =>
                {
                    if !bind_style.is_empty() {
                        bind_style.push(' ');
                    }
                    bind_style.push_str(&format!("--{css}: {val}px;"));
                }
                _ => {
                    if !bind_style.is_empty() {
                        bind_style.push(' ');
                    }
                    bind_style.push_str(&format!("--{css}: {val};"));
                }
            }
        }
    }
    if !bridge_props.is_empty() {
        let list = bridge_props.join(" ");
        if let Some(slot) = snippet_attrs
            .iter_mut()
            .find(|(n, _)| n == "data-sail-bridge")
        {
            let prev = slot.1.trim_matches(['"', '\'']);
            slot.1 = format!("\"{prev} {list}\"");
        } else {
            snippet_attrs.push(("data-sail-bridge".into(), format!("\"{list}\"")));
        }
    }
    if !bind_style.is_empty() {
        if let Some(slot) = snippet_attrs.iter_mut().find(|(n, _)| n == "style") {
            let prev = slot.1.trim_matches(['"', '\'']);
            if prev.starts_with('{') {
                // Dynamic style expression — leave alone (runtime reflection owns it).
            } else {
                slot.1 = format!("\"{prev} {bind_style}\"");
            }
        } else {
            snippet_attrs.push(("style".into(), format!("\"{bind_style}\"")));
        }
    }
    let mut outb = format!("<{tag_name}");
    for (n, v) in &snippet_attrs {
        outb.push(' ');
        outb.push_str(n);
        if !v.is_empty() {
            outb.push('=');
            outb.push_str(v);
        }
    }
    // COMPILE-TIME SLOT PROJECTION. Partition the use-site children by their
    // `slot="name"` attribute (un-slotted nodes → the default slot), then fill
    // each `<slot [name=…]>fallback</slot>` region of the structure. A slot
    // with no matching projected content keeps its fallback; children with no
    // matching slot are dropped (shadow parity, same as the runtime).
    let (default_children, named_children) = project_children(children);
    let body = fill_slots(rest, &default_children, &named_children);
    let body = fill_prop_holes(&body, &snippet_attrs);
    outb.push_str(&body);
    outb
}

/// Replace `<!--sail:prop:title-->` with the use-site attr (static first
/// paint). `{expr}` values stay in the lowered template as a page binding.
fn fill_prop_holes(body: &str, attrs: &[(String, String)]) -> String {
    let mut out = String::with_capacity(body.len());
    let mut rest = body;
    const OPEN: &str = "<!--sail:prop:";
    while let Some(start) = rest.find(OPEN) {
        out.push_str(&rest[..start]);
        let after = &rest[start + OPEN.len()..];
        let Some(end) = after.find("-->") else {
            out.push_str(&rest[start..]);
            return out;
        };
        let name = after[..end].trim();
        let raw = attrs
            .iter()
            .find(|(n, _)| n == name || camel_eq(n, name))
            .map(|(_, v)| v.trim_matches(['"', '\'']).to_string())
            .unwrap_or_default();
        if raw.starts_with('{') {
            out.push_str(&raw);
        } else {
            out.push_str(&html_escape_text(&raw));
        }
        rest = &after[end + 3..];
    }
    out.push_str(rest);
    out
}

fn html_escape_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

fn camel_eq(a: &str, b: &str) -> bool {
    crate::emit::util::camel_of(a) == crate::emit::util::camel_of(b)
}

fn kebab_of(name: &str) -> String {
    let mut out = String::with_capacity(name.len() + 4);
    for (i, c) in name.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i != 0 {
                out.push('-');
            }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

/// Partition use-site children into the default bucket (un-slotted nodes, in
/// source order) and named buckets keyed by each top-level node's `slot=`
/// attribute. A node's slot assigns its WHOLE subtree — a `slot=` deeper in
/// the tree belongs to that descendant, not to us. Quote/void/comment aware,
/// depth-aware on the matching close (mirrors `lower_into`).
fn project_children(children: &str) -> (String, std::collections::HashMap<String, String>) {
    let mut default = String::new();
    let mut named: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let bytes = children.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if children[i..].starts_with("<!--") {
            let end = children[i..]
                .find("-->")
                .map(|e| i + e + 3)
                .unwrap_or(bytes.len());
            default.push_str(&children[i..end]);
            i = end;
            continue;
        }
        if bytes[i] == b'<' && i + 1 < bytes.len() && bytes[i + 1].is_ascii_alphabetic() {
            let tag_start = i + 1;
            let mut te = tag_start;
            while te < bytes.len() && (bytes[te].is_ascii_alphanumeric() || bytes[te] == b'-') {
                te += 1;
            }
            let tag = &children[tag_start..te];
            // Open-tag end, quote-aware.
            let mut k = te;
            let mut quote: Option<u8> = None;
            while k < bytes.len() {
                match (quote, bytes[k]) {
                    (Some(q), b) if b == q => quote = None,
                    (None, b'"') | (None, b'\'') => quote = Some(bytes[k]),
                    (None, b'>') => break,
                    _ => {}
                }
                k += 1;
            }
            if k >= bytes.len() {
                default.push_str(&children[i..]);
                break;
            }
            let open_tag = &children[i..=k];
            let self_closed = bytes[k.saturating_sub(1)] == b'/';
            let node_end = if self_closed || crate::emit::element::is_void_element(tag) {
                k + 1
            } else {
                let open_prefix = format!("<{tag}");
                let close = format!("</{tag}>");
                let mut depth = 1usize;
                let mut scan = k + 1;
                loop {
                    if scan >= bytes.len() {
                        break bytes.len();
                    }
                    if bytes[scan..].starts_with(close.as_bytes()) {
                        depth -= 1;
                        if depth == 0 {
                            break scan + close.len();
                        }
                        scan += close.len();
                    } else if bytes[scan..].starts_with(open_prefix.as_bytes())
                        && bytes
                            .get(scan + open_prefix.len())
                            .is_some_and(|&b| b.is_ascii_whitespace() || b == b'>')
                    {
                        depth += 1;
                        scan += open_prefix.len();
                    } else {
                        scan += 1;
                    }
                }
            };
            let node = &children[i..node_end];
            match open_tag_attr(open_tag, "slot") {
                Some(name) => named.entry(name).or_default().push_str(node),
                None => default.push_str(node),
            }
            i = node_end;
            continue;
        }
        let next = bytes[i + 1..]
            .iter()
            .position(|&b| b == b'<')
            .map(|n| i + 1 + n)
            .unwrap_or(bytes.len());
        default.push_str(&children[i..next]);
        i = next;
    }
    (default, named)
}

/// Fill each `<slot [name=…]>fallback</slot>` in a structure snippet with the
/// projected content for that slot name (default slot = no `name`). An empty
/// or absent bucket falls back to the slot's own inner content.
fn fill_slots(
    rest: &str,
    default: &str,
    named: &std::collections::HashMap<String, String>,
) -> String {
    let bytes = rest.as_bytes();
    let mut out = String::with_capacity(rest.len());
    let mut i = 0;
    while i < bytes.len() {
        if rest[i..].starts_with("<slot")
            && bytes
                .get(i + 5)
                .is_some_and(|&b| b.is_ascii_whitespace() || b == b'>')
        {
            let mut k = i + 5;
            let mut quote: Option<u8> = None;
            while k < bytes.len() {
                match (quote, bytes[k]) {
                    (Some(q), b) if b == q => quote = None,
                    (None, b'"') | (None, b'\'') => quote = Some(bytes[k]),
                    (None, b'>') => break,
                    _ => {}
                }
                k += 1;
            }
            if k >= bytes.len() {
                out.push_str(&rest[i..]);
                break;
            }
            let open_tag = &rest[i..=k];
            let close_at = rest[k..].find("</slot>").map(|p| k + p);
            let (fallback, end) = match close_at {
                Some(c) => (&rest[k + 1..c], c + "</slot>".len()),
                None => ("", k + 1),
            };
            let projected = match open_tag_attr(open_tag, "name") {
                Some(n) => named.get(&n).map(String::as_str).filter(|s| !s.is_empty()),
                None => Some(default).filter(|s| !s.is_empty()),
            };
            out.push_str(projected.unwrap_or(fallback));
            i = end;
            continue;
        }
        let next = bytes[i + 1..]
            .iter()
            .position(|&b| b == b'<')
            .map(|n| i + 1 + n)
            .unwrap_or(bytes.len());
        out.push_str(&rest[i..next]);
        i = next;
    }
    out
}

/// Read a single attribute's value off an element's open tag (`<tag a="1" …>`),
/// quotes stripped. Returns None if the attribute is absent.
fn open_tag_attr(open_tag: &str, attr: &str) -> Option<String> {
    let inner = open_tag
        .trim_start_matches('<')
        .trim_end_matches('>')
        .trim_end_matches('/');
    let after_tag = inner
        .find(char::is_whitespace)
        .map(|p| &inner[p + 1..])
        .unwrap_or("");
    parse_attr_list(after_tag)
        .into_iter()
        .find(|(n, _)| n == attr)
        .map(|(_, v)| v.trim_matches(['"', '\'']).to_string())
}

/// Parse a flat attribute list `a="1" b c={x}` into (name, raw-value) pairs
/// (value includes its quotes; empty for bare attrs). Quote-aware.
fn parse_attr_list(s: &str) -> Vec<(String, String)> {
    let bytes = s.as_bytes();
    let mut i = 0;
    let mut outv = Vec::new();
    while i < bytes.len() {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() || bytes[i] == b'/' {
            break;
        }
        let name_start = i;
        while i < bytes.len()
            && !bytes[i].is_ascii_whitespace()
            && bytes[i] != b'='
            && bytes[i] != b'>'
            && bytes[i] != b'/'
        {
            i += 1;
        }
        let name = s[name_start..i].to_string();
        if name.is_empty() {
            break;
        }
        let mut value = String::new();
        if bytes.get(i) == Some(&b'=') {
            i += 1;
            if let Some(&q) = bytes.get(i).filter(|&&b| b == b'"' || b == b'\'') {
                let vstart = i;
                i += 1;
                while i < bytes.len() && bytes[i] != q {
                    i += 1;
                }
                i = (i + 1).min(bytes.len());
                value = s[vstart..i].to_string();
            } else {
                let vstart = i;
                while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'>' {
                    i += 1;
                }
                value = s[vstart..i].to_string();
            }
        }
        outv.push((name, value));
    }
    outv
}

/// Bare framework-enum shorthand (authoring sugar): on framework tags, a bare
/// attribute matching a framework enum value desugars to its prop —
/// `<button primary solid sm>` → `<button tone="primary" variant="solid"
/// size="sm">`. Rules that keep it deterministic:
///   - Framework tags only: the L1 control set. Userland components keep
///     explicit attributes (their prop names could collide).
///   - The vocabularies are disjoint by construction (tones ∪ sizes ∪
///     variants share no member); `default` is excluded (ambiguous between
///     tone and variant, and it IS the default — writing it is a no-op).
///   - An explicit attribute wins: a tag already carrying `tone=` gets no
///     tone sugar. First bare value wins per category; later ones pass
///     through untouched (harmless attributes a lint can flag).
///   - Static only, like all sugar — the DOM carries the real attributes, so
///     skins, contracts, and CEM see nothing new.
fn lower_prop_shorthand(source: &str) -> String {
    const TONES: &[&str] = &[
        "primary",
        "info",
        "success",
        "warning",
        "destructive",
        "accent",
        "muted",
    ];
    const SIZES: &[&str] = &["xs", "sm", "md", "lg", "xl"];
    const VARIANTS: &[&str] = &["solid", "soft", "outline", "ghost"];
    const L1_TAGS: &[&str] = &["button", "input", "textarea", "select"];

    let bytes = source.as_bytes();
    let mut out = String::with_capacity(source.len());
    let mut i = 0;
    while i < bytes.len() {
        if source[i..].starts_with("<!--") {
            let end = source[i..]
                .find("-->")
                .map(|e| i + e + 3)
                .unwrap_or(bytes.len());
            out.push_str(&source[i..end]);
            i = end;
            continue;
        }
        if bytes[i] == b'<' && i + 1 < bytes.len() && bytes[i + 1].is_ascii_alphabetic() {
            // Tag name + framework check.
            let mut j = i + 1;
            while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'-') {
                j += 1;
            }
            let tag_name = &source[i + 1..j];
            // Framework (L1 control) tags only. The retired `sail-`-prefixed
            // spelling — the old component namespace the L1 set absorbed —
            // no longer participates in prop shorthand: there is no
            // `<sail-button>` to desugar TO (the L1 element is `<button>`),
            // and live `sail-*` custom elements are not the compiler's
            // responsibility (they carry their own prop contract). The
            // pre-L1 source files that used the retired spelling were
            // codemodded to the bare L1 names.
            let is_framework = L1_TAGS.contains(&tag_name);
            // Find tag end honoring quotes.
            let mut k = j;
            let mut quote: Option<u8> = None;
            while k < bytes.len() {
                match (quote, bytes[k]) {
                    (Some(q), b) if b == q => quote = None,
                    (None, b'"') | (None, b'\'') => quote = Some(bytes[k]),
                    (None, b'>') => break,
                    _ => {}
                }
                k += 1;
            }
            if k < bytes.len() && is_framework {
                let tag = &source[i..=k];
                out.push_str(&desugar_tag(tag, TONES, SIZES, VARIANTS));
                i = k + 1;
                continue;
            }
            if k < bytes.len() {
                out.push_str(&source[i..=k]);
                i = k + 1;
                continue;
            }
        }
        let next = bytes[i + 1..]
            .iter()
            .position(|&b| b == b'<')
            .map(|n| i + 1 + n)
            .unwrap_or(bytes.len());
        out.push_str(&source[i..next]);
        i = next;
    }
    out
}

/// Desugar one framework open-tag: replace bare vocab attributes with their
/// prop form, honoring explicit-attribute precedence and first-wins.
fn desugar_tag(tag: &str, tones: &[&str], sizes: &[&str], variants: &[&str]) -> String {
    let has = |needle: &str| tag.contains(needle);
    let mut tone_done = has(" tone=") || has("\ntone=");
    let mut size_done = has(" size=") || has("\nsize=");
    let mut variant_done = has(" variant=") || has("\nvariant=");

    let bytes = tag.as_bytes();
    let mut out = String::with_capacity(tag.len() + 24);
    let mut i = 0;
    let mut quote: Option<u8> = None;
    // Only desugar bare tone/size/variant shorthands at *attribute* depth.
    // Tag-position `style({…})` / match/compound maps can contain bare words like
    // `outline` / `ghost` as Style condition values — rewriting those into
    // `variant="outline"` poisons compound CSS to `[variant='variant']`.
    let mut depth: i32 = 0;
    while i < bytes.len() {
        match (quote, bytes[i]) {
            (Some(q), b) if b == q => {
                quote = None;
                out.push(bytes[i] as char);
                i += 1;
                continue;
            }
            (Some(_), _) => {
                // Inside a quoted value — copy one full UTF-8 char.
                let mut n = i + 1;
                while n < bytes.len() && (bytes[n] & 0xC0) == 0x80 {
                    n += 1;
                }
                out.push_str(&tag[i..n]);
                i = n;
                continue;
            }
            (None, b'"') | (None, b'\'') => {
                quote = Some(bytes[i]);
                out.push(bytes[i] as char);
                i += 1;
                continue;
            }
            (None, b'(' | b'{' | b'[') => {
                depth += 1;
                out.push(bytes[i] as char);
                i += 1;
                continue;
            }
            (None, b')' | b'}' | b']') => {
                depth = (depth - 1).max(0);
                out.push(bytes[i] as char);
                i += 1;
                continue;
            }
            _ => {}
        }
        // A bare word starts after whitespace — attribute level only.
        if depth == 0
            && i > 0
            && bytes[i - 1].is_ascii_whitespace()
            && bytes[i].is_ascii_alphabetic()
        {
            let mut j = i;
            while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'-') {
                j += 1;
            }
            let word = &tag[i..j];
            let ends_bare = bytes
                .get(j)
                .is_none_or(|&b| b.is_ascii_whitespace() || b == b'>' || b == b'/');
            if ends_bare {
                let prop = if !tone_done && tones.contains(&word) {
                    tone_done = true;
                    Some("tone")
                } else if !size_done && sizes.contains(&word) {
                    size_done = true;
                    Some("size")
                } else if !variant_done && variants.contains(&word) {
                    variant_done = true;
                    Some("variant")
                } else {
                    None
                };
                if let Some(p) = prop {
                    out.push_str(&format!("{p}=\"{word}\""));
                    i = j;
                    continue;
                }
            }
            out.push_str(word);
            i = j;
            continue;
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

/// Merge every open tag's duplicate static `class="…"` attributes into the
/// first one (values space-joined, source order). `lower_sx` rewrites `sx=` to
/// a fresh `class=` without looking for an existing one, so
/// `<span class="badge" sx="p:4">` arrives here as two class attrs — the HTML
/// parser keeps both and the emitted `setAttribute("class")` pair lets the
/// second wipe the first at runtime (the sail-badge `.badge` regression).
/// Conservative: a tag containing a dynamic `class="{…}"` is left untouched.
fn merge_duplicate_class_attrs(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut out = String::with_capacity(source.len());
    let mut i = 0;
    while i < bytes.len() {
        if source[i..].starts_with("<!--") {
            let end = source[i..]
                .find("-->")
                .map(|e| i + e + 3)
                .unwrap_or(bytes.len());
            out.push_str(&source[i..end]);
            i = end;
            continue;
        }
        // Only open tags: `<` + ASCII letter (not `</`, `<!`, `<{`).
        if bytes[i] == b'<' && i + 1 < bytes.len() && bytes[i + 1].is_ascii_alphabetic() {
            // Find the tag end, honoring quoted attribute values (a `>` inside
            // a quoted value must not terminate the tag).
            let mut j = i + 1;
            let mut quote: Option<u8> = None;
            while j < bytes.len() {
                match (quote, bytes[j]) {
                    (Some(q), b) if b == q => quote = None,
                    (None, b'"') | (None, b'\'') => quote = Some(bytes[j]),
                    (None, b'>') => break,
                    _ => {}
                }
                j += 1;
            }
            if j < bytes.len() {
                let tag = &source[i..=j];
                out.push_str(&merge_tag_classes(tag));
                i = j + 1;
                continue;
            }
        }
        // Copy up to the next `<` (or the rest). Byte-scan, NOT a str slice
        // at `i + 1` — when `i` sits on a multi-byte char, `source[i + 1..]`
        // is mid-codepoint and panics (the sail-date-picker `‹` regression).
        // A `b'<'` byte position is always a char boundary.
        let next = bytes[i + 1..]
            .iter()
            .position(|&b| b == b'<')
            .map(|n| i + 1 + n)
            .unwrap_or(bytes.len());
        out.push_str(&source[i..next]);
        i = next;
    }
    out
}

/// Merge the static `class="…"` attributes inside one open-tag slice. Returns
/// the tag unchanged when it has fewer than two, or when any class value is a
/// `{…}` expression (dynamic — the emitter owns that path).
fn merge_tag_classes(tag: &str) -> String {
    let bytes = tag.as_bytes();
    // (attr_start, value_start, value_end) for each `class="…"`, where
    // attr_start points at the `c` and the value span excludes the quotes.
    let mut hits: Vec<(usize, usize, usize)> = Vec::new();
    let mut i = 0;
    let mut quote: Option<u8> = None;
    while i < bytes.len() {
        match (quote, bytes[i]) {
            (Some(q), b) if b == q => {
                quote = None;
                i += 1;
                continue;
            }
            (Some(_), _) => {
                i += 1;
                continue;
            }
            (None, b'"') | (None, b'\'') => {
                quote = Some(bytes[i]);
                i += 1;
                continue;
            }
            _ => {}
        }
        // `bytes[i] == b'c'` first: it guarantees `i` is a char boundary
        // (an ASCII byte is never a continuation byte), so the `tag[i..]`
        // slice below cannot land mid-codepoint.
        if i > 0
            && bytes[i] == b'c'
            && bytes[i - 1].is_ascii_whitespace()
            && tag[i..].starts_with("class=\"")
        {
            let vstart = i + "class=\"".len();
            if let Some(rel) = tag[vstart..].find('"') {
                hits.push((i, vstart, vstart + rel));
                i = vstart + rel + 1;
                continue;
            }
        }
        i += 1;
    }
    if hits.len() < 2
        || hits
            .iter()
            .any(|&(_, vs, ve)| tag[vs..ve].trim_start().starts_with('{'))
    {
        return tag.to_string();
    }
    let merged = hits
        .iter()
        .map(|&(_, vs, ve)| tag[vs..ve].trim())
        .filter(|v| !v.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let mut out = String::with_capacity(tag.len());
    let (_, first_vs, _) = hits[0];
    out.push_str(&tag[..first_vs]);
    out.push_str(&merged);
    // Copy the remainder, skipping every later `class="…"` attribute (and the
    // single whitespace run that precedes it).
    let mut pos = hits[0].2; // at the closing quote of the first value
    for &(start, _, vend) in &hits[1..] {
        let mut cut = start;
        while cut > pos && bytes[cut - 1].is_ascii_whitespace() {
            cut -= 1;
        }
        out.push_str(&tag[pos..cut]);
        pos = vend + 1; // past the closing quote
    }
    out.push_str(&tag[pos..]);
    out
}
/// Lower each layout keyword (`<column>`, `<row>`, `<grid>`, `<center>`,
/// `<stack>`) to a `<div>` + atomic layout classes, returning the rewritten body
/// and the deduped CSS. Axis props (gap/align/justify/min/flow) become classes
/// and drop off the tag; every other attribute passes through, and an existing
/// `class` (static or dynamic) is merged into the computed classes. Close tags
/// become `</div>`. The class/CSS mapping is `sail_lint::rules::layout` — the same
/// vocabulary the `sail-layout` lint validates against the raw source.

/// Dedup atomic CSS rules file-wide, first occurrence keeping its slot.
/// Each lowering pass (style / layout / stdlib skins) dedups only within
/// itself, so the same atomic (`s-wfull`) was emitted once per pass — and
/// the LAST copy won the shared-layer cascade. Atomic rules are identical
/// declarations, so first-occurrence dedup is semantics-preserving and
/// makes rule ORDER the single, reviewable mechanism.
fn dedup_css_rules(css: &str) -> String {
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut out = String::with_capacity(css.len());
    for rule in split_css_rules(css) {
        let trimmed = rule.trim();
        if trimmed.is_empty() {
            continue;
        }
        if seen.insert(trimmed.to_string()) {
            out.push_str(trimmed);
        }
    }
    out
}

/// Split a concatenated CSS stylesheet (from `lower_style`) into individual
/// rule strings. Handles nested `@media { … }` by tracking brace depth.
fn split_css_rules(css: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = css.as_bytes();
    let mut i = 0;
    let mut start = 0;
    let mut depth = 0i32;
    while i < bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    out.push(css[start..=i].to_string());
                    start = i + 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    if start < css.len() {
        let tail = css[start..].trim();
        if !tail.is_empty() {
            out.push(tail.to_string());
        }
    }
    out
}

fn lower_layout(source: &str) -> (String, String) {
    use sail_lint::rules::layout;
    let opens = layout::scan_open_tags(source);
    if opens.is_empty() {
        return (source.to_string(), String::new());
    }
    let open_at: std::collections::HashMap<usize, &layout::OpenTag> =
        opens.iter().map(|t| (t.start, t)).collect();

    let bytes = source.as_bytes();
    let mut out = String::with_capacity(source.len() + 256);
    let mut css = String::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut i = 0;
    while i < bytes.len() {
        if source[i..].starts_with("<!--") {
            match source[i..].find("-->") {
                Some(end) => {
                    out.push_str(&source[i..i + end + 3]);
                    i += end + 3;
                    continue;
                }
                None => break,
            }
        }
        if let Some(t) = open_at.get(&i)
            && let Some((base_class, base_rules)) = {
                // `inline` is a boolean axis — presence swaps the base class
                // and the base CSS for the inline variant. Detected once
                // here so the rest of the loop sees a consistent base; the
                // attribute is consumed (not passed through) below.
                // `cluster` absorbs inline + nowrap: hug, one line, intrinsic kids.
                let cluster = t.tag == "row" && t.attrs.iter().any(|(n, _)| n == "cluster");
                let inline = cluster || t.attrs.iter().any(|(n, _)| n == "inline");
                layout::base(&t.tag, inline)
            }
        {
            for r in base_rules {
                if seen.insert(r.clone()) {
                    css.push_str(&r);
                }
            }
            // Static class parts join into `class="…"`; dynamic parts (a dynamic
            // axis or a user `class={expr}`) each become a JS expression, and the
            // whole thing lowers to a `class={…}` binding.
            let mut static_parts: Vec<String> = vec![base_class];
            let mut dyn_parts: Vec<String> = Vec::new();
            // `cols` vs `min` precedence on `<grid>`: a fixed column count and a
            // responsive `auto-fill minmax` axis are mutually exclusive — both
            // own `grid-template-columns`. `cols` wins, so when present we
            // skip the `min` lowering entirely (the underlying div only gets
            // the `cols` rule, no `min` class, no minmax track list).
            let has_cols = t.attrs.iter().any(|(n, _)| n == "cols");
            let mut skip_props: std::collections::HashSet<&str> = std::collections::HashSet::new();
            if t.tag == "grid" && has_cols {
                skip_props.insert("min");
            }
            // `nowrap` is a boolean axis (row only): it layers a
            // `flex-wrap:nowrap` override on top of the base's wrap. Emitted
            // AFTER the base rule (see below) so source order wins — no
            // specificity game. Detected once here; consumed (not passed
            // through) in the attr loop below. `cluster` implies nowrap.
            let cluster = t.tag == "row" && t.attrs.iter().any(|(n, _)| n == "cluster");
            let nowrap = cluster || (t.tag == "row" && t.attrs.iter().any(|(n, _)| n == "nowrap"));
            for &prop in layout::tag_props(&t.tag) {
                // `inline`/`nowrap`/`cluster` are boolean axes handled outside
                // the enum-modifier machinery — never fed into `lower_axis`
                // and never emitted as raw HTML attrs.
                if prop == "inline" || prop == "nowrap" || prop == "cluster" {
                    continue;
                }
                if skip_props.contains(prop) {
                    continue;
                }
                let attr = t
                    .attrs
                    .iter()
                    .find(|(n, _)| n == prop)
                    .map(|(_, v)| v.as_str());
                match layout::lower_axis(&t.tag, prop, attr) {
                    Some(layout::AxisLower::Static { class, rule }) => {
                        if seen.insert(rule.clone()) {
                            css.push_str(&rule);
                        }
                        static_parts.push(class);
                    }
                    Some(layout::AxisLower::Dynamic { class_expr, rules }) => {
                        for r in rules {
                            if seen.insert(r.clone()) {
                                css.push_str(&r);
                            }
                        }
                        dyn_parts.push(class_expr);
                    }
                    None => {}
                }
            }
            // Boolean `nowrap` override. Source-order ALONE cannot win here:
            // the base `.sail-row{flex-wrap:wrap}` also ships in the framework
            // stylesheet (element-skins.css), whose document position vs the
            // per-page injected sheet is host-defined. A compound selector
            // (`.sail-row.sail-nowrap`, specificity 0-2-0) beats the base
            // (0-1-0) in every load order — no order dependency, no !important.
            if nowrap {
                let rule = if cluster {
                    ".sail-row-i.sail-nowrap{flex-wrap:nowrap}".to_string()
                } else {
                    ".sail-row.sail-nowrap{flex-wrap:nowrap}".to_string()
                };
                if seen.insert(rule.clone()) {
                    css.push_str(&rule);
                }
                static_parts.push("sail-nowrap".to_string());
            }
            if cluster {
                // Compound beats `.sail-row-i>*{min-width:0}` in any sheet order.
                let rule = ".sail-row-i.sail-cluster>*{min-width:auto;flex:0 0 auto}".to_string();
                if seen.insert(rule.clone()) {
                    css.push_str(&rule);
                }
                static_parts.push("sail-cluster".to_string());
            }
            let mut rest = String::new();
            for (name, value) in &t.attrs {
                match name.as_str() {
                    // Layout axis props — already consumed by the lowering loop
                    // above; their classes/rules are folded into the div's
                    // `class` and the injected stylesheet, NEVER emitted as
                    // raw HTML attrs. `inline` is special: its effect is the
                    // base swap (already applied above), not a class
                    // modifier, so it joins the consumed set here to keep the
                    // `inline` attribute from reappearing on the div.
                    "gap" | "align" | "justify" | "min" | "flow" | "cols" | "rows"
                    | "direction" | "tracks" | "inline" | "nowrap" | "cluster" => {}
                    "class" if value.starts_with('{') => {
                        dyn_parts.push(format!("({})", &value[1..value.len() - 1]));
                    }
                    "class" => {
                        if !value.is_empty() {
                            static_parts.push(value.clone());
                        }
                    }
                    name if layout::is_passthrough_attr(name) => {
                        // Recognized HTML/global pass-through (id, data-*,
                        // aria-*, on:*, type/value/href/src/…). Emit verbatim
                        // onto the div. The lint and the compiler share this
                        // pass-through list via `layout::is_passthrough_attr`,
                        // so a typo or unsupported layout attr never silently
                        // rides through to the div as a dead attribute.
                        rest.push(' ');
                        rest.push_str(name);
                        if !value.is_empty() {
                            rest.push_str("=\"");
                            rest.push_str(value);
                            rest.push('"');
                        }
                    }
                    _ => {
                        // An attribute the lint would have already rejected
                        // as an unknown layout-attr. We never reach here on
                        // well-formed sources (the lint is the build gate);
                        // emit nothing defensively so a bad attr can't
                        // silently flow through to the div.
                    }
                }
            }
            let static_str = static_parts.join(" ");
            out.push_str("<div class=");
            if dyn_parts.is_empty() {
                out.push_str(&format!("\"{static_str}\""));
            } else {
                // `class={'<static> ' + <dyn1> + ' ' + <dyn2> …}` — a quoted brace
                // attr, what quote_brace_attrs would have emitted for a binding.
                let dyn_expr = dyn_parts.join(" + ' ' + ");
                out.push_str(&format!("\"{{'{static_str} ' + {dyn_expr}}}\""));
            }
            out.push_str(&rest);
            out.push('>');
            if t.self_closing {
                out.push_str("</div>");
            }
            i = t.end;
            continue;
        }
        // Layout close tag → `</div>` (every layout tag lowers to a div).
        if source[i..].starts_with("</")
            && let Some(tag) = layout::TAGS.iter().find(|&&tg| {
                source[i + 2..].starts_with(tg) && bytes.get(i + 2 + tg.len()) == Some(&b'>')
            })
        {
            out.push_str("</div>");
            i += 2 + tag.len() + 1;
            continue;
        }
        let mut n = i + 1;
        while n < bytes.len() && (bytes[n] & 0xC0) == 0x80 {
            n += 1;
        }
        out.push_str(&source[i..n]);
        i = n;
    }
    out.push_str(&source[i..]);
    (out, css)
}

/// Index of the first top-level `:` in `s` (skips `:` inside `{}`/`[]`/strings —
/// e.g. the `:` in an sx value `'bg:tone'`).
fn top_level_colon(s: &str) -> Option<usize> {
    let mut depth = 0i32;
    let mut in_str: Option<char> = None;
    for (i, c) in s.char_indices() {
        match in_str {
            Some(q) => {
                if c == q {
                    in_str = None;
                }
            }
            None => match c {
                '\'' | '"' => in_str = Some(c),
                '{' | '[' => depth += 1,
                '}' | ']' => depth -= 1,
                ':' if depth == 0 => return Some(i),
                _ => {}
            },
        }
    }
    None
}

/// Strip a single layer of matching quotes.
pub(crate) fn unquote(s: &str) -> String {
    let s = s.trim();
    let b = s.as_bytes();
    if b.len() >= 2 && (b[0] == b'\'' || b[0] == b'"') && b[b.len() - 1] == b[0] {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

/// Parse `{ key: value, … }` into `(key, raw-value)` pairs. Values keep their
/// raw text (a quoted string, a nested `{…}`, or an `[…]`) for the caller.
/// Strip a leading comment run (`/** … */`, `/* … */`, `// …`) off an object-field
/// entry, returning `(doc, rest)` where `doc` is the JSDoc text if there was one.
///
/// WHY THIS EXISTS. `split_top_level(inner, ',')` splits on commas, and a comment has
/// none — so a documented prop arrived here as
///
/// ```text
/// "/** The visual tone. */\n    tone: 'default' | 'primary'"
/// ```
///
/// and everything before the colon became the prop NAME. The compiler happily emitted
/// `__props.____The_visual_tone_______` and the prop `tone` simply did not exist. It
/// compiled clean and shipped a component whose props were gone. Documenting a prop —
/// the first thing anyone writing a real component does — silently broke it.
fn strip_leading_doc(entry: &str) -> (Option<String>, &str) {
    let mut rest = entry.trim_start();
    let mut doc: Option<String> = None;
    loop {
        if let Some(after) = rest.strip_prefix("/*") {
            let is_jsdoc = after.starts_with('*');
            let Some(end) = after.find("*/") else { break };
            if is_jsdoc && doc.is_none() {
                // `/** text */` → "text": drop the leading `*`s and the per-line `*`.
                let body = &after[1..end];
                let text = body
                    .lines()
                    .map(|l| l.trim().trim_start_matches('*').trim())
                    .filter(|l| !l.is_empty())
                    .collect::<Vec<_>>()
                    .join(" ");
                if !text.is_empty() {
                    doc = Some(text);
                }
            }
            rest = after[end + 2..].trim_start();
            continue;
        }
        if let Some(after) = rest.strip_prefix("//") {
            let end = after.find('\n').unwrap_or(after.len());
            rest = after[end..].trim_start();
            continue;
        }
        break;
    }
    (doc, rest)
}

pub(crate) fn parse_object_fields(obj: &str) -> Vec<(String, String)> {
    let inner = obj
        .trim()
        .trim_start_matches('{')
        .trim_end_matches('}')
        .trim();
    // Scope grammar is newline- OR comma-separated (props, channels, CVA maps).
    // Comma-only split silently merged `default: {…}\nsize: {…}` into one field
    // whose last value swallowed the rest — spinner emitted `--bw:'control-stroke' }`
    // and dropped size rekeys. Treat both separators at depth 0.
    split_top_level_fields(inner)
        .into_iter()
        .filter_map(|entry| {
            // Comments are stripped here too: EVERY caller reading a field name needs
            // the name, not the documentation glued to the front of it.
            let (_, entry) = strip_leading_doc(entry.trim());
            let entry = entry.trim();
            if entry.is_empty() {
                return None;
            }
            // A colon-less field is a bare shorthand (`tone`, `size`) — key with
            // an empty value. Without this, ES-style shorthands were silently
            // dropped, so a built-in prop declared as `{ tone, size }` vanished.
            match top_level_colon(entry) {
                Some(idx) => Some((
                    unquote(entry[..idx].trim()),
                    entry[idx + 1..].trim().to_string(),
                )),
                None => Some((unquote(entry), String::new())),
            }
        })
        .collect()
}

/// Split object-field text on top-level `,` **or** newline. Nested `{}`/`[]`,
/// strings, and comments are opaque — same discipline as [`split_top_level`].
fn split_top_level_fields(s: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut in_str: Option<char> = None;
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut prev = '\0';
    let mut cur = String::new();
    for c in s.chars() {
        if in_line_comment {
            // Line comments end at newline; the newline is also a field separator
            // only when depth is 0 and we are not mid-value (depth already 0).
            if c == '\n' {
                in_line_comment = false;
                prev = c;
                if depth == 0 && !cur.trim().is_empty() {
                    parts.push(cur.trim().to_string());
                    cur.clear();
                } else {
                    cur.push(c);
                }
                continue;
            }
            cur.push(c);
            prev = c;
            continue;
        }
        if in_block_comment {
            cur.push(c);
            if prev == '*' && c == '/' {
                in_block_comment = false;
                prev = '\0';
                continue;
            }
            prev = c;
            continue;
        }
        if in_str.is_none() && prev == '/' && (c == '/' || c == '*') {
            if c == '/' {
                in_line_comment = true;
            } else {
                in_block_comment = true;
            }
            cur.push(c);
            prev = c;
            continue;
        }
        prev = c;
        match in_str {
            Some(q) => {
                cur.push(c);
                if c == q {
                    in_str = None;
                }
            }
            None => match c {
                '\'' | '"' => {
                    in_str = Some(c);
                    cur.push(c);
                }
                '{' | '[' => {
                    depth += 1;
                    cur.push(c);
                }
                '}' | ']' => {
                    depth -= 1;
                    cur.push(c);
                }
                ',' | '\n' if depth == 0 => {
                    if !cur.trim().is_empty() {
                        parts.push(cur.trim().to_string());
                        cur.clear();
                    }
                }
                _ => cur.push(c),
            },
        }
    }
    if !cur.trim().is_empty() {
        parts.push(cur.trim().to_string());
    }
    parts
}

/// Prepare a `<controller>` body for tsc/emit: inject the ambient runtime
/// import. Every controller writes the ONE form — `export default
/// controller((ctx) => {...})` — authored directly; there is no bare-export
/// sugar to lift anymore (retired with the object-form sections; a bare
/// scope with no `controller(...)` call is simply not a valid controller
/// body, and `check::controller_parse`/`check::paired_controller` catch that
/// upstream of this step). One emit path (inv-sail-one-module-emit): no
/// emitted controller module ever carries a free `controller`/`signal`/
/// `mount`/… identifier.
pub fn wrap_controller_body(body: &str) -> String {
    inject_ambient_runtime_imports(body)
}

/// Prepend an import for the ambient runtime idents (`runtime_ambient::RUNTIME_AMBIENT`)
/// the module references but has not itself imported or declared, so every
/// emitted controller module is self-contained. The ambient set + specifier are
/// the table's; nothing here hardcodes a runtime name. Each table entry is
/// `(authored_ident, exported_ident)` — aliased on import when they differ
/// (`mount` → `import { mountAmbient as mount } from '...'`).
fn inject_ambient_runtime_imports(module: &str) -> String {
    let bound = module_bound_idents(module);
    let needed: Vec<(&str, &str, &str)> = crate::runtime_ambient::RUNTIME_AMBIENT
        .iter()
        .copied()
        .filter(|(authored, _, _)| !bound.contains(*authored) && references_ident(module, authored))
        .collect();
    if needed.is_empty() {
        return module.to_string();
    }
    let specifiers: Vec<String> = needed
        .iter()
        .map(|(authored, exported, _)| {
            if authored == exported {
                (*authored).to_string()
            } else {
                format!("{exported} as {authored}")
            }
        })
        .collect();
    format!(
        "import {{ {} }} from '{}';\n{}",
        specifiers.join(", "),
        crate::runtime_ambient::RUNTIME_SPECIFIER,
        module
    )
}

/// Idents the module already binds: named-import bindings (single-line OR
/// multi-line `import { a, b as c } from '...'`) and top-level declarations
/// (`const/let/var/function/class NAME`). An ambient ident that is already
/// bound must not be re-imported (a duplicate binding is a compile error).
fn module_bound_idents(module: &str) -> std::collections::HashSet<String> {
    let mut out = std::collections::HashSet::new();
    let mut in_import_braces = false;
    for line in module.lines() {
        let t = line.trim();
        if in_import_braces {
            let end = t.find('}');
            for part in t[..end.unwrap_or(t.len())].split(',') {
                bind_specifier(&mut out, part);
            }
            if end.is_some() {
                in_import_braces = false;
            }
            continue;
        }
        if t.starts_with("import") {
            if let Some(lb) = t.find('{') {
                let after = &t[lb + 1..];
                if let Some(rb) = after.find('}') {
                    for part in after[..rb].split(',') {
                        bind_specifier(&mut out, part);
                    }
                } else {
                    for part in after.split(',') {
                        bind_specifier(&mut out, part);
                    }
                    in_import_braces = true;
                }
            }
        } else {
            for kw in ["const ", "let ", "var ", "function ", "class "] {
                if let Some(rest) = t.strip_prefix(kw) {
                    let name: String = rest
                        .chars()
                        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '$')
                        .collect();
                    if !name.is_empty() {
                        out.insert(name);
                    }
                }
            }
        }
    }
    out
}

/// Record the local binding a single import specifier introduces: `orig as bound`
/// binds `bound`, `type X` binds `X`, a bare `name` binds itself.
fn bind_specifier(out: &mut std::collections::HashSet<String>, part: &str) {
    let name = part
        .trim()
        .strip_prefix("type ")
        .unwrap_or(part.trim())
        .trim();
    let name = name.rsplit(" as ").next().unwrap_or(name).trim();
    let bound: String = name
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '$')
        .collect();
    if !bound.is_empty() {
        out.insert(bound);
    }
}

/// Whether `ident` appears in `src` as a standalone identifier (word-boundary
/// match — not a substring of a longer name). Coarse by design: a spurious hit
/// only injects an unused import (tree-shaken away), while a miss would leave a
/// free variable — so this errs toward presence.
fn references_ident(src: &str, ident: &str) -> bool {
    let bytes = src.as_bytes();
    let idb = ident.as_bytes();
    let n = idb.len();
    if n == 0 || bytes.len() < n {
        return false;
    }
    let is_ident_char = |b: u8| b.is_ascii_alphanumeric() || b == b'_' || b == b'$';
    let mut i = 0;
    while i + n <= bytes.len() {
        if &bytes[i..i + n] == idb {
            let before_ok = i == 0 || !is_ident_char(bytes[i - 1]);
            let after_ok = i + n == bytes.len() || !is_ident_char(bytes[i + n]);
            if before_ok && after_ok {
                return true;
            }
        }
        i += 1;
    }
    false
}

/// First occurrence of `needle` outside an HTML comment.
///
/// This runs before there is a tree to walk, so it scans raw text — and comments
/// are exactly where the language's own tokens turn up as prose.
fn find_outside_comments(source: &str, needle: &str) -> Option<usize> {
    let mut i = 0usize;
    while i < source.len() {
        // `i` stays on a char boundary: advancing by byte and then slicing panics
        // on the first non-ASCII character in the file.
        let rest = &source[i..];
        if let Some(after) = rest.strip_prefix("<!--") {
            // An unterminated comment is the parser's error to report, not ours.
            match after.find("-->") {
                Some(rel) => i += 4 + rel + 3,
                None => return None,
            }
            continue;
        }
        if rest.starts_with(needle) {
            return Some(i);
        }
        i += rest.chars().next().map_or(1, char::len_utf8);
    }
    None
}

/// SPEC-PROJECT-STRUCTURE-1 §4 — block order is grammar.
///
/// When both `<scope>` and `<controller>` appear in the source, `<scope>` must
/// precede `<controller>`. Wrong order is a **parse/compile error** (not a
/// lint): legal order is scope → controller → template.
///
/// Tags inside HTML comments are ignored (same scan rule as
/// [`extract_controller_block`]).
pub fn check_sfc_block_order(source: &str) -> Result<(), CompileError> {
    let scope_at = find_outside_comments(source, "<scope");
    let controller_at = find_outside_comments(source, "<controller");
    let partial_at = find_outside_comments(source, "<partial");
    match (scope_at, controller_at) {
        (Some(scope_pos), Some(controller_pos)) if controller_pos < scope_pos => {
            return Err(CompileError::new(
                "`<scope>` must come before `<controller>` — block order is grammar. \
                 Put the `<scope>` block first, then `<controller>`, then `<partial>`, then the template.",
            )
            .with_category("sail-block-order")
            .with_span(Span::new(
                controller_pos,
                controller_pos + "<controller".len(),
            )));
        }
        _ => {}
    }
    if let (Some(controller_pos), Some(partial_pos)) = (controller_at, partial_at) {
        if partial_pos < controller_pos {
            return Err(CompileError::new(
                "`<partial>` must come after `<controller>` — block order is grammar. \
                 `<scope>`, then `<controller>`, then `<partial>`, then the template.",
            )
            .with_category("sail-block-order")
            .with_span(Span::new(partial_pos, partial_pos + "<partial".len())));
        }
    }
    Ok(())
}

/// Extract an inline `<controller>` block for the single-file-component model.
/// Returns the template + the block's raw TS body (the one locked-down
/// `controller({…})` function). `None` if there's no block.
///
/// The block is BLANKED, not deleted: its bytes are replaced with spaces, and
/// its newlines kept. The returned template is therefore byte-for-byte the same
/// LENGTH as the source, so a span into it is a span into the file the author
/// wrote — every diagnostic's line and column land on the right row. Deleting
/// the block instead shifts every template offset left by the block's size, and
/// the compiler reports errors N lines above where they are.
pub fn extract_controller_block(source: &str) -> (String, Option<String>) {
    // Skip comments: a comment above the block that mentions `<controller>` would
    // otherwise be taken as the block's opening tag, and everything from inside the
    // comment through `</controller>` spliced out — carrying the comment's `-->`
    // with it, leaving the file unparseable.
    let Some(start) = find_outside_comments(source, "<controller") else {
        return (source.to_string(), None);
    };
    let Some(open_rel) = source[start..].find('>') else {
        return (source.to_string(), None);
    };
    let body_start = start + open_rel + 1;
    // Close tag must ignore `</controller>` that appears inside usage-example
    // template strings / JS string literals (common in docs corpus helpers).
    let Some(body_end) = find_controller_close(source, body_start) else {
        return (source.to_string(), None);
    };
    let body = wrap_controller_body(source[body_start..body_end].trim());
    let block_end = body_end + "</controller>".len();
    let mut stripped = String::with_capacity(source.len());
    stripped.push_str(&source[..start]);
    // Replacing each byte of a multi-byte char with an ASCII space is safe: the
    // whole char is covered, so the result stays valid UTF-8.
    for b in source[start..block_end].bytes() {
        stripped.push(if b == b'\n' { '\n' } else { ' ' });
    }
    stripped.push_str(&source[block_end..]);
    (stripped, Some(body))
}

/// First `</controller>` at/after `body_start` that is outside JS/TS string,
/// template, and regex literals (and line / block comments).
///
/// Two failure modes share a symptom (empty template scope):
/// - a fake `</controller>` *inside* a string is taken as the close
/// - quote mode desyncs (regex `/'.…` or `${'`'`}`), so the *real* close
///   is treated as still inside a string and the extract returns `None`
pub(crate) fn find_controller_close(source: &str, body_start: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    if body_start > bytes.len() {
        return None;
    }
    let mut i = body_start;
    while i < bytes.len() {
        let c = bytes[i];
        // Line comment — stay out of quote mode for prose like `// don't`.
        if c == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            i += 2;
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        // Block comment.
        if c == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i = (i + 2).min(bytes.len());
            continue;
        }
        // `/'/g` is a regex, not an open quote — otherwise the real close is
        // swallowed and extract returns None.
        if c == b'/' {
            if let Some(next) = crate::scan::try_skip_regex(bytes, i, bytes.len()) {
                i = next;
                continue;
            }
        }
        if c == b'"' || c == b'\'' || c == b'`' {
            i = crate::scan::skip_string_or_template(bytes, i, bytes.len());
            continue;
        }
        if c == b'<'
            && source
                .get(i..)
                .is_some_and(|s| s.starts_with("</controller>"))
        {
            return Some(i);
        }
        // UTF-8 safe advance when not on an ASCII-only match path.
        i += source[i..]
            .chars()
            .next()
            .map(|ch| ch.len_utf8())
            .unwrap_or(1);
    }
    None
}

/// Derive the framework-owned host display from the component's root template
/// element — the host presents as that element naturally would. Block is the
/// framework default, so this emits an override only for inline-natured roots
/// (a link is inline, a label is inline-block, a row/button control is
/// inline-flex). No per-component declaration, no renamed CSS.
fn derive_host_display(source: &str) -> String {
    let display = match first_template_tag(source).as_deref() {
        Some("a") => "inline",
        Some("label") => "inline-block",
        Some("row") | Some("button") | Some("input") | Some("select") | Some("span") => {
            "inline-flex"
        }
        // Block-natured or unknown roots keep the framework block default.
        _ => return String::new(),
    };
    format!(":host {{ display: {display}; }}\n")
}

/// The tag name of the first real template element — skipping HTML comments
/// and the non-visual blocks (`<scope>`, `<controller>`, `<style>`).
fn first_template_tag(source: &str) -> Option<String> {
    const SKIP: &[&str] = &["scope", "controller", "style", "partial"];
    let bytes = source.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'<' {
            i += 1;
            continue;
        }
        if source[i..].starts_with("<!--") {
            match source[i..].find("-->") {
                Some(end) => {
                    i += end + 3;
                    continue;
                }
                None => break,
            }
        }
        // A close tag `</…>` or a non-name char after `<` — step over it.
        let name: String = source[i + 1..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
            .collect();
        if name.is_empty() {
            i += 1;
            continue;
        }
        if SKIP.contains(&name.as_str()) {
            match source[i..].find('>') {
                Some(gt) => {
                    i += gt + 1;
                    continue;
                }
                None => break,
            }
        }
        return Some(name);
    }
    None
}

/// Return the raw body of the `<component>` block (the `{ props, variants, meta }`
/// consumes, then lower it. No `<variants>` block, no desugar.
/// Emit CSS from `(selector-dsl, sx)` variant pairs — the one variant→CSS core.

fn quote_brace_attrs(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut out = String::with_capacity(source.len());
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] != b'<' {
            // Append one UTF-8 char (1–4 bytes). `bytes[i] as char` would
            // mangle multi-byte sequences like `…` (e2 80 a6) into latin-1
            // codepoints and re-encode them as gibberish.
            let mut j = i + 1;
            while j < bytes.len() && (bytes[j] & 0xC0) == 0x80 {
                j += 1;
            }
            out.push_str(&source[i..j]);
            i = j;
            continue;
        }
        // HTML comments — skip the body verbatim. The tag-mode quote
        // tracker doesn't understand comments; an apostrophe inside
        // (e.g. "doesn't") would leak `in_single` state into every
        // later attribute on the page, breaking `={…}` brace-attr
        // quoting for the rest of the file.
        if source[i..].starts_with("<!--") {
            let body_start = i + 4;
            let close_rel = source[body_start..].find("-->");
            let end = match close_rel {
                Some(rel) => body_start + rel + 3,
                None => bytes.len(),
            };
            out.push_str(&source[i..end]);
            i = end;
            continue;
        }
        out.push('<');
        i += 1;
        let mut in_single = false;
        let mut in_double = false;
        while i < bytes.len() {
            let ch = bytes[i];
            if ch == b'"' && !in_single {
                in_double = !in_double;
                out.push('"');
                i += 1;
                continue;
            }
            if ch == b'\'' && !in_double {
                in_single = !in_single;
                out.push('\'');
                i += 1;
                continue;
            }
            if !in_single && !in_double && ch == b'>' {
                out.push('>');
                i += 1;
                break;
            }
            if !in_single
                && !in_double
                && ch == b'='
                && i + 1 < bytes.len()
                && bytes[i + 1] == b'{'
                && let Some(end) = find_close(source, i + 1)
            {
                let expr = &source[i + 1..=end];
                out.push('=');
                out.push('"');
                for c in expr.chars() {
                    if c == '"' {
                        out.push_str("&quot;");
                    } else {
                        out.push(c);
                    }
                }
                out.push('"');
                i = end + 1;
                continue;
            }
            // Not the start of a brace attr — copy verbatim.
            // Use char boundary safe append.
            let ch_start = i;
            // Find next char boundary
            let mut j = ch_start + 1;
            while j < bytes.len() && (bytes[j] & 0xC0) == 0x80 {
                j += 1;
            }
            out.push_str(&source[ch_start..j]);
            i = j;
        }
    }
    out
}

fn find_close(s: &str, open_brace_at: usize) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut depth = 0i32;
    let mut str_quote: Option<u8> = None;
    let mut i = open_brace_at;
    while i < bytes.len() {
        let c = bytes[i];
        if let Some(q) = str_quote {
            if c == b'\\' {
                i += 2;
                continue;
            }
            if c == q {
                str_quote = None;
            }
            i += 1;
            continue;
        }
        if c == b'"' || c == b'\'' || c == b'`' {
            str_quote = Some(c);
            i += 1;
            continue;
        }
        if c == b'{' {
            depth += 1;
        } else if c == b'}' {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

fn expand_self_close(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut out = String::with_capacity(source.len());
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] != b'<' {
            // Append a single UTF-8 char.
            let mut j = i + 1;
            while j < bytes.len() && (bytes[j] & 0xC0) == 0x80 {
                j += 1;
            }
            out.push_str(&source[i..j]);
            i = j;
            continue;
        }
        // HTML comments may contain `>` (`<!-- <file> … -->`). Find `-->`,
        // not the first `>`.
        if source[i..].starts_with("<!--") {
            let end = source[i..]
                .find("-->")
                .map(|e| i + e + 3)
                .unwrap_or(bytes.len());
            out.push_str(&source[i..end]);
            i = end;
            continue;
        }
        // Skip </…> close tags and <!…> directives whole.
        if i + 1 < bytes.len() && (bytes[i + 1] == b'/' || bytes[i + 1] == b'!') {
            if let Some(end_rel) = source[i..].find('>') {
                out.push_str(&source[i..=i + end_rel]);
                i += end_rel + 1;
                continue;
            }
            out.push_str(&source[i..]);
            break;
        }
        // Match a tag name `[A-Za-z][A-Za-z0-9-]*` after `<`.
        let tag_start = i + 1;
        let mut tag_end = tag_start;
        if tag_end >= bytes.len() || !(bytes[tag_end].is_ascii_alphabetic()) {
            // Not actually a tag; copy `<` and continue.
            out.push('<');
            i += 1;
            continue;
        }
        tag_end += 1;
        while tag_end < bytes.len()
            && (bytes[tag_end].is_ascii_alphanumeric() || bytes[tag_end] == b'-')
        {
            tag_end += 1;
        }
        let tag = source[tag_start..tag_end].to_ascii_lowercase();
        // Find matching `>`.
        let mut j = tag_end;
        let mut in_single = false;
        let mut in_double = false;
        while j < bytes.len() {
            let ch = bytes[j];
            if ch == b'"' && !in_single {
                in_double = !in_double;
            } else if ch == b'\'' && !in_double {
                in_single = !in_single;
            } else if !in_single && !in_double && ch == b'>' {
                break;
            }
            j += 1;
        }
        if j == bytes.len() {
            out.push_str(&source[i..]);
            break;
        }
        let head = &source[i..j];
        // Self-close requires an actual `/` before `>`. Trailing whitespace
        // alone (multi-line attr layout `<button\n  …\n>`) MUST NOT be
        // treated as self-close — that would flatten every multi-line
        // element's children into siblings.
        let trimmed_ws = head.trim_end_matches(|c: char| c.is_whitespace());
        let is_self_close = trimmed_ws.ends_with('/');
        if is_self_close && !is_void_tag(&tag) {
            // Strip the slash, then any whitespace before it (`<x  />` → `<x>`).
            let trimmed_head = trimmed_ws
                .trim_end_matches('/')
                .trim_end_matches(|c: char| c.is_whitespace());
            out.push_str(trimmed_head);
            out.push('>');
            out.push_str("</");
            out.push_str(&tag);
            out.push('>');
        } else {
            out.push_str(&source[i..=j]);
        }
        i = j + 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Usage-example template strings may embed a fake `</controller>`; that
    /// must not terminate the real SFC block (docs corpus / configurator pages).
    #[test]
    fn controller_close_inside_template_string_is_ignored() {
        let src = r#"
<controller>
export default controller((ctx) => {
  const ex = () =>
    `<controller>\nexport default controller(() => ({ n: signal(0) }))\n</controller>\n\n<button>{n()}</button>`
  return { ex }
})
</controller>
<button type="button">ok</button>
"#;
        let (stripped, body) = extract_controller_block(src);
        let body = body.expect("real controller body");
        assert!(
            body.contains("const ex"),
            "body must include code after the embedded fake close: {body}"
        );
        assert!(
            body.contains("return { ex }"),
            "body must reach the real end of the controller: {body}"
        );
        assert!(
            stripped.contains("<button type=\"button\">ok</button>"),
            "template after the real close must remain: {stripped}"
        );
        assert!(
            !stripped.contains("export default controller"),
            "controller body must be blanked from the template stream"
        );
    }

    /// Product form: `/'/g` (and fenced / quoted-attr / `{expr}` samples)
    /// must not desync quote mode so the real close is missed.
    #[test]
    fn controller_close_survives_regex_quote_and_sample_literals() {
        let src = r#"
<controller>
export default controller(() => {
  const heading = () => 'ok'
  const escaped = ''.replace(/\\/g, '\\\\').replace(/'/g, "\\'")
  const fenced = "```sail\n<controller>\nexport default controller(() => ({}))\n</controller>\n```"
  const quotedAttr = 'code="</controller>"'
  const exprTmpl = `<code-block code={'</controller>'}>${'</controller>'}</code-block>`
  return { heading, escaped, fenced, quotedAttr, exprTmpl }
})
</controller>
<view name={heading()}></view>
"#;
        let (stripped, body) = extract_controller_block(src);
        let body = body.expect("real controller body");
        assert!(
            body.contains("return { heading"),
            "regex/sample literals must not cut the body short: {body}"
        );
        assert!(
            stripped.contains("name={heading()}"),
            "template after the real close must remain: {stripped}"
        );
        assert!(
            !stripped.contains("export default controller"),
            "controller body must be blanked from the template stream"
        );
    }

    /// A comment may name `<controller>` above the block it describes.
    #[test]
    fn a_comment_may_name_the_controller_block_it_describes() {
        let src = "<!-- counter.sail — when there's state, add an inline <controller> block. -->\n\
                   <controller>\nexport default controller({})\n</controller>\n<p>hi</p>";
        let (stripped, body) = extract_controller_block(src);
        assert!(body.is_some(), "the REAL block is the one extracted");
        assert!(
            stripped.contains("-->"),
            "and the comment survives intact — it was not spliced through:\n{stripped}"
        );
        assert!(
            !stripped.contains("</controller>"),
            "while the real block is gone from the template:\n{stripped}"
        );
    }

    /// ...and a comment with no real block after it must not invent one.
    #[test]
    fn a_controller_named_only_in_a_comment_is_not_a_block() {
        let src = "<!-- state goes in a <controller> block -->\n<p>hi</p>";
        let (stripped, body) = extract_controller_block(src);
        assert!(
            body.is_none(),
            "there is no controller here — only prose about one"
        );
        assert_eq!(stripped, src, "and the source is untouched");
    }

    /// Non-ASCII markup must not panic the scan: advancing by byte and slicing
    /// lands mid-codepoint on the first `…` or `é`.
    #[test]
    fn prose_with_non_ascii_does_not_panic_the_scan() {
        let src = "<aside>…</aside>\n<p>naïve — “curly” ✓</p>\n<controller>\nexport default controller({})\n</controller>";
        let (_, body) = extract_controller_block(src);
        assert!(
            body.is_some(),
            "the block is still found past the non-ASCII text"
        );
        // And with no block at all, the scan must still walk off the end cleanly.
        let (s2, b2) = extract_controller_block("<p>… é — ✓</p>");
        assert!(b2.is_none());
        assert_eq!(s2, "<p>… é — ✓</p>");
    }

    #[test]
    fn mime_slash_star_does_not_eat_controller_or_template() {
        // `image/*` is a MIME token, not a JS block comment. Extract +
        // preprocess must keep the real `</controller>` and the tags after it.
        let src = r#"
<scope>
  style fullW = { size({ w: full }) }
</scope>
<controller>
export default controller(() => {
  const accept = signal('image/*')
  const ex = () => `<file accept="image/*" />`
  return { accept, ex }
})
</controller>
<file accept="image/*" style(fullW) />
<input type="text" placeholder="image/*" style(fullW) />
"#;
        let (stripped, body) = extract_controller_block(src);
        let body = body.expect("controller body");
        assert!(
            body.contains("image/*") && body.contains("return { accept"),
            "MIME slash-star must not cut the controller: {body}"
        );
        assert!(
            stripped.contains("placeholder=\"image/*\""),
            "template after extract must remain: {stripped}"
        );
        let out = preprocess(&stripped).unwrap_or_else(|e| panic!("preprocess: {e:?}"));
        assert!(
            out.contains("sail-file") || out.contains("<file") || out.contains("s-"),
            "file + named style must lower:\n{out}"
        );
        crate::compile(src, crate::CompileOpts::default())
            .unwrap_or_else(|e| panic!("compile: {e:?}"));
    }

    #[test]
    fn boolean_attr_before_named_style_self_close_stays_a_tag() {
        // `multiple style(fullW) />` must not become `multiple/>`.
        let src = r#"
<scope>
  style fullW = { size({ w: full }) }
</scope>
<file accept="image/*" multiple style(fullW) />
"#;
        let out = preprocess(src).unwrap_or_else(|e| panic!("preprocess: {e:?}"));
        assert!(
            !out.contains("multiple/"),
            "boolean attr must not glue to self-close slash:\n{out}"
        );
        crate::parse::parse(src).unwrap_or_else(|e| panic!("parse: {e:?}"));
    }

    #[test]
    fn html_comment_may_contain_tag_gt() {
        let src = "<!-- <file> docs host -->\n<p>ok</p>";
        let out = expand_self_close(src);
        assert!(
            out.contains("<!-- <file> docs host -->"),
            "comment must keep inner `>`:\n{out}"
        );
        crate::parse::parse(src).unwrap_or_else(|e| panic!("parse: {e:?}"));
    }

    #[test]
    fn quotes_unquoted_brace_attr() {
        let out = quote_brace_attrs(r#"<x disabled={a || b} />"#);
        assert!(out.contains(r#"disabled="{a || b}""#));
    }

    #[test]
    fn escapes_double_quote_inside_expr() {
        let out = quote_brace_attrs(r#"<x v={a("z")}>"#);
        assert!(out.contains(r#"v="{a(&quot;z&quot;)}""#));
    }

    #[test]
    fn expands_non_void_selfclose() {
        let out = expand_self_close("<custom-tag attr=\"v\" />");
        assert_eq!(out, "<custom-tag attr=\"v\"></custom-tag>");
    }

    #[test]
    fn leaves_void_selfclose() {
        let out = expand_self_close("<br />");
        assert_eq!(out, "<br />");
    }

    #[test]
    fn multiline_open_tag_is_not_self_close() {
        // Multi-line attribute layout — trailing newline before `>` must
        // NOT be treated as a self-close marker. Regression for sail-button
        // emitting children as siblings instead of nesting them inside.
        let src = "<button\n  type=\"button\"\n  disabled\n>\n  <span>x</span>\n</button>";
        let out = expand_self_close(src);
        assert_eq!(
            out, src,
            "multi-line open tag should pass through unchanged"
        );
    }

    #[test]
    fn slash_with_leading_whitespace_is_self_close() {
        // `<x  />` (whitespace before `/`) is still XHTML-style self-close.
        let out = expand_self_close("<x  />");
        assert_eq!(out, "<x></x>");
    }

    #[test]
    fn sx_merges_into_existing_class_attr() {
        // Regression: `<span class="badge" sx={…}>` must merge author class +
        // Style classes into one attribute (not two setAttribute("class") writes).
        let out =
            preprocess(r#"<span class="badge" style({ pad(4) })> x</span>"#).expect("preprocess");
        assert!(
            out.contains(r#"class="badge "#) && out.contains("s-"),
            "author class + Style classes must merge into one attribute, got:\n{out}"
        );
        assert_eq!(
            out.matches("class=").count(),
            1,
            "exactly one class attr:\n{out}"
        );
    }

    #[test]
    fn sx_before_class_also_merges() {
        let out =
            preprocess(r#"<span class="badge" style({ pad(4) })> x</span>"#).expect("preprocess");
        assert!(
            out.contains("s-") && out.contains("badge"),
            "merge must work regardless of attribute order, got:\n{out}"
        );
        assert_eq!(out.matches("class=").count(), 1);
    }

    #[test]
    fn dynamic_class_with_sx_is_left_alone() {
        // A `{…}` class binding is the emitter's business — the merge pass
        // must not fold static Style classes into an expression it can't evaluate.
        let out =
            preprocess(r#"<span class={cls()} style({ pad(4) })> x</span>"#).expect("preprocess");
        assert_eq!(
            out.matches("class=").count(),
            2,
            "dynamic class + Style stays two attrs for the emitter, got:\n{out}"
        );
    }

    #[test]
    fn quoted_gt_inside_attr_does_not_break_class_merge() {
        let out = preprocess(r#"<span title="a > b" class="x" style({ pad(4) })> y</span>"#)
            .expect("preprocess");
        assert!(
            out.contains(r#"class="x "#) && out.contains("s-"),
            "got:\n{out}"
        );
    }

    #[test]
    fn freeform_banned_input_types_error() {
        for (src, needle) in [
            (r#"<input type="range" />"#, "slider"),
            (r#"<input type="checkbox" />"#, "checkbox"),
            (r#"<input type="file" />"#, "file"),
            (r#"<input type="color" />"#, "color"),
            (r#"<input type="date" />"#, "date"),
        ] {
            let err = preprocess(src).expect_err(src);
            let msg = format!("{err:?}");
            assert!(
                msg.contains("not part of the Sail API") && msg.contains(needle),
                "src={src} msg={msg}"
            );
        }
        // Preferred tags still lower
        let ok = preprocess(r#"<slider value="40" /><checkbox /><file /><color /><date />"#)
            .expect("product tags ok");
        assert!(
            ok.contains("type=\"range\"")
                || ok.contains("type='range'")
                || ok.contains("sail-slider"),
            "{ok}"
        );
    }

    #[test]
    fn author_void_l1_self_close_splices() {
        let out =
            preprocess(r#"<card><avatar name="Ada" tone="primary" /></card>"#).expect("preprocess");
        assert!(
            out.contains("sail-avatar"),
            "void self-close must splice L1 structure:\n{out}"
        );
        assert!(
            !out.contains("<avatar"),
            "bare <avatar> must not remain after splice:\n{out}"
        );
    }

    #[test]
    fn author_void_l1_omitted_closer_splices() {
        let out = preprocess(r#"<column><separator><p>after</p></column>"#).expect("preprocess");
        assert!(
            out.contains("sail-separator") || out.contains("<hr"),
            "omitted closer must splice:\n{out}"
        );
        assert!(
            !out.contains("<separator"),
            "bare <separator> must not remain:\n{out}"
        );
        assert!(out.contains("<p>") || out.contains("after"), "{out}");
    }

    #[test]
    fn stdlib_spinner_lowers_at_use_site() {
        let out = preprocess(
            r#"<div class="page"><spinner size="sm" aria-label="Saving"></spinner></div>"#,
        )
        .expect("preprocess");
        // Root class stamped, use-site attrs merged, use-site wins the
        // aria-label collision with the snippet default.
        assert!(out.contains("sail-spinner"), "{out}");
        assert!(
            !out.contains("sail-spinner spin"),
            "no residual part class: {out}"
        );
        assert!(out.contains(r#"size="sm""#), "{out}");
        assert!(out.contains(r#"aria-label="Saving""#), "{out}");
        assert!(
            !out.contains("aria-label=\"Loading\""),
            "use-site wins: {out}"
        );
        // Ring child is structure + Style atoms (s-animatespin etc.), not a part class.
        assert!(
            out.contains("aria-hidden=\"true\"") && out.contains("s-animatespin"),
            "structure spliced with Style ring paint: {out}"
        );
        // The SKIN — including its @keyframes — ships in element-skins.css, not in the
        // component's emitted <style>. This test guards the LOWERING, which is what
        // changed nothing: structure, root class, and use-site attributes.
        assert!(
            !out.contains(".sail-spinner[size='sm']"),
            "skin is in the sheet: {out}"
        );
        assert!(
            !out.contains("@keyframes sail-spin"),
            "keyframes ride with it: {out}"
        );
    }

    #[test]
    fn stdlib_separator_lowers_and_dedupes_skin() {
        let out =
            preprocess("<separator orientation=\"vertical\"></separator><separator></separator>")
                .expect("preprocess");
        assert_eq!(out.matches("<hr").count(), 2, "{out}");
        assert!(out.contains("sail-separator"), "{out}");
        assert!(out.contains(r#"orientation="vertical""#), "{out}");
        // The SKIN is not here. Stdlib element skins ship in the framework stylesheet
        // (`element-skins.css`), not spliced into every component's emitted <style> —
        // inlining duplicated the same rules into every chunk that used the element
        // (28.7 KB of CSS inside the JS of one 12-component page). What this test
        // guards is the LOWERING, which is unchanged: the element becomes the right
        // structure with the right classes and the use-site attributes on it.
        assert!(
            !out.contains("--tone-bg: var(--border-subtle)"),
            "the skin comes from element-skins.css, not the emitted style: {out}"
        );
    }

    #[test]
    fn stdlib_element_children_fill_default_slot_or_drop() {
        // spinner is author-void (no `<slot` in the stdlib source). A
        // following `<b>` is a sibling, not a projected child.
        let out = preprocess("<spinner><b>x</b></spinner>").expect("preprocess");
        assert!(
            out.contains("sail-spinner") && !out.contains("<spinner"),
            "void L1 must splice: {out}"
        );
    }

    #[test]
    fn stdlib_alert_projects_named_slots_end_to_end() {
        // The first registry element with named slots — proves compile-time
        // projection distributes use-site slotted children into the matching
        // regions (icon / title / actions) and the un-slotted node into body.
        let out = preprocess(
            r#"<alert tone="success" dismissible><svg slot="icon" class="i"></svg><span slot="title">Saved</span>All changes saved.<button slot="actions">Undo</button></alert>"#,
        )
        .expect("preprocess");
        // root merges use-site attrs
        assert!(out.contains("sail-alert"), "root class stamped: {out}");
        assert!(out.contains(r#"tone="success""#), "tone merged: {out}");
        assert!(out.contains("dismissible"), "dismissible merged: {out}");
        assert!(
            out.contains("use:liveRegion"),
            "liveRegion rides the splice: {out}"
        );
        assert!(
            out.contains("use:dismissControl"),
            "dismissControl rides the splice: {out}"
        );
        // each slot region receives its projected child, none survive as <slot>
        assert!(
            !out.contains("<slot"),
            "no slot tags survive projection: {out}"
        );
        // Structure projection (no residual part classes).
        assert!(
            out.contains(r#"<svg slot="icon" class="i"></svg>"#),
            "icon slot projected: {out}"
        );
        assert!(
            out.contains(r#"<span slot="title">Saved</span>"#),
            "title slot projected: {out}"
        );
        assert!(
            out.contains("All changes saved."),
            "un-slotted body projected: {out}"
        );
        // The slotted <button> is a native stdlib tag — stamping adds its
        // sail-button + s-* classes (and stamped defaults like `size`) ahead
        // of / around the use-site slot= attribute.
        assert!(
            out.contains("sail-button")
                && out.contains(r#"slot="actions""#)
                && out.contains(">Undo</button>"),
            "actions slot projected: {out}"
        );
    }

    #[test]
    fn stdlib_hyphenated_element_lowers_and_projects() {
        // empty-state is the first multi-word (hyphenated) L1 element — proves
        // the registry-gated naming seam (a hyphenated REGISTERED tag compiles
        // away) plus named-slot projection through it.
        let out = preprocess(
            r#"<empty-state variant="placeholder"><svg slot="icon"></svg><span slot="title">No results</span>Try a different search.<button slot="action">Reset</button></empty-state>"#,
        )
        .expect("preprocess");
        assert!(
            out.contains("sail-empty-state"),
            "root class stamped: {out}"
        );
        assert!(
            out.contains(r#"variant="placeholder""#),
            "attr merged: {out}"
        );
        assert!(
            out.contains("use:variantLive"),
            "action rides the splice: {out}"
        );
        assert!(!out.contains("<slot"), "no slot survives: {out}");
        // Named slots project into structure regions (no residual part classes).
        assert!(
            out.contains(r#"aria-hidden="true""#) && out.contains(r#"<svg slot="icon"></svg>"#),
            "icon slot: {out}"
        );
        assert!(
            out.contains(r#"<span slot="title">No results</span>"#),
            "title slot: {out}"
        );
        assert!(
            out.contains("Try a different search."),
            "default slot copy: {out}"
        );
        // Same native stamping as alert: the slotted <button> carries its
        // sail-button + s-* classes around the use-site slot= attribute.
        assert!(
            out.contains("sail-button")
                && out.contains(r#"slot="action""#)
                && out.contains(">Reset</button>"),
            "action slot: {out}"
        );
        assert!(
            !out.contains("--es-"),
            "empty-state structure: no residual --es-* custom props: {out}"
        );
    }

    #[test]
    fn tab_title_attr_fills_prop_hole() {
        let out = preprocess(r#"<tabs><tab title="One">Panel</tab></tabs>"#).expect("preprocess");
        assert!(
            out.contains(">One<") || out.contains(">One\n"),
            "use-site title must fill <!--sail:prop:title-->:\n{out}"
        );
        assert!(
            !out.contains("<!--sail:prop:title-->"),
            "prop hole must not survive a static title:\n{out}"
        );
    }

    #[test]
    fn project_children_partitions_by_slot_attr() {
        let (def, named) =
            project_children(r#"<i slot="leading">L</i>Body<b slot="actions">A</b>more"#);
        assert_eq!(def, "Bodymore", "un-slotted nodes → default, in order");
        assert_eq!(
            named.get("leading").map(String::as_str),
            Some(r#"<i slot="leading">L</i>"#)
        );
        assert_eq!(
            named.get("actions").map(String::as_str),
            Some(r#"<b slot="actions">A</b>"#)
        );
    }

    #[test]
    fn project_children_assigns_whole_subtree_not_nested_slot() {
        // a `slot=` deeper in the tree belongs to that descendant, not hoisted.
        let (def, named) = project_children(r#"<div slot="a"><span slot="b">in</span></div>tail"#);
        assert_eq!(
            named.get("a").map(String::as_str),
            Some(r#"<div slot="a"><span slot="b">in</span></div>"#)
        );
        assert!(named.get("b").is_none(), "nested slot not hoisted");
        assert_eq!(def, "tail");
    }

    #[test]
    fn fill_slots_projects_named_default_and_keeps_fallback() {
        let mut named = std::collections::HashMap::new();
        named.insert("leading".to_string(), "<svg></svg>".to_string());
        let rest = r#">.<span class="lead"><slot name="leading"></slot></span><slot></slot><span><slot name="empty">FB</slot></span></div>"#;
        let out = fill_slots(rest, "DEFAULT", &named);
        assert!(
            out.contains(r#"<span class="lead"><svg></svg></span>"#),
            "named slot filled: {out}"
        );
        assert!(out.contains(">DEFAULT<"), "default slot filled: {out}");
        assert!(
            out.contains(">FB<"),
            "unmatched named slot keeps fallback: {out}"
        );
        assert!(!out.contains("<slot"), "no slot tags survive: {out}");
    }

    #[test]
    fn stdlib_icon_lowers_at_use_site() {
        let out = preprocess(r#"<icon size="lg"><svg viewBox="0 0 24 24"></svg></icon>"#)
            .expect("preprocess");
        // Root class stamped, use-site attr forwarded, slotted svg spliced in.
        assert!(out.contains("sail-icon"), "{out}");
        assert!(out.contains(r#"size="lg""#), "{out}");
        assert!(
            out.contains("<svg viewBox=\"0 0 24 24\"></svg>"),
            "svg slotted: {out}"
        );
        // The skin ships in element-skins.css; the LOWERING is what this guards.
        assert!(
            !out.contains(".sail-icon[size='lg']"),
            "skin is in the sheet: {out}"
        );
        assert_eq!(
            out.matches("--sz: var(--icon-size-md)").count(),
            0,
            "skin once: {out}"
        );
    }

    #[test]
    fn stdlib_label_is_native_kind_stamped_not_spliced() {
        // label's template root IS <label> — NATIVE KIND: the tag is never
        // spliced (splicing would rewrite every real HTML label inside
        // component templates — the sonnet-agent find). But the host paint
        // (root class + atomic sx classes) reaches real labels only by being
        // STAMPED onto the use-site tag — nothing else ever adds it.
        let out = preprocess(r#"<label for="email" required>Email</label>"#).expect("preprocess");
        assert!(
            out.contains("sail-label"),
            "native kind stamps its host classes: {out}"
        );
        assert!(
            out.contains(r#"for="email""#)
                && out.contains("required")
                && out.contains(">Email</label>"),
            "use-site attributes survive stamping: {out}"
        );
        // No splice: the element's internals (the .req marker span) do NOT
        // appear at the use site.
        assert!(
            !out.contains(r#"class="req""#),
            "native kind never splices: {out}"
        );
        // The SKIN is not here. Stdlib element skins ship in the framework stylesheet
        // (`element-skins.css`), not spliced into every component's emitted <style> —
        // inlining duplicated the same rules into every chunk that used the element
        // (28.7 KB of CSS inside the JS of one 12-component page). What this test
        // guards is the LOWERING, which is unchanged: the element becomes the right
        // structure with the right classes and the use-site attributes on it.
        assert!(
            !out.contains("font-family: var(--font-body)"),
            "the tag-keyed skin ships in element-skins.css: {out}"
        );
    }

    #[test]
    fn select_stamps_pad_s_ladder_and_pad_e_chevron_room() {
        let out =
            preprocess(r#"<select size="sm"><option>All</option></select>"#).expect("preprocess");
        assert!(out.contains("sail-select"), "host stamp: {out}");
        assert!(
            out.to_ascii_lowercase().contains("pade8") || out.contains("padE8"),
            "chevron room on base: {out}"
        );
        assert!(
            out.to_ascii_lowercase().contains("pads25"),
            "size ladder uses pads not padX: {out}"
        );
        assert!(
            !out.contains("s-sizesm-padX") && !out.contains("s-padX3"),
            "size/base must not set padX (clobbers end pad): {out}"
        );
        assert!(
            out.contains("wfull") || out.contains("w-full"),
            "bare select keeps full width: {out}"
        );
    }

    #[test]
    fn select_drops_default_wfull_when_author_sets_width_on_box_channel() {
        // Bare w/min-w attrs are retired — author width is box channel only.
        let via_w = preprocess(
            r#"<select size="sm" style({ size({ w: 28 }) })> <option>x</option></select>"#,
        )
        .expect("box w");
        assert!(
            via_w.contains("s-w28") || via_w.contains("w28"),
            "author w: {via_w}"
        );
        assert!(
            !via_w.contains("s-wfull"),
            "default w(full) dropped for box w: {via_w}"
        );

        let via_min = preprocess(
            r#"<select size="sm" style({ size({ min-w: ch-16, w: ch-20 }) })> <option>x</option></select>"#,
        )
        .expect("box minW/w");
        assert!(
            !via_min.contains("s-wfull"),
            "default w(full) dropped for minW/w: {via_min}"
        );

        let via_sx = preprocess(
            r#"<select size="sm" style({ size({ w: 24 }) })> <option>x</option></select>"#,
        )
        .expect("sx w");
        assert!(
            !via_sx.contains("s-wfull"),
            "default w(full) dropped for Style w: {via_sx}"
        );
        assert!(
            via_sx.contains("s-w24") || via_sx.contains("w24"),
            "author sx w: {via_sx}"
        );
    }

    #[test]
    fn stdlib_pass_never_touches_style_blocks() {
        // The other sonnet-agent find: CSS text mentioning an element name
        // (a /* comment */ or selector containing `<label>`-ish text) must
        // pass through untouched — <style> blocks are opaque to the pass.
        let src = "<style>\n/* the <spinner> and <label> here are prose */\n.x { color: red; }\n</style>\n<div class=\"y\">z</div>";
        let out = preprocess(src).expect("preprocess");
        assert!(
            out.contains("/* the <spinner> and <label> here are prose */"),
            "{out}"
        );
        assert!(out.contains(".x { color: red; }"), "{out}");
        assert!(
            !out.contains("sail-spinner"),
            "no splice inside style: {out}"
        );
    }

    #[test]
    fn bare_enum_shorthand_desugars_on_framework_tags() {
        let out =
            preprocess(r#"<button primary solid sm on:click={f}>Go</button>"#).expect("preprocess");
        assert!(out.contains(r#"tone="primary""#), "{out}");
        assert!(out.contains(r#"variant="solid""#), "{out}");
        assert!(out.contains(r#"size="sm""#), "{out}");
    }

    #[test]
    fn bare_enum_shorthand_does_not_rewrite_inside_style_product() {
        // Regression: compound row values like `variant: outline` must stay as
        // the enum member — not rewrite to variant="outline" which parses as
        // compound value "variant" → CSS [variant='variant'].
        let src = r#"
<button style({ compound: [ { orientation: vertical, variant: outline, margin({ t: -px }) } ] })>x</button>
"#;
        let out = preprocess(src).expect("preprocess");
        assert!(
            !out.contains("variant='variant'"),
            "skin CSS poisoned: {out}"
        );
        assert!(
            out.contains("variant='outline'") || out.contains("orientationverticalvariantoutline"),
            "expected vertical+outline compound classes: {out}"
        );
    }

    #[test]
    fn bare_enum_shorthand_skips_legacy_sail_prefixed_tags() {
        // The `sail-` prefix is no longer a "framework tag" marker for the
        // prop shorthand. The L1 element is the BARE name (`<button>`,
        // `<chip>`); the retired `sail-`-prefixed spelling was the old
        // component namespace, and a tag carrying it now passes through
        // untouched (and likely a compile error downstream if there is no
        // matching custom element). This test is the boundary: the bare
        // words remain literal attributes, not desugared prop values.
        let out = preprocess(r#"<sail-chip accent outline>x</sail-chip>"#).expect("preprocess");
        assert!(!out.contains(r#"tone="accent""#), "no tone desugar: {out}");
        assert!(
            !out.contains(r#"variant="outline""#),
            "no variant desugar: {out}"
        );
        assert!(out.contains(" accent"), "bare word passes through: {out}");
        assert!(out.contains(" outline"), "bare word passes through: {out}");
    }

    #[test]
    fn bare_enum_shorthand_respects_explicit_and_scope() {
        // Explicit attribute wins — no double tone.
        let out = preprocess(r#"<button tone="info" primary>x</button>"#).expect("preprocess");
        assert!(out.contains(r#"tone="info""#), "{out}");
        assert!(!out.contains(r#"tone="primary""#), "explicit wins: {out}");
        assert!(out.contains(" primary"), "bare word passes through: {out}");
        // Userland/HTML tags are untouched.
        let out = preprocess(r#"<my-card primary>x</my-card><p sm>y</p>"#).expect("preprocess");
        assert!(out.contains("<my-card primary>"), "{out}");
        assert!(out.contains("<p sm>"), "{out}");
        // Boolean props and quoted values never collide.
        let out =
            preprocess(r#"<button disabled data-x="sm primary">x</button>"#).expect("preprocess");
        assert!(out.contains(" disabled"), "{out}");
        assert!(
            out.contains(r#"data-x="sm primary""#),
            "quoted values untouched: {out}"
        );
    }

    #[test]
    fn retired_component_tag_is_not_a_known_element() {
        // `<component>` was replaced by `<scope>`. A stray one parses as a
        // plain bare tag and dies in the reserved-names check like any other
        // unknown element — no bespoke error, no fossilization. (The tag is
        // concatenated so fixture codemods can never mistake it for a block.)
        let src = format!("<{t}>x</{t}><div>x</div>", t = "compo\u{6e}ent");
        let program = crate::parse::parse(&src).expect("parses as a template element");
        let err = crate::check::reserved_names::check(&program).unwrap_err();
        assert!(
            err.message.to_lowercase().contains("element"),
            "{}",
            err.message
        );
    }

    #[test]
    fn multibyte_chars_do_not_panic_class_merge() {
        // Regression: the merge pass sliced `source[i + 1..]` while `i` sat on
        // a multi-byte char (sail-date-picker's `‹`/`›` in a comment) — panic.
        let src =
            "<!-- ‹prev / next› —— déjà-vu … -->\n<span class=\"x\" style({ pad(4) })> é…‹›</span>";
        let out = preprocess(src).expect("preprocess");
        assert!(
            out.contains(r#"class="x "#) && out.contains("s-"),
            "got:\n{out}"
        );
        assert!(out.contains("é…‹›"), "text content preserved:\n{out}");
    }

    #[test]
    fn full_pipeline() {
        let src = r#"<time datetime="2026-07-19" />
<button on:click={() => x()}>hi</button>"#;
        let out = preprocess(src).expect("preprocess");
        assert!(out.contains("</time>"));
        assert!(
            out.contains(r#"on:click="{() =&gt; x()}""#)
                || out.contains(r#"on:click="{() => x()}""#)
        );
    }

    #[test]
    fn html_comment_with_apostrophe_does_not_leak_quote_state() {
        // Regression: an apostrophe inside an HTML comment (e.g.
        // "doesn't") used to toggle the preprocessor's `in_single`
        // tracker, which then leaked into every subsequent attribute
        // — every brace-attr after the comment failed to get quoted
        // and the HTML parser later rejected the unquoted `={` shape.
        // Real repro: views/eval/page.sail's header comment.
        let src = r#"<!--
  Note: the renderable subset doesn't admit `.items({…})` here.
-->
<article data-passed={c ? 'true' : 'false'}></article>"#;
        let out = preprocess(src).expect("preprocess");
        assert!(
            out.contains(r#"data-passed="{c ? 'true' : 'false'}""#),
            "brace-attr after a comment containing an apostrophe must \
             still get quoted, got:\n{out}",
        );
    }

    #[test]
    fn html_comment_with_nested_tag_does_not_leak() {
        // The fake `<time>` inside this comment used to be read as a
        // tag boundary. The first `>` (inside `<time>`) ended the
        // inner-while's "tag mode" early; subsequent text was
        // misclassified. With the comment-aware skip, the body is
        // copied verbatim and the real `<button>` is parsed correctly.
        let src = r#"<!-- see <time> usage --><button on:click={() => f()}>x</button>"#;
        let out = preprocess(src).expect("preprocess");
        assert!(out.contains("</time>") || out.contains("<time>"));
        assert!(
            out.contains(r#"on:click="{() => f()}""#)
                || out.contains(r#"on:click="{() =&gt; f()}""#),
            "on:click brace-attr after comment must get quoted, got:\n{out}",
        );
    }

    #[test]
    fn lower_sx_handles_multibyte_chars_without_panicking() {
        // Regression: `lower_sx` byte-walked with `i += 1` and sliced
        // `source[i..]`, panicking mid-codepoint on any multi-byte char — the
        // crash the async golden fixtures hit (an em-dash / ellipsis in copy).
        let src = "<p>Failed — retry…</p>\n<sail-card style({ pad(4) })> café ☕</sail-card>";
        let out = preprocess(src).expect("preprocess");
        assert!(
            out.contains("café ☕"),
            "multibyte content preserved: {out}"
        );
        assert!(
            out.contains("s-") && out.contains("class="),
            "Style still lowered: {out}"
        );
    }

    #[test]
    fn lowers_column_to_div_with_classes_and_style() {
        let out = preprocess(r#"<column gap="4"><p>hi</p></column>"#).expect("preprocess");
        assert!(
            out.contains(r#"<div class="sail-col sail-g-4 sail-ai-stretch">"#),
            "col lowered: {out}"
        );
        assert!(out.contains("</div>"), "close tag lowered: {out}");
        assert!(!out.contains("<column"), "no column tag remains: {out}");
        assert!(
            out.contains(".sail-col{display:flex;flex-direction:column"),
            "base css: {out}"
        );
        assert!(
            out.contains(".sail-g-4{gap:var(--space-4)}"),
            "gap css: {out}"
        );
    }

    #[test]
    fn box_channel_grow_on_any_host_mints_style_class() {
        // Participation is Style only (`style({ grow(1) })` → `s-grow1`).
        let out = preprocess(r#"<div style({ grow(1) })>x</div>"#).expect("preprocess");
        assert!(out.contains("s-grow1"), "grow class: {out}");
        assert!(out.contains(".s-grow1{flex-grow:1}"), "grow css: {out}");
        assert!(
            !out.contains(" grow") && !out.contains("grow="),
            "no bare attr: {out}"
        );
    }

    #[test]
    fn box_channel_grow_on_layout_keyword_merges_into_div() {
        let out =
            preprocess(r#"<column gap="2" style({ grow(1) })> x</column>"#).expect("preprocess");
        assert!(out.contains("sail-col"), "layout base: {out}");
        assert!(out.contains("s-grow1"), "grow class folded: {out}");
        assert!(out.contains(".s-grow1{flex-grow:1}"), "grow css: {out}");
        assert!(!out.contains("grow="), "grow not raw attr: {out}");
        assert_eq!(out.matches("class=").count(), 1, "one class attr: {out}");
    }

    #[test]
    fn box_channel_w_full_and_shrink_zero() {
        let out = preprocess(r#"<main style({ size({ w: full }).shrink(0) })> x</main>"#)
            .expect("preprocess");
        assert!(
            out.contains("s-wfull") || out.contains("s-sizewfull"),
            "w full: {out}"
        );
        assert!(out.contains("s-shrink0"), "shrink 0: {out}");
        assert!(
            out.contains("width:100%") || out.contains("width: 100%"),
            "w css: {out}"
        );
        assert!(
            out.contains(".s-shrink0{flex-shrink:0}"),
            "shrink css: {out}"
        );
    }

    #[test]
    fn bare_participation_attrs_are_not_lowered() {
        // Residual attr lower deleted — bare grow/w stay unconsumed (lint refuses).
        let out = preprocess(r#"<div grow w="full">x</div>"#).expect("preprocess");
        assert!(
            !out.contains("s-grow1") && !out.contains("s-wfull"),
            "must not mint from bare attrs: {out}"
        );
        assert!(
            out.contains("grow") || out.contains("w="),
            "attrs left for lint surface: {out}"
        );
    }

    #[test]
    fn layout_element_sx_merges_into_one_class() {
        // Style lowers first (→ class), then layout folds it into the div's classes.
        let out =
            preprocess(r#"<column gap="2" style({ pad(4) })> x</column>"#).expect("preprocess");
        assert!(
            out.contains("sail-col") && out.contains("sail-g-2") && out.contains("s-"),
            "merged: {out}"
        );
        assert_eq!(
            out.matches("class=").count(),
            1,
            "exactly one class attr: {out}"
        );
    }

    #[test]
    fn passes_through_id_and_merges_static_class() {
        let out = preprocess(r#"<row id="bar" class="mine">x</row>"#).expect("preprocess");
        assert!(out.contains(r#"id="bar""#), "id preserved: {out}");
        assert!(
            out.contains("sail-row") && out.contains("mine"),
            "class merged: {out}"
        );
    }

    #[test]
    fn nested_layout_lowers_every_box() {
        let out =
            preprocess(r#"<column><row><center>x</center></row></column>"#).expect("preprocess");
        assert_eq!(out.matches("<div").count(), 3, "three divs: {out}");
        assert_eq!(out.matches("</div>").count(), 3, "three closes: {out}");
    }

    // --- `<grid cols="…">` --------------------------------------------------
    //
    // The original bug: `cols="7"` was treated as an unknown attribute and
    // silently flowed through to the lowered `<div>` as a dead HTML attr —
    // collapsing the calendar grid to a single column with no compile error.
    // These tests pin both halves of the fix:
    //   (1) `<grid cols="7">` lowers to a fixed-column grid (the API works).
    //   (2) An unknown attribute name on a layout keyword is dropped from
    //       the emit (the lint surfaces it as a hard error before this
    //       pass runs; we defend in depth here so a future regression on
    //       the lint side can't quietly produce broken HTML again).

    #[test]
    fn grid_cols_emits_repeat_minmax_track_list() {
        let out = preprocess(r#"<grid cols="7" gap="4">x</grid>"#).expect("preprocess");
        assert!(
            out.contains(".sail-gc-7{grid-template-columns:repeat(7,minmax(0,1fr))}"),
            "cols lowering produced the repeat(minmax) track: {out}"
        );
        assert!(
            out.contains("sail-gc-7"),
            "the atomic class is present: {out}"
        );
        assert!(
            out.contains("sail-grid"),
            "the base grid class still applied: {out}"
        );
        assert!(out.contains("sail-g-4"), "other axes still apply: {out}");
        assert!(
            !out.contains("cols=\"7\""),
            "cols doesn't leak as a raw attr: {out}"
        );
    }

    #[test]
    fn grid_cols_precedence_min_is_suppressed() {
        // `cols` and `min` both own `grid-template-columns`; the fixed count
        // wins. With both present the responsive axis is dropped so the
        // class list doesn't double the property.
        let out = preprocess(r#"<grid cols="5" min="lg">x</grid>"#).expect("preprocess");
        assert!(out.contains("sail-gc-5"), "cols class applied: {out}");
        assert!(!out.contains("sail-gm-"), "min class suppressed: {out}");
        assert!(
            !out.contains("auto-fill"),
            "no auto-fill track when cols is set: {out}"
        );
    }

    #[test]
    fn grid_cols_dynamic_lowers_to_a_class_binding_with_all_rules() {
        // Same dynamic-axis shape as `<row align={pick()}>`: an inline
        // expression concatenated with the prefix, and EVERY rule for the
        // axis injected so any runtime value matches.
        let out = preprocess(r#"<grid cols={n()}>x</grid>"#).expect("preprocess");
        assert!(
            out.contains("'sail-gc-' + (n())"),
            "dynamic cols fragment: {out}"
        );
        for n in 1..=12 {
            assert!(
                out.contains(&format!(
                    ".sail-gc-{n}{{grid-template-columns:repeat({n},minmax(0,1fr))}}"
                )),
                "rule for cols={n} present: {out}"
            );
        }
    }

    // --- `<row nowrap>` / `<grid rows="N">` --------------------------------

    #[test]
    fn row_nowrap_emits_flex_wrap_override() {
        let out = preprocess(r#"<row nowrap gap="2">x</row>"#).expect("preprocess");
        assert!(out.contains("sail-nowrap"), "nowrap class present: {out}");
        assert!(
            out.contains(".sail-row.sail-nowrap{flex-wrap:nowrap}"),
            "flex-wrap:nowrap rule emitted: {out}"
        );
        // Injected AFTER the base row rule so it wins by source order.
        let base = out.find(".sail-row{").expect("base row rule");
        let nowrap = out.find(".sail-nowrap{").expect("nowrap rule");
        assert!(nowrap > base, "nowrap rule follows the base: {out}");
        assert!(
            !out.contains("nowrap=\""),
            "nowrap doesn't leak as a raw attr: {out}"
        );
        assert!(
            !out.contains(" nowrap>") && !out.contains(" nowrap "),
            "nowrap attr consumed: {out}"
        );
    }

    #[test]
    fn row_inline_nowrap_composes() {
        let out = preprocess(r#"<row inline nowrap>x</row>"#).expect("preprocess");
        assert!(out.contains("sail-row-i"), "inline base: {out}");
        assert!(out.contains("sail-nowrap"), "nowrap override: {out}");
        assert!(
            out.contains(".sail-row-i{display:inline-flex"),
            "inline base rule: {out}"
        );
        assert!(
            out.contains(".sail-row.sail-nowrap{flex-wrap:nowrap}"),
            "nowrap rule: {out}"
        );
    }

    #[test]
    fn row_cluster_hug_nowrap_intrinsic() {
        let out = preprocess(r#"<row cluster gap="2">x</row>"#).expect("preprocess");
        assert!(
            out.contains("sail-row-i"),
            "cluster is the inline hug: {out}"
        );
        assert!(out.contains("sail-nowrap"), "cluster nowraps: {out}");
        assert!(out.contains("sail-cluster"), "cluster class: {out}");
        assert!(out.contains("sail-g-2"), "gap survives on the pack: {out}");
        assert!(
            out.contains(".sail-row-i.sail-cluster>*")
                && out.contains("min-width:auto")
                && out.contains("flex:0 0 auto"),
            "cluster kids stay intrinsic: {out}"
        );
        assert!(
            !out.contains(" cluster>") && !out.contains(" cluster="),
            "cluster attr consumed: {out}"
        );
    }

    #[test]
    fn grid_rows_emits_repeat_minmax_track_list() {
        let out = preprocess(r#"<grid rows="3" gap="4">x</grid>"#).expect("preprocess");
        assert!(
            out.contains(".sail-gr-3{grid-template-rows:repeat(3,minmax(0,1fr))}"),
            "rows lowering produced the repeat(minmax) track: {out}"
        );
        assert!(
            out.contains("sail-gr-3"),
            "the atomic class is present: {out}"
        );
        assert!(
            out.contains("sail-grid"),
            "the base grid class still applied: {out}"
        );
        assert!(
            !out.contains("rows=\"3\""),
            "rows doesn't leak as a raw attr: {out}"
        );
    }

    #[test]
    fn grid_cols_and_rows_compose_into_a_fixed_matrix() {
        // `cols` owns the column track, `rows` the row track — different CSS
        // properties, so both survive (no precedence, unlike cols/min).
        let out = preprocess(r#"<grid cols="4" rows="3">x</grid>"#).expect("preprocess");
        assert!(out.contains("sail-gc-4"), "cols class: {out}");
        assert!(out.contains("sail-gr-3"), "rows class: {out}");
        assert!(
            out.contains("grid-template-columns:repeat(4,minmax(0,1fr))"),
            "{out}"
        );
        assert!(
            out.contains("grid-template-rows:repeat(3,minmax(0,1fr))"),
            "{out}"
        );
    }

    #[test]
    fn grid_rows_dynamic_lowers_to_a_class_binding_with_all_rules() {
        let out = preprocess(r#"<grid rows={n()}>x</grid>"#).expect("preprocess");
        assert!(
            out.contains("'sail-gr-' + (n())"),
            "dynamic rows fragment: {out}"
        );
        for n in 1..=12 {
            assert!(
                out.contains(&format!(
                    ".sail-gr-{n}{{grid-template-rows:repeat({n},minmax(0,1fr))}}"
                )),
                "rule for rows={n} present: {out}"
            );
        }
    }

    #[test]
    fn column_justify_and_evenly_emit() {
        let out = preprocess(r#"<column justify="between">x</column>"#).expect("preprocess");
        assert!(
            out.contains("sail-jc-between"),
            "column justify class: {out}"
        );
        assert!(
            out.contains(".sail-jc-between{justify-content:space-between}"),
            "{out}"
        );
        // A plain column emits no justify class (no default).
        let plain = preprocess(r#"<column>x</column>"#).expect("preprocess");
        assert!(
            !plain.contains("sail-jc-"),
            "no implicit column justify: {plain}"
        );

        let evenly = preprocess(r#"<row justify="evenly">x</row>"#).expect("preprocess");
        assert!(
            evenly.contains(".sail-jc-evenly{justify-content:space-evenly}"),
            "{evenly}"
        );
    }

    #[test]
    fn unknown_layout_attribute_is_dropped_not_passed_through() {
        // Defensive: even if the lint weren't running, an unknown attr on a
        // layout keyword must not silently flow through to the lowered div.
        // The compile-emitted output must not carry a dead `bogus="x"`.
        let out = preprocess(r#"<grid bogus="x">x</grid>"#).expect("preprocess");
        assert!(
            !out.contains("bogus="),
            "unknown attr dropped from emit: {out}"
        );
        assert!(
            out.contains("sail-grid"),
            "the base layout class still there: {out}"
        );
    }

    #[test]
    fn recognized_passthroughs_ride_to_the_div() {
        // The compiler's `lower_layout` consults the lint's
        // `is_passthrough_attr` (single source of truth). Assert each known
        // pass-through survives on the lowered div.
        let out = preprocess(
            r#"<grid id="hero" title="t" data-x="1" aria-label="x" role="region">x</grid>"#,
        )
        .expect("preprocess");
        assert!(out.contains(r#"id="hero""#), "id survived: {out}");
        assert!(out.contains(r#"title="t""#), "title survived: {out}");
        assert!(out.contains(r#"data-x="1""#), "data-* survived: {out}");
        assert!(out.contains(r#"aria-label="x""#), "aria-* survived: {out}");
        assert!(out.contains(r#"role="region""#), "role survived: {out}");
    }

    #[test]
    fn surface_and_domdec_markers_ride_to_the_div() {
        // DOMDEC-1 bare markers on layout keywords must survive lowering so
        // chrome / test hooks stay addressable on the emitted div.
        let out = preprocess(r#"<row surface="edit" edit-surface tab-id="a" grow>x</row>"#)
            .expect("preprocess");
        assert!(out.contains(r#"surface="edit""#), "surface survived: {out}");
        assert!(
            out.contains("edit-surface"),
            "hyphenated boolean marker survived: {out}"
        );
        assert!(
            out.contains(r#"tab-id="a""#),
            "hyphenated value marker survived: {out}"
        );
        assert!(
            !out.contains("colz="),
            "unknown axis typos must not invent attrs: {out}"
        );
    }
    #[test]
    fn directive_prefixes_ride_to_the_div() {
        // SAIL directive attrs (`bind:value`, `use:*`, plain fields,
        // etc.) are framework bindings — they ride through the layout
        // lowering unchanged so the compiler's later directive-handling
        // pass can resolve them.
        let out = preprocess(r#"<row bind:value={name} hidden={!cr.cell.inMonth}>x</row>"#)
            .expect("preprocess");
        assert!(out.contains("bind:value="), "bind:value survives: {out}");
        assert!(out.contains("hidden="), "plain hidden survives: {out}");
    }

    #[test]
    fn column_does_not_emit_sail_dir_override() {
        let out = preprocess(r#"<column gap="2">x</column>"#).expect("preprocess");
        assert!(!out.contains("sail-dir-"), "no sail-dir modifier: {out}");
    }

    #[test]
    fn dynamic_axis_lowers_to_a_class_binding_with_full_rules() {
        // a component parameterizing its layout: `align={pick()}`
        let out = preprocess(r#"<column align={pick()}>x</column>"#).expect("preprocess");
        assert!(
            out.contains(r#"class="{"#),
            "emits a dynamic class binding: {out}"
        );
        assert!(
            out.contains("'sail-ai-' + (pick())"),
            "dynamic align fragment: {out}"
        );
        // EVERY align rule injected, so whatever `pick()` returns has a match.
        assert!(
            out.contains(".sail-ai-start{")
                && out.contains(".sail-ai-center{")
                && out.contains(".sail-ai-stretch{"),
            "full align rule set injected: {out}"
        );
    }

    #[test]
    fn condition_prefix_wraps_in_media() {
        // Utility-string condition prefixes are residual product (hard error /
        // empty lower). Style product uses object-key when: for browser state;
        // media-condition guards remain a follow-up on the Style path.
        let out = preprocess(r#"<div style({ when: { hover: elevation(none) } })> x</div>"#)
            .expect("preprocess");
        assert!(out.contains(":hover"), "when hover emits pseudo:\n{out}");
        assert!(
            out.contains("box-shadow") || out.contains("shadow") || out.contains("s-"),
            "{out}"
        );
    }

    #[test]
    fn unterminated_comment_copies_to_eof() {
        // Defensive: a malformed file with no closing `-->` shouldn't
        // panic or read past end-of-file. Copy the rest verbatim and
        // let the downstream parser produce its own diagnostic.
        let src = "<!-- never closed";
        let out = preprocess(src).expect("preprocess");
        assert_eq!(out, src);
    }

    #[test]
    fn inline_controller_full_form_is_self_contained() {
        // inv-sail-one-module-emit (sail-project-model): an inline <controller>
        // that uses ambient runtime idents without importing them must compile
        // to a SELF-CONTAINED module — the compiler injects the imports for the
        // ambient idents it references. A verbatim passthrough leaves them as
        // free variables that become `ReferenceError: controller is not
        // defined` at runtime (the blank-render regression).
        let src = "<controller>\n\
export default controller({ setup: () => { const c = signal(0); return { d: derived(() => c()) } } })\n\
</controller>\n\
<column></column>";
        let (_, body) = extract_controller_block(src);
        let body = body.expect("controller block present");
        for ident in ["controller", "signal", "derived"] {
            let imported = body.lines().any(|l| {
                let t = l.trim_start();
                t.starts_with("import") && t.contains(ident)
            });
            assert!(
                imported,
                "ambient runtime ident `{ident}` is used but not imported \
                 (free variable -> ReferenceError). Module:\n{body}"
            );
        }
    }

    #[test]
    fn inline_controller_does_not_double_import_ambient() {
        // A controller that ALREADY imports the ambient idents (here multi-line,
        // the realistic authored shape) must not get a second injected import —
        // a duplicate binding is a compile error. Injection fills gaps only.
        let src = "<controller>\n\
import {\n  controller,\n  derived,\n  signal,\n} from '@ship/sail-runtime'\n\
export default controller({ setup: () => ({ d: derived(() => signal(0)()) }) })\n\
</controller>\n\
<column></column>";
        let (_, body) = extract_controller_block(src);
        let body = body.expect("controller block present");
        assert!(
            body.contains("from '@ship/sail-runtime'"),
            "the author's own import is preserved:\n{body}"
        );
        assert!(
            !body.contains("@ship/sail-runtime/internal"),
            "ambient idents the author already imports must NOT be re-injected from \
             /internal (a duplicate binding is a compile error). Module:\n{body}"
        );
    }

    #[test]
    fn lower_sx_var_ref_binds_an_atomic_class() {
        // Product form: `style({ translateY(bind(rowY)) })` — reactive paint via
        // CSS custom property. JS writes `--row-y`; Style declares the read.
        let out = preprocess(r#"<grid style({ translateY(bind(rowY)) })>row</grid>"#)
            .expect("preprocess");
        assert!(
            out.contains("var(--row-y)") || out.contains("translate"),
            "bind translateY emits var/read:\n{out}"
        );
        assert!(out.contains("s-"), "emits Style class:\n{out}");
    }

    #[test]
    fn lower_sx_var_ref_rejects_garbage_identifier() {
        // Utility-string sx no longer a special preprocess refuse; plain attr survives.
        let out = preprocess(r#"<div sx="top-(--row.y)">x</div>"#).expect("no sx tombstone");
        assert!(out.contains("sx="), "{out}");
    }

    #[test]
    fn tag_style_statement_lowers_to_class_legacy() {
        let out =
            preprocess(r#"<div style({ pad({ x: 2 }).bg(primary) })>x</div>"#).expect("preprocess");
        assert!(
            out.contains("s-") || out.contains("var(--primary)"),
            "{out}"
        );
        assert!(!out.contains("style({ bg(primary) })"), "{out}");
    }
    #[test]
    fn block_style_const_ref() {
        let out = preprocess(
            r#"
<scope>
  const base = style({ pad(2) })
</scope>
<div style(base)>x</div>"#,
        )
        .expect("preprocess");
        assert!(out.contains("s-"), "{out}");
    }
    #[test]
    fn flat_style_method_chain_is_refused_on_product_path() {
        let err = preprocess(r#"<div style.pad(4)>x</div>"#).expect_err("flat chain");
        assert!(
            err.message.contains("style(") || err.message.contains("call"),
            "got: {}",
            err.message
        );
    }

    #[test]
    fn sx_attr_passes_preprocess_as_plain_attr() {
        // sx= is no longer a special preprocess tombstone; unknown-attr is attrs/compile.
        let out = preprocess(r#"<div sx="x">x</div>"#).expect("sx not special in preprocess");
        assert!(out.contains("sx="), "{out}");
    }

    #[test]
    fn style_attr_product_is_refused() {
        let err = preprocess(r#"<div style={style({ bg(primary) })}>x</div>"#).expect_err("style=");
        assert!(
            err.message.contains("style(") || err.message.contains("call"),
            "must teach call form: {}",
            err.message
        );
    }

    #[test]
    fn raw_html_style_string_attr_is_loud_error() {
        let err = preprocess(r#"<div style="color: red">x</div>"#).expect_err("style=");
        assert!(
            err.message.contains("style(") || err.message.contains("call"),
            "got: {}",
            err.message
        );
    }

    #[test]
    fn textish_refused_on_non_text_host() {
        let err = preprocess(r#"<div style({ type({ size: sm }) })>x</div>"#)
            .expect_err("textish setters on div");
        let msg = &err.message;
        assert!(
            msg.contains("text-bearing") || msg.contains("typography"),
            "must name text-host law: {msg}"
        );
        assert!(
            msg.contains("<div>") || msg.contains("div"),
            "must name the host tag: {msg}"
        );
    }

    #[test]
    fn textish_allowed_on_text_bearing_hosts() {
        let out =
            preprocess(r#"<p style({ type({ size: sm }) })>x</p>"#).expect("p is text-bearing");
        assert!(out.contains("s-") || out.contains("font-size"), "{out}");

        let out = preprocess(r#"<button style({ type({ size: sm }).bg(primary) })>go</button>"#)
            .expect("button is text-bearing via registry");
        assert!(out.contains("s-"), "{out}");
    }

    #[test]
    fn non_text_host_box_paint_only_is_ok() {
        let out = preprocess(r#"<div style({ pad(4).bg(primary) })>x</div>"#)
            .expect("box+paint on div is legal");
        assert!(out.contains("s-"), "{out}");
        assert!(!out.contains("style({"), "block Style must lift: {out}");
    }

    #[test]
    fn tag_style_object_lowers_to_class() {
        let out =
            preprocess(r#"<div style({ pad({ x: 2 }).bg(primary) })>x</div>"#).expect("preprocess");
        assert!(
            out.contains("s-") || out.contains("var(--primary)"),
            "{out}"
        );
        assert!(
            !out.contains("style({"),
            "tag Style must lift off open tag: {out}"
        );
    }

    #[test]
    fn tag_style_void_host_no_lift_rule() {
        let out = preprocess(r#"<input style({ size({ w: full }) })>"#).expect("void host");
        assert!(out.contains("s-"), "{out}");
        assert!(!out.contains("style({"), "{out}");
    }

    #[test]
    fn body_style_statement_is_unknown_statement() {
        let err = preprocess(
            "<div>
  style({ pad(2) })
  x
</div>",
        )
        .expect_err("body style is unknown statement");
        let msg = err.message.to_lowercase();
        assert!(
            msg.contains("unknown statement") || msg.contains("open tag"),
            "got: {msg}"
        );
    }

    #[test]
    fn bare_style_in_tag_is_loud_error() {
        let err = preprocess(r#"<div style>x</div>"#).expect_err("bare style");
        assert!(
            err.message.contains("style(") || err.message.contains("call"),
            "must teach style(…): {}",
            err.message
        );
    }

    #[test]
    fn style_eq_in_tag_is_loud_error() {
        let err = preprocess(r#"<div style={foo}>x</div>"#).expect_err("style=");
        assert!(
            err.message.contains("style(") || err.message.contains("call"),
            "must teach call form: {}",
            err.message
        );
    }

    #[test]
    fn silent_drop_canary_tag_style_attaches() {
        let out = preprocess(
            r#"<button style({ pad(3).bg(primary).fg(primary-foreground) })>ok</button>"#,
        )
        .expect("tag style must compile");
        assert!(
            out.contains("s-") || out.contains("var(--primary)"),
            "must attach: {out}"
        );
        assert!(!out.contains("style({"), "must lower off the tag: {out}");
    }

    #[test]
    fn type_size_from_style_ir_threads_to_layout() {
        // expand_elements=false: this branch's stdlib spinner is already
        // uncompilable (`style(ring)`); the seam under test is Style → TypeHost.
        let src_xl = r#"<span style({ type({ size: xl }) })>Hello</span>"#;
        let src_sm = r#"<span style({ type({ size: sm }) })>Hello</span>"#;
        let (pre_xl, hosts_xl) = preprocess_impl_full(src_xl, false).expect("xl");
        let (pre_sm, hosts_sm) = preprocess_impl_full(src_sm, false).expect("sm");
        assert!(
            hosts_xl
                .types
                .iter()
                .any(|h| h.size.as_deref() == Some("xl")),
            "type({{ size: xl }}) must resolve xl: {:?}",
            hosts_xl.types
        );
        assert!(
            hosts_sm
                .types
                .iter()
                .any(|h| h.size.as_deref() == Some("sm")),
            "type({{ size: sm }}) must resolve sm: {:?}",
            hosts_sm.types
        );
        let xl = sail_layout::check_preprocessed_with_hosts(
            &pre_xl,
            (1440.0, 900.0),
            sail_layout::CheckOptions::default(),
            &hosts_xl.types,
            &hosts_xl.boxes,
        )
        .expect("xl layout");
        let sm = sail_layout::check_preprocessed_with_hosts(
            &pre_sm,
            (1440.0, 900.0),
            sail_layout::CheckOptions::default(),
            &hosts_sm.types,
            &hosts_sm.boxes,
        )
        .expect("sm layout");
        let xl_h = xl
            .boxes
            .iter()
            .find(|b| b.tag == "span")
            .expect("xl span")
            .height;
        let sm_h = sm
            .boxes
            .iter()
            .find(|b| b.tag == "span")
            .expect("sm span")
            .height;
        assert!(xl_h > sm_h, "xl height ({xl_h}) must exceed sm ({sm_h})");
    }

    #[test]
    fn box_from_style_ir_threads_to_layout() {
        let src = r#"<div style({ display(flex).grow(1).overflow({ all: hidden }).position({ scheme: absolute }) })>x</div>"#;
        let (pre, hosts) = preprocess_impl_full(src, false).expect("box style");
        let host = hosts
            .boxes
            .iter()
            .find(|h| {
                h.resolved.display.as_deref() == Some("flex")
                    && h.resolved.grow == Some(1.0)
                    && h.resolved.overflow_x.as_deref() == Some("hidden")
                    && h.resolved.position.as_deref() == Some("absolute")
            })
            .unwrap_or_else(|| panic!("box host missing: {:?}", hosts.boxes));
        let mut elements = sail_layout::html::parse_source(&pre);
        sail_layout::apply_resolved_boxes(&mut elements, &hosts.boxes);
        let elem = elements.iter().find(|e| e.tag == "div").expect("div");
        let resolved = sail_layout::style::resolve(elem, &elements, 1440.0);
        use sail_layout::taffy::prelude::*;
        use sail_layout::taffy::style::Overflow;
        assert_eq!(resolved.style.display, Display::Flex);
        assert!((resolved.style.flex_grow - 1.0).abs() < f32::EPSILON);
        assert_eq!(resolved.style.overflow.x, Overflow::Hidden);
        assert_eq!(resolved.style.position, Position::Absolute);
        assert!(
            !host
                .classes
                .iter()
                .any(|c| c.contains("displayflex")
                    && elem.box_style.display.as_deref() != Some("flex")),
            "measure must come from Style IR, not class spelling"
        );
        let _ = pre;
    }
}
