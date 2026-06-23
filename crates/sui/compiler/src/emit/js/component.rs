use super::super::styles::strip_shared_rules;
use super::super::writer::{js_string, Writer};
use crate::ir::*;
use crate::lower::directives::prop_default_literal;
use crate::CompileOpts;

#[allow(clippy::expect_used)]
pub(super) fn emit_component_export(
    w: &mut Writer,
    program: &Program,
    body: Writer,
    opts: &CompileOpts,
) {
    let meta = opts
        .component_meta
        .as_ref()
        .expect("component_meta required for component mode");
    w.line(&format!("export default component({}, {{", js_string(&meta.name)));
    w.enter();
    // Stamp the component config so the bundled artifact carries
    // compiler identity. The runtime's `component()` factory ignores
    // unknown fields; this travels with the default export and is
    // therefore safe from tree-shaking.
    let cv = crate::compiler_version();
    w.line(&format!(
        "_compilerVersion: {{ cargo: {}, git: {}, dirty: {}, builtAt: {} }},",
        js_string(cv.cargo),
        js_string(cv.git),
        cv.dirty,
        js_string(cv.built_at),
    ));
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
    let is_shadow = meta.shadow;
    let emitted_styles = if is_shadow {
        strip_shared_rules(&program.styles)
    } else {
        program.styles.clone()
    };
    if !emitted_styles.trim().is_empty() {
        w.line(&format!("styles: {},", js_string(&emitted_styles)));
    }
    w.line("setup(ctx) {");
    w.push_str(&body.into_string());
    w.line("},");
    w.leave();
    w.line("});");
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
