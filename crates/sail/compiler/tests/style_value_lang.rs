//! Style value language — shipped parse/emit + product entry + badge proving ground.

use sail_compiler::compile_element;
use sail_compiler::parse::preprocess::preprocess;
use sail_lint::style::{clash, lower_style, parse, StyleDiagnostic};
use std::collections::HashMap;

#[test]
fn chain_emits_token_vars_on_product_path() {
    let out = preprocess(r#"<div sx={style.bg(primary).padX(2)}>x</div>"#);
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
        r#"<div sx={match(variant, { solid: style.bg(primary), soft: style.bg(muted) })}>x</div>"#,
    );
    assert!(out.contains("[variant='solid']"), "{out}");
    assert!(out.contains("[variant='soft']"), "{out}");
}

#[test]
fn when_emits_hover_not_events() {
    let out = preprocess(r#"<div sx={when({ hover: style.bg(muted) })}>x</div>"#);
    assert!(out.contains(":hover"), "{out}");
    assert!(!out.contains("onclick"), "{out}");
}

#[test]
fn compound_emits_intersection() {
    let out = preprocess(r#"<div sx={compound({ pill: true }, style.radius(full))}>x</div>"#);
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
    let (style, _) = parse("style.bg(primary).bg(muted)");
    let diags = clash::detect_clashes(&style, &HashMap::new());
    assert!(diags.iter().any(|d| matches!(d, StyleDiagnostic::Clash { .. })));
}

#[test]
fn token_fail_closed() {
    let (_, diags) = parse("style.bg(notARealTokenZzz999)");
    assert!(diags.iter().any(|d| matches!(d, StyleDiagnostic::BadToken { .. })));
}

#[test]
fn badge_proving_ground_no_cva_no_tone_star() {
    let src = include_str!("../stdlib/badge.sail");
    assert!(
        !src.contains("soft: '")
            && !src.contains("solid: '")
            && !src.contains("md: '")
            && !src.contains("compound: ["),
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
        src.contains("style.") && src.contains("match(") && src.contains("when("),
        "badge must use Style value language"
    );
}

#[test]
fn badge_compiles_and_emits_token_css() {
    let src = include_str!("../stdlib/badge.sail");
    let out = compile_element(src, "badge").expect("badge compiles on Style path");
    assert!(
        !out.structure_html.is_empty(),
        "structure present"
    );
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
        !src.contains("default: '")
            && !src.contains("md: '")
            && !src.contains("compound: ["),
        "card must not use CVA string maps or scope compound arrays"
    );
    assert!(
        src.contains("style.") && src.contains("match(") && src.contains("when("),
        "card must use Style value language"
    );
    // Component token cssNames stay live (orch component packs).
    assert!(
        src.contains("card") && src.contains("card-foreground"),
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
    let (style, diags) = parse("[ style.bg(primary), style.translateY(bind(rowY)) ]");
    assert!(diags.is_empty(), "{:?}", diags.iter().map(|d| d.message()).collect::<Vec<_>>());
    let (_, css) = lower_style(&style, &HashMap::new());
    assert!(css.contains("var(--"), "{css}");
}

#[test]
fn scope_style_const_resolves_in_host_sx() {
    let out = preprocess(
        r#"
<scope>
  const filled = style.bg(primary).padX(2)
</scope>
<div sx={filled}>x</div>
"#,
    );
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
fn scope_style_const_in_match_arms() {
    let out = preprocess(
        r#"
<scope>
  const solid = style.bg(primary)
  const soft = style.bg(muted)
</scope>
<div sx={match(variant, { solid: solid, soft: soft })}>x</div>
"#,
    );
    assert!(out.contains("[variant='solid']"), "{out}");
    assert!(out.contains("[variant='soft']"), "{out}");
    assert!(
        out.contains("var(--primary)") || out.contains("sx-"),
        "match arm refs must resolve:\n{out}"
    );
}

#[test]
fn badge_uses_scope_style_consts() {
    let src = include_str!("../stdlib/badge.sail");
    assert!(
        src.contains("const base = style.") && src.contains("const soft = style."),
        "badge proving ground should declare scope Style consts"
    );
    assert!(
        src.contains("soft: soft") || src.contains("soft:soft"),
        "badge match arms should ref Style consts"
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
                && !src.contains("compound: [")
                && !src.contains("tone-fg")
                && !src.contains("tone-bg")
                && src.contains("style.")
                && (src.contains("match(") || src.contains("when(") || src.contains("compound(")),
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
                && !src.contains("compound: [")
                && src.contains("style.")
                && (src.contains("match(") || src.contains("when(")),
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


