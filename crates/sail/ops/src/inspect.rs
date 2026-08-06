//! `SailInspect` RPCs — Model, Tree, State, TokenUsage.
//!
//! Canonical per-file model: source-derived, span-keyed, versioned (T-I1/3).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::{Value, json};

use crate::ctx::Ctx;
use crate::error::OpError;
use crate::watch::relativize_watch_path;

pub const MODEL_VERSION: u32 = 1;

// ── public types (contracts re-export shapes via Json) ──────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct ModelSpan {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub col: usize,
    pub end_line: usize,
    pub end_col: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct TreeNode {
    pub id: String,
    pub file: String,
    pub span: ModelSpan,
    pub tag: String,
    pub layer: String,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub attrs: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub control_flow: Option<ControlFlowEdge>,
    pub children: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ControlFlowEdge {
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expr: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LayoutBox {
    pub id: String,
    pub file: String,
    pub span: ModelSpan,
    pub viewport: Viewport,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub collapsed: bool,
    pub overflow: bool,
    pub unknown_subtree: bool,
    pub tag: String,
    /// True when no unique tree node was available for this box (honest join).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub unknown_join: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Viewport {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct StateBinding {
    pub id: String,
    pub file: String,
    pub span: ModelSpan,
    pub name: String,
    pub kind: String,
    pub source: String,
    pub ssr_tier: String,
    pub readers: Vec<StateEdge>,
    pub writers: Vec<StateEdge>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StateEdge {
    pub id: String,
    pub file: String,
    pub span: ModelSpan,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expr: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelDiagnostic {
    pub id: String,
    pub file: String,
    pub span: ModelSpan,
    pub severity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule: Option<String>,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DataKey {
    pub id: String,
    pub file: String,
    pub span: ModelSpan,
    pub kind: String,
    pub key: String,
    pub ssr_fetchable: bool,
}

// ── path confinement ────────────────────────────────────────────────────────

fn resolve_sail_path(path: &str, ctx: &Ctx) -> Result<(PathBuf, String), OpError> {
    let raw = path.trim();
    if raw.is_empty() {
        return Err(OpError::InvalidArgument("path is required".into()));
    }
    if raw.contains("..") {
        return Err(OpError::PermissionDenied(
            "path must not contain `..`".into(),
        ));
    }
    if !raw.ends_with(".sail") {
        return Err(OpError::InvalidArgument("path must end with .sail".into()));
    }
    // Workspace-relative only — refuse absolute host paths (agents must pass rel ids).
    if Path::new(raw).is_absolute() {
        return Err(OpError::PermissionDenied(
            "path must be workspace-relative (absolute paths refused)".into(),
        ));
    }
    let under_ws = ctx.workspace_root.join(raw);
    let candidate = if under_ws.exists() {
        under_ws
    } else {
        ctx.project_root.join(raw)
    };
    // Pre-check: joined path must not escape workspace even before canonicalize.
    let canon_root = ctx
        .workspace_root
        .canonicalize()
        .map_err(|e| OpError::Internal(format!("workspace root: {e}")))?;
    if let Ok(partial) = candidate.canonicalize() {
        if !partial.starts_with(&canon_root) {
            return Err(OpError::PermissionDenied("path escapes workspace".into()));
        }
    } else {
        // Missing file: still require the parent chain sits under workspace.
        let parent = candidate.parent().unwrap_or(candidate.as_path());
        if let Ok(cp) = parent.canonicalize() {
            if !cp.starts_with(&canon_root) {
                return Err(OpError::PermissionDenied("path escapes workspace".into()));
            }
        }
        return Err(OpError::NotFound(format!("read {raw}: not found")));
    }
    let canon = candidate
        .canonicalize()
        .map_err(|e| OpError::NotFound(format!("read {raw}: {e}")))?;
    if !canon.starts_with(&canon_root) {
        return Err(OpError::PermissionDenied("path escapes workspace".into()));
    }
    let rel = relativize_watch_path(&ctx.workspace_root, &canon);
    Ok((canon, rel))
}

fn read_source(abs: &Path) -> Result<String, OpError> {
    std::fs::read_to_string(abs)
        .map_err(|e| OpError::NotFound(format!("read {}: {e}", abs.display())))
}

// ── span helpers ────────────────────────────────────────────────────────────

fn model_span(source: &str, start: usize, end: usize) -> ModelSpan {
    let start = start.min(source.len());
    let end = end.min(source.len()).max(start);
    let (line, col) = sail_compiler::diagnostics::line_col(source, start);
    let (end_line, end_col) = sail_compiler::diagnostics::line_col(source, end);
    ModelSpan {
        start,
        end,
        line,
        col,
        end_line,
        end_col,
    }
}

fn author_expr_span(source: &str, expr: &str, fallback: sail_compiler::Span) -> ModelSpan {
    author_expr_span_nth(source, expr, 0, fallback)
}

fn author_expr_span_nth(
    source: &str,
    expr: &str,
    n: usize,
    fallback: sail_compiler::Span,
) -> ModelSpan {
    if let Some((s, e)) = find_expr_in_author_nth(source, expr, n) {
        return model_span(source, s, e);
    }
    model_span(
        source,
        fallback.start.min(source.len()),
        fallback.end.min(source.len()),
    )
}

fn short_hash(input: &str) -> String {
    // FNV-1a 32-bit — deterministic, no external dep.
    let mut h: u32 = 0x811c_9dc5;
    for b in input.as_bytes() {
        h ^= u32::from(*b);
        h = h.wrapping_mul(0x0100_0193);
    }
    format!("{h:08x}")
}

fn node_id(file: &str, ordinal_path: &str, content: &str) -> String {
    let h = short_hash(content);
    format!("{file}#{ordinal_path}#{h}")
}

// ── layer classification ────────────────────────────────────────────────────

fn classify_layer(tag: &str) -> String {
    let lower = tag.to_ascii_lowercase();
    if matches!(
        lower.as_str(),
        "row"
            | "column"
            | "grid"
            | "center"
            | "stack"
            | "if"
            | "else"
            | "else-if"
            | "for"
            | "match"
            | "when"
            | "case"
            | "default"
    ) {
        return "l1-keyword".into();
    }
    // Stdlib L1 elements
    if sail_compiler::l1_entries()
        .iter()
        .any(|e| e.tag.eq_ignore_ascii_case(&lower))
    {
        return "l1-element".into();
    }
    if lower.contains('-') {
        // Pack components: known prefixes / paths from component packs.
        if lower.starts_with("pack-") || lower.starts_with("ship-") {
            return "pack".into();
        }
        if lower.starts_with("sail-") {
            return "l2-component".into();
        }
        return "userland".into();
    }
    // Native HTML tags without a stdlib L1 row
    "l1-element".into()
}

// ── tree walk ───────────────────────────────────────────────────────────────

struct TreeBuild<'a> {
    /// Author `.sail` source (spans remapped here).
    source: &'a str,
    file: &'a str,
    nodes: Vec<TreeNode>,
    /// span.start → id (for layout join / diagnostic join)
    by_start: BTreeMap<usize, String>,
    /// tag → ordered ids (document order)
    by_tag: BTreeMap<String, Vec<String>>,
    /// Per-tag occurrence counters into author source for span remap.
    tag_occ: BTreeMap<String, usize>,
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
                let span = self.author_span_for_tag(&el.tag);
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
                let children = self.walk_block(&el.children, &path);
                self.by_start.insert(span.start, id.clone());
                self.by_tag
                    .entry(el.tag.to_ascii_lowercase())
                    .or_default()
                    .push(id.clone());
                self.nodes.push(TreeNode {
                    id: id.clone(),
                    file: self.file.to_string(),
                    span,
                    tag: el.tag.clone(),
                    layer: classify_layer(&el.tag),
                    attrs,
                    control_flow: None,
                    children,
                });
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
                let mut kids = Vec::new();
                for (bi, br) in i.branches.iter().enumerate() {
                    let bp = format!("{path}.b{bi}");
                    kids.extend(self.walk_block(&br.body, &bp));
                }
                if let Some(else_body) = &i.else_body {
                    kids.extend(self.walk_block(else_body, &format!("{path}.else")));
                }
                let span = self.author_span_for_tag("if");
                self.by_start.insert(span.start, id.clone());
                self.by_tag.entry("if".into()).or_default().push(id.clone());
                self.nodes.push(TreeNode {
                    id: id.clone(),
                    file: self.file.to_string(),
                    span,
                    tag: "if".into(),
                    layer: "l1-keyword".into(),
                    attrs: BTreeMap::new(),
                    control_flow: Some(ControlFlowEdge {
                        kind: "if".into(),
                        expr: i.branches.first().map(|b| b.cond.source.clone()),
                    }),
                    children: kids,
                });
                Some(id)
            }
            Instr::For(f) => {
                let content = format!("for|{}|{}", f.var_name, f.iterable.source);
                let id = node_id(self.file, &path, &content);
                let mut kids = self.walk_block(&f.body, &path);
                if let Some(else_body) = &f.else_body {
                    kids.extend(self.walk_block(else_body, &format!("{path}.else")));
                }
                let span = self.author_span_for_tag("for");
                self.by_start.insert(span.start, id.clone());
                self.by_tag
                    .entry("for".into())
                    .or_default()
                    .push(id.clone());
                self.nodes.push(TreeNode {
                    id: id.clone(),
                    file: self.file.to_string(),
                    span,
                    tag: "for".into(),
                    layer: "l1-keyword".into(),
                    attrs: BTreeMap::new(),
                    control_flow: Some(ControlFlowEdge {
                        kind: "for".into(),
                        expr: Some(f.iterable.source.clone()),
                    }),
                    children: kids,
                });
                Some(id)
            }
            Instr::Match(m) => {
                let content = format!("match|{}", m.discriminant.source);
                let id = node_id(self.file, &path, &content);
                let mut kids = Vec::new();
                for (ci, c) in m.cases.iter().enumerate() {
                    kids.extend(self.walk_block(&c.body, &format!("{path}.c{ci}")));
                }
                if let Some(d) = &m.default {
                    kids.extend(self.walk_block(d, &format!("{path}.default")));
                }
                let span = self.author_span_for_tag("match");
                self.by_start.insert(span.start, id.clone());
                self.by_tag
                    .entry("match".into())
                    .or_default()
                    .push(id.clone());
                self.nodes.push(TreeNode {
                    id: id.clone(),
                    file: self.file.to_string(),
                    span,
                    tag: "match".into(),
                    layer: "l1-keyword".into(),
                    attrs: BTreeMap::new(),
                    control_flow: Some(ControlFlowEdge {
                        kind: "match".into(),
                        expr: Some(m.discriminant.source.clone()),
                    }),
                    children: kids,
                });
                Some(id)
            }
            Instr::When(w) => {
                let content = format!("when|{}", w.token);
                let id = node_id(self.file, &path, &content);
                let mut kids = self.walk_block(&w.body, &path);
                if let Some(e) = &w.else_body {
                    kids.extend(self.walk_block(e, &format!("{path}.else")));
                }
                let span = self.author_span_for_tag("when");
                self.by_start.insert(span.start, id.clone());
                self.by_tag
                    .entry("when".into())
                    .or_default()
                    .push(id.clone());
                self.nodes.push(TreeNode {
                    id: id.clone(),
                    file: self.file.to_string(),
                    span,
                    tag: "when".into(),
                    layer: "l1-keyword".into(),
                    attrs: BTreeMap::new(),
                    control_flow: Some(ControlFlowEdge {
                        kind: "when".into(),
                        expr: Some(w.token.clone()),
                    }),
                    children: kids,
                });
                Some(id)
            }
            Instr::StaticText { .. } | Instr::DynamicText { .. } | Instr::Comment { .. } => None,
        }
    }

    /// Next open-tag occurrence of `tag` in the author source (document order).
    fn author_span_for_tag(&mut self, tag: &str) -> ModelSpan {
        let key = tag.to_ascii_lowercase();
        let n = *self.tag_occ.get(&key).unwrap_or(&0);
        self.tag_occ.insert(key.clone(), n + 1);
        if let Some((s, e)) = find_nth_open_tag(self.source, &key, n) {
            model_span(self.source, s, e)
        } else {
            // Fallback: zero-width at 0 only when tag truly absent (should be rare).
            model_span(self.source, 0, 0)
        }
    }
}

/// Find the `n`th open tag `<tag` (word-boundary) in author source.
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

/// Locate the `n`th expression occurrence in author source (state edge spans).
fn find_expr_in_author_nth(source: &str, expr: &str, n: usize) -> Option<(usize, usize)> {
    let trimmed = expr.trim();
    if trimmed.is_empty() {
        return None;
    }
    let braced = format!("{{{trimmed}}}");
    let mut from = 0usize;
    let mut found = 0usize;
    while let Some(rel) = source[from..].find(&braced) {
        let s = from + rel;
        if found == n {
            return Some((s, s + braced.len()));
        }
        found += 1;
        from = s + braced.len();
    }
    from = 0;
    found = 0;
    while let Some(rel) = source[from..].find(trimmed) {
        let s = from + rel;
        if found == n {
            return Some((s, s + trimmed.len()));
        }
        found += 1;
        from = s + trimmed.len();
    }
    None
}

fn build_tree(source: &str, file: &str) -> Result<(Vec<TreeNode>, TreeBuildMeta), OpError> {
    // Author IR: no stdlib/layout expansion so tags and layers stay faithful.
    let program = sail_compiler::parse::parse_author(source)
        .map_err(|e| OpError::InvalidArgument(format!("parse: {e}")))?;
    let mut b = TreeBuild {
        source,
        file,
        nodes: Vec::new(),
        by_start: BTreeMap::new(),
        by_tag: BTreeMap::new(),
        tag_occ: BTreeMap::new(),
    };
    let roots = b.walk_block(&program.template, "");
    let meta = TreeBuildMeta {
        by_start: b.by_start,
        by_tag: b.by_tag,
        root_ids: roots,
        nodes: b.nodes.clone(),
    };
    Ok((b.nodes, meta))
}

struct TreeBuildMeta {
    by_start: BTreeMap<usize, String>,
    by_tag: BTreeMap<String, Vec<String>>,
    root_ids: Vec<String>,
    nodes: Vec<TreeNode>,
}

// ── layout join ─────────────────────────────────────────────────────────────

fn build_layout(
    source: &str,
    file: &str,
    meta: &TreeBuildMeta,
    viewport: Option<(f32, f32)>,
) -> Result<Vec<LayoutBox>, OpError> {
    let preprocessed = sail_compiler::parse::preprocess::preprocess(source)
        .map_err(|e| OpError::InvalidArgument(format!("preprocess: {e}")))?;
    let viewports = if let Some(vp) = viewport {
        vec![vp]
    } else {
        sail_compiler::check::layout::layout_viewports(&preprocessed)
    };

    let mut out = Vec::new();
    for vp in viewports {
        let scan = match sail_layout::check_preprocessed(&preprocessed, vp) {
            Ok(s) => s,
            Err(e) => {
                return Err(OpError::Internal(format!("layout: {e}")));
            }
        };
        // Overflow flags from layout diagnostics (element_index → box).
        let mut overflow_idxs: BTreeSet<usize> = BTreeSet::new();
        for d in &scan.diagnostics {
            if d.kind.contains("overflow") || d.kind.contains("collapsed-into-overflow") {
                overflow_idxs.insert(d.element_index);
            }
        }
        // Document-order tag cursors join layout boxes → tree nodes (class-aware).
        let mut tag_cursors: BTreeMap<String, usize> = BTreeMap::new();
        // Last successfully joined L1/keyword host — expansion chrome (card
        // header/footer divs, ticker track spans) attaches here instead of root.
        let mut last_host_id: Option<String> = None;
        let mut last_if_id: Option<String> = None;
        for b in &scan.boxes {
            let author_tag = author_tag_from_layout(&b.tag, &b.class);
            let (mut id, mut unknown_join) =
                resolve_layout_node_id(meta, &author_tag, &mut tag_cursors);

            if !unknown_join {
                if is_layout_host_tag(&author_tag) {
                    last_host_id = Some(id.clone());
                }
                if author_tag == "if" {
                    last_if_id = Some(id.clone());
                }
            } else {
                // Control-flow residual from preprocess (`else`) → parent if.
                if matches!(author_tag.as_str(), "else" | "else-if") {
                    if let Some(ref if_id) = last_if_id {
                        id = if_id.clone();
                        unknown_join = false;
                    }
                } else if is_anonymous_expansion_box(&b.tag, &author_tag) {
                    // Spliced L1 chrome (e.g. card header/footer) shares host id.
                    if let Some(ref host) = last_host_id {
                        id = host.clone();
                        unknown_join = false;
                    }
                }
            }

            let span = meta
                .nodes
                .iter()
                .find(|n| n.id == id)
                .map(|n| n.span.clone())
                .unwrap_or_else(|| model_span(source, 0, 0));
            let collapsed = b.width < 0.5 || b.height < 0.5;
            let overflow = overflow_idxs.contains(&b.index);
            out.push(LayoutBox {
                id,
                file: file.to_string(),
                span,
                viewport: Viewport {
                    width: vp.0,
                    height: vp.1,
                },
                x: b.x,
                y: b.y,
                w: b.width,
                h: b.height,
                collapsed,
                overflow,
                unknown_subtree: b.unknown_subtree,
                tag: b.tag.clone(),
                unknown_join,
            });
        }
    }
    Ok(out)
}

fn resolve_layout_node_id(
    meta: &TreeBuildMeta,
    author_tag: &str,
    tag_cursors: &mut BTreeMap<String, usize>,
) -> (String, bool) {
    let key = author_tag.to_ascii_lowercase();
    let cursor = tag_cursors.entry(key.clone()).or_insert(0);
    if let Some(ids) = meta.by_tag.get(&key) {
        if let Some(id) = ids.get(*cursor) {
            *cursor += 1;
            return (id.clone(), false);
        }
    }
    // Honest fallback: root with unknown_join — never invent "?" ids.
    let id = meta
        .root_ids
        .first()
        .cloned()
        .or_else(|| meta.nodes.first().map(|n| n.id.clone()))
        .unwrap_or_else(|| node_id("_unjoined", "0", author_tag));
    (id, true)
}

/// Recover author-level tag from layout engine tag + class list.
///
/// Preprocess lowers keywords to `div.sail-col` and L1 hosts to
/// `div.sail-card` / `span.sail-badge` / `div.sail-ticker` (etc.).
fn author_tag_from_layout(layout_tag: &str, class: &str) -> String {
    for tok in class.split_whitespace() {
        match tok {
            "sail-col" | "sail-column" => return "column".into(),
            "sail-row" => return "row".into(),
            "sail-grid" => return "grid".into(),
            "sail-center" => return "center".into(),
            "sail-stack" => return "stack".into(),
            _ => {}
        }
    }
    // L1 host class: sail-{tag} stamped/spliced by the compiler.
    for tok in class.split_whitespace() {
        if let Some(name) = tok.strip_prefix("sail-") {
            if is_sail_layout_utility(name) {
                continue;
            }
            if is_known_l1_tag(name) {
                return name.to_string();
            }
        }
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

/// Bare div/span with no recovered host tag — expansion chrome, not author host.
fn is_anonymous_expansion_box(layout_tag: &str, author_tag: &str) -> bool {
    let t = layout_tag.to_ascii_lowercase();
    let a = author_tag.to_ascii_lowercase();
    matches!(t.as_str(), "div" | "span") && a == t
}

// ── state ───────────────────────────────────────────────────────────────────

fn build_state(
    source: &str,
    file: &str,
    meta: &TreeBuildMeta,
) -> Result<Vec<StateBinding>, OpError> {
    let analysis = sail_compiler::analyze(source, Some(Path::new(file)), &[]);
    let program = sail_compiler::parse::parse_author(source)
        .map_err(|e| OpError::InvalidArgument(format!("parse: {e}")))?;

    // Collect template reads + handler writes.
    let mut readers: BTreeMap<String, Vec<StateEdge>> = BTreeMap::new();
    let mut writers: BTreeMap<String, Vec<StateEdge>> = BTreeMap::new();
    let mut expr_occ: BTreeMap<String, usize> = BTreeMap::new();
    collect_state_edges(
        source,
        file,
        &program.template,
        meta,
        &mut readers,
        &mut writers,
        &mut expr_occ,
        "",
        &mut 0,
    );

    let tier_map: BTreeMap<String, String> = analysis
        .ssr_tiers
        .into_iter()
        .map(|(n, t)| (n, t.as_str().to_string()))
        .collect();

    let mut out = Vec::new();
    for sym in analysis.symbols {
        let path = format!("state.{}", sym.name);
        let content = format!("{}:{}", sym.kind, sym.name);
        let id = node_id(file, &path, &content);
        // Controller decl line/col when available; else point at file start.
        let (start, end) = if sym.line > 0 {
            let s = offset_of(source, sym.line, sym.col);
            (s, s)
        } else {
            (0, 0)
        };
        let span = model_span(source, start, end);
        let ssr_tier = tier_map
            .get(&sym.name)
            .cloned()
            .unwrap_or_else(|| "I".into());
        out.push(StateBinding {
            id,
            file: file.to_string(),
            span,
            name: sym.name.clone(),
            kind: sym.kind,
            source: sym.source,
            ssr_tier,
            readers: readers.remove(&sym.name).unwrap_or_default(),
            writers: writers.remove(&sym.name).unwrap_or_default(),
        });
    }
    Ok(out)
}

fn collect_state_edges(
    source: &str,
    file: &str,
    instrs: &[sail_compiler::ir::Instr],
    meta: &TreeBuildMeta,
    readers: &mut BTreeMap<String, Vec<StateEdge>>,
    writers: &mut BTreeMap<String, Vec<StateEdge>>,
    expr_occ: &mut BTreeMap<String, usize>,
    path_prefix: &str,
    ordinal: &mut usize,
) {
    use sail_compiler::ir::{AttrInstr, Instr};
    for instr in instrs {
        let path = if path_prefix.is_empty() {
            format!("{ordinal}")
        } else {
            format!("{path_prefix}.{ordinal}")
        };
        *ordinal += 1;
        match instr {
            Instr::DynamicText { expr } => {
                let n = {
                    let c = expr_occ.entry(expr.source.clone()).or_insert(0);
                    let n = *c;
                    *c += 1;
                    n
                };
                for name in sail_compiler::parse::expr::idents::collect_idents(
                    &expr.source,
                    expr.allow_arrow,
                ) {
                    let span = author_expr_span_nth(source, &expr.source, n, expr.span);
                    let edge = StateEdge {
                        id: meta
                            .by_start
                            .range(..=span.start)
                            .next_back()
                            .map(|(_, id)| id.clone())
                            .unwrap_or_else(|| node_id(file, &path, "text")),
                        file: file.to_string(),
                        span,
                        kind: "template-read".into(),
                        expr: Some(expr.source.clone()),
                    };
                    readers.entry(name).or_default().push(edge);
                }
            }
            Instr::Element(el) => {
                for a in &el.attrs {
                    match a {
                        AttrInstr::Expr { expr, .. }
                        | AttrInstr::BindValue { expr, .. }
                        | AttrInstr::BindChecked { expr, .. } => {
                            for name in sail_compiler::parse::expr::idents::collect_idents(
                                &expr.source,
                                expr.allow_arrow,
                            ) {
                                let span = author_expr_span(source, &expr.source, expr.span);
                                readers.entry(name).or_default().push(StateEdge {
                                    id: meta
                                        .by_start
                                        .range(..=span.start)
                                        .next_back()
                                        .map(|(_, id)| id.clone())
                                        .unwrap_or_else(|| node_id(file, &path, &el.tag)),
                                    file: file.to_string(),
                                    span,
                                    kind: "template-read".into(),
                                    expr: Some(expr.source.clone()),
                                });
                            }
                        }
                        AttrInstr::On {
                            event,
                            handler,
                            span,
                            ..
                        } => {
                            for name in sail_compiler::parse::expr::idents::collect_idents(
                                &handler.source,
                                true,
                            ) {
                                let sp = author_expr_span(source, &handler.source, *span);
                                writers.entry(name).or_default().push(StateEdge {
                                    id: meta
                                        .by_start
                                        .range(..=sp.start)
                                        .next_back()
                                        .map(|(_, id)| id.clone())
                                        .unwrap_or_else(|| node_id(file, &path, &el.tag)),
                                    file: file.to_string(),
                                    span: sp,
                                    kind: format!("on:{event}"),
                                    expr: Some(handler.source.clone()),
                                });
                            }
                        }
                        _ => {}
                    }
                }
                let mut child_ord = 0usize;
                collect_state_edges(
                    source,
                    file,
                    &el.children,
                    meta,
                    readers,
                    writers,
                    expr_occ,
                    &path,
                    &mut child_ord,
                );
            }
            Instr::If(i) => {
                for br in &i.branches {
                    for name in sail_compiler::parse::expr::idents::collect_idents(
                        &br.cond.source,
                        br.cond.allow_arrow,
                    ) {
                        let span = author_expr_span(source, &br.cond.source, br.cond.span);
                        readers.entry(name).or_default().push(StateEdge {
                            id: meta
                                .by_start
                                .range(..=span.start)
                                .next_back()
                                .map(|(_, id)| id.clone())
                                .unwrap_or_else(|| node_id(file, &path, "if")),
                            file: file.to_string(),
                            span,
                            kind: "template-read".into(),
                            expr: Some(br.cond.source.clone()),
                        });
                    }
                    let mut o = 0;
                    collect_state_edges(
                        source, file, &br.body, meta, readers, writers, expr_occ, &path, &mut o,
                    );
                }
                if let Some(e) = &i.else_body {
                    let mut o = 0;
                    collect_state_edges(
                        source, file, e, meta, readers, writers, expr_occ, &path, &mut o,
                    );
                }
            }
            Instr::For(f) => {
                for name in sail_compiler::parse::expr::idents::collect_idents(
                    &f.iterable.source,
                    f.iterable.allow_arrow,
                ) {
                    let span = author_expr_span(source, &f.iterable.source, f.iterable.span);
                    readers.entry(name).or_default().push(StateEdge {
                        id: meta
                            .by_start
                            .range(..=span.start)
                            .next_back()
                            .map(|(_, id)| id.clone())
                            .unwrap_or_else(|| node_id(file, &path, "for")),
                        file: file.to_string(),
                        span,
                        kind: "template-read".into(),
                        expr: Some(f.iterable.source.clone()),
                    });
                }
                let mut o = 0;
                collect_state_edges(
                    source, file, &f.body, meta, readers, writers, expr_occ, &path, &mut o,
                );
            }
            Instr::Match(m) => {
                for name in sail_compiler::parse::expr::idents::collect_idents(
                    &m.discriminant.source,
                    m.discriminant.allow_arrow,
                ) {
                    let span =
                        author_expr_span(source, &m.discriminant.source, m.discriminant.span);
                    readers.entry(name).or_default().push(StateEdge {
                        id: meta
                            .by_start
                            .range(..=span.start)
                            .next_back()
                            .map(|(_, id)| id.clone())
                            .unwrap_or_else(|| node_id(file, &path, "match")),
                        file: file.to_string(),
                        span,
                        kind: "template-read".into(),
                        expr: Some(m.discriminant.source.clone()),
                    });
                }
                for c in &m.cases {
                    let mut o = 0;
                    collect_state_edges(
                        source, file, &c.body, meta, readers, writers, expr_occ, &path, &mut o,
                    );
                }
                if let Some(d) = &m.default {
                    let mut o = 0;
                    collect_state_edges(
                        source, file, d, meta, readers, writers, expr_occ, &path, &mut o,
                    );
                }
            }
            _ => {}
        }
    }
}

fn offset_of(source: &str, target_line: usize, target_col: usize) -> usize {
    let bytes = source.as_bytes();
    let (mut line, mut col, mut i) = (1usize, 1usize, 0usize);
    while i < bytes.len() {
        if line == target_line && col == target_col {
            return i;
        }
        if bytes[i] == b'\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
        i += 1;
    }
    bytes.len()
}

// ── diagnostics ─────────────────────────────────────────────────────────────

fn build_diagnostics(source: &str, file: &str, meta: &TreeBuildMeta) -> Vec<ModelDiagnostic> {
    let analysis = sail_compiler::analyze(source, Some(Path::new(file)), &[]);
    let mut out = Vec::new();
    for (i, d) in analysis.diagnostics.into_iter().enumerate() {
        let start = offset_of(source, d.line, d.col);
        let end = offset_of(source, d.end_line, d.end_col);
        let span = model_span(source, start, end);
        let node_id = meta
            .by_start
            .range(..=start)
            .next_back()
            .map(|(_, id)| id.clone());
        let content = format!(
            "{}|{}|{}",
            d.rule.as_deref().unwrap_or(""),
            d.line,
            d.message
        );
        out.push(ModelDiagnostic {
            id: node_id_diag(file, i, &content),
            file: file.to_string(),
            span,
            severity: match d.severity {
                sail_compiler::DiagSeverity::Error => "error".into(),
                sail_compiler::DiagSeverity::Warning => "warning".into(),
                sail_compiler::DiagSeverity::Fix => "fix".into(),
            },
            rule: d.rule,
            message: d.message,
            node_id,
        });
    }
    out
}

fn node_id_diag(file: &str, i: usize, content: &str) -> String {
    node_id(file, &format!("diag.{i}"), content)
}

// ── data keys ───────────────────────────────────────────────────────────────

fn build_data(source: &str, file: &str) -> Vec<DataKey> {
    let analysis = sail_compiler::analyze(source, Some(Path::new(file)), &[]);
    analysis
        .query_keys
        .into_iter()
        .enumerate()
        .map(|(i, q)| {
            let start = offset_of(source, q.line, q.col);
            DataKey {
                id: node_id(file, &format!("data.{i}"), &q.key),
                file: file.to_string(),
                span: model_span(source, start, start),
                kind: q.kind,
                key: q.key,
                ssr_fetchable: q.ssr_fetchable,
            }
        })
        .collect()
}

// ── theme section (alias to Graph summary for the file model) ───────────────

fn build_theme_section(ctx: &Ctx) -> Value {
    // Model embeds a light graph summary from catalog default; full Graph is SailTheme/Graph.
    match crate::theme::graph_internal(None, ctx) {
        Ok(v) => v,
        Err(_) => json!({ "edges": [], "nodes": [], "tiers": [] }),
    }
}

// ── RPCs ────────────────────────────────────────────────────────────────────

pub fn model(args: &Value, ctx: &Ctx) -> Result<Value, OpError> {
    let path = args
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| OpError::InvalidArgument("Model requires `path`".into()))?;
    let viewport = parse_viewport(args);
    let (abs, rel) = resolve_sail_path(path, ctx)?;
    let source = read_source(&abs)?;
    let (nodes, meta) = build_tree(&source, &rel)?;
    let layout = build_layout(&source, &rel, &meta, viewport)?;
    let state = build_state(&source, &rel, &meta)?;
    let diagnostics = build_diagnostics(&source, &rel, &meta);
    let data = build_data(&source, &rel);
    let theme = build_theme_section(ctx);

    Ok(json!({
        "modelVersion": MODEL_VERSION,
        "file": rel,
        // Author source so Studio Source view can open without a separate read RPC.
        "source": source,
        "tree": {
            "roots": meta.root_ids,
            "nodes": nodes,
        },
        "layout": { "boxes": layout },
        "state": { "bindings": state },
        "diagnostics": diagnostics,
        "data": { "keys": data },
        "theme": theme,
    }))
}

pub fn tree(args: &Value, ctx: &Ctx) -> Result<Value, OpError> {
    let path = args
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| OpError::InvalidArgument("Tree requires `path`".into()))?;
    let (abs, rel) = resolve_sail_path(path, ctx)?;
    let source = read_source(&abs)?;
    let (nodes, meta) = build_tree(&source, &rel)?;
    Ok(json!({
        "modelVersion": MODEL_VERSION,
        "file": rel,
        "roots": meta.root_ids,
        "nodes": nodes,
    }))
}

pub fn state(args: &Value, ctx: &Ctx) -> Result<Value, OpError> {
    let path = args
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| OpError::InvalidArgument("State requires `path`".into()))?;
    let (abs, rel) = resolve_sail_path(path, ctx)?;
    let source = read_source(&abs)?;
    let (_nodes, meta) = build_tree(&source, &rel)?;
    let bindings = build_state(&source, &rel, &meta)?;
    Ok(json!({
        "modelVersion": MODEL_VERSION,
        "file": rel,
        "bindings": bindings,
    }))
}

fn parse_viewport(args: &Value) -> Option<(f32, f32)> {
    let vp = args.get("viewport")?;
    let w = vp.get("width").and_then(|v| v.as_f64())? as f32;
    let h = vp.get("height").and_then(|v| v.as_f64())? as f32;
    Some((w, h))
}

// ── TokenUsage ──────────────────────────────────────────────────────────────

/// Static cssName / path fragment → component ids (blast radius when CEM unavailable).
static STATIC_COMPONENT_USAGE: &[(&str, &[&str])] = &[
    ("primary", &["button", "badge", "chip", "tab"]),
    ("primary-foreground", &["button", "badge"]),
    ("accent", &["button", "badge", "chip"]),
    ("accent-foreground", &["button", "badge"]),
    ("destructive", &["button", "badge", "alert"]),
    ("muted", &["badge", "chip", "empty-state", "card"]),
    ("muted-foreground", &["badge", "field", "label", "card"]),
    ("border", &["card", "input", "separator", "badge"]),
    ("border-subtle", &["card", "separator"]),
    ("background", &["page", "card"]),
    ("foreground", &["button", "label", "card"]),
    ("surface-1", &["card", "alert", "popover"]),
    ("color.primary.default", &["button", "badge"]),
    ("color.muted.default", &["badge", "chip"]),
    ("font-body", &["prose", "label"]),
    ("font-body-app", &["prose"]),
    (
        "radius",
        &["button", "card", "input", "badge", "chip", "alert"],
    ),
    ("radius-sm", &["button", "badge", "chip", "input"]),
    ("radius-md", &["button", "card", "input", "badge"]),
    ("radius-lg", &["card", "popover", "dialog", "alert"]),
    ("radius-xl", &["card", "popover", "dialog"]),
    ("radius-full", &["button", "badge", "avatar", "chip"]),
];

pub fn token_usage(args: &Value, ctx: &Ctx) -> Result<Value, OpError> {
    let token = args
        .get("token")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| OpError::InvalidArgument("TokenUsage requires non-empty `token`".into()))?;

    let mut components: BTreeSet<String> = BTreeSet::new();
    for (k, comps) in STATIC_COMPONENT_USAGE {
        if *k == token || token.ends_with(&format!(".{k}")) || token.ends_with(k) {
            for c in *comps {
                components.insert((*c).to_string());
            }
        }
        let tail = token.rsplit('.').next().unwrap_or(token);
        if tail == *k {
            for c in *comps {
                components.insert((*c).to_string());
            }
        }
    }

    // Sites: apps + packages (skip node_modules/target); paths always workspace-relative.
    let mut sites = Vec::new();
    scan_token_sites(&ctx.workspace_root, token, &mut sites)?;

    Ok(json!({
        "token": token,
        "components": components.len(),
        "componentIds": components.into_iter().collect::<Vec<_>>(),
        "sites": sites,
    }))
}

/// Per-root site budget so packages/ is not starved by apps/ (TOOL-2 re-review).
const TOKEN_SITES_PER_ROOT: usize = 100;
const TOKEN_SITES_TOTAL: usize = 200;

fn scan_token_sites(workspace: &Path, token: &str, sites: &mut Vec<Value>) -> Result<(), OpError> {
    let needles = [
        token.to_string(),
        format!("{{{token}}}"),
        format!("--{token}"),
        token.replace('.', "-"),
    ];
    // Two-phase: collect up to PER_ROOT from each root, then merge (apps then
    // packages) up to TOTAL — guarantees pack hits when they exist.
    let mut per_root: Vec<Vec<Value>> = Vec::new();
    for root_name in ["apps", "packages"] {
        let root = workspace.join(root_name);
        let mut bucket: Vec<Value> = Vec::new();
        if root.is_dir() {
            walk_sail_files(workspace, &root, &mut |abs, rel| {
                if bucket.len() >= TOKEN_SITES_PER_ROOT {
                    return;
                }
                let Ok(src) = std::fs::read_to_string(abs) else {
                    return;
                };
                for needle in &needles {
                    if needle.is_empty() {
                        continue;
                    }
                    let mut from = 0usize;
                    while let Some(rel_off) = src[from..].find(needle.as_str()) {
                        let start = from + rel_off;
                        let end = start + needle.len();
                        let span = model_span(&src, start, end);
                        bucket.push(json!({
                            "file": rel,
                            "span": span,
                        }));
                        from = end;
                        if bucket.len() >= TOKEN_SITES_PER_ROOT {
                            return;
                        }
                    }
                }
            });
        }
        per_root.push(bucket);
    }
    for bucket in per_root {
        for site in bucket {
            if sites.len() >= TOKEN_SITES_TOTAL {
                return Ok(());
            }
            sites.push(site);
        }
    }
    Ok(())
}

fn walk_sail_files(workspace: &Path, dir: &Path, f: &mut dyn FnMut(&Path, &str)) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for ent in rd.flatten() {
        let p = ent.path();
        if p.is_dir() {
            let name = ent.file_name().to_string_lossy().into_owned();
            if name == "node_modules" || name == "target" || name.starts_with('.') {
                continue;
            }
            walk_sail_files(workspace, &p, f);
        } else if p.extension().and_then(|e| e.to_str()) == Some("sail") {
            let rel = relativize_watch_path(workspace, &p);
            // Never emit absolute host paths.
            if rel.starts_with('/') || (rel.len() > 1 && rel.as_bytes()[1] == b':') {
                continue;
            }
            f(&p, &rel);
        }
    }
}

// ── tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn workspace_ctx() -> Ctx {
        let cwd = std::env::current_dir().unwrap();
        let mut cur = cwd.as_path();
        let mut root = cwd.clone();
        for _ in 0..8 {
            if cur.join("Cargo.toml").is_file() && cur.join("apps").is_dir() {
                root = cur.to_path_buf();
                break;
            }
            if let Some(p) = cur.parent() {
                cur = p;
            } else {
                break;
            }
        }
        Ctx::new(&root, root.join("apps/sail-site"), &root)
    }

    #[test]
    fn model_page_has_sections_and_node_ids() {
        let ctx = workspace_ctx();
        let out = model(&json!({"path": "apps/sail-site/routes/page.sail"}), &ctx).expect("model");
        assert_eq!(out["modelVersion"], MODEL_VERSION);
        assert!(!out["tree"]["nodes"].as_array().unwrap().is_empty());
        assert!(out["layout"]["boxes"].as_array().is_some());
        assert!(out["state"]["bindings"].as_array().is_some());
        assert!(out["diagnostics"].as_array().is_some());
        assert!(out["data"]["keys"].as_array().is_some());
        assert!(out.get("theme").is_some());
        let mut layers = BTreeSet::new();
        let src =
            std::fs::read_to_string(ctx.workspace_root.join("apps/sail-site/routes/page.sail"))
                .unwrap();
        let mut open_ok = 0usize;
        for n in out["tree"]["nodes"].as_array().unwrap() {
            assert!(n.get("id").and_then(|v| v.as_str()).is_some());
            assert!(n.get("span").is_some());
            assert!(n.get("file").and_then(|v| v.as_str()).is_some());
            let span = &n["span"];
            let start = span["start"].as_u64().unwrap() as usize;
            let end = span["end"].as_u64().unwrap() as usize;
            assert!(
                end > start,
                "zero-width span for tag={:?} span={span}",
                n["tag"]
            );
            assert!(end <= src.len(), "span past EOF");
            let tag = n["tag"].as_str().unwrap_or("");
            if src[start..end]
                .to_ascii_lowercase()
                .starts_with(&format!("<{tag}"))
            {
                open_ok += 1;
            }
            if let Some(l) = n["layer"].as_str() {
                layers.insert(l.to_string());
            }
        }
        let total = out["tree"]["nodes"].as_array().unwrap().len();
        assert!(
            open_ok * 100 / total >= 95,
            "author open-tag span quality {open_ok}/{total}"
        );
        // Author tree retains layout keywords + L1 elements (ticker, a, …).
        assert!(
            layers.contains("l1-keyword"),
            "expected l1-keyword layers, got {layers:?}"
        );
        assert!(
            layers.contains("l1-element"),
            "expected l1-element layers, got {layers:?}"
        );
    }

    #[test]
    fn model_fixture_snapshot_stable_ids_and_spans() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let sail_path = root.join("apps/demo/routes/card.sail");
        std::fs::create_dir_all(sail_path.parent().unwrap()).unwrap();
        let src = r#"
<controller>
import type { ControllerScope } from '@ship/sail-runtime'
export default controller((_ctx: ControllerScope<any>) => {
  const pure = signal(0)
  const ticks = signal(0)
  effect(() => {
    const t = setInterval(() => ticks.set((n) => n + 1), 1000)
    return () => clearInterval(t)
  })
  return { pure, ticks }
})
</controller>
<column gap="4">
  <ticker>hello</ticker>
  <my-widget primary>x</my-widget>
  <span>{ticks()}</span>
  <span>{pure()}</span>
</column>
"#;
        std::fs::write(&sail_path, src).unwrap();
        let ctx = Ctx::new(root, root.join("apps/demo"), root);
        let out = model(&json!({"path": "apps/demo/routes/card.sail"}), &ctx).expect("model");
        // Snapshot-ish: stable tree shape
        let nodes = out["tree"]["nodes"].as_array().unwrap();
        let tags: Vec<&str> = nodes.iter().filter_map(|n| n["tag"].as_str()).collect();
        assert!(tags.contains(&"column"), "{tags:?}");
        assert!(tags.contains(&"ticker"), "{tags:?}");
        assert!(tags.contains(&"my-widget"), "{tags:?}");
        let widget = nodes
            .iter()
            .find(|n| n["tag"] == "my-widget")
            .expect("widget");
        assert_eq!(widget["layer"], "userland");
        let ticker = nodes.iter().find(|n| n["tag"] == "ticker").expect("ticker");
        assert_eq!(ticker["layer"], "l1-element");
        // Spans cover real author tags
        let wspan = &widget["span"];
        let ws = wspan["start"].as_u64().unwrap() as usize;
        let we = wspan["end"].as_u64().unwrap() as usize;
        assert!(we > ws);
        assert!(src[ws..we].to_ascii_lowercase().contains("my-widget"));
        // Golden id stability across runs
        let again = model(&json!({"path": "apps/demo/routes/card.sail"}), &ctx).unwrap();
        assert_eq!(out["tree"]["nodes"], again["tree"]["nodes"]);
        // State tiers: pure S, ticks C; readers point at template sites
        let bindings = out["state"]["bindings"].as_array().unwrap();
        let pure = bindings.iter().find(|b| b["name"] == "pure").unwrap();
        let ticks = bindings.iter().find(|b| b["name"] == "ticks").unwrap();
        assert_eq!(pure["ssr_tier"], "S", "{pure}");
        assert_eq!(ticks["ssr_tier"], "C", "{ticks}");
        let readers = ticks["readers"].as_array().unwrap();
        assert!(!readers.is_empty());
        let r0 = &readers[0]["span"];
        let rs = r0["start"].as_u64().unwrap() as usize;
        let re = r0["end"].as_u64().unwrap() as usize;
        assert!(
            src[rs..re].contains("ticks"),
            "reader span {:?}",
            &src[rs..re]
        );
    }

    #[test]
    fn layout_box_ids_resolve_to_tree_nodes() {
        let ctx = workspace_ctx();
        let out = model(
            &json!({"path": "apps/sail-site/routes/page.sail", "viewport": {"width": 1440, "height": 900}}),
            &ctx,
        )
        .expect("model");
        let tree_ids: BTreeSet<String> = out["tree"]["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|n| n["id"].as_str().map(|s| s.to_string()))
            .collect();
        let root = out["tree"]["roots"][0].as_str().unwrap_or("");
        let boxes = out["layout"]["boxes"].as_array().unwrap();
        let mut unknown = 0usize;
        let mut root_dump = 0usize;
        for b in boxes {
            let id = b["id"].as_str().expect("box id");
            assert!(tree_ids.contains(id), "layout box id {id} not in tree");
            assert!(!id.contains("#?"), "synthetic ? id forbidden: {id}");
            if b["unknown_join"].as_bool() == Some(true) {
                unknown += 1;
                if id == root {
                    root_dump += 1;
                }
            }
        }
        let n = boxes.len().max(1);
        assert!(
            unknown * 100 / n < 5,
            "unknown_join too high: {unknown}/{n} (root_dump={root_dump})"
        );
        assert_eq!(
            root_dump, 0,
            "unknown boxes must not glue to root (root_dump={root_dump})"
        );
    }

    #[test]
    fn state_lists_ticks_with_readers_and_ssr_tier() {
        let ctx = workspace_ctx();
        let out = state(&json!({"path": "apps/sail-site/routes/page.sail"}), &ctx).expect("state");
        let bindings = out["bindings"].as_array().unwrap();
        let ticks = bindings
            .iter()
            .find(|b| b["name"] == "ticks")
            .expect("ticks binding");
        assert_eq!(ticks["kind"], "signal");
        assert_eq!(ticks["ssr_tier"], "C", "ticks uses setInterval → C");
        let readers = ticks["readers"].as_array().unwrap();
        assert!(readers.len() >= 2, "two template readers expected: {ticks}");
        // Distinct author sites for ticks()
        let mut starts = BTreeSet::new();
        for r in readers {
            starts.insert(r["span"]["start"].as_u64().unwrap());
            let s = r["span"]["start"].as_u64().unwrap() as usize;
            let e = r["span"]["end"].as_u64().unwrap() as usize;
            assert!(e > s, "reader zero-width span");
        }
        assert!(starts.len() >= 2, "reader spans must differ: {starts:?}");
    }

    #[test]
    fn path_escape_refused() {
        let ctx = workspace_ctx();
        let err = model(&json!({"path": "../secret.sail"}), &ctx).unwrap_err();
        assert!(matches!(err, OpError::PermissionDenied(_)));
        let abs = model(&json!({"path": "/etc/passwd.sail"}), &ctx).unwrap_err();
        assert!(matches!(abs, OpError::PermissionDenied(_)));
        let bad = model(&json!({"path": "apps/nope.txt"}), &ctx).unwrap_err();
        assert!(matches!(bad, OpError::InvalidArgument(_)));
    }

    #[test]
    fn tree_deterministic_ids() {
        let ctx = workspace_ctx();
        let a = tree(&json!({"path": "apps/sail-site/routes/page.sail"}), &ctx).unwrap();
        let b = tree(&json!({"path": "apps/sail-site/routes/page.sail"}), &ctx).unwrap();
        assert_eq!(a["nodes"], b["nodes"]);
    }

    #[test]
    fn token_usage_wire_shape_and_relative_sites() {
        let ctx = workspace_ctx();
        let out = token_usage(&json!({"token": "primary"}), &ctx).unwrap();
        assert!(out["components"].as_u64().unwrap() >= 1);
        assert!(out.get("componentIds").and_then(|v| v.as_array()).is_some());
        // Deserialize into typed contract
        let typed: crate::contracts::InspectTokenUsageRet =
            serde_json::from_value(out.clone()).expect("typed TokenUsage");
        assert!(
            typed
                .component_ids
                .as_ref()
                .map(|c| !c.is_empty())
                .unwrap_or(false)
        );
        let mut has_apps = false;
        let mut has_packages = false;
        for site in out["sites"].as_array().unwrap() {
            let f = site["file"].as_str().unwrap();
            assert!(!f.starts_with('/'), "absolute path leak: {f}");
            assert!(f.ends_with(".sail"), "{f}");
            if f.starts_with("apps/") {
                has_apps = true;
            }
            if f.starts_with("packages/") {
                has_packages = true;
            }
        }
        assert!(has_apps, "expected apps/ sites for primary");
        assert!(
            has_packages,
            "expected packages/ sites for primary (fair per-root budget)"
        );
    }
}
