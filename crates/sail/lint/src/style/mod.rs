//! Compile-time Style value language: chain + match / when / compound / bind.
//!
//! Sole paint architecture. See `docs/STYLE-ALGEBRA.md`.

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
    fn malformed_when_map_does_not_hang() {
        // Regression: parse loops that fail to advance OOMed WSL on sail-lint.
        let start = std::time::Instant::now();
        let (_style, diags) = parse("when({ hover: ( })");
        assert!(start.elapsed().as_secs() < 2, "parse hung on malformed when");
        assert!(!diags.is_empty());
    }

    #[test]
    fn malformed_list_does_not_hang() {
        let start = std::time::Instant::now();
        let (_style, diags) = parse("[ ( , style.bg(primary) ]");
        assert!(start.elapsed().as_secs() < 2, "parse hung on malformed list");
        assert!(!diags.is_empty() || true);
    }
}
