//! `sui-attr-bool-prop` — boolean props on registry components must be bound
//! with `prop:`, not `attr:`.
//!
//! `attr:` stringifies its value and sets it as an HTML attribute. For a
//! reflected boolean prop that inverts the intent: `attr:disabled={false}`
//! reflects `disabled="false"`, and an attribute that is *present* (even with
//! value `"false"`) reads as truthy — so the component is wrongly disabled.
//! `prop:disabled={false}` passes the live boolean to the property.
//!
//! Scope and limits:
//! - Only `attr:NAME={…}` bindings on tags whose registry entry
//!   (`primitives.json`) declares `NAME` with `type="bool"` are flagged.
//! - Native elements (`<option>`, `<button>`, `<input>`, `<details>`…) are not
//!   in the registry, so their `attr:` bindings (correct there) never trip it.
//! - Unknown tags and non-bool props are left alone.
//! - HTML comments are skipped (primitives embed example tags in docblocks).

use std::path::Path;

use crate::prop_registry::registry;
use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sail-attr-bool-prop";

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let reg = registry();
    let bytes = source.as_bytes();
    let mut findings = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        // Skip HTML comments wholesale.
        if bytes[i] == b'<' && bytes.get(i + 1..i + 4) == Some(b"!--") {
            match find_subslice(&bytes[i + 4..], b"-->") {
                Some(end) => {
                    i += 4 + end + 3;
                    continue;
                }
                None => break,
            }
        }
        // Any element opening tag; the registry lookup below is the filter
        // (stdlib tags are bare — `alert`, `badge` — and unknown tags miss).
        if !(bytes[i] == b'<' && bytes.get(i + 1).is_some_and(|b| b.is_ascii_lowercase())) {
            i += 1;
            continue;
        }
        let name_start = i + 1;
        let mut j = name_start;
        while j < bytes.len() && is_tag_name_byte(bytes[j]) {
            j += 1;
        }
        let tag = &source[name_start..j];
        let Some(end_rel) = find_byte(&bytes[j..], b'>') else { break };
        let tag_end = j + end_rel;
        let props = reg.get(tag);
        if let Some(props) = props {
            scan_attr_bool(source, j, tag_end, tag, props, &mut findings, path);
        }
        i = tag_end + 1;
    }
    findings
}

/// Scan an opening tag's interior `[start, end)` for `attr:NAME={…}` bindings
/// whose `NAME` is a registry bool prop on `tag`.
fn scan_attr_bool(
    source: &str,
    start: usize,
    end: usize,
    tag: &str,
    props: &std::collections::BTreeMap<String, crate::prop_registry::PropRule>,
    findings: &mut Vec<Finding>,
    path: &Path,
) {
    let bytes = source.as_bytes();
    let mut i = start;
    while i + 5 <= end {
        if &bytes[i..i + 5] != b"attr:" {
            i += 1;
            continue;
        }
        let prop_start = i + 5;
        let mut k = prop_start;
        while k < end && is_attr_name_byte(bytes[k]) {
            k += 1;
        }
        let prop = &source[prop_start..k];
        // Must be a brace binding (`attr:x={…}`) — skip static `attr:x="…"`.
        let mut m = k;
        while m < end && bytes[m].is_ascii_whitespace() {
            m += 1;
        }
        let is_binding = m < end && bytes[m] == b'=' && {
            let mut n = m + 1;
            while n < end && bytes[n].is_ascii_whitespace() {
                n += 1;
            }
            n < end && bytes[n] == b'{'
        };
        if is_binding
            && props.get(prop).and_then(|r| r.ty.as_deref()) == Some("bool")
        {
            let (line, col) = byte_offset_to_line_col(source, i);
            findings.push(Finding {
                file: path.to_path_buf(),
                line,
                col,
                rule: RULE_ID,
                severity: Severity::Error,
                message: format!(
                    "`attr:{prop}` on <{tag}> stringifies a boolean prop — \
                     attr:{prop}={{false}} reflects {prop}=\"false\" (truthy). \
                     Bind the boolean property with `prop:{prop}=`."
                ),
                suggest: Some(format!("Replace `attr:{prop}=` with `prop:{prop}=`.")),
            });
        }
        i = k.max(i + 1);
    }
}

fn is_tag_name_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-'
}

fn is_attr_name_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'_'
}

fn find_byte(haystack: &[u8], needle: u8) -> Option<usize> {
    haystack.iter().position(|&b| b == needle)
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

fn byte_offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let bytes = source.as_bytes();
    let (mut line, mut col) = (1usize, 1usize);
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
