//! Style value language — shipped parse/emit + product entry + badge proving ground.

use sail_compiler::compile_element;
use sail_compiler::parse::preprocess::preprocess;
use sail_lint::style::{StyleDiagnostic, clash, lower_style, parse};
use std::collections::HashMap;


fn is_style_product(src: &str) -> bool {
    src.contains("style.") || src.contains("style(")
}

#[test]
fn chain_emits_token_vars_on_product_path() {
    let out = preprocess(r#"<div style({ box: padX(2), paint: bg(primary) })> x</div>"#).expect("preprocess");
    assert!(
        out.contains("var(--primary)") || out.contains("sx-bgprimary"),
        "chain must emit token paint:\n{out}"
    );
    assert!(
        out.contains("var(--space-2)") || out.contains("padding"),
        "padX must emit space token:\n{out}"
    );
}

#[test]
fn match_emits_attribute_selectors() {
    let out = preprocess(
        r#"<div style({ match: { variant: { solid: { paint: bg(primary) }, soft: { paint: bg(muted) } } } })> x</div>"#,
    )
    .expect("preprocess");
    assert!(out.contains("[variant='solid']"), "{out}");
    assert!(out.contains("[variant='soft']"), "{out}");
}

#[test]
fn when_emits_hover_not_events() {
    let out =
        preprocess(r#"<div style({ when: { hover: { paint: bg(muted) } } })> x</div>"#).expect("preprocess");
    assert!(out.contains(":hover"), "{out}");
    assert!(!out.contains("onclick"), "{out}");
}

#[test]
fn compound_emits_intersection() {
    let out = preprocess(r#"<div style({ compound: [ { pill: true, paint: radius(full) } ] })> x</div>"#)
        .expect("preprocess");
    assert!(out.contains("[pill]"), "{out}");
}

#[test]
fn utility_string_product_errors_at_parse() {
    let (_, diags) = parse("bg-primary px-2");
    assert!(
        diags
            .iter()
            .any(|d| matches!(d, StyleDiagnostic::ResidualProduct { .. })),
        "utility string must be residual product error"
    );
}

#[test]
fn clash_fail_closed() {
    let (style, _) = parse("style({ paint: bg(primary).bg(muted) })");
    let diags = clash::detect_clashes(&style, &HashMap::new());
    assert!(
        diags
            .iter()
            .any(|d| matches!(d, StyleDiagnostic::Clash { .. }))
    );
}

#[test]
fn cross_setter_chain_clash_on_shared_slot() {
    // Slot-keyed detection narrows per call (op_slots): `border(1)` writes
    // the `border-width` paint slot, the same slot `borderWidth(2)` writes —
    // a cross-setter Chain clash, detected without any lowering.
    let (style, _) = parse("style({ paint: border(1).borderWidth(2) })");
    let diags = clash::detect_clashes(&style, &HashMap::new());
    assert!(
        diags.iter().any(|d| matches!(d, StyleDiagnostic::Clash { slot, .. } if slot.starts_with("border-width@"))),
        "border(1) vs borderWidth must clash on border-width: {diags:?}"
    );
}

#[test]
fn cross_setter_disjoint_slots_do_not_clash() {
    // A color-token `border` narrows to border-color only, so it composes
    // with `borderWidth` — the stdlib focus/border idiom stays clash-free.
    let (style, _) = parse("style({ paint: borderWidth(1).border(primary) })");
    let diags = clash::detect_clashes(&style, &HashMap::new());
    assert!(
        !diags
            .iter()
            .any(|d| matches!(d, StyleDiagnostic::Clash { .. })),
        "disjoint per-call slots must not clash: {diags:?}"
    );
}

#[test]
fn list_shadows_silently_no_clash() {
    // List = ordered composition: later items shadow earlier ones.
    let (style, _) = parse("style({ paint: bg(primary) }, { paint: bg(muted) })");
    let diags = clash::detect_clashes(&style, &HashMap::new());
    assert!(
        !diags
            .iter()
            .any(|d| matches!(d, StyleDiagnostic::Clash { .. })),
        "list items must shadow, not clash: {diags:?}"
    );
}

#[test]
fn list_shadowed_rule_emitted_after_earlier_rule() {
    // Stylesheet order implements the precedence: the shadowing (later) rule
    // lands after the shadowed (earlier) rule in the emitted CSS text.
    let (style, diags) = parse("style({ paint: bg(primary) }, { paint: bg(muted) })");
    assert!(
        diags.is_empty(),
        "{:?}",
        diags.iter().map(|d| d.message()).collect::<Vec<_>>()
    );
    let (_, css) = lower_style(&style, &HashMap::new());
    let first = css.find("var(--primary)").expect("earlier bg rule emitted");
    let second = css.find("var(--muted)").expect("later bg rule emitted");
    assert!(
        second > first,
        "later list item must be emitted after the earlier one:\n{css}"
    );
}

#[test]
fn token_fail_closed() {
    let (_, diags) = parse("style({ paint: bg(notARealTokenZzz999) })");
    assert!(
        diags
            .iter()
            .any(|d| matches!(d, StyleDiagnostic::BadToken { .. }))
    );
}

#[test]
fn badge_proving_ground_no_cva_no_tone_star() {
    let src = include_str!("../stdlib/badge.sail");
    assert!(
        !src.contains("soft: '")
            && !src.contains("solid: '")
            && !src.contains("md: '")
            && !src.contains("compound({"),
        "badge must not use CVA string maps or scope compound arrays"
    );
    assert!(
        !src.contains("tone-bg")
            && !src.contains("tone-fg")
            && !src.contains("tone-border")
            && !src.contains("tone-muted"),
        "badge must not use tone-* product paint"
    );
    assert!(
        is_style_product(src) && src.contains("match:") && src.contains("when:"),
        "badge must use Style value language"
    );
}

#[test]
fn badge_compiles_and_emits_token_css() {
    let src = include_str!("../stdlib/badge.sail");
    let out = compile_element(src, "badge").expect("badge compiles on Style path");
    assert!(!out.structure_html.is_empty(), "structure present");
    let paint = format!("{}{}", out.structure_html, out.skin_css);
    assert!(
        paint.contains("var(--") || paint.contains("sx-"),
        "token/class paint present:\nstruct={}\nskin={}",
        out.structure_html,
        out.skin_css
    );
}

#[test]
fn card_proving_ground_no_cva() {
    let src = include_str!("../stdlib/card.sail");
    assert!(
        !src.contains("default: '") && !src.contains("md: '") && !src.contains("compound({"),
        "card must not use CVA string maps or scope compound arrays"
    );
    assert!(
        is_style_product(src) && src.contains("match:") && src.contains("when:"),
        "card must use Style value language"
    );
    // Component token cssNames stay live (orch component packs; D1 dotted form).
    assert!(
        src.contains("card")
            && (src.contains("card.foreground") || src.contains("card-foreground")),
        "card must paint via component.card tokens"
    );
}

#[test]
fn card_compiles_and_emits_token_css() {
    let src = include_str!("../stdlib/card.sail");
    let out = compile_element(src, "card").expect("card compiles on Style path");
    let paint = format!("{}{}", out.structure_html, out.skin_css);
    assert!(
        paint.contains("var(--") || paint.contains("sx-"),
        "token/class paint present:\nstruct={}\nskin={}",
        out.structure_html,
        out.skin_css
    );
}

#[test]
fn list_and_bind_parse() {
    let (style, diags) =
        parse("style({ paint: bg(primary) }, { paint: translateY(bind(rowY)) })");
    assert!(
        diags.is_empty(),
        "{:?}",
        diags.iter().map(|d| d.message()).collect::<Vec<_>>()
    );
    let (_, css) = lower_style(&style, &HashMap::new());
    assert!(css.contains("var(--"), "{css}");
}

#[test]
fn scope_style_const_resolves_in_host_sx() {
    let out = preprocess(
        r#"
<scope>
  const filled = style({ box: padX(2), paint: bg(primary) })
</scope>
<div style(filled)> x</div>
"#,
    )
    .expect("preprocess");
    assert!(
        out.contains("var(--primary)") || out.contains("sx-bgprimary"),
        "scope Style const must resolve via StyleEnv:\n{out}"
    );
    assert!(
        out.contains("var(--space-2)") || out.contains("padding") || out.contains("sx-padx"),
        "const chain setters must emit:\n{out}"
    );
}

#[test]
fn scope_style_decl_resolves_in_host_sx() {
    let out = preprocess(
        r#"
<scope>
  style host = { box: padX(2), paint: bg(primary) }
</scope>
<div style(host)> x</div>
"#,
    )
    .expect("preprocess");
    assert!(
        out.contains("var(--primary)") || out.contains("sx-bgprimary"),
        "style name = must resolve via StyleEnv:\n{out}"
    );
    assert!(
        out.contains("var(--space-2)") || out.contains("padding") || out.contains("sx-padx"),
        "style decl setters must emit:\n{out}"
    );
}

#[test]
fn scope_style_decl_compose_merges() {
    let out = preprocess(
        r#"
<scope>
  style a = { box: pad(1), paint: bg(primary) }
  style b = a, { box: pad(2) }
</scope>
<div style(b)> x</div>
"#,
    )
    .expect("preprocess");
    // pad(2) should win/merge from compose partial; primary paint from base.
    assert!(
        out.contains("var(--primary)") || out.contains("sx-bgprimary"),
        "compose base paint must resolve:\n{out}"
    );
    assert!(
        out.contains("var(--space-2)")
            || out.contains("padding")
            || out.contains("sx-pad")
            || out.contains("space-2"),
        "compose partial box must emit:\n{out}"
    );
}

#[test]
fn scope_style_const_in_match_arms() {
    let out = preprocess(
        r#"
<scope>
  const solid = style({ paint: bg(primary) })
  const soft = style({ paint: bg(muted) })
</scope>
<div style({ match: { variant: { solid: solid, soft: soft } } })> x</div>
"#,
    )
    .expect("preprocess");
    assert!(out.contains("[variant='solid']"), "{out}");
    assert!(out.contains("[variant='soft']"), "{out}");
    assert!(
        out.contains("var(--primary)") || out.contains("sx-"),
        "match arm refs must resolve:\n{out}"
    );
}

#[test]
fn badge_uses_scope_style_decls() {
    let src = include_str!("../stdlib/badge.sail");
    assert!(
        src.contains("style base =") && src.contains("style soft ="),
        "badge proving ground should declare scope Style with style name ="
    );
    assert!(
        src.contains("soft: style(soft") || src.contains("soft: soft") || src.contains("style soft ="),
        "badge match arms should ref Style decls"
    );
}

#[test]
fn batch_a_chrome_compile_on_style_path() {
    for (tag, src) in [
        ("spinner", include_str!("../stdlib/spinner.sail")),
        ("separator", include_str!("../stdlib/separator.sail")),
        ("avatar", include_str!("../stdlib/avatar.sail")),
        ("tooltip", include_str!("../stdlib/tooltip.sail")),
        ("toast", include_str!("../stdlib/toast.sail")),
        ("toast-region", include_str!("../stdlib/toast-region.sail")),
    ] {
        assert!(
            !src.contains("md: '")
                && !src.contains("compound({")
                && !src.contains("tone-fg")
                && !src.contains("tone-bg")
                && is_style_product(src)
                && (src.contains("match:") || src.contains("when:") || src.contains("compound:")),
            "{tag} must use Style product (no CVA string maps / tone-* paint):\n{src}"
        );
        let out = compile_element(src, tag).unwrap_or_else(|e| panic!("{tag} compile: {e}"));
        let paint = format!("{}{}", out.structure_html, out.skin_css);
        assert!(
            paint.contains("var(--") || paint.contains("sx-") || paint.contains("sail-grp"),
            "{tag} must emit token/class paint:\nstruct={}\nskin={}",
            out.structure_html,
            out.skin_css
        );
    }
}

#[test]
fn batch_b_controls_compile_on_style_path() {
    for (tag, src) in [
        ("label", include_str!("../stdlib/label.sail")),
        ("field", include_str!("../stdlib/field.sail")),
        ("switch", include_str!("../stdlib/switch.sail")),
        ("toggle", include_str!("../stdlib/toggle.sail")),
        ("tab", include_str!("../stdlib/tab.sail")),
        ("tab-list", include_str!("../stdlib/tab-list.sail")),
    ] {
        assert!(
            !src.contains("md: '")
                && !src.contains("compound({")
                && is_style_product(src)
                && (src.contains("match:") || src.contains("when:")),
            "{tag} must use Style product (no CVA string maps):\n{src}"
        );
        let out = compile_element(src, tag).unwrap_or_else(|e| panic!("{tag} compile: {e}"));
        let paint = format!("{}{}", out.structure_html, out.skin_css);
        assert!(
            paint.contains("var(--") || paint.contains("sx-") || paint.contains("sail-grp"),
            "{tag} must emit token/class paint:\nstruct={}\nskin={}",
            out.structure_html,
            out.skin_css
        );
    }
}

#[test]
fn batch_c_surfaces_compile_on_style_path() {
    for (tag, src) in [
        ("alert", include_str!("../stdlib/alert.sail")),
        ("chip", include_str!("../stdlib/chip.sail")),
        ("empty-state", include_str!("../stdlib/empty-state.sail")),
        ("error-state", include_str!("../stdlib/error-state.sail")),
        ("hover-card", include_str!("../stdlib/hover-card.sail")),
        ("popover", include_str!("../stdlib/popover.sail")),
    ] {
        assert!(
            !src.contains("md: '")
                && !src.contains("compound({")
                && !src.contains("tone-bg")
                && !src.contains("tone-fg")
                && !src.contains("tone-border")
                && !src.contains("tone-contrast")
                && is_style_product(src)
                && (src.contains("match:")
                    || src.contains("when:")
                    || src.contains("compound:")
                    || src.contains("style(")
                    || is_style_product(src)),
            "{tag} must use Style product (no CVA string maps / tone-* paint):\n{src}"
        );
        let out = compile_element(src, tag).unwrap_or_else(|e| panic!("{tag} compile: {e}"));
        let paint = format!("{}{}", out.structure_html, out.skin_css);
        assert!(
            paint.contains("var(--") || paint.contains("sx-") || paint.contains("sail-grp"),
            "{tag} must emit token/class paint:\nstruct={}\nskin={}",
            out.structure_html,
            out.skin_css
        );
        assert!(
            !out.skin_css.contains("var(--tone-bg)")
                && !out.skin_css.contains("var(--tone-fg)")
                && !out.skin_css.contains("var(--tone-border)")
                && !out.skin_css.contains("var(--tone-contrast)"),
            "{tag} must not emit tone channel vars:\n{}",
            out.skin_css
        );
    }
}

#[test]
fn batch_d_chrome_menus_compile_on_style_path() {
    for (tag, src) in [
        ("dropdown", include_str!("../stdlib/dropdown.sail")),
        ("context-menu", include_str!("../stdlib/context-menu.sail")),
        ("menu-item", include_str!("../stdlib/menu-item.sail")),
        ("combobox", include_str!("../stdlib/combobox.sail")),
        ("navbar", include_str!("../stdlib/navbar.sail")),
        ("page", include_str!("../stdlib/page.sail")),
        ("prose", include_str!("../stdlib/prose.sail")),
        ("code-block", include_str!("../stdlib/code-block.sail")),
        ("radio", include_str!("../stdlib/radio.sail")),
        ("radio-group", include_str!("../stdlib/radio-group.sail")),
        ("scroll-area", include_str!("../stdlib/scroll-area.sail")),
    ] {
        assert!(
            !src.contains("md: '")
                && !src.contains("compound({")
                && !src.contains("sx=\"")
                && is_style_product(src),
            "{tag} must use Style product (no CVA string maps / string sx):\n{src}"
        );
        let out = compile_element(src, tag).unwrap_or_else(|e| panic!("{tag} compile: {e}"));
        let paint = format!("{}{}", out.structure_html, out.skin_css);
        assert!(
            paint.contains("var(--") || paint.contains("sx-") || paint.contains("sail-grp"),
            "{tag} must emit token/class paint:\nstruct={}\nskin={}",
            out.structure_html,
            out.skin_css
        );
    }
}

#[test]
fn batch_e_residuals_compile_on_style_path() {
    // Structure-only (no host paint): pagination, slider — excluded from paint asserts.
    for (tag, src) in [
        ("breadcrumb", include_str!("../stdlib/breadcrumb.sail")),
        ("accordion", include_str!("../stdlib/accordion.sail")),
        ("kbd", include_str!("../stdlib/kbd.sail")),
        ("skeleton", include_str!("../stdlib/skeleton.sail")),
        (
            "error-boundary",
            include_str!("../stdlib/error-boundary.sail"),
        ),
        ("icon", include_str!("../stdlib/icon.sail")),
        ("tab-panel", include_str!("../stdlib/tab-panel.sail")),
        ("tabs", include_str!("../stdlib/tabs.sail")),
        ("theme-picker", include_str!("../stdlib/theme-picker.sail")),
        ("ticker", include_str!("../stdlib/ticker.sail")),
    ] {
        assert!(
            !src.contains("md: '")
                && !src.contains("compound({")
                && !src.contains("sx=\"")
                && !src.contains("tone-bg")
                && !src.contains("tone-fg")
                && is_style_product(src),
            "{tag} must use Style product (no CVA string maps / string sx / tone-*):\n{src}"
        );
        let out = compile_element(src, tag).unwrap_or_else(|e| panic!("{tag} compile: {e}"));
        let paint = format!("{}{}", out.structure_html, out.skin_css);
        assert!(
            paint.contains("var(--") || paint.contains("sx-") || paint.contains("sail-grp"),
            "{tag} must emit token/class paint:\nstruct={}\nskin={}",
            out.structure_html,
            out.skin_css
        );
        assert!(
            !out.skin_css.contains("var(--tone-bg)")
                && !out.skin_css.contains("var(--tone-fg)")
                && !out.skin_css.contains("var(--tone-border)"),
            "{tag} must not emit tone channel vars:\n{}",
            out.skin_css
        );
    }
    // Pure structure residuals: no host Style paint required.
    for (tag, src) in [
        ("pagination", include_str!("../stdlib/pagination.sail")),
        ("slider", include_str!("../stdlib/slider.sail")),
    ] {
        assert!(
            !src.contains("sx=\"") && !src.contains("md: '") && !src.contains("compound({"),
            "{tag} residual structure must not carry utility-string sx / CVA maps:\n{src}"
        );
        let _ = compile_element(src, tag).unwrap_or_else(|e| panic!("{tag} compile: {e}"));
    }
}
