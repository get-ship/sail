//! Compile-time Style value language: chain + match / when / compound / bind.
//!
//! Sole paint architecture: chain + match / when / compound / bind.

pub mod clash;
pub mod emit;
pub mod ir;
pub mod parse;
pub mod setters;

pub use emit::{emit_style, lower_style, lower_style_source, CssRule, StyleEnv};
pub use ir::{PaintArg, PaintOp, Style, StyleDiagnostic};
pub use parse::{looks_like_utility_string_product, parse, parse_style_const_rhs};

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn chain_emits_token_vars() {
        let (style, diags) = parse("style.bg(primary).padX(2).radius(sm)");
        assert!(diags.is_empty(), "{:?}", diags.iter().map(|d| d.message()).collect::<Vec<_>>());
        let (classes, css) = lower_style(&style, &HashMap::new());
        assert!(classes.contains("sx-"), "{classes}");
        assert!(css.contains("var(--primary)") || css.contains("background"), "{css}");
        assert!(css.contains("var(--space-2)") || css.contains("padding"), "{css}");
        assert!(css.contains("var(--radius-sm)") || css.contains("border-radius"), "{css}");
    }

    #[test]
    fn match_emits_attr_selectors() {
        let (style, diags) = parse(
            "match(variant, { solid: style.bg(primary), soft: style.bg(muted) })",
        );
        assert!(diags.is_empty(), "{:?}", diags.iter().map(|d| d.message()).collect::<Vec<_>>());
        let (_, css) = lower_style(&style, &HashMap::new());
        assert!(css.contains("[variant='solid']"), "{css}");
        assert!(css.contains("[variant='soft']"), "{css}");
    }

    #[test]
    fn when_map_emits_hover_pseudo() {
        let (style, diags) = parse("when({ hover: style.bg(muted) })");
        assert!(diags.is_empty(), "{:?}", diags.iter().map(|d| d.message()).collect::<Vec<_>>());
        let (_, css) = lower_style(&style, &HashMap::new());
        assert!(css.contains(":hover"), "{css}");
        assert!(!css.contains("onclick") && !css.contains("addEventListener"), "{css}");
    }

    /// Style `when` media keys share string-sx CONDITION_PREFIXES (breakpoints).
    #[test]
    fn when_breakpoints_match_string_sx_and_tokens() {
        use crate::rules::sx::{condition_media, CONDITION_PREFIXES};

        for name in ["mobile", "tablet", "desktop"] {
            let query = condition_media(name).expect("string-sx condition exists");
            assert_eq!(
                setters::when_target(name),
                Some(setters::WhenTarget::Media(query)),
                "Style when({{{name}:…}}) must match string-sx `{name}:`"
            );
            let value = crate::tokens::token_value(&format!("breakpoint-{name}"))
                .unwrap_or_else(|| panic!("breakpoint-{name} token missing from registry"));
            assert!(
                query.contains(value.trim()),
                "`{name}` media {query} must embed token breakpoint-{name}={value}"
            );
        }

        // Full condition table — Style must accept every string-sx condition key.
        for (name, query) in CONDITION_PREFIXES {
            assert_eq!(
                setters::when_target(name),
                Some(setters::WhenTarget::Media(*query)),
                "missing Style when parity for condition `{name}`"
            );
        }

        let (style, diags) = parse(
            "style.fontSize(sm).when({ tablet: style.fontSize(lg), desktop: style.fontSize(xl) })",
        );
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let (_, css) = lower_style(&style, &HashMap::new());
        assert!(
            css.contains("@media (min-width: 768px)") || css.contains("@media(min-width: 768px)"),
            "tablet when must emit tablet media: {css}"
        );
        assert!(
            css.contains("@media (min-width: 1280px)") || css.contains("@media(min-width: 1280px)"),
            "desktop when must emit desktop media: {css}"
        );
        // CamelCase alias for multi-word conditions.
        let (style2, diags2) = parse("when({ motionReduce: style.opacity(100) })");
        assert!(diags2.is_empty(), "{:?}", diags2);
        let (_, css2) = lower_style(&style2, &HashMap::new());
        assert!(
            css2.contains("prefers-reduced-motion"),
            "motionReduce camelCase must resolve: {css2}"
        );
    }

    #[test]
    fn compound_emits_multi_attr() {
        let (style, diags) =
            parse("compound({ pill: true }, style.radius(full))");
        assert!(diags.is_empty(), "{:?}", diags.iter().map(|d| d.message()).collect::<Vec<_>>());
        let (_, css) = lower_style(&style, &HashMap::new());
        assert!(css.contains("[pill]"), "{css}");
    }

    #[test]
    fn chain_when_method() {
        let (style, diags) = parse("style.bg(primary).when({ hover: style.bg(muted) })");
        assert!(diags.is_empty(), "{:?}", diags.iter().map(|d| d.message()).collect::<Vec<_>>());
        let (_, css) = lower_style(&style, &HashMap::new());
        assert!(css.contains(":hover"), "{css}");
    }

    #[test]
    fn token_fail_closed() {
        let (_, diags) = parse("style.bg(notARealTokenZzz)");
        assert!(
            diags.iter().any(|d| matches!(d, StyleDiagnostic::BadToken { .. })),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn utility_string_rejected() {
        let (_, diags) = parse("bg-primary px-2");
        assert!(
            diags.iter().any(|d| matches!(d, StyleDiagnostic::ResidualProduct { .. })),
            "utility string must error"
        );
        let (_, diags2) = parse("bg: primary px: 2");
        assert!(
            diags2.iter().any(|d| matches!(d, StyleDiagnostic::ResidualProduct { .. })),
            "colon utility soup must error"
        );
    }

    #[test]
    fn clash_on_duplicate_slot() {
        let (style, _) = parse("style.bg(primary).bg(muted)");
        let diags = clash::detect_clashes(&style, &HashMap::new());
        assert!(
            diags.iter().any(|d| matches!(d, StyleDiagnostic::Clash { .. })),
            "{:?}",
            diags
        );
    }

    #[test]
    fn bind_parses() {
        let (style, diags) = parse("style.translateY(bind(rowY))");
        assert!(diags.is_empty(), "{:?}", diags.iter().map(|d| d.message()).collect::<Vec<_>>());
        let (_, css) = lower_style(&style, &HashMap::new());
        assert!(css.contains("var(--row-y)") || css.contains("translate"), "{css}");
    }

    #[test]
    fn list_composition() {
        let (style, diags) = parse("[ style.bg(primary), when({ hover: style.bg(muted) }) ]");
        assert!(diags.is_empty(), "{:?}", diags.iter().map(|d| d.message()).collect::<Vec<_>>());
        let (_, css) = lower_style(&style, &HashMap::new());
        assert!(css.contains(":hover"), "{css}");
    }

    #[test]
    fn style_const_ref_resolves() {
        let mut env = HashMap::new();
        env.insert("filled".into(), parse("style.bg(primary)").0);
        let (style, diags) = parse("filled");
        assert!(diags.is_empty());
        let (_, css) = lower_style(&style, &env);
        assert!(css.contains("var(--primary)") || css.contains("background"), "{css}");
    }

    #[test]
    fn malformed_when_map_finishes_quickly() {
        let start = std::time::Instant::now();
        let (_style, diags) = parse("when({ hover: ( })");
        assert!(start.elapsed().as_millis() < 200);
        assert!(!diags.is_empty());
    }

    #[test]
    fn malformed_list_finishes_quickly() {
        let start = std::time::Instant::now();
        let (_style, diags) = parse("[ ( , style.bg(primary) ]");
        assert!(start.elapsed().as_millis() < 200);
        let _ = diags;
    }

    #[test]
    fn malformed_chain_finishes_quickly() {
        let start = std::time::Instant::now();
        let (_style, diags) = parse("style.bg(primary).(");
        assert!(start.elapsed().as_millis() < 200);
        let _ = diags;
    }

    #[test]
    fn nested_garbage_finishes_quickly() {
        let start = std::time::Instant::now();
        let input = format!("{}{}", "[[[[", "match(x, {".repeat(40) + "style.bg(primary)");
        let (_style, diags) = parse(&input);
        assert!(start.elapsed().as_millis() < 500);
        let _ = diags;
    }

    #[test]
    fn negative_ms_and_first_when() {
        let (style, diags) = parse(
            "style.ms(-px).when({ first: style.ms(0).radiusL(md), last: style.radiusR(md) })",
        );
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let (_, css) = lower_style(&style, &HashMap::new());
        assert!(css.contains(":first-child") || css.contains("margin"), "{css}");
    }

    #[test]
    fn group_required_when_emits_ancestor() {
        let (style, diags) = parse("when({ group-required: style.display(inline) })");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let (_, css) = lower_style(&style, &HashMap::new());
        assert!(css.contains(".sail-grp[required]"), "{css}");
    }

    #[test]
    fn group_caught_and_group_inline_when_emit_ancestors() {
        let (style, diags) = parse(
            "style.group().when({ group-caught: style.display(none), group-inline: style.display(none) })",
        );
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let (classes, css) = lower_style(&style, &HashMap::new());
        assert!(classes.contains("sail-grp"), "{classes}");
        assert!(css.contains(".sail-grp[data-caught]"), "{css}");
        assert!(css.contains(".sail-grp[inline]"), "{css}");
    }

    #[test]
    fn glyph_fill_and_list_setters_lower() {
        let (style, diags) = parse("style.list(none).glyphFill(true).isolate()");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let (_, css) = lower_style(&style, &HashMap::new());
        assert!(css.contains("list-style-type:none") || css.contains("list-style-type: none"), "{css}");
        assert!(css.contains(" svg") || css.contains("width:100%"), "{css}");
        assert!(css.contains("isolation:isolate") || css.contains("isolation: isolate"), "{css}");
    }

    #[test]
    fn site_shell_setters_and_state_live() {
        let (style, diags) = parse(
            "style.truncate().textCase(uppercase).nums(tabular).origin(bottom).textGradient(primary).bgGlow(accent).when({ stateLive: style.bg(primary), current: style.fg(primary-foreground) })",
        );
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let (_, css) = lower_style(&style, &HashMap::new());
        assert!(css.contains("text-overflow") || css.contains("ellipsis"), "{css}");
        assert!(css.contains("text-transform") || css.contains("uppercase"), "{css}");
        assert!(css.contains("font-variant-numeric") || css.contains("tabular"), "{css}");
        assert!(css.contains("transform-origin") || css.contains("bottom"), "{css}");
        assert!(css.contains("data-state") || css.contains("live"), "{css}");
        assert!(css.contains("aria-current") || css.contains("primary"), "{css}");
    }

    #[test]
    fn site_shell_wave_exprs_parse() {
        let exprs = [
            "style.w(full).minW(0).overflowX(hidden).bg(surface-0)",
            "style.bgGlow(accent).left(0).right(0).top(0).h(64).pointerEvents(none).z(base)",
            "style.fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide).fg(accent)",
            "style.fontSize(display).fontFamily(display).fontWeight(bold).tracking(tighter).leading(none).textAlign(center).fg(foreground)",
            "style.position(absolute).left(-10).right(-10).top(12).rotate(-2).z(popover)",
            "style.borderTWidth(1).borderBWidth(1).borderT(border).borderB(border).bg(surface-2).padY(3)",
            "style.h(full).overflow(hidden).bg(muted).borderWidth(1).border(primary)",
            "style.fontSize(display).fontFamily(display).fontWeight(bold).tracking(tighter).leading(none).textAlign(center).textGradient(primary).select(none).mb(-8)",
            "style.padX(3).padY(1).radius(full).fontSize(sm).fontWeight(semibold).fg(muted-foreground).when({ hover: style.fg(foreground), current: style.bg(primary).fg(primary-foreground) })",
            "style.pad(3).borderWidth(1).border(border).opacity(50).when({ stateLive: style.opacity(100).border(primary).bg(muted) })",
            "style.w(1).radius(sm).bg(primary).origin(bottom)",
            "style.fontFamily(mono).fontSize(sm).nums(tabular).w(8).textAlign(center)",
            "style.fontFamily(mono).fontSize(2xl).nums(tabular)",
            "style.pad(4).w(full).maxW(panel-max-w-sm)",
            "style.w(full).maxW(breakpoint-desktop).mx(auto)",
            "style.fontSize(lg).fg(muted-foreground).leading(relaxed).textAlign(center).maxW(breakpoint-mobile)",
            "style.display(block).underline(none).minW(0)",
            "style.fg(muted-foreground).when({ hover: style.fg(primary) })",
            "style.grow(1).minW(0).truncate()",
            "style.pad(6).grow(1).minW(0)",
        ];
        for e in exprs {
            let (_, diags) = parse(e);
            assert!(
                diags.is_empty(),
                "expr failed: {e}\n{:?}",
                diags.iter().map(|d| d.message()).collect::<Vec<_>>()
            );
        }
    }
}
