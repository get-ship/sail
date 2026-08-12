//! Pretty-printer over the compiler HTML CST (`parse::html::Node`).
// FILE_CAP_OVERRIDE: one concern — document/element/inline layout of the CST.

use std::path::Path;

use crate::parse::html::{Element, Node};

use super::FormatOpts;
use super::oxfmt::format_ts_region;
use super::util::{
    dedent_block, indent, inline_text_piece, looks_like_style_stmt, print_attrs, write_attr,
};

const VOID_TAGS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source", "track",
    "wbr", "textarea",
];
const RAW_TS_TAGS: &[&str] = &["scope", "controller"];
const RAW_TEXT_TAGS: &[&str] = &["script", "style", "scope", "controller"];

pub(super) fn print_document(nodes: &[Node], opts: &FormatOpts, oxfmt: Option<&Path>) -> String {
    let mut out = String::new();
    let mut first = true;
    let mut prev_was_block = false;
    for n in nodes {
        if is_ignorable_ws(n) {
            continue;
        }
        let block = is_blockish(n);
        if !first && (block || prev_was_block) {
            // Blank line between top-level regions (scope / controller / template).
            if !out.ends_with("\n\n") {
                if !out.ends_with('\n') {
                    out.push('\n');
                }
                out.push('\n');
            }
        } else if !first && !out.ends_with('\n') {
            out.push('\n');
        }
        print_node(n, 0, opts, oxfmt, &mut out);
        if !out.ends_with('\n') {
            out.push('\n');
        }
        first = false;
        prev_was_block = block;
    }
    if !out.ends_with('\n') {
        out.push('\n');
    }
    while out.contains("\n\n\n") {
        out = out.replace("\n\n\n", "\n\n");
    }
    out
}

fn is_ignorable_ws(n: &Node) -> bool {
    matches!(n, Node::Text(t) if t.data.chars().all(|c| c.is_ascii_whitespace()))
}

fn is_blockish(n: &Node) -> bool {
    match n {
        Node::Element(_) | Node::Comment(_) => true,
        Node::Text(t) => !t.data.trim().is_empty(),
    }
}

fn print_node(
    node: &Node,
    depth: usize,
    opts: &FormatOpts,
    oxfmt: Option<&Path>,
    out: &mut String,
) {
    match node {
        Node::Comment(c) => {
            indent(out, depth, opts);
            out.push_str("<!--");
            out.push_str(&c.data);
            out.push_str("-->");
        }
        Node::Text(t) => print_text(&t.data, depth, opts, out),
        Node::Element(el) => print_element(el, depth, opts, oxfmt, out),
    }
}

fn print_element(
    el: &Element,
    depth: usize,
    opts: &FormatOpts,
    oxfmt: Option<&Path>,
    out: &mut String,
) {
    let tag = el.tag.as_str();
    let is_void = VOID_TAGS.contains(&tag);
    let is_raw_ts = RAW_TS_TAGS.contains(&tag);
    let is_raw_text = RAW_TEXT_TAGS.contains(&tag);

    let children: Vec<&Node> = el.children.iter().filter(|c| !is_ignorable_ws(c)).collect();

    indent(out, depth, opts);
    out.push('<');
    out.push_str(tag);
    print_attrs(&el.attributes, depth, opts, out);
    out.push('>');

    if is_void {
        return;
    }

    if children.is_empty() {
        out.push_str("</");
        out.push_str(tag);
        out.push('>');
        return;
    }

    if is_raw_ts {
        print_raw_ts_body(tag, &children, depth, opts, oxfmt, out);
        return;
    }

    if is_raw_text {
        let body = dedent_block(&raw_text_concat(&children));
        if body.trim().is_empty() {
            out.push_str("</");
            out.push_str(tag);
            out.push('>');
            return;
        }
        out.push('\n');
        for line in body.lines() {
            if line.is_empty() {
                out.push('\n');
                continue;
            }
            indent(out, depth + 1, opts);
            out.push_str(line.trim_end());
            out.push('\n');
        }
        indent(out, depth, opts);
        out.push_str("</");
        out.push_str(tag);
        out.push('>');
        return;
    }

    if can_inline(&children, depth, opts) {
        let only_text = children.len() == 1 && matches!(children[0], Node::Text(_));
        for child in &children {
            match child {
                Node::Text(t) => {
                    if only_text {
                        out.push_str(t.data.trim());
                    } else {
                        out.push_str(&inline_text_piece(&t.data));
                    }
                }
                Node::Element(e) => print_element_inline(e, out),
                Node::Comment(c) => {
                    out.push_str("<!--");
                    out.push_str(&c.data);
                    out.push_str("-->");
                }
            }
        }
        out.push_str("</");
        out.push_str(tag);
        out.push('>');
        return;
    }

    out.push('\n');
    for child in &children {
        print_node(child, depth + 1, opts, oxfmt, out);
        if !out.ends_with('\n') {
            out.push('\n');
        }
    }
    indent(out, depth, opts);
    out.push_str("</");
    out.push_str(tag);
    out.push('>');
}

fn can_inline(children: &[&Node], depth: usize, opts: &FormatOpts) -> bool {
    if children.is_empty() {
        return true;
    }
    let element_kids = children
        .iter()
        .filter(|c| matches!(c, Node::Element(_)))
        .count();
    if element_kids > 1 {
        return false;
    }
    let budget = opts.max_width.saturating_sub(depth * opts.indent);
    let mut len = 0usize;
    for child in children {
        match child {
            Node::Text(t) => {
                let t = t.data.trim();
                if t.is_empty() {
                    continue;
                }
                if t.contains('\n') || looks_like_style_stmt(t) {
                    return false;
                }
                len += t.len();
            }
            Node::Element(e) => {
                if e.children.iter().any(|c| matches!(c, Node::Element(_))) {
                    return false;
                }
                if RAW_TS_TAGS.contains(&e.tag.as_str()) || RAW_TEXT_TAGS.contains(&e.tag.as_str())
                {
                    return false;
                }
                len += e.tag.len() * 2 + 5;
                for a in &e.attributes {
                    len += a.name.len() + a.value.len() + 4;
                }
                for c in &e.children {
                    if let Node::Text(t) = c {
                        len += t.data.trim().len();
                    }
                }
            }
            Node::Comment(_) => return false,
        }
        if len > budget {
            return false;
        }
    }
    true
}

fn print_element_inline(el: &Element, out: &mut String) {
    let tag = el.tag.as_str();
    out.push('<');
    out.push_str(tag);
    for a in &el.attributes {
        out.push(' ');
        write_attr(a, out);
    }
    out.push('>');
    if VOID_TAGS.contains(&tag) {
        return;
    }
    let children: Vec<&Node> = el.children.iter().filter(|c| !is_ignorable_ws(c)).collect();
    for child in children {
        match child {
            Node::Text(t) => out.push_str(&inline_text_piece(&t.data)),
            Node::Element(e) => print_element_inline(e, out),
            Node::Comment(c) => {
                out.push_str("<!--");
                out.push_str(&c.data);
                out.push_str("-->");
            }
        }
    }
    out.push_str("</");
    out.push_str(tag);
    out.push('>');
}

fn print_raw_ts_body(
    tag: &str,
    children: &[&Node],
    depth: usize,
    opts: &FormatOpts,
    oxfmt: Option<&Path>,
    out: &mut String,
) {
    let raw = raw_text_concat(children);
    let trimmed = raw.trim_matches(|c| c == '\n' || c == '\r');
    if trimmed.is_empty() {
        out.push_str("</");
        out.push_str(tag);
        out.push('>');
        return;
    }

    let dedented = dedent_block(trimmed);
    let body = if let Some(bin) = oxfmt {
        let hint = if tag == "controller" {
            "controller.ts"
        } else {
            "scope.ts"
        };
        format_ts_region(bin, &dedented, hint).unwrap_or(dedented)
    } else {
        dedented
    };
    let body = body.trim_matches(|c| c == '\n' || c == '\r');

    out.push('\n');
    for line in body.lines() {
        if line.is_empty() {
            out.push('\n');
            continue;
        }
        indent(out, depth + 1, opts);
        out.push_str(line.trim_end());
        out.push('\n');
    }
    indent(out, depth, opts);
    out.push_str("</");
    out.push_str(tag);
    out.push('>');
}

fn raw_text_concat(children: &[&Node]) -> String {
    let mut s = String::new();
    for c in children {
        if let Node::Text(t) = c {
            s.push_str(&t.data);
        }
    }
    s
}

fn print_text(data: &str, depth: usize, opts: &FormatOpts, out: &mut String) {
    let trimmed = data.trim();
    if trimmed.is_empty() {
        return;
    }
    if trimmed.contains('\n') {
        for (i, line) in trimmed.lines().enumerate() {
            let line = line.trim_end();
            if line.is_empty() {
                if i > 0 {
                    out.push('\n');
                }
                continue;
            }
            if !out.is_empty() && !out.ends_with('\n') {
                out.push('\n');
            }
            indent(out, depth, opts);
            out.push_str(line.trim_start());
            out.push('\n');
        }
        if out.ends_with('\n') {
            out.pop();
        }
        return;
    }
    indent(out, depth, opts);
    out.push_str(trimmed);
}
