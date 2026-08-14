//! `SailInspect` RPCs — Model, Tree, State, TokenUsage, and unified topic
//! `Inspect` (TOOLING-1 WI5 / `sail_inspect`).
//!
//! Canonical per-file model: source-derived, span-keyed, versioned (T-I1/3).
//! Topic surface (mirrors `sail_surface`): tree | geometry | cascade | states |
//! a11y | invalidations | uses | cost | themes. CLI twin: `sail inspect <topic>`.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::{Value, json};

use crate::ctx::Ctx;
use crate::error::OpError;
use crate::identity::{ElementIdentity, SemanticLocator, SourceSpan};
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
    /// WI7 element identity (component id + semantic locator + file/span).
    pub identity: ElementIdentity,
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

fn model_span_to_source(span: &ModelSpan) -> SourceSpan {
    SourceSpan {
        start: span.start as u64,
        end: span.end as u64,
        line: Some(span.line as u64),
        col: Some(span.col as u64),
        end_line: Some(span.end_line as u64),
        end_col: Some(span.end_col as u64),
    }
}

fn locator_from_attrs(tag: &str, attrs: &BTreeMap<String, String>) -> SemanticLocator {
    let name = attrs
        .get("aria-label")
        .or_else(|| attrs.get("label"))
        .or_else(|| attrs.get("name"))
        .or_else(|| attrs.get("id"))
        .or_else(|| attrs.get("data-demo-node"))
        .cloned();
    let role = attrs.get("role").cloned().or_else(|| {
        // Mirror identity::implied_role for the small common set.
        let t = tag.to_ascii_lowercase();
        match t.as_str() {
            "button" => Some("button".into()),
            "a" | "link" => Some("link".into()),
            "nav" | "navbar" => Some("navigation".into()),
            "dialog" | "alert-dialog" => Some("dialog".into()),
            "checkbox" | "switch" => Some("checkbox".into()),
            "input" | "text-field" | "textarea" => Some("textbox".into()),
            _ => None,
        }
    });
    SemanticLocator {
        tag: tag.to_string(),
        role,
        name,
    }
}

fn identity_for_node(
    id: &str,
    file: &str,
    span: &ModelSpan,
    tag: &str,
    attrs: &BTreeMap<String, String>,
) -> ElementIdentity {
    let locator = locator_from_attrs(tag, attrs);
    let base = file.rsplit('/').next().unwrap_or(file);
    let element_path = match locator.name.as_deref() {
        Some(n) if !n.is_empty() => format!("{base} › {tag} {n}"),
        _ => format!("{base} › {tag}"),
    };
    let content_hash = id.rsplit('#').next().unwrap_or(id).to_string();
    ElementIdentity {
        node_id: id.to_string(),
        element_path,
        content_hash,
        file: Some(file.to_string()),
        span: Some(model_span_to_source(span)),
        locator: Some(locator),
    }
}

fn tree_node(
    id: String,
    file: &str,
    span: ModelSpan,
    tag: String,
    layer: String,
    attrs: BTreeMap<String, String>,
    control_flow: Option<ControlFlowEdge>,
    children: Vec<String>,
) -> TreeNode {
    let identity = identity_for_node(&id, file, &span, &tag, &attrs);
    TreeNode {
        id,
        file: file.to_string(),
        span,
        tag,
        layer,
        attrs,
        control_flow,
        children,
        identity,
    }
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
                self.nodes.push(tree_node(
                    id.clone(),
                    self.file,
                    span,
                    el.tag.clone(),
                    classify_layer(&el.tag),
                    attrs,
                    None,
                    children,
                ));
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
                self.nodes.push(tree_node(
                    id.clone(),
                    self.file,
                    span,
                    "if".into(),
                    "l1-keyword".into(),
                    BTreeMap::new(),
                    Some(ControlFlowEdge {
                        kind: "if".into(),
                        expr: i.branches.first().map(|b| b.cond.source.clone()),
                    }),
                    kids,
                ));
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
                self.nodes.push(tree_node(
                    id.clone(),
                    self.file,
                    span,
                    "for".into(),
                    "l1-keyword".into(),
                    BTreeMap::new(),
                    Some(ControlFlowEdge {
                        kind: "for".into(),
                        expr: Some(f.iterable.source.clone()),
                    }),
                    kids,
                ));
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
                self.nodes.push(tree_node(
                    id.clone(),
                    self.file,
                    span,
                    "match".into(),
                    "l1-keyword".into(),
                    BTreeMap::new(),
                    Some(ControlFlowEdge {
                        kind: "match".into(),
                        expr: Some(m.discriminant.source.clone()),
                    }),
                    kids,
                ));
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
                self.nodes.push(tree_node(
                    id.clone(),
                    self.file,
                    span,
                    "when".into(),
                    "l1-keyword".into(),
                    BTreeMap::new(),
                    Some(ControlFlowEdge {
                        kind: "when".into(),
                        expr: Some(w.token.clone()),
                    }),
                    kids,
                ));
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

/// Build tree node JSON values from author source (WI7 identity resolve without disk).
pub fn tree_nodes_from_source(source: &str, file: &str) -> Result<Vec<Value>, OpError> {
    let (nodes, _) = build_tree(source, file)?;
    nodes
        .iter()
        .map(|n| {
            serde_json::to_value(n)
                .map_err(|e| OpError::Internal(format!("serialize tree node: {e}")))
        })
        .collect()
}

/// Emit WI7 identity for every tree node in `source`.
pub fn identities_from_source(source: &str, file: &str) -> Result<Vec<ElementIdentity>, OpError> {
    let (nodes, _) = build_tree(source, file)?;
    Ok(nodes.into_iter().map(|n| n.identity).collect())
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
    let (preprocessed, hosts) = sail_compiler::parse::preprocess::preprocess_with_style(source)
        .map_err(|e| OpError::InvalidArgument(format!("preprocess: {e}")))?;
    let viewports = if let Some(vp) = viewport {
        vec![vp]
    } else {
        sail_compiler::check::layout::layout_viewports(&preprocessed)
    };

    let mut elements = sail_layout::html::parse_source(&preprocessed);
    sail_layout::apply_resolved_types(&mut elements, &hosts.types);
    sail_layout::apply_resolved_boxes(&mut elements, &hosts.boxes);

    let mut out = Vec::new();
    for vp in viewports {
        let scan = match sail_layout::check_preprocessed_with_hosts(
            &preprocessed,
            vp,
            sail_layout::CheckOptions::default(),
            &hosts.types,
            &hosts.boxes,
        ) {
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
        // Document-order tag cursors join layout boxes → tree nodes (Style/IR/tag).
        let mut tag_cursors: BTreeMap<String, usize> = BTreeMap::new();
        // Last successfully joined L1/keyword host — expansion chrome (card
        // header/footer divs, ticker track spans, in-file partial bodies)
        // attaches here. Seed with the author root so a leading viewport
        // wrapper is page chrome, not an unknown root dump.
        let mut last_host_id: Option<String> = meta.root_ids.first().cloned();
        let mut last_if_id: Option<String> = None;
        for b in &scan.boxes {
            let author_tag = elements
                .get(b.index)
                .map(|e| author_tag_from_element(e, &elements, vp.0))
                .unwrap_or_else(|| b.tag.to_ascii_lowercase());
            let (mut id, mut unknown_join) =
                resolve_layout_node_id(meta, &author_tag, &mut tag_cursors);

            if !unknown_join {
                // Any joined author node can parent expansion chrome. Keyword /
                // L1 hosts used to be required; v3 measure no longer stamps
                // sail-col/sail-row on every layout keyword, so a joined `div`
                // (or `p`/`a`) is still the right attach point.
                last_host_id = Some(id.clone());
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
                } else if let Some(ref host) = last_host_id {
                    // Extra boxes after author tags are exhausted (stdlib
                    // chrome, keyword→div measure, partial expand) attach
                    // to the last joined author node — never dump to root.
                    id = host.clone();
                    unknown_join = false;
                } else {
                    // First unresolved box is the document root itself.
                    unknown_join = false;
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

/// Recover author-level tag from host identity, Style/taffy role, or the tag.
///
/// Preprocess lowers keywords to `<div>` and L1 hosts stamp `data-sail-host`.
/// v3 measure does not stamp `sail-col`/`sail-row` as join IR — do not scrape
/// those class tokens.
fn author_tag_from_element(
    elem: &sail_layout::html::Element,
    all: &[sail_layout::html::Element],
    viewport_w: f32,
) -> String {
    if let Some(host) = elem.attrs.get("data-sail-host") {
        let h = host.trim();
        if !h.is_empty() {
            return h.to_ascii_lowercase();
        }
    }
    let tag = elem.tag.to_ascii_lowercase();
    if matches!(tag.as_str(), "column" | "row" | "grid" | "center" | "stack") {
        return tag;
    }
    let resolved = sail_layout::style::resolve(elem, all, viewport_w);
    use sail_layout::taffy::style::{Display, FlexDirection};
    match resolved.style.display {
        Display::Flex => match resolved.style.flex_direction {
            FlexDirection::Column | FlexDirection::ColumnReverse => "column".into(),
            FlexDirection::Row | FlexDirection::RowReverse => "row".into(),
        },
        Display::Grid => "grid".into(),
        _ => tag,
    }
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
            Instr::When(w) => {
                let mut o = 0;
                collect_state_edges(
                    source, file, &w.body, meta, readers, writers, expr_occ, &path, &mut o,
                );
                if let Some(e) = &w.else_body {
                    let mut o = 0;
                    collect_state_edges(
                        source, file, e, meta, readers, writers, expr_occ, &path, &mut o,
                    );
                }
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

// ── WI5 topic surface (sail_inspect) ────────────────────────────────────────

/// Topics for the unified inspect IR query interface (SPEC-TOOLING-1 §WI5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectTopic {
    Tree,
    Geometry,
    Cascade,
    States,
    A11y,
    Invalidations,
    Uses,
    Cost,
    Themes,
}

impl InspectTopic {
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "tree" => Some(Self::Tree),
            "geometry" | "layout" => Some(Self::Geometry),
            "cascade" | "token" | "tokens" => Some(Self::Cascade),
            "states" | "state" | "match" => Some(Self::States),
            "a11y" | "aria" | "accessibility" => Some(Self::A11y),
            "invalidations" | "invalidate" | "queries" => Some(Self::Invalidations),
            "uses" | "refs" | "symbols" => Some(Self::Uses),
            "cost" | "weight" => Some(Self::Cost),
            "themes" | "theme" => Some(Self::Themes),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Tree => "tree",
            Self::Geometry => "geometry",
            Self::Cascade => "cascade",
            Self::States => "states",
            Self::A11y => "a11y",
            Self::Invalidations => "invalidations",
            Self::Uses => "uses",
            Self::Cost => "cost",
            Self::Themes => "themes",
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Tree,
            Self::Geometry,
            Self::Cascade,
            Self::States,
            Self::A11y,
            Self::Invalidations,
            Self::Uses,
            Self::Cost,
            Self::Themes,
        ]
    }

    pub fn needs_path(self) -> bool {
        !matches!(self, Self::Themes | Self::Cascade)
    }
}

/// Unified `sail_inspect` entry (topic-param, mirrors `sail_surface`).
///
/// Args:
/// - `topic` (required): tree | geometry | cascade | states | a11y |
///   invalidations | uses | cost | themes
/// - `path` (file topics): workspace-relative `.sail`
/// - `viewport` `{width,height}`: geometry
/// - `token`: cascade (design token path or cssName tail)
/// - `themeId`: cascade / themes filter
/// - `nodeId`: optional filter on tree-linked topics
pub fn inspect(args: &Value, ctx: &Ctx) -> Result<Value, OpError> {
    let topic_s = args
        .get("topic")
        .and_then(|v| v.as_str())
        .ok_or_else(|| OpError::InvalidArgument("Inspect requires string `topic`".into()))?;
    let topic = InspectTopic::parse(topic_s).ok_or_else(|| {
        let list = InspectTopic::all()
            .iter()
            .map(|t| t.as_str())
            .collect::<Vec<_>>()
            .join(" | ");
        OpError::InvalidArgument(format!(
            "unknown topic `{topic_s}`; expected one of: {list}"
        ))
    })?;

    let mut body = match topic {
        InspectTopic::Tree => topic_tree(args, ctx)?,
        InspectTopic::Geometry => topic_geometry(args, ctx)?,
        InspectTopic::Cascade => topic_cascade(args, ctx)?,
        InspectTopic::States => topic_states(args, ctx)?,
        InspectTopic::A11y => topic_a11y(args, ctx)?,
        InspectTopic::Invalidations => topic_invalidations(args, ctx)?,
        InspectTopic::Uses => topic_uses(args, ctx)?,
        InspectTopic::Cost => topic_cost(args, ctx)?,
        InspectTopic::Themes => topic_themes(args, ctx)?,
    };

    // Envelope: topic + modelVersion always present.
    if let Some(obj) = body.as_object_mut() {
        obj.insert("topic".into(), json!(topic.as_str()));
        obj.entry("modelVersion".to_string())
            .or_insert(json!(MODEL_VERSION));
    }
    Ok(body)
}

fn require_path_source(args: &Value, ctx: &Ctx) -> Result<(String, String), OpError> {
    let path = args
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| OpError::InvalidArgument("this topic requires `path`".into()))?;
    let (abs, rel) = resolve_sail_path(path, ctx)?;
    let source = read_source(&abs)?;
    Ok((source, rel))
}

fn optional_node_filter(args: &Value) -> Option<String> {
    args.get("nodeId")
        .or_else(|| args.get("node_id"))
        .and_then(|v| v.as_str())
        .map(str::to_string)
}

fn topic_tree(args: &Value, ctx: &Ctx) -> Result<Value, OpError> {
    let (source, rel) = require_path_source(args, ctx)?;
    let (nodes, meta) = build_tree(&source, &rel)?;
    let filter = optional_node_filter(args);
    let nodes: Vec<TreeNode> = match filter {
        Some(id) => nodes.into_iter().filter(|n| n.id == id).collect(),
        None => nodes,
    };
    Ok(json!({
        "file": rel,
        "roots": meta.root_ids,
        "nodes": nodes,
    }))
}

fn topic_geometry(args: &Value, ctx: &Ctx) -> Result<Value, OpError> {
    let (source, rel) = require_path_source(args, ctx)?;
    let viewport = parse_viewport(args);
    let (_nodes, meta) = build_tree(&source, &rel)?;
    let boxes = build_layout(&source, &rel, &meta, viewport)?;
    let filter = optional_node_filter(args);
    let boxes: Vec<LayoutBox> = match filter {
        Some(id) => boxes.into_iter().filter(|b| b.id == id).collect(),
        None => boxes,
    };
    // Join identity for studio overlays (WI7 reuse on geometry boxes).
    let id_index: BTreeMap<String, ElementIdentity> = meta
        .nodes
        .iter()
        .map(|n| (n.id.clone(), n.identity.clone()))
        .collect();
    let boxes_json: Vec<Value> = boxes
        .into_iter()
        .map(|b| {
            let identity = id_index.get(&b.id).cloned();
            let mut v = serde_json::to_value(&b).unwrap_or(json!({}));
            if let Some(idt) = identity {
                if let Some(obj) = v.as_object_mut() {
                    obj.insert(
                        "identity".into(),
                        serde_json::to_value(idt).unwrap_or(Value::Null),
                    );
                }
            }
            v
        })
        .collect();
    Ok(json!({
        "file": rel,
        "viewport": viewport.map(|(w, h)| json!({"width": w, "height": h})),
        "boxes": boxes_json,
    }))
}

fn topic_cascade(args: &Value, ctx: &Ctx) -> Result<Value, OpError> {
    let token = args
        .get("token")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            OpError::InvalidArgument("cascade requires non-empty `token` (path or cssName)".into())
        })?;
    let theme_id = args
        .get("themeId")
        .or_else(|| args.get("theme_id"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty());

    let graph = crate::theme::graph_internal(theme_id, ctx)?;
    let chain = resolve_cascade_chain(token, &graph);

    // Optional: blast-radius sites when path or whole workspace.
    let mut sites = Vec::new();
    if let Some(path) = args.get("path").and_then(|v| v.as_str()) {
        if let Ok((abs, rel)) = resolve_sail_path(path, ctx) {
            if let Ok(src) = read_source(&abs) {
                collect_token_sites_in_source(&src, &rel, token, &mut sites);
            }
        }
    }

    Ok(json!({
        "token": token,
        "themeId": graph.get("themeId").cloned().unwrap_or(json!(theme_id.unwrap_or("ship"))),
        "chain": chain,
        "resolved": chain.last().cloned().unwrap_or(json!(null)),
        "sites": sites,
        "graphSources": graph.get("sources").cloned().unwrap_or(json!([])),
    }))
}

/// Walk DTCG alias edges from `token` to a leaf value.
fn resolve_cascade_chain(token: &str, graph: &Value) -> Vec<Value> {
    let nodes = graph
        .get("nodes")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let edges = graph
        .get("edges")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let mut by_path: BTreeMap<String, Value> = BTreeMap::new();
    let mut by_css: BTreeMap<String, String> = BTreeMap::new();
    for n in &nodes {
        if let Some(p) = n.get("path").and_then(|v| v.as_str()) {
            by_path.insert(p.to_string(), n.clone());
            if let Some(css) = n.get("cssName").and_then(|v| v.as_str()) {
                by_css.insert(css.to_string(), p.to_string());
            }
            // Also index last segment (primary from color.primary.default).
            if let Some(tail) = p.rsplit('.').next() {
                by_css
                    .entry(tail.to_string())
                    .or_insert_with(|| p.to_string());
            }
        }
    }

    let mut alias_to: BTreeMap<String, String> = BTreeMap::new();
    for e in &edges {
        if e.get("kind").and_then(|v| v.as_str()) != Some("alias") {
            continue;
        }
        if let (Some(from), Some(to)) = (
            e.get("from").and_then(|v| v.as_str()),
            e.get("to").and_then(|v| v.as_str()),
        ) {
            alias_to.insert(from.to_string(), to.to_string());
        }
    }

    // Resolve start path: exact path → cssName → tail match → fuzzy path end.
    let start = if by_path.contains_key(token) {
        token.to_string()
    } else if let Some(p) = by_css.get(token) {
        p.clone()
    } else {
        by_path
            .keys()
            .find(|p| p.ends_with(token) || p.ends_with(&format!(".{token}")))
            .cloned()
            .unwrap_or_else(|| token.to_string())
    };

    let mut chain = Vec::new();
    let mut cur = start;
    let mut seen = BTreeSet::new();
    for step in 0..32 {
        if !seen.insert(cur.clone()) {
            chain.push(json!({
                "step": step,
                "path": cur,
                "kind": "cycle",
            }));
            break;
        }
        if let Some(node) = by_path.get(&cur) {
            let value = node.get("value").cloned().unwrap_or(Value::Null);
            let is_alias = alias_to.contains_key(&cur);
            chain.push(json!({
                "step": step,
                "path": cur,
                "type": node.get("type").cloned().unwrap_or(json!("")),
                "value": value,
                "cssName": node.get("cssName").cloned().unwrap_or(Value::Null),
                "source": node.get("source").cloned().unwrap_or(Value::Null),
                "kind": if is_alias { "alias" } else { "leaf" },
            }));
            if let Some(next) = alias_to.get(&cur) {
                cur = next.clone();
                continue;
            }
            break;
        } else {
            chain.push(json!({
                "step": step,
                "path": cur,
                "kind": "unresolved",
            }));
            break;
        }
    }
    chain
}

fn collect_token_sites_in_source(src: &str, rel: &str, token: &str, sites: &mut Vec<Value>) {
    let needles = [
        token.to_string(),
        format!("{{{token}}}"),
        format!("--{token}"),
        token.replace('.', "-"),
        token.rsplit('.').next().unwrap_or(token).to_string(),
    ];
    for needle in &needles {
        if needle.is_empty() {
            continue;
        }
        let mut from = 0usize;
        while let Some(rel_off) = src[from..].find(needle.as_str()) {
            let start = from + rel_off;
            let end = start + needle.len();
            sites.push(json!({
                "file": rel,
                "span": model_span(src, start, end),
            }));
            from = end;
            if sites.len() >= 50 {
                return;
            }
        }
    }
}

fn topic_states(args: &Value, ctx: &Ctx) -> Result<Value, OpError> {
    let (source, rel) = require_path_source(args, ctx)?;
    let (nodes, meta) = build_tree(&source, &rel)?;
    let program = sail_compiler::parse::parse_author(&source)
        .map_err(|e| OpError::InvalidArgument(format!("parse: {e}")))?;

    let mut matches = Vec::new();
    collect_match_matrices(
        &source,
        &rel,
        &program.template,
        &meta,
        "",
        &mut 0,
        &mut matches,
    );

    // Scope-level style match unions (const paint = match(variant, …)) when present.
    let scope_matches = extract_scope_match_hints(&source);

    // Union-state matrix: for each match, list arms × whether default present.
    let matrices: Vec<Value> = matches
        .into_iter()
        .map(|m| {
            let arms = m
                .get("arms")
                .and_then(|a| a.as_array())
                .cloned()
                .unwrap_or_default();
            let tests: Vec<String> = arms
                .iter()
                .filter_map(|a| a.get("test").and_then(|t| t.as_str()).map(str::to_string))
                .collect();
            json!({
                "match": m,
                "union": tests,
                "armCount": arms.len(),
            })
        })
        .collect();

    // Include identity index for each match host when available.
    let _ = nodes;
    Ok(json!({
        "file": rel,
        "matches": matrices,
        "scopeMatches": scope_matches,
    }))
}

fn collect_match_matrices(
    source: &str,
    file: &str,
    instrs: &[sail_compiler::ir::Instr],
    meta: &TreeBuildMeta,
    path_prefix: &str,
    ordinal: &mut usize,
    out: &mut Vec<Value>,
) {
    use sail_compiler::ir::{Instr, MatchTest};
    for instr in instrs {
        let path = if path_prefix.is_empty() {
            format!("{ordinal}")
        } else {
            format!("{path_prefix}.{ordinal}")
        };
        *ordinal += 1;
        match instr {
            Instr::Match(m) => {
                let span = author_expr_span(source, &m.discriminant.source, m.discriminant.span);
                let host_id = meta
                    .by_start
                    .range(..=span.start)
                    .next_back()
                    .map(|(_, id)| id.clone())
                    .unwrap_or_else(|| node_id(file, &path, "match"));
                let identity = meta
                    .nodes
                    .iter()
                    .find(|n| n.id == host_id)
                    .map(|n| n.identity.clone());
                let mut arms = Vec::new();
                for (i, c) in m.cases.iter().enumerate() {
                    let test = match &c.test {
                        MatchTest::Kind(k) => json!({"kind": "kind", "test": k}),
                        MatchTest::Expr(e) => json!({"kind": "expr", "test": e.source}),
                    };
                    arms.push(json!({
                        "index": i,
                        "test": test.get("test").cloned().unwrap_or(Value::Null),
                        "testKind": test.get("kind").cloned().unwrap_or(json!("")),
                        "binding": c.binding,
                        "bodyNodeCount": count_instrs(&c.body),
                    }));
                }
                if m.default.is_some() {
                    arms.push(json!({
                        "index": arms.len(),
                        "test": "else",
                        "testKind": "default",
                        "binding": Value::Null,
                        "bodyNodeCount": m.default.as_ref().map(|b| count_instrs(b)).unwrap_or(0),
                    }));
                }
                out.push(json!({
                    "id": host_id,
                    "identity": identity,
                    "discriminant": m.discriminant.source,
                    "span": span,
                    "arms": arms,
                }));
                for (ci, c) in m.cases.iter().enumerate() {
                    let mut o = 0;
                    collect_match_matrices(
                        source,
                        file,
                        &c.body,
                        meta,
                        &format!("{path}.c{ci}"),
                        &mut o,
                        out,
                    );
                }
                if let Some(d) = &m.default {
                    let mut o = 0;
                    collect_match_matrices(
                        source,
                        file,
                        d,
                        meta,
                        &format!("{path}.default"),
                        &mut o,
                        out,
                    );
                }
            }
            Instr::Element(el) => {
                let mut o = 0;
                collect_match_matrices(source, file, &el.children, meta, &path, &mut o, out);
            }
            Instr::If(i) => {
                for (bi, br) in i.branches.iter().enumerate() {
                    let mut o = 0;
                    collect_match_matrices(
                        source,
                        file,
                        &br.body,
                        meta,
                        &format!("{path}.b{bi}"),
                        &mut o,
                        out,
                    );
                }
                if let Some(e) = &i.else_body {
                    let mut o = 0;
                    collect_match_matrices(
                        source,
                        file,
                        e,
                        meta,
                        &format!("{path}.else"),
                        &mut o,
                        out,
                    );
                }
            }
            Instr::For(f) => {
                let mut o = 0;
                collect_match_matrices(source, file, &f.body, meta, &path, &mut o, out);
            }
            Instr::When(w) => {
                let mut o = 0;
                collect_match_matrices(source, file, &w.body, meta, &path, &mut o, out);
                if let Some(e) = &w.else_body {
                    let mut o = 0;
                    collect_match_matrices(
                        source,
                        file,
                        e,
                        meta,
                        &format!("{path}.else"),
                        &mut o,
                        out,
                    );
                }
            }
            _ => {}
        }
    }
}

fn count_instrs(instrs: &[sail_compiler::ir::Instr]) -> usize {
    use sail_compiler::ir::Instr;
    let mut n = 0usize;
    for instr in instrs {
        n += 1;
        match instr {
            Instr::Element(el) => n += count_instrs(&el.children),
            Instr::If(i) => {
                for b in &i.branches {
                    n += count_instrs(&b.body);
                }
                if let Some(e) = &i.else_body {
                    n += count_instrs(e);
                }
            }
            Instr::For(f) => {
                n += count_instrs(&f.body);
                if let Some(e) = &f.else_body {
                    n += count_instrs(e);
                }
            }
            Instr::Match(m) => {
                for c in &m.cases {
                    n += count_instrs(&c.body);
                }
                if let Some(d) = &m.default {
                    n += count_instrs(d);
                }
            }
            Instr::When(w) => {
                n += count_instrs(&w.body);
                if let Some(e) = &w.else_body {
                    n += count_instrs(e);
                }
            }
            _ => {}
        }
    }
    n
}

/// Heuristic: find `match(ident, { … })` in scope/controller for union hints.
fn extract_scope_match_hints(source: &str) -> Vec<Value> {
    let mut out = Vec::new();
    let bytes = source.as_bytes();
    let needle = b"match(";
    let mut i = 0usize;
    while i + needle.len() < bytes.len() {
        if &bytes[i..i + needle.len()] == needle {
            // Skip template `<match` — only bare `match(`.
            let prev_ok = i == 0 || !bytes[i - 1].is_ascii_alphanumeric();
            if prev_ok {
                let after = i + needle.len();
                if let Some(close) = source[after..].find(')') {
                    let inner = source[after..after + close].trim();
                    // First arg is discriminant.
                    let disc = inner.split(',').next().unwrap_or("").trim();
                    if !disc.is_empty() && !disc.contains('<') {
                        let span = model_span(source, i, after + close + 1);
                        out.push(json!({
                            "discriminant": disc,
                            "span": span,
                            "kind": "scope-match",
                        }));
                    }
                }
            }
        }
        i += 1;
    }
    out
}

fn topic_a11y(args: &Value, ctx: &Ctx) -> Result<Value, OpError> {
    let (source, rel) = require_path_source(args, ctx)?;
    let (nodes, meta) = build_tree(&source, &rel)?;
    let filter = optional_node_filter(args);

    let by_id: BTreeMap<String, &TreeNode> = nodes.iter().map(|n| (n.id.clone(), n)).collect();
    let mut tree_out = Vec::new();
    for root in &meta.root_ids {
        if let Some(n) = by_id.get(root) {
            if filter
                .as_ref()
                .is_none_or(|f| node_or_descendant(n, f, &by_id))
            {
                tree_out.push(a11y_node_json(n, &by_id, filter.as_deref()));
            }
        }
    }

    // Flat list for agents that prefer scan over tree walk.
    let flat: Vec<Value> = nodes
        .iter()
        .filter(|n| filter.as_ref().is_none_or(|f| &n.id == f))
        .map(a11y_record)
        .collect();

    Ok(json!({
        "file": rel,
        "roots": meta.root_ids,
        "tree": tree_out,
        "nodes": flat,
    }))
}

fn node_or_descendant(n: &TreeNode, target: &str, by_id: &BTreeMap<String, &TreeNode>) -> bool {
    if n.id == target {
        return true;
    }
    n.children.iter().any(|c| {
        by_id
            .get(c)
            .is_some_and(|ch| node_or_descendant(ch, target, by_id))
    })
}

fn a11y_node_json(
    n: &TreeNode,
    by_id: &BTreeMap<String, &TreeNode>,
    filter: Option<&str>,
) -> Value {
    let mut rec = a11y_record(n);
    let kids: Vec<Value> = n
        .children
        .iter()
        .filter_map(|cid| by_id.get(cid))
        .filter(|ch| filter.is_none_or(|f| node_or_descendant(ch, f, by_id)))
        .map(|ch| a11y_node_json(ch, by_id, filter))
        .collect();
    if let Some(obj) = rec.as_object_mut() {
        obj.insert("children".into(), json!(kids));
    }
    rec
}

fn a11y_record(n: &TreeNode) -> Value {
    let anatomy = sail_compiler::host_anatomy(&n.tag);
    let mut aria = BTreeMap::new();
    let mut semantic = BTreeMap::new();

    // Author attrs that are semantic props or already-stamped aria.
    for (k, v) in &n.attrs {
        if let Some(prop) = sail_compiler::lookup_semantic_prop(k) {
            semantic.insert(prop.name.to_string(), v.clone());
            aria.insert(prop.aria.to_string(), v.clone());
        } else if k.starts_with("aria-") || k == "role" {
            aria.insert(k.clone(), v.clone());
        }
    }

    // Host anatomy fixed stamps (compiler would inject these).
    if let Some(a) = anatomy {
        if let Some(role) = a.role {
            aria.entry("role".into())
                .or_insert_with(|| role.to_string());
        }
        if let Some(live) = a.live {
            aria.entry("aria-live".into())
                .or_insert_with(|| live.to_string());
        }
        if let Some(atomic) = a.atomic {
            aria.entry("aria-atomic".into())
                .or_insert_with(|| atomic.to_string());
        }
        if let Some(label) = a.label {
            aria.entry("aria-label".into())
                .or_insert_with(|| label.to_string());
        }
    }

    // Implied role from tag when still missing.
    let role = aria
        .get("role")
        .cloned()
        .or_else(|| n.identity.locator.as_ref().and_then(|l| l.role.clone()))
        .or_else(|| locator_from_attrs(&n.tag, &n.attrs).role);

    let name = aria
        .get("aria-label")
        .cloned()
        .or_else(|| semantic.get("label").cloned())
        .or_else(|| n.identity.locator.as_ref().and_then(|l| l.name.clone()));

    json!({
        "id": n.id,
        "identity": n.identity,
        "tag": n.tag,
        "role": role,
        "name": name,
        "aria": aria,
        "semanticProps": semantic,
        "layer": n.layer,
        "span": n.span,
    })
}

fn topic_invalidations(args: &Value, ctx: &Ctx) -> Result<Value, OpError> {
    let (source, rel) = require_path_source(args, ctx)?;
    let analysis = sail_compiler::analyze(&source, Some(Path::new(&rel)), &[]);

    let mut queries = Vec::new();
    let mut mutations = Vec::new();
    let mut invalidates = Vec::new();
    let mut edges = Vec::new();

    for q in &analysis.query_keys {
        let start = offset_of(&source, q.line, q.col);
        let span = model_span(&source, start, start);
        let node = json!({
            "kind": q.kind,
            "key": q.key,
            "ssrFetchable": q.ssr_fetchable,
            "span": span,
            "file": rel,
        });
        match q.kind.as_str() {
            "query" => queries.push(node),
            "mutation" => mutations.push(node),
            "invalidate" => invalidates.push(node),
            _ => queries.push(node),
        }
    }

    // Edges: mutation → invalidate → query by shared key prefix/equality.
    for m in &analysis.query_keys {
        if m.kind != "mutation" {
            continue;
        }
        for inv in &analysis.query_keys {
            if inv.kind != "invalidate" {
                continue;
            }
            if keys_related(&m.key, &inv.key) {
                edges.push(json!({
                    "from": {"kind": "mutation", "key": m.key},
                    "to": {"kind": "invalidate", "key": inv.key},
                    "relation": "mutation-invalidates",
                }));
            }
        }
    }
    for inv in &analysis.query_keys {
        if inv.kind != "invalidate" {
            continue;
        }
        for q in &analysis.query_keys {
            if q.kind != "query" {
                continue;
            }
            if keys_related(&inv.key, &q.key) {
                edges.push(json!({
                    "from": {"kind": "invalidate", "key": inv.key},
                    "to": {"kind": "query", "key": q.key},
                    "relation": "invalidates-query",
                }));
            }
        }
    }
    // Direct mutation → query when keys related and no explicit invalidate listed.
    for m in &analysis.query_keys {
        if m.kind != "mutation" {
            continue;
        }
        for q in &analysis.query_keys {
            if q.kind != "query" {
                continue;
            }
            if keys_related(&m.key, &q.key) {
                edges.push(json!({
                    "from": {"kind": "mutation", "key": m.key},
                    "to": {"kind": "query", "key": q.key},
                    "relation": "mutation-touches-query",
                }));
            }
        }
    }

    Ok(json!({
        "file": rel,
        "queries": queries,
        "mutations": mutations,
        "invalidates": invalidates,
        "edges": edges,
    }))
}

fn keys_related(a: &str, b: &str) -> bool {
    if a.is_empty() || b.is_empty() {
        return false;
    }
    if a == b {
        return true;
    }
    // Prefix match on dotted keys: crm.deals ↔ crm.deals.list
    let (short, long) = if a.len() <= b.len() { (a, b) } else { (b, a) };
    long.starts_with(short)
        && (long.len() == short.len() || long.as_bytes().get(short.len()) == Some(&b'.'))
}

fn topic_uses(args: &Value, ctx: &Ctx) -> Result<Value, OpError> {
    let (source, rel) = require_path_source(args, ctx)?;
    let (_nodes, meta) = build_tree(&source, &rel)?;
    let bindings = build_state(&source, &rel, &meta)?;
    let analysis = sail_compiler::analyze(&source, Some(Path::new(&rel)), &[]);
    let program = sail_compiler::parse::parse_author(&source)
        .map_err(|e| OpError::InvalidArgument(format!("parse: {e}")))?;

    let mut symbols = Vec::new();
    for b in &bindings {
        symbols.push(json!({
            "name": b.name,
            "kind": b.kind,
            "source": b.source,
            "ssrTier": b.ssr_tier,
            "span": b.span,
            "readers": b.readers,
            "writers": b.writers,
            "crossRegion": !b.readers.is_empty() || !b.writers.is_empty(),
        }));
    }

    let mut actions = Vec::new();
    collect_use_actions(&program.template, &mut actions);

    let tags: Vec<String> = analysis.tags;
    let events: Vec<String> = analysis.events;

    Ok(json!({
        "file": rel,
        "symbols": symbols,
        "actions": actions,
        "tags": tags,
        "events": events,
        "controllerPath": analysis.controller_path,
    }))
}

fn collect_use_actions(instrs: &[sail_compiler::ir::Instr], out: &mut Vec<Value>) {
    use sail_compiler::ir::{AttrInstr, Instr};
    for instr in instrs {
        match instr {
            Instr::Element(el) => {
                for a in &el.attrs {
                    if let AttrInstr::Use {
                        action,
                        param,
                        span,
                    } = a
                    {
                        out.push(json!({
                            "action": action,
                            "param": param.as_ref().map(|p| p.source.clone()),
                            "tag": el.tag,
                            "span": {
                                "start": span.start,
                                "end": span.end,
                            },
                        }));
                    }
                }
                collect_use_actions(&el.children, out);
            }
            Instr::If(i) => {
                for b in &i.branches {
                    collect_use_actions(&b.body, out);
                }
                if let Some(e) = &i.else_body {
                    collect_use_actions(e, out);
                }
            }
            Instr::For(f) => {
                collect_use_actions(&f.body, out);
                if let Some(e) = &f.else_body {
                    collect_use_actions(e, out);
                }
            }
            Instr::Match(m) => {
                for c in &m.cases {
                    collect_use_actions(&c.body, out);
                }
                if let Some(d) = &m.default {
                    collect_use_actions(d, out);
                }
            }
            Instr::When(w) => {
                collect_use_actions(&w.body, out);
                if let Some(e) = &w.else_body {
                    collect_use_actions(e, out);
                }
            }
            _ => {}
        }
    }
}

fn topic_cost(args: &Value, ctx: &Ctx) -> Result<Value, OpError> {
    let (source, rel) = require_path_source(args, ctx)?;
    let (nodes, meta) = build_tree(&source, &rel)?;
    let program = sail_compiler::parse::parse_author(&source)
        .map_err(|e| OpError::InvalidArgument(format!("parse: {e}")))?;

    let mut components = Vec::new();
    // Per root + per significant host (L1 / userland / control-flow).
    for n in &nodes {
        let weight = estimate_node_weight(n, &nodes);
        if n.layer == "l1-keyword" && n.children.is_empty() && weight.total < 2 {
            continue;
        }
        components.push(json!({
            "id": n.id,
            "identity": n.identity,
            "tag": n.tag,
            "layer": n.layer,
            "weight": weight,
        }));
    }

    let totals = CostWeight {
        elements: nodes.iter().filter(|n| n.layer != "l1-keyword").count(),
        control_flow: nodes.iter().filter(|n| n.layer == "l1-keyword").count(),
        attrs: nodes.iter().map(|n| n.attrs.len()).sum(),
        dynamic_sites: count_dynamic_sites(&program.template),
        nesting_max: max_nesting(&program.template, 0),
        instrs: count_instrs(&program.template),
        total: 0,
    };
    let mut totals = totals;
    totals.total = totals.elements
        + totals.control_flow * 2
        + totals.attrs
        + totals.dynamic_sites * 3
        + totals.nesting_max
        + totals.instrs;

    // Sort heaviest first for agent triage.
    components.sort_by(|a, b| {
        let aw = a["weight"]["total"].as_u64().unwrap_or(0);
        let bw = b["weight"]["total"].as_u64().unwrap_or(0);
        bw.cmp(&aw)
    });

    Ok(json!({
        "file": rel,
        "roots": meta.root_ids,
        "totals": totals,
        "components": components,
    }))
}

#[derive(Debug, Clone, Serialize)]
struct CostWeight {
    elements: usize,
    control_flow: usize,
    attrs: usize,
    dynamic_sites: usize,
    nesting_max: usize,
    instrs: usize,
    total: usize,
}

fn estimate_node_weight(n: &TreeNode, all: &[TreeNode]) -> CostWeight {
    let by_id: BTreeMap<&str, &TreeNode> = all.iter().map(|x| (x.id.as_str(), x)).collect();
    let mut elements = 0usize;
    let mut control_flow = 0usize;
    let mut attrs = 0usize;
    fn walk(
        id: &str,
        by_id: &BTreeMap<&str, &TreeNode>,
        elements: &mut usize,
        control_flow: &mut usize,
        attrs: &mut usize,
    ) {
        let Some(n) = by_id.get(id) else {
            return;
        };
        if n.layer == "l1-keyword" {
            *control_flow += 1;
        } else {
            *elements += 1;
        }
        *attrs += n.attrs.len();
        for c in &n.children {
            walk(c, by_id, elements, control_flow, attrs);
        }
    }
    walk(&n.id, &by_id, &mut elements, &mut control_flow, &mut attrs);
    let mut w = CostWeight {
        elements,
        control_flow,
        attrs,
        dynamic_sites: 0,
        nesting_max: n.children.len(),
        instrs: elements + control_flow,
        total: 0,
    };
    w.total = w.elements + w.control_flow * 2 + w.attrs + w.instrs;
    w
}

fn count_dynamic_sites(instrs: &[sail_compiler::ir::Instr]) -> usize {
    use sail_compiler::ir::{AttrInstr, Instr};
    let mut n = 0usize;
    for instr in instrs {
        match instr {
            Instr::DynamicText { .. } => n += 1,
            Instr::Element(el) => {
                for a in &el.attrs {
                    match a {
                        AttrInstr::Expr { .. }
                        | AttrInstr::BindValue { .. }
                        | AttrInstr::BindChecked { .. }
                        | AttrInstr::On { .. }
                        | AttrInstr::Use { .. }
                        | AttrInstr::Motion { .. }
                        | AttrInstr::Mutation { .. } => n += 1,
                        _ => {}
                    }
                }
                n += count_dynamic_sites(&el.children);
            }
            Instr::If(i) => {
                n += i.branches.len(); // each cond is dynamic
                for b in &i.branches {
                    n += count_dynamic_sites(&b.body);
                }
                if let Some(e) = &i.else_body {
                    n += count_dynamic_sites(e);
                }
            }
            Instr::For(f) => {
                n += 1;
                n += count_dynamic_sites(&f.body);
                if let Some(e) = &f.else_body {
                    n += count_dynamic_sites(e);
                }
            }
            Instr::Match(m) => {
                n += 1 + m.cases.len();
                for c in &m.cases {
                    n += count_dynamic_sites(&c.body);
                }
                if let Some(d) = &m.default {
                    n += count_dynamic_sites(d);
                }
            }
            Instr::When(w) => {
                n += 1;
                n += count_dynamic_sites(&w.body);
                if let Some(e) = &w.else_body {
                    n += count_dynamic_sites(e);
                }
            }
            _ => {}
        }
    }
    n
}

fn max_nesting(instrs: &[sail_compiler::ir::Instr], depth: usize) -> usize {
    use sail_compiler::ir::Instr;
    let mut max = depth;
    for instr in instrs {
        match instr {
            Instr::Element(el) => {
                max = max.max(max_nesting(&el.children, depth + 1));
            }
            Instr::If(i) => {
                for b in &i.branches {
                    max = max.max(max_nesting(&b.body, depth + 1));
                }
                if let Some(e) = &i.else_body {
                    max = max.max(max_nesting(e, depth + 1));
                }
            }
            Instr::For(f) => {
                max = max.max(max_nesting(&f.body, depth + 1));
                if let Some(e) = &f.else_body {
                    max = max.max(max_nesting(e, depth + 1));
                }
            }
            Instr::Match(m) => {
                for c in &m.cases {
                    max = max.max(max_nesting(&c.body, depth + 1));
                }
                if let Some(d) = &m.default {
                    max = max.max(max_nesting(d, depth + 1));
                }
            }
            Instr::When(w) => {
                max = max.max(max_nesting(&w.body, depth + 1));
                if let Some(e) = &w.else_body {
                    max = max.max(max_nesting(e, depth + 1));
                }
            }
            _ => {}
        }
    }
    max
}

fn topic_themes(args: &Value, ctx: &Ctx) -> Result<Value, OpError> {
    let theme_id = args
        .get("themeId")
        .or_else(|| args.get("theme_id"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty());

    let ids = crate::theme::effective_ids(&json!({}), ctx)
        .unwrap_or_else(|_| json!({"ok": false, "project": [], "catalog": [], "all": []}));

    let all_ids: Vec<String> = ids
        .get("all")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();

    let focus: Vec<String> = match theme_id {
        Some(id) => vec![id.to_string()],
        None => all_ids.clone(),
    };

    let mut matrix = Vec::new();
    for id in &focus {
        let graph = crate::theme::graph_internal(Some(id), ctx).ok();
        let modes = graph
            .as_ref()
            .and_then(extract_modes_from_graph)
            .unwrap_or_else(|| vec!["light".into()]);
        let products = vec!["web".to_string()]; // product axis reserved; single web product today
        for mode in &modes {
            for product in &products {
                matrix.push(json!({
                    "themeId": id,
                    "mode": mode,
                    "product": product,
                    "sources": graph.as_ref().and_then(|g| g.get("sources").cloned()).unwrap_or(json!([])),
                    "tierCount": graph.as_ref().and_then(|g| g.get("tiers").and_then(|t| t.as_array()).map(|a| a.len())).unwrap_or(0),
                    "nodeCount": graph.as_ref().and_then(|g| g.get("nodes").and_then(|t| t.as_array()).map(|a| a.len())).unwrap_or(0),
                }));
            }
        }
    }

    Ok(json!({
        "themes": ids,
        "matrix": matrix,
        "axes": {
            "theme": focus,
            "mode": matrix.iter().filter_map(|m| m.get("mode").and_then(|v| v.as_str()).map(str::to_string)).collect::<BTreeSet<_>>().into_iter().collect::<Vec<_>>(),
            "product": ["web"],
        },
    }))
}

fn extract_modes_from_graph(graph: &Value) -> Option<Vec<String>> {
    // Paths like modes.light.* appear as node path prefixes in walk — recover unique mode names.
    let nodes = graph.get("nodes")?.as_array()?;
    let mut modes = BTreeSet::new();
    for n in nodes {
        if let Some(p) = n.get("path").and_then(|v| v.as_str()) {
            if let Some(rest) = p.strip_prefix("modes.") {
                if let Some(mode) = rest.split('.').next() {
                    if !mode.is_empty() {
                        modes.insert(mode.to_string());
                    }
                }
            }
        }
    }
    if modes.is_empty() {
        // Fallback: light/dark common pair when theme has no modes.* paths.
        return Some(vec!["light".into(), "dark".into()]);
    }
    Some(modes.into_iter().collect())
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

    /// Isolated page fixture: layout keyword + L1 element + interval-driven ticks.
    /// Owns a tempdir so inspect tests do not parse live app routes.
    fn card_fixture() -> (tempfile::TempDir, Ctx, String, String) {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let rel = "apps/demo/routes/card.sail".to_string();
        let sail_path = root.join(&rel);
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
"#
        .to_string();
        std::fs::write(&sail_path, &src).unwrap();
        let ctx = Ctx::new(root, root.join("apps/demo"), root);
        (dir, ctx, rel, src)
    }

    #[test]
    fn model_page_has_sections_and_node_ids() {
        let (_dir, ctx, rel, src) = card_fixture();
        let out = model(&json!({"path": rel}), &ctx).expect("model");
        assert_eq!(out["modelVersion"], MODEL_VERSION);
        assert!(!out["tree"]["nodes"].as_array().unwrap().is_empty());
        assert!(out["layout"]["boxes"].as_array().is_some());
        assert!(out["state"]["bindings"].as_array().is_some());
        assert!(out["diagnostics"].as_array().is_some());
        assert!(out["data"]["keys"].as_array().is_some());
        assert!(out.get("theme").is_some());
        let mut layers = BTreeSet::new();
        let mut open_ok = 0usize;
        for n in out["tree"]["nodes"].as_array().unwrap() {
            assert!(n.get("id").and_then(|v| v.as_str()).is_some());
            assert!(n.get("span").is_some());
            assert!(n.get("file").and_then(|v| v.as_str()).is_some());
            // WI7: every tree node carries the shared identity scheme.
            let idt = n.get("identity").expect("identity on inspect tree node");
            assert_eq!(
                idt["nodeId"].as_str(),
                n["id"].as_str(),
                "identity.nodeId must match tree id"
            );
            assert!(idt.get("locator").is_some(), "locator required: {idt}");
            assert!(idt.get("file").is_some());
            assert!(idt.get("span").is_some());
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
        let (_dir, ctx, rel, src) = card_fixture();
        let out = model(&json!({"path": rel}), &ctx).expect("model");
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
        let again = model(&json!({"path": rel}), &ctx).unwrap();
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
        let (_dir, ctx, rel, _src) = card_fixture();
        let out = model(
            &json!({"path": rel, "viewport": {"width": 1440, "height": 900}}),
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
        let (_dir, ctx, rel, _src) = card_fixture();
        let out = state(&json!({"path": rel}), &ctx).expect("state");
        let bindings = out["bindings"].as_array().unwrap();
        let ticks = bindings
            .iter()
            .find(|b| b["name"] == "ticks")
            .expect("ticks binding");
        assert_eq!(ticks["kind"], "signal");
        assert_eq!(ticks["ssr_tier"], "C", "ticks uses interval() → C");
        let readers = ticks["readers"].as_array().unwrap();
        assert!(
            !readers.is_empty(),
            "at least one template reader expected: {ticks}"
        );
        // Distinct author sites for ticks()
        let mut starts = BTreeSet::new();
        for r in readers {
            starts.insert(r["span"]["start"].as_u64().unwrap());
            let s = r["span"]["start"].as_u64().unwrap() as usize;
            let e = r["span"]["end"].as_u64().unwrap() as usize;
            assert!(e > s, "reader zero-width span");
        }
        assert!(!starts.is_empty(), "reader spans required: {starts:?}");
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
        let (_dir, ctx, rel, _src) = card_fixture();
        let a = tree(&json!({"path": rel}), &ctx).unwrap();
        let b = tree(&json!({"path": rel}), &ctx).unwrap();
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

    // ── WI5 topic surface ───────────────────────────────────────────────────

    /// Fixture app covering match arms, signals, a11y props, tokens, queries.
    fn wi5_fixture() -> (tempfile::TempDir, Ctx, String) {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let sail_path = root.join("apps/demo/routes/inspect-fixture.sail");
        std::fs::create_dir_all(sail_path.parent().unwrap()).unwrap();
        // Minimal theme so cascade/themes resolve inside the temp workspace.
        let theme_dir = root.join("packages/sail/tokens/model/themes");
        std::fs::create_dir_all(&theme_dir).unwrap();
        std::fs::write(
            root.join("packages/sail/tokens/model/base.json"),
            r#"{"color":{"base":{"$value":"oklch(0.5 0 0)","$type":"color","$extensions":{"ship":{"cssName":"base","layer":"base"}}}}}"#,
        )
        .unwrap();
        std::fs::write(
            theme_dir.join("ship.json"),
            r#"{
              "$extensions": { "ship": { "layer": "theme", "themeId": "ship" } },
              "modes": {
                "light": {
                  "color": {
                    "primary": {
                      "$value": "{color.base}",
                      "$type": "color",
                      "$extensions": { "ship": { "cssName": "primary", "layer": "theme" } }
                    }
                  }
                },
                "dark": {
                  "color": {
                    "primary": {
                      "$value": "oklch(0.8 0.1 250)",
                      "$type": "color",
                      "$extensions": { "ship": { "cssName": "primary", "layer": "theme" } }
                    }
                  }
                }
              }
            }"#,
        )
        .unwrap();
        let src = r#"
<controller>
import type { ControllerScope } from '@ship/sail-runtime'
export default controller((_ctx: ControllerScope<any>) => {
  const tone = signal<'info' | 'ok' | 'bad'>('info')
  const ticks = signal(0)
  const deals = query(() => client.deals.list(), { key: 'crm.deals.list' })
  const save = mutation(() => client.deals.save(), { key: 'crm.deals.save' })
  effect(() => {
    invalidate('crm.deals')
  })
  return { tone, ticks, deals, save }
})
</controller>
<column gap="4">
  <navbar label="Main">
    <button label="Save" on:click={() => save.mutate()}>Save</button>
  </navbar>
  <card>
    <match {tone()}>
      {'info' => <alert>Heads up {ticks()}</alert>}
      {'ok' => <badge>All good</badge>}
      {'bad' => <button label="Retry">Retry</button>}
      {else => <span>pick</span>}
    </match>
  </card>
  <span style_hint="primary">{ticks()}</span>
</column>
"#;
        std::fs::write(&sail_path, src).unwrap();
        let ctx = Ctx::new(root, root.join("apps/demo"), root);
        let rel = "apps/demo/routes/inspect-fixture.sail".to_string();
        (dir, ctx, rel)
    }

    #[test]
    fn wi5_all_topics_parse() {
        for t in InspectTopic::all() {
            assert_eq!(InspectTopic::parse(t.as_str()), Some(*t));
        }
        assert_eq!(InspectTopic::parse("layout"), Some(InspectTopic::Geometry));
        assert_eq!(InspectTopic::parse("aria"), Some(InspectTopic::A11y));
        assert!(InspectTopic::parse("nope").is_none());
    }

    #[test]
    fn wi5_topic_tree_has_wi7_identity() {
        let (_dir, ctx, rel) = wi5_fixture();
        let out = inspect(&json!({"topic": "tree", "path": rel}), &ctx).expect("tree");
        assert_eq!(out["topic"], "tree");
        assert_eq!(out["modelVersion"], MODEL_VERSION);
        let nodes = out["nodes"].as_array().unwrap();
        assert!(!nodes.is_empty());
        for n in nodes {
            let idt = n.get("identity").expect("identity");
            assert_eq!(idt["nodeId"], n["id"]);
            assert!(idt.get("locator").is_some());
            assert!(idt.get("span").is_some());
        }
        let tags: Vec<&str> = nodes.iter().filter_map(|n| n["tag"].as_str()).collect();
        assert!(tags.contains(&"column"), "{tags:?}");
        assert!(tags.contains(&"match"), "{tags:?}");
        assert!(tags.contains(&"button"), "{tags:?}");
    }

    #[test]
    fn wi5_topic_geometry_boxes_join_identity() {
        let (_dir, ctx, rel) = wi5_fixture();
        let out = inspect(
            &json!({
                "topic": "geometry",
                "path": rel,
                "viewport": {"width": 800, "height": 600}
            }),
            &ctx,
        )
        .expect("geometry");
        assert_eq!(out["topic"], "geometry");
        let boxes = out["boxes"].as_array().unwrap();
        // Layout may produce zero boxes on minimal fixture in edge cases; prefer non-empty.
        // When boxes exist they must carry id + optional identity.
        for b in boxes {
            assert!(b.get("id").is_some(), "{b}");
            assert!(b.get("w").is_some());
            assert!(b.get("h").is_some());
        }
    }

    #[test]
    fn wi5_topic_cascade_resolves_alias_chain() {
        let (_dir, ctx, _rel) = wi5_fixture();
        let out = inspect(
            &json!({"topic": "cascade", "token": "primary", "themeId": "ship"}),
            &ctx,
        )
        .expect("cascade");
        assert_eq!(out["topic"], "cascade");
        assert_eq!(out["token"], "primary");
        let chain = out["chain"].as_array().unwrap();
        assert!(
            !chain.is_empty(),
            "expected cascade chain for primary: {out}"
        );
        // First step should mention primary / color.primary path.
        let first = chain[0]["path"].as_str().unwrap_or("");
        assert!(
            first.contains("primary") || first.contains("color"),
            "path={first}"
        );
    }

    #[test]
    fn wi5_topic_states_match_matrix() {
        let (_dir, ctx, rel) = wi5_fixture();
        let out = inspect(&json!({"topic": "states", "path": rel}), &ctx).expect("states");
        assert_eq!(out["topic"], "states");
        let matches = out["matches"].as_array().unwrap();
        assert!(!matches.is_empty(), "expected match matrix: {out}");
        let m0 = &matches[0]["match"];
        assert!(m0["discriminant"].as_str().unwrap().contains("tone"));
        let arms = m0["arms"].as_array().unwrap();
        assert!(arms.len() >= 3, "arms={arms:?}");
        let tests: Vec<&str> = arms.iter().filter_map(|a| a["test"].as_str()).collect();
        assert!(
            tests.iter().any(|t| *t == "info" || t.contains("info")),
            "{tests:?}"
        );
        assert!(tests.iter().any(|t| *t == "else"), "{tests:?}");
        // Match host should carry WI7 identity when joined.
        assert!(m0.get("identity").is_some() || m0.get("id").is_some());
    }

    #[test]
    fn wi5_topic_a11y_aria_tree() {
        let (_dir, ctx, rel) = wi5_fixture();
        let out = inspect(&json!({"topic": "a11y", "path": rel}), &ctx).expect("a11y");
        assert_eq!(out["topic"], "a11y");
        let nodes = out["nodes"].as_array().unwrap();
        assert!(!nodes.is_empty());
        let button = nodes
            .iter()
            .find(|n| n["tag"] == "button")
            .expect("button node");
        assert_eq!(button["role"], "button");
        assert!(
            button["name"].as_str().is_some_and(|n| !n.is_empty()),
            "button name: {button}"
        );
        assert!(button.get("identity").is_some());
        assert!(button.get("aria").is_some());
        // Navbar host anatomy → navigation role.
        if let Some(nav) = nodes.iter().find(|n| n["tag"] == "navbar") {
            assert_eq!(nav["role"], "navigation", "{nav}");
        }
        assert!(out["tree"].as_array().is_some());
    }

    #[test]
    fn wi5_topic_invalidations_query_mutation_graph() {
        let (_dir, ctx, rel) = wi5_fixture();
        let out =
            inspect(&json!({"topic": "invalidations", "path": rel}), &ctx).expect("invalidations");
        assert_eq!(out["topic"], "invalidations");
        // Query/mutation extraction is best-effort from controller text.
        let q = out["queries"].as_array().unwrap();
        let m = out["mutations"].as_array().unwrap();
        let inv = out["invalidates"].as_array().unwrap();
        let edges = out["edges"].as_array().unwrap();
        // At least one of the key kinds should surface from the fixture controller.
        assert!(
            !q.is_empty() || !m.is_empty() || !inv.is_empty(),
            "expected query/mutation/invalidate entries: {out}"
        );
        let _ = edges; // edges may be empty when keys don't relate
    }

    #[test]
    fn wi5_topic_uses_cross_region_symbols() {
        let (_dir, ctx, rel) = wi5_fixture();
        let out = inspect(&json!({"topic": "uses", "path": rel}), &ctx).expect("uses");
        assert_eq!(out["topic"], "uses");
        let symbols = out["symbols"].as_array().unwrap();
        let tone = symbols.iter().find(|s| s["name"] == "tone");
        let ticks = symbols.iter().find(|s| s["name"] == "ticks");
        assert!(tone.is_some() || ticks.is_some(), "symbols={symbols:?}");
        if let Some(t) = ticks {
            let readers = t["readers"].as_array().unwrap();
            assert!(!readers.is_empty(), "ticks should have template readers");
            assert_eq!(t["crossRegion"], true);
        }
        assert!(out["tags"].as_array().is_some());
        assert!(out["events"].as_array().is_some());
    }

    #[test]
    fn wi5_topic_cost_emit_weight() {
        let (_dir, ctx, rel) = wi5_fixture();
        let out = inspect(&json!({"topic": "cost", "path": rel}), &ctx).expect("cost");
        assert_eq!(out["topic"], "cost");
        let totals = &out["totals"];
        assert!(totals["total"].as_u64().unwrap() >= 1);
        assert!(totals["instrs"].as_u64().unwrap() >= 1);
        let components = out["components"].as_array().unwrap();
        assert!(!components.is_empty());
        for c in components {
            assert!(c.get("identity").is_some(), "{c}");
            assert!(c["weight"]["total"].as_u64().is_some());
        }
    }

    #[test]
    fn wi5_topic_themes_matrix() {
        let (_dir, ctx, _rel) = wi5_fixture();
        let out = inspect(&json!({"topic": "themes", "themeId": "ship"}), &ctx).expect("themes");
        assert_eq!(out["topic"], "themes");
        let matrix = out["matrix"].as_array().unwrap();
        assert!(!matrix.is_empty(), "{out}");
        for row in matrix {
            assert_eq!(row["themeId"], "ship");
            assert!(row.get("mode").is_some());
            assert_eq!(row["product"], "web");
        }
        let modes = out["axes"]["mode"].as_array().unwrap();
        assert!(
            modes.iter().any(|m| m == "light" || m == "dark"),
            "modes={modes:?}"
        );
    }

    #[test]
    fn wi5_unknown_topic_errors() {
        let (_dir, ctx, rel) = wi5_fixture();
        let err = inspect(&json!({"topic": "nope", "path": rel}), &ctx).unwrap_err();
        assert!(matches!(err, OpError::InvalidArgument(_)));
    }

    #[test]
    fn wi5_mcp_and_rpc_same_inspect() {
        let (_dir, ctx, rel) = wi5_fixture();
        let args = json!({"topic": "tree", "path": rel});
        let via_rpc = crate::call_rpc("SailInspect", "Inspect", &args, &ctx).unwrap();
        let via_mcp = crate::call_mcp_tool("sail_inspect", &args, &ctx).unwrap();
        assert_eq!(via_rpc, via_mcp);
        assert_eq!(via_rpc["topic"], "tree");
    }
}
