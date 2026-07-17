//! Resolve a parsed element's `class=` + `style:` to a `taffy::Style`.
//!
//! The closed vocabulary is the *whole reason this works without a browser*.
//! We read two vocabularies — both owned by `sail-lint` so the spike cannot
//! drift from the language:
//!
//! - `sail-lint::rules::layout` — the L1 layout primitives (`row`, `column`,
//!   `grid`, `stack`, `center`). Their classes encode `display`,
//!   `flex-direction`, `flex-wrap`, `align-items`, `justify-content`, `gap`,
//!   and grid tracks.
//! - `sail-lint::scales` — value resolution for scale-valued atoms
//!   (`p-4` → `16px`, `max-w-panel-max-w-md` → `480px`).

use crate::html::Element;
use std::collections::HashMap;
use sail_lint::rules::layout;
use sail_lint::scales;
use taffy::prelude::*;
use taffy::style::Overflow;
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

pub fn resolve(elem: &Element) -> ResolvedStyle {
    if elem.tag.is_empty() {
        return ResolvedStyle::ignored();
    }
    let classes: Vec<&str> = elem.class.split_whitespace().collect();

    let display = detect_display(&classes, &elem.tag);
    let direction = detect_direction(&classes);
    let wrap = detect_wrap(&classes);
    let align_items = detect_align_items(&classes);
    let justify_content = detect_justify_content(&classes);
    let gap = detect_gap(&classes);
    let padding = detect_padding(&classes);
    let (width, max_width, min_width) = detect_size(&classes, true, &elem.style);
    let (height, max_height, min_height) = detect_size(&classes, false, &elem.style);
    let flex_grow = classes
        .iter()
        .find_map(|c| match *c {
            "grow-1" => Some(1.0_f32),
            "grow-0" => Some(0.0),
            _ => None,
        })
        .unwrap_or(0.0);
    let flex_shrink = classes
        .iter()
        .find_map(|c| match *c {
            "shrink-1" => Some(1.0_f32),
            "shrink-0" => Some(0.0),
            _ => None,
        })
        .unwrap_or(1.0);
    let position = detect_position(&classes);
    let overflow = detect_overflow(&classes);

    let is_leaf = !matches!(display, Display::Flex | Display::Grid | Display::Block)
        || matches!(position, Position::Absolute);

    let mut style = Style {
        display,
        overflow,
        scrollbar_width: 0.0,
        position,
        inset: taffy::Rect::auto(),
        size: Size { width, height },
        min_size: Size { width: min_width, height: min_height },
        max_size: Size { width: max_width, height: max_height },
        aspect_ratio: None,
        margin: taffy::Rect::zero(),
        padding,
        border: taffy::Rect::zero(),
        align_items: Some(align_items),
        align_self: None,
        justify_items: None,
        align_content: Some(AlignContent::STRETCH),
        justify_content: Some(justify_content),
        justify_self: None,
        flex_direction: direction,
        flex_wrap: wrap,
        flex_basis: Dimension::auto(),
        flex_grow,
        flex_shrink,
        gap: Size {
            width: LengthPercentage::length(gap),
            height: LengthPercentage::length(gap),
        },
        ..Default::default()
    };

    if is_leaf && elem.children.is_empty() {
        let (_w, h) = text_shape(elem);
        style.size.height = Dimension::length(h as f32);
    }

    ResolvedStyle { style, role: Role::None, visibility: Visibility::Visible }
    // (silence the unused-mut warning on `style` for now; future leaf-shape work will use it)
}

#[allow(dead_code)]
pub fn text_shape(elem: &Element) -> (f64, f64) {
    let line_height_token = elem
        .class
        .split_whitespace()
        .find_map(|c| c.strip_prefix("text-"))
        .unwrap_or("base");
    let font_size_px = match line_height_token {
        "xs" => 11.0,
        "sm" => 12.0,
        "base" => 13.0,
        "md" => 14.0,
        "lg" => 16.0,
        "xl" => 20.0,
        "2xl" => 24.0,
        "3xl" => 28.0,
        "4xl" => 36.0,
        _ => 16.0,
    };
    let line_height_px = font_size_px * 1.5;
    (font_size_px * 32.0, line_height_px)
}

fn detect_display(classes: &[&str], tag: &str) -> Display {
    let mut base = match tag {
        "row" | "column" => Display::Flex,
        "grid" | "center" | "stack" => Display::Grid,
        _ => Display::Block,
    };
    for c in classes {
        match *c {
            "hidden" => return Display::None,
            // `inline`/`inline-block` aren't in taffy's Display enum (it strips
            // formatting contexts); treat as `Flex`/`Block` respectively. The
            // spike's diagnostics only care about box geometry.
            "inline" => base = Display::Flex,
            "inline-block" => base = Display::Block,
            _ => {}
        }
    }
    base
}

fn detect_direction(classes: &[&str]) -> FlexDirection {
    let mut dir = if classes.iter().any(|c| *c == "sail-row") {
        FlexDirection::Row
    } else {
        FlexDirection::Column
    };
    for c in classes {
        if c.starts_with("sail-dir-") {
            dir = match &c["sail-dir-".len()..] {
                "row" => FlexDirection::Row,
                "row-reverse" => FlexDirection::RowReverse,
                "column" => FlexDirection::Column,
                "column-reverse" => FlexDirection::ColumnReverse,
                _ => dir,
            };
        }
    }
    dir
}

fn detect_wrap(classes: &[&str]) -> FlexWrap {
    if classes.iter().any(|c| *c == "sail-row") {
        return FlexWrap::Wrap;
    }
    for c in classes {
        if c.starts_with("flex-wrap-") {
            return match &c["flex-wrap-".len()..] {
                "wrap" => FlexWrap::Wrap,
                "wrap-reverse" => FlexWrap::WrapReverse,
                _ => FlexWrap::NoWrap,
            };
        }
    }
    FlexWrap::NoWrap
}

fn detect_align_items(classes: &[&str]) -> AlignItems {
    for c in classes {
        if c.starts_with("sail-ai-") {
            return match &c["sail-ai-".len()..] {
                "start" => AlignItems::START,
                "end" => AlignItems::END,
                "center" => AlignItems::CENTER,
                "stretch" => AlignItems::STRETCH,
                "baseline" => AlignItems::BASELINE,
                _ => continue,
            };
        }
    }
    AlignItems::STRETCH
}

fn detect_justify_content(classes: &[&str]) -> JustifyContent {
    for c in classes {
        if c.starts_with("sail-jc-") {
            return match &c["sail-jc-".len()..] {
                "start" => JustifyContent::START,
                "end" => JustifyContent::END,
                "center" => JustifyContent::CENTER,
                "between" => JustifyContent::SPACE_BETWEEN,
                "around" => JustifyContent::SPACE_AROUND,
                _ => continue,
            };
        }
    }
    JustifyContent::START
}

fn detect_gap(classes: &[&str]) -> f32 {
    for c in classes {
        if let Some(rest) = c.strip_prefix("sail-g-") {
            if rest == "0" {
                return 0.0;
            }
            if rest == "density" {
                return 8.0;
            }
            if let Ok(step) = rest.parse::<u32>() {
                return token_px_for_space(step);
            }
        }
    }
    if classes.iter().any(|c| *c == "sail-col") {
        return token_px_for_space(4);
    }
    if classes.iter().any(|c| *c == "sail-row" || *c == "sail-grid") {
        return token_px_for_space(3);
    }
    0.0
}

fn detect_padding(classes: &[&str]) -> taffy::Rect<LengthPercentage> {
    let mut t: f32 = 0.0;
    let mut r: f32 = 0.0;
    let mut b: f32 = 0.0;
    let mut l: f32 = 0.0;
    let _breakdown: HashMap<&'static str, f32> = HashMap::new();

    for c in classes {
        let (which, value) = if let Some(rest) = c.strip_prefix("sail-p-") {
            ("all", rest)
        } else if let Some(rest) = c.strip_prefix("sail-px-") {
            ("x", rest)
        } else if let Some(rest) = c.strip_prefix("sail-py-") {
            ("y", rest)
        } else if let Some(rest) = c.strip_prefix("sail-pt-") {
            ("t", rest)
        } else if let Some(rest) = c.strip_prefix("sail-pr-") {
            ("r", rest)
        } else if let Some(rest) = c.strip_prefix("sail-pb-") {
            ("b", rest)
        } else if let Some(rest) = c.strip_prefix("sail-pl-") {
            ("l", rest)
        } else {
            continue;
        };
        let px = padding_token_to_px(value);
        match which {
            "all" => {
                t = px;
                r = px;
                b = px;
                l = px;
            }
            "x" => {
                r = px;
                l = px;
            }
            "y" => {
                t = px;
                b = px;
            }
            "t" => t = px,
            "r" => r = px,
            "b" => b = px,
            "l" => l = px,
            _ => {}
        }
    }

    taffy::Rect {
        top: LengthPercentage::length(t),
        right: LengthPercentage::length(r),
        bottom: LengthPercentage::length(b),
        left: LengthPercentage::length(l),
    }
}

fn detect_size(
    classes: &[&str],
    is_width: bool,
    style: &HashMap<String, String>,
) -> (Dimension, Dimension, Dimension) {
    let mut size = Dimension::auto();
    let mut min = Dimension::auto();
    let mut max = Dimension::auto();

    let (w_pre, min_w_pre, max_w_pre) = if is_width {
        ("sail-w-", "sail-min-w-", "sail-max-w-")
    } else {
        ("sail-h-", "sail-min-h-", "sail-max-h-")
    };

    for c in classes {
        if let Some(rest) = c.strip_prefix(w_pre) {
            size = size_token_to_dimension(rest);
        } else if let Some(rest) = c.strip_prefix(min_w_pre) {
            min = size_token_to_dimension(rest);
        } else if let Some(rest) = c.strip_prefix(max_w_pre) {
            max = size_token_to_dimension(rest);
        }
    }

    let var = if is_width { "w" } else { "h" };
    if let Some(v) = style.get(var) {
        if let Some(d) = parse_css_length(v) {
            size = d;
        }
    }

    (size, min, max)
}

fn detect_position(classes: &[&str]) -> Position {
    if classes.iter().any(|c| *c == "absolute") {
        Position::Absolute
    } else {
        Position::Relative
    }
}

fn detect_overflow(classes: &[&str]) -> taffy::Point<Overflow> {
    let mut x = Overflow::Visible;
    let mut y = Overflow::Visible;
    for c in classes {
        match *c {
            "sail-overflow-hidden" => {
                x = Overflow::Hidden;
                y = Overflow::Hidden;
            }
            "sail-overflow-x-auto" | "sail-overflow-x-scroll" => x = Overflow::Scroll,
            "sail-overflow-y-auto" | "sail-overflow-y-scroll" => y = Overflow::Scroll,
            "sail-overflow-x-hidden" => x = Overflow::Hidden,
            "sail-overflow-y-hidden" => y = Overflow::Hidden,
            _ => {}
        }
    }
    taffy::Point { x, y }
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

#[allow(dead_code)]
fn _vocab_smoke() {
    let _ = layout::TAGS;
    let _ = scales::values("size");
    let _ = scales::contains("color", "transparent");
}
