//! Job A — mailbox death.
//!
//! Overlay position is CSS `anchor()` / `position-anchor` / `position-area`,
//! not bind(anchorLeft) custom properties. `fill` is the viewport, not
//! `--chrome-offset`. Acceptance is emit shape, not a grep of old channel
//! names as a feature.

use sail_compiler::compile_element;
use spinnaker::scales;

fn overlay_src(tag: &str) -> &'static str {
    sail_compiler::stdlib::sources()
        .iter()
        .find(|(t, _)| *t == tag)
        .map(|(_, s)| *s)
        .unwrap_or_else(|| panic!("{tag} registered"))
}

#[test]
fn overlay_skins_emit_css_anchor_positioning() {
    for tag in ["popover", "dropdown", "hover-card", "context-menu"] {
        let src = overlay_src(tag);
        let out = compile_element(src, tag).unwrap_or_else(|e| panic!("{tag}: {e:?}"));
        let skin = &out.skin_css;
        let structure = &out.structure_html;
        let css_anchor = skin.contains("position-anchor")
            || skin.contains("position-area")
            || skin.contains("anchor(")
            || structure.contains("position-anchor")
            || structure.contains("position-area");
        assert!(
            css_anchor,
            "{tag} overlay position is CSS anchor(), not a mailbox. skin={skin}\nstruct={structure}"
        );
        assert!(
            !skin.contains("var(--anchor-left)") && !skin.contains("var(--anchor-top)"),
            "{tag} skin must not read mailbox geometry: {skin}"
        );
    }
}

#[test]
fn tooltip_skin_does_not_read_tt_mailbox() {
    let src = overlay_src("tooltip");
    let out = compile_element(src, "tooltip").expect("tooltip compiles");
    assert!(
        !out.skin_css.contains("var(--tt-"),
        "tooltip geometry is Style / CSS anchor, not --tt-* mailbox: {}",
        out.skin_css
    );
}

#[test]
fn size_fill_is_viewport_not_chrome_offset() {
    let fill = scales::lower("size", "fill").expect("fill is a size token");
    assert_eq!(
        fill, "100dvh",
        "fill is the viewport; do not leave a lying chrome-offset mailbox, got {fill}"
    );
    let rail = scales::lower("size", "rail-fill").expect("rail-fill is a size token");
    assert!(
        !rail.contains("--chrome-offset") && !rail.contains("--rail-offset"),
        "rail-fill must not be a mailbox, got {rail}"
    );
}

#[test]
fn popover_still_compiles_as_popover_tag() {
    let out = compile_element(overlay_src("popover"), "popover").expect("popover compiles");
    assert_eq!(out.manifest["tag"], "popover");
    assert!(
        out.structure_html.contains("popover"),
        "native popover attr stays: {}",
        out.structure_html
    );
}
