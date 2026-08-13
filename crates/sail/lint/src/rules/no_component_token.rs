//! `sail-no-component-token` — L1 component token names are unnameable
//! in unsanctioned app `.sail` attributes (THEMES-1-ENFORCE / THEMES-2 §1).
//!
//! Invisibility applies at token-*reference* sites: unsanctioned attr names
//! (`button.bg=`) here, and Style setter args (`bg(button)`, `border(page)`)
//! via `ThemeOwnedToken`. Local declaration identifiers never resolve against
//! the token registry, so they cannot violate the law.

use std::path::Path;

use super::ambient::{finding, is_ambient_app_path};
use crate::report::Finding;

pub const RULE_ID: &str = "sail-no-component-token";

const REMEDY: &str = "Component tokens are theme-owned; override via theme= or the theme document.";

/// Raw-text hosts — declaration bodies (`style page =`), not markup attrs.
const RAW_TEXT_TAGS: &[&str] = &["scope", "controller", "style", "script"];

/// Scan app `.sail` for unsanctioned component-token *attribute names*.
///
/// Walks tags the way the product parser does (attrs, brace-primaries,
/// tag-position `style(…)`, raw-text bodies). Declaration identifiers,
/// for/match binders, and setter args are not attr names.
pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    if !is_ambient_app_path(path) {
        return Vec::new();
    }
    if path.extension().and_then(|e| e.to_str()) != Some("sail") {
        return Vec::new();
    }

    let mut findings = Vec::new();
    let bytes = source.as_bytes();
    let mut i = 0usize;
    let mut brace_depth = 0i32;
    let mut in_str: Option<u8> = None;

    while i < bytes.len() {
        if in_str.is_none() && bytes[i] == b'<' && source[i..].starts_with("<!--") {
            if let Some(end) = source[i..].find("-->") {
                i += end + 3;
                continue;
            }
        }
        if in_str.is_none() && bytes[i] == b'/' && bytes.get(i + 1) == Some(&b'/') {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if in_str.is_none() && bytes[i] == b'/' && bytes.get(i + 1) == Some(&b'*') {
            if let Some(end) = source[i + 2..].find("*/") {
                i += 2 + end + 2;
                continue;
            }
        }

        if let Some(q) = in_str {
            if bytes[i] == b'\\' && i + 1 < bytes.len() {
                i += 2;
                continue;
            }
            if bytes[i] == q {
                in_str = None;
            }
            i += 1;
            continue;
        }

        // Quote-awareness only inside `{…}` so prose apostrophes stay text.
        if brace_depth > 0 && matches!(bytes[i], b'"' | b'\'' | b'`') {
            in_str = Some(bytes[i]);
            i += 1;
            continue;
        }
        if bytes[i] == b'{' {
            brace_depth += 1;
            i += 1;
            continue;
        }
        if bytes[i] == b'}' {
            if brace_depth > 0 {
                brace_depth -= 1;
            }
            i += 1;
            continue;
        }

        // Match/for arms keep tags inside `{…}` — still real attr sites.
        if bytes[i] == b'<' && is_tag_name_start(bytes.get(i + 1).copied()) {
            if bytes.get(i + 1) == Some(&b'/') {
                i = skip_until_gt(bytes, i + 2);
                continue;
            }
            let tag_start = i + 1;
            let mut j = tag_start;
            while j < bytes.len() && is_tag_name_byte(bytes[j]) {
                j += 1;
            }
            let tag = source[tag_start..j].to_ascii_lowercase();
            let (attrs_end, names) = scan_open_tag_attrs(source, j);
            for (name_start, name) in names {
                if !is_unsanctioned_token_attr(&name) {
                    continue;
                }
                let msg = format!(
                    "component token `{name}` is theme-owned; override via theme= or the theme document"
                );
                if let Some(f) = finding(path, source, name_start, RULE_ID, msg, REMEDY) {
                    findings.push(f);
                }
            }
            i = attrs_end;
            if RAW_TEXT_TAGS.contains(&tag.as_str()) {
                i = skip_raw_text_body(source, i, &tag);
            }
            continue;
        }

        i += 1;
    }

    findings.sort_by_key(|f| (f.line, f.col));
    findings.dedup_by(|a, b| a.line == b.line && a.col == b.col);
    findings
}

/// Token-reference attr names: dotted author paths (`button.bg`) and emit
/// labels (`button-foreground`). Bare pack stems (`page`, `label`) collide
/// with tags and semantic props — they are not token refs.
fn is_unsanctioned_token_attr(name: &str) -> bool {
    if name == "theme" || name == "style" || name.is_empty() {
        return false;
    }
    if name.contains('.') {
        return sail_style::is_theme_owned_token_name(name);
    }
    sail_style::is_theme_owned_token_name(name) && !sail_style::is_theme_owned_author_root(name)
}

fn is_tag_name_start(b: Option<u8>) -> bool {
    b.is_some_and(|c| c.is_ascii_alphabetic() || c == b'/')
}

fn is_tag_name_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b':'
}

fn is_attr_name_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.' || b == b':'
}

fn skip_until_gt(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() && bytes[i] != b'>' {
        i += 1;
    }
    if i < bytes.len() {
        i + 1
    } else {
        i
    }
}

/// Parse open-tag attrs starting at `start` (first byte after the tag name).
/// Returns the byte after `>` and (offset, name) for each attr name.
fn scan_open_tag_attrs(source: &str, start: usize) -> (usize, Vec<(usize, String)>) {
    let bytes = source.as_bytes();
    let mut i = start;
    let mut names = Vec::new();
    loop {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            return (i, names);
        }
        if bytes[i] == b'>' {
            return (i + 1, names);
        }
        if bytes[i] == b'/' {
            i += 1;
            continue;
        }
        if bytes[i] == b'{' {
            i = skip_brace_group(bytes, i);
            continue;
        }
        // Tag-position `style(…)` is a Style call, not an unsanctioned attr.
        if bytes.get(i..).is_some_and(|s| s.starts_with(b"style")) {
            let after = i + 5;
            let boundary = after >= bytes.len()
                || (!bytes[after].is_ascii_alphanumeric()
                    && bytes[after] != b'_'
                    && bytes[after] != b'$'
                    && bytes[after] != b'-'
                    && bytes[after] != b':');
            if boundary {
                let mut k = after;
                while k < bytes.len() && bytes[k].is_ascii_whitespace() {
                    k += 1;
                }
                if k < bytes.len() && bytes[k] == b'(' {
                    if let Some(end) = consume_style_call(source, i) {
                        i = end;
                        continue;
                    }
                }
            }
        }
        if !is_attr_name_byte(bytes[i]) {
            i += 1;
            continue;
        }
        let name_start = i;
        while i < bytes.len() && is_attr_name_byte(bytes[i]) {
            i += 1;
        }
        names.push((name_start, source[name_start..i].to_ascii_lowercase()));
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i < bytes.len() && bytes[i] == b'=' {
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            i = skip_attr_value(bytes, i);
        }
    }
}

fn skip_brace_group(bytes: &[u8], start: usize) -> usize {
    if start >= bytes.len() || bytes[start] != b'{' {
        return start;
    }
    let mut depth = 1i32;
    let mut i = start + 1;
    while i < bytes.len() && depth > 0 {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => depth -= 1,
            _ => {}
        }
        i += 1;
    }
    i
}

fn skip_attr_value(bytes: &[u8], start: usize) -> usize {
    if start >= bytes.len() {
        return start;
    }
    match bytes[start] {
        q @ (b'"' | b'\'') => {
            let mut i = start + 1;
            while i < bytes.len() && bytes[i] != q {
                if bytes[i] == b'\\' && i + 1 < bytes.len() {
                    i += 2;
                    continue;
                }
                i += 1;
            }
            if i < bytes.len() {
                i + 1
            } else {
                i
            }
        }
        b'{' => skip_brace_group(bytes, start),
        _ => {
            let mut i = start;
            while i < bytes.len() {
                let c = bytes[i];
                if c.is_ascii_whitespace() || c == b'>' || c == b'/' {
                    break;
                }
                i += 1;
            }
            i
        }
    }
}

fn consume_style_call(src: &str, start: usize) -> Option<usize> {
    let s = &src[start..];
    if !s.starts_with("style") {
        return None;
    }
    let bytes = s.as_bytes();
    let mut i = 0;
    let mut paren = 0i32;
    let mut brack = 0i32;
    let mut brace = 0i32;
    let mut saw = false;
    while i < bytes.len() {
        match bytes[i] {
            b'"' | b'\'' => {
                let q = bytes[i];
                i += 1;
                while i < bytes.len() && bytes[i] != q {
                    if bytes[i] == b'\\' && i + 1 < bytes.len() {
                        i += 1;
                    }
                    i += 1;
                }
                if i < bytes.len() {
                    i += 1;
                }
                continue;
            }
            b'(' => {
                paren += 1;
                saw = true;
            }
            b')' => {
                paren -= 1;
                if paren < 0 {
                    return None;
                }
            }
            b'[' => {
                brack += 1;
                saw = true;
            }
            b']' => {
                brack -= 1;
                if brack < 0 {
                    return None;
                }
            }
            b'{' => {
                brace += 1;
                saw = true;
            }
            b'}' => {
                brace -= 1;
                if brace < 0 {
                    return None;
                }
            }
            _ => {}
        }
        i += 1;
        if saw && paren == 0 && brack == 0 && brace == 0 {
            let mut j = i;
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b'.' {
                let after = j + 1;
                let mut k = after;
                while k < bytes.len()
                    && (bytes[k].is_ascii_alphanumeric() || bytes[k] == b'_' || bytes[k] == b'-')
                {
                    k += 1;
                }
                if k > after {
                    let mut m = k;
                    while m < bytes.len() && bytes[m].is_ascii_whitespace() {
                        m += 1;
                    }
                    if m < bytes.len() && bytes[m] == b'(' {
                        i = m;
                        continue;
                    }
                }
            }
            return Some(start + i);
        }
    }
    None
}

fn skip_raw_text_body(source: &str, start: usize, tag: &str) -> usize {
    let needle = format!("</{tag}");
    let bytes = source.as_bytes();
    let mut i = start;
    while i < bytes.len() {
        // Probe `<` on bytes first — a mid-rune slice of `source[i..]` panics.
        if bytes[i] == b'<'
            && source[i..].len() >= needle.len()
            && source[i..]
                .get(..needle.len())
                .is_some_and(|s| s.eq_ignore_ascii_case(&needle))
        {
            return skip_until_gt(bytes, i + needle.len());
        }
        i += 1;
    }
    source.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app(src: &str) -> Vec<Finding> {
        check(Path::new("apps/demo/src/page.sail"), src)
    }

    #[test]
    fn flags_unsanctioned_component_token_attr() {
        let f = app(r#"<div button.bg="primary">x</div>"#);
        assert_eq!(f.len(), 1, "{f:?}");
        assert_eq!(f[0].rule, RULE_ID);
        assert!(
            f[0].message.contains("theme-owned") && f[0].message.contains("theme="),
            "{}",
            f[0].message
        );
    }

    #[test]
    fn flags_card_foreground_attr() {
        let f = check(
            Path::new("apps/demo/routes/page.sail"),
            r#"<column card.foreground="primary">x</column>"#,
        );
        assert!(!f.is_empty(), "{f:?}");
        assert!(f[0].message.contains("card.foreground"), "{}", f[0].message);
    }

    #[test]
    fn flags_multiline_unsanctioned_attr() {
        let f = app("<div\n  button.bg=\"primary\"\n>x</div>");
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(f[0].message.contains("button.bg"), "{}", f[0].message);
    }

    #[test]
    fn allows_theme_attr_and_tags() {
        let f = app(r#"<column theme="ship"><button>ok</button><card>x</card></column>"#);
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn packages_exempt() {
        let src = r#"<div button.bg="primary">x</div>"#;
        assert!(check(Path::new("packages/components/sail/x.sail"), src).is_empty());
        assert!(check(Path::new("crates/sail/compiler/stdlib/button.sail"), src).is_empty());
    }

    #[test]
    fn allows_local_style_page_const() {
        // Corpus idiom: file-local `style page = {…}` (doc-slider.sail and 90 peers).
        let f = app(r#"
<scope>
  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full).overflow(visible) }
</scope>
<column style(page)>x</column>
"#);
        assert!(f.is_empty(), "local style page must not wolf-cry: {f:?}");
    }

    #[test]
    fn allows_legacy_style_const_named_page() {
        let f = app(r#"
<scope>
  const page = style({ box: w(full) })
</scope>
<div style(page)>x</div>
"#);
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn allows_for_binding_named_page() {
        let f = app(r#"<for {items as page} key={page.id}><text>{page.name}</text></for>"#);
        assert!(f.is_empty(), "for binder is author-owned: {f:?}");
    }

    #[test]
    fn allows_match_binding_named_page() {
        let f = app(r#"<match {status()}>{ok as page => <text>{page}</text>}</match>"#);
        assert!(f.is_empty(), "match binder is author-owned: {f:?}");
    }

    #[test]
    fn allows_scope_destructure_named_page() {
        let f = app(r#"
<scope>
  props: { page: string, card: string }
</scope>
<text>{page}</text>
"#);
        assert!(f.is_empty(), "scope prop names are author-owned: {f:?}");
    }

    #[test]
    fn setter_arg_is_not_a_lint_attr_site() {
        // Style pipeline owns `border(page)` / `bg(button)` — this rule must
        // not invent a second finding from identifiers inside `style(…)`.
        let f = app(r#"<div style({ paint: border(page) })>x</div>"#);
        assert!(f.is_empty(), "setter args are not attr names: {f:?}");
    }

    #[test]
    fn scope_body_with_multibyte_does_not_panic() {
        let f = app(
            "<scope>\n  style page = { box: w(full) }\n  /** Parent width — host. */\n</scope>\n<div>x</div>",
        );
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn allows_semantic_prop_named_label() {
        let f = app(r#"<slider bind:value={sizeIdx} min="0" max="2" label="Size"></slider>"#);
        assert!(
            f.is_empty(),
            "label= is a semantic prop, not a token ref: {f:?}"
        );
    }

    #[test]
    fn still_flags_attr_after_style_page_const() {
        let f = app(r#"
<scope>
  style page = { box: w(full) }
</scope>
<div button.bg="primary">x</div>
"#);
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(f[0].message.contains("button.bg"), "{}", f[0].message);
    }
}
