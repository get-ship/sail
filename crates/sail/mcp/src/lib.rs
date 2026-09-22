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
pub use install::{InstallReport, format_report, install, install_with_bin};
pub use sail_ops::authoring::{run_check as sail_check, sail_layout_check};
pub use sail_ops::{CheckDiag, SurfaceTopic, surface_markdown};

pub use http::{mcp_http_router, parse_bind, run_http, run_http_blocking};
pub use protocol::{
    PROTOCOL_VERSION, SUPPORTED_VERSIONS, call_tool, call_tool_with_ctx, handle_message,
    handle_message_with_ctx, list_tools, modern_meta, modern_request, tool_descriptors,
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
        let bind_spec = args
            .get(i + 1)
            .map(|s| s.as_str())
            .filter(|s| !s.starts_with("--"));
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
    use sail_ops::Ctx;
    use serde_json::json;

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
        assert!(names.contains(&"sail_theme_derive_scale"));
        assert!(names.contains(&"sail_theme_check"));
        assert!(names.contains(&"sail_theme_registry"));
    }

    #[test]
    fn derive_and_check_theme_call_engine_without_writes() {
        let ctx = Ctx::from_env();
        let derived = call_tool_with_ctx(
            "sail_theme_derive_scale",
            &json!({
                "anchors": {
                    "primary": { "space": "oklch", "l": 0.54, "c": 0.18, "h": 245 }
                },
                "mode": "light"
            }),
            &ctx,
        )
        .unwrap();
        assert_eq!(derived["ok"], true);
        let solid = derived["scales"]["primary"]["solid"].as_str().unwrap();
        assert!(solid.starts_with("oklch("), "{solid}");

        let mut theme = json!({
            "$extensions": { "sail": { "themeId": "mcp-probe", "defaultMode": "light", "layer": "theme" } },
            "anchors": {
                "primary": { "space": "oklch", "l": 0.45, "c": 0.12, "h": 250 },
                "accent": { "space": "oklch", "l": 0.45, "c": 0.16, "h": 300 },
                "secondary": { "space": "oklch", "l": 0.92, "c": 0.02, "h": 250 },
                "success": { "space": "oklch", "l": 0.62, "c": 0.15, "h": 150 },
                "warning": { "space": "oklch", "l": 0.72, "c": 0.15, "h": 75 },
                "info": { "space": "oklch", "l": 0.58, "c": 0.16, "h": 230 },
                "danger": { "space": "oklch", "l": 0.58, "c": 0.22, "h": 27 },
                "neutral": { "space": "oklch", "l": 0.92, "c": 0.01, "h": 250 }
            },
            "modes": {
                "light": {
                    "color": {
                        "background": { "$value": "oklch(0.98 0 0)", "$type": "color" },
                        "foreground": { "$value": "oklch(0.20 0 0)", "$type": "color" },
                        "muted": {
                            "default": { "$value": "oklch(0.92 0 0)", "$type": "color" },
                            "foreground": { "$value": "oklch(0.35 0 0)", "$type": "color" }
                        }
                    }
                }
            }
        });
        let ok = call_tool_with_ctx("sail_theme_check", &json!({ "theme": theme }), &ctx).unwrap();
        assert_eq!(ok["ok"], true, "{ok}");
        theme["modes"]["light"]["color"]["muted"]["foreground"]["$value"] =
            json!("oklch(0.85 0 0)");
        let bad = call_tool_with_ctx(
            "sail_theme_check",
            &json!({ "theme": theme, "derive": true }),
            &ctx,
        )
        .unwrap();
        assert_eq!(bad["ok"], false, "{bad}");
        assert!(
            bad["errors"][0].as_str().unwrap().contains("floor 4.5:1"),
            "{bad}"
        );

        let reg = call_tool_with_ctx(
            "sail_theme_registry",
            &json!({ "section": "catalog" }),
            &ctx,
        )
        .unwrap();
        assert_eq!(reg["schema"], "spinnaker.registry.v2");
        assert_eq!(reg["catalog"].as_array().unwrap().len(), 8);
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
        assert!(
            !names.contains(&"sail_apply"),
            "sail_apply must stay retired"
        );
    }
}
