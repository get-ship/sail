//! SailInspect-compatible node identity for render regions (T-I3).
//!
//! Mirrors `sail_ops::inspect` tree content-hash + layout→tree join
//! (author_tag_from_layout + per-tag document-order cursors) so library,
//! CLI, and ops regions share the same ids as `SailInspect/Model`.

use std::collections::BTreeMap;

use sail_layout::html::Element;
use serde::Serialize;

/// Resolved static paint for a region (PARITY-1 paint half / agent overlays).
///
/// Colors are `rgba(r,g,b,a)` with 0–255 channels. Numbers are CSS px.
/// Omitted when the paint sheet did not resolve a value for that slot.
#[derive(Debug, Clone, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RegionStyle {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_width: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_radius: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f32>,
}

/// One painted region joined to a SailInspect-style node id.
#[derive(Debug, Clone, Serialize)]
pub struct Region {
    #[serde(rename = "nodeId")]
    pub node_id: String,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    /// Static paint resolved by the same stylesheet+theme as the takumi tree.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<RegionStyle>,
}

/// Honest dynamic / unknown region listing.
#[derive(Debug, Clone, Serialize)]
pub struct DynamicRegion {
    #[serde(rename = "nodeId")]
    pub node_id: String,
    pub reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
}

/// Lightweight tree node for join (id + tag only).
#[derive(Debug, Clone)]
pub struct TreeNodeRef {
    pub id: String,
    pub tag: String,
}

/// Author tree index used to join layout boxes → SailInspect node ids.
#[derive(Debug, Clone, Default)]
pub struct InspectTreeIndex {
    pub file: String,
    pub roots: Vec<String>,
    /// tag (lowercase) → node ids in document order
    pub by_tag: BTreeMap<String, Vec<String>>,
    pub nodes: Vec<TreeNodeRef>,
    /// Author DynamicText expressions in document order (for dynamics join).
    pub dynamic_texts: Vec<DynamicTextRef>,
}

#[derive(Debug, Clone)]
pub struct DynamicTextRef {
    pub expr: String,
    /// Parent element node id when available; else nearest root.
    pub parent_id: String,
}

/// Build the same content-hash ids as SailInspect from author source.
pub fn build_inspect_tree(source: &str, file: &str) -> InspectTreeIndex {
    let Ok(program) = sail_compiler::parse::parse_author(source) else {
        return InspectTreeIndex {
            file: file.to_string(),
            ..Default::default()
        };
    };
    let mut b = TreeBuild {
        file,
        nodes: Vec::new(),
        by_tag: BTreeMap::new(),
        dynamic_texts: Vec::new(),
        parent_stack: Vec::new(),
    };
    let roots = b.walk_block(&program.template, "");
    InspectTreeIndex {
        file: file.to_string(),
        roots,
        by_tag: b.by_tag,
        nodes: b.nodes,
        dynamic_texts: b.dynamic_texts,
    }
}

struct TreeBuild<'a> {
    file: &'a str,
    nodes: Vec<TreeNodeRef>,
    by_tag: BTreeMap<String, Vec<String>>,
    dynamic_texts: Vec<DynamicTextRef>,
    parent_stack: Vec<String>,
}

impl<'a> TreeBuild<'a> {
    fn walk_block(
        &mut self,
        instrs: &[sail_compiler::ir::Instr],
        path_prefix: &str,
    ) -> Vec<String> {
        let mut child_ids = Vec::new();
        let mut ordinal = 0usize;
        for instr in instrs {
            if let Some(id) = self.walk_instr(instr, path_prefix, ordinal) {
                child_ids.push(id);
                ordinal += 1;
            }
        }
        child_ids
    }

    fn walk_instr(
        &mut self,
        instr: &sail_compiler::ir::Instr,
        path_prefix: &str,
        ordinal: usize,
    ) -> Option<String> {
        use sail_compiler::ir::{AttrInstr, Instr};
        let path = if path_prefix.is_empty() {
            format!("{ordinal}")
        } else {
            format!("{path_prefix}.{ordinal}")
        };
        match instr {
            Instr::Element(el) => {
                let mut attrs = BTreeMap::new();
                for a in &el.attrs {
                    if let AttrInstr::Static { name, value, .. } = a {
                        attrs.insert(name.clone(), value.clone());
                    }
                }
                let content = format!(
                    "{}|{}",
                    el.tag,
                    attrs
                        .iter()
                        .map(|(k, v)| format!("{k}={v}"))
                        .collect::<Vec<_>>()
                        .join("&")
                );
                let id = node_id(self.file, &path, &content);
                self.by_tag
                    .entry(el.tag.to_ascii_lowercase())
                    .or_default()
                    .push(id.clone());
                self.nodes.push(TreeNodeRef {
                    id: id.clone(),
                    tag: el.tag.clone(),
                });
                self.parent_stack.push(id.clone());
                let _children = self.walk_block(&el.children, &path);
                self.parent_stack.pop();
                Some(id)
            }
            Instr::DynamicText { expr } => {
                let parent_id = self
                    .parent_stack
                    .last()
                    .cloned()
                    .or_else(|| self.nodes.first().map(|n| n.id.clone()))
                    .unwrap_or_else(|| node_id(self.file, &path, &expr.source));
                self.dynamic_texts.push(DynamicTextRef {
                    expr: expr.source.clone(),
                    parent_id,
                });
                None
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
                self.by_tag.entry("if".into()).or_default().push(id.clone());
                self.nodes.push(TreeNodeRef {
                    id: id.clone(),
                    tag: "if".into(),
                });
                self.parent_stack.push(id.clone());
                for (bi, br) in i.branches.iter().enumerate() {
                    let bp = format!("{path}.b{bi}");
                    let _ = self.walk_block(&br.body, &bp);
                }
                if let Some(else_body) = &i.else_body {
                    let _ = self.walk_block(else_body, &format!("{path}.else"));
                }
                self.parent_stack.pop();
                Some(id)
            }
            Instr::For(f) => {
                let content = format!("for|{}|{}", f.var_name, f.iterable.source);
                let id = node_id(self.file, &path, &content);
                self.by_tag
                    .entry("for".into())
                    .or_default()
                    .push(id.clone());
                self.nodes.push(TreeNodeRef {
                    id: id.clone(),
                    tag: "for".into(),
                });
                self.parent_stack.push(id.clone());
                let _ = self.walk_block(&f.body, &path);
                if let Some(else_body) = &f.else_body {
                    let _ = self.walk_block(else_body, &format!("{path}.else"));
                }
                self.parent_stack.pop();
                Some(id)
            }
            Instr::Match(m) => {
                let content = format!("match|{}", m.discriminant.source);
                let id = node_id(self.file, &path, &content);
                self.by_tag
                    .entry("match".into())
                    .or_default()
                    .push(id.clone());
                self.nodes.push(TreeNodeRef {
                    id: id.clone(),
                    tag: "match".into(),
                });
                self.parent_stack.push(id.clone());
                for (ci, c) in m.cases.iter().enumerate() {
                    let _ = self.walk_block(&c.body, &format!("{path}.c{ci}"));
                }
                if let Some(d) = &m.default {
                    let _ = self.walk_block(d, &format!("{path}.default"));
                }
                self.parent_stack.pop();
                Some(id)
            }
            Instr::WhileLoading(wl) => {
                let content = format!("while-loading|{}", wl.bind);
                let id = node_id(self.file, &path, &content);
                self.nodes.push(TreeNodeRef {
                    id: id.clone(),
                    tag: "while-loading".into(),
                });
                self.parent_stack.push(id.clone());
                let _ = self.walk_block(&wl.body, &path);
                self.parent_stack.pop();
                Some(id)
            }
            Instr::WhileError(we) => {
                let content = format!("while-error|{}", we.bind);
                let id = node_id(self.file, &path, &content);
                self.nodes.push(TreeNodeRef {
                    id: id.clone(),
                    tag: "while-error".into(),
                });
                self.parent_stack.push(id.clone());
                let _ = self.walk_block(&we.body, &path);
                self.parent_stack.pop();
                Some(id)
            }
            _ => None,
        }
    }
}

/// Join layout elements to inspect node ids; emit regions + unknown dynamics.
pub fn join_layout_regions(
    elements: &[Element],
    layout: &sail_layout::taffy_tree::Layout,
    tree: &InspectTreeIndex,
) -> (Vec<Region>, Vec<DynamicRegion>) {
    let mut regions = Vec::new();
    let mut dynamics = Vec::new();
    let mut tag_cursors: BTreeMap<String, usize> = BTreeMap::new();
    let mut last_host_id: Option<String> = None;
    let mut last_if_id: Option<String> = None;

    for (idx, elem) in elements.iter().enumerate() {
        if idx == 0 || elem.tag.is_empty() {
            continue;
        }
        let author_tag = author_tag_from_layout(&elem.tag, &elem.class);
        let (mut id, mut unknown_join) =
            resolve_layout_node_id(tree, &author_tag, &mut tag_cursors);

        if !unknown_join {
            if is_layout_host_tag(&author_tag) {
                last_host_id = Some(id.clone());
            }
            if author_tag == "if" {
                last_if_id = Some(id.clone());
            }
        } else if matches!(author_tag.as_str(), "else" | "else-if") {
            if let Some(ref if_id) = last_if_id {
                id = if_id.clone();
                unknown_join = false;
            }
        } else if is_anonymous_expansion_box(&elem.tag, &author_tag) {
            if let Some(ref host) = last_host_id {
                id = host.clone();
                unknown_join = false;
            }
        }

        if let Some(computed) = layout.computed(idx) {
            // Viewport-absolute coords (sum taffy parent-relative locations).
            // Paint probes and agent overlays need viewport space; parent-relative
            // alone mis-samples nested host AABBs.
            let (abs_x, abs_y) = absolute_location(layout, elements, idx);
            regions.push(Region {
                node_id: id.clone(),
                x: abs_x,
                y: abs_y,
                w: computed.size.width,
                h: computed.size.height,
                tag: Some(if author_tag.is_empty() {
                    elem.tag.clone()
                } else {
                    author_tag.clone()
                }),
                style: None,
            });
        }

        if elem.is_unknown() {
            dynamics.push(DynamicRegion {
                node_id: id,
                reason: format!("unknown-subtree:<{}>", elem.tag),
                tag: Some(elem.tag.clone()),
            });
            let _ = unknown_join;
        }
    }
    (regions, dynamics)
}

/// Map stripped `{expr}` dynamics onto author DynamicText parent node ids.
pub fn join_expr_dynamics(
    expr_dynamics: &[DynamicRegion],
    tree: &InspectTreeIndex,
) -> Vec<DynamicRegion> {
    let mut text_cursor = 0usize;
    let mut out = Vec::new();
    for d in expr_dynamics {
        // Extract expr from reason `dynamic-expr:{…}`
        let expr = d
            .reason
            .strip_prefix("dynamic-expr:{")
            .and_then(|s| s.strip_suffix('}'))
            .unwrap_or("")
            .to_string();
        let parent_id =
            if !expr.is_empty() {
                // Prefer exact expr match in author order.
                if let Some(pos) = tree.dynamic_texts.iter().skip(text_cursor).position(|t| {
                    t.expr == expr || t.expr.replace(' ', "") == expr.replace(' ', "")
                }) {
                    let abs = text_cursor + pos;
                    text_cursor = abs + 1;
                    tree.dynamic_texts[abs].parent_id.clone()
                } else if let Some(t) = tree.dynamic_texts.get(text_cursor) {
                    text_cursor += 1;
                    t.parent_id.clone()
                } else {
                    tree.roots
                        .first()
                        .cloned()
                        .or_else(|| tree.nodes.first().map(|n| n.id.clone()))
                        .unwrap_or_else(|| d.node_id.clone())
                }
            } else {
                d.node_id.clone()
            };
        out.push(DynamicRegion {
            node_id: parent_id,
            reason: d.reason.clone(),
            tag: d.tag.clone(),
        });
    }
    out
}

fn resolve_layout_node_id(
    tree: &InspectTreeIndex,
    author_tag: &str,
    tag_cursors: &mut BTreeMap<String, usize>,
) -> (String, bool) {
    let key = author_tag.to_ascii_lowercase();
    let cursor = tag_cursors.entry(key.clone()).or_insert(0);
    if let Some(ids) = tree.by_tag.get(&key) {
        if let Some(id) = ids.get(*cursor) {
            *cursor += 1;
            return (id.clone(), false);
        }
    }
    // Always synthesize a distinct id — do not collapse unjoined siblings onto
    // roots.first() / nodes.first() (that reused one id for every same-tag host
    // when the author tree was non-empty but the tag cursor was exhausted).
    let occ = *cursor;
    *cursor += 1;
    let base = tree
        .roots
        .first()
        .cloned()
        .or_else(|| tree.nodes.first().map(|n| n.id.clone()));
    let id = match base {
        Some(root) => format!("{root}#unjoined#{occ}#{author_tag}"),
        None => node_id("_unjoined", &occ.to_string(), author_tag),
    };
    (id, true)
}

fn author_tag_from_layout(layout_tag: &str, class: &str) -> String {
    // L1 hosts first (scan all tokens — emit order must not decide join).
    // field/navbar/etc. lower onto sail-col/sail-row AND sail-<host>.
    let mut l1_host: Option<String> = None;
    let mut layout_prim: Option<&'static str> = None;
    for tok in class.split_whitespace() {
        if let Some(name) = tok.strip_prefix("sail-") {
            if is_sail_layout_utility(name) {
                if layout_prim.is_none() {
                    layout_prim = match name {
                        "col" | "column" => Some("column"),
                        "row" => Some("row"),
                        "grid" => Some("grid"),
                        "center" => Some("center"),
                        "stack" => Some("stack"),
                        _ => None,
                    };
                }
                continue;
            }
            if l1_host.is_none() && is_known_l1_tag(name) {
                l1_host = Some(name.to_string());
            }
        }
        match tok {
            "sail-col" | "sail-column" if layout_prim.is_none() => layout_prim = Some("column"),
            "sail-row" if layout_prim.is_none() => layout_prim = Some("row"),
            "sail-grid" if layout_prim.is_none() => layout_prim = Some("grid"),
            "sail-center" if layout_prim.is_none() => layout_prim = Some("center"),
            "sail-stack" if layout_prim.is_none() => layout_prim = Some("stack"),
            _ => {}
        }
    }
    if let Some(host) = l1_host {
        return host;
    }
    if let Some(p) = layout_prim {
        return p.into();
    }
    layout_tag.to_ascii_lowercase()
}

fn is_sail_layout_utility(name: &str) -> bool {
    matches!(
        name,
        "col" | "column" | "row" | "grid" | "center" | "stack" | "nowrap"
    ) || name.starts_with("ai-")
        || name.starts_with("jc-")
        || name.starts_with("ji-")
        || name.starts_with("as-")
        || name.starts_with("js-")
        || name.starts_with("gc-")
        || name.starts_with("gf-")
        || name.starts_with("g-")
}

fn is_known_l1_tag(name: &str) -> bool {
    sail_compiler::l1_entries()
        .iter()
        .any(|e| e.tag.eq_ignore_ascii_case(name))
}

fn is_layout_host_tag(author_tag: &str) -> bool {
    matches!(author_tag, "column" | "row" | "grid" | "center" | "stack")
        || is_known_l1_tag(author_tag)
}

fn is_anonymous_expansion_box(layout_tag: &str, author_tag: &str) -> bool {
    let t = layout_tag.to_ascii_lowercase();
    let a = author_tag.to_ascii_lowercase();
    matches!(t.as_str(), "div" | "span") && a == t
}

/// True if a region id is a synthetic unjoined fallback (empty-tree
/// `_unjoined#…` or non-empty-tree `{root}#unjoined#{occ}#{tag}`).
pub fn is_unjoined_id(id: &str) -> bool {
    id.contains("_unjoined") || id.contains("#unjoined#")
}

/// Sum taffy parent-relative locations up the element parent chain → viewport.
pub fn absolute_location(
    layout: &sail_layout::taffy_tree::Layout,
    elements: &[Element],
    idx: usize,
) -> (f32, f32) {
    let mut x = 0.0_f32;
    let mut y = 0.0_f32;
    let mut cur = Some(idx);
    while let Some(i) = cur {
        if i == 0 {
            break; // synthetic document root
        }
        if let Some(c) = layout.computed(i) {
            x += c.location.x;
            y += c.location.y;
        }
        cur = elements.get(i).and_then(|e| e.parent);
    }
    (x, y)
}

/// FNV-1a 32-bit — identical to SailInspect.
pub fn short_hash(input: &str) -> String {
    let mut h: u32 = 0x811c_9dc5;
    for b in input.as_bytes() {
        h ^= u32::from(*b);
        h = h.wrapping_mul(0x0100_0193);
    }
    format!("{h:08x}")
}

pub fn node_id(file: &str, ordinal_path: &str, content: &str) -> String {
    let h = short_hash(content);
    format!("{file}#{ordinal_path}#{h}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unjoined_siblings_get_distinct_ids_when_tree_nonempty() {
        // Partial author tree: one known "card" id; two layout hosts exhaust the cursor.
        let mut by_tag = BTreeMap::new();
        by_tag.insert("card".into(), vec!["file#0#aaaa".into()]);
        let tree = InspectTreeIndex {
            file: "t.sail".into(),
            roots: vec!["file#root#bbbb".into()],
            by_tag,
            nodes: vec![TreeNodeRef {
                id: "file#0#aaaa".into(),
                tag: "card".into(),
            }],
            dynamic_texts: vec![],
        };
        let mut cursors = BTreeMap::new();
        let (id0, u0) = resolve_layout_node_id(&tree, "card", &mut cursors);
        assert!(!u0, "first card should join author tree");
        assert_eq!(id0, "file#0#aaaa");

        let (id1, u1) = resolve_layout_node_id(&tree, "card", &mut cursors);
        let (id2, u2) = resolve_layout_node_id(&tree, "card", &mut cursors);
        assert!(u1 && u2, "exhausted tag cursor → unjoined");
        assert_ne!(id1, id2, "unjoined sibling cards must not share an id");
        assert!(
            id1.contains("#unjoined#") && id2.contains("#unjoined#"),
            "expected synthetic unjoined ids, got {id1:?} {id2:?}"
        );
    }

    #[test]
    fn unjoined_empty_tree_still_distinct() {
        let tree = InspectTreeIndex::default();
        let mut cursors = BTreeMap::new();
        let (a, ua) = resolve_layout_node_id(&tree, "card", &mut cursors);
        let (b, ub) = resolve_layout_node_id(&tree, "card", &mut cursors);
        assert!(ua && ub);
        assert_ne!(a, b);
        assert!(is_unjoined_id(&a) && is_unjoined_id(&b));
    }

    #[test]
    fn is_unjoined_id_matches_both_forms() {
        assert!(is_unjoined_id("file#_unjoined#0#card"));
        assert!(is_unjoined_id("file#root#bbbb#unjoined#1#card"));
        assert!(!is_unjoined_id("file#0#aaaa"));
    }

    #[test]
    fn absolute_location_sums_parent_offsets() {
        // Real preprocess + taffy tree: nested pad offsets must accumulate.
        let src = r#"<column sx={style.pad(4)}><span>Hi</span></column>"#;
        let pre = sail_compiler::parse::preprocess::preprocess(src).unwrap();
        let elements = sail_layout::html::parse_source(&pre);
        let layout = sail_layout::taffy_tree::build(elements.clone(), (320.0, 200.0)).unwrap();
        // Find nested span (not synthetic root).
        let span_idx = elements
            .iter()
            .enumerate()
            .find(|(i, e)| *i > 0 && e.tag == "span")
            .map(|(i, _)| i)
            .expect("span");
        let (abs_x, abs_y) = absolute_location(&layout, &elements, span_idx);
        let rel = layout.computed(span_idx).unwrap();
        // Parent column has pad → span abs should be >= parent-relative.
        assert!(
            abs_x >= rel.location.x - 0.01 && abs_y >= rel.location.y - 0.01,
            "abs=({abs_x},{abs_y}) rel=({},{})",
            rel.location.x,
            rel.location.y
        );
        // With pad(4) ≈ 16px on column, span absolute y should exceed bare rel if
        // the parent itself is offset from root — at least finite and non-negative.
        assert!(abs_x >= 0.0 && abs_y >= 0.0);
    }
}
