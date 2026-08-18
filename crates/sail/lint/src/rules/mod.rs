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

pub mod action_host_only;
pub mod action_no_network;
pub mod action_no_state;
pub mod action_no_style;
pub mod action_signature;
pub mod ambient;
pub mod animate_token_only;
pub mod attr_bool_prop;
pub mod attr_presence;
pub mod controller_context;
pub mod controller_empty;
pub mod controller_import_convention;
pub mod controller_matchmedia_layout;
pub mod controller_navigation;
pub mod controller_no_ambient_authority;
pub mod controller_no_load_state;
pub mod controller_purity;
pub mod controller_redundant_ambient_import;
pub mod controller_set_value;
pub mod controller_size;
pub mod controller_state_cast;
pub mod controller_state_set_call;
pub mod css_content_text;
pub mod css_literal;
pub mod css_literals;
pub mod css_token;
pub mod dimension_literal;
pub mod directory_jurisdiction;
pub mod display_on_layout;
pub mod event_name;
pub mod flex_trap;
pub mod font_literal;
pub mod for_key;
pub mod framework_import_teaching;
pub mod grow_outside_host;
pub mod height_chain;
pub mod layout;
pub mod layout_primitive;
// S2.3: dual ambient-DOM floors retired (TypeEnv::Sail is the law).
pub mod no_ambient_network;
pub mod no_ambient_timers;
pub mod no_class;
pub mod no_component_token;
pub mod no_data_attr;
pub mod no_free_aria;
pub mod no_imperative_style;
pub mod no_inline_style;
pub mod no_node_import;
pub mod no_raw_container;
pub mod no_script;
pub mod no_style_block;
pub mod no_undeclared_stylesheet;
pub mod participation_style;
pub mod prefer_primitive;
pub mod prefer_query;
pub mod primary_for_status;
pub mod prop_value;
pub mod route_inline_style;
pub mod space_literal;
pub mod style_concern;
pub mod style_privilege;
pub mod style_rule;
pub mod style_value_enumeration;
pub mod surface_not_card;
pub mod virtualizer;
pub type RuleFn = fn(&Path, &str) -> Vec<Finding>;

/// Rules that lint a paired `<stem>.controller.ts` source. The dispatcher
/// in `lib.rs::lint_controller_source` walks this slice in order.
pub const CONTROLLER_RULES: &[(&str, RuleFn)] = &[
    (no_imperative_style::RULE_ID, no_imperative_style::check),
    (controller_purity::RULE_ID, controller_purity::check),
    (controller_empty::RULE_ID, controller_empty::check),
    (controller_size::RULE_ID, controller_size::check),
    (
        controller_redundant_ambient_import::RULE_ID,
        controller_redundant_ambient_import::check,
    ),
    (
        framework_import_teaching::RULE_ID,
        framework_import_teaching::check,
    ),
    (
        controller_import_convention::RULE_ID,
        controller_import_convention::check,
    ),
    (no_node_import::RULE_ID, no_node_import::check),
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
    // Also on the TEMPLATE path, not just when a components package happens to have a .ts file
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
    (style_privilege::RULE_ID, style_privilege::check),
    (surface_not_card::RULE_ID, surface_not_card::check),
    (for_key::RULE_ID, for_key::check),
    (style_rule::RULE_ID, style_rule::check),
    (
        style_value_enumeration::RULE_ID,
        style_value_enumeration::check,
    ),
    (layout::RULE_ID, layout::check),
    (height_chain::RULE_ID, height_chain::check),
    (virtualizer::RULE_ID, virtualizer::check),
    (participation_style::RULE_ID, participation_style::check),
    (grow_outside_host::RULE_ID, grow_outside_host::check),
    (flex_trap::RULE_ID, flex_trap::check),
    (display_on_layout::RULE_ID, display_on_layout::check),
    (no_raw_container::RULE_ID, no_raw_container::check),
    (no_script::RULE_ID, no_script::check),
    (no_style_block::RULE_ID, no_style_block::check),
    (no_class::RULE_ID, no_class::check),
    (no_component_token::RULE_ID, no_component_token::check),
    (no_inline_style::RULE_ID, no_inline_style::check),
    (animate_token_only::RULE_ID, animate_token_only::check),
    // SPEC-DOMDEC-1: data-* ban on app .sail only (path-gated inside rule).
    (no_data_attr::RULE_ID, no_data_attr::check),
    // SPEC-ARIA-1: free aria-*/role= Warning this cycle (apps only).
    (no_free_aria::RULE_ID, no_free_aria::check),
];

/// Rules that lint a components package's `.ts` source files (composition modules,
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
    (no_node_import::RULE_ID, no_node_import::check),
];

/// Rules that lint plain app TypeScript under `apps/*/src/**/*.ts`.
///
/// Closes the escape hatch where studio (and sibling apps) put CSSOM in a
/// side module that was never a `.controller.ts` and therefore never saw the
/// floor. Ambient `document`/`window`/`localStorage`/`location` are unnameable
/// under TypeEnv::Sail (S2.1) — dual-floor regex rules for those were retired
/// in S2.3. Kept: imperative-style, timers, network, prefer-query.
/// Not the full controller purity suite (those rules assume controller shape).
pub const APP_SOURCE_RULES: &[(&str, RuleFn)] = &[
    (no_imperative_style::RULE_ID, no_imperative_style::check),
    // SPEC-DOMDEC-1 residual ambient family (path-gated inside each rule).
    // Timers: type-env alone may not ban top-level timer *calls*.
    (no_ambient_timers::RULE_ID, no_ambient_timers::check),
    // Network: security floor, not dual with DOM lib (SCOUT-S2 §4b).
    (no_ambient_network::RULE_ID, no_ambient_network::check),
    // SPEC-AMBIENT-SURFACE-1 §2 — framework import teaching (Error).
    (
        framework_import_teaching::RULE_ID,
        framework_import_teaching::check,
    ),
    // Node/Bun tooling boundary (product src only).
    (no_node_import::RULE_ID, no_node_import::check),
    // SPEC-CTX-1 teach-side (warn only).
    (prefer_query::RULE_ID, prefer_query::check),
    // Action-typed functions outside src/actions/** — LAW: compositions wire
    // state; the DOM floor lives in src/actions/.
    (
        directory_jurisdiction::RULE_ID,
        directory_jurisdiction::check,
    ),
];

/// Rules that lint an **author** `use:` Action body (ACTIONS-1 regime).
///
/// Dispatches on app `src/actions/**` (and any path `is_action_source_path`
/// admits). The framework catalog (`packages/sail/runtime/src/actions/*`) is
/// **trusted** and not subject to this floor.
///
/// Floor (ERROR): no style/class/layout writes; no state primitives; no
/// network; handed host only; Action signature.
pub const ACTION_RULES: &[(&str, RuleFn)] = &[
    (action_no_style::RULE_ID, action_no_style::check),
    (action_no_state::RULE_ID, action_no_state::check),
    (action_no_network::RULE_ID, action_no_network::check),
    (action_host_only::RULE_ID, action_host_only::check),
    (action_signature::RULE_ID, action_signature::check),
];
