use super::super::writer::{js_string, Writer};
use crate::ir::Program;

pub(super) fn emit_view_export(w: &mut Writer, program: &Program, body: Writer) {
    // The view() factory returns `{ setup }`; we splice the version
    // stamp onto the same object so it travels with the default
    // export. Bun.build / esbuild tree-shake module-level
    // `export const _compilerVersion` away when no consumer touches
    // it, but every host that hydrates a view reads the default
    // export — so attaching the stamp here keeps it visible after
    // bundling without changing the view's hydration contract.
    let cv = crate::compiler_version();
    let stamp = format!(
        "{{\"cargo\":\"{}\",\"git\":\"{}\",\"dirty\":{},\"builtAt\":\"{}\"}}",
        cv.cargo, cv.git, cv.dirty, cv.built_at,
    );
    // Declared `<client>` names travel on the default export too — the
    // mount contract's introspection surface. mount() compares this list
    // against the host-supplied `opts.clients` map and warns on missing
    // keys instead of letting `<data source>` thunks fail opaquely.
    let clients = program
        .clients
        .iter()
        .map(|c| js_string(&c.name))
        .collect::<Vec<_>>()
        .join(", ");
    w.line("export default Object.assign(view((ctx) => {");
    w.push_str(&body.into_string());
    w.line(&format!(
        "}}), {{ _compilerVersion: {stamp}, clients: [{clients}] }});"
    ));
}
