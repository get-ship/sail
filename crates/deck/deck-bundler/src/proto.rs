//! Bundler result types.
//!
//! Formerly the bun-worker JSON-line protocol; with the in-process Rolldown
//! bundler (#163) only the build manifest remains — constructed in Rust, not
//! deserialized from a worker.

use std::path::PathBuf;

/// A side-effect-only module that the bundler dropped (via the importing
/// package's `sideEffects: false` / array contract) so the build can emit a
/// loud warning. Without this, a CSS or `customElements.define` import gets
/// silently tree-shaken and the author has no way to know why their
/// registration never runs (sail-sideeffects footgun).
#[derive(Debug, Clone)]
pub struct DroppedSideEffectModule {
    /// Absolute path of the module that was dropped.
    pub module_path: PathBuf,
    /// Path of the importer that pulled it in. Empty when the importer
    /// was the entry itself (no other source referenced the module).
    pub importer: String,
    /// Why it was dropped: the resolved `sideEffects` value from the
    /// owning `package.json` (e.g. `false` or the array form).
    pub reason: String,
}

/// What one target produced.
#[derive(Debug, Clone)]
pub struct BundleManifest {
    /// Logical target name (e.g. `sail-runtime`, `views/git/view`).
    pub target: String,
    /// Absolute output path.
    pub out_path: String,
    /// Output size in bytes.
    pub bytes: usize,
    /// Source inputs (entry / aggregate sources) — for watch dependency hints.
    pub inputs: Vec<String>,
    /// Resolved module graph: every on-disk source file that contributed to
    /// this bundle (absolute paths, vendored dirs excluded). Populated from
    /// Rolldown's `OutputChunk.module_ids` after each build; empty for
    /// synthesized manifests (single-flight coalesced waiters).
    pub modules: Vec<PathBuf>,
    /// Side-effect-only modules the bundler dropped because of an owning
    /// `package.json` `sideEffects: false` / array match. The build emits a
    /// warning per entry so an author whose `customElements.define` import
    /// disappeared sees why. Empty when no drops occurred (the common case
    /// for a clean tree-shake).
    pub dropped_side_effect_modules: Vec<DroppedSideEffectModule>,
}
