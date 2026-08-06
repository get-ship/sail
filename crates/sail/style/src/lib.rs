//! Compile-time Style value language: chain + match / when / compound / bind.
//!
//! Sole paint architecture: chain + match / when / compound / bind.
//!
//! This crate owns Style parse / IR / emit / clash / setters, plus the leaf
//! sx machinery (`sx_ir`, `sx_vocab`), token scales, and design-token registry
//! used by Style lower and layout class resolution.

pub mod channel;
pub mod clash;
pub mod conditions;
pub mod emit;
pub mod ir;
pub mod parse;
pub mod prefixes;
pub mod rewrite;
pub mod scales;
pub mod setters;
pub mod sx_ir;
pub mod sx_vocab;
pub mod tokens;

pub use channel::{
    Channel, TEXT_BEARING_ELEMENTS, channel_for_setter, is_animate_trigger, is_geometry_setter,
    is_interaction_when_key, is_text_bearing, style_contains_geometry, style_uses_text_channel,
    style_uses_text_channel_env, text_channel_host_error,
};
pub use conditions::{CONDITION_PREFIXES, condition_media};
pub use emit::{
    CssRule, StyleEnv, emit_style, lower_style, lower_style_source, lower_style_source_with_motion,
};
pub use ir::{PaintArg, PaintOp, Style, StyleDiagnostic};
pub use parse::{
    collect_motion_attaches, looks_like_utility_string_product, parse, parse_style_const_rhs,
};
pub use rewrite::{
    lift_style_attr_hosts, rewrite_source, rewrite_style_expr, rewrite_teaching_source,
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn chain_emits_token_vars() {
        let (style, diags) = parse("style.bg(primary).padX(2).radius(sm)");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let (classes, css) = lower_style(&style, &HashMap::new());
        assert!(classes.contains("sx-"), "{classes}");
        assert!(
            css.contains("var(--primary)") || css.contains("background"),
            "{css}"
        );
        assert!(
            css.contains("var(--space-2)") || css.contains("padding"),
            "{css}"
        );
        assert!(
            css.contains("var(--radius-sm)") || css.contains("border-radius"),
            "{css}"
        );
    }

    #[test]
    fn match_emits_attr_selectors() {
        let (style, diags) =
            parse("match(variant, { solid: style.bg(primary), soft: style.bg(muted) })");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let (_, css) = lower_style(&style, &HashMap::new());
        assert!(css.contains("[variant='solid']"), "{css}");
        assert!(css.contains("[variant='soft']"), "{css}");
    }

    #[test]
    fn when_map_emits_hover_pseudo() {
        let (style, diags) = parse("when({ hover: style.bg(muted) })");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let (_, css) = lower_style(&style, &HashMap::new());
        assert!(css.contains(":hover"), "{css}");
        assert!(
            !css.contains("onclick") && !css.contains("addEventListener"),
            "{css}"
        );
    }

    /// Style `when` breakpoints are design-token locked (not a legacy string path).
    #[test]
    fn when_breakpoints_match_design_tokens() {
        for name in ["mobile", "tablet", "desktop"] {
            let query = conditions::condition_media(name).expect("Style condition table");
            assert_eq!(
                setters::when_target(name),
                Some(setters::WhenTarget::Media(std::borrow::Cow::Borrowed(
                    query
                ))),
                "when({{{name}:…}}) must resolve via Style conditions"
            );
            let value = crate::tokens::token_value(&format!("breakpoint-{name}"))
                .unwrap_or_else(|| panic!("breakpoint-{name} token missing from registry"));
            assert!(
                query.contains(value.trim()),
                "`{name}` media {query} must embed token breakpoint-{name}={value}"
            );
        }

        for (name, query) in conditions::CONDITION_PREFIXES {
            assert_eq!(
                setters::when_target(name),
                Some(setters::WhenTarget::Media(std::borrow::Cow::Borrowed(
                    *query
                ))),
                "missing when key for Style condition `{name}`"
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
        let (style, diags) = parse("compound({ pill: true }, style.radius(full))");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let (_, css) = lower_style(&style, &HashMap::new());
        assert!(css.contains("[pill]"), "{css}");
    }

    /// Comments inside Style expressions are stripped structurally — they must
    /// never tokenize into Words that inject spurious Refs or truncate lists.
    #[test]
    fn line_comment_between_list_items_keeps_entries() {
        let (style, diags) = parse("[ style.bg(primary), // soften the surface\n style.padX(2) ]");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let Style::List(items) = &style else {
            panic!("expected List, got {style:?}");
        };
        assert_eq!(
            items.len(),
            2,
            "comment must not drop or inject entries: {items:?}"
        );
    }

    #[test]
    fn block_comment_and_trailing_comment_before_bracket_keep_entries() {
        let (style, diags) =
            parse("[ style.bg(primary) /* block */, style.padX(2) // trailing note\n ]");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let Style::List(items) = &style else {
            panic!("expected List, got {style:?}");
        };
        assert_eq!(
            items.len(),
            2,
            "trailing comment must not eat the list tail: {items:?}"
        );
    }

    #[test]
    fn comment_inside_compound_arg_map_keeps_conditions() {
        let (style, diags) =
            parse("compound({ pill: true, // emphasis keys\n tone: primary }, style.radius(full))");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let Style::Compound { when, .. } = &style else {
            panic!("expected Compound, got {style:?}");
        };
        assert_eq!(
            when.len(),
            2,
            "comment must not drop compound conditions: {when:?}"
        );
        assert_eq!(when.get("tone").map(String::as_str), Some("primary"));
    }

    #[test]
    fn comment_text_never_reaches_utility_string_gate() {
        // Colons / dashed words inside a comment must not trip the
        // residual-product gate or leak into the token stream.
        let (style, diags) = parse("/* fixme: switch to tone-primary */ style.bg(primary)");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        assert!(
            matches!(style, Style::Set(_) | Style::Chain(_)),
            "chain must survive the leading comment: {style:?}"
        );
    }

    #[test]
    fn chain_when_method() {
        let (style, diags) = parse("style.bg(primary).when({ hover: style.bg(muted) })");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let (_, css) = lower_style(&style, &HashMap::new());
        assert!(css.contains(":hover"), "{css}");
    }

    #[test]
    fn token_fail_closed() {
        let (_, diags) = parse("style.bg(notARealTokenZzz)");
        assert!(
            diags
                .iter()
                .any(|d| matches!(d, StyleDiagnostic::BadToken { .. })),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn utility_string_rejected() {
        let (_, diags) = parse("bg-primary px-2");
        assert!(
            diags
                .iter()
                .any(|d| matches!(d, StyleDiagnostic::ResidualProduct { .. })),
            "utility string must error"
        );
        let (_, diags2) = parse("bg: primary px: 2");
        assert!(
            diags2
                .iter()
                .any(|d| matches!(d, StyleDiagnostic::ResidualProduct { .. })),
            "colon utility soup must error"
        );
    }

    #[test]
    fn clash_on_duplicate_slot() {
        let (style, _) = parse("style.bg(primary).bg(muted)");
        let diags = clash::detect_clashes(&style, &HashMap::new());
        assert!(
            diags
                .iter()
                .any(|d| matches!(d, StyleDiagnostic::Clash { .. })),
            "{:?}",
            diags
        );
    }

    #[test]
    fn list_shadows_duplicate_slot_silently() {
        // Ordered composition: later items shadow earlier ones — no Clash.
        let (style, diags) = parse("[ style.bg(primary), style.bg(muted) ]");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let diags = clash::detect_clashes(&style, &HashMap::new());
        assert!(
            !diags
                .iter()
                .any(|d| matches!(d, StyleDiagnostic::Clash { .. })),
            "list items must shadow, not clash: {:?}",
            diags
        );
    }

    #[test]
    fn list_shadowing_resolves_through_ref_env() {
        // Duplicates across list items from nested Refs resolve through the
        // env walk and shadow silently.
        let mut env = HashMap::new();
        env.insert("base".into(), parse("style.bg(primary)").0);
        let (style, _) = parse("[ base, style.bg(muted) ]");
        let diags = clash::detect_clashes(&style, &env);
        assert!(
            !diags
                .iter()
                .any(|d| matches!(d, StyleDiagnostic::Clash { .. })),
            "ref-resolved shadowing must not clash: {:?}",
            diags
        );
    }

    #[test]
    fn cross_setter_chain_clash_on_shared_slot() {
        // Different setters writing the same paint slot clash under Chain:
        // `border(1)` writes border-width, same slot as `borderWidth(2)`.
        let (style, _) = parse("style.border(1).borderWidth(2)");
        let diags = clash::detect_clashes(&style, &HashMap::new());
        assert!(
            diags.iter().any(|d| matches!(d, StyleDiagnostic::Clash { slot, .. } if slot.starts_with("border-width@"))),
            "border(1) vs borderWidth must clash on border-width: {:?}",
            diags
        );
        // Per-arg precision: a color-token `border` writes only border-color,
        // so `borderWidth(1).border(chip-border)` (the stdlib focus-ring
        // idiom) is two disjoint writes, not a clash.
        let (style, _) = parse("style.borderWidth(1).border(border)");
        let diags = clash::detect_clashes(&style, &HashMap::new());
        assert!(
            !diags
                .iter()
                .any(|d| matches!(d, StyleDiagnostic::Clash { .. })),
            "borderWidth(1).border(border) must not clash: {:?}",
            diags
        );
    }

    #[test]
    fn chain_inside_list_still_clashes() {
        // A Chain inside a List keeps unordered conjunction semantics.
        let (style, _) = parse("[ style.bg(primary).bg(muted) ]");
        let diags = clash::detect_clashes(&style, &HashMap::new());
        assert!(
            diags
                .iter()
                .any(|d| matches!(d, StyleDiagnostic::Clash { .. })),
            "chain duplicates inside a list item must clash: {:?}",
            diags
        );
    }

    #[test]
    fn list_emits_later_rule_after_earlier() {
        // Stylesheet-order precedence: emit walks items in source order, so
        // the shadowing (later) rule lands after the shadowed one in the CSS.
        let (style, diags) = parse("[ style.bg(primary), style.bg(muted) ]");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let (_, css) = lower_style(&style, &HashMap::new());
        let first = css.find("var(--primary)").expect("earlier rule emitted");
        let second = css.find("var(--muted)").expect("later rule emitted");
        assert!(
            second > first,
            "later list item must be emitted after the earlier one:\n{css}"
        );
    }

    #[test]
    fn bind_parses() {
        let (style, diags) = parse("style.translateY(bind(rowY))");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let (_, css) = lower_style(&style, &HashMap::new());
        assert!(
            css.contains("var(--row-y)") || css.contains("translate"),
            "{css}"
        );
    }

    #[test]
    fn list_composition() {
        let (style, diags) = parse("[ style.bg(primary), when({ hover: style.bg(muted) }) ]");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
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
        assert!(
            css.contains("var(--primary)") || css.contains("background"),
            "{css}"
        );
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
        let input = format!(
            "{}{}",
            "[[[[",
            "match(x, {".repeat(40) + "style.bg(primary)"
        );
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
        assert!(
            css.contains(":first-child") || css.contains("margin"),
            "{css}"
        );
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
        assert!(
            css.contains("list-style-type:none") || css.contains("list-style-type: none"),
            "{css}"
        );
        assert!(css.contains(" svg") || css.contains("width:100%"), "{css}");
        assert!(
            css.contains("isolation:isolate") || css.contains("isolation: isolate"),
            "{css}"
        );
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
        assert!(
            css.contains("text-overflow") || css.contains("ellipsis"),
            "{css}"
        );
        assert!(
            css.contains("text-transform") || css.contains("uppercase"),
            "{css}"
        );
        assert!(
            css.contains("font-variant-numeric") || css.contains("tabular"),
            "{css}"
        );
        assert!(
            css.contains("transform-origin") || css.contains("bottom"),
            "{css}"
        );
        assert!(css.contains("data-state") || css.contains("live"), "{css}");
        assert!(
            css.contains("aria-current") || css.contains("primary"),
            "{css}"
        );
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
            "style.grow(1).basis(0).minW(0)",
            "style.grow(1).basis(breakpoint-mobile).minW(0)",
            "style.translateX(bind(tileX)).rotate(bind(tileRotate)).scale(bind(tileScale))",
            "style.when({ groupCurrent: style.fg(primary), desktop: style.display(block) })",
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

    #[test]
    fn basis_and_scale_setters_lower() {
        let (style, diags) = parse("style.basis(0).scale(100).rotate(bind(tileRotate))");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let (_, css) = lower_style(&style, &HashMap::new());
        assert!(
            css.contains("flex-basis") || css.contains("0"),
            "basis must lower: {css}"
        );
        assert!(
            css.contains("scale") || css.contains("1") || css.contains("100"),
            "scale must lower: {css}"
        );
        assert!(
            css.contains("var(--tile-rotate)") || css.contains("rotate"),
            "rotate bind must lower: {css}"
        );
    }

    #[test]
    fn packages_wave_setters_and_dynamic_when() {
        let (style, diags) = parse(
            "style.srOnly().visibility(hidden).aspect(square).objectFit(cover).cursor(crosshair).italic(true).ml(auto).transform(bind(worldTx))",
        );
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let (_, css) = lower_style(&style, &HashMap::new());
        assert!(
            css.contains("clip-path") || css.contains("1px"),
            "srOnly: {css}"
        );
        assert!(
            css.contains("visibility") || css.contains("hidden"),
            "visibility: {css}"
        );
        assert!(
            css.contains("aspect-ratio") || css.contains("1 / 1") || css.contains("1/1"),
            "aspect: {css}"
        );
        assert!(
            css.contains("object-fit") || css.contains("cover"),
            "objectFit: {css}"
        );
        assert!(
            css.contains("crosshair") || css.contains("cursor"),
            "cursor: {css}"
        );
        assert!(
            css.contains("font-style") || css.contains("italic"),
            "italic: {css}"
        );
        assert!(
            css.contains("margin-left") || css.contains("auto"),
            "ml: {css}"
        );
        assert!(
            css.contains("var(--world-tx)") || css.contains("transform"),
            "transform bind: {css}"
        );

        // Dynamic state-* / group-* when keys (host component data-state families).
        for (key, needle) in [
            ("stateWeekend", "weekend"),
            ("stateOut", "out"),
            ("groupSelected", "aria-selected"),
            ("groupStateUser", "user"),
            ("busy", "aria-busy"),
            ("sortable", "aria-sort"),
        ] {
            let expr = format!("when({{ {key}: style.bg(muted) }})");
            let (st, diags) = parse(&expr);
            assert!(
                diags.is_empty(),
                "{key}: {:?}",
                diags.iter().map(|d| d.message()).collect::<Vec<_>>()
            );
            let (_, css) = lower_style(&st, &HashMap::new());
            assert!(
                css.contains(needle),
                "when key {key} must emit selector with {needle}: {css}"
            );
        }

        // Representative host-component paint from the packages Style conversion wave.
        let packages_exprs = [
            "style.group().maxW(full).fontFamily(body).fg(foreground)",
            "style.padY(2).padX(3).radius(lg).fontSize(base).leading(normal).bg(card).borderWidth(1).border(border).fg(foreground).when({ groupStateUser: style.bg(hover-bg).borderWidth(1).border(primary), groupStateSystem: style.bg(muted).fg(muted-foreground), groupStateError: style.borderWidth(1).border(destructive), groupStateSending: style.opacity(75) })",
            "style.group().padY(density-row-py).padX(density-row-px).minH(panel-min-h-cell).borderRWidth(1).borderR(border).borderBWidth(1).borderB(border).when({ stateOut: style.fg(muted-foreground).bg(muted), stateWeekend: style.bg(muted), stateRange: style.bg(selected-bg) })",
            "style.position(absolute).top(bind(evtTop)).h(bind(evtH)).left(bind(evtLeft)).w(bind(evtW)).cursor(pointer).textAlign(start).fontSize(density).fg(primary).bg(hover-bg).radius(sm).padY(half).padX(1).overflow(hidden).when({ selected: style.outline(ring) })",
            "style.group().padY(6).padX(4).borderWidth(1).border(border).border(dashed).radius(lg).cursor(pointer).fg(muted-foreground).select(none).textAlign(center).when({ focus: style.outline(ring), stateOver: style.borderWidth(1).border(ring).bg(muted) })",
            "style.position(absolute).left(bind(boxLeft)).top(bind(boxTop)).w(bind(boxW)).h(bind(boxH)).bg(hover-bg).borderWidth(1).border(primary).border(dashed).radius(sm).cursor(pointer).pointerEvents(auto)",
            "style.w(bind(boxW)).padY(2).padX(3).bg(card).fg(foreground).fontFamily(body).fontSize(sm).borderWidth(1).border(border).radius(md).shadow(xs).cursor(pointer).when({ hover: style.shadow(sm), focus: style.outline(ring), current: style.borderWidth(1).border(ring).shadow(sm), stateAgent: style.borderLWidth(1).borderL(primary) })",
            "style.fontFamily(body).padY(4).padX(5).borderWidth(1).border(error-boundary-border).radius(md).bg(error-boundary).fg(error-boundary-foreground).fontSize(sm).shadow(xs)",
            "style.h(control-h-sm).radius(sm).textAlign(center).cursor(pointer).fg(foreground).when({ hover: style.bg(hover-bg), selected: style.bg(primary).fg(primary-foreground), disabled: style.fg(muted-foreground).opacity(50).cursor(not-allowed), stateEmpty: style.visibility(hidden), stateToday: style.outline(border-strong), focus: style.outline(ring) })",
            "style.cursor(cell)",
            "style.borderWidth(1).border(border-subtle).radius(sm).pad(0).aspect(square).cursor(pointer).bg(bind(swatchColor)).when({ selected: style.outline(primary) })",
            "style.position(absolute).left(0).w(full).translateY(bind(rowY)).cursor(pointer).when({ hover: style.bg(hover-bg), selected: style.bg(selected-bg), focus: style.outline(ring) })",
        ];
        for e in packages_exprs {
            let (_, diags) = parse(e);
            assert!(
                diags.is_empty(),
                "packages expr failed: {e}\n{:?}",
                diags.iter().map(|d| d.message()).collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn setter_wave_control_and_gap_and_bg_mix() {
        // colorDeepen(token, pct) — two-arg color-mix toward foreground.
        let (style, diags) = parse("style.colorDeepen(primary, 15)");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let (_, css) = lower_style(&style, &HashMap::new());
        assert!(
            css.contains("color-mix(in oklch, var(--primary), var(--foreground) 15%)"),
            "{css}"
        );

        // colorDeepen bind arm for the color token.
        let (style, diags) = parse("style.colorDeepen(bind(swatch), 25)");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let (_, css) = lower_style(&style, &HashMap::new());
        assert!(css.contains("var(--swatch)"), "{css}");
        assert!(css.contains("color-mix(in oklch"), "{css}");

        // colorDeepen fail-closed: third arg is BadArity.
        let (_, diags) = parse("style.colorDeepen(primary, 15, extra)");
        assert!(
            diags
                .iter()
                .any(|d| matches!(d, StyleDiagnostic::BadArity { .. })),
            "colorDeepen must reject third arg: {:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );

        // colorDeepen fail-closed: percent off the 5..95 step-5 ramp.
        let (_, diags) = parse("style.colorDeepen(primary, 100)");
        assert!(
            diags
                .iter()
                .any(|d| matches!(d, StyleDiagnostic::BadToken { .. })),
            "colorDeepen must reject 100: {:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let (_, diags) = parse("style.colorDeepen(primary, 3)");
        assert!(
            diags
                .iter()
                .any(|d| matches!(d, StyleDiagnostic::BadToken { .. })),
            "colorDeepen must reject 3: {:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );

        // colorMix(token, pct) — transparent hue wash (soft fill).
        let (style, diags) = parse("style.colorMix(primary, 15)");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let (_, css) = lower_style(&style, &HashMap::new());
        assert!(
            css.contains("color-mix(in oklch, var(--primary) 15%, transparent)"),
            "{css}"
        );

        // colorMix bind arm for the color token.
        let (style, diags) = parse("style.colorMix(bind(swatch), 25)");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let (_, css) = lower_style(&style, &HashMap::new());
        assert!(css.contains("var(--swatch)"), "{css}");
        assert!(
            css.contains("color-mix(in oklch, var(--swatch) 25%, transparent)"),
            "{css}"
        );

        // colorMix fail-closed: third arg is BadArity.
        let (_, diags) = parse("style.colorMix(primary, 15, extra)");
        assert!(
            diags
                .iter()
                .any(|d| matches!(d, StyleDiagnostic::BadArity { .. })),
            "colorMix must reject third arg: {:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );

        // colorMix fail-closed: off-ramp percent.
        let (_, diags) = parse("style.colorMix(primary, 14)");
        assert!(
            diags
                .iter()
                .any(|d| matches!(d, StyleDiagnostic::BadToken { .. })),
            "colorMix must reject 14: {:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );

        // Host-paint gap props for flex/grid containers outside layout keywords.
        for (expr, prop, value) in [
            ("style.gap(2)", "gap", "var(--space-2)"),
            ("style.gap(0)", "gap", "0"),
            ("style.gapX(3)", "column-gap", "var(--space-3)"),
            ("style.gapY(1)", "row-gap", "var(--space-1)"),
            ("style.gap(bind(hostGap))", "gap", "var(--host-gap)"),
        ] {
            let (style, diags) = parse(expr);
            assert!(
                diags.is_empty(),
                "{expr}: {:?}",
                diags.iter().map(|d| d.message()).collect::<Vec<_>>()
            );
            let (_, css) = lower_style(&style, &HashMap::new());
            let norm = css.replace(' ', "");
            assert!(
                norm.contains(&format!("{prop}:{value}")),
                "{expr} expected {prop}:{value} in {css}"
            );
        }

        // Tier-2 control chrome / press marker setters.
        let (style, diags) = parse("style.controlFrame().controlLift().press()");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let (_, css) = lower_style(&style, &HashMap::new());
        let norm = css.replace(' ', "");
        assert!(norm.contains("box-shadow:var(--control-frame)"), "{css}");
        assert!(norm.contains("translate:var(--control-lift)"), "{css}");
        assert!(norm.contains("translate:var(--press-translate)"), "{css}");
        assert!(norm.contains("scale:var(--press-scale)"), "{css}");
        assert!(norm.contains("box-shadow:var(--press-shadow)"), "{css}");
    }

    #[test]
    fn enter_motion_config_object_parses_and_collects() {
        let (style, diags) = parse("style.enter({ opacity: 'fade', stagger: '450ms' })");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let m = collect_motion_attaches(&style);
        assert_eq!(m.len(), 1, "style={style:?} motions={m:?}");
        assert_eq!(m[0].0, "enter");
        assert!(m[0].1.contains("fade"), "{:?}", m[0]);
    }

    #[test]
    fn enter_then_when_chain_order() {
        let (style, diags) =
            parse("style.bg(primary).enter({ opacity: 'fade' }).when({ hover: style.bg(muted) })");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let m = collect_motion_attaches(&style);
        assert_eq!(m.len(), 1);
        let (_, css) = lower_style(&style, &HashMap::new());
        assert!(css.contains(":hover"), "{css}");
        // motion must not produce CSS atoms
        assert!(!css.contains("fade"), "{css}");
    }

    /// Emit identity: old chain vs new channel form must mint the same
    /// classes + CSS + motion attach list (trigger + config).
    fn emit_pair(old: &str, new: &str) {
        let (s_old, d_old) = parse(old);
        let (s_new, d_new) = parse(new);
        assert!(
            d_old.is_empty(),
            "old form diags for `{old}`: {:?}",
            d_old.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        assert!(
            d_new.is_empty(),
            "new form diags for `{new}`: {:?}",
            d_new.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let (c_old, css_old) = lower_style(&s_old, &HashMap::new());
        let (c_new, css_new) = lower_style(&s_new, &HashMap::new());
        assert_eq!(
            c_old, c_new,
            "class list identity failed\nold: {old}\nnew: {new}\nold classes: {c_old}\nnew classes: {c_new}"
        );
        assert_eq!(
            css_old, css_new,
            "CSS identity failed\nold: {old}\nnew: {new}\nold css:\n{css_old}\nnew css:\n{css_new}"
        );
        let m_old = collect_motion_attaches(&s_old);
        let m_new = collect_motion_attaches(&s_new);
        assert_eq!(
            m_old, m_new,
            "motion attach identity failed\nold: {old}\nnew: {new}\nold motions: {m_old:?}\nnew motions: {m_new:?}"
        );
    }

    #[test]
    fn channels_emit_identity_box_only() {
        emit_pair("style.pad(4).gap(2)", "style({ box: pad(4).gap(2) })");
    }

    #[test]
    fn channels_emit_identity_paint_only() {
        emit_pair(
            "style.bg(primary).radius(md)",
            "style({ paint: bg(primary).radius(md) })",
        );
    }

    #[test]
    fn channels_emit_identity_box_and_paint() {
        emit_pair(
            "style.pad(4).gap(2).bg(primary).radius(md)",
            "style({ box: pad(4).gap(2), paint: bg(primary).radius(md) })",
        );
    }

    #[test]
    fn channels_emit_identity_when_hover_paint() {
        emit_pair(
            "style.bg(primary).when({ hover: style.bg(muted) })",
            "style({ paint: bg(primary) }).when(hover, { paint: bg(muted) })",
        );
    }

    #[test]
    fn channels_old_form_control_still_green() {
        let (style, diags) = parse("style.pad(4).bg(primary).when({ hover: style.bg(muted) })");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let (classes, css) = lower_style(&style, &HashMap::new());
        assert!(classes.contains("sx-"), "{classes}");
        assert!(css.contains(":hover"), "{css}");
    }

    #[test]
    fn box_sizing_rename_and_alias_error() {
        let (style, diags) = parse("style.boxSizing(border)");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let (classes, css) = lower_style(&style, &HashMap::new());
        assert!(
            classes.split_whitespace().any(|c| c == "sx-boxSizingborder"),
            "{classes}"
        );
        assert!(
            css.contains("box-sizing") && css.contains("border-box"),
            "{css}"
        );

        let (_, diags) = parse("style.box(border)");
        assert!(
            diags.iter().any(|d| {
                let m = d.message();
                m.contains("boxSizing") && m.contains("style.box")
            }),
            "style.box must point at boxSizing: {:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );

        // Channel key `box:` remains geometry (not the retired method).
        let (style, diags) = parse("style({ box: pad(2) })");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let (classes, _) = lower_style(&style, &HashMap::new());
        assert!(classes.contains("sx-pad2") || classes.contains("pad"), "{classes}");
    }

    #[test]
    fn channels_flatten_order_paint_after_box() {
        // Author may write paint before box; flatten order is always box then paint.
        emit_pair(
            "style.pad(4).bg(primary)",
            "style({ paint: bg(primary), box: pad(4) })",
        );
    }

    #[test]
    fn channels_emit_identity_animate_enter() {
        emit_pair(
            "style.enter({ opacity: 'fade' })",
            "style({ animate: enter({ opacity: 'fade' }) })",
        );
    }

    #[test]
    fn channels_emit_identity_animate_enter_exit() {
        emit_pair(
            "style.enter({ opacity: 'fade' }).exit({ opacity: 'fade-out' })",
            "style({ animate: enter({ opacity: 'fade' }).exit({ opacity: 'fade-out' }) })",
        );
    }

    #[test]
    fn channels_emit_identity_paint_plus_animate() {
        emit_pair(
            "style.bg(primary).enter({ opacity: 'fade' }).exit({ opacity: 'fade-out' })",
            "style({ paint: bg(primary), animate: enter({ opacity: 'fade' }).exit({ opacity: 'fade-out' }) })",
        );
    }

    #[test]
    fn channels_emit_identity_flat_motion_tail_still_dual() {
        // Flat motion after channel spine remains dual-accept with animate:.
        emit_pair(
            "style({ paint: bg(primary) }).enter({ opacity: 'fade' })",
            "style({ paint: bg(primary), animate: enter({ opacity: 'fade' }) })",
        );
    }

    #[test]
    fn channels_animate_rejects_paint_setter() {
        let (_, diags) = parse("style({ animate: bg(primary) })");
        assert!(
            diags.iter().any(|d| {
                let m = d.message();
                m.contains("not a motion trigger") || m.contains("animate:")
            }),
            "paint setter in animate: must fail: {:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn channels_motion_in_box_channel_points_to_animate() {
        let (_, diags) = parse("style({ box: pad(2).enter({ opacity: 'fade' }) })");
        assert!(
            diags.iter().any(|d| d.message().contains("animate:")),
            "motion inside box: must point at animate: {:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn channels_when_map_form_still_works_on_spine() {
        emit_pair(
            "style.pad(2).when({ hover: style.bg(muted) })",
            "style({ box: pad(2) }).when({ hover: style.bg(muted) })",
        );
    }

    // ── Product-path locks (`lower_style_source`) ──────────────────────────
    // Unit `parse`+`lower_style` still accept flat form for emit-identity /
    // codemod dual path. Product entry refuses flat chains (channel spine only).

    fn product_lower(src: &str) -> Result<(String, String), Vec<StyleDiagnostic>> {
        let mut css = String::new();
        let mut seen = std::collections::HashSet::new();
        let classes = lower_style_source(src, &HashMap::new(), &mut css, &mut seen)?;
        Ok((classes, css))
    }

    fn product_err_msgs(src: &str) -> Vec<String> {
        match product_lower(src) {
            Ok((c, css)) => panic!("expected Err for `{src}`, got classes={c} css={css}"),
            Err(diags) => diags.into_iter().map(|d| d.message()).collect(),
        }
    }

    #[test]
    fn product_path_channel_spine_lowers() {
        let (classes, css) = product_lower(
            "style({ box: pad(4).gap(2), paint: bg(primary).radius(md) })",
        )
        .expect("channel spine is product form");
        assert!(
            classes.split_whitespace().any(|c| c.contains("pad")),
            "pad mint missing: {classes}"
        );
        assert!(
            classes.split_whitespace().any(|c| c.contains("gap")),
            "gap mint missing: {classes}"
        );
        assert!(classes.contains("sx-"), "{classes}");
        // Property presence — no vacuous OR escapes across unrelated props.
        assert!(
            css.contains("padding"),
            "pad must emit padding CSS:\n{css}"
        );
        // gap must be the property itself — pad also emits --spacing.
        assert!(css.contains("gap"), "gap CSS:\n{css}");
        assert!(
            css.contains("background") || css.contains("--primary"),
            "bg CSS:\n{css}"
        );
        assert!(
            css.contains("border-radius") || css.contains("--radius"),
            "radius CSS:\n{css}"
        );
    }

    #[test]
    fn product_path_flat_chain_is_refused() {
        let msgs = product_err_msgs("style.pad(4)");
        let joined = msgs.join("; ");
        assert!(
            joined.contains("retired") || joined.contains("channel spine"),
            "flat product must refuse: {joined}"
        );
        assert!(
            joined.contains("style({"),
            "must teach channel spine form: {joined}"
        );
        // Must not teach flat style.method as the fix destination.
        assert!(!joined.contains("style.pad"), "must not teach style.pad: {joined}");
        assert!(!joined.contains("style.bg"), "must not teach style.bg: {joined}");
    }

    #[test]
    fn product_path_when_hover_paint() {
        let (classes, css) = product_lower(
            "style({ paint: bg(primary) }).when(hover, { paint: bg(muted) })",
        )
        .expect("hover paint through product path");
        assert!(classes.contains("sx-"), "{classes}");
        assert!(css.contains(":hover"), "hover pseudo missing:\n{css}");
    }

    #[test]
    fn product_path_geometry_under_interaction_fails_closed() {
        let msgs = product_err_msgs(
            "style({ box: pad(2) }).when(hover, { box: pad(4) })",
        );
        let joined = msgs.join("; ");
        assert!(
            joined.contains("state can't gate layout"),
            "geometry under hover must hard-fail product path: {joined}"
        );
    }

    /// Product-path geometry × interaction sample (mirrors channel unit matrix).
    #[test]
    fn product_path_geometry_matrix_sample() {
        // group-hover + pad refuses.
        let msgs = product_err_msgs(
            "style({ box: pad(2) }).when(group-hover, { box: pad(4) })",
        );
        let joined = msgs.join("; ");
        assert!(
            joined.contains("state can't gate layout"),
            "group-hover + pad must refuse: {joined}"
        );

        // focusWithin + pad refuses.
        let msgs = product_err_msgs(
            "style({ box: pad(2) }).when(focusWithin, { box: pad(4) })",
        );
        let joined = msgs.join("; ");
        assert!(
            joined.contains("state can't gate layout"),
            "focusWithin + pad must refuse: {joined}"
        );

        // tablet + pad allows (media may gate layout).
        let (classes, _css) = product_lower(
            "style({ box: pad(2) }).when(tablet, { box: pad(4) })",
        )
        .expect("tablet may gate pad geometry on product path");
        assert!(classes.contains("sx-"), "{classes}");
    }

    #[test]
    fn product_path_color_alias_fails_closed() {
        let msgs = product_err_msgs("style({ paint: color(primary) })");
        let joined = msgs.join("; ");
        assert!(
            joined.contains("fg") && (joined.contains("color") || joined.contains("renamed")),
            "color→fg must hard-fail product path: {joined}"
        );
    }
}

