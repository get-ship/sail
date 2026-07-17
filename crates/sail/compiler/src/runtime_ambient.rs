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

/// Specifier every ambient runtime ident is imported from. `/internal` is a
/// total re-export of the public runtime API (`packages/sail/runtime/src/internal.ts`
/// is `export * from './index'`), and it is what the compiled render module
/// already imports its helpers from — so bundle externalization treats a
/// controller module's imports identically to a render module's.
pub const RUNTIME_SPECIFIER: &str = "@ship/sail-runtime/internal";

/// The ambient authoring vocabulary of a controller: the reactive primitives,
/// the controller factory, and its ambient lifecycle (`mount`/`onCleanup`). A
/// block that references one of these without importing or declaring it gets
/// the import injected. Sorted by authored name; the emitter preserves this
/// order so injected import lists are stable.
///
/// Each entry is `(authored_ident, exported_ident)`. Most idents import
/// under their own name (`exported_ident == authored_ident`). `mount` is the
/// one exception: `@ship/sail-runtime/internal` already exports a `mount`
/// (the F1 "mount a view into the DOM" host entry point — see
/// `packages/sail/runtime/src/mount.ts`), so the ambient lifecycle function a
/// controller body calls is exported under `mountAmbient` and imported
/// aliased back to `mount` (`import { mountAmbient as mount } from '...'`).
pub const RUNTIME_AMBIENT: &[(&str, &str)] = &[
    ("batch", "batch"),
    ("controller", "controller"),
    ("derived", "derived"),
    ("effect", "effect"),
    ("mount", "mountAmbient"),
    ("onCleanup", "onCleanup"),
    ("signal", "signal"),
    ("untrack", "untrack"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_is_sorted_and_unique_by_authored_name() {
        let mut sorted = RUNTIME_AMBIENT.to_vec();
        sorted.sort_unstable_by_key(|(authored, _)| *authored);
        sorted.dedup_by_key(|(authored, _)| *authored);
        assert_eq!(
            sorted.as_slice(),
            RUNTIME_AMBIENT,
            "RUNTIME_AMBIENT must stay sorted + unique by authored name (stable injected import order)"
        );
    }
}
