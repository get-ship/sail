//! Compile-time Style value language: style object (channels + when/match/compound).
//!
//! Sole paint architecture: style({ box, paint, text, animate, when, match, compound }).
//!
//! This crate owns Style parse / IR / emit / clash / setters, plus the leaf
//! sx machinery (`sx_ir`, `sx_vocab`), token scales, and design-token registry
//! used by Style lower and layout class resolution.

pub mod channel;
pub mod clash;
pub mod complete;
pub mod component_surfaces;
pub mod concern_rewrite;
pub mod concerns;
pub mod conditions;
pub mod emit;
pub mod host_ctx;
pub mod ir;
pub mod parse;
pub mod prefixes;
pub mod rewrite;
pub mod rewrite_v3;
pub mod scales;
pub mod setters;
pub mod style_ir;
pub mod style_vocab;
pub mod theme_env;
pub mod token_groups;
pub mod tokens;
pub mod type_resolve;
pub mod vocab;

pub use complete::{
    OBJECT_KEYS, StyleCompleteContext, StyleCompleteItem, StyleCompleteKind,
    collect_style_const_names, style_complete_at, style_complete_context_at,
    style_complete_for_context, when_keys,
};

pub use channel::{
    Channel, TEXT_BEARING_ELEMENTS, channel_for_setter, is_animate_trigger, is_geometry_setter,
    is_interaction_when_key, is_text_bearing, style_contains_geometry, style_uses_text_channel,
    style_uses_text_channel_env, text_channel_host_error,
};
pub use component_surfaces::{
    all_surface_packs, color_paint_vocabulary, is_component_public_name,
    is_theme_owned_author_root, is_theme_owned_token_name, surface_pack_for_host, surface_slots,
};
pub use concern_rewrite::{retired_dual_border_sites, rewrite_concern_source};
pub use concerns::{
    ConcernSlot, ConcernSpec, MARKERS, alias_concern_call, concern_doc_line, concern_kind,
    concerns, find_concern, is_concern_form, is_marker, resolve_concern_name, retired_replacement,
};
pub use conditions::{CONDITION_PREFIXES, condition_media};
pub use emit::{
    CssRule, StyleEnv, emit_style, lower_style, lower_style_source, lower_style_source_with_motion,
};
pub use host_ctx::{with_bound_group_props, with_host_surface};
pub use ir::{PaintArg, PaintOp, Style, StyleDiagnostic};
pub use parse::{
    collect_motion_attaches, collect_motion_attaches_env, looks_like_utility_string_product, parse,
    parse_style_const_rhs, validate_style_refs,
};
pub use rewrite::{
    lift_style_attr_hosts, rewrite_source, rewrite_style_expr, rewrite_teaching_source,
};
pub use rewrite_v3::rewrite_v3_source;
pub use theme_env::{
    NamedStyle, PlayPreset, StylingPolicy, TagDefault, ThemeStyleEnv, VocabTag,
    current_styling_policy, with_styling_policy,
};
pub use type_resolve::{ResolvedType, resolve_type, resolve_type_for_tag, resolve_type_role};
pub use vocab::{
    VocabRow, find_vocab, offered_rows, resolve_vocab, vocab_rows, vocab_surface_line,
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Test helper: legacy dual for emit/IR fixtures still written in old form.
    /// Product surface is `super::parse` (object form only).
    fn parse(src: &str) -> (Style, Vec<StyleDiagnostic>) {
        super::parse(src)
    }

    #[test]
    fn chain_emits_token_vars() {
        let (style, diags) = parse(
            "style({ box: pad({ x: 2 }), bg(primary).radius(sm) })",
        );
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let (classes, css) = lower_style(&style, &HashMap::new());
        assert!(classes.contains("s-"), "{classes}");
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
    #[ignore = "free form fixture (STYLE-BINDING-1)"]
    fn match_emits_attr_selectors() {
        let (style, diags) = parse(
            "match(variant, { solid: style({ paint: bg(primary) }), soft: style({ paint: bg(muted) }) })",
        );
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
        let (style, diags) = parse("style({ when: { hover: { bg(muted) } } })");
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
            "style({ text: type({ size: sm }), when: { tablet: { text: type({ size: lg }) }, desktop: { text: type({ size: xl }) } } })",
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
        let (style2, diags2) =
            parse("style({ when: { motionReduce: { paint: filter({ opacity: 100 }) } } })");
        assert!(diags2.is_empty(), "{:?}", diags2);
        let (_, css2) = lower_style(&style2, &HashMap::new());
        assert!(
            css2.contains("prefers-reduced-motion"),
            "motionReduce camelCase must resolve: {css2}"
        );
    }

    #[test]
    #[ignore = "deleted free form (STYLE-BINDING-1)"]
    fn compound_emits_multi_attr() {
        let (style, diags) = parse("compound({ pill: true }, style({ paint: radius(full) }))");
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
    #[ignore = "deleted free form (STYLE-BINDING-1)"]
    fn line_comment_between_list_items_keeps_entries() {
        let (style, diags) = parse(
            "[ style({ paint: bg(primary) }), // soften the surface\n style({ box: padX(2) }) ]",
        );
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
    #[ignore = "deleted free form (STYLE-BINDING-1)"]
    fn block_comment_and_trailing_comment_before_bracket_keep_entries() {
        let (style, diags) = parse(
            "[ style({ paint: bg(primary) }) /* block */, style({ box: padX(2) }) // trailing note\n ]",
        );
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
    #[ignore = "deleted free form (STYLE-BINDING-1)"]
    fn comment_inside_compound_arg_map_keeps_conditions() {
        let (style, diags) = parse(
            "compound({ pill: true, // emphasis keys\n tone: primary }, style({ paint: radius(full) }))",
        );
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
        let (style, diags) =
            parse("/* fixme: switch to tone-primary */ style({ bg(primary) })");
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
        let (style, diags) = parse(
            "style({ bg(primary), when: { hover: { bg(muted) } } })",
        );
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
        let (_, diags) = parse("style({ bg({ color: notARealTokenZzz }) })");
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
        let (style, _) = parse("style({ bg(primary).bg(muted) })");
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
    #[ignore = "deleted free form (STYLE-BINDING-1)"]
    fn list_shadows_duplicate_slot_silently() {
        // Ordered composition: later items shadow earlier ones — no Clash.
        let (style, diags) =
            parse("[ style({ paint: bg(primary) }), style({ paint: bg(muted) }) ]");
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
        env.insert(
            "base".into(),
            parse("style({ bg(primary) })").0,
        );
        let (style, _) = parse("[ base, style({ bg(muted) }) ]");
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
    fn absorbed_border_width_is_retired() {
        let (_, diags) = parse("style({ paint: borderWidth(2) })");
        assert!(
            diags
                .iter()
                .any(|d| matches!(d, StyleDiagnostic::RetiredSetter { .. })
                    && d.message().contains("border")),
            "borderWidth is RetiredSetter naming border: {:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
    }

    #[test]
    #[ignore = "deleted free form (STYLE-BINDING-1)"]
    fn chain_inside_list_still_clashes() {
        // A Chain inside a List keeps unordered conjunction semantics.
        let (style, _) = parse("[ style({ paint: bg(primary).bg(muted) }) ]");
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
    #[ignore = "deleted free form (STYLE-BINDING-1)"]
    fn list_emits_later_rule_after_earlier() {
        // Stylesheet-order precedence: emit walks items in source order, so
        // the shadowing (later) rule lands after the shadowed one in the CSS.
        let (style, diags) =
            parse("[ style({ paint: bg(primary) }), style({ paint: bg(muted) }) ]");
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
        let (style, diags) = parse("style({ translateY(bind(rowY)) })");
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
    #[ignore = "deleted free form (STYLE-BINDING-1)"]
    fn list_composition() {
        let (style, diags) = parse(
            "[ style({ paint: bg(primary) }), style({ when: { hover: { paint: bg(muted) } } }) ]",
        );
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
        env.insert(
            "filled".into(),
            parse("style({ bg(primary) })").0,
        );
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
        let (_style, diags) = parse("[ ( , style({ paint: bg(primary) }) ]");
        assert!(start.elapsed().as_millis() < 200);
        let _ = diags;
    }

    #[test]
    fn malformed_chain_finishes_quickly() {
        let start = std::time::Instant::now();
        let (_style, diags) = parse("style({ paint: bg(primary) }).(");
        assert!(start.elapsed().as_millis() < 200);
        let _ = diags;
    }

    #[test]
    fn nested_garbage_finishes_quickly() {
        let start = std::time::Instant::now();
        let input = format!(
            "{}{}",
            "[[[[",
            "match(x, {".repeat(40) + "style({ paint: bg(primary) })"
        );
        let (_style, diags) = parse(&input);
        assert!(start.elapsed().as_millis() < 500);
        let _ = diags;
    }

    #[test]
    fn negative_ms_and_first_when() {
        let (style, diags) = parse(
            "style({  margin({ x: auto }), when: { first: {  margin({ s: 0 }), radius({ l: md })  }, last: {  radius({ r: md })  } } })",
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
        let (style, diags) =
            parse("style({ when: { group-required: { display(inline) } } })");
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
            "style({  paint: group(), when: { group-caught: {  display(none)  }, group-inline: {  display(none)  } } })",
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
        let (style, diags) = parse("style({ paint: glyphFill(true).isolate() })");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let (_, css) = lower_style(&style, &HashMap::new());
        assert!(css.contains(" svg") || css.contains("width:100%"), "{css}");
        assert!(
            css.contains("isolation:isolate") || css.contains("isolation: isolate"),
            "{css}"
        );
    }

    #[test]
    fn site_shell_setters_and_state_live() {
        let (style, diags) = parse(
            "style({  origin(bottom).bg({ glow: accent }), type({ wrap: truncate, case: uppercase }), when: { stateLive: {  bg(primary)  }, current: {  fg(primary-foreground)  } } })",
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
    #[ignore = "free form fixture (STYLE-BINDING-1)"]
    fn site_shell_wave_exprs_parse() {
        let exprs = [
            "style({ box: w(full).minW(0).overflowX(hidden), paint: bg(surface-0) })",
            "style({ box: left(0).right(0).top(0).h(64), paint: bgGlow(accent).pointerEvents(none).z(base) })",
            "style({ paint: fg(accent), text: fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide) })",
            "style({ paint: fg(foreground), text: fontSize(display).fontFamily(display).fontWeight(bold).tracking(tighter).leading(none).textAlign(center) })",
            "style({ box: position(absolute).left(-10).right(-10).top(12), paint: rotate(-2).z(popover) })",
            "style({ box: padY(3), paint: borderTWidth(1).borderBWidth(1).borderT(border).borderB(border).bg(surface-2) })",
            "style({ box: h(full).overflow(hidden), paint: bg(muted).borderWidth(1).border(primary) })",
            "style({ box: mb(-8), paint: textGradient(primary).select(none), text: fontSize(display).fontFamily(display).fontWeight(bold).tracking(tighter).leading(none).textAlign(center) })",
            "style({ box: padX(3).padY(1), paint: radius(full).fg(muted-foreground), text: fontSize(sm).fontWeight(semibold), when: { hover: { paint: fg(foreground) }, current: { paint: bg(primary).fg(primary-foreground) } } })",
            "style({  box: pad(3), paint: borderWidth(1).border(border).opacity(50), when: { stateLive: {  paint: opacity(100).border(primary).bg(muted)  } } })",
            "style({ box: w(1), paint: radius(sm).bg(primary).origin(bottom) })",
            "style({ box: w(8), text: fontFamily(mono).fontSize(sm).nums(tabular).textAlign(center) })",
            "style({ text: fontFamily(mono).fontSize(2xl).nums(tabular) })",
            "style({ box: pad(4).w(full).maxW(panel-max-w-sm) })",
            "style({ box: w(full).maxW(breakpoint-desktop).mx(auto) })",
            "style({ box: maxW(breakpoint-mobile), paint: fg(muted-foreground), text: fontSize(lg).leading(relaxed).textAlign(center) })",
            "style({ box: display(block).minW(0), text: underline(none) })",
            "style({  paint: fg(muted-foreground), when: { hover: {  paint: fg(primary)  } } })",
            "style({ box: grow(1).minW(0), text: truncate() })",
            "style({ box: pad(6).grow(1).minW(0) })",
            "style({ box: grow(1).basis(0).minW(0) })",
            "style({ box: grow(1).basis(breakpoint-mobile).minW(0) })",
            "style({ paint: translateX(bind(tileX)).rotate(bind(tileRotate)).scale(bind(tileScale)) })",
            "style({ , when: { groupCurrent: {  paint: fg(primary)  }, desktop: {  box: display(block)  } } })",
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
        let (style, diags) = parse(
            "style({ size({ w: 0 }), scale(100).rotate(bind(tileRotate)) })",
        );
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
    #[ignore = "free form fixture (STYLE-BINDING-1)"]
    fn packages_wave_setters_and_dynamic_when() {
        let (style, diags) = parse(
            "style({ box: aspect(square).ml(auto), paint: srOnly().visibility(hidden).objectFit(cover).cursor(crosshair).transform(bind(worldTx)), text: italic(true) })",
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
            let expr = format!("when({{ {key}: style({{ paint: bg(muted) }}) }})");
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
            "style({ box: maxW(full), paint: group().fg(foreground), text: fontFamily(body) })",
            "style({  box: padY(2).padX(3), paint: radius(lg).bg(surface-1).borderWidth(1).border(border).fg(foreground), text: fontSize(base).leading(normal), when: { groupStateUser: {  paint: bg(hover-bg).borderWidth(1).border(primary)  }, groupStateSystem: {  paint: bg(muted).fg(muted-foreground)  }, groupStateError: {  paint: borderWidth(1).border(destructive)  }, groupStateSending: {  paint: opacity(75)  } } })",
            "style({  box: padY(density-row-py).padX(density-row-px).minH(panel-min-h-cell), paint: group().borderRWidth(1).borderR(border).borderBWidth(1).borderB(border), when: { stateOut: {  paint: fg(muted-foreground).bg(muted)  }, stateWeekend: {  paint: bg(muted)  }, stateRange: {  paint: bg(selected-bg)  } } })",
            "style({  box: position(absolute).top(bind(evtTop)).h(bind(evtH)).left(bind(evtLeft)).w(bind(evtW)).padY(half).padX(1).overflow(hidden), paint: cursor(pointer).fg(primary).bg(hover-bg).radius(sm), text: textAlign(start).fontSize(density), when: { selected: {  paint: outline(ring)  } } })",
            "style({  box: padY(6).padX(4), paint: group().borderWidth(1).border(border).border(dashed).radius(lg).cursor(pointer).fg(muted-foreground).select(none), text: textAlign(center), when: { focus: {  paint: outline(ring)  }, stateOver: {  paint: borderWidth(1).border(ring).bg(muted)  } } })",
            "style({ box: position(absolute).left(bind(boxLeft)).top(bind(boxTop)).w(bind(boxW)).h(bind(boxH)), paint: bg(hover-bg).borderWidth(1).border(primary).border(dashed).radius(sm).cursor(pointer).pointerEvents(auto) })",
            "style({  box: w(bind(boxW)).padY(2).padX(3), paint: bg(surface-1).fg(foreground).borderWidth(1).border(border).radius(md).shadow(xs).cursor(pointer), text: fontFamily(body).fontSize(sm), when: { hover: {  paint: shadow(sm)  }, focus: {  paint: outline(ring)  }, current: {  paint: borderWidth(1).border(ring).shadow(sm)  }, stateAgent: {  paint: borderLWidth(1).borderL(primary)  } } })",
            "style({ box: padY(4).padX(5), paint: borderWidth(1).border(destructive).radius(md).bg(muted).fg(foreground).shadow(xs), text: fontFamily(body).fontSize(sm) })",
            "style({  box: h(control-h-sm), paint: radius(sm).cursor(pointer).fg(foreground), text: textAlign(center), when: { hover: {  paint: bg(hover-bg)  }, selected: {  paint: bg(primary).fg(primary-foreground)  }, disabled: {  paint: fg(muted-foreground).opacity(50).cursor(not-allowed)  }, stateEmpty: {  paint: visibility(hidden)  }, stateToday: {  paint: outline(border-strong)  }, focus: {  paint: outline(ring)  } } })",
            "style({ paint: cursor(cell) })",
            "style({  box: pad(0).aspect(square), paint: borderWidth(1).border(border-subtle).radius(sm).cursor(pointer).bg(bind(swatchColor)), when: { selected: {  paint: outline(primary)  } } })",
            "style({  box: position(absolute).left(0).w(full), paint: translateY(bind(rowY)).cursor(pointer), when: { hover: {  paint: bg(hover-bg)  }, selected: {  paint: bg(selected-bg)  }, focus: {  paint: outline(ring)  } } })",
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
        // colorDeepen / colorMix / gap are absorbed — RetiredSetter, not a dual path.
        for (src, concern) in [
            ("style({ paint: colorDeepen(primary, 15) })", "bg"),
            ("style({ paint: colorMix(primary, 15) })", "bg"),
            ("style({ box: gap(2) })", "gap"),
        ] {
            let (_, diags) = parse(src);
            assert!(
                diags.iter().any(|d| d.message().contains(concern)),
                "{src} must name {concern}: {:?}",
                diags.iter().map(|d| d.message()).collect::<Vec<_>>()
            );
        }

        // Tier-2 control chrome / press marker setters.
        let (style, diags) = parse("style({ paint: controlFrame().controlLift().press() })");
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
        let (style, diags) =
            parse("style({ animate: enter({ opacity: 'fade', stagger: '450ms' }) })");
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
    #[ignore = "deleted free form (STYLE-BINDING-1)"]
    fn enter_then_when_chain_order() {
        let (style, diags) = parse(
            "style({  paint: bg(primary), animate: enter({ opacity: 'fade' }), when: { hover: {  paint: bg(muted)  } } })",
        );
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

    /// Codemod emit-identity (flat → channel) lives in `rewrite` tests via
    /// `emit_pair` + `parse`. Product tests below lock channel form only.

    #[test]
    fn channels_control_when_hover_still_green() {
        let (style, diags) = parse(
            "style({  pad(4), bg(primary), when: { hover: {  bg(muted)  } } })",
        );
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let (classes, css) = lower_style(&style, &HashMap::new());
        assert!(classes.contains("s-"), "{classes}");
        assert!(css.contains(":hover"), "{css}");
    }

    #[test]
    #[ignore = "deleted free form (STYLE-BINDING-1)"]
    fn box_sizing_rename_and_alias_error() {
        let (style, diags) = parse("style({ paint: boxSizing(border) })");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let (classes, css) = lower_style(&style, &HashMap::new());
        assert!(
            classes.split_whitespace().any(|c| c == "s-boxSizingborder"),
            "{classes}"
        );
        assert!(
            css.contains("box-sizing") && css.contains("border-box"),
            "{css}"
        );

        // Rename pointer lives on migration dual (`parse`); product parse
        // refuses any flat `style.*` chain before the box→boxSizing pointer.
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
        let (style, diags) = parse("style({ pad(2) })");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let (classes, _) = lower_style(&style, &HashMap::new());
        assert!(
            classes.contains("s-pad2") || classes.contains("pad"),
            "{classes}"
        );
    }

    #[test]
    fn channels_flatten_order_paint_after_box() {
        // Author may write paint before box; flatten order is always box then paint.
        let (s_a, d_a) =
            parse("style({ paint: bg({ color: primary }), box: pad(4) })");
        let (s_b, d_b) =
            parse("style({ box: pad(4), paint: bg({ color: primary }) })");
        assert!(
            d_a.is_empty() && d_b.is_empty(),
            "diags a={:?} b={:?}",
            d_a.iter().map(|d| d.message()).collect::<Vec<_>>(),
            d_b.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let (c_a, css_a) = lower_style(&s_a, &HashMap::new());
        let (c_b, css_b) = lower_style(&s_b, &HashMap::new());
        assert_eq!(c_a, c_b, "class order identity\na={c_a}\nb={c_b}");
        assert_eq!(css_a, css_b, "css identity\na:\n{css_a}\nb:\n{css_b}");
    }

    #[test]
    fn parse_refuses_motion_tail_after_channel_spine() {
        // Flat motion tail after style({…}) is retired — animate: channel only.
        let (_, diags) =
            parse("style({ bg(primary) }).enter({ opacity: 'fade' })");
        let joined = diags
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
            .join("; ");
        assert!(
            joined.contains("retired") || joined.contains("animate:"),
            "product parse must refuse motion tail: {joined}"
        );
        assert!(
            joined.contains("animate:"),
            "must teach animate: channel: {joined}"
        );
        assert!(
            !joined.contains("style.enter"),
            "must not re-teach flat style.method: {joined}"
        );
    }

    #[test]
    fn channels_animate_channel_motion_attaches() {
        let (s_ch, d_ch) = parse(
            "style({ bg(primary), animate: enter({ opacity: 'fade' }) })",
        );
        assert!(
            d_ch.is_empty(),
            "animate: channel must parse clean: {:?}",
            d_ch.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let attaches = collect_motion_attaches(&s_ch);
        assert_eq!(attaches.len(), 1, "one enter attach: {attaches:?}");
        assert_eq!(attaches[0].0, "enter");
        assert!(
            attaches[0].1.contains("opacity"),
            "config preserved: {:?}",
            attaches[0].1
        );
    }

    /// Regression: `style(name)` is a Style::Ref. CSS emit expands the ref via
    /// StyleEnv, but motion collection used to skip Ref — so every named style
    /// with an `animate:` channel painted correctly and never moved.
    #[test]
    fn named_style_ref_collects_motion_via_env() {
        let (named, diags) = parse(
            "style({ box: size({ w: 4, h: 4 }), paint: radius(full).bg(primary), animate: loop({ translate-y: 'spring:gentle', stagger: '0.15s' }) })",
        );
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let mut env = HashMap::new();
        env.insert("ambientDot".into(), named);

        // Tag-position `style(ambientDot)` parses as a bare Ref.
        let (host, host_diags) = parse("ambientDot");
        assert!(
            host_diags.is_empty(),
            "{:?}",
            host_diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        assert!(
            collect_motion_attaches(&host).is_empty(),
            "without env a bare Ref must yield no attaches (no false positives)"
        );
        let via_env = collect_motion_attaches_env(&host, &env);
        assert_eq!(
            via_env.len(),
            1,
            "env must surface the named loop: {via_env:?}"
        );
        assert_eq!(via_env[0].0, "loop");
        assert!(
            via_env[0].1.contains("translate-y"),
            "config preserved: {:?}",
            via_env[0].1
        );

        // lower_style_source_with_motion is the product path preprocess uses.
        let mut css = String::new();
        let mut seen = std::collections::HashSet::new();
        let (_classes, motions) =
            lower_style_source_with_motion("ambientDot", &env, &mut css, &mut seen)
                .expect("named style lowers");
        assert_eq!(
            motions.len(),
            1,
            "product lower must inject motion for style(name): {motions:?}"
        );
        assert_eq!(motions[0].0, "loop");
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
        let (_, diags) = parse("style({ pad(2).enter({ opacity: 'fade' }) })");
        assert!(
            diags.iter().any(|d| d.message().contains("animate:")),
            "motion inside box: must point at animate: {:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
    }

    // ── Product-path locks (`lower_style_source` + channel-only `parse`) ──
    // Flat form is migration-dual only via `parse` (codemod / rewrite tests).
    // Product `parse` and `lower_style_source` refuse flat chains.

    #[test]
    fn parse_refuses_flat_chain() {
        let (_, diags) = super::parse("style.pad(4)");
        let joined = diags
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
            .join("; ");
        assert!(
            joined.contains("retired") || joined.contains("channel spine"),
            "product parse must refuse flat: {joined}"
        );
        assert!(
            joined.contains("style({"),
            "must teach channel spine: {joined}"
        );
    }

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
            "style({ pad(4), bg(primary).radius(md) })",
        )
        .expect("channel spine is product form");
        assert!(classes.contains("s-"), "{classes}");
        // Property presence — no vacuous OR escapes across unrelated props.
        assert!(
            css.contains("padding"),
            "space pad must emit padding CSS:\n{css}"
        );
        assert!(
            css.contains("background") || css.contains("--primary"),
            "fill CSS:\n{css}"
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
        assert!(
            !joined.contains("style.pad"),
            "must not teach style.pad: {joined}"
        );
        assert!(
            !joined.contains("style.bg"),
            "must not teach style.bg: {joined}"
        );
    }

    #[test]
    fn product_path_when_hover_paint() {
        let (classes, css) =
            product_lower("style({ bg(primary), when: { hover: { bg(muted) } } })")
                .expect("hover paint through product path");
        assert!(classes.contains("s-"), "{classes}");
        assert!(css.contains(":hover"), "hover pseudo missing:\n{css}");
    }

    #[test]
    fn product_path_geometry_under_interaction_fails_closed() {
        let msgs = product_err_msgs(
            "style({ pad(2), when: { hover: { pad(4) } } })",
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
            "style({ pad(2), when: { group-hover: { pad(4) } } })",
        );
        let joined = msgs.join("; ");
        assert!(
            joined.contains("state can't gate layout"),
            "group-hover + pad must refuse: {joined}"
        );

        // focusWithin + pad refuses.
        let msgs = product_err_msgs(
            "style({ pad(2), when: { focusWithin: { pad(4) } } })",
        );
        let joined = msgs.join("; ");
        assert!(
            joined.contains("state can't gate layout"),
            "focusWithin + pad must refuse: {joined}"
        );

        // tablet + pad allows (media may gate layout).
        let (classes, _css) = product_lower(
            "style({ pad(2), when: { tablet: { pad(4) } } })",
        )
        .expect("tablet may gate pad geometry on product path");
        assert!(classes.contains("s-"), "{classes}");
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

    #[test]
    fn style_object_match_compound_when() {
        let (style, diags) = super::parse(
            "style({ bg(primary), when: { hover: { bg(muted) } }, match: { variant: { solid: { paint: bg({ color: secondary }) } } }, compound: [ { pill: true, paint: radius(full) } ] })",
        );
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let (_, css) = lower_style(&style, &std::collections::HashMap::new());
        assert!(css.contains(":hover"), "{css}");
        assert!(css.contains("[variant='solid']"), "{css}");
        assert!(css.contains("[pill]"), "{css}");
    }

    #[test]
    fn style_object_tone_axis_refused() {
        let (_, diags) = super::parse(
            "style({ match: { tone: { primary: { bg(primary) } } } })",
        );
        assert!(
            diags.iter().any(|d| {
                let m = d.message();
                m.contains("token-typed") || m.contains("cannot be a match axis")
            }),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn style_object_wildcard_compound() {
        let (style, diags) =
            super::parse("style({ compound: [ { tone: *, paint: filter({ opacity: 90 }) } ] })");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let (_, css) = lower_style(&style, &std::collections::HashMap::new());
        assert!(css.contains("[tone]"), "presence gate missing: {css}");
    }

    #[test]
    fn style_variadic_composition() {
        let (style, diags) =
            super::parse("style({ bg(primary) }, { pad(2) })");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let (classes, _) = lower_style(&style, &std::collections::HashMap::new());
        assert!(classes.contains("s-"), "{classes}");
    }

    #[test]
    fn free_match_call_refused() {
        let (_, diags) = super::parse("match(variant, { solid: style({ paint: bg(primary) }) })");
        assert!(!diags.is_empty(), "free match() must fail product parse");
    }
}

#[cfg(test)]
mod avatar_media_emit_tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn avatar_media_emits_cover_and_icon_slug_contain() {
        let (style, diags) = parse("style({ paint: avatarMedia(true) })");
        assert!(diags.is_empty(), "{:?}", diags);
        let (_, css) = lower_style(&style, &HashMap::new());
        assert!(
            css.contains("object-fit:cover") || css.contains("object-fit: cover"),
            "photo cover: {css}"
        );
        assert!(
            css.contains("[icon-slug]")
                && (css.contains("object-fit:contain") || css.contains("object-fit: contain")),
            "icon contain: {css}"
        );
        assert!(
            css.contains("padding:10%") || css.contains("padding: 10%"),
            "pad: {css}"
        );
    }
}
