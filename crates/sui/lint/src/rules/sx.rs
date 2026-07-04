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
//! `sx` is compile-time and static: it lowers to atomic classes + injected
//! CSS at build. A dynamic `sx={…}` therefore cannot be lowered and used to
//! silently vanish (`class=""`) — the footgun this rule now closes: dynamic
//! `sx` is a hard `Error`, steering conditional styling to a primitive prop
//! (`tone={…}`) or a precomputed static class. Attributes inside HTML comments
//! are skipped.

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
    for hit in scan_sx(source) {
        let (val, offset) = match hit {
            SxHit::Dynamic(offset) => {
                let (line, col) = offset_to_line_col(source, offset);
                findings.push(mk_dynamic(path, line, col));
                continue;
            }
            SxHit::Static(val, offset) => (val, offset),
        };
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

/// Dynamic `sx={…}` — cannot be lowered (sx is compile-time). Reject it loudly
/// instead of silently emitting `class=""`.
fn mk_dynamic(path: &Path, line: usize, col: usize) -> Finding {
    Finding {
        file: path.to_path_buf(),
        line,
        col,
        rule: RULE_ID,
        severity: Severity::Error,
        message: "dynamic `sx={…}` is not supported — sx is a compile-time, static \
                  surface style, so a dynamic value silently lowers to nothing"
            .to_string(),
        suggest: Some(
            "Drive a primitive prop instead (e.g. `tone={ok() ? 'success' : 'destructive'}`), \
             or select between static `sx=\"…\"` values with an {#if} branch."
                .to_string(),
        ),
    }
}

/// One `sx` attribute occurrence: a static literal (value + offset) to validate,
/// or a dynamic `sx={…}` binding (offset of the `sx`) to reject.
enum SxHit<'a> {
    Static(&'a str, usize),
    Dynamic(usize),
}

/// Find every `sx=` attribute — static `sx="…"` or dynamic `sx={…}`. Skips HTML
/// comments.
fn scan_sx(source: &str) -> Vec<SxHit<'_>> {
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
        if i > 0 && bytes[i - 1].is_ascii_whitespace() && source[i..].starts_with("sx") {
            if let Some(rel) = sx_value_delim(&bytes[i + 2..]) {
                let dpos = i + 2 + rel;
                if bytes[dpos] == b'"' {
                    let vstart = dpos + 1;
                    if let Some(q) = source[vstart..].find('"') {
                        out.push(SxHit::Static(&source[vstart..vstart + q], vstart));
                        i = vstart + q + 1;
                        continue;
                    }
                } else {
                    // dynamic `sx={…}` — flag at the `sx`, skip past the brace.
                    out.push(SxHit::Dynamic(i));
                    if let Some(close) = source[dpos..].find('}') {
                        i = dpos + close + 1;
                        continue;
                    }
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

/// After `sx`: `(ws)* = (ws)*` then `"` (static) or `{` (dynamic). Returns the
/// byte offset (within `rest`) of that delimiter, or None if this isn't an
/// `sx=`-valued attribute (e.g. `sxy=`, or `sx` with no following `=`).
fn sx_value_delim(rest: &[u8]) -> Option<usize> {
    let mut k = 0;
    while k < rest.len() && rest[k].is_ascii_whitespace() {
        k += 1;
    }
    if k >= rest.len() || rest[k] != b'=' {
        return None;
    }
    k += 1;
    while k < rest.len() && rest[k].is_ascii_whitespace() {
        k += 1;
    }
    if k < rest.len() && (rest[k] == b'"' || rest[k] == b'{') {
        Some(k)
    } else {
        None
    }
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
    fn skips_commented_out_sx() {
        // sx inside an HTML comment is not real markup — no findings.
        assert!(run(r#"<!-- <sui-card sx="p:99"> --> <sui-card sx="p:4">x</sui-card>"#).is_empty());
    }

    #[test]
    fn rejects_dynamic_sx() {
        // sx can't be lowered from a runtime value; it must be a hard error,
        // not the old silent `class=""`.
        let m = run(r#"<sui-card sx={cond() ? 'a' : 'b'}>x</sui-card>"#);
        assert_eq!(m.len(), 1, "dynamic sx must produce exactly one error: {m:?}");
        assert!(m[0].contains("dynamic"), "message names the problem: {}", m[0]);
    }

    #[test]
    fn static_sx_after_a_dynamic_one_still_validates() {
        // the scanner must recover past `sx={…}` and keep checking later attrs.
        let m = run(r#"<sui-card sx={x}>a</sui-card><sui-card sx="p:99">b</sui-card>"#);
        assert_eq!(m.len(), 2, "one dynamic + one off-scale static: {m:?}");
    }
}
