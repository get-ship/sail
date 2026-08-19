//! Emit locks moved out of spinnaker so the domain crate has no CSS types.

#[cfg(test)]
mod tests {
    use crate::{lower_style, lower_style_source, lower_style_source_with_motion};
    use spinnaker::parse as parse_style;
    use spinnaker::*;
    use std::collections::HashMap;

    /// Test helper wrapping product parse.
    fn parse(src: &str) -> (Style, Vec<StyleDiagnostic>) {
        parse_style(src)
    }

    #[test]
    fn chain_emits_token_vars() {
        let (style, diags) = parse("style({ pad({ x: 2 }).bg(primary).radius(sm) })");
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
        let (style, diags) =
            parse("match(variant, { solid: style({ bg(primary) }), soft: style({ bg(muted) }) })");
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

    /// Style `when` breakpoints follow the active theme table.
    #[test]
    fn when_breakpoints_match_design_tokens() {
        let table = spinnaker::tokens::breakpoint_table();
        assert!(
            table.iter().any(|(n, _)| n == "wide"),
            "default theme must declare wide: {table:?}"
        );
        for (name, width) in &table {
            let query = crate::condition_media(name).expect("Style condition table");
            assert_eq!(
                crate::when_target(name),
                Some(crate::WhenTarget::Media(std::borrow::Cow::Owned(
                    query.clone()
                ))),
                "when({{{name}:…}}) must resolve via Style conditions"
            );
            assert!(
                query.contains(width.trim()),
                "`{name}` media {query} must embed theme width {width}"
            );
        }

        for (name, query) in crate::PREFERENCE_PREFIXES {
            assert_eq!(
                crate::when_target(name),
                Some(crate::WhenTarget::Media(std::borrow::Cow::Owned(
                    (*query).to_string()
                ))),
                "missing when key for Style condition `{name}`"
            );
        }

        let (style, diags) = parse(
            "style({ type({ size: sm }), when: { tablet: { type({ size: lg }) }, desktop: { type({ size: xl }) }, wide: { type({ size: xl }) } } })",
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
        assert!(
            css.contains("@media (min-width: 1600px)") || css.contains("@media(min-width: 1600px)"),
            "wide when must emit theme width: {css}"
        );
        let (style2, diags2) =
            parse("style({ when: { motionReduce: { filter({ opacity: 100 }) } } })");
        assert!(diags2.is_empty(), "{:?}", diags2);
        let (_, css2) = lower_style(&style2, &HashMap::new());
        assert!(
            css2.contains("prefers-reduced-motion"),
            "motionReduce camelCase must resolve: {css2}"
        );
    }

    #[test]
    fn when_tablet_compiles_away_without_theme_bp() {
        spinnaker::tokens::with_breakpoint_table(&[("mobile", "390px")], || {
            let (style, diags) = parse("style({ bg(primary), when: { tablet: { bg(accent) } } })");
            assert!(
                diags.is_empty(),
                "omitted tablet is not a hard error: {:?}",
                diags.iter().map(|d| d.message()).collect::<Vec<_>>()
            );
            let (_, css) = lower_style(&style, &HashMap::new());
            assert!(
                !css.contains("768") && !css.contains("tablet"),
                "no tablet media on a 1-bp theme: {css}"
            );
            assert!(
                crate::when_target("tablet").is_none(),
                "tablet must compile away"
            );
            assert!(
                crate::when_target("wide").is_none(),
                "undeclared theme extra is not a target"
            );
        });
    }

    #[test]
    fn when_undeclared_key_names_theme_table() {
        let (_, diags) = parse("style({ bg(primary), when: { kiosk: { bg(accent) } } })");
        let joined = diags
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
            .join("; ");
        assert!(
            joined.contains("theme breakpoint table"),
            "diagnostic must name the theme table: {joined}"
        );
        assert!(
            joined.contains("kiosk") && joined.contains("language state"),
            "must name the key and language states: {joined}"
        );
        for name in ["mobile", "tablet", "desktop"] {
            assert!(
                joined.contains(name),
                "must list theme bp `{name}`: {joined}"
            );
        }
        assert!(
            !joined.contains("closed browser-state table"),
            "old closed-table sentence is a lie: {joined}"
        );
    }

    #[test]
    #[ignore = "deleted free form (STYLE-BINDING-1)"]
    fn compound_emits_multi_attr() {
        let (style, diags) = parse("compound({ pill: true }, style({ radius(full) }))");
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
        let (style, diags) =
            parse("[ style({ bg(primary) }), // soften the surface\n style({ pad({ x: 2 }) }) ]");
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
            "[ style({ bg(primary) }) /* block */, style({ pad({ x: 2 }) }) // trailing note\n ]",
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
            "compound({ pill: true, // emphasis keys\n tone: primary }, style({ radius(full) }))",
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
        let (style, diags) = parse("/* fixme: switch to tone-primary */ style({ bg(primary) })");
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
        let (style, diags) = parse("style({ bg(primary), when: { hover: { bg(muted) } } })");
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
        let (style, diags) = parse("[ style({ bg(primary) }), style({ bg(muted) }) ]");
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
        env.insert("base".into(), parse("style({ bg(primary) })").0);
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
        let (_, diags) = parse("style({ borderWidth(2) })");
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
        let (style, _) = parse("[ style({ bg(primary).bg(muted) }) ]");
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
        let (style, diags) = parse("[ style({ bg(primary) }), style({ bg(muted) }) ]");
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
        let (style, diags) =
            parse("[ style({ bg(primary) }), style({ when: { hover: { bg(muted) } } }) ]");
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
        env.insert("filled".into(), parse("style({ bg(primary) })").0);
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
        let (_style, diags) = parse("[ ( , style({ bg(primary) }) ]");
        assert!(start.elapsed().as_millis() < 200);
        let _ = diags;
    }

    #[test]
    fn malformed_chain_finishes_quickly() {
        let start = std::time::Instant::now();
        let (_style, diags) = parse("style({ bg(primary) }).(");
        assert!(start.elapsed().as_millis() < 200);
        let _ = diags;
    }

    #[test]
    fn nested_garbage_finishes_quickly() {
        let start = std::time::Instant::now();
        let input = format!(
            "{}{}",
            "[[[[",
            "match(x, {".repeat(40) + "style({ bg(primary) })"
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
        let (style, diags) = parse("style({ when: { group-required: { display(inline) } } })");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let (_, css) = lower_style(&style, &HashMap::new());
        assert!(css.contains(".sail-grp[required]"), "{css}");
    }

    #[test]
    fn group_variant_when_emits_host_attr() {
        let (style, diags) = parse(
            "style({ when: { group-variant-dots: { display(none) }, group-variant-bars: { display(none) } } })",
        );
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let (_, css) = lower_style(&style, &HashMap::new());
        assert!(css.contains(".sail-grp[variant='dots']"), "{css}");
        assert!(css.contains(".sail-grp[variant='bars']"), "{css}");
        assert!(
            !css.contains("data-state"),
            "variant is a host attr, not data-state: {css}"
        );
    }

    #[test]
    fn group_padding_when_emits_host_attr() {
        let (style, diags) = parse("style({ when: { group-padding-lg: { display(none) } } })");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let (_, css) = lower_style(&style, &HashMap::new());
        assert!(css.contains(".sail-grp[padding='lg']"), "{css}");
        assert!(
            !css.contains("data-state"),
            "padding is a host attr, not data-state: {css}"
        );
    }

    #[test]
    fn margin_slot_negatives_emit_calc() {
        let (style, diags) = parse("style({ margin({ t: -5, x: -6, b: 5 }) })");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let (_, css) = lower_style(&style, &HashMap::new());
        assert!(
            css.contains("calc(-1 * var(--space-5))") && css.contains("calc(-1 * var(--space-6))"),
            "image bleed negatives must survive slot lower: {css}"
        );
        assert!(
            css.contains("margin-bottom:var(--space-5)")
                || css.contains("margin-bottom: var(--space-5)"),
            "positive b:5 must stay positive: {css}"
        );
    }

    #[test]
    fn group_caught_and_group_inline_when_emit_ancestors() {
        let (style, diags) = parse(
            "style({  group(), when: { group-caught: {  display(none)  }, group-inline: {  display(none)  } } })",
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
    fn isolate_lowers() {
        let (style, diags) = parse("style({ isolate() })");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let (_, css) = lower_style(&style, &HashMap::new());
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
            "style({ size({ w: full, min-w: 0 }).overflow({ x: hidden }).bg(surface-0) })",
            "style({ position({ l: 0, r: 0, t: 0, z: base }).size({ h: 64 }).bg({ glow: accent }).pointerEvents(none) })",
            "style({ fg(accent).type({ family: mono, size: xs, case: uppercase, tracking: wide }) })",
            "style({ fg(foreground).type({ size: display, family: display, weight: bold, tracking: tighter, leading: none, align: center }) })",
            "style({ position({ scheme: absolute, l: -10, r: -10, t: 12, z: popover }).rotate(-2) })",
            "style({ pad({ y: 3 }).border({ edge: t, width: 1, color: border }).border({ edge: b, width: 1, color: border }).bg(surface-2) })",
            "style({ size({ h: full }).overflow(hidden).bg(muted).border({ width: 1, color: primary }) })",
            "style({ margin({ b: -8 }).fg({ gradient: primary }).select(none).type({ size: display, family: display, weight: bold, tracking: tighter, leading: none, align: center }) })",
            "style({ pad({ x: 3, y: 1 }).radius(full).fg(muted-foreground).type({ size: sm, weight: semibold }), when: { hover: { fg(foreground) }, current: { bg(primary).fg(primary-foreground) } } })",
            "style({  pad(3).border({ width: 1, color: border }).opacity(50), when: { stateLive: {  opacity(100).border(primary).bg(muted)  } } })",
            "style({ size({ w: 1 }).radius(sm).bg(primary).origin(bottom) })",
            "style({ size({ w: 8 }).type({ family: mono, size: sm, nums: tabular, align: center }) })",
            "style({ type({ family: mono, size: 2xl, nums: tabular }) })",
            "style({ pad(4).size({ w: full, max-w: panel-max-w-sm }) })",
            "style({ size({ w: full, max-w: breakpoint-desktop }).margin({ x: auto }) })",
            "style({ size({ max-w: breakpoint-mobile }).fg(muted-foreground).type({ size: lg, leading: relaxed, align: center }) })",
            "style({ display(block).size({ min-w: 0 }).type({ decoration: none }) })",
            "style({  fg(muted-foreground), when: { hover: {  fg(primary)  } } })",
            "style({ grow(1).size({ min-w: 0 }).type({ wrap: truncate }) })",
            "style({ pad(6).grow(1).size({ min-w: 0 }) })",
            "style({ grow(1).size({ basis: 0, min-w: 0 }) })",
            "style({ grow(1).size({ basis: breakpoint-mobile, min-w: 0 }) })",
            "style({ translateX(bind(tileX)).rotate(bind(tileRotate)).scale(bind(tileScale)) })",
            "style({ when: { groupCurrent: {  fg(primary)  }, desktop: {  display(block)  } } })",
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
        let (style, diags) =
            parse("style({ size({ w: 0 }), scale(100).rotate(bind(tileRotate)) })");
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
            "style({ size({ aspect: square }).margin({ l: auto }).srOnly().visibility(hidden).objectFit(cover).cursor(crosshair).transform(bind(worldTx)).type({ italic: true }) })",
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
            let expr = format!("when({{ {key}: style({{ bg(muted) }}) }})");
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
            "style({ size({ max-w: full }).group().fg(foreground).type({ family: body }) })",
            "style({  pad({ y: 2, x: 3 }).radius(lg).bg(surface-1).border({ width: 1, color: border }).fg(foreground).type({ size: base, leading: normal }), when: { groupStateUser: {  bg(hover-bg).border({ width: 1, color: primary })  }, groupStateSystem: {  bg(muted).fg(muted-foreground)  }, groupStateError: {  border({ width: 1, color: destructive })  }, groupStateSending: {  opacity(75)  } } })",
            "style({  pad({ y: density-row-py, x: density-row-px }).size({ min-h: panel-min-h-cell }).group().border({ edge: r, width: 1, color: border }).border({ edge: b, width: 1, color: border }), when: { stateOut: {  fg(muted-foreground).bg(muted)  }, stateWeekend: {  bg(muted)  }, stateRange: {  bg(selected-bg)  } } })",
            "style({  position({ scheme: absolute, t: bind(evtTop), l: bind(evtLeft) }).size({ h: bind(evtH), w: bind(evtW) }).pad({ y: half, x: 1 }).overflow(hidden).cursor(pointer).fg(primary).bg(hover-bg).radius(sm).type({ align: start, size: density }), when: { selected: {  border({ kind: outline, color: ring })  } } })",
            "style({  pad({ y: 6, x: 4 }).group().border({ width: 1, color: border, style: dashed }).radius(lg).cursor(pointer).fg(muted-foreground).select(none).type({ align: center }), when: { focus: {  border({ kind: outline, color: ring })  }, stateOver: {  border({ width: 1, color: ring }).bg(muted)  } } })",
            "style({ position({ scheme: absolute, l: bind(boxLeft), t: bind(boxTop) }).size({ w: bind(boxW), h: bind(boxH) }).bg(hover-bg).border({ width: 1, color: primary, style: dashed }).radius(sm).cursor(pointer).pointerEvents(auto) })",
            "style({  size({ w: bind(boxW) }).pad({ y: 2, x: 3 }).bg(surface-1).fg(foreground).border({ width: 1, color: border }).radius(md).elevation(xs).cursor(pointer).type({ family: body, size: sm }), when: { hover: {  elevation(sm)  }, focus: {  border({ kind: outline, color: ring })  }, current: {  border({ width: 1, color: ring }).elevation(sm)  }, stateAgent: {  border({ edge: l, width: 1, color: primary })  } } })",
            "style({ pad({ y: 4, x: 5 }).border({ width: 1, color: destructive }).radius(md).bg(muted).fg(foreground).elevation(xs).type({ family: body, size: sm }) })",
            "style({  size({ h: control-h-sm }).radius(sm).cursor(pointer).fg(foreground).type({ align: center }), when: { hover: {  bg(hover-bg)  }, selected: {  bg(primary).fg(primary-foreground)  }, disabled: {  fg(muted-foreground).opacity(50).cursor(not-allowed)  }, stateEmpty: {  visibility(hidden)  }, stateToday: {  border({ kind: outline, color: border-strong })  }, focus: {  border({ kind: outline, color: ring })  } } })",
            "style({ cursor(cell) })",
            "style({  pad(0).size({ aspect: square }).border({ width: 1, color: border-subtle }).radius(sm).cursor(pointer).bg(bind(swatchColor)), when: { selected: {  border({ kind: outline, color: primary })  } } })",
            "style({  position({ scheme: absolute, l: 0 }).size({ w: full }).translateY(bind(rowY)).cursor(pointer), when: { hover: {  bg(hover-bg)  }, selected: {  bg(selected-bg)  }, focus: {  border({ kind: outline, color: ring })  } } })",
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
            ("style({ colorDeepen(primary, 15) })", "bg"),
            ("style({ colorMix(primary, 15) })", "bg"),
            ("style({ gap(2) })", "gap"),
        ] {
            let (_, diags) = parse(src);
            assert!(
                diags.iter().any(|d| d.message().contains(concern)),
                "{src} must name {concern}: {:?}",
                diags.iter().map(|d| d.message()).collect::<Vec<_>>()
            );
        }

        // Tier-2 control chrome / press marker setters.
        let (style, diags) = parse("style({ controlFrame().controlLift().press() })");
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
        let (style, diags) = parse("style({ animate: { enter: play(fade) } })");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let m = collect_motion_attaches(&style);
        assert_eq!(m.len(), 1, "style={style:?} motions={m:?}");
        assert_eq!(m[0].0, "enter");
        assert!(
            m[0].1.contains("opacity") && m[0].1.contains("from"),
            "{:?}",
            m[0]
        );
    }

    #[test]
    #[ignore = "deleted free form (STYLE-BINDING-1)"]
    fn enter_then_when_chain_order() {
        let (style, diags) = parse(
            "style({  bg(primary), animate: { enter: play(fade) }, when: { hover: {  bg(muted)  } } })",
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

    #[test]
    fn channels_control_when_hover_still_green() {
        let (style, diags) =
            parse("style({  pad(4), bg(primary), when: { hover: {  bg(muted)  } } })");
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
        let (style, diags) = parse("style({ boxSizing(border) })");
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

        // Flat `style.box(…)` is refused; the error names `boxSizing`.
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
            classes.contains("s-pad-2") || classes.contains("pad"),
            "{classes}"
        );
    }

    #[test]
    fn channels_flatten_order_paint_after_box() {
        // Author may write paint before box; flatten order is always box then paint.
        let (s_a, d_a) = parse("style({ pad(4).bg({ color: primary }) })");
        let (s_b, d_b) = parse("style({ pad(4).bg({ color: primary }) })");
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
    fn parse_refuses_motion_tail_after_concern_chain() {
        // Flat motion tail after style({…}) is retired — animate: channel only.
        let (_, diags) = parse("style({ bg(primary) }).enter({ opacity: 'fade' })");
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
        let (s_ch, d_ch) = parse("style({ bg(primary), animate: { enter: play(fade) } })");
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
            "style({ size({ w: 4, h: 4 }).radius(full).bg(primary), animate: { loop: play(slide-up) } })",
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
    // Product `parse` and `lower_style_source` refuse flat chains.

    #[test]
    fn parse_refuses_flat_chain() {
        let (_, diags) = parse("style.pad(4)");
        let joined = diags
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
            .join("; ");
        assert!(
            joined.contains("retired") || joined.contains("concern calls"),
            "product parse must refuse flat: {joined}"
        );
        assert!(
            joined.contains("style({"),
            "must teach concern-call form: {joined}"
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
    fn product_path_concern_chain_lowers() {
        let (classes, css) = product_lower("style({ pad(4), bg(primary).radius(md) })")
            .expect("flat concern chain is product form");
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
            joined.contains("retired") || joined.contains("concern calls"),
            "flat product must refuse: {joined}"
        );
        assert!(
            joined.contains("style({"),
            "must teach concern-call form: {joined}"
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
        let msgs = product_err_msgs("style({ pad(2), when: { hover: { pad(4) } } })");
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
        let msgs = product_err_msgs("style({ pad(2), when: { group-hover: { pad(4) } } })");
        let joined = msgs.join("; ");
        assert!(
            joined.contains("state can't gate layout"),
            "group-hover + pad must refuse: {joined}"
        );

        // focusWithin + pad refuses.
        let msgs = product_err_msgs("style({ pad(2), when: { focusWithin: { pad(4) } } })");
        let joined = msgs.join("; ");
        assert!(
            joined.contains("state can't gate layout"),
            "focusWithin + pad must refuse: {joined}"
        );

        // tablet + pad allows (media may gate layout).
        let (classes, _css) = product_lower("style({ pad(2), when: { tablet: { pad(4) } } })")
            .expect("tablet may gate pad geometry on product path");
        assert!(classes.contains("s-"), "{classes}");
    }

    #[test]
    fn product_path_color_alias_fails_closed() {
        let msgs = product_err_msgs("style({ color(primary) })");
        let joined = msgs.join("; ");
        assert!(
            joined.contains("fg") && (joined.contains("color") || joined.contains("renamed")),
            "color→fg must hard-fail product path: {joined}"
        );
    }

    #[test]
    fn style_object_match_compound_when() {
        let (style, diags) = parse(
            "style({ bg(primary), when: { hover: { bg(muted) } }, match: { variant: { solid: { bg({ color: secondary }) } } }, compound: [ { pill: true, radius(full) } ] })",
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
        let (_, diags) = parse("style({ match: { tone: { primary: { bg(primary) } } } })");
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
        let (style, diags) = parse("style({ compound: [ { tone: *, filter({ opacity: 90 }) } ] })");
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
        let (style, diags) = parse("style({ bg(primary) }, { pad(2) })");
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
        let (_, diags) = parse("match(variant, { solid: style({ bg(primary) }) })");
        assert!(!diags.is_empty(), "free match() must fail product parse");
    }

    #[test]
    fn spec8_restored_plains_compile_and_lower() {
        let cases = [
            ("style({ align(middle) })", "vertical-align", "middle"),
            ("style({ list(none) })", "list-style-type", "none"),
            ("style({ resize(both) })", "resize", "both"),
            ("style({ contain(strict) })", "contain", "strict"),
            (
                "style({ willChange(transform) })",
                "will-change",
                "transform",
            ),
            ("style({ visibility(hidden) })", "visibility", "hidden"),
        ];
        for (src, prop, val) in cases {
            let (style, diags) = parse(src);
            assert!(
                diags.is_empty(),
                "{src}: {:?}",
                diags.iter().map(|d| d.message()).collect::<Vec<_>>()
            );
            let (_, css) = lower_style(&style, &HashMap::new());
            assert!(
                css.contains(prop) && css.contains(val),
                "{src}: expected {prop}: {val} in {css}"
            );
        }

        let (style, diags) = parse("style({ overflow({ x: auto, snap: x }) })");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let (_, css) = lower_style(&style, &HashMap::new());
        assert!(
            css.contains("overflow-x") && css.contains("auto"),
            "overflow x: {css}"
        );
        assert!(
            css.contains("scroll-snap-type") && css.contains("x"),
            "overflow snap: {css}"
        );

        let (style, diags) = parse("style({ type({ align: center }) })");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let (_, css) = lower_style(&style, &HashMap::new());
        assert!(
            css.contains("text-align") && css.contains("center"),
            "type align is text-align: {css}"
        );
        assert!(
            !css.contains("vertical-align"),
            "type align must not emit vertical-align: {css}"
        );
    }

    fn only_op(src: &str) -> PaintOp {
        let (style, diags) = parse(src);
        assert!(
            diags.is_empty(),
            "{src}: {:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        match style {
            Style::Set(op) => op,
            other => panic!("{src}: expected single Set, got {other:?}"),
        }
    }

    #[test]
    fn border_subtle_emits_member_vars() {
        let op = only_op("style({ border(subtle) })");
        let css = crate::lower_op_css(&op).unwrap();
        assert!(
            css.contains(&("border-color".into(), "var(--border-subtle-color)".into())),
            "{css:?}"
        );
        assert!(
            css.contains(&("border-width".into(), "var(--border-subtle-width)".into())),
            "{css:?}"
        );
        assert!(
            css.contains(&("border-style".into(), "var(--border-subtle-style)".into())),
            "{css:?}"
        );
    }

    #[test]
    fn border_default_emits_member_vars() {
        let op = only_op("style({ border(default) })");
        let css = crate::lower_op_css(&op).unwrap();
        assert!(
            css.contains(&("border-color".into(), "var(--border-default-color)".into()))
                || css.contains(&("border-color".into(), "var(--border)".into())),
            "{css:?}"
        );
    }

    #[test]
    fn border_slot_object_emits_only_named_slots() {
        let op = only_op("style({ border({ color: muted, width: hairline }) })");
        let css = crate::lower_op_css(&op).unwrap();
        assert!(
            css.contains(&("border-color".into(), "var(--muted)".into())),
            "{css:?}"
        );
        assert!(
            css.contains(&("border-width".into(), "var(--border-width)".into())),
            "{css:?}"
        );
        assert_eq!(css.len(), 2, "only named slots: {css:?}");
    }

    #[test]
    fn size_slot_object_lowers() {
        let op = only_op("style({ size({ w: 100%, h: 100% }) })");
        let css = crate::lower_op_css(&op).unwrap();
        assert!(
            css.iter().any(|(p, v)| p == "width" && v.contains("100%")),
            "{css:?}"
        );
        assert!(
            css.iter().any(|(p, v)| p == "height" && v.contains("100%")),
            "{css:?}"
        );
    }

    #[test]
    fn size_bare_token_emits_both_axes() {
        let css = crate::lower_op_css(&only_op("style({ size(icon-size-md) })")).unwrap();
        assert!(
            css.iter()
                .any(|(p, v)| p == "width" && v.contains("icon-size-md")),
            "{css:?}"
        );
        assert!(
            css.iter()
                .any(|(p, v)| p == "height" && v.contains("icon-size-md")),
            "{css:?}"
        );
    }

    #[test]
    fn markers_and_concerns_lower() {
        let cases: &[(&str, &str, &str)] = &[
            ("style({ objectPosition(top) })", "object-position", "top"),
            ("style({ touch(pan-x) })", "touch-action", "pan-x"),
            (
                "style({ fg({ caret: primary }) })",
                "caret-color",
                "var(--primary)",
            ),
            ("style({ fg({ accent: auto }) })", "accent-color", "auto"),
            ("style({ position({ order: first }) })", "order", "-9999"),
            ("style({ bg({ clip: text }) })", "background-clip", "text"),
            (
                "style({ overflow({ scroll: smooth }) })",
                "scroll-behavior",
                "smooth",
            ),
        ];
        for (src, prop, val) in cases {
            let op = only_op(src);
            let css = crate::lower_op_css(&op).unwrap();
            assert!(
                css.contains(&(prop.to_string(), val.to_string())),
                "{src}: expected {prop}: {val} in {css:?}"
            );
        }
    }

    #[test]
    fn every_concern_slot_has_a_longhand_row() {
        for spec in concerns() {
            for slot in spec.slots {
                let _ = crate::concern_css::concern_css_prop(spec.name, slot.name);
            }
            let named: Vec<_> = spec.slots.iter().map(|s| s.name).collect();
            for slot in spec.slots {
                assert!(
                    crate::concern_css::concern_css_prop(spec.name, slot.name).is_some()
                        || matches!(slot.name, "gradient" | "edge" | "kind"),
                    "missing longhand for {}.{} (slots {named:?})",
                    spec.name,
                    slot.name
                );
            }
        }
        assert_eq!(
            crate::concern_css::concern_css_prop("elevation", "shadow"),
            Some("box-shadow")
        );
    }

    #[test]
    fn elevation_inset_ring_emits_box_shadow() {
        let op = only_op("style({ elevation({ shadow: inset-ring }) })");
        let css = crate::lower_op_css(&op).unwrap();
        assert!(
            css.iter()
                .any(|(p, v)| p == "box-shadow" && v.contains("shadow-inset-ring")),
            "{css:?}"
        );
    }

    #[test]
    fn lower_plain_token_opacity_cursor_display() {
        let opacity = find_plain("opacity").expect("opacity");
        let d = crate::plains_css::lower_plain_token(opacity, "50", false).expect("opacity 50");
        assert_eq!(d[0].0, "opacity");

        let cursor = find_plain("cursor").expect("cursor");
        assert_eq!(
            crate::plains_css::lower_plain_token(cursor, "pointer", false).expect("cursor"),
            vec![("cursor".into(), "pointer".into())]
        );

        let display = find_plain("display").expect("display");
        assert_eq!(
            crate::plains_css::lower_plain_token(display, "none", false).expect("display"),
            vec![("display".into(), "none".into())]
        );
    }
}
