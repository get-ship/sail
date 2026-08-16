//! `sail-style-privilege` — L3 routes/pages may not write Style (SPEC-STYLE-ARCH-1).
//!
//! Privilege is path + file kind. Config cannot grant Full. M1 flips None on
//! for product trees (`apps/**`, `packages/sail/blocks/**`). Identities
//! slot-reject is M2 — this rule only fires None.

use std::path::Path;

use crate::report::Finding;
use crate::rules::ambient::finding;

pub const RULE_ID: &str = "sail-style-privilege";

const REMEDY: &str = "L3 routes write tags, not Style. Use an existing L1 tag \
     (`<label>`, `<card>`, `<button>`), extract an L2 component that spends \
     presets, or extend the theme. Chrome tags (`<pane>`, `<toolbar>`, `<form>`) \
     are M4 — ledger `needs <pane>` instead of inventing a twin. Do not move \
     Style into a route partial.";

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    if !spinnaker::privilege_enforced_for_path(path) {
        return Vec::new();
    }
    match spinnaker::privilege_for_path(path) {
        spinnaker::Privilege::None => none_findings(path, source),
        spinnaker::Privilege::Identities => identities_findings(path, source),
        spinnaker::Privilege::Full => Vec::new(),
    }
}

fn none_findings(path: &Path, source: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (offset, kind) in style_sites(source) {
        if let Some(f) = finding(
            path,
            source,
            offset,
            RULE_ID,
            format!("Style is illegal in a route ({kind}). {REMEDY}"),
            REMEDY,
        ) {
            findings.push(f);
        }
    }
    findings
}

// Slot form only. `pad(page)` / `pad(4)` / `margin(auto)` are presets — L2 may
// spend them. `pad({ x: 8 })` is the slot form and stays L1-only.
const L1_ONLY_SLOTS: &[&str] = &["pad({", "margin({", "position({", "filter({"];
const L1_ONLY_BARE: &[&str] = &["position(", "filter("];
const SLOT_CONCERNS: &[&str] = &[
    "type({",
    "size({",
    "border({",
    "bg({",
    "fg({",
    "elevation({",
    "radius({",
];

fn identities_findings(path: &Path, source: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (offset, kind) in identity_sites(source) {
        let (msg, suggest) = match kind {
            "pad({" | "margin({" | "position({" | "filter({" => (
                format!(
                    "`{kind}` is the slot form — L1-only. L2 spends presets (`pad(page)`, `pad(4)`, `margin(auto)`)."
                ),
                "pad({{ x: 8 }}) → pad(page) or pad(4). Do not invent a pad-only tag.",
            ),
            "position(" | "filter(" => (
                format!("`{kind}` is L1-only. L2 spends presets only."),
                "Do not invent a chrome tag to hold position/filter.",
            ),
            "grow(" | "shrink(" => (
                format!(
                    "`{kind}` is retired. Participation is `size(grow)` / `size(hug)` (M2b lowering)."
                ),
                "Rewrite to size(grow) or size(hug). Do not invent fill.",
            ),
            "size(full)" => (
                "`size(full)` is a DTCG CSS trap. Use `size(grow)` or `size(max)`.".into(),
                "size(grow) shares leftover; size(max) stretches the containing block.",
            ),
            _ => (
                format!(
                    "L2 Style takes a preset, not slots (`{kind}`). Type presets: body | body-sm | heading | label | mono | mono-sm | display-sm."
                ),
                "type({ size: xs, weight: semibold, tracking: wide, case: uppercase }) → type(label) when that census matches. Else leave red.",
            ),
        };
        if let Some(f) = finding(path, source, offset, RULE_ID, msg, suggest) {
            findings.push(f);
        }
    }
    findings
}

fn identity_sites(source: &str) -> Vec<(usize, &'static str)> {
    let bytes = source.as_bytes();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < bytes.len() {
        if !source.is_char_boundary(i) {
            i += 1;
            continue;
        }
        if bytes[i] == b'<' && source[i..].starts_with("<!--") {
            if let Some(end) = source[i..].find("-->") {
                i += end + 3;
                continue;
            }
        }
        if bytes[i] == b'/' && bytes.get(i + 1) == Some(&b'/') {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if bytes[i] == b'/' && bytes.get(i + 1) == Some(&b'*') {
            if let Some(end) = source[i + 2..].find("*/") {
                i += 2 + end + 2;
                continue;
            }
        }
        let rest = &source[i..];
        let mut hit = None;
        if rest.starts_with("grow(") && is_word_start(bytes, i) {
            hit = Some("grow(");
        } else if rest.starts_with("shrink(") && is_word_start(bytes, i) {
            hit = Some("shrink(");
        } else if rest.starts_with("size(full)") && is_word_start(bytes, i) {
            hit = Some("size(full)");
        } else {
            for needle in SLOT_CONCERNS
                .iter()
                .chain(L1_ONLY_SLOTS.iter())
                .chain(L1_ONLY_BARE.iter())
            {
                if rest.starts_with(needle) && is_word_start(bytes, i) {
                    hit = Some(*needle);
                    break;
                }
            }
        }
        if let Some(kind) = hit {
            out.push((i, kind));
            i += kind.len();
            continue;
        }
        i += utf8_step(source, i);
    }
    out
}

/// Sites of product Style: `style(` attach or `style name =` / `style foo =`.
fn style_sites(source: &str) -> Vec<(usize, &'static str)> {
    let bytes = source.as_bytes();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < bytes.len() {
        if !source.is_char_boundary(i) {
            i += 1;
            continue;
        }
        if bytes[i] == b'<' && source[i..].starts_with("<!--") {
            if let Some(end) = source[i..].find("-->") {
                i += end + 3;
                continue;
            }
        }
        if bytes[i] == b'/' && bytes.get(i + 1) == Some(&b'/') {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if bytes[i] == b'/' && bytes.get(i + 1) == Some(&b'*') {
            if let Some(end) = source[i + 2..].find("*/") {
                i += 2 + end + 2;
                continue;
            }
        }

        if source[i..].starts_with("style(") {
            if is_word_start(bytes, i) {
                out.push((i, "style("));
            }
            i += 6;
            continue;
        }
        if source[i..].starts_with("style ") && is_word_start(bytes, i) {
            if let Some(kind) = named_style_decl(&source[i..]) {
                out.push((i, kind));
                i += 6;
                continue;
            }
        }
        i += utf8_step(source, i);
    }
    out
}

fn utf8_step(source: &str, i: usize) -> usize {
    source[i..]
        .chars()
        .next()
        .map(|c| c.len_utf8())
        .unwrap_or(1)
}

fn named_style_decl(rest: &str) -> Option<&'static str> {
    // `style name =` or `style ident =`
    let after = rest.get(6..)?;
    let after = after.trim_start();
    if after.is_empty() {
        return None;
    }
    if after.starts_with("name ") || after.starts_with("name=") {
        return Some("style name =");
    }
    let ident_end = after
        .find(|c: char| !c.is_ascii_alphanumeric() && c != '_' && c != '-')
        .unwrap_or(after.len());
    if ident_end == 0 {
        return None;
    }
    let tail = after[ident_end..].trim_start();
    if tail.starts_with('=') {
        Some("style name =")
    } else {
        None
    }
}

fn is_word_start(bytes: &[u8], i: usize) -> bool {
    if i == 0 {
        return true;
    }
    !bytes[i - 1].is_ascii_alphanumeric() && bytes[i - 1] != b'_'
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::Severity;
    use std::path::PathBuf;

    fn route() -> PathBuf {
        PathBuf::from("apps/sail-site/routes/docs/page.sail")
    }

    fn component() -> PathBuf {
        PathBuf::from("apps/studio/src/studio-panel/studio-panel.sail")
    }

    fn matrix() -> PathBuf {
        PathBuf::from("crates/sail/compiler/tests/fixtures/l1-matrix/routes/button/page.sail")
    }

    #[test]
    fn route_style_call_errors() {
        let src = "<column style({ pad(6) })></column>";
        let findings = check(&route(), src);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].rule, RULE_ID);
        assert_eq!(findings[0].severity, Severity::Error);
        assert!(findings[0].message.contains("illegal"));
    }

    #[test]
    fn route_named_style_errors() {
        let src = "style shell = { pad(6).bg(background) }\n<column style(shell)></column>";
        let findings = check(&route(), src);
        assert!(findings.len() >= 2, "const + use: {findings:?}");
    }

    #[test]
    fn route_without_style_passes() {
        let src = "<column gap=\"4\"><h1>Hello</h1></column>";
        assert!(check(&route(), src).is_empty());
    }

    #[test]
    fn l2_preset_only_is_legal() {
        let src = "style eyebrow = { type(label).fg(muted) }\n<p style(eyebrow)>x</p>";
        assert!(
            check(&component(), src).is_empty(),
            "preset calls are Identities-legal: {src}"
        );
    }

    #[test]
    fn l2_type_slots_error() {
        let src = "style eyebrow = { type({ size: xs }) }";
        let findings = check(&component(), src);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert!(findings[0].message.contains("preset"));
    }

    #[test]
    fn l2_pad_preset_is_legal() {
        let src = "style box = { pad(page).bg(surface) }";
        assert!(
            check(&component(), src).is_empty(),
            "pad(page) is a preset: {src}"
        );
        let src4 = "style box = { pad(4).bg(surface) }";
        assert!(
            check(&component(), src4).is_empty(),
            "pad(4) is a preset: {src4}"
        );
    }

    #[test]
    fn l2_pad_slots_error() {
        let src = "style box = { pad({ x: 8 }).bg(surface) }";
        let findings = check(&component(), src);
        assert!(
            findings.iter().any(|f| f.message.contains("slot form")),
            "{findings:?}"
        );
    }

    #[test]
    fn utf8_ellipsis_does_not_panic() {
        let src = "<!-- … ⌘ -->\n<style eyebrow = { type(label) }>\n<p style(eyebrow)>x</p>";
        let _ = check(&component(), src);
        let route_src = "<!-- … -->\n<style shell = { pad(4) }>\n<p>x</p>";
        let _ = check(&route(), route_src);
    }

    #[test]
    fn matrix_fixture_not_enforced_in_m1() {
        let src = "style shell = { pad(6).bg(background) }";
        assert!(
            check(&matrix(), src).is_empty(),
            "M1 must not change matrix compile"
        );
    }
}
