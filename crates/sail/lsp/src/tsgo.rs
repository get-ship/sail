//! tsgo LSP sidecar — spawn, discover, and proxy position-based requests.
//!
//! Binary discovery is `sail_compiler::ts_native` (peer dir / XDG / env).
//! `workspace_hint` is for `write_tsconfig` paths and `@/` homes, not the ELF.
//!
//! Speaks standard LSP over stdio (`tsgo --lsp -stdio`). Diagnostics use the
//! pull model (`textDocument/diagnostic`).

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{Value, json};

use crate::virtual_ts::{VirtualFile, VirtualTs, lsp_position_to_offset, offset_to_lsp_position};

static DEGRADE_LOGGED: AtomicBool = AtomicBool::new(false);
static TSGO_AVAILABLE: AtomicBool = AtomicBool::new(false);

/// Discover the native TypeScript sidecar. `start` is unused (workspace hint
/// is for tsconfig / `@/` only).
pub fn discover_tsgo(_start: Option<&Path>) -> Option<PathBuf> {
    match sail_compiler::ts_native::resolve_ts_native() {
        Some(tool) => {
            TSGO_AVAILABLE.store(true, Ordering::SeqCst);
            Some(tool.program)
        }
        None => {
            TSGO_AVAILABLE.store(false, Ordering::SeqCst);
            None
        }
    }
}

/// Log the degrade message once. Returns true if this call logged it.
pub fn log_degrade_once() -> bool {
    if DEGRADE_LOGGED
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
    {
        eprintln!(
            "sail-lsp: TypeScript 7 native compiler not found next to sail \
             (dirname(sail)/sail-ts-native/). Controller and {{expr}} types are off; \
             structure stays on."
        );
        true
    } else {
        false
    }
}

#[cfg(test)]
pub fn reset_discovery_state_for_tests() {
    DEGRADE_LOGGED.store(false, Ordering::SeqCst);
    TSGO_AVAILABLE.store(false, Ordering::SeqCst);
}

#[allow(dead_code)]
pub fn tsgo_is_available() -> bool {
    TSGO_AVAILABLE.load(Ordering::SeqCst)
}

// ── LSP client ─────────────────────────────────────────────────────────────

/// A live tsgo language-server subprocess.
pub struct TsgoClient {
    child: Child,
    stdin: ChildStdin,
    /// All inbound messages from the reader thread.
    inbound: Receiver<Value>,
    next_id: AtomicU64,
    /// uri → version for open documents.
    open: HashMap<String, i32>,
    #[allow(dead_code)]
    root: PathBuf,
}

impl TsgoClient {
    pub fn spawn(tsgo: &Path, root: &Path) -> anyhow::Result<Self> {
        let mut child = Command::new(tsgo)
            .args(["--lsp", "-stdio"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .current_dir(root)
            .spawn()
            .map_err(|e| anyhow::anyhow!("spawn tsgo: {e}"))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow::anyhow!("tsgo stdin missing"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow::anyhow!("tsgo stdout missing"))?;

        let (tx, rx) = mpsc::channel::<Value>();
        thread::Builder::new()
            .name("tsgo-lsp-reader".into())
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
                .name("tsgo-lsp-stderr".into())
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
            root: root.to_path_buf(),
        };

        let root_uri = path_to_uri(root);
        let init = client.request(
            "initialize",
            json!({
                "processId": std::process::id(),
                "clientInfo": { "name": "sail-lsp", "version": "0.1.0" },
                "rootUri": root_uri,
                "capabilities": {
                    "textDocument": {
                        "hover": { "contentFormat": ["markdown", "plaintext"] },
                        "completion": { "completionItem": { "snippetSupport": false } },
                        "definition": { "linkSupport": false },
                        "publishDiagnostics": {},
                        "diagnostic": {}
                    },
                    "workspace": { "configuration": true, "workspaceFolders": true }
                },
                "workspaceFolders": [{ "uri": root_uri, "name": "sail-virtual" }]
            }),
            Duration::from_secs(15),
        )?;
        if init.get("error").is_some() {
            anyhow::bail!("tsgo initialize error: {init}");
        }
        client.notify("initialized", json!({}))?;
        // Drain any immediate server requests (registerCapability, etc.).
        client.drain_server_requests(Duration::from_millis(200));
        Ok(client)
    }

    fn next_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }

    fn write_message(&mut self, msg: &Value) -> anyhow::Result<()> {
        let body = serde_json::to_vec(msg)?;
        write!(self.stdin, "Content-Length: {}\r\n\r\n", body.len())?;
        self.stdin.write_all(&body)?;
        self.stdin.flush()?;
        Ok(())
    }

    fn notify(&mut self, method: &str, params: Value) -> anyhow::Result<()> {
        self.write_message(&json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        }))
    }

    /// Reply to a server→client request (id may be string or number).
    fn reply(&mut self, id: &Value, result: Value) -> anyhow::Result<()> {
        self.write_message(&json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": result
        }))
    }

    fn handle_server_msg(&mut self, msg: &Value) -> anyhow::Result<()> {
        let Some(method) = msg.get("method").and_then(|m| m.as_str()) else {
            return Ok(());
        };
        let Some(id) = msg.get("id") else {
            return Ok(()); // notification
        };
        match method {
            "workspace/configuration" => {
                // Return one empty config object per requested item.
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
                // Best-effort null result so the server does not stall.
                let _ = self.reply(id, Value::Null);
            }
        }
        Ok(())
    }

    fn drain_server_requests(&mut self, budget: Duration) {
        let deadline = Instant::now() + budget;
        while Instant::now() < deadline {
            match self.inbound.recv_timeout(Duration::from_millis(20)) {
                Ok(msg) => {
                    if msg.get("id").is_some() && msg.get("method").is_some() {
                        let _ = self.handle_server_msg(&msg);
                    }
                    // Drop notifications and late responses.
                }
                Err(RecvTimeoutError::Timeout) => break,
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }
    }

    fn request(&mut self, method: &str, params: Value, timeout: Duration) -> anyhow::Result<Value> {
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
                anyhow::bail!("tsgo request {method} timed out");
            }
            match self.inbound.recv_timeout(remaining) {
                Ok(msg) => {
                    // Server request interleaved.
                    if msg.get("method").is_some() && msg.get("id").is_some() {
                        let _ = self.handle_server_msg(&msg);
                        continue;
                    }
                    // Our response?
                    let msg_id = msg.get("id").and_then(|v| {
                        v.as_u64()
                            .or_else(|| v.as_i64().map(|i| i as u64))
                            .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
                    });
                    if msg_id == Some(id) {
                        return Ok(msg);
                    }
                    // Unrelated response/notification — ignore.
                }
                Err(RecvTimeoutError::Timeout) => {
                    anyhow::bail!("tsgo request {method} timed out")
                }
                Err(RecvTimeoutError::Disconnected) => {
                    anyhow::bail!("tsgo reader disconnected during {method}")
                }
            }
        }
    }

    /// Open or update a virtual document.
    pub fn sync_virtual(&mut self, uri: &str, text: &str) -> anyhow::Result<()> {
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
        // Let publishDiagnostics / registerCapability drain.
        self.drain_server_requests(Duration::from_millis(50));
        Ok(())
    }

    pub fn hover(&mut self, uri: &str, line: u32, character: u32) -> anyhow::Result<Option<Value>> {
        let resp = self.request(
            "textDocument/hover",
            json!({
                "textDocument": { "uri": uri },
                "position": { "line": line, "character": character }
            }),
            Duration::from_secs(8),
        )?;
        Ok(resp.get("result").cloned().filter(|v| !v.is_null()))
    }

    pub fn completion(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> anyhow::Result<Option<Value>> {
        let resp = self.request(
            "textDocument/completion",
            json!({
                "textDocument": { "uri": uri },
                "position": { "line": line, "character": character }
            }),
            Duration::from_secs(8),
        )?;
        Ok(resp.get("result").cloned().filter(|v| !v.is_null()))
    }

    pub fn definition(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> anyhow::Result<Option<Value>> {
        let resp = self.request(
            "textDocument/definition",
            json!({
                "textDocument": { "uri": uri },
                "position": { "line": line, "character": character }
            }),
            Duration::from_secs(8),
        )?;
        Ok(resp.get("result").cloned().filter(|v| !v.is_null()))
    }

    pub fn diagnostics(&mut self, uri: &str) -> anyhow::Result<Vec<Value>> {
        let resp = self.request(
            "textDocument/diagnostic",
            json!({ "textDocument": { "uri": uri } }),
            Duration::from_secs(15),
        )?;
        let result = resp.get("result").cloned().unwrap_or(Value::Null);
        if let Some(items) = result.get("items").and_then(|v| v.as_array()) {
            return Ok(items.clone());
        }
        Ok(Vec::new())
    }

    pub fn shutdown(&mut self) {
        // Best-effort LSP shutdown; never block forever on a stubborn tsgo.
        let _ = self.request("shutdown", Value::Null, Duration::from_millis(500));
        let _ = self.notify("exit", Value::Null);
        // Force-kill if still alive — `wait` alone hangs when tsgo ignores exit.
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Drop for TsgoClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
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
        path.to_path_buf()
    };
    lsp_types::Url::from_file_path(&abs)
        .map(|u| u.to_string())
        .unwrap_or_else(|_| format!("file://{}", abs.display()))
}

// ── Session ────────────────────────────────────────────────────────────────

pub struct TsgoSession {
    client: Option<TsgoClient>,
    work_dir: PathBuf,
    virtuals: HashMap<String, VirtualTs>,
    tool: Option<sail_compiler::ts_native::TsNativeTool>,
}

impl TsgoSession {
    fn new_work_dir() -> PathBuf {
        // Unique per session: parallel tests in one process can share a
        // millisecond clock tick, and Drop removes the work_dir — colliding
        // names delete another session's virtual project mid-flight.
        static WORK_DIR_SEQ: AtomicU64 = AtomicU64::new(0);
        let work_dir = std::env::temp_dir().join(format!(
            "sail-lsp-tsgo-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0),
            WORK_DIR_SEQ.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::create_dir_all(&work_dir);
        work_dir
    }

    /// Dead sidecar without consulting resolve. The only in-process disable.
    pub fn structure_only() -> Self {
        Self {
            client: None,
            work_dir: Self::new_work_dir(),
            virtuals: HashMap::new(),
            tool: None,
        }
    }

    pub fn start(workspace_hint: Option<&Path>) -> Self {
        let work_dir = Self::new_work_dir();
        let tool = sail_compiler::ts_native::resolve_ts_native();
        let client = match &tool {
            Some(tool) => {
                write_tsconfig(&work_dir, workspace_hint);
                match TsgoClient::spawn(&tool.program, &work_dir) {
                    Ok(c) => {
                        eprintln!(
                            "sail-lsp: typescript {} live ({}) {}",
                            tool.version,
                            tool.source.as_str(),
                            tool.program.display()
                        );
                        Some(c)
                    }
                    Err(e) => {
                        eprintln!("sail-lsp: failed to start typescript sidecar ({e}) — degrading");
                        let _ = log_degrade_once();
                        None
                    }
                }
            }
            None => {
                let _ = log_degrade_once();
                None
            }
        };

        Self {
            client,
            work_dir,
            virtuals: HashMap::new(),
            tool,
        }
    }

    pub fn is_live(&self) -> bool {
        self.client.is_some()
    }

    pub fn probed_version(&self) -> Option<&str> {
        self.tool.as_ref().map(|t| t.version.as_str())
    }

    pub fn tool_source_str(&self) -> Option<&'static str> {
        self.tool.as_ref().map(|t| t.source.as_str())
    }

    pub fn virtual_for(&self, sail_uri: &str) -> Option<&VirtualTs> {
        self.virtuals.get(sail_uri)
    }

    pub fn on_sail_change(&mut self, sail_uri: &str, sail_text: &str) {
        let Some(virt) = crate::virtual_ts::build_virtual_ts(sail_text) else {
            self.virtuals.remove(sail_uri);
            return;
        };
        if let Some(client) = self.client.as_mut() {
            // Materialize controller as `__ctrl__.ts` relative import target
            // from the view file, and the view itself under a stable name.
            // Write first, then mint URIs — `path_to_uri` canonicalizes when
            // the path exists, so didOpen and diagnostic pull share one URI.
            let ctrl_path = self
                .work_dir
                .join(format!("{}.ctrl.ts", doc_stem(sail_uri)));
            let view_path = self
                .work_dir
                .join(format!("{}.view.ts", doc_stem(sail_uri)));
            // View imports `./__ctrl__.ts` — write a symlink/copy under that name
            // per document is racy for multi-doc; rewrite import to the real
            // relative name instead.
            let mut view_text = virt.view_text.clone();
            let ctrl_name = format!("{}.ctrl.ts", doc_stem(sail_uri));
            view_text =
                view_text.replace("from \"./__ctrl__.ts\"", &format!("from \"./{ctrl_name}\""));
            // Keep a mutated copy for position mapping (import path length may differ).
            let mut virt = virt;
            // Controller lives in a temp work_dir — rewrite relative and `@/`
            // imports to absolute paths against the real project so siblings
            // and `project_map` homes resolve under tsgo.
            if let Some(sail_path) = uri_to_path(sail_uri) {
                if let Some(dir) = sail_path.parent() {
                    let original = virt.controller_text.clone();
                    let rewritten = rewrite_at_alias_imports(
                        &rewrite_relative_imports(&original, dir),
                        &sail_path,
                    );
                    if rewritten != original {
                        let delta = rewritten.len() as isize - original.len() as isize;
                        for seg in &mut virt.segments {
                            if seg.file == VirtualFile::Controller {
                                seg.virtual_end = (seg.virtual_end as isize + delta) as usize;
                                // Specifier lengths changed — drop 1:1 map.
                                seg.one_to_one = false;
                            }
                        }
                        virt.controller_text = rewritten;
                    }
                }
            }
            let _ = std::fs::write(&ctrl_path, &virt.controller_text);
            let _ = std::fs::write(&view_path, &view_text);
            let (ctrl_uri, view_uri) = virtual_uris(sail_uri, &self.work_dir);
            // Adjust view segment offsets if the import path rewrite shifted text.
            if view_text.len() != virt.view_text.len() {
                let delta = view_text.len() as isize - virt.view_text.len() as isize;
                // The import line is near the top; only segments after the
                // original import string shift. Find the import in original.
                if let Some(imp_at) = virt.view_text.find("from \"./__ctrl__.ts\"") {
                    let cut = imp_at + "from \"./__ctrl__.ts\"".len();
                    for seg in &mut virt.segments {
                        if seg.file == VirtualFile::View && seg.virtual_start >= cut {
                            seg.virtual_start = (seg.virtual_start as isize + delta) as usize;
                            seg.virtual_end = (seg.virtual_end as isize + delta) as usize;
                        }
                    }
                }
                virt.view_text = view_text;
            } else {
                virt.view_text = view_text;
            }
            if !virt.controller_text.trim().is_empty() {
                let _ = client.sync_virtual(&ctrl_uri, &virt.controller_text);
            }
            let _ = client.sync_virtual(&view_uri, &virt.view_text);
            self.virtuals.insert(sail_uri.to_string(), virt);
        } else {
            self.virtuals.insert(sail_uri.to_string(), virt);
        }
    }

    pub fn on_sail_close(&mut self, sail_uri: &str) {
        self.virtuals.remove(sail_uri);
    }

    pub fn hover_mapped(
        &mut self,
        sail_uri: &str,
        sail_text: &str,
        sail_offset: usize,
    ) -> Option<Value> {
        let virt = self.virtuals.get(sail_uri)?;
        if !virt.is_ts_region(sail_offset) {
            return None;
        }
        let (file, v_off) = virt.sail_to_virtual(sail_offset)?;
        let text = virt.text(file);
        let (line, col) = offset_to_lsp_position(text, v_off);
        let uri = file_uri(sail_uri, file, &self.work_dir);
        let client = self.client.as_mut()?;
        let mut result = client.hover(&uri, line, col).ok().flatten()?;
        map_hover_ranges(&mut result, virt, file, sail_text);
        Some(result)
    }

    pub fn completion_mapped(
        &mut self,
        sail_uri: &str,
        _sail_text: &str,
        sail_offset: usize,
    ) -> Option<Value> {
        let virt = self.virtuals.get(sail_uri)?;
        if !virt.is_ts_region(sail_offset) {
            return None;
        }
        let (file, v_off) = virt.sail_to_virtual(sail_offset)?;
        let text = virt.text(file);
        let (line, col) = offset_to_lsp_position(text, v_off);
        let uri = file_uri(sail_uri, file, &self.work_dir);
        let client = self.client.as_mut()?;
        client.completion(&uri, line, col).ok().flatten()
    }

    pub fn definition_mapped(
        &mut self,
        sail_uri: &str,
        sail_text: &str,
        sail_offset: usize,
    ) -> Option<Value> {
        let virt = self.virtuals.get(sail_uri)?;
        if !virt.is_ts_region(sail_offset) {
            return None;
        }
        let (file, v_off) = virt.sail_to_virtual(sail_offset)?;
        let text = virt.text(file);
        let (line, col) = offset_to_lsp_position(text, v_off);
        let uri = file_uri(sail_uri, file, &self.work_dir);
        let client = self.client.as_mut()?;
        let mut result = client.definition(&uri, line, col).ok().flatten()?;
        let (ctrl_uri, view_uri) = virtual_uris(sail_uri, &self.work_dir);
        map_definition_locations(&mut result, virt, sail_uri, sail_text, &ctrl_uri, &view_uri);
        Some(result)
    }

    pub fn diagnostics_mapped(
        &mut self,
        sail_uri: &str,
        sail_text: &str,
    ) -> Vec<lsp_types::Diagnostic> {
        let Some(virt) = self.virtuals.get(sail_uri).cloned() else {
            return Vec::new();
        };
        let Some(client) = self.client.as_mut() else {
            return Vec::new();
        };
        let (ctrl_uri, view_uri) = virtual_uris(sail_uri, &self.work_dir);
        let mut out = Vec::new();
        // Pull from both virtual files.
        for (file, uri) in [
            (VirtualFile::Controller, ctrl_uri.as_str()),
            (VirtualFile::View, view_uri.as_str()),
        ] {
            if file == VirtualFile::Controller && virt.controller_text.trim().is_empty() {
                continue;
            }
            let Ok(items) = client.diagnostics(uri) else {
                continue;
            };
            for item in items {
                if let Some(d) = map_diagnostic(&item, &virt, file, sail_text) {
                    out.push(d);
                }
            }
        }
        out
    }
}

impl Drop for TsgoSession {
    fn drop(&mut self) {
        if let Some(mut c) = self.client.take() {
            c.shutdown();
        }
        let _ = std::fs::remove_dir_all(&self.work_dir);
    }
}

fn doc_stem(sail_uri: &str) -> String {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    sail_uri.hash(&mut h);
    format!("d{:x}", h.finish())
}

fn virtual_uris(sail_uri: &str, work_dir: &Path) -> (String, String) {
    let stem = doc_stem(sail_uri);
    (
        path_to_uri(&work_dir.join(format!("{stem}.ctrl.ts"))),
        path_to_uri(&work_dir.join(format!("{stem}.view.ts"))),
    )
}

fn file_uri(sail_uri: &str, file: VirtualFile, work_dir: &Path) -> String {
    let (c, v) = virtual_uris(sail_uri, work_dir);
    match file {
        VirtualFile::Controller => c,
        VirtualFile::View => v,
    }
}

/// Framework import specs → monorepo-relative entry paths.
/// Specs are **import forms** (not always package.json `name`): e.g.
/// `@ship/deck/router` is the authored alias for `packages/deck/router`
/// (`@ship/deck-router` on npm).
const FRAMEWORK_TS_PATHS: &[(&str, &str)] = &[
    ("@ship/sail-runtime", "packages/sail/runtime/src/index.ts"),
    (
        "@ship/sail-runtime/internal",
        "packages/sail/runtime/src/internal.ts",
    ),
    ("@ship/deck/router", "packages/deck/router/src/index.ts"),
    (
        "@ship/sail-studio-shell",
        "packages/sail/studio-shell/src/index.ts",
    ),
    (
        "@ship/sail-studio-client",
        "packages/sail/studio-client/src/index.ts",
    ),
    (
        "@ship/sail-studio-bridge",
        "packages/sail/studio-bridge/src/index.ts",
    ),
    ("@ship/sail-dom", "packages/sail/dom/src/index.ts"),
];

const AMBIENTS_RS: &str = include_str!("../../compiler/src/ambients.rs");

fn import_type_names(src: &str, specifier: &str) -> std::collections::BTreeSet<String> {
    let needle = format!("from '{specifier}'");
    let mut out = std::collections::BTreeSet::new();
    let mut rest = src;
    while let Some(idx) = rest.find(&needle) {
        let before = &rest[..idx];
        if let Some(brace) = before.rfind('{') {
            for part in before[brace + 1..].split(',') {
                let name = part
                    .lines()
                    .last()
                    .unwrap_or(part)
                    .trim()
                    .trim_end_matches(',')
                    .trim();
                if !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                    out.insert(name.to_string());
                }
            }
        }
        rest = &rest[idx + needle.len()..];
    }
    out
}

fn import_call_names(src: &str, specifier: &str) -> std::collections::BTreeSet<String> {
    let needle = format!("import('{specifier}').");
    let mut out = std::collections::BTreeSet::new();
    let mut rest = src;
    while let Some(idx) = rest.find(&needle) {
        let after = &rest[idx + needle.len()..];
        let name: String = after
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if !name.is_empty() {
            out.insert(name);
        }
        rest = &rest[idx + needle.len()..];
    }
    out
}

fn runtime_stub_names() -> std::collections::BTreeSet<String> {
    let mut names = import_type_names(AMBIENTS_RS, "@ship/sail-runtime");
    names.extend(import_call_names(AMBIENTS_RS, "@ship/sail-runtime"));
    names.extend(import_type_names(
        AMBIENTS_RS,
        "@ship/sail-runtime/internal",
    ));
    names.extend(import_call_names(
        AMBIENTS_RS,
        "@ship/sail-runtime/internal",
    ));
    names
}

fn runtime_stub_dts() -> String {
    let names = runtime_stub_names();
    let mut body = String::from(
        "// enough that signal() is not any.\n\
         declare module '@ship/sail-runtime' {\n\
         \x20 export interface Signal<T> {\n\
         \x20   (): T\n\
         \x20   set(value: T): void\n\
         \x20 }\n\
         \x20 export function signal<T>(init: T): Signal<T>\n\
         \x20 export function controller<S>(setup: (ctx: unknown) => S): { __def: (ctx: unknown) => S }\n\
         \x20 export interface RouteSignals {\n\
         \x20   readonly path: string\n\
         \x20   readonly params: Record<string, string>\n\
         \x20 }\n",
    );
    for name in &names {
        if matches!(name.as_str(), "Signal" | "RouteSignals") {
            continue;
        }
        body.push_str(&format!("  export type {name} = unknown\n"));
    }
    body.push_str(
        "}\n\
         declare module '@ship/sail-runtime/internal' {\n\
         \x20 export * from '@ship/sail-runtime'\n\
         \x20 export function mountAmbient(...args: never[]): void\n\
         }\n",
    );
    body
}

fn write_tsconfig(work_dir: &Path, workspace_hint: Option<&Path>) {
    let mut paths = String::new();
    let runtime_src = find_package_src(workspace_hint, "packages/sail/runtime/src/index.ts");
    let internal_src = find_package_src(workspace_hint, "packages/sail/runtime/src/internal.ts");
    if runtime_src.is_none() || internal_src.is_none() {
        let _ = std::fs::write(work_dir.join("sail-runtime-stub.d.ts"), runtime_stub_dts());
        if runtime_src.is_none() {
            paths.push_str("      \"@ship/sail-runtime\": [\"./sail-runtime-stub.d.ts\"],\n");
        }
        if internal_src.is_none() {
            paths.push_str(
                "      \"@ship/sail-runtime/internal\": [\"./sail-runtime-stub.d.ts\"],\n",
            );
        }
    }
    for (spec, rel) in FRAMEWORK_TS_PATHS {
        if *spec == "@ship/sail-runtime" && runtime_src.is_none() {
            continue;
        }
        if *spec == "@ship/sail-runtime/internal" && internal_src.is_none() {
            continue;
        }
        if let Some(p) = find_package_src(workspace_hint, rel) {
            paths.push_str(&format!(
                "      \"{}\": [\"{}\"],\n",
                spec,
                escape_json_path(&p)
            ));
        }
    }
    // Subpath wildcard for runtime internals (`@ship/sail-runtime/foo`).
    if let Some(internal) =
        find_package_src(workspace_hint, "packages/sail/runtime/src/internal.ts")
    {
        if let Some(parent) = internal.parent() {
            paths.push_str(&format!(
                "      \"@ship/sail-runtime/*\": [\"{}\"],\n",
                escape_json_path(parent)
            ));
        }
    }
    // Product `@/` homes — SSOT `project_map::at_alias_paths`. Only when this
    // session is rooted on a Sail project (standalone `sail check` cwd).
    if let Some(hint) = workspace_hint {
        if let Some(project) = sail_lint::Project::discover(hint) {
            for m in sail_compiler::project_map::at_alias_paths() {
                let target = m.target.trim_start_matches("./");
                let abs = project.root().join(target);
                paths.push_str(&format!(
                    "      \"{}\": [\"{}\"],\n",
                    m.pattern,
                    escape_json_path(&abs)
                ));
            }
        }
    }
    let cfg = format!(
        r#"{{
  "compilerOptions": {{
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "strict": true,
    "noImplicitAny": true,
    "skipLibCheck": true,
    "noEmit": true,
    "allowImportingTsExtensions": true,
    "paths": {{
{paths}      "ship": ["./ship-shim.d.ts"]
    }}
  }},
  "include": ["./**/*.ts", "./sail-types.d.ts"]
}}
"#
    );
    let _ = std::fs::write(work_dir.join("tsconfig.json"), cfg);
    // Host-dispatch module — not subsumed by the sail ambient pack (different
    // namespace: `import … from 'ship'` vs controller globals).
    let _ = std::fs::write(
        work_dir.join("ship-shim.d.ts"),
        "declare module 'ship' {\n  export function call(op: string, args?: Record<string, unknown>): Promise<unknown>\n  export function emit(value: unknown): void\n}\n",
    );
    // Sail ambient pack (runtime + Style) — same emit as `sail-compiler types`.
    let _ = std::fs::write(
        work_dir.join("sail-types.d.ts"),
        sail_compiler::ambients::emit_types_dts(),
    );
    // Host Scope stub — `import type { Scope } from './page.sail'` is rewritten
    // here so tsgo sees route/navigate (sibling .d.ts is often empty).
    let _ = std::fs::write(
        work_dir.join("__sail_host_scope__.ts"),
        "import type { RouteSignals } from \"@ship/sail-runtime\";\n\
         export interface Scope {\n\
         \x20 readonly route: RouteSignals;\n\
         \x20 readonly navigate: (path: string) => void;\n\
         }\n",
    );
}

/// Map `file://` / absolute path URIs to a filesystem path (for import rewrite).
fn uri_to_path(uri: &str) -> Option<PathBuf> {
    let uri = uri.trim();
    if uri.is_empty() {
        return None;
    }
    if let Some(rest) = uri.strip_prefix("file://") {
        // file:///abs  |  file://localhost/abs  |  file://host/abs (rare)
        let path_str = if let Some(r) = rest.strip_prefix("localhost") {
            r
        } else {
            rest
        };
        // percent-decode minimal %20 etc. — paths we mint are unescaped.
        let pb = PathBuf::from(path_str);
        return Some(pb);
    }
    let pb = PathBuf::from(uri);
    if pb.is_absolute() { Some(pb) } else { None }
}

/// Rewrite `@/…` specs via [`sail_compiler::project_map::resolve_at_alias`]
/// (SSOT) to absolute on-disk modules. Same reason as relatives: the virtual
/// controller is not inside the project, so tsconfig `paths` from a monorepo
/// workspace hint cannot see a standalone app's `src/`.
fn rewrite_at_alias_imports(text: &str, sail_path: &Path) -> String {
    let Some(project) = sail_lint::Project::discover(sail_path) else {
        return text.to_string();
    };
    rewrite_from_specs(text, |spec| {
        if !spec.starts_with("@/") {
            return None;
        }
        let resolved = sail_compiler::project_map::resolve_at_alias(project.root(), spec)?;
        existing_module_file(&resolved)
    })
}

/// Rewrite `from "./x"` / `from '../x'` specs to absolute paths resolved
/// against `from_dir` (the real `.sail` parent). Virtual controllers live in a
/// temp work_dir, so bare relative imports cannot resolve siblings otherwise.
fn rewrite_relative_imports(text: &str, from_dir: &Path) -> String {
    rewrite_from_specs(text, |spec| {
        if spec.starts_with("./") || spec.starts_with("../") {
            resolve_ts_module(from_dir, spec)
        } else {
            None
        }
    })
}

fn rewrite_from_specs(text: &str, mut resolve: impl FnMut(&str) -> Option<PathBuf>) -> String {
    let bytes = text.as_bytes();
    let mut replacements: Vec<(usize, usize, String)> = Vec::new();
    let mut i = 0;
    while i + 4 < bytes.len() {
        if &bytes[i..i + 4] == b"from"
            && (i == 0 || !is_ident_byte(bytes[i - 1]))
            && (i + 4 >= bytes.len() || !is_ident_byte(bytes[i + 4]))
        {
            let mut j = i + 4;
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            if j < bytes.len() && (bytes[j] == b'\'' || bytes[j] == b'"') {
                let q = bytes[j];
                let start = j + 1;
                if let Some(rel_end) = text[start..].find(q as char) {
                    let end = start + rel_end;
                    let spec = &text[start..end];
                    if let Some(abs) = resolve(spec) {
                        let abs_s = abs.display().to_string().replace('\\', "/");
                        replacements.push((start, end, abs_s));
                    }
                    i = end + 1;
                    continue;
                }
            }
        }
        i += 1;
    }
    if replacements.is_empty() {
        return text.to_string();
    }
    let mut result = text.to_string();
    for (start, end, abs) in replacements.into_iter().rev() {
        result.replace_range(start..end, &abs);
    }
    result
}

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'$'
}

/// Resolve a relative TS module specifier against `from_dir`.
fn resolve_ts_module(from_dir: &Path, spec: &str) -> Option<PathBuf> {
    existing_module_file(&from_dir.join(spec))
}

/// Append TS extensions / index — do not `with_extension` (clobbers multi-dot stems).
fn existing_module_file(base: &Path) -> Option<PathBuf> {
    let base_s = base.to_string_lossy();
    let candidates = [
        base.to_path_buf(),
        PathBuf::from(format!("{base_s}.ts")),
        PathBuf::from(format!("{base_s}.tsx")),
        PathBuf::from(format!("{base_s}.d.ts")),
        base.join("index.ts"),
        base.join("index.tsx"),
    ];
    for c in candidates {
        if c.is_file() {
            return c.canonicalize().ok().or(Some(c));
        }
    }
    None
}

fn escape_json_path(p: &Path) -> String {
    p.display().to_string().replace('\\', "/")
}

fn find_package_src(hint: Option<&Path>, rel: &str) -> Option<PathBuf> {
    let mut d = hint
        .map(|p| {
            if p.is_file() {
                p.parent().unwrap_or(p).to_path_buf()
            } else {
                p.to_path_buf()
            }
        })
        .or_else(|| std::env::current_dir().ok());
    while let Some(cur) = d {
        let p = cur.join(rel);
        if p.exists() {
            return p.canonicalize().ok().or(Some(p));
        }
        d = cur.parent().map(|x| x.to_path_buf());
    }
    None
}

fn map_hover_ranges(hover: &mut Value, virt: &VirtualTs, file: VirtualFile, sail_text: &str) {
    let Some(range) = hover.get_mut("range") else {
        return;
    };
    if let Some(mapped) = map_lsp_range_value(range, virt, file, sail_text) {
        *range = mapped;
    } else {
        hover.as_object_mut().map(|o| o.remove("range"));
    }
}

fn map_definition_locations(
    result: &mut Value,
    virt: &VirtualTs,
    sail_uri: &str,
    sail_text: &str,
    ctrl_uri: &str,
    view_uri: &str,
) {
    match result {
        Value::Array(arr) => {
            for loc in arr.iter_mut() {
                map_one_location(loc, virt, sail_uri, sail_text, ctrl_uri, view_uri);
            }
        }
        Value::Object(_) => {
            map_one_location(result, virt, sail_uri, sail_text, ctrl_uri, view_uri);
        }
        _ => {}
    }
}

fn map_one_location(
    loc: &mut Value,
    virt: &VirtualTs,
    sail_uri: &str,
    sail_text: &str,
    ctrl_uri: &str,
    view_uri: &str,
) {
    let uri = loc
        .get("uri")
        .or_else(|| loc.get("targetUri"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let file = if uri == ctrl_uri || uri.ends_with(".ctrl.ts") {
        VirtualFile::Controller
    } else if uri == view_uri || uri.ends_with(".view.ts") {
        VirtualFile::View
    } else {
        return;
    };
    let range_key = if loc.get("range").is_some() {
        "range"
    } else if loc.get("targetRange").is_some() {
        "targetRange"
    } else {
        return;
    };
    if let Some(range) = loc.get(range_key).cloned() {
        if let Some(mapped) = map_lsp_range_value(&range, virt, file, sail_text) {
            if let Some(obj) = loc.as_object_mut() {
                obj.insert(range_key.to_string(), mapped.clone());
                if range_key == "targetRange" {
                    obj.insert("targetSelectionRange".into(), mapped);
                    obj.insert("targetUri".into(), Value::String(sail_uri.to_string()));
                } else {
                    obj.insert("uri".into(), Value::String(sail_uri.to_string()));
                }
            }
        }
    }
}

fn map_lsp_range_value(
    range: &Value,
    virt: &VirtualTs,
    file: VirtualFile,
    sail_text: &str,
) -> Option<Value> {
    let start = range.get("start")?;
    let end = range.get("end")?;
    let sl = start.get("line")?.as_u64()? as u32;
    let sc = start.get("character")?.as_u64()? as u32;
    let el = end.get("line")?.as_u64()? as u32;
    let ec = end.get("character")?.as_u64()? as u32;
    let text = virt.text(file);
    let v_start = lsp_position_to_offset(text, sl, sc);
    let v_end = lsp_position_to_offset(text, el, ec);
    let (s_start, s_end) = virt.map_virtual_range(file, v_start, v_end)?;
    let (s_line, s_col) = offset_to_lsp_position(sail_text, s_start);
    let (e_line, e_col) = offset_to_lsp_position(sail_text, s_end);
    Some(json!({
        "start": { "line": s_line, "character": s_col },
        "end": { "line": e_line, "character": e_col }
    }))
}

fn map_diagnostic(
    item: &Value,
    virt: &VirtualTs,
    file: VirtualFile,
    sail_text: &str,
) -> Option<lsp_types::Diagnostic> {
    let range = item.get("range")?;
    let mapped = map_lsp_range_value(range, virt, file, sail_text)?;
    // Drop diagnostics that don't map into a TS region (machinery noise).
    let start = mapped.get("start")?;
    let end = mapped.get("end")?;
    let s_line = start.get("line")?.as_u64()? as u32;
    let s_col = start.get("character")?.as_u64()? as u32;
    let s_off = lsp_position_to_offset(sail_text, s_line, s_col);
    if !virt.is_ts_region(s_off) && !virt.is_ts_region(s_off.saturating_sub(1)) {
        // Allow controller-mapped diags even if region edge.
        if file == VirtualFile::View {
            // Only keep expression / scope diags that map into regions.
            return None;
        }
    }
    let message = item.get("message")?.as_str()?.to_string();
    // Filter pure unused-locals noise from controller (TS6133) unless it
    // maps — keep property/name errors that prove typing.
    if let Some(code) = item.get("code").and_then(|c| c.as_i64()) {
        if code == 6133 {
            return None;
        }
    }
    let severity = match item.get("severity").and_then(|v| v.as_u64()) {
        Some(1) => Some(lsp_types::DiagnosticSeverity::ERROR),
        Some(2) => Some(lsp_types::DiagnosticSeverity::WARNING),
        Some(3) => Some(lsp_types::DiagnosticSeverity::INFORMATION),
        Some(4) => Some(lsp_types::DiagnosticSeverity::HINT),
        _ => Some(lsp_types::DiagnosticSeverity::ERROR),
    };
    let code = item.get("code").and_then(|c| {
        if let Some(n) = c.as_i64() {
            Some(lsp_types::NumberOrString::Number(n as i32))
        } else {
            c.as_str()
                .map(|s| lsp_types::NumberOrString::String(s.to_string()))
        }
    });
    Some(lsp_types::Diagnostic {
        range: lsp_types::Range {
            start: lsp_types::Position {
                line: s_line,
                character: s_col,
            },
            end: lsp_types::Position {
                line: end.get("line")?.as_u64()? as u32,
                character: end.get("character")?.as_u64()? as u32,
            },
        },
        severity,
        code,
        source: Some("typescript".into()),
        message,
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discover_finds_workspace_tsgo() {
        reset_discovery_state_for_tests();
        let found = discover_tsgo(None)
            .unwrap_or_else(|| panic!("{}", sail_compiler::ts_native::missing_sidecar_hint()));
        assert!(found.exists(), "{}", found.display());
        assert!(
            !found.to_string_lossy().contains("node_modules"),
            "discover must not walk node_modules: {}",
            found.display()
        );
    }

    #[test]
    fn degrade_logs_once() {
        reset_discovery_state_for_tests();
        assert!(log_degrade_once());
        assert!(!log_degrade_once());
    }

    #[test]
    fn session_degrades_without_panic_when_forced_missing() {
        let mut session = TsgoSession::structure_only();
        session.on_sail_change(
            "file:///x.sail",
            "<controller>\nconst x = 1\nexport default { x }\n</controller>\n<div>{x}</div>",
        );
        assert!(session.virtual_for("file:///x.sail").is_some());
        assert!(
            session
                .hover_mapped(
                    "file:///x.sail",
                    "<controller>\nconst x = 1\n</controller>",
                    0
                )
                .is_none()
        );
        assert!(session.diagnostics_mapped("file:///x.sail", "").is_empty());
    }

    #[test]
    fn runtime_stub_covers_ambient_runtime_names() {
        let src = AMBIENTS_RS;
        let stub = runtime_stub_dts();
        for spec in ["@ship/sail-runtime", "@ship/sail-runtime/internal"] {
            for name in import_type_names(src, spec) {
                assert!(
                    stub.contains(&name),
                    "stub missing import-type {name} from {spec}"
                );
            }
            for name in import_call_names(src, spec) {
                assert!(
                    stub.contains(&name),
                    "stub missing import() {name} from {spec}"
                );
            }
        }
        assert!(
            stub.contains("HostPropertyDesc"),
            "top import block misses HostPropertyDesc; import() form must be included"
        );
    }

    #[test]
    fn live_session_initialize_and_diag_smoke() {
        reset_discovery_state_for_tests();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .canonicalize()
            .unwrap();
        if discover_tsgo(None).is_none() {
            panic!("{}", sail_compiler::ts_native::missing_sidecar_hint());
        }
        let mut session = TsgoSession::start(Some(&root));
        if !session.is_live() {
            panic!("sidecar present but TsgoSession failed to start");
        }
        let src = r#"<controller>
import { controller, signal } from '@ship/sail-runtime'
export default controller((ctx) => ({ count: signal(0) }))
</controller>
<div>{countt()}</div>
"#;
        let uri = "file:///tmp/live-smoke.sail";
        session.on_sail_change(uri, src);
        let diags = session.diagnostics_mapped(uri, src);
        let d = diags
            .iter()
            .find(|d| d.message.contains("countt"))
            .unwrap_or_else(|| panic!("expected countt diagnostic, got {diags:?}"));
        let countt_off = src.find("countt").unwrap();
        let (line, _) = offset_to_lsp_position(src, countt_off);
        assert_eq!(
            d.range.start.line, line,
            "countt diagnostic must map to .sail line"
        );
    }

    fn owned_route_tree() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "sail-lsp-owned-route-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let from_dir = dir.join("routes");
        let sibling = dir.join("src/compositions/shell/shell-rail.ts");
        std::fs::create_dir_all(sibling.parent().unwrap()).unwrap();
        std::fs::create_dir_all(&from_dir).unwrap();
        std::fs::write(&sibling, "export const appRouteBase = () => ''\n").unwrap();
        dir
    }

    #[test]
    fn rewrite_relative_imports_resolves_sibling_ts() {
        let dir = owned_route_tree();
        let from_dir = dir.join("routes");
        let sibling = dir.join("src/compositions/shell/shell-rail.ts");
        let src = "import { appRouteBase } from '../src/compositions/shell/shell-rail'\n";
        let out = rewrite_relative_imports(src, &from_dir);
        assert!(
            out.contains("shell-rail.ts") || out.contains("shell-rail"),
            "expected absolute shell-rail path, got: {out}"
        );
        assert!(
            !out.contains("from '../src/compositions/shell/shell-rail'"),
            "relative spec must be rewritten: {out}"
        );
        let want = sibling.canonicalize().unwrap();
        assert!(
            out.contains(&want.to_string_lossy().replace('\\', "/")) || out.contains("shell-rail"),
            "path must land on the owned sibling: {out}"
        );
    }

    #[test]
    fn rewrite_relative_imports_leaves_package_specs() {
        let dir = PathBuf::from("/tmp");
        let src = "import { navigate } from '@ship/deck/router'\n";
        let out = rewrite_relative_imports(src, &dir);
        assert_eq!(out, src);
    }

    fn isolated_at_alias_app() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "sail-lsp-at-alias-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(dir.join("routes")).unwrap();
        std::fs::create_dir_all(dir.join("src/compositions")).unwrap();
        std::fs::create_dir_all(dir.join("src/stores")).unwrap();
        std::fs::write(dir.join("sail.config.jsonc"), "{}\n").unwrap();
        std::fs::write(
            dir.join("src/compositions/double.ts"),
            "export function double(n: number): number { return n * 2 }\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("src/stores/tally.ts"),
            "export const tally = { clicks: 0 }\n",
        )
        .unwrap();
        dir
    }

    #[test]
    fn rewrite_at_alias_imports_resolves_ssot_homes() {
        let project = isolated_at_alias_app();
        let page = project.join("routes/page.sail");
        std::fs::write(&page, "").unwrap();
        let src = "import { double } from '@/compositions/double'\nimport { tally } from '@/stores/tally'\n";
        let out = rewrite_at_alias_imports(src, &page);
        let _ = std::fs::remove_dir_all(&project);
        assert!(
            out.contains("double.ts") && out.contains("tally.ts"),
            "expected absolute @/ targets, got: {out}"
        );
        assert!(
            !out.contains("'@/compositions/double'") && !out.contains("'@/stores/tally'"),
            "@/ specs must be rewritten: {out}"
        );
    }

    #[test]
    fn rewrite_at_alias_imports_needs_sail_config() {
        let dir = std::env::temp_dir().join(format!(
            "sail-lsp-at-noconfig-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(dir.join("src/stores")).unwrap();
        std::fs::write(dir.join("src/stores/tally.ts"), "export const tally = 0\n").unwrap();
        let page = dir.join("routes/page.sail");
        std::fs::create_dir_all(page.parent().unwrap()).unwrap();
        let src = "import { tally } from '@/stores/tally'\n";
        let out = rewrite_at_alias_imports(src, &page);
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(out, src, "no project marker → leave @/ for TS2307");
    }

    #[test]
    fn live_session_resolves_at_alias() {
        reset_discovery_state_for_tests();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .canonicalize()
            .unwrap();
        if discover_tsgo(None).is_none() {
            panic!("{}", sail_compiler::ts_native::missing_sidecar_hint());
        }
        let project = isolated_at_alias_app();
        // Discover tsgo from this checkout; `@/` rewrite keys off the .sail path.
        let mut session = TsgoSession::start(Some(&root));
        if !session.is_live() {
            let _ = std::fs::remove_dir_all(&project);
            panic!("tsgo session failed to start");
        }
        let page = project.join("routes/page.sail");
        let src = r#"<controller>
import { double } from '@/compositions/double'
import { tally } from '@/stores/tally'
export default controller(() => {
  const n = signal(1)
  const twice = derived(() => double(n()))
  return { n, twice, tally }
})
</controller>
<p>{twice()}</p>
"#;
        std::fs::write(&page, src).unwrap();
        let uri = path_to_uri(&page);
        session.on_sail_change(&uri, src);
        let diags = session.diagnostics_mapped(&uri, src);
        let module_miss: Vec<_> = diags
            .iter()
            .filter(|d| d.message.contains("Cannot find module"))
            .map(|d| d.message.clone())
            .collect();
        let _ = std::fs::remove_dir_all(&project);
        assert!(
            module_miss.is_empty(),
            "standalone @/ imports must resolve under intelligence tsgo; got: {module_miss:?}"
        );
    }

    #[test]
    fn framework_ts_paths_cover_studio_route_imports() {
        let specs: Vec<&str> = FRAMEWORK_TS_PATHS.iter().map(|(s, _)| *s).collect();
        assert!(specs.contains(&"@ship/deck/router"));
        assert!(specs.contains(&"@ship/sail-studio-shell"));
        assert!(specs.contains(&"@ship/sail-runtime"));
    }

    #[test]
    fn live_session_resolves_deck_router_and_relative() {
        reset_discovery_state_for_tests();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .canonicalize()
            .unwrap();
        if discover_tsgo(None).is_none() {
            panic!("{}", sail_compiler::ts_native::missing_sidecar_hint());
        }
        let mut session = TsgoSession::start(Some(&root));
        if !session.is_live() {
            panic!("sidecar present but TsgoSession failed to start");
        }
        let dir = owned_route_tree();
        let page = dir.join("routes/page.sail");
        let src = r#"<controller>
import { navigate } from '@ship/deck/router'
import { appRouteBase } from '../src/compositions/shell/shell-rail'
export default controller(() => ({ navigate, appRouteBase }))
</controller>
<div />
"#;
        std::fs::write(&page, src).unwrap();
        let uri = path_to_uri(&page);
        session.on_sail_change(&uri, src);
        let diags = session.diagnostics_mapped(&uri, src);
        let module_miss: Vec<_> = diags
            .iter()
            .filter(|d| d.message.contains("Cannot find module"))
            .map(|d| d.message.clone())
            .collect();
        assert!(
            module_miss.is_empty(),
            "owned route must resolve @ship/deck/router + relative sibling; got: {module_miss:?}"
        );
    }

    #[test]
    fn live_tsgo_named_style_ref_is_in_scope() {
        reset_discovery_state_for_tests();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .canonicalize()
            .unwrap();
        if discover_tsgo(None).is_none() {
            panic!("{}", sail_compiler::ts_native::missing_sidecar_hint());
        }
        let mut session = TsgoSession::start(Some(&root));
        assert!(
            session.is_live(),
            "discover found tsgo but TsgoSession failed to start"
        );
        // Tiny SFC: first-class `style name =` + tag-position `style(name)`.
        // Controller is present so the view is a module with an import —
        // the declare must remain visible to `void (() => (style(host)))`.
        let src = r#"<scope>
  style host = { pad(4) }
</scope>
<controller>
import { controller } from '@ship/sail-runtime'
export default controller(() => ({}))
</controller>
<div style(host)>x</div>
"#;
        let uri = "file:///tmp/named-style-host.sail";
        session.on_sail_change(uri, src);
        std::thread::sleep(std::time::Duration::from_millis(200));
        let view_text = session
            .virtual_for(uri)
            .expect("virtual ts built for named style SFC")
            .view_text
            .clone();
        assert!(
            view_text.contains("declare const host: any"),
            "collector must declare host:\n{view_text}"
        );
        assert!(
            view_text.contains("style(host)"),
            "style(host) must embed:\n{view_text}"
        );
        let diags = session.diagnostics_mapped(uri, src);
        let hits: Vec<_> = diags
            .iter()
            .filter(|d| d.message.contains("Cannot find name 'host'"))
            .map(|d| d.message.clone())
            .collect();
        assert!(
            hits.is_empty(),
            "named style host must be in scope for style(host); view:\n{view_text}\ndiags: {diags:?}"
        );
    }

    #[test]
    fn live_tsgo_multiline_scope_type_keeps_named_style_in_scope() {
        reset_discovery_state_for_tests();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .canonicalize()
            .unwrap();
        if discover_tsgo(None).is_none() {
            panic!("{}", sail_compiler::ts_native::missing_sidecar_hint());
        }
        let mut session = TsgoSession::start(Some(&root));
        assert!(session.is_live(), "tsgo session failed to start");
        let src = r#"<scope>
  type Row = {
    id: string
  }
  style card = { pad(4) }
</scope>
<controller>
export default controller(() => ({}))
</controller>
<div style(card)>x</div>
"#;
        let uri = "file:///tmp/named-style-multiline-type.sail";
        session.on_sail_change(uri, src);
        std::thread::sleep(std::time::Duration::from_millis(200));
        let view_text = session.virtual_for(uri).expect("virtual").view_text.clone();
        let diags = session.diagnostics_mapped(uri, src);
        let hits: Vec<_> = diags
            .iter()
            .filter(|d| d.message.contains("Cannot find name 'card'"))
            .map(|d| d.message.clone())
            .collect();
        assert!(
            hits.is_empty(),
            "multiline scope type must not hide style card; view:\n{view_text}\ndiags: {diags:?}"
        );
    }

    #[test]
    fn live_tsgo_landing_shaped_named_styles_are_in_scope() {
        reset_discovery_state_for_tests();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .canonicalize()
            .unwrap();
        if discover_tsgo(None).is_none() {
            panic!("{}", sail_compiler::ts_native::missing_sidecar_hint());
        }
        let mut session = TsgoSession::start(Some(&root));
        assert!(session.is_live(), "tsgo session failed to start");
        let src = r#"<scope>
  style page = { size({ w: 100% }).bg(surface-0) }
  style hero = { pad(4), when: { tablet: pad(8) } }
  style cardLink = { display(block) }
  style swatch = { size(6) }
</scope>
<controller>
export default controller(() => ({ ticks: signal(0) }))
</controller>
<a style(cardLink)>
  <div style(swatch, { bg(primary) })></div>
</a>
<column style(page)>
  <section style(hero, { shrink(0) })>x</section>
  <when {tablet}>
    <p>{ticks()}</p>
  {else => <p>{ticks()}</p>}
  </when>
</column>
"#;
        let uri = "file:///tmp/named-style-landing-shaped.sail";
        session.on_sail_change(uri, src);
        std::thread::sleep(std::time::Duration::from_millis(200));
        let view_text = session.virtual_for(uri).expect("virtual").view_text.clone();
        let diags = session.diagnostics_mapped(uri, src);
        let missing: Vec<_> = diags
            .iter()
            .filter(|d| {
                d.message.contains("Cannot find name '")
                    && ["page", "hero", "cardLink", "swatch", "tablet"]
                        .iter()
                        .any(|n| d.message.contains(&format!("'{n}'")))
            })
            .map(|d| d.message.clone())
            .collect();
        assert!(
            missing.is_empty(),
            "landing-shaped named styles / when-tag must typecheck; view:\n{view_text}\nmissing: {missing:?}\nall: {diags:?}"
        );
    }
}
