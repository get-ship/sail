//! Typed request/response contracts for every sail-ops method.
//!
//! Rust types here are the only hand-written schema source (T-I2 / T-I4).
//! MCP descriptors, TS client, and `/sail.ops.v1.schema` are generated from
//! [`METHOD_CONTRACTS`].

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::typed::{EmptyArgs, Json, MethodContract, object_schema_for, schema_value_for};

// ── SailAuthoring ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SurfaceArgs {
    /// core | layout | style | tokens | l1 | control-flow
    pub topic: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SurfaceRet {
    pub topic: String,
    pub markdown: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CheckArgs {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Auto-fix: `off` | `suggest` | `apply`. Default `off` (unattended).
    /// `suggest` attaches machine fix payloads without writing; `apply` rewrites
    /// the file when `path` is set (or returns fixed `source` when inline).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fix: Option<String>,
    /// Attribution when fix=apply (e.g. `agent`, `human:name`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor: Option<String>,
    /// When true (default), merge mapped tsgo TypeScript diagnostics into
    /// structure/check results (SPEC-LSP-1 studio gutters). Soft-noops when
    /// tsgo is absent. Set `false` for structure-only / faster agent polls.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(rename = "includeTs")]
    pub include_ts: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CheckRet {
    pub ok: bool,
    pub diagnostics: Vec<Json>,
    pub count: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fixes: Option<Vec<Json>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(rename = "autoFixed")]
    pub auto_fixed: Option<Vec<Json>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EditArgs {
    /// Workspace-relative `.sail` path. Optional when `identity.file` is set
    /// (WI7 identity-scoped target).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Atomic batch of structured ops (`insert`/`replace`/`wrap`/`setAttr`/
    /// `removeAttr`/`setSx`/`move`).
    pub ops: Vec<Json>,
    /// Optional inline source (else file is read). Dry-run without write when
    /// combined with dryRun.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Auto-fix mode: `off` | `suggest` | `apply` (default `off`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub autofix: Option<String>,
    /// Attribution for applied auto-fixes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor: Option<String>,
    /// When true, validate and return result without writing disk.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(rename = "dryRun")]
    pub dry_run: Option<bool>,
    /// WI7 element identity target. Scopes the edit to the resolved component
    /// (path from `identity.file`; ops nodeIds rewritten when the element moved).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identity: Option<Json>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EditRet {
    pub applied: bool,
    #[serde(rename = "modelRevision")]
    pub model_revision: String,
    pub diagnostics: Vec<Json>,
    #[serde(rename = "autoFixed")]
    pub auto_fixed: Vec<Json>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Resolved live identity when an identity-scoped edit was requested.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identity: Option<Json>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LayoutCheckArgs {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LayoutCheckRet {
    #[serde(flatten)]
    pub body: Json,
}

/// List workspace-relative project source paths under a project (explorer seed).
/// Includes sail/ts/json/css/… with node_modules/.git/target/dist ignored.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ListFilesArgs {
    /// Workspace-relative project root (e.g. `.` for an external app, or
    /// `apps/<name>` in a monorepo). Default: ops project.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ListFilesRet {
    pub ok: bool,
    pub project: String,
    /// Workspace-relative paths (`routes/page.sail`, `package.json`, …).
    pub files: Vec<String>,
}

/// Hover at a 1-based position in a `.sail` source (TS regions via virtual_ts+tsgo).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HoverArgs {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// 1-based line (matches Check diagnostics / compiler hover).
    pub line: u32,
    /// 1-based column.
    pub column: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HoverRet {
    /// Markdown hover contents. Empty/absent when nothing to show.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contents: Option<String>,
    /// Optional range in 1-based line/col.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range: Option<Json>,
}

/// Completions at a 1-based position (tsgo in TS regions; structure fallback).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CompleteArgs {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CompleteRet {
    pub items: Vec<Json>,
}

/// Compiler policy registry projection (SPEC-POLICY-1).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PolicyArgs {
    /// `list` (registry rows) or `get` (registry + resolved values).
    pub mode: String,
    /// Optional project directory for `get` (workspace-relative or absolute).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PolicyRet {
    /// Shared serialization payload (`mode`, `entries`, optional `values`).
    #[serde(flatten)]
    pub body: Json,
}

// ── SailInspect ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InspectModelArgs {
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub viewport: Option<InspectViewport>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InspectViewport {
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InspectModelRet {
    #[serde(rename = "modelVersion")]
    pub model_version: u32,
    pub file: String,
    /// Author `.sail` source (same bytes Model inspected). Studio Source view.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    pub tree: Json,
    pub layout: Json,
    pub state: Json,
    pub diagnostics: Vec<Json>,
    pub data: Json,
    pub theme: Json,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InspectTreeArgs {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InspectTreeRet {
    #[serde(rename = "modelVersion")]
    pub model_version: u32,
    pub file: String,
    pub roots: Vec<String>,
    pub nodes: Vec<Json>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InspectStateArgs {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InspectStateRet {
    #[serde(rename = "modelVersion")]
    pub model_version: u32,
    pub file: String,
    pub bindings: Vec<Json>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InspectTokenUsageArgs {
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InspectTokenUsageRet {
    pub token: String,
    pub components: usize,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "componentIds"
    )]
    pub component_ids: Option<Vec<String>>,
    pub sites: Vec<Json>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ThemeGraphArgs {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "themeId",
        alias = "theme_id"
    )]
    pub theme_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ThemeGraphRet {
    #[serde(flatten)]
    pub body: Json,
}

// ── SailTheme ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ThemeListArgs {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ThemeListRet {
    pub ok: bool,
    pub path: String,
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ThemeReadArgs {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ThemeReadRet {
    pub ok: bool,
    pub path: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ThemeWriteArgs {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ThemeWriteRet {
    pub ok: bool,
    pub path: String,
    pub bytes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ThemeDeleteArgs {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ThemeDeleteRet {
    pub ok: bool,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ThemeForkArgs {
    #[serde(alias = "source")]
    pub source_path: String,
    #[serde(alias = "dest")]
    pub dest_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ThemeForkRet {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ok: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dest_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ThemeGenerateArgs {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub out: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ThemeGenerateRet {
    #[serde(flatten)]
    pub body: Json,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ThemeEffectiveIdsArgs {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ThemeEffectiveIdsRet {
    #[serde(flatten)]
    pub body: Json,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ThemeFontSearchArgs {
    #[serde(default)]
    pub query: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ThemeFontSearchRet {
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    #[serde(default)]
    pub families: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub count: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ThemeFontAddArgs {
    pub family: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub axes: Option<String>,
    /// App root (project-relative; external app or monorepo `apps/<name>`).
    /// Defaults to ctx project / first apps/* with routes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ThemeFontAddRet {
    #[serde(flatten)]
    pub body: Json,
}

// ── SailDevServer ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DevServerStartArgs {
    pub project: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub addr: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DevServerStartRet {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ok: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub addr: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DevServerStatusRet {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ok: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub running: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub addr: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DevServerStopRet {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ok: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stopped: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub running: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
}

// ── SailTask ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskListArgs {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Filter by last comment actor: `human` | `agent` | `all`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor: Option<String>,
    /// Filter by drift: `ok` | `moved` | `lost` | `all`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drift: Option<String>,
    /// When true, only open statuses (backlog|todo|in_progress|review).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskListRet {
    pub ok: bool,
    pub tasks: Vec<Json>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskGetArgs {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskGetRet {
    pub ok: bool,
    pub task: Json,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskActor {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskCreateArgs {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route: Option<String>,
    /// Optional initial comment body.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    /// WI7 identity / A-anchor `{ nodeId, elementPath?, contentHash?, file?, span?, locator? }`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anchor: Option<Json>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor: Option<TaskActor>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskCreateRet {
    #[serde(flatten)]
    pub body: Json,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskUpdateArgs {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assignee: Option<TaskActor>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor: Option<TaskActor>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskUpdateRet {
    #[serde(flatten)]
    pub body: Json,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskCommentArgs {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    pub id: String,
    pub body: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor: Option<TaskActor>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskCommentRet {
    #[serde(flatten)]
    pub body: Json,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskAddArtifactArgs {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_base64: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mime: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// `file` (default) | `render` (region/viewport crop).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Region metadata for `kind: "render"` (nodeId, rect, …).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: Option<Json>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor: Option<TaskActor>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskThreadCreateArgs {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    /// Required WI7 identity / A-anchor `{ nodeId, elementPath?, contentHash?, file?, span?, locator? }`.
    pub anchor: Json,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor: Option<TaskActor>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskThreadCreateRet {
    #[serde(flatten)]
    pub body: Json,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskThreadReplyArgs {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    pub id: String,
    pub body: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor: Option<TaskActor>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskThreadReplyRet {
    #[serde(flatten)]
    pub body: Json,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskThreadListArgs {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drift: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskThreadListRet {
    pub ok: bool,
    pub tasks: Vec<Json>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskReanchorArgs {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    pub id: String,
    pub anchor: Json,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor: Option<TaskActor>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskReanchorRet {
    #[serde(flatten)]
    pub body: Json,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskCommitIntentArgs {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor: Option<TaskActor>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskCommitIntentRet {
    #[serde(flatten)]
    pub body: Json,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskAddArtifactRet {
    #[serde(flatten)]
    pub body: Json,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskAgentNoteArgs {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    pub id: String,
    pub body: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor: Option<TaskActor>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskAgentNoteRet {
    #[serde(flatten)]
    pub body: Json,
}

// ── SailRender ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RenderScreenshotArgs {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// `WxH` string or `{width,height}` object.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub viewport: Option<Value>,
    /// Theme id (e.g. `ship`). Mode may be folded as `ship-dark`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    /// `light` | `dark` (data-mode).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RenderScreenshotRet {
    #[serde(rename = "imagePng")]
    pub image_png: String,
    pub regions: Vec<Json>,
    pub dynamics: Vec<Json>,
    pub viewport: Json,
    pub theme: String,
    pub ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
}

// ── Contract table ──────────────────────────────────────────────────────────

macro_rules! contract {
    (
        $service:literal, $method:literal, $mcp:literal, $desc:literal,
        $args:ty, $ret:ty,
        $ts_args:literal, $ts_ret:literal,
        $ts_args_body:expr, $ts_ret_body:expr
    ) => {
        MethodContract {
            service: $service,
            method: $method,
            mcp_tool: $mcp,
            description: $desc,
            input_schema: || object_schema_for::<$args>(),
            output_schema: || schema_value_for::<$ret>(),
            ts_args: $ts_args,
            ts_ret: $ts_ret,
            ts_args_body: $ts_args_body,
            ts_ret_body: $ts_ret_body,
        }
    };
}

/// Full method table — SSOT for codegen + MCP + CLI help.
pub const METHOD_CONTRACTS: &[MethodContract] = &[
    contract!(
        "SailAuthoring",
        "Surface",
        "sail_surface",
        "Progressive disclosure of the Sail authoring surface from compiler SSOT. Pass topic: core | layout | style | tokens | l1 | control-flow.",
        SurfaceArgs,
        SurfaceRet,
        "SurfaceArgs",
        "SurfaceRet",
        "  topic: string\n",
        "  topic: string\n  markdown: string\n"
    ),
    contract!(
        "SailAuthoring",
        "Check",
        "sail_check",
        "Run Sail lint/compile diagnostics for a file path or inline source. Returns structured JSON diagnostics (severity, rule, message, line/col, optional rewrite FixPlan). Optional `fix`: off|suggest|apply for confluent allow-listed rewrites (sail-compose-rhythm, sail-compose-contrast, sail-compose-spacing, sail-no-raw-container, sail-sx typo, participation-style, display-on-layout, prefer-primitive). Prefer `source` for snippets; `path` reads any readable filesystem path (local agent trust model).",
        CheckArgs,
        CheckRet,
        "CheckArgs",
        "CheckRet",
        "  path?: string\n  source?: string\n  fix?: string\n  actor?: string\n  includeTs?: boolean\n",
        "  ok: boolean\n  diagnostics: JsonValue[]\n  count: number\n  fixes?: JsonValue[]\n  source?: string\n  autoFixed?: JsonValue[]\n"
    ),
    contract!(
        "SailAuthoring",
        "Hover",
        "sail_hover",
        "Typed hover at a 1-based .sail position. Uses sail-lsp virtual_ts + tsgo for controller/scope/template TS regions; structure symbols otherwise. Soft-degrades when tsgo is absent.",
        HoverArgs,
        HoverRet,
        "HoverArgs",
        "HoverRet",
        "  path?: string\n  source?: string\n  line: number\n  column: number\n",
        "  contents?: string\n  range?: JsonValue\n"
    ),
    contract!(
        "SailAuthoring",
        "Complete",
        "sail_complete",
        "Completions at a 1-based .sail position. Proxies tsgo inside TS regions; falls back to structure scope symbols. Soft-degrades when tsgo is absent.",
        CompleteArgs,
        CompleteRet,
        "CompleteArgs",
        "CompleteRet",
        "  path?: string\n  source?: string\n  line: number\n  column: number\n",
        "  items: JsonValue[]\n"
    ),
    contract!(
        "SailAuthoring",
        "Edit",
        "sail_edit",
        "Structured grammar-aware Sail edits (insert/replace/wrap/setAttr/removeAttr/setSx/move). Atomic batch; validates before write so the file never holds an uncompilable intermediate. Optional WI7 `identity` scopes the edit to a component (path from identity.file; nodeIds rewritten on drift). Response: applied, modelRevision, diagnostics, autoFixed[].",
        EditArgs,
        EditRet,
        "EditArgs",
        "EditRet",
        "  path?: string\n  ops: JsonValue[]\n  source?: string\n  autofix?: string\n  actor?: string\n  dryRun?: boolean\n  identity?: JsonValue\n",
        "  applied: boolean\n  modelRevision: string\n  diagnostics: JsonValue[]\n  autoFixed: JsonValue[]\n  source?: string\n  path?: string\n  message?: string\n  identity?: JsonValue\n"
    ),
    contract!(
        "SailAuthoring",
        "LayoutCheck",
        "sail_layout_check",
        "Taffy layout feedback for agents: multi-viewport diagnostics + computed box tree (x/y/w/h, collapsed flags). Run after layout edits. Prefer `source` for snippets.",
        LayoutCheckArgs,
        LayoutCheckRet,
        "LayoutCheckArgs",
        "LayoutCheckRet",
        "  path?: string\n  source?: string\n",
        "  [key: string]: JsonValue | undefined\n"
    ),
    contract!(
        "SailAuthoring",
        "ListFiles",
        "sail_list_files",
        "List workspace-relative .sail paths under a project (routes/src/packs). Studio explorer seed.",
        ListFilesArgs,
        ListFilesRet,
        "ListFilesArgs",
        "ListFilesRet",
        "  project?: string\n",
        "  ok: boolean\n  project: string\n  files: string[]\n"
    ),
    contract!(
        "SailAuthoring",
        "Policy",
        "sail_policy",
        "Compiler policy registry (SPEC-POLICY-1). mode=list returns types/defaults/descriptions; mode=get returns the same rows plus resolved values. Same payload as `sail-compiler policy --json` and GET /_deck/policy.",
        PolicyArgs,
        PolicyRet,
        "PolicyArgs",
        "PolicyRet",
        "  mode: string\n  project?: string\n",
        "  [key: string]: JsonValue | undefined\n"
    ),
    contract!(
        "SailInspect",
        "Model",
        "sail_inspect_model",
        "Canonical per-file Sail model (tree, layout, state, diagnostics, data, theme). Source-derived; modelVersion 1.",
        InspectModelArgs,
        InspectModelRet,
        "InspectModelArgs",
        "InspectModelRet",
        "  path: string\n  viewport?: { width: number; height: number }\n",
        "  modelVersion: number\n  file: string\n  tree: JsonValue\n  layout: JsonValue\n  state: JsonValue\n  diagnostics: JsonValue[]\n  data: JsonValue\n  theme: JsonValue\n"
    ),
    contract!(
        "SailInspect",
        "Tree",
        "sail_inspect_tree",
        "IR element tree with span-keyed node ids for a .sail file.",
        InspectTreeArgs,
        InspectTreeRet,
        "InspectTreeArgs",
        "InspectTreeRet",
        "  path: string\n",
        "  modelVersion: number\n  file: string\n  roots: string[]\n  nodes: JsonValue[]\n"
    ),
    contract!(
        "SailInspect",
        "State",
        "sail_inspect_state",
        "Scope bindings with template readers/writers and SSR tier badges (S/I/C).",
        InspectStateArgs,
        InspectStateRet,
        "InspectStateArgs",
        "InspectStateRet",
        "  path: string\n",
        "  modelVersion: number\n  file: string\n  bindings: JsonValue[]\n"
    ),
    contract!(
        "SailInspect",
        "TokenUsage",
        "sail_inspect_token_usage",
        "Blast radius for a design token: component count + source sites (file + span).",
        InspectTokenUsageArgs,
        InspectTokenUsageRet,
        "InspectTokenUsageArgs",
        "InspectTokenUsageRet",
        "  token: string\n",
        "  token: string\n  components: number\n  componentIds?: string[]\n  sites: JsonValue[]\n"
    ),
    // componentIds wire name matches modelVersion camelCase style (see InspectTokenUsageRet).
    contract!(
        "SailTheme",
        "Graph",
        "sail_theme_graph",
        "DTCG theme graph: tiers, {alias} edges, values from theme JSON only (never CSS).",
        ThemeGraphArgs,
        ThemeGraphRet,
        "ThemeGraphArgs",
        "ThemeGraphRet",
        "  themeId?: string\n",
        "  [key: string]: JsonValue | undefined\n"
    ),
    contract!(
        "SailTheme",
        "List",
        "sail_theme_list",
        "List theme JSON files under a directory. Path must be themes/**, tokens/themes/** (legacy), or packages/sail/tokens/model/themes.",
        ThemeListArgs,
        ThemeListRet,
        "ThemeListArgs",
        "ThemeListRet",
        "  path: string\n",
        "  ok: boolean\n  path: string\n  files: string[]\n"
    ),
    contract!(
        "SailTheme",
        "Read",
        "sail_theme_read",
        "Read a project or catalog theme JSON file (workspace-relative path).",
        ThemeReadArgs,
        ThemeReadRet,
        "ThemeReadArgs",
        "ThemeReadRet",
        "  path: string\n",
        "  ok: boolean\n  path: string\n  content: string\n"
    ),
    contract!(
        "SailTheme",
        "Write",
        "sail_theme_write",
        "Write a project theme JSON. Allowlist: themes/**/*.json or legacy tokens/themes/**/*.json.",
        ThemeWriteArgs,
        ThemeWriteRet,
        "ThemeWriteArgs",
        "ThemeWriteRet",
        "  path: string\n  content: string\n",
        "  ok: boolean\n  path: string\n  bytes: number\n"
    ),
    contract!(
        "SailTheme",
        "Delete",
        "sail_theme_delete",
        "Delete a project theme JSON. Allowlist: themes/**/*.json or legacy tokens/themes/**/*.json (catalog refused).",
        ThemeDeleteArgs,
        ThemeDeleteRet,
        "ThemeDeleteArgs",
        "ThemeDeleteRet",
        "  path: string\n",
        "  ok: boolean\n  path: string\n"
    ),
    contract!(
        "SailTheme",
        "Fork",
        "sail_theme_fork",
        "Fork a catalog or project theme into themes/ (or legacy tokens/themes/). Optional new_id rewrites $extensions.ship.themeId.",
        ThemeForkArgs,
        ThemeForkRet,
        "ThemeForkArgs",
        "ThemeForkRet",
        "  source_path: string\n  dest_path: string\n  new_id?: string\n  content?: string\n",
        "  [key: string]: JsonValue | undefined\n"
    ),
    contract!(
        "SailTheme",
        "Generate",
        "sail_tokens_generate",
        "Run DTCG model → tokens CSS/registry generate. mode=package (full catalog) or mode=app (T0 effective set → project/.deck/themes/themes.css only).",
        ThemeGenerateArgs,
        ThemeGenerateRet,
        "ThemeGenerateArgs",
        "ThemeGenerateRet",
        "  mode?: string\n  project?: string\n  model?: string\n  out?: string\n  workspace?: string\n",
        "  [key: string]: JsonValue | undefined\n"
    ),
    contract!(
        "SailTheme",
        "EffectiveIds",
        "sail_theme_effective_ids",
        "List effective theme ids for a project: project stems + catalog allowlist (T0 enabled ∩ catalog files).",
        ThemeEffectiveIdsArgs,
        ThemeEffectiveIdsRet,
        "ThemeEffectiveIdsArgs",
        "ThemeEffectiveIdsRet",
        "  project?: string\n  workspace?: string\n",
        "  [key: string]: JsonValue | undefined\n"
    ),
    contract!(
        "SailTheme",
        "FontSearch",
        "sail_theme_font_search",
        "Search Google Fonts catalog (cached) by family query. Offline fallback list when network unavailable.",
        ThemeFontSearchArgs,
        ThemeFontSearchRet,
        "ThemeFontSearchArgs",
        "ThemeFontSearchRet",
        "  query?: string\n",
        "  ok: boolean\n  query?: string\n  families: string[]\n  count?: number\n"
    ),
    contract!(
        "SailTheme",
        "FontAdd",
        "sail_theme_font_add",
        "Vendor a font family: download subsetted woff2 → project/fonts + @font-face + license manifest + typography tokens. Offline after selection; no runtime CDN.",
        ThemeFontAddArgs,
        ThemeFontAddRet,
        "ThemeFontAddArgs",
        "ThemeFontAddRet",
        "  family: string\n  axes?: string\n  project?: string\n",
        "  [key: string]: JsonValue | undefined\n"
    ),
    contract!(
        "SailDevServer",
        "Start",
        "sail_dev_server_start",
        "Start (or ensure) a local sail dev server for a project so HMR/studio HTTP exists. Theme ops do not require a server.",
        DevServerStartArgs,
        DevServerStartRet,
        "DevServerStartArgs",
        "DevServerStartRet",
        "  project: string\n  addr?: string\n",
        "  [key: string]: JsonValue | undefined\n"
    ),
    contract!(
        "SailDevServer",
        "Status",
        "sail_dev_server_status",
        "Status of the MCP-tracked dev server (running, url).",
        EmptyArgs,
        DevServerStatusRet,
        "DevServerStatusArgs",
        "DevServerStatusRet",
        "",
        "  [key: string]: JsonValue | undefined\n"
    ),
    contract!(
        "SailDevServer",
        "Stop",
        "sail_dev_server_stop",
        "Stop the MCP-started dev server only (does not kill unrelated sail dev processes).",
        EmptyArgs,
        DevServerStopRet,
        "DevServerStopArgs",
        "DevServerStopRet",
        "",
        "  [key: string]: JsonValue | undefined\n"
    ),
    contract!(
        "SailTask",
        "List",
        "sail_task_list",
        "List ephemeral project tasks under <project>/.sail/tasks (gitignored). Filters: status/label/actor/drift/open. Includes anchor drift.",
        TaskListArgs,
        TaskListRet,
        "TaskListArgs",
        "TaskListRet",
        "  project?: string\n  status?: string\n  label?: string\n  actor?: string\n  drift?: string\n  open?: boolean\n",
        "  ok: boolean\n  tasks: JsonValue[]\n"
    ),
    contract!(
        "SailTask",
        "Get",
        "sail_task_get",
        "Get full task JSON (title, status, anchor, events, artifacts, drift).",
        TaskGetArgs,
        TaskGetRet,
        "TaskGetArgs",
        "TaskGetRet",
        "  project?: string\n  id: string\n",
        "  ok: boolean\n  task: JsonValue\n"
    ),
    contract!(
        "SailTask",
        "Create",
        "sail_task_create",
        "Create a task (ULID id) under <project>/.sail/tasks. Optional anchor + body. Ephemeral — not team SSOT.",
        TaskCreateArgs,
        TaskCreateRet,
        "TaskCreateArgs",
        "TaskCreateRet",
        "  project?: string\n  title: string\n  labels?: string[]\n  status?: string\n  route?: string\n  body?: string\n  anchor?: JsonValue\n  actor?: { kind?: string; name?: string }\n",
        "  [key: string]: JsonValue | undefined\n"
    ),
    contract!(
        "SailTask",
        "Update",
        "sail_task_update",
        "Update task fields. Status changes append a status event (any→any in v1).",
        TaskUpdateArgs,
        TaskUpdateRet,
        "TaskUpdateArgs",
        "TaskUpdateRet",
        "  project?: string\n  id: string\n  title?: string\n  status?: string\n  labels?: string[]\n  assignee?: { kind?: string; name?: string }\n  route?: string\n  actor?: { kind?: string; name?: string }\n",
        "  [key: string]: JsonValue | undefined\n"
    ),
    contract!(
        "SailTask",
        "Comment",
        "sail_task_comment",
        "Append a comment to the task thread.",
        TaskCommentArgs,
        TaskCommentRet,
        "TaskCommentArgs",
        "TaskCommentRet",
        "  project?: string\n  id: string\n  body: string\n  actor?: { kind?: string; name?: string }\n",
        "  [key: string]: JsonValue | undefined\n"
    ),
    contract!(
        "SailTask",
        "AddArtifact",
        "sail_task_add_artifact",
        "Attach an artifact (≤5MB). kind file|render; render carries region metadata. content_base64+filename or path.",
        TaskAddArtifactArgs,
        TaskAddArtifactRet,
        "TaskAddArtifactArgs",
        "TaskAddArtifactRet",
        "  project?: string\n  id: string\n  filename?: string\n  content_base64?: string\n  mime?: string\n  path?: string\n  kind?: string\n  label?: string\n  region?: JsonValue\n  actor?: { kind?: string; name?: string }\n",
        "  [key: string]: JsonValue | undefined\n"
    ),
    contract!(
        "SailTask",
        "AgentNote",
        "sail_task_agent_note",
        "Append an agent progress/summary note (actor kind=agent).",
        TaskAgentNoteArgs,
        TaskAgentNoteRet,
        "TaskAgentNoteArgs",
        "TaskAgentNoteRet",
        "  project?: string\n  id: string\n  body: string\n  actor?: { kind?: string; name?: string }\n",
        "  [key: string]: JsonValue | undefined\n"
    ),
    contract!(
        "SailTask",
        "ThreadCreate",
        "sail_task_thread_create",
        "Create an A-anchored thread (required WI7 identity anchor { nodeId, elementPath?, contentHash?, locator? }). Same store as Create.",
        TaskThreadCreateArgs,
        TaskThreadCreateRet,
        "TaskThreadCreateArgs",
        "TaskThreadCreateRet",
        "  project?: string\n  title: string\n  body?: string\n  anchor: JsonValue\n  labels?: string[]\n  status?: string\n  route?: string\n  actor?: { kind?: string; name?: string }\n",
        "  [key: string]: JsonValue | undefined\n"
    ),
    contract!(
        "SailTask",
        "ThreadReply",
        "sail_task_thread_reply",
        "Reply on a thread (comment with actor attribution).",
        TaskThreadReplyArgs,
        TaskThreadReplyRet,
        "TaskThreadReplyArgs",
        "TaskThreadReplyRet",
        "  project?: string\n  id: string\n  body: string\n  actor?: { kind?: string; name?: string }\n",
        "  [key: string]: JsonValue | undefined\n"
    ),
    contract!(
        "SailTask",
        "ThreadList",
        "sail_task_thread_list",
        "List threads with anchor drift (ok|moved|lost). Filters: open/actor/drift/status.",
        TaskThreadListArgs,
        TaskThreadListRet,
        "TaskThreadListArgs",
        "TaskThreadListRet",
        "  project?: string\n  status?: string\n  label?: string\n  actor?: string\n  drift?: string\n  open?: boolean\n",
        "  ok: boolean\n  tasks: JsonValue[]\n"
    ),
    contract!(
        "SailTask",
        "Reanchor",
        "sail_task_reanchor",
        "Explicitly re-anchor a thread to a new node (lost never rebinds silently).",
        TaskReanchorArgs,
        TaskReanchorRet,
        "TaskReanchorArgs",
        "TaskReanchorRet",
        "  project?: string\n  id: string\n  anchor: JsonValue\n  actor?: { kind?: string; name?: string }\n",
        "  [key: string]: JsonValue | undefined\n"
    ),
    contract!(
        "SailTask",
        "CommitIntent",
        "sail_task_commit_intent",
        "Export thread as a decision record under <project>/decisions/ (committable). .sail/tasks stays gitignored.",
        TaskCommitIntentArgs,
        TaskCommitIntentRet,
        "TaskCommitIntentArgs",
        "TaskCommitIntentRet",
        "  project?: string\n  id: string\n  actor?: { kind?: string; name?: string }\n",
        "  [key: string]: JsonValue | undefined\n"
    ),
    contract!(
        "SailRender",
        "Screenshot",
        "sail_render",
        "Static themed screenshot from a .sail file or inline source (no browser). Returns PNG base64 + regions (node ids) + dynamics list.",
        RenderScreenshotArgs,
        RenderScreenshotRet,
        "RenderScreenshotArgs",
        "RenderScreenshotRet",
        "  path?: string\n  source?: string\n  viewport?: JsonValue\n  theme?: string\n  mode?: string\n",
        "  imagePng: string\n  regions: JsonValue[]\n  dynamics: JsonValue[]\n  viewport: JsonValue\n  theme: string\n  ms: number\n  file?: string\n"
    ),
];

/// Lookup contract by service/method.
pub fn find_contract(service: &str, method: &str) -> Option<&'static MethodContract> {
    METHOD_CONTRACTS
        .iter()
        .find(|c| c.service == service && c.method == method)
}

/// Re-export Value for flatten helpers (handlers still build dynamic JSON).
pub type DynJson = Value;
