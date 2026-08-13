//! Rewrite covered `border` property chains to concern form (SPIN-4).
//!
//! Folds `borderWidth(…).border(…)` (and two-arg `border(width, color)`) into
//! `border(name)` when the color concern-resolves, otherwise a slot object.
//! `radius` and directional border setters are left alone.

use crate::concerns::{find_concern, resolve_concern_name};
use crate::scales;

/// Rewrite every foldable covered-border chain in `src`.
pub fn rewrite_concern_source(src: &str) -> String {
    let bytes = src.as_bytes();
    let mut out = String::with_capacity(src.len());
    let mut i = 0usize;
    let mut in_scope = false;
    while i < bytes.len() {
        if let Some(end) = skip_comment(src, i) {
            out.push_str(&src[i..end]);
            i = end;
            continue;
        }
        if let Some(end) = skip_html_comment(src, i) {
            out.push_str(&src[i..end]);
            i = end;
            continue;
        }
        if let Some(end) = skip_string(src, i) {
            out.push_str(&src[i..end]);
            i = end;
            continue;
        }
        if let Some(end) = skip_raw_tag_body(src, i) {
            out.push_str(&src[i..end]);
            i = end;
            continue;
        }
        if bytes[i] == b'<' {
            if tag_opens(src, i, "scope") {
                in_scope = true;
            } else if tag_opens(src, i, "/scope") {
                in_scope = false;
            }
        }
        // Template text is interpolation, not Style. Folding `inset(0)` there
        // into `place({ inset: 0 })` authors `{ inset: … }` as a scope lookup.
        if !in_scope {
            if let Some(end) = skip_template_text(src, i) {
                out.push_str(&src[i..end]);
                i = end;
                continue;
            }
        }
        if is_ident_start(bytes[i]) && !is_ident_continue_prev(bytes, i) {
            if let Some((end, calls)) = scan_call_chain(src, i) {
                if let Some(rewritten) = fold_border_chain(&calls) {
                    out.push_str(&rewritten);
                    i = end;
                    continue;
                }
                // Not foldable — fall through one char so inner `style({ paint: … })`
                // chains are still scanned.
            }
        }
        let ch = src[i..].chars().next().expect("i in bounds");
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

/// Source spans of retired dual form (`borderWidth` next to property `border(`)
/// in one setter chain). Empty after a successful first-party scrub.
pub fn retired_dual_border_sites(src: &str) -> Vec<String> {
    let bytes = src.as_bytes();
    let mut sites = Vec::new();
    let mut i = 0usize;
    while i < bytes.len() {
        if let Some(end) = skip_comment(src, i)
            .or_else(|| skip_html_comment(src, i))
            .or_else(|| skip_string(src, i))
            .or_else(|| skip_raw_tag_body(src, i))
        {
            i = end;
            continue;
        }
        if is_ident_start(bytes[i]) && !is_ident_continue_prev(bytes, i) {
            if let Some((end, calls)) = scan_call_chain(src, i) {
                if is_retired_dual(&calls) {
                    sites.push(
                        calls
                            .iter()
                            .map(|c| c.source.clone())
                            .collect::<Vec<_>>()
                            .join("."),
                    );
                    i = end;
                    continue;
                }
            }
        }
        i += src[i..].chars().next().expect("i in bounds").len_utf8();
    }
    sites
}

struct Call {
    name: String,
    args_inner: String,
    source: String,
}

fn fold_border_chain(calls: &[Call]) -> Option<String> {
    let mut width: Option<String> = None;
    let mut color: Option<String> = None;
    let mut style: Option<String> = None;
    let mut fold_idx: Vec<usize> = Vec::new();

    for (i, call) in calls.iter().enumerate() {
        match call.name.as_str() {
            "borderWidth" => {
                let w = single_arg(&call.args_inner)?;
                if w.starts_with('{') {
                    return None;
                }
                width = Some(w.to_string());
                fold_idx.push(i);
            }
            "border" => {
                if call.args_inner.trim().starts_with('{') {
                    return None;
                }
                let parts = split_args(&call.args_inner);
                match parts.as_slice() {
                    [one] => match classify_border_arg(one) {
                        ArgKind::Slots => return None,
                        ArgKind::Style => {
                            style = Some((*one).to_string());
                            fold_idx.push(i);
                        }
                        ArgKind::Width => {
                            width = Some((*one).to_string());
                            fold_idx.push(i);
                        }
                        ArgKind::Color => {
                            color = Some((*one).to_string());
                            fold_idx.push(i);
                        }
                    },
                    [w, c] => {
                        if classify_border_arg(w) != ArgKind::Width {
                            return None;
                        }
                        if classify_border_arg(c) != ArgKind::Color {
                            return None;
                        }
                        width = Some((*w).to_string());
                        color = Some((*c).to_string());
                        fold_idx.push(i);
                    }
                    _ => return None,
                }
            }
            _ => {}
        }
    }

    if fold_idx.is_empty() {
        return None;
    }
    // Already a named composite (`border(dialog)`) — leave it.
    if fold_idx.len() == 1 && color.is_some() && width.is_none() && style.is_none() {
        if preferred_concern_name(color.as_deref()?).is_some() {
            return None;
        }
    }

    let concern = emit_concern(color.as_deref(), width.as_deref(), style.as_deref())?;
    let fold_set: std::collections::HashSet<usize> = fold_idx.into_iter().collect();
    let mut out: Vec<String> = Vec::new();
    let mut inserted = false;
    for (i, call) in calls.iter().enumerate() {
        if fold_set.contains(&i) {
            if !inserted {
                out.push(concern.clone());
                inserted = true;
            }
            continue;
        }
        out.push(call.source.clone());
    }
    if !inserted {
        return None;
    }
    Some(out.join("."))
}

fn emit_concern(color: Option<&str>, width: Option<&str>, style: Option<&str>) -> Option<String> {
    let default_width = width.is_none_or(is_default_border_width);
    let default_style = style.is_none_or(|s| s == "solid");
    if let Some(c) = color {
        if let Some(name) = preferred_concern_name(c) {
            if default_width && default_style {
                return Some(format!("border({name})"));
            }
        }
    }
    let mut slots = Vec::new();
    if let Some(c) = color {
        slots.push(format!("color: {c}"));
    }
    if let Some(w) = width {
        // Concern width vocab is the ramp + hairline; `border-width` is the
        // property-setter spelling of the same default stroke.
        let w = if w == "border-width" { "hairline" } else { w };
        slots.push(format!("width: {w}"));
    }
    if let Some(s) = style {
        slots.push(format!("style: {s}"));
    }
    if slots.is_empty() {
        return None;
    }
    Some(format!("border({{ {} }})", slots.join(", ")))
}

/// Short concern name when `raw` is a composite identity or concern-resolves.
fn preferred_concern_name(raw: &str) -> Option<String> {
    let spec = find_concern("border")?;
    if resolve_concern_name(spec, raw).is_some() {
        return Some(raw.strip_prefix("border.").unwrap_or(raw).to_string());
    }
    if let Some(prefix) = raw.strip_suffix(".border") {
        if resolve_concern_name(spec, prefix).is_some() {
            return Some(prefix.to_string());
        }
    }
    None
}

fn is_retired_dual(calls: &[Call]) -> bool {
    let mut has_width = false;
    let mut has_property_border = false;
    for call in calls {
        match call.name.as_str() {
            "borderWidth" => has_width = true,
            "border" => {
                if call.args_inner.trim().starts_with('{') {
                    continue;
                }
                let parts = split_args(&call.args_inner);
                if parts.len() == 2 {
                    has_width = true;
                    has_property_border = true;
                    continue;
                }
                if parts.len() == 1 {
                    match classify_border_arg(parts[0]) {
                        ArgKind::Slots => {}
                        ArgKind::Width => has_width = true,
                        ArgKind::Style | ArgKind::Color => has_property_border = true,
                    }
                }
            }
            _ => {}
        }
    }
    has_width && has_property_border
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArgKind {
    Color,
    Width,
    Style,
    Slots,
}

fn classify_border_arg(raw: &str) -> ArgKind {
    let arg = raw.trim();
    if arg.starts_with('{') {
        return ArgKind::Slots;
    }
    if matches!(
        arg,
        "solid" | "dashed" | "dotted" | "double" | "hidden" | "none"
    ) {
        return ArgKind::Style;
    }
    if is_width_token(arg) {
        return ArgKind::Width;
    }
    ArgKind::Color
}

fn is_width_token(arg: &str) -> bool {
    arg == "hairline"
        || arg == "control-stroke"
        || arg == "border-width"
        || scales::contains("border-width", arg)
}

fn is_default_border_width(arg: &str) -> bool {
    matches!(arg, "1" | "hairline" | "border-width")
}

fn single_arg(inner: &str) -> Option<&str> {
    let parts = split_args(inner);
    (parts.len() == 1).then_some(parts[0])
}

fn split_args(inner: &str) -> Vec<&str> {
    let s = inner.trim();
    if s.is_empty() {
        return Vec::new();
    }
    let bytes = s.as_bytes();
    let mut parts = Vec::new();
    let mut start = 0usize;
    let mut depth_paren = 0i32;
    let mut depth_brace = 0i32;
    let mut depth_brack = 0i32;
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'\'' | b'"' => {
                i = scan_string_end(s, i);
                continue;
            }
            b'(' => depth_paren += 1,
            b')' => depth_paren -= 1,
            b'{' => depth_brace += 1,
            b'}' => depth_brace -= 1,
            b'[' => depth_brack += 1,
            b']' => depth_brack -= 1,
            b',' if depth_paren == 0 && depth_brace == 0 && depth_brack == 0 => {
                parts.push(s[start..i].trim());
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    let tail = s[start..].trim();
    if !tail.is_empty() {
        parts.push(tail);
    }
    parts
}

fn scan_call_chain(src: &str, start: usize) -> Option<(usize, Vec<Call>)> {
    let bytes = src.as_bytes();
    let mut pos = start;
    let mut calls = Vec::new();
    loop {
        let name_start = pos;
        if !scan_ident(src, &mut pos) {
            break;
        }
        let name = src[name_start..pos].to_string();
        skip_ws(src, &mut pos);
        if pos >= bytes.len() || bytes[pos] != b'(' {
            break;
        }
        let args_open = pos;
        let args_end = scan_balanced(src, pos);
        if args_end <= args_open + 1 {
            break;
        }
        let args_inner = src[args_open + 1..args_end - 1].to_string();
        calls.push(Call {
            name,
            args_inner,
            source: src[name_start..args_end].to_string(),
        });
        pos = args_end;
        let before_dot = pos;
        skip_ws(src, &mut pos);
        if pos < bytes.len() && bytes[pos] == b'.' {
            pos += 1;
            skip_ws(src, &mut pos);
            continue;
        }
        pos = before_dot;
        break;
    }
    if calls.is_empty() {
        return None;
    }
    Some((pos, calls))
}

fn scan_ident(src: &str, pos: &mut usize) -> bool {
    let bytes = src.as_bytes();
    if *pos >= bytes.len() || !is_ident_start(bytes[*pos]) {
        return false;
    }
    *pos += 1;
    while *pos < bytes.len() && is_ident_continue(bytes[*pos]) {
        *pos += 1;
    }
    true
}

fn skip_ws(src: &str, pos: &mut usize) {
    let bytes = src.as_bytes();
    while *pos < bytes.len() && bytes[*pos].is_ascii_whitespace() {
        *pos += 1;
    }
}

fn is_ident_start(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'_'
}

fn is_ident_continue(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'-'
}

fn is_ident_continue_prev(bytes: &[u8], i: usize) -> bool {
    i > 0 && is_ident_continue(bytes[i - 1])
}

fn skip_comment(src: &str, i: usize) -> Option<usize> {
    let bytes = src.as_bytes();
    if i + 1 >= bytes.len() {
        return None;
    }
    if bytes[i] == b'/' && bytes[i + 1] == b'/' {
        let mut j = i + 2;
        while j < bytes.len() && bytes[j] != b'\n' {
            j += 1;
        }
        return Some(j);
    }
    if bytes[i] == b'/' && bytes[i + 1] == b'*' {
        let mut j = i + 2;
        while j + 1 < bytes.len() && !(bytes[j] == b'*' && bytes[j + 1] == b'/') {
            j += 1;
        }
        return Some((j + 2).min(bytes.len()));
    }
    None
}

fn skip_html_comment(src: &str, i: usize) -> Option<usize> {
    if src[i..].starts_with("<!--") {
        return src[i..].find("-->").map(|rel| i + rel + 3);
    }
    None
}

fn skip_string(src: &str, i: usize) -> Option<usize> {
    let bytes = src.as_bytes();
    if i >= bytes.len() || (bytes[i] != b'\'' && bytes[i] != b'"') {
        return None;
    }
    Some(scan_string_end(src, i))
}

fn tag_opens(src: &str, i: usize, name: &str) -> bool {
    let bytes = src.as_bytes();
    if bytes[i] != b'<' {
        return false;
    }
    let rest = &src[i + 1..];
    if !rest.starts_with(name) {
        return false;
    }
    let after = i + 1 + name.len();
    after >= bytes.len() || {
        let c = bytes[after];
        c.is_ascii_whitespace() || c == b'>' || c == b'/'
    }
}

/// Copy markup text through to the next tag. Style lives on attributes / `<scope>`.
fn skip_template_text(src: &str, i: usize) -> Option<usize> {
    let bytes = src.as_bytes();
    if i >= bytes.len() || bytes[i] == b'<' {
        return None;
    }
    if !preceded_by_markup_tag_close(src, i) {
        return None;
    }
    src[i..].find('<').map(|rel| i + rel)
}

/// True when `src[..i]` already closed an open tag at brace/paren depth 0.
/// `{n > 0}` / `{n>0}` on the same tag is a comparison, not a closer.
fn preceded_by_markup_tag_close(src: &str, i: usize) -> bool {
    let bytes = src.as_bytes();
    let Some(lt) = bytes[..i].iter().rposition(|&c| c == b'<') else {
        return false;
    };
    let mut pos = lt + 1;
    let mut depth_brace = 0i32;
    let mut depth_paren = 0i32;
    let mut depth_brack = 0i32;
    while pos < i {
        let c = bytes[pos];
        if c == b'\'' || c == b'"' {
            pos = scan_string_end(src, pos);
            continue;
        }
        match c {
            b'{' => depth_brace += 1,
            b'}' => depth_brace = depth_brace.saturating_sub(1),
            b'(' => depth_paren += 1,
            b')' => depth_paren = depth_paren.saturating_sub(1),
            b'[' => depth_brack += 1,
            b']' => depth_brack = depth_brack.saturating_sub(1),
            b'>' if depth_brace == 0 && depth_paren == 0 && depth_brack == 0 => return true,
            _ => {}
        }
        pos += 1;
    }
    false
}

fn skip_raw_tag_body(src: &str, i: usize) -> Option<usize> {
    let bytes = src.as_bytes();
    if bytes[i] != b'<' {
        return None;
    }
    for tag in ["style", "script", "controller"] {
        if !src[i..].starts_with(&format!("<{tag}")) {
            continue;
        }
        let after = i + 1 + tag.len();
        if after < bytes.len() {
            let c = bytes[after];
            if !(c.is_ascii_whitespace() || c == b'>' || c == b'/') {
                continue;
            }
        }
        let close = format!("</{tag}>");
        return src[i..].find(&close).map(|rel| i + rel + close.len());
    }
    None
}

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
            pos = scan_string_end(src, pos);
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
        if matches!(c, b'(' | b'{' | b'[') && c != open {
            pos = scan_balanced(src, pos);
            continue;
        }
        pos += 1;
    }
    bytes.len()
}

fn scan_string_end(src: &str, start: usize) -> usize {
    let bytes = src.as_bytes();
    let quote = bytes[start];
    let mut pos = start + 1;
    while pos < bytes.len() {
        if bytes[pos] == b'\\' && pos + 1 < bytes.len() {
            pos += 2;
            continue;
        }
        if bytes[pos] == quote {
            return pos + 1;
        }
        pos += 1;
    }
    pos
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rw(src: &str) -> String {
        rewrite_concern_source(src)
    }

    #[test]
    fn dialog_width_plus_member_becomes_concern_name() {
        assert_eq!(
            rw("style({ paint: borderWidth(1).border(dialog.border) })"),
            "style({ paint: border(dialog) })"
        );
    }

    #[test]
    fn card_two_arg_becomes_slot_object() {
        assert_eq!(
            rw("style({ paint: border(1, border) })"),
            "style({ paint: border({ color: border, width: 1 }) })"
        );
    }

    #[test]
    fn border_width_token_becomes_hairline() {
        assert_eq!(
            rw("style({ paint: borderWidth(border-width).border(border) })"),
            "style({ paint: border({ color: border, width: hairline }) })"
        );
    }

    #[test]
    fn primary_color_becomes_slot_object() {
        assert_eq!(
            rw("style({ paint: borderWidth(1).border(primary) })"),
            "style({ paint: border({ color: primary, width: 1 }) })"
        );
    }

    #[test]
    fn subtle_name_prefers_concern() {
        assert_eq!(
            rw("style({ paint: borderWidth(1).border(subtle) })"),
            "style({ paint: border(subtle) })"
        );
    }

    #[test]
    fn radius_stays_in_chain() {
        assert_eq!(
            rw("style({ paint: borderWidth(1).border(primary).radius(md) })"),
            "style({ paint: border({ color: primary, width: 1 }).radius(md) })"
        );
    }

    #[test]
    fn directional_width_is_not_folded() {
        let src = "style({ paint: borderBWidth(1).border({ color: primary }) })";
        assert_eq!(rw(src), src);
    }

    #[test]
    fn lone_border_width_reset_is_untouched() {
        let src = "style({ paint: border({ width: 0 }).bg(transparent) })";
        assert_eq!(rw(src), src);
    }

    #[test]
    fn already_concern_form_is_identity() {
        let src = "style({ paint: border(dialog).radius(xl) })";
        assert_eq!(rw(src), src);
        let src = "style({ paint: border({ color: muted, width: 1 }) })";
        assert_eq!(rw(src), src);
    }

    #[test]
    fn width_style_color_folds_slots() {
        assert_eq!(
            rw("style({ paint: borderWidth(1).border(solid).border(field.border) })"),
            "style({ paint: border({ color: field.border, width: 1, style: solid }) })"
        );
    }

    #[test]
    fn non_default_width_keeps_slot_object() {
        assert_eq!(
            rw("style({ paint: borderWidth(2).border(dialog.border) })"),
            "style({ paint: border({ color: dialog.border, width: 2 }) })"
        );
    }

    #[test]
    fn comment_is_not_rewritten() {
        let src = "// borderWidth(1).border(primary)\nstyle({ paint: bg(card) })";
        assert_eq!(rw(src), src);
    }

    #[test]
    fn quoted_string_is_not_rewritten() {
        let src = r#"const note = "borderWidth(1).border(primary)";"#;
        assert_eq!(rw(src), src);
    }

    #[test]
    fn controller_body_is_not_rewritten() {
        let src = "<controller>\nconst s = borderWidth(1).border(primary);\n</controller>";
        assert_eq!(rw(src), src);
    }

    #[test]
    fn two_arg_directional_border_is_untouched() {
        let src = "style({ paint: borderB(1, border) })";
        assert_eq!(rw(src), src);
    }

    #[test]
    fn rewrite_is_idempotent() {
        let src = "style({ paint: borderWidth(1).border(dialog.border).radius(md) })";
        let once = rw(src);
        assert_eq!(rw(&once), once);
    }

    #[test]
    fn retired_sites_name_the_chain() {
        let sites = retired_dual_border_sites(
            "style({ paint: borderWidth(1).border(primary).radius(md) })",
        );
        assert_eq!(sites.len(), 1, "{sites:?}");
        assert!(sites[0].contains("borderWidth(1)"), "{sites:?}");
        assert!(sites[0].contains("border(primary)"), "{sites:?}");
        assert!(retired_dual_border_sites("style({ paint: border(dialog) })").is_empty());
    }

    #[test]
    fn retired_sites_ignore_directional() {
        assert!(
            retired_dual_border_sites("style({ paint: borderBWidth(1).border(primary) })")
                .is_empty()
        );
    }

    #[test]
    fn template_text_is_not_rewritten() {
        let src = "<p>Sticky top/left inset (number). borderWidth(1).border(primary)</p>";
        assert_eq!(rw(src), src);
        let attr = "<div style({ paint: borderWidth(1).border(dialog.border) })>x</div>";
        assert_eq!(rw(attr), "<div style({ paint: border(dialog) })>x</div>");
    }

    #[test]
    fn comparison_gt_on_same_tag_still_folds_style() {
        let spaced =
            "<div hidden={n > 0} style({ paint: borderWidth(1).border(dialog.border) })></div>";
        assert_eq!(
            rw(spaced),
            "<div hidden={n > 0} style({ paint: border(dialog) })></div>"
        );
        let tight =
            "<div hidden={n>0} style({ paint: borderWidth(1).border(dialog.border) })></div>";
        assert_eq!(
            rw(tight),
            "<div hidden={n>0} style({ paint: border(dialog) })></div>"
        );
    }

    #[test]
    fn scope_declarations_still_fold() {
        let src =
            "<scope>\n  style foo = { paint: borderWidth(1).border(dialog.border) }\n</scope>";
        assert_eq!(
            rw(src),
            "<scope>\n  style foo = { paint: border(dialog) }\n</scope>"
        );
    }
}
