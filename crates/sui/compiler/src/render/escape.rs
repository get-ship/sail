//! HTML escaping + void-element classification.
//!
//! Both helpers are tiny and pure; they live here so the walker stays
//! focused on traversal.

/// HTML5 void elements — cannot have a closing tag.
/// List per <https://html.spec.whatwg.org/multipage/syntax.html#void-elements>.
const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source",
    "track", "wbr",
];

/// Returns `true` if the lowercased tag is an HTML5 void element.
pub fn is_void_element(tag_lower: &str) -> bool {
    VOID_ELEMENTS.contains(&tag_lower)
}

/// Escape an attribute value: `&`, `"`, `<`, `>` become entities.
///
/// Only attribute values flow through here. Static text in the
/// template is emitted verbatim — the parser preserves the user's
/// original byte sequence and we don't second-guess them.
pub fn escape_attr_value(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            c => out.push(c),
        }
    }
    out
}

/// Escape an evaluator-produced string for emission as text content
/// (between `<!--sui:text:bN-->` markers, or any element text). Quotes
/// are left intact — only `&`, `<`, `>` need entity-escaping in PCDATA.
pub fn escape_text_value(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn void_elements_are_recognised() {
        for tag in ["br", "input", "img", "hr", "meta", "link", "wbr"] {
            assert!(is_void_element(tag), "{tag} should be void");
        }
    }

    #[test]
    fn non_void_elements_rejected() {
        for tag in ["div", "p", "button", "ship-button", "template"] {
            assert!(!is_void_element(tag), "{tag} should not be void");
        }
    }

    #[test]
    fn escape_attr_handles_all_dangerous_chars() {
        assert_eq!(escape_attr_value(r#"a & b"c<d>e"#), "a &amp; b&quot;c&lt;d&gt;e");
    }

    #[test]
    fn escape_attr_passes_through_safe_chars() {
        assert_eq!(escape_attr_value("hello world 123"), "hello world 123");
    }

    #[test]
    fn escape_text_handles_html_specials() {
        assert_eq!(escape_text_value("a & b<c>d"), "a &amp; b&lt;c&gt;d");
    }

    #[test]
    fn escape_text_leaves_quotes_alone() {
        assert_eq!(escape_text_value(r#""hi""#), r#""hi""#);
    }
}
