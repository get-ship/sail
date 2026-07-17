//! The L1 STDLIB REGISTRY — the language's element sources, authored in
//! `.sui` (one technology per layer) and compiled BY THIS CRATE on first use
//! (self-hosting: the compiler compiles its own standard library; no build
//! codegen, no drift). Element sources compile through
//! `parse_element_source` — the pipeline minus the stdlib pass — so there is
//! no recursion and no element-in-element.

use crate::emit::element::ElementOutput;
use std::sync::LazyLock;

pub struct StdlibElement {
    pub tag: &'static str,
    pub out: ElementOutput,
}

const SOURCES: &[(&str, &str)] = &[
    ("spinner", include_str!("../stdlib/spinner.sui")),
    ("separator", include_str!("../stdlib/separator.sui")),
    ("icon", include_str!("../stdlib/icon.sui")),
    ("label", include_str!("../stdlib/label.sui")),
    ("card", include_str!("../stdlib/card.sui")),
    ("switch", include_str!("../stdlib/switch.sui")),
    ("badge", include_str!("../stdlib/badge.sui")),
    ("avatar", include_str!("../stdlib/avatar.sui")),
    ("tooltip", include_str!("../stdlib/tooltip.sui")),
    ("alert", include_str!("../stdlib/alert.sui")),
    ("empty-state", include_str!("../stdlib/empty-state.sui")),
    ("error-state", include_str!("../stdlib/error-state.sui")),
    ("error-boundary", include_str!("../stdlib/error-boundary.sui")),
    ("code-block", include_str!("../stdlib/code-block.sui")),
    ("chip", include_str!("../stdlib/chip.sui")),
    ("field", include_str!("../stdlib/field.sui")),
    ("collapsible", include_str!("../stdlib/collapsible.sui")),
    ("slider", include_str!("../stdlib/slider.sui")),
    ("radio", include_str!("../stdlib/radio.sui")),
    ("radio-group", include_str!("../stdlib/radio-group.sui")),
    ("tabs", include_str!("../stdlib/tabs.sui")),
    ("tab-list", include_str!("../stdlib/tab-list.sui")),
    ("tab", include_str!("../stdlib/tab.sui")),
    ("tab-panel", include_str!("../stdlib/tab-panel.sui")),
    ("toggle-group", include_str!("../stdlib/toggle-group.sui")),
    ("toggle", include_str!("../stdlib/toggle.sui")),
    ("scroll-area", include_str!("../stdlib/scroll-area.sui")),
    ("ticker", include_str!("../stdlib/ticker.sui")),
    ("theme-picker", include_str!("../stdlib/theme-picker.sui")),
    ("navbar", include_str!("../stdlib/navbar.sui")),
    ("pagination", include_str!("../stdlib/pagination.sui")),
    ("page", include_str!("../stdlib/page.sui")),
    ("breadcrumb", include_str!("../stdlib/breadcrumb.sui")),
    ("popover", include_str!("../stdlib/popover.sui")),
    ("prose", include_str!("../stdlib/prose.sui")),
    ("combobox", include_str!("../stdlib/combobox.sui")),
    ("dropdown", include_str!("../stdlib/dropdown.sui")),
    ("menu-item", include_str!("../stdlib/menu-item.sui")),
    ("context-menu", include_str!("../stdlib/context-menu.sui")),
    ("hover-card", include_str!("../stdlib/hover-card.sui")),
    ("toast-region", include_str!("../stdlib/toast-region.sui")),
    ("toast", include_str!("../stdlib/toast.sui")),
];

/// Compiled stdlib elements, memoized for the process lifetime. A stdlib
/// source that fails to compile is a TOOLCHAIN bug — panic with the error
/// (the element_mode integration tests catch it long before a user can).
/// The raw (tag, source) pairs — the prop-registry generator re-parses these
/// to emit the typed-props registry the lint validates against.
pub fn sources() -> &'static [(&'static str, &'static str)] {
    SOURCES
}

pub fn elements() -> &'static [StdlibElement] {
    static REG: LazyLock<Vec<StdlibElement>> = LazyLock::new(|| {
        SOURCES
            .iter()
            .map(|(tag, src)| StdlibElement {
                tag,
                out: crate::compile_element(src, tag).unwrap_or_else(|e| {
                    panic!("stdlib element `{tag}` failed to compile: {e:?}")
                }),
            })
            .collect()
    });
    &REG
}

/// Lookup by tag.
pub fn element(tag: &str) -> Option<&'static StdlibElement> {
    elements().iter().find(|e| e.tag == tag)
}

/// The concatenated stdlib element skins — the compiler's half of the
/// framework stylesheet (the runtime's `LIGHT_CONTRACTS_CSS` is the other
/// half). This is the SSOT: the served `sui.css` and the materialized
/// `packages/sail/tokens/src/element-skins.css` both derive from here, and a
/// drift test re-derives and compares. Each element contributes its skin
/// once, under a header naming the tag.
pub fn skins_css() -> String {
    let mut out = String::new();
    for el in elements() {
        if el.out.skin_css.trim().is_empty() {
            continue;
        }
        out.push_str(&format!("/* {} */\n", el.tag));
        out.push_str(el.out.skin_css.trim_end());
        out.push_str("\n\n");
    }
    out.truncate(out.trim_end().len());
    out.push('\n');
    out
}
