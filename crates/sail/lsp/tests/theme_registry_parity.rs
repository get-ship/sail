//! THEMES-2 §3: LSP intelligence + compiler registry are the same API.

use sail_compiler::{
    ThemeTokenKind, completions_at_offset, hover_at, resolve_theme_token, theme_complete_at,
};
use sail_lsp::{complete_sail, hover_sail};

#[test]
fn lsp_and_compiler_complete_theme_eq_same_ids() {
    let src = r#"<column theme="s">x</column>"#;
    let off = src.find("s\"").expect("caret") + 1;
    let via_api: Vec<String> = theme_complete_at(src, off, None)
        .into_iter()
        .map(|i| i.name)
        .collect();
    let via_analysis: Vec<String> = completions_at_offset(src, None, off, None)
        .into_iter()
        .filter(|s| s.source == "theme")
        .map(|s| s.name)
        .collect();
    assert_eq!(via_api, via_analysis, "analysis must use theme_complete_at");

    let line = 1u32;
    let col = (off + 1) as u32;
    let via_lsp: Vec<String> = complete_sail(src, Some("apps/demo/src/page.sail"), line, col)
        .into_iter()
        .map(|i| i.label)
        .collect();
    assert_eq!(
        via_api, via_lsp,
        "LSP intelligence must serve the same theme= registry as compiler"
    );
    assert!(via_api.iter().any(|id| id == "ship"), "{via_api:?}");
    assert!(via_api.iter().all(|id| {
        theme_complete_at(src, off, None)
            .iter()
            .any(|t| t.name == *id && t.kind == ThemeTokenKind::ThemeId)
    }));
}

#[test]
fn lsp_and_compiler_hover_primary_same_resolved_value() {
    let src = r#"<div theme="ship" style({ paint: bg(primary) })>x</div>"#;
    let off = src.find("primary").expect("primary");
    let (line, col) = {
        let mut l = 1usize;
        let mut c = 1usize;
        for (i, ch) in src.char_indices() {
            if i >= off {
                break;
            }
            if ch == '\n' {
                l += 1;
                c = 1;
            } else {
                c += 1;
            }
        }
        (l, c)
    };
    let via_resolve = resolve_theme_token("primary", "ship", "light").expect("resolve");
    let via_hover = hover_at(src, None, line, col).expect("hover");
    assert_eq!(via_hover.kind, "theme-token");
    assert!(
        via_hover.source.contains(&via_resolve.value),
        "hover source must include resolved value {}\n{}",
        via_resolve.value,
        via_hover.source
    );
    let via_lsp = hover_sail(
        src,
        Some("apps/demo/src/page.sail"),
        line as u32,
        col as u32,
    )
    .expect("lsp hover");
    assert!(
        via_lsp.contents.contains(&via_resolve.value),
        "MCP/LSP hover must match registry value {}\n{}",
        via_resolve.value,
        via_lsp.contents
    );
    if let Some(div) = &via_resolve.mode_divergence {
        assert!(
            via_lsp.contents.contains("Mode divergence")
                && via_lsp.contents.contains(&div.other_value),
            "hover must flag mode divergence: {}",
            via_lsp.contents
        );
    }
}
