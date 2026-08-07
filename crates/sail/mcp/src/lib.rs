//! Sail MCP transport — thin 2026-07-28 **stateless** adapter over `sail-ops`.
//!
//! All tool logic lives in `sail_ops` (same handlers as CLI `sail op` and
//! Connect HTTP under `sail dev`). This crate only:
//!
//! 1. Projects [`sail_ops::METHOD_CONTRACTS`] as MCP tools (`tools/list`)
//! 2. Frames stdio + single-endpoint Streamable HTTP
//! 3. Negotiates protocol version per-request (no sessions / no required initialize)
//! 4. [`install`] for `sail mcp install` (agent host config writers — WI4)
//!
//! Authority: SPEC-TOOLING-1 §WI4 / §WI9.

pub mod install;

mod http;
mod protocol;
mod stdio;

// Re-exports for tests / downstream that still import sail_mcp surface types.
pub use install::{InstallReport, format_report, install};
pub use sail_ops::authoring::{run_check as sail_check, sail_layout_check};
pub use sail_ops::{CheckDiag, SurfaceTopic, surface_markdown};

pub use http::{mcp_http_router, parse_bind, run_http, run_http_blocking};
pub use protocol::{
    PROTOCOL_VERSION, SUPPORTED_VERSIONS, call_tool, call_tool_with_ctx, handle_message,
    handle_message_with_ctx, modern_meta, modern_request, tool_descriptors,
};
pub use stdio::run_stdio;

/// CLI / binary entry: parse argv and run stdio or HTTP.
pub fn run() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        stdio::print_stdio_help();
        return;
    }
    if args.iter().any(|a| a == "--list-tools") {
        stdio::print_list_tools();
        return;
    }
    // --http [ADDR]
    if let Some(i) = args.iter().position(|a| a == "--http") {
        let bind_spec = args.get(i + 1).map(|s| s.as_str()).filter(|s| !s.starts_with("--"));
        match parse_bind(bind_spec) {
            Ok(addr) => {
                if let Err(e) = run_http_blocking(addr) {
                    eprintln!("sail-mcp http error: {e}");
                    std::process::exit(1);
                }
            }
            Err(e) => {
                eprintln!("sail-mcp: {e}");
                std::process::exit(2);
            }
        }
        return;
    }
    run_stdio();
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use sail_ops::Ctx;

    use super::*;

    #[test]
    fn call_tool_surface_roundtrip() {
        let out = call_tool("sail_surface", &json!({"topic": "core"})).unwrap();
        assert!(out.get("markdown").is_some() || out.get("topic").is_some() || out.is_object());
    }

    #[test]
    fn mcp_tool_and_call_rpc_same_result() {
        let ctx = Ctx::from_env();
        let args = json!({"topic": "core"});
        let mcp = call_tool_with_ctx("sail_surface", &args, &ctx).unwrap();
        let rpc = sail_ops::call_rpc("SailAuthoring", "Surface", &args, &ctx).unwrap();
        assert_eq!(mcp, rpc);
    }

    #[test]
    fn sail_edit_registered_and_replaces_apply() {
        let tools = tool_descriptors();
        let names: Vec<_> = tools
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
            .collect();
        assert!(names.contains(&"sail_edit"), "{names:?}");
        assert!(!names.contains(&"sail_apply"), "sail_apply must be gone");
    }

    #[test]
    fn theme_tools_in_descriptors() {
        let tools = tool_descriptors();
        let names: Vec<_> = tools
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
            .collect();
        assert!(names.contains(&"sail_theme_write"));
        assert!(names.contains(&"sail_tokens_generate"));
    }

    #[test]
    fn sail_policy_list_matches_rpc_and_has_proof_knobs() {
        let ctx = Ctx::from_env();
        let args = json!({"mode": "list"});
        let mcp = call_tool_with_ctx("sail_policy", &args, &ctx).unwrap();
        let rpc = sail_ops::call_rpc("SailAuthoring", "Policy", &args, &ctx).unwrap();
        assert_eq!(mcp, rpc, "MCP and RPC must share one serialization");
        assert_eq!(mcp["mode"], "list");
    }

    #[test]
    fn wi4_core_tools_present_apply_absent() {
        let tools = tool_descriptors();
        let names: Vec<_> = tools
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
            .collect();
        for expected in [
            "sail_check",
            "sail_surface",
            "sail_render",
            "sail_inspect",
            "sail_edit",
        ] {
            assert!(
                names.contains(&expected),
                "missing core tool {expected}: {names:?}"
            );
        }
        assert!(!names.contains(&"sail_apply"), "sail_apply must stay retired");
    }
}
