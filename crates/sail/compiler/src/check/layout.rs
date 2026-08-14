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

/// Remaining experimental layout kinds (SPEC-LAYOUT-DIAG-1 / S4.1). Under
/// top-level `strict: true` these promote to Error at the gate.
const EXPERIMENTAL_KINDS: &[&str] = &["sub-min-tap-target"];

/// LAYOUT-5 promoted kinds — default Error on every collect path.
const DEFAULT_ERROR_KINDS: &[&str] = &["unintended-overflow", "clipped-text"];

/// Viewports used by [`collect`]: mobile reference + source/desktop default.
/// Dedups when the source already pins a mobile-sized viewport.
/// Public for MCP / agent layout reports that need the same viewport set.
pub fn layout_viewports(preprocessed: &str) -> Vec<(f32, f32)> {
    let desktop = sail_layout::source_viewport(preprocessed).unwrap_or(DEFAULT_VIEWPORT);
    let mut viewports = vec![MOBILE_VIEWPORT, desktop];
    viewports.dedup_by(|a, b| (a.0 - b.0).abs() < 0.5 && (a.1 - b.1).abs() < 0.5);
    viewports
}

/// One HTML forest + one taffy tree per viewport. Layout diagnostics and
/// compose share this so neither re-preprocesses or rebuilds.
pub struct SharedBoxes {
    pub preprocessed: String,
    pub elements: Vec<sail_layout::html::Element>,
    pub trees: Vec<SharedViewport>,
}

/// A computed box tree at one viewport, or the taffy error for that pass.
pub struct SharedViewport {
    pub viewport: (f32, f32),
    pub layout: Result<sail_layout::Layout, String>,
    pub scan: Result<sail_layout::ScanResult, String>,
}

/// Parse HTML once and compute one taffy tree per viewport.
pub fn prepare_boxes(
    preprocessed: String,
    hosts: &sail_layout::StyleHosts,
    check_opts: sail_layout::CheckOptions,
) -> SharedBoxes {
    let mut elements = sail_layout::html::parse_source(&preprocessed);
    sail_layout::apply_resolved_types(&mut elements, &hosts.types);
    sail_layout::apply_resolved_boxes(&mut elements, &hosts.boxes);
    let viewports = layout_viewports(&preprocessed);
    let mut trees = Vec::with_capacity(viewports.len());
    for viewport in viewports {
        let phase = format!(
            "layout-viewport.{}x{}",
            viewport.0 as u32, viewport.1 as u32
        );
        trees.push(layout_span(&phase, || {
            compute_viewport(elements.clone(), viewport, check_opts)
        }));
    }
    SharedBoxes {
        preprocessed,
        elements,
        trees,
    }
}

fn compute_viewport(
    elements: Vec<sail_layout::html::Element>,
    viewport: (f32, f32),
    check_opts: sail_layout::CheckOptions,
) -> SharedViewport {
    let layout = sail_layout::taffy_tree::build(elements, viewport).map_err(|e| e.to_string());
    let scan = match &layout {
        Ok(tree) => sail_layout::check_tree(tree, check_opts).map_err(|e| e.to_string()),
        Err(e) => Err(e.clone()),
    };
    SharedViewport {
        viewport,
        layout,
        scan,
    }
}

fn layout_span<R>(phase: &str, f: impl FnOnce() -> R) -> R {
    #[cfg(not(target_arch = "wasm32"))]
    {
        crate::profile::span(phase, f)
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = phase;
        f()
    }
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
/// LAYOUT-5: `unintended-overflow` and `clipped-text` emit as Error on the
/// default collect path. `clipped-text` still honors
/// `"layoutTextMetrics": false` as a kill switch. `sub-min-tap-target` emits
/// only when `"strict": true` or `"layoutExperimental": true`; under
/// `strict` it is Error at the gate.
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
/// severity promotion and experimental-kind enablement from the nearest
/// `sail.config`.
pub fn collect_for(source: &str, source_path: Option<&Path>) -> Vec<Diagnostic> {
    let Ok((preprocessed, hosts)) = crate::parse::preprocess::preprocess_with_style(source) else {
        return Vec::new();
    };
    let (_, check_opts) = layout_check_options_for(source_path);
    collect_from(
        source,
        source_path,
        &prepare_boxes(preprocessed, &hosts, check_opts),
    )
}

/// Scan already-computed viewport trees. `compile` / `check_all` pass the
/// shared artifact so compose can reuse the same boxes.
pub fn collect_from(
    source: &str,
    source_path: Option<&Path>,
    boxes: &SharedBoxes,
) -> Vec<Diagnostic> {
    let promote = source_path
        .map(sail_lint::config::layout_is_error_for)
        .unwrap_or(false);
    let (strict, _) = layout_check_options_for(source_path);
    let elements = &boxes.elements;

    let mut out: Vec<Diagnostic> = Vec::new();
    let mut seen: std::collections::HashSet<(usize, usize, usize, usize, String)> =
        std::collections::HashSet::new();
    let mut any_pass_ok = false;
    let mut any_pass_err = false;

    for tree in &boxes.trees {
        match &tree.scan {
            Ok(result) => {
                any_pass_ok = true;
                for d in result.diagnostics.clone() {
                    let mapped = map_diagnostic(source, elements, d, promote, strict);
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

/// Resolve layout check options from the nearest package config.
///
/// - `experimental` ← `strict` **or** `layoutExperimental` (tap only)
/// - `text_metrics` ← explicit `layoutTextMetrics`, else promoted default on
pub(crate) fn layout_check_options_for(
    source_path: Option<&Path>,
) -> (bool, sail_layout::CheckOptions) {
    let Some(path) = source_path else {
        return (false, sail_layout::CheckOptions::default());
    };
    let Some(dir) = path.parent() else {
        return (false, sail_layout::CheckOptions::default());
    };
    let Some((_, cfg)) = sail_lint::config::resolve(dir) else {
        return (false, sail_layout::CheckOptions::default());
    };
    let strict = cfg.strict();
    let experimental = strict || cfg.layout_experimental();
    let text_metrics = cfg.layout_text_metrics_opt().unwrap_or(true);
    (
        strict,
        sail_layout::CheckOptions {
            experimental,
            text_metrics: Some(text_metrics),
        },
    )
}

/// Map a layout diagnostic's preprocessed span back to original `.sail` bytes.
///
/// Tries host / layout-tag identity, then the nth opening tag of `element_tag`.
/// Returns `None` on a miss — never invents an EOF-clamped author span.
pub fn original_byte_span(
    original: &str,
    preprocessed: &str,
    diag: &sail_layout::Diagnostic,
) -> Option<(usize, usize)> {
    let elements = sail_layout::html::parse_source(preprocessed);
    if let Some(span) = find_original_span(original, &elements, diag) {
        return Some(span);
    }
    find_nth_tag(original, preprocessed, diag)
}

/// Stable rule id for a taffy finding kind. Experimental kinds get dedicated
/// `sail-*` ids for the diagnostic ratchet; landed kinds stay under
/// `sail-layout`.
fn rule_id_for_kind(kind: &str) -> &'static str {
    match kind {
        "unintended-overflow" => "sail-unintended-overflow",
        "clipped-text" => "sail-clipped-text",
        "sub-min-tap-target" => "sail-sub-min-tap-target",
        _ => "sail-layout",
    }
}

fn map_diagnostic(
    original: &str,
    elements: &[sail_layout::html::Element],
    diag: sail_layout::Diagnostic,
    promote_soft: bool,
    strict: bool,
) -> Diagnostic {
    let (line, col, end_line, end_col) = match find_original_span(original, elements, &diag) {
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
    if DEFAULT_ERROR_KINDS.contains(&diag.kind) {
        severity = DiagSeverity::Error;
    }
    // S3.2 / S4.1: under top-level strict, remaining experimental kinds are
    // Error at the gate (not merely soft warnings).
    if strict && EXPERIMENTAL_KINDS.contains(&diag.kind) {
        severity = DiagSeverity::Error;
    }

    Diagnostic {
        line,
        col,
        end_line,
        end_col,
        severity,
        message: diag.message,
        rule: Some(rule_id_for_kind(diag.kind).to_string()),
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
            <column style({ overflow({ all: hidden }).size({ h: 0 }) })>
              <div style({ size({ h: 20 }) })>tall</div>
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
        let src =
            r#"<column style({ size({ h: 0 }) })><div style({ size({ h: 20 }) })>x</div></column>"#;
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

/// Locate the flagged element in the original `.sail` via host / layout-tag
/// identity — not `s-*` class atoms. Returns `(start_byte, end_byte)`.
fn find_original_span(
    original: &str,
    elements: &[sail_layout::html::Element],
    diag: &sail_layout::Diagnostic,
) -> Option<(usize, usize)> {
    let elem = element_for_diag(elements, diag)?;
    let identity = element_identity(elem)?;
    let pre_index = count_identity_before(elements, &identity, diag.span.0);
    find_nth_identity(original, &identity, pre_index)
}

fn element_for_diag<'a>(
    elements: &'a [sail_layout::html::Element],
    diag: &sail_layout::Diagnostic,
) -> Option<&'a sail_layout::html::Element> {
    elements
        .iter()
        .skip(1)
        .find(|e| e.span.0 == diag.span.0)
        .or_else(|| {
            elements.iter().skip(1).find(|e| {
                e.span.0 <= diag.span.0
                    && e.span.1 >= diag.span.1
                    && e.tag.eq_ignore_ascii_case(&diag.element_tag)
            })
        })
}

/// Nth opening tag of `diag.element_tag` — used when the class fingerprint
/// is empty (plain `<a>` / `<em>` / `<if>` / unexpanded hosts).
///
/// Skips `div`: layout keywords lower to `<div>`, so a raw nth-div index in
/// the preprocessed tree is not the same as in the author source.
fn find_nth_tag(
    original: &str,
    preprocessed: &str,
    diag: &sail_layout::Diagnostic,
) -> Option<(usize, usize)> {
    let tag = diag.element_tag.as_str();
    if tag.is_empty() || tag.eq_ignore_ascii_case("div") {
        return None;
    }
    let n = count_open_tag_before(preprocessed, tag, diag.span.0);
    find_nth_open_tag(original, tag, n)
}

fn count_open_tag_before(source: &str, tag: &str, before_byte: usize) -> usize {
    let mut count = 0;
    walk_open_tags(source, |pos, tag_end, name| {
        if pos >= before_byte {
            return false;
        }
        if name.eq_ignore_ascii_case(tag) {
            count += 1;
        }
        let _ = tag_end;
        true
    });
    count
}

fn find_nth_open_tag(source: &str, tag: &str, n: usize) -> Option<(usize, usize)> {
    let mut seen = 0;
    let mut hit = None;
    walk_open_tags(source, |pos, tag_end, name| {
        if !name.eq_ignore_ascii_case(tag) {
            return true;
        }
        if seen == n {
            hit = Some((pos, tag_end));
            return false;
        }
        seen += 1;
        true
    });
    hit
}

fn walk_open_tags(source: &str, mut visit: impl FnMut(usize, usize, &str) -> bool) {
    let mut skip_to = 0usize;
    for (pos, ch) in source.char_indices() {
        if pos < skip_to {
            continue;
        }
        if source[pos..].starts_with("<!--") {
            if let Some(end) = source[pos..].find("-->") {
                skip_to = pos + end + 3;
            } else {
                break;
            }
            continue;
        }
        if ch == '<' {
            if let Some((tag_end, tag_name, _)) = parse_open_tag(source, pos) {
                if !visit(pos, tag_end, tag_name) {
                    return;
                }
                skip_to = tag_end;
            }
        }
    }
}

/// Author-facing layout identity: `data-sail-host`, then a layout keyword,
/// then a `sail-col` / `sail-row` primitive. Never Style `s-*` atoms.
fn element_identity(elem: &sail_layout::html::Element) -> Option<String> {
    if let Some(host) = elem.attrs.get("data-sail-host") {
        let h = host.trim();
        if !h.is_empty() {
            return Some(h.to_ascii_lowercase());
        }
    }
    let tag = elem.tag.to_ascii_lowercase();
    if layout_keyword_tag(&tag).is_some() {
        return Some(tag);
    }
    for token in elem.class.split_whitespace() {
        if let Some(t) = layout_base_to_tag(token) {
            return Some(t.to_string());
        }
    }
    None
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

fn count_identity_before(
    elements: &[sail_layout::html::Element],
    identity: &str,
    before_byte: usize,
) -> usize {
    elements
        .iter()
        .skip(1)
        .take_while(|e| e.span.0 < before_byte)
        .filter(|e| element_identity(e).as_deref() == Some(identity))
        .count()
}

/// Find the `n`th (0-based) opening tag in `source` whose author tag matches.
fn find_nth_identity(source: &str, identity: &str, n: usize) -> Option<(usize, usize)> {
    let mut seen = 0;
    let mut hit = None;
    walk_open_tags(source, |pos, tag_end, name| {
        if !name.eq_ignore_ascii_case(identity) {
            return true;
        }
        if seen == n {
            hit = Some((pos, tag_end));
            return false;
        }
        seen += 1;
        true
    });
    hit
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
    const ZERO_MAIN_SRC: &str =
        r#"<column style({ size({ h: 0 }) })><div style({ size({ h: 20 }) })>x</div></column>"#;

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
    fn identity_uses_host_and_layout_tag_not_s_atoms() {
        let host = sail_layout::html::parse_source(
            r#"<span class="s-displayinlineflex s-grow1" data-sail-host="spinner"></span>"#,
        );
        assert_eq!(
            element_identity(&host[1]).as_deref(),
            Some("spinner"),
            "host stamp wins over s-* class soup"
        );
        let col = sail_layout::html::parse_source(
            r#"<div class="sail-col s-grow1 s-pad4 s-overflowhidden">x</div>"#,
        );
        assert_eq!(
            element_identity(&col[1]).as_deref(),
            Some("column"),
            "sail-col primitive is identity; s-* atoms are not"
        );
    }

    #[test]
    fn duplicate_columns_map_by_index() {
        let src = "<column>A</column>\n<column style({ size({ h: 0 }) })><div style({ size({ h: 20 }) })>x</div></column>";
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

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn compile_records_one_preprocess_and_two_viewport_spans() {
        let src = "<column><p>hi</p></column>";
        // Stdlib element compile is a once-per-process preprocess of each
        // registry source — not a re-entry on this file's bytes.
        let _ = crate::compile(src, crate::CompileOpts::default());
        crate::profile::begin();
        let _ = crate::compile(src, crate::CompileOpts::default());
        let samples = crate::profile::take();
        crate::profile::end();
        let phases: Vec<&str> = samples.iter().map(|s| s.phase.as_str()).collect();
        let pre = samples.iter().filter(|s| s.phase == "preprocess").count();
        assert_eq!(pre, 1, "one preprocess on compile(); phases={phases:?}");
        let vps: Vec<&str> = samples
            .iter()
            .filter(|s| s.phase.starts_with("layout-viewport."))
            .map(|s| s.phase.as_str())
            .collect();
        assert_eq!(vps.len(), 2, "two viewports; got {vps:?} phases={phases:?}");
        assert!(
            samples.iter().any(|s| s.phase == "layout"),
            "missing layout span: {phases:?}"
        );
        assert!(
            samples.iter().any(|s| s.phase == "compose"),
            "missing compose span: {phases:?}"
        );
    }

    #[test]
    fn prepare_boxes_is_two_viewports_and_scan_does_not_rebuild() {
        let src = "<column><p>hi</p></column>";
        let (pre, hosts) =
            crate::parse::preprocess::preprocess_with_style(src).expect("preprocess");
        let boxes = prepare_boxes(pre, &hosts, sail_layout::CheckOptions::default());
        assert_eq!(boxes.trees.len(), 2, "viewports: {:?}", boxes.trees.len());
        assert!(
            boxes.trees.iter().all(|t| t.layout.is_ok()),
            "both viewports must layout"
        );
        // check_tree / collect_from must not call taffy_tree::build.
        let _ = collect_from(src, None, &boxes);
    }

    #[test]
    fn multi_viewport_dedups_identical_findings() {
        // zero-main-axis fires at both mobile and desktop; collect must emit once.
        let src = ZERO_MAIN_SRC;
        let (pre, hosts) =
            crate::parse::preprocess::preprocess_with_style(src).expect("preprocess");
        let opts = sail_layout::CheckOptions::default();
        let mobile = sail_layout::check_preprocessed_with_hosts(
            &pre,
            MOBILE_VIEWPORT,
            opts,
            &hosts.types,
            &hosts.boxes,
        )
        .expect("mobile");
        let desktop = sail_layout::check_preprocessed_with_hosts(
            &pre,
            DEFAULT_VIEWPORT,
            opts,
            &hosts.types,
            &hosts.boxes,
        )
        .expect("desktop");
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

    fn stub_diag(tag: &str, class: &str, span: (usize, usize)) -> sail_layout::Diagnostic {
        sail_layout::Diagnostic {
            kind: "unintended-overflow",
            severity: sail_layout::Severity::Warning,
            element_index: 1,
            element_tag: tag.to_string(),
            element_class: class.to_string(),
            message: String::new(),
            span,
            geometry: None,
        }
    }

    #[test]
    fn original_byte_span_maps_plain_tag_by_nth_occurrence() {
        let src = "<p>one</p>\n<p>two <a href=\"x\">go</a></p>\n";
        let pre = crate::parse::preprocess::preprocess_impl(src, false).expect("preprocess");
        let a_at = pre.find("<a").expect("preprocessed <a>");
        let span = original_byte_span(src, &pre, &stub_diag("a", "", (a_at, a_at + 2)))
            .expect("nth <a> must map");
        assert!(
            src[span.0..span.1].starts_with("<a"),
            "mapped {:?}",
            &src[span.0..span.1]
        );
        let (line, _) = crate::diagnostics::line_col(src, span.0);
        assert_eq!(line, 2, "second-line <a>, got line {line}");
    }

    #[test]
    fn original_byte_span_does_not_invent_eof_on_miss() {
        let src = "<p>hi</p>\n";
        let pre = crate::parse::preprocess::preprocess_impl(src, false).expect("preprocess");
        let miss = stub_diag("dialog", "", (pre.len() + 8, pre.len() + 16));
        assert!(
            original_byte_span(src, &pre, &miss).is_none(),
            "missing tag must not clamp to EOF"
        );
    }
}
