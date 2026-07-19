#![allow(clippy::if_same_then_else)]
//! `.d.ts` declaration file emitter.
//!
//! Walks `<prop>` and `<use-* as=>` declarations from the IR and emits a
//! TypeScript declaration file that authors can import as `Scope`.
//! Reactive state lives in the controller's setup function and is
//! typed by TS itself, so it has no `.d.ts` entry.

use crate::check::controller_parse::ParsedController;
use crate::ir::{Instr, Program, PropDecl, PropKind};

/// One CAPABILITY the host grants this file: a name the runtime puts on the
/// module's scope, with the TypeScript type it carries.
///
/// The compiler names no capability and no provider — the caller resolves the
/// list (app manifest + whatever context the host injects) and hands it in, the
/// same way `route_params` arrives as a position grant. `ts_type` is written
/// verbatim into the generated `Scope`; `ts_import` names the module the type
/// is a named export of, when it needs importing.
///
/// See `emit::synthetic::scope_capabilities` for the manifest-derived default
/// (every declared client is typed through the SAIL-local `Clients` port).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeCapability {
    /// The scope member name, e.g. `route` or `api`.
    pub name: String,
    /// The member's TypeScript type, verbatim, e.g. `Clients['api']`.
    pub ts_type: String,
    /// Module specifier to import `ts_type`'s root name from, when the type
    /// is not self-contained. `None` for inline types like `(p: string) => void`.
    pub ts_import: Option<(String, String)>,
}

impl ScopeCapability {
    /// A capability typed through the SAIL-local `Clients` port — the default
    /// for every client an app manifest declares.
    pub fn client(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            ts_type: format!("Clients['{name}']"),
            ts_import: Some(("Clients".to_string(), RUNTIME_SPECIFIER.to_string())),
            name,
        }
    }
}

/// The package the generated `Scope`'s runtime types are imported from.
const RUNTIME_SPECIFIER: &str = "@ship/sail-runtime";

/// Emit the `.sail.d.ts` text for a compiled `.sail` file.
///
/// When a paired controller is available, the `Scope` interface also
/// includes every key surfaced by the controller's setup function's
/// factory. Each entry is typed by a lightweight heuristic — see
/// `super::scope_factory` for the supported shapes. Authors who want a
/// tighter inferred type than the heuristic can always annotate the
/// call site (`signal<MyType>(…)`).
///
/// `capabilities` is the resolved capability list for this file (see
/// [`ScopeCapability`]). It is emitted into `Scope` alongside the `clients`
/// map itself, mirroring the runtime `baseScope`
/// (`{ clients, ...clients, route, navigate, ...scope }`) exactly — so the
/// generated type, the runtime scope, and `MutableState`'s ambient block all
/// agree, instead of a hand-maintained wrapper drifting from them. An empty
/// list leaves `Scope` exactly as before: a file compiled with no host context
/// gets no ambient members, and reading one is a type error.
pub fn emit_dts(
    program: &Program,
    controller: Option<&ParsedController>,
    capabilities: &[ScopeCapability],
) -> String {
    let mut members: Vec<(String, String)> = Vec::new();

    for p in &program.props {
        // Prop names in templates are kebab-case to match HTML attribute
        // convention (e.g. `'font-size' = ''` in `<scope>`); the runtime
        // camelCases them to JS identifiers (`fontSize`) via
        // ui-runtime/component-factory.ts::camelOf. Scope must agree —
        // emit the camelCased identifier so `state.fontSize` typechecks.
        members.push((super::util::camel_of(&p.name), prop_ts_type(p)));
    }
    collect_use_bindings(&program.template, &mut members);

    let scope_factory_fields: Vec<(String, String)> = controller
        .map(super::scope_factory::extract)
        .unwrap_or_default();

    // Controller scope-factory fields are OMITTED from the generated Scope
    // entirely — they are typed solely by the controller's inferred SC.
    // Emitting them here put the same key in BOTH Ctx (= ControllerScope<Scope>) and
    // SC, so `MutableState<Ctx & SC>.$signals.x` became `Signal<unknown> &
    // Signal<T>` and calling that intersection returned `unknown` (first call
    // signature wins) — the root cause of the `as unknown as` state casts.
    // Dropping them here lets SC be the sole, precise source: `$signals.x() = T`.
    // We still drop any template-derived member of the same name that
    // clashes, so the field is SC-sourced, not a stale template type. Do NOT
    // re-add factory fields to Scope.
    let factory_names: std::collections::HashSet<String> =
        scope_factory_fields.iter().map(|(n, _)| n.clone()).collect();
    members.retain(|(n, _)| !factory_names.contains(n));

    // HOST-GRANTED capabilities, prepended in `baseScope` order: the `clients`
    // map, then one member per declared client. A name the template or the
    // controller factory already declares WINS — same as the runtime, where
    // `...(ctx.scope ?? {})` is spread last over the client map.
    let declared: std::collections::HashSet<&str> =
        members.iter().map(|(n, _)| n.as_str()).chain(factory_names.iter().map(|n| n.as_str())).collect();
    let granted: Vec<&ScopeCapability> =
        capabilities.iter().filter(|c| !declared.contains(c.name.as_str())).collect();
    let mut ambient: Vec<(String, String)> = Vec::new();
    if !capabilities.is_empty() && !declared.contains("clients") {
        // The map itself, as the runtime provides it (`ctx.clients.api`), next
        // to the spread-in per-name members (`ctx.api`). Both paths are real.
        ambient.push(("clients".to_string(), "Clients".to_string()));
    }
    for c in &granted {
        ambient.push((c.name.clone(), c.ts_type.clone()));
    }
    let members: Vec<(String, String)> = ambient.into_iter().chain(members).collect();

    if members.is_empty() && program.scope_types.is_empty() {
        return "// generated by sail-compiler — no scope declarations\nexport interface Scope {}\n"
            .to_string();
    }

    // Which runtime type aliases the emitted members reference. `Signal<…>`
    // covers prop / ref / use-hook entries; capability members bring their own
    // named imports (see `ScopeCapability::ts_import`). Imports are driven only
    // by the emitted `members` — factory fields are no longer written into
    // Scope (see the omit rationale above).
    let mut imports: Vec<(String, String)> = Vec::new();
    if members.iter().any(|(_, t)| t.contains("Signal<")) {
        imports.push(("Signal".to_string(), RUNTIME_SPECIFIER.to_string()));
    }
    if members.iter().any(|(n, _)| n == "clients") {
        imports.push(("Clients".to_string(), RUNTIME_SPECIFIER.to_string()));
    }
    for c in &granted {
        if let Some(imp) = &c.ts_import {
            imports.push(imp.clone());
        }
    }
    let import_line = render_imports(&imports);

    let mut out = String::with_capacity(256);
    out.push_str("// generated by sail-compiler — DO NOT EDIT\n");
    let cv = crate::compiler_version();
    out.push_str(&format!(
        "// _compilerVersion {{\"cargo\":\"{}\",\"git\":\"{}\",\"dirty\":{},\"builtAt\":\"{}\"}}\n",
        cv.cargo, cv.git, cv.dirty, cv.built_at,
    ));
    out.push_str(&import_line);
    if !import_line.is_empty() {
        out.push('\n');
    }
    if !program.scope_types.is_empty() {
        // `<scope>` type aliases, verbatim: exported ones are the component's
        // public type API; the rest let `Scope` members reference them.
        out.push_str(program.scope_types.trim_end());
        out.push_str("\n\n");
    }
    out.push_str("export interface Scope {\n");
    for (name, ty) in &members {
        out.push_str(&format!("  {name}: {ty}\n"));
    }
    out.push_str("}\n");
    out
}

/// Render `(type, specifier)` pairs as `import type` lines — one per module,
/// deduped, specifier order stable (first use wins) so the emit is
/// byte-deterministic for the same inputs.
fn render_imports(imports: &[(String, String)]) -> String {
    let mut modules: Vec<(&str, Vec<&str>)> = Vec::new();
    for (ty, spec) in imports {
        let entry = match modules.iter_mut().find(|(s, _)| *s == spec.as_str()) {
            Some(e) => e,
            None => {
                modules.push((spec.as_str(), Vec::new()));
                modules.last_mut().expect("just pushed")
            }
        };
        if !entry.1.contains(&ty.as_str()) {
            entry.1.push(ty.as_str());
        }
    }
    modules
        .iter()
        .map(|(spec, types)| format!("import type {{ {} }} from '{spec}'\n", types.join(", ")))
        .collect()
}

fn prop_ts_type(p: &PropDecl) -> String {
    let inner = match &p.kind {
        PropKind::String => "string".to_string(),
        PropKind::Bool => "boolean".to_string(),
        PropKind::Number => "number".to_string(),
        PropKind::Union(values) => values
            .iter()
            .map(|v| format!("'{v}'"))
            .collect::<Vec<_>>()
            .join(" | "),
        // props-as-ts: the prop's type IS the TypeScript you wrote — preserved
        // verbatim (`Signal<Column[]>`), resolved through your own imports.
        PropKind::Identity(t) => t.clone(),
    };
    format!("Signal<{inner}>")
}


/// Walk template instructions and collect `<use-X as="Y">` bindings.
fn collect_use_bindings(instrs: &[Instr], out: &mut Vec<(String, String)>) {
    for instr in instrs {
        if let Instr::Element(el) = instr {
            if let Some(hook_name) = el.tag.strip_prefix("use-") {
                // Look for an `as=` attribute that binds the hook under an alias.
                for attr in &el.attrs {
                    if let crate::ir::AttrInstr::Static { name, value, .. } = attr
                        && name == "as" && !value.is_empty() {
                            let camel = to_camel(hook_name);
                            out.push((
                                value.clone(),
                                format!("ReturnType<typeof use{camel}>"),
                            ));
                        }
                }
            }
            collect_use_bindings(&el.children, out);
        }
    }
}

fn to_camel(kebab: &str) -> String {
    kebab
        .split('-')
        .enumerate()
        .map(|(i, part)| {
            if i == 0 {
                let mut c = part.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                }
            } else {
                let mut c = part.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                }
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::controller_parse::parse as parse_controller;
    use crate::diagnostics::Span;
    use std::path::PathBuf;

    /// An empty (template-less) `Program` — the d.ts under test draws all of
    /// its members from the paired controller's `setup:` factory.
    fn empty_program() -> Program {
        Program {
            props: Vec::new(),
            styles: String::new(),
            scope_types: String::new(),
            template: Vec::new(),
            source_len: 0,
            span: Span::new(0, 0),
        }
    }

    fn ctrl(src: &str) -> crate::check::controller_parse::ParsedController {
        parse_controller(PathBuf::from("controller.ts"), src.to_string()).expect("parsed")
    }

    #[test]
    fn scope_factory_fields_are_omitted_from_scope() {
        // Controller scope-factory fields are typed solely by the inferred SC,
        // NOT re-declared in Scope (emitting them in both collapsed
        // `$signals.x` to `unknown`). A factory-only scope over an empty
        // template therefore yields an empty Scope with no spurious imports.
        let c = ctrl(
            "export default controller((ctx) => ({\n    \
               q: signal(''),\n    \
               rail: live(() => liveExpr(`ship.skills.summaries()`)),\n  \
             }));",
        );
        let dts = emit_dts(&empty_program(), Some(&c), &[]);
        assert!(dts.contains("export interface Scope {}"), "expected empty Scope, got:\n{dts}");
        assert!(!dts.contains("rail:"), "factory field must not be in Scope, got:\n{dts}");
        assert!(!dts.contains("q:"), "factory field must not be in Scope, got:\n{dts}");
        // No member references a runtime type → no import line at all.
        assert!(!dts.contains("import type"), "no imports when Scope is empty, got:\n{dts}");
    }
}
