//! Unknown-subtree placeholder rendering.
//!
//! Custom elements and replaced elements that `sail_layout` marks as
//! `is_unknown()` cannot be truthfully rendered. We draw a muted placeholder
//! box with the tag name centered, exactly as the spec requires.

use takumi::prelude::*;

const PLACEHOLDER_MIN_W: f32 = 200.0;
const PLACEHOLDER_MIN_H: f32 = 100.0;
const PLACEHOLDER_BG: Color = Color::from_rgb(0xf1f5f9);
const PLACEHOLDER_BORDER: Color = Color::from_rgb(0x94a3b8);
const PLACEHOLDER_TEXT: Color = Color::from_rgb(0x64748b);

/// Build a placeholder node for an unknown subtree.
///
/// `computed_width` and `computed_height` come from `sail_layout`'s computed
/// layout, if any. If they are zero, a minimum visible size is enforced.
pub fn node(tag: &str, computed_width: f32, computed_height: f32) -> Node {
    let w = computed_width.max(PLACEHOLDER_MIN_W);
    let h = computed_height.max(PLACEHOLDER_MIN_H);

    Node::container(vec![
        Node::text(tag.to_string()).with_style(
            Style::default()
                .with(StyleDeclaration::color(ColorInput::Value(PLACEHOLDER_TEXT)))
                .with(StyleDeclaration::font_size(FontSize::Length(Length::Px(12.0))))
                .with(StyleDeclaration::font_family(FontFamily::from_names(["JetBrains Mono".to_string()])))
                .with(StyleDeclaration::text_align(TextAlign::Center)),
        ),
    ])
    .with_style(
        Style::default()
            .with(StyleDeclaration::width(Length::Px(w)))
            .with(StyleDeclaration::height(Length::Px(h)))
            .with(StyleDeclaration::display(Display::Flex))
            .with(StyleDeclaration::align_items(AlignItems::Center))
            .with(StyleDeclaration::justify_content(JustifyContent::Center))
            .with(StyleDeclaration::background_color(ColorInput::Value(PLACEHOLDER_BG)))
            .with(StyleDeclaration::border_top_color(ColorInput::Value(PLACEHOLDER_BORDER)))
            .with(StyleDeclaration::border_right_color(ColorInput::Value(PLACEHOLDER_BORDER)))
            .with(StyleDeclaration::border_bottom_color(ColorInput::Value(PLACEHOLDER_BORDER)))
            .with(StyleDeclaration::border_left_color(ColorInput::Value(PLACEHOLDER_BORDER)))
            .with(StyleDeclaration::border_top_width(LineWidth::Length(Length::Px(2.0))))
            .with(StyleDeclaration::border_right_width(LineWidth::Length(Length::Px(2.0))))
            .with(StyleDeclaration::border_bottom_width(LineWidth::Length(Length::Px(2.0))))
            .with(StyleDeclaration::border_left_width(LineWidth::Length(Length::Px(2.0))))
            .with(StyleDeclaration::border_top_style(BorderStyle::Dashed))
            .with(StyleDeclaration::border_right_style(BorderStyle::Dashed))
            .with(StyleDeclaration::border_bottom_style(BorderStyle::Dashed))
            .with(StyleDeclaration::border_left_style(BorderStyle::Dashed)),
    )
}
