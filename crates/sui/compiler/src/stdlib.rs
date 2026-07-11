//! The L1 STDLIB REGISTRY — the language's element sources, authored in
//! `.sui` (one technology per layer) and compiled BY THIS CRATE on first use
//! (self-hosting: the compiler compiles its own standard library; no build
//! codegen, no drift). Element sources compile through
//! `parse_element_source` — the pipeline minus the stdlib pass — so there is
//! no recursion and no element-in-element.

use crate::emit::element::ElementOutput;
use std::sync::OnceLock;

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
    ("chip", include_str!("../stdlib/chip.sui")),
    ("field", include_str!("../stdlib/field.sui")),
    ("collapsible", include_str!("../stdlib/collapsible.sui")),
];

/// Compiled stdlib elements, memoized for the process lifetime. A stdlib
/// source that fails to compile is a TOOLCHAIN bug — panic with the error
/// (the element_mode integration tests catch it long before a user can).
pub fn elements() -> &'static [StdlibElement] {
    static REG: OnceLock<Vec<StdlibElement>> = OnceLock::new();
    REG.get_or_init(|| {
        SOURCES
            .iter()
            .map(|(tag, src)| StdlibElement {
                tag,
                out: crate::compile_element(src, tag).unwrap_or_else(|e| {
                    panic!("stdlib element `{tag}` failed to compile: {e:?}")
                }),
            })
            .collect()
    })
}

/// Lookup by tag.
pub fn element(tag: &str) -> Option<&'static StdlibElement> {
    elements().iter().find(|e| e.tag == tag)
}

/// The concatenated stdlib element skins — the compiler's half of the
/// framework stylesheet (the runtime's `LIGHT_CONTRACTS_CSS` is the other
/// half). This is the SSOT: the served `sui.css` and the materialized
/// `packages/sui/tokens/src/element-skins.css` both derive from here, and a
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
