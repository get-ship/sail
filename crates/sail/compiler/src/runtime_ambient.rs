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
//! This is the fifth declared table, alongside the sx vocabulary, the L1
//! manifest, and the action manifest: the closed vocabulary is data, not
//! scattered string literals.

/// Specifier every ambient runtime ident is imported from. `/internal` is the
/// compiler ABI barrel (`packages/sail/runtime/src/internal.ts`): curated emit
/// symbols plus a re-export of the public language surface so ambient inject
/// and render modules share one specifier — bundle externalization treats a
/// controller module's imports identically to a render module's.
pub const RUNTIME_SPECIFIER: &str = "@ship/sail-runtime/internal";

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
    // `__invalidate` is never authored — it's the compiler's lowering
    // target for `invalidate(ref)` (see `check::query_keys`), so it can
    // only ever be REFERENCED (by the rewrite pass's own emitted call),
    // never bound by the author. Ambient injection is exactly the "self-
    // contained emitted module" property this table already exists for.
    (
        "__invalidate",
        "__invalidate",
        "compiler-only lowering target for `invalidate(ref)`; never authored directly",
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
        "onCleanup",
        "onCleanup",
        "register a callback to run when the controller tears down",
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
