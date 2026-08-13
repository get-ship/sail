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
    /// Parse a `text-{token}` class. Unknown / missing → `text-base` (the
    /// same default the CSS body rule sets in `tokens.css`).
    pub fn from_class_token(tok: &str) -> Size {
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

/// Resolve the `(family, size)` for a text leaf by walking:
///   1. The element's own `font-*` and `text-*` classes.
///   2. The element's tag — `h1` → 4xl, `h2` → 2xl, etc.
///   3. Ancestors' classes (closest wins).
///   4. Default body / base.
pub fn resolve_family_size(elem_index: usize, all: &[Element]) -> (Family, Size) {
    let mut family: Option<Family> = None;
    let mut size: Option<Size> = None;

    let mut cursor = Some(elem_index);
    while (family.is_none() || size.is_none()) && cursor.is_some() {
        let i = cursor.unwrap();
        let node = &all[i];
        for c in node.class.split_whitespace() {
            if family.is_none() {
                family = family_from_class(c);
            }
            if size.is_none() {
                size = size_from_class(c);
            }
        }
        if size.is_none() {
            size = tag_default_size(&node.tag);
        }
        cursor = node.parent;
    }

    (family.unwrap_or(Family::Body), size.unwrap_or(Size::Base))
}

fn tag_default_size(tag: &str) -> Option<Size> {
    match tag {
        "h1" => Some(Size::Xxxxl),
        "h2" => Some(Size::Xxl),
        "h3" => Some(Size::Xl),
        "h4" => Some(Size::Lg),
        "h5" => Some(Size::Md),
        "h6" => Some(Size::Sm),
        "small" => Some(Size::Sm),
        "code" => Some(Size::Sm),
        _ => None,
    }
}

/// Product `s-fontSize4xl` / legacy `text-4xl`.
pub fn size_from_class(c: &str) -> Option<Size> {
    if let Some(tok) = c.strip_prefix("s-fontSize") {
        return Some(Size::from_class_token(tok));
    }
    if let Some(tok) = c.strip_prefix("text-") {
        // Avoid matching text-align / text-transform utility leftovers.
        if matches!(
            tok,
            "xs" | "sm" | "base" | "md" | "lg" | "xl" | "2xl" | "3xl" | "4xl" | "display"
        ) {
            return Some(Size::from_class_token(tok));
        }
    }
    None
}

/// Product `s-fontFamilydisplay` / legacy `font-mono`.
pub fn family_from_class(c: &str) -> Option<Family> {
    if let Some(tok) = c.strip_prefix("s-fontFamily") {
        return match tok {
            "mono" | "monofont" => Some(Family::Mono),
            "display" | "displayfont" => Some(Family::Display),
            "body" | "bodyfont" => Some(Family::Body),
            "pressstart" | "pressstart2p" => Some(Family::PressStart),
            _ => None,
        };
    }
    match c {
        "font-mono" => Some(Family::Mono),
        "font-display" => Some(Family::Display),
        "font-press-start" => Some(Family::PressStart),
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
    fn resolve_family_size_h1_uses_4xl() {
        let mut all = vec![Element::default()];
        let h1 = Element {
            tag: "h1".to_string(),
            ..Default::default()
        };
        all.push(h1);
        let (family, size) = resolve_family_size(1, &all);
        assert_eq!(family, Family::Body);
        assert_eq!(size, Size::Xxxxl);
    }

    #[test]
    fn resolve_family_size_small_uses_sm() {
        let mut all = vec![Element::default()];
        let small = Element {
            tag: "small".to_string(),
            ..Default::default()
        };
        all.push(small);
        let (_, size) = resolve_family_size(1, &all);
        assert_eq!(size, Size::Sm);
    }

    #[test]
    fn resolve_family_size_mono_overrides_default_family() {
        let mut all = vec![Element::default()];
        let code = Element {
            tag: "code".to_string(),
            class: "font-mono".to_string(),
            ..Default::default()
        };
        all.push(code);
        let (family, _) = resolve_family_size(1, &all);
        assert_eq!(family, Family::Mono);
    }

    #[test]
    fn resolve_family_size_reads_product_sx_font_atoms() {
        let mut all = vec![Element::default()];
        let span = Element {
            tag: "span".to_string(),
            class: "s-fontSize4xl s-fontFamilydisplay".to_string(),
            ..Default::default()
        };
        all.push(span);
        let (family, size) = resolve_family_size(1, &all);
        assert_eq!(family, Family::Display);
        assert_eq!(size, Size::Xxxxl);
    }
}
