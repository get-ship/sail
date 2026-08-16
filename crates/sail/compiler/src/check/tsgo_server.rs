//! Long-lived `tsgo --lsp` used by the type gate.
//!
//! One language-service process per `(TypeEnv, project)` so a cache miss is
//! `didChange` + pull diagnostics, not a new `tsgo` spawn. The product gate
//! stays OnSave; this file only changes the invocation shape.
//!
//! Kind sessions initialize at the real project root so opened files attach
//! to the on-disk kind tsconfig.

use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{Value, json};

use crate::ambients::TypeEnv;
use crate::check::tscheck::{TsDiag, TsgoTool};

static LSP_DISABLED: AtomicBool = AtomicBool::new(false);
static REUSE_HITS: AtomicU64 = AtomicU64::new(0);

/// How many type-gate checks reused an already-live tsgo program.
pub fn reuse_hits() -> u64 {
    REUSE_HITS.load(Ordering::Relaxed)
}

/// Drop every live server. Measurement-only (true cold after a corpus walk).
pub fn reset() {
    if let Ok(mut pool) = harness_pool().lock() {
        pool.sessions.clear();
    }
    if let Ok(mut pool) = project_pool().lock() {
        pool.sessions.clear();
    }
}

pub fn lsp_is_disabled() -> bool {
    LSP_DISABLED.load(Ordering::Relaxed)
}

// ── LSP client ─────────────────────────────────────────────────────────────

struct TsgoLsp {
    child: Child,
    stdin: ChildStdin,
    inbound: Receiver<Value>,
    next_id: AtomicU64,
    open: HashMap<String, i32>,
}

impl TsgoLsp {
    fn spawn(tool: &TsgoTool, root: &Path) -> std::io::Result<Self> {
        super::note_tsgo_spawn();
        let mut cmd = Command::new(&tool.program);
        if let Some(js) = &tool.js_entry {
            cmd.arg(js);
        }
        let mut child = cmd
            .args(["--lsp", "-stdio"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .current_dir(root)
            .spawn()?;

        let stdin = child.stdin.take().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::BrokenPipe, "tsgo stdin missing")
        })?;
        let stdout = child.stdout.take().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::BrokenPipe, "tsgo stdout missing")
        })?;

        let (tx, rx) = mpsc::channel::<Value>();
        thread::Builder::new()
            .name("sail-tsgo-lsp-reader".into())
            .spawn(move || {
                let mut reader = BufReader::new(stdout);
                while let Ok(msg) = read_message(&mut reader) {
                    if tx.send(msg).is_err() {
                        break;
                    }
                }
            })
            .ok();

        if let Some(stderr) = child.stderr.take() {
            thread::Builder::new()
                .name("sail-tsgo-lsp-stderr".into())
                .spawn(move || {
                    let mut r = BufReader::new(stderr);
                    let mut line = String::new();
                    while r.read_line(&mut line).ok().filter(|&n| n > 0).is_some() {
                        line.clear();
                    }
                })
                .ok();
        }

        let mut client = Self {
            child,
            stdin,
            inbound: rx,
            next_id: AtomicU64::new(1),
            open: HashMap::new(),
        };

        let root_uri = path_to_uri(root);
        let init = client.request(
            "initialize",
            json!({
                "processId": std::process::id(),
                "clientInfo": { "name": "sail-type-gate", "version": "0.1.0" },
                "rootUri": root_uri,
                "rootPath": root.to_string_lossy(),
                "capabilities": {
                    "textDocument": {
                        "publishDiagnostics": {},
                        "diagnostic": {}
                    },
                    "workspace": {
                        "configuration": true,
                        "workspaceFolders": true,
                        "didChangeWatchedFiles": { "dynamicRegistration": false },
                        "diagnostics": {}
                    }
                },
                "workspaceFolders": [{ "uri": root_uri, "name": "sail-type-gate" }]
            }),
            Duration::from_secs(15),
        )?;
        if init.get("error").is_some() {
            return Err(std::io::Error::other(format!(
                "tsgo initialize error: {init}"
            )));
        }
        client.notify("initialized", json!({}))?;
        client.drain(Duration::from_millis(80));
        Ok(client)
    }

    fn next_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    fn write_message(&mut self, msg: &Value) -> std::io::Result<()> {
        let body = serde_json::to_vec(msg)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        write!(self.stdin, "Content-Length: {}\r\n\r\n", body.len())?;
        self.stdin.write_all(&body)?;
        self.stdin.flush()
    }

    fn notify(&mut self, method: &str, params: Value) -> std::io::Result<()> {
        self.write_message(&json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        }))
    }

    fn reply(&mut self, id: &Value, result: Value) -> std::io::Result<()> {
        self.write_message(&json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": result
        }))
    }

    fn handle_server_msg(&mut self, msg: &Value) -> std::io::Result<()> {
        let Some(method) = msg.get("method").and_then(|m| m.as_str()) else {
            return Ok(());
        };
        let Some(id) = msg.get("id") else {
            return Ok(());
        };
        match method {
            "workspace/configuration" => {
                let items = msg
                    .get("params")
                    .and_then(|p| p.get("items"))
                    .and_then(|i| i.as_array())
                    .map(|a| a.len())
                    .unwrap_or(1);
                let result: Vec<Value> = (0..items).map(|_| json!({})).collect();
                self.reply(id, Value::Array(result))?;
            }
            "client/registerCapability"
            | "client/unregisterCapability"
            | "window/workDoneProgress/create" => {
                self.reply(id, Value::Null)?;
            }
            _ => {
                let _ = self.reply(id, Value::Null);
            }
        }
        Ok(())
    }

    fn drain(&mut self, budget: Duration) {
        let deadline = Instant::now() + budget;
        while Instant::now() < deadline {
            match self.inbound.recv_timeout(Duration::from_millis(10)) {
                Ok(msg) => {
                    if msg.get("id").is_some() && msg.get("method").is_some() {
                        let _ = self.handle_server_msg(&msg);
                    }
                }
                Err(RecvTimeoutError::Timeout) => break,
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }
    }

    fn request(
        &mut self,
        method: &str,
        params: Value,
        timeout: Duration,
    ) -> std::io::Result<Value> {
        let id = self.next_id();
        self.write_message(&json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        }))?;
        let deadline = Instant::now() + timeout;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    format!("tsgo request {method} timed out"),
                ));
            }
            match self.inbound.recv_timeout(remaining) {
                Ok(msg) => {
                    if msg.get("method").is_some() && msg.get("id").is_some() {
                        let _ = self.handle_server_msg(&msg);
                        continue;
                    }
                    let msg_id = msg.get("id").and_then(|v| {
                        v.as_u64()
                            .or_else(|| v.as_i64().map(|i| i as u64))
                            .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
                    });
                    if msg_id == Some(id) {
                        if let Some(err) = msg.get("error") {
                            return Err(std::io::Error::other(format!(
                                "tsgo {method} error: {err}"
                            )));
                        }
                        return Ok(msg);
                    }
                }
                Err(RecvTimeoutError::Timeout) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::TimedOut,
                        format!("tsgo request {method} timed out"),
                    ));
                }
                Err(RecvTimeoutError::Disconnected) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::BrokenPipe,
                        format!("tsgo reader disconnected during {method}"),
                    ));
                }
            }
        }
    }

    fn sync_uri(&mut self, uri: &str, text: &str) -> std::io::Result<()> {
        if let Some(ver) = self.open.get_mut(uri) {
            *ver += 1;
            let version = *ver;
            self.notify(
                "textDocument/didChange",
                json!({
                    "textDocument": { "uri": uri, "version": version },
                    "contentChanges": [{ "text": text }]
                }),
            )?;
        } else {
            self.open.insert(uri.to_string(), 1);
            self.notify(
                "textDocument/didOpen",
                json!({
                    "textDocument": {
                        "uri": uri,
                        "languageId": "typescript",
                        "version": 1,
                        "text": text
                    }
                }),
            )?;
        }
        Ok(())
    }

    fn close_uri(&mut self, uri: &str) -> std::io::Result<()> {
        if self.open.remove(uri).is_none() {
            return Ok(());
        }
        self.notify(
            "textDocument/didClose",
            json!({ "textDocument": { "uri": uri } }),
        )
    }

    fn pull_diagnostics(&mut self, uri: &str) -> std::io::Result<Vec<TsDiag>> {
        let resp = crate::profile::span("tsgo", || {
            self.request(
                "textDocument/diagnostic",
                json!({ "textDocument": { "uri": uri } }),
                Duration::from_secs(15),
            )
        })?;
        let result = resp.get("result").cloned().unwrap_or(Value::Null);
        Ok(diags_from_report(&result, uri))
    }

    fn notify_file_changed(&mut self, path: &Path) -> std::io::Result<()> {
        self.notify(
            "workspace/didChangeWatchedFiles",
            json!({
                "changes": [{ "uri": path_to_uri(path), "type": 2 }]
            }),
        )
    }
}

impl Drop for TsgoLsp {
    fn drop(&mut self) {
        let _ = self.notify("exit", json!({}));
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn diags_from_report(report: &Value, fallback_uri: &str) -> Vec<TsDiag> {
    let items = report
        .get("items")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let mut out = Vec::new();
    for item in items {
        if let Some(d) = lsp_item_to_diag(&item, fallback_uri) {
            out.push(d);
        }
    }
    out
}

fn lsp_item_to_diag(item: &Value, fallback_uri: &str) -> Option<TsDiag> {
    let severity = item.get("severity").and_then(|v| v.as_u64()).unwrap_or(1);
    // 1 = Error, 2 = Warning. Hints/info are not product-red.
    if severity > 2 {
        return None;
    }
    let range = item.get("range")?;
    let start = range.get("start")?;
    let line = start.get("line")?.as_u64()? as usize + 1;
    let col = start.get("character")?.as_u64()? as usize + 1;
    let message = item.get("message")?.as_str()?.to_string();
    let code = match item.get("code") {
        Some(Value::String(s)) => {
            if s.starts_with("TS") {
                s.clone()
            } else {
                format!("TS{s}")
            }
        }
        Some(Value::Number(n)) => format!("TS{n}"),
        _ => return None,
    };
    if !code.starts_with("TS") {
        return None;
    }
    let file = item
        .get("data")
        .and_then(|v| v.get("uri"))
        .and_then(|v| v.as_str())
        .unwrap_or(fallback_uri);
    let file = uri_to_path_string(file);
    Some(TsDiag {
        file,
        line,
        col,
        code,
        message,
    })
}

fn read_message<R: Read>(reader: &mut BufReader<R>) -> std::io::Result<Value> {
    let mut content_length: Option<usize> = None;
    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line)?;
        if n == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "tsgo stdout closed",
            ));
        }
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            break;
        }
        let lower = trimmed.to_ascii_lowercase();
        if let Some(num) = lower.strip_prefix("content-length:") {
            content_length = Some(
                num.trim()
                    .parse()
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?,
            );
        }
    }
    let len = content_length.ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "missing Content-Length")
    })?;
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)?;
    serde_json::from_slice(&buf)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

pub fn path_to_uri(path: &Path) -> String {
    let abs = if path.exists() {
        path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
    } else {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
                .map(|c| c.join(path))
                .unwrap_or_else(|_| path.to_path_buf())
        }
    };
    let s = abs.to_string_lossy().replace('\\', "/");
    if s.starts_with('/') {
        format!("file://{s}")
    } else {
        format!("file:///{s}")
    }
}

fn uri_to_path_string(uri: &str) -> String {
    let rest = uri.strip_prefix("file://").unwrap_or(uri);
    let rest = rest.strip_prefix("localhost").unwrap_or(rest);
    rest.to_string()
}

// ── region harness sessions ────────────────────────────────────────────────

#[derive(Clone, PartialEq, Eq, Hash)]
struct HarnessKey {
    env: TypeEnv,
    project: String,
}

struct HarnessSession {
    work_dir: PathBuf,
    client: Option<TsgoLsp>,
    tsconfig: String,
    started: bool,
}

struct HarnessPool {
    sessions: HashMap<HarnessKey, HarnessSession>,
}

fn harness_pool() -> &'static Mutex<HarnessPool> {
    static POOL: OnceLock<Mutex<HarnessPool>> = OnceLock::new();
    POOL.get_or_init(|| {
        Mutex::new(HarnessPool {
            sessions: HashMap::new(),
        })
    })
}

fn lock_harness() -> std::sync::MutexGuard<'static, HarnessPool> {
    harness_pool().lock().unwrap_or_else(|e| e.into_inner())
}

fn harness_key(env: TypeEnv, project_root: Option<&Path>) -> HarnessKey {
    let project = project_root
        .map(|p| {
            p.canonicalize()
                .unwrap_or_else(|_| p.to_path_buf())
                .to_string_lossy()
                .into_owned()
        })
        .unwrap_or_else(|| "<none>".into());
    HarnessKey { env, project }
}

fn new_work_dir(prefix: &str) -> PathBuf {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "{prefix}-{}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0),
        n
    ));
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// Prepare (or reuse) the persistent harness dir for this env/project.
pub fn harness_work_dir(env: TypeEnv, project_root: Option<&Path>) -> PathBuf {
    let key = harness_key(env, project_root);
    let mut pool = lock_harness();
    if let Some(s) = pool.sessions.get(&key) {
        return s.work_dir.clone();
    }
    evict_if_needed(&mut pool.sessions, 8);
    let work_dir = new_work_dir("sail-type-gate-reuse");
    pool.sessions.insert(
        key,
        HarnessSession {
            work_dir: work_dir.clone(),
            client: None,
            tsconfig: String::new(),
            started: false,
        },
    );
    work_dir
}

/// Sync `region_rel` and pull diagnostics from the live program.
pub fn diagnose_region(
    tool: &TsgoTool,
    env: TypeEnv,
    project_root: Option<&Path>,
    work_dir: &Path,
    region_rel: &str,
    region_text: &str,
    tsconfig: &str,
    sail_stubs: &[String],
) -> std::io::Result<Vec<TsDiag>> {
    if LSP_DISABLED.load(Ordering::Relaxed) {
        return Err(std::io::Error::other("tsgo lsp disabled"));
    }
    let key = harness_key(env, project_root);
    let mut pool = lock_harness();
    let session = pool
        .sessions
        .get_mut(&key)
        .ok_or_else(|| std::io::Error::other("harness session missing"))?;

    let tsconfig_path = work_dir.join("tsconfig.json");
    let tsconfig_changed = session.tsconfig != tsconfig;
    if tsconfig_changed {
        std::fs::write(&tsconfig_path, tsconfig)?;
        session.tsconfig = tsconfig.to_string();
    }

    if session.client.is_none() {
        match TsgoLsp::spawn(tool, work_dir) {
            Ok(c) => {
                session.client = Some(c);
                session.started = true;
            }
            Err(e) => {
                LSP_DISABLED.store(true, Ordering::Relaxed);
                return Err(e);
            }
        }
    } else if tsconfig_changed {
        if let Some(c) = session.client.as_mut() {
            let _ = c.notify_file_changed(&tsconfig_path);
        }
    }

    let region_path = work_dir.join(region_rel);
    let uri = path_to_uri(&region_path);
    let reused = session.started && session.client.as_ref().is_some_and(|c| !c.open.is_empty());
    let client = session
        .client
        .as_mut()
        .ok_or_else(|| std::io::Error::other("tsgo lsp not started"))?;
    // Ambients written after the first spawn must enter the live program
    // (open-any `@ship` stubs, action observation, …).
    for name in [
        "sail-runtime-ambient.d.ts",
        "ship-open-stubs.d.ts",
        "action-observation.d.ts",
    ] {
        let p = work_dir.join(name);
        if p.is_file() {
            if let Ok(text) = std::fs::read_to_string(&p) {
                let _ = client.sync_uri(&path_to_uri(&p), &text);
            }
        }
    }
    // Merged `.sail` stubs must `didChange` — disk rewrite alone leaves
    // the first writer's names in the live program.
    for rel in sail_stubs {
        let p = work_dir.join(rel);
        if p.is_file() {
            if let Ok(text) = std::fs::read_to_string(&p) {
                let _ = client.sync_uri(&path_to_uri(&p), &text);
            }
        }
    }
    client.sync_uri(&uri, region_text)?;
    let diags = client.pull_diagnostics(&uri)?;
    if reused {
        REUSE_HITS.fetch_add(1, Ordering::Relaxed);
    }
    Ok(diags)
}

fn evict_if_needed<K: Clone + Eq + std::hash::Hash, V>(map: &mut HashMap<K, V>, cap: usize) {
    if map.len() < cap {
        return;
    }
    // Drop an arbitrary entry so long-lived processes cannot retain every
    // temp-dir fixture session from cargo test.
    if let Some(k) = map.keys().next().cloned() {
        map.remove(&k);
    }
}

// ── project sessions ───────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Eq, Hash)]
struct ProjectKey {
    root: String,
    env: TypeEnv,
}

struct ProjectSession {
    work_dir: PathBuf,
    client: Option<TsgoLsp>,
    /// path display → content hash
    files: HashMap<String, u64>,
    tsconfig: String,
    last_diags: Vec<TsDiag>,
}

struct ProjectPool {
    sessions: HashMap<ProjectKey, ProjectSession>,
}

fn project_pool() -> &'static Mutex<ProjectPool> {
    static POOL: OnceLock<Mutex<ProjectPool>> = OnceLock::new();
    POOL.get_or_init(|| {
        Mutex::new(ProjectPool {
            sessions: HashMap::new(),
        })
    })
}

fn lock_project() -> std::sync::MutexGuard<'static, ProjectPool> {
    project_pool().lock().unwrap_or_else(|e| e.into_inner())
}

fn project_key(project_root: &Path, env: TypeEnv) -> ProjectKey {
    let root = project_root
        .canonicalize()
        .unwrap_or_else(|_| project_root.to_path_buf())
        .to_string_lossy()
        .into_owned();
    ProjectKey { root, env }
}

/// `in_place` keeps the language service rooted at the real project so
/// opened product files attach to the on-disk kind tsconfig.
pub fn project_work_dir(project_root: &Path, env: TypeEnv, in_place: bool) -> PathBuf {
    let key = project_key(project_root, env);
    let mut pool = lock_project();
    let in_place_dir = project_root
        .canonicalize()
        .unwrap_or_else(|_| project_root.to_path_buf());
    if let Some(s) = pool.sessions.get(&key) {
        let rooted_here = s.work_dir == in_place_dir;
        if in_place == rooted_here {
            return s.work_dir.clone();
        }
        // Kind vs mirror root must not share a session — the live program
        // would keep the other project's include/paths.
        pool.sessions.remove(&key);
    }
    evict_if_needed(&mut pool.sessions, 8);
    let work_dir = if in_place {
        in_place_dir
    } else {
        new_work_dir(&format!("sail-proj-gate-{}", env_tag(env)))
    };
    pool.sessions.insert(
        key,
        ProjectSession {
            work_dir: work_dir.clone(),
            client: None,
            files: HashMap::new(),
            tsconfig: String::new(),
            last_diags: Vec::new(),
        },
    );
    work_dir
}

fn env_tag(env: TypeEnv) -> &'static str {
    match env {
        TypeEnv::Sail => "sail",
        TypeEnv::Action => "action",
        TypeEnv::Framework => "framework",
    }
}

fn hash_bytes(s: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}

/// Sync product files into a live program and pull diagnostics.
///
/// `entries` are `(absolute path to open, source text, diagnostic file label)`.
/// The label is what `TsDiag.file` should carry for product mapping.
///
/// When `persist_tsconfig` is false the on-disk kind config is the program
/// identity — do not write a wrapper `tsconfig.json` (that would replace
/// the solution entry at a real project root).
pub fn diagnose_project_files(
    tool: &TsgoTool,
    project_root: &Path,
    env: TypeEnv,
    work_dir: &Path,
    tsconfig: &str,
    entries: &[(PathBuf, String, String)],
    persist_tsconfig: bool,
) -> std::io::Result<Vec<TsDiag>> {
    if LSP_DISABLED.load(Ordering::Relaxed) {
        return Err(std::io::Error::other("tsgo lsp disabled"));
    }
    let key = project_key(project_root, env);
    let mut pool = lock_project();
    let session = pool
        .sessions
        .get_mut(&key)
        .ok_or_else(|| std::io::Error::other("project session missing"))?;

    let tsconfig_path = work_dir.join("tsconfig.json");
    let tsconfig_changed = session.tsconfig != tsconfig;
    if tsconfig_changed {
        if persist_tsconfig {
            std::fs::write(&tsconfig_path, tsconfig)?;
        }
        session.tsconfig = tsconfig.to_string();
    }

    if session.client.is_none() {
        match TsgoLsp::spawn(tool, work_dir) {
            Ok(c) => session.client = Some(c),
            Err(e) => {
                LSP_DISABLED.store(true, Ordering::Relaxed);
                return Err(e);
            }
        }
    } else if tsconfig_changed {
        if let Some(c) = session.client.as_mut() {
            if persist_tsconfig {
                let _ = c.notify_file_changed(&tsconfig_path);
            } else {
                let rel = match env {
                    TypeEnv::Sail => crate::ambients::PROJECT_TSCONFIG_SAIL_REL,
                    TypeEnv::Action => crate::ambients::PROJECT_TSCONFIG_ACTION_REL,
                    TypeEnv::Framework => "tsconfig.json",
                };
                let _ = c.notify_file_changed(&project_root.join(rel));
            }
        }
    }

    let reused = session.client.as_ref().is_some_and(|c| !c.open.is_empty());
    let client = session
        .client
        .as_mut()
        .ok_or_else(|| std::io::Error::other("tsgo lsp not started"))?;

    let mut want: HashMap<String, (PathBuf, String, String, u64)> = HashMap::new();
    let mut product_uris: HashSet<String> = HashSet::new();
    for (path, src, label) in entries {
        let uri = path_to_uri(path);
        product_uris.insert(uri.clone());
        want.insert(
            uri,
            (path.clone(), src.clone(), label.clone(), hash_bytes(src)),
        );
    }

    // Kind `files` siblings share identity/bytes with product entries so
    // last_diags cannot survive an emptied css-modules.d.ts. Action-only
    // observation stays off the Pure program (tsc -p kind omits it).
    let sail_dir = project_root.join(crate::ambients::PROJECT_SAIL_DIR_REL);
    if sail_dir.is_dir() {
        let mut names = vec!["types.d.ts", "css-modules.d.ts"];
        if env == TypeEnv::Action {
            names.push("action-observation.d.ts");
        }
        for name in names {
            let p = sail_dir.join(name);
            if p.is_file() {
                if let Ok(text) = std::fs::read_to_string(&p) {
                    let uri = path_to_uri(&p);
                    want.insert(uri, (p, text.clone(), name.to_string(), hash_bytes(&text)));
                }
            }
        }
    }

    let stale: Vec<String> = session
        .files
        .keys()
        .filter(|u| !want.contains_key(*u))
        .cloned()
        .collect();
    for uri in &stale {
        let _ = client.close_uri(uri);
        session.files.remove(uri);
    }

    let mut changed_uris = Vec::new();
    for (uri, (_path, src, _label, hash)) in &want {
        let prev = session.files.get(uri).copied();
        if prev != Some(*hash) {
            client.sync_uri(uri, src)?;
            session.files.insert(uri.clone(), *hash);
            changed_uris.push(uri.clone());
        }
    }

    if reused && !tsconfig_changed && changed_uris.is_empty() && stale.is_empty() {
        REUSE_HITS.fetch_add(1, Ordering::Relaxed);
        return Ok(session.last_diags.clone());
    }

    // Dependents of a changed product or ambient still need a pull.
    let pull: Vec<String> = product_uris.into_iter().collect();
    let _ = changed_uris;

    let mut out = Vec::new();
    for uri in pull {
        let diags = client.pull_diagnostics(&uri)?;
        let label = want
            .get(&uri)
            .map(|e| e.2.clone())
            .unwrap_or_else(|| uri_to_path_string(&uri));
        for mut d in diags {
            d.file = label.clone();
            out.push(d);
        }
    }

    if reused {
        REUSE_HITS.fetch_add(1, Ordering::Relaxed);
    }
    session.last_diags = out.clone();
    Ok(out)
}
