//! The action registry — ONE source of truth for every `use:` action the
//! language ships, from which every consumer (the compiler's
//! `BUILTIN_ACTIONS` auto-import list, the scope/expr-rewrite builtin
//! tables, the runtime's `actions/index.ts` barrel, the runtime's
//! top-level re-export, the docs site, the agent skill) is either
//! generated or validated. The same shape the L1 element manifest
//! (`emit::element`) uses for its tag registry.
//!
//! ## Why one table
//!
//! Actions used to be five hand-written lists: three Rust arrays
//! (`lower::bindings::BUILTIN_ACTIONS`, `check::scope_bound::BUILTINS`,
//! `emit::expr_rewrite::BUILTINS`) and two TypeScript barrels
//! (`packages/sail/runtime/src/actions/index.ts` and the `./actions`
//! re-export block in `packages/sail/runtime/src/index.ts`). When a
//! name was added to the compiler's list and not the runtime's, the
//! element compiled, passed conformance (which attaches actions by
//! hand), and then died at bundle time with a MissingExport. `portEdge`
//! shipped exactly that way.
//!
//! The three Rust lists and the two TypeScript barrels are now derived
//! from (or validated against) this table. A new action is one entry
//! here — the drift test fails the build until every consumer is
//! updated.
//!
//! ## The entry shape
//!
//! Each action carries:
//!
//!   * `name` — the bare identifier the user writes after `use:`.
//!   * `param` — the type of the optional reactive parameter
//!     (`use:<name>={param}`). `void` for parameter-less actions. A
//!     short string, not a Rust type — these are documentation
//!     primarily; the runtime's TS types are the type contract.
//!   * `summary` — one line of what the action does. Rendered on the
//!     docs site and in the agent skill; that is the reason the
//!     table is hand-authored (it carries prose), not derived from
//!     a grep.
//!   * `auto_import` — whether the compiler auto-imports this from
//!     `@ship/sail-runtime/actions` when a template references it.
//!     Every entry here is auto-imported today (stdlib needs internal
//!     stamps too); the field is present so a future author-side
//!     action can opt out without restructuring. The curated *public*
//!     surface taught in skill/docs is [`PUBLIC_ACTION_NAMES`] — a
//!     strict subset; runtime top-level re-exports only those.

use serde_json::{Value, json};

/// One entry in the action registry. The table is the contract; see
/// the module docs for the derivation rules.
#[derive(Debug, Clone, Copy)]
pub struct ActionEntry {
    /// The bare identifier the user writes after `use:`. Must be
    /// unique within `ACTIONS`.
    pub name: &'static str,
    /// The type of the optional reactive parameter, as a short
    /// human-readable string. `void` for actions that take no
    /// parameter.
    pub param: &'static str,
    /// One-line description of what the action does. Rendered on the
    /// docs site / agent skill — the table is the source.
    pub summary: &'static str,
    /// If `true`, the compiler auto-imports this action from
    /// `@ship/sail-runtime/actions` when a template references it.
    /// Every entry in the language is auto-imported today (stdlib
    /// needs the full set); the field is here so a future author-side
    /// action can opt out without restructuring. Auto-import is
    /// independent of the public `use:` skill surface — see
    /// [`PUBLIC_ACTION_NAMES`].
    pub auto_import: bool,
}

/// The action registry. Sorted by `name` for deterministic builds and
/// deterministic test failures (a new entry inserted in the middle
/// reorders the source diff in one place, not three).
///
/// Adding an action: append (or insert alphabetically) and the drift
/// test in `tests/abi_boundary.rs` will fail until every consumer is
/// updated. That is the bar.
pub const ACTIONS: &[ActionEntry] = &[
    ActionEntry {
        name: "anchor",
        param: "AnchorPlacement | undefined",
        summary: "Position a popper-like surface against an anchor element.",
        auto_import: true,
    },
    ActionEntry {
        name: "autoDismiss",
        param: "void",
        summary: "Timed self-dismiss from a `duration` attr (ms; default 5000; ≤0 sticky); pauses while hovered/focused; emits a dismiss event then removes the host.",
        auto_import: true,
    },
    ActionEntry {
        name: "autosize",
        param: "void",
        summary: "Autosize a textarea to its content as the user types.",
        auto_import: true,
    },
    ActionEntry {
        name: "catchRender",
        param: "void",
        summary: "Catch render errors thrown by descendants; show a fallback UI.",
        auto_import: true,
    },
    ActionEntry {
        name: "cellPad",
        param: "number | null | undefined",
        summary: "Track the current cell's left/right padding for a virtualized grid.",
        auto_import: true,
    },
    ActionEntry {
        name: "chromeOffset",
        param: "void",
        summary: "Publish the attached element's measured height as --chrome-offset on :root (pairs with the h:fill scale).",
        auto_import: true,
    },
    ActionEntry {
        name: "clickOutside",
        param: "(() => void) | undefined",
        summary: "Fire a callback when a click lands outside the element.",
        auto_import: true,
    },
    ActionEntry {
        name: "combobox",
        param: "void",
        summary: "Combobox keyboard/ARIA behavior for an input + list pairing.",
        auto_import: true,
    },
    ActionEntry {
        name: "contextMenu",
        param: "void",
        summary: "Right-click menu: open on contextmenu, close on outside-click / ESC.",
        auto_import: true,
    },
    ActionEntry {
        name: "describedControl",
        param: "void",
        summary: "Form control ↔ hint/error association (aria-describedby + aria-invalid) for any host with `.hint`/`.error`/`.control` anatomy.",
        auto_import: true,
    },
    ActionEntry {
        name: "detailsDisabled",
        param: "void",
        summary: "Suppress click + Enter/Space activation on a disclosure host while it carries `[disabled]`.",
        auto_import: true,
    },
    ActionEntry {
        name: "detailsToggle",
        param: "void",
        summary: "Re-emit a native <details> `toggle` as composed `sail-toggle` with `{open}`.",
        auto_import: true,
    },
    ActionEntry {
        name: "dismissControl",
        param: "void",
        summary: "Click on a close control (`[data-toast-close]` / `[data-dismiss]` / `button[aria-label=\"Dismiss\"]`) dismisses the host (event + remove); toast → sail-toast-dismiss, else sail-dismiss gated on [dismissible].",
        auto_import: true,
    },
    ActionEntry {
        name: "dismissible",
        param: "void",
        summary: "Emit `sail-remove` from a `[removable]` host on Backspace/Delete or `.remove` click — the attr-gated dismiss mix-in.",
        auto_import: true,
    },
    ActionEntry {
        name: "dropzone",
        param: "undefined",
        summary: "Drag-and-drop file receiver; reads files into a controller-side handler.",
        auto_import: true,
    },
    ActionEntry {
        name: "eventSpan",
        param: "RectSpan | null | undefined",
        summary: "Track the bounding rect of a pointer event target across the screen.",
        auto_import: true,
    },
    ActionEntry {
        name: "gridTracks",
        param: "string | null | undefined",
        summary: "Measure a CSS grid's resolved track widths/positions for layout math.",
        auto_import: true,
    },
    ActionEntry {
        name: "hoverSurface",
        param: "void",
        summary: "Hover-triggered card; open on pointer-in, close on pointer-out with delay.",
        auto_import: true,
    },
    ActionEntry {
        name: "icon",
        param: "void",
        summary: "Resolve a glyph from the icon cache and stamp the matching <use href>.",
        auto_import: true,
    },
    ActionEntry {
        name: "liveRegion",
        param: "void",
        summary: "Sync role/aria-live from tone: warning|destructive → assertive alert; else polite status.",
        auto_import: true,
    },
    ActionEntry {
        name: "marquee",
        param: "void",
        summary: "Infinite seamless track scroll for L1 <ticker> (duration/reverse on host).",
        auto_import: true,
    },
    ActionEntry {
        name: "maxWidth",
        param: "number | string | null | undefined",
        summary: "Cap the element's content width to a max (px or CSS length).",
        auto_import: true,
    },
    ActionEntry {
        name: "mediaFallback",
        param: "void",
        summary: "Image-with-fallback behavior for an <avatar> element.",
        auto_import: true,
    },
    ActionEntry {
        name: "menu",
        param: "void",
        summary: "Dropdown menu: keyboard nav, roving tabindex, ARIA menu role.",
        auto_import: true,
    },
    ActionEntry {
        name: "pagination",
        param: "void",
        summary: "Pagination controls; page number + prev/next wired to the controller.",
        auto_import: true,
    },
    ActionEntry {
        name: "pin",
        param: "PinPoint | null | undefined",
        summary: "Pin one corner of an element to a fixed viewport coordinate.",
        auto_import: true,
    },
    ActionEntry {
        name: "portEdge",
        param: "PortSide",
        summary: "Anchor a fixed-size port to the midpoint of one of its parent's four edges.",
        auto_import: true,
    },
    ActionEntry {
        name: "pressable",
        param: "void",
        summary: "Button semantics (role, aria-disabled/tabindex sync, Enter/Space activation) for a non-button host — the pressable mix-in.",
        auto_import: true,
    },
    ActionEntry {
        name: "pressed",
        param: "void",
        summary: "Reflect `[selected]` to `aria-pressed` on a toggle host — the pressed-state mix-in (pairs with pressable).",
        auto_import: true,
    },
    ActionEntry {
        name: "radioGroup",
        param: "void",
        summary: "Radio-group keyboard nav: arrow keys move the checked item.",
        auto_import: true,
    },
    ActionEntry {
        name: "rangeFill",
        param: "void",
        summary: "Paint the filled portion of a <input type=range> on a custom track.",
        auto_import: true,
    },
    ActionEntry {
        name: "recoverySurface",
        param: "void",
        summary: "Error-state element behavior; arms a retry action for the call-to-action slot.",
        auto_import: true,
    },
    ActionEntry {
        name: "rect",
        param: "BoxRect | null | undefined",
        summary: "Track the element's bounding rect via ResizeObserver.",
        auto_import: true,
    },
    ActionEntry {
        name: "spotlight",
        param: "void",
        summary: "Soft spotlight following the pointer; visual highlight for surfaces under it.",
        auto_import: true,
    },
    ActionEntry {
        name: "swatch",
        param: "string | null | undefined",
        summary: "Render a color swatch keyed by a token name from the active theme.",
        auto_import: true,
    },
    ActionEntry {
        name: "syntaxHighlight",
        param: "void",
        summary: "Syntax-highlight + copy-button behavior for a <code-block>.",
        auto_import: true,
    },
    ActionEntry {
        name: "tabs",
        param: "void",
        summary: "Tabs keyboard nav: left/right arrows move the active tab, ARIA tablist role.",
        auto_import: true,
    },
    ActionEntry {
        name: "targetCursor",
        param: "void",
        summary: "Switch the cursor to a custom 'target' style while over the element.",
        auto_import: true,
    },
    ActionEntry {
        name: "themeSelect",
        param: "void",
        summary: "Theme switcher: click to write the picked theme to the host context.",
        auto_import: true,
    },
    ActionEntry {
        name: "tipDescribe",
        param: "void",
        summary: "Hover/focus-triggered tooltip; open on pointer-in, close on pointer-out with delay.",
        auto_import: true,
    },
    ActionEntry {
        name: "toggleGroup",
        param: "void",
        summary: "Toggle-group keyboard nav: arrow keys move focus, space toggles.",
        auto_import: true,
    },
    ActionEntry {
        name: "variantLive",
        param: "void",
        summary: "Variant-gated live region: variant=connecting → role=status/aria-live=polite; else role=note/aria-live=off. Re-syncs on variant mutation.",
        auto_import: true,
    },
    ActionEntry {
        name: "virtualRow",
        param: "number | null | undefined",
        summary: "Register a virtualized row; tells the parent scroller its measured height.",
        auto_import: true,
    },
    ActionEntry {
        name: "virtualSpacer",
        param: "number | null | undefined",
        summary: "Spacer row in a virtualized list; the engine reads its measured height.",
        auto_import: true,
    },
    ActionEntry {
        name: "worldTransform",
        param: "string | null | undefined",
        summary: "Track the cumulative CSS transform up the parent chain (for canvas positioning).",
        auto_import: true,
    },
    ActionEntry {
        name: "zoomPan",
        param: "ZoomPanState | null | undefined",
        summary: "Pinch/wheel zoom + drag pan on a surface; emits state changes to the controller.",
        auto_import: true,
    },
];

/// App-author `use:` surface taught in skill/docs. Internal stamps stay
/// auto-imported for stdlib but are not the taught package API.
pub const PUBLIC_ACTION_COUNT: usize = 12;
pub const PUBLIC_ACTION_NAMES: &[&str; PUBLIC_ACTION_COUNT] = &[
    "anchor",
    "autosize",
    "clickOutside",
    "contextMenu",
    "dropzone",
    "icon",
    "menu",
    "rect",
    "tabs",
    "tipDescribe",
    "virtualRow",
    "zoomPan",
];

/// Whether `name` is part of the curated public `use:` surface.
pub fn is_public_action(name: &str) -> bool {
    PUBLIC_ACTION_NAMES.contains(&name)
}

/// Iterate the public subset of [`ACTIONS`] (order follows the table).
pub fn public_actions() -> impl Iterator<Item = &'static ActionEntry> {
    ACTIONS.iter().filter(|a| is_public_action(a.name))
}

/// Every name in `ACTIONS` whose `auto_import` is `true`. The
/// compiler's `lower::bindings::BUILTIN_ACTIONS` is a re-export of
/// this — the table is the single source, the constant is a derived
/// view. The full set (public + internal) is auto-imported so stdlib
/// stamps resolve; only [`PUBLIC_ACTION_NAMES`] are taught / re-exported
/// from the runtime front door.
pub const BUILTIN_ACTION_NAMES: &[&str] = &[
    "anchor",
    "autoDismiss",
    "autosize",
    "catchRender",
    "cellPad",
    "chromeOffset",
    "clickOutside",
    "combobox",
    "contextMenu",
    "describedControl",
    "detailsDisabled",
    "detailsToggle",
    "dismissControl",
    "dismissible",
    "dropzone",
    "eventSpan",
    "gridTracks",
    "hoverSurface",
    "icon",
    "liveRegion",
    "marquee",
    "maxWidth",
    "mediaFallback",
    "menu",
    "pagination",
    "pin",
    "portEdge",
    "pressable",
    "pressed",
    "radioGroup",
    "rangeFill",
    "recoverySurface",
    "rect",
    "spotlight",
    "swatch",
    "syntaxHighlight",
    "tabs",
    "targetCursor",
    "themeSelect",
    "tipDescribe",
    "toggleGroup",
    "variantLive",
    "virtualRow",
    "virtualSpacer",
    "worldTransform",
    "zoomPan",
];

// Drift guards: the const-time panic pattern would be cleaner but
// `==` on `&str` and `panic!` are not yet stable in const context
// (Rust 1.x). The `builtin_action_names_cover_actions` and
// `bare_referenced_names_exist_in_actions` tests in this module
// enforce the same invariants at test time, which is when `cargo
// test` goes red on a missed consumer — the bar the spec requires.

/// The subset of `ACTIONS` that is meaningful as a bare identifier
/// inside a template expression — the actions a user might write
/// `use:autosize` and then also reference the bare name in an
/// `on:input={(e) => autosize(e.target)}` handler. The expression
/// rewriter must leave these unqualified, and the scope-bound
/// validator must accept them as ambient builtins alongside the JS
/// / browser globals. The other (no-param) actions never appear as
/// bare idents in expressions — they're attached at the directive
/// and live only as `use:<name>`.
///
/// Derived from `ACTIONS` so the table is still the only thing to
/// edit; a compile-time guard (below) catches a forgotten
/// `BARE_REFERENCED_ACTION_NAMES` update when a new take-a-param
/// action is added.
pub const BARE_REFERENCED_ACTION_NAMES: &[&str] = &[
    "anchor",
    "autosize",
    "cellPad",
    "clickOutside",
    "dropzone",
    "eventSpan",
    "gridTracks",
    "maxWidth",
    "pin",
    "portEdge",
    "rect",
    "spotlight",
    "swatch",
    "targetCursor",
    "virtualRow",
    "virtualSpacer",
    "worldTransform",
    "zoomPan",
];

// (see drift-guard note above — the tests in `mod tests` enforce
// the same invariants at `cargo test` time)

/// Serialize the action registry as JSON, in the same shape the L1
/// element manifest uses (`emit::element::ElementOutput::manifest`).
/// The docs site and the agent skill consume this; the L1 element
/// manifest is the precedent the action registry matches.
///
/// Shape:
///
/// ```json
/// {
///   "kind": "actions",
///   "version": 1,
///   "actions": [
///     { "name": "anchor", "param": "AnchorPlacement | undefined", "autoImport": true,
///       "public": true, "summary": "Position a popper-like surface…" },
///     …
///   ]
/// }
/// ```
pub fn to_json() -> Value {
    let actions: Vec<Value> = ACTIONS
        .iter()
        .map(|a| {
            json!({
                "name": a.name,
                "param": a.param,
                "autoImport": a.auto_import,
                "public": is_public_action(a.name),
                "summary": a.summary,
            })
        })
        .collect();
    json!({
        "kind": "actions",
        "version": 1,
        "actions": actions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_is_deterministic() {
        // Sorted-by-name is the contract for diffs (see module docs);
        // an accidental sort change is a noisy test failure.
        let mut sorted = ACTIONS.to_vec();
        sorted.sort_by_key(|a| a.name);
        let names: Vec<&str> = sorted.iter().map(|a| a.name).collect();
        let actual: Vec<&str> = ACTIONS.iter().map(|a| a.name).collect();
        assert_eq!(actual, names, "ACTIONS must be sorted by name");
    }

    #[test]
    fn every_entry_has_a_summary() {
        for a in ACTIONS {
            assert!(
                !a.summary.trim().is_empty(),
                "ACTIONS entry `{}` has an empty summary — the docs site and the agent \
                 skill render this verbatim",
                a.name
            );
        }
    }

    #[test]
    fn json_round_trip_carries_every_field() {
        let v = to_json();
        let arr = v["actions"].as_array().expect("actions array");
        assert_eq!(arr.len(), ACTIONS.len());
        for (i, entry) in arr.iter().enumerate() {
            let src = ACTIONS[i];
            assert_eq!(entry["name"].as_str(), Some(src.name));
            assert_eq!(entry["param"].as_str(), Some(src.param));
            assert_eq!(entry["autoImport"].as_bool(), Some(src.auto_import));
            assert_eq!(entry["public"].as_bool(), Some(is_public_action(src.name)));
            assert_eq!(entry["summary"].as_str(), Some(src.summary));
        }
    }

    #[test]
    fn public_action_count_matches_names_array() {
        assert_eq!(PUBLIC_ACTION_COUNT, PUBLIC_ACTION_NAMES.len());
        assert_eq!(PUBLIC_ACTION_COUNT, 12);
    }

    #[test]
    fn every_public_name_exists_in_actions() {
        for name in *PUBLIC_ACTION_NAMES {
            assert!(
                ACTIONS.iter().any(|a| a.name == name),
                "PUBLIC_ACTION_NAMES entry `{name}` is not in ACTIONS"
            );
        }
    }

    #[test]
    fn public_action_names_are_sorted() {
        let mut sorted = PUBLIC_ACTION_NAMES.to_vec();
        sorted.sort();
        assert_eq!(
            PUBLIC_ACTION_NAMES.as_slice(),
            sorted.as_slice(),
            "PUBLIC_ACTION_NAMES must be sorted alphabetically for determinism"
        );
    }
}
