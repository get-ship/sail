//! Module seam (TOOL-11): manifest load, capability gate, dynamic ops registry.
//!
//! Core knows nothing about paid product modules. Discovery is path-based;
//! ops methods register into the same dispatch table as core (CLI + MCP + HTTP).
//! Studio activities are additive rail entries only (no core-surface injection).
//!
//! # Capability enforcement (T-I6)
//!
//! Undeclared fs/net power is denied **at `LoadedModule::call`**, not only via
//! optional helpers. Args are scanned for path/url intents before the backend
//! runs; side-effect backends must also use [`ModuleCallCtx`] for any further
//! fs/net access.
//!
//! # Signature (v1)
//!
//! Outside Dev, `signature` must be a **non-empty non-whitespace string**.
//! v1 does **not** perform cryptographic verification — presence is the gate
//! (fixture-grade). Crypto verify is reserved for a later api bump.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{OnceLock, RwLock};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::error::OpError;
use crate::typed::MethodContract;

/// Manifest API major version supported by this loader.
pub const MODULE_API_VERSION: u32 = 1;

/// Named error codes (stable for CLI/MCP/HTTP).
pub const ERR_UNSIGNED: &str = "module_unsigned";
pub const ERR_API_UNSUPPORTED: &str = "module_api_unsupported";
pub const ERR_CAPABILITY: &str = "module_capability_denied";
pub const ERR_MANIFEST: &str = "module_manifest_invalid";
pub const ERR_LOAD: &str = "module_load_failed";

/// Runtime mode: production refuses unsigned modules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleMode {
    /// Allow unsigned modules (loud banner / `dev: true` on activities).
    Dev,
    /// Require non-empty `signature` on every loaded module.
    Production,
}

impl ModuleMode {
    /// `SAIL_MODULES_DEV=1` / `SAIL_DEV=1` → Dev; else Production.
    pub fn from_env() -> Self {
        let dev = env_truthy("SAIL_MODULES_DEV") || env_truthy("SAIL_DEV");
        if dev { Self::Dev } else { Self::Production }
    }
}

fn env_truthy(key: &str) -> bool {
    match std::env::var(key) {
        Ok(v) => {
            let v = v.trim();
            v == "1" || v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("yes")
        }
        Err(_) => false,
    }
}

/// `sail-module.json` on disk.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModuleManifest {
    pub name: String,
    pub version: String,
    /// Manifest schema major version. Loader rejects `api > MODULE_API_VERSION`.
    pub api: u32,
    #[serde(default)]
    pub provides: ModuleProvides,
    #[serde(default)]
    pub capabilities: ModuleCapabilities,
    /// Required outside Dev.
    ///
    /// **v1 semantics:** non-empty (after trim) string = "signed" for load
    /// purposes. Empty, whitespace-only, or missing → unsigned. No crypto
    /// verify in api 1 — real signatures need a later manifest api.
    #[serde(default)]
    pub signature: Option<String>,
    #[serde(default)]
    pub backend: ModuleBackend,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ModuleProvides {
    #[serde(default)]
    pub ops: Option<ModuleOpsProvide>,
    #[serde(default)]
    pub activity: Option<ModuleActivity>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModuleOpsProvide {
    pub namespace: String,
    #[serde(default)]
    pub methods: Vec<ModuleMethodDecl>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModuleMethodDecl {
    pub name: String,
    #[serde(default)]
    pub mcp_tool: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    /// Optional JSON Schema for args (object).
    #[serde(default)]
    pub input_schema: Option<Value>,
    /// Optional JSON Schema for return.
    #[serde(default)]
    pub output_schema: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModuleActivity {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub icon: Option<String>,
    /// Served under Studio shell, e.g. `/_studio/x/hello`.
    pub route: String,
}

/// Deny-by-default power. Empty lists grant nothing.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ModuleCapabilities {
    /// e.g. `read:project`, `write:.sail/`, `write:project`
    #[serde(default)]
    pub fs: Vec<String>,
    /// Host allowlist (exact or suffix `.example.com`).
    #[serde(default)]
    pub net: Vec<String>,
}

/// How the module implements its ops.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ModuleBackend {
    /// Deterministic responses from manifest / handlers map (reference modules).
    Static {
        #[serde(default)]
        methods: HashMap<String, Value>,
    },
    /// Reserved for process sidecars (not executed in v1 beyond static).
    Process {
        #[serde(default)]
        command: Vec<String>,
    },
}

impl Default for ModuleBackend {
    fn default() -> Self {
        Self::Static {
            methods: HashMap::new(),
        }
    }
}

/// A successfully loaded module.
#[derive(Debug, Clone)]
pub struct LoadedModule {
    pub root: PathBuf,
    pub manifest: ModuleManifest,
    pub signed: bool,
    pub dev_unsigned: bool,
}

impl LoadedModule {
    pub fn namespace(&self) -> Option<&str> {
        self.manifest
            .provides
            .ops
            .as_ref()
            .map(|o| o.namespace.as_str())
    }

    pub fn activity(&self) -> Option<&ModuleActivity> {
        self.manifest.provides.activity.as_ref()
    }

    /// Capability gate: filesystem write.
    pub fn check_fs_write(&self, rel_or_abs: &str) -> Result<(), OpError> {
        check_fs_cap(&self.manifest.capabilities.fs, "write", rel_or_abs)
    }

    /// Capability gate: filesystem read.
    pub fn check_fs_read(&self, rel_or_abs: &str) -> Result<(), OpError> {
        check_fs_cap(&self.manifest.capabilities.fs, "read", rel_or_abs)
    }

    /// Capability gate: network host.
    pub fn check_net(&self, host: &str) -> Result<(), OpError> {
        let host = host.trim().to_ascii_lowercase();
        if host.is_empty() {
            return Err(capability_err("net: empty host"));
        }
        for entry in &self.manifest.capabilities.net {
            let e = entry.trim().to_ascii_lowercase();
            if e.is_empty() {
                continue;
            }
            if host == e || host.ends_with(&format!(".{e}")) {
                return Ok(());
            }
        }
        Err(capability_err(format!(
            "net:{host} not in allowlist {:?}",
            self.manifest.capabilities.net
        )))
    }

    /// Scan RPC args for fs/net intents and enforce capabilities (call-edge gate).
    ///
    /// Keys that imply filesystem access: `path`, `file`, `filepath`, `write_path`,
    /// `read_path`, `dest`, `dest_path`, `source_path`, `dir`.
    /// Keys that imply network: `host`, `url`, `endpoint`, `base_url`, `baseUrl`.
    /// Write intent is inferred from key name (`write*`) or boolean `write`/`overwrite`.
    pub fn gate_call_args(&self, args: &Value) -> Result<(), OpError> {
        gate_value_capabilities(self, args, false)
    }

    /// Invoke a provided method. Capabilities are enforced **before** the backend
    /// runs: undeclared fs/net intents in `args` → `module_capability_denied`.
    pub fn call(&self, method: &str, args: &Value) -> Result<Value, OpError> {
        let ops = self.manifest.provides.ops.as_ref().ok_or_else(|| {
            OpError::NotFound(format!("module `{}` provides no ops", self.manifest.name))
        })?;
        if !ops.methods.iter().any(|m| m.name == method) {
            return Err(OpError::NotFound(format!(
                "unknown RPC `{}/{}`",
                ops.namespace, method
            )));
        }
        // T-I6: deny undeclared power at the call edge (not only via helpers).
        self.gate_call_args(args)?;
        let _ctx = ModuleCallCtx { module: self };
        match &self.manifest.backend {
            ModuleBackend::Static { methods } => {
                // Static backends must not perform fs/net; args already gated.
                if let Some(v) = methods.get(method) {
                    return Ok(v.clone());
                }
                // Convention: Ping → pong when declared but body omitted.
                if method == "Ping" {
                    return Ok(json!({
                        "ok": true,
                        "message": "pong",
                        "module": self.manifest.name,
                        "version": self.manifest.version,
                    }));
                }
                Err(OpError::Internal(format!(
                    "module `{}` static backend has no body for `{method}`",
                    self.manifest.name
                )))
            }
            ModuleBackend::Process { command } => {
                // Process sidecars are not executed in-process in v1; still
                // refuse if the call already declared undeclared power (above).
                let _ = _ctx;
                Err(OpError::FailedPrecondition(format!(
                    "module `{}` process backend not executed in-process (command={command:?})",
                    self.manifest.name
                )))
            }
        }
    }

    /// Optional surface HTML for the activity mount (`surface.html` next to manifest).
    pub fn surface_html(&self) -> Option<String> {
        let p = self.root.join("surface.html");
        fs::read_to_string(p).ok()
    }
}

fn capability_err(msg: impl Into<String>) -> OpError {
    OpError::PermissionDenied(format!("{ERR_CAPABILITY}: {}", msg.into()))
}

/// Call-time capability context for side-effect backends.
/// All fs/net access from a module handler must go through these methods.
#[derive(Debug)]
pub struct ModuleCallCtx<'a> {
    module: &'a LoadedModule,
}

impl ModuleCallCtx<'_> {
    pub fn fs_write(&self, path: &str) -> Result<(), OpError> {
        self.module.check_fs_write(path)
    }

    pub fn fs_read(&self, path: &str) -> Result<(), OpError> {
        self.module.check_fs_read(path)
    }

    pub fn net(&self, host: &str) -> Result<(), OpError> {
        self.module.check_net(host)
    }

    pub fn module(&self) -> &LoadedModule {
        self.module
    }
}

const FS_PATH_KEYS: &[&str] = &[
    "path",
    "file",
    "filepath",
    "write_path",
    "read_path",
    "dest",
    "dest_path",
    "source_path",
    "dir",
];

const NET_KEYS: &[&str] = &["host", "url", "endpoint", "base_url", "baseUrl"];

fn gate_value_capabilities(
    module: &LoadedModule,
    value: &Value,
    parent_write: bool,
) -> Result<(), OpError> {
    match value {
        Value::Object(map) => {
            let write_flag = parent_write
                || map.get("write").and_then(|v| v.as_bool()).unwrap_or(false)
                || map
                    .get("overwrite")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

            for (key, child) in map {
                let key_l = key.to_ascii_lowercase();
                let is_write_key =
                    key_l.contains("write") || key_l == "dest" || key_l == "dest_path";
                let want_write = write_flag || is_write_key;

                if FS_PATH_KEYS.iter().any(|k| key.eq_ignore_ascii_case(k)) {
                    if let Some(path) = child.as_str() {
                        if want_write {
                            module.check_fs_write(path)?;
                        } else {
                            module.check_fs_read(path)?;
                        }
                    }
                }
                if NET_KEYS.iter().any(|k| key.eq_ignore_ascii_case(k)) {
                    if let Some(s) = child.as_str() {
                        let host = host_from_net_arg(s);
                        module.check_net(&host)?;
                    }
                }
                gate_value_capabilities(module, child, want_write)?;
            }
            Ok(())
        }
        Value::Array(arr) => {
            for v in arr {
                gate_value_capabilities(module, v, parent_write)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn host_from_net_arg(s: &str) -> String {
    let s = s.trim();
    // url-like: https://host/path → host
    if let Some(rest) = s
        .strip_prefix("https://")
        .or_else(|| s.strip_prefix("http://"))
    {
        return rest
            .split(['/', '?', '#'])
            .next()
            .unwrap_or(rest)
            .split('@')
            .next_back()
            .unwrap_or(rest)
            .split(':')
            .next()
            .unwrap_or(rest)
            .to_string();
    }
    // bare host or host:port
    s.split(['/', '?', '#', ':'])
        .next()
        .unwrap_or(s)
        .to_string()
}

fn check_fs_cap(caps: &[String], kind: &str, path: &str) -> Result<(), OpError> {
    let path_norm = path.replace('\\', "/");
    for cap in caps {
        let cap = cap.trim();
        let Some((k, scope)) = cap.split_once(':') else {
            continue;
        };
        if k != kind {
            continue;
        }
        let scope = scope.trim();
        if scope == "project" || scope == "*" {
            return Ok(());
        }
        // Prefix scopes like `.sail/` — path starts with scope, or has `/{scope}` segment.
        let scope_norm = scope.trim_start_matches("./");
        let path_trim = path_norm.trim_start_matches("./");
        if path_trim.starts_with(scope_norm) || path_trim.contains(&format!("/{scope_norm}")) {
            return Ok(());
        }
    }
    Err(capability_err(format!(
        "fs {kind} denied for `{path}` (caps={caps:?})"
    )))
}

/// Load errors with stable names.
#[derive(Debug, Clone, thiserror::Error)]
pub enum ModuleLoadError {
    #[error("{ERR_MANIFEST}: {0}")]
    Manifest(String),
    #[error("{ERR_API_UNSUPPORTED}: module api {got} > supported {MODULE_API_VERSION}")]
    ApiUnsupported { got: u32 },
    #[error("{ERR_UNSIGNED}: module `{name}` has no signature (dev-only)")]
    Unsigned { name: String },
    #[error("{ERR_LOAD}: {0}")]
    Io(String),
}

impl From<ModuleLoadError> for OpError {
    fn from(e: ModuleLoadError) -> Self {
        match e {
            ModuleLoadError::Manifest(m) => OpError::InvalidArgument(m),
            ModuleLoadError::ApiUnsupported { got } => OpError::FailedPrecondition(format!(
                "{ERR_API_UNSUPPORTED}: module api {got} > supported {MODULE_API_VERSION}"
            )),
            ModuleLoadError::Unsigned { name } => OpError::PermissionDenied(format!(
                "{ERR_UNSIGNED}: module `{name}` has no signature (dev-only)"
            )),
            ModuleLoadError::Io(m) => OpError::Internal(m),
        }
    }
}

/// Parse + validate a manifest file without registering it.
pub fn load_manifest_at(path: &Path) -> Result<ModuleManifest, ModuleLoadError> {
    let text = fs::read_to_string(path)
        .map_err(|e| ModuleLoadError::Io(format!("read {}: {e}", path.display())))?;
    let manifest: ModuleManifest = serde_json::from_str(&text)
        .map_err(|e| ModuleLoadError::Manifest(format!("{}: {e}", path.display())))?;
    validate_manifest(&manifest)?;
    Ok(manifest)
}

fn validate_manifest(m: &ModuleManifest) -> Result<(), ModuleLoadError> {
    if m.name.trim().is_empty() {
        return Err(ModuleLoadError::Manifest("name is required".into()));
    }
    if m.version.trim().is_empty() {
        return Err(ModuleLoadError::Manifest("version is required".into()));
    }
    if m.api == 0 {
        return Err(ModuleLoadError::Manifest("api must be >= 1".into()));
    }
    if m.api > MODULE_API_VERSION {
        return Err(ModuleLoadError::ApiUnsupported { got: m.api });
    }
    if let Some(ops) = &m.provides.ops {
        if ops.namespace.trim().is_empty() {
            return Err(ModuleLoadError::Manifest(
                "provides.ops.namespace required".into(),
            ));
        }
        // Core namespaces reserved
        if ops.namespace.starts_with("Sail") {
            return Err(ModuleLoadError::Manifest(format!(
                "namespace `{}` is reserved for core",
                ops.namespace
            )));
        }
        for method in &ops.methods {
            if method.name.trim().is_empty() {
                return Err(ModuleLoadError::Manifest("method name required".into()));
            }
        }
    }
    if let Some(act) = &m.provides.activity {
        if act.id.trim().is_empty() || act.route.trim().is_empty() {
            return Err(ModuleLoadError::Manifest(
                "activity id and route are required".into(),
            ));
        }
        if !act.route.starts_with("/_studio/x/") {
            return Err(ModuleLoadError::Manifest(
                "activity.route must start with /_studio/x/".into(),
            ));
        }
    }
    Ok(())
}

/// v1: non-empty after trim. Whitespace-only / missing → unsigned.
/// No cryptographic verification in api 1.
fn is_signed(m: &ModuleManifest) -> bool {
    m.signature
        .as_ref()
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false)
}

/// Load a module directory containing `sail-module.json`.
pub fn load_module_dir(dir: &Path, mode: ModuleMode) -> Result<LoadedModule, ModuleLoadError> {
    let manifest_path = dir.join("sail-module.json");
    let manifest = load_manifest_at(&manifest_path)?;
    let signed = is_signed(&manifest);
    if !signed && mode == ModuleMode::Production {
        return Err(ModuleLoadError::Unsigned {
            name: manifest.name,
        });
    }
    Ok(LoadedModule {
        root: dir.to_path_buf(),
        manifest,
        signed,
        dev_unsigned: !signed && mode == ModuleMode::Dev,
    })
}

/// Discover module roots under a parent directory (`*/sail-module.json`).
pub fn discover_module_dirs(parent: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(rd) = fs::read_dir(parent) else {
        return out;
    };
    for ent in rd.flatten() {
        let p = ent.path();
        if p.is_dir() && p.join("sail-module.json").is_file() {
            out.push(p);
        }
    }
    out.sort();
    out
}

/// Standard discovery roots for a workspace + project.
pub fn default_discovery_roots(workspace: &Path, project: &Path) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    // Explicit colon/semicolon-separated dirs of modules (or single module roots).
    if let Ok(extra) = std::env::var("SAIL_MODULES") {
        for part in extra.split([':', ';']) {
            let part = part.trim();
            if !part.is_empty() {
                roots.push(PathBuf::from(part));
            }
        }
    }
    roots.push(workspace.join("modules"));
    roots.push(project.join(".sail").join("modules"));
    // Reference modules in-repo (OSS fixtures); always discovered when present.
    roots.push(workspace.join("examples"));
    roots
}

/// Load all modules from discovery roots. Failures of individual modules are
/// collected; a hard error only if `strict` and any fail.
pub fn load_all(
    workspace: &Path,
    project: &Path,
    mode: ModuleMode,
) -> (Vec<LoadedModule>, Vec<ModuleLoadError>) {
    let mut modules = Vec::new();
    let mut errors = Vec::new();
    let mut seen_names = std::collections::HashSet::new();

    for root in default_discovery_roots(workspace, project) {
        // Root may be a single module dir or a parent of modules.
        if root.join("sail-module.json").is_file() {
            match load_module_dir(&root, mode) {
                Ok(m) => {
                    if seen_names.insert(m.manifest.name.clone()) {
                        modules.push(m);
                    }
                }
                Err(e) => errors.push(e),
            }
            continue;
        }
        // Parent: only scan dirs named module-* under examples/, else all children.
        let children = if root.ends_with("examples") {
            discover_module_dirs(&root)
                .into_iter()
                .filter(|p| {
                    p.file_name()
                        .and_then(|n| n.to_str())
                        .is_some_and(|n| n.starts_with("module-"))
                })
                .collect()
        } else {
            discover_module_dirs(&root)
        };
        for dir in children {
            match load_module_dir(&dir, mode) {
                Ok(m) => {
                    if seen_names.insert(m.manifest.name.clone()) {
                        modules.push(m);
                    }
                }
                Err(e) => errors.push(e),
            }
        }
    }
    (modules, errors)
}

// ── Global runtime registry ─────────────────────────────────────────────────

#[derive(Debug, Default)]
struct ModuleRuntime {
    modules: Vec<LoadedModule>,
    /// service → (method → module index)
    index: HashMap<String, HashMap<String, usize>>,
    /// mcp tool → (service, method)
    mcp: HashMap<String, (String, String)>,
}

impl ModuleRuntime {
    fn rebuild_index(&mut self) {
        self.index.clear();
        self.mcp.clear();
        for (i, m) in self.modules.iter().enumerate() {
            let Some(ops) = m.manifest.provides.ops.as_ref() else {
                continue;
            };
            let entry = self.index.entry(ops.namespace.clone()).or_default();
            for method in &ops.methods {
                entry.insert(method.name.clone(), i);
                let tool = method.mcp_tool.clone().unwrap_or_else(|| {
                    format!(
                        "{}_{}",
                        camel_to_snake(&ops.namespace),
                        camel_to_snake(&method.name)
                    )
                });
                self.mcp
                    .insert(tool, (ops.namespace.clone(), method.name.clone()));
            }
        }
    }
}

fn camel_to_snake(s: &str) -> String {
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                out.push('_');
            }
            out.extend(c.to_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

fn runtime() -> &'static RwLock<ModuleRuntime> {
    static RT: OnceLock<RwLock<ModuleRuntime>> = OnceLock::new();
    RT.get_or_init(|| RwLock::new(ModuleRuntime::default()))
}

/// Replace the global module set (tests + process bootstrap).
pub fn install_modules(modules: Vec<LoadedModule>) {
    let mut rt = runtime().write().expect("module runtime poisoned");
    rt.modules = modules;
    rt.rebuild_index();
}

/// Clear all loaded modules (tests).
pub fn clear_modules() {
    install_modules(Vec::new());
}

/// Bootstrap from env roots using [`ModuleMode::from_env`].
pub fn bootstrap_from_env(workspace: &Path, project: &Path) {
    let mode = ModuleMode::from_env();
    let (modules, errors) = load_all(workspace, project, mode);
    for e in &errors {
        // Soft: discovery noise (unsigned in prod for random dirs) stays quiet unless SAIL_MODULES_VERBOSE.
        if env_truthy("SAIL_MODULES_VERBOSE") {
            eprintln!("sail modules: {e}");
        }
    }
    install_modules(modules);
}

/// Ensure modules are loaded once for this process (lazy, from env roots).
pub fn ensure_bootstrapped(workspace: &Path, project: &Path) {
    static ONCE: OnceLock<()> = OnceLock::new();
    // If tests already installed modules, don't clobber.
    {
        let rt = runtime().read().expect("module runtime poisoned");
        if !rt.modules.is_empty() {
            return;
        }
    }
    let _ = ONCE.get_or_init(|| {
        bootstrap_from_env(workspace, project);
    });
}

/// Force re-bootstrap (tests).
pub fn rebootstrap(workspace: &Path, project: &Path) {
    bootstrap_from_env(workspace, project);
}

pub fn loaded_modules() -> Vec<LoadedModule> {
    runtime()
        .read()
        .expect("module runtime poisoned")
        .modules
        .clone()
}

/// Dispatch a module RPC if registered.
pub fn call_module_rpc(
    service: &str,
    method: &str,
    args: &Value,
) -> Option<Result<Value, OpError>> {
    let rt = runtime().read().expect("module runtime poisoned");
    let idx = rt.index.get(service)?.get(method).copied()?;
    let m = rt.modules.get(idx)?;
    Some(m.call(method, args))
}

/// Map MCP tool name → (service, method) for module tools.
pub fn module_mcp_tool_to_rpc(name: &str) -> Option<(String, String)> {
    let rt = runtime().read().expect("module runtime poisoned");
    rt.mcp.get(name).cloned()
}

/// `"Service/Method"` list for health.
pub fn module_rpc_methods() -> Vec<String> {
    let rt = runtime().read().expect("module runtime poisoned");
    let mut out = Vec::new();
    for (svc, methods) in &rt.index {
        for method in methods.keys() {
            out.push(format!("{svc}/{method}"));
        }
    }
    out.sort();
    out
}

/// MCP tool descriptors for loaded modules (merged by callers).
pub fn module_mcp_tool_descriptors() -> Vec<Value> {
    let rt = runtime().read().expect("module runtime poisoned");
    let mut tools = Vec::new();
    for m in &rt.modules {
        let Some(ops) = m.manifest.provides.ops.as_ref() else {
            continue;
        };
        for method in &ops.methods {
            let tool = method.mcp_tool.clone().unwrap_or_else(|| {
                format!(
                    "{}_{}",
                    camel_to_snake(&ops.namespace),
                    camel_to_snake(&method.name)
                )
            });
            let description = method.description.clone().unwrap_or_else(|| {
                format!(
                    "{}/{} (module {})",
                    ops.namespace, method.name, m.manifest.name
                )
            });
            let input_schema = method
                .input_schema
                .clone()
                .unwrap_or_else(|| json!({"type": "object", "properties": {}}));
            tools.push(json!({
                "name": tool,
                "description": description,
                "inputSchema": input_schema,
            }));
        }
    }
    tools.sort_by(|a, b| {
        a.get("name")
            .and_then(|n| n.as_str())
            .cmp(&b.get("name").and_then(|n| n.as_str()))
    });
    tools
}

/// Activities for Studio rail mount.
pub fn module_activities() -> Vec<Value> {
    let rt = runtime().read().expect("module runtime poisoned");
    rt.modules
        .iter()
        .filter_map(|m| {
            let act = m.activity()?;
            Some(json!({
                "id": act.id,
                "label": act.label,
                "icon": act.icon,
                "route": act.route,
                "module": m.manifest.name,
                "version": m.manifest.version,
                "devUnsigned": m.dev_unsigned,
            }))
        })
        .collect()
}

/// Connect paths for every loaded module method (HTTP mount).
pub fn module_connect_paths() -> Vec<String> {
    module_rpc_methods()
        .into_iter()
        .filter_map(|rpc| {
            let (s, m) = rpc.split_once('/')?;
            Some(format!("/sail.ops.v1.{s}/{m}"))
        })
        .collect()
}

/// Public modules listing (health / schema extension).
pub fn modules_json() -> Value {
    let rt = runtime().read().expect("module runtime poisoned");
    let modules: Vec<Value> = rt
        .modules
        .iter()
        .map(|m| {
            json!({
                "name": m.manifest.name,
                "version": m.manifest.version,
                "api": m.manifest.api,
                "signed": m.signed,
                "devUnsigned": m.dev_unsigned,
                "root": m.root.display().to_string(),
                "ops": m.manifest.provides.ops.as_ref().map(|o| json!({
                    "namespace": o.namespace,
                    "methods": o.methods.iter().map(|mm| &mm.name).collect::<Vec<_>>(),
                })),
                "activity": m.activity(),
                "capabilities": m.manifest.capabilities,
            })
        })
        .collect();
    json!({
        "ok": true,
        "api": MODULE_API_VERSION,
        "modules": modules,
        "activities": module_activities(),
        "methods": module_rpc_methods(),
    })
}

/// Dynamic TS client fragment for loaded modules (no core codegen change).
/// Callers can write this next to the generated client or evaluate at runtime.
pub fn generate_module_ts_extension(modules: &[LoadedModule]) -> String {
    let mut out = String::new();
    out.push_str("/**\n");
    out.push_str(" * Module ops extension — generated from loaded sail-module manifests.\n");
    out.push_str(" * Uses callRpc; core generated client is unchanged (TOOL-11).\n");
    out.push_str(" */\n\n");
    out.push_str("import { callRpc, type ClientOptions, type JsonObject } from './client'\n\n");
    for m in modules {
        let Some(ops) = m.manifest.provides.ops.as_ref() else {
            continue;
        };
        for method in &ops.methods {
            let fn_name = format!("{}{}", lowercase_first(&ops.namespace), method.name);
            out.push_str(&format!(
                "/** {}/{} — module {} */\n",
                ops.namespace, method.name, m.manifest.name
            ));
            out.push_str(&format!(
                "export async function {fn_name}(\n  args: object = {{}},\n  opts?: ClientOptions,\n): Promise<JsonObject> {{\n  return callRpc('{}', '{}', args, opts)\n}}\n\n",
                ops.namespace, method.name
            ));
        }
    }
    out
}

fn lowercase_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_lowercase().collect::<String>() + c.as_str(),
    }
}

/// Build ephemeral [`MethodContract`]-like metadata is not needed for static tables;
/// modules stay dynamic. This helper exists for schema catalog merge.
pub fn module_schema_catalog_entries() -> Vec<Value> {
    let rt = runtime().read().expect("module runtime poisoned");
    let mut methods = Vec::new();
    for m in &rt.modules {
        let Some(ops) = m.manifest.provides.ops.as_ref() else {
            continue;
        };
        for method in &ops.methods {
            let tool = method.mcp_tool.clone().unwrap_or_else(|| {
                format!(
                    "{}_{}",
                    camel_to_snake(&ops.namespace),
                    camel_to_snake(&method.name)
                )
            });
            methods.push(json!({
                "service": ops.namespace,
                "method": method.name,
                "mcp_tool": tool,
                "description": method.description.clone().unwrap_or_default(),
                "path": format!("/sail.ops.v1.{}/{}", ops.namespace, method.name),
                "inputSchema": method.input_schema.clone().unwrap_or(json!({"type":"object","properties":{}})),
                "outputSchema": method.output_schema.clone().unwrap_or(json!({"type":"object"})),
                "module": m.manifest.name,
            }));
        }
    }
    methods
}

/// Resolve activity id → surface HTML (if module provides a file).
pub fn activity_surface_html(activity_id: &str) -> Option<(LoadedModule, String)> {
    let rt = runtime().read().expect("module runtime poisoned");
    for m in &rt.modules {
        if m.activity().is_some_and(|a| a.id == activity_id) {
            if let Some(html) = m.surface_html() {
                return Some((m.clone(), html));
            }
            // Minimal default surface when file missing.
            let label = m
                .activity()
                .map(|a| a.label.as_str())
                .unwrap_or(activity_id);
            let html = format!(
                "<!doctype html><html><body data-module-surface=\"{}\"><h1>{}</h1><p>module {} v{}</p></body></html>",
                activity_id, label, m.manifest.name, m.manifest.version
            );
            return Some((m.clone(), html));
        }
    }
    None
}

// Silence unused import when MethodContract not used in all cfgs.
#[allow(dead_code)]
fn _contract_marker(_: &MethodContract) {}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn write_hello(dir: &Path, api: u32, signature: Option<&str>, caps_fs: &[&str]) {
        write_hello_with_net(dir, api, signature, caps_fs, &[]);
    }

    fn write_hello_with_net(
        dir: &Path,
        api: u32,
        signature: Option<&str>,
        caps_fs: &[&str],
        caps_net: &[&str],
    ) {
        let mut methods = HashMap::new();
        methods.insert(
            "Ping".into(),
            json!({"ok": true, "message": "pong", "module": "module-hello"}),
        );
        let manifest = ModuleManifest {
            name: "module-hello".into(),
            version: "0.1.0".into(),
            api,
            provides: ModuleProvides {
                ops: Some(ModuleOpsProvide {
                    namespace: "ModuleHello".into(),
                    methods: vec![ModuleMethodDecl {
                        name: "Ping".into(),
                        mcp_tool: Some("module_hello_ping".into()),
                        description: Some("Reference module ping".into()),
                        input_schema: None,
                        output_schema: None,
                    }],
                }),
                activity: Some(ModuleActivity {
                    id: "hello".into(),
                    label: "Hello".into(),
                    icon: Some("hand".into()),
                    route: "/_studio/x/hello".into(),
                }),
            },
            capabilities: ModuleCapabilities {
                fs: caps_fs.iter().map(|s| (*s).to_string()).collect(),
                net: caps_net.iter().map(|s| (*s).to_string()).collect(),
            },
            signature: signature.map(|s| s.to_string()),
            backend: ModuleBackend::Static { methods },
        };
        fs::create_dir_all(dir).unwrap();
        fs::write(
            dir.join("sail-module.json"),
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();
        fs::write(
            dir.join("surface.html"),
            "<!doctype html><html><body data-module-surface=\"hello\">Hello module</body></html>",
        )
        .unwrap();
    }

    #[test]
    fn rejects_api_greater_than_one() {
        let dir = tempdir().unwrap();
        write_hello(dir.path(), 2, Some("sig"), &[]);
        let err = load_module_dir(dir.path(), ModuleMode::Dev).unwrap_err();
        assert!(matches!(err, ModuleLoadError::ApiUnsupported { got: 2 }));
        let msg = err.to_string();
        assert!(msg.contains(ERR_API_UNSUPPORTED), "{msg}");
    }

    #[test]
    fn unsigned_refused_outside_dev() {
        let dir = tempdir().unwrap();
        write_hello(dir.path(), 1, None, &[]);
        let err = load_module_dir(dir.path(), ModuleMode::Production).unwrap_err();
        assert!(matches!(err, ModuleLoadError::Unsigned { .. }));
        assert!(err.to_string().contains(ERR_UNSIGNED));
        // Dev allows unsigned
        let m = load_module_dir(dir.path(), ModuleMode::Dev).unwrap();
        assert!(m.dev_unsigned);
        assert!(!m.signed);
    }

    #[test]
    fn capability_less_fs_write_denied() {
        let dir = tempdir().unwrap();
        write_hello(dir.path(), 1, Some("dev-sig"), &[]);
        let m = load_module_dir(dir.path(), ModuleMode::Production).unwrap();
        let err = m.check_fs_write(".sail/x").unwrap_err();
        assert!(matches!(err, OpError::PermissionDenied(_)));
        assert!(err.to_string().contains(ERR_CAPABILITY));
    }

    #[test]
    fn capability_write_sail_allows_prefix() {
        let dir = tempdir().unwrap();
        write_hello(dir.path(), 1, Some("dev-sig"), &["write:.sail/"]);
        let m = load_module_dir(dir.path(), ModuleMode::Production).unwrap();
        m.check_fs_write(".sail/tasks/a.json").unwrap();
        let err = m.check_fs_write("src/main.rs").unwrap_err();
        assert!(err.to_string().contains(ERR_CAPABILITY));
    }

    #[test]
    fn call_denies_undeclared_fs_write_in_args() {
        let dir = tempdir().unwrap();
        write_hello(dir.path(), 1, Some("dev-sig"), &[]);
        let m = load_module_dir(dir.path(), ModuleMode::Production).unwrap();
        // Empty args still allowed (no fs/net intent).
        m.call("Ping", &json!({})).unwrap();
        // path + write without capability → denied at call edge.
        let err = m
            .call("Ping", &json!({"path": ".sail/x", "write": true}))
            .unwrap_err();
        assert!(err.to_string().contains(ERR_CAPABILITY), "{err}");
        let err = m
            .call("Ping", &json!({"write_path": "src/main.rs"}))
            .unwrap_err();
        assert!(err.to_string().contains(ERR_CAPABILITY), "{err}");
    }

    #[test]
    fn call_denies_undeclared_net_in_args() {
        let dir = tempdir().unwrap();
        write_hello(dir.path(), 1, Some("dev-sig"), &[]);
        let m = load_module_dir(dir.path(), ModuleMode::Production).unwrap();
        let err = m
            .call("Ping", &json!({"url": "https://evil.example/deploy"}))
            .unwrap_err();
        assert!(err.to_string().contains(ERR_CAPABILITY), "{err}");
    }

    #[test]
    fn call_allows_declared_net_host() {
        let dir = tempdir().unwrap();
        write_hello_with_net(dir.path(), 1, Some("dev-sig"), &[], &["deploy.example.com"]);
        let m = load_module_dir(dir.path(), ModuleMode::Production).unwrap();
        m.call("Ping", &json!({"url": "https://deploy.example.com/v1"}))
            .unwrap();
        let err = m
            .call("Ping", &json!({"host": "other.example.com"}))
            .unwrap_err();
        assert!(err.to_string().contains(ERR_CAPABILITY), "{err}");
    }

    #[test]
    fn call_allows_declared_fs_write() {
        let dir = tempdir().unwrap();
        write_hello(dir.path(), 1, Some("dev-sig"), &["write:.sail/"]);
        let m = load_module_dir(dir.path(), ModuleMode::Production).unwrap();
        m.call("Ping", &json!({"path": ".sail/out.json", "write": true}))
            .unwrap();
    }

    #[test]
    fn whitespace_signature_is_unsigned() {
        let dir = tempdir().unwrap();
        write_hello(dir.path(), 1, Some("   \t  "), &[]);
        let err = load_module_dir(dir.path(), ModuleMode::Production).unwrap_err();
        assert!(matches!(err, ModuleLoadError::Unsigned { .. }));
        // Dev still loads
        let m = load_module_dir(dir.path(), ModuleMode::Dev).unwrap();
        assert!(m.dev_unsigned);
        assert!(!m.signed);
    }

    #[test]
    fn ping_via_registry_parity() {
        let dir = tempdir().unwrap();
        write_hello(dir.path(), 1, Some("sig"), &[]);
        let m = load_module_dir(dir.path(), ModuleMode::Production).unwrap();
        install_modules(vec![m]);
        let v = call_module_rpc("ModuleHello", "Ping", &json!({}))
            .unwrap()
            .unwrap();
        assert_eq!(v["ok"], true);
        assert_eq!(v["message"], "pong");
        assert_eq!(
            module_mcp_tool_to_rpc("module_hello_ping")
                .as_ref()
                .map(|(s, m)| (s.as_str(), m.as_str())),
            Some(("ModuleHello", "Ping"))
        );
        clear_modules();
    }

    #[test]
    fn module_ts_extension_emits_callrpc() {
        let dir = tempdir().unwrap();
        write_hello(dir.path(), 1, Some("sig"), &[]);
        let m = load_module_dir(dir.path(), ModuleMode::Production).unwrap();
        let ts = generate_module_ts_extension(&[m]);
        assert!(ts.contains("export async function moduleHelloPing"));
        assert!(ts.contains("callRpc('ModuleHello', 'Ping'"));
    }
}
