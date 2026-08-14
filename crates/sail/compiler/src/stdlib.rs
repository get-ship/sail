//! The L1 STDLIB REGISTRY — the language's element sources, authored in
//! `.sail` (one technology per layer) and compiled BY THIS CRATE on first use
//! (self-hosting: the compiler compiles its own standard library; no build
//! codegen, no drift). Element sources compile through
//! `parse_element_source` — the pipeline minus the stdlib pass — so there is
//! no recursion and no element-in-element.
//!
//! AUTHORING NOTE — the `empty:hidden` gate. Optional-region wrappers (card
//! header/footer, alert icon/title/actions, empty-state / error-state
//! sections, field hint) collapse when unfilled via
//! `when({ empty: style.display(none) })` on the wrapper's sx. The gate is
//! spelled out per element, per wrapper, because stdlib sources compile
//! STANDALONE (one `compile_element` per file below; the Style const env is
//! built from that file's own `<scope>`) and there is no include/import
//! between stdlib `.sail` files — a shared `emptyGate` Style const has no
//! home a second file could reference. If a cross-stdlib Style module
//! mechanism ever lands, this gate is the first candidate to hoist.

use crate::emit::element::ElementOutput;
use std::sync::LazyLock;

pub struct StdlibElement {
    pub tag: &'static str,
    pub out: ElementOutput,
}

const SOURCES: &[(&str, &str)] = &[
    ("spinner", include_str!("../stdlib/spinner.sail")),
    ("skeleton", include_str!("../stdlib/skeleton.sail")),
    ("button", include_str!("../stdlib/button.sail")),
    ("input", include_str!("../stdlib/input.sail")),
    ("checkbox", include_str!("../stdlib/checkbox.sail")),
    ("date", include_str!("../stdlib/date.sail")),
    ("file", include_str!("../stdlib/file.sail")),
    ("color", include_str!("../stdlib/color.sail")),
    ("textarea", include_str!("../stdlib/textarea.sail")),
    ("select", include_str!("../stdlib/select.sail")),
    ("dialog", include_str!("../stdlib/dialog.sail")),
    ("accordion", include_str!("../stdlib/accordion.sail")),
    ("table", include_str!("../stdlib/table.sail")),
    ("fieldset", include_str!("../stdlib/fieldset.sail")),
    ("legend", include_str!("../stdlib/legend.sail")),
    ("progress", include_str!("../stdlib/progress.sail")),
    ("meter", include_str!("../stdlib/meter.sail")),
    ("a", include_str!("../stdlib/anchor.sail")),
    ("separator", include_str!("../stdlib/separator.sail")),
    ("kbd", include_str!("../stdlib/kbd.sail")),
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
    (
        "error-boundary",
        include_str!("../stdlib/error-boundary.sail"),
    ),
    ("code-block", include_str!("../stdlib/code-block.sail")),
    ("chip", include_str!("../stdlib/chip.sail")),
    ("field", include_str!("../stdlib/field.sail")),
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
                out: crate::compile_element(src, tag)
                    .unwrap_or_else(|e| panic!("stdlib element `{tag}` failed to compile: {e:?}")),
            })
            .collect()
    });
    &REG
}

/// Lookup by tag.
pub fn element(tag: &str) -> Option<&'static StdlibElement> {
    elements().iter().find(|e| e.tag == tag)
}

/// Platform adaptor CSS — vendor/UA/native plumbing the vocabulary cannot
/// yet express. Authored as files under `stdlib/platform/`; never Rust strings.
const PLATFORM: &[(&str, &str)] = &[
    (
        "accordion",
        include_str!("../stdlib/platform/accordion.css"),
    ),
    (
        "breadcrumb",
        include_str!("../stdlib/platform/breadcrumb.css"),
    ),
    ("button", include_str!("../stdlib/platform/button.css")),
    ("card", include_str!("../stdlib/platform/card.css")),
    ("checkbox", include_str!("../stdlib/platform/checkbox.css")),
    ("chip", include_str!("../stdlib/platform/chip.css")),
    (
        "code-block",
        include_str!("../stdlib/platform/code-block.css"),
    ),
    ("color", include_str!("../stdlib/platform/color.css")),
    ("dialog", include_str!("../stdlib/platform/dialog.css")),
    ("file", include_str!("../stdlib/platform/file.css")),
    (
        "menu-item",
        include_str!("../stdlib/platform/menu-item.css"),
    ),
    ("meter", include_str!("../stdlib/platform/meter.css")),
    ("progress", include_str!("../stdlib/platform/progress.css")),
    ("prose", include_str!("../stdlib/platform/prose.css")),
    ("radio", include_str!("../stdlib/platform/radio.css")),
    ("select", include_str!("../stdlib/platform/select.css")),
    ("skeleton", include_str!("../stdlib/platform/skeleton.css")),
    ("slider", include_str!("../stdlib/platform/slider.css")),
    ("table", include_str!("../stdlib/platform/table.css")),
];

fn platform_css(tag: &str) -> Option<&'static str> {
    PLATFORM
        .iter()
        .find(|(t, _)| *t == tag)
        .map(|(_, css)| *css)
}

/// The concatenated stdlib element skins — the compiler's half of the
/// framework stylesheet (hand-authored `packages/sail/tokens/src/sail-base.css`
/// is the document floor). This is the SSOT for element skins: the served
/// `sail.css` and the materialized `packages/sail/tokens/src/element-skins.css`
/// both derive from here, and a drift test re-derives and compares. Each
/// element contributes lowered host Style, then any platform adaptor file,
/// under structured provenance headers.
pub fn skins_css() -> String {
    let mut out = String::new();
    for el in elements() {
        let style = el.out.skin_css.trim_end();
        let platform = platform_css(el.tag)
            .map(str::trim_end)
            .filter(|s| !s.is_empty());
        if style.is_empty() && platform.is_none() {
            continue;
        }
        out.push_str("/* @element ");
        out.push_str(el.tag);
        out.push_str(" */\n");
        if !style.is_empty() {
            out.push_str(style);
            out.push('\n');
        }
        if let Some(css) = platform {
            out.push_str("/* @provenance platform/");
            out.push_str(el.tag);
            out.push_str(".css */\n");
            out.push_str(css);
            out.push('\n');
        }
        out.push('\n');
    }
    out.truncate(out.trim_end().len());
    out.push('\n');
    out
}
