use super::super::styles::{light_dom_selectors, strip_shared_rules};
use super::super::writer::{js_string, Writer};
use crate::ir::*;
use crate::lower::directives::prop_default_literal;
use crate::ComponentMeta;

/// Emit a TAGGED source: the mount-form module bound to a const, then a
/// `component(tag, def, module)` registration. The runtime registration builds
/// the observed-attribute schema from `props`, injects `styles` (data-component
/// scoped, or into the shadow root), wires form-association, and on connect
/// mounts the module via `mountComponent` — the one engine the page /
/// internal-composition paths also use. Setup/lifecycle live in the module
/// (via `emit_setup_body` + `mountComponent`), not here.
pub(super) fn emit_registered_component(
    w: &mut Writer,
    program: &Program,
    body: Writer,
    meta: &ComponentMeta,
) {
    // The module — same mount-form shape a page exports, bound to a const so
    // the registration + default export can both reference it.
    super::mount::emit_module(w, program, body, "const __default = ");

    w.line(&format!("component({}, {{", js_string(&meta.name)));
    w.enter();
    emit_props_object(w, &program.props);
    if meta.shadow {
        w.line("shadow: true,");
    }
    let form_value_prop = program.props.iter().find(|p| p.form_value);
    let form_disabled_prop = program.props.iter().find(|p| p.form_disabled);
    if form_value_prop.is_some() || form_disabled_prop.is_some() {
        w.line("formAssociated: true,");
        if let Some(p) = form_value_prop {
            w.line(&format!("formValue: {},", js_string(&p.name)));
        }
        if let Some(p) = form_disabled_prop {
            w.line(&format!("formDisabled: {},", js_string(&p.name)));
        }
    }
    let emitted_styles = if meta.shadow {
        strip_shared_rules(&program.styles)
    } else {
        // LIGHT DOM (the default architecture): the runtime wraps these styles
        // in an `@scope ([data-component="tag"]) to ([data-component])` donut,
        // so shadow idioms rewrite — :host → :scope, ::slotted(X) → slot > X.
        // Shared-rule stripping also applies: the document-level contract
        // sheets carry those rules once.
        light_dom_selectors(&strip_shared_rules(&program.styles))
    };
    if !emitted_styles.trim().is_empty() {
        w.line(&format!("styles: {},", js_string(&emitted_styles)));
    }
    w.leave();
    w.line("}, __default);");
    w.line("export default __default;");
}

fn emit_props_object(w: &mut Writer, props: &[PropDecl]) {
    if props.is_empty() {
        return;
    }
    w.line("props: {");
    w.enter();
    for p in props {
        w.line(&format!("{}: {},", js_string(&p.name), prop_call(p)));
    }
    w.leave();
    w.line("},");
}

fn prop_call(p: &PropDecl) -> String {
    let opts_parts = build_prop_opts(p);
    let opts_str = if opts_parts.is_empty() {
        String::new()
    } else {
        format!("{{ {} }}", opts_parts.join(", "))
    };
    match &p.kind {
        PropKind::String => format!("prop.string({opts_str})"),
        PropKind::Bool => format!("prop.bool({opts_str})"),
        PropKind::Number => format!("prop.number({opts_str})"),
        PropKind::Union(values) => {
            let values_js = values
                .iter()
                .map(|v| js_string(v))
                .collect::<Vec<_>>()
                .join(", ");
            if opts_str.is_empty() {
                format!("prop.union([{values_js}])")
            } else {
                format!("prop.union([{values_js}], {opts_str})")
            }
        }
        // A plain reactive property — signal + accessor, no attribute. Its TS
        // type lives in the `.d.ts`, not in the runtime registration.
        PropKind::Identity(_) => format!("prop.property({opts_str})"),
    }
}

fn build_prop_opts(p: &PropDecl) -> Vec<String> {
    let mut parts = Vec::new();
    if let Some(d) = prop_default_literal(p) {
        parts.push(format!("default: {d}"));
    }
    if p.reflect {
        parts.push("reflect: true".into());
    }
    if p.bindable {
        parts.push("bindable: true".into());
    }
    parts
}
