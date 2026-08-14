//! SPIN-3: LSP intelligence + compiler complete/hover share one registry.

use sail_compiler::{completions_at_offset, hover_at, resolve_theme_token};
use sail_lsp::{complete_sail, hover_sail};

fn at_caret(src: &str) -> (String, usize, u32, u32) {
    let i = src.find('¦').expect("caret marker ¦");
    let mut s = String::with_capacity(src.len() - '¦'.len_utf8());
    s.push_str(&src[..i]);
    s.push_str(&src[i + '¦'.len_utf8()..]);
    let mut line = 1u32;
    let mut col = 1u32;
    for (idx, ch) in s.char_indices() {
        if idx >= i {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (s, i, line, col)
}

#[test]
fn lsp_and_compiler_complete_border_concern() {
    let (src, off, line, col) = at_caret("style({ border(¦) })");
    let via_style: Vec<String> = spinnaker::style_complete_at(&src, None, off, None)
        .into_iter()
        .map(|i| i.label)
        .collect();
    let via_analysis: Vec<String> = completions_at_offset(&src, None, off, None)
        .into_iter()
        .map(|s| s.name)
        .collect();
    assert_eq!(
        via_style, via_analysis,
        "analysis must use style_complete_at"
    );
    let via_lsp: Vec<String> = complete_sail(&src, Some("apps/demo/src/home.sail"), line, col)
        .into_iter()
        .map(|i| i.label)
        .collect();
    assert_eq!(
        via_style, via_lsp,
        "LSP must serve the same border(¦) registry as compiler"
    );
    for want in ["subtle", "card"] {
        assert!(
            via_lsp.iter().any(|l| l == want),
            "{want} missing: {via_lsp:?}"
        );
    }
}

#[test]
fn lsp_and_compiler_complete_border_color_slot() {
    let (src, off, line, col) = at_caret("style({ border({ color: ¦ }) })");
    let via_style: Vec<String> = spinnaker::style_complete_at(&src, None, off, None)
        .into_iter()
        .map(|i| i.label)
        .collect();
    let via_lsp: Vec<String> = complete_sail(&src, Some("apps/demo/src/home.sail"), line, col)
        .into_iter()
        .map(|i| i.label)
        .collect();
    assert_eq!(via_style, via_lsp, "color-slot complete must match");
    assert!(
        via_lsp.iter().any(|l| l == "primary" || l == "muted"),
        "color roles: {via_lsp:?}"
    );
    for width in ["hairline", "control-stroke"] {
        assert!(
            !via_lsp.iter().any(|l| l == width),
            "width token `{width}` in color slot: {via_lsp:?}"
        );
    }
}

#[test]
fn lsp_and_compiler_hover_border_subtle() {
    let (src, _off, line, col) = at_caret("style({ border(border.su¦btle) })");
    let via_resolve = resolve_theme_token("border.subtle", "ship", "light").expect("resolve");
    let via_hover = hover_at(&src, None, line as usize, col as usize).expect("hover");
    assert_eq!(via_hover.kind, "theme-token");
    assert!(
        via_hover.source.contains(&via_resolve.value),
        "hover source must include resolved members {}\n{}",
        via_resolve.value,
        via_hover.source
    );
    let via_lsp = hover_sail(&src, Some("apps/demo/src/home.sail"), line, col).expect("lsp hover");
    assert!(
        via_lsp.contents.contains(&via_resolve.value),
        "MCP/LSP hover must match registry members {}\n{}",
        via_resolve.value,
        via_lsp.contents
    );
    assert!(
        via_lsp.contents.contains("color:") && via_lsp.contents.contains("style:"),
        "hover members: {}",
        via_lsp.contents
    );
}

fn labels_at(src_caret: &str) -> (Vec<String>, Vec<String>, Vec<String>) {
    let (src, off, line, col) = at_caret(src_caret);
    let via_style: Vec<String> = spinnaker::style_complete_at(&src, None, off, None)
        .into_iter()
        .map(|i| i.label)
        .collect();
    let via_analysis: Vec<String> = completions_at_offset(&src, None, off, None)
        .into_iter()
        .map(|s| s.name)
        .collect();
    let via_lsp: Vec<String> = complete_sail(&src, Some("apps/demo/src/home.sail"), line, col)
        .into_iter()
        .map(|i| i.label)
        .collect();
    assert_eq!(
        via_style, via_analysis,
        "analysis must use style_complete_at"
    );
    assert_eq!(
        via_style, via_lsp,
        "LSP must serve the same registry as compiler"
    );
    (via_style, via_analysis, via_lsp)
}

#[test]
fn lsp_and_compiler_complete_paint_channel_concerns() {
    let (_, _, via_lsp) = labels_at("style({ bg(primary).¦ })");
    for want in ["bg", "fg", "border", "radius", "elevation"] {
        assert!(
            via_lsp.iter().any(|l| l == want),
            "{want} missing: {via_lsp:?}"
        );
    }
    for dead in ["fill", "ink", "shadow"] {
        assert!(
            !via_lsp.iter().any(|l| l == dead),
            "dead `{dead}` offered: {via_lsp:?}"
        );
    }
}

#[test]
fn lsp_and_compiler_complete_box_and_text_channels() {
    let (_, _, box_labs) = labels_at("style({ bg(primary).¦ })");
    for want in ["size", "pad", "margin", "position", "overflow"] {
        assert!(
            box_labs.iter().any(|l| l == want),
            "{want} missing: {box_labs:?}"
        );
    }
    for dead in ["space", "place", "flow", "padX", "w", "minW"] {
        assert!(
            !box_labs.iter().any(|l| l == dead),
            "dead `{dead}` offered: {box_labs:?}"
        );
    }

    let (_, _, text_labs) = labels_at("style({ bg(primary).¦ })");
    assert!(
        text_labs.iter().any(|l| l == "type"),
        "type missing: {text_labs:?}"
    );
    for absorbed in ["fontSize", "truncate", "fontWeight", "fontFamily"] {
        assert!(
            !text_labs.iter().any(|l| l == absorbed),
            "absorbed `{absorbed}` offered: {text_labs:?}"
        );
    }
}

#[test]
fn lsp_and_compiler_complete_size_type_elevation_fill_names() {
    let (_, _, size) = labels_at("style({ size(¦) })");
    for want in ["control-sm", "control-md", "control-lg", "full"] {
        assert!(size.iter().any(|l| l == want), "{want} missing: {size:?}");
    }

    let (_, _, typ) = labels_at("style({ type(¦) })");
    for want in ["body", "body-sm", "label", "heading", "mono"] {
        assert!(typ.iter().any(|l| l == want), "{want} missing: {typ:?}");
    }

    let (_, _, elev) = labels_at("style({ elevation(¦) })");
    for want in ["xs", "sm", "md", "lg", "xl", "none"] {
        assert!(elev.iter().any(|l| l == want), "{want} missing: {elev:?}");
    }

    let (_, _, fill) = labels_at("style({ bg(¦) })");
    for want in ["surface", "card", "muted"] {
        assert!(fill.iter().any(|l| l == want), "{want} missing: {fill:?}");
    }
}

#[test]
fn lsp_and_compiler_complete_border_slots_include_kind_not_radius() {
    let (_, _, labs) = labels_at("style({ border({ ¦ }) })");
    for want in ["color", "width", "style", "edge", "kind"] {
        assert!(labs.iter().any(|l| l == want), "{want} missing: {labs:?}");
    }
    assert!(
        !labs.iter().any(|l| l == "radius"),
        "radius is not a border slot: {labs:?}"
    );
}

#[test]
fn lsp_and_compiler_hover_size_type_elevation_fill() {
    for (src, resolve_name, member) in [
        ("style({ size(size.control-¦md) })", "size.control-md", "h:"),
        ("style({ type(type.head¦ing) })", "type.heading", "family:"),
        (
            "style({ elevation(elevation.s¦m) })",
            "elevation.sm",
            "shadow:",
        ),
        ("style({ bg(surface-¦1) })", "surface-1", "oklch"),
    ] {
        let (src, _off, line, col) = at_caret(src);
        let via_resolve = resolve_theme_token(resolve_name, "ship", "light")
            .unwrap_or_else(|| panic!("resolve {resolve_name}"));
        let via_hover = hover_at(&src, None, line as usize, col as usize)
            .unwrap_or_else(|| panic!("hover {resolve_name}"));
        assert_eq!(via_hover.kind, "theme-token", "{resolve_name}");
        assert!(
            via_hover.source.contains(&via_resolve.value),
            "hover source must include resolved members {}\n{}",
            via_resolve.value,
            via_hover.source
        );
        let via_lsp = hover_sail(&src, Some("apps/demo/src/home.sail"), line, col)
            .unwrap_or_else(|| panic!("lsp hover {resolve_name}"));
        assert!(
            via_lsp.contents.contains(&via_resolve.value),
            "MCP/LSP hover must match registry members {}\n{}",
            via_resolve.value,
            via_lsp.contents
        );
        assert!(
            via_lsp.contents.contains(member),
            "hover {resolve_name} missing {member}: {}",
            via_lsp.contents
        );
    }
}

#[test]
fn lsp_and_compiler_hover_card_global() {
    let (src, _off, line, col) = at_caret("style({ bg(ca¦rd) })");
    let via_resolve = resolve_theme_token("card", "ship", "light").expect("resolve");
    let via_lsp = hover_sail(&src, Some("apps/demo/src/home.sail"), line, col).expect("lsp hover");
    assert!(
        via_lsp.contents.contains(&via_resolve.value),
        "card hover {}\n{}",
        via_resolve.value,
        via_lsp.contents
    );
}
