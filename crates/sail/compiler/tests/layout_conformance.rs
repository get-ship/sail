//! Conformance pin: the taffy box tree must match the CSS the compiler emits.
//!
//! Each test preprocesses a `.sail` fragment, parses the emitted CSS, resolves
//! the same element in `sail-layout`, and asserts parity on the properties that
//! drive layout diagnostics. This catches drift between the compiler's lowering
//! and the layout crate's mapping table.

use std::collections::HashMap;

fn preprocess(src: &str) -> String {
    sail_compiler::parse::preprocess::preprocess(src).expect("preprocess")
}

fn split_html_css(preprocessed: &str) -> (&str, &str) {
    let Some(start) = preprocessed.find("<style>") else {
        return (preprocessed, "");
    };
    let css_start = start + "<style>".len();
    let Some(end) = preprocessed[css_start..].find("</style>") else {
        return (preprocessed, "");
    };
    let html = &preprocessed[end + css_start + "</style>".len()..];
    (html, &preprocessed[css_start..css_start + end])
}

fn parse_css(css: &str) -> HashMap<String, HashMap<String, String>> {
    let mut rules = HashMap::new();
    let mut depth = 0i32;
    let mut i = 0;
    let bytes = css.as_bytes();
    let mut selector_start = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'{' if depth == 0 => {
                let selector = css[selector_start..i].trim().to_string();
                let body_start = i + 1;
                let mut j = body_start;
                depth = 1;
                while j < bytes.len() && depth > 0 {
                    match bytes[j] {
                        b'{' => depth += 1,
                        b'}' => depth -= 1,
                        _ => {}
                    }
                    j += 1;
                }
                let body = &css[body_start..j - 1];
                let mut decls = HashMap::new();
                for decl in body.split(';') {
                    let decl = decl.trim();
                    if decl.is_empty() {
                        continue;
                    }
                    if let Some((k, v)) = decl.split_once(':') {
                        decls.insert(k.trim().to_string(), v.trim().to_string());
                    }
                }
                rules.insert(selector, decls);
                i = j;
                selector_start = i;
                continue;
            }
            _ => {}
        }
        i += 1;
    }
    rules
}

fn selector_has_class(selector: &str, class: &str) -> bool {
    // Treat `.` as the delimiter. Match `class` as a whole segment.
    selector
        .split('.')
        .any(|seg| seg == class || seg.starts_with(&format!("{class} ")))
}

fn rule_for_class<'a>(
    rules: &'a HashMap<String, HashMap<String, String>>,
    class: &str,
) -> Option<&'a HashMap<String, String>> {
    rules
        .iter()
        .find(|(sel, _)| selector_has_class(sel, class))
        .map(|(_, decls)| decls)
}

/// First `sx-*` class token on an element after Style preprocess.
fn extract_first_sx_class(preprocessed: &str) -> Option<String> {
    let (html, _) = split_html_css(preprocessed);
    let mut search = html;
    while let Some(i) = search.find("class=\"") {
        let rest = &search[i + 7..];
        let end = rest.find('"')?;
        for tok in rest[..end].split_whitespace() {
            if tok.starts_with("sx-") {
                return Some(tok.to_string());
            }
        }
        search = &rest[end + 1..];
    }
    None
}

fn resolve_element_with_class(
    preprocessed: &str,
    class: &str,
    viewport_width: f32,
) -> sail_layout::ResolvedStyle {
    let elements = sail_layout::html::parse_source(preprocessed);
    let idx = elements
        .iter()
        .position(|e| e.class.split_whitespace().any(|c| c == class))
        .unwrap_or_else(|| panic!("no element with class {class:?} in {preprocessed:?}"));
    sail_layout::style::resolve(&elements[idx], &elements, viewport_width)
}

fn resolve_child_of_class(
    preprocessed: &str,
    parent_class: &str,
    viewport_width: f32,
) -> sail_layout::ResolvedStyle {
    let elements = sail_layout::html::parse_source(preprocessed);
    let parent_idx = elements
        .iter()
        .position(|e| e.class.split_whitespace().any(|c| c == parent_class))
        .unwrap_or_else(|| panic!("no parent with class {parent_class:?}"));
    let child_idx = elements[parent_idx]
        .children
        .first()
        .copied()
        .unwrap_or_else(|| panic!("parent {parent_class:?} has no children"));
    sail_layout::style::resolve(&elements[child_idx], &elements, viewport_width)
}

#[test]
fn column_base_matches_css() {
    let src = r#"<column><p>hi</p></column>"#;
    let preprocessed = preprocess(src);
    let (_, css) = split_html_css(&preprocessed);
    let rules = parse_css(css);
    let col = rule_for_class(&rules, "sail-col").expect("sail-col rule present");
    assert_eq!(col["display"], "flex");
    assert_eq!(col["flex-direction"], "column");

    let style = resolve_element_with_class(&preprocessed, "sail-col", 1440.0);
    use sail_layout::taffy::prelude::*;
    assert_eq!(style.style.display, Display::Flex);
    assert_eq!(style.style.flex_direction, FlexDirection::Column);
}

#[test]
fn row_wraps_by_default_and_nowrap_overrides() {
    let src = r#"<row><p>x</p></row>"#;
    let preprocessed = preprocess(src);
    let (_, css) = split_html_css(&preprocessed);
    let rules = parse_css(css);
    let row = rule_for_class(&rules, "sail-row").expect("sail-row rule present");
    assert_eq!(row["display"], "flex");
    assert_eq!(row["flex-wrap"], "wrap");

    let style = resolve_element_with_class(&preprocessed, "sail-row", 1440.0);
    use sail_layout::taffy::prelude::*;
    assert_eq!(style.style.display, Display::Flex);
    assert_eq!(style.style.flex_wrap, FlexWrap::Wrap);

    let nowrap_src = r#"<row nowrap><p>x</p></row>"#;
    let nowrap_pre = preprocess(nowrap_src);
    let (_, nowrap_css) = split_html_css(&nowrap_pre);
    let nowrap_rules = parse_css(nowrap_css);
    let nowrap_rule = rule_for_class(&nowrap_rules, "sail-nowrap").expect("nowrap rule present");
    assert_eq!(nowrap_rule["flex-wrap"], "nowrap");
    let nowrap_style = resolve_element_with_class(&nowrap_pre, "sail-row", 1440.0);
    assert_eq!(nowrap_style.style.flex_wrap, FlexWrap::NoWrap);
}

#[test]
fn gap_modifier_matches_css() {
    let src = r#"<column gap="4"><p>x</p></column>"#;
    let preprocessed = preprocess(src);
    let (_, css) = split_html_css(&preprocessed);
    let rules = parse_css(css);
    let gap = rule_for_class(&rules, "sail-g-4").expect("sail-g-4 rule present");
    assert_eq!(gap["gap"], "var(--space-4)");

    let style = resolve_element_with_class(&preprocessed, "sail-col", 1440.0);
    use sail_layout::taffy::prelude::*;
    assert_eq!(style.style.gap.height, LengthPercentage::length(16.0));
    assert_eq!(style.style.gap.width, LengthPercentage::length(16.0));
}

#[test]
fn center_grid_alignment_matches_css() {
    let src = r#"<center>x</center>"#;
    let preprocessed = preprocess(src);
    let (_, css) = split_html_css(&preprocessed);
    let rules = parse_css(css);
    let center = rule_for_class(&rules, "sail-center").expect("sail-center rule present");
    assert_eq!(center["display"], "grid");
    assert_eq!(center["place-items"], "center");
    assert_eq!(center["place-content"], "center");
    assert_eq!(center["width"], "100%");

    let style = resolve_element_with_class(&preprocessed, "sail-center", 1440.0);
    use sail_layout::taffy::prelude::*;
    assert_eq!(style.style.display, Display::Grid);
    assert_eq!(style.style.align_items, Some(AlignItems::CENTER));
    assert_eq!(style.style.justify_content, Some(JustifyContent::CENTER));
    assert_eq!(style.style.size.width, Dimension::percent(1.0));
}

#[test]
fn grid_cols_count_matches_css() {
    let src = r#"<grid cols="3"><div>x</div></grid>"#;
    let preprocessed = preprocess(src);
    let (_, css) = split_html_css(&preprocessed);
    let rules = parse_css(css);
    let cols = rule_for_class(&rules, "sail-gc-3").expect("sail-gc-3 rule present");
    assert_eq!(cols["grid-template-columns"], "repeat(3,minmax(0,1fr))");

    let style = resolve_element_with_class(&preprocessed, "sail-grid", 1440.0);
    use sail_layout::taffy::style::{
        GridTemplateComponent, GridTemplateRepetition, RepetitionCount,
    };
    let col = style
        .style
        .grid_template_columns
        .first()
        .expect("grid has template columns");
    match col {
        GridTemplateComponent::Repeat(GridTemplateRepetition {
            count: RepetitionCount::Count(n),
            ..
        }) => assert_eq!(*n, 3),
        other => panic!("expected repeat count 3, got {other:?}"),
    }
}

#[test]
fn stack_child_layers_at_line_one() {
    let src = r#"<stack><span>x</span></stack>"#;
    let preprocessed = preprocess(src);
    let (_, css) = split_html_css(&preprocessed);
    let rules = parse_css(css);
    let child = rule_for_class(&rules, "sail-stackz")
        .and_then(|_| {
            // The child rule is keyed on the `.sail-stackz>*` selector.
            rules
                .iter()
                .find(|(sel, _)| sel.starts_with(".sail-stackz>*"))
                .map(|(_, decls)| decls)
        })
        .expect("stack child rule present");
    assert_eq!(child["grid-area"], "1/1");
    assert_eq!(child["position"], "relative");

    let style = resolve_child_of_class(&preprocessed, "sail-stackz", 1440.0);
    use sail_layout::taffy::prelude::*;
    assert_eq!(style.style.position, Position::Relative);
    assert_eq!(style.style.grid_row, line(1));
    assert_eq!(style.style.grid_column, line(1));
}

#[test]
fn flex_child_gets_min_width_zero() {
    let src = r#"<row><p>x</p></row>"#;
    let preprocessed = preprocess(src);
    let style = resolve_child_of_class(&preprocessed, "sail-row", 1440.0);
    use sail_layout::taffy::prelude::*;
    assert_eq!(style.style.min_size.width, Dimension::length(0.0));
}

#[test]
fn sx_width_full_matches_css() {
    let src = r#"<div> style.w(full) x</div>"#;
    let preprocessed = preprocess(src);
    let (_, css) = split_html_css(&preprocessed);
    let rules = parse_css(css);
    // Style emit mints atomic class names from paint ops (not fixed sx-wfull).
    let class = extract_first_sx_class(&preprocessed).expect("sx class present");
    let rule = rule_for_class(&rules, &class).expect("width rule present");
    assert_eq!(rule["width"], "100%");

    let style = resolve_element_with_class(&preprocessed, &class, 1440.0);
    use sail_layout::taffy::prelude::*;
    assert_eq!(style.style.size.width, Dimension::percent(1.0));
}

/// Product participation `grow` / Style emit → `sx-grow1` (not residual `sx-grow`).
#[test]
fn grow_attr_maps_to_taffy_flex_grow() {
    let src = r#"<column><div grow>body</div></column>"#;
    let preprocessed = preprocess(src);
    assert!(
        preprocessed.contains("sx-grow1"),
        "product grow class missing: {preprocessed}"
    );
    let style = resolve_element_with_class(&preprocessed, "sx-grow1", 1440.0);
    assert!(
        (style.style.flex_grow - 1.0).abs() < f32::EPSILON,
        "sx-grow1 must set flex_grow=1, got {}",
        style.style.flex_grow
    );
}

/// Product `style.minW(0)` / `min-w="0"` → `sx-minW0` (camelCase, not `sx-minw0`).
#[test]
fn min_w_attr_maps_to_taffy_min_width() {
    let src = r#"<div min-w="0">x</div>"#;
    let preprocessed = preprocess(src);
    assert!(
        preprocessed.contains("sx-minW0"),
        "product minW class missing: {preprocessed}"
    );
    let style = resolve_element_with_class(&preprocessed, "sx-minW0", 1440.0);
    use sail_layout::taffy::prelude::*;
    assert_eq!(style.style.min_size.width, Dimension::length(0.0));
}

/// Product `style.pad(4)` → `sx-pad4` (not residual `sx-p4`).
#[test]
fn style_pad_maps_to_taffy_padding() {
    let src = r#"<div> style.pad(4) x</div>"#;
    let preprocessed = preprocess(src);
    assert!(
        preprocessed.contains("sx-pad4"),
        "product pad class missing: {preprocessed}"
    );
    let style = resolve_element_with_class(&preprocessed, "sx-pad4", 1440.0);
    use sail_layout::taffy::prelude::*;
    assert_eq!(style.style.padding.top, LengthPercentage::length(16.0));
    assert_eq!(style.style.padding.left, LengthPercentage::length(16.0));
}

#[test]
fn sx_hidden_matches_css() {
    let src = r#"<div> style.display(none) x</div>"#;
    let preprocessed = preprocess(src);
    let (_, css) = split_html_css(&preprocessed);
    let rules = parse_css(css);
    // Find the atomic class that carries display:none (Style may mint hashed names).
    let (class, rule) = rules
        .iter()
        .find_map(|(sel, decls)| {
            if decls.get("display").map(|v| v.as_str()) == Some("none") {
                let c = sel
                    .trim()
                    .trim_start_matches('.')
                    .split(|ch: char| ch == ':' || ch == ' ' || ch == '[')
                    .next()
                    .unwrap_or("")
                    .to_string();
                if c.starts_with("sx-") {
                    return Some((c, decls));
                }
            }
            None
        })
        .expect(&format!(
            "display:none rule present in css:\n{css}\npre:\n{preprocessed}"
        ));
    assert_eq!(rule["display"], "none");

    let style = resolve_element_with_class(&preprocessed, &class, 1440.0);
    use sail_layout::taffy::prelude::*;
    assert_eq!(
        style.style.display,
        Display::None,
        "class={class} pre={preprocessed} css={css}"
    );
}

#[test]
fn grid_col_span_collapses_below_breakpoint() {
    let src = r#"<grid cols="2"><div col-span="2">x</div></grid>"#;
    let preprocessed = preprocess(src);

    let wide = resolve_child_of_class(&preprocessed, "sail-grid", 1440.0);
    use sail_layout::taffy::prelude::*;
    assert_eq!(wide.style.grid_column, span(2));

    let narrow = resolve_child_of_class(&preprocessed, "sail-grid", 600.0);
    assert_eq!(
        narrow.style.grid_column,
        sail_layout::taffy::style::Style::default().grid_column
    );
}

#[test]
fn check_all_emits_layout_warning_with_original_span() {
    // Collapsed parent + sized child: the zero-main-axis contract (an empty
    // child set totaling 0 is suppressed as analyzer noise by the layout
    // crate's gate — see check::layout tests).
    let src = "\n\n<column sx={style.h(0)}><div sx={style.h(20)}>x</div></column>";
    let diags = sail_compiler::check_all(src, None, &[], None);
    let layout = diags
        .iter()
        .find(|d| d.rule.as_deref() == Some("sail-layout"))
        .expect("expected a sail-layout warning");
    assert_eq!(
        layout.line, 3,
        "warning must point at the original <column> line"
    );
    assert!(layout.message.contains("0px on its main axis"));
}

#[test]
fn unknown_subtree_suppresses_layout_collapse() {
    for unknown in [
        "<sail-code-editor></sail-code-editor>",
        "<iframe src=\"x\"></iframe>",
    ] {
        let src = format!("<column gap=\"0\">{unknown}</column>");
        let diags = sail_compiler::check_all(&src, None, &[], None);
        assert!(
            !diags
                .iter()
                .any(|d| d.rule.as_deref() == Some("sail-layout")
                    && d.message.contains("0px on its main axis")),
            "unknown element {unknown:?} should suppress zero-main-axis"
        );
    }
}
