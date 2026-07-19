use std::path::Path;

pub use deck_host::render_helpers::is_valid_route_name;

/// Read a route's source `.sail` under an EXPLICIT project `root` (the caller
/// resolves it from its per-request [`crate::Resolution`], never a process
/// global) — HOST-AGNOSTIC: any host that carries the workspace root resolves
/// `views/<name>/page.sail` the same way the daemon does.
pub fn render_route_source(root: &Path, name: &str) -> Option<String> {
    deck_host::render_helpers::render_route_source(root, name)
}
