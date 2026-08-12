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
        instead: "prefer accessible name, visible text, or authored id; event wiring via on: + closed-over state. `data-sail-host` is compiler-only L1 part identity for closed widgets — never author it; app tests use role/name/text",
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
    AuthoringBan {
        syntax: r#"<input type="checkbox">"#,
        rule: "sail-no-freeform-input-type",
        instead: "<checkbox>",
    },
    AuthoringBan {
        syntax: r#"<input type="radio">"#,
        rule: "sail-no-freeform-input-type",
        instead: "<radio> inside <radio-group>",
    },
    AuthoringBan {
        syntax: r#"<input type="range">"#,
        rule: "sail-no-freeform-input-type",
        instead: "<slider>",
    },
    AuthoringBan {
        syntax: r#"<input type="file">"#,
        rule: "sail-no-freeform-input-type",
        instead: "<file>",
    },
    AuthoringBan {
        syntax: r#"<input type="color">"#,
        rule: "sail-no-freeform-input-type",
        instead: "<color>",
    },
    AuthoringBan {
        syntax: r#"<input type="date|time|datetime-local|month|week">"#,
        rule: "sail-no-freeform-input-type",
        instead: r#"<date type="date|time|datetime-local|month|week">"#,
    },
    AuthoringBan {
        syntax: r#"<input type="submit">"#,
        rule: "sail-no-freeform-input-type",
        instead: r#"<button type="submit">"#,
    },
];

/// Freeform `<input type=…>` values that are not part of the Sail public API.
/// Hosts may still emit these types in closed structure; authors must use the
/// product tags listed in AUTHORING_BANS.
pub fn banned_input_type(ty: &str) -> Option<&'static str> {
    match ty {
        "checkbox" => Some("<checkbox>"),
        "radio" => Some("<radio> inside <radio-group>"),
        "range" => Some("<slider>"),
        "file" => Some("<file>"),
        "color" => Some("<color>"),
        "date" | "time" | "datetime-local" | "month" | "week" => {
            Some(r#"<date type="…">"#)
        }
        "submit" | "image" | "reset" | "button" => Some(r#"<button type="…">"#),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn banned_types_redirect() {
        assert_eq!(banned_input_type("range"), Some("<slider>"));
        assert_eq!(banned_input_type("checkbox"), Some("<checkbox>"));
        assert_eq!(banned_input_type("file"), Some("<file>"));
        assert_eq!(banned_input_type("color"), Some("<color>"));
        assert!(banned_input_type("date").unwrap().contains("date"));
        assert!(banned_input_type("text").is_none());
        assert!(banned_input_type("password").is_none());
    }
}
