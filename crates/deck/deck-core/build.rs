//! Cargo validates SAIL bundle production.
//!
//! `include_str!` in `src/assets.rs` bakes bundle outputs into the
//! framework binary at compile time. Every asset under `assets/` is
//! gitignored — the build emits them on demand and `dispatch.sh`
//! bootstraps fresh worktrees by copying from the launching repo (or
//! rebuilding when that's empty). The framework bundles (sail-runtime,
//! sail-dom, sail-sdk-client, sail-components) are produced by
//! `cargo run -p deck-cli -- build-assets` and land in `.deck/framework/`.
//! This build script copies them into `crates/deck/deck-core/assets/` (the
//! `include_str!` staging path) before the existence/empty guard runs.
//! Icon glyphs are not committed here. Sail resolves used roles; Deck
//! writes those SVGs into the app `.deck` graph. The daemon serves that
//! bake — it does not embed a pack.
//! The workspace catalog is emitted to `.deck/catalogs/workspace.json`
//! by `build-assets` and read at runtime by `src/packs.rs`.
//!
//! What this build script DOES:
//! 1. Copy `.deck/framework/sail-*.js` → `assets/sail-*.js` (the staging path
//!    `include_str!` requires). This keeps the canonical bake target in
//!    `.deck/framework/` while the Rust compiler still reads a fixed path.
//! 2. Ensure every staged framework bundle EXISTS so `include_str!` can compile.
//!    When `.deck/framework/*.js` is absent or empty (a from-zero checkout, or
//!    the very build that produces those bundles: `deck-cli` depends on
//!    `deck-core`), write a bootstrap STUB in its place and emit a
//!    `cargo:warning` rather than panicking. The stub lets the deck stack
//!    compile so `build-assets` can run and regenerate the real bundles; the
//!    `rerun-if-changed` hooks below then re-stage the real bytes on the next
//!    `cargo build`. This breaks the chicken/egg: from-zero `build-assets` no
//!    longer needs the bundles it is itself producing.
//!
//! Bootstrap contract (now self-healing): run
//! `cargo run -p deck-cli -- build-assets` to produce `.deck/framework/*.js`,
//! then `cargo build` to bake them in. The first `build-assets` compiles
//! `deck-core` against stubs (see above), so no pre-seeded `.deck/framework/`
//! or sibling-worktree copy is required. `live-reload.js` and
//! `error-overlay.js` are committed source — already present in a fresh
//! clone, so those remain hard requirements (a missing one is a broken
//! checkout, not the bootstrap race).

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Framework singletons that must be baked into the binary.
/// Sourced from `.deck/framework/`, staged to `assets/` by this script.
/// `deck-route.js` is the `@ship/deck/router` framework singleton
/// (defineRoutes / route / navigate / setEditMode) — every per-app
/// bundler-generated entry imports it, so the framework ships it as
/// `include_str!("../assets/deck-route.js")` from `assets.rs`.
const FRAMEWORK_BUNDLES: &[&str] = &[
    "sail-runtime.js",
    "sail-dom.js",
    "sail-sdk-client.js",
    "deck-route.js",
    // In-browser compiler + LSP singletons. Built by `deck build-assets` from
    // `@ship/sail-compiler-wasm` (wasm-pack `pkg/`) and `@ship/sail-lsp-browser`
    // subpaths; the importmap points every app at `/_deck/*.js`, so they must be
    // baked into the dev daemon the same way runtime/dom are.
    "sail-compiler-wasm.js",
    "sail-lsp-completions.js",
    "sail-lsp-diagnostics.js",
    // Framework-shipped Sail Studio bridge (hotkeys/FAB under sail dev).
    "sail-studio-bridge.js",
];

/// Committed source assets that must be non-empty before the build proceeds.
/// Unlike the framework bundles, these ship in a fresh clone — a missing one
/// is a broken checkout, not the build-assets chicken/egg, so they stay a hard
/// requirement (no stub, no graceful degrade).
const COMMITTED_ASSETS: &[&str] = &[];

/// Content written to a framework bundle's staging path when the real bake
/// output is not yet available (from-zero checkout, or the very build that
/// produces the bundles). It only has to let `include_str!` compile;
/// `build-assets` overwrites `.deck/framework/` with the real bytes, which
/// the `rerun-if-changed` hooks re-stage on the next `cargo build`.
const BOOTSTRAP_STUB: &str =
    "/* deck bootstrap stub — regenerate with `cargo run -p deck-cli -- build-assets`. */\n";

// Deck's OWN source roots — the JS/TS that deck's framework singletons are
// built from. Host-app source roots (e.g. apps/shell/src) are NOT listed
// here: they are runtime cfg (`deck.jsonc`) and cannot be read in a build
// script. The bake writes host assets at runtime; build.rs need not track
// them for rerun invalidation.
const COARSE_SOURCE_ROOTS: &[&str] = &[
    "packages/sail/runtime/src",
    "packages/sail/dom/src",
    "packages/deck/sdk-client/src",
    "packages/sail/studio-bridge/src",
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
    let framework_src = deck_config::deck_out_path(&repo_root).join("framework");
    let framework_stamp = deck_config::framework_stamp_path(&repo_root);

    fs::create_dir_all(&assets).unwrap_or_else(|e| panic!("build.rs: mkdir assets: {e}"));

    // ── Stage: copy .deck/framework/sail-*.js → assets/sail-*.js ───────────────
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
                copy_if_changed(&src, &dst).unwrap_or_else(|e| {
                    panic!("build.rs: failed to stage {name} from .deck/framework/: {e}")
                });
            }
        }
    }

    // ── Bootstrap: stub any framework bundle that isn't staged yet ────────────
    //
    // `include_str!("../assets/<name>")` (src/assets.rs) needs the file to
    // exist at compile time. On a from-zero checkout — and during the very
    // `build-assets` run that produces these bundles, since `deck-cli` depends
    // on `deck-core` — `.deck/framework/*.js` is absent, so nothing was staged
    // above. Write a stub so the deck stack compiles; `build-assets` then bakes
    // the real bundles and `rerun-if-changed` re-stages them next build. This
    // is the fix for the build-assets chicken/egg (from-zero cannot borrow a
    // sibling worktree's `.deck/framework/`).
    let mut stubbed: Vec<&str> = Vec::new();
    for name in FRAMEWORK_BUNDLES {
        let dst = assets.join(name);
        if !dst.exists() || file_is_empty(&dst) {
            write_atomic(&dst, BOOTSTRAP_STUB.as_bytes())
                .unwrap_or_else(|e| panic!("build.rs: writing bootstrap stub for {name}: {e}"));
            stubbed.push(name);
        }
    }
    if !stubbed.is_empty() {
        println!(
            "cargo:warning=deck-core: framework bundles not yet baked ({}) — staged bootstrap \
             stubs so the deck stack can compile. Run `cargo run -p deck-cli -- build-assets` to \
             produce .deck/framework/*.js, then rebuild to bake the real bundles.",
            stubbed.join(", ")
        );
    }

    // ── Guard: committed source assets are a hard requirement ─────────────────
    // These ship in a fresh clone; a missing/empty one is a broken checkout,
    // not the bootstrap race, so keep the actionable panic.
    let mut missing: Vec<&str> = Vec::new();
    let mut empty: Vec<&str> = Vec::new();
    for rel in COMMITTED_ASSETS {
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
            "build.rs: committed source assets are missing or empty.\n{lines}\n\n\
             These ship in a fresh clone (unlike the framework bundles, which build.rs\n\
             now stubs and `build-assets` bakes). A missing one means a broken checkout."
        );
    }

    // ── rerun-if-changed: watch the framework bake stamp ───────────────────────
    //
    // `deck build-assets` touches crates/deck/deck-core/framework-stamp after
    // it finishes baking. Watching that stamp (rather than the individual
    // `.deck/framework/*.js` files) lets Cargo re-run this build script once
    // per explicit bake; tests pass `--no-stamp` so they do not re-trigger
    // while downstream doctests are queued.
    //
    // The stamp is a tracked empty file in this crate — never under `.deck/`.
    // `build-assets` updates mtime only. Cargo treats a *missing*
    // `rerun-if-changed` path as a stale fingerprint, so the file must exist
    // on a fresh checkout.
    //
    // We intentionally do NOT watch `assets/<name>`: those files are WRITTEN by
    // this build script. Watching outputs created a self-triggering rebuild loop
    // (fs::copy touched mtime every run, so Cargo reran build.rs, which copied
    // again). That loop was the other source of the E0463 invalidation.
    // See claim c-2026-07-19T18-12-45-685Z-f499ce4b.
    println!("cargo:rerun-if-changed={}", framework_stamp.display());
    // Watch committed source assets (inputs we do not write) so a manual edit
    // triggers a rebuild.
    for rel in COMMITTED_ASSETS {
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

/// Copy `from` to `to`, but only if the bytes differ. Preserves `to`'s mtime
/// when the source has not changed so Cargo does not re-run this build script
/// in a loop.
fn copy_if_changed(from: &Path, to: &Path) -> std::io::Result<()> {
    let src = fs::read(from)?;
    if let Ok(dst_meta) = fs::metadata(to) {
        if dst_meta.len() == src.len() as u64 {
            if let Ok(dst) = fs::read(to) {
                if dst == src {
                    return Ok(());
                }
            }
        }
    }
    write_atomic(to, &src)
}

/// Write `contents` to `path` atomically via a same-directory temp file +
/// `rename`. A reader sees the old bytes or the new bytes, never a truncated
/// file.
fn write_atomic(path: &Path, contents: &[u8]) -> std::io::Result<()> {
    let tmp = path.with_extension("tmp");
    {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(contents)?;
        f.flush()?;
    }
    fs::rename(&tmp, path)
}
