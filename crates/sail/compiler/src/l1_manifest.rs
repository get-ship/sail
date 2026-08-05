//! L1 surface manifest — stdlib `.sail` rows + residual native-floor rows.
//! Consistency tests: `crates/sail/compiler/tests/l1_manifest.rs`.

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
    /// trigger) must appear as a selector in `sail-base.css`.
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
                            name: a
                                .get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
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
/// has no docblock. Handles single-line (`<!-- <tag> — desc -->`) and
/// multi-line docblocks.
fn first_line_description(source: &str) -> Option<String> {
    let mut in_block = false;
    for line in source.lines() {
        let trimmed = line.trim();
        if !in_block {
            let Some(rest) = trimmed.strip_prefix("<!--") else {
                continue;
            };
            let rest = rest.trim_start();
            if let Some(inner) = rest.strip_suffix("-->") {
                let inner = inner.trim();
                if inner.is_empty() {
                    return None;
                }
                return Some(strip_tag_emdash(inner));
            }
            in_block = true;
            if !rest.is_empty() {
                return Some(strip_tag_emdash(rest));
            }
            continue;
        }
        if let Some(inner) = trimmed.strip_suffix("-->") {
            let inner = inner.trim();
            if inner.is_empty() {
                return None;
            }
            return Some(strip_tag_emdash(inner));
        }
        if trimmed.is_empty() {
            continue;
        }
        return Some(strip_tag_emdash(trimmed));
    }
    None
}

/// Strip the leading `<name> — ` heading separator; later em-dashes stay.
fn strip_tag_emdash(trimmed: &str) -> String {
    trimmed
        .find('—')
        .map(|i| trimmed[i + '—'.len_utf8()..].trim_start().to_string())
        .unwrap_or_else(|| trimmed.to_string())
}

const EMPTY_ATTRS: [(&'static str, &'static [&'static str]); 0] = [];

const RESKINS: &[ReskinRow] = &[
    ReskinRow {
        tag: "hr",
        selectors: &["hr {"],
        attributes: &EMPTY_ATTRS,
        description: "Native reskin of HTML <hr> — document rule divider. Prefer <separator> for orientation/variant/tone.",
    },
    ReskinRow {
        tag: "kbd",
        selectors: &["kbd {"],
        attributes: &EMPTY_ATTRS,
        description: "Native reskin of HTML <kbd> — mono keycap on muted chip, doubled bottom edge.",
    },
];

/// The row shape we declare statically. The public `L1Entry` uses
/// `Vec<L1Attribute>` (the public contract); this struct is the
/// authoring-only shape (slice literals are clearer in source).
struct ReskinRow {
    tag: &'static str,
    /// Selectors the document floor MUST contain for this reskin. At least
    /// one must match a substring of `sail-base.css` (the consistency test
    /// enforces this).
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
                        values
                            .iter()
                            .map(|s| format!("\"{s}\""))
                            .collect::<Vec<_>>()
                            .join(" | ")
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
    l1_entries()
        .into_iter()
        .find(|e| e.tag == tag)
        .map(|e| e.kind)
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

/// HTML void elements — tags that the parser must treat as self-closing
/// open tags with no closing tag. This is the grammar's source of truth
/// for void-tag parsing; it is emitted to the grammar generator so the
/// tree-sitter grammar cannot drift from the compiler's element surface.
pub const VOID_TAGS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source", "track",
    "wbr",
];
