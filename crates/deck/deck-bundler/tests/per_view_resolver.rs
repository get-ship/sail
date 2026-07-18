//! End-to-end proof of the per-view resolver (#267 wave 0, capability gap #1):
//! with `DECK_PER_VIEW_RESOLVER` on, a view that uses `<sail-widget>` has the
//! component's code tree-shaken INTO its bundle (resolved from source via the
//! CEM index); with it off, the component is absent (the umbrella registers it
//! at runtime). Uses the LOCAL sail-compiler — which carries
//! `--emit-component-imports` — so it does not depend on the installed compiler
//! being current (#155). This is the invariant that makes islands+catalogs
//! subsumable: the bundle carries exactly what the view uses.

use std::path::Path;

use deck_bundler::rolldown_bundle::bundle_target;
use deck_bundler::targets::{BundleLayout, BundleTarget, TargetEntry};
use deck_config::DeckConfig;
fn write(path: &Path, body: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, body).unwrap();
}

/// A workspace with one component package (`sail-widget`, marker text
/// `sail-widget-marker`) and one view that uses the tag.
fn fixture(root: &Path) {
    write(&root.join("package.json"), r#"{"workspaces":["packages/*"]}"#);
    write(
        &root.join("packages/demo/package.json"),
        r#"{"name":"@ship/demo","customElements":"dist/custom-elements.json"}"#,
    );
    write(
        &root.join("packages/demo/dist/custom-elements.json"),
        r#"{"schemaVersion":"1","modules":[{"path":"sail-widget.sail","declarations":[{"kind":"class","name":"sail-widget","tagName":"sail-widget","customElement":true,"description":"demo"}]}]}"#,
    );
    // stem "sail-widget" contains a hyphen → detect_mode compiles this as a Component.
    write(
        &root.join("packages/demo/src/sail-widget/sail-widget.sail"),
        "<div>sail-widget-marker</div>\n",
    );
    write(&root.join("routes/usewidget/page.sail"), "<div><sail-widget></sail-widget></div>\n");
    // A second view that also uses the tag — for the shared-chunk dedup test.
    write(&root.join("routes/usewidget2/page.sail"), "<div><sail-widget></sail-widget></div>\n");
}

fn view_target(root: &Path) -> BundleTarget {
    BundleTarget {
        name: "routes/usewidget/route".into(),
        entry: TargetEntry::Real(root.join("routes/usewidget/page.sail")),
        externals: vec![],
        out_file: "routes/usewidget/route.js".into(),
        lazy: false,
    }
}

fn second_view_target(root: &Path) -> BundleTarget {
    BundleTarget {
        name: "routes/usewidget2/route".into(),
        entry: TargetEntry::Real(root.join("routes/usewidget2/page.sail")),
        externals: vec![],
        out_file: "routes/usewidget2/route.js".into(),
        lazy: false,
    }
}

/// All `.js` files under `dir`, recursively (view chunks nest under `.deck/`).
fn js_files(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else { return out };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            out.extend(js_files(&p));
        } else if p.extension().and_then(|x| x.to_str()) == Some("js") {
            out.push(p);
        }
    }
    out
}

async fn bundle(root: &Path, out_root: &Path) -> String {
    let layout =
        BundleLayout::with_config(root.to_path_buf(), None, DeckConfig::defaults()).with_out_root(out_root.to_path_buf());
    let staging = out_root.join("_staging");
    std::fs::create_dir_all(&staging).unwrap();
    bundle_target(&layout, &view_target(root), &staging, false)
        .await
        .expect("view bundles");
    std::fs::read_to_string(out_root.join(".deck/routes/usewidget/route.js"))
        .expect("route.js written")
}

#[tokio::test(flavor = "multi_thread")]
async fn view_bundle_carries_its_component() {
    // In-process compiler (#255), per-view resolver always on for `routes/`: a
    // view using <sail-widget> tree-shakes the component INTO its bundle from
    // source (via the CEM index). No binary to locate, no gate to flip — the
    // component is in the bundle *because* the resolver pulled it from source
    // (without it, the component lives only in the umbrella, loaded separately).
    let src = tempfile::tempdir().unwrap();
    let root = src.path();
    fixture(root);
    let out = tempfile::tempdir().unwrap();
    let js = bundle(root, out.path()).await;
    assert!(
        js.contains("sail-widget-marker"),
        "the view bundle must carry its component's code; got:\n{js}"
    );
}

/// Capability gap #2: a component used by N views must dedup into ONE shared
/// chunk in a multi-entry build, not inline N times. This is the dedup the
/// umbrella does today — required before the umbrella can be deleted.
#[tokio::test(flavor = "multi_thread")]
async fn shared_component_dedups_into_one_chunk_across_views() {
    let src = tempfile::tempdir().unwrap();
    let root = src.path();
    fixture(root);
    let out = tempfile::tempdir().unwrap();

    // Both views import <sail-widget>; the multi-entry build must hoist it.
    deck_bundler::routes_bundle::bundle_routes(
        &BundleLayout::with_config(root.to_path_buf(), None, DeckConfig::defaults()).with_out_root(out.path().to_path_buf()),
        &[view_target(root), second_view_target(root)],
        &out.path().join("_staging"),
        false,
    )
    .await
    .expect("multi-entry views bundle");

    let chunks_with_component = js_files(&out.path().join(".deck"))
        .into_iter()
        .filter(|p| {
            std::fs::read_to_string(p).map(|s| s.contains("sail-widget-marker")).unwrap_or(false)
        })
        .count();
    assert_eq!(
        chunks_with_component, 1,
        "a component shared by 2 views must dedup into ONE shared chunk, not inline per view"
    );
}
