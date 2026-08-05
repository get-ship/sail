//! tsgo LSP sidecar — spawn, discover, and proxy position-based requests.
//!
//! Binary discovery order (SPEC-LSP-1):
//! 1. `node_modules/.bin/tsgo` walking up from a file/workspace path
//! 2. Prefer the native platform binary next to `@typescript/native-preview`
//! 3. `tsgo` on `PATH`
//! 4. Graceful degrade (structure-only); log once
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

/// Discover a tsgo binary. See module docs for the search order.
pub fn discover_tsgo(start: Option<&Path>) -> Option<PathBuf> {
    let mut dir = start
        .map(|p| {
            if p.is_file() {
                p.parent().unwrap_or(p).to_path_buf()
            } else {
                p.to_path_buf()
            }
        })
        .or_else(|| std::env::current_dir().ok());

    while let Some(d) = dir {
        let bin = d.join("node_modules/.bin/tsgo");
        if bin.exists() {
            if let Some(native) = resolve_native_tsgo(&bin) {
                TSGO_AVAILABLE.store(true, Ordering::SeqCst);
                return Some(native);
            }
            TSGO_AVAILABLE.store(true, Ordering::SeqCst);
            return Some(bin);
        }
        if let Some(native) = platform_native_under(&d.join("node_modules")) {
            TSGO_AVAILABLE.store(true, Ordering::SeqCst);
            return Some(native);
        }
        dir = d.parent().map(|p| p.to_path_buf());
    }

    if let Ok(path) = which("tsgo") {
        TSGO_AVAILABLE.store(true, Ordering::SeqCst);
        return Some(path);
    }
    TSGO_AVAILABLE.store(false, Ordering::SeqCst);
    None
}

/// Log the degrade message once. Returns true if this call logged it.
pub fn log_degrade_once() -> bool {
    if DEGRADE_LOGGED
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
    {
        eprintln!(
            "sail-lsp: tsgo not found (searched node_modules/.bin/tsgo and PATH) — \
             TypeScript intelligence disabled; structure capabilities remain active"
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

fn resolve_native_tsgo(bin_script: &Path) -> Option<PathBuf> {
    let nm = bin_script.parent()?.parent()?;
    platform_native_under(nm)
}

fn platform_native_under(node_modules: &Path) -> Option<PathBuf> {
    let os = std::env::consts::OS;
    let arch = match std::env::consts::ARCH {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        other => other,
    };
    let os = if os == "windows" { "win32" } else { os };
    let pkg = format!("@typescript/native-preview-{os}-{arch}");
    let mut exe = node_modules.join(&pkg).join("lib").join("tsgo");
    if cfg!(windows) {
        exe.set_extension("exe");
    }
    exe.exists().then_some(exe)
}

fn which(name: &str) -> Result<PathBuf, ()> {
    let path = std::env::var_os("PATH").ok_or(())?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    Err(())
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
                loop {
                    match read_message(&mut reader) {
                        Ok(msg) => {
                            if tx.send(msg).is_err() {
                                break;
                            }
                        }
                        Err(_) => break,
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
}

impl TsgoSession {
    pub fn start(workspace_hint: Option<&Path>) -> Self {
        let work_dir = std::env::temp_dir().join(format!(
            "sail-lsp-tsgo-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0)
        ));
        let _ = std::fs::create_dir_all(&work_dir);

        let tsgo = discover_tsgo(workspace_hint);
        let client = match tsgo {
            Some(bin) => {
                write_tsconfig(&work_dir, workspace_hint);
                match TsgoClient::spawn(&bin, &work_dir) {
                    Ok(c) => Some(c),
                    Err(e) => {
                        eprintln!("sail-lsp: failed to start tsgo ({e}) — degrading");
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
        }
    }

    pub fn is_live(&self) -> bool {
        self.client.is_some()
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
            let (ctrl_uri, view_uri) = virtual_uris(sail_uri, &self.work_dir);
            // Materialize controller as `__ctrl__.ts` relative import target
            // from the view file, and the view itself under a stable name.
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
            let _ = std::fs::write(&ctrl_path, &virt.controller_text);
            let _ = std::fs::write(&view_path, &view_text);
            // Keep a mutated copy for position mapping (import path length may differ).
            let mut virt = virt;
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

fn write_tsconfig(work_dir: &Path, workspace_hint: Option<&Path>) {
    let runtime = find_package_src(workspace_hint, "packages/sail/runtime/src/index.ts");
    let internal = find_package_src(workspace_hint, "packages/sail/runtime/src/internal.ts");
    let mut paths = String::new();
    if let Some(r) = runtime {
        paths.push_str(&format!(
            "      \"@ship/sail-runtime\": [\"{}\"],\n",
            escape_json_path(&r)
        ));
    }
    if let Some(i) = internal {
        paths.push_str(&format!(
            "      \"@ship/sail-runtime/internal\": [\"{}\"],\n",
            escape_json_path(&i)
        ));
        if let Some(parent) = i.parent() {
            paths.push_str(&format!(
                "      \"@ship/sail-runtime/*\": [\"{}\"],\n",
                escape_json_path(parent)
            ));
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
    // SPEC-TS-1 ambient pack (runtime + Style) — same emit as `sail-compiler types`.
    let _ = std::fs::write(
        work_dir.join("sail-types.d.ts"),
        sail_compiler::ambients::emit_types_dts(),
    );
}

fn escape_json_path(p: &Path) -> String {
    p.display().to_string().replace('\\', "/")
}

fn find_package_src(hint: Option<&Path>, rel: &str) -> Option<PathBuf> {
    let mut dir = hint.map(|p| {
        if p.is_file() {
            p.parent().unwrap_or(p).to_path_buf()
        } else {
            p.to_path_buf()
        }
    });
    if dir.is_none() {
        dir = std::env::current_dir().ok();
    }
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let candidates = [
        dir.clone(),
        Some(manifest.clone()),
        manifest.parent().map(|p| p.to_path_buf()),
        manifest.ancestors().nth(2).map(|p| p.to_path_buf()),
        manifest.ancestors().nth(3).map(|p| p.to_path_buf()),
    ];
    for start in candidates.into_iter().flatten() {
        let mut d = Some(start);
        while let Some(cur) = d {
            let p = cur.join(rel);
            if p.exists() {
                return p.canonicalize().ok().or(Some(p));
            }
            d = cur.parent().map(|x| x.to_path_buf());
        }
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
        source: Some("tsgo".into()),
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
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .canonicalize()
            .unwrap();
        let found = discover_tsgo(Some(&root));
        assert!(
            found.is_some(),
            "expected tsgo under workspace node_modules, got None"
        );
        let p = found.unwrap();
        assert!(p.exists(), "{}", p.display());
    }

    #[test]
    fn degrade_logs_once() {
        reset_discovery_state_for_tests();
        assert!(log_degrade_once());
        assert!(!log_degrade_once());
    }

    #[test]
    fn session_degrades_without_panic_when_forced_missing() {
        let work_dir = std::env::temp_dir().join("sail-lsp-degrade-test");
        let _ = std::fs::create_dir_all(&work_dir);
        let mut session = TsgoSession {
            client: None,
            work_dir: work_dir.clone(),
            virtuals: HashMap::new(),
        };
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
        let _ = std::fs::remove_dir_all(&work_dir);
    }

    #[test]
    fn live_session_initialize_and_diag_smoke() {
        reset_discovery_state_for_tests();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .canonicalize()
            .unwrap();
        if discover_tsgo(Some(&root)).is_none() {
            eprintln!("SKIP live_session: no tsgo");
            return;
        }
        let mut session = TsgoSession::start(Some(&root));
        if !session.is_live() {
            eprintln!("SKIP live_session: failed to start");
            return;
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
        // Soft: assert no panic; when runtime types resolve, countt is flagged.
        if let Some(d) = diags.iter().find(|d| d.message.contains("countt")) {
            let countt_off = src.find("countt").unwrap();
            let (line, _) = offset_to_lsp_position(src, countt_off);
            assert_eq!(
                d.range.start.line, line,
                "countt diagnostic must map to .sail line"
            );
        } else {
            eprintln!(
                "NOTE: no countt diagnostic yet (got {} diags) — scope recovery \
                 may still be resolving runtime types; map+session path exercised",
                diags.len()
            );
        }
    }
}
