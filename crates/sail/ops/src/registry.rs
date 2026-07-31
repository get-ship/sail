//! Method + MCP tool registry (single table for all transports).
//!
//! Descriptors and MCP `inputSchema` are derived from [`crate::contracts::METHOD_CONTRACTS`]
//! (typed Rust SSOT). Do not hand-maintain parallel JSON schemas.

use serde_json::Value;

use crate::contracts::METHOD_CONTRACTS;
use crate::typed::{MethodContract, cli_help_text, mcp_descriptors_from};

/// One RPC method under a service (view over [`METHOD_CONTRACTS`]).
#[derive(Debug, Clone, Copy)]
pub struct MethodDesc {
    pub service: &'static str,
    pub method: &'static str,
    pub mcp_tool: &'static str,
    pub description: &'static str,
}

impl MethodDesc {
    pub const fn from_contract(c: &MethodContract) -> Self {
        Self {
            service: c.service,
            method: c.method,
            mcp_tool: c.mcp_tool,
            description: c.description,
        }
    }
}

/// Service descriptor for docs / health.
#[derive(Debug, Clone, Copy)]
pub struct ServiceDesc {
    pub name: &'static str,
    pub methods: &'static [&'static str],
}

/// Method table derived from contracts (same order / descriptions as MCP/CLI).
///
/// Prefer iterating [`method_contracts`] / [`METHOD_CONTRACTS`] for new code.
pub fn method_table() -> Vec<MethodDesc> {
    METHOD_CONTRACTS
        .iter()
        .map(MethodDesc::from_contract)
        .collect()
}

/// Map MCP tool name → (service, method). Core table only; modules via
/// [`crate::module::module_mcp_tool_to_rpc`].
pub fn mcp_tool_to_rpc(name: &str) -> Option<(&'static str, &'static str)> {
    METHOD_CONTRACTS
        .iter()
        .find(|m| m.mcp_tool == name)
        .map(|m| (m.service, m.method))
}

/// List of `"Service/Method"` strings for health/docs (core only).
pub fn rpc_methods() -> Vec<String> {
    METHOD_CONTRACTS
        .iter()
        .map(|m| format!("{}/{}", m.service, m.method))
        .collect()
}

/// MCP `tools/list` descriptors for **core** contracts only (codegen / drift SSOT).
pub fn mcp_tool_descriptors_core() -> Value {
    mcp_descriptors_from(METHOD_CONTRACTS)
}

/// MCP `tools/list` descriptors — core contracts + loaded module tools (live transport).
pub fn mcp_tool_descriptors() -> Value {
    let mut tools = mcp_tool_descriptors_core();
    if let Some(arr) = tools.as_array_mut() {
        arr.extend(crate::module::module_mcp_tool_descriptors());
    }
    tools
}

/// CLI help text generated from the same contracts (+ module methods when loaded).
pub fn cli_methods_help() -> String {
    let mut text = cli_help_text(METHOD_CONTRACTS);
    let module_methods = crate::module::module_rpc_methods();
    if !module_methods.is_empty() {
        text.push_str("\nmodule methods:\n");
        for m in module_methods {
            text.push_str(&format!("  {m}\n"));
        }
    }
    text
}

/// Typed contracts slice (codegen / tests).
pub fn method_contracts() -> &'static [MethodContract] {
    METHOD_CONTRACTS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn method_table_matches_contracts() {
        let table = method_table();
        assert_eq!(table.len(), METHOD_CONTRACTS.len());
        for (a, b) in table.iter().zip(METHOD_CONTRACTS.iter()) {
            assert_eq!(a.service, b.service);
            assert_eq!(a.method, b.method);
            assert_eq!(a.mcp_tool, b.mcp_tool);
            assert_eq!(a.description, b.description);
        }
    }

    #[test]
    fn mcp_descriptors_cover_all_tools() {
        // Core-only: live `mcp_tool_descriptors()` may include loaded modules
        // and must not flake this SSOT length check (TOOL-11 residual).
        let tools = mcp_tool_descriptors_core();
        let arr = tools.as_array().unwrap();
        assert_eq!(arr.len(), METHOD_CONTRACTS.len());
        for c in METHOD_CONTRACTS {
            assert!(
                arr.iter()
                    .any(|t| t.get("name").and_then(|n| n.as_str()) == Some(c.mcp_tool)),
                "missing mcp tool {}",
                c.mcp_tool
            );
        }
        // Live list is a superset of core when modules are loaded.
        let live = mcp_tool_descriptors();
        let live_arr = live.as_array().unwrap();
        assert!(live_arr.len() >= arr.len());
    }
}
