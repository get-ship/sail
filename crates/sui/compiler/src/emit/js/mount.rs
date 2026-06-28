use super::super::writer::{js_string, Writer};
use crate::ir::Program;

/// Emit the tagless **mount-form**: `export default component((ctx) => { … })`.
///
/// `component(setup)` (the function overload) returns `{ setup }`; we splice the
/// version stamp + declared `<client>` names onto that object so they travel
/// with the default export. esbuild/Bun.build tree-shake a module-level
/// `export const _compilerVersion` when no consumer touches it, but every host
/// that mounts a component reads the default export — so attaching the stamp
/// here keeps it visible after bundling without changing the mount contract.
///
/// Declared `<client>` names travel on the default export too — the mount
/// contract's introspection surface. `mount()` compares this list against the
/// host-supplied `opts.clients` map and warns on missing keys instead of
/// letting `<data source>` thunks fail opaquely.
pub(super) fn emit_mount_export(w: &mut Writer, program: &Program, body: Writer) {
    let cv = crate::compiler_version();
    let stamp = format!(
        "{{\"cargo\":\"{}\",\"git\":\"{}\",\"dirty\":{},\"builtAt\":\"{}\"}}",
        cv.cargo, cv.git, cv.dirty, cv.built_at,
    );
    let clients = program
        .clients
        .iter()
        .map(|c| js_string(&c.name))
        .collect::<Vec<_>>()
        .join(", ");
    w.line("export default Object.assign(component((ctx) => {");
    w.push_str(&body.into_string());
    w.line(&format!(
        "}}), {{ _compilerVersion: {stamp}, clients: [{clients}] }});"
    ));
}
