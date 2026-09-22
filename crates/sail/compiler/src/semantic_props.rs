//! Semantic a11y props — author IR that the compiler stamps as role/aria.
//!
//! Authors never write free `aria-*` / free ARIA `role=` (Warning this cycle,
//! Error next). Genuine accessible state is authored as typed semantic props
//! (`label`, `current`, `expanded`, …). L1 anatomy declares fixed host roles
//! (navigation, status, tablist, …); the compiler lowers props → aria attrs
//! on the correct part.
//!
//! Vocabulary is fixed by the ARIA-1 audit of apps/studio + apps/sail-site
//! (helm scratch `ARIA-1-audit.md`). No other source.

use std::borrow::Cow;

use crate::ir::Instr;

/// How a semantic prop's value is typed for contracts / skill surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticKind {
    /// Free string (accessible name, description text).
    String,
    /// Bool → ARIA `"true"` / `"false"` (or remove when false for `hidden`).
    Bool,
    /// Closed string union.
    Union(&'static [&'static str]),
    /// Integer (aria-level).
    Number,
}

/// One semantic prop in the IR vocabulary.
#[derive(Debug, Clone, Copy)]
pub struct SemanticProp {
    /// Author-facing prop name (`label`, `current`, …).
    pub name: &'static str,
    /// DOM attribute stamped at lowering (`aria-label`, …).
    pub aria: &'static str,
    pub kind: SemanticKind,
    /// One-line description for skill / sail_surface.
    pub summary: &'static str,
}

/// Core vocabulary derived from the ARIA-1 audit.
///
/// `invalid` is intentional: it remains the design-system presence attr
/// (`[invalid]` CSS + `describedControl` stamps `aria-invalid`). It is still
/// a semantic prop authors type — it does not rewrite its attribute name.
pub const SEMANTIC_PROPS: &[SemanticProp] = &[
    SemanticProp {
        name: "label",
        aria: "aria-label",
        kind: SemanticKind::String,
        summary: "Accessible name (required on icon-only buttons)",
    },
    SemanticProp {
        name: "description",
        aria: "aria-description",
        kind: SemanticKind::String,
        summary: "Accessible description text (IDREF association is L1 anatomy)",
    },
    SemanticProp {
        name: "expanded",
        aria: "aria-expanded",
        kind: SemanticKind::Bool,
        summary: "Expand/collapse state for disclosures and drawers",
    },
    SemanticProp {
        name: "selected",
        // Dual-stamp: keeps design attr `selected` (chip compound)
        // AND stamps aria-selected (or aria-pressed on chip). See lower path.
        aria: "aria-selected",
        kind: SemanticKind::Bool,
        summary: "Selection state — stamps aria-selected; keeps selected for compound/pressed",
    },
    SemanticProp {
        name: "pressed",
        aria: "aria-pressed",
        kind: SemanticKind::Bool,
        summary: "Toggle pressed state for buttons",
    },
    SemanticProp {
        name: "current",
        aria: "aria-current",
        kind: SemanticKind::Union(&["page", "step", "location", "date", "time", "true", "false"]),
        summary: "Nav currency (page/step/…) for current location in a set",
    },
    SemanticProp {
        name: "level",
        aria: "aria-level",
        kind: SemanticKind::Number,
        summary: "Hierarchical level (heading / tree item)",
    },
    SemanticProp {
        name: "live",
        aria: "aria-live",
        kind: SemanticKind::Union(&["off", "polite", "assertive"]),
        summary: "Live region politeness",
    },
    SemanticProp {
        name: "invalid",
        aria: "invalid",
        kind: SemanticKind::Bool,
        summary: "Field invalidity (CSS [invalid]; describedControl stamps aria-invalid)",
    },
    SemanticProp {
        name: "hidden",
        aria: "aria-hidden",
        kind: SemanticKind::Bool,
        summary: "Hide decorative content from the accessibility tree",
    },
];

/// Lookup a semantic prop by author name.
pub fn lookup(name: &str) -> Option<&'static SemanticProp> {
    SEMANTIC_PROPS.iter().find(|p| p.name == name)
}

/// True when `name` is a free ARIA attribute authors must not write
/// (`aria-*` or the HTML `role` attribute used as ARIA role).
pub fn is_free_aria_attr(name: &str) -> bool {
    let n = name.as_bytes();
    if n.len() >= 5 && n[..5].eq_ignore_ascii_case(b"aria-") {
        return true;
    }
    name.eq_ignore_ascii_case("role")
}

/// Whether this semantic prop rewrites its primary attribute name at emit.
/// - `invalid` keeps its design-system name (describedControl stamps aria-invalid).
/// - `selected` keeps `selected` as primary (chip compound + use:pressed) and
///   dual-stamps `aria-selected` via [`dual_stamp_aria`].
pub fn rewrites_name(prop: &SemanticProp) -> bool {
    prop.name != prop.aria && prop.name != "selected"
}

/// Extra ARIA attr dual-stamped alongside the primary name, when any.
/// `selected` → `aria-pressed` on chip (toggle); `aria-selected` elsewhere.
/// Not on `<option>` — IDL `selected` is the platform contract there.
pub fn dual_stamp_aria(tag: &str, name: &str) -> Option<&'static str> {
    if name != "selected" {
        return None;
    }
    if tag.eq_ignore_ascii_case("option") {
        None
    } else if tag.eq_ignore_ascii_case("chip") {
        Some("aria-pressed")
    } else {
        Some("aria-selected")
    }
}

/// Lower an author attribute name to the DOM attribute for a given tag.
///
/// - Semantic props rewrite to their aria target (except `invalid` / `selected`).
/// - `selected` on `<option>` stays the IDL selected property name.
/// - Everything else is unchanged.
pub fn lower_attr_name(tag: &str, name: &str) -> Cow<'static, str> {
    if name == "selected" && tag.eq_ignore_ascii_case("option") {
        return Cow::Borrowed("selected");
    }
    if let Some(p) = lookup(name) {
        if rewrites_name(p) {
            return Cow::Borrowed(p.aria);
        }
        return Cow::Borrowed(p.name);
    }
    // name is not 'static — only return Borrowed for static tables
    Cow::Owned(name.to_string())
}

/// Lower when the result can be a static str (avoids alloc for common path).
pub fn lower_attr_name_str<'a>(tag: &str, name: &'a str) -> &'a str {
    lower_attr_name_str_valued(tag, name, None)
}

/// Like [`lower_attr_name_str`] but distinguishes HTML `hidden` (presence /
/// empty / `"hidden"`) from semantic `hidden="true"` → `aria-hidden`.
pub fn lower_attr_name_str_valued<'a>(tag: &str, name: &'a str, value: Option<&str>) -> &'a str {
    if name == "selected" && tag.eq_ignore_ascii_case("option") {
        return name;
    }
    // HTML global `hidden` must not become aria-hidden — it hides the box.
    // Semantic decorative hide is `hidden="true"` / `hidden={expr}` → aria-hidden.
    if name == "hidden" {
        match value {
            None => return "aria-hidden", // dynamic expr — semantic
            Some(v) if v.is_empty() || v.eq_ignore_ascii_case("hidden") => return "hidden",
            Some(_) => return "aria-hidden",
        }
    }
    if let Some(p) = lookup(name) {
        if rewrites_name(p) {
            return p.aria;
        }
        return p.name;
    }
    name
}

/// Whether this (possibly already-lowered) attr is an ARIA bool that should
/// stamp `"true"`/`"false"` strings rather than HTML presence/`""`.
pub fn is_aria_bool_attr(dom_name: &str) -> bool {
    matches!(
        dom_name,
        "aria-expanded"
            | "aria-selected"
            | "aria-pressed"
            | "aria-hidden"
            | "aria-invalid"
            | "aria-atomic"
            | "aria-busy"
            | "aria-disabled"
            | "aria-modal"
    )
}

/// Coerce a static semantic/ARIA bool value for DOM emit.
/// Presence / empty / `"true"` / `"1"` → `"true"`; `"false"` / `"0"` → `"false"`.
pub fn coerce_aria_bool_static(value: &str) -> &'static str {
    match value {
        "" | "true" | "1" | "True" | "TRUE" => "true",
        "false" | "0" | "False" | "FALSE" => "false",
        _ => {
            if value.eq_ignore_ascii_case("true") {
                "true"
            } else if value.eq_ignore_ascii_case("false") {
                "false"
            } else {
                // Non-bool string on a bool attr — pass through (defensive).
                // Callers should only use this for known bool attrs.
                if value.is_empty() {
                    "true"
                } else {
                    // leak-free: only static outcomes above; for odd values keep "true"
                    // when non-empty truthy-looking.
                    "true"
                }
            }
        }
    }
}

/// `<icon>` conditional anatomy. Labeled icons are images (role=img); unlabeled
/// (decorative) icons stay hidden (aria-hidden). Decided statically at emit —
/// no runtime switcher. Returns the single attribute to inject, or `None` when
/// the host already carries the relevant semantic on its own.
pub fn icon_anatomy_attr(attrs: &[crate::ir::AttrInstr]) -> Option<(&'static str, &'static str)> {
    use crate::ir::AttrInstr;
    let labeled = attrs.iter().any(|a| match a {
        AttrInstr::Static { name, value, .. } => {
            matches!(name.as_str(), "label" | "aria-label")
                && !value.trim_matches(['"', '\'']).is_empty()
        }
        AttrInstr::Expr { name, .. } => matches!(name.as_str(), "label" | "aria-label"),
        _ => false,
    });
    let has = |n: &str| {
        attrs.iter().any(|a| match a {
            AttrInstr::Static { name, .. } | AttrInstr::Expr { name, .. } => name == n,
            _ => false,
        })
    };
    if labeled {
        if has("role") {
            None
        } else {
            Some(("role", "img"))
        }
    } else if has("aria-hidden") {
        None
    } else {
        Some(("aria-hidden", "true"))
    }
}

/// Is this element the lowered `<icon>` host (identity via the L1 part stamp)?
pub fn is_icon_host_ir(attrs: &[crate::ir::AttrInstr]) -> bool {
    use crate::ir::AttrInstr;
    attrs.iter().any(
        |a| matches!(a, AttrInstr::Static { name, value, .. } if name == "data-sail-host" && value.trim_matches(['"', '\'']) == "icon"),
    )
}

/// Fixed host anatomy for L1 elements — role + fixed ARIA stamped onto the
/// element's structure root at compile time. Authors never write these.
#[derive(Debug, Clone, Copy)]
pub struct HostAnatomy {
    pub role: Option<&'static str>,
    pub live: Option<&'static str>,
    pub atomic: Option<&'static str>,
    pub label: Option<&'static str>,
}

/// Host anatomy table (stdlib element name → fixed semantics).
///
/// Widget-internal multi-part roles (theme-picker listbox/tabs/options, tooltip
/// bubble, field/error-boundary alert parts) are stamped by closed `use:` actions
/// (themeSelect / tipDescribe / describedControl / catchRender) — not free
/// author markup. Semantic props still cover labels on those parts.
pub const HOST_ANATOMY: &[(&str, HostAnatomy)] = &[
    (
        "navbar",
        HostAnatomy {
            role: Some("navigation"),
            live: None,
            atomic: None,
            label: None,
        },
    ),
    (
        "pagination",
        HostAnatomy {
            role: Some("navigation"),
            live: None,
            atomic: None,
            label: None,
        },
    ),
    (
        "page",
        HostAnatomy {
            role: Some("region"),
            live: None,
            atomic: None,
            label: Some("Page"),
        },
    ),
    (
        "breadcrumb",
        HostAnatomy {
            role: None, // native <nav>
            live: None,
            atomic: None,
            label: Some("Breadcrumb"),
        },
    ),
    (
        "alert",
        HostAnatomy {
            role: Some("status"),
            live: Some("polite"),
            atomic: Some("true"),
            label: None,
        },
    ),
    (
        "toast",
        HostAnatomy {
            role: Some("status"),
            live: Some("polite"),
            atomic: None,
            label: None,
        },
    ),
    (
        "empty-state",
        HostAnatomy {
            role: Some("note"),
            live: Some("off"),
            atomic: None,
            label: None,
        },
    ),
    (
        "error-state",
        HostAnatomy {
            role: Some("alert"),
            live: None,
            atomic: None,
            label: None,
        },
    ),
    (
        "spinner",
        HostAnatomy {
            role: Some("status"),
            live: None,
            atomic: None,
            label: Some("Loading"),
        },
    ),
    (
        "skeleton",
        HostAnatomy {
            role: Some("status"),
            live: None,
            atomic: None,
            label: Some("Loading"),
        },
    ),
    (
        "switch",
        HostAnatomy {
            role: Some("switch"),
            live: None,
            atomic: None,
            label: None,
        },
    ),
    (
        "tab-list",
        HostAnatomy {
            role: Some("tablist"),
            live: None,
            atomic: None,
            label: None,
        },
    ),
    (
        "tabs",
        HostAnatomy {
            role: Some("tablist"),
            live: None,
            atomic: None,
            label: None,
        },
    ),
    (
        "tab",
        HostAnatomy {
            // Trigger is the inner button (stamped in emit_element). The
            // host is display(contents) and must not itself be role=tab.
            role: None,
            live: None,
            atomic: None,
            label: None,
        },
    ),
    (
        "tab-panel",
        HostAnatomy {
            role: Some("tabpanel"),
            live: None,
            atomic: None,
            label: None,
        },
    ),
    (
        "radio-group",
        HostAnatomy {
            role: Some("radiogroup"),
            live: None,
            atomic: None,
            label: None,
        },
    ),
    (
        "tooltip",
        HostAnatomy {
            role: None, // role on bubble part, not host
            live: None,
            atomic: None,
            label: None,
        },
    ),
    (
        "field",
        HostAnatomy {
            role: Some("group"),
            live: None,
            atomic: None,
            label: None,
        },
    ),
    (
        "card",
        HostAnatomy {
            role: Some("region"),
            live: None,
            atomic: None,
            label: Some("Card"),
        },
    ),
    (
        "error-boundary",
        HostAnatomy {
            role: Some("group"),
            live: None,
            atomic: None,
            label: None,
        },
    ),
    (
        "code-block",
        HostAnatomy {
            role: Some("region"),
            live: None,
            atomic: None,
            label: None,
        },
    ),
    (
        "prose",
        HostAnatomy {
            role: Some("article"),
            live: None,
            atomic: None,
            label: None,
        },
    ),
    (
        "toast-region",
        HostAnatomy {
            role: Some("region"),
            live: None,
            atomic: None,
            label: None,
        },
    ),
    (
        "avatar",
        HostAnatomy {
            role: Some("img"),
            live: None,
            atomic: None,
            label: None,
        },
    ),
    (
        "popover",
        HostAnatomy {
            role: Some("dialog"),
            live: None,
            atomic: None,
            label: None,
        },
    ),
    (
        "ticker",
        HostAnatomy {
            role: Some("status"),
            live: None,
            atomic: None,
            label: None,
        },
    ),
    (
        "scroll-area",
        HostAnatomy {
            role: Some("region"),
            live: None,
            atomic: None,
            label: None,
        },
    ),
    (
        "file",
        HostAnatomy {
            role: Some("group"),
            live: None,
            atomic: None,
            label: None,
        },
    ),
];

pub fn host_anatomy(tag: &str) -> Option<&'static HostAnatomy> {
    HOST_ANATOMY.iter().find(|(t, _)| *t == tag).map(|(_, a)| a)
}

/// Stamp compiler anatomy onto a language widget's template IR (mount-form).
/// `tab` trigger/body are inner nodes; other hosts take `HOST_ANATOMY.role`.
pub fn stamp_language_anatomy(program: &mut crate::ir::Program, tag: &str) {
    if tag == "tab" {
        stamp_first_tag(&mut program.template, "button", "role", "tab");
        stamp_first_tag(&mut program.template, "div", "role", "tabpanel");
        return;
    }
    let Some(role) = host_anatomy(tag).and_then(|a| a.role) else {
        return;
    };
    if let Some(Instr::Element(root)) = program
        .template
        .iter_mut()
        .find(|i| matches!(i, Instr::Element(_)))
    {
        stamp_attr(&mut root.attrs, "role", role);
    }
}

fn stamp_first_tag(instrs: &mut [Instr], tag: &str, name: &str, value: &str) {
    for i in instrs {
        match i {
            Instr::Element(el) => {
                if el.tag.eq_ignore_ascii_case(tag) {
                    stamp_attr(&mut el.attrs, name, value);
                    return;
                }
                stamp_first_tag(&mut el.children, tag, name, value);
            }
            Instr::If(v) => {
                for b in &mut v.branches {
                    stamp_first_tag(&mut b.body, tag, name, value);
                }
                if let Some(eb) = &mut v.else_body {
                    stamp_first_tag(eb, tag, name, value);
                }
            }
            Instr::For(v) => {
                stamp_first_tag(&mut v.body, tag, name, value);
                if let Some(eb) = &mut v.else_body {
                    stamp_first_tag(eb, tag, name, value);
                }
            }
            Instr::Match(v) => {
                for c in &mut v.cases {
                    stamp_first_tag(&mut c.body, tag, name, value);
                }
                if let Some(d) = &mut v.default {
                    stamp_first_tag(d, tag, name, value);
                }
            }
            Instr::When(v) => {
                stamp_first_tag(&mut v.body, tag, name, value);
                if let Some(e) = &mut v.else_body {
                    stamp_first_tag(e, tag, name, value);
                }
            }
            _ => {}
        }
    }
}

fn stamp_attr(attrs: &mut Vec<crate::ir::AttrInstr>, name: &str, value: &str) {
    if attrs.iter().any(|a| match a {
        crate::ir::AttrInstr::Static { name: n, .. } => n == name,
        _ => false,
    }) {
        return;
    }
    attrs.push(crate::ir::AttrInstr::Static {
        name: name.into(),
        value: value.into(),
        span: crate::diagnostics::Span::point(0),
    });
}

/// Live-region `role` / `aria-live` from L1 `tone`. Compile-time; not an action.
/// `danger` is the stdlib token; `destructive` is a leftover synonym.
pub fn live_region_for_tone(tone: &str) -> (&'static str, &'static str) {
    match tone {
        "warning" | "danger" | "destructive" => ("alert", "assertive"),
        _ => ("status", "polite"),
    }
}

/// Live-region from L1 `<empty-state>` `variant`. `connecting` is polite status;
/// every other variant stays `note` / `off` (HOST_ANATOMY default).
pub fn live_region_for_empty_variant(variant: &str) -> (&'static str, &'static str) {
    if variant == "connecting" {
        ("status", "polite")
    } else {
        ("note", "off")
    }
}

/// Rewrite spliced host `role` / `aria-live` from a resolved `tone` attr.
/// Dynamic `{…}` tones keep the anatomy default (first paint).
pub fn apply_tone_live_region(attrs: &mut Vec<(String, String)>) {
    apply_live_from_attr(attrs, "tone", "info", live_region_for_tone);
}

/// Rewrite spliced host `role` / `aria-live` from `<empty-state>` `variant`.
pub fn apply_variant_live_region(attrs: &mut Vec<(String, String)>) {
    apply_live_from_attr(attrs, "variant", "default", live_region_for_empty_variant);
}

fn apply_live_from_attr(
    attrs: &mut Vec<(String, String)>,
    attr: &str,
    default: &str,
    map: fn(&str) -> (&'static str, &'static str),
) {
    let raw = attrs
        .iter()
        .find(|(n, _)| n == attr)
        .map(|(_, v)| v.trim_matches(['"', '\'']).to_string())
        .unwrap_or_else(|| default.to_string());
    if raw.starts_with('{') {
        return;
    }
    let (role, live) = map(&raw);
    upsert_attr(attrs, "role", &format!("\"{role}\""));
    upsert_attr(attrs, "aria-live", &format!("\"{live}\""));
}

fn upsert_attr(attrs: &mut Vec<(String, String)>, name: &str, value: &str) {
    if let Some(slot) = attrs.iter_mut().find(|(n, _)| n == name) {
        slot.1 = value.to_string();
    } else {
        attrs.push((name.to_string(), value.to_string()));
    }
}

/// Inject missing host-anatomy attributes into a serialized structure open tag.
/// `structure_html` is the full snippet from `serialize_static`.
pub fn stamp_host_anatomy(element_name: &str, structure_html: &str) -> String {
    let Some(anatomy) = host_anatomy(element_name) else {
        return structure_html.to_string();
    };
    let Some(gt) = structure_html.find('>') else {
        return structure_html.to_string();
    };
    // Void / self-close safety: only rewrite the first open tag.
    let open = &structure_html[..=gt];
    let rest = &structure_html[gt + 1..];
    let mut inject = String::new();
    let ensure = |open: &str, attr: &str, value: &str, inject: &mut String| {
        // Already present (author or prior stamp)?
        let needle = format!("{attr}=");
        let bare = format!(" {attr} ");
        let bare_end = format!(" {attr}>");
        if open.contains(&needle) || open.contains(&bare) || open.contains(&bare_end) {
            return;
        }
        inject.push(' ');
        inject.push_str(attr);
        inject.push_str("=\"");
        inject.push_str(value);
        inject.push('"');
    };
    if let Some(role) = anatomy.role {
        ensure(open, "role", role, &mut inject);
    }
    if let Some(live) = anatomy.live {
        ensure(open, "aria-live", live, &mut inject);
    }
    if let Some(atomic) = anatomy.atomic {
        ensure(open, "aria-atomic", atomic, &mut inject);
    }
    if let Some(label) = anatomy.label {
        ensure(open, "aria-label", label, &mut inject);
    }
    if inject.is_empty() {
        return structure_html.to_string();
    }
    // Insert before the closing `>` of the open tag.
    let mut out = String::with_capacity(structure_html.len() + inject.len());
    out.push_str(&open[..open.len() - 1]);
    out.push_str(&inject);
    out.push('>');
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vocabulary_covers_audit_core() {
        for name in [
            "label",
            "description",
            "expanded",
            "selected",
            "pressed",
            "current",
            "level",
            "live",
            "invalid",
            "hidden",
        ] {
            assert!(lookup(name).is_some(), "missing semantic prop {name}");
        }
    }

    #[test]
    fn lowers_label_to_aria_label() {
        assert_eq!(lower_attr_name_str("button", "label"), "aria-label");
        assert_eq!(lower_attr_name_str("div", "current"), "aria-current");
        assert_eq!(lower_attr_name_str("button", "pressed"), "aria-pressed");
        assert_eq!(lower_attr_name_str("span", "hidden"), "aria-hidden");
    }

    #[test]
    fn invalid_and_selected_keep_design_name() {
        assert_eq!(lower_attr_name_str("input", "invalid"), "invalid");
        assert_eq!(lower_attr_name_str("button", "selected"), "selected");
        assert_eq!(dual_stamp_aria("button", "selected"), Some("aria-selected"));
    }

    #[test]
    fn option_selected_stays_idl() {
        assert_eq!(lower_attr_name_str("option", "selected"), "selected");
        assert_eq!(dual_stamp_aria("option", "selected"), None);
    }

    #[test]
    fn free_aria_detection() {
        assert!(is_free_aria_attr("aria-label"));
        assert!(is_free_aria_attr("ARIA-current"));
        assert!(is_free_aria_attr("role"));
        assert!(!is_free_aria_attr("label"));
        assert!(!is_free_aria_attr("token"));
    }

    #[test]
    fn stamp_navbar_role() {
        let s = stamp_host_anatomy(
            "navbar",
            r#"<row class="sail-navbar" data-sail-host="navbar"></row>"#,
        );
        assert!(s.contains(r#"role="navigation""#), "{s}");
    }

    #[test]
    fn stamp_does_not_duplicate() {
        let s = stamp_host_anatomy(
            "navbar",
            r#"<row role="navigation" class="sail-navbar"></row>"#,
        );
        assert_eq!(s.matches("role=").count(), 1, "{s}");
    }

    #[test]
    fn stamp_spinner_label() {
        let s = stamp_host_anatomy("spinner", r#"<span class="sail-spinner"></span>"#);
        assert!(s.contains(r#"role="status""#), "{s}");
        assert!(s.contains(r#"aria-label="Loading""#), "{s}");
    }

    #[test]
    fn live_region_for_tone_urgent_vs_status() {
        assert_eq!(live_region_for_tone("info"), ("status", "polite"));
        assert_eq!(live_region_for_tone("success"), ("status", "polite"));
        assert_eq!(live_region_for_tone("warning"), ("alert", "assertive"));
        assert_eq!(live_region_for_tone("danger"), ("alert", "assertive"));
        assert_eq!(live_region_for_tone("destructive"), ("alert", "assertive"));
    }

    #[test]
    fn apply_tone_live_region_overrides_anatomy_default() {
        let mut attrs = vec![
            ("role".into(), "\"status\"".into()),
            ("aria-live".into(), "\"polite\"".into()),
            ("tone".into(), "\"warning\"".into()),
        ];
        apply_tone_live_region(&mut attrs);
        let role = attrs
            .iter()
            .find(|(n, _)| n == "role")
            .map(|(_, v)| v.as_str());
        let live = attrs
            .iter()
            .find(|(n, _)| n == "aria-live")
            .map(|(_, v)| v.as_str());
        assert_eq!(role, Some("\"alert\""));
        assert_eq!(live, Some("\"assertive\""));
    }

    #[test]
    fn live_region_for_empty_variant_connecting() {
        assert_eq!(live_region_for_empty_variant("default"), ("note", "off"));
        assert_eq!(
            live_region_for_empty_variant("placeholder"),
            ("note", "off")
        );
        assert_eq!(
            live_region_for_empty_variant("connecting"),
            ("status", "polite")
        );
    }

    #[test]
    fn apply_variant_live_region_upgrades_connecting() {
        let mut attrs = vec![
            ("role".into(), "\"note\"".into()),
            ("aria-live".into(), "\"off\"".into()),
            ("variant".into(), "\"connecting\"".into()),
        ];
        apply_variant_live_region(&mut attrs);
        let role = attrs
            .iter()
            .find(|(n, _)| n == "role")
            .map(|(_, v)| v.as_str());
        let live = attrs
            .iter()
            .find(|(n, _)| n == "aria-live")
            .map(|(_, v)| v.as_str());
        assert_eq!(role, Some("\"status\""));
        assert_eq!(live, Some("\"polite\""));
    }
}
