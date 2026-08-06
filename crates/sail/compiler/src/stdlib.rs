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
    ("input", include_str!("../stdlib/input.sail")),
    ("textarea", include_str!("../stdlib/textarea.sail")),
    ("select", include_str!("../stdlib/select.sail")),
    ("dialog", include_str!("../stdlib/dialog.sail")),
    ("details", include_str!("../stdlib/details.sail")),
    ("summary", include_str!("../stdlib/summary.sail")),
    ("table", include_str!("../stdlib/table.sail")),
    ("fieldset", include_str!("../stdlib/fieldset.sail")),
    ("legend", include_str!("../stdlib/legend.sail")),
    ("progress", include_str!("../stdlib/progress.sail")),
    ("meter", include_str!("../stdlib/meter.sail")),
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
    (
        "error-boundary",
        include_str!("../stdlib/error-boundary.sail"),
    ),
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

/// The concatenated stdlib element skins — the compiler's half of the
/// framework stylesheet (hand-authored `packages/sail/tokens/src/sail-base.css`
/// is the document floor). This is the SSOT for element skins: the served
/// `sail.css` and the materialized `packages/sail/tokens/src/element-skins.css`
/// both derive from here, and a drift test re-derives and compares. Each
/// element contributes its lowered skin once (host sx + CVA-in-props), under
/// a header naming the tag. No residual parallel stylesheet.
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
    let Some(map) = descendant_map(tag) else {
        return;
    };
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
        "breadcrumb" => Some(BREADCRUMB_MAP),
        "dialog" => Some(DIALOG_MAP),
        // details + collapsible both host a native <summary> chevron; collapsible
        // is its own CE (`.sail-collapsible`) so it needs the same map — without
        // it summary::after is only a solid bg square (mask never applied).
        "details" => Some(DETAILS_MAP),
        "collapsible" => Some(DETAILS_MAP),
        "summary" => Some(SUMMARY_MAP),
        "table" => Some(TABLE_MAP),
        "progress" => Some(PROGRESS_MAP),
        "meter" => Some(METER_MAP),
        "chip" => Some(CHIP_MAP),
        "input" => Some(INPUT_MAP),
        "select" => Some(SELECT_MAP),
        "slider" => Some(RANGE_MAP),
        "radio" => Some(RADIO_INPUT_MAP),
        _ => None,
    }
}

// White check / dash on tone fill (product marks — not Style bgImage vocab).
const CHECK_MARK: &str = "url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 16 16'%3E%3Cpath fill='none' stroke='white' stroke-width='2.5' stroke-linecap='round' stroke-linejoin='round' d='M3.5 8.5l3 3 6-6'/%3E%3C/svg%3E\")";
const DASH_MARK: &str = "url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 16 16'%3E%3Cpath fill='none' stroke='white' stroke-width='2.5' stroke-linecap='round' d='M4 8h8'/%3E%3C/svg%3E\")";
const SELECT_CHEVRON: &str = "url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='16' height='16' viewBox='0 0 16 16' fill='none' stroke='%23888888' stroke-width='1.75' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpath d='M4 6l4 4 4-4'/%3E%3C/svg%3E\")";
// Mask-source chevron (black stroke) — colored via background on summary::after.
const SUMMARY_CHEVRON_MASK: &str = "url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 16 16' fill='none' stroke='%23000' stroke-width='1.75' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpath d='M4 6l4 4 4-4'/%3E%3C/svg%3E\") center / 1rem no-repeat";
const INDET_GRADIENT: &str =
    "linear-gradient(90deg, transparent 0%, var(--primary) 50%, transparent 100%)";

const INPUT_MAP: &[DescRule] = &[
    DescRule {
        sel: "[type='checkbox']:checked",
        paint: DescPaint(&[
            ("background-image", CHECK_MARK),
            ("background-position", "center"),
            ("background-size", "0.9em"),
            ("background-repeat", "no-repeat"),
        ]),
    },
    DescRule {
        sel: "[type='checkbox']:indeterminate",
        paint: DescPaint(&[
            ("background-color", "var(--primary)"),
            ("border-color", "var(--primary)"),
            ("background-image", DASH_MARK),
            ("background-position", "center"),
            ("background-size", "0.9em"),
            ("background-repeat", "no-repeat"),
        ]),
    },
    DescRule {
        sel: "[type='radio']:checked",
        paint: DescPaint(&[("box-shadow", "inset 0 0 0 0.22em var(--background)")]),
    },
    // Range: vendor track/thumb (Style cannot express these pseudos).
    DescRule {
        sel: "[type='range']",
        paint: DescPaint(&[
            ("--rng-track", "var(--space-1_5)"),
            ("--rng-thumb", "var(--space-4)"),
        ]),
    },
    DescRule {
        sel: "[type='range'][size='sm']",
        paint: DescPaint(&[
            ("--rng-track", "var(--space-1)"),
            ("--rng-thumb", "var(--space-3)"),
        ]),
    },
    DescRule {
        sel: "[type='range'][size='lg']",
        paint: DescPaint(&[
            ("--rng-track", "var(--space-2)"),
            ("--rng-thumb", "var(--space-5)"),
        ]),
    },
    DescRule {
        sel: "[type='range']::-webkit-slider-runnable-track",
        paint: DescPaint(&[
            ("height", "var(--rng-track)"),
            ("border-radius", "var(--shape-pill)"),
            (
                "background",
                "linear-gradient(to right, var(--primary) 0 var(--rng-pct, 0%), var(--muted) var(--rng-pct, 0%) 100%)",
            ),
            ("box-shadow", "inset 0 0 0 1px var(--border)"),
        ]),
    },
    DescRule {
        sel: "[type='range']::-moz-range-track",
        paint: DescPaint(&[
            ("height", "var(--rng-track)"),
            ("border-radius", "var(--shape-pill)"),
            ("background", "var(--muted)"),
            ("box-shadow", "inset 0 0 0 1px var(--border)"),
        ]),
    },
    DescRule {
        sel: "[type='range']::-moz-range-progress",
        paint: DescPaint(&[
            ("height", "var(--rng-track)"),
            ("border-radius", "var(--shape-pill)"),
            ("background", "var(--primary)"),
        ]),
    },
    DescRule {
        sel: "[type='range']::-webkit-slider-thumb",
        paint: DescPaint(&[
            ("appearance", "none"),
            ("-webkit-appearance", "none"),
            ("box-sizing", "border-box"),
            ("width", "var(--rng-thumb)"),
            ("height", "var(--rng-thumb)"),
            (
                "margin-top",
                "calc((var(--rng-track) - var(--rng-thumb)) / 2)",
            ),
            ("border-radius", "50%"),
            ("background", "var(--surface-1, var(--background))"),
            ("border", "2px solid var(--primary)"),
            ("box-shadow", "var(--shadow-sm)"),
        ]),
    },
    DescRule {
        sel: "[type='range']::-moz-range-thumb",
        paint: DescPaint(&[
            ("box-sizing", "border-box"),
            ("width", "var(--rng-thumb)"),
            ("height", "var(--rng-thumb)"),
            ("border-radius", "50%"),
            ("background", "var(--surface-1, var(--background))"),
            ("border", "2px solid var(--primary)"),
            ("box-shadow", "var(--shadow-sm)"),
        ]),
    },
    DescRule {
        sel: "[type='checkbox'][invalid], [type='radio'][invalid], [type='checkbox'][aria-invalid='true'], [type='radio'][aria-invalid='true']",
        paint: DescPaint(&[("border-color", "var(--destructive)")]),
    },
];

const SELECT_MAP: &[DescRule] = &[
    DescRule {
        // Host self: chevron glyph only. End pad lives on Style base (`padE(8)`)
        // so the size ladder can set `padS` without clobbering chevron room.
        sel: "",
        paint: DescPaint(&[
            ("background-image", SELECT_CHEVRON),
            ("background-repeat", "no-repeat"),
            ("background-position", "right var(--space-3) center"),
            ("background-size", "1rem"),
        ]),
    },
    DescRule {
        sel: "option",
        paint: DescPaint(&[
            ("color", "var(--foreground)"),
            ("background", "var(--background)"),
        ]),
    },
];

/// Nested control paint for `<radio>` label hosts (`.sail-radio > input`).
const RADIO_INPUT_MAP: &[DescRule] = &[
    DescRule {
        sel: "input:checked, input[aria-checked='true']",
        paint: DescPaint(&[("box-shadow", "inset 0 0 0 0.22em var(--background)")]),
    },
    DescRule {
        sel: "input[invalid], input[aria-invalid='true']",
        paint: DescPaint(&[("border-color", "var(--destructive)")]),
    },
];

/// Range vendor chrome for `<slider>` (host `.sail-slider`).
const RANGE_MAP: &[DescRule] = &[
    DescRule {
        sel: "",
        paint: DescPaint(&[
            ("--rng-track", "var(--space-1_5)"),
            ("--rng-thumb", "var(--space-4)"),
        ]),
    },
    DescRule {
        sel: "[size='sm']",
        paint: DescPaint(&[
            ("--rng-track", "var(--space-1)"),
            ("--rng-thumb", "var(--space-3)"),
        ]),
    },
    DescRule {
        sel: "[size='lg']",
        paint: DescPaint(&[
            ("--rng-track", "var(--space-2)"),
            ("--rng-thumb", "var(--space-5)"),
        ]),
    },
    DescRule {
        sel: "::-webkit-slider-runnable-track",
        paint: DescPaint(&[
            ("height", "var(--rng-track)"),
            ("border-radius", "var(--shape-pill)"),
            (
                "background",
                "linear-gradient(to right, var(--primary) 0 var(--rng-pct, 0%), var(--muted) var(--rng-pct, 0%) 100%)",
            ),
            ("box-shadow", "inset 0 0 0 1px var(--border)"),
        ]),
    },
    DescRule {
        sel: "::-moz-range-track",
        paint: DescPaint(&[
            ("height", "var(--rng-track)"),
            ("border-radius", "var(--shape-pill)"),
            ("background", "var(--muted)"),
            ("box-shadow", "inset 0 0 0 1px var(--border)"),
        ]),
    },
    DescRule {
        sel: "::-moz-range-progress",
        paint: DescPaint(&[
            ("height", "var(--rng-track)"),
            ("border-radius", "var(--shape-pill)"),
            ("background", "var(--primary)"),
        ]),
    },
    DescRule {
        sel: "::-webkit-slider-thumb",
        paint: DescPaint(&[
            ("appearance", "none"),
            ("-webkit-appearance", "none"),
            ("box-sizing", "border-box"),
            ("width", "var(--rng-thumb)"),
            ("height", "var(--rng-thumb)"),
            (
                "margin-top",
                "calc((var(--rng-track) - var(--rng-thumb)) / 2)",
            ),
            ("border-radius", "50%"),
            ("background", "var(--surface-1, var(--background))"),
            ("border", "2px solid var(--primary)"),
            ("box-shadow", "var(--shadow-sm)"),
        ]),
    },
    DescRule {
        sel: "::-moz-range-thumb",
        paint: DescPaint(&[
            ("box-sizing", "border-box"),
            ("width", "var(--rng-thumb)"),
            ("height", "var(--rng-thumb)"),
            ("border-radius", "50%"),
            ("background", "var(--surface-1, var(--background))"),
            ("border", "2px solid var(--primary)"),
            ("box-shadow", "var(--shadow-sm)"),
        ]),
    },
];

const CHIP_MAP: &[DescRule] = &[
    DescRule {
        sel: ":not([removable]) button[aria-label='Remove']",
        paint: DescPaint(&[("display", "none")]),
    },
    DescRule {
        sel: "[removable] button[aria-label='Remove']",
        paint: DescPaint(&[
            ("display", "inline-flex"),
            ("align-items", "center"),
            ("justify-content", "center"),
        ]),
    },
];

const DIALOG_MAP: &[DescRule] = &[
    DescRule {
        sel: "::backdrop",
        paint: DescPaint(&[("background", "var(--backdrop)"), ("opacity", "1")]),
    },
    DescRule {
        sel: "[backdrop='blur']::backdrop",
        paint: DescPaint(&[
            ("background", "var(--backdrop)"),
            ("backdrop-filter", "blur(6px)"),
            ("-webkit-backdrop-filter", "blur(6px)"),
        ]),
    },
    DescRule {
        sel: "[backdrop='solid']::backdrop",
        paint: DescPaint(&[("background", "var(--background)")]),
    },
    // Side sheets: dock geometry Style cannot express (min() width, edge margin, flush border).
    DescRule {
        sel: "[side='start'], [side='end']",
        paint: DescPaint(&[
            (
                "width",
                "min(calc(100vw - var(--space-7)), var(--sheet-size, var(--panel-max-w-sm)))",
            ),
            ("height", "100vh"),
            ("max-width", "none"),
            ("max-height", "none"),
            ("border-radius", "0"),
        ]),
    },
    DescRule {
        sel: "[side='start']",
        paint: DescPaint(&[("margin", "0 auto 0 0"), ("border-left", "0")]),
    },
    DescRule {
        sel: "[side='end']",
        paint: DescPaint(&[("margin", "0 0 0 auto"), ("border-right", "0")]),
    },
    DescRule {
        sel: "[side='top'], [side='bottom']",
        paint: DescPaint(&[
            ("width", "100vw"),
            (
                "max-height",
                "min(calc(100vh - var(--space-7)), var(--panel-vh-md, 80vh))",
            ),
            ("max-width", "none"),
            ("border-radius", "0"),
        ]),
    },
    DescRule {
        sel: "[side='top']",
        paint: DescPaint(&[("margin", "0 0 auto 0"), ("border-top", "0")]),
    },
    DescRule {
        sel: "[side='bottom']",
        paint: DescPaint(&[("margin", "auto 0 0 0"), ("border-bottom", "0")]),
    },
    DescRule {
        sel: "[side][size='md']",
        paint: DescPaint(&[("--sheet-size", "var(--panel-max-w-md)")]),
    },
    DescRule {
        sel: "[side][size='lg']",
        paint: DescPaint(&[("--sheet-size", "var(--panel-max-w-lg)")]),
    },
    // Flat content (no anatomy slots) — still a real surface, not a pad-0 pill.
    // Chained :not(:has) avoids relying on comma-in-:has (also fixed in split_selector_list).
    DescRule {
        sel: "[open]:not(:has(> header)):not(:has(> section)):not(:has(> footer))",
        paint: DescPaint(&[
            ("padding", "var(--space-5) var(--space-6)"),
            ("gap", "var(--space-4)"),
        ]),
    },
    DescRule {
        sel: "> header",
        paint: DescPaint(&[
            ("flex", "0 0 auto"),
            ("padding", "var(--space-4_5) var(--space-6)"),
            ("border-bottom", "1px solid var(--border-subtle)"),
            ("font-size", "var(--text-lg)"),
            ("font-weight", "var(--font-weight-semibold)"),
            ("color", "var(--foreground)"),
        ]),
    },
    DescRule {
        sel: "> section",
        paint: DescPaint(&[
            ("flex", "1 1 auto"),
            ("min-height", "0"),
            ("overflow-y", "auto"),
            ("overscroll-behavior", "contain"),
            ("padding", "var(--space-4) var(--space-6)"),
            ("font-size", "var(--text-base)"),
            ("line-height", "var(--leading-normal)"),
            ("color", "var(--muted-foreground)"),
        ]),
    },
    DescRule {
        sel: "> footer",
        paint: DescPaint(&[
            ("flex", "0 0 auto"),
            ("display", "flex"),
            ("gap", "var(--space-2)"),
            ("align-items", "center"),
            ("justify-content", "flex-end"),
            ("padding", "var(--space-4) var(--space-6)"),
            ("border-top", "1px solid var(--border-subtle)"),
            ("background", "var(--muted)"),
        ]),
    },
];

const DETAILS_MAP: &[DescRule] = &[
    // Kill UA disclosure marker on the hosted summary (collapsible does not
    // wrap the <summary> L1 component, so SUMMARY_MAP never runs on it).
    DescRule {
        sel: "> summary::-webkit-details-marker",
        paint: DescPaint(&[("display", "none")]),
    },
    DescRule {
        sel: "> summary::marker",
        paint: DescPaint(&[("content", "''")]),
    },
    // Chevron via mask + background color (sx after:bg supplies the fill).
    // Without mask, after:size + bg paints a solid square — the broken caret.
    DescRule {
        sel: "> summary::after",
        paint: DescPaint(&[
            ("flex", "0 0 auto"),
            ("-webkit-mask", SUMMARY_CHEVRON_MASK),
            ("mask", SUMMARY_CHEVRON_MASK),
            (
                "transition",
                "transform var(--duration-medium) var(--ease-out), background var(--duration-fast) var(--ease-out)",
            ),
        ]),
    },
    DescRule {
        sel: "[open] > summary",
        paint: DescPaint(&[
            ("background", "var(--muted)"),
            ("border-bottom", "1px solid var(--border-subtle)"),
            ("border-radius", "var(--radius-lg) var(--radius-lg) 0 0"),
        ]),
    },
    DescRule {
        sel: "[open] > summary::after",
        paint: DescPaint(&[
            ("transform", "rotate(180deg)"),
            ("background", "var(--foreground)"),
        ]),
    },
    DescRule {
        sel: "> summary ~ *",
        paint: DescPaint(&[("margin", "var(--space-3) var(--space-4)")]),
    },
    // Stacked accordion: merge shared edges when details sit back-to-back.
    DescRule {
        sel: ":has(+ .sail-details)",
        paint: DescPaint(&[
            ("border-bottom-left-radius", "0"),
            ("border-bottom-right-radius", "0"),
        ]),
    },
    DescRule {
        sel: "+ .sail-details",
        paint: DescPaint(&[
            ("border-top", "0"),
            ("border-top-left-radius", "0"),
            ("border-top-right-radius", "0"),
        ]),
    },
    // Collapsible hosts stack the same way (class is .sail-collapsible).
    DescRule {
        sel: ":has(+ .sail-collapsible)",
        paint: DescPaint(&[
            ("border-bottom-left-radius", "0"),
            ("border-bottom-right-radius", "0"),
        ]),
    },
    DescRule {
        sel: "+ .sail-collapsible",
        paint: DescPaint(&[
            ("border-top", "0"),
            ("border-top-left-radius", "0"),
            ("border-top-right-radius", "0"),
        ]),
    },
];

const SUMMARY_MAP: &[DescRule] = &[
    DescRule {
        sel: "::-webkit-details-marker",
        paint: DescPaint(&[("display", "none")]),
    },
    DescRule {
        sel: "::marker",
        paint: DescPaint(&[("content", "''")]),
    },
    DescRule {
        sel: "::after",
        paint: DescPaint(&[
            ("flex", "0 0 auto"),
            ("-webkit-mask", SUMMARY_CHEVRON_MASK),
            ("mask", SUMMARY_CHEVRON_MASK),
            (
                "transition",
                "transform var(--duration-medium) var(--ease-out), background var(--duration-fast) var(--ease-out)",
            ),
        ]),
    },
];

const TABLE_MAP: &[DescRule] = &[
    DescRule {
        sel: "caption",
        paint: DescPaint(&[
            ("padding", "var(--space-2) var(--space-3)"),
            ("color", "var(--muted-foreground)"),
            ("text-align", "left"),
        ]),
    },
    DescRule {
        sel: "th",
        paint: DescPaint(&[
            ("padding", "var(--space-2_5) var(--space-3)"),
            ("font-weight", "var(--font-weight-semibold)"),
            ("color", "var(--muted-foreground)"),
            (
                "border-bottom",
                "1px solid color-mix(in srgb, var(--foreground) 16%, transparent)",
            ),
            ("white-space", "nowrap"),
            ("text-align", "left"),
        ]),
    },
    DescRule {
        sel: "td",
        paint: DescPaint(&[
            ("padding", "var(--space-2_5) var(--space-3)"),
            (
                "border-bottom",
                "1px solid color-mix(in srgb, var(--foreground) 9%, transparent)",
            ),
        ]),
    },
    DescRule {
        sel: "tbody > tr:last-child > td",
        paint: DescPaint(&[("border-bottom", "0")]),
    },
    DescRule {
        sel: "[dense] th, [dense] td",
        paint: DescPaint(&[("padding", "var(--space-1_5) var(--space-2)")]),
    },
    DescRule {
        sel: "[striped] > tbody > tr:nth-child(even)",
        paint: DescPaint(&[("background", "var(--surface-2)")]),
    },
    DescRule {
        sel: "[hoverable] > tbody > tr:hover",
        paint: DescPaint(&[("background", "var(--surface-2)")]),
    },
    DescRule {
        sel: "[bordered] th, [bordered] td",
        paint: DescPaint(&[(
            "border",
            "1px solid color-mix(in srgb, var(--foreground) 9%, transparent)",
        )]),
    },
    DescRule {
        sel: "thead[sticky] th",
        paint: DescPaint(&[
            ("position", "sticky"),
            ("top", "0"),
            ("z-index", "1"),
            ("background", "var(--background)"),
        ]),
    },
    DescRule {
        sel: "th[numeric], td[numeric]",
        paint: DescPaint(&[
            ("text-align", "right"),
            ("font-variant-numeric", "tabular-nums"),
        ]),
    },
];

const PROGRESS_MAP: &[DescRule] = &[
    DescRule {
        sel: "::-webkit-progress-bar",
        paint: DescPaint(&[
            ("background", "var(--muted)"),
            ("border-radius", "var(--shape-pill)"),
        ]),
    },
    DescRule {
        sel: "::-webkit-progress-value",
        paint: DescPaint(&[
            ("background", "var(--primary)"),
            ("border-radius", "var(--shape-pill)"),
        ]),
    },
    DescRule {
        sel: "::-moz-progress-bar",
        paint: DescPaint(&[
            ("background", "var(--primary)"),
            ("border-radius", "var(--shape-pill)"),
        ]),
    },
    // Indeterminate (no value attr): shimmer on host + WebKit bar.
    DescRule {
        sel: ":not([value])",
        paint: DescPaint(&[
            ("background-image", INDET_GRADIENT),
            ("background-repeat", "no-repeat"),
            ("background-size", "40% 100%"),
            ("animation", "var(--animate-progress-indeterminate)"),
        ]),
    },
    DescRule {
        sel: ":not([value])::-webkit-progress-bar",
        paint: DescPaint(&[
            ("background-image", INDET_GRADIENT),
            ("background-repeat", "no-repeat"),
            ("background-size", "40% 100%"),
            ("animation", "var(--animate-progress-indeterminate)"),
        ]),
    },
];

const METER_MAP: &[DescRule] = &[
    DescRule {
        sel: "::-webkit-meter-bar",
        paint: DescPaint(&[
            ("background", "var(--muted)"),
            ("border", "0"),
            ("border-radius", "var(--shape-pill)"),
        ]),
    },
    DescRule {
        sel: "::-webkit-meter-optimum-value",
        paint: DescPaint(&[
            ("background", "var(--success)"),
            ("border-radius", "var(--shape-pill)"),
        ]),
    },
    DescRule {
        sel: "::-webkit-meter-suboptimum-value",
        paint: DescPaint(&[
            ("background", "var(--warning)"),
            ("border-radius", "var(--shape-pill)"),
        ]),
    },
    DescRule {
        sel: "::-webkit-meter-even-less-good-value",
        paint: DescPaint(&[
            ("background", "var(--destructive)"),
            ("border-radius", "var(--shape-pill)"),
        ]),
    },
    // Firefox: fill bar + optimum bands (vendor pseudo-classes).
    DescRule {
        sel: "::-moz-meter-bar",
        paint: DescPaint(&[("border-radius", "var(--shape-pill)")]),
    },
    DescRule {
        sel: ":-moz-meter-optimum::-moz-meter-bar",
        paint: DescPaint(&[("background", "var(--success)")]),
    },
    DescRule {
        sel: ":-moz-meter-sub-optimum::-moz-meter-bar",
        paint: DescPaint(&[("background", "var(--warning)")]),
    },
    DescRule {
        sel: ":-moz-meter-sub-sub-optimum::-moz-meter-bar",
        paint: DescPaint(&[("background", "var(--destructive)")]),
    },
];

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
            ("transition", "color var(--duration-fast) var(--ease-out)"),
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
    DescRule {
        sel: "h3",
        paint: DescPaint(&[("font-size", "var(--text-lg)")]),
    },
    DescRule {
        sel: "h4, h5, h6",
        paint: DescPaint(&[("font-size", "var(--text-md)")]),
    },
    DescRule {
        sel: "p",
        paint: DescPaint(&[("margin-block", "var(--space-3)")]),
    },
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
            ("line-height", "var(--leading-loose)"),
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
    DescRule {
        sel: "li",
        paint: DescPaint(&[("margin-block", "var(--space-1)")]),
    },
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
    DescRule {
        sel: ".tok-tag",
        paint: DescPaint(&[("color", "var(--syntax-tag)")]),
    },
    DescRule {
        sel: ".tok-attr",
        paint: DescPaint(&[("color", "var(--syntax-attr)")]),
    },
    DescRule {
        sel: ".tok-string",
        paint: DescPaint(&[("color", "var(--syntax-string)")]),
    },
    DescRule {
        sel: ".tok-comment",
        paint: DescPaint(&[("color", "var(--syntax-comment)"), ("font-style", "italic")]),
    },
    DescRule {
        sel: ".tok-keyword",
        paint: DescPaint(&[("color", "var(--syntax-keyword)")]),
    },
    DescRule {
        sel: ".tok-fn",
        paint: DescPaint(&[("color", "var(--syntax-function)")]),
    },
    DescRule {
        sel: ".tok-number",
        paint: DescPaint(&[("color", "var(--syntax-number)")]),
    },
    DescRule {
        sel: ".tok-bind",
        paint: DescPaint(&[("color", "var(--syntax-function)")]),
    },
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
            panic!(
                "descendant skin for {host} selector `{}` lowered empty",
                rule.sel
            );
        }
        // Empty sel = host self. Leading `:` / `[` = compound on host. Else descendant.
        // Comma-split is paren/bracket-aware so `:has(a, b)` and `:is(.x, .y)` stay whole.
        let sels: Vec<String> = if rule.sel.trim().is_empty() {
            vec![host.to_string()]
        } else {
            split_selector_list(rule.sel)
                .into_iter()
                .map(|s| {
                    if s.starts_with(':') || s.starts_with('[') {
                        format!("{host}{s}")
                    } else {
                        format!("{host} {s}")
                    }
                })
                .collect()
        };
        out.push_str(&sels.join(", "));
        out.push_str(" { ");
        out.push_str(&body);
        out.push_str(" }\n");
    }
    out
}

/// Split a CSS selector list on top-level commas only.
/// Nested commas inside `()`, `[]` (e.g. `:has(> a, > b)`) are preserved.
fn split_selector_list(sel: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut depth: i32 = 0;
    for (i, c) in sel.char_indices() {
        match c {
            '(' | '[' => depth += 1,
            ')' | ']' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                let part = sel[start..i].trim();
                if !part.is_empty() {
                    parts.push(part);
                }
                start = i + 1;
            }
            _ => {}
        }
    }
    let tail = sel[start..].trim();
    if !tail.is_empty() {
        parts.push(tail);
    }
    parts
}
