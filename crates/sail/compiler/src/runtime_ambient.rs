//! The ambient runtime vocabulary — the single source of truth for the runtime
//! identifiers an inline `<controller>` block may use WITHOUT importing them.
//!
//! The authoring ergonomic is that a controller writes `controller(...)`,
//! `signal(...)`, `derived(...)` as if they were language keywords. For the
//! emitted module to be a valid, self-contained ES module, the compiler injects
//! the import for exactly the ambient idents a block references — from this
//! table, in one place (`preprocess::wrap_controller_body`). That closes
//! `inv-sail-one-module-emit`: no emitted module carries a free variable, and no
//! consumer (deck) generates one by slicing text.
//!
//! This is the fifth declared table, alongside Style vocab (`offered_rows()`), the L1
//! manifest, and the action manifest: the closed vocabulary is data, not
//! scattered string literals.

/// Default specifier for ambient inject. `/internal` is the compiler ABI
/// barrel plus the language surface hello-world actually needs. Heavy
/// batteries (`query`, `createVirtualizer`, `timeline`) inject from their
/// own subpaths so a page that never mentions them does not download them.
pub const RUNTIME_SPECIFIER: &str = "@ship/sail-runtime/internal";

/// Where an ambient ident's *value* lives. Most stay on [`RUNTIME_SPECIFIER`].
/// Batteries get their own singleton (same split as `use:` → `/actions`).
pub fn ambient_module(authored: &str) -> &'static str {
    match authored {
        "createVirtualizer" => "@ship/sail-runtime/virtual",
        "query" | "mutation" | "__invalidate" => "@ship/sail-runtime/query",
        "timeline" => "@ship/sail-runtime/motion",
        _ => RUNTIME_SPECIFIER,
    }
}

/// The ambient authoring vocabulary of a controller: the reactive primitives,
/// the controller factory, and its ambient lifecycle (`mount`/`onCleanup`). A
/// block that references one of these without importing or declaring it gets
/// the import injected. Sorted by authored name; the emitter preserves this
/// order so injected import lists are stable.
///
/// Each entry is `(authored_ident, exported_ident, doc)`. Most idents import
/// under their own name (`exported_ident == authored_ident`). `mount` is the
/// one exception: `@ship/sail-runtime/internal` already exports a `mount`
/// (the F1 "mount a route into the DOM" host entry point — see
/// `packages/sail/runtime/src/mount.ts`), so the ambient lifecycle function a
/// controller body calls is exported under `mountAmbient` and imported
/// aliased back to `mount` (`import { mountAmbient as mount } from '...'`).
/// `doc` is one line of prose — the reference generator's source for the
/// controller-vocabulary section; it plays no role in import injection.
pub const RUNTIME_AMBIENT: &[(&str, &str, &str)] = &[
    (
        "__invalidate",
        "__invalidate",
        "compiler-only lowering target for `invalidate(ref)`; never authored directly",
    ),
    (
        "applyTheme",
        "applyTheme",
        "paint the document theme session (tokens API on runtime ambient)",
    ),
    (
        "asSailHost",
        "asSailHost",
        "coerce an action node (or host value) to opaque SailHost",
    ),
    (
        "attachBoundary",
        "attachBoundary",
        "register an error-boundary handle on a host element",
    ),
    (
        "batch",
        "batch",
        "coalesce multiple signal writes into one flush",
    ),
    (
        "contextKey",
        "contextKey",
        "mint a typed context key for provideContext / injectContext",
    ),
    (
        "controller",
        "controller",
        "the controller factory — wraps a setup function into a bound scope",
    ),
    (
        "createVirtualizer",
        "createVirtualizer",
        "windowed list adapter — reactive Virtual surface over the virtual core",
    ),
    (
        "debounce",
        "debounce",
        "coalesce rapid calls to a function until a quiet window elapses",
    ),
    (
        "defineHostProperty",
        "defineHostProperty",
        "install an identity property on a SailHost (get/set) without Object.defineProperty",
    ),
    (
        "defineStore",
        "defineStore",
        "declare a shared store definition (id + setup)",
    ),
    (
        "derived",
        "derived",
        "a read-only signal computed from other signals, recomputed on dependency change",
    ),
    (
        "detachBoundary",
        "detachBoundary",
        "remove an error-boundary handle from a host element",
    ),
    (
        "effect",
        "effect",
        "run a side effect on mount and whenever its read signals change",
    ),
    (
        "filteredList",
        "filteredList",
        "derive a filtered/sorted list signal from a source list",
    ),
    ("form", "form", "reactive form state helper"),
    (
        "hostCreateElement",
        "hostCreateElement",
        "create an element without naming document",
    ),
    (
        "hostCssVar",
        "hostCssVar",
        "read a CSS custom property from the document element",
    ),
    (
        "hostDispatch",
        "hostDispatch",
        "dispatch a named event on a handed node (Pure-safe Event stand-in)",
    ),
    (
        "hostFindById",
        "hostFindById",
        "document.getElementById without naming document",
    ),
    (
        "hostGetAttr",
        "hostGetAttr",
        "read a host attribute via SailHost (Pure-safe)",
    ),
    (
        "hostHasAttr",
        "hostHasAttr",
        "test host attribute presence via SailHost (Pure-safe)",
    ),
    (
        "hostIsDarkMode",
        "hostIsDarkMode",
        "true when html data-mode is dark",
    ),
    (
        "hostListenDocument",
        "hostListenDocument",
        "add a document event listener; returns disposer",
    ),
    (
        "hostObserveRootAttrs",
        "hostObserveRootAttrs",
        "MutationObserver on documentElement attributes; returns disposer",
    ),
    (
        "hostSetAttr",
        "hostSetAttr",
        "write or remove a host attribute via SailHost (Pure-safe)",
    ),
    (
        "injectContext",
        "injectContext",
        "read a value published via provideContext on an ancestor scope",
    ),
    (
        "interval",
        "interval",
        "run a callback on a fixed period while the controller is mounted",
    ),
    (
        "mount",
        "mountAmbient",
        "run a callback once, after the controller's first render",
    ),
    (
        "mutation",
        "mutation",
        "public write primitive — returns a Mutation handle",
    ),
    (
        "navigate",
        "navigate",
        "navigate to a path via the host router port",
    ),
    (
        "objectUrlFromBytes",
        "objectUrlFromBytes",
        "mint a blob object URL from bytes (Pure-safe Blob/createObjectURL stand-in)",
    ),
    (
        "observeContentBox",
        "observeContentBox",
        "observe an element content-box size (Pure-safe ResizeObserver stand-in)",
    ),
    (
        "observeHostAttrs",
        "observeHostAttrs",
        "observe attribute mutations on a host node; returns disposer",
    ),
    (
        "onCleanup",
        "onCleanup",
        "register a callback to run when the controller tears down",
    ),
    (
        "persistTheme",
        "persistTheme",
        "persist the chosen theme id for next boot",
    ),
    (
        "provideContext",
        "provideContext",
        "publish a value on the current controller scope for injectContext",
    ),
    (
        "provideStore",
        "provideStore",
        "park a scoped store instance on the current controller scope",
    ),
    (
        "query",
        "query",
        "public read primitive — compiler derives the cache key",
    ),
    (
        "readStoredTheme",
        "readStoredTheme",
        "read the persisted theme id",
    ),
    (
        "revokeObjectUrl",
        "revokeObjectUrl",
        "revoke a URL from objectUrlFromBytes",
    ),
    (
        "scheduleFrame",
        "scheduleFrame",
        "schedule work on the next animation frame (Pure-safe rAF substitute)",
    ),
    (
        "signal",
        "signal",
        "a reactive value cell — read by calling it, written by calling it with a value",
    ),
    (
        "theme",
        "theme",
        "runtime theme surface — current / preview / commit / applyScope",
    ),
    (
        "timeline",
        "timeline",
        "compose multi-track motion timelines",
    ),
    (
        "untrack",
        "untrack",
        "read signals inside without subscribing the enclosing effect/derived",
    ),
    (
        "useStore",
        "useStore",
        "resolve nearest-or-root store instance for a StoreDef",
    ),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_is_sorted_and_unique_by_authored_name() {
        let mut sorted = RUNTIME_AMBIENT.to_vec();
        sorted.sort_unstable_by_key(|(authored, _, _)| *authored);
        sorted.dedup_by_key(|(authored, _, _)| *authored);
        assert_eq!(
            sorted.as_slice(),
            RUNTIME_AMBIENT,
            "RUNTIME_AMBIENT must stay sorted + unique by authored name (stable injected import order)"
        );
    }
}
