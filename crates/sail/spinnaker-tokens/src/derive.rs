//! Anchor → chromatic scale derivation (solid/hover/subtle/border/text).

use anyhow::{Result, bail};
use serde_json::{Map, Value, json};

use crate::color::{Oklch, contrast_driven_text, parse_color, parse_structured_color};

/// Chromatic roles the engine derives. `danger` is the status name;
/// `destructive` is accepted as an authored alias of the same role.
pub const CHROMATIC_ROLES: &[&str] = &[
    "primary",
    "accent",
    "secondary",
    "success",
    "warning",
    "info",
    "danger",
    "neutral",
];

/// Step semantics every chromatic role answers.
pub const CHROMATIC_STEPS: &[&str] = &["solid", "hover", "subtle", "border", "text"];

#[derive(Debug, Clone)]
pub struct DerivedScale {
    pub role: String,
    pub solid: Oklch,
    pub hover: Oklch,
    pub subtle: Oklch,
    pub border: Oklch,
    pub text: Oklch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaintMode {
    Light,
    Dark,
}

impl PaintMode {
    pub fn parse(s: &str) -> Self {
        if s == "dark" { Self::Dark } else { Self::Light }
    }
}

/// Derive the five steps from an anchor. `bg` tints subtle/border;
/// `fg` pulls hover in light mode.
pub fn derive_scale(
    role: &str,
    anchor: Oklch,
    mode: PaintMode,
    bg: Option<Oklch>,
    fg: Option<Oklch>,
) -> DerivedScale {
    let solid = anchor;
    let hover = match mode {
        PaintMode::Light => {
            let toward = fg.unwrap_or(Oklch::opaque(0.18, 0.0, solid.h));
            solid.mix(toward, 0.12)
        }
        PaintMode::Dark => {
            let toward = fg.unwrap_or(Oklch::opaque(0.98, 0.0, solid.h));
            solid.mix(toward, 0.10)
        }
    };
    let bg = bg.unwrap_or(match mode {
        PaintMode::Light => Oklch::opaque(0.98, 0.0, solid.h),
        PaintMode::Dark => Oklch::opaque(0.16, 0.0, solid.h),
    });
    let subtle = bg.mix(solid, 0.12);
    // Hairline is a surface mix so it does not compete with solid.
    let border = bg.mix(solid, 0.28);
    let text = contrast_driven_text(&solid);
    DerivedScale {
        role: role.to_string(),
        solid,
        hover,
        subtle,
        border,
        text,
    }
}

impl DerivedScale {
    pub fn step(&self, name: &str) -> Option<Oklch> {
        match name {
            "solid" | "default" => Some(self.solid),
            "hover" => Some(self.hover),
            "subtle" => Some(self.subtle),
            "border" => Some(self.border),
            "text" | "foreground" => Some(self.text),
            _ => None,
        }
    }

    pub fn to_dtcg_group(&self) -> Value {
        let step = |c: Oklch| json!({ "$value": c.to_css(), "$type": "color" });
        json!({
            "solid": step(self.solid),
            "hover": step(self.hover),
            "subtle": step(self.subtle),
            "border": step(self.border),
            "text": step(self.text),
        })
    }
}

pub fn parse_anchor(v: &Value) -> Option<Oklch> {
    match v {
        Value::String(s) => parse_color(s),
        Value::Object(obj) => {
            if let Some(inner) = obj.get("$value") {
                return parse_anchor(inner);
            }
            parse_structured_color(v).or_else(|| {
                obj.get("default")
                    .or_else(|| obj.get("solid"))
                    .and_then(parse_anchor)
            })
        }
        _ => None,
    }
}

/// Insert derived chromatic steps into a mode body when the role has an
/// anchor and the step is not already authored.
pub fn fulfill_mode_scales(
    mode_body: &mut Value,
    anchors: &Map<String, Value>,
    mode: PaintMode,
) -> Result<Vec<String>> {
    let bg = pick_surface(mode_body, true);
    let fg = pick_surface(mode_body, false);
    let color = mode_body
        .as_object_mut()
        .and_then(|m| {
            if !m.contains_key("color") {
                m.insert("color".into(), json!({}));
            }
            m.get_mut("color")
        })
        .and_then(|c| c.as_object_mut());
    let Some(color) = color else {
        bail!("derive: mode body must be an object");
    };
    let mut derived = Vec::new();
    for role in CHROMATIC_ROLES {
        let authored_name = if *role == "danger" && !anchors.contains_key(*role) {
            "destructive"
        } else if *role == "accent" && !anchors.contains_key(*role) {
            "brandAccent"
        } else {
            *role
        };
        let Some(anchor_v) = anchors.get(*role).or_else(|| anchors.get(authored_name)) else {
            continue;
        };
        let Some(anchor) = parse_anchor(anchor_v) else {
            bail!("derive: anchor `{role}` is not a parseable color");
        };
        let scale = derive_scale(role, anchor, mode, bg, fg);
        let dest_key = (*role).to_string();
        if !color.contains_key(&dest_key) {
            color.insert(dest_key.clone(), json!({}));
        }
        let group = color
            .get_mut(&dest_key)
            .and_then(|v| v.as_object_mut())
            .ok_or_else(|| anyhow::anyhow!("derive: color.{role} must be an object"))?;
        for step in CHROMATIC_STEPS {
            if group.contains_key(*step) || has_legacy_step(group, step) {
                continue;
            }
            group.insert(
                (*step).to_string(),
                json!({ "$value": scale.step(step).unwrap().to_css(), "$type": "color" }),
            );
            derived.push(format!("color.{role}.{step}"));
        }
    }
    Ok(derived)
}

fn has_legacy_step(group: &Map<String, Value>, step: &str) -> bool {
    match step {
        "solid" => group.contains_key("default"),
        "text" => group.contains_key("foreground"),
        _ => false,
    }
}

fn pick_surface(mode_body: &Value, bg: bool) -> Option<Oklch> {
    let color = mode_body.get("color")?;
    let paths: &[&str] = if bg {
        &["background", "app", "surface"]
    } else {
        &["foreground"]
    };
    for p in paths {
        if let Some(c) = parse_anchor(color.get(*p)?) {
            return Some(c);
        }
    }
    None
}

/// Collect anchors from a theme document: top-level `anchors`, per-mode
/// `anchors.{mode}`, or `modes.{mode}.anchors`.
pub fn anchors_for_mode(doc: &Value, mode: &str) -> Map<String, Value> {
    let mut out = Map::new();
    if let Some(obj) = doc.get("anchors").and_then(|v| v.as_object()) {
        if obj.contains_key("light") || obj.contains_key("dark") {
            if let Some(m) = obj.get(mode).and_then(|v| v.as_object()) {
                merge_anchors(&mut out, m);
            } else if let Some(m) = obj.get("light").and_then(|v| v.as_object()) {
                merge_anchors(&mut out, m);
            }
        } else {
            merge_anchors(&mut out, obj);
        }
    }
    if let Some(m) = doc
        .pointer(&format!("/modes/{mode}/anchors"))
        .and_then(|v| v.as_object())
    {
        merge_anchors(&mut out, m);
    }
    out
}

fn merge_anchors(dst: &mut Map<String, Value>, src: &Map<String, Value>) {
    for (k, v) in src {
        if k.starts_with('$') {
            continue;
        }
        dst.insert(k.clone(), v.clone());
    }
}

/// Apply derivation to every declared mode on a theme document (in place).
pub fn apply_theme_derivation(doc: &mut Value) -> Result<Vec<String>> {
    let mode_names: Vec<String> = doc
        .get("modes")
        .and_then(|v| v.as_object())
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default();
    if mode_names.is_empty() {
        return Ok(Vec::new());
    }
    let mut all = Vec::new();
    for mode in mode_names {
        let anchors = anchors_for_mode(doc, &mode);
        if anchors.is_empty() {
            continue;
        }
        let paint = PaintMode::parse(&mode);
        let Some(body) = doc
            .get_mut("modes")
            .and_then(|v| v.as_object_mut())
            .and_then(|m| m.get_mut(&mode))
        else {
            continue;
        };
        let mut derived = fulfill_mode_scales(body, &anchors, paint)?;
        all.append(&mut derived);
    }
    fulfill_inverse(doc);
    Ok(all)
}

/// Each mode's inverse is the other mode's surface. Single-mode themes
/// fall back to white (dark) or near-black (light).
fn fulfill_inverse(doc: &mut Value) {
    let Some(modes) = doc.get("modes").and_then(|v| v.as_object()) else {
        return;
    };
    let mut surface: std::collections::BTreeMap<String, Value> = std::collections::BTreeMap::new();
    for (name, body) in modes {
        if let Some(s) = body.pointer("/color/surface").cloned() {
            surface.insert(name.clone(), s);
        }
    }
    let names: Vec<String> = modes.keys().cloned().collect();
    let Some(modes_mut) = doc.get_mut("modes").and_then(|v| v.as_object_mut()) else {
        return;
    };
    for name in names {
        let other = if name == "light" { "dark" } else { "light" };
        let fallback = if name == "dark" {
            json!({ "$value": { "space": "oklch", "l": 1, "c": 0, "h": 0 }, "$type": "color" })
        } else {
            json!({ "$value": { "space": "oklch", "l": 0.16, "c": 0, "h": 0 }, "$type": "color" })
        };
        let value = surface.get(other).cloned().unwrap_or(fallback);
        let Some(color) = modes_mut
            .get_mut(&name)
            .and_then(|b| b.as_object_mut())
            .and_then(|b| {
                if !b.contains_key("color") {
                    b.insert("color".into(), json!({}));
                }
                b.get_mut("color")
            })
            .and_then(|c| c.as_object_mut())
        else {
            continue;
        };
        if !color.contains_key("inverse") {
            color.insert("inverse".into(), value);
        }
    }
}

/// Apply derivation to a base document (no `modes` wrapper) using its
/// `color.anchors` or top-level `anchors`.
pub fn apply_base_derivation(doc: &mut Value) -> Result<Vec<String>> {
    let anchors = doc
        .pointer("/color/anchors")
        .and_then(|v| v.as_object())
        .cloned()
        .or_else(|| doc.get("anchors").and_then(|v| v.as_object()).cloned())
        .unwrap_or_default();
    if anchors.is_empty() {
        return Ok(Vec::new());
    }
    fulfill_mode_scales(doc, &anchors, PaintMode::Light)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scale_has_five_steps() {
        let s = derive_scale(
            "primary",
            Oklch::opaque(0.54, 0.18, 245.0),
            PaintMode::Light,
            None,
            None,
        );
        assert!(s.text.l > 0.8);
        assert!((s.hover.l - s.solid.l).abs() > 0.01);
    }

    #[test]
    fn fulfills_missing_steps_keeps_authored() {
        let mut body = json!({
            "color": {
                "primary": {
                    "hover": { "$value": "oklch(0.40 0.18 245)", "$type": "color" }
                }
            }
        });
        let mut anchors = Map::new();
        anchors.insert(
            "primary".into(),
            json!({ "space": "oklch", "l": 0.54, "c": 0.18, "h": 245 }),
        );
        fulfill_mode_scales(&mut body, &anchors, PaintMode::Light).unwrap();
        assert!(body["color"]["primary"]["solid"].is_object());
        assert!(body["color"]["primary"]["text"].is_object());
        assert_eq!(
            body["color"]["primary"]["hover"]["$value"],
            "oklch(0.40 0.18 245)"
        );
    }

    #[test]
    fn inverse_is_the_other_mode_surface() {
        let mut doc = json!({
            "modes": {
                "light": { "color": { "surface": { "$value": "oklch(1 0 0)", "$type": "color" } } },
                "dark": { "color": { "surface": { "$value": "oklch(0.2 0 0)", "$type": "color" } } }
            }
        });
        apply_theme_derivation(&mut doc).unwrap();
        assert_eq!(doc["modes"]["light"]["color"]["inverse"], doc["modes"]["dark"]["color"]["surface"]);
        assert_eq!(doc["modes"]["dark"]["color"]["inverse"], doc["modes"]["light"]["color"]["surface"]);
    }
}
