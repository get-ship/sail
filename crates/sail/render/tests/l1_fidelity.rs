//! RENDER-1: L1 chrome paint + flow parity with the check layout pass.

use std::path::PathBuf;

use sail_render::{RenderOptions, assert_pixel_match, render_file, write_png};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/")).join(name)
}

fn fonts() -> PathBuf {
    fixture("fonts")
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

fn opts(viewport: (u32, u32), mode: &str) -> RenderOptions {
    RenderOptions {
        viewport,
        fonts_dir: Some(fonts()),
        app_fonts_dir: None,
        theme: Some("ship".into()),
        mode: Some(mode.into()),
        workspace: workspace_root(),
        file: None,
        drop_bg_token: None,
    }
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

/// Non-white pixels (any channel delta from pure white) — chrome paint signal.
fn non_bg_pixel_count(result: &sail_render::RenderOutput) -> usize {
    let rgba = result.image.as_raw();
    // Root uses theme background, not pure white — count pixels that aren't the
    // majority corner color (top-left).
    let bg = [rgba[0], rgba[1], rgba[2], rgba[3]];
    rgba.chunks_exact(4).filter(|c| c != &bg.as_slice()).count()
}

#[test]
fn probe_s003_settings_stacked_cards_with_chrome() {
    let result = render_file(
        &fixture("probe-s003-settings.sail"),
        opts((480, 720), "light"),
    )
    .expect("render s003");
    assert!(!result.html.contains('{'), "expr leak: {}", result.html);

    // Two cards must stack (different y), both full-ish width.
    let cards: Vec<_> = result
        .regions
        .iter()
        .filter(|r| r.tag.as_deref() == Some("card"))
        .collect();
    assert!(
        cards.len() >= 2,
        "expected ≥2 card regions, got {:?}",
        result
            .regions
            .iter()
            .map(|r| r.tag.clone())
            .collect::<Vec<_>>()
    );
    let c0 = cards[0];
    let c1 = cards[1];
    assert!(
        c1.y > c0.y + c0.h * 0.5,
        "cards must stack vertically: y0={} h0={} y1={}",
        c0.y,
        c0.h,
        c1.y
    );
    assert!(
        c0.w > 200.0 && c1.w > 200.0,
        "cards should be wide, got {} and {}",
        c0.w,
        c1.w
    );
    // Card host must grow with content — not collapsed by pseudo
    // accent-bar `h(half)` compounds mis-applied to the host.
    assert!(
        c0.h > 100.0 && c1.h > 80.0,
        "card shells must cover content (not ::before height): h0={} h1={}",
        c0.h,
        c1.h
    );

    // Visible chrome (card bg / borders / controls), not a blank page.
    assert!(
        non_bg_pixel_count(&result) > 500,
        "expected painted chrome pixels"
    );

    // Control chrome boxes must be non-zero (regression for skin size atoms).
    let switch = result
        .regions
        .iter()
        .find(|r| r.tag.as_deref() == Some("switch"))
        .expect("switch region");
    assert!(
        switch.w >= 20.0 && switch.h >= 12.0,
        "switch must have visible track geometry, got {}×{}",
        switch.w,
        switch.h
    );
    let input = result
        .regions
        .iter()
        .find(|r| r.tag.as_deref() == Some("input"))
        .expect("input region");
    assert!(
        input.h >= 28.0 && input.w > 100.0,
        "input must have control height, got {}×{}",
        input.w,
        input.h
    );
    let button = result
        .regions
        .iter()
        .find(|r| r.tag.as_deref() == Some("button"))
        .expect("button region");
    assert!(
        button.h >= 28.0,
        "button min-height near control-h-md, got h={}",
        button.h
    );

    ensure_or_compare("probe-s003-settings-480x720", &result);
}

#[test]
fn probe_s004_dashboard_sidebar_left_row() {
    let result = render_file(
        &fixture("probe-s004-dashboard.sail"),
        opts((800, 500), "light"),
    )
    .expect("render s004");

    // Navbar is top; content row places sidebar left of main.
    let navbars: Vec<_> = result
        .regions
        .iter()
        .filter(|r| r.tag.as_deref() == Some("navbar"))
        .collect();
    assert!(!navbars.is_empty(), "navbar region missing");

    // Content row under the navbar (taffy coords are parent-relative; the row
    // itself sits at y ≈ navbar height under the root column).
    let content_rows: Vec<_> = result
        .regions
        .iter()
        .filter(|r| r.tag.as_deref() == Some("row") && r.w > 400.0 && r.h > 100.0)
        .collect();
    assert!(
        !content_rows.is_empty(),
        "expected content row; regions={:?}",
        result
            .regions
            .iter()
            .map(|r| (r.tag.clone(), r.x, r.y, r.w, r.h))
            .collect::<Vec<_>>()
    );

    // Sidebar column: narrow, tall, left edge. Main column: wider, to its right.
    // Both are children of the content row so share parent-relative coords.
    let columns: Vec<_> = result
        .regions
        .iter()
        .filter(|r| r.tag.as_deref() == Some("column") && r.h > 100.0 && r.w > 40.0)
        .collect();
    let side = columns
        .iter()
        .find(|r| r.x < 20.0 && r.w < 280.0)
        .expect("left sidebar column");
    let main = columns
        .iter()
        .find(|r| r.x >= side.x + side.w - 2.0 && r.w > side.w)
        .expect("main column right of sidebar");
    assert!(
        main.x > side.x,
        "sidebar-left row: side.x={} main.x={}",
        side.x,
        main.x
    );

    assert!(non_bg_pixel_count(&result) > 500);
    ensure_or_compare("probe-s004-dashboard-800x500", &result);
}

#[test]
fn l1_components_golden_ship_light() {
    let result = render_file(&fixture("l1-components.sail"), opts((640, 900), "light"))
        .expect("render l1 light");
    assert!(!result.html.contains('{'));
    assert!(non_bg_pixel_count(&result) > 800, "light L1 chrome missing");
    ensure_or_compare("l1-components-ship-light-640x900", &result);
}

#[test]
fn l1_components_golden_ship_dark() {
    let result = render_file(&fixture("l1-components.sail"), opts((640, 900), "dark"))
        .expect("render l1 dark");
    assert!(non_bg_pixel_count(&result) > 800, "dark L1 chrome missing");
    ensure_or_compare("l1-components-ship-dark-640x900", &result);
}

/// Per-component smoke: each of the ten surfaces produces a painted region
/// with non-zero chrome where skins assign control size.
#[test]
fn l1_ten_components_emit_regions() {
    let result =
        render_file(&fixture("l1-components.sail"), opts((640, 900), "light")).expect("render");
    let tags: std::collections::BTreeSet<_> = result
        .regions
        .iter()
        .filter_map(|r| r.tag.clone())
        .collect();
    for need in [
        "button", "card", "field", "input", "switch", "badge", "navbar", "dialog", "table",
    ] {
        assert!(
            tags.iter().any(|t| t == need),
            "missing region tag {need}; have {tags:?}"
        );
    }
    // Sidebar is tokenized column (no L1 host tag) — row geometry is covered
    // by probe-s004; here assert sidebar paint tokens resolved (column exists).
    assert!(
        tags.contains("column") || tags.contains("row"),
        "layout hosts missing: {tags:?}"
    );

    let by = |tag: &str| {
        result
            .regions
            .iter()
            .find(|r| r.tag.as_deref() == Some(tag))
            .unwrap_or_else(|| panic!("region {tag}"))
    };
    let sw = by("switch");
    assert!(sw.w >= 20.0 && sw.h >= 12.0, "switch {}×{}", sw.w, sw.h);
    let inp = by("input");
    assert!(inp.h >= 28.0, "input h={}", inp.h);
    let btn = by("button");
    assert!(btn.h >= 28.0, "button h={}", btn.h);
    let badge = by("badge");
    assert!(
        badge.h >= 12.0 && badge.w > 0.0,
        "badge {}×{}",
        badge.w,
        badge.h
    );
}

/// Regions geometry equals the **check pass** layout boxes (same source + viewport).
///
/// `check_preprocessed` is the entry the compiler layout diagnostic uses.
/// Render joins its own taffy tree; both share `taffy_tree::build` + style
/// resolve, so boxes must agree within rounding.
#[test]
fn regions_match_layout_pass_taffy() {
    let source = std::fs::read_to_string(fixture("probe-s003-settings.sail")).unwrap();
    let viewport = (480u32, 720u32);
    let result = sail_render::render_source(&source, opts(viewport, "light")).expect("render");

    let preprocessed = sail_compiler::parse::preprocess::preprocess(&source).unwrap();
    let scan =
        sail_layout::check_preprocessed(&preprocessed, (viewport.0 as f32, viewport.1 as f32))
            .expect("layout check (check_preprocessed)");

    // check_preprocessed is the check-pass entry; it must produce the box tree
    // even when overflow diagnostics fire (card overflow:clip + chrome is a
    // known taffy/skin tension — not a region join failure).
    assert!(
        !scan.boxes.is_empty(),
        "check_preprocessed produced no boxes"
    );

    // Match regions to check-pass boxes by document order (both skip root).
    assert_eq!(
        result.regions.len(),
        scan.boxes.len(),
        "region count vs check_preprocessed boxes"
    );
    let tol = 0.6_f32;
    for (i, (r, b)) in result.regions.iter().zip(scan.boxes.iter()).enumerate() {
        assert!(
            (r.x - b.x).abs() <= tol
                && (r.y - b.y).abs() <= tol
                && (r.w - b.width).abs() <= tol
                && (r.h - b.height).abs() <= tol,
            "region/check box mismatch at {i}: region=({}, {}, {}, {}) check=({}, {}, {}, {}) tag={}",
            r.x,
            r.y,
            r.w,
            r.h,
            b.x,
            b.y,
            b.width,
            b.height,
            b.tag
        );
    }
}
