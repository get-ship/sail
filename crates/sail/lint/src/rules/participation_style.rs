//! `sail-participation-style` — flex/grid **participation** is spelled on the
//! **box channel** (`style({ box: grow(1).minW(0) })`), not bare attrs.
//!
//! Bare `grow` / `shrink` / `w` / `min-w` / `max-w` on open tags are refused
//! with a pointer to the box channel. Layout-keyword **axis** attrs
//! (`gap` / `align` / `justify` on row|column|grid|…) stay structural sugar
//! (Phase 4–5 — not this rule).
//!
//! Severity `Error` — dual path closed for product after the codemod.

use std::path::Path;

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sail-participation-style";

/// Bare participation attr names (author surface — refused).
const ATTRS: &[(&str, &str)] = &[
    ("grow", "grow"),
    ("shrink", "shrink"),
    ("w", "w"),
    ("min-w", "minW"),
    ("max-w", "maxW"),
];

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let bytes = source.as_bytes();
    let mut findings = Vec::new();
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
        // Skip style/script/controller/scope bodies — attrs only on tags.
        if bytes[i] == b'<' && i + 1 < bytes.len() && bytes[i + 1].is_ascii_alphabetic() {
            let tag_start = i + 1;
            let mut tag_end = tag_start;
            while tag_end < bytes.len()
                && (bytes[tag_end].is_ascii_alphanumeric()
                    || bytes[tag_end] == b'-'
                    || bytes[tag_end] == b'_'
                    || bytes[tag_end] == b':')
            {
                tag_end += 1;
            }
            let mut k = tag_end;
            let mut quote: Option<u8> = None;
            let attr_region_start = k;
            while k < bytes.len() {
                match (quote, bytes[k]) {
                    (Some(q), b) if b == q => quote = None,
                    (None, b'"') | (None, b'\'') => quote = Some(bytes[k]),
                    (None, b'>') => break,
                    _ => {}
                }
                k += 1;
            }
            if k > attr_region_start {
                let region = &source[attr_region_start..k];
                scan_attrs(path, source, attr_region_start, region, &mut findings);
            }
            i = k.min(bytes.len().saturating_sub(1)).saturating_add(1);
            continue;
        }
        i += 1;
        while i < bytes.len() && (bytes[i] & 0xC0) == 0x80 {
            i += 1;
        }
    }
    findings
}

fn scan_attrs(
    path: &Path,
    source: &str,
    region_abs: usize,
    region: &str,
    findings: &mut Vec<Finding>,
) {
    let bytes = region.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // skip whitespace
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        // Control / value braces on tags: `<if {expr}>`, `attr={expr}` — skip
        // bare `{…}` groups so `{w.canExpand}` is not read as attr `w`.
        if bytes[i] == b'{' {
            let mut d = 0i32;
            while i < bytes.len() {
                if bytes[i] == b'{' {
                    d += 1;
                } else if bytes[i] == b'}' {
                    d -= 1;
                    i += 1;
                    if d == 0 {
                        break;
                    }
                    continue;
                }
                i += 1;
            }
            continue;
        }
        // attr name
        let name_start = i;
        while i < bytes.len()
            && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'-' || bytes[i] == b'_' || bytes[i] == b':')
        {
            i += 1;
        }
        if i == name_start {
            i += 1;
            continue;
        }
        let name = &region[name_start..i];
        // value?
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        let mut value = "";
        if i < bytes.len() && bytes[i] == b'=' {
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if i < bytes.len() && (bytes[i] == b'"' || bytes[i] == b'\'') {
                let q = bytes[i];
                i += 1;
                let vs = i;
                while i < bytes.len() && bytes[i] != q {
                    i += 1;
                }
                value = &region[vs..i];
                if i < bytes.len() {
                    i += 1;
                }
            } else if i < bytes.len() && bytes[i] == b'{' {
                let vs = i;
                let mut d = 0i32;
                while i < bytes.len() {
                    if bytes[i] == b'{' {
                        d += 1;
                    } else if bytes[i] == b'}' {
                        d -= 1;
                        if d == 0 {
                            i += 1;
                            break;
                        }
                    }
                    i += 1;
                }
                value = &region[vs..i];
            } else {
                let vs = i;
                while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'>' && bytes[i] != b'/'
                {
                    i += 1;
                }
                value = &region[vs..i];
            }
        }
        if let Some(&(attr, setter)) = ATTRS.iter().find(|&&(a, _)| a == name) {
            let abs = region_abs + name_start;
            let (line, col) = offset_to_line_col(source, abs);
            let example = match (attr, value) {
                ("grow", "") | ("grow", "1") | ("grow", "true") => "style({ box: grow(1) })".into(),
                ("grow", v) => format!("style({{ box: grow({v}) }})"),
                ("shrink", "") | ("shrink", "1") | ("shrink", "true") => {
                    "style({ box: shrink(1) })".into()
                }
                ("shrink", v) => format!("style({{ box: shrink({v}) }})"),
                ("w", v) if !v.is_empty() => format!("style({{ box: w({v}) }})"),
                ("min-w", v) if !v.is_empty() => format!("style({{ box: minW({v}) }})"),
                ("max-w", v) if !v.is_empty() => format!("style({{ box: maxW({v}) }})"),
                _ => format!("style({{ box: {setter}(…) }})"),
            };
            findings.push(Finding {
                file: path.to_path_buf(),
                line,
                col,
                rule: RULE_ID,
                severity: Severity::Error,
                message: format!(
                    "bare `{attr}` is layout participation — use the box channel: {example}"
                ),
                suggest: Some(format!(
                    "Move participation onto Style: attach `{example}` as a block statement \
                     (merge into an existing `style({{ box: … }})` when present). Layout-keyword \
                     axis attrs `gap` / `align` / `justify` stay on row|column|grid."
                )),
            });
        }
    }
}

fn offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 1usize;
    let mut col = 1usize;
    for (idx, b) in source.bytes().enumerate() {
        if idx == offset {
            break;
        }
        if b == b'\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(src: &str) -> Vec<Finding> {
        check(Path::new("t.sail"), src)
    }

    #[test]
    fn flags_bare_grow_and_shrink_attrs() {
        let f = run(r#"<div grow min-w="0">x</div>"#);
        assert!(f.iter().any(|x| x.message.contains("grow")), "{f:?}");
        assert!(f.iter().any(|x| x.message.contains("min-w")), "{f:?}");
        assert!(f.iter().all(|x| x.rule == RULE_ID));
    }

    #[test]
    fn flags_w_full_attr() {
        let f = run(r#"<row w="full" gap="2">x</row>"#);
        assert!(f.iter().any(|x| x.message.contains("`w`")), "{f:?}");
        // gap is axis sugar — not participation
        assert!(!f.iter().any(|x| x.message.contains("gap")), "{f:?}");
    }

    #[test]
    fn allows_box_channel_grow_and_paint() {
        assert!(run(
            r#"<div>
  style({ box: grow(1).minW(0), paint: bg(card) })
  x
</div>"#
        )
        .is_empty());
        assert!(run(
            r#"<row gap="2" align="center">
  style({ box: grow(1).w(full) })
  x
</row>"#
        )
        .is_empty());
    }

    #[test]
    fn skips_html_comments() {
        assert!(run(r#"<!-- grow min-w="0" --> <div>x</div>"#).is_empty());
    }

    #[test]
    fn skips_control_brace_expr_with_w_binding() {
        // `<if {w.canExpand}>` must not flag bare `w` (for-as binding).
        assert!(run(
            r#"<for {items() as w} key={w.key}>
  <if {w.canExpand}>
    <row>x</row>
  </if>
</for>"#
        )
        .is_empty());
    }
}
