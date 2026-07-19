//! Mechanical conversion from `sail_layout`'s resolved `taffy::Style` to
//! `takumi::Style` declarations.
//!
//! This is intentionally one place: if takumi's style model needs a value, it
//! comes from the `ResolvedStyle` that the diagnostics already consume.

use sail_layout::taffy::style as ts;
use sail_layout::taffy::style::CompactLength;
use takumi::prelude::*;

/// Convert a `taffy::Style` into a `takumi::Style` builder chain.
pub fn style(taffy_style: &sail_layout::taffy::Style) -> Style {
    let mut style = Style::default();

    style = style.with(StyleDeclaration::display(display(taffy_style.display)));

    if taffy_style.display != ts::Display::None {
        if let Some(fd) = flex_direction(taffy_style.flex_direction) {
            style = style.with(StyleDeclaration::flex_direction(fd));
        }
        if let Some(fw) = flex_wrap(taffy_style.flex_wrap) {
            style = style.with(StyleDeclaration::flex_wrap(fw));
        }
        if let Some(ai) = taffy_style.align_items.and_then(align_items) {
            style = style.with(StyleDeclaration::align_items(ai));
        }
        if let Some(jc) = taffy_style.justify_content.and_then(justify_content) {
            style = style.with(StyleDeclaration::justify_content(jc));
        }
        if let Some(ac) = taffy_style.align_content.and_then(align_content) {
            style = style.with(StyleDeclaration::align_content(ac));
        }

        style = style.with(StyleDeclaration::column_gap(gap(taffy_style.gap.width)));
        style = style.with(StyleDeclaration::row_gap(gap(taffy_style.gap.height)));

        style = style.with(StyleDeclaration::width(dimension(taffy_style.size.width)));
        style = style.with(StyleDeclaration::height(dimension(taffy_style.size.height)));
        style = style.with(StyleDeclaration::min_width(dimension(taffy_style.min_size.width)));
        style = style.with(StyleDeclaration::min_height(dimension(taffy_style.min_size.height)));
        style = style.with(StyleDeclaration::max_width(max_size(taffy_style.max_size.width)));
        style = style.with(StyleDeclaration::max_height(max_size(taffy_style.max_size.height)));

        style = style.with(StyleDeclaration::padding_top(length(taffy_style.padding.top)));
        style = style.with(StyleDeclaration::padding_right(length(taffy_style.padding.right)));
        style = style.with(StyleDeclaration::padding_bottom(length(taffy_style.padding.bottom)));
        style = style.with(StyleDeclaration::padding_left(length(taffy_style.padding.left)));

        style = style.with(StyleDeclaration::position(position(taffy_style.position)));

        style = style.with(StyleDeclaration::top(length_auto(taffy_style.inset.top)));
        style = style.with(StyleDeclaration::right(length_auto(taffy_style.inset.right)));
        style = style.with(StyleDeclaration::bottom(length_auto(taffy_style.inset.bottom)));
        style = style.with(StyleDeclaration::left(length_auto(taffy_style.inset.left)));

        style = style.with(StyleDeclaration::flex_grow(Some(FlexGrow(taffy_style.flex_grow))));
        style = style.with(StyleDeclaration::flex_shrink(Some(FlexGrow(taffy_style.flex_shrink))));

        if let Some(overflow) = overflow(taffy_style.overflow.x) {
            style = style.with(StyleDeclaration::overflow_x(overflow));
        }
        if let Some(overflow) = overflow(taffy_style.overflow.y) {
            style = style.with(StyleDeclaration::overflow_y(overflow));
        }
    }

    style
}

fn display(d: ts::Display) -> takumi::prelude::Display {
    match d {
        ts::Display::Flex => takumi::prelude::Display::Flex,
        ts::Display::Grid => takumi::prelude::Display::Grid,
        ts::Display::Block => takumi::prelude::Display::Block,
        ts::Display::None => takumi::prelude::Display::None,
    }
}

fn flex_direction(d: ts::FlexDirection) -> Option<FlexDirection> {
    match d {
        ts::FlexDirection::Row => Some(FlexDirection::Row),
        ts::FlexDirection::Column => Some(FlexDirection::Column),
        ts::FlexDirection::RowReverse => Some(FlexDirection::RowReverse),
        ts::FlexDirection::ColumnReverse => Some(FlexDirection::ColumnReverse),
    }
}

fn flex_wrap(w: ts::FlexWrap) -> Option<FlexWrap> {
    match w {
        ts::FlexWrap::Wrap => Some(FlexWrap::Wrap),
        ts::FlexWrap::NoWrap => Some(FlexWrap::NoWrap),
        ts::FlexWrap::WrapReverse => Some(FlexWrap::WrapReverse),
    }
}

fn align_items(a: ts::AlignItems) -> Option<AlignItems> {
    let mapped = match a.keyword {
        ts::AlignItemsKeyword::Start => AlignItems::Start,
        ts::AlignItemsKeyword::End => AlignItems::End,
        ts::AlignItemsKeyword::FlexStart => AlignItems::FlexStart,
        ts::AlignItemsKeyword::FlexEnd => AlignItems::FlexEnd,
        ts::AlignItemsKeyword::Center => AlignItems::Center,
        ts::AlignItemsKeyword::Baseline => AlignItems::Baseline,
        ts::AlignItemsKeyword::Stretch => AlignItems::Stretch,
    };
    Some(mapped)
}

fn justify_content(j: ts::JustifyContent) -> Option<JustifyContent> {
    let mapped = match j.keyword {
        ts::AlignContentKeyword::Start => JustifyContent::Start,
        ts::AlignContentKeyword::End => JustifyContent::End,
        ts::AlignContentKeyword::FlexStart => JustifyContent::FlexStart,
        ts::AlignContentKeyword::FlexEnd => JustifyContent::FlexEnd,
        ts::AlignContentKeyword::Center => JustifyContent::Center,
        ts::AlignContentKeyword::Stretch => JustifyContent::Stretch,
        ts::AlignContentKeyword::SpaceBetween => JustifyContent::SpaceBetween,
        ts::AlignContentKeyword::SpaceAround => JustifyContent::SpaceAround,
        ts::AlignContentKeyword::SpaceEvenly => JustifyContent::SpaceEvenly,
    };
    Some(mapped)
}

fn align_content(a: ts::AlignContent) -> Option<JustifyContent> {
    justify_content(a)
}

fn gap(value: ts::LengthPercentage) -> Gap {
    Gap::Length(length(value))
}

fn length(lp: ts::LengthPercentage) -> Length {
    let raw = lp.into_raw();
    if raw.tag() == CompactLength::LENGTH_TAG {
        Length::Px(raw.value())
    } else {
        Length::Percentage(raw.value() * 100.0)
    }
}

fn length_auto(lp: ts::LengthPercentageAuto) -> Length {
    let raw = lp.into_raw();
    if raw.tag() == CompactLength::LENGTH_TAG {
        Length::Px(raw.value())
    } else if raw.tag() == CompactLength::PERCENT_TAG {
        Length::Percentage(raw.value() * 100.0)
    } else {
        Length::Auto
    }
}

fn dimension(d: ts::Dimension) -> Length {
    let raw = d.into_raw();
    if raw.tag() == CompactLength::LENGTH_TAG {
        Length::Px(raw.value())
    } else if raw.tag() == CompactLength::PERCENT_TAG {
        Length::Percentage(raw.value() * 100.0)
    } else {
        Length::Auto
    }
}

fn max_size(d: ts::Dimension) -> MaxSize {
    let raw = d.into_raw();
    if raw.tag() == CompactLength::LENGTH_TAG {
        MaxSize::Length(Length::Px(raw.value()))
    } else if raw.tag() == CompactLength::PERCENT_TAG {
        MaxSize::Length(Length::Percentage(raw.value() * 100.0))
    } else {
        MaxSize::None
    }
}

fn position(p: ts::Position) -> takumi::prelude::Position {
    match p {
        ts::Position::Relative => takumi::prelude::Position::Relative,
        ts::Position::Absolute => takumi::prelude::Position::Absolute,
    }
}

fn overflow(o: ts::Overflow) -> Option<takumi::prelude::Overflow> {
    match o {
        ts::Overflow::Visible => Some(takumi::prelude::Overflow::Visible),
        ts::Overflow::Clip => Some(takumi::prelude::Overflow::Clip),
        ts::Overflow::Hidden | ts::Overflow::Scroll => Some(takumi::prelude::Overflow::Hidden),
    }
}
