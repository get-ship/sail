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
use crate::lower::Lowered;
use crate::lower::presets;

/// Built-in `use:` actions shipped by the runtime. When a `use:` directive
/// names one of these (a bare identifier), the compiler auto-imports it from
/// `@ship/sail-runtime/actions` so it resolves as an ESM binding — no
/// controller import, no `$scope.` lookup. The full table is imported
/// (public + internal); only the public subset is re-exported from the
/// runtime front door.
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
            // prop table → bindProp; dialog open → bindOpen; else bindAttr.
            let binder = plain_field_binder(tag, name);
            out.runtime_imports.insert(binder);
            if direct_emit {
                out.dom_imports.insert("collectElementMarkers");
                if binder == "bindProp" {
                    out.dom_imports.insert("hydrateProp");
                } else if binder == "bindOpen" {
                    // bindOpen has no separate hydrate helper — reuses full binder.
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
            // Recorded on `runtime_imports`; the imports emitter splits
            // action symbols onto `@ship/sail-runtime/actions` so
            // internal stamps resolve without the front-door export.
            if let Some(&builtin) = BUILTIN_ACTIONS.iter().find(|&&b| b == action.trim()) {
                out.runtime_imports.insert(builtin);
            }
        }
        AttrInstr::Motion {
            trigger,
            props,
            stagger_ms: _,
            timing: _,
            span,
        } => {
            // `fx:` directives bake to literal `el.animate(...)`
            // calls plus a `__sail_motion_attach(...)` wrapper. The
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
                    "fx:{} — failed to load animation presets: {e}",
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
                            "fx:{} uses unknown preset '{}'. Known presets: {}. \
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
                // `__sail_motion_attach` helper that the dom emit
                // calls. Recording it here drives the side-effect
                // import the imports walker emits; future
                // per-preset spring/tween helpers will live in a
                // separate runtime module and be imported only
                // when their functionality is actually needed.
                out.motion_builtins.insert("__sail_motion_attach");
            }
        }
    }
    Ok(())
}

/// Whether `name` is a real IDL property on native tag `tag`.
/// Used so `selected={…}` on a layout host (`div` from `<row>`) binds as an
/// attribute (CSS `[selected]`), not a dead JS property.
pub fn native_idl_prop(tag: &str, name: &str) -> bool {
    let t = tag.to_ascii_lowercase();
    match name {
        "value" | "defaultValue" => matches!(
            t.as_str(),
            "input" | "textarea" | "select" | "option" | "button" | "output" | "meter" | "progress"
        ),
        "checked" | "indeterminate" | "defaultChecked" => t == "input",
        "selected" => t == "option",
        // details only — dialog open is bindOpen (showModal), not IDL prop assign.
        "open" => t == "details",
        "disabled" => matches!(
            t.as_str(),
            "button"
                | "input"
                | "textarea"
                | "select"
                | "option"
                | "optgroup"
                | "fieldset"
                | "link"
        ),
        "multiple" => matches!(t.as_str(), "select" | "input"),
        "readOnly" | "readonly" | "required" => {
            matches!(t.as_str(), "input" | "textarea" | "select")
        }
        "innerHTML" | "textContent" | "scrollTop" | "scrollLeft" => true,
        _ => false,
    }
}

/// Pick the web runtime binder for a plain dynamic field.
pub fn plain_field_binder(tag: &str, name: &str) -> &'static str {
    let t = tag.to_ascii_lowercase();
    // Callback props (`on-select={fn}`) are identity properties — functions
    // cannot ride attributes. Always bindProp (camelCase) on every host.
    if name.starts_with("on-") {
        return "bindProp";
    }
    // Native <dialog open={sig}> must use showModal via bindOpen — assigning
    // the IDL `open` property is non-modal (no top-layer, no backdrop dismiss).
    if t == "dialog" && name == "open" {
        return "bindOpen";
    }
    if tag.contains('-') {
        "bindDynamic"
    } else if native_idl_prop(tag, name) {
        "bindProp"
    } else {
        "bindAttr"
    }
}

/// `open={captureOpen()}` → signal cell `captureOpen` for bindOpen write-back.
/// Leaves complex expressions unchanged (caller still gets a boolean read).
pub fn dialog_open_signal_expr(source: &str) -> String {
    let t = source.trim();
    if let Some(inner) = t.strip_suffix("()") {
        let id = inner.trim();
        if !id.is_empty()
            && id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'.' || b == b'$')
        {
            return id.to_string();
        }
    }
    t.to_string()
}

/// True when a dialog's `dismissible` attr is a compile-time false literal.
///
/// Author forms: `dismissible={false}` or `dismissible="false"`. Presence,
/// `true`, omitted, and non-literal expressions keep the runtime default
/// (`true`) — emit stays two-arg `bindOpen(el, sig)`.
pub fn dialog_dismissible_is_false(attrs: &[AttrInstr]) -> bool {
    for a in attrs {
        match a {
            AttrInstr::Static { name, value, .. } if name == "dismissible" => {
                return value.trim().eq_ignore_ascii_case("false");
            }
            AttrInstr::Expr { name, expr, .. } if name == "dismissible" => {
                return expr.source.trim() == "false";
            }
            _ => {}
        }
    }
    false
}
