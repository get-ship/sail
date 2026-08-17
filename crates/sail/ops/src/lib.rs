//! Sail ops handlers (`sail.ops.v1`) — [`call_rpc`] / [`Ctx`] dispatch.
//!
//! Typed contracts live in [`contracts`]; codegen in [`codegen`]; watch SSE in [`watch`].

pub mod authoring;
pub mod codegen;
pub mod contracts;
mod ctx;
pub mod dev_server;
mod error;
mod fonts;
mod http;
pub mod identity;
pub mod inspect;
pub mod module;
pub mod project_store;
mod registry;
pub mod render;
pub mod task;
pub mod theme;
pub mod typed;
pub mod watch;

pub use authoring::{CheckDiag, SurfaceTopic, surface_markdown};
pub use contracts::METHOD_CONTRACTS;
pub use ctx::Ctx;
pub use error::OpError;
pub use http::{connect_router, connect_router_with_ctx};
pub use identity::{
    Anchor, ElementIdentity, ResolveStatus, ResolvedIdentity, SemanticLocator, SourceSpan,
    content_hash_from_node_id, element_path_from_node, parse_identity, resolve_identity,
    schema_example as identity_schema_example,
};
pub use inspect::InspectTopic;
pub use module::{
    ERR_API_UNSUPPORTED, ERR_CAPABILITY, ERR_UNSIGNED, LoadedModule, MODULE_API_VERSION,
    ModuleCallCtx, ModuleLoadError, ModuleManifest, ModuleMode, activity_surface_html,
    bootstrap_from_env, clear_modules, generate_module_ts_extension, install_modules,
    load_module_dir, loaded_modules, module_connect_paths, modules_json, rebootstrap,
};
pub use project_store::{
    CANONICAL_GITIGNORE, CRITICAL_GITIGNORE_LINES, EnsureGitignoreResult, ensure_project_store,
    ensure_sail_gitignore, ensure_sail_readme,
};
pub use registry::{
    MethodDesc, ServiceDesc, cli_methods_help, mcp_tool_descriptors, mcp_tool_to_rpc,
    method_contracts, method_table, rpc_methods,
};
pub use typed::{
    EmptyArgs, Json, MethodContract, generate_ts_client, mcp_descriptors_from, schema_catalog,
};
pub use watch::{
    WatchEvent, WatchHub, WatchKind, publish as publish_watch, publish_file, publish_file_under,
    relativize_watch_path,
};

use serde_json::{Value, json};

use crate::typed::parse_args;

/// Dispatch `service`/`method` with JSON args against `ctx`.
///
/// Args are deserialized into the method's typed contract (validation at the edge)
/// then handlers run. `service` accepts short names (`SailAuthoring`, `SailTheme`) or
/// fully qualified Connect service names (`sail.ops.v1.SailAuthoring`).
pub fn call_rpc(
    service: &str,
    method: &str,
    args_json: &Value,
    ctx: &Ctx,
) -> Result<Value, OpError> {
    let service = normalize_service(service);
    let method = method.trim();
    // Lazy-load modules from workspace/project discovery (once per process).
    module::ensure_bootstrapped(&ctx.workspace_root, &ctx.project_root);
    // Validate against typed Args where the contract exists (all methods).
    // Handlers still accept Value so dynamic/optional edge cases stay compatible;
    // missing required fields fail at serde deserialize.
    match (service.as_str(), method) {
        ("SailAuthoring", "Surface") => {
            let _a: contracts::SurfaceArgs = parse_args(args_json)?;
            authoring::surface_with_ctx(args_json, Some(ctx))
        }
        ("SailAuthoring", "Check") => {
            let _a: contracts::CheckArgs = parse_args(args_json)?;
            authoring::check(args_json, ctx)
        }
        ("SailAuthoring", "Hover") => {
            let _a: contracts::HoverArgs = parse_args(args_json)?;
            authoring::hover(args_json, ctx)
        }
        ("SailAuthoring", "Complete") => {
            let _a: contracts::CompleteArgs = parse_args(args_json)?;
            authoring::complete(args_json, ctx)
        }
        ("SailAuthoring", "Edit") => {
            let _a: contracts::EditArgs = parse_args(args_json)?;
            authoring::edit(args_json, ctx)
        }
        ("SailAuthoring", "LayoutCheck") => {
            let _a: contracts::LayoutCheckArgs = parse_args(args_json)?;
            authoring::layout_check(args_json)
        }
        ("SailAuthoring", "ListFiles") => {
            let _a: contracts::ListFilesArgs = parse_args(args_json)?;
            authoring::list_files(args_json, ctx)
        }
        ("SailAuthoring", "Policy") => {
            let _a: contracts::PolicyArgs = parse_args(args_json)?;
            authoring::policy(args_json, ctx)
        }
        ("SailInspect", "Inspect") => {
            let _a: contracts::InspectArgs = parse_args(args_json)?;
            inspect::inspect(args_json, ctx)
        }
        ("SailInspect", "Model") => {
            let _a: contracts::InspectModelArgs = parse_args(args_json)?;
            inspect::model(args_json, ctx)
        }
        ("SailInspect", "Tree") => {
            let _a: contracts::InspectTreeArgs = parse_args(args_json)?;
            inspect::tree(args_json, ctx)
        }
        ("SailInspect", "State") => {
            let _a: contracts::InspectStateArgs = parse_args(args_json)?;
            inspect::state(args_json, ctx)
        }
        ("SailInspect", "TokenUsage") => {
            let _a: contracts::InspectTokenUsageArgs = parse_args(args_json)?;
            inspect::token_usage(args_json, ctx)
        }
        ("SailTheme", "Graph") => {
            let _a: contracts::ThemeGraphArgs = parse_args(args_json)?;
            theme::graph(args_json, ctx)
        }
        ("SailTheme", "List") => {
            let _a: contracts::ThemeListArgs = parse_args(args_json)?;
            theme::list(args_json, ctx)
        }
        ("SailTheme", "Read") => {
            let _a: contracts::ThemeReadArgs = parse_args(args_json)?;
            theme::read(args_json, ctx)
        }
        ("SailTheme", "Write") => {
            let _a: contracts::ThemeWriteArgs = parse_args(args_json)?;
            theme::write(args_json, ctx)
        }
        ("SailTheme", "Delete") => {
            let _a: contracts::ThemeDeleteArgs = parse_args(args_json)?;
            theme::delete(args_json, ctx)
        }
        ("SailTheme", "Fork") => {
            // Accept source/dest aliases before strict typed parse.
            let normalized = normalize_fork_args(args_json);
            let _a: contracts::ThemeForkArgs = parse_args(&normalized)?;
            theme::fork(&normalized, ctx)
        }
        ("SailTheme", "Generate") => {
            let _a: contracts::ThemeGenerateArgs = parse_args(args_json)?;
            theme::generate(args_json, ctx)
        }
        ("SailTheme", "EffectiveIds") => {
            let _a: contracts::ThemeEffectiveIdsArgs = parse_args(args_json)?;
            theme::effective_ids(args_json, ctx)
        }
        ("SailTheme", "DeriveScale") => {
            let _a: contracts::ThemeDeriveScaleArgs = parse_args(args_json)?;
            theme::derive_scale(args_json, ctx)
        }
        ("SailTheme", "CheckTheme") => {
            let _a: contracts::ThemeCheckArgs = parse_args(args_json)?;
            theme::check_theme(args_json, ctx)
        }
        ("SailTheme", "Registry") => {
            let _a: contracts::ThemeRegistryArgs = parse_args(args_json)?;
            theme::registry(args_json, ctx)
        }
        ("SailTheme", "FontSearch") => {
            let _a: contracts::ThemeFontSearchArgs = parse_args(args_json)?;
            fonts::font_search(args_json, ctx)
        }
        ("SailTheme", "FontAdd") => {
            let _a: contracts::ThemeFontAddArgs = parse_args(args_json)?;
            fonts::font_add(args_json, ctx)
        }
        ("SailDevServer", "Start") => {
            let _a: contracts::DevServerStartArgs = parse_args(args_json)?;
            dev_server::start(args_json, ctx)
        }
        ("SailDevServer", "Status") => {
            let _a: typed::EmptyArgs = parse_args(args_json)?;
            dev_server::status(args_json, ctx)
        }
        ("SailDevServer", "Stop") => {
            let _a: typed::EmptyArgs = parse_args(args_json)?;
            dev_server::stop(args_json, ctx)
        }
        ("SailTask", "List") => {
            let _a: contracts::TaskListArgs = parse_args(args_json)?;
            task::list(args_json, ctx)
        }
        ("SailTask", "Get") => {
            let _a: contracts::TaskGetArgs = parse_args(args_json)?;
            task::get(args_json, ctx)
        }
        ("SailTask", "Create") => {
            let _a: contracts::TaskCreateArgs = parse_args(args_json)?;
            task::create(args_json, ctx)
        }
        ("SailTask", "Update") => {
            let _a: contracts::TaskUpdateArgs = parse_args(args_json)?;
            task::update(args_json, ctx)
        }
        ("SailTask", "Comment") => {
            let _a: contracts::TaskCommentArgs = parse_args(args_json)?;
            task::comment(args_json, ctx)
        }
        ("SailTask", "AddArtifact") => {
            let _a: contracts::TaskAddArtifactArgs = parse_args(args_json)?;
            task::add_artifact(args_json, ctx)
        }
        ("SailTask", "AgentNote") => {
            let _a: contracts::TaskAgentNoteArgs = parse_args(args_json)?;
            task::agent_note(args_json, ctx)
        }
        ("SailTask", "ThreadCreate") => {
            let _a: contracts::TaskThreadCreateArgs = parse_args(args_json)?;
            task::thread_create(args_json, ctx)
        }
        ("SailTask", "ThreadReply") => {
            let _a: contracts::TaskThreadReplyArgs = parse_args(args_json)?;
            task::thread_reply(args_json, ctx)
        }
        ("SailTask", "ThreadList") => {
            let _a: contracts::TaskThreadListArgs = parse_args(args_json)?;
            task::thread_list(args_json, ctx)
        }
        ("SailTask", "Reanchor") => {
            let _a: contracts::TaskReanchorArgs = parse_args(args_json)?;
            task::reanchor(args_json, ctx)
        }
        ("SailTask", "CommitIntent") => {
            let _a: contracts::TaskCommitIntentArgs = parse_args(args_json)?;
            task::commit_intent(args_json, ctx)
        }
        ("SailRender", "Screenshot") => {
            let _a: contracts::RenderScreenshotArgs = parse_args(args_json)?;
            render::screenshot(args_json, ctx)
        }
        _ => {
            // Module registry extension (TOOL-11) — same transports as core.
            if let Some(result) = module::call_module_rpc(&service, method, args_json) {
                return result;
            }
            Err(OpError::NotFound(format!(
                "unknown RPC `{service}/{method}`"
            )))
        }
    }
}

fn normalize_fork_args(args: &Value) -> Value {
    let mut obj = args.as_object().cloned().unwrap_or_default();
    if !obj.contains_key("source_path") {
        if let Some(v) = obj.get("source").cloned() {
            obj.insert("source_path".into(), v);
        }
    }
    if !obj.contains_key("dest_path") {
        if let Some(v) = obj.get("dest").cloned() {
            obj.insert("dest_path".into(), v);
        }
    }
    Value::Object(obj)
}

/// Parse a Connect path `/sail.ops.v1.Service/Method` into (service, method).
pub fn parse_connect_path(path: &str) -> Option<(String, String)> {
    let path = path.trim().trim_start_matches('/');
    let (svc, method) = path.rsplit_once('/')?;
    if svc.is_empty() || method.is_empty() {
        return None;
    }
    Some((normalize_service(svc), method.to_string()))
}

/// Connect unary path for a service/method pair.
pub fn connect_path(service: &str, method: &str) -> String {
    let svc = normalize_service(service);
    format!("/sail.ops.v1.{svc}/{method}")
}

fn normalize_service(service: &str) -> String {
    let s = service.trim().trim_start_matches('/');
    if let Some(rest) = s.strip_prefix("sail.ops.v1.") {
        rest.to_string()
    } else {
        s.to_string()
    }
}

/// MCP-facing tool dispatch: map tool name → same handler as Connect/CLI.
pub fn call_mcp_tool(name: &str, args: &Value, ctx: &Ctx) -> Result<Value, OpError> {
    module::ensure_bootstrapped(&ctx.workspace_root, &ctx.project_root);
    let (service, method) = match mcp_tool_to_rpc(name) {
        Some((s, m)) => (s.to_string(), m.to_string()),
        None => module::module_mcp_tool_to_rpc(name)
            .ok_or_else(|| OpError::NotFound(format!("unknown tool `{name}`")))?,
    };
    // Task tools default actor to agent when omitted (HTTP/UI defaults human).
    let args_owned;
    let args = if service == "SailTask" && !matches!(method.as_str(), "List" | "Get" | "ThreadList")
    {
        args_owned = task::inject_mcp_actor_default(args);
        &args_owned
    } else {
        args
    };
    call_rpc(&service, &method, args, ctx)
}

/// Convenience: default [`Ctx`] from env + cwd (MCP/CLI style).
pub fn default_ctx() -> Ctx {
    Ctx::from_env()
}

/// Stable health payload (optional diagnostics).
pub fn health_json(ctx: &Ctx) -> Value {
    module::ensure_bootstrapped(&ctx.workspace_root, &ctx.project_root);
    let mut methods = rpc_methods();
    methods.extend(module::module_rpc_methods());
    methods.sort();
    methods.dedup();
    json!({
        "ok": true,
        "package": "sail.ops.v1",
        "workspace_root": ctx.workspace_root.display().to_string(),
        "project_root": ctx.project_root.display().to_string(),
        "apply_root": ctx.apply_root.display().to_string(),
        "methods": methods,
        "modules": module::modules_json(),
    })
}

/// Run codegen (`sail op gen`).
pub fn op_gen(workspace_root: Option<&std::path::Path>) -> Result<Vec<std::path::PathBuf>, String> {
    let root = workspace_root
        .map(|p| p.to_path_buf())
        .unwrap_or_else(codegen::find_workspace_root);
    let paths = codegen::GenPaths::from_workspace(root);
    codegen::generate_all(&paths)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn connect_path_roundtrip() {
        let p = connect_path("SailTheme", "Write");
        assert_eq!(p, "/sail.ops.v1.SailTheme/Write");
        let (s, m) = parse_connect_path(&p).unwrap();
        assert_eq!(s, "SailTheme");
        assert_eq!(m, "Write");
    }

    #[test]
    fn mcp_and_call_rpc_same_surface() {
        let ctx = Ctx::from_env();
        let args = json!({"topic": "core"});
        let via_rpc = call_rpc("SailAuthoring", "Surface", &args, &ctx).unwrap();
        let via_mcp = call_mcp_tool("sail_surface", &args, &ctx).unwrap();
        assert_eq!(via_rpc, via_mcp);
        assert_eq!(via_rpc["topic"], "core");
        assert!(via_rpc["markdown"].as_str().unwrap().contains("Hard rules"));
    }

    #[test]
    fn mcp_tool_descriptors_include_theme() {
        let tools = mcp_tool_descriptors();
        let arr = tools.as_array().unwrap();
        let names: Vec<_> = arr
            .iter()
            .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
            .collect();
        assert!(names.contains(&"sail_surface"));
        assert!(names.contains(&"sail_theme_list"));
        assert!(names.contains(&"sail_theme_write"));
        assert!(names.contains(&"sail_tokens_generate"));
        assert!(names.contains(&"sail_theme_effective_ids"));
        assert!(names.contains(&"sail_theme_derive_scale"));
        assert!(names.contains(&"sail_theme_check"));
        assert!(names.contains(&"sail_theme_registry"));
        assert!(names.contains(&"sail_dev_server_start"));
        assert!(names.contains(&"sail_dev_server_status"));
        assert!(names.contains(&"sail_dev_server_stop"));
        assert!(names.contains(&"sail_task_list"));
        assert!(names.contains(&"sail_task_create"));
        assert!(names.contains(&"sail_task_agent_note"));
    }

    #[test]
    fn schema_snapshot_per_method() {
        for c in METHOD_CONTRACTS {
            let snap = typed::method_schema_snapshot(c);
            assert!(
                snap.contains(c.service) && snap.contains(c.method),
                "snapshot for {}/{}",
                c.service,
                c.method
            );
            let v: Value = serde_json::from_str(&snap).expect("schema snapshot json");
            assert!(v.get("inputSchema").is_some(), "{}", c.mcp_tool);
            assert!(v.get("outputSchema").is_some(), "{}", c.mcp_tool);
        }
    }

    #[test]
    fn input_schemas_resolve_all_refs() {
        for c in METHOD_CONTRACTS {
            let input = (c.input_schema)();
            typed::schema_refs_resolve(&input).unwrap_or_else(|e| {
                panic!("{} inputSchema: {e}\n{}", c.mcp_tool, input);
            });
        }
    }

    #[test]
    fn mcp_stdio_descriptors_match_committed_artifact() {
        // Non-tautological: core descriptors == committed mcp-tools.json from `sail op gen`.
        // Module tools are additive at runtime and must not drift the committed SSOT.
        let from_registry = registry::mcp_tool_descriptors_core();
        let root = codegen::find_workspace_root();
        let path = root.join("crates/sail/ops/schemas/mcp-tools.json");
        let committed = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("missing {}; run `sail op gen`: {e}", path.display()));
        let from_file: Value = serde_json::from_str(&committed).expect("mcp-tools.json");
        assert_eq!(
            from_registry, from_file,
            "MCP tools/list must match committed schemas/mcp-tools.json"
        );
        assert_eq!(from_registry, typed::mcp_descriptors_from(METHOD_CONTRACTS));
    }

    #[test]
    fn typed_args_reject_missing_required() {
        let ctx = Ctx::from_env();
        let err = call_rpc("SailAuthoring", "Surface", &json!({}), &ctx).unwrap_err();
        assert!(matches!(err, OpError::InvalidArgument(_)));
    }

    #[test]
    fn call_rpc_unknown_method_not_found() {
        let ctx = Ctx::from_env();
        let err = call_rpc("Nope", "X", &json!({}), &ctx).unwrap_err();
        assert!(matches!(err, OpError::NotFound(_)), "{err}");
    }

    #[test]
    fn call_mcp_unknown_tool_not_found() {
        let ctx = Ctx::from_env();
        let err = call_mcp_tool("not_a_tool", &json!({}), &ctx).unwrap_err();
        assert!(matches!(err, OpError::NotFound(_)), "{err}");
    }

    #[test]
    fn ts_args_fields_match_input_schema_properties() {
        // T-I4: hand-written ts_args_body must match schemars property names.
        let ts = typed::generate_ts_client(METHOD_CONTRACTS);
        for c in METHOD_CONTRACTS {
            let schema = (c.input_schema)();
            let schema_props = typed::schema_property_names(&schema);
            let ts_fields = typed::ts_interface_field_names(&ts, c.ts_args);
            // Schema may list optional null-wrapped fields; TS bodies may omit empty
            // interfaces (EmptyArgs). Compare set equality of declared fields.
            assert_eq!(
                schema_props,
                ts_fields,
                "{} / {} schema props vs TS fields",
                c.rpc_name(),
                c.ts_args
            );
        }
    }

    #[test]
    fn generated_ts_client_matches_committed() {
        let root = codegen::find_workspace_root();
        let paths = codegen::GenPaths::from_workspace(root);
        codegen::assert_no_drift(&paths).expect("run `sail op gen` to refresh client/schemas");
    }

    /// TOOLING-1 WI4 — core agent MCP tools are a forever contract.
    ///
    /// House names (not retired sail_apply):
    /// sail_check | sail_surface | sail_render | sail_inspect | sail_edit
    ///
    /// Pins each tool's input+output schema snapshot and mcp-tools descriptor.
    #[test]
    fn wi4_core_mcp_schema_drift() {
        /// Forever agent-facing MCP surface (SPEC-TOOLING-1 §WI4, house names).
        const CORE_MCP_TOOLS: &[&str] = &[
            "sail_check",
            "sail_surface",
            "sail_render",
            "sail_inspect",
            "sail_edit",
        ];

        let root = codegen::find_workspace_root();
        let schema_dir = root.join("crates/sail/ops/schemas");
        let mcp_tools_path = schema_dir.join("mcp-tools.json");
        let committed_mcp: Value = serde_json::from_str(
            &std::fs::read_to_string(&mcp_tools_path)
                .unwrap_or_else(|e| panic!("missing {}: {e}", mcp_tools_path.display())),
        )
        .expect("mcp-tools.json");

        let names: Vec<&str> = committed_mcp
            .as_array()
            .expect("mcp-tools array")
            .iter()
            .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
            .collect();
        assert!(
            !names.contains(&"sail_apply"),
            "sail_apply is retired; house write path is sail_edit"
        );

        for tool in CORE_MCP_TOOLS {
            let contract = METHOD_CONTRACTS
                .iter()
                .find(|c| c.mcp_tool == *tool)
                .unwrap_or_else(|| panic!("core tool {tool} missing from METHOD_CONTRACTS"));

            // Full method snapshot (inputSchema + outputSchema) is committed.
            let snap_name = format!("{}_{}.json", contract.service, contract.method);
            let snap_path = schema_dir.join(&snap_name);
            let expected = format!("{}\n", typed::method_schema_snapshot(contract));
            let actual = std::fs::read_to_string(&snap_path)
                .unwrap_or_else(|e| panic!("missing schema snapshot {}: {e}", snap_path.display()));
            assert_eq!(
                actual, expected,
                "WI4 schema drift on {tool}: run `sail op gen`"
            );

            // tools/list descriptor row matches live generation.
            let live = (contract.input_schema)();
            let row = committed_mcp
                .as_array()
                .unwrap()
                .iter()
                .find(|t| t.get("name").and_then(|n| n.as_str()) == Some(*tool))
                .unwrap_or_else(|| panic!("{tool} missing from mcp-tools.json"));
            assert_eq!(
                row.get("inputSchema").cloned().unwrap_or(Value::Null),
                live,
                "WI4 mcp-tools inputSchema drift on {tool}"
            );
            // Output schema is the forever agent return contract.
            let out = (contract.output_schema)();
            assert!(
                out.get("type").is_some() || out.get("$schema").is_some() || out.is_object(),
                "{tool} outputSchema must be object-shaped"
            );
            let snap_v: Value = serde_json::from_str(actual.trim()).expect("snap json");
            assert_eq!(
                snap_v.get("outputSchema").cloned().unwrap_or(Value::Null),
                out,
                "WI4 outputSchema pin on {tool}"
            );
        }
    }

    #[test]
    fn call_rpc_covers_every_contract() {
        // Exhaustiveness: every contract is dispatchable (may fail validation, not NotFound).
        let ctx = Ctx::from_env();
        for c in METHOD_CONTRACTS {
            let err = call_rpc(c.service, c.method, &json!({}), &ctx).err();
            if let Some(e) = err {
                assert!(
                    !matches!(e, OpError::NotFound(_)),
                    "contract {}/{} not wired in call_rpc: {e}",
                    c.service,
                    c.method
                );
            }
        }
    }
}
