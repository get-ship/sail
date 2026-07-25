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
    ("button", include_str!("../stdlib/button.sail")),
    ("a", include_str!("../stdlib/anchor.sail")),
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
/// half). This is the SSOT: the served `sail.css` and the materialized
/// `packages/sail/tokens/src/element-skins.css` both derive from here, and a
/// drift test re-derives and compares. Each element contributes its lowered
/// skin once (host sx + CVA-in-props), under a header naming the tag.
/// No residual parallel stylesheet.
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

/// Append freeform-HTML / runtime-anatomy descendant paint for `tag`.
/// Owned by the compiler element map → skins (not hand contracts).
pub fn append_descendant_skin(tag: &str, skin_css: &mut String) {
    let Some(map) = descendant_map(tag) else { return };
    let prefix = format!(".sail-{tag}");
    let css = lower_descendant_map(&prefix, map);
    if css.is_empty() {
        return;
    }
    if !skin_css.is_empty() && !skin_css.ends_with('\n') {
        skin_css.push('\n');
    }
    skin_css.push_str(&css);
}

fn descendant_map(tag: &str) -> Option<&'static [DescRule]> {
    match tag {
        "prose" => Some(PROSE_MAP),
        "code-block" => Some(CODE_BLOCK_MAP),
        // Ring PE must live in element-skins (host CVA only sizes/tones the host).
        "spinner" => Some(SPINNER_MAP),
        // list-trail anatomy for author-supplied li/a crumbs.
        "breadcrumb" => Some(BREADCRUMB_MAP),
        _ => None,
    }
}

const BREADCRUMB_MAP: &[DescRule] = &[
    DescRule {
        sel: "ol",
        paint: DescPaint(&[
            ("display", "flex"),
            ("align-items", "center"),
            ("flex-wrap", "wrap"),
            ("gap", "var(--space-2)"),
        ]),
    },
    DescRule {
        sel: "li",
        paint: DescPaint(&[
            ("display", "inline-flex"),
            ("align-items", "center"),
            ("gap", "var(--space-2)"),
        ]),
    },
    DescRule {
        sel: "li:not(:first-child)::before",
        paint: DescPaint(&[
            ("content", "'/'"),
            ("color", "var(--muted-foreground)"),
            ("user-select", "none"),
        ]),
    },
    DescRule {
        sel: "a",
        paint: DescPaint(&[
            ("color", "var(--muted-foreground)"),
            ("text-decoration", "none"),
            ("border-radius", "var(--radius-sm)"),
            (
                "transition",
                "color var(--duration-fast) var(--ease-out)",
            ),
        ]),
    },
    DescRule {
        sel: "a:hover",
        paint: DescPaint(&[("color", "var(--foreground)")]),
    },
    DescRule {
        sel: "a:focus-visible",
        paint: DescPaint(&[
            ("outline", "var(--ring-width) solid var(--ring)"),
            ("outline-offset", "var(--ring-offset)"),
        ]),
    },
    DescRule {
        sel: "[aria-current='page']",
        paint: DescPaint(&[
            ("color", "var(--foreground)"),
            ("font-weight", "var(--font-weight-medium)"),
        ]),
    },
];

const SPINNER_MAP: &[DescRule] = &[
    DescRule {
        sel: ".ring",
        paint: DescPaint(&[
            ("display", "block"),
            ("box-sizing", "border-box"),
            ("width", "100%"),
            ("height", "100%"),
            (
                "border",
                "var(--control-stroke) solid var(--border-subtle)",
            ),
            ("border-right-color", "currentColor"),
            ("border-radius", "var(--radius-full)"),
            ("animation", "var(--animate-spin)"),
        ]),
    },
];

struct DescRule {
    sel: &'static str,
    paint: DescPaint,
}

struct DescPaint(&'static [(&'static str, &'static str)]);

const PROSE_MAP: &[DescRule] = &[
    DescRule {
        sel: "h1, h2, h3, h4, h5, h6",
        paint: DescPaint(&[
            ("font-family", "var(--heading-font)"),
            ("font-weight", "var(--heading-weight)"),
            ("letter-spacing", "var(--heading-tracking)"),
            ("line-height", "var(--leading-tight)"),
            ("margin-top", "var(--space-5)"),
            ("margin-bottom", "var(--space-2)"),
        ]),
    },
    DescRule {
        sel: "h1",
        paint: DescPaint(&[
            ("font-size", "var(--text-2xl)"),
            ("margin-top", "var(--space-6)"),
            ("margin-bottom", "var(--space-3)"),
        ]),
    },
    DescRule {
        sel: "h2",
        paint: DescPaint(&[
            ("font-size", "var(--text-xl)"),
            ("margin-top", "var(--space-6)"),
            ("margin-bottom", "var(--space-3)"),
        ]),
    },
    DescRule { sel: "h3", paint: DescPaint(&[("font-size", "var(--text-lg)")]) },
    DescRule { sel: "h4, h5, h6", paint: DescPaint(&[("font-size", "var(--text-md)")]) },
    DescRule { sel: "p", paint: DescPaint(&[("margin-block", "var(--space-3)")]) },
    DescRule {
        sel: "a",
        paint: DescPaint(&[
            ("color", "var(--primary)"),
            ("text-decoration-line", "underline"),
            ("text-decoration-color", "var(--selected-border)"),
            ("text-underline-offset", "var(--ring-offset)"),
        ]),
    },
    DescRule {
        sel: "a:hover",
        paint: DescPaint(&[("text-decoration-color", "var(--primary)")]),
    },
    DescRule {
        sel: "code",
        paint: DescPaint(&[
            ("font-family", "var(--mono-font)"),
            ("font-size", "var(--text-sm)"),
            ("font-variant-numeric", "tabular-nums"),
            ("padding-block", "var(--space-1)"),
            ("padding-inline", "var(--space-2)"),
            ("border-radius", "var(--radius-sm)"),
            ("background-color", "var(--surface-1)"),
            ("border-width", "1px"),
            ("border-style", "solid"),
            ("border-color", "var(--border-subtle)"),
        ]),
    },
    DescRule {
        sel: "pre",
        paint: DescPaint(&[
            ("font-family", "var(--mono-font)"),
            ("font-size", "var(--text-sm)"),
            ("background-color", "var(--surface-2)"),
            ("border-width", "1px"),
            ("border-style", "solid"),
            ("border-color", "var(--border-subtle)"),
            ("border-radius", "var(--radius-md)"),
            ("padding-block", "var(--space-3)"),
            ("padding-inline", "var(--space-4)"),
            ("line-height", "var(--leading-normal-plus)"),
            ("white-space", "pre-wrap"),
            ("overflow-wrap", "anywhere"),
            ("word-break", "anywhere"),
        ]),
    },
    DescRule {
        sel: "pre code",
        paint: DescPaint(&[
            ("background-color", "transparent"),
            ("border-width", "0px"),
            ("padding", "0"),
        ]),
    },
    DescRule {
        sel: "blockquote",
        paint: DescPaint(&[
            ("margin-block", "var(--space-4)"),
            ("padding-block", "var(--space-2)"),
            ("padding-inline", "var(--space-4)"),
            ("border-left-width", "1px"),
            ("border-left-style", "solid"),
            ("border-left-width", "3px"),
            ("border-left-color", "var(--primary)"),
            ("color", "var(--muted-foreground)"),
            ("background-color", "var(--hover-bg)"),
            ("border-start-end-radius", "var(--radius-sm)"),
            ("border-end-end-radius", "var(--radius-sm)"),
        ]),
    },
    DescRule {
        sel: "ul, ol",
        paint: DescPaint(&[
            ("margin-block", "var(--space-3)"),
            ("padding-inline-start", "var(--space-5)"),
        ]),
    },
    DescRule { sel: "li", paint: DescPaint(&[("margin-block", "var(--space-1)")]) },
    DescRule {
        sel: "li::marker",
        paint: DescPaint(&[("color", "var(--muted-foreground)")]),
    },
    DescRule {
        sel: "strong",
        paint: DescPaint(&[("font-weight", "var(--font-weight-semibold)")]),
    },
    DescRule {
        sel: "table",
        paint: DescPaint(&[
            ("border-collapse", "collapse"),
            ("width", "100%"),
            ("margin", "var(--space-3) 0"),
            ("font-size", "var(--text-sm)"),
            ("border", "1px solid var(--border-subtle)"),
            ("border-radius", "var(--radius-md)"),
            ("overflow", "hidden"),
        ]),
    },
    DescRule {
        sel: "thead th",
        paint: DescPaint(&[
            ("background-color", "var(--muted)"),
            ("color", "var(--foreground)"),
            ("font-weight", "var(--font-weight-semibold)"),
        ]),
    },
    DescRule {
        sel: "th, td",
        paint: DescPaint(&[
            ("border-bottom-width", "1px"),
            ("border-bottom-style", "solid"),
            ("border-bottom-color", "var(--border-subtle)"),
            ("padding-block", "var(--space-2)"),
            ("padding-inline", "var(--space-3)"),
            ("text-align", "start"),
            ("font-variant-numeric", "tabular-nums"),
        ]),
    },
    DescRule {
        sel: "tbody tr:last-child td",
        paint: DescPaint(&[("border-bottom", "0")]),
    },
];

const CODE_BLOCK_MAP: &[DescRule] = &[
    DescRule { sel: ".tok-tag", paint: DescPaint(&[("color", "var(--syntax-tag)")]) },
    DescRule { sel: ".tok-attr", paint: DescPaint(&[("color", "var(--syntax-attr)")]) },
    DescRule { sel: ".tok-string", paint: DescPaint(&[("color", "var(--syntax-string)")]) },
    DescRule {
        sel: ".tok-comment",
        paint: DescPaint(&[("color", "var(--syntax-comment)"), ("font-style", "italic")]),
    },
    DescRule {
        sel: ".tok-keyword",
        paint: DescPaint(&[("color", "var(--syntax-keyword)")]),
    },
    DescRule { sel: ".tok-fn", paint: DescPaint(&[("color", "var(--syntax-function)")]) },
    DescRule {
        sel: ".tok-number",
        paint: DescPaint(&[("color", "var(--syntax-number)")]),
    },
    DescRule { sel: ".tok-bind", paint: DescPaint(&[("color", "var(--syntax-function)")]) },
];

fn lower_descendant_map(host: &str, rules: &[DescRule]) -> String {
    let mut out = String::new();
    for rule in rules {
        let body: String = rule
            .paint
            .0
            .iter()
            .map(|(p, v)| format!("{p}:{v};"))
            .collect();
        if body.is_empty() {
            panic!("descendant skin for {host} selector `{}` lowered empty", rule.sel);
        }
        let sels: Vec<String> = rule
            .sel
            .split(',')
            .map(|s| format!("{host} {}", s.trim()))
            .collect();
        out.push_str(&sels.join(", "));
        out.push_str(" { ");
        out.push_str(&body);
        out.push_str(" }\n");
    }
    out
}

