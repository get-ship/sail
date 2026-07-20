//! The template-directive vocabulary — one source of truth for every
//! attribute-position directive the parser accepts (`on:`, `bind:`, `use:`,
//! `prop:`, `attr:`, `motion:`, slots, `mutation`), every `on:`
//! event modifier, every `bind:` target, and every retired directive with
//! the thing to use instead.
//!
//! A sibling of `block_manifest` / `action_manifest` / `runtime_ambient`:
//! the closed vocabulary is data, not a hand-listed paragraph on the docs
//! page. The parser (`parse::template::attrs`) is the enforcement point;
//! its unknown-modifier and unknown-bind errors render THESE tables, and
//! the tests below compile one snippet per row, so a directive added to
//! the parser without a row here (or a row without parser support) fails
//! the build.

use serde::Serialize;

/// One attribute-position directive: authored syntax and a one-line summary.
#[derive(Debug, Clone, Serialize)]
pub struct DirectiveEntry {
    /// The directive as authored, e.g. `on:<event>={handler}`.
    pub syntax: &'static str,
    /// One-line description of what it does.
    pub summary: &'static str,
}

/// Attribute-position directive prefixes that win the lexer race against
/// plain attribute names. The grammar generates a dedicated rule for
/// each prefix so `bind:value`, `on:click`, etc. parse as directives
/// rather than as ordinary `attr_name` tokens. Keep this in sync with
/// the `DIRECTIVES` entries whose syntax starts with `<prefix>:`.
pub const DIRECTIVE_PREFIXES: &[&str] = &["bind", "on", "attr", "prop", "use", "motion"];

/// Element names that are directive-like in the grammar: the parser treats
/// them as ordinary HTML elements, but the highlight query gives them a
/// directive keyword scope so they stand out in editors. Must be kept in
/// sync with the rows in `DIRECTIVES` that describe `<scope>` / `<slot>`.
pub const DIRECTIVE_ELEMENTS: &[&str] = &["scope", "slot"];

/// The directive registry, in the order the docs present them.
pub const DIRECTIVES: &[DirectiveEntry] = &[
    DirectiveEntry {
        syntax: "on:<event>|<modifier>…={handler}",
        summary: "attach a DOM event handler; chain modifiers with | (full modifier table below)",
    },
    DirectiveEntry {
        syntax: "bind:value / bind:checked / bind:this",
        summary: "two-way binding sugar; the three targets are the whole surface (table below)",
    },
    DirectiveEntry {
        syntax: "use:<action> / use:<action>={param}",
        summary: "attach behavior bound to the element's lifecycle; built-ins live in the actions reference, a scope value works too",
    },
    DirectiveEntry {
        syntax: "prop:<name>={expr}",
        summary: "set a JS property on the element — for identity values that are not attributes",
    },
    DirectiveEntry {
        syntax: "attr:<name>={expr}",
        summary: "force attribute semantics for a dynamic value",
    },
    DirectiveEntry {
        syntax: "attr:{nameExpr}={expr}",
        summary: "dynamic attribute name — both the name and the value are expressions",
    },
    DirectiveEntry {
        syntax: "sx=\"atom atom …\"",
        summary: "static styling atoms from the closed sx vocabulary; sx={expr} is a compile error",
    },
    DirectiveEntry {
        syntax: "motion:<trigger>={…}",
        summary: "the one animation directive — triggers, properties, and config in the motion reference",
    },
    DirectiveEntry {
        syntax: "(--name) inside sx values",
        summary: "read a CSS custom property: any sx atom accepts (--name) and lowers to var(--name); typed actions write them",
    },
    DirectiveEntry {
        syntax: "slot=\"name\" (use site)",
        summary: "assign this subtree to the component's named slot; unassigned children fill the default slot",
    },
    DirectiveEntry {
        syntax: "<slot name=\"…\">fallback</slot>",
        summary: "declare a fill region; children of the <slot> render when the use site provides nothing",
    },
    DirectiveEntry {
        syntax: "<slot reflect-empty=\"data-attr\">",
        summary: "mirror the slot's emptiness onto a static data attribute, live — slot elements only",
    },
    DirectiveEntry {
        syntax: "mutation=\"ship.X.Y\" (on <form>)",
        summary: "wire a form's submit to a typed mutation",
    },
    DirectiveEntry {
        syntax: "&#123; and &#125; in text",
        summary: "a { with no matching } stays literal text; entity-escape a brace to force a literal one — there is no {{ escape",
    },
];

/// One `on:` event modifier. `canonical` is the long form the parser
/// documents; `alias` is the accepted short form when one exists; `arg`
/// names the optional `:arg` payload.
#[derive(Debug, Clone, Serialize)]
pub struct ModifierEntry {
    pub canonical: &'static str,
    pub alias: Option<&'static str>,
    pub arg: Option<&'static str>,
    pub summary: &'static str,
}

/// Every event modifier `on:` accepts, in documentation order. The parser's
/// `parse_modifier` match and this table are held in lockstep by the parity
/// test in `parse::template::attrs`.
pub const EVENT_MODIFIERS: &[ModifierEntry] = &[
    ModifierEntry {
        canonical: "preventDefault",
        alias: Some("prevent"),
        arg: None,
        summary: "call event.preventDefault() before the handler runs",
    },
    ModifierEntry {
        canonical: "stopPropagation",
        alias: Some("stop"),
        arg: None,
        summary: "call event.stopPropagation()",
    },
    ModifierEntry {
        canonical: "stopImmediatePropagation",
        alias: None,
        arg: None,
        summary: "call event.stopImmediatePropagation()",
    },
    ModifierEntry {
        canonical: "self",
        alias: None,
        arg: None,
        summary: "run only when the event's target is this element, not a descendant",
    },
    ModifierEntry {
        canonical: "trusted",
        alias: None,
        arg: None,
        summary: "run only for user-generated events (event.isTrusted)",
    },
    ModifierEntry {
        canonical: "key",
        alias: None,
        arg: Some("Name"),
        summary: "run only when the pressed key matches, e.g. on:keydown|key:Enter",
    },
    ModifierEntry {
        canonical: "debounce",
        alias: None,
        arg: Some("ms"),
        summary: "wait for ms of quiet before running the handler",
    },
    ModifierEntry {
        canonical: "throttle",
        alias: None,
        arg: Some("ms"),
        summary: "run the handler at most once per ms",
    },
];

/// One `bind:` target.
#[derive(Debug, Clone, Serialize)]
pub struct BindTargetEntry {
    pub target: &'static str,
    pub summary: &'static str,
}

/// Every `bind:` target. Anything else is a compile error that renders
/// this list.
pub const BIND_TARGETS: &[BindTargetEntry] = &[
    BindTargetEntry { target: "value", summary: "two-way binding to the element's value" },
    BindTargetEntry {
        target: "checked",
        summary: "two-way binding to a checkbox or radio checked state",
    },
    BindTargetEntry {
        target: "this",
        summary: "capture the element handle into a bare scope variable — write-once, not derivable",
    },
];

/// One retired directive and its replacement. The parser rejects these
/// with an error naming the replacement; this table is the docs' copy of
/// the same redirect.
#[derive(Debug, Clone, Serialize)]
pub struct RetiredEntry {
    pub syntax: &'static str,
    pub instead: &'static str,
}

/// Every directive the parser refuses by name, with what to use instead.
pub const RETIRED_DIRECTIVES: &[RetiredEntry] = &[
    RetiredEntry {
        syntax: "class:<name>",
        instead: "styling is sx (or a named use: action writing a custom property sx reads via (--name)); state is aria-*/closed sx state prefixes/layout — not a global CSS class toggle. Bare class= is also banned",
    },
    RetiredEntry {
        syntax: "style:<prop>",
        instead: "runtime geometry is owned by named, typed actions (use:rect, use:pin, use:virtualRow, …) — sx reads the variables they write via (--name)",
    },
    RetiredEntry {
        syntax: "part=\"…\"",
        instead: "style with sx on the element itself",
    },
    RetiredEntry {
        syntax: "transition: / in: / out:",
        instead: "motion:enter / motion:exit",
    },
];

/// One value form the `motion:` config grammar accepts.
#[derive(Debug, Clone, Serialize)]
pub struct MotionValueForm {
    pub form: &'static str,
    pub summary: &'static str,
}

/// The value grammar inside a `motion:` config object — what each shape of
/// value means. Enforced by `parse::template::attrs::parse_motion_value`.
pub const MOTION_VALUE_FORMS: &[MotionValueForm] = &[
    MotionValueForm {
        form: "'quoted'",
        summary: "a literal: a preset name, an easing keyword, or a unit-bearing magnitude like '120px'",
    },
    MotionValueForm {
        form: "bare number",
        summary: "milliseconds for time keys (duration, delay, stagger); a native magnitude for properties",
    },
    MotionValueForm {
        form: "[x1, y1, x2, y2]",
        summary: "an inline cubic-bezier easing (x values in 0..1)",
    },
    MotionValueForm {
        form: "unquoted expression",
        summary: "a reactive expression — how a signal drives motion; unit-bearing literals must be quoted",
    },
    MotionValueForm {
        form: "{ from, to, preset, duration, delay, easing, iterations, direction }",
        summary: "per-channel config object; preset supplies defaults, explicit keys override it",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tables_are_unique() {
        let mut names: Vec<&str> = EVENT_MODIFIERS.iter().map(|m| m.canonical).collect();
        names.extend(EVENT_MODIFIERS.iter().filter_map(|m| m.alias));
        let before = names.len();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), before, "modifier names and aliases must be unique");

        let mut targets: Vec<&str> = BIND_TARGETS.iter().map(|b| b.target).collect();
        targets.sort_unstable();
        targets.dedup();
        assert_eq!(targets.len(), BIND_TARGETS.len(), "bind targets must be unique");

        let mut syntaxes: Vec<&str> = DIRECTIVES.iter().map(|d| d.syntax).collect();
        syntaxes.sort_unstable();
        syntaxes.dedup();
        assert_eq!(syntaxes.len(), DIRECTIVES.len(), "directive rows must be unique");
    }
}
