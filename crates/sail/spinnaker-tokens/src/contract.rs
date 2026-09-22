//! Floor lists + composite-shape contract. Base owns the contract; themes fulfill.

use anyhow::{Result, bail};
use serde_json::Value;

use crate::color::{CONTRAST_FLOOR, Oklch, parse_color, require_contrast};
use crate::derive::{CHROMATIC_ROLES, CHROMATIC_STEPS};

/// Hard-error contrast pairs (public css names).
pub const FLOOR_PAIRS: &[(&str, &str)] = &[
    ("foreground", "background"),
    ("primary-foreground", "primary"),
    ("accent-foreground", "accent"),
];

/// Soft-warn contrast pairs.
pub const SOFT_PAIRS: &[(&str, &str)] = &[
    ("muted-foreground", "muted"),
    ("muted-foreground", "background"),
];

/// Chromatic roles the floor requires as anchors or solid steps.
pub fn chromatic_floor() -> &'static [&'static str] {
    CHROMATIC_ROLES
}

pub fn chromatic_steps() -> &'static [&'static str] {
    CHROMATIC_STEPS
}

/// Neutral / bg purpose steps (ruled).
pub const NEUTRAL_STEPS: &[&str] = &["app", "subtle", "surface", "inset", "overlay"];

/// Type molecules (8).
pub const TYPE_MOLECULES: &[&str] = &[
    "body",
    "body-sm",
    "label",
    "heading",
    "mono",
    "mono-sm",
    "display",
    "display-sm",
];

/// Motion durations that stay (glacial is dead).
pub const MOTION_DURATIONS: &[&str] = &["instant", "fast", "medium", "slow", "deliberate"];

/// Universal keyframe floor (eq/bob/pulse/rise dead).
pub const KEYFRAME_FLOOR: &[&str] = &[
    "fade",
    "fade-out",
    "fade-scale-in",
    "slide-up",
    "spin",
    "press",
];

/// Validate a base document against the ruled floor. Missing required
/// chromatic anchors (when the new shape is in use) is a hard error.
/// Legacy documents without `anchors` skip the new-shape floor.
pub fn validate_base_contract(doc: &Value, label: &str) -> Result<()> {
    let using_anchors = doc.get("anchors").is_some() || doc.pointer("/color/anchors").is_some();
    if !using_anchors {
        return Ok(());
    }
    let anchors = doc
        .pointer("/color/anchors")
        .or_else(|| doc.get("anchors"))
        .and_then(|v| v.as_object());
    let Some(anchors) = anchors else {
        bail!("{label}: anchors must be an object");
    };
    // Mode-split anchors (light/dark) — check the default/light map.
    let map = if anchors.contains_key("light") || anchors.contains_key("dark") {
        anchors
            .get("light")
            .or_else(|| anchors.get("dark"))
            .and_then(|v| v.as_object())
            .ok_or_else(|| anyhow::anyhow!("{label}: anchors.light/dark must be objects"))?
    } else {
        anchors
    };
    let mut missing = Vec::new();
    for role in CHROMATIC_ROLES {
        let alt = match *role {
            "danger" => "destructive",
            _ => *role,
        };
        if !map.contains_key(*role) && !map.contains_key(alt) {
            missing.push(*role);
        }
    }
    if !missing.is_empty() {
        bail!(
            "{label}: missing chromatic anchor(s) {missing:?} — floor is {}",
            CHROMATIC_ROLES.join(", ")
        );
    }
    Ok(())
}

/// Contrast reports (never fails). Use [`enforce_floor_pairs`] to hard-error.
pub fn contrast_reports(
    lookup: &dyn Fn(&str) -> Option<String>,
    label: &str,
) -> Vec<ContrastReport> {
    let mut reports = Vec::new();
    for (fg_name, bg_name) in FLOOR_PAIRS {
        match pair_colors(lookup, fg_name, bg_name) {
            PairLookup::Both(fg, bg) => {
                let ratio = crate::color::contrast_pair(&fg, &bg);
                let ok = ratio + 1e-6 >= CONTRAST_FLOOR;
                reports.push(ContrastReport {
                    fg: (*fg_name).into(),
                    bg: (*bg_name).into(),
                    ratio,
                    floor: CONTRAST_FLOOR,
                    ok,
                    soft: false,
                });
            }
            PairLookup::Missing | PairLookup::Unparseable(_, _) => {}
        }
    }
    for (fg_name, bg_name) in SOFT_PAIRS {
        if let PairLookup::Both(fg, bg) = pair_colors(lookup, fg_name, bg_name) {
            let ratio = crate::color::contrast_pair(&fg, &bg);
            let ok = ratio + 1e-6 >= CONTRAST_FLOOR;
            if !ok {
                eprintln!(
                    "warn: {label}: {fg_name} on {bg_name} is {ratio:.2}:1, soft floor {CONTRAST_FLOOR:.1}:1"
                );
            }
            reports.push(ContrastReport {
                fg: (*fg_name).into(),
                bg: (*bg_name).into(),
                ratio,
                floor: CONTRAST_FLOOR,
                ok,
                soft: true,
            });
        }
    }
    reports
}

/// Contrast gate: floor pairs hard-error, soft pairs warn.
/// `lookup` maps a public css name to a parseable oklch string.
pub fn contrast_gate(
    lookup: &dyn Fn(&str) -> Option<String>,
    label: &str,
) -> Result<Vec<ContrastReport>> {
    let reports = contrast_reports(lookup, label);
    enforce_floor_pairs(&reports, lookup, label)?;
    Ok(reports)
}

/// Hard-error any floor pair that failed or is unparseable when present.
pub fn enforce_floor_pairs(
    reports: &[ContrastReport],
    lookup: &dyn Fn(&str) -> Option<String>,
    label: &str,
) -> Result<()> {
    for (fg_name, bg_name) in FLOOR_PAIRS {
        match pair_colors(lookup, fg_name, bg_name) {
            PairLookup::Both(fg, bg) => {
                require_contrast(&fg, &bg, fg_name, bg_name, CONTRAST_FLOOR, label)?;
            }
            PairLookup::Unparseable(name, raw) => {
                bail!(
                    "{label}: contrast pair {fg_name} on {bg_name} — {name} is not parseable oklch (got {raw:?})"
                );
            }
            PairLookup::Missing => {}
        }
    }
    let _ = reports;
    Ok(())
}

enum PairLookup {
    Both(Oklch, Oklch),
    Missing,
    Unparseable(String, String),
}

fn pair_colors(
    lookup: &dyn Fn(&str) -> Option<String>,
    fg_name: &str,
    bg_name: &str,
) -> PairLookup {
    let Some(fg_raw) = lookup(fg_name) else {
        return PairLookup::Missing;
    };
    let Some(bg_raw) = lookup(bg_name) else {
        return PairLookup::Missing;
    };
    let Some(fg) = parse_color(&fg_raw) else {
        return PairLookup::Unparseable(fg_name.into(), fg_raw);
    };
    let Some(bg) = parse_color(&bg_raw) else {
        return PairLookup::Unparseable(bg_name.into(), bg_raw);
    };
    PairLookup::Both(fg, bg)
}

#[derive(Debug, Clone)]
pub struct ContrastReport {
    pub fg: String,
    pub bg: String,
    pub ratio: f64,
    pub floor: f64,
    pub ok: bool,
    pub soft: bool,
}

impl ContrastReport {
    pub fn to_json(&self) -> Value {
        serde_json::json!({
            "fg": self.fg,
            "bg": self.bg,
            "ratio": (self.ratio * 100.0).round() / 100.0,
            "floor": self.floor,
            "ok": self.ok,
            "soft": self.soft,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_anchor_is_named() {
        let doc = serde_json::json!({
            "anchors": { "primary": { "space": "oklch", "l": 0.5, "c": 0.1, "h": 40 } }
        });
        let err = validate_base_contract(&doc, "base")
            .unwrap_err()
            .to_string();
        assert!(err.contains("missing chromatic anchor"));
        assert!(err.contains("accent"));
    }

    #[test]
    fn legacy_without_anchors_skips() {
        validate_base_contract(&serde_json::json!({ "color": {} }), "base").unwrap();
    }

    #[test]
    fn floor_pair_hard_errors() {
        let lookup = |n: &str| match n {
            "foreground" => Some("oklch(0.90 0 0)".into()),
            "background" => Some("oklch(0.95 0 0)".into()),
            _ => None,
        };
        let err = contrast_gate(&lookup, "probe").unwrap_err().to_string();
        assert!(err.contains("foreground"));
        assert!(err.contains("background"));
        assert!(err.contains("floor 4.5:1"));
    }
}
