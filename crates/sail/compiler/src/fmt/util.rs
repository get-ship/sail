//! Shared helpers for the CST pretty-printer.

use crate::parse::html::{self, Attr};

use super::FormatOpts;

pub(super) fn looks_like_style_stmt(s: &str) -> bool {
    let t = s.trim_start();
    t.starts_with("style(") || t.starts_with("style (")
}

pub(super) fn print_attrs(attrs: &[Attr], depth: usize, opts: &FormatOpts, out: &mut String) {
    if attrs.is_empty() {
        return;
    }
    let mut single = String::new();
    for a in attrs {
        single.push(' ');
        write_attr(a, &mut single);
    }
    let open_budget = opts
        .max_width
        .saturating_sub(depth * opts.indent)
        .saturating_sub(4);
    let multiline = single.len() > open_budget || attrs.len() > 3;

    if !multiline {
        out.push_str(&single);
        return;
    }
    out.push('\n');
    for a in attrs {
        indent(out, depth + 1, opts);
        write_attr(a, out);
        out.push('\n');
    }
    indent(out, depth, opts);
}

pub(super) fn write_attr(a: &Attr, out: &mut String) {
    if a.name == html::PRIMARY_BRACE_ATTR {
        out.push_str(&a.value);
        return;
    }
    if a.value.is_empty() {
        out.push_str(&a.name);
        return;
    }
    let v = a.value.as_str();
    if is_brace_expr(v) {
        out.push_str(&a.name);
        out.push('=');
        let cleaned = v.replace("&quot;", "\"");
        out.push_str(&cleaned);
        return;
    }
    out.push_str(&a.name);
    out.push('=');
    if !v.contains('"') {
        out.push('"');
        out.push_str(v);
        out.push('"');
    } else if !v.contains('\'') {
        out.push('\'');
        out.push_str(v);
        out.push('\'');
    } else {
        out.push('"');
        out.push_str(&v.replace('"', "&quot;"));
        out.push('"');
    }
}

fn is_brace_expr(v: &str) -> bool {
    let t = v.trim();
    t.starts_with('{') && t.ends_with('}')
}

pub(super) fn indent(out: &mut String, depth: usize, opts: &FormatOpts) {
    for _ in 0..depth * opts.indent {
        out.push(' ');
    }
}

/// Strip the common leading whitespace from non-empty lines (spaces/tabs).
pub(super) fn dedent_block(s: &str) -> String {
    let mut min_indent: Option<usize> = None;
    for line in s.lines() {
        if line.chars().all(|c| c.is_ascii_whitespace()) {
            continue;
        }
        let n = line.chars().take_while(|c| *c == ' ' || *c == '\t').count();
        min_indent = Some(match min_indent {
            Some(m) => m.min(n),
            None => n,
        });
    }
    let skip = min_indent.unwrap_or(0);
    if skip == 0 {
        return s.trim_matches(|c| c == '\n' || c == '\r').to_string();
    }
    let mut out = String::with_capacity(s.len());
    for (i, line) in s.lines().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        if line.chars().all(|c| c.is_ascii_whitespace()) {
            continue;
        }
        let stripped: String = line.chars().skip(skip).collect();
        out.push_str(stripped.trim_end());
    }
    out
}

/// Keep intentional spaces; drop indent-only newlines between inline siblings.
pub(super) fn inline_text_piece(s: &str) -> String {
    if !s.contains('\n') {
        return s.to_string();
    }
    let parts: Vec<&str> = s.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
    if parts.is_empty() {
        String::new()
    } else {
        parts.join(" ")
    }
}
