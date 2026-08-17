//! `bind(name)` resolution — the closed style-value binding law.
//!
//! Resolution order, checked once per distinct bind found in author source
//! (tag-position Style products and `<scope>` Style consts):
//! design tokens → declared props → setup-returned signal / timeline /
//! derived → action-declared writes (`use:` + manifest `writes`) → refusal
//! speaking `bind(name)`.
//!
//! A token or action-write match needs no component-local `@property` /
//! host reflection (the token is global; the action drives the channel).
//! A prop / signal / derived match is returned so the emitter can register
//! a typed CSS `@property` and reflect the value onto the host.
//!
//! **Units:** number sources bound to length setters realize as px
//! (`syntax: '<length>'` + `${n}px` reflection). Number → color is a
//! compile error. Unannotated deriveds bound to length setters are a named
//! diagnostic (never silent invalid CSS).

use std::collections::{BTreeMap, BTreeSet};

use crate::action_manifest;
use crate::check::controller_parse::{ParsedController, ScopeKind};
use crate::diagnostics::{CompileError, Span};
use crate::emit::util::camel_of;
use crate::ir::{Program, PropKind};

const RULE_TOKEN_GROUP: &str = "spinnaker-token-group";
/// Which returned-scope shapes a `bind(name)` may resolve to.
///
/// `signal(...)` and stock progress drivers are plain signals. A
/// `timeline(...)` track and a setup-returned `derived(...)` are read-only
/// getters — the reflection effect only ever *calls* them.
fn is_bind_readable(kind: ScopeKind) -> bool {
    matches!(
        kind,
        ScopeKind::Signal | ScopeKind::Timeline | ScopeKind::Derived
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StyleBindingSource {
    Prop,
    Signal,
    /// Setup-returned `derived` — read-only, same reflection shape as signal.
    Derived,
}

/// How the bound value should realize as CSS.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StyleBindingUnit {
    /// Number → length setter: `@property` length + px reflection.
    LengthPx,
    /// Number → number setter (`opacity`, `z`, …): bare number.
    Number,
    /// String / untyped / unannotated derived: `syntax: '*'`.
    Untyped,
    /// Color token-group typed prop: host writes var *references*
    /// (`--tone: var(--primary)`, `--shade-foreground: var(--primary-foreground)`).
    /// Type-keyed (any prop whose union is the color-group set). No `@property`
    /// — bridge vars are untyped custom props.
    TokenGroup,
}

/// Dimension of the setter that accepted the bind (for units reconciliation).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SetterDim {
    Length,
    Number,
    Color,
    Other,
}

/// A `bind(name)` that resolved to a component-owned prop / signal / derived —
/// the pair the emitter wires `@property` + host reflection for.
#[derive(Debug, Clone)]
pub struct StyleBinding {
    /// Custom-property name without `--` (kebab-case: `row-y`).
    pub css_name: String,
    /// Scope member / prop name (camelCase: `rowY`).
    pub scope_name: String,
    pub source: StyleBindingSource,
    pub unit: StyleBindingUnit,
}

/// Resolve every `bind(name)` and every color token-group path (`bg(tone)`,
/// `fg(shade.foreground)`) in this source. `Ok` carries the prop/signal/
/// derived matches to wire at emit (token + action-write matches need no
/// component-local emission). `Err` is the first refusal, in source order.
pub fn resolve(
    source: &str,
    program: &Program,
    controller: Option<&ParsedController>,
) -> Result<Vec<StyleBinding>, CompileError> {
    let action_writes = collect_action_writes(source);
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut out: Vec<StyleBinding> = Vec::new();
    // Per-name: strongest unit demand + color contact (for number→color error).
    let mut unit_demand: BTreeMap<String, StyleBindingUnit> = BTreeMap::new();
    let mut color_touch: BTreeSet<String> = BTreeSet::new();
    let mut length_touch: BTreeSet<String> = BTreeSet::new();

    // §2.5: color-group typed props used as paint roots ride the same
    // machinery as bind — host var-reference reflection, type-keyed.
    // Only props whose union is the color-group set qualify (so `border(solid)`
    // / surface names never collide). Prop *name* is irrelevant (`tone`, `shade`).
    for site in find_group_path_sites(source) {
        let scope_name = site.root.clone();
        let Some(p) = program
            .props
            .iter()
            .find(|p| camel_of(&p.name) == scope_name)
        else {
            continue;
        };
        if !prop_is_color_group_typed(p) {
            // Non color-group prop name collided with a paint-arg scan — ignore
            // unless the author wrote an explicit member path (`shade.foreground`).
            if site.member.is_some() {
                return Err(CompileError::new(format!(
                    "`{scope_name}` is used as a token-group paint path \
                     (`{scope_name}.{}`) but its type is not a color group \
                     (Tone) — annotate the prop with the Tone union \
                     (primary·info·success·warning·danger·accent·muted).",
                    site.member.as_deref().unwrap_or("")
                ))
                .with_span(site.span)
                .with_category(RULE_TOKEN_GROUP));
            }
            continue;
        }
        if !seen.insert(scope_name.clone()) {
            continue;
        }
        // Invalid member on a known path (tone.hover when not universal).
        if let Some(ref member) = site.member {
            let path = vec![member.clone()];
            if let Err(msg) = sail_lint::token_groups::bound_member_valid(&path) {
                return Err(CompileError::new(format!(
                    "`{scope_name}.{member}` is not a valid color-group member — {msg}"
                ))
                .with_span(site.span)
                .with_category(RULE_TOKEN_GROUP));
            }
        }
        let css_name = camel_to_kebab(&scope_name);
        unit_demand.insert(scope_name.clone(), StyleBindingUnit::TokenGroup);
        out.push(StyleBinding {
            css_name,
            scope_name,
            source: StyleBindingSource::Prop,
            unit: StyleBindingUnit::TokenGroup,
        });
    }

    for site in find_bind_sites(source) {
        let css_name = camel_to_kebab(&site.name);
        let scope_name = site.name.clone();
        let dim = setter_dim(site.setter.as_deref());

        if dim == SetterDim::Color {
            color_touch.insert(scope_name.clone());
        }
        if dim == SetterDim::Length {
            length_touch.insert(scope_name.clone());
        }

        if !seen.insert(scope_name.clone()) {
            // Reconcile unit demand across multiple sites of the same name.
            if let Some(u) = unit_demand.get_mut(&scope_name) {
                *u = merge_unit(*u, dim);
            }
            continue;
        }

        // 1. Design token (registry without action-geometry convention tokens
        //    that have moved to `writes` — those must go through action writes).
        if is_design_token(&css_name) {
            unit_demand.insert(scope_name, StyleBindingUnit::Untyped);
            continue;
        }

        // 2. Declared prop.
        if let Some(p) = program
            .props
            .iter()
            .find(|p| camel_of(&p.name) == scope_name)
        {
            let numeric = matches!(p.kind, PropKind::Number);
            if numeric && dim == SetterDim::Color {
                return Err(number_to_color_error(
                    &scope_name,
                    site.setter.as_deref(),
                    site.span,
                ));
            }
            let unit = unit_for_numeric(numeric, dim);
            unit_demand.insert(scope_name.clone(), unit);
            out.push(StyleBinding {
                css_name,
                scope_name,
                source: StyleBindingSource::Prop,
                unit,
            });
            continue;
        }

        // 3. Setup-returned signal / timeline / derived.
        if let Some(c) = controller {
            if let Some(m) = c
                .returned
                .iter()
                .find(|m| m.name == scope_name && is_bind_readable(m.kind))
            {
                let init = c
                    .source
                    .get(m.value_span.start..m.value_span.end)
                    .unwrap_or("");
                let (numeric, source) = match m.kind {
                    ScopeKind::Signal => (signal_is_numeric(init), StyleBindingSource::Signal),
                    ScopeKind::Timeline => (false, StyleBindingSource::Signal),
                    ScopeKind::Derived => {
                        let ann = derived_return_is_number(init);
                        match ann {
                            Some(true) => (true, StyleBindingSource::Derived),
                            Some(false) => (false, StyleBindingSource::Derived),
                            None => {
                                // Unannotated: free for color/string; length needs proof.
                                if dim == SetterDim::Length {
                                    return Err(unannotated_derived_length_error(
                                        &scope_name,
                                        site.span,
                                    ));
                                }
                                (false, StyleBindingSource::Derived)
                            }
                        }
                    }
                    _ => unreachable!(),
                };
                if numeric && dim == SetterDim::Color {
                    return Err(number_to_color_error(
                        &scope_name,
                        site.setter.as_deref(),
                        site.span,
                    ));
                }
                let unit = unit_for_numeric(numeric, dim);
                unit_demand.insert(scope_name.clone(), unit);
                out.push(StyleBinding {
                    css_name,
                    scope_name,
                    source,
                    unit,
                });
                continue;
            }
        }

        // 4. Action-declared write — no component-local reflection.
        if action_writes.contains(scope_name.as_str()) {
            unit_demand.insert(scope_name, StyleBindingUnit::Untyped);
            continue;
        }

        return Err(refusal(
            &scope_name,
            &css_name,
            program,
            controller,
            &action_writes,
            site.span,
        ));
    }

    // Reconcile units for multi-site names (later length site upgrades number).
    for b in &mut out {
        if let Some(u) = unit_demand.get(&b.scope_name) {
            b.unit = *u;
        }
        // Number source that also touched color on a later site.
        if color_touch.contains(&b.scope_name)
            && matches!(
                b.unit,
                StyleBindingUnit::Number | StyleBindingUnit::LengthPx
            )
        {
            return Err(number_to_color_error(&b.scope_name, None, Span::new(0, 0)));
        }
        // Length site after first non-length classification.
        if length_touch.contains(&b.scope_name) && b.unit == StyleBindingUnit::Number {
            b.unit = StyleBindingUnit::LengthPx;
        }
    }

    Ok(out)
}

fn merge_unit(prev: StyleBindingUnit, dim: SetterDim) -> StyleBindingUnit {
    match (prev, dim) {
        (_, SetterDim::Length) if prev != StyleBindingUnit::Untyped => StyleBindingUnit::LengthPx,
        (StyleBindingUnit::LengthPx, _) => StyleBindingUnit::LengthPx,
        _ => prev,
    }
}

fn unit_for_numeric(numeric: bool, dim: SetterDim) -> StyleBindingUnit {
    if !numeric {
        return StyleBindingUnit::Untyped;
    }
    match dim {
        SetterDim::Length => StyleBindingUnit::LengthPx,
        SetterDim::Number => StyleBindingUnit::Number,
        SetterDim::Other | SetterDim::Color => StyleBindingUnit::Number,
    }
}

fn setter_dim(setter: Option<&str>) -> SetterDim {
    let Some(s) = setter else {
        return SetterDim::Other;
    };
    if LENGTH_SETTERS.contains(&s) {
        return SetterDim::Length;
    }
    if NUMBER_SETTERS.contains(&s) {
        return SetterDim::Number;
    }
    if COLOR_SETTERS.contains(&s) {
        return SetterDim::Color;
    }
    SetterDim::Other
}

/// Geometry / space / size setters that take lengths (number → px).
const LENGTH_SETTERS: &[&str] = &[
    "top",
    "right",
    "bottom",
    "left",
    "inset",
    "ms",
    "me",
    "ml",
    "mr",
    "mt",
    "mb",
    "pad",
    "padX",
    "padY",
    "padT",
    "padB",
    "padL",
    "padR",
    "padS",
    "padE",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "min-w",
    "min-h",
    "max-w",
    "max-h",
    "t",
    "l",
    "r",
    "b",
    "position",
    "margin",
    "gap",
    "gapX",
    "gapY",
    "basis",
    "size",
    "scrollM",
    "scrollP",
    "translateX",
    "translateY",
    "outlineOffset",
];

const NUMBER_SETTERS: &[&str] = &[
    "opacity", "z", "grow", "shrink", "order", "flex", "scale", "rotate",
];

const COLOR_SETTERS: &[&str] = &[
    "bg",
    "fg",
    "border",
    "borderX",
    "borderY",
    "borderT",
    "borderB",
    "borderL",
    "borderR",
    "borderS",
    "borderE",
    "outline",
    "accent",
    "caret",
    "fill",
    "stroke",
    "from",
    "via",
    "to",
    "decoration",
    "shadow",
    "ring",
    "divide",
];

/// Design-token registry (includes non-action convention tokens). Action
/// geometry channels resolve only via `writes`, not as tokens.
fn is_design_token(css_kebab: &str) -> bool {
    if ACTION_GEOMETRY_TOKENS.contains(&css_kebab) {
        return false;
    }
    sail_lint::tokens::registry().contains(css_kebab)
}

/// Former convention tokens that are now action `writes` — must not short-
/// circuit bind resolution as tokens.
const ACTION_GEOMETRY_TOKENS: &[&str] = &[
    "box-top",
    "box-left",
    "box-w",
    "box-h",
    "evt-top",
    "evt-h",
    "evt-left",
    "evt-w",
    "row-y",
    "rows-h",
    "cell-ps",
    "pin-x",
    "pin-y",
    "port-top",
    "port-left",
    "port-right",
    "port-bottom",
    "world-tx",
    "zoom-tx",
    "max-w",
    "swatch-color",
    "tc-x",
    "tc-y",
    "spot-x",
    "spot-y",
    "spot-strength",
    "sail-grid-template",
    "rng-pct",
    "anchor-left",
    "anchor-top",
    "anchor-width",
    "autosize-h",
    // tipDescribe channels
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
    "chrome-offset",
];

#[derive(Debug)]
struct BindSite {
    name: String,
    /// Setter that wraps the bind, if recoverable (`top` in `top(bind(x))`).
    setter: Option<String>,
    span: Span,
}

#[derive(Debug)]
struct GroupPathSite {
    /// Prop root (`tone`, `shade`).
    root: String,
    /// Optional member (`foreground`, `hover`).
    member: Option<String>,
    /// True when found inside a paint-ish setter call (heuristic).
    #[allow(dead_code)]
    in_paint: bool,
    span: Span,
}

/// Color token-group typed prop: union of color-group names (Tone set; no `'default'`).
/// Type-keyed — prop name is irrelevant (`tone`, `shade`, …).
fn prop_is_color_group_typed(p: &crate::ir::PropDecl) -> bool {
    match &p.kind {
        PropKind::Union(members) => sail_lint::token_groups::is_color_group_union(members),
        _ => false,
    }
}

/// Find bound group paint paths: `tone`, `tone.foreground`, `shade.hover` used
/// as paint-arg roots (not match axes / compounds). Heuristic scan over author
/// source — same spirit as `find_bind_sites`.
fn find_group_path_sites(source: &str) -> Vec<GroupPathSite> {
    let mut out = Vec::new();
    let bytes = source.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // Byte-prefix compares — `i` may land mid-multibyte char after +1.
        if bytes[i..].starts_with(b"<controller") {
            if let Some(rel) = find_bytes(bytes, i, b"</controller>") {
                i = rel + b"</controller>".len();
                continue;
            }
            break;
        }
        // Skip comments (HTML comments may contain multi-byte punctuation).
        if bytes[i..].starts_with(b"<!--") {
            if let Some(rel) = find_bytes(bytes, i, b"-->") {
                i = rel + 3;
                continue;
            }
            break;
        }
        // Identifier start (ASCII only).
        if bytes[i].is_ascii_alphabetic() || bytes[i] == b'_' {
            let start = i;
            i += 1;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            let root = std::str::from_utf8(&bytes[start..i]).unwrap_or("");
            // `bind(name)` is plain bind, not a group path root.
            if root == "bind" || (i < bytes.len() && bytes[i] == b'(') {
                continue;
            }
            // Skip known static group stems and color tokens — those are not binds.
            if sail_lint::token_groups::static_token_group(root).is_some() {
                continue;
            }
            if sail_lint::tokens::color_values().contains(&root) {
                continue;
            }
            // Member segment?
            let mut member = None;
            let mut end = i;
            if i < bytes.len() && bytes[i] == b'.' {
                let mem_start = i + 1;
                let mut j = mem_start;
                while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') {
                    j += 1;
                }
                if j > mem_start {
                    let m = std::str::from_utf8(&bytes[mem_start..j]).unwrap_or("");
                    if matches!(m, "foreground" | "hover" | "default") {
                        member = Some(m.to_string());
                        end = j;
                    }
                }
            }
            let in_paint = looks_like_paint_arg_context(source, start);
            if looks_like_match_key(source, start, end) {
                continue;
            }
            if !in_paint && member.is_none() {
                continue;
            }
            if in_paint || member.is_some() {
                if root.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                    && !root.is_empty()
                    && root.chars().next().is_some_and(|c| c.is_ascii_lowercase())
                {
                    out.push(GroupPathSite {
                        root: root.to_string(),
                        member,
                        in_paint,
                        span: Span::new(start, end),
                    });
                }
            }
            continue;
        }
        // UTF-8 safe advance past this byte / multi-byte sequence.
        i += 1;
        while i < bytes.len() && (bytes[i] & 0xC0) == 0x80 {
            i += 1;
        }
    }
    out
}

fn find_bytes(hay: &[u8], from: usize, needle: &[u8]) -> Option<usize> {
    hay[from..]
        .windows(needle.len())
        .position(|w| w == needle)
        .map(|p| from + p)
}

/// True when `ident_at` sits in a color-setter call arg list:
/// `bg(tone)`, `fg(tone.foreground)`, `colorMix(tone, 15)`, `border(tone)`.
fn looks_like_paint_arg_context(source: &str, ident_at: usize) -> bool {
    let bytes = source.as_bytes();
    let mut j = ident_at;
    while j > 0 && bytes[j - 1].is_ascii_whitespace() {
        j -= 1;
    }
    if j == 0 {
        return false;
    }
    // Second arg: `border(1, tone)` / `colorMix(tone, 15)` — walk to `(`.
    if bytes[j - 1] == b',' {
        while j > 0 && bytes[j - 1] != b'(' {
            j -= 1;
        }
    }
    if j == 0 || bytes[j - 1] != b'(' {
        return false;
    }
    // Identifier immediately before `(`.
    let mut k = j - 1; // points at `(`
    while k > 0 && bytes[k - 1].is_ascii_whitespace() {
        k -= 1;
    }
    let end = k;
    while k > 0 && (bytes[k - 1].is_ascii_alphanumeric() || bytes[k - 1] == b'_') {
        k -= 1;
    }
    if k == end {
        return false;
    }
    let setter = &source[k..end];
    COLOR_SETTERS.contains(&setter)
        || matches!(
            setter,
            "colorMix" | "colorDeepen" | "fill" | "stroke" | "from" | "via" | "to"
        )
}

fn looks_like_match_key(source: &str, start: usize, end: usize) -> bool {
    let bytes = source.as_bytes();
    let mut j = end;
    while j < bytes.len() && bytes[j].is_ascii_whitespace() {
        j += 1;
    }
    // `tone: primary` compound/match key form.
    if j < bytes.len() && bytes[j] == b':' {
        // Exclude TypeScript type positions in props — those use `tone: 'primary' |…`
        // which we still want to skip (not a paint path).
        let _ = start;
        return true;
    }
    false
}

/// Find every `bind(name)` in author source (Style products + scope consts).
fn find_bind_sites(source: &str) -> Vec<BindSite> {
    let mut out = Vec::new();
    let bytes = source.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // Skip controller blocks — binds there are not Style product.
        if source[i..].starts_with("<controller") {
            if let Some(end) = source[i..].find("</controller>") {
                i += end + "</controller>".len();
                continue;
            }
            break;
        }
        if source[i..].starts_with("bind(") {
            let name_start = i + 5;
            if let Some(close_rel) = source[name_start..].find(')') {
                let name = &source[name_start..name_start + close_rel];
                if !name.is_empty()
                    && name
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
                {
                    // Normalize kebab bind names to camel for resolution.
                    let scope = if name.contains('-') {
                        camel_of(name)
                    } else {
                        name.to_string()
                    };
                    // Reject expression forms: bind(derived(...)) / bind(a+b).
                    if name.contains('(') || name.contains('+') || name.contains(' ') {
                        let span = Span::new(i, name_start + close_rel + 1);
                        // Still surface as a site so resolve can refuse.
                        out.push(BindSite {
                            name: scope,
                            setter: None,
                            span,
                        });
                    } else {
                        let setter = find_setter_before(source, i);
                        let span = Span::new(i, name_start + close_rel + 1);
                        out.push(BindSite {
                            name: scope,
                            setter,
                            span,
                        });
                    }
                }
                i = name_start + close_rel + 1;
                continue;
            }
            break;
        }
        let mut n = i + 1;
        while n < bytes.len() && (bytes[n] & 0xC0) == 0x80 {
            n += 1;
        }
        i = n;
    }
    out
}

/// Recover `setter` from `…setter(bind(…` or `concern({ slot: bind(…` before `bind_at`.
fn find_setter_before(source: &str, bind_at: usize) -> Option<String> {
    let before = &source[..bind_at];
    let trimmed = before.trim_end();
    if let Some(from_slot) = setter_from_concern_slot(trimmed) {
        return Some(from_slot);
    }
    if !trimmed.ends_with('(') {
        return None;
    }
    let without_paren = &trimmed[..trimmed.len() - 1];
    let end = without_paren.len();
    let mut start = end;
    let bytes = without_paren.as_bytes();
    while start > 0 {
        let c = bytes[start - 1];
        if c.is_ascii_alphanumeric() || c == b'_' {
            start -= 1;
        } else {
            break;
        }
    }
    if start == end {
        return None;
    }
    let word = &without_paren[start..end];
    if word.is_empty() || !word.chars().next()?.is_ascii_alphabetic() {
        return None;
    }
    Some(word.to_string())
}

/// `size({ w: bind(` / `position({ t: bind(` → concern name (`size`, `position`).
fn setter_from_concern_slot(trimmed: &str) -> Option<String> {
    let s = trimmed.trim_end();
    if !s.ends_with(':') {
        return None;
    }
    let before_colon = s[..s.len() - 1].trim_end();
    let bytes = before_colon.as_bytes();
    let mut slot_start = before_colon.len();
    while slot_start > 0 {
        let c = bytes[slot_start - 1];
        if c.is_ascii_alphanumeric() || c == b'_' || c == b'-' {
            slot_start -= 1;
        } else {
            break;
        }
    }
    if slot_start == before_colon.len() {
        return None;
    }
    let slot = &before_colon[slot_start..];
    let before_slot = before_colon[..slot_start].trim_end();
    if !before_slot.ends_with('{') {
        return Some(slot.to_string());
    }
    let before_brace = before_slot[..before_slot.len() - 1].trim_end();
    if !before_brace.ends_with('(') {
        return Some(slot.to_string());
    }
    let without_paren = &before_brace[..before_brace.len() - 1];
    let end = without_paren.len();
    let mut start = end;
    let b = without_paren.as_bytes();
    while start > 0 {
        let c = b[start - 1];
        if c.is_ascii_alphanumeric() || c == b'_' {
            start -= 1;
        } else {
            break;
        }
    }
    if start == end {
        return Some(slot.to_string());
    }
    let concern = &without_paren[start..end];
    if concern.is_empty() || !concern.chars().next()?.is_ascii_alphabetic() {
        return Some(slot.to_string());
    }
    Some(concern.to_string())
}

/// Collect bare `use:action` names present in the source.
fn collect_action_writes(source: &str) -> BTreeSet<&'static str> {
    let mut names: BTreeSet<String> = BTreeSet::new();
    let bytes = source.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if source[i..].starts_with("use:") {
            let start = i + 4;
            let mut end = start;
            while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
                end += 1;
            }
            if end > start {
                names.insert(source[start..end].to_string());
            }
            i = end;
            continue;
        }
        let mut n = i + 1;
        while n < bytes.len() && (bytes[n] & 0xC0) == 0x80 {
            n += 1;
        }
        i = n;
    }
    action_manifest::writes_for_actions(names.iter().map(|s| s.as_str()))
}

fn camel_to_kebab(name: &str) -> String {
    let mut kebab = String::with_capacity(name.len() + 4);
    for (i, c) in name.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                kebab.push('-');
            }
            kebab.extend(c.to_lowercase());
        } else {
            kebab.push(c);
        }
    }
    kebab
}

/// `signal(<numeric-literal>)` initializer.
fn signal_is_numeric(init_text: &str) -> bool {
    let Some(rest) = init_text.trim_start().strip_prefix("signal") else {
        return false;
    };
    let rest = rest.trim_start();
    let rest = match rest.strip_prefix('<') {
        Some(r) => match r.find('>') {
            Some(i) => &r[i + 1..],
            None => return false,
        },
        None => rest,
    };
    let Some(inner) = rest.trim_start().strip_prefix('(') else {
        return false;
    };
    let bytes = inner.trim_start().as_bytes();
    let mut i = 0;
    if i < bytes.len() && (bytes[i] == b'-' || bytes[i] == b'+') {
        i += 1;
    }
    let mut has_digit = false;
    while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b'.') {
        has_digit = has_digit || bytes[i].is_ascii_digit();
        i += 1;
    }
    has_digit && (i >= bytes.len() || matches!(bytes[i], b')' | b',' | b' ' | b'\t' | b'\n'))
}

/// `derived((): number => …)` / `derived((): number=>{…})` — `Some(true)` when
/// annotated number, `Some(false)` when annotated non-number, `None` unannotated.
fn derived_return_is_number(init_text: &str) -> Option<bool> {
    let rest = init_text.trim_start();
    let rest = rest
        .strip_prefix("derived")
        .or_else(|| rest.strip_prefix("computed"))?
        .trim_start();
    // derived<number>(...) type param form
    if let Some(r) = rest.strip_prefix('<') {
        if let Some(close) = r.find('>') {
            let ty = r[..close].trim();
            return Some(ty == "number" || ty.starts_with("number "));
        }
    }
    // Scan for `): <type>` before `=>`
    if let Some(arrow) = rest.find("=>") {
        let head = &rest[..arrow];
        if let Some(colon) = head.rfind(':') {
            // Avoid matching `derived(() =>` with no type (no `):`).
            let after_colon = head[colon + 1..].trim();
            // Must look like a return annotation: previous non-space is `)`.
            let before = head[..colon].trim_end();
            if before.ends_with(')') {
                let ty = after_colon
                    .split(|c: char| c == '{' || c == ',' || c.is_whitespace())
                    .next()
                    .unwrap_or("")
                    .trim();
                if ty.is_empty() {
                    return None;
                }
                return Some(ty == "number");
            }
        }
        return None;
    }
    None
}

fn number_to_color_error(name: &str, setter: Option<&str>, span: Span) -> CompileError {
    let setter_bit = setter.map(|s| format!(" `{s}`")).unwrap_or_default();
    CompileError::new(format!(
        "`bind({name})` is a number but{setter_bit} is a color setter — number→color is not a \
         legal unit realization. Use a string/color source, or bind the number to a length/number \
         setter."
    ))
    .with_span(span)
    .with_category("spinnaker-bind")
}

fn unannotated_derived_length_error(name: &str, span: Span) -> CompileError {
    CompileError::new(format!(
        "can't prove `{name}` is a number — annotate the return type \
         (`derived((): number => …)`) so `bind({name})` can realize as a length (px). \
         Unannotated deriveds bind freely to color/string contexts only."
    ))
    .with_span(span)
    .with_category("spinnaker-bind")
}

fn refusal(
    scope_name: &str,
    css_name: &str,
    program: &Program,
    controller: Option<&ParsedController>,
    action_writes: &BTreeSet<&'static str>,
    span: Span,
) -> CompileError {
    let mut candidates: Vec<String> = program.props.iter().map(|p| camel_of(&p.name)).collect();
    if let Some(c) = controller {
        candidates.extend(
            c.returned
                .iter()
                .filter(|m| is_bind_readable(m.kind))
                .map(|m| m.name.clone()),
        );
    }
    candidates.extend(action_writes.iter().map(|s| (*s).to_string()));
    // Token names as camel for did-you-mean when author used camel-ish form.
    candidates.extend(
        sail_lint::tokens::registry()
            .iter()
            .filter(|s| !ACTION_GEOMETRY_TOKENS.contains(s))
            .map(|s| camel_of(s)),
    );

    let mut msg = format!(
        "`bind({scope_name})` does not match a design token, a declared prop, a setup-returned \
         signal/timeline/derived, or an action-declared write. Style values track sail concepts \
         via `bind(name)` — the name must be a token, a `<scope>` prop, a value `setup()` returns \
         via `signal(...)` / `derived(...)` / a `timeline(...)` track, or a channel declared in \
         an action's `writes` for a `use:` on this file."
    );
    if let Some(s) = closest(scope_name, &candidates).or_else(|| closest(css_name, &candidates)) {
        msg.push_str(&format!(" Did you mean `bind({s})`?"));
    }
    CompileError::new(msg)
        .with_span(span)
        .with_category("spinnaker-bind")
}

fn closest(name: &str, candidates: &[String]) -> Option<String> {
    let max_dist = (name.len() / 2).max(2);
    candidates
        .iter()
        .map(|c| (c, levenshtein(name, c)))
        .filter(|(_, d)| *d <= max_dist)
        .min_by_key(|(_, d)| *d)
        .map(|(c, _)| c.clone())
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0usize; b.len() + 1];
    for i in 1..=a.len() {
        cur[0] = i;
        for j in 1..=b.len() {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            cur[j] = (prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::PropDecl;

    fn prop(name: &str, kind: PropKind) -> PropDecl {
        PropDecl {
            name: name.to_string(),
            kind,
            default: None,
            reflect: false,
            bindable: false,
            description: None,
            span: Span::new(0, 0),
        }
    }

    fn program_with_props(props: Vec<PropDecl>) -> Program {
        Program {
            props,
            styles: String::new(),
            scope_types: String::new(),
            template: Vec::new(),
            source_len: 0,
            span: Span::new(0, 0),
        }
    }

    #[test]
    fn a_prop_match_returns_a_binding() {
        let program = program_with_props(vec![prop("glowRadius", PropKind::Number)]);
        let source = r#"<div style({ size({ w: bind(glowRadius) }) })></div>"#;
        let out = resolve(source, &program, None).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].scope_name, "glowRadius");
        assert_eq!(out[0].source, StyleBindingSource::Prop);
        assert_eq!(out[0].unit, StyleBindingUnit::LengthPx);
    }

    #[test]
    fn a_signal_match_returns_a_binding() {
        let program = program_with_props(vec![]);
        let controller = crate::check::controller_parse::parse(
            Default::default(),
            "controller((ctx) => { const hue = signal(220); return { hue }; })".to_string(),
        )
        .unwrap();
        // hue is number but bg is color → error
        let source = r#"<div style({ opacity(bind(hue)) })></div>"#;
        let out = resolve(source, &program, Some(&controller)).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].scope_name, "hue");
        assert_eq!(out[0].source, StyleBindingSource::Signal);
        assert_eq!(out[0].unit, StyleBindingUnit::Number);
    }

    #[test]
    fn number_to_color_refuses() {
        let program = program_with_props(vec![prop("n", PropKind::Number)]);
        let source = r#"<div style({ bg(bind(n)) })></div>"#;
        let err = resolve(source, &program, None).unwrap_err();
        assert!(err.message.contains("number"), "{}", err.message);
        assert!(
            err.message.contains("color") || err.message.contains("bg"),
            "{}",
            err.message
        );
    }

    #[test]
    fn derived_annotated_number_binds_to_length() {
        let program = program_with_props(vec![]);
        let controller = crate::check::controller_parse::parse(
            Default::default(),
            "controller((ctx) => { const doubled = derived((): number => 2); return { doubled }; })"
                .to_string(),
        )
        .unwrap();
        let source = r#"<div style({ position({ t: bind(doubled) }) })></div>"#;
        let out = resolve(source, &program, Some(&controller)).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].source, StyleBindingSource::Derived);
        assert_eq!(out[0].unit, StyleBindingUnit::LengthPx);
    }

    #[test]
    fn derived_unannotated_length_refuses_with_annotation_hint() {
        let program = program_with_props(vec![]);
        let controller = crate::check::controller_parse::parse(
            Default::default(),
            "controller((ctx) => { const doubled = derived(() => 2); return { doubled }; })"
                .to_string(),
        )
        .unwrap();
        let source = r#"<div style({ position({ t: bind(doubled) }) })></div>"#;
        let err = resolve(source, &program, Some(&controller)).unwrap_err();
        assert!(err.message.contains("annotate"), "{}", err.message);
        assert!(err.message.contains("doubled"), "{}", err.message);
    }

    #[test]
    fn action_write_resolves_without_reflection() {
        let program = program_with_props(vec![]);
        let source = r#"<div use:anchor style({ position({ l: bind(anchorLeft) }) })></div>"#;
        let out = resolve(source, &program, None).unwrap();
        assert!(out.is_empty(), "action writes need no component wiring");
    }

    #[test]
    fn action_write_without_use_refuses() {
        let program = program_with_props(vec![]);
        let source = r#"<div style({ position({ l: bind(anchorLeft) }) })></div>"#;
        let err = resolve(source, &program, None).unwrap_err();
        assert!(err.message.contains("bind(anchorLeft)"), "{}", err.message);
    }

    #[test]
    fn color_group_typed_prop_binds_as_token_group() {
        let members: Vec<String> = sail_lint::token_groups::color_group_names()
            .iter()
            .map(|s| (*s).to_string())
            .collect();
        let program = program_with_props(vec![prop("tone", PropKind::Union(members.clone()))]);
        let source = r#"<div style({ bg(tone).fg(tone.foreground) })></div>"#;
        let out = resolve(source, &program, None).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].scope_name, "tone");
        assert_eq!(out[0].unit, StyleBindingUnit::TokenGroup);
    }

    #[test]
    fn shade_color_group_prop_is_type_keyed() {
        let members: Vec<String> = sail_lint::token_groups::color_group_names()
            .iter()
            .map(|s| (*s).to_string())
            .collect();
        let program = program_with_props(vec![prop("shade", PropKind::Union(members))]);
        let source = r#"<div style({ bg(shade) })></div>"#;
        let out = resolve(source, &program, None).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].scope_name, "shade");
        assert_eq!(out[0].unit, StyleBindingUnit::TokenGroup);
    }

    #[test]
    fn color_group_hover_member_refuses_when_not_universal() {
        let members: Vec<String> = sail_lint::token_groups::color_group_names()
            .iter()
            .map(|s| (*s).to_string())
            .collect();
        let program = program_with_props(vec![prop("tone", PropKind::Union(members))]);
        let source = r#"<div style({ bg(tone.hover) })></div>"#;
        let err = resolve(source, &program, None).unwrap_err();
        assert!(
            err.message.contains("hover") || err.message.contains("member"),
            "{}",
            err.message
        );
    }

    #[test]
    fn size_union_is_recognized_not_color_group() {
        assert!(sail_lint::token_groups::is_size_union(&[
            "sm".into(),
            "md".into(),
            "lg".into(),
        ]));
        assert!(sail_lint::token_groups::is_size_union(&[
            "xs".into(),
            "sm".into(),
            "md".into(),
            "lg".into(),
            "xl".into(),
        ]));
        assert!(!sail_lint::token_groups::is_size_union(&[
            "primary".into(),
            "info".into(),
        ]));
        let program = program_with_props(vec![prop(
            "size",
            PropKind::Union(vec!["sm".into(), "md".into(), "lg".into()]),
        )]);
        // Size is match conditionality, not color-group paint bind.
        let source = r#"<div style({ bg(size) })></div>"#;
        let out = resolve(source, &program, None).unwrap();
        assert!(
            out.iter().all(|b| b.unit != StyleBindingUnit::TokenGroup),
            "size must not bind as color-group"
        );
    }

    #[test]
    fn style_bind_form_returns_a_binding() {
        let program = program_with_props(vec![]);
        let controller = crate::check::controller_parse::parse(
            Default::default(),
            "controller((ctx) => { const tileX = signal('0px'); return { tileX }; })".to_string(),
        )
        .unwrap();
        let source = r#"<div style({ translateX(bind(tileX)) })></div>"#;
        let out = resolve(source, &program, Some(&controller)).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].css_name, "tile-x");
        assert_eq!(out[0].scope_name, "tileX");
        assert_eq!(out[0].unit, StyleBindingUnit::Untyped);
    }

    #[test]
    fn a_token_var_is_unaffected_and_wires_nothing() {
        let program = program_with_props(vec![]);
        let source = r#"<div style({ bg(surface) })></div>"#;
        let out = resolve(source, &program, None).unwrap();
        assert!(out.is_empty());
    }

    #[test]
    fn an_unmatched_name_refuses_with_did_you_mean() {
        let program = program_with_props(vec![prop("rowY", PropKind::Number)]);
        let source = r#"<div style({ position({ t: bind(rowX) }) })></div>"#;
        let err = resolve(source, &program, None).unwrap_err();
        assert!(err.message.contains("does not match"), "{}", err.message);
        assert!(
            err.message.contains("rowY") || err.message.contains("bind(rowY)"),
            "{}",
            err.message
        );
    }

    #[test]
    fn a_signal_not_returned_from_setup_is_not_reachable() {
        let program = program_with_props(vec![]);
        let controller = crate::check::controller_parse::parse(
            Default::default(),
            "controller((ctx) => { const hue = signal(220); return {}; })".to_string(),
        )
        .unwrap();
        let source = r#"<div style({ opacity(bind(hue)) })></div>"#;
        let err = resolve(source, &program, Some(&controller)).unwrap_err();
        assert!(
            err.message.contains("does not match") || err.message.contains("hue"),
            "{}",
            err.message
        );
    }

    #[test]
    fn an_unread_name_emits_no_wiring() {
        let program = program_with_props(vec![prop("rowY", PropKind::Number)]);
        let source = r#"<div class="static"></div>"#;
        let out = resolve(source, &program, None).unwrap();
        assert!(out.is_empty());
    }

    #[test]
    fn derived_return_annotation_detection() {
        assert_eq!(
            derived_return_is_number("derived((): number => count() * 2)"),
            Some(true)
        );
        assert_eq!(
            derived_return_is_number("derived((): string => 'x')"),
            Some(false)
        );
        assert_eq!(derived_return_is_number("derived(() => 2)"), None);
    }
}
