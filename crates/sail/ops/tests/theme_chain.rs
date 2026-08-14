//! THEMES-2 §5: studio edit → registry → generated CSS → sail_render screenshot.

use std::path::PathBuf;

use sail_ops::{Ctx, theme};
use sail_render::{RenderOptions, render_source};
use serde_json::json;

fn workspace_root() -> PathBuf {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for _ in 0..8 {
        if dir.join("Cargo.toml").is_file() && dir.join("packages/sail/tokens").is_dir() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    panic!(
        "workspace root not found from {}",
        env!("CARGO_MANIFEST_DIR")
    );
}

fn fonts_dir() -> Option<PathBuf> {
    let render_fonts =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../render/tests/fixtures/fonts");
    if render_fonts.is_dir() {
        return Some(render_fonts);
    }
    let pkg = workspace_root().join("packages/sail/tokens/src/fonts");
    if pkg.is_dir() { Some(pkg) } else { None }
}

/// Distinctive primary — far from catalog ship light primary.
const PROBE_PRIMARY: &str = "oklch(0.42 0.20 25)";

fn probe_theme() -> String {
    format!(
        r#"{{
  "$extensions": {{ "sail": {{ "themeId": "themes-chain-probe", "defaultMode": "light", "layer": "theme" }} }},
  "modes": {{
    "light": {{
      "color": {{
        "background": {{ "$value": "oklch(0.99 0 0)", "$type": "color" }},
        "foreground": {{ "$value": "oklch(0.18 0 0)", "$type": "color" }},
        "primary": {{
          "default": {{ "$value": "{PROBE_PRIMARY}", "$type": "color" }},
          "foreground": {{ "$value": "oklch(0.99 0 0)", "$type": "color" }}
        }},
        "muted": {{
          "default": {{ "$value": "oklch(0.92 0 0)", "$type": "color" }},
          "foreground": {{ "$value": "oklch(0.35 0 0)", "$type": "color" }}
        }},
        "brandAccent": {{
          "default": {{ "$value": "oklch(0.45 0.16 300)", "$type": "color" }},
          "foreground": {{ "$value": "oklch(0.98 0 0)", "$type": "color" }}
        }}
      }}
    }}
  }}
}}"#
    )
}

#[test]
fn studio_edit_to_screenshot_paints_registry() {
    let tmp = tempfile::tempdir().unwrap();
    let ws = tmp.path();
    std::fs::create_dir_all(ws.join("themes")).unwrap();
    std::fs::create_dir_all(ws.join("routes")).unwrap();
    let ctx = Ctx::new(ws, ws, ws);

    // 1. Studio edit = SailTheme write (gate-enforced).
    let written = theme::write(
        &json!({
            "path": "themes/themes-chain-probe.json",
            "content": probe_theme()
        }),
        &ctx,
    )
    .expect("write valid theme");
    assert_eq!(written["ok"], true);

    // 2. Registry = SailTheme read.
    let read =
        theme::read(&json!({ "path": "themes/themes-chain-probe.json" }), &ctx).expect("read");
    let content = read["content"].as_str().expect("content");
    assert!(
        content.contains(PROBE_PRIMARY),
        "registry must retain studio value: {content}"
    );

    // 3. Generated CSS.
    let generated = theme::generate(
        &json!({
            "mode": "app",
            "project": ws.display().to_string(),
            "workspace": ws.display().to_string()
        }),
        &ctx,
    )
    .expect("generate");
    assert_eq!(generated["ok"], true);
    let css_path = generated["themes_css"].as_str().expect("themes_css");
    let css = std::fs::read_to_string(css_path).expect("read generated css");
    assert!(
        css.contains(PROBE_PRIMARY),
        "generated CSS must emit registry primary {PROBE_PRIMARY}:\n{css}"
    );
    assert!(
        css.contains("theme='themes-chain-probe'") || css.contains("theme=\"themes-chain-probe\""),
        "generated CSS must target the written theme id:\n{css}"
    );

    // 4. Static screenshot (sail_render) — painted primary matches registry.
    let src = r#"<div style({ size({ w: full, h: full }).bg(primary) })>probe</div>"#;
    let Some(fonts) = fonts_dir() else {
        eprintln!("skip screenshot sample — token fonts not found");
        return;
    };
    let out = render_source(
        src,
        RenderOptions {
            viewport: (64, 64),
            fonts_dir: Some(fonts),
            app_fonts_dir: None,
            theme: Some("themes-chain-probe".into()),
            mode: Some("light".into()),
            workspace: Some(ws.to_path_buf()),
            file: Some("probe.sail".into()),
            drop_bg_token: None,
        },
    )
    .expect("render");
    let expected = sail_render::paint::parse_color(PROBE_PRIMARY).expect("parse probe primary");
    let w = out.image.width();
    let h = out.image.height();
    assert!(w > 0 && h > 0, "empty screenshot");
    let data = out.image.as_raw();
    let px = ((h / 2) * w + (w / 2)) as usize;
    let got = [
        data[px * 4],
        data[px * 4 + 1],
        data[px * 4 + 2],
        data[px * 4 + 3],
    ];
    let want = expected.0;
    let drift = |a: u8, b: u8| a.abs_diff(b);
    assert!(
        drift(got[0], want[0]) <= 12
            && drift(got[1], want[1]) <= 12
            && drift(got[2], want[2]) <= 12,
        "screenshot pixel {got:?} must match registry {want:?} ({PROBE_PRIMARY})"
    );
}
