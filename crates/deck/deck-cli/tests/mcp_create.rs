//! Stranger proof: `sail create` then attach `sail mcp` over stdio.
//!
//! Agents launch `sail mcp` (see `sail mcp install`). This test is the
//! cut gate: tools/list + check + hover + inspect + edit on the created
//! `.sail`, cwd = the new app (not this repo, not ops fixtures).

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};
use tempfile::TempDir;

const PROTOCOL_VERSION: &str = "2026-07-28";
const META_PROTOCOL_VERSION: &str = "io.modelcontextprotocol/protocolVersion";

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

fn pos_of(src: &str, needle: &str) -> (u32, u32) {
    let off = src
        .find(needle)
        .unwrap_or_else(|| panic!("needle `{needle}` missing"));
    let mut line = 1u32;
    let mut col = 1u32;
    for (i, ch) in src.char_indices() {
        if i >= off {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

fn modern_meta() -> Value {
    json!({
        META_PROTOCOL_VERSION: PROTOCOL_VERSION,
        "io.modelcontextprotocol/clientInfo": { "name": "sail-mcp-e2e", "version": "0.1.0" },
        "io.modelcontextprotocol/clientCapabilities": {},
    })
}

struct McpClient {
    child: Child,
    stdin: ChildStdin,
    inbound: mpsc::Receiver<Value>,
    next_id: i64,
}

impl McpClient {
    fn attach(root: &Path) -> Self {
        let mut child = Command::new(sail_bin())
            .arg("mcp")
            .current_dir(root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn sail mcp");
        let stdin = child.stdin.take().expect("mcp stdin");
        let stdout = child.stdout.take().expect("mcp stdout");
        if let Some(stderr) = child.stderr.take() {
            thread::Builder::new()
                .name("sail-mcp-stderr".into())
                .spawn(move || {
                    let mut r = BufReader::new(stderr);
                    let mut line = String::new();
                    while r.read_line(&mut line).ok().filter(|&n| n > 0).is_some() {
                        line.clear();
                    }
                })
                .ok();
        }
        let (tx, rx) = mpsc::channel();
        thread::Builder::new()
            .name("sail-mcp-reader".into())
            .spawn(move || {
                let mut reader = BufReader::new(stdout);
                let mut line = String::new();
                while reader
                    .read_line(&mut line)
                    .ok()
                    .filter(|&n| n > 0)
                    .is_some()
                {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        line.clear();
                        continue;
                    }
                    match serde_json::from_str::<Value>(trimmed) {
                        Ok(msg) => {
                            if tx.send(msg).is_err() {
                                break;
                            }
                        }
                        Err(_) => {}
                    }
                    line.clear();
                }
            })
            .ok();
        Self {
            child,
            stdin,
            inbound: rx,
            next_id: 1,
        }
    }

    fn request(&mut self, method: &str, mut params: Value, timeout: Duration) -> Value {
        if let Some(obj) = params.as_object_mut() {
            obj.entry("_meta").or_insert_with(modern_meta);
        } else if params.is_null() {
            params = json!({ "_meta": modern_meta() });
        }
        let id = self.next_id;
        self.next_id += 1;
        let msg = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });
        let body = serde_json::to_string(&msg).expect("serialize mcp");
        writeln!(self.stdin, "{body}").expect("write mcp");
        self.stdin.flush().expect("flush mcp");
        let deadline = Instant::now() + timeout;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            assert!(!remaining.is_zero(), "sail mcp request {method} timed out");
            match self.inbound.recv_timeout(remaining) {
                Ok(msg) => {
                    let msg_id = msg.get("id").and_then(|v| {
                        v.as_i64()
                            .or_else(|| v.as_u64().map(|u| u as i64))
                            .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
                    });
                    if msg_id == Some(id) {
                        return msg;
                    }
                }
                Err(RecvTimeoutError::Timeout) => {
                    panic!("sail mcp request {method} timed out")
                }
                Err(RecvTimeoutError::Disconnected) => {
                    panic!("sail mcp stdout closed during {method}")
                }
            }
        }
    }

    fn call_tool(&mut self, name: &str, arguments: Value) -> Value {
        let resp = self.request(
            "tools/call",
            json!({ "name": name, "arguments": arguments }),
            Duration::from_secs(20),
        );
        assert!(resp.get("error").is_none(), "{name} protocol error: {resp}");
        let result = resp.get("result").unwrap_or(&resp);
        assert_eq!(
            result.get("isError").and_then(|v| v.as_bool()),
            Some(false),
            "{name} tool error: {resp}"
        );
        result
            .get("structuredContent")
            .cloned()
            .unwrap_or_else(|| result.clone())
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn create_app(tmp: &Path) -> PathBuf {
    let app = tmp.join(unique_stem("create-mcp"));
    let create = Command::new(sail_bin())
        .args(["create", app.to_str().unwrap()])
        .current_dir(tmp)
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
    app
}

#[test]
fn create_then_mcp_list_check_hover_inspect() {
    let tmp = TempDir::new().expect("tempdir");
    let app = create_app(tmp.path());

    let install = Command::new(sail_bin())
        .args(["mcp", "install", "--project", app.to_str().unwrap()])
        .current_dir(tmp.path())
        .output()
        .expect("spawn sail mcp install");
    assert!(
        install.status.success(),
        "mcp install failed: {}",
        String::from_utf8_lossy(&install.stderr)
    );
    let cfg: Value = serde_json::from_str(
        &std::fs::read_to_string(app.join(".mcp.json")).expect("read .mcp.json"),
    )
    .expect("parse .mcp.json");
    let sail = &cfg["mcpServers"]["sail"];
    assert_eq!(sail["command"].as_str(), Some("sail"), "{cfg}");
    assert_eq!(
        sail["args"].as_array().map(|a| a.as_slice()),
        Some([json!("mcp")].as_slice()),
        "{cfg}"
    );

    let page = app.join("routes/page.sail");
    let src = std::fs::read_to_string(&page).expect("read created page");
    assert!(
        src.contains("<column"),
        "created page should be the hello-world column: {src}"
    );

    let mut mcp = McpClient::attach(&app);

    let list = mcp.request("tools/list", json!({}), Duration::from_secs(10));
    assert!(list.get("error").is_none(), "tools/list error: {list}");
    let tools = list["result"]["tools"].as_array().expect("tools array");
    let names: Vec<&str> = tools
        .iter()
        .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
        .collect();
    for expected in ["sail_check", "sail_hover", "sail_inspect", "sail_edit"] {
        assert!(names.contains(&expected), "missing {expected}: {names:?}");
    }

    let check = mcp.call_tool(
        "sail_check",
        json!({ "path": "routes/page.sail", "includeTs": true }),
    );
    assert!(
        check
            .get("diagnostics")
            .and_then(|d| d.as_array())
            .is_some(),
        "check diagnostics: {check}"
    );

    let (line, column) = pos_of(&src, "column");
    let hover = mcp.call_tool(
        "sail_hover",
        json!({
            "path": "routes/page.sail",
            "source": src,
            "line": line,
            "column": column,
        }),
    );
    let contents = hover
        .get("contents")
        .and_then(|c| c.as_str())
        .unwrap_or_default();
    assert!(
        contents.to_ascii_lowercase().contains("column"),
        "hover on created <column> must name the layout: {hover}"
    );

    let inspect = mcp.call_tool(
        "sail_inspect",
        json!({ "topic": "tree", "path": "routes/page.sail" }),
    );
    let nodes = inspect["nodes"].as_array().expect("inspect nodes");
    let tags: Vec<&str> = nodes.iter().filter_map(|n| n["tag"].as_str()).collect();
    assert!(
        tags.iter().any(|t| *t == "column"),
        "inspect tree must include column: {tags:?}\n{inspect}"
    );

    let before = std::fs::read(&page).expect("page bytes");
    let refuse = mcp.call_tool(
        "sail_edit",
        json!({
            "path": "routes/page.sail",
            "ops": [],
            "writeSource": true,
            "source": "<column style=\"display:flex\"><div>bad</div></column>\n",
        }),
    );
    assert_eq!(
        refuse["applied"], false,
        "bad writeSource must refuse: {refuse}"
    );
    assert!(
        refuse["diagnostics"]
            .as_array()
            .is_some_and(|d| d.iter().any(|row| {
                row["severity"]
                    .as_str()
                    .is_some_and(|s| s.eq_ignore_ascii_case("error"))
            })),
        "refused edit must return error diagnostics: {refuse}"
    );
    assert_eq!(
        std::fs::read(&page).expect("page after refuse"),
        before,
        "refused write must leave the created page unchanged"
    );

    let next = "<column gap=\"4\">\n  <h1>edited</h1>\n  <p>from sail_edit</p>\n</column>\n";
    let apply = mcp.call_tool(
        "sail_edit",
        json!({
            "path": "routes/page.sail",
            "ops": [],
            "writeSource": true,
            "source": next,
        }),
    );
    assert_eq!(
        apply["applied"], true,
        "good writeSource must apply: {apply}"
    );
    assert!(
        apply["diagnostics"].is_array(),
        "applied edit returns diagnostics: {apply}"
    );
    assert_eq!(
        std::fs::read_to_string(&page).expect("page after apply"),
        next
    );
}
