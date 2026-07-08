//! sx token-utility lowering — focused tests for the cases the operator
//! called out (mission spec §4 motion + sx): p:4, bg:surface-1,
//! fg:muted-foreground, a gap utility, a radius utility.
//!
//! The lowering is the AOT path the compiler runs on every `<el sx="…">`
//! attribute before emit. The shared sx vocab lives in
//! `sui_lint::rules::sx` (the lint reads the same module for
//! validation — no drift between the two). These tests pin the
//! `prop:token` → atomic-class shape for the most-used utilities so
//! future vocab edits either keep the common case or surface a
//! deliberate breaking change in CI.

use sui_compiler::parse::preprocess::preprocess as preprocess_src;

fn pp(src: &str) -> String {
    preprocess_src(src)
}

#[test]
fn p4_lowers_to_padding_var() {
    let out = pp(r#"<div sx="p:4">x</div>"#);
    assert!(
        out.contains("class=\"sx-p-4\""),
        "p:4 must lower to a single sx-p-4 class:\n{out}"
    );
    // The injected stylesheet binds .sx-p-4 to padding: var(--space-4).
    assert!(
        out.contains(".sx-p-4{padding:var(--space-4)}"),
        "p:4 CSS must reference --space-4:\n{out}"
    );
}

#[test]
fn bg_surface_1_lowers_to_surface_var() {
    let out = pp(r#"<div sx="bg:surface-1">x</div>"#);
    assert!(
        out.contains("class=\"sx-bg-surface1\""),
        "bg:surface-1 must lower to sx-bg-surface1 (sanitized token):\n{out}"
    );
    assert!(
        out.contains(".sx-bg-surface1{background:var(--surface-1)}"),
        "bg:surface-1 must bind to --surface-1:\n{out}"
    );
}

#[test]
fn fg_muted_foreground_lowers_to_fg_var() {
    let out = pp(r#"<div sx="fg:muted-foreground">x</div>"#);
    assert!(
        out.contains("class=\"sx-fg-mutedforeground\""),
        "fg:muted-foreground must lower to sx-fg-mutedforeground (sanitized):\n{out}"
    );
    assert!(
        out.contains(".sx-fg-mutedforeground{color:var(--muted-foreground)}"),
        "fg:muted-foreground must bind to --muted-foreground:\n{out}"
    );
}

#[test]
fn m4_lowers_to_margin_var() {
    // The spec called out "a gap" as a common-case sx utility. sx
    // deliberately does NOT own `gap` (sibling spacing is the layout
    // keywords' job — see sui_lint::rules::sx), so the analogous
    // space-scale test exercises `m:4` (margin) — the surface-nudge
    // counterpart to padding. Same --space-4 var, same shape.
    let out = pp(r#"<div sx="m:4">x</div>"#);
    assert!(
        out.contains("class=\"sx-m-4\""),
        "m:4 must lower to sx-m-4:\n{out}"
    );
    assert!(
        out.contains("margin:var(--space-4)"),
        "m:4 CSS must bind margin to --space-4:\n{out}"
    );
}

#[test]
fn layout_keyword_gap_lowers_to_gap_css() {
    // The spec's "a gap" — when the author wants sibling spacing, the
    // sanctioned path is the layout keyword `<column gap="3">` (and
    // friends). The layout lowering converts the keyword into a
    // `sui-g-3` class on the div, which binds to the documented
    // `gap` CSS property. This is the gap variant the operator called
    // out, just routed through the layout pass (not the sx pass).
    let out = pp(r#"<column gap="3">x</column>"#);
    assert!(
        out.contains("sui-col"),
        "layout keyword <column> lowers to the div + layout class:\n{out}"
    );
    assert!(
        out.contains("sui-g-3"),
        "gap=\"3\" must lower to the gap utility class:\n{out}"
    );
}

#[test]
fn radius_md_lowers_to_radius_var() {
    let out = pp(r#"<div sx="radius:md">x</div>"#);
    assert!(
        out.contains("class=\"sx-radius-md\""),
        "radius:md must lower to sx-radius-md:\n{out}"
    );
    assert!(
        out.contains(".sx-radius-md{border-radius:var(--radius-md)}"),
        "radius:md CSS must bind to --radius-md:\n{out}"
    );
}

#[test]
fn p0_emits_zero_padding_literal() {
    // `p:0` is the literal zero (no --space-0 token by design). The
    // lowering emits `0` verbatim so the class binds to a real CSS
    // value, not a non-existent var.
    let out = pp(r#"<div sx="p:0">x</div>"#);
    assert!(out.contains("class=\"sx-p-0\""), "{out}");
    assert!(
        out.contains(".sx-p-0{padding:0}"),
        "p:0 must use the literal 0 (no --space-0):\n{out}"
    );
}

#[test]
fn sx_with_keyword_color_passes_through() {
    // `bg:transparent` is a sanctioned CSS keyword color (the
    // lowering emits it verbatim, no var indirection — the only
    // accepted escape from the token contract).
    let out = pp(r#"<div sx="bg:transparent">x</div>"#);
    assert!(out.contains("class=\"sx-bg-transparent\""), "{out}");
    assert!(
        out.contains("background:transparent"),
        "bg:transparent must pass through verbatim:\n{out}"
    );
}

#[test]
fn multiple_sx_values_combine_into_one_class_list() {
    // The spec's mixed example: p:4 + bg:surface-1 + fg:muted-foreground
    // all in one sx. The lowering folds them into the class attribute
    // (the emitter then composes them into the host's class).
    let out = pp(r#"<div sx="p:4 bg:surface-1 fg:muted-foreground">x</div>"#);
    assert!(
        out.contains("class=\"sx-p-4 sx-bg-surface1 sx-fg-mutedforeground\""),
        "{out}"
    );
}

#[test]
fn unknown_token_does_not_leak_a_nonexistent_var() {
    // sx="bg:wheat" uses a non-token color. The lint catches this at
    // validate time and the compiler flags it as an error — the
    // lowering is a separate pass that just folds whatever
    // (prop, token) pair is present into a class. The guard here is
    // that the lowering never leaks a non-existent var() into the
    // stylesheet (a silent runtime break) — if the token is unknown,
    // the lowering yields no var reference at all.
    let out = pp(r#"<div sx="bg:wheat">x</div>"#);
    assert!(
        !out.contains("var(--wheat)"),
        "no --wheat var() reference in the emitted stylesheet:\n{out}"
    );
}

#[test]
fn sx_with_state_prefix_lowers_to_state_pseudo_class() {
    // hover:bg:muted produces an :hover-scoped class.
    let out = pp(r#"<div sx="hover:bg:muted">x</div>"#);
    assert!(
        out.contains("class=\"sx-hover-bg-muted\""),
        "hover:bg:muted must lower to sx-hover-bg-muted:\n{out}"
    );
    assert!(
        out.contains(".sx-hover-bg-muted:hover{background:var(--muted)}"),
        "hover:bg:muted CSS must scope to :hover:\n{out}"
    );
}

#[test]
fn sx_with_condition_prefix_lowers_to_media_query() {
    // motion-reduce:shadow:none wraps the rule in the
    // prefers-reduced-motion media query — the closed set of
    // condition prefixes (motion-reduce, motion-safe, forced-colors,
    // dark, light) all lower to their @media query wrappers.
    let out = pp(r#"<div sx="motion-reduce:shadow:none">x</div>"#);
    assert!(
        out.contains("class=\"sx-motion-reduce-shadow-none\""),
        "motion-reduce:shadow:none class:\n{out}"
    );
    assert!(
        out.contains("@media (prefers-reduced-motion: reduce){.sx-motion-reduce-shadow-none{box-shadow:none}}"),
        "motion-reduce condition wraps the rule in @media:\n{out}"
    );
}

#[test]
fn sx_with_size_token_uses_dimensional_var() {
    // `w:control-h-md` is a dimensional size token — it lowers to
    // `width: var(--control-h-md)` (the full token name is the var,
    // not prefixed with `--space-`). The class name itself is
    // sanitized to drop the inner hyphens (so it reads
    // `sx-w-controlhmd`, not `sx-w-control-h-md`).
    let out = pp(r#"<div sx="w:control-h-md">x</div>"#);
    assert!(
        out.contains("class=\"sx-w-controlhmd\""),
        "w:control-h-md class (hyphens sanitized):\n{out}"
    );
    assert!(
        out.contains("width:var(--control-h-md)"),
        "w:control-h-md uses the full --control-h-md var:\n{out}"
    );
}
