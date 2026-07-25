//! `sail-sx` — the inline style surface.
//!
//! **STATUS: the string form is DELETED as product.** Any static `sx="…"`
//! in the legacy space- or colon-separated utility form is a hard error
//! directing authors to the new Style chain syntax
//! (`style.bg(primary).padX(2)` in `sx={…}` or `sx={[…]}`). Dynamic
//! `sx={…}` is the product surface and does not error here. The tables
//! and helpers below remain as internal machinery (Style-IR lowering,
//! surface documentation, layout refusals) — not an authoring surface.
//!
//! `sx="p-4 bg-surface-2 rounded-lg text-sm hover:bg-surface-3"` — Tailwind's
//! NAMES (the prior every agent already has, and terse: an agent pays for every
//! character) with OUR values.
//!
//! THE RULE: every value is a design token or a plain CSS keyword. Never a raw
//! number, never a colour, never a unit. There is no bracket escape (cf.
//! Tailwind's `p-[13px]`).
//!
//! The vocabulary is a TABLE (`sx-vocabulary.json`, read by `crate::sx_vocab`),
//! not code. A scale-valued utility spends a token and nothing else; a
//! keyword-valued one takes its CSS keywords, all of them, with no allowlist —
//! because there is no design decision inside `object-fit: cover`, nothing to
//! tokenize, and no way for two authors to diverge. The allowlist was the
//! mistake; the value discipline is the point.
//!
//! SCOPE — sx styles ONE box. How its children sit is a layout element
//! (`<row>`, `<column>`, `<grid>`), which carries the semantics, the a11y and
//! the overflow handling a hand-rolled flex box loses. Every layout atom is
//! REFUSED BY NAME, and the refusal names the element to use instead: the
//! mission is never "find another way".
//!
//! sx is compile-time and static: it lowers to atomic classes plus injected CSS.
//! A dynamic `sx={…}` cannot be lowered and used to silently vanish
//! (`class=""`), so it is a hard error — conditional styling goes through a
//! primitive prop (`tone={…}`) or a state prefix keyed on ARIA.

use std::path::Path;

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sail-sx";

/// A condition prefix (`motion-reduce:`/`forced-colors:`/…) → its `@media` query.
/// A CLOSED set of the standard user-preference / environment queries — never an
/// arbitrary `@media` (that would be a raw-CSS escape). An sx pair carrying a
/// condition lowers into that media block.
/// Condition prefixes (name → media query) — table form so surface
/// documentation renders the same list the lowering consults.
pub const CONDITION_PREFIXES: &[(&str, &str)] = &[
    // Responsive, mobile-first (min-width), as Tailwind's `md:` is — the widths ARE
    // the breakpoint tokens. They are written literally because a media query cannot
    // read a custom property; `breakpoints_match_the_tokens` keeps them honest.
    ("mobile", "(min-width: 390px)"),
    ("tablet", "(min-width: 768px)"),
    ("desktop", "(min-width: 1280px)"),
    ("motion-reduce", "(prefers-reduced-motion: reduce)"),
    ("motion-safe", "(prefers-reduced-motion: no-preference)"),
    ("forced-colors", "(forced-colors: active)"),
    ("dark", "(prefers-color-scheme: dark)"),
    ("light", "(prefers-color-scheme: light)"),
];

pub fn condition_media(name: &str) -> Option<&'static str> {
    CONDITION_PREFIXES.iter().find(|(n, _)| *n == name).map(|(_, m)| *m)
}

/// The sx prefix/value forms that are grammar rather than named tables —
/// the `group` ancestor marker, the `group-<state>:` and `state-<value>:`
/// prefix families, and the `(--name)` custom-property value read. Table
/// form (form → one-line semantics) so surface documentation renders the
/// same shapes `is_prefix` / `state_data` / `Utility::is_var_ref` enforce.
pub const SX_EXTRA_FORMS: &[(&str, &str)] = &[
    (
        "group",
        "bare marker in an ancestor's sx; descendants react to that ancestor with group-<state>:",
    ),
    (
        "group-<state>:",
        "apply when the group-marked ancestor is in <state> — any state prefix, or a state-<value>",
    ),
    (
        "state-<value>:",
        "component-state prefix keyed on the reserved data-state attribute; lowers to [data-state~='<value>'], value is [a-z0-9-]+",
    ),
    (
        "(--name)",
        "read a CSS custom property as any atom's value — lowers to var(--name); typed actions write the property at runtime",
    ),
];

/// A state prefix (`hover:`/`focus:`/…) → its CSS pseudo-class. `focus` maps to
/// `:focus-visible` (the modern default — no ring on a mouse click). Shared by
/// the lint (validation) and preprocess (lowering) so they can't diverge.
/// State prefixes (name → selector) — table form for surface documentation.
/// `current` keys off aria-current (the author stamps it for assistive tech
/// anyway; the style reads the same source of truth).
pub const STATE_PREFIXES: &[(&str, &str)] = &[
    ("hover", ":hover"),
    ("focus", ":focus-visible"),
    ("focus-within", ":focus-within"),
    ("active", ":active"),
    ("disabled", ":is(:disabled,[aria-disabled='true'])"),
    ("current", "[aria-current]:not([aria-current='false'])"),
    // ARIA states — styling keys on the SAME attribute assistive tech reads,
    // so the visual is only reachable by stamping the semantics. A closed
    // set: new states are rulings, not a namespace.
    ("selected", "[aria-selected='true']"),
    ("expanded", "[aria-expanded='true']"),
    ("busy", "[aria-busy='true']"),
    ("checked", "[aria-checked='true']"),
    ("pressed", "[aria-pressed='true']"),
    ("open", "[open]"),
    ("sortable", "[aria-sort]"),
];

/// `state-<value>:` — the component-state prefix. Keys on the reserved
/// `data-state` attribute (`data-state="weekend"`), the single sanctioned
/// place a component declares a visual state that is neither an ARIA state
/// nor a CSS pseudo (out-of-month, weekend, in-range). One attribute, not a
/// namespace: `data-*` at large stays closed.
pub fn state_data(state: &str) -> Option<String> {
    let v = state.strip_prefix("state-")?;
    if v.is_empty() || !v.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-') {
        return None;
    }
    Some(format!("[data-state~='{v}']"))
}

pub fn state_pseudo(state: &str) -> Option<&'static str> {
    STATE_PREFIXES.iter().find(|(n, _)| *n == state).map(|(_, p)| *p)
}

/// The complete sx vocabulary — every name with everything it accepts. THE single
/// source for surface documentation (the agent-skill generator, the /reference
/// page and the LSP all render this), so what an agent is told is what the lint
/// enforces, by construction.
pub fn vocabulary() -> Vec<(String, Vec<String>)> {
    crate::sx_vocab::vocabulary()
}

/// The CSS declarations one atom lowers to. `None` when the atom is not
/// vocabulary — the caller has already reported it.
pub fn lower_atom(atom: &str) -> Option<Vec<(String, String)>> {
    match crate::sx_vocab::resolve(atom) {
        crate::sx_vocab::Resolved::Ok(decls) => Some(decls),
        _ => None,
    }
}

/// Split an sx pair into its prefixes and its atom. `hover:bg-surface-2` is one
/// prefix and one atom; `bg-surface-2` is none and one.
/// Kept for callers outside the IR path (tests / diagnostics helpers).
#[allow(dead_code)]
fn split_prefix(pair: &str) -> (Option<&str>, &str) {
    match pair.rsplit_once(':') {
        Some((prefix, atom)) => (Some(prefix), atom),
        None => (None, pair),
    }
}

/// Boolean presence attrs that `group-<name>:` may key (host prop reflected as
/// a bare attribute). Closed — free attrs stay closed.
pub fn group_attr_presence(name: &str) -> Option<&'static str> {
    const PRESENCE: &[&str] = &[
        "dismissible",
        "inline",
        "sticky",
        "interactive",
        "bordered",
        "fill",
        "centered",
        "required",
        "invalid",
        "disabled",
        "open",
        "pill",
    ];
    PRESENCE.iter().copied().find(|&n| n == name)
}

/// Pseudo-element / structural prefixes (TW-shaped): attach as `::before`,
/// `::after`, or `:empty` on the styled element.
pub fn pseudo_element_prefix(p: &str) -> Option<&'static str> {
    match p {
        "before" => Some("::before"),
        "after" => Some("::after"),
        "empty" => Some(":empty"),
        _ => None,
    }
}

/// Is this a legal prefix? The set is closed and every member keys on semantics
/// an assistive technology also reads — which is what makes the accessible
/// component the cheapest one to build rather than a chore.
pub fn is_prefix(p: &str) -> bool {
    state_pseudo(p).is_some()
        || condition_media(p).is_some()
        || state_data(p).is_some()
        || pseudo_element_prefix(p).is_some()
        || p.strip_prefix("group-").is_some_and(|b| {
            state_pseudo(b).is_some()
                || state_data(b).is_some()
                || group_attr_presence(b).is_some()
        })
}


pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    for hit in scan_sx(source) {
        let (val, offset) = match hit {
            // Dynamic `sx={…}` / `sx={[…]}` is the product surface — the Style
            // chain syntax lives there, and the compiler owns its validation.
            // This rule only deletes the legacy string form.
            SxHit::Dynamic(_) => continue,
            SxHit::Static(val, offset) => (val, offset),
        };
        let (line, col) = offset_to_line_col(source, offset);

        // Style algebra extension: match/compound/phase/media guards and
        // fragment refs use the Style IR. Validate through that parser so the
        // lint and the compiler share one truth.
        if crate::style::parse::uses_style_syntax(val) {
            let (_style, style_diags) = crate::style::parse::parse(val);
            for d in style_diags {
                findings.push(mk(path, line, col, d.message()));
            }
            continue;
        }

        // Everything else in a static string is the legacy utility form
        // (space- or colon-separated atoms) — deleted as product.
        findings.push(mk_legacy_deleted(path, line, col, val));
    }
    findings
}

/// Static `sx="…"` in the legacy utility form — deleted as product by the
/// Style syntax rewrite. Hard error that names the replacement.
fn mk_legacy_deleted(path: &Path, line: usize, col: usize, val: &str) -> Finding {
    Finding {
        file: path.to_path_buf(),
        line,
        col,
        rule: RULE_ID,
        severity: Severity::Error,
        message: format!(
            "sx=\"{val}\" is deleted as product — use `style.bg(primary).padX(2)` chain \
             syntax in `sx={{…}}` or `sx={{[…]}}`"
        ),
        suggest: None,
    }
}


fn mk(path: &Path, line: usize, col: usize, message: String) -> Finding {
    Finding {
        file: path.to_path_buf(),
        line,
        col,
        rule: RULE_ID,
        severity: Severity::Error,
        message,
        suggest: Some(
            "sx is surface-only: spacing (p/m…, the Tailwind-aligned space ramp \
             1..64 plus half-steps), colors (bg/fg), radius, text. \
             Layout is the primitives' job (<row>/<column>/<grid>). \
             Raw px/hex are not expressible."
                .to_string(),
        ),
    }
}

/// One `sx` attribute occurrence: a static literal (value + offset) to validate,
/// or a dynamic `sx={…}` binding (offset of the `sx`; allowed — the new
/// Style chain syntax lives there).
enum SxHit<'a> {
    Static(&'a str, usize),
    Dynamic(usize),
}

/// Find every `sx=` attribute — static `sx="…"` or dynamic `sx={…}`. Skips HTML
/// comments.
fn scan_sx(source: &str) -> Vec<SxHit<'_>> {
    let bytes = source.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if source[i..].starts_with("<!--") {
            match source[i..].find("-->") {
                Some(end) => {
                    i += end + 3;
                    continue;
                }
                None => break,
            }
        }
        if i > 0 && bytes[i - 1].is_ascii_whitespace() && source[i..].starts_with("sx") {
            if let Some(rel) = sx_value_delim(&bytes[i + 2..]) {
                let dpos = i + 2 + rel;
                if bytes[dpos] == b'"' {
                    let vstart = dpos + 1;
                    if let Some(q) = source[vstart..].find('"') {
                        out.push(SxHit::Static(&source[vstart..vstart + q], vstart));
                        i = vstart + q + 1;
                        continue;
                    }
                } else {
                    // dynamic `sx={…}` — allowed (the product surface); skip past the brace.
                    out.push(SxHit::Dynamic(i));
                    if let Some(close) = source[dpos..].find('}') {
                        i = dpos + close + 1;
                        continue;
                    }
                }
            }
        }
        // Advance one full UTF-8 char so `source[i..]` stays on a boundary — a
        // bare `i += 1` panics mid-codepoint on any multi-byte char (…, emoji).
        i += 1;
        while i < bytes.len() && (bytes[i] & 0xC0) == 0x80 {
            i += 1;
        }
    }
    out
}

/// After `sx`: `(ws)* = (ws)*` then `"` (static) or `{` (dynamic). Returns the
/// byte offset (within `rest`) of that delimiter, or None if this isn't an
/// `sx=`-valued attribute (e.g. `sxy=`, or `sx` with no following `=`).
fn sx_value_delim(rest: &[u8]) -> Option<usize> {
    let mut k = 0;
    while k < rest.len() && rest[k].is_ascii_whitespace() {
        k += 1;
    }
    if k >= rest.len() || rest[k] != b'=' {
        return None;
    }
    k += 1;
    while k < rest.len() && rest[k].is_ascii_whitespace() {
        k += 1;
    }
    if k < rest.len() && (rest[k] == b'"' || rest[k] == b'{') {
        Some(k)
    } else {
        None
    }
}

fn offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 1usize;
    let mut col = 1usize;
    for (i, b) in source.bytes().enumerate() {
        if i == offset {
            break;
        }
        if b == b'\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(src: &str) -> Vec<String> {
        check(Path::new("t.sail"), src).into_iter().map(|f| f.message).collect()
    }

    #[test]
    fn tokens_pass_and_raw_values_do_not() {
        assert!(run(r#"<card sx="p-4 bg-surface-2 rounded-lg text-sm text-muted-foreground">x</card>"#).is_empty());
        assert_eq!(run(r#"<card sx="p-16px">x</card>"#).len(), 1);
        assert_eq!(run(r#"<card sx="bg-#3366ff">x</card>"#).len(), 1);
    }

    #[test]
    fn structured_form_passes_and_rejects_raw_without_escape() {
        // IR v0: `prop: value` dual-parsed with legacy.
        assert!(
            run(r#"<div sx="bg: primary px: 2 rounded: sm hover:bg: muted">x</div>"#).is_empty(),
            "structured tokens must pass"
        );
        assert!(
            run(r#"<div sx="bg: primary px-2">x</div>"#).is_empty(),
            "mixed structured+legacy must pass"
        );
        let raw = run(r#"<div sx="bg: #3366ff">x</div>"#);
        assert_eq!(raw.len(), 1, "{raw:?}");
        assert!(raw[0].contains("raw:"), "must demand raw: escape: {}", raw[0]);
        let px = run(r#"<div sx="p: 16px">x</div>"#);
        assert_eq!(px.len(), 1, "{px:?}");
        assert!(px[0].contains("raw:"), "length needs raw:: {}", px[0]);
    }

    #[test]
    fn a_bad_value_prints_the_scale_so_the_author_knows_what_to_spend() {
        let m = &run(r#"<card sx="p-99">x</card>"#)[0];
        assert!(m.contains("accepts:"), "{m}");
        assert!(m.contains('4'), "the space scale must be printed: {m}");
    }

    #[test]
    fn the_layout_prior_gets_one_redirect_not_three_refusals() {
        // `flex items-center gap-2` is ONE breath of Tailwind muscle memory. Three
        // separate rejections is where "puzzle, not toolkit" comes back — and the
        // author writes it again on the next element regardless.
        let ms = run(r#"<div sx="flex items-center gap-2 p-4 bg-surface-2">x</div>"#);
        assert_eq!(ms.len(), 1, "one message, not three: {ms:?}");
        let m = &ms[0];
        assert!(m.contains(r#"<row gap="2" align="center" sx="p-4 bg-surface-2">"#), "{m}");
    }

    #[test]
    fn the_surviving_sx_comes_along_so_it_can_be_pasted_whole() {
        let m = &run(r#"<div sx="flex-col justify-between p-6">x</div>"#)[0];
        assert!(m.contains(r#"<column justify="between" sx="p-6">"#), "{m}");
    }

    #[test]
    fn a_grid_is_inferred_from_its_own_props() {
        // No `grid` atom, but `grid-cols-3` can only mean one thing.
        let m = &run(r#"<div sx="grid-cols-3 gap-4">x</div>"#)[0];
        assert!(m.contains(r#"<grid cols="3" gap="4">"#), "{m}");
    }

    #[test]
    fn flex_nowrap_redirects_to_a_buildable_row_nowrap() {
        // job/layout-pass: the redirect used to say "no layout keyword yet".
        // Now the whole breath collapses into a single buildable element.
        let m = &run(r#"<div sx="flex-nowrap items-center gap-2">x</div>"#)[0];
        assert!(m.contains(r#"<row nowrap gap="2" align="center">"#), "{m}");
        // And alone, it still names the built form.
        let m = &run(r#"<div sx="flex-nowrap">x</div>"#)[0];
        assert!(m.contains("<row nowrap"), "{m}");
    }

    #[test]
    fn grid_rows_redirects_to_a_buildable_rows_axis() {
        let m = &run(r#"<div sx="grid-rows-3 gap-4">x</div>"#)[0];
        assert!(m.contains(r#"<grid rows="3" gap="4">"#), "{m}");
    }

    #[test]
    fn a_refusal_names_the_element_to_use_instead() {
        // THE mission: never "not allowed, find another way".
        let m = &run(r#"<div sx="flex">x</div>"#)[0];
        assert!(m.contains("<row>"), "{m}");
        let m = &run(r#"<div sx="items-center">x</div>"#)[0];
        assert!(m.contains(r#"align="center""#), "{m}");
        let m = &run(r#"<div sx="gap-4">x</div>"#)[0];
        assert!(m.contains(r#"gap="4""#), "{m}");
    }

    #[test]
    fn the_mechanics_that_cost_components_their_designs_are_vocabulary_now() {
        assert!(run(r#"<img sx="object-cover">"#).is_empty(), "media's fit prop");
        assert!(run(r#"<div sx="touch-none origin-top-left">x</div>"#).is_empty(), "the canvas");
        assert!(run(r#"<div sx="border-l-3">x</div>"#).is_empty(), "the tool-call rail");
        assert!(run(r#"<div sx="opacity-85">x</div>"#).is_empty(), "the bar chart's rest state");
        assert!(run(r#"<div sx="rotate-1">x</div>"#).is_empty(), "sail-band's tilt");
        assert!(run(r#"<div sx="tabular-nums">x</div>"#).is_empty(), "chat timestamps");
        assert!(run(r#"<div sx="transition-colors duration-fast ease-out">x</div>"#).is_empty(), "change-driven motion");
        assert!(run(r#"<table sx="sr-only"><tr><td>1</td></tr></table>"#).is_empty(), "a chart's data table");
        assert!(run(r#"<p sx="text-gradient-primary">SHIP</p>"#).is_empty(), "the wordmark's gradient clip");
        assert!(run(r#"<p sx="text-gradient-accent">SHIP</p>"#).is_empty(), "gradient clip, accent tone");
        assert!(run(r#"<div sx="bg-glow-primary"></div>"#).is_empty(), "the hero's radial glow");
        assert!(run(r#"<div sx="bg-glow-accent"></div>"#).is_empty(), "radial glow, accent tone");
    }

    #[test]
    fn state_prefixes_are_closed_and_keyed_on_semantics() {
        assert!(run(r#"<a sx="hover:bg-surface-2 current:text-primary group-selected:text-foreground">x</a>"#).is_empty());
        assert_eq!(run(r#"<a sx="wobble:bg-surface-2">x</a>"#).len(), 1);
    }

    #[test]
    fn breakpoints_match_the_tokens() {
        // A media query cannot read a custom property, so the widths are literal
        // here. That is the one place sx can drift from the theme — so it can't.
        // Values come from the generated token registry (DTCG emit).
        for (name, query) in CONDITION_PREFIXES.iter().filter(|(n, _)| {
            matches!(*n, "mobile" | "tablet" | "desktop")
        }) {
            let value = crate::tokens::token_value(&format!("breakpoint-{name}"))
                .expect("the breakpoint token exists in the registry");
            assert!(
                query.contains(value),
                "`{name}:` is {query} but --breakpoint-{name} is {value} — sx has drifted from the theme"
            );
        }
    }

    #[test]
    fn conditions_are_closed() {
        assert!(run(r#"<div sx="p-2 tablet:p-4 motion-reduce:transition-none">x</div>"#).is_empty());
    }

    #[test]
    fn a_dynamic_sx_cannot_lower_so_it_is_an_error() {
        assert_eq!(run(r#"<card sx={cls()}>x</card>"#).len(), 1);
    }

    #[test]
    fn theme_declarations_never_widen_the_vocabulary() {
        // THE CLOSED BACK DOOR: a `<theme channels="…">` declaration must not make
        // its channel a legal sx value — ad-hoc channels were how translation snuck
        // past the design system. Rejected DECLARED OR NOT.
        let themed = "<theme prop=\"size\" channels=\"cb-size\">\n  default: icon-size-md\n</theme>\n<card sx=\"w-cb-size\">x</card>";
        assert_eq!(run(themed).len(), 1, "a declared ad-hoc channel is still rejected");
        // The FRAMEWORK channels come from the token registry and need no declaration.
        assert!(run(r#"<card sx="bg-tone text-tone-contrast border-tone-border">x</card>"#).is_empty());
    }
}
