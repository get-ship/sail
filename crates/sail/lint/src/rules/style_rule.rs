//! `spinnaker` — the Style paint surface.
//!
//! Product form is a **tag-position** `style(…)` call on the open tag:
//! ```text
//! <div style({ pad(4).bg(surface-2).radius(lg), when: { hover: bg(muted) } })>
//!   children
//! </div>
//! ```
//! Also: `style({ match: {…}, when: {…} })`, `style(constRef)`, or multi-partial
//! composition `style(a, b, …)`. Attribute hosts `style={…}` / `sx={…}` and
//! body-nested Style statements are **not** the product attach form.
//! Residual `.when(…)` method chains are a hard error (object key `when:`).
//! Utility-string atoms (`style="p-4 bg-surface-2"`) are a **hard residual
//! error** — not architecture.
//!
//! THE RULE: every value is a design token or a plain CSS keyword. Never a raw
//! number, never a colour, never a unit. There is no bracket escape (cf.
//! Tailwind's `p-[13px]`).
//!
//! The author vocabulary is `spinnaker::vocab` (`offered_rows()` —
//! concern | axis | plain). Product attach is tag-position
//! `style({ pad(4).bg(surface), when: { hover: bg(muted) } })`.
//! Emit is `vocab.rs` + `plains.rs` / `concerns.rs` + `scales.rs`.
//! A scale-valued setter spends a token and nothing else; a keyword-valued
//! one takes its CSS keywords. The value discipline is the point.
//!
//! SCOPE — Style paints ONE box. How its children sit is a layout element
//! (`<row>`, `<column>`, `<grid>`), which carries the semantics, the a11y and
//! the overflow handling a hand-rolled flex box loses. Every layout atom is
//! REFUSED BY NAME, and the refusal names the element to use instead: the
//! mission is never "find another way".
//!
//! Style is compile-time and static: it lowers to atomic classes plus injected
//! CSS. A **runtime** dynamic expression cannot be lowered and is a hard error
//! — conditional styling goes through a primitive prop (`tone={…}`) or a
//! `when` / `match` Style guard.

use std::path::Path;

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "spinnaker";

// Residual string lowerers still call these names. SSOT is the CSS adapter:
// `spinnaker_css::conditions` (`when({ tablet: … })`) and `spinnaker_css::prefixes`.
pub use spinnaker_css::{
    CONDITION_PREFIXES, STATE_PREFIXES, SX_EXTRA_FORMS, condition_media, condition_prefixes,
    group_attr_presence, is_prefix, pseudo_element_prefix, state_data, state_pseudo,
};

/// Internal emit/lint helper: every emit-leaf utility name with the values
/// it accepts. Surface docs, skill, and LSP walk `offered_rows()` in
/// `vocab.rs` — not this table.
pub fn vocabulary() -> Vec<(String, Vec<String>)> {
    spinnaker::offered_rows()
        .map(|r| (r.name.to_string(), Vec::new()))
        .collect()
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
            SxHit::AttrHost(offset) => {
                let (line, col) = offset_to_line_col(source, offset);
                findings.push(Finding {
                    file: path.to_path_buf(),
                    line,
                    col,
                    rule: RULE_ID,
                    severity: Severity::Error,
                    message: "style={…} / sx= is not Style attachment — use a tag call: \
                              style({ pad(4).bg(surface), when: { hover: bg(muted) }, match: { … } })"
                        .to_string(),
                    suggest: Some(
                        "Attach Style as style({…}) on the open tag (no `=`)."
                            .to_string(),
                    ),
                });
                continue;
            }
            SxHit::BodyAttach(offset) => {
                let (line, col) = offset_to_line_col(source, offset);
                findings.push(Finding {
                    file: path.to_path_buf(),
                    line,
                    col,
                    rule: RULE_ID,
                    severity: Severity::Error,
                    message: "unknown statement — Style attaches as style({…}) on the open tag, \
                              not as a body statement"
                        .to_string(),
                    suggest: Some(
                        "Move Style onto the open tag: <div style({ pad(4).bg(surface) })>…</div>"
                            .to_string(),
                    ),
                });
                continue;
            }
            SxHit::Static(val, offset) => (val, offset),
        };
        let (line, col) = offset_to_line_col(source, offset);

        // Residual utility-string product is a hard error — not a second language.
        if spinnaker::looks_like_utility_string_product(val) {
            findings.push(mk(
                path,
                line,
                col,
                "utility-string sx is not the Style product — use style({ pad(4).bg(surface), when:, match:, compound: })"
                    .to_string(),
            ));
            continue;
        }

        let (style, style_diags) = spinnaker::parse(val);
        for d in &style_diags {
            findings.push(mk(path, line, col, d.message()));
        }
        for d in spinnaker::clash::detect_clashes(&style, &std::collections::HashMap::new()) {
            findings.push(mk(path, line, col, d.message()));
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
            "sx is surface-only: spacing (p/m…, the Tailwind-aligned space ramp \
             1..64 plus half-steps), colors (bg/fg), radius, text. \
             Layout is the primitives' job (<row>/<column>/<grid>). \
             Raw px/hex are not expressible."
                .to_string(),
        ),
    }
}

/// Dynamic `style={…}` — cannot be lowered (sx is compile-time). Reject it loudly
/// instead of silently emitting `class=""`.
fn mk_dynamic(path: &Path, line: usize, col: usize) -> Finding {
    Finding {
        file: path.to_path_buf(),
        line,
        col,
        rule: RULE_ID,
        severity: Severity::Error,
        message: "dynamic `style={…}` is not supported — sx is a compile-time, static \
                  surface style, so a dynamic value silently lowers to nothing"
            .to_string(),
        suggest: Some(
            "Drive a primitive prop instead (e.g. `tone={ok() ? 'success' : 'destructive'}`), \
             attach Style as a tag call (`style({ pad(4).bg(surface) })` on the open tag), or \
             branch with when / match / an `{#if}` between static Style values."
                .to_string(),
        ),
    }
}

/// One Style host occurrence: a static Style value (value + offset) to validate,
/// a runtime dynamic `style={jsExpr}` to reject, a product Style attr host that
/// must migrate to tag call, or a body-nested Style statement (not product).
enum SxHit<'a> {
    Static(&'a str, usize),
    Dynamic(usize),
    /// `style={StyleProduct}` / `sx={…}` — not product attach form.
    AttrHost(usize),
    /// Body-nested Style after `>` — unknown statement; teach open-tag call.
    BodyAttach(usize),
}

/// Find Style hosts: tag-position `style(…)`, residual `style="…"`, dynamic
/// `style={js}`, retired product attr hosts, and body-nested Style (error).
/// Skips HTML comments.
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
        // Open tag → attrs / tag-position style(…); body Style after `>` is refused.
        if bytes[i] == b'<' && i + 1 < bytes.len() && bytes[i + 1] != b'/' && bytes[i + 1] != b'!' {
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
            if tag_end > tag_start {
                let mut k = tag_end;
                // Scan attrs on this open tag for style=/sx= hosts.
                while k < bytes.len() {
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
                    // Tag-position product: style(…)
                    if source[k..].starts_with("style") {
                        let after = k + 5;
                        let boundary_ok = after >= bytes.len()
                            || (!bytes[after].is_ascii_alphanumeric()
                                && bytes[after] != b'_'
                                && bytes[after] != b'$'
                                && bytes[after] != b'-'
                                && bytes[after] != b':');
                        if boundary_ok {
                            let mut m = after;
                            while m < bytes.len() && bytes[m].is_ascii_whitespace() {
                                m += 1;
                            }
                            if m < bytes.len() && bytes[m] == b'(' {
                                if let Some((rel_lo, rel_hi, _expr)) =
                                    try_block_style_statement(&source[k..])
                                {
                                    let abs_lo = k + rel_lo;
                                    let abs_hi = k + rel_hi;
                                    out.push(SxHit::Static(&source[abs_lo..abs_hi], abs_lo));
                                    k = abs_hi;
                                    continue;
                                }
                            }
                        }
                    }
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
                        // Full UTF-8 char — bare `k += 1` panics mid-codepoint when
                        // the next attr token is multi-byte (e.g. `{x === '●'}` on
                        // `<if>` / free text in a tag stream). Mirrors the outer
                        // loop's char-boundary advance.
                        k += 1;
                        while k < bytes.len() && (bytes[k] & 0xC0) == 0x80 {
                            k += 1;
                        }
                        continue;
                    }
                    let name = &source[name_lo..k];
                    while k < bytes.len() && bytes[k].is_ascii_whitespace() {
                        k += 1;
                    }
                    if k >= bytes.len() || bytes[k] != b'=' {
                        continue;
                    }
                    k += 1;
                    while k < bytes.len() && bytes[k].is_ascii_whitespace() {
                        k += 1;
                    }
                    if k >= bytes.len() {
                        break;
                    }
                    if bytes[k] == b'"' {
                        let vstart = k + 1;
                        if let Some(q) = source[vstart..].find('"') {
                            if name == "style" || name == "sx" {
                                out.push(SxHit::Static(&source[vstart..vstart + q], vstart));
                            }
                            k = vstart + q + 1;
                            continue;
                        }
                    } else if bytes[k] == b'\'' {
                        let vstart = k + 1;
                        if let Some(q) = source[vstart..].find('\'') {
                            if name == "style" || name == "sx" {
                                out.push(SxHit::Static(&source[vstart..vstart + q], vstart));
                            }
                            k = vstart + q + 1;
                            continue;
                        }
                    } else if bytes[k] == b'{' {
                        if let Some(close) = balanced_brace_end(bytes, k) {
                            let val = &source[k + 1..close];
                            if name == "style" || name == "sx" {
                                if is_style_product_value(val) {
                                    out.push(SxHit::AttrHost(name_lo));
                                } else {
                                    out.push(SxHit::Dynamic(name_lo));
                                }
                            }
                            k = close + 1;
                            continue;
                        }
                    } else {
                        // Unquoted value.
                        while k < bytes.len()
                            && !bytes[k].is_ascii_whitespace()
                            && bytes[k] != b'>'
                            && bytes[k] != b'/'
                        {
                            k += 1;
                        }
                    }
                }
                if k < bytes.len() && (bytes[k] == b'>' || bytes[k] == b'/') {
                    let gt = if bytes[k] == b'/' { k + 1 } else { k };
                    let self_closing = bytes[k] == b'/';
                    let body_start = if gt < bytes.len() && bytes[gt] == b'>' {
                        gt + 1
                    } else {
                        k + 1
                    };
                    // Body-nested Style is not product — hard error with tag-call teaching
                    // (align with preprocess refuse_body_style_statements).
                    if !self_closing {
                        if let Some((rel_lo, _rel_hi, _expr)) =
                            try_block_style_statement(&source[body_start..])
                        {
                            let abs_lo = body_start + rel_lo;
                            out.push(SxHit::BodyAttach(abs_lo));
                            i = body_start;
                            continue;
                        }
                    }
                    i = body_start;
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

/// First non-ws body content is a Style product statement?
fn try_block_style_statement(body: &str) -> Option<(usize, usize, &str)> {
    let bytes = body.as_bytes();
    let mut start = 0;
    while start < bytes.len() && bytes[start].is_ascii_whitespace() {
        start += 1;
    }
    if start >= bytes.len() {
        return None;
    }
    if bytes[start] == b'<' || bytes[start] == b'{' {
        return None;
    }
    let rest = &body[start..];
    if !is_style_product_value(rest) {
        return None;
    }
    let end = consume_style_expr_len(rest)?;
    let expr = rest[..end].trim_end();
    if expr.is_empty() {
        return None;
    }
    Some((start, start + end, expr))
}

/// Balanced Style product expression length (includes `.method(` chain tails).
fn consume_style_expr_len(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut i = 0;
    let mut paren = 0i32;
    let mut brack = 0i32;
    let mut brace = 0i32;
    let mut saw_group = false;

    while i < bytes.len() {
        match bytes[i] {
            b'"' | b'\'' => {
                let q = bytes[i];
                i += 1;
                while i < bytes.len() && bytes[i] != q {
                    if bytes[i] == b'\\' && i + 1 < bytes.len() {
                        i += 1;
                    }
                    i += 1;
                }
                if i < bytes.len() {
                    i += 1;
                }
                continue;
            }
            b'(' => {
                paren += 1;
                saw_group = true;
            }
            b')' => {
                paren -= 1;
                if paren < 0 {
                    return None;
                }
            }
            b'[' => {
                brack += 1;
                saw_group = true;
            }
            b']' => {
                brack -= 1;
                if brack < 0 {
                    return None;
                }
            }
            b'{' => {
                brace += 1;
                saw_group = true;
            }
            b'}' => {
                brace -= 1;
                if brace < 0 {
                    return None;
                }
            }
            b'.' if paren == 0 && brack == 0 && brace == 0 && saw_group => {
                // Chain tail `.when(` / `.press(` — continue.
                i += 1;
                while i < bytes.len()
                    && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'-')
                {
                    i += 1;
                }
                continue;
            }
            c if paren == 0 && brack == 0 && brace == 0 && saw_group => {
                if c.is_ascii_whitespace() || c == b'<' || c == b'{' || c == b'/' {
                    return Some(i);
                }
                // Bare ident / text — end of statement.
                if !c.is_ascii_alphanumeric() && c != b'_' {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
        if paren == 0 && brack == 0 && brace == 0 && saw_group {
            // Peek end after closing the root group.
            let mut j = i;
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b'.' {
                continue;
            }
            return Some(i);
        }
    }
    if saw_group && paren == 0 && brack == 0 && brace == 0 {
        Some(i)
    } else {
        None
    }
}

/// Offset of the `}` matching `{` at `open`, or None if unbalanced.
fn balanced_brace_end(bytes: &[u8], open: usize) -> Option<usize> {
    if bytes.get(open) != Some(&b'{') {
        return None;
    }
    let mut depth = 0i32;
    let mut i = open;
    while i < bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            b'"' | b'\'' => {
                let q = bytes[i];
                i += 1;
                while i < bytes.len() && bytes[i] != q {
                    if bytes[i] == b'\\' && i + 1 < bytes.len() {
                        i += 1;
                    }
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// True when brace contents are a compile-time Style expression (not JS).
fn is_style_product_value(val: &str) -> bool {
    let t = val.trim();
    // Outer braces from nested quote forms: `{ style.… }` already stripped by scan.
    let t = t
        .strip_prefix('{')
        .and_then(|s| s.strip_suffix('}'))
        .map(str::trim)
        .unwrap_or(t);
    t.starts_with("style.")
        || t.starts_with("style(")
        || t.starts_with("match(")
        || t.starts_with("when(")
        || t.starts_with("compound(")
        || t.starts_with('[')
        // Leftover `row.enter({…})` is not a utility string — parse teaches
        // the play map. Not a valid product after Phase 5.
        || spinnaker::parse::is_const_ref_motion_chain_pub(t)
        || {
            // bare Style const ref
            let mut chars = t.chars();
            match chars.next() {
                Some(c) if c.is_ascii_alphabetic() || c == '_' => {
                    t.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
                }
                _ => false,
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn run(src: &str) -> Vec<String> {
        check(Path::new("t.sail"), src)
            .into_iter()
            .map(|f| f.message)
            .collect()
    }

    #[test]
    fn multibyte_in_if_expr_does_not_panic_scan() {
        // sail-tree unsaved marker: `<if {w.trailing === '●'}>` — attr scan must
        // walk full UTF-8 chars so `source[k..]` never slices mid-codepoint.
        let src = "\
<if {w.trailing === '\u{25cf}'}>\n\
  <span aria-label=\"Unsaved\" style({ fg(warning) })>\u{25cf}</span>\n\
</if>\n";
        let _ = run(src); // must not panic
        // Tag-position style on the span is valid product — no findings required,
        // but the scan must complete.
        assert!(run(src).is_empty(), "{:?}", run(src));
    }

    #[test]
    fn style_block_channel_product_passes() {
        let src = r#"<div style({ bg(primary).pad({ x: 2 }) })>  x
</div>"#;
        assert!(run(src).is_empty(), "{:?}", run(src));
    }

    #[test]
    fn style_block_channel_with_when_passes() {
        // Product when: is an object key inside style({…}) — not a `.when(…)` chain.
        let src = r#"<div style({ pad(2).bg(primary), animate: { enter: play(fade) }, when: { hover: bg(muted) } })>  x
</div>"#;
        assert!(run(src).is_empty(), "{:?}", run(src));
    }

    #[test]
    fn style_block_list_match_when_product_passes() {
        let src = r#"<div style({ match: { variant: { solid: bg(primary) } }, when: { hover: opacity(90) } })>  x
</div>"#;
        assert!(run(src).is_empty(), "{:?}", run(src));
    }

    #[test]
    fn style_block_const_ref_is_static_product() {
        let src = r#"<div style(filled)>  x
</div>"#;
        assert!(run(src).is_empty(), "{:?}", run(src));
    }

    #[test]
    fn style_attr_host_is_not_product_happy_path() {
        // `style={…}` is NOT the tag call form — must refuse with tag-call teaching.
        let ms = run(r#"<div style={style({ bg(primary) })}>x</div>"#);
        assert!(
            ms.iter().any(|m| m.contains("style={")
                || m.contains("tag call")
                || m.contains("not Style attachment")),
            "attr host must not green as product: {ms:?}"
        );
    }

    #[test]
    fn utility_string_product_is_residual_error() {
        let ms = run(r#"<card style="p-4 bg-surface-2">x</card>"#);
        assert!(!ms.is_empty(), "utility-string sx must fail closed");
        assert!(
            ms.iter()
                .any(|m| m.contains("utility-string") || m.contains("Style product")),
            "{ms:?}"
        );
    }

    #[test]
    fn structured_colon_soup_is_residual_error() {
        let ms = run(r#"<div style="bg: primary px: 2">x</div>"#);
        assert!(!ms.is_empty(), "{ms:?}");
    }

    #[test]
    fn bad_token_fail_closed() {
        let ms = run(r#"<div style({ bg(notARealTokenZzz999) })>  x
</div>"#);
        assert!(!ms.is_empty(), "{ms:?}");
        assert!(
            ms.iter()
                .any(|m| m.contains("token") || m.contains("not a token")),
            "{ms:?}"
        );
    }

    #[test]
    fn unknown_setter_fail_closed() {
        let ms = run(r#"<div style({ notASetter(primary) })>  x
</div>"#);
        assert!(!ms.is_empty(), "{ms:?}");
    }

    #[test]
    fn runtime_dynamic_sx_is_an_error() {
        assert_eq!(run(r#"<card style={cls()}>x</card>"#).len(), 1);
        assert_eq!(run(r#"<card style={ok ? 'a' : 'b'}>x</card>"#).len(), 1);
    }

    #[test]
    fn breakpoints_match_the_tokens() {
        // Residual path must not diverge from Style conditions / design tokens.
        for (name, width) in crate::tokens::breakpoint_table() {
            let query = condition_media(&name).expect("theme breakpoint media");
            assert!(
                query.contains(width.trim()),
                "Style condition `{name}` is {query} but theme width is {width}"
            );
        }
    }
}
