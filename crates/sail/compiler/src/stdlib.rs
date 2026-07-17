//! The L1 STDLIB REGISTRY — the language's element sources, authored in
//! `.sail` (one technology per layer) and compiled BY THIS CRATE on first use
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
    ("spinner", include_str!("../stdlib/spinner.sail")),
    ("separator", include_str!("../stdlib/separator.sail")),
    ("icon", include_str!("../stdlib/icon.sail")),
    ("label", include_str!("../stdlib/label.sail")),
    ("card", include_str!("../stdlib/card.sail")),
    ("switch", include_str!("../stdlib/switch.sail")),
    ("badge", include_str!("../stdlib/badge.sail")),
    ("avatar", include_str!("../stdlib/avatar.sail")),
    ("tooltip", include_str!("../stdlib/tooltip.sail")),
    ("alert", include_str!("../stdlib/alert.sail")),
    ("empty-state", include_str!("../stdlib/empty-state.sail")),
    ("error-state", include_str!("../stdlib/error-state.sail")),
    ("error-boundary", include_str!("../stdlib/error-boundary.sail")),
    ("code-block", include_str!("../stdlib/code-block.sail")),
    ("chip", include_str!("../stdlib/chip.sail")),
    ("field", include_str!("../stdlib/field.sail")),
    ("collapsible", include_str!("../stdlib/collapsible.sail")),
    ("slider", include_str!("../stdlib/slider.sail")),
    ("radio", include_str!("../stdlib/radio.sail")),
    ("radio-group", include_str!("../stdlib/radio-group.sail")),
    ("tabs", include_str!("../stdlib/tabs.sail")),
    ("tab-list", include_str!("../stdlib/tab-list.sail")),
    ("tab", include_str!("../stdlib/tab.sail")),
    ("tab-panel", include_str!("../stdlib/tab-panel.sail")),
    ("toggle-group", include_str!("../stdlib/toggle-group.sail")),
    ("toggle", include_str!("../stdlib/toggle.sail")),
    ("scroll-area", include_str!("../stdlib/scroll-area.sail")),
    ("ticker", include_str!("../stdlib/ticker.sail")),
    ("theme-picker", include_str!("../stdlib/theme-picker.sail")),
    ("navbar", include_str!("../stdlib/navbar.sail")),
    ("pagination", include_str!("../stdlib/pagination.sail")),
    ("page", include_str!("../stdlib/page.sail")),
    ("breadcrumb", include_str!("../stdlib/breadcrumb.sail")),
    ("popover", include_str!("../stdlib/popover.sail")),
    ("prose", include_str!("../stdlib/prose.sail")),
    ("combobox", include_str!("../stdlib/combobox.sail")),
    ("dropdown", include_str!("../stdlib/dropdown.sail")),
    ("menu-item", include_str!("../stdlib/menu-item.sail")),
    ("context-menu", include_str!("../stdlib/context-menu.sail")),
    ("hover-card", include_str!("../stdlib/hover-card.sail")),
    ("toast-region", include_str!("../stdlib/toast-region.sail")),
    ("toast", include_str!("../stdlib/toast.sail")),
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
