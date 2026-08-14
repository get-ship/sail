//! Baked typography metrics for the closed font/size vocabulary.
//!
//! The underlying table is generated from the actual framework fonts in
//! `packages/sail/tokens/src/fonts/` so every value is deterministic and
//! reproducible across machines. It is intentionally a first slice:
//! - Latin + ASCII coverage only; non-ASCII falls back to a typical Latin
//!   glyph width.
//! - Kerning, ligatures, BiDi, and complex shaping are cut. The goal is
//!   "honest bounds for layout diagnostics", not pixel-perfect rendering.
//!
//! Regeneration: see `scripts/generate-metrics/` in this crate.

mod metrics_generated;

pub use metrics_generated::{Family, Metrics, Size, table};

use crate::html::Element;

impl Size {
    /// Parse a type scale step (`xl`, `2xl`). Unknown → `base` (the same
    /// default the CSS body rule sets). Input is the token name from Style,
    /// not a class string.
    pub fn from_scale_token(tok: &str) -> Size {
        match tok {
            "xs" => Size::Xs,
            "sm" => Size::Sm,
            "base" => Size::Base,
            "md" => Size::Md,
            "lg" => Size::Lg,
            "xl" => Size::Xl,
            "2xl" => Size::Xxl,
            "3xl" => Size::Xxxl,
            "4xl" => Size::Xxxxl,
            "display" => Size::Display,
            _ => Size::Base,
        }
    }

    /// Alias of [`Self::from_scale_token`] — token-name parse, not class scrape.
    pub fn from_class_token(tok: &str) -> Size {
        Self::from_scale_token(tok)
    }
}

/// Width of a single character using the baked ASCII table. Non-ASCII
/// characters fall back to the per-family/size average Latin glyph width.
pub fn char_width(m: &Metrics, c: char) -> f32 {
    let b = c as u32;
    if (32..=126).contains(&b) {
        m.ascii_widths[(b - 32) as usize]
    } else {
        m.fallback_width_px
    }
}

/// Width of a whole word (sum of character advances).
pub fn word_width(m: &Metrics, word: &str) -> f32 {
    word.chars().map(|c| char_width(m, c)).sum()
}

/// Style-resolved type tokens joined onto a parsed host (class list is
/// identity only — measure never parses `s-fontSize*`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TypeHost {
    pub classes: Vec<String>,
    pub size: Option<String>,
    pub family: Option<String>,
}

impl TypeHost {
    pub fn from_resolved(classes: &str, resolved: &spinnaker::ResolvedType) -> Option<Self> {
        if resolved.is_empty() {
            return None;
        }
        let classes: Vec<String> = classes
            .split_whitespace()
            .filter(|c| !c.is_empty())
            .map(|c| c.to_string())
            .collect();
        if classes.is_empty() {
            return None;
        }
        Some(Self {
            classes,
            size: resolved.size.clone(),
            family: resolved.family.clone(),
        })
    }
}

/// Fill unset type fields from Style theme tag defaults (`h2` → heading role).
pub fn apply_tag_default_types(elements: &mut [Element]) {
    for elem in elements.iter_mut().skip(1) {
        if elem.tag.is_empty() {
            continue;
        }
        let resolved = spinnaker::resolve_type_for_tag(&elem.tag);
        if elem.type_size.is_none() {
            elem.type_size = resolved.size;
        }
        if elem.type_family.is_none() {
            elem.type_family = resolved.family;
        }
    }
}

/// Overlay Style-resolved type onto hosts whose emitted class identity matches.
/// Tag defaults fill remaining unset fields.
pub fn apply_resolved_types(elements: &mut [Element], hosts: &[TypeHost]) {
    apply_tag_default_types(elements);
    if hosts.is_empty() {
        return;
    }
    for elem in elements.iter_mut().skip(1) {
        let class_set: std::collections::HashSet<&str> = elem.class.split_whitespace().collect();
        let mut best: Option<&TypeHost> = None;
        let mut best_n = 0usize;
        for host in hosts {
            if host.classes.is_empty() {
                continue;
            }
            if host.classes.iter().all(|c| class_set.contains(c.as_str()))
                && host.classes.len() > best_n
            {
                best_n = host.classes.len();
                best = Some(host);
            }
        }
        if let Some(host) = best {
            if host.size.is_some() {
                elem.type_size.clone_from(&host.size);
            }
            if host.family.is_some() {
                elem.type_family.clone_from(&host.family);
            }
        }
    }
}

/// Resolve the `(family, size)` for a text leaf by walking:
///   1. The element's own Style-resolved type tokens.
///   2. Ancestors' resolved tokens (closest wins).
///   3. Default body / base.
///
/// Tag defaults and `type({ size })` must already be applied onto
/// [`Element::type_size`] / [`Element::type_family`].
pub fn resolve_family_size(elem_index: usize, all: &[Element]) -> (Family, Size) {
    let mut family: Option<Family> = None;
    let mut size: Option<Size> = None;

    let mut cursor = Some(elem_index);
    while (family.is_none() || size.is_none()) && cursor.is_some() {
        let i = cursor.unwrap();
        let node = &all[i];
        if family.is_none() {
            family = node.type_family.as_deref().and_then(family_from_token);
        }
        if size.is_none() {
            if let Some(tok) = node.type_size.as_deref() {
                size = Some(Size::from_scale_token(tok));
            }
        }
        cursor = node.parent;
    }

    (family.unwrap_or(Family::Body), size.unwrap_or(Size::Base))
}

/// Font scale token (`display`, `mono`, `heading`) → baked family.
pub fn family_from_token(tok: &str) -> Option<Family> {
    match tok {
        "mono" | "monofont" | "mono-font" => Some(Family::Mono),
        "display" | "displayfont" | "display-font" => Some(Family::Display),
        "heading" | "headingfont" | "heading-font" => Some(Family::Display),
        "body" | "bodyfont" | "body-font" => Some(Family::Body),
        "pressstart" | "pressstart2p" | "press-start" => Some(Family::PressStart),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_covers_every_size_for_all_families() {
        let t = table();
        let families = [
            Family::Body,
            Family::Mono,
            Family::Display,
            Family::PressStart,
        ];
        let sizes = [
            Size::Xs,
            Size::Sm,
            Size::Base,
            Size::Md,
            Size::Lg,
            Size::Xl,
            Size::Xxl,
            Size::Xxxl,
            Size::Xxxxl,
            Size::Display,
        ];
        for family in families {
            for size in sizes {
                assert!(
                    t.contains_key(&(family, size)),
                    "missing metrics for family={family:?} size={size:?}"
                );
            }
        }
    }

    #[test]
    fn line_height_is_one_and_a_half_em() {
        let t = table();
        for &(family, size) in t.keys() {
            let m = &t[&(family, size)];
            assert!(
                (m.line_height_px / size.px() - 1.5).abs() < 0.001,
                "line-height mismatch for family={family:?} size={size:?}"
            );
        }
    }

    #[test]
    fn press_start_is_fixed_width() {
        let t = table();
        let m = &t[&(Family::PressStart, Size::Base)];
        let first = m.ascii_widths[0];
        for w in &m.ascii_widths {
            assert!(
                (w - first).abs() < 0.001,
                "Press Start 2P should be fixed-width"
            );
        }
    }

    #[test]
    fn resolve_family_size_reads_style_tokens() {
        let mut all = vec![Element::default()];
        let span = Element {
            tag: "span".to_string(),
            type_size: Some("4xl".into()),
            type_family: Some("display".into()),
            ..Default::default()
        };
        all.push(span);
        let (family, size) = resolve_family_size(1, &all);
        assert_eq!(family, Family::Display);
        assert_eq!(size, Size::Xxxxl);
    }

    #[test]
    fn class_spelling_does_not_drive_measure() {
        let mut all = vec![Element::default()];
        all.push(Element {
            tag: "span".to_string(),
            class: "s-fontSize4xl s-fontFamilydisplay".to_string(),
            ..Default::default()
        });
        let (family, size) = resolve_family_size(1, &all);
        assert_eq!(family, Family::Body);
        assert_eq!(size, Size::Base);
    }

    #[test]
    fn tag_default_types_come_from_style_theme() {
        let mut all = vec![Element::default()];
        all.push(Element {
            tag: "h2".to_string(),
            ..Default::default()
        });
        apply_tag_default_types(&mut all);
        let (family, size) = resolve_family_size(1, &all);
        let expected = spinnaker::resolve_type_for_tag("h2");
        assert_eq!(
            size,
            Size::from_scale_token(expected.size.as_deref().unwrap_or("base"))
        );
        assert_eq!(size, Size::Xl);
        assert_eq!(family, Family::Display);
    }

    #[test]
    fn inherit_type_from_ancestor_tokens() {
        let mut all = vec![Element::default()];
        all.push(Element {
            tag: "p".to_string(),
            type_size: Some("xl".into()),
            type_family: Some("mono".into()),
            children: vec![2],
            parent: Some(0),
            ..Default::default()
        });
        all.push(Element {
            tag: "span".to_string(),
            parent: Some(1),
            ..Default::default()
        });
        let (family, size) = resolve_family_size(2, &all);
        assert_eq!(family, Family::Mono);
        assert_eq!(size, Size::Xl);
    }

    #[test]
    fn style_type_size_xl_changes_measure() {
        let (style, diags) = spinnaker::parse("style({ type({ size: xl }) })");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let resolved = spinnaker::resolve_type(&style, &Default::default());
        assert_eq!(resolved.size.as_deref(), Some("xl"));

        let xl = Size::from_scale_token(resolved.size.as_deref().unwrap());
        let sm = {
            let (s, _) = spinnaker::parse("style({ type({ size: sm }) })");
            let r = spinnaker::resolve_type(&s, &Default::default());
            Size::from_scale_token(r.size.as_deref().unwrap())
        };
        assert_ne!(xl, sm);
        assert!(xl.px() > sm.px());
        let t = table();
        let m_xl = &t[&(Family::Body, xl)];
        let m_sm = &t[&(Family::Body, sm)];
        assert!(m_xl.font_size_px > m_sm.font_size_px);
        assert!(m_xl.line_height_px > m_sm.line_height_px);
    }
}
