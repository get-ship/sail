//! Shared helpers for BUCKETS §4 style convictions (application-default,
//! const-eq-name, dead-const). Not a lint rule.

use std::collections::HashSet;

use spinnaker::concerns::{find_concern, resolve_concern_name};
use spinnaker::ir::{PaintArg, PaintOp, Style};
use spinnaker::parse::{parse, parse_style_const_rhs};
use spinnaker::theme_env::ThemeStyleEnv;

use super::layout::OpenTag;

pub struct StyleConstDecl {
    pub name: String,
    pub rhs: String,
    pub name_offset: usize,
}

pub fn harvest_style_consts(source: &str) -> Vec<StyleConstDecl> {
    let mut out = Vec::new();
    for (body, body_start) in scope_bodies(source) {
        for mut decl in harvest_style_consts_in(body) {
            decl.name_offset += body_start;
            out.push(decl);
        }
    }
    out
}

fn scope_bodies(source: &str) -> Vec<(&str, usize)> {
    let bytes = source.as_bytes();
    let mut out = Vec::new();
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
        if source[i..].starts_with("<scope") {
            let after = i + "<scope".len();
            let next = bytes.get(after).copied();
            if matches!(
                next,
                Some(b'>') | Some(b' ') | Some(b'\n') | Some(b'\t') | Some(b'/')
            ) {
                if let Some(open_end) = source[i..].find('>') {
                    let body_start = i + open_end + 1;
                    if let Some(rel) = source[body_start..].find("</scope>") {
                        out.push((&source[body_start..body_start + rel], body_start));
                        i = body_start + rel + "</scope>".len();
                        continue;
                    }
                }
            }
        }
        i += utf8_step(source, i);
    }
    out
}

fn harvest_style_consts_in(source: &str) -> Vec<StyleConstDecl> {
    let bytes = source.as_bytes();
    let mut out = Vec::new();
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
        if is_style_kw(source, i) {
            let after = i + "style".len();
            let mut j = skip_ws(bytes, after);
            if j < bytes.len() && bytes[j] == b'(' {
                i = after;
                continue;
            }
            let name_start = j;
            while j < bytes.len()
                && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_' || bytes[j] == b'$')
            {
                j += 1;
            }
            if j > name_start {
                let name = source[name_start..j].to_string();
                j = skip_ws(bytes, j);
                if bytes.get(j) == Some(&b'=') {
                    j += 1;
                    j = skip_ws(bytes, j);
                    let rhs_end = scan_style_rhs_end(source, j);
                    let rhs = source[j..rhs_end].trim().to_string();
                    out.push(StyleConstDecl {
                        name,
                        rhs,
                        name_offset: i,
                    });
                    i = rhs_end;
                    continue;
                }
            }
        }
        i += utf8_step(source, i);
    }
    out
}

pub fn parse_const_rhs(rhs: &str) -> Style {
    parse_style_const_rhs(rhs).0
}

pub fn parse_tag_style(sx: &str) -> Style {
    parse(sx).0
}

pub fn tag_style(tag: &OpenTag) -> Option<&str> {
    tag.attrs
        .iter()
        .find(|(n, _)| n == "style")
        .map(|(_, v)| v.as_str())
}

pub fn authored_style_inner(sx: &str) -> String {
    let s = sx.trim();
    let s = s.strip_prefix("style").unwrap_or(s).trim();
    let inner = s
        .strip_prefix('(')
        .and_then(|t| t.strip_suffix(')'))
        .unwrap_or(s)
        .trim();
    inner
        .strip_prefix('{')
        .and_then(|t| t.strip_suffix('}'))
        .map(str::trim)
        .unwrap_or(inner)
        .to_string()
}

pub fn restates_tag_default(tag: &str, style: &Style) -> bool {
    let Some(def) = ThemeStyleEnv::from_registry().resolve_tag_default(tag) else {
        return false;
    };
    let spend = type_spend(style);
    if spend.extra || (spend.role.is_none() && spend.size.is_none()) {
        return false;
    }
    if let Some(role) = &spend.role
        && role != &def.type_role
    {
        return false;
    }
    if let Some(size) = &spend.size
        && def.size.as_deref() != Some(size.as_str())
    {
        return false;
    }
    true
}

/// Exact alias of a theme / tagDefault / preset / type-role name.
/// Returns the author-facing `already names \`…\`` payload.
pub fn exact_alias_meaning(name: &str, style: &Style) -> Option<String> {
    if is_type_role(name) {
        let spend = type_spend(style);
        if !spend.extra && spend.size.is_none() && spend.role.as_deref() == Some(name) {
            return Some(format!("type({name})"));
        }
        return None;
    }
    let env = ThemeStyleEnv::from_registry();
    if env.resolve_named_style(name).is_some() {
        let expected = env.named_style_to_ir(&env.resolve_named_style(name)?);
        if styles_match(&expected, style) {
            return Some(name.to_string());
        }
        return None;
    }
    if let Some(def) = env.resolve_tag_default(name) {
        let spend = type_spend(style);
        if spend.extra {
            return None;
        }
        if spend.role.as_deref() != Some(def.type_role.as_str()) {
            return None;
        }
        match &def.size {
            Some(step) if spend.size.as_deref() == Some(step.as_str()) => {
                return Some(format!("type({})", def.type_role));
            }
            None if spend.size.is_none() => {
                return Some(format!("type({})", def.type_role));
            }
            _ => return None,
        }
    }
    if env.resolve_play(name).is_some() {
        let spend = type_spend(style);
        if spend.extra {
            return None;
        }
        // Play alias is a single play/animate token matching the name.
        if play_token(style).as_deref() == Some(name) {
            return Some(format!("play({name})"));
        }
    }
    None
}

pub fn collect_style_refs(style: &Style, out: &mut HashSet<String>) {
    match style {
        Style::Ref(name) => {
            out.insert(name.clone());
        }
        Style::Chain(items) | Style::List(items) => {
            for item in items {
                collect_style_refs(item, out);
            }
        }
        Style::Match { arms, .. } => {
            for (_, body) in arms {
                collect_style_refs(body, out);
            }
        }
        Style::Compound { body, .. } => collect_style_refs(body, out),
        Style::When { entries } => {
            for (_, body) in entries {
                collect_style_refs(body, out);
            }
        }
        Style::Empty | Style::Set(_) | Style::MotionAttach { .. } => {}
    }
}

pub fn spent_style_names(source: &str, decls: &[StyleConstDecl]) -> HashSet<String> {
    let mut spent = HashSet::new();
    for tag in super::layout::scan_any_open_tags(source) {
        if let Some(sx) = tag_style(&tag) {
            collect_style_refs(&parse_tag_style(sx), &mut spent);
        }
    }
    for decl in decls {
        collect_style_refs(&parse_const_rhs(&decl.rhs), &mut spent);
    }
    spent
}

struct TypeSpend {
    role: Option<String>,
    size: Option<String>,
    extra: bool,
}

fn type_spend(style: &Style) -> TypeSpend {
    let mut spend = TypeSpend {
        role: None,
        size: None,
        extra: false,
    };
    walk_type(style, &mut spend);
    spend
}

fn walk_type(style: &Style, spend: &mut TypeSpend) {
    match style {
        Style::Empty => {}
        Style::Set(op) => apply_type_op(op, spend),
        Style::Chain(items) | Style::List(items) => {
            for item in items {
                walk_type(item, spend);
            }
        }
        Style::Ref(_)
        | Style::When { .. }
        | Style::Match { .. }
        | Style::MotionAttach { .. }
        | Style::Compound { .. } => {
            spend.extra = true;
        }
    }
}

fn apply_type_op(op: &PaintOp, spend: &mut TypeSpend) {
    if op.setter != "type" {
        spend.extra = true;
        return;
    }
    match op.first_arg() {
        PaintArg::Token(name) if !name.is_empty() => {
            if spend.role.as_deref().is_some_and(|r| r != name) {
                spend.extra = true;
            }
            spend.role = Some(name.clone());
        }
        PaintArg::Slots(slots) => {
            for (k, v) in slots {
                match k.as_str() {
                    "size" => match v {
                        PaintArg::Token(step) if !step.is_empty() => {
                            spend.size = Some(step.clone());
                        }
                        _ => spend.extra = true,
                    },
                    _ => spend.extra = true,
                }
            }
        }
        _ => spend.extra = true,
    }
}

fn is_type_role(name: &str) -> bool {
    let Some(spec) = find_concern("type") else {
        return false;
    };
    resolve_concern_name(spec, name).is_some()
}

fn styles_match(expected: &Style, got: &Style) -> bool {
    flatten_ops(expected) == flatten_ops(got)
}

fn flatten_ops(style: &Style) -> Option<Vec<(String, String)>> {
    let mut ops = Vec::new();
    let mut extra = false;
    flatten_ops_inner(style, &mut ops, &mut extra);
    if extra { None } else { Some(ops) }
}

fn flatten_ops_inner(style: &Style, ops: &mut Vec<(String, String)>, extra: &mut bool) {
    match style {
        Style::Empty => {}
        Style::Set(op) => ops.push((op.setter.clone(), op.first_arg().display())),
        Style::Chain(items) | Style::List(items) => {
            for item in items {
                flatten_ops_inner(item, ops, extra);
            }
        }
        _ => *extra = true,
    }
}

fn play_token(style: &Style) -> Option<String> {
    match style {
        Style::Set(op) if matches!(op.setter.as_str(), "play" | "animate") => {
            match op.first_arg() {
                PaintArg::Token(name) if !name.is_empty() => Some(name.clone()),
                _ => None,
            }
        }
        Style::Chain(items) | Style::List(items) if items.len() == 1 => play_token(&items[0]),
        _ => None,
    }
}

fn is_style_kw(source: &str, i: usize) -> bool {
    let bytes = source.as_bytes();
    if !source[i..].starts_with("style") {
        return false;
    }
    if i > 0 && (bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'_') {
        return false;
    }
    let after = i + "style".len();
    after >= bytes.len() || (!bytes[after].is_ascii_alphanumeric() && bytes[after] != b'_')
}

fn skip_ws(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    i
}

fn scan_style_rhs_end(source: &str, start: usize) -> usize {
    let bytes = source.as_bytes();
    let mut i = start;
    let mut depth = 0i32;
    while i < bytes.len() {
        if source[i..].starts_with("<!--") {
            break;
        }
        match bytes[i] {
            b'{' | b'(' | b'[' => depth += 1,
            b'}' | b')' | b']' => {
                if depth == 0 {
                    return i;
                }
                depth -= 1;
                if depth == 0 {
                    return i + 1;
                }
            }
            b'\n' | b';' if depth == 0 => return i,
            _ => {}
        }
        i += 1;
    }
    bytes.len()
}

fn utf8_step(source: &str, i: usize) -> usize {
    source[i..].chars().next().map(char::len_utf8).unwrap_or(1)
}
