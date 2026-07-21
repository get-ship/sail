//! Per-element gates for scope `channels:`.
//!
//! Aggregate drift tests (channel_integrity / channel_discipline) cannot catch
//! a single broken lowerer path — spinner shipped `--bw:'control-stroke'` with
//! dropped size rekeys while other elements looked fine. Every stdlib element
//! that declares `channels:` must:
//!   1. emit each written custom property as `var(--token)` or a raw length
//!      (never a quoted token name),
//!   2. land every prop-axis value key as `:host([prop='value'])` (or the
//!      element-skin rewrite of that selector),
//!   3. have at least one reader of each written `--name` in structure sx or
//!      skin CSS (`var(--name)` / `(--name)`).
//!
//! A failure here means fix the lowerer or the element — not hand CSS.

use std::collections::BTreeSet;

use sail_compiler::compile_element;
use sail_compiler::stdlib;

/// Pull every `--ident:` write from lowered CSS (declarations only).
fn declared_custom_props(css: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let b = css.as_bytes();
    let mut i = 0;
    while i + 2 < b.len() {
        if b[i] == b'-' && b[i + 1] == b'-' {
            let start = i + 2;
            let mut j = start;
            while j < b.len() && (b[j].is_ascii_alphanumeric() || b[j] == b'-' || b[j] == b'_') {
                j += 1;
            }
            if j > start && j < b.len() && b[j] == b':' {
                out.insert(css[start..j].to_string());
            }
            i = j.max(i + 1);
        } else {
            i += 1;
        }
    }
    out
}

/// True if a declaration value for `--name` is a quoted string (broken token).
fn has_quoted_channel_value(css: &str) -> Vec<String> {
    let mut bad = Vec::new();
    for part in css.split('}') {
        for decl in part.split(';') {
            let t = decl.trim();
            if let Some(rest) = t.strip_prefix("--") {
                if let Some((name, val)) = rest.split_once(':') {
                    let val = val.trim();
                    if (val.starts_with('\'') || val.starts_with('"'))
                        && !val.starts_with("var(")
                    {
                        bad.push(format!("--{name}:{val}"));
                    }
                }
            }
        }
    }
    bad
}

/// Layout axes allowed inside channel decl maps (not custom props).
const LAYOUT_AXES: &[&str] = &[
    "gap",
    "direction",
    "align",
    "justify",
    "width",
    "height",
    "min-width",
    "min-height",
    "max-width",
    "max-height",
    "display",
];

/// Channel custom-prop keys written in a `channels:` map. Walks the nested
/// structure so prop-axis value names (`sm`, `solid`, `inline`) are not
/// mistaken for `--sm` / `--solid` writes.
fn channel_keys_in_source(src: &str) -> BTreeSet<String> {
    // Prefer the real scope member, not a comment that mentions "channels:".
    let start = src
        .find("\n  channels:")
        .or_else(|| src.find("\n\tchannels:"))
        .or_else(|| src.find("channels: {"))
        .or_else(|| src.find("channels:{"));
    let Some(start) = start else {
        return BTreeSet::new();
    };
    let rest = &src[start..];
    let Some(rel) = rest.find("channels:") else {
        return BTreeSet::new();
    };
    let rest = &rest[rel + "channels:".len()..];
    let Some(open) = rest.find('{') else {
        return BTreeSet::new();
    };
    let Some(body) = brace_slice(rest, open) else {
        return BTreeSet::new();
    };
    let mut keys = BTreeSet::new();
    collect_channel_decl_keys(body, &mut keys);
    keys
}

/// `s[open]` is `{`; return the full `{…}` slice including braces.
fn brace_slice(s: &str, open: usize) -> Option<&str> {
    let bytes = s.as_bytes();
    if open >= bytes.len() || bytes[open] != b'{' {
        return None;
    }
    let mut depth = 0i32;
    for (i, &c) in bytes[open..].iter().enumerate() {
        match c {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&s[open..=open + i]);
                }
            }
            _ => {}
        }
    }
    None
}

/// Split a `{…}` body into top-level `key: value` pairs (comma or newline).
fn object_entries(obj: &str) -> Vec<(String, String)> {
    let inner = obj
        .trim()
        .trim_start_matches('{')
        .trim_end_matches('}')
        .trim();
    let mut entries = Vec::new();
    let mut depth = 0i32;
    let mut in_str: Option<char> = None;
    let mut cur = String::new();
    for c in inner.chars() {
        match in_str {
            Some(q) => {
                cur.push(c);
                if c == q {
                    in_str = None;
                }
            }
            None => match c {
                '\'' | '"' => {
                    in_str = Some(c);
                    cur.push(c);
                }
                '{' | '[' => {
                    depth += 1;
                    cur.push(c);
                }
                '}' | ']' => {
                    depth -= 1;
                    cur.push(c);
                }
                ',' | '\n' if depth == 0 => {
                    if let Some(e) = split_entry(cur.trim()) {
                        entries.push(e);
                    }
                    cur.clear();
                }
                _ => cur.push(c),
            },
        }
    }
    if let Some(e) = split_entry(cur.trim()) {
        entries.push(e);
    }
    entries
}

fn split_entry(entry: &str) -> Option<(String, String)> {
    let entry = entry.trim();
    if entry.is_empty() {
        return None;
    }
    // Find top-level colon.
    let mut depth = 0i32;
    let mut in_str: Option<char> = None;
    for (i, c) in entry.char_indices() {
        match in_str {
            Some(q) => {
                if c == q {
                    in_str = None;
                }
            }
            None => match c {
                '\'' | '"' => in_str = Some(c),
                '{' | '[' => depth += 1,
                '}' | ']' => depth -= 1,
                ':' if depth == 0 => {
                    let k = entry[..i].trim().trim_matches(|c| c == '\'' || c == '"');
                    let v = entry[i + 1..].trim();
                    return Some((k.to_string(), v.to_string()));
                }
                _ => {}
            },
        }
    }
    None
}

fn unquote_val(v: &str) -> &str {
    let v = v.trim();
    if v.len() >= 2
        && ((v.starts_with('\'') && v.ends_with('\'')) || (v.starts_with('"') && v.ends_with('"')))
    {
        &v[1..v.len() - 1]
    } else {
        v
    }
}

/// Recurse: axis maps nest objects; decl maps have string/token values.
fn collect_channel_decl_keys(obj: &str, keys: &mut BTreeSet<String>) {
    for (k, v) in object_entries(obj) {
        let vt = v.trim();
        if vt.starts_with('{') {
            // Nested object: either an axis (`size: { sm: {…} }`) or a decl map.
            // If any nested value is itself an object, treat this as an axis map
            // and recurse into each value; otherwise these are decl keys.
            let nested = object_entries(vt);
            let is_axis = nested.iter().any(|(_, nv)| nv.trim().starts_with('{'));
            if is_axis {
                for (_, nv) in nested {
                    if nv.trim().starts_with('{') {
                        collect_channel_decl_keys(&nv, keys);
                    }
                }
            } else {
                // Decl map: keys are custom props (or layout axes).
                for (dk, dv) in nested {
                    if LAYOUT_AXES.contains(&dk.as_str()) {
                        continue;
                    }
                    if dk.starts_with('.') || dk.starts_with('#') || dk.starts_with(':') {
                        continue;
                    }
                    // Skip empty / non-ident.
                    if dk
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
                        && !dk.is_empty()
                    {
                        let _ = unquote_val(&dv); // value exists
                        keys.insert(dk);
                    }
                }
            }
        } else if k.starts_with('.') || k.starts_with('#') || k.contains(' ') {
            // Descendant selector key with a decl map value already handled above
            // when value is `{…}`. Bare selector without object — ignore.
        }
    }
}

fn elements_with_channels() -> Vec<(&'static str, &'static str)> {
    stdlib::sources()
        .iter()
        .copied()
        .filter(|(_, src)| src.contains("channels:"))
        .collect()
}

#[test]
fn every_channels_element_is_registered() {
    let list = elements_with_channels();
    assert!(
        list.len() >= 6,
        "expected several stdlib channels: users, got {} — {:?}",
        list.len(),
        list.iter().map(|(n, _)| *n).collect::<Vec<_>>()
    );
}

#[test]
fn per_element_channels_lower_clean_and_complete() {
    let mut failures: Vec<String> = Vec::new();

    for (name, src) in elements_with_channels() {
        let out = match compile_element(src, name) {
            Ok(o) => o,
            Err(e) => {
                failures.push(format!("{name}: compile failed: {e:?}"));
                continue;
            }
        };
        let css = &out.skin_css;
        // Parent host may write channels that child elements spend (tabs→tab,
        // toggle-group→toggle). Scan the whole stdlib for readers.
        let stdlib_blob: String = stdlib::sources()
            .iter()
            .map(|(_, s)| *s)
            .collect::<Vec<_>>()
            .join("\n");
        let combined = format!("{css}\n{}\n{stdlib_blob}", out.structure_html);

        // 1. No quoted token values on custom props.
        let quoted = has_quoted_channel_value(css);
        if !quoted.is_empty() {
            failures.push(format!(
                "{name}: quoted channel values (token names left as strings):\n  {}",
                quoted.join("\n  ")
            ));
        }

        // 2. Every channels: custom-prop key writes a declaration.
        let written = declared_custom_props(css);
        let expected = channel_keys_in_source(src);
        let missing: Vec<_> = expected
            .iter()
            .filter(|k| !written.contains(k.as_str()))
            .cloned()
            .collect();
        if !missing.is_empty() {
            failures.push(format!(
                "{name}: channels: keys never declared in skin:\n  --{}\n  written: {:?}",
                missing.join("\n  --"),
                written
            ));
        }

        // 3. Each written channel has a reader somewhere in stdlib (host, child, or skin).
        let mut unread = Vec::new();
        for k in &expected {
            let needle_var = format!("var(--{k})");
            let needle_ref = format!("(--{k})");
            if !combined.contains(&needle_var) && !combined.contains(&needle_ref) {
                unread.push(k.clone());
            }
        }
        if !unread.is_empty() {
            failures.push(format!(
                "{name}: channel writes with no reader in structure/skin/stdlib:\n  --{}",
                unread.join("\n  --")
            ));
        }

        // 4. Size axis rekeys under the real channels: block.
        if let Some(idx) = src.find("\n  channels:").or_else(|| src.find("channels: {")) {
            let channels_slice = &src[idx..];
            let end = channels_slice.find("</scope>").unwrap_or(channels_slice.len());
            let channels_slice = &channels_slice[..end];
            if channels_slice.contains("size:")
                && (channels_slice.contains("sm:")
                    || channels_slice.contains("lg:")
                    || channels_slice.contains("xs:"))
            {
                let has_size_sel = css.contains("[size='sm']")
                    || css.contains("[size='lg']")
                    || css.contains("[size='xs']")
                    || css.contains("[size=\"sm\"]");
                if !has_size_sel {
                    failures.push(format!(
                        "{name}: channels: has size axis but skin has no [size=…] rekey"
                    ));
                }
            }
        }
    }

    assert!(
        failures.is_empty(),
        "per-element channel gates failed (fix lowerer, do not patch with hand CSS):\n\n{}",
        failures.join("\n\n")
    );
}
