//! Per-attribute import collection. Mirrors the runtime helpers
//! exposed in `packages/sail/runtime/src/index.ts`.
//!
//! The `BUILTIN_ACTIONS` list is the compiler's view of the action
//! registry, derived from `crate::action_manifest::BUILTIN_ACTION_NAMES`
//! so the action table is the single source of truth. The runtime's
//! `actions/index.ts` barrel and the top-level `index.ts` re-export
//! are validated against the same source by the drift test in
//! `tests/abi_boundary.rs` — adding an action means updating the
//! table, never the three Rust arrays independently.
use crate::diagnostics::CompileError;
use crate::ir::*;
use crate::lower::presets;
use crate::lower::Lowered;

/// Built-in `use:` actions shipped by the runtime. When a `use:` directive
/// names one of these (a bare identifier), the compiler auto-imports it from
/// `@ship/sail-runtime` so it resolves as an ESM binding — no controller
/// import, no `$scope.` lookup.
///
/// This is the same slice as
/// `crate::action_manifest::BUILTIN_ACTION_NAMES` — the table is the
/// single source. The const appears under this path for downstream
/// code that historically reached for
/// `sail_compiler::lower::bindings::BUILTIN_ACTIONS`.
pub const BUILTIN_ACTIONS: &[&str] = crate::action_manifest::BUILTIN_ACTION_NAMES;

pub fn collect_attr_imports(
    a: &AttrInstr,
    out: &mut Lowered,
    is_route: bool,
    depth: usize,
    tag: &str,
) -> Result<(), CompileError> {
    // Phase 3b.4: depth-0 element bindings in routes are direct-emit.
    // The hydrate(root, ctx) closure walks SSR'd elements once via
    // `collectElementMarkers` and dispatches to the matching `hydrate*`
    // helper by binding key. The legacy `__mount` path keeps using the
    // runtime helpers below — both paths coexist until the walker is
    // deleted in 3b.5.
    let direct_emit = is_route && depth == 0;
    match a {
        AttrInstr::Static { .. } => {}
        AttrInstr::Expr { name, .. } => {
            // Web backend (hidden from author): CE → bindDynamic; native IDL
            // prop table → bindProp; else bindAttr. See `native_idl_prop`.
            let binder = plain_field_binder(tag, name);
            out.runtime_imports.insert(binder);
            if direct_emit {
                out.dom_imports.insert("collectElementMarkers");
                if binder == "bindProp" {
                    out.dom_imports.insert("hydrateProp");
                } else {
                    out.dom_imports.insert("hydrateAttr");
                }
            }
        }
        AttrInstr::BindValue { .. } => {
            out.runtime_imports.insert("bindValue");
            if direct_emit {
                out.dom_imports.insert("collectElementMarkers");
                out.dom_imports.insert("hydrateBindValue");
            }
        }
        AttrInstr::BindChecked { .. } => {
            out.runtime_imports.insert("bindChecked");
            if direct_emit {
                out.dom_imports.insert("collectElementMarkers");
                out.dom_imports.insert("hydrateBindChecked");
            }
        }
        AttrInstr::BindThis { .. } => {
            // `bind:this={var}` is an element capture: it writes the mounted
            // element into a plain signal slot declared by the controller.
            // The directive is lowered to the same `runAction` lifecycle the
            // `use:` directive uses, so it auto-disposes (sets the slot back
            // to null) when the element unmounts. The target must be declared
            // in scope; undeclared targets are caught by scope_bound.
            out.runtime_imports.insert("runAction");
        }
        AttrInstr::SlotReflectEmpty { .. } => {
            out.runtime_imports.insert("bindSlotEmpty");
        }
        AttrInstr::On { .. } => {
            out.runtime_imports.insert("on");
            if direct_emit {
                out.dom_imports.insert("collectElementMarkers");
                out.dom_imports.insert("hydrateOn");
            }
        }
        AttrInstr::Mutation { .. } => {
            out.runtime_imports.insert("wireFormMutation");
            if direct_emit {
                out.dom_imports.insert("collectElementMarkers");
            }
        }
            AttrInstr::Use { action, .. } => {
            out.runtime_imports.insert("runAction");
            // A bare built-in action name (`use:anchor`, `use:autosize`, …)
            // is auto-imported from the runtime so it resolves as an ESM
            // binding rather than an undefined `$scope.` lookup. Non-builtin
            // actions (a hook handle or a controller-imported fn) resolve
            // from scope and need no import.
            if let Some(&builtin) = BUILTIN_ACTIONS.iter().find(|&&b| b == action.trim()) {
                out.runtime_imports.insert(builtin);
            }
        }
        AttrInstr::Motion { trigger, props, stagger_ms: _, timing: _, span } => {
            // `motion:` directives bake to literal `el.animate(...)`
            // calls plus a `__sui_motion_attach(...)` wrapper. The
            // attach wrapper and any animation-side runtime helpers
            // come from `@ship/sail-runtime/motion` (separate from
            // the umbrella `@ship/sail-runtime` import); we record
            // each used helper here so the imports emitter can
            // emit a side-effect import per symbol.
            //
            // Walking the preset DB here (rather than at emit time)
            // lets us validate that every `props.preset` resolves
            // to a known preset — fail-loud is preferred over a
            // silent runtime fallback. The DOM emit trusts this
            // arm and uses `.expect("validated in lower")`.
            let db = presets::load().map_err(|e| {
                CompileError::new(format!(
                    "motion:{} — failed to load animation presets: {e}",
                    trigger
                ))
                .with_span(*span)
                .with_category("motion")
            })?;
            for prop in props {
                // A preset is now OPTIONAL — an inline `{ from, to }` channel
                // needs no named default, so only validate the name when the
                // author actually reached for one.
                if let Some(name) = prop.preset.as_deref() {
                    let Some(spec) = db.preset(name) else {
                        let known = db.known_preset_names();
                        return Err(CompileError::new(format!(
                            "motion:{} uses unknown preset '{}'. Known presets: {}. \
                             Or drop the preset and write the values inline: \
                             `{}: {{ from: 0, to: 1 }}`.",
                            trigger,
                            name,
                            known.join(", "),
                            prop.property,
                        ))
                        .with_span(*span)
                        .with_category("motion"));
                    };
                    let _ = spec.kind;
                }
                // The motion runtime is a single symbol — the
                // `__sui_motion_attach` helper that the dom emit
                // calls. Recording it here drives the side-effect
                // import the imports walker emits; future
                // per-preset spring/tween helpers will live in a
                // separate runtime module and be imported only
                // when their functionality is actually needed.
                out.motion_builtins.insert("__sui_motion_attach");
            }
        }
    }
    Ok(())
}

/// Closed IDL property table for **native** HTML tags — backend knowledge,
/// not an author directive. When a plain `name={expr}` targets one of these
/// names on a non-custom element, emit `bindProp` so boolean/object identity
/// (checked, value, indeterminate, …) is preserved. Everything else on a
/// native tag is `bindAttr`. Custom elements always use `bindDynamic`.
pub fn native_idl_prop(name: &str) -> bool {
    matches!(
        name,
        "value"
            | "checked"
            | "indeterminate"
            | "selected"
            | "open"
            | "disabled"
            | "multiple"
            | "readOnly"
            | "readonly"
            | "required"
            | "defaultValue"
            | "defaultChecked"
            | "innerHTML"
            | "textContent"
            | "scrollTop"
            | "scrollLeft"
    )
}

/// Pick the web runtime binder for a plain dynamic field.
pub fn plain_field_binder(tag: &str, name: &str) -> &'static str {
    if tag.contains('-') {
        "bindDynamic"
    } else if native_idl_prop(name) {
        "bindProp"
    } else {
        "bindAttr"
    }
}
