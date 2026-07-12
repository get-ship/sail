//! `sui-route-inline-style` — a route `page.sui` that contains an inline
//! `<style>` block.
//!
//! The deck bundler auto-links a co-located `style.css` as a route stylesheet
//! (the `deck-route-css` convention). An inline `<style>` block inside a
//! route `page.sui` is **silently dropped** — the compiler emits no JS for it
//! and the bundler emits no CSS import. The author gets no styling and no
//! error, which is confusing and hard to debug.
//!
//! The fix is to move the styles to a sibling `style.css`; the bundler then
//! picks it up automatically.
//!
//! Only fires on files whose name is exactly `page.sui` — the deck route
//! template convention. Component `.sui` files (e.g. `sui-button.sui`) are
//! expected to carry `<style>` blocks (shadow-DOM scoped), so this rule
//! intentionally skips them.
//!
//! **Severity: Warning** — advisory, not a build failure.

use std::path::Path;

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sui-route-inline-style";

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    // Only route templates (`page.sui`) — component .sui files legitimately
    // carry <style> blocks.
    if path.file_name().and_then(|n| n.to_str()) != Some("page.sui") {
        return Vec::new();
    }

    let mut findings = Vec::new();
    let bytes = source.as_bytes();
    let mut i = 0;

    while let Some(rel) = find_subslice(&bytes[i..], b"<style") {
        let tag_start = i + rel;
        let after_tag = tag_start + b"<style".len();

        // A `<style>` quoted inside an HTML comment is prose, not a block —
        // a page whose comment says "no <style>" must not warn about one.
        if let Some(c) = find_subslice(&bytes[i..], b"<!--").map(|r| i + r)
            && c < tag_start
        {
            i = match find_subslice(&bytes[c + 4..], b"-->") {
                Some(r) => c + 4 + r + 3,
                None => bytes.len(),
            };
            continue;
        }

        // Must be followed by whitespace, `>`, or `/` — not part of a longer
        // identifier like `<style-provider>`.
        if !matches!(
            bytes.get(after_tag).copied(),
            Some(b' ' | b'\t' | b'\n' | b'\r' | b'>' | b'/')
        ) {
            i = after_tag;
            continue;
        }

        let (line, col) = byte_offset_to_line_col(source, tag_start);
        findings.push(Finding {
            file: path.to_path_buf(),
            line,
            col,
            rule: RULE_ID,
            severity: Severity::Warn,
            message: "inline `<style>` in a route `page.sui` is silently dropped by the bundler \
                      — move styles to a sibling `style.css` (the deck-route-css convention)"
                .to_string(),
            suggest: Some(
                "Create `style.css` next to `page.sui`; the bundler picks it up automatically \
                 (deck-route-css convention). Remove the inline `<style>` block."
                    .to_string(),
            ),
        });

        i = after_tag;
    }

    findings
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    let last = haystack.len() - needle.len();
    (0..=last).find(|&i| &haystack[i..i + needle.len()] == needle)
}

fn byte_offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let bytes = source.as_bytes();
    let mut line = 1usize;
    let mut col = 1usize;
    let cap = offset.min(bytes.len());
    let mut i = 0;
    while i < cap {
        if bytes[i] == b'\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
        i += 1;
    }
    (line, col)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn page_path() -> PathBuf {
        PathBuf::from("/routes/agents/page.sui")
    }

    fn component_path() -> PathBuf {
        PathBuf::from("/components/sui-button/sui-button.sui")
    }

    #[test]
    fn inline_style_in_page_sui_warns() {
        let src = "<style>.foo { color: red; }</style><p>hello</p>";
        let findings = check(&page_path(), src);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].rule, RULE_ID);
        assert_eq!(findings[0].severity, Severity::Warn);
        assert!(findings[0].message.contains("silently dropped"), "{}", findings[0].message);
    }

    #[test]
    fn inline_style_in_component_sui_passes() {
        // Component .sui files legitimately carry <style> blocks.
        let src = "<style>:host { display: block; }</style><slot></slot>";
        let findings = check(&component_path(), src);
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn page_sui_without_style_passes() {
        let src = "<div><p>no styles here</p></div>";
        let findings = check(&page_path(), src);
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn multiple_style_blocks_warn_separately() {
        let src = "<style>.a{}</style><div></div><style>.b{}</style>";
        let findings = check(&page_path(), src);
        assert_eq!(findings.len(), 2, "each <style> block is a separate finding: {findings:?}");
    }

    #[test]
    fn style_like_tag_name_not_matched() {
        // `<style-provider>` starts with `<style` but is not a `<style>` block.
        let src = "<style-provider src=\"x\"></style-provider>";
        let findings = check(&page_path(), src);
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn position_points_at_style_open_tag() {
        let src = "<div></div>\n<style>.foo{}</style>";
        let findings = check(&page_path(), src);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].line, 2, "finding should be on line 2");
        assert_eq!(findings[0].col, 1, "finding should start at col 1");
    }

    #[test]
    fn style_mentioned_in_html_comment_passes() {
        let src = "<!-- authored under the floor: no <style>, no raw <div> -->\n<p>x</p>";
        let findings = check(&page_path(), src);
        assert!(findings.is_empty(), "a comment is prose: {findings:?}");
        // …and a REAL block after a comment still warns.
        let src2 = "<!-- no <style> here --><style>.a{}</style>";
        assert_eq!(check(&page_path(), src2).len(), 1);
    }

    #[test]
    fn only_page_sui_filename_triggers() {
        // A file named something other than `page.sui` even in a routes dir.
        let other = PathBuf::from("/routes/agents/layout.sui");
        let src = "<style>.foo{}</style><slot></slot>";
        let findings = check(&other, src);
        assert!(findings.is_empty(), "only page.sui should trigger: {findings:?}");
    }
}
