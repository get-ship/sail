//! Compile-time layout diagnostics for `.sail` source.
//!
//! This pass runs after parse/preprocess: it lowers the source to the same HTML
//! the IR builder sees, builds a taffy box tree, and emits layout warnings with
//! spans mapped back to the author's original source.

use crate::diagnostics::{DiagSeverity, Diagnostic};

const DEFAULT_VIEWPORT: (f32, f32) = (1440.0, 900.0);

/// Collect layout diagnostics as compiler `Diagnostic`s. Always returns
/// warnings — layout findings never block the build.
pub fn collect(source: &str) -> Vec<Diagnostic> {
    let preprocessed = crate::parse::preprocess::preprocess(source);
    let viewport = sail_layout::source_viewport(&preprocessed).unwrap_or(DEFAULT_VIEWPORT);

    let Ok(result) = sail_layout::check_preprocessed(&preprocessed, viewport) else {
        return Vec::new();
    };

    result
        .diagnostics
        .into_iter()
        .map(|d| map_diagnostic(source, &preprocessed, d))
        .collect()
}

fn map_diagnostic(
    original: &str,
    preprocessed: &str,
    diag: sail_layout::Diagnostic,
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

    Diagnostic {
        line,
        col,
        end_line,
        end_col,
        severity: DiagSeverity::Warning,
        message: diag.message,
        rule: Some("sail-layout".to_string()),
    }
}

/// Try to locate the flagged element in the original `.sail` source using its
/// class fingerprint. Returns `(start_byte, end_byte)` of the opening tag.
fn find_original_span(
    original: &str,
    preprocessed: &str,
    diag: &sail_layout::Diagnostic,
) -> Option<(usize, usize)> {
    let fingerprint = element_fingerprint(&diag.element_class,
        diag.element_tag.as_str(),
    )?;

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
        let has_base = elem
            .class
            .split_whitespace()
            .any(|c| layout_base_to_tag(c).map(|t| t == base.as_str()).unwrap_or(false))
            || layout_keyword_tag(&elem.tag).map(|t| t == base.as_str()).unwrap_or(false);
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
        fingerprint
            .sx_atoms
            .iter()
            .all(|a| lowered.contains(a))
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
        .find(|(n, _)| n == "sx")
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

    fingerprint.sx_atoms.iter().all(|a| source_atoms.contains(a))
}

/// Parse an opening tag at `pos`, returning `(end_byte, tag_name, attrs)`.
/// Closing tags, declarations, and malformed tags return `None`.
fn parse_open_tag(
    source: &str,
    pos: usize,
) -> Option<(usize, &str, Vec<(String, String)>)> {
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
        while i < bytes.len()
            && !bytes[i].is_ascii_whitespace()
            && bytes[i] != b'='
        {
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

    #[test]
    fn maps_zero_main_axis_back_to_column_tag() {
        let src = r#"<column gap="0"><div></div></column>"#;
        let diags = collect(src);
        let zero = diags
            .iter()
            .find(|d| d.message.contains("0px on its main axis"))
            .expect("expected zero-main-axis warning");
        assert_eq!(zero.line, 1, "must point at <column> on line 1");
    }

    #[test]
    fn unknown_custom_element_suppresses_zero_main_axis() {
        let src = r#"<column gap="0"><sail-code-editor></sail-code-editor></column>"#;
        let diags = collect(src);
        assert!(!diags.iter().any(|d| d.message.contains("0px on its main axis")));
    }

    #[test]
    fn unknown_iframe_suppresses_zero_main_axis() {
        let src = r#"<column gap="0"><iframe src="x"></iframe></column>"#;
        let diags = collect(src);
        assert!(!diags.iter().any(|d| d.message.contains("0px on its main axis")));
    }

    #[test]
    fn duplicate_columns_map_by_index() {
        let src = "<column>A</column>\n<column gap=\"0\"><div></div></column>";
        let diags = collect(src);
        let zero = diags
            .iter()
            .find(|d| d.message.contains("0px on its main axis"))
            .expect("expected zero-main-axis warning");
        assert_eq!(zero.line, 2, "must point at the second <column>");
    }
}
