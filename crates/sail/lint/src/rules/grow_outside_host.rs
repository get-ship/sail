//! `sail-grow-outside-host` — `size(grow)` shares leftover inside a host.
//!
//! `size(grow)` is participation, not a box. Leftover exists only as a child
//! of `<row>` `<column>` `<grid>` `<center>` `<stack>`. Outside a host it is
//! a compile error. Suggest-only FixPlan rewrites to `size(hug)`.

use std::collections::HashSet;
use std::path::Path;

use crate::report::{Finding, Severity};
use crate::rules::layout::{self, OpenTag};

pub const RULE_ID: &str = "sail-grow-outside-host";

const HOSTS: &[&str] = &["row", "column", "grid", "center", "stack"];

pub const MESSAGE: &str = "`size(grow)` shares leftover inside a host (`<row>` `<column>` `<grid>` `<center>` `<stack>`). This element is not a child of a host.";

pub const SUGGEST: &str = "Put this element inside a layout host, or rewrite to size(hug).";

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let grow_consts = grow_bearing_const_names(source);
    let mut findings = Vec::new();
    let mut stack: Vec<String> = Vec::new();
    for ev in walk_tags(source) {
        match ev {
            TagEvent::Open(tag) => {
                if spends_grow(&tag, &grow_consts) {
                    let parent = stack.last().map(String::as_str);
                    if !parent.is_some_and(|p| HOSTS.contains(&p)) {
                        let (line, col) = layout::offset_to_line_col(source, tag.start);
                        findings.push(Finding {
                            file: path.to_path_buf(),
                            line,
                            col,
                            rule: RULE_ID,
                            severity: Severity::Error,
                            message: MESSAGE.into(),
                            suggest: Some(SUGGEST.into()),
                        });
                    }
                }
                if !tag.self_closing && !is_void(&tag.tag) {
                    stack.push(tag.tag);
                }
            }
            TagEvent::Close(name) => {
                if let Some(i) = stack.iter().rposition(|t| t == &name) {
                    stack.truncate(i);
                }
            }
        }
    }
    findings
}

enum TagEvent {
    Open(OpenTag),
    Close(String),
}

fn walk_tags(source: &str) -> Vec<TagEvent> {
    let mut events: Vec<(usize, TagEvent)> = layout::scan_any_open_tags(source)
        .into_iter()
        .map(|t| (t.start, TagEvent::Open(t)))
        .collect();
    for (start, name) in scan_close_tags(source) {
        events.push((start, TagEvent::Close(name)));
    }
    events.sort_by_key(|(s, _)| *s);
    events.into_iter().map(|(_, e)| e).collect()
}

fn scan_close_tags(source: &str) -> Vec<(usize, String)> {
    let bytes = source.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if source[i..].starts_with("<!--") {
            match source[i..].find("-->") {
                Some(end) => {
                    i += end + 3;
                    continue;
                }
                None => break,
            }
        }
        if bytes[i] == b'<' && bytes.get(i + 1) == Some(&b'/') {
            let name_start = i + 2;
            let mut j = name_start;
            while j < bytes.len()
                && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'-' || bytes[j] == b'_')
            {
                j += 1;
            }
            if j > name_start {
                out.push((i, source[name_start..j].to_string()));
            }
            i = j;
            continue;
        }
        i += 1;
        while i < bytes.len() && (bytes[i] & 0xC0) == 0x80 {
            i += 1;
        }
    }
    out
}

fn spends_grow(tag: &OpenTag, grow_consts: &HashSet<String>) -> bool {
    let Some((_, sx)) = tag.attrs.iter().find(|(n, _)| n == "style") else {
        return false;
    };
    if sx_contains_size_grow(sx) {
        return true;
    }
    sx_bare_const_ref(sx).is_some_and(|name| grow_consts.contains(&name))
}

fn sx_contains_size_grow(sx: &str) -> bool {
    let bytes = sx.as_bytes();
    for (i, _) in sx.char_indices() {
        if sx[i..].starts_with("size") {
            let before_ok =
                i == 0 || (!bytes[i - 1].is_ascii_alphanumeric() && bytes[i - 1] != b'_');
            if !before_ok {
                continue;
            }
            let mut j = i + "size".len();
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            if bytes.get(j) != Some(&b'(') {
                continue;
            }
            j += 1;
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            if sx[j..].starts_with("grow") {
                let after = j + "grow".len();
                if after == bytes.len()
                    || (!bytes[after].is_ascii_alphanumeric() && bytes[after] != b'_')
                {
                    return true;
                }
            }
        }
    }
    false
}

fn sx_bare_const_ref(sx: &str) -> Option<String> {
    let mut s = sx.trim();
    if let Some(rest) = s.strip_prefix("style") {
        let rest = rest.trim_start();
        if let Some(inner) = rest.strip_prefix('(').and_then(|t| t.strip_suffix(')')) {
            s = inner.trim();
        } else {
            return None;
        }
    } else {
        return None;
    }
    if s.is_empty() {
        return None;
    }
    if !s
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
    {
        return None;
    }
    if s.chars().next()?.is_ascii_digit() {
        return None;
    }
    Some(s.to_string())
}

fn grow_bearing_const_names(source: &str) -> HashSet<String> {
    let mut out = HashSet::new();
    let bytes = source.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if (bytes[i] & 0xC0) == 0x80 {
            i += 1;
            continue;
        }
        if source[i..].starts_with("<!--") {
            match source[i..].find("-->") {
                Some(end) => {
                    i += end + 3;
                    continue;
                }
                None => break,
            }
        }
        if source[i..].starts_with("style") && (i == 0 || !bytes[i - 1].is_ascii_alphanumeric()) {
            let after = i + "style".len();
            if after < bytes.len() && bytes[after].is_ascii_whitespace() {
                let mut j = after;
                while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                    j += 1;
                }
                let name_start = j;
                while j < bytes.len()
                    && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_' || bytes[j] == b'$')
                {
                    j += 1;
                }
                if j > name_start {
                    let name = &source[name_start..j];
                    while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                        j += 1;
                    }
                    if bytes.get(j) == Some(&b'=') {
                        j += 1;
                        let init_start = j;
                        while j < bytes.len() && bytes[j] != b'\n' {
                            if (bytes[j] & 0xC0) != 0x80 && source[j..].starts_with("</scope>") {
                                break;
                            }
                            j += 1;
                        }
                        if sx_contains_size_grow(&source[init_start..j]) {
                            out.insert(name.to_string());
                        }
                        i = j;
                        continue;
                    }
                }
            }
        }
        i += 1;
        while i < bytes.len() && (bytes[i] & 0xC0) == 0x80 {
            i += 1;
        }
    }
    out
}

fn is_void(tag: &str) -> bool {
    matches!(
        tag,
        "br" | "hr" | "img" | "input" | "meta" | "link" | "area" | "col" | "wbr"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn run(src: &str) -> Vec<Finding> {
        check(Path::new("t.sail"), src)
    }

    #[test]
    fn flags_grow_on_root_column() {
        let f = run(r#"<column style({ size(grow) })>x</column>"#);
        assert_eq!(f.len(), 1, "{f:?}");
        assert_eq!(f[0].rule, RULE_ID);
        assert_eq!(f[0].message, MESSAGE);
    }

    #[test]
    fn flags_grow_inside_div() {
        let f = run(r#"<div><column style({ size(grow), bg(accent) })>x</column></div>"#);
        assert_eq!(f.len(), 1, "{f:?}");
    }

    #[test]
    fn allows_grow_inside_row() {
        let f = run(r#"
<row>
  <column style({ size(hug) })>hug</column>
  <column style({ size(grow) })>grow</column>
</row>
"#);
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn allows_named_grow_const_inside_host() {
        let f = run(r#"
<scope>
  style leftover = { size(grow), bg(accent) }
</scope>
<row>
  <column style(leftover)>grow</column>
</row>
"#);
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn flags_named_grow_const_outside_host() {
        let f = run(r#"
<scope>
  style leftover = { size(grow), bg(accent) }
</scope>
<column style(leftover)>grow</column>
"#);
        assert_eq!(f.len(), 1, "{f:?}");
    }

    #[test]
    fn quoted_trap_in_comment_is_ignored() {
        let f = run(r#"
<!--
        <column style({ size(grow) })>x</column>
-->
<row>
  <column style({ size(grow) })>grow</column>
</row>
"#);
        assert!(f.is_empty(), "{f:?}");
    }
}
