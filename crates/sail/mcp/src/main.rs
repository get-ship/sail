//! `sail-mcp` — package binary for the Sail MCP server.
//!
//! Default: stdio JSON-RPC (MCP 2026-07-28 stateless).
//! Optional: `--http [ADDR]` Streamable HTTP single endpoint.
//! Same protocol as `sail mcp` (unified CLI). This bin is the cargo-package
//! entry used by `.mcp.json` (`cargo run -p sail-mcp`).

fn main() {
    sail_mcp::run();
}
