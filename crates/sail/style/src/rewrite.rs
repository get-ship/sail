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
}
