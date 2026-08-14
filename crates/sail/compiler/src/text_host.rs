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
/// HTML phrasing/heading tags publish text. L1 registry elements publish
/// text when their template root is an HTML text tag (`button` is native
/// `<button>`; `chip` hosts a `<span>`). Layout keywords and structure
/// (`div`, `svg`, `scroll-area`) do not.
pub fn is_text_host(tag: &str) -> bool {
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
    // Read raw stdlib sources — do not compile the registry (preprocess
    // of stdlib would re-enter this function).
    if let Some(root) = registry_root_tag(&lower) {
        return is_html_text_tag(&root);
    }
    false
}

fn is_html_text_tag(tag: &str) -> bool {
    HTML_TEXT_TAGS.contains(&tag)
}

/// Template root tag of a registered L1 element, parsed from the raw
/// `.sail` source (no compile). Native elements root on their own tag.
fn registry_root_tag(tag: &str) -> Option<String> {
    let src = crate::stdlib::sources()
        .iter()
        .find(|(name, _)| *name == tag)
        .map(|(_, src)| *src)?;
    template_root_tag(src).map(|s| s.to_ascii_lowercase())
}

fn template_root_tag(source: &str) -> Option<&str> {
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
        for t in [
            "p", "h1", "span", "a", "button", "input", "li", "th", "pre", "text", "kbd",
        ] {
            assert!(is_text_host(t), "{t}");
        }
        assert!(is_text_host("BUTTON"), "HTML tags are case-insensitive");
        // L1 native inherits the HTML tag.
        assert!(is_text_host("button"), "registry native button");
        // L1 spliced whose structure root is a text tag.
        assert!(is_text_host("chip"), "chip hosts a span");
        assert!(is_text_host("badge"), "badge hosts a span");
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
