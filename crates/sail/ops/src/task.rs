//! `SailTask` RPCs under `<project>/.sail/tasks/<id>/task.json` + `artifacts/`.
//!
//! Threads are A-anchored: `{ nodeId, elementPath, contentHash }` with drift
//! `ok | moved | lost` (lost never silently rebinds). Storage is gitignored;
//! `CommitIntent` exports a decision record into the repo explicitly.
//!
//! Read–modify–write with temp file + atomic rename (no flock; last writer wins).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use ulid::Ulid;

use crate::ctx::Ctx;
use crate::error::OpError;
use crate::inspect;
use crate::project_store;

/// Max artifact payload (decoded) — ~5 MiB.
pub const MAX_ARTIFACT_BYTES: usize = 5 * 1024 * 1024;

/// Valid task status values (any → any in v1).
const STATUSES: &[&str] = &[
    "backlog",
    "todo",
    "in_progress",
    "review",
    "done",
    "cancelled",
];

const OPEN_STATUSES: &[&str] = &["backlog", "todo", "in_progress", "review"];

// ── Types ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Actor {
    kind: String,
    name: String,
}

impl Actor {
    fn human() -> Self {
        Self {
            kind: "human".into(),
            name: "user".into(),
        }
    }

    fn agent() -> Self {
        let name = std::env::var("SAIL_AGENT_NAME").unwrap_or_else(|_| "agent".into());
        Self {
            kind: "agent".into(),
            name,
        }
    }

    fn from_value(v: Option<&Value>) -> Self {
        let Some(v) = v else {
            return Self::human();
        };
        let kind = v
            .get("kind")
            .and_then(|k| k.as_str())
            .unwrap_or("human")
            .to_string();
        let name = v
            .get("name")
            .and_then(|n| n.as_str())
            .unwrap_or(if kind == "agent" { "agent" } else { "user" })
            .to_string();
        Self { kind, name }
    }
}

/// Source span for an anchored node (byte offsets + optional line/col).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct AnchorSpan {
    start: u64,
    end: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    line: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    col: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    end_line: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    end_col: Option<u64>,
}

impl AnchorSpan {
    fn from_model_span(v: &Value) -> Option<Self> {
        let start = v.get("start")?.as_u64()?;
        let end = v.get("end")?.as_u64()?;
        Some(Self {
            start,
            end,
            line: v.get("line").and_then(|x| x.as_u64()),
            col: v.get("col").and_then(|x| x.as_u64()),
            end_line: v.get("end_line").and_then(|x| x.as_u64()),
            end_col: v.get("end_col").and_then(|x| x.as_u64()),
        })
    }

    fn display(&self) -> String {
        match (self.line, self.end_line) {
            (Some(a), Some(b)) if a != b => format!("{a}–{b}"),
            (Some(a), _) => format!("{a}"),
            _ => format!("{}–{}", self.start, self.end),
        }
    }
}

/// A-anchor: node identity + path + content hash (drift: ok | moved | lost).
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Anchor {
    #[serde(rename = "nodeId")]
    node_id: String,
    #[serde(rename = "elementPath")]
    element_path: String,
    #[serde(rename = "contentHash")]
    content_hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    span: Option<AnchorSpan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ArtifactRef {
    id: String,
    path: String,
    mime: String,
    /// `file` | `render`
    kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    label: Option<String>,
    /// Region metadata for render artifacts (nodeId, rect, …).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    region: Option<Value>,
    created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Event {
    id: String,
    at: String,
    #[serde(rename = "type")]
    event_type: String,
    actor: Actor,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    body: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    meta: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TaskDoc {
    id: String,
    version: u32,
    title: String,
    status: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    labels: Vec<String>,
    created_at: String,
    updated_at: String,
    created_by: Actor,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    assignee: Option<Actor>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    route: Option<String>,
    /// Node anchor (threads on components). Absent for free-floating tasks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    anchor: Option<Anchor>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    artifacts: Vec<ArtifactRef>,
    #[serde(default)]
    events: Vec<Event>,
}

/// Drift resolution result (computed; not always persisted).
#[derive(Debug, Clone, PartialEq, Eq)]
enum DriftStatus {
    Ok,
    Moved,
    Lost,
    /// No anchor on the task.
    None,
}

impl DriftStatus {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Moved => "moved",
            Self::Lost => "lost",
            Self::None => "none",
        }
    }
}

#[derive(Debug, Clone)]
struct DriftResolved {
    status: DriftStatus,
    /// Current span when ok/moved; None when lost/none.
    span: Option<AnchorSpan>,
    /// Live node id when ok/moved (may differ from stored when moved).
    live_node_id: Option<String>,
    live_hash: Option<String>,
}

// ── Path policy ─────────────────────────────────────────────────────────────

/// Crockford Base32 alphabet used by ULID (excludes I, L, O, U).
fn is_crockford_char(b: u8) -> bool {
    matches!(
        b,
        b'0'..=b'9'
            | b'A'..=b'H'
            | b'J'..=b'K'
            | b'M'..=b'N'
            | b'P'..=b'T'
            | b'V'..=b'Z'
            | b'a'..=b'h'
            | b'j'..=b'k'
            | b'm'..=b'n'
            | b'p'..=b't'
            | b'v'..=b'z'
    )
}

/// Task id: Crockford ULID (26 chars) or later human slug `^[a-z0-9][a-z0-9_-]{2,64}$`.
pub fn is_valid_task_id(id: &str) -> bool {
    if id.is_empty() || id.len() > 64 {
        return false;
    }
    if id.contains("..") || id.contains('/') || id.contains('\\') {
        return false;
    }
    // ULID (Crockford base32, 26 chars) — case-insensitive, exclude I/L/O/U.
    if id.len() == 26 {
        return id.bytes().all(is_crockford_char);
    }
    let mut chars = id.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_lowercase() && !first.is_ascii_digit() {
        return false;
    }
    let rest_ok =
        chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-');
    rest_ok && id.len() >= 3
}

/// Sanitize artifact filename: basename only, no `..`, no absolute.
pub fn sanitize_artifact_filename(name: &str) -> Result<String, OpError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(OpError::InvalidArgument(
            "filename must be non-empty".into(),
        ));
    }
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        return Err(OpError::PermissionDenied(
            "artifact filename must not contain path separators or ..".into(),
        ));
    }
    if Path::new(name).is_absolute() {
        return Err(OpError::PermissionDenied(
            "artifact filename must not be absolute".into(),
        ));
    }
    // Keep safe charset; allow dots for extensions.
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-')
    {
        return Err(OpError::InvalidArgument(
            "artifact filename has invalid characters".into(),
        ));
    }
    if name.starts_with('.') {
        return Err(OpError::InvalidArgument(
            "artifact filename must not start with .".into(),
        ));
    }
    Ok(name.to_string())
}

fn validate_status(s: &str) -> Result<(), OpError> {
    if STATUSES.contains(&s) {
        Ok(())
    } else {
        Err(OpError::InvalidArgument(format!(
            "status must be one of {}: got {s:?}",
            STATUSES.join("|")
        )))
    }
}

fn now_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let secs = ms / 1000;
    let millis = ms % 1000;
    format_utc_millis(secs as i64, millis as u32)
}

fn format_utc_millis(secs: i64, millis: u32) -> String {
    let days = secs.div_euclid(86_400);
    let day_secs = secs.rem_euclid(86_400) as u32;
    let h = day_secs / 3600;
    let m = (day_secs % 3600) / 60;
    let s = day_secs % 60;

    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = (z - era * 146_097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if mo <= 2 { y + 1 } else { y };

    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{m:02}:{s:02}.{millis:03}Z")
}

fn new_ulid() -> String {
    Ulid::new().to_string()
}

/// Resolve project root from args `project` or ctx.project_root; confine under workspace.
fn resolve_project(args: &Value, ctx: &Ctx) -> Result<PathBuf, OpError> {
    let workspace = confine_existing(&ctx.workspace_root, &ctx.workspace_root)?;
    if let Some(p) = args.get("project").and_then(|v| v.as_str()).map(str::trim) {
        if p.is_empty() {
            return Err(OpError::InvalidArgument("project must be non-empty".into()));
        }
        if p.contains("..") {
            return Err(OpError::PermissionDenied(
                "project path must not contain ..".into(),
            ));
        }
        let pb = PathBuf::from(p);
        let abs = if pb.is_absolute() {
            pb
        } else {
            workspace.join(pb)
        };
        return confine_existing(&workspace, &abs);
    }
    confine_existing(&workspace, &ctx.project_root)
}

fn tasks_root(project: &Path) -> PathBuf {
    project.join(".sail").join("tasks")
}

fn task_dir(project: &Path, id: &str) -> PathBuf {
    tasks_root(project).join(id)
}

fn task_json_path(project: &Path, id: &str) -> PathBuf {
    task_dir(project, id).join("task.json")
}

fn ensure_under_project(project: &Path, target: &Path) -> Result<PathBuf, OpError> {
    let canon_proj = project
        .canonicalize()
        .map_err(|e| OpError::Internal(format!("project root {}: {e}", project.display())))?;
    if target.exists() {
        let canon = target
            .canonicalize()
            .map_err(|e| OpError::NotFound(format!("canonicalize {}: {e}", target.display())))?;
        if !canon.starts_with(&canon_proj) {
            return Err(OpError::PermissionDenied(
                "path escapes project root".into(),
            ));
        }
        return Ok(canon);
    }
    let parent = target.parent().unwrap_or(project);
    if !parent.exists() {
        std::fs::create_dir_all(parent)
            .map_err(|e| OpError::Internal(format!("mkdir {}: {e}", parent.display())))?;
    }
    let canon_parent = parent
        .canonicalize()
        .map_err(|e| OpError::InvalidArgument(format!("parent path invalid: {e}")))?;
    if !canon_parent.starts_with(&canon_proj) {
        return Err(OpError::PermissionDenied(
            "path escapes project root".into(),
        ));
    }
    Ok(canon_parent.join(target.file_name().unwrap_or_default()))
}

fn confine_existing(workspace: &Path, path: &Path) -> Result<PathBuf, OpError> {
    let canon_root = workspace
        .canonicalize()
        .map_err(|e| OpError::Internal(format!("workspace root {}: {e}", workspace.display())))?;
    let canon = path
        .canonicalize()
        .map_err(|e| OpError::InvalidArgument(format!("path invalid {}: {e}", path.display())))?;
    if !canon.starts_with(&canon_root) {
        return Err(OpError::PermissionDenied("path escapes workspace".into()));
    }
    Ok(canon)
}

/// Ensure `.sail/tasks` exists. Returns soft warnings when gitignore/README
/// ensure fails (task create still proceeds so the store is usable).
fn ensure_store(project: &Path) -> Result<Vec<String>, OpError> {
    let mut warnings = Vec::new();
    if let Err(e) = project_store::ensure_project_store(project) {
        warnings.push(format!("gitignore_ensure_failed: {e}"));
        let sail = project.join(".sail");
        if let Err(e) = std::fs::create_dir_all(&sail) {
            return Err(OpError::Internal(format!("mkdir {}: {e}", sail.display())));
        }
    }
    let tasks = tasks_root(project);
    std::fs::create_dir_all(&tasks)
        .map_err(|e| OpError::Internal(format!("mkdir {}: {e}", tasks.display())))?;
    Ok(warnings)
}

fn load_task(project: &Path, id: &str) -> Result<TaskDoc, OpError> {
    if !is_valid_task_id(id) {
        return Err(OpError::InvalidArgument(format!("invalid task id: {id}")));
    }
    let path = task_json_path(project, id);
    let resolved = ensure_under_project(project, &path)?;
    if !resolved.is_file() {
        return Err(OpError::NotFound(format!("task {id}")));
    }
    let raw = std::fs::read_to_string(&resolved)
        .map_err(|e| OpError::Internal(format!("read task {id}: {e}")))?;
    serde_json::from_str(&raw).map_err(|e| OpError::Internal(format!("parse task {id}: {e}")))
}

fn save_task(project: &Path, task: &TaskDoc) -> Result<(), OpError> {
    if !is_valid_task_id(&task.id) {
        return Err(OpError::InvalidArgument(format!(
            "invalid task id: {}",
            task.id
        )));
    }
    let dir = task_dir(project, &task.id);
    let dir_resolved = ensure_under_project(project, &dir)?;
    std::fs::create_dir_all(&dir_resolved)
        .map_err(|e| OpError::Internal(format!("mkdir {}: {e}", dir_resolved.display())))?;
    let art = dir_resolved.join("artifacts");
    if !art.exists() {
        std::fs::create_dir_all(&art)
            .map_err(|e| OpError::Internal(format!("mkdir {}: {e}", art.display())))?;
    }
    let path = dir_resolved.join("task.json");
    let path_resolved = ensure_under_project(project, &path)?;
    let body = serde_json::to_string_pretty(task)
        .map_err(|e| OpError::Internal(format!("serialize task: {e}")))?;
    let tmp = dir_resolved.join(format!("task.json.{}.tmp", Ulid::new()));
    let tmp_resolved = ensure_under_project(project, &tmp)?;
    std::fs::write(&tmp_resolved, body + "\n")
        .map_err(|e| OpError::Internal(format!("write task temp: {e}")))?;
    std::fs::rename(&tmp_resolved, &path_resolved).map_err(|e| {
        let _ = std::fs::remove_file(&tmp_resolved);
        OpError::Internal(format!("rename task.json: {e}"))
    })?;
    Ok(())
}

// ── Anchor / drift ──────────────────────────────────────────────────────────

/// Extract content hash segment from a model node id (`file#path#hash`).
pub fn content_hash_from_node_id(node_id: &str) -> String {
    node_id
        .rsplit('#')
        .next()
        .filter(|s| !s.is_empty())
        .unwrap_or(node_id)
        .to_string()
}

/// Human element path from a model node JSON.
pub fn element_path_from_node(node: &Value) -> String {
    let file = node.get("file").and_then(|f| f.as_str()).unwrap_or("");
    let base = file.rsplit('/').next().unwrap_or(file);
    let tag = node.get("tag").and_then(|t| t.as_str()).unwrap_or("?");
    let attrs = node.get("attrs").and_then(|a| a.as_object());
    let label = attrs
        .and_then(|a| {
            a.get("name")
                .or_else(|| a.get("id"))
                .or_else(|| a.get("data-demo-node"))
                .and_then(|v| v.as_str())
        })
        .unwrap_or("");
    if label.is_empty() {
        format!("{base} › {tag}")
    } else {
        format!("{base} › {tag} {label}")
    }
}

fn parse_anchor(v: &Value) -> Result<Anchor, OpError> {
    let node_id = v
        .get("nodeId")
        .or_else(|| v.get("node_id"))
        .and_then(|x| x.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| OpError::InvalidArgument("anchor requires non-empty `nodeId`".into()))?
        .to_string();
    let content_hash = v
        .get("contentHash")
        .or_else(|| v.get("content_hash"))
        .and_then(|x| x.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| content_hash_from_node_id(&node_id));
    let element_path = v
        .get("elementPath")
        .or_else(|| v.get("element_path"))
        .and_then(|x| x.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| {
            // Fallback: use node id tail for path label.
            format!("? › {node_id}")
        });
    let file = v
        .get("file")
        .and_then(|x| x.as_str())
        .map(str::to_string)
        .or_else(|| {
            // node id often starts with workspace-relative file path before first #
            let before = node_id.split('#').next()?;
            if before.ends_with(".sail") {
                Some(before.to_string())
            } else {
                None
            }
        });
    let span = v.get("span").and_then(AnchorSpan::from_model_span);
    Ok(Anchor {
        node_id,
        element_path,
        content_hash,
        file,
        span,
    })
}

/// Resolve anchor drift against a set of live model nodes.
///
/// - exact `nodeId` match → `ok` (span from live model)
/// - unique match by `elementPath` or content hash → `moved` (new span; does **not**
///   mutate stored anchor — caller may re-anchor explicitly)
/// - no match → `lost` (never rebinds to a different element silently)
fn resolve_anchor_drift(anchor: &Anchor, nodes: &[Value]) -> DriftResolved {
    let by_id: HashMap<&str, &Value> = nodes
        .iter()
        .filter_map(|n| n.get("id").and_then(|i| i.as_str()).map(|id| (id, n)))
        .collect();

    if let Some(n) = by_id.get(anchor.node_id.as_str()) {
        let span = n.get("span").and_then(AnchorSpan::from_model_span);
        let live_hash = content_hash_from_node_id(anchor.node_id.as_str());
        return DriftResolved {
            status: DriftStatus::Ok,
            span,
            live_node_id: Some(anchor.node_id.clone()),
            live_hash: Some(live_hash),
        };
    }

    // Prefer unique elementPath match.
    let path_matches: Vec<&Value> = nodes
        .iter()
        .filter(|n| element_path_from_node(n) == anchor.element_path)
        .collect();
    if path_matches.len() == 1 {
        let n = path_matches[0];
        let id = n
            .get("id")
            .and_then(|i| i.as_str())
            .unwrap_or("")
            .to_string();
        return DriftResolved {
            status: DriftStatus::Moved,
            span: n.get("span").and_then(AnchorSpan::from_model_span),
            live_node_id: Some(id.clone()),
            live_hash: Some(content_hash_from_node_id(&id)),
        };
    }

    // Unique content-hash match within same file (when known).
    let hash_matches: Vec<&Value> = nodes
        .iter()
        .filter(|n| {
            let id = n.get("id").and_then(|i| i.as_str()).unwrap_or("");
            if content_hash_from_node_id(id) != anchor.content_hash {
                return false;
            }
            if let Some(f) = anchor.file.as_deref() {
                let nf = n.get("file").and_then(|x| x.as_str()).unwrap_or("");
                return nf == f || nf.ends_with(f) || f.ends_with(nf);
            }
            true
        })
        .collect();
    if hash_matches.len() == 1 {
        let n = hash_matches[0];
        let id = n
            .get("id")
            .and_then(|i| i.as_str())
            .unwrap_or("")
            .to_string();
        return DriftResolved {
            status: DriftStatus::Moved,
            span: n.get("span").and_then(AnchorSpan::from_model_span),
            live_node_id: Some(id.clone()),
            live_hash: Some(content_hash_from_node_id(&id)),
        };
    }

    DriftResolved {
        status: DriftStatus::Lost,
        span: None,
        live_node_id: None,
        live_hash: None,
    }
}

fn load_nodes_for_file(file: &str, ctx: &Ctx) -> Result<Vec<Value>, OpError> {
    let ret = inspect::tree(&json!({ "path": file }), ctx)?;
    Ok(ret
        .get("nodes")
        .and_then(|n| n.as_array())
        .cloned()
        .unwrap_or_default())
}

fn resolve_task_drift(task: &TaskDoc, ctx: &Ctx) -> DriftResolved {
    let Some(anchor) = task.anchor.as_ref() else {
        return DriftResolved {
            status: DriftStatus::None,
            span: None,
            live_node_id: None,
            live_hash: None,
        };
    };
    let Some(file) = anchor.file.as_deref().or_else(|| {
        let before = anchor.node_id.split('#').next()?;
        before.ends_with(".sail").then_some(before)
    }) else {
        // No file to inspect — treat as ok with stored span (offline).
        return DriftResolved {
            status: DriftStatus::Ok,
            span: anchor.span.clone(),
            live_node_id: Some(anchor.node_id.clone()),
            live_hash: Some(anchor.content_hash.clone()),
        };
    };
    match load_nodes_for_file(file, ctx) {
        Ok(nodes) => resolve_anchor_drift(anchor, &nodes),
        Err(_) => DriftResolved {
            // File missing → lost (element/source gone).
            status: DriftStatus::Lost,
            span: None,
            live_node_id: None,
            live_hash: None,
        },
    }
}

fn drift_json(d: &DriftResolved) -> Value {
    json!({
        "status": d.status.as_str(),
        "span": d.span,
        "liveNodeId": d.live_node_id,
        "liveHash": d.live_hash,
        "spanDisplay": d.span.as_ref().map(|s| s.display()),
    })
}

/// Last conversational actor — same rule as actor filter on ThreadList.
fn last_conversational_actor(task: &TaskDoc) -> &Actor {
    task.events
        .iter()
        .rev()
        .find(|e| matches!(e.event_type.as_str(), "comment" | "created" | "agent_note"))
        .map(|e| &e.actor)
        .unwrap_or(&task.created_by)
}

fn summary(task: &TaskDoc, drift: Option<&DriftResolved>) -> Value {
    let mut v = json!({
        "id": task.id,
        "title": task.title,
        "status": task.status,
        "updated_at": task.updated_at,
        "labels": task.labels,
        "created_by": task.created_by,
        "assignee": task.assignee,
        "route": task.route,
        "anchor": task.anchor,
        "artifact_count": task.artifacts.len(),
        // Align list chips with actor filter (last comment|created|agent_note).
        "last_actor": last_conversational_actor(task),
    });
    if let Some(d) = drift {
        v.as_object_mut()
            .expect("object")
            .insert("drift".into(), drift_json(d));
    }
    v
}

fn task_to_value(task: &TaskDoc) -> Result<Value, OpError> {
    serde_json::to_value(task).map_err(|e| OpError::Internal(format!("task json: {e}")))
}

fn task_with_drift(task: &TaskDoc, ctx: &Ctx) -> Result<Value, OpError> {
    let mut v = task_to_value(task)?;
    let d = resolve_task_drift(task, ctx);
    if let Some(obj) = v.as_object_mut() {
        obj.insert("drift".into(), drift_json(&d));
    }
    Ok(v)
}

// ── RPCs ────────────────────────────────────────────────────────────────────

/// List tasks under project `.sail/tasks` (scan dirs; optional status/label filter).
pub fn list(args: &Value, ctx: &Ctx) -> Result<Value, OpError> {
    thread_list(args, ctx)
}

/// Thread list with drift resolution + open/actor/drift filters.
pub fn thread_list(args: &Value, ctx: &Ctx) -> Result<Value, OpError> {
    let project = resolve_project(args, ctx)?;
    let root = tasks_root(&project);
    if !root.is_dir() {
        return Ok(json!({ "ok": true, "tasks": [] }));
    }
    let status_filter = args.get("status").and_then(|v| v.as_str());
    if let Some(s) = status_filter {
        validate_status(s)?;
    }
    let label_filter = args.get("label").and_then(|v| v.as_str());
    let actor_filter = args
        .get("actor")
        .and_then(|v| v.as_str())
        .map(|s| s.to_ascii_lowercase());
    let drift_filter = args
        .get("drift")
        .and_then(|v| v.as_str())
        .map(|s| s.to_ascii_lowercase());
    let open_only = args.get("open").and_then(|v| v.as_bool()).unwrap_or(false)
        || status_filter.is_none()
            && args
                .get("open")
                .and_then(|v| v.as_str())
                .is_some_and(|s| s == "true" || s == "open");

    let mut tasks: Vec<(String, TaskDoc)> = Vec::new();
    let rd =
        std::fs::read_dir(&root).map_err(|e| OpError::Internal(format!("read tasks dir: {e}")))?;
    for ent in rd.flatten() {
        let name = ent.file_name().to_string_lossy().into_owned();
        if !is_valid_task_id(&name) {
            continue;
        }
        if !ent.path().is_dir() {
            continue;
        }
        match load_task(&project, &name) {
            Ok(t) => tasks.push((t.updated_at.clone(), t)),
            Err(_) => continue,
        }
    }
    tasks.sort_by(|a, b| b.0.cmp(&a.0));

    let out: Vec<Value> = tasks
        .into_iter()
        .filter_map(|(_, t)| {
            if let Some(s) = status_filter {
                if t.status != s {
                    return None;
                }
            } else if open_only && args.get("open").is_some() {
                if !OPEN_STATUSES.contains(&t.status.as_str()) {
                    return None;
                }
            }
            if let Some(lab) = label_filter {
                if !t.labels.iter().any(|l| l == lab) {
                    return None;
                }
            }
            if let Some(ref ak) = actor_filter {
                if ak != "all" {
                    let last_kind = last_conversational_actor(&t).kind.as_str();
                    let matches = match ak.as_str() {
                        "human" | "you" => last_kind == "human",
                        "agent" => last_kind == "agent",
                        _ => true,
                    };
                    if !matches {
                        return None;
                    }
                }
            }
            let drift = resolve_task_drift(&t, ctx);
            if let Some(ref df) = drift_filter {
                if df != "all" && drift.status.as_str() != df.as_str() {
                    return None;
                }
            }
            Some(summary(&t, Some(&drift)))
        })
        .collect();

    Ok(json!({ "ok": true, "tasks": out }))
}

/// Get full task document (+ drift).
pub fn get(args: &Value, ctx: &Ctx) -> Result<Value, OpError> {
    let project = resolve_project(args, ctx)?;
    let id = require_id(args)?;
    let task = load_task(&project, id)?;
    Ok(json!({ "ok": true, "task": task_with_drift(&task, ctx)? }))
}

/// Create a new task (ULID id). Optional `anchor` for component threads.
pub fn create(args: &Value, ctx: &Ctx) -> Result<Value, OpError> {
    let project = resolve_project(args, ctx)?;
    let store_warnings = ensure_store(&project)?;

    let title = args
        .get("title")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| OpError::InvalidArgument("Create requires non-empty `title`".into()))?
        .to_string();

    let status = args
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("todo");
    validate_status(status)?;

    let labels: Vec<String> = args
        .get("labels")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();

    let route = args
        .get("route")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let actor = Actor::from_value(args.get("actor"));
    let id = new_ulid();
    let at = now_iso();

    let anchor = if let Some(av) = args.get("anchor") {
        Some(parse_anchor(av)?)
    } else {
        None
    };

    let mut events = vec![Event {
        id: new_ulid(),
        at: at.clone(),
        event_type: "created".into(),
        actor: actor.clone(),
        body: None,
        meta: anchor.as_ref().map(|a| {
            json!({
                "nodeId": a.node_id,
                "elementPath": a.element_path,
                "contentHash": a.content_hash,
            })
        }),
    }];

    // Optional initial body as first comment (ThreadCreate style).
    if let Some(body) = args
        .get("body")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        events.push(Event {
            id: new_ulid(),
            at: at.clone(),
            event_type: "comment".into(),
            actor: actor.clone(),
            body: Some(body.to_string()),
            meta: None,
        });
    }

    let task = TaskDoc {
        id,
        version: 1,
        title,
        status: status.to_string(),
        labels,
        created_at: at.clone(),
        updated_at: at,
        created_by: actor,
        assignee: None,
        route,
        anchor,
        artifacts: Vec::new(),
        events,
    };
    save_task(&project, &task)?;
    let mut out = json!({ "ok": true, "task": task_with_drift(&task, ctx)? });
    if !store_warnings.is_empty() {
        out.as_object_mut()
            .expect("object")
            .insert("warnings".into(), json!(store_warnings));
    }
    Ok(out)
}

/// Create an anchored thread (title + optional body + required-ish anchor).
pub fn thread_create(args: &Value, ctx: &Ctx) -> Result<Value, OpError> {
    // Require anchor for ThreadCreate (workbench threads are component-scoped).
    if args.get("anchor").is_none() {
        return Err(OpError::InvalidArgument(
            "ThreadCreate requires `anchor` { nodeId, elementPath?, contentHash? }".into(),
        ));
    }
    create(args, ctx)
}

/// Update task fields; appends status event when status changes.
pub fn update(args: &Value, ctx: &Ctx) -> Result<Value, OpError> {
    let project = resolve_project(args, ctx)?;
    let id = require_id(args)?;
    let mut task = load_task(&project, id)?;
    let actor = Actor::from_value(args.get("actor"));
    let at = now_iso();
    let mut changed = false;

    if let Some(title) = args.get("title").and_then(|v| v.as_str()).map(str::trim) {
        if title.is_empty() {
            return Err(OpError::InvalidArgument("title must be non-empty".into()));
        }
        if title != task.title {
            task.title = title.to_string();
            changed = true;
        }
    }

    if let Some(status) = args.get("status").and_then(|v| v.as_str()) {
        validate_status(status)?;
        if status != task.status {
            let from = task.status.clone();
            task.status = status.to_string();
            task.events.push(Event {
                id: new_ulid(),
                at: at.clone(),
                event_type: "status".into(),
                actor: actor.clone(),
                body: None,
                meta: Some(json!({ "from": from, "to": status })),
            });
            changed = true;
        }
    }

    if let Some(labels) = args.get("labels").and_then(|v| v.as_array()) {
        let next: Vec<String> = labels
            .iter()
            .filter_map(|x| x.as_str().map(str::to_string))
            .collect();
        if next != task.labels {
            task.labels = next;
            changed = true;
        }
    }

    if args.get("assignee").is_some() {
        if args.get("assignee").and_then(|v| v.as_object()).is_some() {
            let assignee = Actor::from_value(args.get("assignee"));
            let same = task
                .assignee
                .as_ref()
                .is_some_and(|a| a.kind == assignee.kind && a.name == assignee.name);
            if !same {
                task.assignee = Some(assignee.clone());
                task.events.push(Event {
                    id: new_ulid(),
                    at: at.clone(),
                    event_type: "assignee".into(),
                    actor: actor.clone(),
                    body: None,
                    meta: Some(json!({ "assignee": assignee })),
                });
                changed = true;
            }
        } else if args.get("assignee").map(|v| v.is_null()).unwrap_or(false)
            && task.assignee.is_some()
        {
            task.assignee = None;
            changed = true;
        }
    }

    if let Some(route) = args.get("route").and_then(|v| v.as_str()) {
        let next = Some(route.to_string());
        if next != task.route {
            task.route = next;
            changed = true;
        }
    }

    if changed {
        task.updated_at = at;
        save_task(&project, &task)?;
    }
    Ok(json!({ "ok": true, "task": task_with_drift(&task, ctx)? }))
}

/// Append a comment event.
pub fn comment(args: &Value, ctx: &Ctx) -> Result<Value, OpError> {
    thread_reply(args, ctx)
}

/// Reply on a thread (comment with actor).
pub fn thread_reply(args: &Value, ctx: &Ctx) -> Result<Value, OpError> {
    let project = resolve_project(args, ctx)?;
    let id = require_id(args)?;
    let body = args
        .get("body")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| OpError::InvalidArgument("Reply requires non-empty `body`".into()))?
        .to_string();
    let actor = Actor::from_value(args.get("actor"));
    let mut task = load_task(&project, id)?;
    let at = now_iso();
    let event = Event {
        id: new_ulid(),
        at: at.clone(),
        event_type: "comment".into(),
        actor,
        body: Some(body),
        meta: None,
    };
    let event_val =
        serde_json::to_value(&event).map_err(|e| OpError::Internal(format!("event json: {e}")))?;
    task.events.push(event);
    task.updated_at = at;
    save_task(&project, &task)?;
    Ok(json!({ "ok": true, "event": event_val, "task": task_with_drift(&task, ctx)? }))
}

/// Explicit re-anchor (never automatic for lost).
pub fn reanchor(args: &Value, ctx: &Ctx) -> Result<Value, OpError> {
    let project = resolve_project(args, ctx)?;
    let id = require_id(args)?;
    let av = args
        .get("anchor")
        .ok_or_else(|| OpError::InvalidArgument("Reanchor requires `anchor`".into()))?;
    let anchor = parse_anchor(av)?;
    let actor = Actor::from_value(args.get("actor"));
    let mut task = load_task(&project, id)?;
    let at = now_iso();
    let prev = task.anchor.clone();
    task.anchor = Some(anchor.clone());
    task.events.push(Event {
        id: new_ulid(),
        at: at.clone(),
        event_type: "reanchor".into(),
        actor,
        body: None,
        meta: Some(json!({
            "from": prev,
            "to": {
                "nodeId": anchor.node_id,
                "elementPath": anchor.element_path,
                "contentHash": anchor.content_hash,
            }
        })),
    });
    task.updated_at = at;
    save_task(&project, &task)?;
    Ok(json!({ "ok": true, "task": task_with_drift(&task, ctx)? }))
}

/// Attach an artifact (base64 body or copy-from path under project).
/// Optional `kind: "render"` + `region` metadata (TOOL-3 region crops).
pub fn add_artifact(args: &Value, ctx: &Ctx) -> Result<Value, OpError> {
    let project = resolve_project(args, ctx)?;
    let id = require_id(args)?;
    let mut task = load_task(&project, id)?;
    let actor = Actor::from_value(args.get("actor"));

    let artifact_id = new_ulid();
    let mime = args
        .get("mime")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let kind = args
        .get("kind")
        .and_then(|v| v.as_str())
        .unwrap_or("file")
        .to_string();
    if kind != "file" && kind != "render" {
        return Err(OpError::InvalidArgument(
            "artifact kind must be file|render".into(),
        ));
    }
    let label = args
        .get("label")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let region = args.get("region").cloned();

    let (rel_path, bytes) = if let Some(b64) = args.get("content_base64").and_then(|v| v.as_str()) {
        let filename = args
            .get("filename")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                OpError::InvalidArgument(
                    "AddArtifact with content_base64 requires `filename`".into(),
                )
            })?;
        let safe = sanitize_artifact_filename(filename)?;
        let stored = if let Some(dot) = safe.rfind('.') {
            format!("{artifact_id}{}", &safe[dot..])
        } else {
            format!("{artifact_id}_{safe}")
        };
        reject_oversized_base64(b64)?;
        let data = decode_base64(b64)?;
        if data.len() > MAX_ARTIFACT_BYTES {
            return Err(OpError::InvalidArgument(format!(
                "artifact exceeds {MAX_ARTIFACT_BYTES} bytes"
            )));
        }
        (format!("artifacts/{stored}"), data)
    } else if let Some(src) = args.get("path").and_then(|v| v.as_str()).map(str::trim) {
        if src.is_empty() || src.contains("..") {
            return Err(OpError::PermissionDenied(
                "artifact source path invalid".into(),
            ));
        }
        let pb = PathBuf::from(src);
        let abs = if pb.is_absolute() {
            pb
        } else {
            project.join(&pb)
        };
        let canon = confine_existing(&project, &abs).map_err(|_| {
            OpError::PermissionDenied("artifact source must be under project root".into())
        })?;
        let data = std::fs::read(&canon)
            .map_err(|e| OpError::NotFound(format!("read artifact source: {e}")))?;
        if data.len() > MAX_ARTIFACT_BYTES {
            return Err(OpError::InvalidArgument(format!(
                "artifact exceeds {MAX_ARTIFACT_BYTES} bytes"
            )));
        }
        let base = canon
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("file.bin");
        let safe = sanitize_artifact_filename(base)?;
        let stored = if let Some(dot) = safe.rfind('.') {
            format!("{artifact_id}{}", &safe[dot..])
        } else {
            format!("{artifact_id}_{safe}")
        };
        (format!("artifacts/{stored}"), data)
    } else {
        return Err(OpError::InvalidArgument(
            "AddArtifact requires `content_base64`+`filename` or `path`".into(),
        ));
    };

    let dest = task_dir(&project, id).join(&rel_path);
    let dest_resolved = ensure_under_project(&project, &dest)?;
    if let Some(parent) = dest_resolved.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| OpError::Internal(format!("mkdir artifact parent: {e}")))?;
    }
    let art_root = task_dir(&project, id).join("artifacts");
    let art_canon = ensure_under_project(&project, &art_root)?;
    let dest_parent = dest_resolved
        .parent()
        .ok_or_else(|| OpError::Internal("artifact path has no parent".into()))?;
    let dest_parent_canon = dest_parent
        .canonicalize()
        .map_err(|e| OpError::Internal(format!("artifact parent: {e}")))?;
    if !dest_parent_canon.starts_with(&art_canon) {
        return Err(OpError::PermissionDenied(
            "artifact path escapes task artifacts/".into(),
        ));
    }
    std::fs::write(&dest_resolved, &bytes)
        .map_err(|e| OpError::Internal(format!("write artifact: {e}")))?;

    let at = now_iso();
    let mime_val = mime.unwrap_or_else(|| guess_mime(&rel_path));
    let art = ArtifactRef {
        id: artifact_id.clone(),
        path: rel_path.clone(),
        mime: mime_val.clone(),
        kind: kind.clone(),
        label: label.clone(),
        region: region.clone(),
        created_at: at.clone(),
    };
    task.artifacts.push(art);
    task.events.push(Event {
        id: new_ulid(),
        at: at.clone(),
        event_type: "artifact_added".into(),
        actor,
        body: None,
        meta: Some(json!({
            "artifact_id": artifact_id,
            "path": rel_path,
            "mime": mime_val,
            "kind": kind,
            "label": label,
            "region": region,
        })),
    });
    task.updated_at = at;
    save_task(&project, &task)?;
    Ok(json!({
        "ok": true,
        "artifact_id": artifact_id,
        "path": rel_path,
        "kind": kind,
        "task": task_with_drift(&task, ctx)?,
    }))
}

/// Agent progress note (actor always kind=agent).
pub fn agent_note(args: &Value, ctx: &Ctx) -> Result<Value, OpError> {
    let project = resolve_project(args, ctx)?;
    let id = require_id(args)?;
    let body = args
        .get("body")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| OpError::InvalidArgument("AgentNote requires non-empty `body`".into()))?
        .to_string();
    let mut actor = Actor::agent();
    if let Some(name) = args
        .get("actor")
        .and_then(|a| a.get("name"))
        .and_then(|n| n.as_str())
    {
        actor.name = name.to_string();
    }
    actor.kind = "agent".into();

    let mut task = load_task(&project, id)?;
    let at = now_iso();
    let event = Event {
        id: new_ulid(),
        at: at.clone(),
        event_type: "agent_note".into(),
        actor,
        body: Some(body),
        meta: None,
    };
    let event_val =
        serde_json::to_value(&event).map_err(|e| OpError::Internal(format!("event json: {e}")))?;
    task.events.push(event);
    task.updated_at = at;
    save_task(&project, &task)?;
    Ok(json!({ "ok": true, "event": event_val, "task": task_with_drift(&task, ctx)? }))
}

/// Export thread as a decision record under `<project>/decisions/<component>.md`.
/// Never touches `.sail/tasks` gitignore policy — tasks stay local.
pub fn commit_intent(args: &Value, ctx: &Ctx) -> Result<Value, OpError> {
    let project = resolve_project(args, ctx)?;
    let id = require_id(args)?;
    let task = load_task(&project, id)?;
    let actor = Actor::from_value(args.get("actor"));

    let component_slug = task
        .anchor
        .as_ref()
        .map(|a| slugify(&a.element_path))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| slugify(&task.title));
    let decisions_dir = project.join("decisions");
    std::fs::create_dir_all(&decisions_dir)
        .map_err(|e| OpError::Internal(format!("mkdir decisions: {e}")))?;
    let out_path = decisions_dir.join(format!("{component_slug}.md"));
    // Confine write under project.
    let out_resolved = ensure_under_project(&project, &out_path)?;

    let body = render_decision_record(&task);
    // Append if file exists for same component.
    let mut final_body = body;
    if out_resolved.is_file() {
        let existing = std::fs::read_to_string(&out_resolved)
            .map_err(|e| OpError::Internal(format!("read decision: {e}")))?;
        final_body = format!("{existing}\n---\n\n{final_body}");
    }
    std::fs::write(&out_resolved, &final_body)
        .map_err(|e| OpError::Internal(format!("write decision: {e}")))?;

    // Record event on task (local only).
    let mut task = task;
    let at = now_iso();
    let rel = format!("decisions/{component_slug}.md");
    task.events.push(Event {
        id: new_ulid(),
        at: at.clone(),
        event_type: "commit_intent".into(),
        actor,
        body: None,
        meta: Some(json!({ "path": rel })),
    });
    task.updated_at = at;
    save_task(&project, &task)?;

    Ok(json!({
        "ok": true,
        "path": rel,
        "bytes": final_body.len(),
        "task": task_with_drift(&task, ctx)?,
    }))
}

fn slugify(s: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = false;
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash && !out.is_empty() {
            out.push('-');
            prev_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        "thread".into()
    } else {
        out.chars().take(64).collect()
    }
}

fn render_decision_record(task: &TaskDoc) -> String {
    let mut md = String::new();
    md.push_str(&format!("# Decision: {}\n\n", task.title));
    if let Some(a) = &task.anchor {
        md.push_str(&format!("**Component:** {}\n", a.element_path));
        md.push_str(&format!(
            "**Anchor:** `{}` · hash `{}`\n",
            a.node_id, a.content_hash
        ));
        if let Some(sp) = &a.span {
            md.push_str(&format!("**Span:** {}\n", sp.display()));
        }
    }
    md.push_str(&format!("**Status:** {}\n", task.status));
    md.push_str(&format!(
        "**Created by:** {} ({})\n\n",
        task.created_by.name, task.created_by.kind
    ));
    md.push_str("## Thread\n\n");
    for e in &task.events {
        match e.event_type.as_str() {
            "comment" | "agent_note" => {
                let who = if e.actor.kind == "agent" {
                    format!("agent · {}", e.actor.name)
                } else {
                    format!("you · {}", e.actor.name)
                };
                md.push_str(&format!("### {who} · {}\n\n", e.at));
                if let Some(b) = &e.body {
                    md.push_str(b);
                    md.push_str("\n\n");
                }
            }
            "artifact_added" => {
                if let Some(m) = &e.meta {
                    md.push_str(&format!(
                        "- artifact `{}` ({})\n",
                        m.get("path").and_then(|p| p.as_str()).unwrap_or("?"),
                        m.get("kind").and_then(|k| k.as_str()).unwrap_or("file")
                    ));
                }
            }
            _ => {}
        }
    }
    if !task.artifacts.is_empty() {
        md.push_str("\n## Artifacts\n\n");
        for a in &task.artifacts {
            md.push_str(&format!(
                "- `{}` · {} · {}\n",
                a.path,
                a.kind,
                a.label.as_deref().unwrap_or("—")
            ));
        }
        md.push('\n');
    }
    md.push_str(&format!(
        "\n<!-- sail:task-id={} · exported from .sail/tasks (gitignored) -->\n",
        task.id
    ));
    md
}

fn require_id(args: &Value) -> Result<&str, OpError> {
    let id = args
        .get("id")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| OpError::InvalidArgument("requires non-empty `id`".into()))?;
    if !is_valid_task_id(id) {
        return Err(OpError::InvalidArgument(format!("invalid task id: {id}")));
    }
    Ok(id)
}

fn guess_mime(path: &str) -> String {
    let lower = path.to_ascii_lowercase();
    if lower.ends_with(".png") {
        "image/png".into()
    } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        "image/jpeg".into()
    } else if lower.ends_with(".webp") {
        "image/webp".into()
    } else if lower.ends_with(".gif") {
        "image/gif".into()
    } else if lower.ends_with(".json") {
        "application/json".into()
    } else if lower.ends_with(".txt") || lower.ends_with(".log") {
        "text/plain".into()
    } else {
        "application/octet-stream".into()
    }
}

/// Strip data-url prefix / whitespace for length estimates and decode.
fn normalize_base64_input(input: &str) -> String {
    let s = input.trim();
    let s = if let Some(idx) = s.find("base64,") {
        &s[idx + 7..]
    } else {
        s
    };
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

/// Early reject when base64 payload cannot fit under [`MAX_ARTIFACT_BYTES`].
fn reject_oversized_base64(input: &str) -> Result<(), OpError> {
    let s = normalize_base64_input(input);
    let max_chars = MAX_ARTIFACT_BYTES
        .saturating_mul(4)
        .div_ceil(3)
        .saturating_add(4);
    if s.len() > max_chars {
        return Err(OpError::InvalidArgument(format!(
            "artifact exceeds {MAX_ARTIFACT_BYTES} bytes"
        )));
    }
    Ok(())
}

fn decode_base64(input: &str) -> Result<Vec<u8>, OpError> {
    let s = normalize_base64_input(input);
    base64_decode(&s).map_err(|e| OpError::InvalidArgument(format!("invalid base64: {e}")))
}

fn b64_val(b: u8) -> Option<u8> {
    match b {
        b'A'..=b'Z' => Some(b - b'A'),
        b'a'..=b'z' => Some(b - b'a' + 26),
        b'0'..=b'9' => Some(b - b'0' + 52),
        b'+' | b'-' => Some(62),
        b'/' | b'_' => Some(63),
        _ => None,
    }
}

/// Minimal standard base64 decoder (no external dep).
fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len() * 3 / 4 + 2);
    let mut buf = [0u8; 4];
    let mut buf_len = 0;
    let mut padding = 0;

    for &b in bytes {
        if b == b'=' {
            padding += 1;
            buf[buf_len] = 0;
            buf_len += 1;
            if buf_len == 4 {
                flush_b64(&mut out, &buf, padding)?;
                buf_len = 0;
                padding = 0;
            }
            continue;
        }
        let Some(v) = b64_val(b) else {
            return Err(format!("invalid base64 char {:?}", b as char));
        };
        buf[buf_len] = v;
        buf_len += 1;
        if buf_len == 4 {
            flush_b64(&mut out, &buf, 0)?;
            buf_len = 0;
        }
    }
    if buf_len > 0 {
        return Err("truncated base64".into());
    }
    Ok(out)
}

fn flush_b64(out: &mut Vec<u8>, buf: &[u8; 4], padding: u8) -> Result<(), String> {
    let n = ((buf[0] as u32) << 18)
        | ((buf[1] as u32) << 12)
        | ((buf[2] as u32) << 6)
        | (buf[3] as u32);
    out.push(((n >> 16) & 0xff) as u8);
    if padding < 2 {
        out.push(((n >> 8) & 0xff) as u8);
    }
    if padding < 1 {
        out.push((n & 0xff) as u8);
    }
    if padding > 2 {
        return Err("too much padding".into());
    }
    Ok(())
}

/// Inject default agent actor for MCP when caller omitted `actor`.
pub fn inject_mcp_actor_default(args: &Value) -> Value {
    if args.get("actor").is_some() {
        return args.clone();
    }
    let mut obj = args.as_object().cloned().unwrap_or_default();
    obj.insert(
        "actor".into(),
        json!({
            "kind": "agent",
            "name": std::env::var("SAIL_AGENT_NAME").unwrap_or_else(|_| "agent".into()),
        }),
    );
    Value::Object(obj)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn demo_ctx(ws: &Path) -> Ctx {
        let project = ws.join("apps/demo");
        std::fs::create_dir_all(&project).unwrap();
        Ctx::new(ws, &project, ws)
    }

    fn node(id: &str, file: &str, tag: &str, name: &str, start: u64, end: u64) -> Value {
        let mut attrs = serde_json::Map::new();
        if !name.is_empty() {
            attrs.insert("name".into(), json!(name));
        }
        json!({
            "id": id,
            "file": file,
            "tag": tag,
            "attrs": attrs,
            "span": { "start": start, "end": end, "line": start, "col": 1, "end_line": end, "end_col": 1 },
            "layer": "l1-element",
            "children": [],
        })
    }

    #[test]
    fn task_id_validation() {
        assert!(is_valid_task_id("01ARZ3NDEKTSV4RRFFQ69G5FAV"));
        assert!(is_valid_task_id("01arz3ndektsv4rrffq69g5fav"));
        assert!(is_valid_task_id("abc"));
        assert!(is_valid_task_id("a_b-c1"));
        assert!(!is_valid_task_id(""));
        assert!(!is_valid_task_id(".."));
        assert!(!is_valid_task_id("a/b"));
        assert!(!is_valid_task_id("/abs"));
        assert!(!is_valid_task_id("AB"));
        assert!(!is_valid_task_id("01ARZ3NDEKTSV4RRFFQ69G5FAI"));
        assert!(!is_valid_task_id("01ARZ3NDEKTSV4RRFFQ69G5FAL"));
        assert!(!is_valid_task_id("01ARZ3NDEKTSV4RRFFQ69G5FAO"));
        assert!(!is_valid_task_id("01ARZ3NDEKTSV4RRFFQ69G5FAU"));
    }

    #[test]
    fn filename_sanitize() {
        assert!(sanitize_artifact_filename("shot.png").is_ok());
        assert!(sanitize_artifact_filename("../x.png").is_err());
        assert!(sanitize_artifact_filename("/etc/passwd").is_err());
        assert!(sanitize_artifact_filename("a/b.png").is_err());
    }

    #[test]
    fn drift_ok_moved_lost() {
        let file = "routes/page.sail";
        let n1 = node(
            "routes/page.sail#0.1#abcd1234",
            file,
            "card",
            "hero",
            42,
            58,
        );
        let anchor = Anchor {
            node_id: "routes/page.sail#0.1#abcd1234".into(),
            element_path: element_path_from_node(&n1),
            content_hash: "abcd1234".into(),
            file: Some(file.into()),
            span: AnchorSpan::from_model_span(&n1["span"]),
        };
        // same id → ok
        let r = resolve_anchor_drift(&anchor, &[n1.clone()]);
        assert_eq!(r.status, DriftStatus::Ok);
        assert_eq!(r.span.as_ref().unwrap().start, 42);

        // moved: same element path, new id + span
        let n2 = node(
            "routes/page.sail#0.2#abcd1234",
            file,
            "card",
            "hero",
            100,
            120,
        );
        let r2 = resolve_anchor_drift(&anchor, &[n2.clone()]);
        assert_eq!(r2.status, DriftStatus::Moved);
        assert_eq!(r2.span.as_ref().unwrap().start, 100);
        assert_eq!(
            r2.live_node_id.as_deref(),
            Some("routes/page.sail#0.2#abcd1234")
        );

        // lost: empty model — never rebinds
        let r3 = resolve_anchor_drift(&anchor, &[]);
        assert_eq!(r3.status, DriftStatus::Lost);
        assert!(r3.live_node_id.is_none());

        // lost: different element present — must NOT rebind
        let other = node(
            "routes/page.sail#0.9#zzzz9999",
            file,
            "avatar",
            "fallback",
            1,
            2,
        );
        let r4 = resolve_anchor_drift(&anchor, &[other]);
        assert_eq!(r4.status, DriftStatus::Lost);
    }

    #[test]
    fn create_list_update_comment_roundtrip() {
        let dir = tempdir().unwrap();
        let ws = dir.path();
        let ctx = demo_ctx(ws);
        let project = "apps/demo";

        let created = create(
            &json!({ "project": project, "title": "Fix focus ring", "labels": ["a11y"] }),
            &ctx,
        )
        .unwrap();
        assert_eq!(created["ok"], true);
        assert!(
            created.get("warnings").is_none(),
            "ensure_project_store should succeed in tempdir: {:?}",
            created.get("warnings")
        );
        let id = created["task"]["id"].as_str().unwrap().to_string();
        assert!(is_valid_task_id(&id));
        let disk = ws.join("apps/demo/.sail/tasks").join(&id).join("task.json");
        assert!(disk.is_file(), "task.json must exist");
        let gi = std::fs::read_to_string(ws.join("apps/demo/.gitignore")).unwrap();
        assert!(
            gi.lines().any(|l| {
                let t = l.trim();
                t == ".sail/" || t == "/.sail/"
            }),
            ".gitignore must contain .sail/ after Create:\n{gi}"
        );

        let listed = list(&json!({ "project": project }), &ctx).unwrap();
        assert_eq!(listed["tasks"].as_array().unwrap().len(), 1);
        assert_eq!(listed["tasks"][0]["title"], "Fix focus ring");

        let got = get(&json!({ "project": project, "id": id }), &ctx).unwrap();
        assert_eq!(got["task"]["status"], "todo");

        let updated = update(
            &json!({
                "project": project,
                "id": id,
                "status": "in_progress",
                "assignee": { "kind": "agent", "name": "claude" },
                "route": "/components"
            }),
            &ctx,
        )
        .unwrap();
        assert_eq!(updated["task"]["status"], "in_progress");
        let events = updated["task"]["events"].as_array().unwrap();
        assert!(
            events
                .iter()
                .any(|e| e["type"] == "status" && e["meta"]["to"] == "in_progress")
        );
        let after_status = updated["task"]["updated_at"].as_str().unwrap().to_string();

        let noop = update(
            &json!({
                "project": project,
                "id": id,
                "labels": ["a11y"],
                "route": "/components"
            }),
            &ctx,
        )
        .unwrap();
        assert_eq!(
            noop["task"]["updated_at"].as_str().unwrap(),
            after_status,
            "no-op labels/route must not rewrite updated_at"
        );

        let c = comment(
            &json!({ "project": project, "id": id, "body": "looking into it" }),
            &ctx,
        )
        .unwrap();
        assert_eq!(c["event"]["type"], "comment");
        assert_eq!(c["event"]["body"], "looking into it");
        assert_eq!(c["event"]["actor"]["kind"], "human");
    }

    #[test]
    fn path_escape_and_unknown_id() {
        let dir = tempdir().unwrap();
        let ws = dir.path();
        let ctx = demo_ctx(ws);

        let outside = tempdir().unwrap();
        let denied = list(
            &json!({ "project": outside.path().to_string_lossy() }),
            &ctx,
        );
        assert!(matches!(denied, Err(OpError::PermissionDenied(_))));

        let bad_dotdot = create(
            &json!({ "project": "apps/demo/../../etc", "title": "x" }),
            &ctx,
        );
        assert!(matches!(bad_dotdot, Err(OpError::PermissionDenied(_))));

        let missing = get(
            &json!({
                "project": "apps/demo",
                "id": "01ARZ3NDEKTSV4RRFFQ69G5FAV"
            }),
            &ctx,
        );
        assert!(matches!(missing, Err(OpError::NotFound(_))));

        let bad_id = get(&json!({ "project": "apps/demo", "id": "../evil" }), &ctx);
        assert!(matches!(bad_id, Err(OpError::InvalidArgument(_))));
    }

    #[test]
    fn thread_anchor_artifact_agent_and_commit_intent() {
        let dir = tempdir().unwrap();
        let ws = dir.path();
        let ctx = demo_ctx(ws);
        let project = "apps/demo";

        let created = thread_create(
            &json!({
                "project": project,
                "title": "hover lift too strong",
                "body": "The lift feels aggressive on dense screens.",
                "anchor": {
                    "nodeId": "routes/page.sail#0.1#9f2c",
                    "elementPath": "page.sail › card hero",
                    "contentHash": "9f2c",
                    "file": "routes/page.sail",
                    "span": { "start": 42, "end": 58, "line": 42, "end_line": 58 }
                },
                "actor": { "kind": "human", "name": "user" }
            }),
            &ctx,
        )
        .unwrap();
        let id = created["task"]["id"].as_str().unwrap().to_string();
        assert_eq!(
            created["task"]["anchor"]["nodeId"],
            "routes/page.sail#0.1#9f2c"
        );
        assert!(
            created["task"]["events"]
                .as_array()
                .unwrap()
                .iter()
                .any(|e| e["type"] == "comment")
        );

        // render artifact with region metadata
        let b64 = "aGVsbG8="; // "hello"
        let art = add_artifact(
            &json!({
                "project": project,
                "id": id,
                "filename": "region.png",
                "content_base64": b64,
                "mime": "image/png",
                "kind": "render",
                "label": "region · card@42",
                "region": { "nodeId": "routes/page.sail#0.1#9f2c", "label": "card@42" },
                "actor": { "kind": "human", "name": "user" }
            }),
            &ctx,
        )
        .unwrap();
        assert_eq!(art["ok"], true);
        assert_eq!(art["kind"], "render");
        assert_eq!(art["task"]["artifacts"].as_array().unwrap().len(), 1);
        assert_eq!(art["task"]["artifacts"][0]["kind"], "render");

        // second + third for batch proof
        for name in ["A.png", "B.png"] {
            add_artifact(
                &json!({
                    "project": project,
                    "id": id,
                    "filename": name,
                    "content_base64": b64,
                    "kind": "render",
                    "label": name,
                    "actor": { "kind": "agent", "name": "agent" }
                }),
                &ctx,
            )
            .unwrap();
        }
        let got = get(&json!({ "project": project, "id": id }), &ctx).unwrap();
        assert_eq!(got["task"]["artifacts"].as_array().unwrap().len(), 3);

        let note = agent_note(
            &json!({ "project": project, "id": id, "body": "Proposed motion.preset = gentle" }),
            &ctx,
        )
        .unwrap();
        assert_eq!(note["event"]["type"], "agent_note");
        assert_eq!(note["event"]["actor"]["kind"], "agent");

        let reply = thread_reply(
            &json!({
                "project": project,
                "id": id,
                "body": "@agent try gentle",
                "actor": { "kind": "human", "name": "user" }
            }),
            &ctx,
        )
        .unwrap();
        assert_eq!(reply["event"]["type"], "comment");

        // actor filter
        let agents = thread_list(&json!({ "project": project, "actor": "agent" }), &ctx).unwrap();
        // last meaningful actor may be human from reply — filter is last comment/note
        // After human reply, actor=human should match
        let humans = thread_list(&json!({ "project": project, "actor": "human" }), &ctx).unwrap();
        assert_eq!(humans["tasks"].as_array().unwrap().len(), 1);
        let _ = agents;

        // commit intent → decisions/ under project; .sail still gitignored
        let committed = commit_intent(
            &json!({
                "project": project,
                "id": id,
                "actor": { "kind": "human", "name": "user" }
            }),
            &ctx,
        )
        .unwrap();
        assert_eq!(committed["ok"], true);
        let rel = committed["path"].as_str().unwrap();
        assert!(rel.starts_with("decisions/"));
        assert!(rel.ends_with(".md"));
        let abs = ws.join("apps/demo").join(rel);
        assert!(abs.is_file());
        let md = std::fs::read_to_string(&abs).unwrap();
        assert!(md.contains("hover lift too strong"));
        assert!(md.contains("sail:task-id="));
        // tasks still under .sail
        assert!(ws.join("apps/demo/.sail/tasks").join(&id).is_dir());
        let gi = std::fs::read_to_string(ws.join("apps/demo/.gitignore")).unwrap();
        assert!(gi.contains(".sail/"));

        // path escape in filename
        assert!(
            add_artifact(
                &json!({
                    "project": project,
                    "id": id,
                    "filename": "../x.txt",
                    "content_base64": b64
                }),
                &ctx,
            )
            .is_err()
        );

        let max_chars = MAX_ARTIFACT_BYTES
            .saturating_mul(4)
            .div_ceil(3)
            .saturating_add(4);
        let too_long = "A".repeat(max_chars + 1);
        let early = add_artifact(
            &json!({
                "project": project,
                "id": id,
                "filename": "huge.bin",
                "content_base64": too_long
            }),
            &ctx,
        );
        assert!(
            matches!(early, Err(OpError::InvalidArgument(ref m)) if m.contains("exceeds")),
            "expected size reject, got {early:?}"
        );
    }

    #[test]
    fn two_model_revisions_moved_and_lost() {
        // Pure drift across two synthetic model revisions (ops test contract).
        let file = "routes/page.sail";
        let rev1 = node("routes/page.sail#0.1#hash1", file, "card", "hero", 42, 58);
        let anchor = Anchor {
            node_id: "routes/page.sail#0.1#hash1".into(),
            element_path: element_path_from_node(&rev1),
            content_hash: "hash1".into(),
            file: Some(file.into()),
            span: AnchorSpan::from_model_span(&rev1["span"]),
        };
        assert_eq!(
            resolve_anchor_drift(&anchor, &[rev1]).status,
            DriftStatus::Ok
        );

        // Edit moves element (new ordinal + span); path label preserved → moved
        let rev2 = node("routes/page.sail#0.3#hash1", file, "card", "hero", 200, 240);
        let moved = resolve_anchor_drift(&anchor, &[rev2]);
        assert_eq!(moved.status, DriftStatus::Moved);
        assert_eq!(moved.span.as_ref().unwrap().start, 200);

        // Element deleted → lost, never rebound to sibling
        let sibling = node("routes/page.sail#0.0#other", file, "card", "sidebar", 1, 10);
        let lost = resolve_anchor_drift(&anchor, &[sibling]);
        assert_eq!(lost.status, DriftStatus::Lost);
        assert!(lost.live_node_id.is_none());
    }

    #[test]
    fn empty_title_rejected() {
        let dir = tempdir().unwrap();
        let ctx = demo_ctx(dir.path());
        let err = create(&json!({ "project": "apps/demo", "title": "  " }), &ctx);
        assert!(matches!(err, Err(OpError::InvalidArgument(_))));
        let err2 = thread_create(&json!({ "project": "apps/demo", "title": "x" }), &ctx);
        assert!(matches!(err2, Err(OpError::InvalidArgument(_))));
    }

    #[test]
    fn base64_roundtrip() {
        let data = b"hello world!!";
        let encoded = "aGVsbG8gd29ybGQhIQ==";
        assert_eq!(decode_base64(encoded).unwrap(), data);
    }

    #[test]
    fn reject_oversized_base64_before_decode() {
        let max_chars = MAX_ARTIFACT_BYTES
            .saturating_mul(4)
            .div_ceil(3)
            .saturating_add(4);
        let too_long = "A".repeat(max_chars + 1);
        assert!(matches!(
            reject_oversized_base64(&too_long),
            Err(OpError::InvalidArgument(_))
        ));
        assert!(reject_oversized_base64("aGVsbG8=").is_ok());
    }
}
