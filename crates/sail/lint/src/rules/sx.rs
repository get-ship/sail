//! `sail-sx` — the inline style surface.
//!
//! Product form is Style: `sx={style.pad(4).bg(surface-2).radius(lg)}`,
//! `sx={when({ hover: style.bg(muted) })}`, `sx={[ match(…), when(…) ]}`, or a
//! scope Style const ref. Utility-string atoms (`sx="p-4 bg-surface-2"`) are a
//! **hard residual error** — not architecture.
//!
//! THE RULE: every value is a design token or a plain CSS keyword. Never a raw
//! number, never a colour, never a unit. There is no bracket escape (cf.
//! Tailwind's `p-[13px]`).
//!
//! The vocabulary is a TABLE (`sx-vocabulary.json`, read by `crate::sx_vocab`),
//! not code. Style setters re-home the same closed heads (`style.bg`, `style.pad`,
//! …). A scale-valued setter spends a token and nothing else; a keyword-valued
//! one takes its CSS keywords. The value discipline is the point.
//!
//! SCOPE — sx styles ONE box. How its children sit is a layout element
//! (`<row>`, `<column>`, `<grid>`), which carries the semantics, the a11y and
//! the overflow handling a hand-rolled flex box loses. Every layout atom is
//! REFUSED BY NAME, and the refusal names the element to use instead: the
//! mission is never "find another way".
//!
//! sx is compile-time and static: it lowers to atomic classes plus injected CSS.
//! A **runtime** dynamic `sx={expr}` (JS ternary, call, etc.) cannot be lowered
//! and is a hard error — conditional styling goes through a primitive prop
//! (`tone={…}`) or a `when` / `match` Style guard.

use std::path::Path;

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sail-sx";

// Residual string lowerers still call these names. SSOT is Style product:
// `crate::style::conditions` (`when({ tablet: … })`). Delete the re-exports
// when string product is removed.
pub use crate::style::conditions::{CONDITION_PREFIXES, condition_media};

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
    if v.is_empty()
        || !v
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    {
        return None;
    }
    Some(format!("[data-state~='{v}']"))
}

pub fn state_pseudo(state: &str) -> Option<&'static str> {
    STATE_PREFIXES
        .iter()
        .find(|(n, _)| *n == state)
        .map(|(_, p)| *p)
}

/// The complete sx vocabulary — every name with everything it accepts. THE single
/// source for surface documentation (the agent-skill generator, the /reference
/// page and the LSP all render this), so what an agent is told is what the lint
/// enforces, by construction.
pub fn vocabulary() -> Vec<(String, Vec<String>)> {
    crate::sx_vocab::vocabulary()
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
        "removable",
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
            state_pseudo(b).is_some() || state_data(b).is_some() || group_attr_presence(b).is_some()
        })
}

/// The layout element an sx attribute was TRYING to be.
///
/// `flex items-center gap-2 p-4` is one breath of Tailwind muscle memory and three
/// separate refusals — and the author will write it again on the next element. So it
/// must produce ONE diagnostic showing the whole replacement, not three rejections.
/// The boundary the prior fights hardest has to be the place the language teaches
/// best; a refusal an author has to reassemble by hand is where "puzzle, not toolkit"
/// comes back.
#[derive(Default)]
struct LayoutIntent {
    tag: Option<&'static str>,
    inline: bool,
    nowrap: bool,
    align: Option<String>,
    justify: Option<String>,
    gap: Option<String>,
    cols: Option<String>,
    rows: Option<String>,
    saw_any: bool,
}

impl LayoutIntent {
    /// Absorb one refused atom. Returns false if it is not a layout atom (so the
    /// caller still reports it on its own).
    fn absorb(&mut self, atom: &str) -> bool {
        let mut hit = true;
        match atom {
            "flex" | "flex-row" => self.tag = Some("row"),
            "inline-flex" => {
                self.tag = Some("row");
                self.inline = true;
            }
            "flex-col" => self.tag = Some("column"),
            "grid" => self.tag = Some("grid"),
            "inline-grid" => {
                self.tag = Some("grid");
                self.inline = true;
            }
            // A `<row>` wraps by default (it absorbed `sail-cluster`), so
            // `flex-wrap`'s intent IS a plain row — absorb it as one.
            "flex-wrap" => self.tag = Some("row"),
            // `flex-nowrap` is the single-line row — `<row nowrap>`. Absorb
            // it into the redirect so the whole Tailwind breath
            // (`flex nowrap items-center`) becomes one buildable element.
            "flex-nowrap" => {
                self.tag = Some("row");
                self.nowrap = true;
            }
            _ => {
                if let Some(v) = atom.strip_prefix("items-") {
                    self.align = Some(v.to_string());
                } else if let Some(v) = atom.strip_prefix("justify-") {
                    self.justify = Some(v.to_string());
                } else if let Some(v) = atom.strip_prefix("gap-") {
                    self.gap = Some(v.to_string());
                } else if let Some(v) = atom.strip_prefix("grid-cols-") {
                    self.cols = Some(v.to_string());
                // `grid-rows-N` is an explicit row count — `<grid rows="N">`.
                // Absorb it the same way `cols` is.
                } else if let Some(v) = atom.strip_prefix("grid-rows-") {
                    self.rows = Some(v.to_string());
                } else {
                    hit = false;
                }
            }
        }
        if hit {
            self.saw_any = true;
        }
        hit
    }

    /// The element to write instead, with the surviving sx carried over so the author
    /// can paste it whole rather than reassemble it.
    fn render(&self, kept: &[&str]) -> String {
        // No explicit `flex`/`grid`, but `items-`/`gap-` present: the author meant a
        // row — that is what those props are FOR.
        let tag = self
            .tag
            .unwrap_or(if self.cols.is_some() || self.rows.is_some() {
                "grid"
            } else {
                "row"
            });
        let mut el = format!("<{tag}");
        if self.inline {
            el.push_str(" inline");
        }
        if self.nowrap {
            el.push_str(" nowrap");
        }
        for (name, v) in [
            ("cols", &self.cols),
            ("rows", &self.rows),
            ("gap", &self.gap),
            ("align", &self.align),
            ("justify", &self.justify),
        ] {
            if let Some(v) = v {
                el.push_str(&format!(" {name}=\"{v}\""));
            }
        }
        if !kept.is_empty() {
            el.push_str(&format!(" sx=\"{}\"", kept.join(" ")));
        }
        el.push('>');
        el
    }
}

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    for hit in scan_sx(source) {
        let (val, offset) = match hit {
            SxHit::Dynamic(offset) => {
                let (line, col) = offset_to_line_col(source, offset);
                findings.push(mk_dynamic(path, line, col));
                continue;
            }
            SxHit::Static(val, offset) => (val, offset),
        };
        let (line, col) = offset_to_line_col(source, offset);

        // Sole Style product path: chain + match/when/compound/bind.
        // Utility-string sx and other residual forms are hard errors.
        {
            let (style, style_diags) = crate::style::parse::parse(val);
            for d in &style_diags {
                findings.push(mk(path, line, col, d.message()));
            }
            for d in crate::style::clash::detect_clashes(&style, &std::collections::HashMap::new())
            {
                findings.push(mk(path, line, col, d.message()));
            }
            // Always Style path — do not walk legacy atoms as a second product.
            continue;
        }

        #[allow(unreachable_code)]
        let (prog, parse_errs) = crate::sx_ir::parse_lenient(val);
        for e in &parse_errs {
            findings.push(mk(path, line, col, e.message()));
        }

        let mut layout = LayoutIntent::default();
        let mut kept: Vec<String> = Vec::new();
        for decl in &prog.decls {
            let atom = crate::sx_ir::decl_atom(decl);
            if matches!(
                crate::sx_vocab::resolve(&atom),
                crate::sx_vocab::Resolved::Refused { .. }
            ) {
                if layout.absorb(&atom) {
                    continue;
                }
            } else {
                kept.push(decl.source.clone());
            }
        }
        if layout.saw_any {
            let kept_refs: Vec<&str> = kept.iter().map(String::as_str).collect();
            findings.push(mk(path, line, col, format!(
                "sx does not do layout — the parent decides how its children sit. Write:\n\n    {}\n\nA layout element names the axis and closes the vocabulary, which is what stops two \
                 authors diverging on four flex properties.",
                layout.render(&kept_refs)
            )));
        }

        for decl in &prog.decls {
            let atom = crate::sx_ir::decl_atom(decl);
            // Layout atoms already reported as one redirect.
            if matches!(
                crate::sx_vocab::resolve(&atom),
                crate::sx_vocab::Resolved::Refused { .. }
            ) && LayoutIntent::default().absorb(&atom)
            {
                continue;
            }
            if let Err(e) = crate::sx_ir::validate_decl(decl) {
                // Enrich BadValue with options when possible.
                let msg = match &e {
                    crate::sx_ir::SxError::BadValue { prop, value, .. } => {
                        let atom = crate::sx_ir::decl_atom(decl);
                        match crate::sx_vocab::resolve(&atom) {
                            crate::sx_vocab::Resolved::BadValue { name, options } => {
                                let raw = value
                                    .chars()
                                    .next()
                                    .is_some_and(|c| c.is_ascii_digit() || c == '#')
                                    || value.ends_with("px")
                                    || value.ends_with("rem")
                                    || value.ends_with('%');
                                let mut m = format!("sx `{atom}` — `{value}` is not a token");
                                if raw {
                                    m.push_str(" (a value is a design decision; spend a token)");
                                }
                                m.push_str(&format!(". `{name}-` accepts: {}", options.join(", ")));
                                let opts: Vec<&str> = options.iter().map(String::as_str).collect();
                                if let Some(s) = nearest(value, &opts) {
                                    m.push_str(&format!("; did you mean `{name}-{s}`?"));
                                }
                                m
                            }
                            _ => {
                                let _ = prop;
                                if crate::sx_ir::looks_like_raw_color(value)
                                    || crate::sx_ir::looks_like_raw_length(value)
                                {
                                    crate::sx_ir::SxError::RawWithoutEscape {
                                        value: value.clone(),
                                        source: decl.source.clone(),
                                    }
                                    .message()
                                } else {
                                    e.message()
                                }
                            }
                        }
                    }
                    crate::sx_ir::SxError::UnknownPrefix { prefix, .. } => {
                        let states: Vec<String> = STATE_PREFIXES
                            .iter()
                            .map(|(n, _)| format!("{n}:"))
                            .collect();
                        let conditions: Vec<String> = CONDITION_PREFIXES
                            .iter()
                            .map(|(n, _)| format!("{n}:"))
                            .collect();
                        format!(
                            "sx unknown prefix `{prefix}:` — the closed set: {}, a condition ({}), \
                             `state-<value>:` for a component state, or group-<state>: to react \
                             to an ancestor",
                            states.join("/"),
                            conditions.join("/")
                        )
                    }
                    crate::sx_ir::SxError::UnknownAtom { atom, .. } => {
                        let mut m = format!("sx `{atom}` — not in the vocabulary");
                        let names = crate::sx_vocab::names();
                        let opts: Vec<&str> = names.iter().map(String::as_str).collect();
                        let head = atom.split('-').next().unwrap_or(atom);
                        if let Some(s) = nearest(head, &opts) {
                            m.push_str(&format!("; did you mean `{s}-`?"));
                        }
                        m
                    }
                    other => other.message(),
                };
                findings.push(mk(path, line, col, msg));
            }
        }
    }
    findings
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

/// Dynamic `sx={…}` — cannot be lowered (sx is compile-time). Reject it loudly
/// instead of silently emitting `class=""`.
fn mk_dynamic(path: &Path, line: usize, col: usize) -> Finding {
    Finding {
        file: path.to_path_buf(),
        line,
        col,
        rule: RULE_ID,
        severity: Severity::Error,
        message: "dynamic `sx={…}` is not supported — sx is a compile-time, static \
                  surface style, so a dynamic value silently lowers to nothing"
            .to_string(),
        suggest: Some(
            "Drive a primitive prop instead (e.g. `tone={ok() ? 'success' : 'destructive'}`), \
             or select between static `sx=\"…\"` values with an {#if} branch."
                .to_string(),
        ),
    }
}

/// One `sx` attribute occurrence: a static Style value (value + offset) to
/// validate, or a runtime dynamic `sx={…}` binding (offset of the `sx`) to reject.
enum SxHit<'a> {
    Static(&'a str, usize),
    Dynamic(usize),
}

/// Find every `sx=` attribute — static `sx="…"` / Style `sx={…}`, or runtime
/// dynamic `sx={jsExpr}`. Skips HTML comments.
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
                } else if bytes[dpos] == b'{' {
                    if let Some(close) = balanced_brace_end(bytes, dpos) {
                        let val = &source[dpos + 1..close];
                        if is_style_product_value(val) {
                            // Compile-time Style: chain / match / when / compound / list / const ref.
                            out.push(SxHit::Static(val, dpos + 1));
                        } else {
                            // Runtime JS expression — cannot lower.
                            out.push(SxHit::Dynamic(i));
                        }
                        i = close + 1;
                        continue;
                    }
                    out.push(SxHit::Dynamic(i));
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

/// Offset of the `}` matching `{` at `open`, or None if unbalanced.
fn balanced_brace_end(bytes: &[u8], open: usize) -> Option<usize> {
    if bytes.get(open) != Some(&b'{') {
        return None;
    }
    let mut depth = 0i32;
    let mut i = open;
    while i < bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            b'"' | b'\'' => {
                let q = bytes[i];
                i += 1;
                while i < bytes.len() && bytes[i] != q {
                    if bytes[i] == b'\\' && i + 1 < bytes.len() {
                        i += 1;
                    }
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// True when brace contents are a compile-time Style expression (not JS).
fn is_style_product_value(val: &str) -> bool {
    let t = val.trim();
    // Outer braces from nested quote forms: `{ style.… }` already stripped by scan.
    let t = t
        .strip_prefix('{')
        .and_then(|s| s.strip_suffix('}'))
        .map(str::trim)
        .unwrap_or(t);
    t.starts_with("style.")
        || t.starts_with("match(")
        || t.starts_with("when(")
        || t.starts_with("compound(")
        || t.starts_with('[')
        // Style const ref with a motion chain: `row.enter({…}).exit({…})` —
        // the dot/parens fail the bare-word check below but the value is a
        // compile-time Style product (the parser lowers the chain).
        || crate::style::parse::is_const_ref_motion_chain_pub(t)
        || {
            // bare Style const ref
            let mut chars = t.chars();
            match chars.next() {
                Some(c) if c.is_ascii_alphabetic() || c == '_' => {
                    t.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
                }
                _ => false,
            }
        }
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

fn nearest(word: &str, opts: &[&str]) -> Option<String> {
    opts.iter()
        .map(|o| (lev(word, o), o.to_string()))
        .filter(|(d, _)| *d <= 3)
        .min_by_key(|(d, _)| *d)
        .map(|(_, o)| o)
}

fn lev(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut d: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.iter().enumerate() {
        let mut prev = d[0];
        d[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let tmp = d[j + 1];
            d[j + 1] = if ca == cb {
                prev
            } else {
                1 + prev.min(d[j]).min(d[j + 1])
            };
            prev = tmp;
        }
    }
    d[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(src: &str) -> Vec<String> {
        check(Path::new("t.sail"), src)
            .into_iter()
            .map(|f| f.message)
            .collect()
    }

    #[test]
    fn style_chain_product_passes() {
        assert!(
            run(r#"<div sx={style.bg(primary).padX(2)}>x</div>"#).is_empty(),
            "{:?}",
            run(r#"<div sx={style.bg(primary).padX(2)}>x</div>"#)
        );
    }

    #[test]
    fn style_list_match_when_product_passes() {
        let src = r#"<div sx={[ match(variant, { solid: style.bg(primary) }), when({ hover: style.opacity(90) }) ]}>x</div>"#;
        assert!(run(src).is_empty(), "{:?}", run(src));
    }

    #[test]
    fn style_const_ref_is_static_product_not_dynamic() {
        // Bare ref is compile-time Style (scope const); not runtime `sx={cls()}`.
        assert!(
            run(r#"<div sx={filled}>x</div>"#).is_empty(),
            "{:?}",
            run(r#"<div sx={filled}>x</div>"#)
        );
    }

    #[test]
    fn utility_string_product_is_residual_error() {
        let ms = run(r#"<card sx="p-4 bg-surface-2">x</card>"#);
        assert!(!ms.is_empty(), "utility-string sx must fail closed");
        assert!(
            ms.iter()
                .any(|m| m.contains("utility-string") || m.contains("Style product")),
            "{ms:?}"
        );
    }

    #[test]
    fn structured_colon_soup_is_residual_error() {
        let ms = run(r#"<div sx="bg: primary px: 2">x</div>"#);
        assert!(!ms.is_empty(), "{ms:?}");
    }

    #[test]
    fn bad_token_fail_closed() {
        let ms = run(r#"<div sx={style.bg(notARealTokenZzz999)}>x</div>"#);
        assert!(!ms.is_empty(), "{ms:?}");
        assert!(
            ms.iter()
                .any(|m| m.contains("token") || m.contains("not a token")),
            "{ms:?}"
        );
    }

    #[test]
    fn unknown_setter_fail_closed() {
        let ms = run(r#"<div sx={style.notASetter(primary)}>x</div>"#);
        assert!(!ms.is_empty(), "{ms:?}");
    }

    #[test]
    fn runtime_dynamic_sx_is_an_error() {
        assert_eq!(run(r#"<card sx={cls()}>x</card>"#).len(), 1);
        assert_eq!(run(r#"<card sx={ok ? 'a' : 'b'}>x</card>"#).len(), 1);
    }

    #[test]
    fn breakpoints_match_the_tokens() {
        // Residual path must not diverge from Style conditions / design tokens.
        for (name, query) in CONDITION_PREFIXES
            .iter()
            .filter(|(n, _)| matches!(*n, "mobile" | "tablet" | "desktop"))
        {
            let value = crate::tokens::token_value(&format!("breakpoint-{name}"))
                .expect("the breakpoint token exists in the registry");
            assert!(
                query.contains(value),
                "Style condition `{name}` is {query} but --breakpoint-{name} is {value}"
            );
        }
    }
}
