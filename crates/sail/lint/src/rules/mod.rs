//! Rules registry. Each module is one rule.
//!
//! Conventions:
//! - Public `RULE_ID: &str` constant per module
//! - Public `check(path, source) -> Vec<Finding>` per module
//! - Findings carry the `RULE_ID` in the `rule` field
//!
//! Adding a rule = create the module, add the entry to the registry below.

use crate::report::Finding;
use std::path::Path;

pub mod action_no_style;
pub mod animate_token_only;
pub mod attr_bool_prop;
pub mod attr_presence;
pub mod controller_context;
pub mod controller_empty;
pub mod controller_matchmedia_layout;
pub mod controller_navigation;
pub mod controller_no_ambient_authority;
pub mod controller_no_load_state;
pub mod controller_purity;
pub mod controller_redundant_ambient_import;
pub mod controller_set_value;
pub mod controller_state_cast;
pub mod controller_state_set_call;
pub mod css_content_text;
pub mod css_literal;
pub mod css_literals;
pub mod css_token;
pub mod dimension_literal;
pub mod display_on_layout;
pub mod event_name;
pub mod font_literal;
pub mod for_key;
pub mod height_chain;
pub mod layout;
pub mod layout_primitive;
pub mod legacy_layout;
pub mod ambient;
pub mod no_ambient_dom;
pub mod no_ambient_events;
pub mod no_ambient_nav;
pub mod no_ambient_network;
pub mod no_ambient_storage;
pub mod no_ambient_timers;
pub mod no_data_attr;
pub mod no_class;
pub mod no_imperative_style;
pub mod no_inline_style;
pub mod no_raw_container;
pub mod no_script;
pub mod no_style_block;
pub mod no_sx;
pub mod no_undeclared_stylesheet;
pub mod no_variants;
pub mod participation_style;
pub mod prefer_primitive;
pub mod prefer_query;
pub mod primary_for_status;
pub mod prop_value;
pub mod route_inline_style;
pub mod space_literal;
pub mod surface_not_card;
pub mod sx;
pub mod virtualizer;
pub type RuleFn = fn(&Path, &str) -> Vec<Finding>;

/// Rules that lint a paired `<stem>.controller.ts` source. The dispatcher
/// in `lib.rs::lint_controller_source` walks this slice in order.
pub const CONTROLLER_RULES: &[(&str, RuleFn)] = &[
    (no_imperative_style::RULE_ID, no_imperative_style::check),
    (controller_purity::RULE_ID, controller_purity::check),
    (controller_empty::RULE_ID, controller_empty::check),
    (
        controller_redundant_ambient_import::RULE_ID,
        controller_redundant_ambient_import::check,
    ),
    (controller_set_value::RULE_ID, controller_set_value::check),
    (controller_state_cast::RULE_ID, controller_state_cast::check),
    (
        controller_state_set_call::RULE_ID,
        controller_state_set_call::check,
    ),
    (controller_navigation::RULE_ID, controller_navigation::check),
    (controller_context::RULE_ID, controller_context::check),
    (
        controller_no_ambient_authority::RULE_ID,
        controller_no_ambient_authority::check,
    ),
    (
        controller_no_load_state::RULE_ID,
        controller_no_load_state::check,
    ),
    (
        controller_matchmedia_layout::RULE_ID,
        controller_matchmedia_layout::check,
    ),
    (prefer_query::RULE_ID, prefer_query::check),
];

/// Rules that lint a `*.sail` template source. The dispatcher in
/// `lib.rs::lint_sail_template_source` walks this slice in order.
pub const SUI_TEMPLATE_RULES: &[(&str, RuleFn)] = &[
    // Also on the TEMPLATE path, not just when a pack happens to have a .ts file
    // linted. Enforcement that depends on the incidental presence of another file
    // is not enforcement — an undeclared stylesheet must fail the compile of any
    // source in the package that owns it.
    (
        no_undeclared_stylesheet::RULE_ID,
        no_undeclared_stylesheet::check,
    ),
    (css_literal::RULE_ID, css_literal::check),
    (css_literals::RULE_ID, css_literals::check),
    (css_content_text::RULE_ID, css_content_text::check),
    (css_token::RULE_ID, css_token::check),
    (font_literal::RULE_ID, font_literal::check),
    (event_name::RULE_ID, event_name::check),
    (prop_value::RULE_ID, prop_value::check),
    (attr_presence::RULE_ID, attr_presence::check),
    (attr_bool_prop::RULE_ID, attr_bool_prop::check),
    (space_literal::RULE_ID, space_literal::check),
    (layout_primitive::RULE_ID, layout_primitive::check),
    (prefer_primitive::RULE_ID, prefer_primitive::check),
    (dimension_literal::RULE_ID, dimension_literal::check),
    (primary_for_status::RULE_ID, primary_for_status::check),
    (route_inline_style::RULE_ID, route_inline_style::check),
    (surface_not_card::RULE_ID, surface_not_card::check),
    (for_key::RULE_ID, for_key::check),
    (sx::RULE_ID, sx::check),
    (layout::RULE_ID, layout::check),
    (legacy_layout::RULE_ID, legacy_layout::check),
    (height_chain::RULE_ID, height_chain::check),
    (virtualizer::RULE_ID, virtualizer::check),
    (participation_style::RULE_ID, participation_style::check),
    (display_on_layout::RULE_ID, display_on_layout::check),
    (no_raw_container::RULE_ID, no_raw_container::check),
    (no_script::RULE_ID, no_script::check),
    (no_style_block::RULE_ID, no_style_block::check),
    (no_sx::RULE_ID, no_sx::check),
    (no_variants::RULE_ID, no_variants::check),
    (no_class::RULE_ID, no_class::check),
    (no_inline_style::RULE_ID, no_inline_style::check),
    (animate_token_only::RULE_ID, animate_token_only::check),
    // SPEC-DOMDEC-1: data-* ban on app .sail only (path-gated inside rule).
    (no_data_attr::RULE_ID, no_data_attr::check),
];

/// Rules that lint a pack's `.ts` source files (composition modules,
/// glue code, helpers). Dispatches only on paths under
/// `packages/components/sail/**/src/**` — the framework layer
/// (`packages/sail/runtime/src/**`) is exempt, and `<stem>.controller.ts`
/// files are matched by `is_controller_path` first.
///
/// Why a separate list, not CONTROLLER_RULES: the floor rule
/// (`sail-no-imperative-style`) is the only one that needs to reach into
/// this surface today; the controller family stays scope-bound.
pub const PACK_SOURCE_RULES: &[(&str, RuleFn)] = &[
    (no_imperative_style::RULE_ID, no_imperative_style::check),
    (
        no_undeclared_stylesheet::RULE_ID,
        no_undeclared_stylesheet::check,
    ),
];

/// Rules that lint plain app TypeScript under `apps/*/src/**/*.ts`.
///
/// Closes the escape hatch where studio (and sibling apps) put CSSOM /
/// ambient DOM in a side module that was never a `.controller.ts` and
/// therefore never saw the floor. Floor + ambient-DOM ban — not the full
/// controller purity suite (those rules assume controller shape).
pub const APP_SOURCE_RULES: &[(&str, RuleFn)] = &[
    (no_imperative_style::RULE_ID, no_imperative_style::check),
    (no_ambient_dom::RULE_ID, no_ambient_dom::check),
    // SPEC-DOMDEC-1 ambient family (path-gated inside each rule).
    (no_ambient_events::RULE_ID, no_ambient_events::check),
    (no_ambient_timers::RULE_ID, no_ambient_timers::check),
    (no_ambient_storage::RULE_ID, no_ambient_storage::check),
    (no_ambient_nav::RULE_ID, no_ambient_nav::check),
    (no_ambient_network::RULE_ID, no_ambient_network::check),
    // SPEC-CTX-1 teach-side (warn only).
    (prefer_query::RULE_ID, prefer_query::check),
];

/// Rules that lint a shipped / author `use:` Action body.
///
/// Dispatches on `packages/sail/runtime/src/actions/**` (and any future
/// path `is_action_source_path` admits). Bench harness sources under
/// `tests/perf/` are exempt at the path gate — interop may need `class=`.
///
/// Floor matches the template invariants: no class, no style assignment,
/// only `setProperty`/`removeProperty` on `--*` custom properties.
pub const ACTION_RULES: &[(&str, RuleFn)] = &[(action_no_style::RULE_ID, action_no_style::check)];
