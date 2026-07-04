//! `sui-sx` — the token-only inline **surface** style attribute.
//!
//! `sx="p:4 bg:surface-2 radius:lg text:sm"` is a space-separated list of
//! `prop:token` pairs from a CLOSED, token-only vocabulary. Off-token (raw
//! px/hex, off-scale, unknown prop) is `Severity::Error` → promoted to a
//! `CompileError` by `check::css_token`, and surfaced identically in the LSP.
//! No bracket escape (cf. Tailwind `p-[13px]`).
//!
//! SCOPE — surface only. `sx` deliberately does NOT own layout: `dir`, `gap`,
//! `align`, `justify` belong to the layout primitives (`<sui-stack>`,
//! `<sui-cluster>`, `<sui-grid>`), which carry the semantics/a11y/overflow
//! handling a hand-rolled flex box loses. Keeping layout out of `sx` (plus
//! `sui-no-raw-container` banning raw `<div>`) forces structure through the
//! primitives; `sx` is only the surface treatment a `<style>` block used to do.
//!
//! Only static `sx="…"` literals are validated; dynamic `sx={…}` and
//! attributes inside HTML comments are skipped.

use std::path::Path;

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sui-sx";

#[derive(Clone, Copy)]
enum Kind {
    Space,
    Color,
    Radius,
    Text,
}

fn prop_kind(prop: &str) -> Option<Kind> {
    use Kind::*;
    Some(match prop {
        // element-level spacing (padding = a surface inset, margin = a nudge).
        // NOTE: `gap` is intentionally absent — sibling spacing is the layout
        // primitive's job (`<sui-stack gap="3">`), not a per-element style.
        "p" | "px" | "py" | "pt" | "pr" | "pb" | "pl" | "m" | "mt" | "mr" | "mb" | "ml" => Space,
        "bg" | "fg" => Color,
        "radius" => Radius,
        "text" => Text,
        _ => return None,
    })
}

// SCALES are NOT hand-maintained here — they are derived from the shared token
// registry (`crate::tokens`), so sx can never drift from `tokens.css` (the
// drift that wrongly rejected `text:3xl`). Only COLOR keeps a curated alias set
// (short ergonomic names like `fg`/`muted` that map to token names below).
const COLOR: &[&str] = &[
    "surface-0", "surface-1", "surface-2", "surface-3", "fg", "muted", "border",
    "primary", "success", "warning", "destructive", "accent", "info",
];
const ALL_PROPS: &[&str] = &[
    "p", "px", "py", "pt", "pr", "pb", "pl", "m", "mt", "mr", "mb", "ml",
    "bg", "fg", "radius", "text",
];

fn allowed(kind: Kind) -> Vec<&'static str> {
    match kind {
        Kind::Space => crate::tokens::space_values().to_vec(),
        Kind::Color => COLOR.to_vec(),
        Kind::Radius => crate::tokens::radius_values().to_vec(),
        Kind::Text => crate::tokens::text_values().to_vec(),
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
        "bg" => &["background"],
        "fg" => &["color"],
        "radius" => &["border-radius"],
        "text" => &["font-size"],
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

/// Lower a VALID `prop:token` to its CSS declarations, or `None` if off-vocab.
/// Single source of truth for the sx → CSS mapping; the compiler's `preprocess`
/// generates atomic CSS from this, the `check` below validates the same set.
pub fn lower(prop: &str, token: &str) -> Option<Vec<(&'static str, String)>> {
    let kind = prop_kind(prop)?;
    if !allowed(kind).contains(&token) {
        return None;
    }
    let decls = match kind {
        Kind::Space => {
            // `0` is the literal zero (no `--space-0` token); everything else is
            // a `--space-N` token derived from the registry.
            let v = if token == "0" { "0".to_string() } else { format!("var(--space-{token})") };
            css_props(prop).iter().map(|p| (*p, v.clone())).collect()
        }
        Kind::Color => css_props(prop).iter().map(|p| (*p, format!("var({})", color_var(token)))).collect(),
        Kind::Radius => {
            let v = if token == "none" { "0".to_string() } else { format!("var(--radius-{token})") };
            css_props(prop).iter().map(|p| (*p, v.clone())).collect()
        }
        Kind::Text => css_props(prop).iter().map(|p| (*p, format!("var(--text-{token})"))).collect(),
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
                let mut m = format!("sx unknown property `{prop}:` — not in the surface vocabulary");
                // steer layout attempts to the primitives, not a bigger sx
                if matches!(prop, "dir" | "gap" | "align" | "justify" | "row" | "col") {
                    m.push_str("; layout belongs to the primitives — use <sui-stack>/<sui-cluster>/<sui-grid>, not sx");
                } else if let Some(s) = nearest(prop, ALL_PROPS) {
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
                if let Some(s) = nearest(token, &opts) {
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
            "sx is surface-only: spacing 0–8 (p/m…), colors (bg/fg), radius, text. \
             Layout is the primitives' job (<sui-stack>/<sui-cluster>/<sui-grid>). \
             Raw px/hex are not expressible."
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
        if source[i..].starts_with("<!--") {
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
            && source[i..].starts_with("sx")
            && is_eq_quote(&bytes[i + 2..])
        {
            let mut j = i + 2;
            while j < bytes.len() && bytes[j] != b'"' && bytes[j] != b'{' {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b'"' {
                let vstart = j + 1;
                if let Some(rel) = source[vstart..].find('"') {
                    out.push((&source[vstart..vstart + rel], vstart));
                    i = vstart + rel + 1;
                    continue;
                }
            }
        }
        // Advance one full UTF-8 char so `source[i..]` stays on a boundary — a
        // bare `i += 1` panics mid-codepoint on any multi-byte char (…, emoji).
        i += 1;
        while i < bytes.len() && (bytes[i] & 0xC0) == 0x80 {
            i += 1;
        }
    }
    out
}

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

    fn run(src: &str) -> Vec<String> {
        check(Path::new("t.sui"), src).into_iter().map(|f| f.message).collect()
    }

    #[test]
    fn accepts_surface_tokens() {
        assert!(run(r#"<sui-card sx="p:4 bg:surface-2 radius:lg text:sm fg:muted">x</sui-card>"#).is_empty());
    }

    #[test]
    fn rejects_layout_props_and_steers_to_primitives() {
        let m = run(r#"<sui-card sx="dir:col">x</sui-card>"#);
        assert_eq!(m.len(), 1);
        assert!(m[0].contains("sui-stack"), "should steer to primitives: {}", m[0]);
        assert_eq!(run(r#"<sui-card sx="gap:3">x</sui-card>"#).len(), 1);
    }

    #[test]
    fn rejects_raw_and_unknown() {
        assert_eq!(run(r#"<sui-card sx="p:13px">x</sui-card>"#).len(), 1);
        assert_eq!(run(r#"<sui-card sx="bg:#0af">x</sui-card>"#).len(), 1);
    }

    #[test]
    fn multibyte_source_does_not_panic() {
        // Regression: `scan_sx` byte-walked and sliced `source[i..]`, panicking
        // mid-codepoint on any multi-byte char (the async golden crash).
        assert!(run(r#"<p>Failed — retry…</p><sui-card sx="p:4">café ☕</sui-card>"#).is_empty());
        assert_eq!(run(r#"<sui-card>☕</sui-card><sui-card sx="p:99">x</sui-card>"#).len(), 1);
    }

    #[test]
    fn skips_comments_and_dynamic() {
        assert!(run(r#"<!-- <sui-card sx="p:99"> --> <sui-card sx={dynamic}>x</sui-card>"#).is_empty());
    }
}
