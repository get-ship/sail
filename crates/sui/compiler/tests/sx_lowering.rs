//! sx token-utility lowering — focused tests for the cases the operator
//! called out (mission spec §4 motion + sx): p-4, bg-surface-1,
//! text-muted-foreground, a gap utility, a radius utility.
//!
//! The lowering is the AOT path the compiler runs on every `<el sx="…">`
//! attribute before emit. The shared sx vocab lives in
//! `crates/sui/lint/sx-vocabulary.json`, consulted by the lint and the
//! lowering (so they cannot drift). These tests pin the `name-value`
//! → atomic-class shape for the most-used utilities so future vocab
//! edits either keep the common case or surface a deliberate breaking
//! change in CI.
//!
//! New syntax (Tailwind names, our values): an atom is `[prefix:]name-value`,
//! or a bare keyword (`hidden`, `truncate`, `italic`). Several utilities
//! share a name and disambiguate by value, exactly as Tailwind does —
//! `text-sm` is a font size, `text-foreground` is a colour, `text-center`
//! is an alignment.

use sui_compiler::parse::preprocess::preprocess as preprocess_src;

fn pp(src: &str) -> String {
    preprocess_src(src)
}

#[test]
fn p4_lowers_to_padding_var() {
    let out = pp(r#"<div sx="p-4">x</div>"#);
    // The atom `p-4` is sanitized to `p4` (the lowerer strips non-alphanumeric
    // characters when minting the atomic class name). The CSS binds the var.
    assert!(
        out.contains(r#"class="sx-p4""#),
        "p-4 must lower to a single sx-p4 class:\n{out}"
    );
    assert!(
        out.contains(".sx-p4{padding:var(--space-4)}"),
        "p-4 CSS must reference --space-4:\n{out}"
    );
}

#[test]
fn bg_surface_1_lowers_to_surface_var() {
    let out = pp(r#"<div sx="bg-surface-1">x</div>"#);
    assert!(
        out.contains(r#"class="sx-bgsurface1""#),
        "bg-surface-1 must lower to sx-bgsurface1 (sanitized token):\n{out}"
    );
    assert!(
        out.contains(".sx-bgsurface1{background-color:var(--surface-1)}"),
        "bg-surface-1 must bind to --surface-1:\n{out}"
    );
}

#[test]
fn fg_muted_foreground_lowers_to_fg_var() {
    // In the new vocabulary, foreground colour is the `text` utility (which
    // also names font-size and alignment, disambiguated by value). `fg` is
    // retired — the `text-<token>` shape reads cleaner and matches the prior.
    let out = pp(r#"<div sx="text-muted-foreground">x</div>"#);
    assert!(
        out.contains(r#"class="sx-textmutedforeground""#),
        "text-muted-foreground must lower to sx-textmutedforeground (sanitized):\n{out}"
    );
    assert!(
        out.contains(".sx-textmutedforeground{color:var(--muted-foreground)}"),
        "text-muted-foreground must bind to --muted-foreground:\n{out}"
    );
}

#[test]
fn m4_lowers_to_margin_var() {
    // The spec called out "a gap" as a common-case sx utility. sx
    // deliberately does NOT own `gap` (sibling spacing is the layout
    // keywords' job — see sui_lint::rules::sx), so the analogous
    // space-scale test exercises `m-4` (margin) — the surface-nudge
    // counterpart to padding. Same --space-4 var, same shape.
    let out = pp(r#"<div sx="m-4">x</div>"#);
    assert!(
        out.contains(r#"class="sx-m4""#),
        "m-4 must lower to sx-m4:\n{out}"
    );
    assert!(
        out.contains("margin:var(--space-4)"),
        "m-4 CSS must bind margin to --space-4:\n{out}"
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
    let out = pp(r#"<div sx="rounded-md">x</div>"#);
    assert!(
        out.contains(r#"class="sx-roundedmd""#),
        "rounded-md must lower to sx-roundedmd:\n{out}"
    );
    assert!(
        out.contains(".sx-roundedmd{border-radius:var(--radius-md)}"),
        "rounded-md CSS must bind to --radius-md:\n{out}"
    );
}

#[test]
fn p0_emits_zero_padding_literal() {
    // `p-0` is the literal zero (no --space-0 token by design). The
    // lowering emits `0` verbatim so the class binds to a real CSS
    // value, not a non-existent var.
    let out = pp(r#"<div sx="p-0">x</div>"#);
    assert!(out.contains(r#"class="sx-p0""#), "{out}");
    assert!(
        out.contains(".sx-p0{padding:0}"),
        "p-0 must use the literal 0 (no --space-0):\n{out}"
    );
}

#[test]
fn sx_with_keyword_color_passes_through() {
    // `bg-transparent` is a sanctioned CSS keyword color (the
    // lowering emits it verbatim, no var indirection — the only
    // accepted escape from the token contract).
    let out = pp(r#"<div sx="bg-transparent">x</div>"#);
    assert!(
        out.contains("background-color:transparent"),
        "bg-transparent must pass through verbatim:\n{out}"
    );
}

#[test]
fn multiple_sx_values_combine_into_one_class_list() {
    // The spec's mixed example: p-4 + bg-surface-1 + text-muted-foreground
    // all in one sx. The lowering folds them into the class attribute
    // (the emitter then composes them into the host's class).
    let out = pp(r#"<div sx="p-4 bg-surface-1 text-muted-foreground">x</div>"#);
    assert!(
        out.contains(r#"class="sx-p4 sx-bgsurface1 sx-textmutedforeground""#),
        "{out}"
    );
}

#[test]
fn unknown_token_does_not_leak_a_nonexistent_var() {
    // sx="bg-wheat" uses a non-token color. The lint catches this at
    // validate time and the compiler flags it as an error — the
    // lowering is a separate pass that just folds whatever
    // (prop, token) pair is present into a class. The guard here is
    // that the lowering never leaks a non-existent var() into the
    // stylesheet (a silent runtime break) — if the token is unknown,
    // the lowering yields no var reference at all.
    let out = pp(r#"<div sx="bg-wheat">x</div>"#);
    assert!(
        !out.contains("var(--wheat)"),
        "no --wheat var() reference in the emitted stylesheet:\n{out}"
    );
}

#[test]
fn sx_with_state_prefix_lowers_to_state_pseudo_class() {
    // hover:bg-muted produces an :hover-scoped class.
    let out = pp(r#"<div sx="hover:bg-muted">x</div>"#);
    assert!(
        out.contains(r#"class="sx-hover-bgmuted""#),
        "hover:bg-muted must lower to sx-hover-bgmuted:\n{out}"
    );
    assert!(
        out.contains(".sx-hover-bgmuted:hover{background-color:var(--muted)}"),
        "hover:bg-muted CSS must scope to :hover:\n{out}"
    );
}

#[test]
fn sx_with_condition_prefix_lowers_to_media_query() {
    // motion-reduce:shadow-none wraps the rule in the
    // prefers-reduced-motion media query — the closed set of
    // condition prefixes (motion-reduce, motion-safe, forced-colors,
    // dark, light) all lower to their @media query wrappers.
    let out = pp(r#"<div sx="motion-reduce:shadow-none">x</div>"#);
    assert!(
        out.contains(r#"class="sx-motionreduce-shadownone""#),
        "motion-reduce:shadow-none class:\n{out}"
    );
    assert!(
        out.contains("@media (prefers-reduced-motion: reduce){.sx-motionreduce-shadownone{box-shadow:none}}"),
        "motion-reduce condition wraps the rule in @media:\n{out}"
    );
}

#[test]
fn sx_with_size_token_uses_dimensional_var() {
    // `w-control-h-md` is a dimensional size token — it lowers to
    // `width: var(--control-h-md)` (the full token name is the var,
    // not prefixed with `--space-`). The class name itself is
    // sanitized to drop the inner hyphens (so it reads
    // `sx-wcontrolhmd`, not `sx-w-control-h-md`).
    let out = pp(r#"<div sx="w-control-h-md">x</div>"#);
    assert!(
        out.contains(r#"class="sx-wcontrolhmd""#),
        "w-control-h-md class (hyphens sanitized):\n{out}"
    );
    assert!(
        out.contains("width:var(--control-h-md)"),
        "w-control-h-md uses the full --control-h-md var:\n{out}"
    );
}
#[test]
fn layout_keyword_inline_swaps_base_class_for_inline_flex() {
    // The hug-wrap variant. `<row inline>` is the suggestion the sx
    // `inline-flex` refusal emits — it MUST compile cleanly, or the
    // compiler is recommending something unbuildable. This is the half of
    // the contract that lives in the compiler pipeline (the lint half is
    // `layout::tests::lowers_row_inline_to_inline_flex_base_class`).
    //
    // The swap is at the base: the inline variant owns `display:inline-flex`
    // (NOT `display:flex` riding on top of `sui-row`). Both bases share the
    // `min-width:0` child guard and the axis modifier ownership.
    //
    // Note: default axis props (`gap="3" align="center"` for `<row>`) are
    // applied even on a bare `<row inline>` — they are the row's defaults,
    // not the block base's. The inline swap is orthogonal to axis
    // selection; default axes still ride on top of the inline base.
    let out = pp(r#"<row inline>x</row>"#);
    // The lowered div's class string starts with the inline base class,
    // not the block one. We assert on the class string opening rather than
    // the standalone token, because the defaults layer their classes in
    // right after the base.
    assert!(
        out.contains(r#"class="sui-row-i"#),
        "inline base class leads the lowered div:\n{out}"
    );
    assert!(
        !out.contains(r#"class="sui-row""#)
            && !out.contains(r#"class="sui-row "#),
        "no block base leaked into the inline variant:\n{out}"
    );
    assert!(
        out.contains(".sui-row-i{display:inline-flex"),
        "inline-flex rule is the one we ship:\n{out}"
    );
    assert!(
        out.contains(".sui-row-i>*{min-width:0}"),
        "the min-width:0 child guard survives the inline swap:\n{out}"
    );
    // Explicit axes ride on top of the inline base.
    let out = pp(r#"<row inline gap="2">x</row>"#);
    assert!(
        out.contains(r#"sui-row-i"#) && out.contains(r#"sui-g-2"#),
        "axis still applies on inline:\n{out}"
    );
    // Grid mirrors the same swap.
    let out = pp(r#"<grid inline>x</grid>"#);
    assert!(
        out.contains(r#"class="sui-grid-i"#),
        "grid inline base leads the lowered div:\n{out}"
    );
    assert!(
        out.contains(".sui-grid-i{display:inline-grid"),
        "inline-grid rule:\n{out}"
    );
}

#[test]
fn layout_keyword_inline_does_not_emit_inline_attr_on_the_div() {
    // `<row inline>` consumes the `inline` axis at the base swap; it must
    // NOT reappear as a dead HTML attribute on the lowered div. The pass-
    // through list and the consumed axis set share the lint/compiler
    // contract — a regression here would put `inline=""` on every `<row
    // inline>`, the very silent-drop class the layout lint exists to
    // prevent.
    //
    // We scan just the lowered `<div …>` (the markup fragment between the
    // style block close and the source's body) so L1 element skins
    // (`:host { display: inline-flex }`) don't trip the assertion.
    let out = pp(r#"<row inline>x</row>"#);
    let lowered = out
        .split("</style>")
        .nth(1)
        .expect("the preprocess output keeps the </style> marker");
    assert!(
        !lowered.contains(" inline"),
        "no raw `inline` attr on the lowered div: {lowered}"
    );
}
