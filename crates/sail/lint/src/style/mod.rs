//! Sail Style — compile-time style language.
//!
//! Setters CHAIN (`style.bg(primary).fg(onPrimary)`), guards WRAP
//! (`match`, `when`, `compound`), reactive paint via `bind(name)`.
//! One parser → IR → CSS emit. No JS runtime.

pub mod clash;
pub mod emit;
pub mod ir;
pub mod parse;

pub use ir::{PaintOp, Style, StyleDiagnostic};
pub use parse::{parse, parse_scope_consts, parse_style_list, uses_style_syntax};

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn only_errors(diags: &[StyleDiagnostic]) -> Vec<String> {
        diags.iter().map(|d| d.message()).collect()
    }

    #[test]
    fn setter_chain_parses_style_dot_form() {
        let (style, diags) = parse("style.bg(primary).fg(onPrimary).radius(md)");
        assert!(diags.is_empty(), "{:?}", only_errors(&diags));
        assert!(matches!(style, Style::Chain(_)));
    }

    #[test]
    fn match_parses_guard_not_setter() {
        let (style, diags) = parse("match(variant, { filled: style.bg(primary), tonal: style.bg(secondary) })");
        assert!(diags.is_empty(), "{:?}", only_errors(&diags));
        assert!(matches!(style, Style::Match { .. }));
    }

    #[test]
    fn when_parses_context_map() {
        let (style, diags) = parse("when({ hover: style.bg(muted), focus: style.ring(focusRing) })");
        assert!(diags.is_empty(), "{:?}", only_errors(&diags));
        assert!(matches!(style, Style::When { .. }));
    }

    #[test]
    fn compound_parses_multi_condition() {
        let (style, diags) = parse("compound({ variant: filled, size: lg }, style.padX(xl))");
        assert!(diags.is_empty(), "{:?}", only_errors(&diags));
        assert!(matches!(style, Style::Compound { .. }));
    }

    #[test]
    fn bind_parses_reactive_paint() {
        let (style, diags) = parse("bind(rowY)");
        assert!(diags.is_empty(), "{:?}", only_errors(&diags));
        assert!(matches!(style, Style::Bind { .. }));
    }

    #[test]
    fn scope_const_ref_parses() {
        let (style, diags) = parse("filled");
        assert!(diags.is_empty(), "{:?}", only_errors(&diags));
        assert!(matches!(style, Style::Ref { .. }));
    }

    #[test]
    fn scope_const_chain_parses() {
        let (style, diags) = parse("filled.padX(lg)");
        assert!(diags.is_empty(), "{:?}", only_errors(&diags));
        assert!(matches!(style, Style::Chain(_)));
    }

    #[test]
    fn style_list_parses() {
        let (items, diags) = parse_style_list("[style.bg(primary), match(variant, { filled: filled })]");
        assert!(diags.is_empty(), "{:?}", only_errors(&diags));
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn token_fail_closed_on_raw_color() {
        // The new syntax does not accept raw color strings as tokens.
        let (style, diags) = parse("style.bg(#ff00ff)");
        assert!(matches!(style, Style::Empty) || !diags.is_empty());
    }

    #[test]
    fn match_emits_attribute_selectors() {
        let mut consts = HashMap::new();
        consts.insert("filled".to_string(), parse("style.bg(primary)").0);
        let (style, _) = parse("match(variant, { filled: filled, tonal: style.bg(secondary) })");
        let rules = emit::emit_style(&style, &consts);
        let css = emit::render_rules(&rules);
        assert!(css.contains("[variant='filled']"), "match arm selector missing: {css}");
        assert!(css.contains("[variant='tonal']"), "match arm selector missing: {css}");
    }

    #[test]
    fn when_emits_pseudo_selectors() {
        let (style, _) = parse("when({ hover: style.bg(muted) })");
        let rules = emit::emit_style(&style, &HashMap::new());
        let css = emit::render_rules(&rules);
        assert!(css.contains(":hover"), "when hover did not emit pseudo: {css}");
    }

    #[test]
    fn clash_detection_flags_duplicate_setter() {
        let (style, _) = parse("style.bg(primary).bg(muted)");
        let diags = clash::detect_clashes(&style);
        let msgs: Vec<_> = diags.iter().map(|d| d.message()).collect();
        assert!(
            msgs.iter().any(|m| m.contains("clash")),
            "expected clash diagnostic, got {msgs:?}"
        );
    }

    #[test]
    fn debug_compound_tokens() {
        let tokens = parse::debug_tokens("compound({ variant: filled, size: lg }, style.padX(xl))");
        println!("Tokens: {:?}", tokens);
        assert!(!tokens.is_empty());
    }

    #[test]
    fn scope_consts_parse() {
        let (consts, diags) = parse_scope_consts("const filled = style.bg(primary).fg(onPrimary)");
        assert!(diags.is_empty(), "{:?}", only_errors(&diags));
        assert!(consts.contains_key("filled"));
    }
}
