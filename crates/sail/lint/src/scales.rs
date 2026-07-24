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
//! Values come from the token registry (`tokens.rs`, loaded from generated
//! `token-registry.json` / DTCG model), so a scale can never drift from the theme.

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
            s.extend(channels());
            s.push("0".into());
            s
        }
        "size" => {
            // The space ramp IS the small end of the size scale — an icon is 5
            // steps tall the same way it is 5 steps from its neighbour. Plus the
            // intrinsic keywords, the dimensional families (icon-size/control-h/
            // panel/breakpoint), and the framework's own size + density channels,
            // which a component spends to sit on the platform's density axis.
            let mut s: Vec<String> = tokens::size_values().iter().map(|t| t.to_string()).collect();
            s.extend(tokens::space_values().iter().map(|t| t.to_string()));
            s.extend(channels());
            s.extend(
                ["auto", "full", "screen", "fill", "rail-fill", "fit", "min", "max", "0"]
                    .iter()
                    .map(|t| t.to_string()),
            );
            // A closed, content-relative width family: a FIXED text slot (a
            // rotating-word `<strong>` that must not reflow the line around
            // it) needs a step sized to a character count, not a rem. Same
            // precedent as border-width: a closed token name that lowers to
            // a literal unit — the author never writes a raw `16ch`.
            s.extend(CH.iter().map(|t| format!("ch-{t}")));
            s
        }
        "color" => tokens::color_values().iter().map(|t| t.to_string()).collect(),
        "radius" => {
            let mut s: Vec<String> = tokens::suffixes("radius-").iter().map(|t| t.to_string()).collect();
            s.push("none".into());
            s
        }
        "text" => {
            let mut s: Vec<String> = tokens::text_values().iter().map(|t| t.to_string()).collect();
            s.push("density".into()); // the platform density axis, not a size step
            s.extend(channels());
            s
        }
        "weight" => tokens::suffixes("font-weight-").iter().map(|t| t.to_string()).collect(),
        "font" => vec!["body".into(), "heading".into(), "mono".into(), "display".into()],
        "leading" => tokens::suffixes("leading-").iter().map(|t| t.to_string()).collect(),
        "tracking" => tokens::suffixes("tracking-").iter().map(|t| t.to_string()).collect(),
        "shadow" => {
            let mut s: Vec<String> = tokens::suffixes("shadow-").iter().map(|t| t.to_string()).collect();
            s.push("none".into());
            s
        }
        "z" => {
            // Named elevation tokens (`popover`, `sticky`, …) plus a closed
            // unitless local-stack ramp (TW-shaped `z-1`/`z-2` for joined
            // toggles — not phantom `var(--z-1)`).
            let mut s: Vec<String> =
                tokens::suffixes("z-").iter().map(|t| t.to_string()).collect();
            s.extend(Z_INDEX.iter().map(|t| t.to_string()));
            s
        }
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
            t if is_channel(t) => format!("var(--{t})"),
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
            // The viewport MINUS the chrome the app publishes. A fixed workbench
            // (the tutorial's editor pane) is not expressible with `screen` — it
            // overruns by the height of the navbar.
            "fill" => "calc(100dvh - var(--chrome-offset, 0px))".into(),
            "rail-fill" => "calc(100dvh - var(--rail-offset, 0px))".into(),
            // A dimensional family token is used by its whole name; anything else
            // on this scale is a step of the space ramp.
            t if is_channel(t)
                || t.starts_with("icon-size-")
                || t.starts_with("control-h-")
                || t.starts_with("panel-")
                || t.starts_with("breakpoint-") =>
            {
                format!("var(--{t})")
            }
            t if t.starts_with("ch-") => format!("{}ch", &t["ch-".len()..]),
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
        "text" => match token {
            // The platform's density axis (<html data-density>), not a step of the
            // type scale — a component reads it, it does not re-decide it.
            "density" => "var(--density-text)".into(),
            t if is_channel(t) => format!("var(--{t})"),
            _ => format!("var(--text-{token})"),
        },
        "weight" => format!("var(--font-weight-{token})"),
        "font" => format!("var(--{token}-font)"),
        "leading" => format!("var(--leading-{token})"),
        "tracking" => format!("var(--tracking-{token})"),
        "shadow" => if token == "none" { "none".into() } else { format!("var(--shadow-{token})") },
        "z" => {
            if Z_INDEX.contains(&token) {
                token.into()
            } else {
                format!("var(--z-{token})")
            }
        }
        "duration" => format!("var(--duration-{token})"),
        "ease" => format!("var(--ease-{token})"),
        "border-width" => format!("{token}px"),
        "opacity" | "percent" => format!("{}", token.parse::<f32>().ok()? / 100.0),
        "angle" => format!("{token}deg"),
        _ => return None,
    })
}

/// The FRAMEWORK channels: the component-size channel (`size-px`/`size-fs`/`size-h`,
/// filled by the size prop) and the platform density axis (`density-*`, set at
/// <html>). They are already whole custom-property names, so they lower to their
/// own var rather than a scale-prefixed one — a component SPENDS them; it never
/// re-decides them.
fn channels() -> Vec<String> {
    let mut c: Vec<String> = tokens::suffixes("density-").iter().map(|t| format!("density-{t}")).collect();
    c.extend(["size-px", "size-fs", "size-h"].iter().map(|t| t.to_string()));
    c
}

fn is_channel(t: &str) -> bool {
    t.starts_with("size-") || t.starts_with("density-")
}

/// Border widths. A closed ramp, because a 3px category stripe reads as a
/// category stripe and 1px reads as a border — that distinction is a design
/// decision, and every border in the old sx lowered to 1px.
const BORDER_WIDTH: &[&str] = &["0", "1", "2", "3", "4", "8"];

/// Unitless local z-index steps (not design-token elevation). Closed so a
/// free `z-9999` never sneaks past the vocabulary.
const Z_INDEX: &[&str] = &["0", "1", "2", "3", "4", "5", "10", "20", "30", "40", "50"];

/// Content-relative width steps, in characters. A fixed text slot (e.g. a
/// rotating word that must not reflow its line) sizes itself to the longest
/// word it will ever show — `ch` is the CSS unit built for exactly that, and
/// this is the closed set an author can spend it through.
const CH: &[&str] = &["8", "10", "12", "16", "20", "24"];

/// Opacity / scale steps, finer than the old quarters (a bar chart rests at 85).
const OPACITY: &[&str] = &[
    "0", "5", "10", "15", "20", "25", "30", "40", "50", "60", "70", "75", "80", "85", "90", "95",
    "100", "105", "110",
];

/// Rotation steps. The old sx spoke 0/45/90/180/270, so a 1deg tilt was
/// unsayable and `<sail-band>`'s `angle` prop styled nothing.
const ANGLE: &[&str] = &["0", "1", "2", "3", "6", "12", "45", "90", "180", "270"];

/// The CSS colour keywords sx lets through unsanctioned — they name no design
/// decision, so there is nothing to tokenize.
pub const KEYWORD_COLORS: &[&str] = &["transparent", "currentColor", "inherit", "Canvas", "CanvasText", "Highlight", "HighlightText", "ButtonText", "ButtonFace"];

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
        assert!(contains("angle", "1"), "sail-band's 1deg tilt");
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
    fn ch_widths_close_the_fixed_text_slot_gap() {
        // the landing page's word-cycler: a fixed-width slot sized to the
        // longest word ("Components", 11 chars) so the trailing text never
        // reflows mid-cycle.
        assert!(contains("size", "ch-16"), "closed content-relative width step");
        assert!(!contains("size", "ch-7"), "not every integer — a closed ramp");
        assert_eq!(lower("size", "ch-16").unwrap(), "16ch");
    }

    #[test]
    fn a_raw_value_never_lowers() {
        assert!(lower("space", "16px").is_none());
        assert!(lower("color", "#3366ff").is_none());
    }
}
