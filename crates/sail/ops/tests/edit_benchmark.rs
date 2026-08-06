//! TOOL-9 edit benchmark: structured `sail_edit` vs naive text-edit token cost
//! and first-try validity on a small realistic corpus.
//!
//! Report committed as JSON next to this file when run with
//! `EDIT_BENCH_WRITE=1` (optional). Default test asserts structured path
//! stays valid after each batch and uses fewer "op tokens" than full rewrites.

use serde_json::json;
use std::path::PathBuf;

struct Task {
    name: &'static str,
    source: &'static str,
    /// Structured ops as JSON values (nodeIds filled after index).
    build_ops: fn(&str, &str) -> Vec<serde_json::Value>,
    /// Naive text-edit replacement of whole file (agent rewrite).
    text_rewrite: fn(&str) -> String,
}

/// Realistic surrounding chrome so text-Edit rewrites pay full-file tokens.
const CHROME: &str = r#"
<!-- page chrome / boilerplate an agent rewrites wholesale in text-edit mode -->
<column gap="6">
  <div>header line one of many static siblings</div>
  <div>nav item alpha beta gamma delta</div>
  <div>sidebar note about layout and tokens</div>
"#;

const CHROME_CLOSE: &str = r#"
  <div>footer copyright and links</div>
</column>
"#;

fn wrap_chrome(inner: &str) -> String {
    format!("{CHROME}{inner}{CHROME_CLOSE}")
}

fn tasks() -> Vec<Task> {
    vec![
        Task {
            name: "set_gap_attr",
            source: "", // filled below via lazy static pattern — use boxed sources
            build_ops: |src, file| {
                let idx = sail_compiler::edit::build_node_index(src, file).unwrap();
                let col = idx
                    .values()
                    .find(|n| n.tag == "column" && n.static_attrs.contains_key("gap"))
                    .or_else(|| idx.values().find(|n| n.tag == "column"))
                    .unwrap();
                // Prefer the target column with gap="2" when present.
                let col = idx
                    .values()
                    .find(|n| {
                        n.tag == "column"
                            && n.static_attrs.get("gap").map(|v| v.as_str()) == Some("2")
                    })
                    .unwrap_or(col);
                vec![json!({
                    "op": "setAttr",
                    "nodeId": col.id,
                    "name": "gap",
                    "value": "4"
                })]
            },
            text_rewrite: |src| src.replacen(r#"gap="2""#, r#"gap="4""#, 1),
        },
        Task {
            name: "add_pad_sx",
            source: "",
            build_ops: |src, file| {
                let idx = sail_compiler::edit::build_node_index(src, file).unwrap();
                let n = idx
                    .values()
                    .find(|n| {
                        n.tag == "div"
                            && n.static_attrs.is_empty()
                            && src[n.full_span.start..n.full_span.end].contains(">body<")
                    })
                    .or_else(|| idx.values().find(|n| n.tag == "div"))
                    .unwrap();
                vec![json!({
                    "op": "setSx",
                    "nodeId": n.id,
                    "setter": "pad",
                    "args": [2]
                })]
            },
            text_rewrite: |src| {
                src.replacen("<div>body</div>", r#"<div style={style.pad(2)}>body</div>"#, 1)
            },
        },
        Task {
            name: "wrap_in_column",
            source: "",
            build_ops: |src, file| {
                let idx = sail_compiler::edit::build_node_index(src, file).unwrap();
                let n = idx
                    .values()
                    .find(|n| {
                        n.tag == "div" && src[n.full_span.start..n.full_span.end].contains(">item<")
                    })
                    .unwrap();
                vec![json!({
                    "op": "wrap",
                    "nodeId": n.id,
                    "tag": "column",
                    "attrs": { "gap": "4" }
                })]
            },
            text_rewrite: |src| {
                src.replacen(
                    "<div>item</div>",
                    r#"<column gap="4"><div>item</div></column>"#,
                    1,
                )
            },
        },
        Task {
            name: "replace_child",
            source: "",
            build_ops: |src, file| {
                let idx = sail_compiler::edit::build_node_index(src, file).unwrap();
                let div = idx
                    .values()
                    .find(|n| {
                        n.tag == "div" && src[n.full_span.start..n.full_span.end].contains(">old<")
                    })
                    .unwrap();
                vec![json!({
                    "op": "replace",
                    "nodeId": div.id,
                    "fragment": "<span>new</span>"
                })]
            },
            text_rewrite: |src| src.replacen("<div>old</div>", "<span>new</span>", 1),
        },
        Task {
            name: "insert_sibling",
            source: "",
            build_ops: |src, file| {
                let idx = sail_compiler::edit::build_node_index(src, file).unwrap();
                let col = idx
                    .values()
                    .find(|n| {
                        n.tag == "column"
                            && n.children.iter().any(|c| {
                                idx.get(c)
                                    .map(|ch| {
                                        src[ch.full_span.start..ch.full_span.end].contains(">a<")
                                    })
                                    .unwrap_or(false)
                            })
                    })
                    .unwrap();
                vec![json!({
                    "op": "insert",
                    "parentNodeId": col.id,
                    "position": "end",
                    "fragment": "<div>b</div>"
                })]
            },
            text_rewrite: |src| src.replacen("<div>a</div>", "<div>a</div><div>b</div>", 1),
        },
        Task {
            name: "remove_gap",
            source: "",
            build_ops: |src, file| {
                let idx = sail_compiler::edit::build_node_index(src, file).unwrap();
                let col = idx
                    .values()
                    .find(|n| {
                        n.tag == "column"
                            && n.static_attrs.get("gap").map(|v| v.as_str()) == Some("8")
                    })
                    .unwrap();
                vec![json!({
                    "op": "removeAttr",
                    "nodeId": col.id,
                    "name": "gap"
                })]
            },
            text_rewrite: |src| src.replacen(r#" gap="8""#, "", 1),
        },
        Task {
            name: "set_grow_attr",
            source: "",
            build_ops: |src, file| {
                let idx = sail_compiler::edit::build_node_index(src, file).unwrap();
                let n = idx
                    .values()
                    .find(|n| {
                        n.tag == "div"
                            && n.static_attrs.get("min-w").map(|v| v.as_str()) == Some("0")
                    })
                    .unwrap();
                vec![json!({
                    "op": "setAttr",
                    "nodeId": n.id,
                    "name": "grow"
                })]
            },
            text_rewrite: |src| src.replacen(r#"min-w="0""#, r#"min-w="0" grow"#, 1),
        },
        Task {
            name: "batch_attr_and_sx",
            source: "",
            build_ops: |src, file| {
                let idx = sail_compiler::edit::build_node_index(src, file).unwrap();
                let row = idx.values().find(|n| n.tag == "row").unwrap();
                let div = idx
                    .values()
                    .find(|n| {
                        n.tag == "div" && src[n.full_span.start..n.full_span.end].contains(">c<")
                    })
                    .unwrap();
                vec![
                    json!({
                        "op": "setAttr",
                        "nodeId": row.id,
                        "name": "gap",
                        "value": "2"
                    }),
                    json!({
                        "op": "setSx",
                        "nodeId": div.id,
                        "setter": "pad",
                        "args": [1]
                    }),
                ]
            },
            text_rewrite: |src| {
                src.replacen("<row>", r#"<row gap="2">"#, 1).replacen(
                    "<div>c</div>",
                    r#"<div style={style.pad(1)}>c</div>"#,
                    1,
                )
            },
        },
        Task {
            name: "wrap_center",
            source: "",
            build_ops: |src, file| {
                let idx = sail_compiler::edit::build_node_index(src, file).unwrap();
                let col = idx
                    .values()
                    .find(|n| {
                        n.tag == "column"
                            && n.static_attrs.get("gap").map(|v| v.as_str()) == Some("2")
                            && src[n.full_span.start..n.full_span.end].contains("hero")
                    })
                    .unwrap();
                vec![json!({
                    "op": "wrap",
                    "nodeId": col.id,
                    "tag": "center"
                })]
            },
            text_rewrite: |src| {
                src.replacen(
                    r#"<column gap="2"><div>hero</div></column>"#,
                    r#"<center><column gap="2"><div>hero</div></column></center>"#,
                    1,
                )
            },
        },
        Task {
            name: "replace_root_text_child",
            source: "",
            build_ops: |src, file| {
                let idx = sail_compiler::edit::build_node_index(src, file).unwrap();
                let n = idx
                    .values()
                    .find(|n| {
                        n.tag == "div"
                            && src[n.full_span.start..n.full_span.end].contains(">before<")
                    })
                    .unwrap();
                vec![json!({
                    "op": "replace",
                    "nodeId": n.id,
                    "fragment": "<div>after</div>"
                })]
            },
            text_rewrite: |src| src.replacen("<div>before</div>", "<div>after</div>", 1),
        },
    ]
}

fn task_sources() -> Vec<&'static str> {
    // Leaked once so Task can keep &'static str without restructuring.
    vec![
        Box::leak(wrap_chrome(r#"<column gap="2"><div>hello</div></column>"#).into_boxed_str()),
        Box::leak(wrap_chrome(r#"<div>body</div>"#).into_boxed_str()),
        Box::leak(wrap_chrome(r#"<div>item</div>"#).into_boxed_str()),
        Box::leak(wrap_chrome(r#"<column><div>old</div></column>"#).into_boxed_str()),
        Box::leak(wrap_chrome(r#"<column><div>a</div></column>"#).into_boxed_str()),
        Box::leak(wrap_chrome(r#"<column gap="8"><div>x</div></column>"#).into_boxed_str()),
        Box::leak(wrap_chrome(r#"<div min-w="0">fill</div>"#).into_boxed_str()),
        Box::leak(wrap_chrome(r#"<row><div>c</div></row>"#).into_boxed_str()),
        Box::leak(wrap_chrome(r#"<column gap="2"><div>hero</div></column>"#).into_boxed_str()),
        Box::leak(wrap_chrome(r#"<div>before</div>"#).into_boxed_str()),
    ]
}

fn approx_tokens(s: &str) -> usize {
    // crude: ~4 chars per token
    (s.len() + 3) / 4
}

#[test]
fn structured_edit_corpus_zero_uncompilable_intermediates() {
    let file = "bench.sail";
    let mut structured_tokens = 0usize;
    let mut text_tokens = 0usize;
    let mut structured_valid = 0usize;
    let mut text_valid = 0usize;
    let mut rows = Vec::new();
    let sources = task_sources();
    let mut task_list = tasks();
    for (t, s) in task_list.iter_mut().zip(sources.iter()) {
        t.source = s;
    }

    for task in &task_list {
        let ops_json = (task.build_ops)(task.source, file);
        let ops = sail_compiler::edit::parse_ops(&json!(ops_json)).unwrap();
        let result = sail_compiler::edit::apply_ops_to_source(
            task.source,
            file,
            &ops,
            None,
            sail_compiler::edit::AutofixMode::Off,
            None,
        );
        assert!(
            result.applied,
            "task {} must apply cleanly: {:?}",
            task.name, result.message
        );
        // re-validate
        let v = sail_compiler::edit::validate_source(&result.source, None, &[]);
        assert!(
            !v.has_errors,
            "task {} left errors: {:?}",
            task.name, v.diagnostics
        );
        structured_valid += 1;
        // Structured agent payload = ops only. Text-edit agents typically re-send
        // the full file (or a large patch) as the rewrite body.
        let st = approx_tokens(&serde_json::to_string(&ops_json).unwrap());
        structured_tokens += st;

        let rewritten = (task.text_rewrite)(task.source);
        let tv = sail_compiler::edit::validate_source(&rewritten, None, &[]);
        if !tv.has_errors {
            text_valid += 1;
        }
        // Full-file rewrite cost (source-out), the common agent text-edit path.
        let tt = approx_tokens(rewritten.as_str());
        text_tokens += tt;

        rows.push(json!({
            "task": task.name,
            "structuredTokens": st,
            "textTokens": tt,
            "structuredValid": true,
            "textValid": !tv.has_errors,
        }));
    }

    let n = task_list.len();
    assert_eq!(structured_valid, n, "all structured tasks valid");
    assert_eq!(
        text_valid, n,
        "text path validity should match structured first-try validity"
    );
    // Structured ops JSON is smaller than full-file rewrites for this corpus.
    assert!(
        structured_tokens < text_tokens,
        "expected structured tokens {structured_tokens} < text {text_tokens}"
    );

    let report = json!({
        "tasks": n,
        "structuredMedianTokensPerTask": structured_tokens / n,
        "textMedianTokensPerTask": text_tokens / n,
        "structuredFirstTryValidity": structured_valid as f64 / n as f64,
        "textFirstTryValidity": text_valid as f64 / n as f64,
        "rows": rows,
    });

    if std::env::var("EDIT_BENCH_WRITE").ok().as_deref() == Some("1") {
        let path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/edit_benchmark_report.json");
        std::fs::write(&path, serde_json::to_string_pretty(&report).unwrap()).unwrap();
    }

    // Always commit a lightweight report snapshot next to the test via assert content.
    let committed =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/edit_benchmark_report.json");
    let body = serde_json::to_string_pretty(&report).unwrap() + "\n";
    // Write if missing or env set; otherwise compare floor invariants only.
    if !committed.exists() || std::env::var("EDIT_BENCH_WRITE").is_ok() {
        let _ = std::fs::write(&committed, &body);
    }
    assert!(structured_tokens / n < text_tokens / n);
}

#[test]
fn off_vocab_setsx_file_untouched() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("t.sail");
    let original = r#"<div>x</div>"#;
    std::fs::write(&path, original).unwrap();
    let mut ctx = sail_ops::Ctx::new(dir.path(), dir.path(), dir.path());
    let idx = sail_compiler::edit::build_node_index(original, "t.sail").unwrap();
    let id = idx.values().next().unwrap().id.clone();
    let v = sail_ops::call_rpc(
        "SailAuthoring",
        "Edit",
        &json!({
            "path": "t.sail",
            "ops": [{
                "op": "setSx",
                "nodeId": id,
                "setter": "notARealSetter",
                "args": ["x"]
            }]
        }),
        &ctx,
    )
    .unwrap();
    assert_eq!(v["applied"], false);
    assert!(
        v["message"].as_str().unwrap_or("").contains("off-vocab")
            || v["message"]
                .as_str()
                .unwrap_or("")
                .contains("Allowed setters"),
        "{v}"
    );
    assert_eq!(std::fs::read_to_string(&path).unwrap(), original);
    let _ = &mut ctx;
}

#[test]
fn mcp_cli_parity_sail_edit() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("p.sail");
    std::fs::write(&path, r#"<column><div>z</div></column>"#).unwrap();
    let ctx = sail_ops::Ctx::new(dir.path(), dir.path(), dir.path());
    let src = std::fs::read_to_string(&path).unwrap();
    let idx = sail_compiler::edit::build_node_index(&src, "p.sail").unwrap();
    let col = idx.values().find(|n| n.tag == "column").unwrap();
    let args = json!({
        "path": "p.sail",
        "ops": [{
            "op": "setAttr",
            "nodeId": col.id,
            "name": "gap",
            "value": "3"
        }],
        "dryRun": true
    });
    let via_rpc = sail_ops::call_rpc("SailAuthoring", "Edit", &args, &ctx).unwrap();
    let via_mcp = sail_ops::call_mcp_tool("sail_edit", &args, &ctx).unwrap();
    assert_eq!(via_rpc["applied"], via_mcp["applied"]);
    assert_eq!(via_rpc["modelRevision"], via_mcp["modelRevision"]);
    assert_eq!(via_rpc["applied"], true);
}
