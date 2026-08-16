//! Stranger proof: `sail create` then attach `sail lsp` over stdio.
//!
//! The VS Code client is `sail` on PATH + this protocol. This test is the
//! cut gate: diagnostic, hover, and complete on the created `.sail`,
//! cwd = the new app (not this repo).

use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};
use tempfile::TempDir;

fn sail_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_sail"))
}

fn unique_stem(label: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{label}-{nanos:x}")
}

fn path_to_uri(path: &Path) -> String {
    let abs = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    format!("file://{}", abs.display())
}

fn pos_of(src: &str, needle: &str) -> (u32, u32) {
    let off = src
        .find(needle)
        .unwrap_or_else(|| panic!("needle `{needle}` missing"));
    let mut line = 0u32;
    let mut col = 0u32;
    for (i, ch) in src.char_indices() {
        if i >= off {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }
    (line, col)
}

struct LspClient {
    child: Child,
    stdin: ChildStdin,
    inbound: mpsc::Receiver<Value>,
    next_id: u64,
    stderr: Arc<Mutex<Vec<String>>>,
    initialize_result: Value,
    _home: Option<TempDir>,
}

impl LspClient {
    fn attach(root: &Path) -> Self {
        Self::attach_inner(root, false)
    }

    fn attach_isolated(root: &Path) -> Self {
        Self::attach_inner(root, true)
    }

    fn attach_inner(root: &Path, isolated: bool) -> Self {
        let mut cmd = Command::new(sail_bin());
        cmd.arg("lsp")
            .current_dir(root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let home = if isolated {
            cmd.env_remove("SAIL_TSC");
            cmd.env_remove("SAIL_TSGO");
            cmd.env_remove("SAIL_TS_NATIVE");
            let home = TempDir::new().expect("empty HOME");
            cmd.env("HOME", home.path());
            cmd.env("XDG_DATA_HOME", home.path().join("xdg-data"));
            #[cfg(windows)]
            {
                cmd.env("USERPROFILE", home.path());
                cmd.env("LOCALAPPDATA", home.path().join("xdg-data"));
                let system = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".into());
                cmd.env("PATH", format!("{system}\\System32"));
            }
            #[cfg(not(windows))]
            {
                cmd.env("PATH", "/usr/bin:/bin");
            }
            Some(home)
        } else {
            None
        };
        let mut child = cmd.spawn().expect("spawn sail lsp");

        let stdin = child.stdin.take().expect("lsp stdin");
        let stdout = child.stdout.take().expect("lsp stdout");
        let stderr_buf = Arc::new(Mutex::new(Vec::new()));
        if let Some(stderr) = child.stderr.take() {
            let buf = Arc::clone(&stderr_buf);
            thread::Builder::new()
                .name("sail-lsp-stderr".into())
                .spawn(move || {
                    let mut r = BufReader::new(stderr);
                    let mut line = String::new();
                    while r.read_line(&mut line).ok().filter(|&n| n > 0).is_some() {
                        buf.lock().unwrap().push(line.trim_end().to_string());
                        line.clear();
                    }
                })
                .ok();
        }

        let (tx, rx) = mpsc::channel();
        thread::Builder::new()
            .name("sail-lsp-reader".into())
            .spawn(move || {
                let mut reader = BufReader::new(stdout);
                while let Ok(msg) = read_message(&mut reader) {
                    if tx.send(msg).is_err() {
                        break;
                    }
                }
            })
            .ok();

        let mut client = Self {
            child,
            stdin,
            inbound: rx,
            next_id: 1,
            stderr: stderr_buf,
            initialize_result: Value::Null,
            _home: home,
        };
        let root_uri = path_to_uri(root);
        let init = client.request(
            "initialize",
            json!({
                "processId": std::process::id(),
                "clientInfo": { "name": "sail-lsp-e2e", "version": "0.1.0" },
                "rootUri": root_uri,
                "capabilities": {
                    "textDocument": {
                        "hover": { "contentFormat": ["markdown", "plaintext"] },
                        "completion": {},
                        "publishDiagnostics": {}
                    }
                },
                "workspaceFolders": [{ "uri": root_uri, "name": "app" }]
            }),
            Duration::from_secs(20),
        );
        assert!(init.get("error").is_none(), "initialize error: {init}");
        client.initialize_result = init.get("result").cloned().unwrap_or(init);
        client.notify("initialized", json!({}));
        client
    }

    fn wait_live_peer_line(&self, peer_dir: &Path) -> String {
        let deadline = Instant::now() + Duration::from_secs(1);
        loop {
            let lines = self.stderr.lock().expect("stderr lock").clone();
            for line in &lines {
                if let Some(path) = parse_live_peer_line(line) {
                    assert!(
                        path.starts_with(peer_dir),
                        "live path {} is not under {}",
                        path.display(),
                        peer_dir.display()
                    );
                    return line.clone();
                }
            }
            if Instant::now() >= deadline {
                panic!("no `sail-lsp: typescript … live (peer)` line within 1s; stderr={lines:?}");
            }
            thread::sleep(Duration::from_millis(20));
        }
    }

    fn write_message(&mut self, msg: &Value) {
        let body = serde_json::to_vec(msg).expect("serialize lsp");
        write!(self.stdin, "Content-Length: {}\r\n\r\n", body.len()).expect("write header");
        self.stdin.write_all(&body).expect("write body");
        self.stdin.flush().expect("flush lsp stdin");
    }

    fn notify(&mut self, method: &str, params: Value) {
        self.write_message(&json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        }));
    }

    fn reply(&mut self, id: &Value, result: Value) {
        self.write_message(&json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": result
        }));
    }

    fn handle_server_msg(&mut self, msg: &Value) {
        let Some(method) = msg.get("method").and_then(|m| m.as_str()) else {
            return;
        };
        let Some(id) = msg.get("id") else {
            return;
        };
        match method {
            "workspace/configuration" => {
                let n = msg
                    .pointer("/params/items")
                    .and_then(|i| i.as_array())
                    .map(|a| a.len())
                    .unwrap_or(1);
                let result: Vec<Value> = (0..n).map(|_| json!({})).collect();
                self.reply(id, Value::Array(result));
            }
            _ => self.reply(id, Value::Null),
        }
    }

    fn request(&mut self, method: &str, params: Value, timeout: Duration) -> Value {
        let id = self.next_id;
        self.next_id += 1;
        self.write_message(&json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        }));
        let deadline = Instant::now() + timeout;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            assert!(!remaining.is_zero(), "sail lsp request {method} timed out");
            match self.inbound.recv_timeout(remaining) {
                Ok(msg) => {
                    if msg.get("method").is_some() && msg.get("id").is_some() {
                        self.handle_server_msg(&msg);
                        continue;
                    }
                    let msg_id = msg.get("id").and_then(|v| {
                        v.as_u64()
                            .or_else(|| v.as_i64().map(|i| i as u64))
                            .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
                    });
                    if msg_id == Some(id) {
                        return msg;
                    }
                }
                Err(RecvTimeoutError::Timeout) => {
                    panic!("sail lsp request {method} timed out")
                }
                Err(RecvTimeoutError::Disconnected) => {
                    panic!("sail lsp stdout closed during {method}")
                }
            }
        }
    }

    fn wait_notification(&mut self, method: &str, timeout: Duration) -> Value {
        let deadline = Instant::now() + timeout;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            assert!(!remaining.is_zero(), "timed out waiting for {method}");
            match self.inbound.recv_timeout(remaining) {
                Ok(msg) => {
                    if msg.get("method").is_some() && msg.get("id").is_some() {
                        self.handle_server_msg(&msg);
                        continue;
                    }
                    if msg.get("method").and_then(|m| m.as_str()) == Some(method) {
                        return msg;
                    }
                }
                Err(RecvTimeoutError::Timeout) => {
                    panic!("timed out waiting for {method}")
                }
                Err(RecvTimeoutError::Disconnected) => {
                    panic!("sail lsp stdout closed waiting for {method}")
                }
            }
        }
    }
}

impl Drop for LspClient {
    fn drop(&mut self) {
        self.notify("exit", Value::Null);
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
                "sail lsp stdout closed",
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

fn parse_live_peer_line(line: &str) -> Option<PathBuf> {
    // sail-lsp: typescript <ver> live (peer) <abs-path>
    let rest = line.strip_prefix("sail-lsp: typescript ")?;
    let marker = " live (peer) ";
    let idx = rest.find(marker)?;
    let path = rest[idx + marker.len()..].trim();
    if path.is_empty() {
        None
    } else {
        Some(PathBuf::from(path))
    }
}

fn require_peer_sidecar() -> PathBuf {
    let peer = sail_bin()
        .parent()
        .expect("sail dir")
        .join("sail-ts-native");
    let tsc = peer.join(if cfg!(windows) { "tsc.exe" } else { "tsc" });
    if !tsc.is_file() || !peer.join("lib.d.ts").is_file() {
        panic!("TypeScript sidecar missing at {}", tsc.display());
    }
    peer
}

#[test]
fn create_then_lsp_hover_complete_diagnostic() {
    let tmp = TempDir::new().expect("tempdir");
    let app = tmp.path().join(unique_stem("create-lsp"));
    let create = Command::new(sail_bin())
        .args(["create", app.to_str().unwrap()])
        .current_dir(tmp.path())
        .output()
        .expect("spawn sail create");
    assert!(
        create.status.success(),
        "create failed: {}",
        String::from_utf8_lossy(&create.stderr)
    );

    let page = app.join("routes/page.sail");
    let src = std::fs::read_to_string(&page).expect("read created page");
    assert!(
        src.contains("<column"),
        "created page should be the hello-world column: {src}"
    );

    let uri = path_to_uri(&page);
    let mut lsp = LspClient::attach(&app);

    lsp.notify(
        "textDocument/didOpen",
        json!({
            "textDocument": {
                "uri": uri,
                "languageId": "sail",
                "version": 1,
                "text": src
            }
        }),
    );
    let opened = lsp.wait_notification("textDocument/publishDiagnostics", Duration::from_secs(10));
    assert_eq!(
        opened.pointer("/params/uri").and_then(|u| u.as_str()),
        Some(uri.as_str()),
        "didOpen diagnostics: {opened}"
    );
    assert!(
        opened
            .pointer("/params/diagnostics")
            .and_then(|d| d.as_array())
            .is_some(),
        "didOpen must publish a diagnostics array: {opened}"
    );

    let (line, col) = pos_of(&src, "column");
    let hover = lsp.request(
        "textDocument/hover",
        json!({
            "textDocument": { "uri": uri },
            "position": { "line": line, "character": col }
        }),
        Duration::from_secs(10),
    );
    let hover_text = hover
        .pointer("/result/contents/value")
        .and_then(|v| v.as_str())
        .or_else(|| hover.pointer("/result/contents").and_then(|v| v.as_str()))
        .unwrap_or("");
    assert!(
        hover_text.contains("column") && hover_text.contains("layout"),
        "hover on created <column> must teach the layout keyword:\n{hover}"
    );

    // Just after the created `<` — empty tag prefix offers the L1 surface.
    // (`column` is a layout keyword, not an L1 catalog row.)
    let (cline, ccol) = pos_of(&src, "<column");
    let complete = lsp.request(
        "textDocument/completion",
        json!({
            "textDocument": { "uri": uri },
            "position": { "line": cline, "character": ccol + 1 }
        }),
        Duration::from_secs(10),
    );
    let items = complete
        .pointer("/result")
        .and_then(|r| {
            r.as_array()
                .or_else(|| r.get("items").and_then(|i| i.as_array()))
        })
        .cloned()
        .unwrap_or_default();
    let labels: Vec<&str> = items
        .iter()
        .filter_map(|i| i.get("label").and_then(|l| l.as_str()))
        .collect();
    assert!(
        labels.iter().any(|l| *l == "button" || *l == "card"),
        "complete after `<` on the created page must offer the L1 surface: {labels:?}\n{complete}"
    );

    let dirty = "<p class=\"x\">hi</p>\n";
    lsp.notify(
        "textDocument/didChange",
        json!({
            "textDocument": { "uri": uri, "version": 2 },
            "contentChanges": [{ "text": dirty }]
        }),
    );
    let dirty_diags =
        lsp.wait_notification("textDocument/publishDiagnostics", Duration::from_secs(10));
    let messages: Vec<String> = dirty_diags
        .pointer("/params/diagnostics")
        .and_then(|d| d.as_array())
        .into_iter()
        .flatten()
        .filter_map(|d| {
            d.get("message")
                .and_then(|m| m.as_str())
                .map(str::to_string)
        })
        .collect();
    assert!(
        messages.iter().any(|m| m.contains("class")),
        "didChange with class= must publish a diagnostic: {messages:?}\n{dirty_diags}"
    );
}

#[test]
fn create_then_lsp_ts_typo_maps_without_node_modules() {
    // hover-on-`signal` is out of scope; the red assertion is a mapped typo.
    let peer_dir = require_peer_sidecar();
    let tmp = TempDir::new().expect("tempdir");
    let app = tmp.path().join(unique_stem("create-lsp-typed"));
    let create = Command::new(sail_bin())
        .args(["create", app.to_str().unwrap()])
        .current_dir(tmp.path())
        .output()
        .expect("spawn sail create");
    assert!(
        create.status.success(),
        "create failed: {}",
        String::from_utf8_lossy(&create.stderr)
    );
    assert!(
        !app.join("node_modules").exists(),
        "created app must not have node_modules"
    );

    let page = app.join("routes/page.sail");
    let src = r#"<controller>
export default controller(() => {
  const c = signal(0)
  return { count: c.nope }
})
</controller>
<column gap="4">
  <p>{countt()}</p>
</column>
"#;
    std::fs::write(&page, src).expect("write typed page");

    let uri = path_to_uri(&page);
    let mut lsp = LspClient::attach_isolated(&app);
    lsp.wait_live_peer_line(&peer_dir);
    assert_eq!(
        lsp.initialize_result
            .pointer("/sail/typescript/source")
            .and_then(|v| v.as_str()),
        Some("peer"),
        "initializeResult.sail.typescript.source: {}",
        lsp.initialize_result
    );

    lsp.notify(
        "textDocument/didOpen",
        json!({
            "textDocument": {
                "uri": uri,
                "languageId": "sail",
                "version": 1,
                "text": src
            }
        }),
    );
    let deadline = Instant::now() + Duration::from_secs(20);
    let mut hit_countt = false;
    let mut hit_nope = false;
    let (countt_line, _) = pos_of(src, "countt");
    let (nope_line, _) = pos_of(src, "nope");
    while Instant::now() < deadline && !(hit_countt && hit_nope) {
        let note = lsp.wait_notification(
            "textDocument/publishDiagnostics",
            deadline.saturating_duration_since(Instant::now()),
        );
        let diags = note
            .pointer("/params/diagnostics")
            .and_then(|d| d.as_array())
            .cloned()
            .unwrap_or_default();
        for d in &diags {
            let msg = d.get("message").and_then(|m| m.as_str()).unwrap_or("");
            let line = d
                .pointer("/range/start/line")
                .and_then(|v| v.as_u64())
                .unwrap_or(u64::MAX) as u32;
            if msg.contains("countt") && line == countt_line {
                hit_countt = true;
            }
            if msg.contains("nope") && line == nope_line {
                hit_nope = true;
            }
        }
    }
    assert!(
        hit_countt && hit_nope,
        "expected mapped countt / nope diagnostics on the .sail spans (countt line {countt_line}, nope line {nope_line})"
    );
}
