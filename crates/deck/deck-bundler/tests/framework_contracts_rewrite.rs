//! End-to-end smoke for `ShipAssetCssPlugin` against the REAL framework
//! `contracts.css` from `@ship/tokens`. The plugin is generic — it
//! rewrites every `url(...)` to a local asset in any `.css` it loads —
//! but the framework case is the load-bearing one: the three OFL
//! variable woff2 ship as real hashed assets and the browser fetches
//! them from the same `assets/` directory the CSS lands in.
//!
//! Why a separate test from the in-module unit test: that test uses a
//! synthetic CSS and a fake woff2 (`b"wOF2"`). This test uses the
//! REAL `packages/sui/tokens/src/contracts.css` (with the three OFL
//! woff2 paths) and the REAL `packages/sui/tokens/src/fonts/*.woff2`
//! binaries. It catches anything the synthetic test can't — a banner
//! comment with illustrative `url(...)` text (e.g. the actual
//! `contracts.css` banner has `url('./<file>-<hash>.woff2')` as a doc
//! example), a token path the plugin would resolve to a non-existent
//! file, etc.
//!
//! This is the smoke test the mission calls for — if THIS passes, the
//! served `contracts.css` will reference the three hashed woff2
//! instead of inlined base64.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use deck_bundler::catalog::component_package_roots;
use deck_bundler::ship_asset_css::ShipAssetCssPlugin;
use deck_bundler::sui_plugin::{component_index, SuiPlugin};

fn write(path: &Path, body: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, body).unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn framework_contracts_css_emits_hashed_woff2_assets() {
    // Re-use the repo root so the workspace's `packages/sui/tokens`
    // resolution actually walks the package map (the
    // `@ship/tokens/contracts.css` subpath is defined in
    // `packages/sui/tokens/package.json`).
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
    // the SuiPlugin's `resolve_id` re-anchors `@ship/...` imports
    // relative to the workspace `repo_root` (the framework's
    // `@ship/tokens` map points at the workspace source). The entry
    // file uses a unique tag suffix so concurrent test runs don't
    // collide. A guard removes it on drop.
    let entry_path = repo_root.join("app/routes/fonts-smoke.sui");
    write(
        &entry_path,
        r#"<controller>
import '@ship/tokens/contracts.css'
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
    let sui_plugin = SuiPlugin {
        repo_root: repo_root.clone(),
        emit_component_imports: false,
        component_index: component_idx.index,
        shadowed_tags: component_idx.shadowed,
        component_roots: component_package_roots(&repo_root),
    };
    let mut bundler = BundlerBuilder::default()
        .with_options(options)
        .with_plugins(vec![Arc::new(sui_plugin), Arc::new(ShipAssetCssPlugin)])
        .build()
        .expect("bundler build");
    bundler
        .write()
        .await
        .expect("bundle + plugin write");

    // The output assets dir must contain BOTH the rewritten
    // contracts.css AND the three woff2 (as hashed assets). We list
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
        3,
        "expected exactly 3 hashed woff2 assets (DM Sans, Syne, JetBrains Mono), got {woff2_assets:?}"
    );
    let contracts_css = names
        .iter()
        .find(|n| n.starts_with("contracts-") && n.ends_with(".css"))
        .expect("contracts.css asset must be emitted");
    // The contracts.css body must reference the woff2 by their hashed
    // asset paths (no `./fonts/X.woff2` source paths; no base64 data
    // URLs). The placeholder prefix must be fully resolved.
    let contracts_path = assets_dir.join(contracts_css);
    let contracts_body = std::fs::read_to_string(&contracts_path)
        .expect("contracts.css must be readable");
    assert!(
        !contracts_body.contains("data:font/woff2"),
        "base64 inlining is GONE — no `data:font/woff2` URLs allowed, got body:\n{}",
        contracts_body
    );
    assert!(
        !contracts_body.contains("__SHIP_CSS_ASSET__#"),
        "the placeholder must be resolved by `generate_bundle`, got body:\n{}",
        contracts_body
    );
    assert!(
        !contracts_body.contains("./fonts/dm-sans-")
            && !contracts_body.contains("./fonts/syne-")
            && !contracts_body.contains("./fonts/jetbrains-"),
        "the source `url('./fonts/X.woff2')` must be rewritten to the hashed asset URL, got body:\n{}",
        contracts_body
    );
    // The body must contain a URL pointing at one of the three woff2
    // assets we emitted. We don't pin the exact hash (content-derived)
    // — just assert at least one match.
    let references_at_least_one = woff2_assets.iter().any(|woff2_name| {
        // The CSS body has `url("./<name>.woff2")` — the basename is
        // `<name>.woff2` (e.g. `dm-sans-latin-wght-normal-<hash>.woff2`).
        let basename = woff2_name.strip_suffix(".woff2").unwrap_or(woff2_name);
        contracts_body.contains(basename)
    });
    assert!(
        references_at_least_one,
        "contracts.css must reference at least one of the emitted woff2 ({woff2_assets:?}), got body:\n{}",
        contracts_body
    );
    // The body must have three `@font-face` blocks (one per family in
    // the FONTS manifest) — confirms the drift lock carried through
    // the bundler without losing rows.
    let font_face_count = contracts_body.matches("@font-face").count();
    assert_eq!(
        font_face_count, 3,
        "expected 3 @font-face rules in contracts.css, got {font_face_count} (manifest drift?)"
    );
    // The body must use `format('woff2-variations')` for every face
    // (variable axis is the reason to vendor these woff2).
    let format_count = contracts_body
        .matches("format('woff2-variations')")
        .count();
    assert_eq!(
        format_count, 3,
        "expected 3 woff2-variations format() decls in contracts.css, got {format_count}"
    );
}
