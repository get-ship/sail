//! Flat Style chain → channel spine rewrite (corpus codemod).

use crate::channel::{self, Channel};

/// Rewrite every flat `style.…` chain in `src` to `style({ box, paint, text, animate })`.
/// Nested chains (when/match/compound bodies) rewrite first; chain-tail `.when({…})`
/// becomes cond-form `.when(key, { channel: … })` (multi-entry → multiple `.when`).
pub fn rewrite_source(src: &str) -> String {
    let mut out = String::with_capacity(src.len() + src.len() / 8);
    let mut i = 0usize;
    while i < src.len() {
        if let Some(end) = find_style_chain_at(src, i) {
            let expr = &src[i..end];
            out.push_str(&rewrite_style_expr(expr));
            i = end;
            continue;
        }
        let ch = src[i..].chars().next().expect("i in bounds");
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

/// True when `src[i..]` starts a flat `style.` chain (not `style(` / already channel).
fn find_style_chain_at(src: &str, i: usize) -> Option<usize> {
    let bytes = src.as_bytes();
    if !src[i..].starts_with("style.") {
        return None;
    }
    // Avoid matching identifiers like `mystyle.foo` — require non-ident before.
    if i > 0 {
        let prev = bytes[i - 1];
        if prev.is_ascii_alphanumeric() || prev == b'_' || prev == b'$' {
            return None;
        }
    }
    let end = scan_style_chain_end(src, i);
    // Require at least `style.` + one method char so we never rewrite bare `style`.
    if end <= i + "style.".len() {
        return None;
    }
    Some(end)
}

fn scan_style_chain_end(src: &str, start: usize) -> usize {
    let mut pos = start + "style".len();
    let bytes = src.as_bytes();
    loop {
        // Do not consume trailing whitespace after the last method — that would
        // swallow the newline between `const a = style.…` and the next statement.
        let before_dot = pos;
        skip_ws_comments(src, &mut pos);
        if pos >= bytes.len() || bytes[pos] != b'.' {
            pos = before_dot;
            break;
        }
        pos += 1;
        skip_ws_comments(src, &mut pos);
        if !scan_ident(src, &mut pos) {
            pos = before_dot;
            break;
        }
        let after_name = pos;
        skip_ws_comments(src, &mut pos);
        if pos < bytes.len() && bytes[pos] == b'(' {
            pos = scan_balanced(src, pos);
        } else {
            // zero-arg bare method without `()` is not product form; keep name only
            // if product requires parens — registry always uses `()`. Leave at name end
            // when no paren (should not appear in corpus).
            pos = after_name;
        }
    }
    pos
}

/// Rewrite one flat `style.…` expression to a channel spine.
pub fn rewrite_style_expr(expr: &str) -> String {
    if !expr.starts_with("style.") {
        return expr.to_string();
    }
    let mut pos = "style".len();
    let mut box_calls: Vec<String> = Vec::new();
    let mut paint_calls: Vec<String> = Vec::new();
    let mut text_calls: Vec<String> = Vec::new();
    let mut animate_calls: Vec<String> = Vec::new();
    let mut when_tails: Vec<String> = Vec::new();
    let mut unknown: Vec<String> = Vec::new();

    let bytes = expr.as_bytes();
    while pos < bytes.len() {
        skip_ws_comments(expr, &mut pos);
        if pos >= bytes.len() || bytes[pos] != b'.' {
            break;
        }
        pos += 1;
        skip_ws_comments(expr, &mut pos);
        let name_start = pos;
        if !scan_ident(expr, &mut pos) {
            break;
        }
        let name = &expr[name_start..pos];
        skip_ws_comments(expr, &mut pos);
        let args = if pos < bytes.len() && bytes[pos] == b'(' {
            let a0 = pos;
            let a1 = scan_balanced(expr, pos);
            pos = a1;
            &expr[a0..a1]
        } else {
            ""
        };

        if name == "when" {
            when_tails.extend(split_when_to_cond_args(args));
            continue;
        }

        // Motion brace form → animate; dual paint form of scroll/press stays paint.
        // `args` already includes outer parens: `({ … })`.
        if channel::is_animate_trigger(name) && is_brace_object_args(args) {
            animate_calls.push(format!("{name}{args}"));
            continue;
        }

        let call = format!("{name}{args}");
        match channel::channel_for_setter(name) {
            Some(Channel::Box) => box_calls.push(call),
            Some(Channel::Paint) => paint_calls.push(call),
            Some(Channel::Text) => text_calls.push(call),
            Some(Channel::Animate) => animate_calls.push(call),
            None => unknown.push(call),
        }
    }

    // Unknown setters keep the expression unchanged (fail closed on rewrite).
    if !unknown.is_empty() {
        return expr.to_string();
    }

    if box_calls.is_empty()
        && paint_calls.is_empty()
        && text_calls.is_empty()
        && animate_calls.is_empty()
        && when_tails.is_empty()
    {
        return expr.to_string();
    }

    let mut parts = Vec::new();
    if !box_calls.is_empty() {
        parts.push(format!("box: {}", box_calls.join(".")));
    }
    if !paint_calls.is_empty() {
        parts.push(format!("paint: {}", paint_calls.join(".")));
    }
    if !text_calls.is_empty() {
        parts.push(format!("text: {}", text_calls.join(".")));
    }
    if !animate_calls.is_empty() {
        parts.push(format!("animate: {}", animate_calls.join(".")));
    }

    let mut out = if parts.is_empty() {
        "style({})".to_string()
    } else {
        format!("style({{ {} }})", parts.join(", "))
    };
    for t in when_tails {
        out.push_str(".when");
        out.push_str(&t);
    }
    out
}

/// Returns one or more parenthesized when-args strings.
fn split_when_to_cond_args(args: &str) -> Vec<String> {
    let inner = strip_parens(args).trim();
    if inner.is_empty() {
        return vec![args.to_string()];
    }
    if !inner.starts_with('{') {
        let rewritten = rewrite_source(inner);
        return vec![format!("({rewritten})")];
    }
    let entries = match parse_when_map_entries(inner) {
        Some(e) if !e.is_empty() => e,
        _ => {
            let rewritten = rewrite_source(inner);
            return vec![format!("({rewritten})")];
        }
    };
    entries
        .into_iter()
        .map(|(key, body)| {
            let partial = body_to_when_partial(&body);
            format!("({key}, {partial})")
        })
        .collect()
}

fn body_to_when_partial(body: &str) -> String {
    let body = body.trim();
    let rewritten = if body.starts_with("style.") {
        rewrite_style_expr(body)
    } else {
        rewrite_source(body)
    };
    if let Some(obj) = strip_style_channel_call(&rewritten) {
        return obj.to_string();
    }
    rewritten
}

/// If `s` is exactly `style({…})`, return the `{…}` object.
fn strip_style_channel_call(s: &str) -> Option<&str> {
    let s = s.trim();
    if !s.starts_with("style(") {
        return None;
    }
    let end = scan_balanced(s, "style".len());
    if end != s.len() {
        return None;
    }
    let inner = s["style(".len()..end - 1].trim();
    if inner.starts_with('{') {
        Some(inner)
    } else {
        None
    }
}

fn parse_when_map_entries(map: &str) -> Option<Vec<(String, String)>> {
    let map = map.trim();
    if !map.starts_with('{') || !map.ends_with('}') {
        return None;
    }
    let mut pos = 0usize;
    let bytes = map.as_bytes();
    pos += 1;
    let mut entries = Vec::new();
    loop {
        skip_ws_comments(map, &mut pos);
        if pos >= bytes.len() {
            return None;
        }
        if bytes[pos] == b'}' {
            break;
        }
        let key_start = pos;
        if !scan_ident(map, &mut pos) {
            return None;
        }
        let key = map[key_start..pos].to_string();
        skip_ws_comments(map, &mut pos);
        if pos >= bytes.len() || bytes[pos] != b':' {
            return None;
        }
        pos += 1;
        skip_ws_comments(map, &mut pos);
        let body_start = pos;
        let body_end = scan_map_value_end(map, pos);
        let body = map[body_start..body_end].trim().to_string();
        pos = body_end;
        entries.push((key, body));
        skip_ws_comments(map, &mut pos);
        if pos < bytes.len() && bytes[pos] == b',' {
            pos += 1;
            continue;
        }
    }
    Some(entries)
}

fn scan_map_value_end(src: &str, start: usize) -> usize {
    let bytes = src.as_bytes();
    let mut pos = start;
    let mut depth_paren = 0i32;
    let mut depth_brace = 0i32;
    let mut depth_brack = 0i32;
    while pos < bytes.len() {
        let c = bytes[pos];
        if c == b'\'' || c == b'"' {
            pos = scan_string(src, pos);
            continue;
        }
        match c {
            b'(' => depth_paren += 1,
            b')' => depth_paren -= 1,
            b'{' => depth_brace += 1,
            b'}' => {
                if depth_brace == 0 && depth_paren == 0 && depth_brack == 0 {
                    return pos;
                }
                depth_brace -= 1;
            }
            b'[' => depth_brack += 1,
            b']' => depth_brack -= 1,
            b',' if depth_paren == 0 && depth_brace == 0 && depth_brack == 0 => return pos,
            _ => {}
        }
        pos += 1;
    }
    pos
}

fn is_brace_object_args(args: &str) -> bool {
    strip_parens(args).trim().starts_with('{')
}

fn strip_parens(args: &str) -> &str {
    let t = args.trim();
    if t.starts_with('(') && t.ends_with(')') {
        &t[1..t.len() - 1]
    } else {
        t
    }
}

fn scan_ident(src: &str, pos: &mut usize) -> bool {
    let bytes = src.as_bytes();
    if *pos >= bytes.len() {
        return false;
    }
    let c = bytes[*pos];
    if !(c.is_ascii_alphabetic() || c == b'_') {
        return false;
    }
    *pos += 1;
    while *pos < bytes.len() {
        let c = bytes[*pos];
        // Method names are camelCase; motion trigger `focus-visible` is the
        // sole hyphenated product method.
        if c.is_ascii_alphanumeric() || c == b'_' || c == b'-' {
            *pos += 1;
        } else {
            break;
        }
    }
    true
}

fn skip_ws_comments(src: &str, pos: &mut usize) {
    let bytes = src.as_bytes();
    loop {
        while *pos < bytes.len() && bytes[*pos].is_ascii_whitespace() {
            *pos += 1;
        }
        if *pos + 1 < bytes.len() && bytes[*pos] == b'/' && bytes[*pos + 1] == b'/' {
            *pos += 2;
            while *pos < bytes.len() && bytes[*pos] != b'\n' {
                *pos += 1;
            }
            continue;
        }
        if *pos + 1 < bytes.len() && bytes[*pos] == b'/' && bytes[*pos + 1] == b'*' {
            *pos += 2;
            while *pos + 1 < bytes.len() && !(bytes[*pos] == b'*' && bytes[*pos + 1] == b'/') {
                *pos += 1;
            }
            if *pos + 1 < bytes.len() {
                *pos += 2;
            }
            continue;
        }
        break;
    }
}

/// `pos` at opening `(`, `{`, or `[` — return index past matching closer.
fn scan_balanced(src: &str, open_pos: usize) -> usize {
    let bytes = src.as_bytes();
    let open = bytes[open_pos];
    let close = match open {
        b'(' => b')',
        b'{' => b'}',
        b'[' => b']',
        _ => return open_pos + 1,
    };
    let mut depth = 0i32;
    let mut pos = open_pos;
    while pos < bytes.len() {
        let c = bytes[pos];
        if c == b'\'' || c == b'"' {
            pos = scan_string(src, pos);
            continue;
        }
        if c == open {
            depth += 1;
            pos += 1;
            continue;
        }
        if c == close {
            depth -= 1;
            pos += 1;
            if depth == 0 {
                return pos;
            }
            continue;
        }
        // Nested groups of other bracket kinds
        if matches!(c, b'(' | b'{' | b'[') && c != open {
            pos = scan_balanced(src, pos);
            continue;
        }
        pos += 1;
    }
    bytes.len()
}

fn scan_string(src: &str, start: usize) -> usize {
    let bytes = src.as_bytes();
    let quote = bytes[start];
    let mut pos = start + 1;
    while pos < bytes.len() {
        let c = bytes[pos];
        if c == b'\\' && pos + 1 < bytes.len() {
            pos += 2;
            continue;
        }
        if c == quote {
            return pos + 1;
        }
        pos += 1;
    }
    pos
}

// ── Teaching codemod: lift `style=` / `sx=` attribute hosts → block statements ──

/// Flat → channel, then lift product Style attrs into block attach form.
pub fn rewrite_teaching_source(src: &str) -> String {
    lift_style_attr_hosts(&rewrite_source(src))
}

/// Product Style on `style=` / `sx=` → remove the attribute and inject a block
/// statement after the open tag’s `>`.
///
/// ```text
/// <div style={style({ box: pad(4) })}>x</div>
/// // →
/// <div>
///   style({ box: pad(4) })
///   x
/// </div>
/// ```
pub fn lift_style_attr_hosts(src: &str) -> String {
    let bytes = src.as_bytes();
    let mut edits: Vec<AttrLiftEdit> = Vec::new();
    let mut i = 0usize;

    while i < bytes.len() {
        // Skip HTML comments.
        if src[i..].starts_with("<!--") {
            if let Some(rel) = src[i..].find("-->") {
                i += rel + 3;
                continue;
            }
            break;
        }
        // Skip raw <style>…</style> blocks (CSS, not Style product).
        if src[i..].starts_with("<style") {
            let after = i + "<style".len();
            if after < bytes.len() {
                let c = bytes[after];
                if c.is_ascii_whitespace() || c == b'>' || c == b'/' {
                    if let Some(rel) = src[i..].find("</style>") {
                        i += rel + "</style>".len();
                        continue;
                    }
                }
            }
        }
        // Skip <script> / <controller> / <scope> raw bodies — attrs still scanned.
        if bytes[i] == b'<' && i + 1 < bytes.len() && bytes[i + 1] != b'/' && bytes[i + 1] != b'!'
        {
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
            if tag_end == tag_start {
                i += 1;
                continue;
            }
            let tag = &src[tag_start..tag_end];
            let mut k = tag_end;
            let mut product_attr: Option<(usize, usize, String)> = None;

            // Scan attributes until `>` / `/>`.
            loop {
                while k < bytes.len() && bytes[k].is_ascii_whitespace() {
                    k += 1;
                }
                if k >= bytes.len() {
                    break;
                }
                if bytes[k] == b'>' {
                    break;
                }
                if bytes[k] == b'/' && k + 1 < bytes.len() && bytes[k + 1] == b'>' {
                    break;
                }
                // Attribute name.
                let name_lo = k;
                while k < bytes.len()
                    && (bytes[k].is_ascii_alphanumeric()
                        || bytes[k] == b'-'
                        || bytes[k] == b'_'
                        || bytes[k] == b':'
                        || bytes[k] == b'.')
                {
                    k += 1;
                }
                if k == name_lo {
                    // Unexpected token — advance one byte to avoid hang.
                    k += 1;
                    continue;
                }
                let name = &src[name_lo..k];
                while k < bytes.len() && bytes[k].is_ascii_whitespace() {
                    k += 1;
                }
                if k >= bytes.len() || bytes[k] != b'=' {
                    // Boolean attr.
                    continue;
                }
                k += 1;
                while k < bytes.len() && bytes[k].is_ascii_whitespace() {
                    k += 1;
                }
                if k >= bytes.len() {
                    break;
                }
                let (val_hi, raw_val) = match bytes[k] {
                    b'{' => {
                        let end = scan_balanced(src, k);
                        (end, src[k..end].to_string())
                    }
                    b'"' | b'\'' => {
                        let end = scan_string(src, k);
                        (end, src[k..end].to_string())
                    }
                    _ => {
                        // Unquoted value — rare; take until ws or `>`/`/`.
                        let start = k;
                        while k < bytes.len()
                            && !bytes[k].is_ascii_whitespace()
                            && bytes[k] != b'>'
                            && bytes[k] != b'/'
                        {
                            k += 1;
                        }
                        (k, src[start..k].to_string())
                    }
                };
                k = val_hi;

                if name == "style" || name == "sx" {
                    let product_src = strip_attr_value_wrapper(&raw_val);
                    if looks_like_style_product_value(product_src) {
                        // Leading whitespace before attr name belongs to the attr span.
                        let mut attr_lo = name_lo;
                        while attr_lo > tag_end && bytes[attr_lo - 1].is_ascii_whitespace() {
                            // Keep one newline context: include spaces/tabs on the same line.
                            if bytes[attr_lo - 1] == b'\n' {
                                break;
                            }
                            attr_lo -= 1;
                        }
                        // Prefer eating the preceding space when attr is mid-line.
                        if attr_lo > tag_end && bytes[attr_lo - 1] == b' ' {
                            attr_lo -= 1;
                        }
                        let stmt = normalize_style_block_stmt(product_src);
                        product_attr = Some((attr_lo, val_hi, stmt));
                    }
                }
            }

            if k >= bytes.len() {
                break;
            }

            let self_closing = bytes[k] == b'/';
            let gt_pos = if self_closing { k + 1 } else { k }; // index of `>`
            // Ensure `>` is where we think.
            let gt_pos = if gt_pos < bytes.len() && bytes[gt_pos] == b'>' {
                gt_pos
            } else if k < bytes.len() && bytes[k] == b'>' {
                k
            } else {
                i = tag_end;
                continue;
            };

            if let Some((attr_lo, attr_hi, stmt)) = product_attr {
                let indent = open_tag_indent(src, i);
                edits.push(AttrLiftEdit {
                    attr_lo,
                    attr_hi,
                    gt_pos,
                    self_closing,
                    tag: tag.to_string(),
                    stmt,
                    indent,
                });
            }

            // Advance past open tag; raw-text tags skip their body.
            let body_start = gt_pos + 1;
            if is_raw_skip_tag(tag) && !self_closing {
                let close = format!("</{tag}>");
                if let Some(rel) = src[body_start..].find(&close) {
                    i = body_start + rel + close.len();
                    continue;
                }
            }
            i = body_start;
            continue;
        }
        // Advance one Unicode scalar so byte indexes stay on char boundaries.
        let mut n = i + 1;
        while n < bytes.len() && (bytes[n] & 0xC0) == 0x80 {
            n += 1;
        }
        i = n;
    }

    if edits.is_empty() {
        return src.to_string();
    }

    // Apply reverse so earlier offsets stay valid.
    let mut out = src.to_string();
    for e in edits.into_iter().rev() {
        apply_attr_lift(&mut out, &e);
    }
    out
}

struct AttrLiftEdit {
    attr_lo: usize,
    attr_hi: usize,
    gt_pos: usize,
    self_closing: bool,
    tag: String,
    stmt: String,
    indent: String,
}

fn apply_attr_lift(out: &mut String, e: &AttrLiftEdit) {
    // gt_pos is still valid relative to pre-attr-removal source only if attr is
    // before `>`. After removing the attr, gt shifts left when attr_lo < gt.
    let gt = e.gt_pos;
    let attr_lo = e.attr_lo;
    let attr_hi = e.attr_hi;
    debug_assert!(attr_hi <= gt, "attr must end before `>`");

    let pad = format!("{}  ", e.indent); // 2-space child indent
    let injection = if e.self_closing {
        // Expand `/>` → `>\n  STMT\n</tag>` (drop the `/` by rewriting from gt-1).
        format!(">\n{pad}{}\n{}</{}>", e.stmt, e.indent, e.tag)
    } else {
        format!(">\n{pad}{}", e.stmt)
    };

    // 1) Remove attribute.
    out.replace_range(attr_lo..attr_hi, "");
    let shift = attr_hi - attr_lo;
    let gt2 = gt - shift;

    // 2) Inject after `>` (or expand self-closing).
    if e.self_closing {
        // `/>` ends at gt2; `/` at gt2-1.
        let slash = if gt2 > 0 && out.as_bytes()[gt2 - 1] == b'/' {
            gt2 - 1
        } else {
            gt2
        };
        // Replace from slash through `>` with injection (which starts with `>`).
        out.replace_range(slash..gt2 + 1, &injection);
    } else {
        // Insert statement right after `>`.
        let insert_at = gt2 + 1;
        let after = &out[insert_at..];
        let only_stmt = if after.starts_with('\n') || after.starts_with("\r\n") {
            format!("\n{pad}{}", e.stmt)
        } else if after.is_empty() || after.starts_with("</") {
            // Empty body or immediate close — still put stmt on its own line.
            format!("\n{pad}{}\n{}", e.stmt, e.indent)
        } else {
            // Inline children: break them onto the next indented line.
            format!("\n{pad}{}\n{pad}", e.stmt)
        };
        out.insert_str(insert_at, &only_stmt);
    }
}

fn open_tag_indent(src: &str, tag_lt: usize) -> String {
    let bytes = src.as_bytes();
    let mut line_start = tag_lt;
    while line_start > 0 && bytes[line_start - 1] != b'\n' {
        line_start -= 1;
    }
    let mut n = 0usize;
    let mut i = line_start;
    while i < tag_lt {
        match bytes[i] {
            b' ' => n += 1,
            b'\t' => n += 2,
            _ => break,
        }
        i += 1;
    }
    " ".repeat(n)
}

fn is_raw_skip_tag(tag: &str) -> bool {
    matches!(
        tag,
        "script" | "style" | "controller" | "scope" | "pre" | "code"
    )
}

/// Strip one layer of attr quoting: `{…}` or `"…"` / `'…'`.
fn strip_attr_value_wrapper(raw: &str) -> &str {
    let t = raw.trim();
    if t.starts_with('{') && t.ends_with('}') {
        // Only strip outer braces when balanced as a single group.
        let end = scan_balanced(t, 0);
        if end == t.len() {
            return t[1..t.len() - 1].trim();
        }
    }
    if (t.starts_with('"') && t.ends_with('"') && t.len() >= 2)
        || (t.starts_with('\'') && t.ends_with('\'') && t.len() >= 2)
    {
        return t[1..t.len() - 1].trim();
    }
    t
}

/// Same product gate idea as compiler `looks_like_style_product` + bare const ref.
fn looks_like_style_product_value(s: &str) -> bool {
    let t = s.trim();
    if looks_like_style_product_value_inner(t) {
        return true;
    }
    // Optional outer `{…}` wrapper (nested quote forms).
    if t.starts_with('{') && t.ends_with('}') {
        let end = scan_balanced(t, 0);
        if end == t.len() {
            return looks_like_style_product_value_inner(t[1..t.len() - 1].trim());
        }
    }
    false
}

fn looks_like_style_product_value_inner(t: &str) -> bool {
    let t = t.trim_start();
    t.starts_with("style.")
        || t.starts_with("style(")
        || t.starts_with("match(")
        || t.starts_with("when(")
        || t.starts_with("compound(")
        || t.starts_with('[')
        || is_ident_method_call(t)
        || is_bare_ident(t)
}

fn is_bare_ident(s: &str) -> bool {
    let t = s.trim();
    if t.is_empty() {
        return false;
    }
    let mut chars = t.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {
            t.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
        }
        _ => false,
    }
}

fn is_ident_method_call(s: &str) -> bool {
    let Some((head, rest)) = s.split_once('.') else {
        return false;
    };
    if head.is_empty()
        || !head
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_')
    {
        return false;
    }
    // Not `style.…` (handled by starts_with).
    if head == "style" {
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

/// Normalize a product Style attr value into a block statement expression.
fn normalize_style_block_stmt(value: &str) -> String {
    let mut v = value.trim().to_string();
    // Strip one outer `{…}` if the whole value is wrapped.
    if v.starts_with('{') && v.ends_with('}') {
        let end = scan_balanced(&v, 0);
        if end == v.len() {
            v = v[1..v.len() - 1].trim().to_string();
        }
    }
    // Nested flats → channel (idempotent if already rewritten).
    v = rewrite_source(&v);

    let t = v.trim();
    if t.starts_with("style(")
        || t.starts_with("when(")
        || t.starts_with("match(")
        || t.starts_with("compound(")
    {
        return t.to_string();
    }
    if t.starts_with('[') {
        return format!("style({t})");
    }
    if t.starts_with("style.") {
        return rewrite_style_expr(t);
    }
    if is_bare_ident(t) {
        return format!("style({t})");
    }
    // Const-ref motion chain etc. — keep as-is (already product).
    t.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{collect_motion_attaches, lower_style, parse};
    use std::collections::HashMap;

    fn class_set(classes: &str) -> Vec<&str> {
        let mut v: Vec<&str> = classes.split_whitespace().collect();
        v.sort_unstable();
        v
    }

    /// Split concatenated `.sel{decls}` rules and sort for order-independent identity.
    fn css_rule_set(css: &str) -> Vec<String> {
        let mut rules = Vec::new();
        let bytes = css.as_bytes();
        let mut i = 0usize;
        while i < bytes.len() {
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if i >= bytes.len() {
                break;
            }
            let start = i;
            // selector until `{`
            while i < bytes.len() && bytes[i] != b'{' {
                i += 1;
            }
            if i >= bytes.len() {
                break;
            }
            let mut depth = 0i32;
            while i < bytes.len() {
                if bytes[i] == b'{' {
                    depth += 1;
                } else if bytes[i] == b'}' {
                    depth -= 1;
                    i += 1;
                    if depth == 0 {
                        break;
                    }
                    continue;
                }
                i += 1;
            }
            rules.push(css[start..i].to_string());
        }
        rules.sort();
        rules
    }

    fn emit_pair(old: &str, new: &str) {
        let (s_old, d_old) = parse(old);
        let (s_new, d_new) = parse(new);
        assert!(
            d_old.is_empty() && d_new.is_empty(),
            "diags old={:?} new={:?}",
            d_old.iter().map(|d| d.message()).collect::<Vec<_>>(),
            d_new.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let (c_old, css_old) = lower_style(&s_old, &HashMap::new());
        let (c_new, css_new) = lower_style(&s_new, &HashMap::new());
        // Channel flatten is box→paint→text→animate; class *order* may differ
        // from interleaved flat chains. Identity is the class multiset + CSS body.
        assert_eq!(
            class_set(&c_old),
            class_set(&c_new),
            "classes\nold={old}\nnew={new}\nold={c_old}\nnew={c_new}"
        );
        // CSS is a concatenation of `.sel{…}` rules; order follows class walk.
        assert_eq!(
            css_rule_set(&css_old),
            css_rule_set(&css_new),
            "css\nold={old}\nnew={new}\nold css:\n{css_old}\nnew css:\n{css_new}"
        );
        assert_eq!(
            collect_motion_attaches(&s_old),
            collect_motion_attaches(&s_new),
            "motion\nold={old}\nnew={new}"
        );
    }

    #[test]
    fn rewrite_box_paint() {
        let old = "style.pad(4).gap(2).bg(primary).radius(md)";
        let new = rewrite_style_expr(old);
        assert_eq!(
            new,
            "style({ box: pad(4).gap(2), paint: bg(primary).radius(md) })"
        );
        emit_pair(old, &new);
    }

    #[test]
    fn rewrite_when_hover_cond() {
        let old = "style.bg(primary).when({ hover: style.bg(muted) })";
        let new = rewrite_style_expr(old);
        assert_eq!(
            new,
            "style({ paint: bg(primary) }).when(hover, { paint: bg(muted) })"
        );
        emit_pair(old, &new);
    }

    #[test]
    fn rewrite_multi_when() {
        let old = "style.cursor(pointer).when({ hover: style.controlLift(), active: style.press(), disabled: style.cursor(not-allowed).opacity(50) })";
        let new = rewrite_style_expr(old);
        assert!(
            new.contains(".when(hover, { paint: controlLift() })"),
            "{new}"
        );
        assert!(new.contains(".when(active, { paint: press() })"), "{new}");
        assert!(
            new.contains(".when(disabled, { paint: cursor(not-allowed).opacity(50) })"),
            "{new}"
        );
        emit_pair(old, &new);
    }

    #[test]
    fn rewrite_animate() {
        let old = "style.bg(primary).enter({ opacity: 'fade' }).exit({ opacity: 'fade-out' })";
        let new = rewrite_style_expr(old);
        assert_eq!(
            new,
            "style({ paint: bg(primary), animate: enter({ opacity: 'fade' }).exit({ opacity: 'fade-out' }) })"
        );
        emit_pair(old, &new);
    }

    #[test]
    fn rewrite_text_channel() {
        let old = "style.fontSize(sm).fontWeight(semibold).fg(muted-foreground)";
        let new = rewrite_style_expr(old);
        assert_eq!(
            new,
            "style({ paint: fg(muted-foreground), text: fontSize(sm).fontWeight(semibold) })"
        );
        emit_pair(old, &new);
    }

    #[test]
    fn rewrite_source_nested_in_const() {
        let src = "const base = style.pad(4).bg(primary)\n";
        let out = rewrite_source(src);
        assert!(
            out.contains("style({ box: pad(4), paint: bg(primary) })"),
            "{out}"
        );
    }

    #[test]
    fn rewrite_source_preserves_newlines_between_consts() {
        let src = "\
  const board = style.pad(4)
  const boardTitle = style.fontSize(sm).textCase(uppercase)
  const badge = style.padX(1).when({
    stateLive: style.bg(primary),
  })
  const list = style.mt(3)
";
        let out = rewrite_source(src);
        assert!(
            out.contains("const board = style({ box: pad(4) })\n"),
            "board newline lost:\n{out}"
        );
        assert!(
            out.contains("const boardTitle = style({ text: fontSize(sm).textCase(uppercase) })\n"),
            "boardTitle newline lost:\n{out}"
        );
        assert!(
            out.contains("const list = style({ box: mt(3) })\n"),
            "list not rewritten or newline lost:\n{out}"
        );
        assert_eq!(
            out.matches("const ").count(),
            4,
            "const count changed:\n{out}"
        );
        assert!(
            !out.contains("})const"),
            "concatenated consts:\n{out}"
        );
    }

    #[test]
    fn rewrite_preserves_style_ref_call() {
        let src = "style(base)";
        assert_eq!(rewrite_source(src), src);
    }

    #[test]
    fn rewrite_compound_body() {
        let src = "compound({ variant: solid, tone: primary }, style.bg(primary).border(primary).fg(primary-foreground))";
        let out = rewrite_source(src);
        assert!(
            out.contains("style({ paint: bg(primary).border(primary).fg(primary-foreground) })"),
            "{out}"
        );
        emit_pair(
            "style.bg(primary).border(primary).fg(primary-foreground)",
            "style({ paint: bg(primary).border(primary).fg(primary-foreground) })",
        );
    }

    #[test]
    fn rewrite_button_base_identity() {
        let old = "style.display(inline-flex).appearance(none).fontWeight(medium).leading(none).radius(md).borderWidth(control-stroke).border(solid).border(transparent).borderImage(corner-image).isolate().position(relative).controlFrame().select(none).whitespace(nowrap).cursor(pointer).when({ hover: style.controlLift(), active: style.press(), disabled: style.cursor(not-allowed).opacity(50) })";
        let new = rewrite_style_expr(old);
        emit_pair(old, &new);
    }

    #[test]
    fn rewrite_bind_and_color_mix() {
        let old = "style.bg(bind(swatchColor)).colorMix(primary, 15)";
        let new = rewrite_style_expr(old);
        assert_eq!(
            new,
            "style({ paint: bg(bind(swatchColor)).colorMix(primary, 15) })"
        );
        emit_pair(old, &new);
    }

    #[test]
    fn rewrite_zero_arg_and_size() {
        let old = "style.group().size(icon-size-md).srOnly()";
        let new = rewrite_style_expr(old);
        assert_eq!(
            new,
            "style({ box: size(icon-size-md), paint: group().srOnly() })"
        );
        emit_pair(old, &new);
    }

    #[test]
    fn rewrite_press_paint_zero_arg_not_animate() {
        let old = "style.press()";
        let new = rewrite_style_expr(old);
        assert_eq!(new, "style({ paint: press() })");
        emit_pair(old, &new);
    }

    #[test]
    fn rewrite_focus_visible_hyphen() {
        let old = "style.focus-visible({ scale: 'spring:snappy' })";
        let new = rewrite_style_expr(old);
        assert_eq!(
            new,
            "style({ animate: focus-visible({ scale: 'spring:snappy' }) })"
        );
        emit_pair(old, &new);
    }

    #[test]
    fn lift_attr_host_to_block() {
        let src = r#"<div style={style.pad(4).bg(primary)}>x</div>"#;
        let out = rewrite_teaching_source(src);
        assert!(
            out.contains("style({ box: pad(4), paint: bg(primary) })"),
            "{out}"
        );
        assert!(!out.contains("style="), "{out}");
        assert!(!out.contains("style={"), "{out}");
    }

    #[test]
    fn lift_list_form() {
        let src = r#"<button style={[ base, match(v, { a: a }) ]}>ok</button>"#;
        let out = rewrite_teaching_source(src);
        assert!(out.contains("style(["), "{out}");
        assert!(out.contains("match(v, { a: a })"), "{out}");
        assert!(!out.contains("style="), "{out}");
    }

    #[test]
    fn lift_when_only() {
        let src = r#"<button style={when({ hover: style.bg(muted) })}>h</button>"#;
        let out = rewrite_teaching_source(src);
        assert!(out.contains("when("), "{out}");
        assert!(
            out.contains("style({ paint: bg(muted) })") || out.contains("bg(muted)"),
            "{out}"
        );
        assert!(!out.contains("style="), "{out}");
    }

    #[test]
    fn lift_const_ref() {
        let src = r#"<span style={filled}>x</span>"#;
        let out = rewrite_teaching_source(src);
        assert!(out.contains("style(filled)"), "{out}");
        assert!(!out.contains("style={filled}"), "{out}");
    }

    #[test]
    fn rewrite_flat_const() {
        let src = "const base = style.pad(2)";
        let out = rewrite_source(src);
        assert_eq!(out, "const base = style({ box: pad(2) })");
    }

    #[test]
    fn raw_css_string_attr_not_lifted() {
        let src = r#"<div style="color: red">x</div>"#;
        let out = rewrite_teaching_source(src);
        assert!(out.contains(r#"style="color: red""#), "{out}");
    }

    #[test]
    fn lift_sx_host_same_as_style() {
        let src = r#"<div sx={style.pad(4).bg(primary)}>x</div>"#;
        let out = rewrite_teaching_source(src);
        assert!(
            out.contains("style({ box: pad(4), paint: bg(primary) })"),
            "{out}"
        );
        assert!(!out.contains("sx="), "{out}");
    }

    #[test]
    fn lift_self_closing_expands() {
        let src = r#"<div style={style.pad(2)} />"#;
        let out = rewrite_teaching_source(src);
        assert!(out.contains("style({ box: pad(2) })"), "{out}");
        assert!(out.contains("</div>"), "{out}");
        assert!(!out.contains("/>"), "{out}");
        assert!(!out.contains("style="), "{out}");
    }

    #[test]
    fn lift_preserves_sibling_attrs() {
        let src = r#"<button on:click={invite} style={style.press({ scale: 'press' })}>Invite</button>"#;
        let out = rewrite_teaching_source(src);
        assert!(out.contains("on:click={invite}"), "{out}");
        assert!(
            out.contains("style({ animate: press({ scale: 'press' }) })")
                || out.contains("press({ scale: 'press' })"),
            "{out}"
        );
        assert!(!out.contains("style="), "{out}");
    }
}
