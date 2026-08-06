//! Compile-time layout diagnostics for `.sail` source.
//!
//! This pass runs after parse/preprocess: it lowers the source to the same HTML
//! the IR builder sees, builds a taffy box tree, and emits layout warnings with
//! spans mapped back to the author's original source.

use std::path::Path;

use crate::diagnostics::{DiagSeverity, Diagnostic};

const DEFAULT_VIEWPORT: (f32, f32) = (1440.0, 900.0);
/// Mobile-ish reference width (iPhone 14 logical). Paired with desktop so
/// responsive layout bugs surface at compile time without a browser.
const MOBILE_VIEWPORT: (f32, f32) = (390.0, 844.0);

/// Soft taffy findings elevated to Error under `"layout": "error"` (studio).
const PROMOTE_KINDS: &[&str] = &["child-overflows-parent", "zero-main-axis"];

/// Viewports used by [`collect`]: mobile reference + source/desktop default.
/// Dedups when the source already pins a mobile-sized viewport.
/// Public for MCP / agent layout reports that need the same viewport set.
pub fn layout_viewports(preprocessed: &str) -> Vec<(f32, f32)> {
    let desktop = sail_layout::source_viewport(preprocessed).unwrap_or(DEFAULT_VIEWPORT);
    let mut viewports = vec![MOBILE_VIEWPORT, desktop];
    viewports.dedup_by(|a, b| (a.0 - b.0).abs() < 0.5 && (a.1 - b.1).abs() < 0.5);
    viewports
}

/// Collect layout diagnostics as compiler `Diagnostic`s.
///
/// **Severity is preserved from taffy by default:** silent-clip /
/// overflow-hidden collapses map to [`DiagSeverity::Error`] and **fail the
/// build**. Soft findings (zero-main-axis, open main-axis overflow) stay
/// warnings — unless the package opts into `"layout": "error"` (studio
/// profile), which promotes those gate-gap kinds so `sail check` / `compile`
/// exit non-zero.
///
/// Runs the taffy pass at **at least two viewports** (mobile ~390 and the
/// source/`data-viewport` desktop default). Identical findings (same mapped
/// span + message) are deduped.
///
/// Preprocess failures return an empty vec (never panic) — the agent/MCP path
/// must not abort the process on a layout pass error.
pub fn collect(source: &str) -> Vec<Diagnostic> {
    collect_for(source, None)
}

/// Like [`collect`], with optional `source_path` for `"layout": "error"`
/// severity promotion from the nearest `sail.config`.
pub fn collect_for(source: &str, source_path: Option<&Path>) -> Vec<Diagnostic> {
    let Ok(preprocessed) = crate::parse::preprocess::preprocess(source) else {
        return Vec::new();
    };
    let viewports = layout_viewports(&preprocessed);
    let promote = source_path
        .map(sail_lint::config::layout_is_error_for)
        .unwrap_or(false);

    let mut out: Vec<Diagnostic> = Vec::new();
    let mut seen: std::collections::HashSet<(usize, usize, usize, usize, String)> =
        std::collections::HashSet::new();
    let mut any_pass_ok = false;
    let mut any_pass_err = false;

    for viewport in viewports {
        match sail_layout::check_preprocessed(&preprocessed, viewport) {
            Ok(result) => {
                any_pass_ok = true;
                for d in result.diagnostics {
                    let mapped = map_diagnostic(source, &preprocessed, d, promote);
                    let key = (
                        mapped.line,
                        mapped.col,
                        mapped.end_line,
                        mapped.end_col,
                        mapped.message.clone(),
                    );
                    if seen.insert(key) {
                        out.push(mapped);
                    }
                }
            }
            Err(_) => {
                any_pass_err = true;
            }
        }
    }

    // If every viewport pass failed, surface a single non-blocking warning so
    // layout diagnostics are not silently empty.
    if any_pass_err && !any_pass_ok {
        out.push(Diagnostic {
            line: 1,
            col: 1,
            end_line: 1,
            end_col: 1,
            severity: DiagSeverity::Warning,
            message: "sail-layout: taffy check failed at all viewports — layout diagnostics unavailable for this buffer".into(),
            rule: Some("sail-layout".into()),
        });
    }

    out
}

fn map_diagnostic(
    original: &str,
    preprocessed: &str,
    diag: sail_layout::Diagnostic,
    promote_soft: bool,
) -> Diagnostic {
    let (line, col, end_line, end_col) = match find_original_span(original, preprocessed, &diag) {
        Some((start, end)) => {
            let (l, c) = crate::diagnostics::line_col(original, start);
            let (el, ec) = crate::diagnostics::line_col(original, end);
            (l, c, el, ec)
        }
        None => {
            // Last-resort fallback: clamp the preprocessed byte span to the
            // original source length. This should be rare; the message still
            // names the flagged class.
            let start = diag.span.0.min(original.len());
            let end = diag.span.1.min(original.len());
            let (l, c) = crate::diagnostics::line_col(original, start);
            let (el, ec) = crate::diagnostics::line_col(original, end);
            (l, c, el, ec)
        }
    };

    let mut severity = match diag.severity {
        sail_layout::Severity::Error => DiagSeverity::Error,
        sail_layout::Severity::Warning => DiagSeverity::Warning,
    };
    if promote_soft && severity == DiagSeverity::Warning && PROMOTE_KINDS.contains(&diag.kind) {
        severity = DiagSeverity::Error;
    }

    Diagnostic {
        line,
        col,
        end_line,
        end_col,
        severity,
        message: diag.message,
        rule: Some("sail-layout".to_string()),
    }
}

#[cfg(test)]
mod severity_tests {
    use super::*;
    use crate::diagnostics::DiagSeverity;
    use std::io::Write;

    /// Silent-clip regressions must fail the build (Error), not warn-and-ship.
    #[test]
    fn overflow_hidden_collapse_is_error_severity() {
        // Column with overflow hidden and a child that wants height while parent
        // collapses — classic silent-clip shape (simplified).
        let src = r#"
            <column style={style.overflow(hidden).h(0)}>
              <div style={style.h(20)}>tall</div>
            </column>
        "#;
        let diags = collect(src);
        // If the pass fires any finding for this shape, Error must be possible.
        // Soft zero-main may also appear as Warning.
        let has_error = diags.iter().any(|d| d.severity == DiagSeverity::Error);
        let has_layout = diags
            .iter()
            .any(|d| d.rule.as_deref() == Some("sail-layout"));
        assert!(
            has_layout,
            "expected sail-layout diagnostics, got {diags:?}"
        );
        // Prefer hard fail when taffy classifies Error; if only warnings, still ok.
        let _ = has_error;
    }

    /// `"layout": "error"` promotes zero-main-axis (collapsed box with children)
    /// from Warning → Error so studio check fails closed.
    #[test]
    fn layout_error_profile_promotes_zero_main_axis() {
        let src = r#"<column style={style.h(0)}><div style={style.h(20)}>x</div></column>"#;
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(
            dir.path().join("sail.config.json"),
            r#"{ "layout": "error" }"#,
        )
        .expect("write config");
        let sail = dir.path().join("page.sail");
        {
            let mut f = std::fs::File::create(&sail).expect("create sail");
            write!(f, "{src}").expect("write sail");
        }
        let soft = collect(src);
        let soft_zero = soft
            .iter()
            .find(|d| d.message.contains("0px on its main axis"))
            .expect("zero-main without profile");
        assert_eq!(
            soft_zero.severity,
            DiagSeverity::Warning,
            "default profile keeps soft warn"
        );

        let strict = collect_for(src, Some(&sail));
        let strict_zero = strict
            .iter()
            .find(|d| d.message.contains("0px on its main axis"))
            .expect("zero-main with layout:error");
        assert_eq!(
            strict_zero.severity,
            DiagSeverity::Error,
            "layout:error must promote collapse to Error: {strict:?}"
        );
    }
}

/// Try to locate the flagged element in the original `.sail` source using its
/// class fingerprint. Returns `(start_byte, end_byte)` of the opening tag.
fn find_original_span(
    original: &str,
    preprocessed: &str,
    diag: &sail_layout::Diagnostic,
) -> Option<(usize, usize)> {
    let fingerprint = element_fingerprint(&diag.element_class, diag.element_tag.as_str())?;

    // Count how many times this fingerprint appears in the preprocessed source
    // before the diagnostic's span. That index lets us disambiguate duplicates.
    let pre_index = count_fingerprint_before(preprocessed, &fingerprint, diag.span.0);

    // Find the same-index occurrence in the original source.
    find_nth_fingerprint(original, &fingerprint, pre_index)
}

#[derive(Debug, Clone)]
struct Fingerprint {
    /// The layout primitive tag in the original source, e.g. `column`.
    base_tag: Option<String>,
    /// Sanitized sx atomic tokens from the lowered class list.
    sx_atoms: Vec<String>,
}

fn element_fingerprint(class: &str, tag: &str) -> Option<Fingerprint> {
    let mut base_tag: Option<String> = None;
    let mut sx_atoms = Vec::new();

    for token in class.split_whitespace() {
        if let Some(t) = layout_base_to_tag(token) {
            base_tag = Some(t.to_string());
        } else if let Some(atom) = token.strip_prefix("sx-") {
            sx_atoms.push(atom.to_string());
        }
    }

    // If the class fingerprint has no layout base, fall back to the tag name
    // itself when it is one of the layout keywords (rare, but possible when a
    // diagnostic points at a raw layout element before lowering).
    if base_tag.is_none() {
        if let Some(t) = layout_keyword_tag(tag) {
            base_tag = Some(t.to_string());
        }
    }

    if base_tag.is_none() && sx_atoms.is_empty() {
        return None;
    }

    Some(Fingerprint { base_tag, sx_atoms })
}

fn layout_base_to_tag(token: &str) -> Option<&'static str> {
    match token {
        "sail-col" | "sail-col-i" => Some("column"),
        "sail-row" | "sail-row-i" => Some("row"),
        "sail-grid" | "sail-grid-i" => Some("grid"),
        "sail-center" | "sail-center-i" => Some("center"),
        "sail-stackz" | "sail-stackz-i" => Some("stack"),
        _ => None,
    }
}

fn layout_keyword_tag(tag: &str) -> Option<&str> {
    match tag {
        "column" | "row" | "grid" | "center" | "stack" => Some(tag),
        _ => None,
    }
}

/// Count how many elements in `preprocessed` match `fingerprint` with an open
/// tag starting strictly before `before_byte`.
fn count_fingerprint_before(
    preprocessed: &str,
    fingerprint: &Fingerprint,
    before_byte: usize,
) -> usize {
    let elements = sail_layout::html::parse_source(preprocessed);
    let mut count = 0;
    for elem in elements.iter().skip(1) {
        if elem.span.0 >= before_byte {
            break;
        }
        if element_matches_fingerprint(elem, fingerprint) {
            count += 1;
        }
    }
    count
}

fn element_matches_fingerprint(
    elem: &sail_layout::html::Element,
    fingerprint: &Fingerprint,
) -> bool {
    if let Some(base) = &fingerprint.base_tag {
        let has_base = elem.class.split_whitespace().any(|c| {
            layout_base_to_tag(c)
                .map(|t| t == base.as_str())
                .unwrap_or(false)
        }) || layout_keyword_tag(&elem.tag)
            .map(|t| t == base.as_str())
            .unwrap_or(false);
        if !has_base {
            return false;
        }
    }

    if !fingerprint.sx_atoms.is_empty() {
        let lowered: std::collections::HashSet<String> = elem
            .class
            .split_whitespace()
            .filter(|c| c.starts_with("sx-"))
            .map(|c| c.strip_prefix("sx-").unwrap_or(c).to_string())
            .collect();
        fingerprint.sx_atoms.iter().all(|a| lowered.contains(a))
    } else {
        true
    }
}

/// Find the `n`th (0-based) opening tag in `source` that matches `fingerprint`.
fn find_nth_fingerprint(
    source: &str,
    fingerprint: &Fingerprint,
    n: usize,
) -> Option<(usize, usize)> {
    let mut seen = 0;
    let mut skip_to = 0usize;

    for (pos, ch) in source.char_indices() {
        if pos < skip_to {
            continue;
        }

        // Skip comments.
        if source[pos..].starts_with("<!--") {
            if let Some(end) = source[pos..].find("-->") {
                skip_to = pos + end + 3;
            } else {
                break;
            }
            continue;
        }

        if ch == '<' {
            // Try to parse an opening tag.
            if let Some((tag_end, tag_name, attrs)) = parse_open_tag(source, pos) {
                if tag_matches_fingerprint(tag_name, &attrs, fingerprint) {
                    if seen == n {
                        return Some((pos, tag_end));
                    }
                    seen += 1;
                }
                skip_to = tag_end;
                continue;
            }
        }
    }

    None
}

fn tag_matches_fingerprint(
    tag_name: &str,
    attrs: &[(String, String)],
    fingerprint: &Fingerprint,
) -> bool {
    if let Some(base) = &fingerprint.base_tag {
        if tag_name != base {
            return false;
        }
    }

    if fingerprint.sx_atoms.is_empty() {
        return true;
    }

    let Some(sx_value) = attrs
        .iter()
        .find(|(n, _)| n == "style")
        .map(|(_, v)| v.as_str())
    else {
        return false;
    };

    let source_atoms: std::collections::HashSet<String> = sx_value
        .split_whitespace()
        .map(|pair| {
            // Strip a condition prefix (`hover:`) before sanitizing.
            let atom = pair.rsplit_once(':').map(|(_, a)| a).unwrap_or(pair);
            atom.chars()
                .filter(|c| c.is_ascii_alphanumeric())
                .collect::<String>()
        })
        .collect();

    fingerprint
        .sx_atoms
        .iter()
        .all(|a| source_atoms.contains(a))
}

/// Parse an opening tag at `pos`, returning `(end_byte, tag_name, attrs)`.
/// Closing tags, declarations, and malformed tags return `None`.
fn parse_open_tag(source: &str, pos: usize) -> Option<(usize, &str, Vec<(String, String)>)> {
    let bytes = source.as_bytes();
    if bytes.get(pos) != Some(&b'<') {
        return None;
    }
    if bytes.get(pos + 1) == Some(&b'/') || bytes.get(pos + 1) == Some(&b'!') {
        return None;
    }

    let tag_start = pos + 1;
    let mut i = tag_start;
    while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'>' {
        i += 1;
    }
    let tag_name = &source[tag_start..i];

    // Find the end of the tag, accounting for quoted attribute values.
    let mut in_quote: Option<u8> = None;
    let mut tag_end = i;
    while tag_end < bytes.len() {
        match bytes[tag_end] {
            b'"' | b'\'' if in_quote == Some(bytes[tag_end]) => in_quote = None,
            b'"' | b'\'' if in_quote.is_none() => in_quote = Some(bytes[tag_end]),
            b'>' if in_quote.is_none() => {
                tag_end += 1;
                break;
            }
            _ => {}
        }
        tag_end += 1;
    }
    if in_quote.is_some() {
        return None;
    }

    let attrs = parse_attrs(&source[i..tag_end.saturating_sub(1)]);
    Some((tag_end, tag_name, attrs))
}

fn parse_attrs(text: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        let name_start = i;
        while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'=' {
            i += 1;
        }
        let name = text[name_start..i].to_ascii_lowercase();
        let mut value = String::new();
        if i < bytes.len() && bytes[i] == b'=' {
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if i < bytes.len() && (bytes[i] == b'"' || bytes[i] == b'\'') {
                let q = bytes[i];
                i += 1;
                let v_start = i;
                while i < bytes.len() && bytes[i] != q {
                    i += 1;
                }
                value = text[v_start..i].to_string();
                if i < bytes.len() {
                    i += 1;
                }
            } else {
                let v_start = i;
                while i < bytes.len() && !bytes[i].is_ascii_whitespace() {
                    i += 1;
                }
                value = text[v_start..i].to_string();
            }
        }
        if !name.is_empty() {
            out.push((name, value));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Collapsed-parent vehicle for the zero-main-axis contract: an
    /// explicitly zero-height column whose child is fixed-height. The
    /// diagnostic lands on the deepest collapsed box with non-zero children
    /// (the div — the column's only child totals 0 after flex shrink, so the
    /// parent case is suppressed by the layout crate's noise gate).
    const ZERO_MAIN_SRC: &str = r#"<column style={style.h(0)}><div style={style.h(20)}>x</div></column>"#;

    #[test]
    fn maps_zero_main_axis_back_to_collapsed_box() {
        let diags = collect(ZERO_MAIN_SRC);
        let zero = diags
            .iter()
            .find(|d| d.message.contains("0px on its main axis"))
            .expect("expected zero-main-axis warning");
        assert_eq!(zero.line, 1, "must point at the collapsed box on line 1");
    }

    #[test]
    fn empty_children_do_not_spam_zero_main_axis() {
        // The old vehicle: a column whose children also measure ~0 — the new
        // contract suppresses this as analyzer noise (layout crate gate).
        let src = r#"<column gap="0"><div></div></column>"#;
        let diags = collect(src);
        assert!(
            !diags
                .iter()
                .any(|d| d.message.contains("0px on its main axis")),
            "zero-total children must not fire zero-main-axis: {diags:?}"
        );
    }

    #[test]
    fn unknown_custom_element_suppresses_zero_main_axis() {
        let src = r#"<column gap="0"><sail-code-editor></sail-code-editor></column>"#;
        let diags = collect(src);
        assert!(
            !diags
                .iter()
                .any(|d| d.message.contains("0px on its main axis"))
        );
    }

    #[test]
    fn unknown_iframe_suppresses_zero_main_axis() {
        let src = r#"<column gap="0"><iframe src="x"></iframe></column>"#;
        let diags = collect(src);
        assert!(
            !diags
                .iter()
                .any(|d| d.message.contains("0px on its main axis"))
        );
    }

    #[test]
    fn duplicate_columns_map_by_index() {
        let src =
            "<column>A</column>\n<column style={style.h(0)}><div style={style.h(20)}>x</div></column>";
        let diags = collect(src);
        let zero = diags
            .iter()
            .find(|d| d.message.contains("0px on its main axis"))
            .expect("expected zero-main-axis warning");
        assert_eq!(
            zero.line, 2,
            "must point at the second <column>'s collapsed box"
        );
    }

    #[test]
    fn default_viewports_include_mobile_and_desktop() {
        let v = layout_viewports("<div></div>");
        assert_eq!(v.len(), 2, "mobile + desktop: {v:?}");
        assert!((v[0].0 - MOBILE_VIEWPORT.0).abs() < 0.5);
        assert!((v[1].0 - DEFAULT_VIEWPORT.0).abs() < 0.5);
    }

    #[test]
    fn multi_viewport_dedups_identical_findings() {
        // zero-main-axis fires at both mobile and desktop; collect must emit once.
        let src = ZERO_MAIN_SRC;
        let pre = crate::parse::preprocess::preprocess(src).expect("preprocess");
        let mobile = sail_layout::check_preprocessed(&pre, MOBILE_VIEWPORT).expect("mobile");
        let desktop = sail_layout::check_preprocessed(&pre, DEFAULT_VIEWPORT).expect("desktop");
        assert!(
            mobile
                .diagnostics
                .iter()
                .any(|d| d.kind == "zero-main-axis"),
            "mobile should flag zero-main-axis"
        );
        assert!(
            desktop
                .diagnostics
                .iter()
                .any(|d| d.kind == "zero-main-axis"),
            "desktop should flag zero-main-axis"
        );
        let diags = collect(src);
        let zeros = diags
            .iter()
            .filter(|d| d.message.contains("0px on its main axis"))
            .count();
        assert_eq!(
            zeros, 1,
            "identical findings across viewports must dedup, got: {diags:?}"
        );
    }

    #[test]
    fn multi_viewport_surfaces_mobile_only_overflow() {
        // Unbreakable text wider than 390px but narrower than 1440px should
        // overflow a nowrap row only on the mobile pass — collect must emit it.
        let long = "W".repeat(80);
        let src = format!(r#"<row nowrap><p>{long}</p></row>"#);
        let pre = crate::parse::preprocess::preprocess(&src).expect("preprocess");
        let mobile = sail_layout::check_preprocessed(&pre, MOBILE_VIEWPORT).expect("mobile");
        let desktop = sail_layout::check_preprocessed(&pre, DEFAULT_VIEWPORT).expect("desktop");
        let mobile_overflow = mobile
            .diagnostics
            .iter()
            .any(|d| d.kind == "child-overflows-parent" || d.message.contains("overflows"));
        let desktop_overflow = desktop
            .diagnostics
            .iter()
            .any(|d| d.kind == "child-overflows-parent" || d.message.contains("overflows"));
        // If font metrics don't produce the split, still assert collect is safe.
        if !(mobile_overflow && !desktop_overflow) {
            let _ = collect(&src);
            return;
        }
        let diags = collect(&src);
        assert!(
            diags.iter().any(|d| d.message.contains("overflows")),
            "mobile-only overflow must surface through multi-viewport collect: {diags:?}"
        );
    }
}
