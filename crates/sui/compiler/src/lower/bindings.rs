//! Per-attribute import collection. Mirrors the runtime helpers
//! exposed in `packages/sui/runtime/src/index.ts`.
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
/// `@ship/sui-runtime` so it resolves as an ESM binding — no controller
/// import, no `$scope.` lookup.
///
/// This is the same slice as
/// `crate::action_manifest::BUILTIN_ACTION_NAMES` — the table is the
/// single source. The const appears under this path for downstream
/// code that historically reached for
/// `sui_compiler::lower::bindings::BUILTIN_ACTIONS`.
pub const BUILTIN_ACTIONS: &[&str] = crate::action_manifest::BUILTIN_ACTION_NAMES;

pub fn collect_attr_imports(
    a: &AttrInstr,
    out: &mut Lowered,
    is_view: bool,
    depth: usize,
    is_custom_element: bool,
) -> Result<(), CompileError> {
    // Phase 3b.4: depth-0 element bindings in views are direct-emit.
    // The hydrate(root, ctx) closure walks SSR'd elements once via
    // `collectElementMarkers` and dispatches to the matching `hydrate*`
    // helper by binding key. The legacy `__mount` path keeps using the
    // runtime helpers below — both paths coexist until the walker is
    // deleted in 3b.5.
    let direct_emit = is_view && depth == 0;
    match a {
        AttrInstr::Static { .. } => {}
        AttrInstr::Expr { .. } => {
            // Native elements are attribute-only (`bindAttr`). Custom elements
            // get `bindDynamic`, which routes attribute-vs-property at runtime
            // so the author writes one plain binding either way.
            out.runtime_imports.insert(if is_custom_element { "bindDynamic" } else { "bindAttr" });
            if direct_emit {
                out.dom_imports.insert("collectElementMarkers");
                out.dom_imports.insert("hydrateAttr");
            }
        }
        AttrInstr::DynamicAttr { .. } => {
            out.runtime_imports.insert("effect");
            if direct_emit {
                out.dom_imports.insert("collectElementMarkers");
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
            // The setup body pre-seeds `$scope.<var> = signal(null)` for
            // every bind:this target the template references — including
            // ones inside conditional / loop branches that haven't
            // rendered yet. Without the pre-seed, a controller reading
            // `scope.headerEl()` before the `{#if open()}` branch mounted
            // would see `undefined` and crash with
            // `headerEl is not a function`. attachRef() reuses the
            // pre-seeded slot when the element finally mounts.
            out.runtime_imports.insert("attachRef");
            out.runtime_imports.insert("signal");
            if direct_emit {
                out.dom_imports.insert("collectElementMarkers");
                out.dom_imports.insert("hydrateRef");
            }
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
        AttrInstr::Class { .. } => {
            out.runtime_imports.insert("bindClass");
            if direct_emit {
                out.dom_imports.insert("collectElementMarkers");
                out.dom_imports.insert("hydrateClass");
            }
        }
        AttrInstr::Style { .. } => {
            out.runtime_imports.insert("effect");
            if direct_emit {
                out.dom_imports.insert("collectElementMarkers");
                out.dom_imports.insert("hydrateStyle");
            }
        }
        AttrInstr::Prop { .. } => {
            out.runtime_imports.insert("bindProp");
            if direct_emit {
                out.dom_imports.insert("collectElementMarkers");
                out.dom_imports.insert("hydrateProp");
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
        AttrInstr::Motion { trigger, props, stagger_ms: _, span } => {
            // `motion:` directives bake to literal `el.animate(...)`
            // calls plus a `__sui_motion_attach(...)` wrapper. The
            // attach wrapper and any animation-side runtime helpers
            // come from `@ship/sui-runtime/motion` (separate from
            // the umbrella `@ship/sui-runtime` import); we record
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
                let Some(spec) = db.preset(&prop.preset) else {
                    let known = db.known_preset_names();
                    return Err(CompileError::new(format!(
                        "motion:{} uses unknown preset '{}'. Known presets: {}",
                        trigger,
                        prop.preset,
                        known.join(", ")
                    ))
                    .with_span(*span)
                    .with_category("motion"));
                };
                // The motion runtime is a single symbol — the
                // `__sui_motion_attach` helper that the dom emit
                // calls. Recording it here drives the side-effect
                // import the imports walker emits; future
                // per-preset spring/tween helpers will live in a
                // separate runtime module and be imported only
                // when their functionality is actually needed.
                let _ = spec.kind;
                out.motion_builtins.insert("__sui_motion_attach");
            }
        }
    }
    Ok(())
}