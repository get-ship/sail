//! Data-driven mapping from SAIL layout classes and sx atoms to `taffy::Style`
//! fields.
//!
//! This table is the conformance pin: the compiler emits CSS from the same
//! vocabulary, and this crate configures taffy to match that CSS. Adding a new
//! layout atom should mean adding one row here and one row to the compiler's
//! lowering, not chasing conditionals across `style.rs`.
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
    /// A sanitized sx atomic class, e.g. product `sx-pad4` / `sx-grow1`
    /// (residual utility names like `sx-p4` / `sx-grow` still match for
    /// fixtures and any leftover emit).
    Sx(&'static str),
    /// The element whose *parent* carries the given base class. Used for child
    /// conventions (`min-width:0`, stack layering) without re-encoding the
    /// parent relationship inside every element's own class list.
    ParentBase(&'static str),
    /// Any element carrying this class. Used sparingly for legacy/standalone
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
    // MIGRATION-DEBT: dies with CHANNELS-1 (layout-keyword migration).
    // --- flex direction modifier (legacy alias) ------------------------------
    LayoutRule {
        selector: Selector::Modifier {
            prefix: "sail-dir-",
            value: "row",
        },
        declarations: &[Declaration::FlexDirection(FlexDirection::Row)],
    },
    LayoutRule {
        selector: Selector::Modifier {
            prefix: "sail-dir-",
            value: "row-reverse",
        },
        declarations: &[Declaration::FlexDirection(FlexDirection::RowReverse)],
    },
    LayoutRule {
        selector: Selector::Modifier {
            prefix: "sail-dir-",
            value: "column",
        },
        declarations: &[Declaration::FlexDirection(FlexDirection::Column)],
    },
    LayoutRule {
        selector: Selector::Modifier {
            prefix: "sail-dir-",
            value: "column-reverse",
        },
        declarations: &[Declaration::FlexDirection(FlexDirection::ColumnReverse)],
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
    // --- finite sx atoms -----------------------------------------------------
    LayoutRule {
        selector: Selector::Sx("hidden"),
        declarations: &[Declaration::Display(Display::None)],
    },
    // Style product: `style.display(none)` mints `sx-displaynone` (not bare `hidden`).
    LayoutRule {
        selector: Selector::Sx("displaynone"),
        declarations: &[Declaration::Display(Display::None)],
    },
    // Product `style.display(inline-flex|flex|block|…)` → `sx-display*`.
    // Longest / more-specific tokens first (inlineflex before inline, etc.).
    LayoutRule {
        selector: Selector::Sx("displayinlineflex"),
        declarations: &[Declaration::Display(Display::Flex)],
    },
    LayoutRule {
        selector: Selector::Sx("displayflex"),
        declarations: &[Declaration::Display(Display::Flex)],
    },
    LayoutRule {
        selector: Selector::Sx("displayinlineblock"),
        declarations: &[Declaration::Display(Display::Block)],
    },
    LayoutRule {
        selector: Selector::Sx("displayblock"),
        declarations: &[Declaration::Display(Display::Block)],
    },
    LayoutRule {
        selector: Selector::Sx("inline"),
        declarations: &[Declaration::Display(Display::Flex)],
    },
    LayoutRule {
        selector: Selector::Sx("inlineblock"),
        declarations: &[Declaration::Display(Display::Block)],
    },
    LayoutRule {
        selector: Selector::Sx("absolute"),
        declarations: &[Declaration::Position(Position::Absolute)],
    },
    LayoutRule {
        selector: Selector::Sx("relative"),
        declarations: &[Declaration::Position(Position::Relative)],
    },
    LayoutRule {
        selector: Selector::Sx("fixed"),
        declarations: &[Declaration::Position(Position::Absolute)],
    },
    LayoutRule {
        selector: Selector::Sx("sticky"),
        declarations: &[Declaration::Position(Position::Relative)],
    },
    // Product Style / participation attrs: `style.grow(1)` / `grow` → `sx-grow1`.
    // Residual bare `sx-grow` kept for fixtures + any leftover emit.
    LayoutRule {
        selector: Selector::Sx("grow1"),
        declarations: &[Declaration::FlexGrow(1.0)],
    },
    LayoutRule {
        selector: Selector::Sx("grow"),
        declarations: &[Declaration::FlexGrow(1.0)],
    },
    LayoutRule {
        selector: Selector::Sx("grow0"),
        declarations: &[Declaration::FlexGrow(0.0)],
    },
    LayoutRule {
        selector: Selector::Sx("shrink1"),
        declarations: &[Declaration::FlexShrink(1.0)],
    },
    LayoutRule {
        selector: Selector::Sx("shrink"),
        declarations: &[Declaration::FlexShrink(1.0)],
    },
    LayoutRule {
        selector: Selector::Sx("shrink0"),
        declarations: &[Declaration::FlexShrink(0.0)],
    },
    LayoutRule {
        selector: Selector::Sx("overflowhidden"),
        declarations: &[Declaration::Overflow {
            x: Overflow::Hidden,
            y: Overflow::Hidden,
        }],
    },
    LayoutRule {
        selector: Selector::Sx("overflowauto"),
        declarations: &[Declaration::Overflow {
            x: Overflow::Scroll,
            y: Overflow::Scroll,
        }],
    },
    LayoutRule {
        selector: Selector::Sx("overflowscroll"),
        declarations: &[Declaration::Overflow {
            x: Overflow::Scroll,
            y: Overflow::Scroll,
        }],
    },
    LayoutRule {
        selector: Selector::Sx("overflowclip"),
        declarations: &[Declaration::Overflow {
            x: Overflow::Clip,
            y: Overflow::Clip,
        }],
    },
    LayoutRule {
        selector: Selector::Sx("overflowvisible"),
        declarations: &[Declaration::Overflow {
            x: Overflow::Visible,
            y: Overflow::Visible,
        }],
    },
    LayoutRule {
        selector: Selector::Sx("overflowxhidden"),
        declarations: &[Declaration::Overflow {
            x: Overflow::Hidden,
            y: Overflow::Visible,
        }],
    },
    LayoutRule {
        selector: Selector::Sx("overflowxauto"),
        declarations: &[Declaration::Overflow {
            x: Overflow::Scroll,
            y: Overflow::Visible,
        }],
    },
    LayoutRule {
        selector: Selector::Sx("overflowxscroll"),
        declarations: &[Declaration::Overflow {
            x: Overflow::Scroll,
            y: Overflow::Visible,
        }],
    },
    LayoutRule {
        selector: Selector::Sx("overflowxclip"),
        declarations: &[Declaration::Overflow {
            x: Overflow::Clip,
            y: Overflow::Visible,
        }],
    },
    LayoutRule {
        selector: Selector::Sx("overflowyhidden"),
        declarations: &[Declaration::Overflow {
            x: Overflow::Visible,
            y: Overflow::Hidden,
        }],
    },
    LayoutRule {
        selector: Selector::Sx("overflowyauto"),
        declarations: &[Declaration::Overflow {
            x: Overflow::Visible,
            y: Overflow::Scroll,
        }],
    },
    LayoutRule {
        selector: Selector::Sx("overflowyscroll"),
        declarations: &[Declaration::Overflow {
            x: Overflow::Visible,
            y: Overflow::Scroll,
        }],
    },
    LayoutRule {
        selector: Selector::Sx("overflowyclip"),
        declarations: &[Declaration::Overflow {
            x: Overflow::Visible,
            y: Overflow::Clip,
        }],
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
// Scale-driven sx prefixes resolved at runtime through `sail_lint::scales`.
// ---------------------------------------------------------------------------

/// A scale prefix that maps a sanitized sx class like `sx-p4` to a layout
/// property. The value tail is fed to the scale resolver.
#[derive(Debug, Clone, Copy)]
pub enum SxScaleRule {
    Padding(SideSet),
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

/// Every `sx-*` prefix whose value lives on a closed scale. Order matters: the
/// longest prefix must be checked first.
///
/// **Product Style** mints classes from setter names + alphanumeric args:
/// `style.pad(4)` → `sx-pad4`, `style.padX(2)` → `sx-padX2`,
/// `style.minW(0)` → `sx-minW0` (camelCase preserved). Residual utility-string
/// prefixes (`sx-p4`, `sx-minw0`, …) stay so fixtures / any leftover emit still
/// resolve.
pub static SX_SCALE_RULES: &[(&'static str, SxScaleRule)] = &[
    // Product pad* (longest first: padX before pad).
    ("sx-padX", SxScaleRule::Padding(SideSet::X)),
    ("sx-padY", SxScaleRule::Padding(SideSet::Y)),
    ("sx-padT", SxScaleRule::Padding(SideSet::T)),
    ("sx-padR", SxScaleRule::Padding(SideSet::R)),
    ("sx-padB", SxScaleRule::Padding(SideSet::B)),
    ("sx-padL", SxScaleRule::Padding(SideSet::L)),
    ("sx-padS", SxScaleRule::Padding(SideSet::L)), // inline-start ≈ left for taffy
    ("sx-padE", SxScaleRule::Padding(SideSet::R)), // inline-end ≈ right for taffy
    ("sx-pad", SxScaleRule::Padding(SideSet::All)),
    // Product size atoms (camelCase minW / maxH).
    ("sx-minW", SxScaleRule::MinWidth),
    ("sx-minH", SxScaleRule::MinHeight),
    ("sx-maxW", SxScaleRule::MaxWidth),
    ("sx-maxH", SxScaleRule::MaxHeight),
    // Residual utility-string prefixes (p/px/…, minw lowercase).
    ("sx-px", SxScaleRule::Padding(SideSet::X)),
    ("sx-py", SxScaleRule::Padding(SideSet::Y)),
    ("sx-pt", SxScaleRule::Padding(SideSet::T)),
    ("sx-pr", SxScaleRule::Padding(SideSet::R)),
    ("sx-pb", SxScaleRule::Padding(SideSet::B)),
    ("sx-pl", SxScaleRule::Padding(SideSet::L)),
    ("sx-p", SxScaleRule::Padding(SideSet::All)),
    ("sx-minw", SxScaleRule::MinWidth),
    ("sx-minh", SxScaleRule::MinHeight),
    ("sx-maxw", SxScaleRule::MaxWidth),
    ("sx-maxh", SxScaleRule::MaxHeight),
    ("sx-w", SxScaleRule::Width),
    ("sx-h", SxScaleRule::Height),
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
