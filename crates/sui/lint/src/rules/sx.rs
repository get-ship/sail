//! `sui-sx` — the token-only inline style attribute.
//!
//! `sx="p:4 gap:3 bg:surface-2 radius:lg"` is a space-separated list of
//! `prop:token` pairs drawn from a CLOSED vocabulary. An unknown property, an
//! off-scale spacing, a raw px/hex, or an unknown color/radius/text token is
//! `Severity::Error` — which `sui-compiler::check::css_token` promotes to a
//! build failure, and the LSP surfaces identically. There is no bracket escape
//! (cf. Tailwind `p-[13px]`): off-token does not compile.
//!
//! Only static `sx="…"` literals are validated; dynamic `sx={…}` and
//! attributes inside HTML comments are skipped. The generated CSS is emitted
//! by `sui-compiler::emit::styles`; this rule owns validation only.

use std::path::Path;

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sui-sx";

#[derive(Clone, Copy)]
enum Kind {
    Space,
    Color,
    Radius,
    Text,
    Align,
    Justify,
    Dir,
}

fn prop_kind(prop: &str) -> Option<Kind> {
    use Kind::*;
    Some(match prop {
        "p" | "px" | "py" | "pt" | "pr" | "pb" | "pl" | "m" | "mt" | "mr" | "mb" | "ml" | "gap" => Space,
        "bg" | "fg" => Color,
        "radius" => Radius,
        "text" => Text,
        "align" => Align,
        "justify" => Justify,
        "dir" => Dir,
        _ => return None,
    })
}

const SPACE: &[&str] = &["0", "1", "2", "3", "4", "5", "6", "7", "8"];
const RADIUS: &[&str] = &["none", "xs", "sm", "md", "lg", "xl", "full"];
const TEXT: &[&str] = &["xs", "sm", "md", "lg", "xl", "2xl"];
const COLOR: &[&str] = &[
    "surface-0", "surface-1", "surface-2", "surface-3", "fg", "muted", "border",
    "primary", "success", "warning", "destructive", "accent", "info",
];
const ALIGN: &[&str] = &["start", "center", "end", "stretch", "baseline"];
const JUSTIFY: &[&str] = &["start", "center", "end", "between", "around"];
const DIR: &[&str] = &["row", "col"];
const ALL_PROPS: &[&str] = &[
    "p", "px", "py", "pt", "pr", "pb", "pl", "m", "mt", "mr", "mb", "ml", "gap",
    "bg", "fg", "radius", "text", "align", "justify", "dir",
];

fn allowed(kind: Kind) -> &'static [&'static str] {
    match kind {
        Kind::Space => SPACE,
        Kind::Color => COLOR,
        Kind::Radius => RADIUS,
        Kind::Text => TEXT,
        Kind::Align => ALIGN,
        Kind::Justify => JUSTIFY,
        Kind::Dir => DIR,
    }
}

/// The CSS property/properties a `prop` shorthand maps to.
fn css_props(prop: &str) -> &'static [&'static str] {
    match prop {
        "p" => &["padding"],
        "px" => &["padding-inline"],
        "py" => &["padding-block"],
        "pt" => &["padding-top"],
        "pr" => &["padding-right"],
        "pb" => &["padding-bottom"],
        "pl" => &["padding-left"],
        "m" => &["margin"],
        "mt" => &["margin-top"],
        "mr" => &["margin-right"],
        "mb" => &["margin-bottom"],
        "ml" => &["margin-left"],
        "gap" => &["gap"],
        "bg" => &["background"],
        "fg" => &["color"],
        "radius" => &["border-radius"],
        "text" => &["font-size"],
        "align" => &["align-items"],
        "justify" => &["justify-content"],
        _ => &[],
    }
}

fn color_var(token: &str) -> &'static str {
    match token {
        "surface-0" => "--surface-0",
        "surface-1" => "--surface-1",
        "surface-2" => "--surface-2",
        "surface-3" => "--surface-3",
        "fg" => "--foreground",
        "muted" => "--muted-foreground",
        "border" => "--border",
        "primary" => "--primary",
        "success" => "--success",
        "warning" => "--warning",
        "destructive" => "--destructive",
        "accent" => "--accent",
        "info" => "--info",
        _ => "--foreground",
    }
}

fn flex_val(token: &str) -> &'static str {
    match token {
        "start" => "flex-start",
        "end" => "flex-end",
        "between" => "space-between",
        "around" => "space-around",
        "center" => "center",
        "stretch" => "stretch",
        "baseline" => "baseline",
        _ => "flex-start",
    }
}

/// Lower a VALID `prop:token` to its CSS declarations, or `None` if the pair is
/// off-vocabulary. Single source of truth for the sx → CSS mapping; the
/// compiler's `preprocess` generates atomic CSS from this, the `check` above
/// validates against the same vocabulary. Kept in sync by construction.
pub fn lower(prop: &str, token: &str) -> Option<Vec<(&'static str, String)>> {
    let kind = prop_kind(prop)?;
    if !allowed(kind).contains(&token) {
        return None;
    }
    let decls = match kind {
        Kind::Space => css_props(prop).iter().map(|p| (*p, format!("var(--space-{token})"))).collect(),
        Kind::Color => css_props(prop).iter().map(|p| (*p, format!("var({})", color_var(token)))).collect(),
        Kind::Radius => {
            let v = if token == "none" { "0".to_string() } else { format!("var(--radius-{token})") };
            css_props(prop).iter().map(|p| (*p, v.clone())).collect()
        }
        Kind::Text => css_props(prop).iter().map(|p| (*p, format!("var(--text-{token})"))).collect(),
        Kind::Align => vec![("align-items", flex_val(token).to_string())],
        Kind::Justify => vec![("justify-content", flex_val(token).to_string())],
        Kind::Dir => vec![
            ("display", "flex".to_string()),
            ("flex-direction", if token == "col" { "column" } else { "row" }.to_string()),
        ],
    };
    Some(decls)
}

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (val, offset) in scan_sx(source) {
        let (line, col) = offset_to_line_col(source, offset);
        for pair in val.split_whitespace() {
            let Some((prop, token)) = pair.split_once(':') else {
                findings.push(mk(path, line, col, format!("sx `{pair}` — expected `prop:token`")));
                continue;
            };
            let Some(kind) = prop_kind(prop) else {
                let mut m = format!("sx unknown property `{prop}:` — not in the styling vocabulary");
                if let Some(s) = nearest(prop, ALL_PROPS) {
                    m.push_str(&format!("; did you mean `{s}:`?"));
                }
                findings.push(mk(path, line, col, m));
                continue;
            };
            let opts = allowed(kind);
            if !opts.contains(&token) {
                let raw = token.chars().next().is_some_and(|c| c.is_ascii_digit() || c == '#')
                    || token.ends_with("px")
                    || token.ends_with("rem")
                    || token.ends_with('%');
                let mut m = format!("sx `{prop}:{token}` — `{token}` is not a token");
                if raw {
                    m.push_str(" (raw values are not allowed; spend a token)");
                }
                m.push_str(&format!(". allowed: {}", opts.join(", ")));
                if let Some(s) = nearest(token, opts) {
                    m.push_str(&format!("; did you mean `{prop}:{s}`?"));
                }
                findings.push(mk(path, line, col, m));
            }
        }
    }
    findings
}

fn mk(path: &Path, line: usize, col: usize, message: String) -> Finding {
    Finding {
        file: path.to_path_buf(),
        line,
        col,
        rule: RULE_ID,
        severity: Severity::Error,
        message,
        suggest: Some(
            "Use a design-system token (spacing 0–8, the surface/tone colors, \
             radius none–full, text xs–2xl). Raw px/hex are intentionally not \
             expressible in sx — that is the point."
                .to_string(),
        ),
    }
}

/// Find every static `sx="…"` value + the byte offset of its value, skipping
/// HTML comments and dynamic `sx={…}`.
fn scan_sx(source: &str) -> Vec<(&str, usize)> {
    let bytes = source.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i..].starts_with(b"<!--") {
            match source[i..].find("-->") {
                Some(end) => {
                    i += end + 3;
                    continue;
                }
                None => break,
            }
        }
        if i > 0
            && bytes[i - 1].is_ascii_whitespace()
            && bytes[i..].starts_with(b"sx")
            && is_eq_quote(&bytes[i + 2..])
        {
            let mut j = i + 2;
            while j < bytes.len() && bytes[j] != b'"' && bytes[j] != b'{' {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b'"' {
                let start = j + 1;
                if let Some(rel) = source[start..].find('"') {
                    out.push((&source[start..start + rel], start));
                    i = start + rel + 1;
                    continue;
                }
            }
        }
        i += 1;
    }
    out
}

/// After the `sx` name: optional ws, `=`, optional ws, then a `"` (a literal).
fn is_eq_quote(rest: &[u8]) -> bool {
    let mut k = 0;
    while k < rest.len() && rest[k].is_ascii_whitespace() {
        k += 1;
    }
    if k >= rest.len() || rest[k] != b'=' {
        return false;
    }
    k += 1;
    while k < rest.len() && rest[k].is_ascii_whitespace() {
        k += 1;
    }
    k < rest.len() && rest[k] == b'"'
}

fn offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 1usize;
    let mut col = 1usize;
    for (i, b) in source.bytes().enumerate() {
        if i == offset {
            break;
        }
        if b == b'\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

fn nearest(word: &str, opts: &[&str]) -> Option<String> {
    opts.iter()
        .map(|o| (lev(word, o), o.to_string()))
        .filter(|(d, _)| *d <= 3)
        .min_by_key(|(d, _)| *d)
        .map(|(_, o)| o)
}

fn lev(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut d: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.iter().enumerate() {
        let mut prev = d[0];
        d[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let tmp = d[j + 1];
            d[j + 1] = if ca == cb { prev } else { 1 + prev.min(d[j]).min(d[j + 1]) };
            prev = tmp;
        }
    }
    d[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn run(src: &str) -> Vec<String> {
        check(Path::new("t.sui"), src).into_iter().map(|f| f.message).collect()
    }

    #[test]
    fn accepts_tokens() {
        assert!(run(r#"<div sx="p:4 gap:3 bg:surface-2 radius:lg dir:col">x</div>"#).is_empty());
    }

    #[test]
    fn rejects_raw_and_unknown() {
        assert_eq!(run(r#"<div sx="p:13px">x</div>"#).len(), 1);
        assert_eq!(run(r#"<div sx="bg:#0af">x</div>"#).len(), 1);
        assert_eq!(run(r#"<div sx="gap:12">x</div>"#).len(), 1);
        assert_eq!(run(r#"<div sx="shadow:4">x</div>"#).len(), 1);
    }

    #[test]
    fn skips_comments_and_dynamic() {
        assert!(run(r#"<!-- <div sx="p:99"> --> <div sx={dynamic}>x</div>"#).is_empty());
    }
}
