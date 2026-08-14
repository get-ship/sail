//! Residual layout-atom refusals — not an author atlas, not an emit table.
//!
//! Style emit is `vocab.rs` + `plains.rs` / `concerns.rs` + `scales.rs`.
//! Utility-string atoms (`flex`, `p-4`) are refused at parse.

/// Residual utility-string leftovers. Not Style vocab.
pub fn refusals() -> Vec<(String, String)> {
    REFUSALS
        .iter()
        .map(|(a, i)| ((*a).to_string(), (*i).to_string()))
        .collect()
}

const REFUSALS: &[(&str, &str)] = &[
    (
        "flex",
        "<row> — a layout context is a layout element",
    ),
    (
        "inline-flex",
        "<row> — a layout context is a layout element",
    ),
    (
        "grid",
        "<grid> — a layout context is a layout element",
    ),
    (
        "inline-grid",
        "<grid> — a layout context is a layout element",
    ),
    ("flex-row", "<row>"),
    ("flex-col", "<column>"),
    ("flex-wrap", "<row> — a row wraps by default"),
    ("flex-nowrap", "<row nowrap> — the single-line row"),
    (
        "items-center",
        "<row align=\"center\"> — the parent decides arrangement",
    ),
    ("items-start", "<row align=\"start\">"),
    ("items-end", "<row align=\"end\">"),
    ("items-stretch", "<row align=\"stretch\">"),
    ("justify-center", "<row justify=\"center\">"),
    ("justify-between", "<row justify=\"between\">"),
    ("justify-start", "<row justify=\"start\">"),
    ("justify-end", "<row justify=\"end\">"),
    ("justify-around", "<row justify=\"around\">"),
    (
        "gap",
        "layout element gap prop: <row gap=\"2\"> — or host style({ gap(2) })",
    ),
    (
        "gap-x",
        "layout element gap prop: <row gap=\"2\"> — or host style({ gapX(2) })",
    ),
    (
        "gap-y",
        "layout element gap prop: <column gap=\"2\"> — or host style({ gapY(2) })",
    ),
    ("grid-cols", "<grid cols=\"3\">"),
    ("grid-rows", "<grid rows=\"3\"> — an explicit row count"),
    (
        "animate",
        "style({ animate: { loop: play(…), enter: play(…) } })",
    ),
    ("sr-only-focusable", "sr-only"),
];
