//! L1 surface manifest — stdlib `.sail` rows only.
//! Consistency tests: `crates/sail/compiler/tests/l1_manifest.rs`.
//!
//! Native reskins (RESKINS table + sail-base.css tag floors for L1) are
//! retired. Every L1 element is a compiled stdlib source.

use serde::Serialize;
use std::sync::LazyLock;

/// One L1 element. `kind` is always `StdlibElement` after the reskin scrub;
/// the field remains so generated surfaces and older clients keep a stable
/// shape.
#[derive(Debug, Clone, Serialize)]
pub struct L1Entry {
    /// The element's tag name as it appears in markup: `card`, `button`,
    /// `select`. Lowercase, no leading `<`.
    pub tag: String,
    /// Where this row came from. Post-reskin-scrub, only `StdlibElement`.
    pub kind: L1Kind,
    /// One-line description, surfaced in the agent skill and the
    /// duplicate report. Pulled from the `.sail` source's leading docblock.
    pub description: String,
    /// Typed props. The shape is `{ name, type, default? }` so the agent
    /// can read a prop table, not a paragraph.
    pub attributes: Vec<L1Attribute>,
}

/// An L1 element's prop. The declared `props:` block in the stdlib source.
#[derive(Debug, Clone, Serialize)]
pub struct L1Attribute {
    pub name: String,
    /// `string` | `bool` | `number` | `"default"|"primary"|...` for a
    /// union (values quoted and joined with `|`). Matches the
    /// `PropKind` shape the stdlib parser produces.
    #[serde(rename = "type")]
    pub ty: String,
    /// The default value as a string, when the source declares one. Absent
    /// for `string` defaults of `''`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    /// For union types, the closed set of legal values. Empty for
    /// `string` / `bool` / `number`.
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
    /// Retired. No rows use it after the reskin scrub; kept so match
    /// arms in generated tools stay exhaustive without a soft deprecation.
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

// ── the public surface ──────────────────────────────────────────────────

/// Every L1 element, sorted by tag.
pub fn l1_entries() -> Vec<L1Entry> {
    STDLIB_ROWS.iter().cloned().collect()
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

/// Split the surface by kind. Reskins are always empty post-scrub; the
/// second tuple arm stays for call-site compatibility.
pub fn l1_by_kind() -> (Vec<L1Entry>, Vec<L1Entry>) {
    (l1_entries(), Vec::new())
}

/// Sail void / childless tags — parser treats these as self-closing.
/// Includes HTML5 void plus `<textarea>` (value is a prop; it does not take
/// children). HTML emit still writes `</textarea>` because the platform
/// element is not void. Grammar SSOT — do not fork this list.
pub const VOID_TAGS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source", "track",
    "wbr", "textarea",
];
