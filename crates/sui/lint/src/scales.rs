//! The token scales sx spends.
//!
//! A scale is a **closed value set plus a lowering template**. It is the half of
//! the language where the value IS a design decision, so it is where we are
//! ruthless: `16px` is a compile error, `#3366ff` is a compile error, and
//! `var(--i-invented-this)` is a compile error.
//!
//! Scales live in Rust rather than in the vocabulary table because they are few,
//! stable, and each carries a `var()` template. Utilities live in the table
//! because they are many and they grow.
//!
//! Values come from the token registry (`tokens.rs`, parsed from `tokens.css`),
//! so a scale can never drift from the theme.

use crate::tokens;

/// Is `value` on this scale?
pub fn contains(scale: &str, value: &str) -> bool {
    values(scale).iter().any(|v| v == value)
}

/// Every value on this scale — this is what a diagnostic prints, so the author is
/// told what to SPEND, not merely what is disallowed.
pub fn values(scale: &str) -> Vec<String> {
    let mut v: Vec<String> = match scale {
        "space" => {
            let mut s: Vec<String> = tokens::space_values().iter().map(|t| t.to_string()).collect();
            s.push("0".into());
            s
        }
        "size" => {
            // The space scale doubles as the small end of the size scale, plus the
            // intrinsic keywords and the framework's dimensional families.
            let mut s: Vec<String> = tokens::size_values().iter().map(|t| t.to_string()).collect();
            s.extend(["auto", "full", "screen", "fit", "min", "max", "0"].iter().map(|t| t.to_string()));
            s
        }
        "color" => tokens::color_values().iter().map(|t| t.to_string()).collect(),
        "radius" => {
            let mut s: Vec<String> = tokens::suffixes("radius-").iter().map(|t| t.to_string()).collect();
            s.push("none".into());
            s
        }
        "text" => tokens::text_values().iter().map(|t| t.to_string()).collect(),
        "weight" => tokens::suffixes("font-weight-").iter().map(|t| t.to_string()).collect(),
        "font" => vec!["body".into(), "heading".into(), "mono".into(), "display".into()],
        "leading" => tokens::suffixes("leading-").iter().map(|t| t.to_string()).collect(),
        "tracking" => tokens::suffixes("tracking-").iter().map(|t| t.to_string()).collect(),
        "shadow" => {
            let mut s: Vec<String> = tokens::suffixes("shadow-").iter().map(|t| t.to_string()).collect();
            s.push("none".into());
            s
        }
        "z" => tokens::suffixes("z-").iter().map(|t| t.to_string()).collect(),
        "duration" => tokens::suffixes("duration-").iter().map(|t| t.to_string()).collect(),
        "ease" => tokens::suffixes("ease-").iter().map(|t| t.to_string()).collect(),
        "ring" => {
            let mut s: Vec<String> = tokens::color_values().iter().map(|t| t.to_string()).collect();
            s.push("none".into());
            s
        }
        // Scales with no CSS variable behind them: the value set is still CLOSED,
        // which is all the discipline requires. A ramp with no token is still a
        // ramp; a raw number is not.
        "border-width" => BORDER_WIDTH.iter().map(|t| t.to_string()).collect(),
        "opacity" => OPACITY.iter().map(|t| t.to_string()).collect(),
        "percent" => OPACITY.iter().map(|t| t.to_string()).collect(),
        "angle" => ANGLE.iter().map(|t| t.to_string()).collect(),
        _ => Vec::new(),
    };
    v.sort();
    v.dedup();
    v
}

/// The CSS value this token lowers to on this scale.
pub fn lower(scale: &str, token: &str) -> Option<String> {
    if !contains(scale, token) {
        return None;
    }
    Some(match scale {
        "space" => match token {
            "0" => "0".into(),
            "auto" => "auto".into(),
            _ => format!("var(--space-{token})"),
        },
        "size" => match token {
            "0" => "0".into(),
            "auto" => "auto".into(),
            "full" => "100%".into(),
            "fit" => "fit-content".into(),
            "min" => "min-content".into(),
            "max" => "max-content".into(),
            "screen" => "100dvh".into(),
            // A dimensional family token is used by its whole name; anything else
            // on this scale is a step of the space ramp.
            t if t.starts_with("size-")
                || t.starts_with("icon-size-")
                || t.starts_with("control-h-")
                || t.starts_with("panel-")
                || t.starts_with("breakpoint-") =>
            {
                format!("var(--{t})")
            }
            _ => format!("var(--space-{token})"),
        },
        // A sanctioned CSS keyword colour passes through; everything else is a
        // token or a tone channel.
        "color" | "ring" => {
            if token == "none" {
                "none".into()
            } else if is_keyword_color(token) {
                token.into()
            } else if scale == "ring" {
                format!(
                    "0 0 0 var(--ring-offset) var(--background), \
                     0 0 0 calc(var(--ring-offset) + var(--ring-width)) var(--{token})"
                )
            } else {
                format!("var(--{token})")
            }
        }
        "radius" => if token == "none" { "0".into() } else { format!("var(--radius-{token})") },
        "text" => format!("var(--text-{token})"),
        "weight" => format!("var(--font-weight-{token})"),
        "font" => format!("var(--{token}-font)"),
        "leading" => format!("var(--leading-{token})"),
        "tracking" => format!("var(--tracking-{token})"),
        "shadow" => if token == "none" { "none".into() } else { format!("var(--shadow-{token})") },
        "z" => format!("var(--z-{token})"),
        "duration" => format!("var(--duration-{token})"),
        "ease" => format!("var(--ease-{token})"),
        "border-width" => format!("{token}px"),
        "opacity" | "percent" => format!("{}", token.parse::<f32>().ok()? / 100.0),
        "angle" => format!("{token}deg"),
        _ => return None,
    })
}

/// Border widths. A closed ramp, because a 3px category stripe reads as a
/// category stripe and 1px reads as a border — that distinction is a design
/// decision, and every border in the old sx lowered to 1px.
const BORDER_WIDTH: &[&str] = &["0", "1", "2", "3", "4", "8"];

/// Opacity / scale steps, finer than the old quarters (a bar chart rests at 85).
const OPACITY: &[&str] = &[
    "0", "5", "10", "15", "20", "25", "30", "40", "50", "60", "70", "75", "80", "85", "90", "95",
    "100", "105", "110",
];

/// Rotation steps. The old sx spoke 0/45/90/180/270, so a 1deg tilt was
/// unsayable and `<sui-band>`'s `angle` prop styled nothing.
const ANGLE: &[&str] = &["0", "1", "2", "3", "6", "12", "45", "90", "180", "270"];

/// The CSS colour keywords sx lets through unsanctioned — they name no design
/// decision, so there is nothing to tokenize.
const KEYWORD_COLORS: &[&str] = &["transparent", "currentColor", "inherit"];

pub fn is_keyword_color(token: &str) -> bool {
    KEYWORD_COLORS.contains(&token)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_scale_is_closed() {
        assert!(contains("space", "4"));
        assert!(!contains("space", "16px"));
        assert!(!contains("color", "#3366ff"));
    }

    #[test]
    fn the_scales_that_were_missing_now_exist() {
        // Every one of these cost a real component its design.
        assert!(contains("border-width", "3"), "a 3px category stripe");
        assert!(contains("opacity", "85"), "a bar chart's rest state");
        assert!(contains("angle", "1"), "sui-band's 1deg tilt");
        assert_eq!(lower("border-width", "3").unwrap(), "3px");
        assert_eq!(lower("opacity", "85").unwrap(), "0.85");
        assert_eq!(lower("angle", "1").unwrap(), "1deg");
    }

    #[test]
    fn tokens_lower_to_their_var() {
        assert_eq!(lower("space", "4").unwrap(), "var(--space-4)");
        assert_eq!(lower("color", "surface-2").unwrap(), "var(--surface-2)");
        assert_eq!(lower("radius", "md").unwrap(), "var(--radius-md)");
        assert_eq!(lower("duration", "fast").unwrap(), "var(--duration-fast)");
    }

    #[test]
    fn a_raw_value_never_lowers() {
        assert!(lower("space", "16px").is_none());
        assert!(lower("color", "#3366ff").is_none());
    }
}
