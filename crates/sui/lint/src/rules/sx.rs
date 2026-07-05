//! `sui-sx` — the token-only inline **surface** style attribute.
//!
//! `sx="p:4 bg:surface-2 radius:lg text:sm"` is a space-separated list of
//! `prop:token` pairs from a CLOSED, token-only vocabulary. Off-token (raw
//! px/hex, off-scale, unknown prop) is `Severity::Error` → promoted to a
//! `CompileError` by `check::css_token`, and surfaced identically in the LSP.
//! No bracket escape (cf. Tailwind `p-[13px]`).
//!
//! SCOPE — surface only. `sx` deliberately does NOT own layout: `dir`, `gap`,
//! `align`, `justify` belong to the layout keywords (`<row>`, `<column>`,
//! `<grid>`), which carry the semantics/a11y/overflow
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
    // token-scaled: value is a token-scale suffix → `var(--<prefix><value>)`
    Space,
    Color,
    Radius,
    Text,
    Shadow,
    Z,
    Weight,
    Leading,
    Tracking,
    // preset: value picks a curated composition
    Ring,
    Border,
    Size,
    Opacity,
    // enum-keyword: value is one of a closed keyword set → the CSS keyword itself
    Display,
    Overflow,
    Position,
    Cursor,
    Visibility,
}

fn prop_kind(prop: &str) -> Option<Kind> {
    use Kind::*;
    Some(match prop {
        // element-level spacing (padding = a surface inset, margin = a nudge).
        // NOTE: `gap`/`align`/`justify` are intentionally absent — sibling spacing
        // and flow are the layout keywords' job (`<column gap="3">`), not sx.
        "p" | "px" | "py" | "pt" | "pr" | "pb" | "pl" | "m" | "mt" | "mr" | "mb" | "ml" => Space,
        "bg" | "fg" => Color,
        "radius" => Radius,
        "text" => Text,
        "shadow" => Shadow,
        "z" => Z,
        "weight" => Weight,
        "leading" => Leading,
        "tracking" => Tracking,
        "ring" => Ring,
        "border" => Border,
        "w" | "h" | "min-w" | "min-h" | "max-w" | "max-h" => Size,
        "opacity" => Opacity,
        "display" => Display,
        "overflow" | "overflow-x" | "overflow-y" => Overflow,
        "position" => Position,
        "cursor" => Cursor,
        "visibility" => Visibility,
        _ => return None,
    })
}

// Vocabularies are NOT hand-maintained here — spacing/text/radius/shadow/z/…
// scales and the color surface all derive from the shared token registry
// (`crate::tokens`), so sx can never drift from `tokens.css` (the drift that
// wrongly rejected `text:3xl` and dropped `bg:success-bg`). Colors are the
// token names directly — the value IS the variable.
const ALL_PROPS: &[&str] = &[
    "p", "px", "py", "pt", "pr", "pb", "pl", "m", "mt", "mr", "mb", "ml",
    "bg", "fg", "radius", "text", "shadow", "z", "weight", "leading", "tracking",
    "ring", "border", "w", "h", "min-w", "min-h", "max-w", "max-h", "opacity",
    "display", "overflow", "overflow-x", "overflow-y", "position", "cursor", "visibility",
];

fn allowed(kind: Kind) -> Vec<&'static str> {
    use Kind::*;
    match kind {
        Space => crate::tokens::space_values().to_vec(),
        Color => crate::tokens::color_values().to_vec(),
        Radius => crate::tokens::radius_values().to_vec(),
        Text => crate::tokens::text_values().to_vec(),
        Shadow => {
            let mut v = vec!["none"];
            v.extend(crate::tokens::suffixes("shadow-"));
            v
        }
        Z => crate::tokens::suffixes("z-"),
        Weight => crate::tokens::suffixes("font-weight-"),
        Leading => crate::tokens::suffixes("leading-"),
        Tracking => crate::tokens::suffixes("tracking-"),
        Ring => {
            let mut v = vec!["none"];
            v.extend(crate::tokens::color_values());
            v
        }
        Border => {
            let mut v = vec!["none", "subtle", "strong"];
            v.extend(crate::tokens::color_values());
            v
        }
        Size => {
            let mut v = vec!["full", "auto", "fit", "screen", "min", "max", "0"];
            v.extend(crate::tokens::suffixes("space-"));
            v.extend(crate::tokens::size_values());
            v
        }
        Opacity => vec!["0", "25", "50", "75", "100"],
        Display => vec!["block", "inline", "inline-block", "inline-flex", "none"],
        Overflow => vec!["visible", "hidden", "scroll", "auto", "clip"],
        Position => vec!["static", "relative", "absolute", "fixed", "sticky"],
        Cursor => vec![
            "auto", "default", "pointer", "text", "wait", "move", "not-allowed", "help", "grab",
            "cell", "progress", "none",
        ],
        Visibility => vec!["visible", "hidden"],
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
        "shadow" => &["box-shadow"],
        "z" => &["z-index"],
        "weight" => &["font-weight"],
        "leading" => &["line-height"],
        "tracking" => &["letter-spacing"],
        "ring" => &["box-shadow"],
        "border" => &["border"],
        "w" => &["width"],
        "h" => &["height"],
        "min-w" => &["min-width"],
        "min-h" => &["min-height"],
        "max-w" => &["max-width"],
        "max-h" => &["max-height"],
        "opacity" => &["opacity"],
        "display" => &["display"],
        "overflow" => &["overflow"],
        "overflow-x" => &["overflow-x"],
        "overflow-y" => &["overflow-y"],
        "position" => &["position"],
        "cursor" => &["cursor"],
        "visibility" => &["visibility"],
        _ => &[],
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
    use Kind::*;
    // Every kind resolves to ONE CSS value string, applied to each of the prop's
    // `css_props` (usually one; the padding/margin longhands are the exception).
    // Token-scaled kinds emit `var(--…)`; enum kinds emit the validated keyword.
    let value: String = match kind {
        // `0`/`none` are the literal zeros (no `--space-0`/`--radius-none` token).
        Space => if token == "0" { "0".into() } else { format!("var(--space-{token})") },
        Color => format!("var(--{token})"),
        Radius => if token == "none" { "0".into() } else { format!("var(--radius-{token})") },
        Text => format!("var(--text-{token})"),
        Shadow => if token == "none" { "none".into() } else { format!("var(--shadow-{token})") },
        Z => format!("var(--z-{token})"),
        Weight => format!("var(--font-weight-{token})"),
        Leading => format!("var(--leading-{token})"),
        Tracking => format!("var(--tracking-{token})"),
        // a focus ring at the standard ring width in the chosen tone (or off).
        Ring => if token == "none" {
            "none".into()
        } else {
            format!(
                "0 0 0 var(--ring-offset) var(--background), \
                 0 0 0 calc(var(--ring-offset) + var(--ring-width)) var(--{token})"
            )
        },
        // a hairline border in the chosen color (or the strength words / off).
        Border => match token {
            "none" => "none".into(),
            "subtle" => "1px solid var(--border-subtle)".into(),
            "strong" => "1px solid var(--border-strong)".into(),
            _ => format!("1px solid var(--{token})"),
        },
        Size => match token {
            "full" => "100%".into(),
            "auto" => "auto".into(),
            "fit" => "fit-content".into(),
            "screen" => if matches!(prop, "h" | "min-h" | "max-h") { "100vh".into() } else { "100vw".into() },
            "min" => "min-content".into(),
            "max" => "max-content".into(),
            "0" => "0".into(),
            // A dimensional size token (icon/control/panel) is used by its full
            // name; the bare-number fallthrough is a spacing-scale step.
            _ if token.starts_with("icon-size-")
                || token.starts_with("control-h-")
                || token.starts_with("panel-") =>
            {
                format!("var(--{token})")
            }
            _ => format!("var(--space-{token})"),
        },
        Opacity => match token {
            "0" => "0",
            "25" => "0.25",
            "50" => "0.5",
            "75" => "0.75",
            _ => "1",
        }
        .into(),
        // enum-keyword kinds: the validated keyword IS the CSS value.
        Display | Overflow | Position | Cursor | Visibility => token.to_string(),
    };
    let decls = css_props(prop).iter().map(|p| (*p, value.clone())).collect();
    Some(decls)
}

/// Lower a `prop:channel` where `channel` is a `<theme>` channel (a file-local
/// CSS custom property the theme fills per prop value). The value is the kind's
/// template with `var(--channel)` as the atom — so `border:tone-border` becomes
/// `1px solid var(--tone-border)` (a full border), `min-h:sel-h` becomes
/// `var(--sel-h)`, `fg:tone-fg` becomes `var(--tone-fg)`. The compiler calls this
/// for values the lint accepted as declared channels.
pub fn lower_channel(prop: &str, channel: &str) -> Option<Vec<(&'static str, String)>> {
    use Kind::*;
    let kind = prop_kind(prop)?;
    let atom = format!("var(--{channel})");
    let value = match kind {
        Border => format!("1px solid {atom}"),
        Ring => format!(
            "0 0 0 var(--ring-offset) var(--background), \
             0 0 0 calc(var(--ring-offset) + var(--ring-width)) {atom}"
        ),
        // Color / Size / Space / Radius / Text / Shadow / … all take the bare
        // var — the channel is already the full variable, no scale prefix.
        _ => atom,
    };
    Some(css_props(prop).iter().map(|p| (*p, value.clone())).collect())
}

/// A state prefix (`hover:`/`focus:`/…) → its CSS pseudo-class. `focus` maps to
/// `:focus-visible` (the modern default — no ring on a mouse click). Shared by
/// the lint (validation) and preprocess (lowering) so they can't diverge.
pub fn state_pseudo(state: &str) -> Option<&'static str> {
    Some(match state {
        "hover" => ":hover",
        "focus" => ":focus-visible",
        "focus-within" => ":focus-within",
        "active" => ":active",
        "disabled" => ":disabled",
        _ => return None,
    })
}

/// Channel names declared by this file's `<theme … channels="…">` blocks — the
/// file-local sx values a component may consume (`border:tone-border`). A CLOSED
/// set: only names the component itself declared, never an open escape.
pub fn source_channels(source: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = source;
    while let Some(start) = rest.find("<theme") {
        let after = &rest[start..];
        let Some(tag_end) = after.find('>') else { break };
        let tag = &after[..tag_end];
        if let Some(key) = tag.find("channels=\"") {
            let vstart = key + "channels=\"".len();
            if let Some(vlen) = tag[vstart..].find('"') {
                out.extend(
                    tag[vstart..vstart + vlen]
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty()),
                );
            }
        }
        rest = &after[tag_end + 1..];
    }
    out
}

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    // Channels declared by this file's `<theme>` blocks are legal sx values
    // (`border:tone-border`) — a closed set, not an open escape.
    let channels = source_channels(source);
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
            // Bare `group` marks an ancestor for `group-<state>:` propagation.
            if pair == "group" {
                continue;
            }
            // `prop:token`, or `state:prop:token` (e.g. `hover:bg:surface-2`), where
            // `state` is a pseudo (`hover`) or a propagation (`group-hover`).
            let (prop, token) = match pair.splitn(3, ':').collect::<Vec<_>>().as_slice() {
                [prop, token] => (*prop, *token),
                [state, prop, token] => {
                    let ok = state_pseudo(state).is_some()
                        || state.strip_prefix("group-").is_some_and(|b| state_pseudo(b).is_some());
                    if !ok {
                        findings.push(mk(
                            path,
                            line,
                            col,
                            format!("sx unknown state `{state}:` — use hover:/focus:/focus-within:/active:/disabled: (or group-hover: to react to an ancestor)"),
                        ));
                        continue;
                    }
                    (*prop, *token)
                }
                _ => {
                    findings.push(mk(path, line, col, format!("sx `{pair}` — expected `prop:token` or `state:prop:token`")));
                    continue;
                }
            };
            let Some(kind) = prop_kind(prop) else {
                let mut m = format!("sx unknown property `{prop}:` — not in the surface vocabulary");
                // steer layout attempts to the primitives, not a bigger sx
                if matches!(prop, "dir" | "gap" | "align" | "justify" | "row" | "col") {
                    m.push_str("; layout belongs to the primitives — use <row>/<column>/<grid>, not sx");
                } else if let Some(s) = nearest(prop, ALL_PROPS) {
                    m.push_str(&format!("; did you mean `{s}:`?"));
                }
                findings.push(mk(path, line, col, m));
                continue;
            };
            let opts = allowed(kind);
            if !opts.contains(&token) && !channels.iter().any(|c| c == token) {
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
             Layout is the primitives' job (<row>/<column>/<grid>). \
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
    fn accepts_declared_theme_channels_only() {
        let themed = "<theme prop=\"tone\" channels=\"tone-fg\">\n  default: foreground\n</theme>\n<sui-card sx=\"fg:tone-fg\">x</sui-card>";
        assert!(run(themed).is_empty(), "declared channel is a legal sx value");
        assert_eq!(run(r#"<sui-card sx="fg:tone-fg">x</sui-card>"#).len(), 1, "undeclared channel is rejected");
    }

    #[test]
    fn accepts_state_prefixes_and_rejects_bad_state() {
        assert!(run(r#"<sui-card sx="bg:surface-1 hover:bg:surface-2 focus:ring:primary disabled:opacity:50">x</sui-card>"#).is_empty());
        let m = run(r#"<sui-card sx="hoverr:bg:surface-2">x</sui-card>"#);
        assert_eq!(m.len(), 1);
        assert!(m[0].contains("unknown state"), "{}", m[0]);
    }

    #[test]
    fn accepts_widened_surface() {
        assert!(run(r#"<sui-card sx="border:strong shadow:sm cursor:pointer w:full z:sticky weight:semibold leading:tight tracking:wide overflow:hidden opacity:75 position:relative ring:primary visibility:visible">x</sui-card>"#).is_empty());
    }

    #[test]
    fn widened_props_lower_from_tokens() {
        assert_eq!(lower("shadow", "sm").unwrap(), vec![("box-shadow", "var(--shadow-sm)".to_string())]);
        assert_eq!(lower("z", "sticky").unwrap(), vec![("z-index", "var(--z-sticky)".to_string())]);
        assert_eq!(lower("weight", "semibold").unwrap(), vec![("font-weight", "var(--font-weight-semibold)".to_string())]);
        assert_eq!(lower("cursor", "pointer").unwrap(), vec![("cursor", "pointer".to_string())]);
        assert_eq!(lower("opacity", "50").unwrap(), vec![("opacity", "0.5".to_string())]);
        assert_eq!(lower("border", "strong").unwrap(), vec![("border", "1px solid var(--border-strong)".to_string())]);
        assert_eq!(lower("w", "full").unwrap(), vec![("width", "100%".to_string())]);
        // off-scale / off-enum are rejected (no silent fallthrough)
        assert!(lower("shadow", "huge").is_none());
        assert!(lower("cursor", "banana").is_none());
        assert!(lower("z", "99").is_none());
    }

    #[test]
    fn size_props_accept_size_tokens_and_spacing() {
        assert!(run(r#"<sui-card sx="max-h:panel-max-h-md w:icon-size-lg h:control-h-md min-w:4">x</sui-card>"#).is_empty());
        // a bogus size token is still rejected
        assert_eq!(run(r#"<sui-card sx="w:icon-size-huge">x</sui-card>"#).len(), 1);
    }

    #[test]
    fn rejects_layout_props_and_steers_to_keywords() {
        let m = run(r#"<sui-card sx="dir:col">x</sui-card>"#);
        assert_eq!(m.len(), 1);
        assert!(m[0].contains("column"), "should steer to the layout keywords: {}", m[0]);
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
