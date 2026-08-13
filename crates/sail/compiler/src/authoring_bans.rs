//! Authoring bans projected into language-surface / sail_surface docs.
//!
//! Not the full lint registry — the closed set of constructs that must not
//! appear in app `.sail` markup, with the rule id that refuses them and the
//! product replacement. Sourced for docs so gen surface and MCP stay in lockstep.

use serde::Serialize;

/// One refused authoring form with its rule id and replacement.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct AuthoringBan {
    /// Authored syntax / construct that is refused.
    pub syntax: &'static str,
    /// Lint / check rule id.
    pub rule: &'static str,
    /// What to write instead.
    pub instead: &'static str,
}

/// Floor / app bans that language-surface and reference pages must list.
pub const AUTHORING_BANS: &[AuthoringBan] = &[
    AuthoringBan {
        syntax: "data-*",
        rule: "sail-no-data-attr",
        instead: "prefer accessible name, visible text, or authored id; event wiring via on: + closed-over state. `data-sail-host` is compiler-only L1 part identity for closed widgets — never author it; app tests use role/name/text. `data-theme` / `data-product` / `data-mode` are theme substrate — use theme=",
    },
    AuthoringBan {
        syntax: "L1 component token in app style() / unsanctioned attrs",
        rule: "sail-no-component-token",
        instead: "component tokens are theme-owned; override via theme= or the theme document. App style() uses semantic slots (bg(primary))",
    },
    AuthoringBan {
        syntax: "free aria-* / role=",
        rule: "sail-no-free-aria",
        instead: "typed semantic props (label, description, expanded, selected, pressed, current, level, live, invalid, hidden); fixed roles from native tags or L1 host anatomy",
    },
    AuthoringBan {
        syntax: "raw <style>",
        rule: "sail-no-style-block",
        instead: "Style product block style({ box: …, paint: … }) and design tokens (floor)",
    },
    AuthoringBan {
        syntax: "raw <script>",
        rule: "sail-no-script",
        instead: "imperative logic lives in the <controller> block",
    },
];
