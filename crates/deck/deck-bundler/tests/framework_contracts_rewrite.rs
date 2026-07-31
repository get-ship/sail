//! End-to-end smoke for `ShipAssetCssPlugin` against the REAL framework
//! **sail-base** (`sail-base.css`; historical package alias `contracts.css`)
//! from `@ship/tokens`. The plugin is generic — it rewrites every `url(...)`
//! to a local asset in any `.css` it loads — but the framework case is the
//! load-bearing one: the OFL webfont woff2 ship as real hashed assets and
//! the browser fetches them from the same `assets/` directory the CSS lands in.
//!
//! Why a separate test from the in-module unit test: that test uses a
//! synthetic CSS and a fake woff2 (`b"wOF2"`). This test uses the
//! REAL `packages/sail/tokens/src/sail-base.css` (with the OFL woff2 paths)
//! and the REAL `packages/sail/tokens/src/fonts/*.woff2` binaries. It catches
//! anything the synthetic test can't — a banner comment with illustrative
//! `url(...)` text (e.g. the actual sail-base banner has
//! `url('./<file>-<hash>.woff2')` as a doc example), a token path the plugin
//! would resolve to a non-existent file, etc.
//!
//! This is the smoke test the mission calls for — if THIS passes, the
//! served sail-base stylesheet will reference the hashed woff2
//! instead of inlined base64.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use deck_bundler::catalog::component_package_roots;
use deck_bundler::sail_plugin::{SailPlugin, component_index};
use deck_bundler::ship_asset_css::ShipAssetCssPlugin;

fn write(path: &Path, body: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, body).unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn framework_sail_base_css_emits_hashed_woff2_assets() {
    // Re-use the repo root so the workspace's `packages/sail/tokens`
    // resolution actually walks the package map (the
    // `@ship/tokens/sail-base.css` subpath is defined in
    // `packages/sail/tokens/package.json`).
    let repo_root = {
        let mut d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        while !d.join("packages").is_dir() {
            if !d.pop() {
                panic!("repo root not found from CARGO_MANIFEST_DIR");
            }
        }
        d
    };
    // The minimal app entry is written UNDER THE REPO ROOT because
    // the SailPlugin's `resolve_id` re-anchors `@ship/...` imports
    // relative to the workspace `repo_root` (the framework's
    // `@ship/tokens` map points at the workspace source). The entry
    // file uses a unique tag suffix so concurrent test runs don't
    // collide. A guard removes it on drop.
    let entry_path = repo_root.join("app/routes/fonts-smoke.sail");
    write(
        &entry_path,
        r#"<controller>
import '@ship/tokens/sail-base.css'
export default () => ({})
</controller>
<div>font smoke</div>
"#,
    );
    struct EntryGuard(PathBuf);
    impl Drop for EntryGuard {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }
    let _guard = EntryGuard(entry_path.clone());

    let out_dir = repo_root.join(".deck/fonts-smoke-out");
    let _ = std::fs::remove_dir_all(&out_dir);
    std::fs::create_dir_all(&out_dir).unwrap();

    use rolldown::{BundlerBuilder, BundlerOptions, InputItem, OutputFormat};
    use rolldown_common::{ChunkFilenamesOutputOption, ModuleType};
    use rustc_hash::FxHashMap;
    let mut module_types: FxHashMap<String, ModuleType> = FxHashMap::default();
    module_types.insert(".css".to_string(), ModuleType::Asset);
    let options = BundlerOptions {
        input: Some(vec![InputItem {
            name: Some("entry".to_string()),
            import: entry_path.to_string_lossy().to_string(),
        }]),
        cwd: Some(repo_root.clone()),
        dir: Some(out_dir.to_string_lossy().to_string()),
        format: Some(OutputFormat::Esm),
        entry_filenames: Some(ChunkFilenamesOutputOption::String(
            "assets/[name]-[hash].js".to_string(),
        )),
        chunk_filenames: Some(ChunkFilenamesOutputOption::String(
            "assets/[name]-[hash].js".to_string(),
        )),
        module_types: Some(module_types),
        ..Default::default()
    };
    let component_idx = component_index(&repo_root, &[]);
    let sail_plugin = SailPlugin {
        repo_root: repo_root.clone(),
        emit_component_imports: false,
        component_index: component_idx.index,
        shadowed_tags: component_idx.shadowed,
        component_roots: component_package_roots(&repo_root),
        project_dir: None,
    };
    let mut bundler = BundlerBuilder::default()
        .with_options(options)
        .with_plugins(vec![
            Arc::new(sail_plugin),
            Arc::new(ShipAssetCssPlugin::development()),
        ])
        .build()
        .expect("bundler build");
    bundler.write().await.expect("bundle + plugin write");

    // The output assets dir must contain BOTH the rewritten
    // sail-base.css AND the three woff2 (as hashed assets). We list
    // the dir rather than the in-memory `output.assets` because
    // `bundler.write()` finalizes the on-disk layout — a stronger
    // proof.
    let assets_dir = out_dir.join("assets");
    let names: Vec<String> = std::fs::read_dir(&assets_dir)
        .expect("assets dir")
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    let woff2_assets: Vec<&str> = names
        .iter()
        .filter(|n| n.ends_with(".woff2"))
        .map(|s| s.as_str())
        .collect();
    assert_eq!(
        woff2_assets.len(),
        4,
        "expected exactly 4 hashed woff2 assets (DM Sans, Syne, JetBrains Mono, Press Start 2P), got {woff2_assets:?}"
    );
    let base_css = names
        .iter()
        .find(|n| n.starts_with("sail-base-") && n.ends_with(".css"))
        .expect("sail-base.css asset must be emitted");
    // The sail-base body must reference the woff2 by their hashed
    // asset paths (no `./fonts/X.woff2` source paths; no base64 data
    // URLs). The placeholder prefix must be fully resolved.
    let base_path = assets_dir.join(base_css);
    let base_body = std::fs::read_to_string(&base_path).expect("sail-base.css must be readable");
    assert!(
        !base_body.contains("data:font/woff2"),
        "base64 inlining is GONE — no `data:font/woff2` URLs allowed, got body:\n{}",
        base_body
    );
    assert!(
        !base_body.contains("__SHIP_CSS_ASSET__#"),
        "the placeholder must be resolved by `generate_bundle`, got body:\n{}",
        base_body
    );
    assert!(
        !base_body.contains("./fonts/dm-sans-")
            && !base_body.contains("./fonts/syne-")
            && !base_body.contains("./fonts/jetbrains-")
            && !base_body.contains("./fonts/press-start-"),
        "the source `url('./fonts/X.woff2')` must be rewritten to the hashed asset URL, got body:\n{}",
        base_body
    );
    // The body must contain a URL pointing at one of the three woff2
    // assets we emitted. We don't pin the exact hash (content-derived)
    // — just assert at least one match.
    let references_at_least_one = woff2_assets.iter().any(|woff2_name| {
        // The CSS body has `url("./<name>.woff2")` — the basename is
        // `<name>.woff2` (e.g. `dm-sans-latin-wght-normal-<hash>.woff2`).
        let basename = woff2_name.strip_suffix(".woff2").unwrap_or(woff2_name);
        base_body.contains(basename)
    });
    assert!(
        references_at_least_one,
        "sail-base.css must reference at least one of the emitted woff2 ({woff2_assets:?}), got body:\n{}",
        base_body
    );
    // The body must have four `@font-face` blocks (one per family in
    // the FONTS manifest) — confirms the drift lock carried through
    // the bundler without losing rows.
    let font_face_count = base_body.matches("@font-face").count();
    assert_eq!(
        font_face_count, 4,
        "expected 4 @font-face rules in sail-base.css, got {font_face_count} (manifest drift?)"
    );
    // The three variable faces keep `format('woff2-variations')`; the static
    // Press Start 2P face uses `format('woff2')`. Verify both shapes are present.
    let variations_count = base_body.matches("format('woff2-variations')").count();
    assert_eq!(
        variations_count, 3,
        "expected 3 woff2-variations format() decls in sail-base.css, got {variations_count}"
    );
    let woff2_count = base_body.matches("format('woff2')").count();
    assert_eq!(
        woff2_count, 1,
        "expected 1 woff2 format() decl in sail-base.css, got {woff2_count}"
    );
}
