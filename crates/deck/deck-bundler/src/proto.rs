//! Bundler result types.
//!
//! Formerly the bun-worker JSON-line protocol; with the in-process Rolldown
//! bundler (#163) only the build manifest remains — constructed in Rust, not
//! deserialized from a worker.

use std::path::PathBuf;

/// What one target produced.
#[derive(Debug, Clone)]
pub struct BundleManifest {
    /// Logical target name (e.g. `sui-runtime`, `views/git/view`).
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
}
