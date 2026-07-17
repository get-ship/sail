//! THE L1 SURFACE — one source of truth for every element the language
//! ships. The compiler is the SSOT: a `.sail` element source is its own
//! declaration, and a native reskin is a row in the table below. Everything
//! the agent (SKILL.md, LSP) and the audit report see is derived from
//! this manifest, so "what elements does SAIL give me?" has a single
//! answer that cannot disagree with what the runtime actually styles.
//!
//! # Why this is not a document
//!
//! The pre-existing SKILL.md enumerated the L1 surface as a hand-written
//! prose list (the `<button>` / `<sail-button>` row near the top). It went
//! stale — the user had to tell the agent `<select>` existed. A document
//! is a comment that is allowed to lie. This module is the build's view
//! of the surface; the consistency test in
//! `crates/sail/compiler/tests/l1_manifest.rs` fails when it disagrees with
//! either the stdlib registry (a `.sail` removed) or the runtime's
//! `LIGHT_CONTRACTS_CSS` (a reskin added without an entry).
//!
//! # What an entry is
//!
//! One row per L1 element. `StdlibElement` rows are derived from the
//! compiled `.sail` (props, root tag, native flag, one-line description
//! pulled from the source's leading docblock). `NativeReskin` rows are
//! declared here, with the CSS selectors the runtime MUST contain, the
//! key attribute(s) that activate the skin, and a one-line description.
//! The two halves are merged into a single sorted list and emitted to
//! the agent-facing surface.
//!
//! # Where the reskin list came from
//!
//! The native elements styled by `packages/sail/runtime/src/styles/index.ts`
//! — every `:where(tag)` selector plus attribute-keyed skins
//! (`<a variant>`, `<span dot>`, `<span skeleton>`, `<thead sticky>`, etc.).
//! They are L1 because a bare `<button>` / `<select>` / `<table>` in a SAIL
//! document IS the design system's surface — same tokens, same tone
//! channel, zero bundle cost until used. The reskin entries make that
//! surface enumerable.

use serde::Serialize;
use std::sync::LazyLock;

/// One L1 element. `kind` is the source of truth: a `StdlibElement` row
/// means the .sail compiler registered the element; a `NativeReskin` row
/// means the runtime styles a native HTML tag. `tag` is the bare author
/// name (the same string the agent types between `<` and `>`).
#[derive(Debug, Clone, Serialize)]
pub struct L1Entry {
    /// The element's tag name as it appears in markup: `card`, `button`,
    /// `select`. Lowercase, no leading `<`.
    pub tag: String,
    /// Where this row came from. Determines what "consistency" means for
    /// the entry: a StdlibElement's `tag` must be in the stdlib registry
    /// AND its `.sail` must compile; a NativeReskin's `tag` (or attribute
    /// trigger) must appear as a selector in `LIGHT_CONTRACTS_CSS`.
    pub kind: L1Kind,
    /// One-line description, surfaced in the agent skill and the
    /// duplicate report. For stdlib rows this is pulled from the
    /// `.sail` source's leading docblock; for reskins it is authored here.
    pub description: String,
    /// Typed props (stdlib) or key attributes (reskin). The shape is
    /// `{ name, type, default? }` so the agent can read a prop table,
    /// not a paragraph.
    pub attributes: Vec<L1Attribute>,
}

/// An L1 element's prop or attribute. For stdlib elements this is the
/// declared `props:` block; for native reskins it is the attribute(s) the
/// skin keys on (with the legal values the CSS contract documents).
#[derive(Debug, Clone, Serialize)]
pub struct L1Attribute {
    pub name: String,
    /// `string` | `bool` | `number` | `"default"|"primary"|...` for a
    /// union (values quoted and joined with `|`). Matches the
    /// `PropKind` shape the stdlib parser produces.
    #[serde(rename = "type")]
    pub ty: String,
    /// The default value as a string, when the source declares one. Absent
    /// for `string` defaults of `''` and for reskin attributes (their
    /// default is the native UA default).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    /// For union types, the closed set of legal values. Empty for
    /// `string` / `bool` / `number` and for reskin attributes whose values
    /// are the HTML platform's.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum L1Kind {
    /// A compiled `.sail` element in `crates/sail/compiler/stdlib/`. The
    /// runtime tag is the same as the `tag` (spliced) or a native tag
    /// when `native` is true.
    StdlibElement,
    /// A native HTML tag whose skin lives in
    /// `packages/sail/runtime/src/styles/index.ts`. Use sites stay as
    /// `<tag>`, no compile-time rewriting.
    NativeReskin,
}

// ── stdlib rows: derived from the .sail sources ──────────────────────────

/// Memoized stdlib rows. Each row is built from the compiled
/// `ElementOutput` (props, native flag, root tag) plus the source's
/// leading docblock (one-line description). The source string is the
/// pre-compile `.sail` text — we read it via the same `SOURCES` table
/// `stdlib::sources()` exposes, so a registry entry is impossible to
/// delete without also changing this.
static STDLIB_ROWS: LazyLock<Vec<L1Entry>> = LazyLock::new(|| {
    let mut out: Vec<L1Entry> = crate::stdlib::elements()
        .iter()
        .map(|el| {
            let description = el
                .out
                .manifest
                .get("description")
                .and_then(|v| v.as_str())
                .map(str::to_string)
                .unwrap_or_else(|| {
                    // Fall back to the source's leading docblock — the
                    // `<!--  <name> — one-line description -->` first
                    // non-empty line. The .sail sources all carry one;
                    // an empty fallback means the description was
                    // forgotten (also caught by the duplicate-report
                    // quality bar).
                    crate::stdlib::sources()
                        .iter()
                        .find(|(tag, _)| *tag == el.tag)
                        .map(|(_, src)| first_line_description(src).unwrap_or_default())
                        .unwrap_or_default()
                });
            let attributes: Vec<L1Attribute> = el
                .out
                .manifest
                .get("attributes")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .map(|a| L1Attribute {
                            name: a.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            ty: a
                                .get("values")
                                .and_then(|v| v.as_array())
                                .map(|vs| {
                                    vs.iter()
                                        .filter_map(|s| s.as_str())
                                        .map(str::to_string)
                                        .collect::<Vec<_>>()
                                })
                                .map(|vs| {
                                    if vs.is_empty() {
                                        "string".to_string()
                                    } else {
                                        vs.join(" | ")
                                    }
                                })
                                .unwrap_or_else(|| {
                                    if a.get("default")
                                        .and_then(|v| v.as_str())
                                        .map(|s| s == "true" || s == "false")
                                        .unwrap_or(false)
                                    {
                                        "bool".to_string()
                                    } else {
                                        "string".to_string()
                                    }
                                }),
                            default: a
                                .get("default")
                                .and_then(|v| v.as_str())
                                .filter(|s| !s.is_empty())
                                .map(str::to_string),
                            values: a
                                .get("values")
                                .and_then(|v| v.as_array())
                                .map(|vs| {
                                    vs.iter()
                                        .filter_map(|s| s.as_str())
                                        .map(str::to_string)
                                        .collect()
                                })
                                .unwrap_or_default(),
                        })
                        .collect()
                })
                .unwrap_or_default();
            L1Entry {
                tag: el.tag.to_string(),
                kind: L1Kind::StdlibElement,
                description,
                attributes,
            }
        })
        .collect();
    out.sort_by(|a, b| a.tag.cmp(&b.tag));
    out
});

/// The first non-blank line of the source's leading `<!--  … -->` block,
/// stripped of the leading `<tag> — ` prefix. Empty string if the source
/// has no docblock.
fn first_line_description(source: &str) -> Option<String> {
    let mut in_block = false;
    for line in source.lines() {
        let trimmed = line.trim();
        if !in_block {
            if trimmed.starts_with("<!--") {
                in_block = true;
                if trimmed.ends_with("-->") {
                    return None;
                }
            }
            continue;
        }
        if trimmed.ends_with("-->") {
            return None;
        }
        if trimmed.is_empty() {
            continue;
        }
        // Strip the `  <name> — ` lead. The first em-dash IS the
        // heading separator; everything else (including later em-dashes
        // inside the sentence) is the description.
        let after_dash = trimmed
            .find('—')
            .map(|i| trimmed[i + '—'.len_utf8()..].trim_start().to_string())
            .unwrap_or_else(|| trimmed.to_string());
        return Some(after_dash);
    }
    None
}

// ── native reskin rows: declared here, verified against the runtime ────

/// Static list of every native HTML tag the runtime styles. Each row is
/// a CONTRACT: the reskin MUST exist in `LIGHT_CONTRACTS_CSS` and MUST
/// key on the listed attribute(s). The consistency test enforces the
/// inverse too — any native tag the runtime styles but this table does
/// not list is a failure.

// Module-level const arrays — slice literals like `&["xs", "sm"][..]` aren't
// stable in const, so every value set is a const array and the table below
const LISTBOX: [&str; 1] = ["listbox"];

// holds references to them by name.
const EMPTY: [&str; 0] = [];
const VARIANT_BTN: [&str; 6] = ["", "default", "solid", "soft", "outline", "ghost"];
const SIZE_XS_SM_MD_LG: [&str; 4] = ["xs", "sm", "md", "lg"];
const SIZE_SM_MD_LG: [&str; 3] = ["sm", "md", "lg"];
const SIZE_SM_MD_LG_FULL: [&str; 4] = ["sm", "md", "lg", "full"];
const TONE_8: [&str; 8] = [
    "default", "primary", "info", "success", "warning", "destructive", "accent", "muted",
];
const TONE_7_NO_MUTED: [&str; 7] = [
    "default", "primary", "info", "success", "warning", "destructive", "accent",
];
const TONE_3_DESTRUCTIVE: [&str; 3] = ["default", "primary", "destructive"];
const BACKDROP_2: [&str; 2] = ["blur", "solid"];
const SIDE_4: [&str; 4] = ["start", "end", "top", "bottom"];
const INPUT_TYPES: [&str; 18] = [
    "text", "email", "password", "number", "search", "tel", "url", "date", "time",
    "datetime-local", "month", "week", "file", "color", "checkbox", "radio", "range",
    "submit",
];
const SHAPE_4: [&str; 4] = ["block", "circle", "text", "line"];
const ORIENTATION_2: [&str; 2] = ["horizontal", "vertical"];
const VARIANT_2: [&str; 2] = ["solid", "dashed"];
const SCOPE_4: [&str; 4] = ["row", "col", "rowgroup", "colgroup"];
const METER_SIZES: [&str; 3] = ["sm", "md", "lg"];

/// Static list of every native HTML tag the runtime styles. Each row is
/// a CONTRACT: the reskin MUST exist in `LIGHT_CONTRACTS_CSS` and MUST
/// key on the listed attribute(s). The consistency test enforces the
/// inverse too — any native tag the runtime styles but this table does
/// not list is a failure.
const RESKINS: &[ReskinRow] = &[
    ReskinRow {
        tag: "a",
        selectors: &[":where(a[variant])"],
        attributes: &[("variant", &VARIANT_BTN)],
        description: "Anchor — when `variant` is set, becomes a button link. Tone rides the global [tone] channel; hover shifts the fill toward the tone.",
    },
    ReskinRow {
        tag: "button",
        selectors: &[":where(button, a[variant])"],
        attributes: &[
            ("variant", &VARIANT_BTN),
            ("size", &SIZE_XS_SM_MD_LG),
            ("tone", &TONE_8),
            ("disabled", &EMPTY),
        ],
        description: "Button — bare `<button>` IS the design system's button. tone/size ride the framework channels; variant chooses default/solid/soft/outline/ghost.",
    },
    ReskinRow {
        tag: "details",
        selectors: &[":where(details)", ":where(summary)"],
        attributes: &[("open", &EMPTY)],
        description: "Disclosure / accordion — bare `<details>`/`<summary>` is the design-system disclosure. Stacked adjacents merge into an accordion. Open/close animates natively.",
    },
    ReskinRow {
        tag: "dialog",
        selectors: &[":where(dialog)"],
        attributes: &[
            ("open", &EMPTY),
            ("size", &SIZE_SM_MD_LG_FULL),
            ("backdrop", &BACKDROP_2),
            ("side", &SIDE_4),
        ],
        description: "Modal / sheet — bare `<dialog>` IS the design-system modal. `side=` docks it as a sheet; showModal() gives backdrop, ESC, focus-trap, focus-return for free.",
    },
    ReskinRow {
        tag: "fieldset",
        selectors: &[":where(fieldset)", ":where(legend)"],
        attributes: &[("disabled", &EMPTY)],
        description: "Form group — bare `<fieldset>`/`<legend>` is a bordered section with the legend riding the top edge. `disabled` dims the whole group.",
    },
    ReskinRow {
        tag: "hr",
        selectors: &[":where(hr)"],
        attributes: &[
            ("orientation", &ORIENTATION_2),
            ("variant", &VARIANT_2),
            ("tone", &TONE_3_DESTRUCTIVE),
        ],
        description: "Separator — horizontal by default; `orientation=vertical` and `variant=dashed` are attribute-keyed. tone tints the rule.",
    },
    ReskinRow {
        tag: "html",
        selectors: &[":where(html):has(dialog[open])"],
        attributes: &EMPTY_ATTRS,
        description: "Document root — the `html:has(dialog[open])` scroll-lock rule freezes the page while a native modal is open. No JS scroll-lock required.",
    },
    ReskinRow {
        tag: "input",
        selectors: &[":where(input, textarea, select)"],
        attributes: &[
            ("type", &INPUT_TYPES),
            ("tone", &TONE_7_NO_MUTED),
            ("size", &SIZE_XS_SM_MD_LG),
            ("invalid", &EMPTY),
            ("disabled", &EMPTY),
            ("placeholder", &EMPTY),
        ],
        description: "Text entry — bare `<input>` (text/email/etc.), `<textarea>`, `<select>` are the design-system field. tone/size/invalid mirror the button contract.",
    },
    ReskinRow {
        tag: "input[type=checkbox]",
        selectors: &[":where(input[type='checkbox'])"],
        attributes: &[("checked", &EMPTY), ("indeterminate", &EMPTY), ("invalid", &EMPTY)],
        description: "Checkbox — native appearance:none checkbox painted as a tone-filled square with a baked-white check. Indeterminate = tri-state.",
    },
    ReskinRow {
        tag: "input[type=radio]",
        selectors: &[":where(input[type='radio'])"],
        attributes: &[("checked", &EMPTY), ("invalid", &EMPTY)],
        description: "Radio — native appearance:none radio painted as a tone-filled circle with a background-punched dot.",
    },
    ReskinRow {
        tag: "input[type=range]",
        selectors: &[":where(input[type='range'])"],
        attributes: &[
            ("value", &EMPTY),
            ("min", &EMPTY),
            ("max", &EMPTY),
            ("step", &EMPTY),
            ("size", &SIZE_SM_MD_LG),
        ],
        description: "Slider — native appearance:none range, rebuilt as a tone-tinted rail with a white thumb ringed in the tone. size drives track/thumb thickness.",
    },
    ReskinRow {
        tag: "input[type=file]",
        selectors: &[":where(input[type='file'])"],
        attributes: &[("accept", &EMPTY), ("multiple", &EMPTY), ("disabled", &EMPTY)],
        description: "File picker — keeps the field box, restyles the platform's file picker as a small secondary button. Use with `<div dropzone use:dropzone>` for drag-and-drop.",
    },
    ReskinRow {
        tag: "input[type=color]",
        selectors: &[":where(input[type='color'])"],
        attributes: &[("value", &EMPTY), ("disabled", &EMPTY)],
        description: "Color swatch — rounded swatch button (not a text field). The platform's value is the selected color; the skin themes the swatch frame.",
    },
    ReskinRow {
        tag: "legend",
        selectors: &[":where(legend)"],
        attributes: &EMPTY_ATTRS,
        description: "Fieldset caption — semibold text-sm riding the top edge of a `<fieldset>`. Inherits the fieldset's own paddings.",
    },
    ReskinRow {
        tag: "meter",
        selectors: &[":where(meter)"],
        attributes: &[
            ("value", &EMPTY),
            ("min", &EMPTY),
            ("low", &EMPTY),
            ("high", &EMPTY),
            ("max", &EMPTY),
            ("optimum", &EMPTY),
            ("size", &METER_SIZES),
        ],
        description: "Gauge — value-in-range; fill reflects WHICH band the value falls in (success/warning/destructive), not just how full. Distinct from `<progress>`.",
    },
    ReskinRow {
        tag: "optgroup",
        selectors: &[":where(optgroup)"],
        attributes: &[("label", &EMPTY), ("disabled", &EMPTY)],
        description: "Option list caption — semibold muted label inside a native `<select>`'s popup. Groups options without changing the value model.",
    },
    ReskinRow {
        tag: "output",
        selectors: &[":where(output)"],
        attributes: &[("for", &EMPTY), ("form", &EMPTY)],
        description: "Form result — tabular figures so recomputes don't jitter. The native element for displaying a derived input value.",
    },
    ReskinRow {
        tag: "progress",
        selectors: &[":where(progress)"],
        attributes: &[("value", &EMPTY), ("max", &EMPTY), ("size", &METER_SIZES)],
        description: "Progress bar — determinate when `value` is set, indeterminate (sliding sheen) when not. tone tints the fill.",
    },
    ReskinRow {
        tag: "select",
        selectors: &[":where(select)"],
        attributes: &[
            ("size", &SIZE_XS_SM_MD_LG),
            ("tone", &TONE_7_NO_MUTED),
            ("invalid", &EMPTY),
            ("disabled", &EMPTY),
            ("multiple", &EMPTY),
        ],
        description: "Picker — native `<select>` with the OS arrow replaced by a theme-aware chevron. option/optgroup reset so the popup is themed too.",
    },
    ReskinRow {
        tag: "span",
        selectors: &[":where(span[dot])", ":where(span[skeleton])"],
        attributes: &[
            ("dot", &EMPTY),
            ("skeleton", &EMPTY),
            ("size", &SIZE_SM_MD_LG),
            ("shape", &SHAPE_4),
            ("pulse", &EMPTY),
        ],
        description: "Status dot (`<span dot>`) and skeleton placeholder (`<span skeleton>`). tone rides the global channel; pulse animates the dot.",
    },
    ReskinRow {
        tag: "summary",
        selectors: &[":where(summary)"],
        attributes: &EMPTY_ATTRS,
        description: "Disclosure trigger — semibold trigger header inside `<details>`. Chevron rotates 180° and brightens to --foreground on open.",
    },
    ReskinRow {
        tag: "table",
        selectors: &[
            ":where(table)",
            ":where(th)",
            ":where(td)",
            ":where(tbody)",
            ":where(thead[sticky])",
        ],
        attributes: &[
            ("dense", &EMPTY),
            ("striped", &EMPTY),
            ("hoverable", &EMPTY),
            ("bordered", &EMPTY),
        ],
        description: "Data table — STRUCTURAL primitive, zero JS. Variants ride the table itself (`[dense] [striped] [hoverable] [bordered]`); `<thead sticky>` pins the header; `[numeric]` columns right-align on tabular figures.",
    },
    ReskinRow {
        tag: "ul",
        selectors: &[":where(ul[role='listbox'])"],
        attributes: &[("role", &LISTBOX)],
        description: "Listbox — a `<ul role='listbox'>` becomes a widget surface (zero list markers, no default padding). A plain `<ul>` keeps the document-list reset; the role distinguishes widget use.",
    },
    ReskinRow {
        tag: "tbody",
        selectors: &[":where(tbody)"],
        attributes: &EMPTY_ATTRS,
        description: "Table body — row-group container inside `<table>`. CSS resets the trailing row's bottom border so a table reads as a single sheet.",
    },
    ReskinRow {
        tag: "td",
        selectors: &[":where(td)"],
        attributes: &[("numeric", &EMPTY)],
        description: "Table cell — body row cell. `[numeric]` right-aligns on tabular figures; bordered tables add a 1px cell border.",
    },
    ReskinRow {
        tag: "textarea",
        selectors: &[":where(textarea)"],
        attributes: &[
            ("rows", &EMPTY),
            ("cols", &EMPTY),
            ("size", &SIZE_XS_SM_MD_LG),
            ("tone", &TONE_7_NO_MUTED),
            ("invalid", &EMPTY),
            ("disabled", &EMPTY),
            ("placeholder", &EMPTY),
        ],
        description: "Multi-line text — shares the field surface with `<input>`. `rows=`/min-height scales the visible line count; vertical-only resize.",
    },
    ReskinRow {
        tag: "th",
        selectors: &[":where(th)"],
        attributes: &[("numeric", &EMPTY), ("scope", &SCOPE_4)],
        description: "Table header cell — de-emphasized (muted-foreground) header; the body carries the data. `[numeric]` right-aligns on tabular figures.",
    },
    ReskinRow {
        tag: "thead",
        selectors: &[":where(thead[sticky])"],
        attributes: &[("sticky", &EMPTY)],
        description: "Table head — `[sticky]` pins the header to the scroll container top with a background to defeat bleed-through.",
    },
];

const EMPTY_ATTRS: [(&'static str, &'static [&'static str]); 0] = [];

/// The row shape we declare statically. The public `L1Entry` uses
/// `Vec<L1Attribute>` (the public contract); this struct is the
/// authoring-only shape (slice literals are clearer in source).
struct ReskinRow {
    tag: &'static str,
    /// Selectors the runtime MUST contain for this reskin. At least one
    /// must match a substring of `LIGHT_CONTRACTS_CSS` (the consistency
    /// test enforces this).
    selectors: &'static [&'static str],
    /// The attributes the skin keys on. `(name, &VALUES)` where VALUES is
    /// a const array (e.g. `&VARIANT_BTN`). An empty values array means
    /// "boolean attribute" / "presence-based".
    attributes: &'static [(&'static str, &'static [&'static str])],
    description: &'static str,
}

fn reskin_entries() -> Vec<L1Entry> {
    RESKINS
        .iter()
        .map(|r| L1Entry {
            tag: r.tag.to_string(),
            kind: L1Kind::NativeReskin,
            description: r.description.to_string(),
            attributes: r
                .attributes
                .iter()
                .map(|(name, values)| L1Attribute {
                    name: (*name).to_string(),
                    ty: if values.is_empty() {
                        "boolean".to_string()
                    } else {
                        values.iter().map(|s| format!("\"{s}\"")).collect::<Vec<_>>().join(" | ")
                    },
                    default: None,
                    values: values.iter().map(|s| s.to_string()).collect(),
                })
                .collect(),
        })
        .collect()
}

// ── the public surface ──────────────────────────────────────────────────

/// Every L1 element, sorted by tag, deduplicated (a tag can't be both
/// stdlib and reskin; a duplicate here is a failure, not a merge — the
/// consistency test names it).
pub fn l1_entries() -> Vec<L1Entry> {
    let mut all: Vec<L1Entry> = STDLIB_ROWS.iter().cloned().collect();
    all.extend(reskin_entries());
    all.sort_by(|a, b| a.tag.cmp(&b.tag));
    all
}

/// Lookup a tag's kind from the unified surface. `None` if the tag is
/// not in the manifest.
pub fn l1_kind(tag: &str) -> Option<L1Kind> {
    l1_entries().into_iter().find(|e| e.tag == tag).map(|e| e.kind)
}

/// Count the L1 surface — the number that answers the question "what
/// elements does SAIL give me?".
pub fn l1_count() -> usize {
    l1_entries().len()
}

/// Split the surface by kind.
pub fn l1_by_kind() -> (Vec<L1Entry>, Vec<L1Entry>) {
    let all = l1_entries();
    let stdlib: Vec<L1Entry> = all
        .iter()
        .filter(|e| e.kind == L1Kind::StdlibElement)
        .cloned()
        .collect();
    let reskins: Vec<L1Entry> = all
        .iter()
        .filter(|e| e.kind == L1Kind::NativeReskin)
        .cloned()
        .collect();
    (stdlib, reskins)
}

/// The CSS selectors the runtime MUST contain for the listed reskin
/// entries. The consistency test asserts every selector here is a
/// substring of `LIGHT_CONTRACTS_CSS`.
pub fn reskin_selectors() -> Vec<(&'static str, &'static str)> {
    RESKINS
        .iter()
        .flat_map(|r| r.selectors.iter().map(move |sel| (r.tag, *sel)))
        .collect()
}

/// The `tag` of every declared reskin — used by the inverse test to
/// assert no native tag the runtime styles is missing from the table.
pub fn reskin_tags() -> Vec<&'static str> {
    RESKINS.iter().map(|r| r.tag).collect()
}
