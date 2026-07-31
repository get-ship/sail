//! `sail-mcp` — package binary for the Sail MCP server (stdio).
//!
//! Same protocol as `sail mcp` (unified CLI). This bin is the cargo-package
//! entry used by `.mcp.json` (`cargo run -p sail-mcp`).

fn main() {
    if std::env::args().any(|a| a == "--help" || a == "-h") {
        eprintln!(
            "sail-mcp — Sail MCP server (stdio)\n\n\
             Usage:\n  sail-mcp\n  sail-mcp --list-tools\n  sail mcp   # unified CLI surface\n"
        );
        return;
    }
    sail_mcp::run_stdio();
}
