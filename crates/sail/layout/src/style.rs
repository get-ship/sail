//! Resolve a parsed element's `class=` + `style:` to a `taffy::Style`.
//!
//! The closed vocabulary is the *whole reason this works without a browser*.
//! `mapping.rs` owns the declarative table; this module walks the table, then
//! applies the scale-driven sx prefixes (`p-4`, `w-full`, …) through the same
//! `sail_lint::scales` resolver the compiler uses.

use crate::html::Element;
use crate::mapping::{
    grid_count_template, Declaration, Dimension as MappingDimension, Display as MappingDisplay,
    FlexDirection as MappingFlexDirection, FlexWrap as MappingFlexWrap, GridAutoFlow as MappingGridAutoFlow,
    GridTemplate, JustifyContent as MappingJustifyContent, Overflow as MappingOverflow,
    PaddingSide, Position as MappingPosition, Selector, SideSet, SxScaleRule, LAYOUT_RULES,
    SX_SCALE_RULES,
};
use sail_lint::scales;
use taffy::prelude::*;
use taffy::style::{GridTemplateComponent, GridTemplateRepetition, Overflow, RepetitionCount};

#[derive(Debug, Default)]
pub struct ResolvedStyle {
    pub style: Style,
    pub role: Role,
    pub visibility: Visibility,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Role {
    #[default]
    None,
    DocumentRoot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Visibility {
    #[default]
    Visible,
    Ignored,
}

impl ResolvedStyle {
    pub fn ignored() -> Self {
        Self { visibility: Visibility::Ignored, ..Default::default() }
    }
}

/// Resolve one element against the mapping table and the runtime scales.
/// `elements` is the full parsed forest (needed for parent-based child
/// conventions). `viewport_width` decides the `data-col-span` breakpoint.
pub fn resolve(elem: &Element, elements: &[Element], viewport_width: f32) -> ResolvedStyle {
    if elem.tag.is_empty() {
        return ResolvedStyle::ignored();
    }
    let classes: Vec<&str> = elem.class.split_whitespace().collect();
    let parent = elem.parent.and_then(|p| elements.get(p));
    let parent_classes: Vec<&str> = parent
        .map(|p| p.class.split_whitespace().collect())
        .unwrap_or_default();

    let mut style = Style::default();

    // 1. Static layout rules.
    for rule in LAYOUT_RULES {
        if rule_matches(rule.selector, &classes, &parent_classes) {
            apply_declarations(&mut style,
                rule.declarations,
                elem,
                viewport_width,
            );
        }
    }

    // 2. Grid column/row count modifiers (`sail-gc-N`, `sail-gr-N`).
    for c in &classes {
        if let Some((is_cols, template)) = grid_count_from_class(c) {
            apply_grid_template(&mut style, is_cols, template);
        }
    }

    // 3. Scale-driven sx prefixes: padding and sizing.
    for c in &classes {
        if let Some((tail, rule)) = match_sx_scale_rule(c) {
            apply_sx_scale(&mut style, rule, tail, &elem.style);
        }
    }

    // 4. Fallback `style:` width / height declarations (e.g. `style:w="full"`).
    apply_style_size(&mut style, &elem.style);

    // Taffy defaults that the SAIL vocabulary never leaves unset.
    if style.align_content.is_none() {
        style.align_content = Some(AlignContent::STRETCH);
    }

    ResolvedStyle { style, role: Role::None, visibility: Visibility::Visible }
}

fn rule_matches(selector: Selector, classes: &[&str], parent_classes: &[&str]) -> bool {
    match selector {
        Selector::Base(s) => classes.contains(&s),
        Selector::Modifier { prefix, value } => classes.iter().any(|c| {
            let rest = c.strip_prefix(prefix);
            matches!(rest, Some(r) if r == value)
        }),
        Selector::Sx(s) => classes.iter().any(|c| c.strip_prefix("sx-") == Some(s)),
        Selector::ParentBase(s) => parent_classes.contains(&s),
        Selector::Any(s) => classes.contains(&s),
    }
}

fn apply_declarations(
    style: &mut Style,
    declarations: &[Declaration],
    elem: &Element,
    viewport_width: f32,
) {
    for decl in declarations {
        match *decl {
            Declaration::Display(d) => style.display = to_taffy_display(d),
            Declaration::FlexDirection(d) => style.flex_direction = to_taffy_flex_direction(d),
            Declaration::FlexWrap(w) => style.flex_wrap = to_taffy_flex_wrap(w),
            Declaration::Gap(row, col) => {
                style.gap = Size {
                    width: LengthPercentage::length(col),
                    height: LengthPercentage::length(row),
                };
            }
            Declaration::AlignItems(a) => style.align_items = Some(to_taffy_align_items(a)),
            Declaration::JustifyContent(j) => style.justify_content = Some(to_taffy_justify_content(j)),
            Declaration::Position(p) => style.position = to_taffy_position(p),
            Declaration::Overflow { x, y } => style.overflow = taffy::Point {
                x: to_taffy_overflow(x),
                y: to_taffy_overflow(y),
            },
            Declaration::FlexGrow(g) => style.flex_grow = g,
            Declaration::FlexShrink(s) => style.flex_shrink = s,
            Declaration::MinWidth(d) => style.min_size.width = to_taffy_dimension(d),
            Declaration::MinHeight(d) => style.min_size.height = to_taffy_dimension(d),
            Declaration::Width(d) => style.size.width = to_taffy_dimension(d),
            Declaration::Height(d) => style.size.height = to_taffy_dimension(d),
            Declaration::MaxWidth(d) => style.max_size.width = to_taffy_dimension(d),
            Declaration::MaxHeight(d) => style.max_size.height = to_taffy_dimension(d),
            Declaration::Padding(side, px) => apply_padding(style, side, px),
            Declaration::StackLayer => {
                style.position = Position::Relative;
                style.grid_row = line(1);
                style.grid_column = line(1);
            }
            Declaration::GridColumnSpan(n) => {
                if viewport_width >= 720.0 {
                    style.grid_column = span(n);
                }
            }
            Declaration::GridRowSpan(n) => {
                if viewport_width >= 720.0 {
                    style.grid_row = span(n);
                }
            }
            Declaration::GridTemplateColumns(template) => {
                style.grid_template_columns = build_grid_template(template);
            }
            Declaration::GridTemplateRows(template) => {
                style.grid_template_rows = build_grid_template(template);
            }
            Declaration::GridAutoFlow(flow) => style.grid_auto_flow = to_taffy_grid_auto_flow(flow),
        }
    }

    // Grid span from `data-col-span` / `data-row-span` is applied after all
    // class rules so it can be collapsed below the breakpoint.
    if viewport_width >= 720.0 {
        if let Some(n) = elem.col_span {
            style.grid_column = span(n);
        }
        if let Some(n) = elem.row_span {
            style.grid_row = span(n);
        }
    }
}

fn to_taffy_display(d: MappingDisplay) -> Display {
    match d {
        MappingDisplay::Flex => Display::Flex,
        MappingDisplay::Grid => Display::Grid,
        MappingDisplay::Block => Display::Block,
        MappingDisplay::None => Display::None,
    }
}

fn to_taffy_flex_direction(d: MappingFlexDirection) -> FlexDirection {
    match d {
        MappingFlexDirection::Row => FlexDirection::Row,
        MappingFlexDirection::RowReverse => FlexDirection::RowReverse,
        MappingFlexDirection::Column => FlexDirection::Column,
        MappingFlexDirection::ColumnReverse => FlexDirection::ColumnReverse,
    }
}

fn to_taffy_flex_wrap(w: MappingFlexWrap) -> FlexWrap {
    match w {
        MappingFlexWrap::Wrap => FlexWrap::Wrap,
        MappingFlexWrap::NoWrap => FlexWrap::NoWrap,
        MappingFlexWrap::WrapReverse => FlexWrap::WrapReverse,
    }
}

fn to_taffy_align_items(a: crate::mapping::AlignItems) -> AlignItems {
    use crate::mapping::AlignItems as A;
    match a {
        A::Start => AlignItems::START,
        A::End => AlignItems::END,
        A::Center => AlignItems::CENTER,
        A::Stretch => AlignItems::STRETCH,
        A::Baseline => AlignItems::BASELINE,
    }
}

fn to_taffy_justify_content(j: MappingJustifyContent) -> JustifyContent {
    use crate::mapping::JustifyContent as J;
    match j {
        J::Start => JustifyContent::START,
        J::End => JustifyContent::END,
        J::Center => JustifyContent::CENTER,
        J::SpaceBetween => JustifyContent::SPACE_BETWEEN,
        J::SpaceAround => JustifyContent::SPACE_AROUND,
        J::SpaceEvenly => JustifyContent::SPACE_EVENLY,
    }
}

fn to_taffy_position(p: MappingPosition) -> Position {
    match p {
        MappingPosition::Relative => Position::Relative,
        MappingPosition::Absolute => Position::Absolute,
    }
}

fn to_taffy_overflow(o: MappingOverflow) -> Overflow {
    match o {
        MappingOverflow::Visible => Overflow::Visible,
        MappingOverflow::Hidden => Overflow::Hidden,
        MappingOverflow::Clip => Overflow::Clip,
        MappingOverflow::Scroll => Overflow::Scroll,
    }
}

fn to_taffy_dimension(d: MappingDimension) -> Dimension {
    match d {
        MappingDimension::Length(n) => Dimension::length(n),
        MappingDimension::Percent(n) => Dimension::percent(n),
        MappingDimension::Auto => Dimension::auto(),
    }
}

fn to_taffy_grid_auto_flow(f: MappingGridAutoFlow) -> GridAutoFlow {
    match f {
        MappingGridAutoFlow::Row => GridAutoFlow::Row,
        MappingGridAutoFlow::RowDense => GridAutoFlow::RowDense,
        MappingGridAutoFlow::Column => GridAutoFlow::Column,
        MappingGridAutoFlow::ColumnDense => GridAutoFlow::ColumnDense,
    }
}

fn build_grid_template(template: GridTemplate) -> Vec<GridTemplateComponent<String>> {
    match template {
        GridTemplate::Fixed(n) => {
            vec![GridTemplateComponent::Repeat(GridTemplateRepetition {
                count: RepetitionCount::Count(n),
                tracks: vec![minmax(zero(), fr(1.0))],
                line_names: Vec::new(),
            })]
        }
        GridTemplate::AutoFillMin(px) => {
            vec![GridTemplateComponent::Repeat(GridTemplateRepetition {
                count: RepetitionCount::AutoFill,
                tracks: vec![minmax(length(px), fr(1.0))],
                line_names: Vec::new(),
            })]
        }
        GridTemplate::Var => Vec::new(),
    }
}

fn apply_grid_template(style: &mut Style, is_cols: bool, template: GridTemplate) {
    if is_cols {
        style.grid_template_columns = build_grid_template(template);
    } else {
        style.grid_template_rows = build_grid_template(template);
    }
}

fn apply_padding(style: &mut Style, side: PaddingSide, px: f32) {
    let v = LengthPercentage::length(px);
    match side {
        PaddingSide::All => style.padding = taffy::Rect { top: v, right: v, bottom: v, left: v },
        PaddingSide::X => {
            style.padding.right = v;
            style.padding.left = v;
        }
        PaddingSide::Y => {
            style.padding.top = v;
            style.padding.bottom = v;
        }
        PaddingSide::T => style.padding.top = v,
        PaddingSide::R => style.padding.right = v,
        PaddingSide::B => style.padding.bottom = v,
        PaddingSide::L => style.padding.left = v,
    }
}

fn grid_count_from_class(c: &str) -> Option<(bool, GridTemplate)> {
    if let Some((_, value)) = c.split_once("sail-gc-") {
        return grid_count_template("sail-gc-", value).map(|(_, t)| (true, t));
    }
    if let Some((_, value)) = c.split_once("sail-gr-") {
        return grid_count_template("sail-gr-", value).map(|(_, t)| (false, t));
    }
    None
}

fn match_sx_scale_rule(c: &str) -> Option<(&str, SxScaleRule)> {
    for (prefix, rule) in SX_SCALE_RULES {
        if let Some(tail) = c.strip_prefix(prefix) {
            return Some((tail, *rule));
        }
    }
    None
}

fn apply_sx_scale(
    style: &mut Style,
    rule: SxScaleRule,
    value: &str,
    _style_map: &std::collections::HashMap<String, String>,
) {
    match rule {
        SxScaleRule::Padding(side) => {
            let px = padding_token_to_px(value);
            apply_padding(style, map_padding_side(side), px);
        }
        SxScaleRule::Width => style.size.width = size_token_to_dimension(value),
        SxScaleRule::Height => style.size.height = size_token_to_dimension(value),
        SxScaleRule::MinWidth => style.min_size.width = size_token_to_dimension(value),
        SxScaleRule::MinHeight => style.min_size.height = size_token_to_dimension(value),
        SxScaleRule::MaxWidth => style.max_size.width = size_token_to_dimension(value),
        SxScaleRule::MaxHeight => style.max_size.height = size_token_to_dimension(value),
    }
}

fn map_padding_side(side: SideSet) -> PaddingSide {
    match side {
        SideSet::All => PaddingSide::All,
        SideSet::X => PaddingSide::X,
        SideSet::Y => PaddingSide::Y,
        SideSet::T => PaddingSide::T,
        SideSet::R => PaddingSide::R,
        SideSet::B => PaddingSide::B,
        SideSet::L => PaddingSide::L,
    }
}

fn apply_style_size(style: &mut Style, map: &std::collections::HashMap<String, String>) {
    if let Some(v) = map.get("w") {
        if let Some(d) = parse_css_length(v) {
            style.size.width = d;
        }
    }
    if let Some(v) = map.get("h") {
        if let Some(d) = parse_css_length(v) {
            style.size.height = d;
        }
    }
}

fn token_px_for_space(step: u32) -> f32 {
    match step {
        1 => 4.0,
        2 => 8.0,
        3 => 12.0,
        4 => 16.0,
        5 => 20.0,
        6 => 24.0,
        7 => 28.0,
        8 => 32.0,
        9 => 36.0,
        10 => 40.0,
        11 => 44.0,
        12 => 48.0,
        14 => 56.0,
        16 => 64.0,
        20 => 80.0,
        24 => 96.0,
        28 => 112.0,
        32 => 128.0,
        _ => step as f32 * 4.0,
    }
}

fn padding_token_to_px(value: &str) -> f32 {
    if value == "0" {
        return 0.0;
    }
    if value == "density" {
        return 8.0;
    }
    if let Ok(step) = value.parse::<u32>() {
        return token_px_for_space(step);
    }
    0.0
}

fn size_token_to_dimension(value: &str) -> Dimension {
    match value {
        "0" => Dimension::length(0.0),
        "auto" => Dimension::auto(),
        "full" => Dimension::percent(1.0),
        "screen" => Dimension::percent(1.0),
        "fill" => Dimension::percent(1.0),
        "fit" => Dimension::auto(),
        "min" => Dimension::auto(),
        "max" => Dimension::auto(),
        other => match scales::lower("size", other) {
            Some(s) => parse_css_length(&s).unwrap_or(Dimension::auto()),
            None => Dimension::auto(),
        },
    }
}

fn parse_css_length(v: &str) -> Option<Dimension> {
    let v = v.trim();
    if v.is_empty() {
        return None;
    }
    if v == "0" {
        return Some(Dimension::length(0.0));
    }
    if let Some(rest) = v.strip_suffix("px") {
        let n: f32 = rest.parse().ok()?;
        return Some(Dimension::length(n));
    }
    if let Some(rest) = v.strip_suffix("rem") {
        let n: f32 = rest.parse().ok()?;
        return Some(Dimension::length(n * 16.0));
    }
    if v.ends_with('%') {
        if let Some(rest) = v.strip_suffix('%') {
            let n: f32 = rest.parse().ok()?;
            return Some(Dimension::percent(n / 100.0));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn column_default_resolves_to_flex_column() {
        let elem = Element {
            tag: "div".into(),
            class: "sail-col sail-g-4 sail-ai-stretch".into(),
            ..Default::default()
        };
        let resolved = resolve(&elem, &[Element::default()], 1440.0);
        assert_eq!(resolved.style.display, Display::Flex);
        assert_eq!(resolved.style.flex_direction, FlexDirection::Column);
        assert_eq!(resolved.style.gap.height, LengthPercentage::length(16.0));
    }

    #[test]
    fn row_wraps_by_default_and_nowrap_overrides() {
        let row = Element {
            tag: "div".into(),
            class: "sail-row".into(),
            ..Default::default()
        };
        let nowrap = Element {
            tag: "div".into(),
            class: "sail-row sail-nowrap".into(),
            ..Default::default()
        };
        assert_eq!(resolve(&row, &[Element::default()], 1440.0).style.flex_wrap, FlexWrap::Wrap);
        assert_eq!(resolve(&nowrap, &[Element::default()], 1440.0).style.flex_wrap, FlexWrap::NoWrap);
    }

    #[test]
    fn child_of_layout_gets_min_width_zero() {
        let parent = Element {
            tag: "div".into(),
            class: "sail-col".into(),
            ..Default::default()
        };
        let child = Element {
            tag: "p".into(),
            class: String::new(),
            parent: Some(1),
            ..Default::default()
        };
        let elements = vec![Element::default(), parent, child];
        let resolved = resolve(&elements[2], &elements, 1440.0);
        assert_eq!(resolved.style.min_size.width, Dimension::length(0.0));
    }

    #[test]
    fn stack_child_layers_at_line_one() {
        let parent = Element {
            tag: "div".into(),
            class: "sail-stackz".into(),
            ..Default::default()
        };
        let child = Element {
            tag: "span".into(),
            class: String::new(),
            parent: Some(1),
            ..Default::default()
        };
        let elements = vec![Element::default(), parent, child];
        let resolved = resolve(&elements[2], &elements, 1440.0);
        assert_eq!(resolved.style.position, Position::Relative);
        assert_eq!(resolved.style.grid_row, line(1));
        assert_eq!(resolved.style.grid_column, line(1));
    }

    #[test]
    fn grid_span_collapses_below_breakpoint() {
        let parent = Element {
            tag: "div".into(),
            class: "sail-grid".into(),
            ..Default::default()
        };
        let child = Element {
            tag: "div".into(),
            class: String::new(),
            parent: Some(1),
            col_span: Some(2),
            ..Default::default()
        };
        let elements = vec![Element::default(), parent, child];
        let wide = resolve(&elements[2], &elements, 1440.0);
        let narrow = resolve(&elements[2], &elements, 600.0);
        assert_eq!(wide.style.grid_column, span(2));
        assert_eq!(narrow.style.grid_column, Style::default().grid_column);
    }

    #[test]
    fn sx_padding_resolves_to_rect() {
        let elem = Element {
            tag: "div".into(),
            class: "sx-p4".into(),
            ..Default::default()
        };
        let resolved = resolve(&elem, &[Element::default()], 1440.0);
        assert_eq!(resolved.style.padding.top, LengthPercentage::length(16.0));
        assert_eq!(resolved.style.padding.left, LengthPercentage::length(16.0));
    }

    #[test]
    fn sx_width_full_resolves_to_percent() {
        let elem = Element {
            tag: "div".into(),
            class: "sx-wfull".into(),
            ..Default::default()
        };
        let resolved = resolve(&elem, &[Element::default()], 1440.0);
        assert_eq!(resolved.style.size.width, Dimension::percent(1.0));
    }

    #[test]
    fn sx_hidden_sets_display_none() {
        let elem = Element {
            tag: "div".into(),
            class: "sail-col sx-hidden".into(),
            ..Default::default()
        };
        let resolved = resolve(&elem, &[Element::default()], 1440.0);
        assert_eq!(resolved.style.display, Display::None);
    }
}
