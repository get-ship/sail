// FILE_CAP_OVERRIDE: layout resolve + mapping application + structural unit tests; scale atoms live in style_scale.rs.
//! Resolve a parsed element's Style IR + layout primitives to a `taffy::Style`.
//!
//! Box and type measure come from Style tokens on the element. `mapping.rs`
//! owns layout primitives (`sail-col`, gap, align). Scale atoms fill pad/size
//! only when Style has not already resolved those tokens.

use crate::html::Element;
use crate::mapping::{
    Declaration, Dimension as MappingDimension, Display as MappingDisplay,
    FlexDirection as MappingFlexDirection, FlexWrap as MappingFlexWrap,
    GridAutoFlow as MappingGridAutoFlow, GridTemplate, JustifyContent as MappingJustifyContent,
    LAYOUT_RULES, Overflow as MappingOverflow, PaddingSide, Position as MappingPosition, Selector,
    TrackRecipe, grid_count_template, grid_recipe_template,
};
use crate::style_scale;
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
        Self {
            visibility: Visibility::Ignored,
            ..Default::default()
        }
    }
}

/// Resolve one element against the mapping table and the runtime scales.
/// `elements` is the full parsed forest (needed for parent-based child
/// conventions). `viewport_width` decides the `col-span` breakpoint.
pub fn resolve(elem: &Element, elements: &[Element], viewport_width: f32) -> ResolvedStyle {
    if elem.tag.is_empty() {
        return ResolvedStyle::ignored();
    }
    let classes: Vec<&str> = elem.class.split_whitespace().collect();
    let parent = elem.parent.and_then(|p| elements.get(p));
    let parent_classes: Vec<&str> = parent
        .map(|p| p.class.split_whitespace().collect())
        .unwrap_or_default();

    // Lowered HTML truth: a bare element is CSS `display:block`, not taffy's
    // default flex-row. Without this, L1 hosts (card/dialog/field chrome) that
    // don't stamp a flex/grid class flow their children side-by-side — diverging
    // from the browser and the skin CSS. Layout primitives (sail-col/row/grid)
    // and Style display tokens overwrite this below.
    let mut style = Style {
        display: Display::Block,
        ..Default::default()
    };

    // 1. Static layout rules (primitives + modifiers, not Style atoms).
    for rule in LAYOUT_RULES {
        if rule_matches(rule.selector, &classes, &parent_classes) {
            apply_declarations(&mut style, rule.declarations, elem, viewport_width);
        }
    }

    // 2. Grid column/row count modifiers (`sail-gc-N`, `sail-gr-N`) and
    //    closed `tracks` recipes / var refs (`sail-gt-*`).
    for c in &classes {
        if let Some((is_cols, template)) = grid_count_from_class(c) {
            apply_grid_template(&mut style, is_cols, template);
        }
        if let Some(template) = grid_recipe_from_class(c) {
            apply_grid_template(&mut style, true, template);
        }
    }

    // 3. Style-resolved box tokens (display / grow / overflow / position / pad / size).
    apply_resolved_box(&mut style, &elem.box_style);

    // 4. Scale-driven pad/size atoms, skipped when Style already resolved them.
    //    L1 skins still stamp compound classes (`s-sizemd-minH…`) because
    //    Match/Compound arms are not yet BoxHost. Do not delete this scrape
    //    without emitting BoxHost from the skin Style + host attrs.
    style_scale::apply_scale_classes(&mut style, &classes, &elem.attrs, &elem.box_style);

    // 5. Finite box atoms on lowered HTML that never carried Style IR
    //    (canary / kinds + L1 display/grow stamps). Style-set fields win.
    apply_class_box_atoms(&mut style, &classes, &elem.box_style);

    // 6. Fallback `style:` width / height declarations (e.g. `style:w="full"`).
    apply_style_size(&mut style, &elem.style);

    // Taffy defaults that the SAIL vocabulary never leaves unset.
    if style.align_content.is_none() {
        style.align_content = Some(AlignContent::STRETCH);
    }

    ResolvedStyle {
        style,
        role: Role::None,
        visibility: Visibility::Visible,
    }
}

fn rule_matches(selector: Selector, classes: &[&str], parent_classes: &[&str]) -> bool {
    match selector {
        Selector::Base(s) => classes.contains(&s),
        Selector::Modifier { prefix, value } => classes.iter().any(|c| {
            let rest = c.strip_prefix(prefix);
            matches!(rest, Some(r) if r == value)
        }),
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
            Declaration::JustifyContent(j) => {
                style.justify_content = Some(to_taffy_justify_content(j))
            }
            Declaration::Position(p) => style.position = to_taffy_position(p),
            Declaration::Overflow { x, y } => {
                style.overflow = taffy::Point {
                    x: to_taffy_overflow(x),
                    y: to_taffy_overflow(y),
                }
            }
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

    // Grid span from `col-span` / `row-span` is applied after all
    // class rules so it can be collapsed below the breakpoint.
    if viewport_width >= 720.0 {
        if let Some(n) = elem.col_span {
            style.grid_column = span(n);
        }
        if let Some(n) = elem.row_span {
            style.grid_row = span(n);
        }
    }
    // Language `col` / `row` on a grid child — same cell is overlap.
    if let Some(c) = elem.attrs.get("col") {
        if let Ok(n) = c.parse::<i16>() {
            if n > 0 {
                style.grid_column = line(n);
            }
        }
    }
    if let Some(r) = elem.attrs.get("row") {
        if let Ok(n) = r.parse::<i16>() {
            if n > 0 {
                style.grid_row = line(n);
            }
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
        GridTemplate::Recipe(recipe) => match recipe {
            // auto 1fr auto — content | flex | content
            TrackRecipe::AutoFrAuto => vec![
                GridTemplateComponent::Single(auto()),
                GridTemplateComponent::Single(fr(1.0)),
                GridTemplateComponent::Single(auto()),
            ],
            // 1fr 1fr
            TrackRecipe::FrFr => vec![
                GridTemplateComponent::Single(fr(1.0)),
                GridTemplateComponent::Single(fr(1.0)),
            ],
        },
    }
}

fn grid_recipe_from_class(c: &str) -> Option<GridTemplate> {
    if !c.starts_with("sail-gt-") {
        return None;
    }
    grid_recipe_template(c)
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
        PaddingSide::All => {
            style.padding = taffy::Rect {
                top: v,
                right: v,
                bottom: v,
                left: v,
            }
        }
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

fn apply_resolved_box(style: &mut Style, box_style: &crate::box_ir::ResolvedBox) {
    if let Some(d) = box_style.display.as_deref() {
        style.display = display_token(d);
    }
    if let Some(g) = box_style.grow {
        style.flex_grow = g;
    }
    if let Some(s) = box_style.shrink {
        style.flex_shrink = s;
    }
    if let Some(x) = box_style.overflow_x.as_deref() {
        style.overflow.x = overflow_token(x);
    }
    if let Some(y) = box_style.overflow_y.as_deref() {
        style.overflow.y = overflow_token(y);
    }
    if box_style.truncate {
        if matches!(style.overflow.x, Overflow::Visible) {
            style.overflow.x = Overflow::Hidden;
        }
        if matches!(style.overflow.y, Overflow::Visible) {
            style.overflow.y = Overflow::Hidden;
        }
    }
    if let Some(p) = box_style.position.as_deref() {
        style.position = position_token(p);
    }
    apply_pad_tokens(style, &box_style.pad);
    if let Some(w) = box_style.width.as_deref() {
        style.size.width = style_scale::size_token_to_dimension(w);
    }
    if let Some(h) = box_style.height.as_deref() {
        style.size.height = style_scale::size_token_to_dimension(h);
    }
    if let Some(w) = box_style.min_width.as_deref() {
        style.min_size.width = style_scale::size_token_to_dimension(w);
    }
    if let Some(h) = box_style.min_height.as_deref() {
        style.min_size.height = style_scale::size_token_to_dimension(h);
    }
    if let Some(w) = box_style.max_width.as_deref() {
        style.max_size.width = style_scale::size_token_to_dimension(w);
    }
    if let Some(h) = box_style.max_height.as_deref() {
        style.max_size.height = style_scale::size_token_to_dimension(h);
    }
}

fn apply_pad_tokens(style: &mut Style, pad: &crate::box_ir::PadTokens) {
    if let Some(v) = pad.all.as_deref() {
        apply_padding(style, PaddingSide::All, style_scale::padding_token_to_px(v));
    }
    if let Some(v) = pad.x.as_deref() {
        apply_padding(style, PaddingSide::X, style_scale::padding_token_to_px(v));
    }
    if let Some(v) = pad.y.as_deref() {
        apply_padding(style, PaddingSide::Y, style_scale::padding_token_to_px(v));
    }
    if let Some(v) = pad.t.as_deref() {
        apply_padding(style, PaddingSide::T, style_scale::padding_token_to_px(v));
    }
    if let Some(v) = pad.r.as_deref() {
        apply_padding(style, PaddingSide::R, style_scale::padding_token_to_px(v));
    }
    if let Some(v) = pad.b.as_deref() {
        apply_padding(style, PaddingSide::B, style_scale::padding_token_to_px(v));
    }
    if let Some(v) = pad.l.as_deref() {
        apply_padding(style, PaddingSide::L, style_scale::padding_token_to_px(v));
    }
}

fn display_token(tok: &str) -> Display {
    match tok {
        "none" | "contents" => Display::None,
        "flex" | "inline-flex" | "inlineflex" | "inline" => Display::Flex,
        "grid" | "inline-grid" | "inlinegrid" => Display::Grid,
        _ => Display::Block,
    }
}

fn overflow_token(tok: &str) -> Overflow {
    match tok {
        "hidden" => Overflow::Hidden,
        "clip" => Overflow::Clip,
        "scroll" | "auto" => Overflow::Scroll,
        _ => Overflow::Visible,
    }
}

fn position_token(tok: &str) -> Position {
    match tok {
        "absolute" | "fixed" => Position::Absolute,
        _ => Position::Relative,
    }
}

/// Fill unset Style-IR box fields from finite lowered atoms on the class list.
/// Used by already-lowered HTML (canary / kinds). Style-set fields win.
fn apply_class_box_atoms(
    style: &mut Style,
    classes: &[&str],
    resolved: &crate::box_ir::ResolvedBox,
) {
    for c in classes {
        let Some(atom) = c.strip_prefix("s-") else {
            continue;
        };
        let atom = atom.to_ascii_lowercase();
        if resolved.display.is_none() {
            if let Some(d) = class_display_atom(&atom) {
                style.display = d;
            }
        }
        if resolved.grow.is_none() {
            if let Some(g) = class_grow_atom(&atom) {
                style.flex_grow = g;
            }
        }
        if resolved.shrink.is_none() {
            if let Some(s) = class_shrink_atom(&atom) {
                style.flex_shrink = s;
            }
        }
        if let Some((x, y)) = class_overflow_atom(&atom) {
            if resolved.overflow_x.is_none() {
                if let Some(v) = x {
                    style.overflow.x = v;
                }
            }
            if resolved.overflow_y.is_none() {
                if let Some(v) = y {
                    style.overflow.y = v;
                }
            }
        }
        if resolved.position.is_none() {
            if let Some(p) = class_position_atom(&atom) {
                style.position = p;
            }
        }
    }
}

fn class_display_atom(atom: &str) -> Option<Display> {
    match atom {
        "hidden" | "displaynone" | "displaycontents" | "contents" => Some(Display::None),
        "displayinlineflex" | "displayflex" | "inline" => Some(Display::Flex),
        "displaygrid" | "displayinlinegrid" => Some(Display::Grid),
        "displayinlineblock" | "displayblock" | "inlineblock" | "block" => Some(Display::Block),
        _ => None,
    }
}

fn class_grow_atom(atom: &str) -> Option<f32> {
    match atom {
        "grow1" | "grow" => Some(1.0),
        "grow0" => Some(0.0),
        _ => None,
    }
}

fn class_shrink_atom(atom: &str) -> Option<f32> {
    match atom {
        "shrink1" | "shrink" => Some(1.0),
        "shrink0" => Some(0.0),
        _ => None,
    }
}

fn class_overflow_atom(atom: &str) -> Option<(Option<Overflow>, Option<Overflow>)> {
    let idx = atom.find("overflow")?;
    let rest = &atom[idx + "overflow".len()..];
    let value = if rest.ends_with("hidden") {
        Overflow::Hidden
    } else if rest.ends_with("clip") {
        Overflow::Clip
    } else if rest.ends_with("scroll") || rest.ends_with("auto") {
        Overflow::Scroll
    } else if rest.ends_with("visible") {
        Overflow::Visible
    } else {
        return None;
    };
    let axis = rest
        .strip_suffix("hidden")
        .or_else(|| rest.strip_suffix("clip"))
        .or_else(|| rest.strip_suffix("scroll"))
        .or_else(|| rest.strip_suffix("auto"))
        .or_else(|| rest.strip_suffix("visible"))
        .unwrap_or("");
    match axis {
        "x" => Some((Some(value), None)),
        "y" => Some((None, Some(value))),
        _ => Some((Some(value), Some(value))),
    }
}

fn class_position_atom(atom: &str) -> Option<Position> {
    if atom == "absolute"
        || atom == "fixed"
        || atom.contains("positionabsolute")
        || atom.contains("positionfixed")
    {
        return Some(Position::Absolute);
    }
    if atom == "relative"
        || atom == "sticky"
        || atom.contains("positionrelative")
        || atom.contains("positionsticky")
    {
        return Some(Position::Relative);
    }
    None
}

fn apply_style_size(style: &mut Style, map: &std::collections::HashMap<String, String>) {
    if let Some(v) = map.get("w") {
        if let Some(n) = parse_simple_px(v) {
            style.size.width = Dimension::length(n);
        } else if v == "full" || v.ends_with('%') {
            if v == "full" {
                style.size.width = Dimension::percent(1.0);
            } else if let Some(rest) = v.strip_suffix('%') {
                if let Ok(n) = rest.parse::<f32>() {
                    style.size.width = Dimension::percent(n / 100.0);
                }
            }
        }
    }
    if let Some(v) = map.get("h") {
        if let Some(n) = parse_simple_px(v) {
            style.size.height = Dimension::length(n);
        }
    }
}

fn parse_simple_px(v: &str) -> Option<f32> {
    let v = v.trim();
    if v == "0" {
        return Some(0.0);
    }
    v.strip_suffix("px")?.trim().parse().ok()
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
        assert_eq!(
            resolve(&row, &[Element::default()], 1440.0).style.flex_wrap,
            FlexWrap::Wrap
        );
        assert_eq!(
            resolve(&nowrap, &[Element::default()], 1440.0)
                .style
                .flex_wrap,
            FlexWrap::NoWrap
        );
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
            box_style: crate::box_ir::ResolvedBox {
                pad: crate::box_ir::PadTokens {
                    all: Some("4".into()),
                    ..Default::default()
                },
                ..Default::default()
            },
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
            class: "s-wfull".into(),
            ..Default::default()
        };
        let resolved = resolve(&elem, &[Element::default()], 1440.0);
        assert_eq!(resolved.style.size.width, Dimension::percent(1.0));
    }

    #[test]
    fn sx_hidden_sets_display_none() {
        let elem = Element {
            tag: "div".into(),
            class: "sail-col s-hidden".into(),
            ..Default::default()
        };
        let resolved = resolve(&elem, &[Element::default()], 1440.0);
        assert_eq!(resolved.style.display, Display::None);
    }
}
