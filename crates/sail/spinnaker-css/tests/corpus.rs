//! Owned Style → CSS corpus. This is the adapter's proof.
//!
//! Not sail-site. Not stdlib skins. Each case is a Style program the
//! language owns; the adapter must emit exact longhands + token vars.
//! Fail-closed cases must stay refused.

use std::collections::HashMap;

use spinnaker::intent_dump;
use spinnaker::parse::parse;
use spinnaker_css::{emit_style, lower_style};

struct OkCase {
    src: &'static str,
    /// Exact (prop, value) pairs that must appear (order-insensitive).
    decls: &'static [(&'static str, &'static str)],
    /// Substrings that must appear in a selector (e.g. `:hover`).
    selector_needles: &'static [&'static str],
}

struct BadCase {
    src: &'static str,
    /// Diagnostic must mention this.
    needle: &'static str,
}

const OK: &[OkCase] = &[
    OkCase {
        src: "style({ pad(4) })",
        decls: &[("padding", "var(--space-4)")],
        selector_needles: &[],
    },
    OkCase {
        src: "style({ pad({ x: 3, y: 2 }) })",
        decls: &[
            ("padding-inline", "var(--space-3)"),
            ("padding-block", "var(--space-2)"),
        ],
        selector_needles: &[],
    },
    OkCase {
        src: "style({ bg(surface) })",
        decls: &[("background-color", "var(--bg-surface-color)")],
        selector_needles: &[],
    },
    OkCase {
        src: "style({ fg(foreground) })",
        decls: &[("color", "var(--fg-foreground-color)")],
        selector_needles: &[],
    },
    OkCase {
        src: "style({ radius(md) })",
        decls: &[("border-radius", "var(--radius-md)")],
        selector_needles: &[],
    },
    OkCase {
        src: "style({ elevation({ shadow: inset-ring }) })",
        decls: &[("box-shadow", "var(--shadow-inset-ring)")],
        selector_needles: &[],
    },
    OkCase {
        src: "style({ size({ w: 100%, h: 100% }) })",
        decls: &[("width", "100%"), ("height", "100%")],
        selector_needles: &[],
    },
    OkCase {
        src: "style({ display(flex) })",
        decls: &[("display", "flex")],
        selector_needles: &[],
    },
    OkCase {
        src: "style({ position({ scheme: relative }) })",
        decls: &[("position", "relative")],
        selector_needles: &[],
    },
    OkCase {
        src: "style({ type({ size: sm, weight: medium }) })",
        decls: &[
            ("font-size", "var(--text-sm)"),
            ("font-weight", "var(--font-weight-medium)"),
        ],
        selector_needles: &[],
    },
    OkCase {
        src: "style({ when: { hover: bg(muted) } })",
        decls: &[("background-color", "var(--muted)")],
        selector_needles: &[":hover"],
    },
    OkCase {
        src: "style({ pad(4).bg(surface), when: { hover: bg(muted) } })",
        decls: &[
            ("padding", "var(--space-4)"),
            ("background-color", "var(--bg-surface-color)"),
        ],
        selector_needles: &[":hover"],
    },
];

const BAD: &[BadCase] = &[
    BadCase {
        src: "style({ pad(16px) })",
        needle: "16px",
    },
    BadCase {
        src: "style({ bg(#3366ff) })",
        needle: "3366ff",
    },
    BadCase {
        src: "style({ outline(solid) })",
        needle: "outline",
    },
    BadCase {
        src: "style({ w(full) })",
        needle: "size",
    },
    BadCase {
        src: "style.pad(4).bg(surface)",
        needle: "flat",
    },
];

#[test]
fn adapter_corpus_emits_exact_decls() {
    for case in OK {
        let (style, diags) = parse(case.src);
        assert!(
            diags.is_empty(),
            "{}: {:?}",
            case.src,
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let dump = intent_dump(&style);
        assert!(
            !dump.contains("background-color")
                && !dump.contains(":hover")
                && !dump.contains("padding:"),
            "{} intent dump leaked CSS:\n{dump}",
            case.src
        );
        let rules = emit_style(&style, &HashMap::new());
        let got: Vec<(String, String)> = rules
            .iter()
            .flat_map(|r| r.declarations.iter().cloned())
            .collect();
        for (prop, val) in case.decls {
            assert!(
                got.iter().any(|(p, v)| p == prop && v == val),
                "{}: missing {prop}: {val} in {got:?}\nrules={rules:?}",
                case.src
            );
        }
        let sheet = rules
            .iter()
            .map(spinnaker_css::render_rule)
            .collect::<String>();
        for needle in case.selector_needles {
            assert!(
                sheet.contains(needle),
                "{}: selector missing {needle:?} in {sheet}",
                case.src
            );
        }
    }
}

#[test]
fn adapter_corpus_refuses_raw_and_retired() {
    for case in BAD {
        let (_, diags) = parse(case.src);
        assert!(
            diags.iter().any(|d| d.message().contains(case.needle)
                || format!("{d:?}").to_ascii_lowercase().contains(case.needle)),
            "{}: expected diagnostic mentioning {:?}, got {:?}",
            case.src,
            case.needle,
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
    }
}

#[test]
fn adapter_emit_is_intent_then_css_on_corpus() {
    for case in OK {
        let (style, diags) = parse(case.src);
        assert!(diags.is_empty(), "{}", case.src);
        let via_style = emit_style(&style, &HashMap::new());
        let via_intent =
            spinnaker_css::emit_intent(&spinnaker::lower_to_intent(&style), &HashMap::new());
        assert_eq!(via_style, via_intent, "{}", case.src);
        let (_, css) = lower_style(&style, &HashMap::new());
        assert!(
            !css.is_empty() || case.src.contains("group"),
            "{}",
            case.src
        );
    }
}
