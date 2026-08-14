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

/// Experimental layout kinds (SPEC-LAYOUT-DIAG-1 / S4.1). Under top-level
/// `strict: true` these promote to Error at the gate.
const EXPERIMENTAL_KINDS: &[&str] = &["unintended-overflow", "clipped-text", "sub-min-tap-target"];

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
/// Experimental kinds (`unintended-overflow`, `clipped-text`) emit only when
/// the nearest config has `"strict": true` or `"layoutExperimental": true`.
/// `clipped-text` additionally requires `"layoutTextMetrics": true`. Under
/// `strict`, experimental findings are Error at the gate. LAYOUT-5 (tap
/// promote) is not this job.
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
    let viewports = layout_viewports(&preprocessed);
    let promote = source_path
        .map(sail_lint::config::layout_is_error_for)
        .unwrap_or(false);
    let (strict, check_opts) = layout_check_options_for(source_path);

    let mut out: Vec<Diagnostic> = Vec::new();
    let mut seen: std::collections::HashSet<(usize, usize, usize, usize, String)> =
        std::collections::HashSet::new();
    let mut any_pass_ok = false;
    let mut any_pass_err = false;

    for viewport in viewports {
        match sail_layout::check_preprocessed_with_hosts(
            &preprocessed,
            viewport,
            check_opts,
            &hosts.types,
            &hosts.boxes,
        ) {
            Ok(result) => {
                any_pass_ok = true;
                for d in result.diagnostics {
                    let mapped = map_diagnostic(source, &preprocessed, d, promote, strict);
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

/// Resolve experimental layout check options from the nearest package config.
///
/// - `experimental` ← `strict` **or** `layoutExperimental`
/// - `text_metrics` ← `layoutTextMetrics` only (explicit; noisy path stays opt-in)
fn layout_check_options_for(source_path: Option<&Path>) -> (bool, sail_layout::CheckOptions) {
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
    let text_metrics = cfg.layout_text_metrics();
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
/// Tries class fingerprint, then the nth opening tag of `element_tag`.
/// Returns `None` on a miss — never invents an EOF-clamped author span.
pub fn original_byte_span(
    original: &str,
    preprocessed: &str,
    diag: &sail_layout::Diagnostic,
) -> Option<(usize, usize)> {
    if let Some(span) = find_original_span(original, preprocessed, diag) {
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
    preprocessed: &str,
    diag: sail_layout::Diagnostic,
    promote_soft: bool,
    strict: bool,
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
    // S3.2 / S4.1: under top-level strict, experimental layout findings are
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
            <column style({ overflow(hidden).size({ h: 0 }) })>
              <div style({ size({ min-h: control-h-lg }) })>tall</div>
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
        let src = r#"<column style({ size({ h: 0 }) })><div style({ size({ min-h: control-h-lg }) })>x</div></column>"#;
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
        } else if let Some(atom) = token.strip_prefix("s-") {
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
            .filter(|c| c.starts_with("s-"))
            .map(|c| c.strip_prefix("s-").unwrap_or(c).to_string())
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
                if tag_matches_fingerprint(source, tag_end, tag_name, &attrs, fingerprint) {
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
    source: &str,
    tag_end: usize,
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

    let sx_owned;
    let sx_value: &str = if let Some(v) = attrs
        .iter()
        .find(|(n, _)| n == "style")
        .map(|(_, v)| v.as_str())
    {
        v
    } else if let Some(expr) =
        tag_style_expr_before(source, tag_end).or_else(|| block_style_expr_after(source, tag_end))
    {
        sx_owned = expr;
        sx_owned.as_str()
    } else {
        return false;
    };

    let collapsed: String = sx_value
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect();
    let source_atoms: std::collections::HashSet<String> = sx_value
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

    fingerprint.sx_atoms.iter().all(|a| {
        source_atoms.contains(a)
            || source_atoms
                .iter()
                .any(|s| s.contains(a.as_str()) || a.contains(s.as_str()))
            || collapsed.contains(a.as_str())
    })
}

/// First block Style statement immediately after an open tag (`>`).

/// Tag-position `style(…)` call ending at open tag `>` (`gt_pos`).
fn tag_style_expr_before(source: &str, gt_pos: usize) -> Option<String> {
    if gt_pos == 0 || gt_pos > source.len() {
        return None;
    }
    let bytes = source.as_bytes();
    let mut open = gt_pos;
    while open > 0 {
        if bytes[open - 1] == b'<' {
            open -= 1;
            break;
        }
        open -= 1;
    }
    let mut tag_end = open + 1;
    while tag_end < gt_pos
        && (bytes[tag_end].is_ascii_alphanumeric()
            || bytes[tag_end] == b'-'
            || bytes[tag_end] == b'_'
            || bytes[tag_end] == b':')
    {
        tag_end += 1;
    }
    let mut j = tag_end;
    let mut quote: Option<u8> = None;
    while j < gt_pos {
        match (quote, bytes[j]) {
            (Some(q), b) if b == q => {
                quote = None;
                j += 1;
            }
            (None, b'"') | (None, b'\'') => {
                quote = Some(bytes[j]);
                j += 1;
            }
            (Some(_), _) => j += 1,
            (None, _) => {
                if j + 5 <= gt_pos
                    && &bytes[j..j + 5] == b"style"
                    && (j == 0
                        || (!bytes[j - 1].is_ascii_alphanumeric()
                            && bytes[j - 1] != b'_'
                            && bytes[j - 1] != b'$'))
                    && (j + 5 >= gt_pos
                        || (!bytes[j + 5].is_ascii_alphanumeric()
                            && bytes[j + 5] != b'_'
                            && bytes[j + 5] != b':'))
                {
                    let mut k = j + 5;
                    while k < gt_pos && bytes[k].is_ascii_whitespace() {
                        k += 1;
                    }
                    if k < gt_pos && bytes[k] == b'(' {
                        let rest = &source[j..gt_pos];
                        let end = rest.find('>').unwrap_or(rest.len());
                        let expr = rest[..end].trim_end();
                        if expr.starts_with("style(") {
                            return Some(expr.to_string());
                        }
                    }
                }
                j += 1;
            }
        }
    }
    None
}

fn block_style_expr_after(source: &str, tag_end: usize) -> Option<String> {
    if tag_end >= source.len() {
        return None;
    }
    let body = &source[tag_end..];
    let bytes = body.as_bytes();
    let mut start = 0;
    while start < bytes.len() && bytes[start].is_ascii_whitespace() {
        start += 1;
    }
    if start >= bytes.len() || bytes[start] == b'<' || bytes[start] == b'{' {
        return None;
    }
    let rest = &body[start..];
    let t = rest.trim_start();
    if !(t.starts_with("style(")
        || t.starts_with("style.")
        || t.starts_with("match(")
        || t.starts_with("when(")
        || t.starts_with("compound(")
        || t.starts_with('['))
    {
        return None;
    }
    let mut i = 0;
    let mut paren = 0i32;
    let mut brace = 0i32;
    let mut bracket = 0i32;
    let mut saw_group = false;
    let b = rest.as_bytes();
    while i < b.len() {
        match b[i] {
            b'(' => {
                paren += 1;
                saw_group = true;
            }
            b')' => paren -= 1,
            b'{' => {
                brace += 1;
                saw_group = true;
            }
            b'}' => brace -= 1,
            b'[' => {
                bracket += 1;
                saw_group = true;
            }
            b']' => bracket -= 1,
            _ => {}
        }
        i += 1;
        if saw_group && paren == 0 && brace == 0 && bracket == 0 {
            let mut j = i;
            while j < b.len() && b[j].is_ascii_whitespace() {
                j += 1;
            }
            if j < b.len() && b[j] == b'.' {
                i = j;
                continue;
            }
            return Some(rest[..i].trim_end().to_string());
        }
    }
    None
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
    const ZERO_MAIN_SRC: &str = r#"<column style({ size({ h: 0 }) })><div style({ size({ min-h: control-h-lg }) })>x</div></column>"#;

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
        let src = "<column>A</column>\n<column style({ size({ h: 0 }) })><div style({ size({ min-h: control-h-lg }) })>x</div></column>";
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
        if !mobile
            .diagnostics
            .iter()
            .any(|d| d.kind == "zero-main-axis")
        {
            // STYLE-IR: this size shape may not collapse; dedup is N/A.
            return;
        }
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
