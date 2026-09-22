//! Space ramp generation from base unit N + granularity rule.

use serde_json::{Map, Value, json};

/// Default base unit (px). Theme-fulfillable.
pub const DEFAULT_N: f64 = 4.0;

/// Integer steps: 1–12 then 14–64 (skip 13; jump by 2 then 4).
pub const SPACE_STEPS: &[u32] = &[
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 14, 16, 20, 24, 28, 32, 36, 40, 44, 48, 52, 56, 60, 64,
];

/// Named half-steps (ratified). 0_75 / 1_25 / 3_5 / 4_5 stay dead.
pub const SPACE_FRACTIONALS: &[(&str, f64)] = &[("1_5", 1.5), ("2_5", 2.5)];

/// Absolute hairlines — not N-scaled.
pub const SPACE_ABSOLUTES: &[(&str, f64)] = &[("px", 1.0), ("half", 2.0)];

#[derive(Debug, Clone, PartialEq)]
pub struct SpaceRule {
    pub n: f64,
    pub fractionals: Vec<(String, f64)>,
}

impl Default for SpaceRule {
    fn default() -> Self {
        Self {
            n: DEFAULT_N,
            fractionals: SPACE_FRACTIONALS
                .iter()
                .map(|(k, v)| ((*k).to_string(), *v))
                .collect(),
        }
    }
}

impl SpaceRule {
    pub fn from_doc(doc: &Value) -> Self {
        let mut rule = Self::default();
        if let Some(n) = doc
            .pointer("/space/$extensions/sail/n")
            .or_else(|| doc.pointer("/$extensions/sail/spaceN"))
            .and_then(|v| v.as_f64().or_else(|| v.as_u64().map(|n| n as f64)))
        {
            if n > 0.0 {
                rule.n = n;
            }
        }
        if let Some(arr) = doc
            .pointer("/space/$extensions/sail/fractionals")
            .and_then(|v| v.as_array())
        {
            let mut frac = Vec::new();
            for item in arr {
                if let Some(s) = item.as_str() {
                    if let Some((_, v)) = SPACE_FRACTIONALS.iter().find(|(k, _)| *k == s) {
                        frac.push((s.to_string(), *v));
                    }
                }
            }
            if !frac.is_empty() {
                rule.fractionals = frac;
            }
        }
        rule
    }

    pub fn px_for_step(&self, step: u32) -> f64 {
        f64::from(step) * self.n
    }

    pub fn px_for_fractional(&self, factor: f64) -> f64 {
        factor * self.n
    }
}

/// `(name, px)` for the full generated ramp.
pub fn generate_ramp(rule: &SpaceRule) -> Vec<(String, f64)> {
    let mut out = Vec::new();
    for (name, px) in SPACE_ABSOLUTES {
        out.push(((*name).to_string(), *px));
    }
    for (name, factor) in &rule.fractionals {
        out.push((name.clone(), rule.px_for_fractional(*factor)));
    }
    for step in SPACE_STEPS {
        out.push((step.to_string(), rule.px_for_step(*step)));
    }
    out
}

/// Merge generated space leaves into `doc.space` without overwriting an
/// authored `$value`. Returns the names that were inserted.
pub fn apply_space_ramp(doc: &mut Value) -> Vec<String> {
    let rule = SpaceRule::from_doc(doc);
    let Some(root) = doc.as_object_mut() else {
        return Vec::new();
    };
    if !root.contains_key("space") {
        // No space group and no N extension — leave fixtures that omit space.
        if doc_n_absent_in_extensions(root) && !space_rule_present(root) {
            return Vec::new();
        }
        root.insert("space".into(), json!({ "$type": "dimension" }));
    }
    let Some(space) = root.get_mut("space").and_then(|v| v.as_object_mut()) else {
        return Vec::new();
    };
    // A fully authored ramp (legacy) is left alone unless N is declared.
    let n_declared = space
        .get("$extensions")
        .and_then(|v| v.pointer("/sail/n"))
        .is_some();
    let already_has_steps = space.contains_key("1") && space.contains_key("12");
    if already_has_steps && !n_declared {
        return Vec::new();
    }
    space.insert("$type".into(), json!("dimension"));
    let mut inserted = Vec::new();
    for (name, px) in generate_ramp(&rule) {
        if let Some(existing) = space.get(&name) {
            if existing.get("$value").is_some() {
                continue;
            }
        }
        let css = if (px - px.round()).abs() < 1e-9 {
            format!("{}px", px.round() as i64)
        } else {
            format!("{px:.2}px")
        };
        space.insert(name.clone(), json!({ "$value": css, "$type": "dimension" }));
        inserted.push(format!("space.{name}"));
    }
    inserted
}

fn space_rule_present(root: &Map<String, Value>) -> bool {
    root.get("$extensions")
        .and_then(|v| v.pointer("/sail/spaceN"))
        .is_some()
}

fn doc_n_absent_in_extensions(root: &Map<String, Value>) -> bool {
    !space_rule_present(root)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_ramp_matches_legacy_values() {
        let ramp = generate_ramp(&SpaceRule::default());
        let get = |n: &str| ramp.iter().find(|(k, _)| k == n).map(|(_, v)| *v).unwrap();
        assert_eq!(get("1"), 4.0);
        assert_eq!(get("12"), 48.0);
        assert_eq!(get("14"), 56.0);
        assert_eq!(get("64"), 256.0);
        assert_eq!(get("1_5"), 6.0);
        assert_eq!(get("2_5"), 10.0);
        assert_eq!(get("px"), 1.0);
        assert_eq!(get("half"), 2.0);
        assert!(!ramp.iter().any(|(k, _)| k == "0_75" || k == "3_5"));
    }

    #[test]
    fn n_scales_the_ramp() {
        let rule = SpaceRule {
            n: 8.0,
            ..SpaceRule::default()
        };
        let ramp = generate_ramp(&rule);
        let one = ramp.iter().find(|(k, _)| k == "1").unwrap().1;
        assert_eq!(one, 8.0);
        let half = ramp.iter().find(|(k, _)| k == "half").unwrap().1;
        assert_eq!(half, 2.0);
    }

    #[test]
    fn apply_does_not_overwrite_authored() {
        let mut doc = json!({
            "space": {
                "$extensions": { "sail": { "n": 4 } },
                "1": { "$value": "5px", "$type": "dimension" }
            }
        });
        apply_space_ramp(&mut doc);
        assert_eq!(doc["space"]["1"]["$value"], "5px");
        assert_eq!(doc["space"]["2"]["$value"], "8px");
    }
}
