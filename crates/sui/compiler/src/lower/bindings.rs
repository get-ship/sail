//! Per-attribute import collection. Mirrors the runtime helpers
//! exposed in `packages/sui/runtime/src/index.ts`.

use crate::ir::*;
use crate::lower::Lowered;

pub fn collect_attr_imports(a: &AttrInstr, out: &mut Lowered, is_view: bool, depth: usize) {
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
            out.runtime_imports.insert("bindAttr");
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
        AttrInstr::Transition { name, .. } => {
            out.runtime_imports.insert("registerTransition");
            match name.as_str() {
                "fade" => { out.transition_builtins.insert("fade"); }
                "slide" => { out.transition_builtins.insert("slide"); }
                "scale" => { out.transition_builtins.insert("scale"); }
                _ => {}
            }
        }
    }
}
