//! Text-host law — which tags may spend textish Style setters.
//!
//! The language crate inspects the Style tree (`style_is_textish`). This
//! module answers “does this tag publish text?” from HTML phrasing/heading
//! tags plus the L1 element registry. Domain crates do not list tags.

/// HTML tags that publish text (phrasing / heading / form text).
///
/// Layout keywords, pure structure (`div`, `svg`), and non-text chrome are
/// not here. `button` is also an L1 native — registry walk agrees.
const HTML_TEXT_TAGS: &[&str] = &[
    // SPEC core
    "p",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "span",
    "a",
    "label",
    "code",
    "summary",
    "legend",
    "kbd",
    // Interactive text hosts
    "button",
    "input",
    "textarea",
    "option",
    // Prose / inline / table text
    "pre",
    "small",
    "strong",
    "em",
    "b",
    "i",
    "li",
    "th",
    "td",
    "dt",
    "dd",
    "figcaption",
    "caption",
    "cite",
    "time",
    "mark",
    "abbr",
    "sub",
    "sup",
    "blockquote",
    // Optional custom text host
    "text",
];

/// True when `tag` may carry textish setters (`type`, `tab`, …).
///
/// Pure: HTML phrasing + optional L1 manifest supplied by host. Compiler
/// never reads `stdlib::sources()` — the registry scan is replaced by
/// caller-provided `extra_text_roots`. No filesystem.
pub fn is_text_host(tag: &str) -> bool {
    is_text_host_with(tag, &[])
}

/// Host may pass extra text-root mappings (e.g. `chip -> span`) derived
/// from the L1 manifest at build. Empty slice = HTML only.
pub fn is_text_host_with(tag: &str, extra: &[(&str, &str)]) -> bool {
    let t = tag.trim();
    if t.is_empty() {
        return false;
    }
    let lower = t.to_ascii_lowercase();
    if is_html_text_tag(&lower) {
        return true;
    }
    if matches!(
        lower.as_str(),
        "row" | "column" | "grid" | "center" | "stack"
    ) {
        return false;
    }
    if let Some(root) = extra
        .iter()
        .find(|(name, _)| *name == lower)
        .map(|(_, root)| *root)
    {
        return is_html_text_tag(&root.to_ascii_lowercase());
    }
    false
}

fn is_html_text_tag(tag: &str) -> bool {
    HTML_TEXT_TAGS.contains(&tag)
}

/// Exposed for host tooling that needs to derive `extra` table from sources.
pub fn template_root_tag(source: &str) -> Option<&str> {
    let mut i = 0usize;
    let bytes = source.as_bytes();
    loop {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            return None;
        }
        if i + 4 <= bytes.len() && &source[i..i + 4] == "<!--" {
            let rest = &source[i + 4..];
            let end = rest.find("-->")?;
            i = i + 4 + end + 3;
            continue;
        }
        if source[i..].starts_with("<scope") {
            let rest = &source[i..];
            let end = rest.find("</scope>")?;
            i += end + "</scope>".len();
            continue;
        }
        if source[i..].starts_with("<controller") {
            let rest = &source[i..];
            let end = rest.find("</controller>")?;
            i += end + "</controller>".len();
            continue;
        }
        if source[i..].starts_with("<style") {
            let rest = &source[i..];
            let end = rest.find("</style>")?;
            i += end + "</style>".len();
            continue;
        }
        if bytes[i] != b'<' {
            return None;
        }
        let rest = &source[i + 1..];
        let name_end = rest.find(|c: char| !c.is_ascii_alphanumeric() && c != '-')?;
        if name_end == 0 {
            return None;
        }
        return Some(&rest[..name_end]);
    }
}

/// Diagnostic when textish setters attach to a non-text-bearing host.
pub fn text_host_error(tag: &str) -> String {
    format!(
        "typography is only legal on text-bearing elements (got `<{tag}>`) — \
         hosts: p, h1–h6, span, a, label, code, summary, legend, button, input, \
         textarea, option, pre, small, strong, em, b, i, li, th, td, and related \
         prose tags, plus L1 elements whose registry root publishes text. Move \
         `type(…)` onto a child text host (span/p/…), not layout keywords or \
         pure structure (row/column/div/svg/…)."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_and_registry_text_hosts() {
        let roots = [("chip", "span"), ("badge", "span")];
        for t in [
            "p", "h1", "span", "a", "button", "input", "li", "th", "pre", "text", "kbd",
        ] {
            assert!(is_text_host(t), "{t}");
        }
        assert!(is_text_host("BUTTON"), "HTML tags are case-insensitive");
        assert!(is_text_host("button"), "registry native button");
        assert!(is_text_host_with("chip", &roots), "chip hosts a span");
        assert!(is_text_host_with("badge", &roots), "badge hosts a span");
    }

    #[test]
    fn structure_and_layout_are_not_text_hosts() {
        for t in [
            "row",
            "column",
            "grid",
            "center",
            "stack",
            "div",
            "svg",
            "scroll-area",
            "main",
        ] {
            assert!(!is_text_host(t), "{t}");
        }
    }
}
