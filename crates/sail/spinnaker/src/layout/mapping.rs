//! Data-driven mapping from SAIL layout primitives to `taffy::Style` fields.
//!
//! Style box tokens (display / grow / overflow / position / pad / size) are
//! applied from Style IR in `style.rs`. This table is the layout-primitive
//! pin: `sail-col` / gap / align / grid tracks. Adding a new primitive means
//! adding one row here and one row to the compiler's lowering.
//!
//! The table deliberately uses its own tiny value types rather than taffy's
//! directly, because several taffy types contain raw pointers and are not
//! `Sync` — they cannot live in a `static`.

/// A single mapping from a class selector to the layout declarations it implies.
#[derive(Debug, Clone, Copy)]
pub struct LayoutRule {
    pub selector: Selector,
    pub declarations: &'static [Declaration],
}

/// What class token(s) activate a rule.
#[derive(Debug, Clone, Copy)]
pub enum Selector {
    /// The base class for a layout primitive, e.g. `sail-col` or `sail-row-i`.
    Base(&'static str),
    /// A modifier class with a prefix and an exact value, e.g. `sail-g-` + `4`.
    Modifier {
        prefix: &'static str,
        value: &'static str,
    },
    /// The element whose *parent* carries the given base class. Used for child
    /// conventions (`min-width:0`, stack layering) without re-encoding the
    /// parent relationship inside every element's own class list.
    ParentBase(&'static str),
    /// Any element carrying this class. Used sparingly for standalone
    /// class names that the parser still sees.
    Any(&'static str),
}

/// One declaration to apply to a `taffy::Style`. The set is intentionally
/// smaller than CSS: only the box-tree properties the layout diagnostic pass
/// cares about. All values are plain data so the table can live in `static`.
#[derive(Debug, Clone, Copy)]
pub enum Declaration {
    Display(Display),
    FlexDirection(FlexDirection),
    FlexWrap(FlexWrap),
    /// Gap as `(row_px, column_px)`. taffy uses a single gap axis per
    /// direction; for SAIL the row and column gaps are always equal.
    Gap(f32, f32),
    AlignItems(AlignItems),
    JustifyContent(JustifyContent),
    Position(Position),
    Overflow {
        x: Overflow,
        y: Overflow,
    },
    FlexGrow(f32),
    FlexShrink(f32),
    GridColumnSpan(u16),
    GridRowSpan(u16),
    GridTemplateColumns(GridTemplate),
    GridTemplateRows(GridTemplate),
    GridAutoFlow(GridAutoFlow),
    MinWidth(Dimension),
    Width(Dimension),
    Height(Dimension),
    MaxWidth(Dimension),
    MaxHeight(Dimension),
    MinHeight(Dimension),
    Padding(PaddingSide, f32),
    /// Stack-layer convention: child becomes `grid-area: 1 / 1` and
    /// `position: relative` so DOM order is paint order.
    StackLayer,
}

#[derive(Debug, Clone, Copy)]
pub enum Display {
    Flex,
    Grid,
    Block,
    None,
}

#[derive(Debug, Clone, Copy)]
pub enum FlexDirection {
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

#[derive(Debug, Clone, Copy)]
pub enum FlexWrap {
    Wrap,
    NoWrap,
    WrapReverse,
}

#[derive(Debug, Clone, Copy)]
pub enum AlignItems {
    Start,
    End,
    Center,
    Stretch,
    Baseline,
}

#[derive(Debug, Clone, Copy)]
pub enum JustifyContent {
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Debug, Clone, Copy)]
pub enum Position {
    Relative,
    Absolute,
}

#[derive(Debug, Clone, Copy)]
pub enum Overflow {
    Visible,
    Hidden,
    Clip,
    Scroll,
}

#[derive(Debug, Clone, Copy)]
pub enum GridAutoFlow {
    Row,
    RowDense,
    Column,
    ColumnDense,
}

#[derive(Debug, Clone, Copy)]
pub enum Dimension {
    Length(f32),
    Percent(f32),
    Auto,
}

#[derive(Debug, Clone, Copy)]
pub enum PaddingSide {
    All,
    X,
    Y,
    T,
    R,
    B,
    L,
}

/// A compact grid-template description that `style.rs` turns into taffy's
/// `Vec<GridTemplateComponent>`.
#[derive(Debug, Clone, Copy)]
pub enum GridTemplate {
    /// `repeat(auto-fill, minmax(Npx, 1fr))`.
    AutoFillMin(f32),
    /// `repeat(N, minmax(0, 1fr))`.
    Fixed(u16),
    /// A custom-property reference (`var(--x)`). taffy cannot represent an
    /// arbitrary variable, so this leaves the explicit grid undefined and
    /// relies on auto-placement.
    Var,
    /// Closed track recipe from `<grid tracks="…">` (see
    /// `sail_lint::rules::layout::TRACK_RECIPES`). Keep the single algorithm:
    /// compiler emits the class; this maps it into taffy track sizes.
    Recipe(TrackRecipe),
}

/// Closed `tracks` recipes the compiler and taffy share.
/// Mirror of `sail_lint::rules::layout::TRACK_RECIPES` (names must match class
/// slugs `sail-gt-<name>`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackRecipe {
    /// `auto 1fr auto` — icon | body | action (also `icon-main-action` alias).
    AutoFrAuto,
    /// `1fr 1fr` — equal two-column split.
    FrFr,
}

// ---------------------------------------------------------------------------
// Static rule table
// ---------------------------------------------------------------------------

pub static LAYOUT_RULES: &[LayoutRule] = &[
    // --- flex containers -----------------------------------------------------
    LayoutRule {
        selector: Selector::Base("sail-col"),
        declarations: DISPLAY_COL,
    },
    LayoutRule {
        selector: Selector::Base("sail-col-i"),
        declarations: DISPLAY_COL,
    },
    LayoutRule {
        selector: Selector::Base("sail-row"),
        declarations: DISPLAY_ROW_WRAP,
    },
    LayoutRule {
        selector: Selector::Base("sail-row-i"),
        declarations: DISPLAY_ROW_WRAP,
    },
    // --- grid containers -----------------------------------------------------
    LayoutRule {
        selector: Selector::Base("sail-grid"),
        declarations: DISPLAY_GRID,
    },
    LayoutRule {
        selector: Selector::Base("sail-grid-i"),
        declarations: DISPLAY_GRID,
    },
    LayoutRule {
        selector: Selector::Base("sail-center"),
        declarations: DISPLAY_CENTER,
    },
    LayoutRule {
        selector: Selector::Base("sail-center-i"),
        declarations: DISPLAY_CENTER_INLINE,
    },
    LayoutRule {
        selector: Selector::Base("sail-stackz"),
        declarations: DISPLAY_GRID,
    },
    LayoutRule {
        selector: Selector::Base("sail-stackz-i"),
        declarations: DISPLAY_GRID,
    },
    // --- gap modifiers -------------------------------------------------------
    LayoutRule {
        selector: Selector::Modifier {
            prefix: "sail-g-",
            value: "0",
        },
        declarations: &[Declaration::Gap(0.0, 0.0)],
    },
    LayoutRule {
        selector: Selector::Modifier {
            prefix: "sail-g-",
            value: "density",
        },
        declarations: &[Declaration::Gap(8.0, 8.0)],
    },
    LayoutRule {
        selector: Selector::Modifier {
            prefix: "sail-g-",
            value: "1",
        },
        declarations: &[Declaration::Gap(4.0, 4.0)],
    },
    LayoutRule {
        selector: Selector::Modifier {
            prefix: "sail-g-",
            value: "2",
        },
        declarations: &[Declaration::Gap(8.0, 8.0)],
    },
    LayoutRule {
        selector: Selector::Modifier {
            prefix: "sail-g-",
            value: "2_5",
        },
        declarations: &[Declaration::Gap(10.0, 10.0)],
    },
    LayoutRule {
        selector: Selector::Modifier {
            prefix: "sail-g-",
            value: "3",
        },
        declarations: &[Declaration::Gap(12.0, 12.0)],
    },
    LayoutRule {
        selector: Selector::Modifier {
            prefix: "sail-g-",
            value: "4",
        },
        declarations: &[Declaration::Gap(16.0, 16.0)],
    },
    LayoutRule {
        selector: Selector::Modifier {
            prefix: "sail-g-",
            value: "5",
        },
        declarations: &[Declaration::Gap(20.0, 20.0)],
    },
    LayoutRule {
        selector: Selector::Modifier {
            prefix: "sail-g-",
            value: "6",
        },
        declarations: &[Declaration::Gap(24.0, 24.0)],
    },
    LayoutRule {
        selector: Selector::Modifier {
            prefix: "sail-g-",
            value: "7",
        },
        declarations: &[Declaration::Gap(28.0, 28.0)],
    },
    LayoutRule {
        selector: Selector::Modifier {
            prefix: "sail-g-",
            value: "8",
        },
        declarations: &[Declaration::Gap(32.0, 32.0)],
    },
    // --- align / justify modifiers -------------------------------------------
    LayoutRule {
        selector: Selector::Modifier {
            prefix: "sail-ai-",
            value: "start",
        },
        declarations: &[Declaration::AlignItems(AlignItems::Start)],
    },
    LayoutRule {
        selector: Selector::Modifier {
            prefix: "sail-ai-",
            value: "end",
        },
        declarations: &[Declaration::AlignItems(AlignItems::End)],
    },
    LayoutRule {
        selector: Selector::Modifier {
            prefix: "sail-ai-",
            value: "center",
        },
        declarations: &[Declaration::AlignItems(AlignItems::Center)],
    },
    LayoutRule {
        selector: Selector::Modifier {
            prefix: "sail-ai-",
            value: "stretch",
        },
        declarations: &[Declaration::AlignItems(AlignItems::Stretch)],
    },
    LayoutRule {
        selector: Selector::Modifier {
            prefix: "sail-ai-",
            value: "baseline",
        },
        declarations: &[Declaration::AlignItems(AlignItems::Baseline)],
    },
    LayoutRule {
        selector: Selector::Modifier {
            prefix: "sail-jc-",
            value: "start",
        },
        declarations: &[Declaration::JustifyContent(JustifyContent::Start)],
    },
    LayoutRule {
        selector: Selector::Modifier {
            prefix: "sail-jc-",
            value: "end",
        },
        declarations: &[Declaration::JustifyContent(JustifyContent::End)],
    },
    LayoutRule {
        selector: Selector::Modifier {
            prefix: "sail-jc-",
            value: "center",
        },
        declarations: &[Declaration::JustifyContent(JustifyContent::Center)],
    },
    LayoutRule {
        selector: Selector::Modifier {
            prefix: "sail-jc-",
            value: "between",
        },
        declarations: &[Declaration::JustifyContent(JustifyContent::SpaceBetween)],
    },
    LayoutRule {
        selector: Selector::Modifier {
            prefix: "sail-jc-",
            value: "around",
        },
        declarations: &[Declaration::JustifyContent(JustifyContent::SpaceAround)],
    },
    LayoutRule {
        selector: Selector::Modifier {
            prefix: "sail-jc-",
            value: "evenly",
        },
        declarations: &[Declaration::JustifyContent(JustifyContent::SpaceEvenly)],
    },
    // --- row nowrap ----------------------------------------------------------
    LayoutRule {
        selector: Selector::Any("sail-nowrap"),
        declarations: &[Declaration::FlexWrap(FlexWrap::NoWrap)],
    },
    // --- grid track modifiers ------------------------------------------------
    LayoutRule {
        selector: Selector::Modifier {
            prefix: "sail-gm-",
            value: "xs",
        },
        declarations: &[Declaration::GridTemplateColumns(GridTemplate::AutoFillMin(
            256.0,
        ))],
    },
    LayoutRule {
        selector: Selector::Modifier {
            prefix: "sail-gm-",
            value: "sm",
        },
        declarations: &[Declaration::GridTemplateColumns(GridTemplate::AutoFillMin(
            288.0,
        ))],
    },
    LayoutRule {
        selector: Selector::Modifier {
            prefix: "sail-gm-",
            value: "md",
        },
        declarations: &[Declaration::GridTemplateColumns(GridTemplate::AutoFillMin(
            320.0,
        ))],
    },
    LayoutRule {
        selector: Selector::Modifier {
            prefix: "sail-gm-",
            value: "lg",
        },
        declarations: &[Declaration::GridTemplateColumns(GridTemplate::AutoFillMin(
            384.0,
        ))],
    },
    LayoutRule {
        selector: Selector::Modifier {
            prefix: "sail-gf-",
            value: "auto",
        },
        declarations: &[Declaration::GridAutoFlow(GridAutoFlow::Row)],
    },
    LayoutRule {
        selector: Selector::Modifier {
            prefix: "sail-gf-",
            value: "dense",
        },
        declarations: &[Declaration::GridAutoFlow(GridAutoFlow::RowDense)],
    },
    // --- child conventions ---------------------------------------------------
    LayoutRule {
        selector: Selector::ParentBase("sail-col"),
        declarations: &[Declaration::MinWidth(Dimension::Length(0.0))],
    },
    LayoutRule {
        selector: Selector::ParentBase("sail-col-i"),
        declarations: &[Declaration::MinWidth(Dimension::Length(0.0))],
    },
    LayoutRule {
        selector: Selector::ParentBase("sail-row"),
        declarations: &[Declaration::MinWidth(Dimension::Length(0.0))],
    },
    LayoutRule {
        selector: Selector::ParentBase("sail-row-i"),
        declarations: &[Declaration::MinWidth(Dimension::Length(0.0))],
    },
    LayoutRule {
        selector: Selector::ParentBase("sail-grid"),
        declarations: &[Declaration::MinWidth(Dimension::Length(0.0))],
    },
    LayoutRule {
        selector: Selector::ParentBase("sail-grid-i"),
        declarations: &[Declaration::MinWidth(Dimension::Length(0.0))],
    },
    LayoutRule {
        selector: Selector::ParentBase("sail-center"),
        declarations: &[Declaration::MinWidth(Dimension::Length(0.0))],
    },
    LayoutRule {
        selector: Selector::ParentBase("sail-center-i"),
        declarations: &[Declaration::MinWidth(Dimension::Length(0.0))],
    },
    LayoutRule {
        selector: Selector::ParentBase("sail-stackz"),
        declarations: &[Declaration::StackLayer],
    },
    LayoutRule {
        selector: Selector::ParentBase("sail-stackz-i"),
        declarations: &[Declaration::StackLayer],
    },
];

// Re-used declaration slices so the table stays readable.
const DISPLAY_COL: &[Declaration] = &[
    Declaration::Display(Display::Flex),
    Declaration::FlexDirection(FlexDirection::Column),
];
const DISPLAY_ROW_WRAP: &[Declaration] = &[
    Declaration::Display(Display::Flex),
    Declaration::FlexDirection(FlexDirection::Row),
    Declaration::FlexWrap(FlexWrap::Wrap),
];
const DISPLAY_GRID: &[Declaration] = &[Declaration::Display(Display::Grid)];
const DISPLAY_CENTER: &[Declaration] = &[
    Declaration::Display(Display::Grid),
    Declaration::AlignItems(AlignItems::Center),
    Declaration::JustifyContent(JustifyContent::Center),
    Declaration::Width(Dimension::Percent(1.0)),
];
const DISPLAY_CENTER_INLINE: &[Declaration] = &[
    Declaration::Display(Display::Grid),
    Declaration::AlignItems(AlignItems::Center),
    Declaration::JustifyContent(JustifyContent::Center),
];

// ---------------------------------------------------------------------------
// Scale-driven sx prefixes resolved at runtime through `crate::scales`.
// ---------------------------------------------------------------------------

/// A scale prefix that maps a product Style atom like `s-pad-4` to a layout
/// property. The value tail is fed to the scale resolver.
#[derive(Debug, Clone, Copy)]
pub enum SxScaleRule {
    Padding(SideSet),
    /// `size(4)` → `s-size-4` sets both width and height.
    Size,
    Width,
    Height,
    MinWidth,
    MinHeight,
    MaxWidth,
    MaxHeight,
}

#[derive(Debug, Clone, Copy)]
pub enum SideSet {
    All,
    X,
    Y,
    T,
    R,
    B,
    L,
}

/// Every product `s-*` prefix whose value lives on a closed scale. Order
/// matters: the longest prefix must be checked first.
///
/// Style mints `s-pad-4`, `s-pad-x2`, `s-size-min-w0`.
pub static SX_SCALE_RULES: &[(&str, SxScaleRule)] = &[
    // Longest first: pad-x before pad, min-h before h.
    ("s-pad-x", SxScaleRule::Padding(SideSet::X)),
    ("s-pad-y", SxScaleRule::Padding(SideSet::Y)),
    ("s-pad-t", SxScaleRule::Padding(SideSet::T)),
    ("s-pad-r", SxScaleRule::Padding(SideSet::R)),
    ("s-pad-b", SxScaleRule::Padding(SideSet::B)),
    ("s-pad-l", SxScaleRule::Padding(SideSet::L)),
    ("s-pad-s", SxScaleRule::Padding(SideSet::L)), // inline-start ≈ left for taffy
    ("s-pad-e", SxScaleRule::Padding(SideSet::R)), // inline-end ≈ right for taffy
    ("s-pad", SxScaleRule::Padding(SideSet::All)),
    ("s-min-w", SxScaleRule::MinWidth),
    ("s-min-h", SxScaleRule::MinHeight),
    ("s-max-w", SxScaleRule::MaxWidth),
    ("s-max-h", SxScaleRule::MaxHeight),
    // Product `size(N)` → width+height (before bare w/h).
    ("s-size", SxScaleRule::Size),
    ("s-w", SxScaleRule::Width),
    ("s-h", SxScaleRule::Height),
];

// ---------------------------------------------------------------------------
// Grid column/row count modifiers (`sail-gc-N`, `sail-gr-N`) are handled by
// `style.rs` because generating 24 near-identical rules here adds noise without
// improving clarity. They still map through this module's `GridTemplate::Fixed`.
// ---------------------------------------------------------------------------

/// Parse a `sail-gc-N` / `sail-gr-N` class tail into the corresponding
/// `GridTemplate` axis.
pub fn grid_count_template(prefix: &str, value: &str) -> Option<(bool, GridTemplate)> {
    let is_cols = prefix == "sail-gc-";
    if !is_cols && prefix != "sail-gr-" {
        return None;
    }
    let n: u16 = value.parse().ok()?;
    if n == 0 || n > 12 {
        return None;
    }
    Some((is_cols, GridTemplate::Fixed(n)))
}

/// Parse a `sail-gt-<recipe>` class into a closed track recipe (columns axis).
/// Returns `None` for custom-property slugs (`--x` → `sail-gt-x`) that taffy
/// cannot resolve — those fall through as auto-placement (`GridTemplate::Var`).
pub fn grid_recipe_template(class: &str) -> Option<GridTemplate> {
    let name = class.strip_prefix("sail-gt-")?;
    match name {
        "auto-fr-auto" | "icon-main-action" => Some(GridTemplate::Recipe(TrackRecipe::AutoFrAuto)),
        "fr-fr" => Some(GridTemplate::Recipe(TrackRecipe::FrFr)),
        // Custom-property slug (no hyphens-only recipe match): leave as Var.
        _ => Some(GridTemplate::Var),
    }
}
