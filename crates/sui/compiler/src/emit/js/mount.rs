use super::super::writer::Writer;
use crate::ir::Program;

/// Emit the tagless **mount-form**: `export default component((ctx) => { … })`.
///
/// `component(setup)` (the function overload) returns `{ setup }`; we splice the
/// version stamp onto that object so it travels with the default export.
/// esbuild/Bun.build tree-shake a module-level `export const _compilerVersion`
/// when no consumer touches it, but every host that mounts a component reads
/// the default export — so attaching the stamp here keeps it visible after
/// bundling without changing the mount contract.
///
/// `clients: []` also travels on the default export — always empty now (the
/// retired `<client>` directive was its only producer), kept for the mount
/// contract's introspection surface (`mount()` iterates it to warn on missing
/// `opts.clients.<name>` keys; an empty list is simply a no-op there).
pub(super) fn emit_mount_export(w: &mut Writer, program: &Program, body: Writer) {
    emit_module(w, program, body, "export default ");
}

/// Write `<prefix>Object.assign(component((ctx) => { <body> }), { _compilerVersion,
/// clients });` — the mount-form module. Shared by the page default export and
/// the registered-component module const (a tagged source binds it to a tag via
/// `defineComponent`).
pub(super) fn emit_module(w: &mut Writer, _program: &Program, body: Writer, prefix: &str) {
    let cv = crate::compiler_version();
    let stamp = format!(
        "{{\"cargo\":\"{}\",\"git\":\"{}\",\"dirty\":{},\"builtAt\":\"{}\"}}",
        cv.cargo, cv.git, cv.dirty, cv.built_at,
    );
    w.line(&format!("{prefix}Object.assign(component((ctx) => {{"));
    w.push_str(&body.into_string());
    w.line(&format!(
        "}}), {{ _compilerVersion: {stamp}, clients: [] }});"
    ));
}
