//! IR node index with SailInspect-compatible node ids.

use std::collections::BTreeMap;

use crate::diagnostics::Span;
use crate::ir::{AttrInstr, Instr};
use crate::parse;

/// Indexed authoring node (element or control-flow host).
#[derive(Debug, Clone)]
pub struct EditNode {
    pub id: String,
    pub tag: String,
    /// Open-tag span in author source `[start, end)`.
    pub open_span: Span,
    /// Full element span including children and close tag (when known).
    pub full_span: Span,
    /// Parent node id (empty for roots).
    pub parent_id: Option<String>,
    /// Immediate child node ids (document order).
    pub children: Vec<String>,
    /// Static attrs only (name → value).
    pub static_attrs: BTreeMap<String, String>,
}

/// Content-hash model revision (stable for identical bytes).
pub fn model_revision(source: &str) -> String {
    format!("{:08x}", fnv1a32(source.as_bytes()))
}

fn fnv1a32(bytes: &[u8]) -> u32 {
    let mut h: u32 = 0x811c_9dc5;
    for b in bytes {
        h ^= u32::from(*b);
        h = h.wrapping_mul(0x0100_0193);
    }
    h
}

fn short_hash(input: &str) -> String {
    format!("{:08x}", fnv1a32(input.as_bytes()))
}

fn node_id(file: &str, ordinal_path: &str, content: &str) -> String {
    let h = short_hash(content);
    format!("{file}#{ordinal_path}#{h}")
}

/// Build node index for `source`. `file` is the identity path fragment used
/// in node ids (workspace-relative path or `_inline`).
pub fn build_node_index(source: &str, file: &str) -> Result<BTreeMap<String, EditNode>, String> {
    let program = parse::parse_author(source).map_err(|e| format!("parse: {e}"))?;
    let mut b = IndexBuild {
        source,
        file,
        nodes: BTreeMap::new(),
        tag_occ: BTreeMap::new(),
    };
    let _roots = b.walk_block(&program.template, "", None);
    Ok(b.nodes)
}

struct IndexBuild<'a> {
    source: &'a str,
    file: &'a str,
    nodes: BTreeMap<String, EditNode>,
    tag_occ: BTreeMap<String, usize>,
}

impl<'a> IndexBuild<'a> {
    fn walk_block(
        &mut self,
        instrs: &[Instr],
        path_prefix: &str,
        parent_id: Option<String>,
    ) -> Vec<String> {
        let mut child_ids = Vec::new();
        let mut ordinal = 0usize;
        for instr in instrs {
            if let Some(id) = self.walk_instr(instr, path_prefix, ordinal, parent_id.clone()) {
                child_ids.push(id);
                ordinal += 1;
            }
        }
        child_ids
    }

    fn walk_instr(
        &mut self,
        instr: &Instr,
        path_prefix: &str,
        ordinal: usize,
        parent_id: Option<String>,
    ) -> Option<String> {
        let path = if path_prefix.is_empty() {
            format!("{ordinal}")
        } else {
            format!("{path_prefix}.{ordinal}")
        };
        match instr {
            Instr::Element(el) => {
                let mut static_attrs = BTreeMap::new();
                for a in &el.attrs {
                    if let AttrInstr::Static { name, value, .. } = a {
                        static_attrs.insert(name.clone(), value.clone());
                    }
                }
                let open_span = self.author_open_span(&el.tag);
                let full_span =
                    full_element_span(self.source, open_span, &el.tag).unwrap_or(open_span);
                let content = format!(
                    "{}|{}",
                    el.tag,
                    static_attrs
                        .iter()
                        .map(|(k, v)| format!("{k}={v}"))
                        .collect::<Vec<_>>()
                        .join("&")
                );
                let id = node_id(self.file, &path, &content);
                let children = self.walk_block(&el.children, &path, Some(id.clone()));
                self.nodes.insert(
                    id.clone(),
                    EditNode {
                        id: id.clone(),
                        tag: el.tag.clone(),
                        open_span,
                        full_span,
                        parent_id,
                        children,
                        static_attrs,
                    },
                );
                Some(id)
            }
            Instr::If(i) => {
                let content = format!(
                    "if|{}",
                    i.branches
                        .first()
                        .map(|b| b.cond.source.as_str())
                        .unwrap_or("")
                );
                let id = node_id(self.file, &path, &content);
                let open_span = self.author_open_span("if");
                let full_span =
                    full_element_span(self.source, open_span, "if").unwrap_or(open_span);
                let mut kids = Vec::new();
                for (bi, br) in i.branches.iter().enumerate() {
                    kids.extend(self.walk_block(
                        &br.body,
                        &format!("{path}.b{bi}"),
                        Some(id.clone()),
                    ));
                }
                if let Some(else_body) = &i.else_body {
                    kids.extend(self.walk_block(
                        else_body,
                        &format!("{path}.else"),
                        Some(id.clone()),
                    ));
                }
                self.nodes.insert(
                    id.clone(),
                    EditNode {
                        id: id.clone(),
                        tag: "if".into(),
                        open_span,
                        full_span,
                        parent_id,
                        children: kids,
                        static_attrs: BTreeMap::new(),
                    },
                );
                Some(id)
            }
            Instr::For(f) => {
                let content = format!("for|{}|{}", f.var_name, f.iterable.source);
                let id = node_id(self.file, &path, &content);
                let open_span = self.author_open_span("for");
                let full_span =
                    full_element_span(self.source, open_span, "for").unwrap_or(open_span);
                let mut kids = self.walk_block(&f.body, &path, Some(id.clone()));
                if let Some(else_body) = &f.else_body {
                    kids.extend(self.walk_block(
                        else_body,
                        &format!("{path}.else"),
                        Some(id.clone()),
                    ));
                }
                self.nodes.insert(
                    id.clone(),
                    EditNode {
                        id: id.clone(),
                        tag: "for".into(),
                        open_span,
                        full_span,
                        parent_id,
                        children: kids,
                        static_attrs: BTreeMap::new(),
                    },
                );
                Some(id)
            }
            Instr::Match(m) => {
                let content = format!("match|{}", m.discriminant.source);
                let id = node_id(self.file, &path, &content);
                let open_span = self.author_open_span("match");
                let full_span =
                    full_element_span(self.source, open_span, "match").unwrap_or(open_span);
                let mut kids = Vec::new();
                for (ci, c) in m.cases.iter().enumerate() {
                    kids.extend(self.walk_block(
                        &c.body,
                        &format!("{path}.c{ci}"),
                        Some(id.clone()),
                    ));
                }
                if let Some(d) = &m.default {
                    kids.extend(self.walk_block(d, &format!("{path}.default"), Some(id.clone())));
                }
                self.nodes.insert(
                    id.clone(),
                    EditNode {
                        id: id.clone(),
                        tag: "match".into(),
                        open_span,
                        full_span,
                        parent_id,
                        children: kids,
                        static_attrs: BTreeMap::new(),
                    },
                );
                Some(id)
            }
            Instr::When(w) => {
                let content = format!("when|{}", w.token);
                let id = node_id(self.file, &path, &content);
                let open_span = self.author_open_span("when");
                let full_span =
                    full_element_span(self.source, open_span, "when").unwrap_or(open_span);
                let mut kids = self.walk_block(&w.body, &path, Some(id.clone()));
                if let Some(e) = &w.else_body {
                    kids.extend(self.walk_block(e, &format!("{path}.else"), Some(id.clone())));
                }
                self.nodes.insert(
                    id.clone(),
                    EditNode {
                        id: id.clone(),
                        tag: "when".into(),
                        open_span,
                        full_span,
                        parent_id,
                        children: kids,
                        static_attrs: BTreeMap::new(),
                    },
                );
                Some(id)
            }
            Instr::WhileLoading(wl) => {
                let content = format!("while-loading|{}", wl.bind);
                let id = node_id(self.file, &path, &content);
                let open_span = self.author_open_span("while-loading");
                let full_span =
                    full_element_span(self.source, open_span, "while-loading").unwrap_or(open_span);
                let kids = self.walk_block(&wl.body, &path, Some(id.clone()));
                self.nodes.insert(
                    id.clone(),
                    EditNode {
                        id: id.clone(),
                        tag: "while-loading".into(),
                        open_span,
                        full_span,
                        parent_id,
                        children: kids,
                        static_attrs: BTreeMap::new(),
                    },
                );
                Some(id)
            }
            Instr::WhileError(we) => {
                let content = format!("while-error|{}", we.bind);
                let id = node_id(self.file, &path, &content);
                let open_span = self.author_open_span("while-error");
                let full_span =
                    full_element_span(self.source, open_span, "while-error").unwrap_or(open_span);
                let kids = self.walk_block(&we.body, &path, Some(id.clone()));
                self.nodes.insert(
                    id.clone(),
                    EditNode {
                        id: id.clone(),
                        tag: "while-error".into(),
                        open_span,
                        full_span,
                        parent_id,
                        children: kids,
                        static_attrs: BTreeMap::new(),
                    },
                );
                Some(id)
            }
            Instr::StaticText { .. } | Instr::DynamicText { .. } | Instr::Comment { .. } => None,
        }
    }

    fn author_open_span(&mut self, tag: &str) -> Span {
        let key = tag.to_ascii_lowercase();
        let n = *self.tag_occ.get(&key).unwrap_or(&0);
        self.tag_occ.insert(key.clone(), n + 1);
        if let Some((s, e)) = find_nth_open_tag(self.source, &key, n) {
            Span::new(s, e)
        } else {
            Span::new(0, 0)
        }
    }
}

fn find_nth_open_tag(source: &str, tag: &str, n: usize) -> Option<(usize, usize)> {
    let lower = source.to_ascii_lowercase();
    let needle = format!("<{tag}");
    let mut from = 0usize;
    let mut found = 0usize;
    while let Some(rel) = lower[from..].find(&needle) {
        let start = from + rel;
        let after = start + needle.len();
        let ok = after >= source.len()
            || matches!(
                source.as_bytes()[after],
                b'>' | b'/' | b' ' | b'\n' | b'\t' | b'\r'
            );
        if ok {
            if found == n {
                let end = source[start..]
                    .find('>')
                    .map(|i| start + i + 1)
                    .unwrap_or(after);
                return Some((start, end));
            }
            found += 1;
        }
        from = after;
    }
    None
}

/// Full element range from open-tag start through matching close (or self-close).
pub(crate) fn full_element_span(source: &str, open: Span, tag: &str) -> Option<Span> {
    if open.end == 0 && open.start == 0 && !source.is_empty() {
        // Unresolved open tag.
        return None;
    }
    let open_text = source.get(open.start..open.end)?;
    if open_text.trim_end().ends_with("/>") {
        return Some(open);
    }
    // Void-ish self close already handled; walk for matching close.
    let close = format!("</{tag}");
    let close_lower = close.to_ascii_lowercase();
    let open_needle = format!("<{tag}").to_ascii_lowercase();
    let lower = source.to_ascii_lowercase();
    let mut depth = 1i32;
    let mut i = open.end;
    while i < source.len() {
        if lower[i..].starts_with(&close_lower) {
            let after = i + close_lower.len();
            let boundary_ok = after >= source.len()
                || matches!(
                    source.as_bytes()[after],
                    b'>' | b' ' | b'\n' | b'\t' | b'\r'
                );
            if boundary_ok {
                depth -= 1;
                if depth == 0 {
                    let end = source[i..].find('>').map(|j| i + j + 1).unwrap_or(after);
                    return Some(Span::new(open.start, end));
                }
            }
        } else if lower[i..].starts_with(&open_needle) {
            let after = i + open_needle.len();
            let boundary_ok = after >= source.len()
                || matches!(
                    source.as_bytes()[after],
                    b'>' | b'/' | b' ' | b'\n' | b'\t' | b'\r'
                );
            if boundary_ok {
                // Self-closing open does not increase depth.
                let tag_end = source[i..].find('>').map(|j| i + j + 1).unwrap_or(after);
                let slice = &source[i..tag_end];
                if !slice.trim_end().ends_with("/>") {
                    depth += 1;
                }
                i = tag_end;
                continue;
            }
        }
        i += 1;
        // skip UTF-8 continuation
        while i < source.len() && (source.as_bytes()[i] & 0xC0) == 0x80 {
            i += 1;
        }
    }
    // Unclosed: use open only.
    Some(open)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indexes_simple_column() {
        let src = r#"<column gap="4"><div>x</div></column>"#;
        let idx = build_node_index(src, "t.sail").unwrap();
        assert!(idx.len() >= 2);
        let col = idx.values().find(|n| n.tag == "column").unwrap();
        assert_eq!(col.children.len(), 1);
        let full = &src[col.full_span.start..col.full_span.end];
        assert!(full.starts_with("<column"));
        assert!(full.ends_with("</column>"));
    }

    #[test]
    fn model_revision_stable() {
        assert_eq!(model_revision("a"), model_revision("a"));
        assert_ne!(model_revision("a"), model_revision("b"));
    }
}
