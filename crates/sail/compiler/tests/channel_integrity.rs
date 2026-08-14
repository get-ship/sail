//! Generator checks for the Tier-2 expression channels.
//!
//! Three invariants, each guarding a failure this repo has actually shipped:
//!
//!   1. NO DANGLING VAR — every `var(--x)` in emitted CSS resolves to a
//!      declared name. `sx="font-display"` lowered to `var(--display-font)`
//!      for its whole life; the token was never declared, so the atom
//!      silently painted nothing. Nothing caught it because the reference is
//!      built by string interpolation in `scales.rs`, so it is not greppable.
//!
//!   2. NO UNREAD CHANNEL — every channel declared in the expression-channel
//!      block is consumed by something. A channel nothing reads is a variable
//!      in a costume: it looks like a theme seam, themes set it, and nothing
//!      happens. This is the check that makes "generator-enforced read" real
//!      rather than a promise in a comment.
//!
//!   3. SKIN/ELEMENT PARITY — every skinned name has a registered element and
//!      every registered element has a skin. `kbd` shipped a skin with no
//!      manifest row; the existing manifest test only walked manifest→CSS, so
//!      the CSS→manifest direction was unguarded.

use std::collections::HashSet;

const TOKENS_CSS: &str = include_str!("../../../../packages/sail/tokens/src/tokens.css");
const THEMES_CSS: &str = include_str!("../../../../packages/sail/tokens/src/themes.css");
const CONTRACTS_CSS: &str = include_str!("../../../../packages/sail/tokens/src/sail-base.css");
const SKINS_CSS: &str = include_str!("../../../../packages/sail/tokens/src/element-skins.css");
const SCALES_RS: &str = include_str!("../../spinnaker/src/scales.rs");
const PLAINS_RS: &str = include_str!("../../spinnaker/src/plains.rs");
const CONCERNS_RS: &str = include_str!("../../spinnaker/src/concerns.rs");
const SETTERS_RS: &str = include_str!("../../spinnaker/src/setters.rs");
const LOWER_RS: &str = include_str!("../../spinnaker-css/src/lower.rs");
const CONCERN_CSS_RS: &str = include_str!("../../spinnaker-css/src/concern_css.rs");
const PLAINS_CSS_RS: &str = include_str!("../../spinnaker-css/src/plains_css.rs");

/// Every `--name:` declaration in `css`, without the `--` prefix.
fn declarations(css: &str) -> HashSet<String> {
    let mut out = HashSet::new();
    let b = css.as_bytes();
    let mut i = 0;
    while i + 2 < b.len() {
        if b[i] == b'-' && b[i + 1] == b'-' {
            let start = i + 2;
            let mut j = start;
            while j < b.len() && (b[j].is_ascii_alphanumeric() || b[j] == b'-' || b[j] == b'_') {
                j += 1;
            }
            // A declaration is `--name:` — the colon must follow the name
            // directly (a `var(--name)` reference has `)` there instead).
            if j > start && j < b.len() && b[j] == b':' {
                out.insert(css[start..j].to_string());
            }
            i = j.max(i + 1);
        } else {
            i += 1;
        }
    }
    out
}

/// Every `var(--name)` reference in `css`, without the `--` prefix.
fn references(css: &str) -> HashSet<String> {
    refs_with_fallback_flag(css)
        .into_iter()
        .map(|(n, _)| n)
        .collect()
}

/// Every `var(--name…)` reference paired with whether it supplied a fallback.
///
/// The distinction is the whole point of the dangling-var check.
/// `var(--knob, 12rem)` is the sanctioned COMPONENT-KNOB pattern: the name is
/// deliberately undeclared so an author can set it, and the fallback is the
/// designed default. `var(--knob)` with no fallback and no declaration is a
/// dead reference — it computes to nothing and the property silently drops.
/// Only the second form is a bug.
fn refs_with_fallback_flag(css: &str) -> Vec<(String, bool)> {
    let mut out = Vec::new();
    for (idx, _) in css.match_indices("var(--") {
        let start = idx + "var(--".len();
        let rest = &css[start..];
        let end = rest
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'))
            .unwrap_or(rest.len());
        if end == 0 {
            continue;
        }
        let has_fallback = rest[end..].trim_start().starts_with(',');
        out.push((rest[..end].to_string(), has_fallback));
    }
    out
}

/// The channel names declared in the expression-channel block of tokens.css.
/// Parsed from the block rather than hand-listed so adding a channel to the
/// CSS automatically puts it under check 2 — the same no-mirror-file
/// discipline the token registry itself follows.
fn expression_channels() -> Vec<String> {
    let start = TOKENS_CSS
        .find("Expression channels (Tier 2)")
        .expect("the expression-channel block must be findable by its banner");
    // The block runs to the end of the :root rule it lives in.
    let rest = &TOKENS_CSS[start..];
    let end = rest.find("\n}").unwrap_or(rest.len());
    let mut names: Vec<String> = declarations(&rest[..end]).into_iter().collect();
    names.sort();
    names
}

#[test]
fn no_emitted_var_is_undeclared() {
    // Names that are legitimately set at runtime or by an ancestor rather than
    // declared globally. Mirrors `sail_lint::tokens::CONVENTION_TOKENS` in
    // spirit; kept local so this test does not depend on the lint crate.
    let declared: HashSet<String> = declarations(TOKENS_CSS)
        .union(&declarations(THEMES_CSS))
        .cloned()
        .collect::<HashSet<_>>()
        // Locally-declared component tokens count: a skin may declare and read
        // its own namespace within the same emitted file.
        .union(&declarations(CONTRACTS_CSS))
        .cloned()
        .collect::<HashSet<_>>()
        .union(&declarations(SKINS_CSS))
        .cloned()
        .collect();

    // Names a runtime `use:<action>` writes onto the element. They are
    // legitimately absent from tokens.css (a measurement, not a design
    // decision) and are referenced without a fallback on purpose.
    // Single source: sail_lint::tokens::CONVENTION_TOKENS + tip-describe geometry.
    let runtime_written: std::collections::HashSet<&str> = sail_lint::tokens::CONVENTION_TOKENS
        .iter()
        .copied()
        .chain([
            // use:tipDescribe / tooltip geometry (action-owned, not in tokens.css)
            "tt-top",
            "tt-bottom",
            "tt-left",
            "tt-right",
            "tt-tx",
            "tt-arrow-top",
            "tt-arrow-bottom",
            "tt-arrow-left",
            "tt-arrow-right",
            "tt-arrow-ml",
            "tt-arrow-mt",
            // use:anchor geometry (paired with --anchor-width)
            "anchor-left",
            "anchor-top",
            "anchor-width",
            // use:anchor / popover offset measurement
            "offset",
            // Host-reflected Tone bind: bg(tone) / fg(tone.foreground)
            "tone",
            "tone-foreground",
        ])
        .collect();

    let mut dangling: Vec<String> = Vec::new();
    for (label, css) in [
        ("sail-base.css", CONTRACTS_CSS),
        ("element-skins.css", SKINS_CSS),
    ] {
        for (name, has_fallback) in refs_with_fallback_flag(css) {
            if has_fallback || declared.contains(&name) || runtime_written.contains(name.as_str()) {
                continue;
            }
            dangling.push(format!("{label}: var(--{name})"));
        }
    }
    dangling.sort();
    dangling.dedup();

    assert!(
        dangling.is_empty(),
        "emitted CSS references custom properties that are never declared.\n\
         Each of these paints nothing at runtime and fails silently — declare \
         it in tokens.css or set it from a runtime action:\n  {}",
        dangling.join("\n  ")
    );
}

#[test]
fn every_expression_channel_has_a_reader() {
    let channels = expression_channels();
    assert!(
        channels.len() >= 13,
        "expected the landed channel set to be parsed, got {channels:?}"
    );

    // A channel is "read" if emitted CSS references it, OR plains/concerns/scales
    // interpolate it. The last case is why `--display-font` went unnoticed:
    // its only reader is `format!("var(--{token}-font)")`.
    let static_reads: HashSet<String> = references(CONTRACTS_CSS)
        .union(&references(SKINS_CSS))
        .cloned()
        .collect::<HashSet<_>>()
        .union(&references(TOKENS_CSS))
        .cloned()
        .collect();
    let emit_reads: HashSet<String> = references(PLAINS_RS)
        .union(&references(CONCERNS_RS))
        .cloned()
        .collect::<HashSet<_>>()
        .union(&references(SETTERS_RS))
        .cloned()
        .collect::<HashSet<_>>()
        .union(&references(LOWER_RS))
        .cloned()
        .collect::<HashSet<_>>()
        .union(&references(CONCERN_CSS_RS))
        .cloned()
        .collect::<HashSet<_>>()
        .union(&references(PLAINS_CSS_RS))
        .cloned()
        .collect();

    let mut unread: Vec<&String> = Vec::new();
    for ch in &channels {
        let read_statically = static_reads.contains(ch);
        let read_by_sx = emit_reads.contains(ch);
        // `--display-font` / `--heading-font` / … are produced by the `font`
        // scale's interpolation. Recognise the pattern rather than the name.
        let read_by_scale = ch
            .strip_suffix("-font")
            .is_some_and(|_| SCALES_RS.contains(r#""font" => format!("var(--{token}-font)")"#));
        if !(read_statically || read_by_sx || read_by_scale) {
            unread.push(ch);
        }
    }

    assert!(
        unread.is_empty(),
        "these expression channels are declared but nothing reads them:\n  {}\n\n\
         A channel with no reader is a variable in a costume — a theme can set \
         it and nothing happens. Either wire a reader into a generator \
         (stdlib skin, sail-base.css, plains/concerns, or scales) or drop the \
         declaration.",
        unread
            .iter()
            .map(|c| format!("--{c}"))
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}

/// Element tags that emitted CSS styles but that are deliberately NOT reskin
/// manifest rows: prose defaults, not control surfaces. The manifest documents
/// the L1 *component* surface (what an author addresses with attributes); a
/// bare `<p>` taking the type scale is typography, not a reskin with a channel.
///
/// This list exists so the gap is DECLARED. Before it, the parity test only
/// walked manifest→CSS, so a skin with no row (kbd) was invisible in both
/// directions. Anything new that gets a skin now has to land here or in the
/// manifest — deliberately, in a diff someone reviews.
const PROSE_DEFAULTS: &[&str] = &[
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "p",
    "small",
    "strong",
    "code",
    "pre",
    "mark",
    "abbr",
    "sub",
    "sup",
    "blockquote",
    "img",
    "video",
    "canvas",
    "svg",
    "figure",
    "figcaption",
    "ol",
    "ul",
    "li",
    "dl",
    "dt",
    "dd",
    "label",
    "html",
    "body",
    "form",
    // Embedded media: `iframe { max-width: 100%; border: 0 }` is an overflow
    // guard, not a surface an author styles through attributes.
    "iframe",
];

#[test]
fn every_skinned_tag_has_a_manifest_row_or_a_declared_exemption() {
    const MANIFEST_RS: &str = include_str!("../src/l1_manifest.rs");

    // Tags carrying a manifest row.
    let registered: HashSet<&str> = MANIFEST_RS
        .match_indices("tag: \"")
        .filter_map(|(i, _)| {
            let rest = &MANIFEST_RS[i + "tag: \"".len()..];
            rest.find('"').map(|e| &rest[..e])
        })
        .collect();

    // Tags styled by a top-level element selector in the emitted contracts.
    // Only plain leading tag selectors count (`kbd { … }`, `h1,h2 { … }`) —
    // this is about element skins, not descendant or class rules.
    let mut skinned: HashSet<&str> = HashSet::new();
    for line in CONTRACTS_CSS.lines() {
        let line = line.trim();
        let Some(brace) = line.find('{') else {
            continue;
        };
        let selector = line[..brace].trim();
        if selector.is_empty() || selector.starts_with('@') || selector.starts_with(':') {
            continue;
        }
        for part in selector.split(',') {
            let part = part.trim();
            if part
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
                && !part.is_empty()
            {
                skinned.insert(part);
            }
        }
    }

    let mut unregistered: Vec<&str> = skinned
        .iter()
        .filter(|t| !registered.contains(*t) && !PROSE_DEFAULTS.contains(*t))
        .copied()
        .collect();
    unregistered.sort();

    assert!(
        unregistered.is_empty(),
        "these tags ship a skin in contracts.css but have no ReskinRow in \
         l1_manifest.rs:\n  {}\n\n\
         The compiler answers \"unknown element\" for them and they are absent \
         from the generated reference — the skin exists but the language does \
         not admit it. Add a manifest row, or add the tag to PROSE_DEFAULTS if \
         it is a typography default rather than an addressable surface.",
        unregistered.join("\n  ")
    );
}

#[test]
fn no_expression_channel_default_is_a_hard_literal_where_it_must_compose() {
    // The frame/press/glow channels are composed into comma-separated
    // box-shadow and background lists. `none` is not a legal list member, so a
    // `none` default silently drops the WHOLE declaration — the failure looks
    // like "the shadow disappeared", nowhere near the token.
    for name in ["control-frame", "press-shadow"] {
        let decl = format!("--{name}:");
        let line = TOKENS_CSS
            .lines()
            .find(|l| l.trim_start().starts_with(&decl))
            .unwrap_or_else(|| panic!("--{name} must be declared in tokens.css"));
        assert!(
            !line.contains("none"),
            "--{name} is composed into a comma-separated box-shadow list, so its \
             no-op default must be a transparent shadow (`0 0 0 0 transparent`), \
             not `none` — `box-shadow: none, var(--shadow-xs)` is invalid CSS and \
             drops the entire declaration.\n  got: {}",
            line.trim()
        );
    }
}
