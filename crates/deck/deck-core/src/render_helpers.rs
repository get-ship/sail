pub use deck_host::render_helpers::is_valid_view_name;

pub fn render_view_source(name: &str) -> Option<String> {
    // Use deck-core's canonical project-root resolver — set via the
    // in-process global (`deck_core::resolver::set_project_root`), or
    // autodetected from cwd — so view resolution is HOST-AGNOSTIC and
    // consistent with the rest of deck-core. A ship-free host (e.g. deck-conformance)
    // that calls set_project_root resolves its `views/<name>/page.sui` the same
    // way the daemon does — no `.ship` dir required.
    let project_root = crate::resolver::resolve_project_root()?;
    deck_host::render_helpers::render_view_source(&project_root, name)
}
