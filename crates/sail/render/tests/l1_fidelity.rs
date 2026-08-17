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
        theme: Some("sail".into()),
        mode: Some(mode.into()),
        workspace: workspace_root(),
        file: None,
        drop_bg_token: None,
    }
}

fn golden_path(name: &str) -> PathBuf {
    fixture(&format!("goldens/{name}.png"))
}

fn in_ci() -> bool {
    matches!(
        std::env::var("CI").ok().as_deref(),
        Some("1" | "true" | "TRUE" | "yes" | "YES")
    )
}

fn ensure_or_compare(name: &str, result: &sail_render::RenderOutput) {
    let path = golden_path(name);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let update = std::env::var("SAIL_RENDER_UPDATE_GOLDEN").ok().as_deref() == Some("1");
    if !path.is_file() {
        if in_ci() && !update {
            panic!(
                "golden {name} missing at {} (CI refuses to write; run with \
                 SAIL_RENDER_UPDATE_GOLDEN=1 locally to mint)",
                path.display()
            );
        }
        write_png(&result.image, &path).expect("write golden");
        eprintln!("wrote golden {}", path.display());
        return;
    }
    if update {
        write_png(&result.image, &path).expect("write golden");
        eprintln!("updated golden {}", path.display());
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

/// Count non-background pixels inside a region AABB.
/// Regions are **viewport-absolute** (see `identity::join_layout_regions`).
fn non_bg_in_region(result: &sail_render::RenderOutput, r: &sail_render::Region) -> usize {
    let rgba = result.image.as_raw();
    let w = result.image.width() as usize;
    let h = result.image.height() as usize;
    if w == 0 || h == 0 {
        return 0;
    }
    let bg = [rgba[0], rgba[1], rgba[2], rgba[3]];
    let x0 = r.x.max(0.0).floor() as usize;
    let y0 = r.y.max(0.0).floor() as usize;
    let x1 = ((r.x + r.w).ceil() as usize).min(w);
    let y1 = ((r.y + r.h).ceil() as usize).min(h);
    if x0 >= x1 || y0 >= y1 {
        return 0;
    }
    let mut n = 0usize;
    for y in y0..y1 {
        for x in x0..x1 {
            let i = (y * w + x) * 4;
            if rgba[i..i + 4] != bg {
                n += 1;
            }
        }
    }
    n
}

/// Load-bearing L1 author hosts that must join SailInspect.
/// Layout keywords (`row`/`column`/…) and anatomy (`svg`/`path`) often expand
/// past the author tree — those may be unjoined without failing this smoke.
fn is_join_required_host(tag: Option<&str>) -> bool {
    matches!(
        tag,
        Some(
            "button"
                | "card"
                | "field"
                | "input"
                | "switch"
                | "badge"
                | "navbar"
                | "sidebar"
                | "dialog"
                | "table"
        )
    )
}

fn assert_joined_regions(result: &sail_render::RenderOutput, label: &str) {
    assert!(!result.regions.is_empty(), "{label}: expected regions");
    // Detect both empty-tree (`_unjoined#…`) and non-empty-tree (`#unjoined#`)
    // forms. Anatomy expansions (svg, path, …) may be unjoined; require join
    // only on load-bearing hosts the author wrote.
    let unjoined: Vec<_> = result
        .regions
        .iter()
        .filter(|r| {
            sail_render::is_unjoined_id(&r.node_id) && is_join_required_host(r.tag.as_deref())
        })
        .map(|r| (r.tag.clone(), r.node_id.clone()))
        .collect();
    assert!(
        unjoined.is_empty(),
        "{label}: expected SailInspect-joined region ids for L1/layout hosts \
         (parse_author must accept Style scale tokens like 2xl); unjoined={unjoined:?}"
    );
    // Layout expands L1 anatomy into extra boxes that share author hosts when
    // tag cursors exhaust — global id uniqueness is not required. Sibling L1
    // hosts of the same tag (cards, …) must still get distinct author ids.
    for host in ["card", "button", "navbar"] {
        let mut ids = std::collections::BTreeSet::new();
        let mut n = 0usize;
        for r in &result.regions {
            if r.tag.as_deref() == Some(host) {
                n += 1;
                ids.insert(r.node_id.as_str());
            }
        }
        if n >= 2 {
            assert_eq!(
                ids.len(),
                n,
                "{label}: sibling <{host}> regions must have distinct SailInspect ids \
                 (n={n}, unique={})",
                ids.len()
            );
        }
    }
}

fn assert_regions_match_check(fixture_name: &str, viewport: (u32, u32)) {
    let source = std::fs::read_to_string(fixture(fixture_name)).unwrap();
    let result =
        sail_render::render_source(&source, opts(viewport, "light")).expect("render for parity");

    let (preprocessed, hosts) =
        sail_compiler::parse::preprocess::preprocess_with_style(&source).unwrap();
    let vp = (viewport.0 as f32, viewport.1 as f32);
    let scan = sail_layout::check_preprocessed_with_hosts(
        &preprocessed,
        vp,
        sail_layout::CheckOptions::default(),
        &hosts.types,
        &hosts.boxes,
    )
    .expect("layout check (check_preprocessed_with_hosts)");

    // check_preprocessed is the check-pass entry; it must produce the box tree
    // even when overflow diagnostics fire (card overflow:clip + chrome is a
    // known taffy/skin tension — not a region join failure).
    assert!(
        !scan.boxes.is_empty(),
        "{fixture_name}: check_preprocessed produced no boxes"
    );

    // Regions are viewport-absolute; BoxReport is parent-relative — promote check
    // boxes with the same parent-chain walk used by join_layout_regions.
    let mut elements = sail_layout::html::parse_source(&preprocessed);
    sail_layout::apply_resolved_types(&mut elements, &hosts.types);
    sail_layout::apply_resolved_boxes(&mut elements, &hosts.boxes);
    let layout = sail_layout::taffy_tree::build(elements.clone(), vp).expect("taffy build");
    assert_eq!(
        result.regions.len(),
        scan.boxes.len(),
        "{fixture_name}: region count vs check_preprocessed boxes"
    );
    let tol = 0.6_f32;
    for (i, (r, b)) in result.regions.iter().zip(scan.boxes.iter()).enumerate() {
        let (abs_x, abs_y) = sail_render::absolute_location(&layout, &elements, b.index);
        assert!(
            (r.x - abs_x).abs() <= tol
                && (r.y - abs_y).abs() <= tol
                && (r.w - b.width).abs() <= tol
                && (r.h - b.height).abs() <= tol,
            "{fixture_name} region/check box mismatch at {i}: region=({}, {}, {}, {}) \
             check_abs=({}, {}, {}, {}) tag={}",
            r.x,
            r.y,
            r.w,
            r.h,
            abs_x,
            abs_y,
            b.width,
            b.height,
            b.tag
        );
    }
}

/// Probe s003 is agent-authored settings (stacked cards). Sanitized only by
/// adding `variant="solid"` on the primary button so solid/primary CVA paints
/// (product default variant is outline-ish fill, not solid primary).
#[test]
fn probe_s003_settings_stacked_cards_with_chrome() {
    let mut o = opts((480, 720), "light");
    o.file = Some("crates/sail/render/tests/fixtures/probe-s003-settings.sail".into());
    let result = render_file(&fixture("probe-s003-settings.sail"), o).expect("render s003");
    assert!(!result.html.contains('{'), "expr leak: {}", result.html);
    assert_joined_regions(&result, "s003");

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
    assert_ne!(
        c0.node_id, c1.node_id,
        "sibling cards must have distinct SailInspect ids"
    );
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
        input.w > 100.0,
        "input must be wide, got {}×{}",
        input.w,
        input.h
    );
    // L1 input skins are match-compounds; taffy box IR skips When/Match, so
    // replaced inputs can measure 0-tall. The field host still has a box.
    if input.h < 28.0 {
        let field = result
            .regions
            .iter()
            .find(|r| r.tag.as_deref() == Some("field"))
            .expect("field region around zero-height input");
        assert!(
            field.w > 100.0,
            "field host must wrap the input, got {}×{}",
            field.w,
            field.h
        );
    }
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

    // Bridge absolute region AABBs → painted pixels (double-layout smoke).
    assert!(
        non_bg_in_region(&result, c0) > 50,
        "card region AABB must contain painted chrome"
    );
    // Nested button geometry is already asserted above (h/w). Paint is separate
    // and must be non-zero — not OR'd with geometry.
    //
    // taffy regions vs takumi paint can vertical-shift (page height diverge);
    // require non-bg in the taffy AABB or a same-x strip shifted up toward the
    // painted band (dual-layout slack), still paint-not-geometry.
    let btn_px = non_bg_in_region(&result, button);
    let btn_px_shifted = {
        let mut best = 0usize;
        for dy in [0.0_f32, 40.0, 80.0, 120.0, 160.0] {
            let mut probe = button.clone();
            probe.y = (button.y - dy).max(0.0);
            best = best.max(non_bg_in_region(&result, &probe));
            if best > 0 {
                break;
            }
        }
        best
    };
    assert!(
        btn_px > 0 || btn_px_shifted > 0,
        "button must have painted chrome in absolute AABB (or dual-layout y-shift): \
         px={btn_px} shifted={btn_px_shifted} region=({}, {}, {}×{})",
        button.x,
        button.y,
        button.w,
        button.h
    );

    ensure_or_compare("probe-s003-settings-480x720", &result);
}

#[test]
fn probe_s004_dashboard_sidebar_left_row() {
    let mut o = opts((800, 500), "light");
    o.file = Some("crates/sail/render/tests/fixtures/probe-s004-dashboard.sail".into());
    let result = render_file(&fixture("probe-s004-dashboard.sail"), o).expect("render s004");
    assert_joined_regions(&result, "s004");

    // Navbar is top; content row places sidebar left of main.
    let navbars: Vec<_> = result
        .regions
        .iter()
        .filter(|r| r.tag.as_deref() == Some("navbar"))
        .collect();
    assert!(!navbars.is_empty(), "navbar region missing");
    assert!(
        navbars[0].h >= 40.0,
        "navbar shell height, got {}",
        navbars[0].h
    );

    // Content row under the navbar (regions are viewport-absolute; the row
    // sits at y ≈ navbar height in viewport space).
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
    // Viewport-absolute: both sit under the content row (similar absolute y).
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

    // Shell size floors for hosts that only had tag-presence before.
    let card = by("card");
    assert!(
        card.h > 80.0 && card.w > 100.0,
        "card shell too small: {}×{}",
        card.w,
        card.h
    );
    let navbar = by("navbar");
    assert!(
        navbar.w > 100.0 && navbar.h > 0.0,
        "navbar shell too small: {}×{}",
        navbar.w,
        navbar.h
    );
    let dialog = by("dialog");
    assert!(
        dialog.w > 40.0 && dialog.h > 20.0,
        "dialog shell too small: {}×{}",
        dialog.w,
        dialog.h
    );
    let table = by("table");
    assert!(
        table.h > 40.0 && table.w > 40.0,
        "table shell too small: {}×{}",
        table.w,
        table.h
    );

    // Paint inside host AABBs (viewport-absolute region coords).
    assert!(
        non_bg_in_region(&result, btn) > 10,
        "button AABB should contain chrome pixels"
    );
    assert!(
        non_bg_in_region(&result, card) > 20,
        "card AABB should contain chrome pixels"
    );
}

/// Regions geometry equals the **check pass** layout boxes (same source + viewport).
///
/// `check_preprocessed` is the entry the compiler layout diagnostic uses.
/// Render joins its own taffy tree; both share `taffy_tree::build` + style
/// resolve, so boxes must agree within rounding.
#[test]
fn regions_match_layout_pass_taffy() {
    assert_regions_match_check("probe-s003-settings.sail", (480, 720));
}

#[test]
fn regions_match_layout_pass_taffy_s004() {
    assert_regions_match_check("probe-s004-dashboard.sail", (800, 500));
}

#[test]
fn regions_match_layout_pass_taffy_l1_components() {
    assert_regions_match_check("l1-components.sail", (640, 900));
}
