use std::path::Path;

pub use deck_host::render_helpers::is_valid_view_name;

/// Read a view's source `.sui` under an EXPLICIT project `root` (the caller
/// resolves it from its per-request [`crate::Resolution`], never a process
/// global) — HOST-AGNOSTIC: any host that carries the workspace root resolves
/// `views/<name>/page.sui` the same way the daemon does.
pub fn render_view_source(root: &Path, name: &str) -> Option<String> {
    deck_host::render_helpers::render_view_source(root, name)
}
