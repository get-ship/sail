//! SSR-RENDER: controller initial state must appear in static render HTML / paint.
//!
//! Regression for the empty-page class (brand-transplant / mini-app evidence):
//! signal-driven copy and list rows must not vanish when takumi paints without JS.

use std::path::PathBuf;

use sail_render::{RenderOptions, render_file, render_source};

fn fonts() -> PathBuf {
    PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/fonts"
    ))
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

fn opts(viewport: (u32, u32)) -> RenderOptions {
    RenderOptions {
        viewport,
        fonts_dir: Some(fonts()),
        app_fonts_dir: None,
        theme: Some("ship".into()),
        mode: Some("light".into()),
        workspace: workspace_root(),
        file: Some("controller-initial.sail".into()),
        drop_bg_token: None,
    }
}

#[test]
fn controller_initial_fixture_html_shows_projected_content() {
    let path = PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/controller-initial.sail"
    ));
    let out = render_file(&path, opts((480, 360))).expect("render controller-initial");

    assert!(
        out.html.contains("Launch checklist"),
        "title from signal() must project into HTML; got:\n{}",
        out.html
    );
    assert!(
        out.html.contains("Brand tokens"),
        "for-row label must project; got:\n{}",
        out.html
    );
    assert!(
        out.html.contains("Mini-app routes"),
        "second for-row label must project; got:\n{}",
        out.html
    );
    assert!(
        !out.html.contains("Nothing on the board."),
        "else branch must not win when items().length > 0; got:\n{}",
        out.html
    );
    assert!(
        !out.html.contains("<controller"),
        "controller block must be stripped before paint; got:\n{}",
        out.html
    );
    // Projected reads should not remain as unresolved dynamics.
    for d in &out.dynamics {
        let r = d.reason.as_str();
        assert!(
            !r.contains("{title()}") && !r.contains("item.label"),
            "projected expr should not be listed as dynamic: {r}"
        );
    }
}

#[test]
fn controller_initial_paint_smoke_has_content_pixels() {
    let path = PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/controller-initial.sail"
    ));
    let out = render_file(&path, opts((480, 360))).expect("render");
    let rgba = out.image.as_raw();
    // Count pixels that are not near-white page chrome.
    let content = rgba
        .chunks_exact(4)
        .filter(|c| {
            let (r, g, b, a) = (c[0], c[1], c[2], c[3]);
            a > 200 && (r < 245 || g < 245 || b < 245)
        })
        .count();
    assert!(
        content > 200,
        "expected non-empty paint from projected controller text (content px={content})"
    );
}

#[test]
fn inline_controller_string_signal_projects() {
    let src = r#"
<controller>
import { controller, signal } from '@ship/sail-runtime'
export default controller(() => {
  const heading = signal("SSR-RENDER ok")
  return { heading }
})
</controller>
<div sx={style.pad(3).bg(surface-0)}>
  <h1 sx={style.fg(foreground)}>{heading()}</h1>
</div>
"#;
    let out = render_source(
        src,
        RenderOptions {
            viewport: (240, 120),
            fonts_dir: Some(fonts()),
            theme: Some("ship".into()),
            mode: Some("light".into()),
            workspace: workspace_root(),
            file: Some("inline.sail".into()),
            ..Default::default()
        },
    )
    .expect("render inline");
    assert!(
        out.html.contains("SSR-RENDER ok"),
        "got:\n{}",
        out.html
    );
    assert!(
        out.projected.iter().any(|p| p.contains("heading")),
        "projected sidecar notes should list the binding: {:?}",
        out.projected
    );
}

#[test]
fn unknown_if_does_not_paint_both_branches() {
    let src = r#"
<div sx={style.pad(2).bg(surface-0)}>
  <if {window.x}>
    <p>yes-branch</p>
    <else>
      <p>no-branch</p>
    </else>
  </if>
  <p sx={style.fg(foreground)}>anchor</p>
</div>
"#;
    let out = render_source(
        src,
        RenderOptions {
            viewport: (240, 120),
            fonts_dir: Some(fonts()),
            theme: Some("ship".into()),
            mode: Some("light".into()),
            workspace: workspace_root(),
            file: Some("unknown-if.sail".into()),
            ..Default::default()
        },
    )
    .expect("render");
    assert!(
        !out.html.contains("yes-branch") && !out.html.contains("no-branch"),
        "both if branches must be dropped; got:\n{}",
        out.html
    );
    assert!(out.html.contains("anchor"), "{}", out.html);
    assert!(
        out.dynamics
            .iter()
            .any(|d| d.reason.contains("window.x")),
        "dynamics must flag the unresolved if: {:?}",
        out.dynamics
    );
}
