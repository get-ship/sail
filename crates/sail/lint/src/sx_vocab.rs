//! **INTERNAL LEGACY MACHINERY — NOT THE PRODUCT PAINT SURFACE.**
//!
//! The legacy string-sx authoring form (`sx="p-4 bg-surface-2"`) is deleted
//! as product: the `sail-sx` lint hard-errors it and directs authors to the
//! new Style chain syntax (`style.bg(primary).padX(2)` in `sx={…}` /
//! `sx={[…]}`). This vocabulary table remains only as internal machinery
//! behind that lowering and the surface documentation; it is not a supported
//! authoring surface.
//!
//! The sx vocabulary — one table, read from `sx-vocabulary.json`.
//!
//! The lint validates against it and the lowering emits from it. Adding a
//! property is one row of data, not a code change in four places (which is why
//! the old hand-written `Kind` enum made forty gaps feel like forty projects).
//!
//! An atom is `[prefix:]…name-value`, or a bare keyword (`hidden`, `truncate`,
//! `italic`). Several utilities SHARE a name and disambiguate by value, exactly
//! as Tailwind does: `text-sm` is a font size, `text-foreground` is a colour,
//! `text-center` is an alignment. The engine picks the entry whose value-set
//! contains the value — so a name is a namespace, not a property.
//!
//! Scales live in `scales.rs`, not here: they are few, stable, and carry the
//! `var()` templates. Utilities live here because they are many and they grow.

use std::collections::HashMap;
use std::sync::OnceLock;

use serde::Deserialize;

use crate::scales;

const VOCAB_JSON: &str = include_str!("../sx-vocabulary.json");

#[derive(Debug, Deserialize)]
struct RawVocab {
    utilities: Vec<RawUtility>,
    refused: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct RawUtility {
    /// The atom's namespace (`p`, `text`, `min-w`). Absent for bare atoms.
    name: Option<String>,
    /// A bare atom IS its keyword (`hidden`, `italic`) — no name, no `-value`.
    #[serde(default)]
    bare: bool,
    /// A bare atom with a fixed declaration list (`truncate`, `sr-only`).
    atom: Option<String>,
    #[serde(default)]
    css: Vec<String>,
    /// The token scale this utility spends. The value must be on it.
    scale: Option<String>,
    /// CSS keywords whose value IS the CSS value.
    #[serde(default)]
    keywords: Vec<String>,
    /// Value → CSS value, when the two differ (`hidden` → `display: none`).
    #[serde(default)]
    map: HashMap<String, String>,
    /// A fixed declaration list, for composites.
    #[serde(default)]
    decls: Vec<Vec<String>>,
    #[serde(default)]
    negatable: bool,
    /// `translate-x` / `translate-y` both write `translate`; the axis says which.
    axis: Option<String>,
    #[serde(default)]
    child: Option<String>,
}

/// One resolved utility. `css` is empty exactly when `decls` is not.
#[derive(Debug)]
pub struct Utility {
    pub name: String,
    pub bare: bool,
    pub css: Vec<String>,
    pub scale: Option<String>,
    pub keywords: Vec<String>,
    pub map: HashMap<String, String>,
    pub decls: Vec<(String, String)>,
    pub negatable: bool,
    pub axis: Option<String>,
    pub child: Option<String>,
}

impl Utility {
    /// True iff `value` is a `(--name)` form — the sx-side seam to runtime
    /// geometry (a number that crossed JS, written to a custom property by
    /// an action). The shape mirrors Tailwind v4's `(--foo)` reader so an
    /// agent's prior transfers; the property's identity is verified so a
    /// typo at write time surfaces in lint, even though it is JS that
    /// sets the value at runtime.
    fn is_var_ref(value: &str) -> Option<&str> {
        if !value.starts_with("(--") || !value.ends_with(')') {
            return None;
        }
        let inner = &value[3..value.len() - 1];
        if inner.is_empty()
            || !inner.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return None;
        }
        Some(inner)
    }

    /// Does this utility accept `value`? A scale utility accepts any token on its
    /// scale; a keyword utility accepts its keywords; a mapped one its keys.
    /// `(--name)` is accepted on EVERY utility — the seam runs in BOTH
    /// directions (JS writes the var, sx declares the read), and the
    /// property's only constraint is that it exists at runtime.
    fn accepts(&self, value: &str) -> bool {
        if Self::is_var_ref(value).is_some() {
            return true;
        }
        if self.keywords.iter().any(|k| k == value) || self.map.contains_key(value) {
            return true;
        }
        match self.scale.as_deref() {
            Some(s) => scales::contains(s, value),
            None => false,
        }
    }

    /// The CSS declarations this utility + value lower to.
    fn lower(&self, value: &str, negative: bool) -> Option<Vec<(String, String)>> {
        if !self.decls.is_empty() {
            return Some(self.decls.clone());
        }
        let css_value = if let Some(var_name) = Self::is_var_ref(value) {
            // `(--row-y)` → `var(--row-y)`. The property's identity is
            // verified by `is_var_ref`; the value lives on the action.
            // `negatable` is irrelevant here — a runtime number has no
            // sign baked in: `-translate-y-(--row-y)` is rejected at the
            // parser for the same reason `-translate-y-300px` is (it isn't
            // a value the utility accepts; sign is the action's job).
            format!("var(--{var_name})")
        } else if let Some(mapped) = self.map.get(value) {
            mapped.clone()
        } else if self.keywords.iter().any(|k| k == value) {
            value.to_string()
        } else {
            let scale = self.scale.as_deref()?;
            let v = scales::lower(scale, value)?;
            if negative { format!("calc(-1 * {v})") } else { v }
        };
        // `translate-x`/`translate-y` share one CSS property; the axis decides
        // which half of the pair the value lands in. The var-ref form is the
        // common case for runtime geometry — `translate-y-(--row-y)` lowers
        // to `translate: 0 var(--row-y)`.
        if let Some(axis) = self.axis.as_deref() {
            let pair = if axis == "x" {
                format!("{css_value} var(--sx-translate-y, 0)")
            } else {
                format!("var(--sx-translate-x, 0) {css_value}")
            };
            return Some(vec![("translate".to_string(), pair)]);
        }
        Some(self.css.iter().map(|p| (p.clone(), css_value.clone())).collect())
    }
}

struct Vocab {
    /// Named utilities, grouped by name — several may share one (see the module doc).
    by_name: HashMap<String, Vec<Utility>>,
    /// Bare atoms: the atom itself is the whole utility.
    bare: HashMap<String, Utility>,
    /// A refused atom and the thing to use instead. The whole point of the table.
    refused: HashMap<String, String>,
    /// Every legal atom name, for suggestions.
    names: Vec<String>,
}

fn vocab() -> &'static Vocab {
    static V: OnceLock<Vocab> = OnceLock::new();
    V.get_or_init(|| {
        let raw: RawVocab = serde_json::from_str(VOCAB_JSON)
            .expect("sx-vocabulary.json is malformed — the vocabulary is the language");
        let mut by_name: HashMap<String, Vec<Utility>> = HashMap::new();
        let mut bare: HashMap<String, Utility> = HashMap::new();
        for r in raw.utilities {
            let u = Utility {
                name: r.name.clone().or_else(|| r.atom.clone()).unwrap_or_default(),
                bare: r.bare,
                css: r.css,
                scale: r.scale,
                keywords: r.keywords,
                map: r.map,
                decls: r
                    .decls
                    .iter()
                    .filter_map(|d| Some((d.first()?.clone(), d.get(1)?.clone())))
                    .collect(),
                negatable: r.negatable,
                axis: r.axis,
                child: r.child,
            };
            if r.bare {
                // A bare entry contributes one atom per key of its map, or the one
                // `atom` it names.
                if let Some(a) = r.atom {
                    bare.insert(a, u);
                } else {
                    for (k, v) in u.map.iter() {
                        bare.insert(
                            k.clone(),
                            Utility {
                                name: k.clone(),
                                bare: true,
                                css: u.css.clone(),
                                scale: None,
                                keywords: Vec::new(),
                                map: HashMap::from([(k.clone(), v.clone())]),
                                decls: Vec::new(),
                                negatable: false,
                                axis: None,
                                child: None,
                            },
                        );
                    }
                }
            } else if let Some(n) = r.name {
                by_name.entry(n).or_default().push(u);
            }
        }
        let mut names: Vec<String> = by_name.keys().cloned().collect();
        names.extend(bare.keys().cloned());
        names.sort();
        Vocab { by_name, bare, refused: raw.refused, names }
    })
}

/// What an atom resolved to.
pub enum Resolved {
    /// Legal: these declarations.
    Ok(Vec<(String, String)>),
    /// A refused atom, and the thing to use instead. Never "find another way".
    Refused { instead: String },
    /// The name is real but the value is not on its scale. `options` is the scale.
    BadValue { name: String, options: Vec<String> },
    /// No such utility.
    Unknown,
}

/// Descendant combinator for bare child-pack atoms.
pub fn child_of(atom: &str) -> Option<&'static str> {
    let body = atom.strip_prefix('-').unwrap_or(atom);
    vocab().bare.get(body).and_then(|u| u.child.as_deref())
}

/// Resolve one atom (already stripped of its prefixes).
pub fn resolve(atom: &str) -> Resolved {
    let v = vocab();

    let (negative, body) = match atom.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, atom),
    };

    // Exact full-atom refusal first (`flex`, not a longer bare allowlist).
    if let Some(instead) = v.refused.get(body) {
        return Resolved::Refused { instead: instead.clone() };
    }

    // Bare exact allowlist BEFORE head refusal so `animate-spin` (bare) is not
    // killed by refused head `animate` (which means motion:, not CSS animation).
    if let Some(u) = v.bare.get(body) {
        return match u.lower(body, false) {
            Some(decls) => Resolved::Ok(decls),
            None => Resolved::Unknown,
        };
    }

    // Head refusal for open ramps: `gap-4`, bare `animate-foo` without allowlist.
    if let Some((head, _)) = body.split_once('-') {
        if let Some(instead) = v.refused.get(head) {
            return Resolved::Refused { instead: instead.clone() };
        }
    }

    // Longest name match: `min-w-full` is (min-w, full), not (min, w-full).
    let mut best: Option<(&str, &str)> = None;
    for name in v.by_name.keys() {
        if let Some(rest) = body.strip_prefix(name.as_str()) {
            if let Some(value) = rest.strip_prefix('-') {
                if best.map(|(b, _)| name.len() > b.len()).unwrap_or(true) {
                    best = Some((name, value));
                }
            }
        }
    }
    let Some((name, value)) = best else {
        return Resolved::Unknown;
    };

    // Several utilities can share a name; the value picks which one.
    // `(--name)` is accepted by every utility, so when multiple share a head
    // (`font` → family vs weight) score the custom-property name against the
    // CSS properties each candidate writes (label-weight → font-weight, not
    // font-family).
    let candidates = &v.by_name[name];
    let mut matched: Vec<&Utility> = candidates.iter().filter(|u| u.accepts(value)).collect();
    if matched.is_empty() {
        return Resolved::BadValue {
            name: name.to_string(),
            options: values_of(candidates),
        };
    }
    if matched.len() > 1 {
        if let Some(var) = Utility::is_var_ref(value) {
            matched.sort_by_key(|u| std::cmp::Reverse(var_ref_score(var, u)));
        }
    }
    let u = matched[0];
    if negative && !u.negatable {
        return Resolved::BadValue {
            name: name.to_string(),
            options: values_of(candidates),
        };
    }
    match u.lower(value, negative) {
        Some(decls) => Resolved::Ok(decls),
        None => Resolved::Unknown,
    }
}

/// Prefer the candidate whose CSS property best matches the custom-property
/// name (e.g. `--label-weight` → `font-weight`, `--label-font` → `font-family`).
fn var_ref_score(var_name: &str, u: &Utility) -> i32 {
    let var = var_name.to_ascii_lowercase();
    let mut score = 0i32;
    for prop in &u.css {
        let p = prop.to_ascii_lowercase();
        // Exact tail match: `--foo-weight` with `font-weight`.
        if let Some(tail) = p.rsplit('-').next() {
            if var.ends_with(tail) || var.contains(&format!("-{tail}")) || var.contains(tail) {
                score += 10;
            }
        }
        // Full property token inside the var name.
        let compact: String = p.chars().filter(|c| *c != '-').collect();
        if var.replace('-', "").contains(&compact) {
            score += 5;
        }
        // Weight vs family disambiguation for the shared `font` head.
        if p.contains("weight") && var.contains("weight") {
            score += 20;
        }
        if p.contains("family") && (var.contains("family") || var.ends_with("-font") || var.contains("-font-")) {
            score += 20;
        }
        if p.contains("family") && var.contains("weight") {
            score -= 15;
        }
        if p.contains("weight") && (var.ends_with("-font") || var.contains("family")) && !var.contains("weight")
        {
            score -= 15;
        }
        // Width vs color for shared `border` / `outline` heads: `--bw`,
        // `--control-stroke`, `*-width` bind to width; color names to color.
        if p.ends_with("width")
            && (var == "bw"
                || var.ends_with("-bw")
                || var.contains("width")
                || var.contains("stroke")
                || var.ends_with("-w") && !var.contains("color") && var.len() <= 4)
        {
            score += 20;
        }
        if p.ends_with("color")
            && (var.contains("color") || var.ends_with("-fg") || var.ends_with("-bg") || var.contains("tone"))
        {
            score += 20;
        }
        if p.ends_with("color")
            && (var == "bw" || var.contains("width") || var.contains("stroke") || var == "sw-w" || var.ends_with("-bw"))
        {
            score -= 15;
        }
        if p.ends_with("width") && (var.contains("color") || var.ends_with("-fg") || var.ends_with("-bg")) {
            score -= 15;
        }
        // Font-size vs color for the shared `text` head: `--tab-fs`, `--tg-fs`.
        if p == "font-size" && (var.ends_with("-fs") || var.contains("font-size") || var.ends_with("-fontsize")) {
            score += 25;
        }
        if p.contains("color") && var.ends_with("-fs") {
            score -= 20;
        }
    }
    score
}

/// Everything a name will accept — this is what an error prints, so the author is
/// told what to spend rather than what not to.
fn values_of(candidates: &[Utility]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for u in candidates {
        out.extend(u.keywords.iter().cloned());
        out.extend(u.map.keys().cloned());
        if let Some(s) = u.scale.as_deref() {
            out.extend(scales::values(s));
        }
    }
    out.sort();
    out.dedup();
    out
}

/// Every legal atom name, for did-you-mean.
pub fn names() -> &'static [String] {
    &vocab().names
}

/// Every name with everything it accepts. The agent-skill generator, the
/// /reference page and the LSP all render THIS — so what an author is told is
/// what the lint enforces, by construction, and the docs can never drift.
pub fn vocabulary() -> Vec<(String, Vec<String>)> {
    let v = vocab();
    let mut out: Vec<(String, Vec<String>)> = v
        .by_name
        .iter()
        .map(|(name, candidates)| (name.clone(), values_of(candidates)))
        .collect();
    // Bare atoms take no value; they ARE the value.
    for name in v.bare.keys() {
        out.push((name.clone(), Vec::new()));
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// A refused atom and what to use instead — so a caller can render the whole
/// "not sx, use this" table rather than discovering it one error at a time.
pub fn refusals() -> Vec<(String, String)> {
    let mut r: Vec<(String, String)> = vocab()
        .refused
        .iter()
        .filter(|(k, _)| !k.starts_with('$'))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    r.sort();
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decls(atom: &str) -> Vec<(String, String)> {
        match resolve(atom) {
            Resolved::Ok(d) => d,
            other => panic!(
                "{atom} did not resolve: {}",
                match other {
                    Resolved::Refused { instead } => format!("refused → {instead}"),
                    Resolved::BadValue { name, .. } => format!("bad value for {name}"),
                    _ => "unknown".into(),
                }
            ),
        }
    }

    #[test]
    fn a_scale_utility_spends_a_token() {
        assert_eq!(decls("p-4"), vec![("padding".into(), "var(--space-4)".into())]);
        assert_eq!(decls("bg-surface-2"), vec![("background-color".into(), "var(--surface-2)".into())]);
    }

    #[test]
    fn one_name_disambiguates_by_value_like_tailwind() {
        // text-sm is a size, text-foreground is a colour, text-center is an alignment.
        assert_eq!(decls("text-sm"), vec![("font-size".into(), "var(--text-sm)".into())]);
        assert_eq!(decls("text-foreground"), vec![("color".into(), "var(--foreground)".into())]);
        assert_eq!(decls("text-center"), vec![("text-align".into(), "center".into())]);
    }

    #[test]
    fn a_mechanic_takes_its_css_keyword_with_no_allowlist() {
        // Every one of these was a "language gap" costing a component a design.
        assert_eq!(decls("object-cover"), vec![("object-fit".into(), "cover".into())]);
        assert_eq!(decls("touch-none"), vec![("touch-action".into(), "none".into())]);
        assert_eq!(decls("origin-top-left"), vec![("transform-origin".into(), "top left".into())]);
        assert_eq!(decls("cursor-crosshair"), vec![("cursor".into(), "crosshair".into())]);
        assert_eq!(decls("select-none"), vec![("user-select".into(), "none".into())]);
    }

    #[test]
    fn bare_atoms_are_their_own_keyword() {
        assert_eq!(decls("hidden"), vec![("display".into(), "none".into())]);
        assert_eq!(decls("relative"), vec![("position".into(), "relative".into())]);
        assert_eq!(decls("italic"), vec![("font-style".into(), "italic".into())]);
    }

    #[test]
    fn composites_lower_to_their_whole_block() {
        let sr = decls("sr-only");
        assert!(sr.iter().any(|(p, v)| p == "clip-path" && v == "inset(50%)"));
        // Neither declaration that would ALSO drop it from the a11y tree.
        assert!(!sr.iter().any(|(p, v)| p == "display" && v == "none"));
        assert!(!sr.iter().any(|(p, v)| p == "visibility" && v == "hidden"));
    }

    #[test]
    fn gradient_and_glow_composites_beat_the_named_prefixes() {
        // text-gradient-* / bg-glow-* are bare exact matches — they must NOT be
        // caught by the "text"/"bg" named-utility longest-prefix matcher first.
        let tg = decls("text-gradient-primary");
        assert!(tg.iter().any(|(p, v)| p == "background-clip" && v == "text"));
        assert!(tg.iter().any(|(p, v)| p == "color" && v == "transparent"));
        assert!(decls("text-gradient-accent").iter().any(|(p, _)| p == "background"));

        let glow = decls("bg-glow-primary");
        assert!(glow.iter().any(|(p, v)| p == "position" && v == "absolute"));
        assert!(glow.iter().any(|(p, _)| p == "background"));
        assert!(decls("bg-glow-accent").iter().any(|(p, _)| p == "background"));
    }

    #[test]
    fn animate_spin_bare_not_killed_by_refused_animate_head() {
        // `animate` is refused (use motion:) but `animate-spin` is a bare CSS PE atom.
        let d = decls("animate-spin");
        assert!(
            d.iter()
                .any(|(p, v)| p == "animation" && v == "var(--animate-spin)"),
            "got {d:?}"
        );
        assert!(matches!(resolve("animate"), Resolved::Refused { .. }));
    }

    #[test]
    fn avatar_pe_gradient_and_bg_none_lower() {
        let g = decls("bg-gradient-primary-accent");
        assert!(
            g.iter().any(|(p, v)| p == "background-image"
                && v.contains("linear-gradient")
                && v.contains("var(--primary)")
                && v.contains("var(--accent)")),
            "got {g:?}"
        );
        assert_eq!(
            decls("bg-none"),
            vec![("background-image".into(), "none".into())]
        );
    }

    #[test]
    fn longest_name_wins() {
        assert_eq!(decls("min-w-full"), vec![("min-width".into(), "100%".into())]);
    }

    #[test]
    fn a_raw_value_is_not_a_token() {
        assert!(matches!(resolve("p-16px"), Resolved::BadValue { .. }));
        assert!(matches!(resolve("bg-#3366ff"), Resolved::BadValue { .. } | Resolved::Unknown));
    }

    #[test]
    fn a_refusal_names_the_thing_to_use_instead() {
        // THE mission: never "find another way".
        match resolve("flex") {
            Resolved::Refused { instead } => assert!(instead.contains("<row>")),
            _ => panic!("flex must be refused, and must name <row>"),
        }
        match resolve("gap-4") {
            Resolved::Refused { instead } => assert!(instead.contains("gap")),
            _ => panic!("gap-4 must be refused, and must name the layout element's prop"),
        }
        match resolve("items-center") {
            Resolved::Refused { instead } => assert!(instead.contains("align")),
            _ => panic!("items-center must be refused, and must name <row align>"),
        }
    }

    // `(--name)` — the seam that lets runtime geometry reach sx without a
    // token. LANGUAGE-FLOOR.md: a value from measurement or from the user
    // is DATA, not design; the number crosses JS, the property stays in
    // sx. Tailwind v4's own reader syntax. Tests pin both directions:
    // validation (accepts the form) and emission (lowers to var(--name)).

    #[test]
    fn a_var_ref_lowers_to_var_for_any_scale_utility() {
        assert_eq!(
            decls("translate-y-(--row-y)"),
            vec![("translate".into(), "var(--sx-translate-x, 0) var(--row-y)".into())],
            "translate-y has the 'y' axis — defaults the x to 0"
        );
        assert_eq!(
            decls("top-(--row-y)"),
            vec![("top".into(), "var(--row-y)".into())],
            "top is just top: the var goes straight through"
        );
        assert_eq!(
            decls("h-(--rows-h)"),
            vec![("height".into(), "var(--rows-h)".into())],
        );
        assert_eq!(
            decls("max-w-(--max-w)"),
            vec![("max-width".into(), "var(--max-w)".into())],
        );
        assert_eq!(
            decls("bg-(--swatch-color)"),
            vec![("background-color".into(), "var(--swatch-color)".into())],
            "a runtime hex is data — colour scale accepts the var form"
        );
    }

    #[test]
    fn a_garbage_var_ref_is_rejected() {
        // Empty name — `(--)` would emit `var()`, which is a parse error in CSS.
        assert!(matches!(resolve("top-(--)"), Resolved::BadValue { .. }));
        // Identifier-unsafe — `(--row.y)` slips a `.` past the parser.
        assert!(matches!(
            resolve("top-(--row.y)"),
            Resolved::BadValue { .. }
        ));
        // Wrong shape — `(-row-y)` lacks the leading `--`.
        assert!(matches!(
            resolve("top-(-row-y)"),
            Resolved::BadValue { .. }
        ));
    }

    #[test]
    fn a_var_ref_is_legal_on_keyword_only_utilities_too() {
        // `transform-(--world-tx)` is the seam for canvas pan/zoom — the
        // utility happens to accept just `none` as a static keyword, but
        // the var form remains the runtime-geometry path.
        assert_eq!(
            decls("transform-(--world-tx)"),
            vec![("transform".into(), "var(--world-tx)".into())],
        );
    }
}
