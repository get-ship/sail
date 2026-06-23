use crate::keys::BindingRegistry;
use crate::ir::Program;
use crate::render::{self, RenderOpts};

/// Emit static SSR HTML for a compiled view. Walks the lowered IR with
/// the default initial scope (loading envelopes for data, empty route)
/// using the same render walker the payload endpoint uses at runtime.
///
/// Returns the body HTML **with** the `<script id="__set_init__">` seed
/// island appended — written to the project mirror tree as
/// `<repo-root>/.ship/sui-out/<src-rel>/<stem>.ssr.html`. The daemon's
/// from-disk SSR path (`load_ssr_from_disk`) splices this verbatim into
/// the host page, so the precompiled artifact must carry the seed for
/// prod to match dev's live-emit path.
pub fn emit_ssr_html(program: &Program, registry: &BindingRegistry, view_name: &str) -> String {
    let opts = RenderOpts { view_name: view_name.to_string() };
    render::render(program, registry, &opts)
        .map(|o| o.html)
        .unwrap_or_default()
}
