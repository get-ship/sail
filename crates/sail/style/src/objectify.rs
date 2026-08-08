//! Hard-cut codemod: list / free match|when|compound / `.when` → style object form.
//!
//! Source-to-source over Style expressions found in `.sail` files. Uses the
//! (migration-dual) parser to IR, then emits SPEC-STYLE-OBJECT-1 surface only.

use std::collections::BTreeMap;

use crate::channel::{self, Channel};
use crate::ir::Style;
use crate::parse;

/// Rewrite every Style product expression in `src` to object form.
pub fn objectify_source(src: &str) -> String {
    let mut out = String::with_capacity(src.len() + src.len() / 8);
    let mut i = 0usize;
    let bytes = src.as_bytes();
    while i < src.len() {
        // Skip HTML comments so teaching examples inside `<!-- style([…]) -->`
        // cannot open an unterminated scan that swallows the file.
        if bytes[i] == b'<' && src[i..].starts_with("<!--") {
            if let Some(rel) = src[i + 4..].find("-->") {
                let end = i + 4 + rel + 3;
                out.push_str(&src[i..end]);
                i = end;
                continue;
            }
        }
        // Skip block / line comments in controller JS / scope text.
        if bytes[i] == b'/' && i + 1 < bytes.len() {
            if bytes[i + 1] == b'/' {
                let mut j = i + 2;
                while j < bytes.len() && bytes[j] != b'\n' {
                    j += 1;
                }
                out.push_str(&src[i..j]);
                i = j;
                continue;
            }
            if bytes[i + 1] == b'*' {
                let mut j = i + 2;
                while j + 1 < bytes.len() && !(bytes[j] == b'*' && bytes[j + 1] == b'/') {
                    j += 1;
                }
                if j + 1 < bytes.len() {
                    j += 2;
                }
                out.push_str(&src[i..j]);
                i = j;
                continue;
            }
        }
        if let Some(end) = find_style_call_at(src, i) {
            let expr = &src[i..end];
            // Refuse to rewrite if the scanned span looks unclosed (hit EOF
            // without balancing) — leave source alone rather than eat the file.
            if end > i && balanced_parens(&src[i..end]) {
                out.push_str(&objectify_style_expr(expr));
                i = end;
                continue;
            }
        }
        // Free `match(` / `when(` / `compound(` product forms (not template tags).
        if let Some(end) = find_free_form_at(src, i) {
            let expr = &src[i..end];
            if end > i && balanced_parens(&src[i..end]) {
                out.push_str(&objectify_style_expr(expr));
                i = end;
                continue;
            }
        }
        let ch = src[i..].chars().next().expect("i in bounds");
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

fn balanced_parens(s: &str) -> bool {
    let bytes = s.as_bytes();
    let mut depth = 0i32;
    let mut i = 0usize;
    while i < bytes.len() {
        let c = bytes[i];
        if c == b'\'' || c == b'"' {
            i = scan_string(s, i);
            continue;
        }
        if c == b'(' {
            depth += 1;
        } else if c == b')' {
            depth -= 1;
            if depth < 0 {
                return false;
            }
        }
        i += 1;
    }
    depth == 0
}

/// True when `src[i..]` starts a `style(` product call (not `style.` flat).
fn find_style_call_at(src: &str, i: usize) -> Option<usize> {
    if !src[i..].starts_with("style(") {
        return None;
    }
    if i > 0 {
        let prev = src.as_bytes()[i - 1];
        if prev.is_ascii_alphanumeric() || prev == b'_' || prev == b'$' {
            return None;
        }
    }
    let mut end = scan_balanced(src, i + "style".len());
    // Optional `.when(…)` / motion tails after style(…)
    end = scan_method_tails(src, end);
    Some(end)
}

fn find_free_form_at(src: &str, i: usize) -> Option<usize> {
    for name in ["match(", "when(", "compound("] {
        if src[i..].starts_with(name) {
            if i > 0 {
                let prev = src.as_bytes()[i - 1];
                if prev.is_ascii_alphanumeric() || prev == b'_' || prev == b'$' {
                    return None;
                }
            }
            // Template tags are `<match` / `<when` — not free forms with `(`.
            let end = scan_balanced(src, i + name.len() - 1); // at '('
            return Some(end);
        }
    }
    None
}

fn scan_method_tails(src: &str, mut pos: usize) -> usize {
    let bytes = src.as_bytes();
    loop {
        let before = pos;
        skip_ws_comments(src, &mut pos);
        if pos >= bytes.len() || bytes[pos] != b'.' {
            return before;
        }
        let after_dot = pos + 1;
        let mut p = after_dot;
        skip_ws_comments(src, &mut p);
        if !scan_ident(src, &mut p) {
            return before;
        }
        let name = &src[after_dot..p];
        // Only consume known product tails.
        let is_when = name == "when";
        let is_motion = channel::is_animate_trigger(name);
        if !is_when && !is_motion {
            return before;
        }
        skip_ws_comments(src, &mut p);
        if p >= bytes.len() || bytes[p] != b'(' {
            return before;
        }
        pos = scan_balanced(src, p);
    }
}

/// Rewrite one Style expression (may already be new form — identity).
pub fn objectify_style_expr(expr: &str) -> String {
    let t = expr.trim();
    // Already a pure channel/object style with no old forms — keep as-is if
    // parse yields and pretty-print would only churn whitespace. Fast path:
    if !needs_objectify(t) {
        return expr.to_string();
    }
    let (style, diags) = parse::parse_legacy(t);
    // If parse failed hard, leave source alone.
    if matches!(style, Style::Empty)
        && diags.iter().any(|d| {
            matches!(
                d,
                crate::ir::StyleDiagnostic::ParseError { .. }
                    | crate::ir::StyleDiagnostic::ResidualProduct { .. }
            )
        })
    {
        return expr.to_string();
    }
    emit_product(&style)
}

fn needs_objectify(t: &str) -> bool {
    t.contains("style([")
        || t.contains("match(")
        || t.contains("compound(")
        || t.contains(".when(")
        || t.contains("when({")
        // free when(key, …) / when({ — when( after start or composition
        || (t.starts_with("when(") || t.starts_with("match(") || t.starts_with("compound("))
        // list without style head
        || t.starts_with('[')
}

/// Emit SPEC object form from IR.
fn emit_product(style: &Style) -> String {
    // Flatten top-level Chain/List into composition parts classified by kind.
    let mut items = Vec::new();
    flatten_top(style, &mut items);

    let mut comp_args: Vec<String> = Vec::new();
    let mut box_ops: Vec<String> = Vec::new();
    let mut paint_ops: Vec<String> = Vec::new();
    let mut text_ops: Vec<String> = Vec::new();
    let mut animate_ops: Vec<String> = Vec::new();
    let mut when_map: Vec<(String, String)> = Vec::new();
    // prop → (arm → body)
    let mut matches: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    let mut match_order: Vec<String> = Vec::new();
    let mut compounds: Vec<String> = Vec::new();
    // Motion attaches that live at top level (from ref.enter etc.)
    let mut loose_motion: Vec<String> = Vec::new();

    for item in items {
        classify_item(
            item,
            &mut comp_args,
            &mut box_ops,
            &mut paint_ops,
            &mut text_ops,
            &mut animate_ops,
            &mut when_map,
            &mut matches,
            &mut match_order,
            &mut compounds,
            &mut loose_motion,
        );
    }

    // Merge loose motion into animate channel.
    animate_ops.extend(loose_motion);

    let mut obj_parts: Vec<String> = Vec::new();
    if !box_ops.is_empty() {
        obj_parts.push(format!("box: {}", box_ops.join(".")));
    }
    if !paint_ops.is_empty() {
        obj_parts.push(format!("paint: {}", paint_ops.join(".")));
    }
    if !text_ops.is_empty() {
        obj_parts.push(format!("text: {}", text_ops.join(".")));
    }
    if !animate_ops.is_empty() {
        obj_parts.push(format!("animate: {}", animate_ops.join(".")));
    }
    if !when_map.is_empty() {
        let entries: Vec<String> = when_map
            .iter()
            .map(|(k, v)| format!("{k}: {v}"))
            .collect();
        obj_parts.push(format!("when: {{ {} }}", entries.join(", ")));
    }
    if !match_order.is_empty() {
        let mut axes = Vec::new();
        for prop in &match_order {
            if let Some(arms) = matches.get(prop) {
                let arm_s: Vec<String> = arms
                    .iter()
                    .map(|(k, v)| format!("{k}: {v}"))
                    .collect();
                axes.push(format!("{prop}: {{ {} }}", arm_s.join(", ")));
            }
        }
        obj_parts.push(format!("match: {{ {} }}", axes.join(", ")));
    }
    if !compounds.is_empty() {
        obj_parts.push(format!("compound: [ {} ]", compounds.join(", ")));
    }

    let mut args = comp_args;
    if !obj_parts.is_empty() {
        args.push(format!("{{ {} }}", obj_parts.join(", ")));
    }
    if args.is_empty() {
        "style({})".into()
    } else {
        format!("style({})", args.join(", "))
    }
}

fn flatten_top<'a>(style: &'a Style, out: &mut Vec<&'a Style>) {
    match style {
        Style::Chain(items) | Style::List(items) => {
            for i in items {
                flatten_top(i, out);
            }
        }
        Style::Empty => {}
        other => out.push(other),
    }
}

fn classify_item<'a>(
    item: &'a Style,
    comp_args: &mut Vec<String>,
    box_ops: &mut Vec<String>,
    paint_ops: &mut Vec<String>,
    text_ops: &mut Vec<String>,
    animate_ops: &mut Vec<String>,
    when_map: &mut Vec<(String, String)>,
    matches: &mut BTreeMap<String, Vec<(String, String)>>,
    match_order: &mut Vec<String>,
    compounds: &mut Vec<String>,
    loose_motion: &mut Vec<String>,
) {
    match item {
        Style::Ref(name) => comp_args.push(name.clone()),
        Style::Set(op) => {
            let call = format_set(op);
            match channel::channel_for_setter(&op.setter) {
                Some(Channel::Box) => box_ops.push(call),
                Some(Channel::Paint) => paint_ops.push(call),
                Some(Channel::Text) => text_ops.push(call),
                Some(Channel::Animate) => animate_ops.push(call),
                None => paint_ops.push(call),
            }
        }
        Style::MotionAttach { trigger, config } => {
            loose_motion.push(format!("{trigger}({config})"));
        }
        Style::When { entries } => {
            for (k, body) in entries {
                let partial = emit_partial(body);
                if let Some((_, existing)) = when_map.iter_mut().find(|(ek, _)| ek == k) {
                    *existing = compose_partials(existing, &partial);
                } else {
                    when_map.push((k.clone(), partial));
                }
            }
        }
        Style::Match { scrutinee, arms } => {
            if !matches.contains_key(scrutinee) {
                match_order.push(scrutinee.clone());
            }
            let slot = matches.entry(scrutinee.clone()).or_default();
            for (k, body) in arms {
                // Same arm key from a later match table → compose partials.
                if let Some((_, existing)) = slot.iter_mut().find(|(ak, _)| ak == k) {
                    // Wrap both as composition via nested style() if both are refs-only?
                    // Emit as style(a, b) when both are simple; else merge object forms.
                    *existing = compose_partials(existing, &emit_partial(body));
                } else {
                    slot.push((k.clone(), emit_partial(body)));
                }
            }
        }
        Style::Compound { when, body } => {
            compounds.push(emit_compound_row(when, body));
        }
        Style::Chain(items) | Style::List(items) => {
            for i in items {
                classify_item(
                    i,
                    comp_args,
                    box_ops,
                    paint_ops,
                    text_ops,
                    animate_ops,
                    when_map,
                    matches,
                    match_order,
                    compounds,
                    loose_motion,
                );
            }
        }
        Style::Empty => {}
    }
}

fn compose_partials(a: &str, b: &str) -> String {
    // Both refs → style(a, b) as arm value (composition of unconditional partials).
    if is_bare_ident(a) && is_bare_ident(b) {
        return format!("style({a}, {b})");
    }
    // a is style(…) already
    if a.starts_with("style(") && is_bare_ident(b) {
        // style(x, y) + z → style(x, y, z)
        if let Some(inner) = a.strip_prefix("style(").and_then(|s| s.strip_suffix(')')) {
            return format!("style({inner}, {b})");
        }
    }
    if is_bare_ident(a) && b.starts_with("style(") {
        if let Some(inner) = b.strip_prefix("style(").and_then(|s| s.strip_suffix(')')) {
            return format!("style({a}, {inner})");
        }
    }
    // Object merge — wrap as style(a, b) composition.
    format!("style({a}, {b})")
}

fn is_bare_ident(s: &str) -> bool {
    !s.is_empty()
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

fn emit_compound_row(when: &BTreeMap<String, String>, body: &Style) -> String {
    let mut parts: Vec<String> = when
        .iter()
        .map(|(k, v)| format!("{k}: {v}"))
        .collect();
    // Body may be channels + nested when.
    append_partial_entries(body, &mut parts);
    format!("{{ {} }}", parts.join(", "))
}

fn append_partial_entries(style: &Style, parts: &mut Vec<String>) {
    let mut box_ops = Vec::new();
    let mut paint_ops = Vec::new();
    let mut text_ops = Vec::new();
    let mut animate_ops = Vec::new();
    let mut when_map = Vec::new();
    let mut motion = Vec::new();
    let mut dummy_comp = Vec::new();
    let mut dummy_match = BTreeMap::new();
    let mut dummy_order = Vec::new();
    let mut dummy_compounds = Vec::new();
    classify_item(
        style,
        &mut dummy_comp,
        &mut box_ops,
        &mut paint_ops,
        &mut text_ops,
        &mut animate_ops,
        &mut when_map,
        &mut dummy_match,
        &mut dummy_order,
        &mut dummy_compounds,
        &mut motion,
    );
    // Nested match/compound inside compound body should not appear after hard cut;
    // if present, fold as composition args (lose structure) — rare.
    animate_ops.extend(motion);
    if !box_ops.is_empty() {
        parts.push(format!("box: {}", box_ops.join(".")));
    }
    if !paint_ops.is_empty() {
        parts.push(format!("paint: {}", paint_ops.join(".")));
    }
    if !text_ops.is_empty() {
        parts.push(format!("text: {}", text_ops.join(".")));
    }
    if !animate_ops.is_empty() {
        parts.push(format!("animate: {}", animate_ops.join(".")));
    }
    if !when_map.is_empty() {
        let entries: Vec<String> = when_map
            .iter()
            .map(|(k, v)| format!("{k}: {v}"))
            .collect();
        parts.push(format!("when: {{ {} }}", entries.join(", ")));
    }
    for r in dummy_comp {
        // bare ref body of compound — unusual
        parts.push(format!("/*ref:{r}*/"));
    }
}

/// Emit a nested partial (no top-level style() wrapper when object; refs bare).
fn emit_partial(style: &Style) -> String {
    match style {
        Style::Ref(n) => n.clone(),
        Style::Empty => "{ }".into(),
        Style::Set(_) | Style::MotionAttach { .. } | Style::Chain(_) | Style::List(_) => {
            let mut parts = Vec::new();
            append_partial_entries(style, &mut parts);
            if parts.is_empty() {
                "{ }".into()
            } else if parts.iter().all(|p| p.starts_with("/*ref:")) {
                // only refs — extract
                parts
                    .iter()
                    .filter_map(|p| p.strip_prefix("/*ref:").and_then(|s| s.strip_suffix("*/")))
                    .collect::<Vec<_>>()
                    .join(", ")
            } else {
                // Filter junk ref markers
                let clean: Vec<_> = parts
                    .into_iter()
                    .filter(|p| !p.starts_with("/*ref:"))
                    .collect();
                format!("{{ {} }}", clean.join(", "))
            }
        }
        Style::When { entries } => {
            let mut parts = Vec::new();
            let entries_s: Vec<String> = entries
                .iter()
                .map(|(k, b)| format!("{k}: {}", emit_partial(b)))
                .collect();
            parts.push(format!("when: {{ {} }}", entries_s.join(", ")));
            format!("{{ {} }}", parts.join(", "))
        }
        Style::Match { .. } | Style::Compound { .. } => {
            // Nested axes illegal in product; wrap as style(…) emission for fidelity.
            emit_product(style)
        }
    }
}

fn format_set(op: &crate::ir::PaintOp) -> String {
    if op.args.len() == 1 {
        let a = &op.args[0];
        if matches!(a, crate::ir::PaintArg::Token(t) if t.is_empty()) {
            return format!("{}()", op.setter);
        }
        return format!("{}({})", op.setter, a.display());
    }
    let args = op
        .args
        .iter()
        .map(|a| a.display())
        .collect::<Vec<_>>()
        .join(", ");
    format!("{}({})", op.setter, args)
}

// ─── scan helpers (mirror rewrite.rs) ───────────────────────────────────────

fn scan_balanced(src: &str, open_pos: usize) -> usize {
    let bytes = src.as_bytes();
    if open_pos >= bytes.len() {
        return open_pos;
    }
    let open = bytes[open_pos];
    let close = match open {
        b'(' => b')',
        b'{' => b'}',
        b'[' => b']',
        _ => return open_pos + 1,
    };
    let mut depth = 0i32;
    let mut i = open_pos;
    while i < bytes.len() {
        let c = bytes[i];
        if c == b'\'' || c == b'"' {
            i = scan_string(src, i);
            continue;
        }
        if c == open {
            depth += 1;
        } else if c == close {
            depth -= 1;
            if depth == 0 {
                return i + 1;
            }
        }
        i += 1;
    }
    bytes.len()
}

fn scan_string(src: &str, start: usize) -> usize {
    let bytes = src.as_bytes();
    let quote = bytes[start];
    let mut i = start + 1;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i += 2;
            continue;
        }
        if bytes[i] == quote {
            return i + 1;
        }
        i += 1;
    }
    bytes.len()
}

fn scan_ident(src: &str, pos: &mut usize) -> bool {
    let bytes = src.as_bytes();
    if *pos >= bytes.len() {
        return false;
    }
    let b0 = bytes[*pos];
    if !(b0.is_ascii_alphabetic() || b0 == b'_' || b0 == b'$') {
        return false;
    }
    *pos += 1;
    while *pos < bytes.len() {
        let b = bytes[*pos];
        if b.is_ascii_alphanumeric() || b == b'_' || b == b'-' {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::emit::lower_style;
    use std::collections::HashMap;

    fn emit_pair(old: &str, env: &HashMap<String, Style>) {
        let (s_old, d_old) = parse::parse_legacy(old);
        assert!(
            d_old.is_empty(),
            "old parse diags: {:?}",
            d_old.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let new = objectify_style_expr(old);
        let (s_new, d_new) = parse::parse(&new);
        assert!(
            d_new.is_empty(),
            "new `{new}` diags: {:?}",
            d_new.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let (_, css_old) = lower_style(&s_old, env);
        let (_, css_new) = lower_style(&s_new, env);
        // Normalize whitespace for compare — class names may differ path tags
        // but selectors+decls should match as sets of rules.
        assert_eq!(
            rule_set(&css_old),
            rule_set(&css_new),
            "css mismatch\nold: {old}\nnew: {new}\nold_css: {css_old}\nnew_css: {css_new}"
        );
    }

    fn rule_set(css: &str) -> Vec<String> {
        // Split on `}` roughly into rules; sort for order-independence.
        let mut rules: Vec<String> = css
            .split('}')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .map(|s| format!("{s}}}"))
            .collect();
        rules.sort();
        rules
    }

    #[test]
    fn objectify_when_chain() {
        let env = HashMap::new();
        emit_pair(
            "style({ paint: bg(primary) }).when(hover, { paint: bg(muted) })",
            &env,
        );
    }

    #[test]
    fn objectify_list_match_compound() {
        let env = HashMap::new();
        emit_pair(
            "style([ style({ paint: bg(primary) }), match(variant, { solid: style({ paint: bg(muted) }) }), compound({ pill: true }, style({ paint: radius(full) })) ])",
            &env,
        );
    }

    #[test]
    fn objectify_chip_like_when_in_arm() {
        let env = HashMap::new();
        emit_pair(
            "style([ match(variant, { solid: style({ paint: bg(primary) }).when(hover, { paint: opacity(90) }) }) ])",
            &env,
        );
    }

    #[test]
    fn objectify_multi_when_map() {
        let env = HashMap::new();
        emit_pair(
            "style({ paint: bg(primary) }).when(hover, { paint: bg(muted) }).when(disabled, { paint: opacity(50) })",
            &env,
        );
    }
}
