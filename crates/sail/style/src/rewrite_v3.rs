//! Rewrite Style v2 source (channel wrappers + atlas names) to the v3 chain.
//!
//! ```text
//! { box: space({ pad-x: 3 }).place({ z: sticky }), paint: fill(surface),
//!   when: { hover: { paint: fill({ color: muted }) } } }
//! ```
//! →
//! ```text
//! { pad({ x: 3 }).position({ z: sticky }).bg(surface),
//!   when: { hover: bg(muted) } }
//! ```
//!
//! Skip template text outside `<scope>`, `<controller>` / `<script>` / `<style>`
//! bodies, comments, and strings (same contract as [`crate::concern_rewrite`]).

/// Rewrite every first-party Style v2 form in `src` to the v3 chain.
pub fn rewrite_v3_source(src: &str) -> String {
    let bytes = src.as_bytes();
    let mut out = String::with_capacity(src.len() + src.len() / 16);
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
        if !in_scope {
            if let Some(end) = skip_template_text(src, i) {
                out.push_str(&src[i..end]);
                i = end;
                continue;
            }
        }
        if is_ident_start(bytes[i]) && !is_ident_continue_prev(bytes, i) {
            if let Some((end, rewritten)) = rewrite_style_site(src, i) {
                out.push_str(&rewritten);
                i = end;
                continue;
            }
            if let Some((end, rewritten)) = rewrite_v2_call_at(src, i) {
                out.push_str(&rewritten);
                i = end;
                continue;
            }
        }
        if bytes[i] == b'{' {
            let keep_braces = !preceded_by_colon(src, i);
            if let Some((end, rewritten)) = try_rewrite_style_object(src, i, keep_braces) {
                out.push_str(&rewritten);
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

/// `style name = {…}` / `style(…)` — rewrite the site, keep the `style` head.
fn rewrite_style_site(src: &str, start: usize) -> Option<(usize, String)> {
    let bytes = src.as_bytes();
    if !src[start..].starts_with("style") {
        return None;
    }
    let after = start + "style".len();
    if after < bytes.len() && is_ident_continue(bytes[after]) {
        return None;
    }
    let mut pos = after;
    skip_ws(src, &mut pos);
    if pos < bytes.len() && bytes[pos] == b'(' {
        let args_end = scan_balanced(src, pos);
        let inner = &src[pos + 1..args_end - 1];
        let rewritten = rewrite_style_args(inner);
        let mut out = String::from("style(");
        out.push_str(&rewritten);
        out.push(')');
        return Some((args_end, out));
    }
    let name_start = pos;
    if !scan_ident(src, &mut pos) {
        return None;
    }
    let name = &src[name_start..pos];
    skip_ws(src, &mut pos);
    if pos >= bytes.len() || bytes[pos] != b'=' {
        return None;
    }
    pos += 1;
    skip_ws(src, &mut pos);
    let rhs_start = pos;
    let rhs_end = scan_decl_rhs(src, pos);
    if rhs_end <= rhs_start {
        return None;
    }
    let rewritten = rewrite_style_args(&src[rhs_start..rhs_end]);
    let mut out = String::from("style ");
    out.push_str(name);
    out.push_str(" = ");
    out.push_str(&rewritten);
    Some((rhs_end, out))
}

/// Compose / call args: `{…}` objects, `style(…)`, and setter chains.
fn rewrite_style_args(inner: &str) -> String {
    let s = inner;
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(s.len());
    let mut i = 0usize;
    while i < bytes.len() {
        if let Some(end) = skip_comment(s, i) {
            out.push_str(&s[i..end]);
            i = end;
            continue;
        }
        if bytes[i].is_ascii_whitespace() || bytes[i] == b',' {
            let ch = s[i..].chars().next().expect("i in bounds");
            out.push(ch);
            i += ch.len_utf8();
            continue;
        }
        if bytes[i] == b'{' {
            if let Some((end, rewritten)) = try_rewrite_style_object(s, i, true) {
                out.push_str(&rewritten);
                i = end;
                continue;
            }
        }
        if s[i..].starts_with("style") {
            if let Some((end, rewritten)) = rewrite_style_site(s, i) {
                out.push_str(&rewritten);
                i = end;
                continue;
            }
        }
        // One compose arg: chain / ref until comma at depth 0.
        let end = scan_arg_end(s, i);
        if end <= i {
            let ch = s[i..].chars().next().expect("i in bounds");
            out.push(ch);
            i += ch.len_utf8();
            continue;
        }
        out.push_str(&rewrite_chain(&s[i..end]));
        i = end;
    }
    out
}

fn scan_decl_rhs(src: &str, start: usize) -> usize {
    let bytes = src.as_bytes();
    let mut pos = start;
    skip_ws(src, &mut pos);
    if pos >= bytes.len() {
        return pos;
    }
    loop {
        skip_ws_not_nl(src, &mut pos);
        if pos >= bytes.len() {
            return pos;
        }
        match bytes[pos] {
            b'{' => pos = scan_balanced(src, pos),
            b'(' => pos = scan_balanced(src, pos),
            b'[' => pos = scan_balanced(src, pos),
            b'\'' | b'"' => pos = scan_string_end(src, pos),
            b',' => {
                pos += 1;
                continue;
            }
            b'/' if pos + 1 < bytes.len() && (bytes[pos + 1] == b'/' || bytes[pos + 1] == b'*') => {
                if let Some(end) = skip_comment(src, pos) {
                    pos = end;
                    continue;
                }
                pos += 1;
            }
            c if is_ident_start(c) => {
                let _ = scan_ident(src, &mut pos);
            }
            b'.' => pos += 1,
            b')' | b'}' | b']' | b'<' | b'\n' => return pos,
            _ => return pos,
        }
        skip_ws_not_nl(src, &mut pos);
        if pos < bytes.len() && bytes[pos] == b',' {
            pos += 1;
            continue;
        }
        return pos;
    }
}

fn skip_ws_not_nl(src: &str, pos: &mut usize) {
    let bytes = src.as_bytes();
    while *pos < bytes.len() && bytes[*pos].is_ascii_whitespace() && bytes[*pos] != b'\n' {
        *pos += 1;
    }
}

fn scan_arg_end(src: &str, start: usize) -> usize {
    let bytes = src.as_bytes();
    let mut pos = start;
    let mut depth_paren = 0i32;
    let mut depth_brace = 0i32;
    let mut depth_brack = 0i32;
    while pos < bytes.len() {
        match bytes[pos] {
            b'\'' | b'"' => {
                pos = scan_string_end(src, pos);
                continue;
            }
            b'/' if pos + 1 < bytes.len() && (bytes[pos + 1] == b'/' || bytes[pos + 1] == b'*') => {
                if let Some(end) = skip_comment(src, pos) {
                    pos = end;
                    continue;
                }
            }
            b'(' => depth_paren += 1,
            b')' => {
                if depth_paren == 0 && depth_brace == 0 && depth_brack == 0 {
                    return pos;
                }
                depth_paren -= 1;
            }
            b'{' => depth_brace += 1,
            b'}' => {
                if depth_brace == 0 && depth_paren == 0 && depth_brack == 0 {
                    return pos;
                }
                depth_brace -= 1;
            }
            b'[' => depth_brack += 1,
            b']' => {
                if depth_brack == 0 && depth_paren == 0 && depth_brace == 0 {
                    return pos;
                }
                depth_brack -= 1;
            }
            b',' if depth_paren == 0 && depth_brace == 0 && depth_brack == 0 => return pos,
            _ => {}
        }
        pos += 1;
    }
    pos
}

struct ObjectMember {
    key: Option<String>,
    value: String,
}

struct StyleObject {
    members: Vec<ObjectMember>,
    spaced_brace: bool,
    spaced_colon: bool,
}

fn try_rewrite_style_object(src: &str, start: usize, keep_braces: bool) -> Option<(usize, String)> {
    let obj = parse_object(src, start)?;
    if !is_style_object(&obj) {
        return None;
    }
    let end = scan_balanced(src, start);
    let rewritten = emit_style_object(&obj, keep_braces);
    Some((end, rewritten))
}

fn parse_object(src: &str, start: usize) -> Option<StyleObject> {
    let bytes = src.as_bytes();
    if start >= bytes.len() || bytes[start] != b'{' {
        return None;
    }
    let end = scan_balanced(src, start);
    if end <= start + 1 {
        return None;
    }
    let inner = &src[start + 1..end - 1];
    let spaced_brace = src[start + 1..]
        .chars()
        .next()
        .map(|c| c.is_whitespace())
        .unwrap_or(false);
    let mut spaced_colon = false;
    let mut members = Vec::new();
    let mut i = 0usize;
    let ib = inner.as_bytes();
    while i < ib.len() {
        if let Some(e) = skip_comment(inner, i) {
            i = e;
            continue;
        }
        if ib[i].is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if ib[i] == b',' {
            i += 1;
            continue;
        }
        let mem_start = i;
        let (key, value_start) = peek_keyed_member(inner, i);
        if key.is_some() && !spaced_colon {
            let mut colon = i;
            let _ = scan_ident(inner, &mut colon);
            skip_ws(inner, &mut colon);
            if colon < ib.len()
                && ib[colon] == b':'
                && colon + 1 < ib.len()
                && ib[colon + 1].is_ascii_whitespace()
            {
                spaced_colon = true;
            }
        }
        let value_end = scan_arg_end(inner, value_start);
        if value_end <= mem_start {
            break;
        }
        let value = inner[value_start..value_end].trim().to_string();
        if key.is_none() && value.is_empty() {
            i = value_end.max(i + 1);
            continue;
        }
        members.push(ObjectMember { key, value });
        i = if value_end > i { value_end } else { i + 1 };
    }
    Some(StyleObject {
        members,
        spaced_brace,
        spaced_colon,
    })
}

fn peek_keyed_member(src: &str, start: usize) -> (Option<String>, usize) {
    let mut pos = start;
    let name_start = pos;
    if !scan_ident(src, &mut pos) {
        return (None, start);
    }
    // Quoted / dashed keys: `accent-bar`, `status-rail`.
    let name = src[name_start..pos].to_string();
    skip_ws(src, &mut pos);
    let bytes = src.as_bytes();
    if pos < bytes.len() && bytes[pos] == b':' {
        pos += 1;
        skip_ws(src, &mut pos);
        return (Some(name), pos);
    }
    (None, start)
}

fn is_style_object(obj: &StyleObject) -> bool {
    obj.members.iter().any(|m| match m.key.as_deref() {
        Some("box" | "paint" | "when" | "match" | "compound" | "animate") => true,
        Some("text") => value_is_chain(&m.value),
        Some(_) => false,
        None => looks_like_setter_chain(&m.value),
    })
}

fn value_is_chain(value: &str) -> bool {
    let v = value.trim();
    if v.is_empty() || v.starts_with('{') {
        return false;
    }
    let mut pos = 0usize;
    if !scan_ident(v, &mut pos) {
        return false;
    }
    skip_ws(v, &mut pos);
    pos < v.len() && v.as_bytes()[pos] == b'('
}

fn looks_like_setter_chain(value: &str) -> bool {
    value_is_chain(value)
}

const CHANNEL_KEYS: &[&str] = &["box", "paint", "text"];

enum Emitted {
    Chain(String),
    Keyed(String, String),
}

fn emit_style_object(obj: &StyleObject, keep_braces: bool) -> String {
    let mut items: Vec<Emitted> = Vec::new();
    let mut pending_chain: Vec<String> = Vec::new();
    let mut animate_extra: Vec<String> = Vec::new();

    let flush_chain = |pending: &mut Vec<String>, items: &mut Vec<Emitted>| {
        let chain = join_chains(pending);
        pending.clear();
        if !chain.is_empty() {
            items.push(Emitted::Chain(chain));
        }
    };

    for m in &obj.members {
        match m.key.as_deref() {
            Some(key) if CHANNEL_KEYS.contains(&key) => {
                let (chain, plays) = rewrite_chain_with_play(&m.value);
                if !chain.is_empty() {
                    pending_chain.push(chain);
                }
                animate_extra.extend(plays);
            }
            Some("when") => {
                flush_chain(&mut pending_chain, &mut items);
                items.push(Emitted::Keyed("when".into(), rewrite_when_map(&m.value)));
            }
            Some("match") => {
                flush_chain(&mut pending_chain, &mut items);
                items.push(Emitted::Keyed("match".into(), rewrite_match_map(&m.value)));
            }
            Some("compound") => {
                flush_chain(&mut pending_chain, &mut items);
                items.push(Emitted::Keyed(
                    "compound".into(),
                    rewrite_compound_array(&m.value),
                ));
            }
            Some("animate") => {
                flush_chain(&mut pending_chain, &mut items);
                items.push(Emitted::Keyed(
                    "animate".into(),
                    rewrite_animate_value(&m.value),
                ));
            }
            Some(key) => {
                flush_chain(&mut pending_chain, &mut items);
                items.push(Emitted::Keyed(
                    key.to_string(),
                    rewrite_predicate_value(&m.value),
                ));
            }
            None => {
                let (chain, plays) = rewrite_chain_with_play(&m.value);
                if !chain.is_empty() {
                    pending_chain.push(chain);
                }
                animate_extra.extend(plays);
            }
        }
    }
    flush_chain(&mut pending_chain, &mut items);

    if !animate_extra.is_empty() {
        merge_animate_items(
            &mut items,
            animate_extra,
            obj.spaced_colon || obj.spaced_brace,
        );
    }

    let only_chain = items.iter().all(|i| matches!(i, Emitted::Chain(_))) && items.len() <= 1;
    if only_chain && !keep_braces {
        return match items.into_iter().next() {
            Some(Emitted::Chain(c)) => c,
            _ => String::new(),
        };
    }

    let colon = if obj.spaced_colon { ": " } else { ":" };
    let comma = ", ";
    let mut body = String::new();
    for item in &items {
        if !body.is_empty() {
            body.push_str(comma);
        }
        match item {
            Emitted::Chain(c) => body.push_str(c),
            Emitted::Keyed(k, v) => {
                body.push_str(k);
                body.push_str(colon);
                body.push_str(v);
            }
        }
    }
    if keep_braces || !only_chain {
        wrap_object(&body, obj.spaced_brace)
    } else {
        body
    }
}

fn wrap_object(body: &str, spaced_brace: bool) -> String {
    if body.is_empty() {
        return "{}".into();
    }
    if spaced_brace {
        format!("{{ {body} }}")
    } else {
        format!("{{{body}}}")
    }
}

fn join_chains(parts: &[String]) -> String {
    let mut out = String::new();
    for p in parts {
        let p = p.trim();
        if p.is_empty() {
            continue;
        }
        if out.is_empty() {
            out.push_str(p);
        } else {
            out.push('.');
            out.push_str(p);
        }
    }
    out
}

fn rewrite_predicate_value(value: &str) -> String {
    let v = value.trim();
    if v.starts_with('{') {
        if let Some((_, rewritten)) = try_rewrite_style_object(v, 0, true) {
            return rewritten;
        }
    }
    if v.starts_with("style") {
        if let Some((_, rewritten)) = rewrite_style_site(v, 0) {
            return rewritten;
        }
    }
    if value_is_chain(v) {
        return rewrite_chain(v);
    }
    v.to_string()
}

fn rewrite_when_map(value: &str) -> String {
    rewrite_keyed_map(value, MapKind::When)
}

fn rewrite_match_map(value: &str) -> String {
    rewrite_keyed_map(value, MapKind::Match)
}

#[derive(Clone, Copy)]
enum MapKind {
    When,
    Match,
}

fn rewrite_keyed_map(value: &str, kind: MapKind) -> String {
    let v = value.trim();
    let Some(obj) = parse_object(v, 0) else {
        return v.to_string();
    };
    let colon = if obj.spaced_colon { ": " } else { ":" };
    let mut parts = Vec::new();
    for m in &obj.members {
        let Some(key) = m.key.as_deref() else {
            continue;
        };
        let rewritten = match kind {
            MapKind::When => rewrite_when_value(&m.value),
            MapKind::Match => rewrite_match_arm_value(&m.value),
        };
        parts.push(format!("{key}{colon}{rewritten}"));
    }
    wrap_object(&parts.join(", "), obj.spaced_brace)
}

fn rewrite_when_value(value: &str) -> String {
    let v = value.trim();
    if v.starts_with("style") {
        if let Some((_, rewritten)) = rewrite_style_site(v, 0) {
            return rewritten;
        }
    }
    if v.starts_with('{') {
        if let Some((_, rewritten)) = try_rewrite_style_object(v, 0, false) {
            return rewritten;
        }
        // Nested `when: { before: {…} }` (when-in-when) or a when map.
        if let Some(obj) = parse_object(v, 0) {
            if obj.members.iter().all(|m| m.key.is_some()) && !is_style_object(&obj) {
                return rewrite_keyed_map(v, MapKind::When);
            }
        }
    }
    if value_is_chain(v) {
        return rewrite_chain(v);
    }
    v.to_string()
}

fn rewrite_match_arm_value(value: &str) -> String {
    let v = value.trim();
    if v.starts_with("style") {
        if let Some((_, rewritten)) = rewrite_style_site(v, 0) {
            return rewritten;
        }
    }
    if v.starts_with('{') {
        // Nested match table (`{ solid: { paint: … }, soft: name }`) or a style object.
        if let Some((_, rewritten)) = try_rewrite_style_object(v, 0, false) {
            return rewritten;
        }
        if let Some(obj) = parse_object(v, 0) {
            if obj.members.iter().all(|m| m.key.is_some()) {
                return rewrite_keyed_map(v, MapKind::Match);
            }
        }
    }
    if value_is_chain(v) {
        return rewrite_chain(v);
    }
    v.to_string()
}

fn rewrite_compound_array(value: &str) -> String {
    let v = value.trim();
    if !v.starts_with('[') {
        return v.to_string();
    }
    let end = scan_balanced(v, 0);
    let inner = &v[1..end - 1];
    let mut out = String::from("[");
    let mut i = 0usize;
    let ib = inner.as_bytes();
    while i < ib.len() {
        if let Some(e) = skip_comment(inner, i) {
            out.push_str(&inner[i..e]);
            i = e;
            continue;
        }
        if ib[i].is_ascii_whitespace() || ib[i] == b',' {
            out.push(inner[i..].chars().next().expect("in bounds"));
            i += 1;
            continue;
        }
        if ib[i] == b'{' {
            if let Some((e, rewritten)) = try_rewrite_style_object(inner, i, true) {
                out.push_str(&rewritten);
                i = e;
                continue;
            }
            let e = scan_balanced(inner, i);
            out.push_str(&inner[i..e]);
            i = e;
            continue;
        }
        let ch = inner[i..].chars().next().expect("in bounds");
        out.push(ch);
        i += ch.len_utf8();
    }
    out.push(']');
    if end < v.len() {
        out.push_str(&v[end..]);
    }
    out
}

fn rewrite_animate_value(value: &str) -> String {
    let v = value.trim();
    if v.starts_with('{') {
        // Trigger map: rewrite play/motion inside values; keys stay.
        if let Some(obj) = parse_object(v, 0) {
            let colon = if obj.spaced_colon { ": " } else { ":" };
            let mut parts = Vec::new();
            for m in &obj.members {
                let Some(key) = m.key.as_deref() else {
                    continue;
                };
                let val = rewrite_chain(&m.value);
                parts.push(format!("{key}{colon}{val}"));
            }
            return wrap_object(&parts.join(", "), obj.spaced_brace);
        }
    }
    rewrite_chain(v)
}

fn merge_animate_items(items: &mut Vec<Emitted>, plays: Vec<String>, spaced: bool) {
    let play_body = plays.join(", ");
    if let Some(Emitted::Keyed(_, existing)) = items
        .iter_mut()
        .find(|i| matches!(i, Emitted::Keyed(k, _) if k == "animate"))
    {
        let e = existing.trim();
        if e.starts_with('{') {
            if let Some(obj) = parse_object(e, 0) {
                if !obj
                    .members
                    .iter()
                    .any(|m| m.key.as_deref() == Some("transition"))
                {
                    let colon = if obj.spaced_colon { ": " } else { ":" };
                    let insert = format!("transition{colon}{play_body}");
                    if e.ends_with('}') {
                        let open = e[..e.len() - 1].trim_end();
                        if open.ends_with('{') {
                            *existing = wrap_object(&insert, obj.spaced_brace);
                        } else {
                            *existing = format!("{open}, {insert}}}");
                        }
                    }
                }
            }
        }
        return;
    }
    let colon = if spaced { ": " } else { ":" };
    items.push(Emitted::Keyed(
        "animate".into(),
        wrap_object(&format!("transition{colon}{play_body}"), spaced),
    ));
}

fn rewrite_v2_call_at(src: &str, start: usize) -> Option<(usize, String)> {
    let (end, calls) = scan_call_chain(src, start)?;
    if !calls.iter().any(|c| is_v2_call(&c.name)) {
        return None;
    }
    let (chain, _plays) = rewrite_calls(&calls);
    if chain.is_empty() {
        return None;
    }
    Some((end, chain))
}

fn is_v2_call(name: &str) -> bool {
    matches!(name, "fill" | "ink" | "place" | "space" | "flow" | "motion")
}

fn rewrite_chain(src: &str) -> String {
    rewrite_chain_with_play(src).0
}

fn rewrite_chain_with_play(src: &str) -> (String, Vec<String>) {
    let s = src.trim();
    if s.is_empty() {
        return (String::new(), Vec::new());
    }
    if let Some((end, calls)) = scan_call_chain(s, 0) {
        if end >= s.len() || s[end..].trim().is_empty() {
            return rewrite_calls(&calls);
        }
    }
    // Mixed / already-rewritten: walk calls inside.
    (rewrite_chain_loose(s), Vec::new())
}

fn rewrite_chain_loose(src: &str) -> String {
    let bytes = src.as_bytes();
    let mut out = String::with_capacity(src.len());
    let mut i = 0usize;
    while i < bytes.len() {
        if let Some(end) = skip_comment(src, i) {
            out.push_str(&src[i..end]);
            i = end;
            continue;
        }
        if let Some(end) = skip_string(src, i) {
            out.push_str(&src[i..end]);
            i = end;
            continue;
        }
        if is_ident_start(bytes[i]) && !is_ident_continue_prev(bytes, i) {
            if let Some((end, rewritten)) = rewrite_v2_call_at(src, i) {
                out.push_str(&rewritten);
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

struct Call {
    name: String,
    args_inner: String,
}

fn rewrite_calls(calls: &[Call]) -> (String, Vec<String>) {
    let mut out: Vec<String> = Vec::new();
    let mut plays: Vec<String> = Vec::new();
    for call in calls {
        match call.name.as_str() {
            "fill" => out.push(rewrite_fill(&call.args_inner)),
            "ink" => out.push(rewrite_ink(&call.args_inner)),
            "place" => out.push(rewrite_place(&call.args_inner)),
            "space" => out.extend(rewrite_space(&call.args_inner)),
            "flow" => out.extend(rewrite_flow(&call.args_inner)),
            "motion" => {
                let (xf, play) = rewrite_motion(&call.args_inner);
                out.extend(xf);
                plays.extend(play);
            }
            _ => {
                // Recurse into args in case a nested chain slipped in.
                let args = rewrite_chain_loose(&call.args_inner);
                out.push(format!("{}({args})", call.name));
            }
        }
    }
    (out.join("."), plays)
}

fn rewrite_fill(args: &str) -> String {
    rewrite_primary_concern("bg", "color", args)
}

fn rewrite_ink(args: &str) -> String {
    rewrite_primary_concern("fg", "color", args)
}

fn rewrite_primary_concern(name: &str, primary: &str, args: &str) -> String {
    let args = args.trim();
    if let Some(slots) = parse_slots(args) {
        if slots.len() == 1 && slots[0].0 == primary {
            return format!("{name}({})", slots[0].1);
        }
        return format!("{name}({})", format_slots(&slots, args_spaced(args)));
    }
    format!("{name}({args})")
}

fn rewrite_place(args: &str) -> String {
    let args = args.trim();
    if let Some(slots) = parse_slots(args) {
        let remapped: Vec<(String, String)> = slots
            .into_iter()
            .map(|(k, v)| (remap_place_slot(&k), v))
            .collect();
        if remapped.len() == 1 && remapped[0].0 == "scheme" {
            return format!("position({})", remapped[0].1);
        }
        return format!("position({})", format_slots(&remapped, args_spaced(args)));
    }
    format!("position({args})")
}

fn remap_place_slot(key: &str) -> String {
    match key {
        "position" => "scheme".into(),
        "top" => "t".into(),
        "bottom" => "b".into(),
        "left" => "l".into(),
        "right" => "r".into(),
        other => other.to_string(),
    }
}

fn rewrite_space(args: &str) -> Vec<String> {
    let args = args.trim();
    let Some(slots) = parse_slots(args) else {
        return vec![format!("pad({args})")];
    };
    let mut pad = Vec::new();
    let mut margin = Vec::new();
    let mut unknown = Vec::new();
    for (k, v) in slots {
        if let Some(pk) = remap_pad_slot(&k) {
            pad.push((pk, v));
        } else if let Some(mk) = remap_margin_slot(&k) {
            margin.push((mk, v));
        } else {
            unknown.push((k, v));
        }
    }
    let spaced = args_spaced(args);
    let mut out = Vec::new();
    if !pad.is_empty() {
        out.push(emit_axis("pad", &pad, spaced));
    }
    if !margin.is_empty() {
        out.push(emit_axis("margin", &margin, spaced));
    }
    if !unknown.is_empty() {
        for (k, v) in &unknown {
            out.push(format!("{k}({v})"));
        }
    }
    if out.is_empty() {
        out.push("pad()".into());
    }
    out
}

fn remap_pad_slot(key: &str) -> Option<String> {
    Some(match key {
        "pad" => "all".into(),
        "pad-x" => "x".into(),
        "pad-y" => "y".into(),
        "pad-t" => "t".into(),
        "pad-b" => "b".into(),
        "pad-l" => "l".into(),
        "pad-r" => "r".into(),
        "pad-s" => "s".into(),
        "pad-e" => "e".into(),
        _ => return None,
    })
}

fn remap_margin_slot(key: &str) -> Option<String> {
    Some(match key {
        "m" => "all".into(),
        "mx" => "x".into(),
        "my" => "y".into(),
        "mt" => "t".into(),
        "mb" => "b".into(),
        "ml" => "l".into(),
        "mr" => "r".into(),
        "ms" => "s".into(),
        "me" => "e".into(),
        _ => return None,
    })
}

fn emit_axis(name: &str, slots: &[(String, String)], spaced: bool) -> String {
    if slots.len() == 1 && slots[0].0 == "all" {
        return format!("{name}({})", slots[0].1);
    }
    format!("{name}({})", format_slots(slots, spaced))
}

fn rewrite_flow(args: &str) -> Vec<String> {
    let args = args.trim();
    if let Some(slots) = parse_slots(args) {
        return slots
            .into_iter()
            .map(|(k, v)| format!("{k}({v})"))
            .collect();
    }
    vec![format!("display({args})")]
}

fn rewrite_motion(args: &str) -> (Vec<String>, Vec<String>) {
    let args = args.trim();
    let Some(slots) = parse_slots(args) else {
        return (vec![format!("animate({args})")], Vec::new());
    };
    let mut xf = Vec::new();
    let mut rest = Vec::new();
    let mut property: Option<String> = None;
    let mut duration: Option<String> = None;
    let mut ease: Option<String> = None;
    for (k, v) in slots {
        match k.as_str() {
            "x" => xf.push(format!("translateX({v})")),
            "y" => xf.push(format!("translateY({v})")),
            "rotate" => xf.push(format!("rotate({v})")),
            "scale" => xf.push(format!("scale({v})")),
            "origin" => xf.push(format!("origin({v})")),
            "transform" => xf.push(format!("transform({v})")),
            "property" => property = Some(v),
            "duration" => duration = Some(v),
            "ease" => ease = Some(v),
            "animate" => xf.push(format!("animate({v})")),
            other => rest.push((other.to_string(), v)),
        }
    }
    let mut plays = Vec::new();
    if let Some(prop) = property {
        // Known play prop-set: `colors`. Other property values become
        // transition/duration/ease plains — do not emit dead `motion()`.
        if prop == "colors" {
            if let Some(dur) = duration {
                plays.push(format!("play({prop}, {dur})"));
            } else {
                plays.push(format!("play({prop})"));
            }
            let _ = ease;
        } else {
            xf.push(format!("transition({prop})"));
            if let Some(d) = duration {
                xf.push(format!("duration({d})"));
            }
            if let Some(e) = ease {
                xf.push(format!("ease({e})"));
            }
        }
    } else {
        if let Some(d) = duration {
            xf.push(format!("duration({d})"));
        }
        if let Some(e) = ease {
            xf.push(format!("ease({e})"));
        }
    }
    for (k, v) in rest {
        xf.push(format!("{k}({v})"));
    }
    (xf, plays)
}

fn parse_slots(args: &str) -> Option<Vec<(String, String)>> {
    let s = args.trim();
    if !s.starts_with('{') || !s.ends_with('}') {
        return None;
    }
    let inner = &s[1..s.len() - 1];
    let bytes = inner.as_bytes();
    let mut slots = Vec::new();
    let mut i = 0usize;
    while i < bytes.len() {
        if let Some(e) = skip_comment(inner, i) {
            i = e;
            continue;
        }
        if bytes[i].is_ascii_whitespace() || bytes[i] == b',' {
            i += 1;
            continue;
        }
        let key_start = i;
        if !scan_ident(inner, &mut i) {
            return None;
        }
        let key = inner[key_start..i].to_string();
        skip_ws(inner, &mut i);
        if i >= bytes.len() || bytes[i] != b':' {
            return None;
        }
        i += 1;
        skip_ws(inner, &mut i);
        let val_end = scan_arg_end(inner, i);
        let val = inner[i..val_end].trim().to_string();
        if val.is_empty() {
            return None;
        }
        slots.push((key, val));
        i = val_end;
    }
    Some(slots)
}

fn args_spaced(args: &str) -> bool {
    let s = args.trim();
    s.starts_with("{ ") || s.contains(": ")
}

fn format_slots(slots: &[(String, String)], spaced: bool) -> String {
    let colon = if spaced { ": " } else { ":" };
    let comma = if spaced { ", " } else { ", " };
    let body = slots
        .iter()
        .map(|(k, v)| format!("{k}{colon}{v}"))
        .collect::<Vec<_>>()
        .join(comma);
    if spaced {
        format!("{{ {body} }}")
    } else {
        format!("{{{body}}}")
    }
}

fn preceded_by_colon(src: &str, i: usize) -> bool {
    let bytes = src.as_bytes();
    let mut k = i;
    while k > 0 && bytes[k - 1].is_ascii_whitespace() {
        k -= 1;
    }
    k > 0 && bytes[k - 1] == b':'
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
            pos = name_start;
            break;
        }
        let args_end = scan_balanced(src, pos);
        if args_end <= pos + 1 {
            break;
        }
        let args_inner = src[pos + 1..args_end - 1].to_string();
        calls.push(Call { name, args_inner });
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
        rewrite_v3_source(src)
    }

    #[test]
    fn job_example_unwraps_channels_and_renames() {
        let src = "{ box: space({ pad-x: 3 }).place({ z: sticky }), paint: fill(surface), when: { hover: { paint: fill({ color: muted }) } } }";
        let got = rw(src);
        assert_eq!(
            got,
            "{ pad({ x: 3 }).position({ z: sticky }).bg(surface), when: { hover: bg(muted) } }"
        );
    }

    #[test]
    fn fill_scalar_and_color_slot() {
        assert_eq!(
            rw("style({ paint: fill(surface) })"),
            "style({ bg(surface) })"
        );
        assert_eq!(
            rw("style({ paint: fill({ color: muted }) })"),
            "style({ bg(muted) })"
        );
        assert_eq!(
            rw("style({ paint: fill({ color: tone, mix: 15 }) })"),
            "style({ bg({ color: tone, mix: 15 }) })"
        );
    }

    #[test]
    fn ink_becomes_fg() {
        assert_eq!(
            rw("style({ paint: ink({ color: primary-foreground }) })"),
            "style({ fg(primary-foreground) })"
        );
        assert_eq!(rw("style({ paint: ink(card) })"), "style({ fg(card) })");
    }

    #[test]
    fn place_remaps_slots() {
        assert_eq!(
            rw("style({ box: place({ z: sticky }) })"),
            "style({ position({ z: sticky }) })"
        );
        assert_eq!(
            rw("style({ box: place({ position: absolute, top: 0, left: 0 }) })"),
            "style({ position({ scheme: absolute, t: 0, l: 0 }) })"
        );
        assert_eq!(
            rw("style({ box: place({ position: relative }) })"),
            "style({ position(relative) })"
        );
    }

    #[test]
    fn space_splits_pad_and_margin() {
        assert_eq!(
            rw("style({ box: space({ pad-x: 3 }) })"),
            "style({ pad({ x: 3 }) })"
        );
        assert_eq!(rw("style({ box: space({ pad: 4 }) })"), "style({ pad(4) })");
        assert_eq!(
            rw("style({ box: space({ mx: auto }) })"),
            "style({ margin({ x: auto }) })"
        );
        assert_eq!(
            rw("style({ box: space({ pad-x: 3, mx: auto }) })"),
            "style({ pad({ x: 3 }).margin({ x: auto }) })"
        );
        assert_eq!(
            rw("style({ box: space({ pad: 0, pad-x: 0 }) })"),
            "style({ pad({ all: 0, x: 0 }) })"
        );
    }

    #[test]
    fn flow_becomes_plain_setters() {
        assert_eq!(
            rw("style({ box: flow({ display: none }) })"),
            "style({ display(none) })"
        );
        assert_eq!(
            rw("style({ box: flow({ display: flex, grow: 1, dir: column }) })"),
            "style({ display(flex).grow(1).dir(column) })"
        );
    }

    #[test]
    fn when_bodies_are_bare_chains() {
        assert_eq!(
            rw("style({ when: { hover: { paint: fill({ color: muted }) } } })"),
            "style({ when: { hover: bg(muted) } })"
        );
        assert_eq!(
            rw(
                "style({ when: { hover: { paint: fill({ color: hover-bg }) }, current: { paint: ink({ color: primary-foreground }) } } })"
            ),
            "style({ when: { hover: bg(hover-bg), current: fg(primary-foreground) } })"
        );
    }

    #[test]
    fn motion_transforms_and_play() {
        assert_eq!(
            rw("style({ paint: motion({ y: bind(rowY) }) })"),
            "style({ translateY(bind(rowY)) })"
        );
        assert_eq!(
            rw("style({ paint: motion({ rotate: 90 }) })"),
            "style({ rotate(90) })"
        );
        assert_eq!(
            rw(
                "style({ paint: fill(card).motion({ property: colors, duration: fast, ease: out }) })"
            ),
            "style({ bg(card), animate: { transition: play(colors, fast) } })"
        );
    }

    #[test]
    fn declared_style_const() {
        let src = "style navLink = { box: space({ pad-x: 3, pad-y: 2 }), paint: radius(md), text: type(label), when: { hover: { paint: fill({ color: surface-2 }) } } }";
        assert_eq!(
            rw(src),
            "style navLink = { pad({ x: 3, y: 2 }).radius(md).type(label), when: { hover: bg(surface-2) } }"
        );
    }

    #[test]
    fn template_text_is_not_rewritten() {
        let src = "<p>Sticky top/left inset (number). space({ pad-x: 3 }) fill(surface)</p>";
        assert_eq!(rw(src), src);
        let attr = "<div style({ paint: fill({ color: muted }) })>fill(surface) in prose</div>";
        assert_eq!(
            rw(attr),
            "<div style({ bg(muted) })>fill(surface) in prose</div>"
        );
    }

    #[test]
    fn controller_and_comments_are_not_rewritten() {
        let src = "<controller>\nconst s = fill({ color: muted });\n</controller>";
        assert_eq!(rw(src), src);
        let cmt = "// fill({ color: primary })\nstyle({ paint: radius(md) })";
        assert_eq!(
            rw(cmt),
            "// fill({ color: primary })\nstyle({ radius(md) })"
        );
    }

    #[test]
    fn match_text_arm_is_not_unwrapped() {
        let src = "style(base, {match: {shape: {text: shapeBar, line: shapeBar}}})";
        assert_eq!(rw(src), src);
    }

    #[test]
    fn compound_predicate_fill_stays() {
        let src = "style(base, {compound: [{fill: true, paint: fill(muted)}]})";
        assert_eq!(
            rw(src),
            "style(base, {compound: [{fill: true, bg(muted)}]})"
        );
    }

    #[test]
    fn comparison_gt_on_same_tag_still_folds() {
        let spaced = "<div hidden={n > 0} style({ paint: fill({ color: muted }) })></div>";
        assert_eq!(
            rw(spaced),
            "<div hidden={n > 0} style({ bg(muted) })></div>"
        );
        let tight = "<div hidden={n>0} style({ box: space({ pad-x: 2 }) })></div>";
        assert_eq!(
            rw(tight),
            "<div hidden={n>0} style({ pad({ x: 2 }) })></div>"
        );
    }

    #[test]
    fn rewrite_is_idempotent() {
        let src = "style({ box: space({ pad-x: 3 }).place({ z: sticky }), paint: fill(surface), when: { hover: { paint: fill({ color: muted }) } } })";
        let once = rw(src);
        assert_eq!(rw(&once), once);
    }

    #[test]
    fn compact_stdlib_row() {
        let src = "style base = {box: flow({ display: inline-block }).size({ w: full, h: 2 }), paint: fill({ color: muted }).ink({ color: primary })}";
        assert_eq!(
            rw(src),
            "style base = {display(inline-block).size({ w: full, h: 2 }).bg(muted).fg(primary)}"
        );
    }

    #[test]
    fn already_v3_is_identity() {
        let src =
            "{ pad({ x: 3 }).position({ z: sticky }).bg(surface), when: { hover: bg(muted) } }";
        assert_eq!(rw(src), src);
    }

    #[test]
    fn scope_declarations_fold() {
        let src = "<scope>\n  style foo = { paint: fill({ color: muted }) }\n</scope>";
        assert_eq!(rw(src), "<scope>\n  style foo = { bg(muted) }\n</scope>");
    }

    /// Walk first-party `.sail` trees. `SAIL_REWRITE_V3_WRITE=1` writes.
    #[test]
    fn apply_first_party_fold() {
        let write = std::env::var("SAIL_REWRITE_V3_WRITE").ok().as_deref() == Some("1");
        let dry = std::env::var("SAIL_REWRITE_V3_DRY").ok().as_deref() == Some("1");
        if !write && !dry {
            return;
        }
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .expect("workspace root")
            .to_path_buf();
        let trees = [
            root.join("crates/sail/compiler/stdlib"),
            root.join("apps/sail-site"),
            root.join("apps/studio"),
            root.join("packages"),
        ];
        let mut files = Vec::new();
        for tree in &trees {
            collect_sail(tree, &mut files);
        }
        files.sort();
        let mut changed = 0usize;
        for path in &files {
            let src =
                std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {path:?}: {e}"));
            let next = rewrite_v3_source(&src);
            if next == src {
                continue;
            }
            changed += 1;
            eprintln!("{}", path.strip_prefix(&root).unwrap_or(path).display());
            if write {
                std::fs::write(path, next).unwrap_or_else(|e| panic!("write {path:?}: {e}"));
            }
        }
        eprintln!(
            "rewrite_v3: {changed} of {} files {}",
            files.len(),
            if write { "written" } else { "would change" }
        );
    }

    fn collect_sail(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        let Ok(rd) = std::fs::read_dir(dir) else {
            return;
        };
        for ent in rd.flatten() {
            let p = ent.path();
            let name = ent.file_name();
            if name == "node_modules" || name == "target" || name == ".git" {
                continue;
            }
            if p.is_dir() {
                collect_sail(&p, out);
            } else if p.extension().and_then(|e| e.to_str()) == Some("sail") {
                out.push(p);
            }
        }
    }
}
