//! Pixel goldens + paint/leak/dynamics assertions for static render.

use std::path::PathBuf;

use sail_render::{RenderOptions, assert_pixel_match, png_bytes, render_file, write_png};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/")).join(name)
}

fn fonts() -> PathBuf {
    fixture("fonts")
}

fn opts(viewport: (u32, u32)) -> RenderOptions {
    RenderOptions {
        viewport,
        fonts_dir: Some(fonts()),
        app_fonts_dir: None,
        theme: Some("ship".into()),
        mode: Some("light".into()),
        workspace: workspace_root(),
        file: None,
        drop_bg_token: None,
    }
}

fn workspace_root() -> Option<PathBuf> {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for _ in 0..8 {
        if dir.join("Cargo.toml").is_file() && dir.join("packages/sail/tokens").is_dir() {
            return Some(dir);
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

fn golden_path(name: &str) -> PathBuf {
    fixture(&format!("goldens/{name}.png"))
}

fn ensure_or_compare(name: &str, result: &sail_render::RenderOutput) {
    let path = golden_path(name);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if !path.is_file() || std::env::var("SAIL_RENDER_UPDATE_GOLDEN").ok().as_deref() == Some("1") {
        write_png(&result.image, &path).expect("write golden");
        eprintln!("wrote golden {}", path.display());
        return;
    }
    let expected = std::fs::read(&path).expect("read golden");
    let diff = path.with_extension("diff.png");
    assert_pixel_match(&result.image, &expected, 3, Some(&diff)).unwrap_or_else(|e| {
        panic!("golden {name} failed: {e} (diff at {})", diff.display());
    });
}

#[test]
fn paint_fixture_golden_light() {
    let result = render_file(&fixture("paint.sail"), opts((400, 300))).unwrap();
    assert!(
        !result.html.contains("{ticks()}",),
        "expression leak in html: {}",
        result.html
    );
    assert!(
        result
            .dynamics
            .iter()
            .any(|d| d.reason.contains("dynamic-expr") || d.reason.contains("ticks")),
        "expected ticks dynamic: {:?}",
        result.dynamics
    );
    assert!(
        result
            .dynamics
            .iter()
            .any(|d| d.reason.contains("my-widget") || d.tag.as_deref() == Some("my-widget")),
        "expected unknown my-widget: {:?}",
        result.dynamics
    );
    assert!(!result.regions.is_empty());
    ensure_or_compare("paint-light-400x300", &result);
}

#[test]
fn paint_fixture_golden_dark() {
    let mut o = opts((400, 300));
    o.mode = Some("dark".into());
    let result = render_file(&fixture("paint.sail"), o).unwrap();
    ensure_or_compare("paint-dark-400x300", &result);
}

#[test]
fn static_only_golden() {
    let result = render_file(&fixture("static-only.sail"), opts((320, 200))).unwrap();
    assert!(
        result.dynamics.is_empty() || result.dynamics.iter().all(|d| !d.reason.contains("expr"))
    );
    ensure_or_compare("static-only-320x200", &result);
}

#[test]
fn drop_bg_mapping_fails_golden_with_diff() {
    // Establish baseline with full paint.
    let baseline = render_file(&fixture("paint.sail"), opts((400, 300))).unwrap();
    let bytes = png_bytes(&baseline.image).unwrap();

    // Drop primary mapping → button no longer paints primary.
    let mut o = opts((400, 300));
    o.drop_bg_token = Some("primary".into());
    let broken = render_file(&fixture("paint.sail"), o).unwrap();

    let dir = tempfile::tempdir().unwrap();
    let diff = dir.path().join("drop-bg.diff.png");
    let err = assert_pixel_match(&broken.image, &bytes, 3, Some(&diff));
    assert!(
        err.is_err(),
        "expected golden fail after dropping bg mapping"
    );
    assert!(diff.is_file(), "diff artifact must be written on failure");
}

#[test]
fn no_expression_leaks_in_output_html() {
    let result = render_file(&fixture("paint.sail"), opts((400, 300))).unwrap();
    assert!(
        !result.html.contains('{'),
        "leaked brace in html: {}",
        result.html
    );
}

#[test]
fn mid_size_route_under_500ms_debug() {
    // Warm fonts/registry once — Studio repaints reuse loaded fonts (O8).
    // Spec budget is for repaint; cold first frame may exceed 500ms.
    let _ = render_file(&fixture("mid-size.sail"), opts((800, 600))).unwrap();
    let result = render_file(&fixture("mid-size.sail"), opts((800, 600))).unwrap();
    assert!(
        result.ms < 500,
        "warm render took {}ms (budget 500ms debug)",
        result.ms
    );
}

#[test]
fn golden_legacy_fixture_still_renders() {
    let result = render_file(&fixture("golden.sail"), opts((800, 600))).unwrap();
    assert_eq!(result.viewport, (800, 600));
    let rgba = result.image.as_raw();
    let non_white = rgba
        .chunks_exact(4)
        .filter(|c| c != &[255, 255, 255, 255])
        .count();
    assert!(non_white > 100, "expected visible content");
}

#[test]
fn sidecar_lists_dynamics() {
    let result = render_file(&fixture("paint.sail"), opts((400, 300))).unwrap();
    let dyn_json = serde_json::to_value(&result.dynamics).unwrap();
    assert!(dyn_json.as_array().unwrap().len() >= 2, "{dyn_json}");
}

#[test]
fn region_and_dynamic_ids_are_inspect_tree_ids() {
    // Library/CLI regions must use SailInspect content-hash ids (true join),
    // not a post-hoc layout-box substitution only on the ops path.
    let mut o = opts((400, 300));
    o.file = Some("crates/sail/render/tests/fixtures/paint.sail".into());
    let result = render_file(&fixture("paint.sail"), o).unwrap();
    let tree: std::collections::BTreeSet<_> = result.tree_ids.iter().cloned().collect();
    assert!(!tree.is_empty(), "author tree must produce node ids");
    assert!(!result.regions.is_empty());
    for r in &result.regions {
        assert!(
            tree.contains(&r.node_id),
            "region {} not in inspect tree {:?}",
            r.node_id,
            tree
        );
    }
    for d in &result.dynamics {
        assert!(
            tree.contains(&d.node_id),
            "dynamic {} not in inspect tree",
            d.node_id
        );
        assert!(
            !d.node_id.contains("dyn.expr"),
            "dynamics must not keep provisional ids"
        );
    }
}

#[test]
fn mid_size_golden_painted_tokens() {
    // Step-6 fixture coverage: multi-card painted route (page.sail 1440×900 is
    // intentionally not golden — cold full landing is heavy; this pins tokens).
    let result = render_file(&fixture("mid-size.sail"), opts((800, 600))).unwrap();
    assert!(!result.html.contains('{'));
    ensure_or_compare("mid-size-800x600", &result);
}

/// RENDER-0: agent-produced dependent-form page that crashed the baseline batch.
#[test]
fn parley_dependent_form_agent_repro_does_not_panic() {
    let result = render_file(
        &fixture("parley-dependent-form-repro.sail"),
        opts((1440, 900)),
    )
    .expect("dependent-form agent page must render without parley panic");
    assert!(
        result.image.width() > 0 && result.image.height() > 0,
        "viewport must be non-zero"
    );
    let png = png_bytes(&result.image).expect("encode png");
    assert!(png.starts_with(b"\x89PNG"), "must produce a PNG");
    assert!(
        !result.regions.is_empty(),
        "agent form must produce layout regions"
    );
    let html = result.html.to_lowercase();
    assert!(
        html.contains("shipping") && (html.contains("country") || html.contains("select")),
        "expected form chrome in static html, got a prefix: {:?}",
        html.chars().take(200).collect::<String>()
    );
}
