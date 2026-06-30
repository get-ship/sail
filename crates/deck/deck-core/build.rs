//! Cargo validates SUI bundle production.
//!
//! `include_str!` in `src/assets.rs` bakes bundle outputs into the
//! framework binary at compile time. Every asset under `assets/` is
//! gitignored — the build emits them on demand and `dispatch.sh`
//! bootstraps fresh worktrees by copying from the launching repo (or
//! rebuilding when that's empty). The framework bundles (sui-runtime,
//! sui-dom, sui-sdk-client, sui-components) are produced by
//! `cargo run -p deck-cli -- build-assets` and land in `.deck/framework/`.
//! This build script copies them into `crates/deck/deck-core/assets/` (the
//! `include_str!` staging path) before the existence/empty guard runs.
//! The lucide icon set is committed source (`assets/lucide-icons.json`) —
//! the daemon embeds it and serves per-icon at `/_deck/icons/{name}.svg`;
//! regenerate after a lucide-static version bump via
//! `cargo run -p xtask-deck -- gen-icons`.
//! The workspace catalog is emitted to `.deck/catalogs/workspace.json`
//! by `build-assets` and read at runtime by `src/packs.rs`.
//!
//! What this build script DOES:
//! 1. Copy `.deck/framework/sui-*.js` → `assets/sui-*.js` (the staging path
//!    `include_str!` requires). This keeps the canonical bake target in
//!    `.deck/framework/` while the Rust compiler still reads a fixed path.
//! 2. Verify the staged assets are populated (panic with actionable message
//!    when they are not).
//!
//! Bootstrap contract: run `cargo run -p deck-cli -- build-assets` before
//! `cargo build`. On fresh worktrees, `dispatch.sh` copies `.deck/framework/`
//! from the launching repo. `lucide-icons.json` and `live-reload.js` are
//! committed source — already present in a fresh clone.

use std::fs;
use std::path::PathBuf;

/// Framework singletons that must be baked into the binary.
/// Sourced from `.deck/framework/`, staged to `assets/` by this script.
/// `deck-route.js` is the `@ship/deck/router` framework singleton
/// (defineRoutes / route / navigate / setEditMode) — every per-app
/// bundler-generated entry imports it, so the framework ships it as
/// `include_str!("../assets/deck-route.js")` from `assets.rs`.
const FRAMEWORK_BUNDLES: &[&str] = &[
    "sui-runtime.js",
    "sui-dom.js",
    "sui-sdk-client.js",
    "deck-route.js",
];

/// All assets that must be non-empty before the build proceeds.
const ASSETS: &[&str] = &[
    "sui-runtime.js",
    "sui-dom.js",
    "sui-sdk-client.js",
    "deck-route.js",
    "lucide-icons.json",
];

// Deck's OWN source roots — the JS/TS that deck's framework singletons are
// built from. Host-app source roots (e.g. apps/shell/src) are NOT listed
// here: they are runtime cfg (`deck.jsonc`) and cannot be read in a build
// script. The bake writes host assets at runtime; build.rs need not track
// them for rerun invalidation.
const COARSE_SOURCE_ROOTS: &[&str] = &[
    "packages/sui/runtime/src",
    "packages/sui/dom/src",
    "packages/deck/sdk-client/src",
];

fn main() {
    let manifest_dir: PathBuf = std::env::var_os("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR")
        .into();
    let assets = manifest_dir.join("assets");
    let repo_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("repo root (three levels above crates/deck/deck-core)")
        .to_path_buf();
    let framework_src = repo_root.join(".deck").join("framework");

    fs::create_dir_all(&assets)
        .unwrap_or_else(|e| panic!("build.rs: mkdir assets: {e}"));

    // ── Stage: copy .deck/framework/sui-*.js → assets/sui-*.js ───────────────
    //
    // The canonical bake target is .deck/framework/ (produced by
    // `cargo run -p deck-cli -- build-assets`). include_str! requires a
    // fixed compile-time path inside the crate, so we stage the bytes here.
    // Only copy when the source exists — on a fresh worktree where dispatch.sh
    // has already populated .deck/framework/, this is the fast path; if neither
    // the framework dir nor the staged copy exists, the guard below fires with
    // the full instructions.
    if framework_src.exists() {
        for name in FRAMEWORK_BUNDLES {
            let src = framework_src.join(name);
            let dst = assets.join(name);
            if src.exists() && !file_is_empty(&src) {
                fs::copy(&src, &dst).unwrap_or_else(|e| {
                    panic!("build.rs: failed to stage {name} from .deck/framework/: {e}")
                });
            }
        }
    }

    // ── Guard: verify every required asset exists and is non-empty ────────────
    let mut missing: Vec<&str> = Vec::new();
    let mut empty: Vec<&str> = Vec::new();
    for rel in ASSETS {
        let p = assets.join(rel);
        if !p.exists() {
            missing.push(rel);
        } else if file_is_empty(&p) {
            empty.push(rel);
        }
    }

    if !missing.is_empty() || !empty.is_empty() {
        let lines = missing
            .iter()
            .map(|r| format!("    - missing: {r}"))
            .chain(empty.iter().map(|r| format!("    - empty:   {r}")))
            .collect::<Vec<_>>()
            .join("\n");
        panic!(
            "build.rs: SUI bundle assets are missing or empty.\n{lines}\n\n\
             Framework bundles are baked to .deck/framework/ by:\n\n  \
             cargo build -p sui-compiler   # the compiler the bundler uses\n  \
             cargo run -p deck-cli -- build-assets   # framework bundles via in-process Rolldown\n\n\
             build.rs then copies them from .deck/framework/ into crates/deck/deck-core/assets/\n\
             (the include_str! staging path). On fresh worktrees, dispatch.sh bootstraps\n\
             .deck/framework/ from the launching repo. lucide-icons.json and live-reload.js\n\
             are committed source — already present in a fresh clone."
        );
    }

    // ── rerun-if-changed: watch .deck/framework/ for the 4 bundles ───────────
    //
    // When any of the 4 framework bundles in .deck/framework/ change,
    // Cargo re-runs build.rs so the staged copies stay in sync.
    for name in FRAMEWORK_BUNDLES {
        let fw_path = framework_src.join(name);
        println!("cargo:rerun-if-changed={}", fw_path.display());
    }
    // Also watch the staged copies so a manual edit to assets/ triggers a rebuild.
    for rel in ASSETS {
        println!("cargo:rerun-if-changed=assets/{rel}");
    }

    // The `.deck/.cache` dir (written by the bake at runtime) could drive
    // fine-grained rerun hints, but build.rs cannot know the host's
    // `host_assets_root` — that is runtime config. Use the coarse source
    // roots below instead; cargo will re-check on any source change.

    for root in COARSE_SOURCE_ROOTS {
        let abs = repo_root.join(root);
        if abs.exists() {
            println!("cargo:rerun-if-changed={}", abs.display());
        }
    }
}

fn file_is_empty(p: &std::path::Path) -> bool {
    fs::metadata(p).map(|m| m.len() == 0).unwrap_or(true)
}
