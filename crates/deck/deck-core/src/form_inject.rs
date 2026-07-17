//! Host-side form-action injection.
//!
//! The SAIL compiler emits `mutation="bN"` markers on `<form>` elements but
//! does NOT bake the `/_deck/mutation/…` URL into the SSR output — that is the
//! host's job, because only the host knows which daemon it is running.
//!
//! `inject_form_actions` scans SSR HTML for open `<form …>` tags that carry a
//! `mutation="bN"` marker, and augments each one with
//! `action="/_deck/mutation/<view>?form=bN" method="post"` — unless the author
//! already set an explicit `action=` attribute (idempotent: safe to call on
//! HTML that was already injected or was produced by an older compiler that
//! baked the URL itself).

use crate::routes::mutation_action;

/// Scan `html` for `<form … mutation="bN" …>` open tags and, for each one
/// that does not already carry an `action=` attribute, insert the host-
/// computed `action` + `method` attributes immediately before the closing `>`.
///
/// Idempotent: forms that already have `action=` are left untouched.
pub fn inject_form_actions(html: &str, view_name: &str) -> String {
    let mut out = String::with_capacity(html.len() + 64);
    let mut rest = html;

    while let Some(tag_start) = rest.find("<form") {
        // Only match if the next char after "<form" is whitespace or ">",
        // so we don't match "<format>" or similar.
        let after = &rest[tag_start + 5..];
        if !after.starts_with(|c: char| c.is_ascii_whitespace() || c == '>') {
            out.push_str(&rest[..tag_start + 5]);
            rest = &rest[tag_start + 5..];
            continue;
        }

        // Find the end of this open tag (the matching unquoted ">").
        let Some(rel_end) = find_tag_end(after) else {
            // No closing ">": pass through verbatim.
            out.push_str(rest);
            return out;
        };
        let tag_inner = &after[..rel_end]; // content between "<form" and ">"

        // Extract mutation="bN" value.
        let Some(key) = extract_attr_value(tag_inner, "mutation") else {
            // No mutation attr on this form — pass it through unchanged.
            out.push_str(&rest[..tag_start + 5 + rel_end + 1]);
            rest = &rest[tag_start + 5 + rel_end + 1..];
            continue;
        };

        // If the form already has an explicit action=, leave it alone.
        if has_attr(tag_inner, "action") {
            out.push_str(&rest[..tag_start + 5 + rel_end + 1]);
            rest = &rest[tag_start + 5 + rel_end + 1..];
            continue;
        }

        // Emit the open-tag with injected action + method before the ">".
        let action = mutation_action(view_name, &key);
        out.push_str(&rest[..tag_start]); // text before "<form"
        out.push_str("<form");
        out.push_str(tag_inner);
        out.push_str(&format!(r#" action="{action}" method="post""#));
        out.push('>');
        rest = &rest[tag_start + 5 + rel_end + 1..];
    }
    out.push_str(rest);
    out
}

/// Find the index of the first unquoted `>` in `s`, which holds the content
/// of an open tag after the tag name (e.g. after `<form`).
/// Returns `None` if no `>` is found.
fn find_tag_end(s: &str) -> Option<usize> {
    let mut in_quote: Option<char> = None;
    for (i, c) in s.char_indices() {
        match in_quote {
            Some(q) if c == q => in_quote = None,
            Some(_) => {}
            None => match c {
                '"' | '\'' => in_quote = Some(c),
                '>' => return Some(i),
                _ => {}
            },
        }
    }
    None
}

/// Extract the value of attribute `attr_name="..."` or `attr_name='...'`
/// from a tag's inner content (the part between tag-name and ">").
fn extract_attr_value(tag_inner: &str, attr_name: &str) -> Option<String> {
    // Look for attr_name= preceded by whitespace (or at start).
    let needle_dq = format!(r#"{attr_name}=""#);
    let needle_sq = format!("{attr_name}='");
    if let Some(pos) = find_attr_start(tag_inner, &needle_dq) {
        let rest = &tag_inner[pos + needle_dq.len()..];
        let end = rest.find('"')?;
        return Some(rest[..end].to_string());
    }
    if let Some(pos) = find_attr_start(tag_inner, &needle_sq) {
        let rest = &tag_inner[pos + needle_sq.len()..];
        let end = rest.find('\'')?;
        return Some(rest[..end].to_string());
    }
    None
}

/// Returns `true` if `tag_inner` contains `attr_name=` as a word-boundary
/// attribute (not as a substring of another attribute name).
fn has_attr(tag_inner: &str, attr_name: &str) -> bool {
    let needle_dq = format!(r#"{attr_name}=""#);
    let needle_sq = format!("{attr_name}='");
    find_attr_start(tag_inner, &needle_dq).is_some()
        || find_attr_start(tag_inner, &needle_sq).is_some()
}

/// Find the byte-start of `needle` in `s` only when preceded by a
/// non-identifier character (whitespace, `>`, `/`, or start of string).
/// This prevents `xaction="..."` from matching `action=`.
fn find_attr_start(s: &str, needle: &str) -> Option<usize> {
    let mut search = s;
    let mut base = 0;
    while let Some(pos) = search.find(needle) {
        let abs = base + pos;
        let preceded_ok = abs == 0
            || s[..abs]
                .chars()
                .next_back()
                .map(|c| !c.is_ascii_alphanumeric() && c != '-' && c != '_')
                .unwrap_or(true);
        if preceded_ok {
            return Some(abs);
        }
        base += pos + 1;
        search = &s[base..];
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn injects_action_and_method_on_mutation_form() {
        let html = r#"<form mutation="b0"><button type="submit">Go</button></form>"#;
        let out = inject_form_actions(html, "todos");
        assert!(
            out.contains(r#"action="/_deck/mutation/todos?form=b0""#),
            "action must be injected; got: {out}"
        );
        assert!(out.contains(r#"method="post""#), "method must be injected; got: {out}");
        assert!(out.contains(r#"mutation="b0""#), "marker must be preserved; got: {out}");
    }

    #[test]
    fn skips_form_with_explicit_action() {
        let html = r#"<form action="/custom" mutation="b0"></form>"#;
        let out = inject_form_actions(html, "todos");
        assert!(
            out.contains(r#"action="/custom""#),
            "author action must be preserved; got: {out}"
        );
        assert!(
            !out.contains("/_deck/mutation"),
            "must not inject when action is explicit; got: {out}"
        );
    }

    #[test]
    fn skips_form_without_mutation() {
        let html = r#"<form action="/search"><input type="text"></form>"#;
        let out = inject_form_actions(html, "todos");
        assert_eq!(out, html, "form without mutation must be unchanged");
    }

    #[test]
    fn idempotent_on_already_injected_html() {
        let html = r#"<form mutation="b0" action="/_deck/mutation/todos?form=b0" method="post"></form>"#;
        let out = inject_form_actions(html, "todos");
        // The action is already present — must not double-inject.
        let count = out.matches("/_deck/mutation/todos?form=b0").count();
        assert_eq!(count, 1, "must not double-inject; got: {out}");
    }

    #[test]
    fn handles_multiple_forms() {
        let html = r#"<form mutation="b0"></form><form mutation="b2"></form>"#;
        let out = inject_form_actions(html, "myview");
        assert!(out.contains(r#"action="/_deck/mutation/myview?form=b0""#), "first form; got: {out}");
        assert!(out.contains(r#"action="/_deck/mutation/myview?form=b2""#), "second form; got: {out}");
    }
}
