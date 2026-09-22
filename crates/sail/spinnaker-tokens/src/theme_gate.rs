//! Theme document gate (SPEC-THEMES-1 §3).
//!
//! A theme JSON is refused unless the envelope is valid, required semantic
//! slots exist in every declared mode, aliases resolve, modes are complete,
//! and named fg/bg pairs meet the WCAG 4.5:1 floor (pair named in the error).

use anyhow::{Result, bail};
use serde_json::Value;

use crate::{
    composite_member_css_name, infer_composite_member_type, is_composite_type, public_name,
    required_composite_members, validate_composite_schema, validate_theme_doc_schema,
};

/// WCAG AA normal-text contrast floor.
pub const CONTRAST_FLOOR: f64 = 4.5;

/// Public names that every declared mode must paint.
pub const REQUIRED_SLOTS: &[&str] = &[
    "background",
    "foreground",
    "primary",
    "primary-foreground",
    "muted",
    "muted-foreground",
    "accent",
    "accent-foreground",
];

/// Named fg/bg pairs checked at [`CONTRAST_FLOOR`].
pub const CONTRAST_PAIRS: &[(&str, &str)] = &[
    ("foreground", "background"),
    ("muted-foreground", "muted"),
    ("muted-foreground", "background"),
    ("primary-foreground", "primary"),
    ("accent-foreground", "accent"),
];

/// Engine contrast pairs for every declared mode (registry metadata shape).
pub fn contrast_reports_json(doc: &Value) -> Vec<Value> {
    let Some(modes) = doc.get("modes").and_then(|v| v.as_object()) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (mode_name, body) in modes {
        let map = paint_map(body);
        let lookup = |name: &str| map.get(name).map(|l| l.value.clone());
        for r in crate::contract::contrast_reports(&lookup, mode_name) {
            let mut j = r.to_json();
            if let Some(obj) = j.as_object_mut() {
                obj.insert("mode".into(), serde_json::json!(mode_name));
            }
            out.push(j);
        }
    }
    out
}

/// Full theme document gate. Schema lock runs first.
pub fn validate_theme_gate(doc: &Value, label: &str) -> Result<()> {
    validate_theme_doc_schema(doc, label)?;
    validate_mode_completeness(doc, label)?;
    let Some(modes) = doc.get("modes").and_then(|v| v.as_object()) else {
        bail!("theme {label}: modes must be an object");
    };
    if modes.is_empty() {
        bail!("theme {label}: modes must declare at least one paint mode");
    }
    for (mode_name, body) in modes {
        validate_required_slots(body, label, mode_name)?;
        validate_composite_members(body, label, mode_name)?;
        validate_aliases(body, label, mode_name)?;
        validate_contrast_floors(body, label, mode_name)?;
        validate_tone_ramp(body, label, mode_name)?;
    }
    validate_path_parity(doc, label)?;
    Ok(())
}

fn declared_mode_names(doc: &Value) -> Result<Vec<String>> {
    let modes = doc
        .get("modes")
        .and_then(|v| v.as_object())
        .ok_or_else(|| anyhow::anyhow!("theme: modes must be an object"))?;
    let supported = doc
        .pointer("/$extensions/sail/supportedModes")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str())
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .filter(|v: &Vec<String>| !v.is_empty());
    if let Some(supported) = supported {
        Ok(supported)
    } else {
        Ok(modes.keys().cloned().collect())
    }
}

fn validate_mode_completeness(doc: &Value, label: &str) -> Result<()> {
    let Some(modes) = doc.get("modes").and_then(|v| v.as_object()) else {
        bail!("theme {label}: modes must be an object");
    };
    let Some(arr) = doc
        .pointer("/$extensions/sail/supportedModes")
        .and_then(|v| v.as_array())
    else {
        return Ok(());
    };
    if arr.is_empty() {
        return Ok(());
    }
    for item in arr {
        let Some(m) = item.as_str() else {
            bail!("theme {label}: supportedModes entries must be strings");
        };
        if !modes.contains_key(m) {
            bail!(
                "theme {label}: missing mode {m:?} — $extensions.sail.supportedModes lists it \
                 but modes.{m} is absent (declared modes: {:?})",
                modes.keys().cloned().collect::<Vec<_>>()
            );
        }
    }
    Ok(())
}

fn validate_path_parity(doc: &Value, label: &str) -> Result<()> {
    let names = declared_mode_names(doc)?;
    if names.len() < 2 {
        return Ok(());
    }
    let modes = doc.get("modes").and_then(|v| v.as_object()).unwrap();
    let sets: Vec<(String, std::collections::BTreeSet<String>)> = names
        .iter()
        .map(|name| {
            let body = modes
                .get(name)
                .cloned()
                .unwrap_or(Value::Object(Default::default()));
            let paths: std::collections::BTreeSet<String> = flatten_parity_paths(&body);
            (name.clone(), paths)
        })
        .collect();
    let reference = &sets[0].1;
    for (name, paths) in sets.iter().skip(1) {
        if paths != reference {
            let only_ref: Vec<_> = reference.difference(paths).cloned().collect();
            let only_this: Vec<_> = paths.difference(reference).cloned().collect();
            bail!(
                "theme {label}: mode path parity required among declared modes \
                 (same $value leaves). mismatch {} vs {name}: \
                 only in {} ({n_r}): {only_ref:?}; only in {name} ({n_t}): {only_this:?}",
                sets[0].0,
                sets[0].0,
                n_r = only_ref.len(),
                n_t = only_this.len(),
            );
        }
    }
    Ok(())
}

struct ColorLeaf {
    path: String,
    css_name: String,
    value: String,
    ty: String,
}

fn flatten_color_leaves(mode_body: &Value) -> Vec<ColorLeaf> {
    let Some(color) = mode_body.get("color") else {
        return Vec::new();
    };
    walk_leaves(color, &["color".into()])
        .into_iter()
        .filter(|l| l.ty == "color")
        .collect()
}

fn walk_leaves(node: &Value, prefix: &[String]) -> Vec<ColorLeaf> {
    let Some(obj) = node.as_object() else {
        return Vec::new();
    };
    if obj.contains_key("$value") {
        let ty = obj
            .get("$type")
            .and_then(|v| v.as_str())
            .unwrap_or("color")
            .to_string();
        if is_composite_type(&ty) && obj.get("$value").is_some_and(Value::is_object) {
            let structured = obj
                .get("$value")
                .and_then(|v| v.as_object())
                .is_some_and(|v| {
                    (ty == "shadow"
                        && (v.contains_key("blur")
                            || v.contains_key("offset")
                            || v.contains_key("offsetX")
                            || v.contains_key("offsetY")))
                        || (ty == "gradient" && (v.contains_key("kind") || v.contains_key("stops")))
                });
            if !structured {
                return expand_composite_leaves(obj, prefix, &ty);
            }
        }
        let value = match obj.get("$value") {
            Some(Value::String(s)) => s.clone(),
            Some(Value::Number(n)) => n.to_string(),
            Some(Value::Bool(b)) => b.to_string(),
            Some(Value::Null) => "null".into(),
            Some(other @ (Value::Object(_) | Value::Array(_))) => {
                let resolve = |name: &str| Some(format!("{{{name}}}"));
                crate::eval::evaluate(other, &resolve)
                    .map(|e| e.css)
                    .unwrap_or_else(|_| other.to_string())
            }
            None => return Vec::new(),
        };
        let path = prefix.join(".");
        return vec![ColorLeaf {
            css_name: public_name(&path),
            path,
            value,
            ty,
        }];
    }
    let mut out = Vec::new();
    for (k, child) in obj {
        if k.starts_with('$') {
            continue;
        }
        if !child.is_object() {
            continue;
        }
        let mut next = prefix.to_vec();
        next.push(k.clone());
        out.extend(walk_leaves(child, &next));
    }
    out
}

fn expand_composite_leaves(
    obj: &serde_json::Map<String, Value>,
    prefix: &[String],
    ty: &str,
) -> Vec<ColorLeaf> {
    let path = prefix.join(".");
    match obj.get("$value") {
        Some(Value::Object(members)) => {
            let mut out = Vec::new();
            for (member, mval) in members {
                if member.starts_with('$') {
                    continue;
                }
                let (mty, mvalue) = match mval {
                    Value::Object(mobj) if mobj.contains_key("$value") => {
                        let mt = mobj
                            .get("$type")
                            .and_then(|v| v.as_str())
                            .unwrap_or_else(|| infer_composite_member_type(ty, member))
                            .to_string();
                        let mv = match mobj.get("$value") {
                            Some(Value::String(s)) => s.clone(),
                            Some(v) => v.to_string(),
                            None => continue,
                        };
                        (mt, mv)
                    }
                    Value::String(s) => (
                        infer_composite_member_type(ty, member).to_string(),
                        s.clone(),
                    ),
                    other => (
                        infer_composite_member_type(ty, member).to_string(),
                        other.to_string(),
                    ),
                };
                let mpath = format!("{path}.{member}");
                out.push(ColorLeaf {
                    css_name: composite_member_css_name(&path, member),
                    path: mpath,
                    value: mvalue,
                    ty: mty,
                });
            }
            out
        }
        Some(Value::String(alias)) => vec![ColorLeaf {
            css_name: crate::composite_public_name(&path),
            path,
            value: alias.clone(),
            ty: ty.to_string(),
        }],
        _ => Vec::new(),
    }
}

/// Required members present; names the missing member when a mode is incomplete.
/// Size pad/space members fail here too (schema error — not an empty required list).
fn validate_composite_members(mode_body: &Value, label: &str, mode: &str) -> Result<()> {
    validate_composite_schema(mode_body, &format!("theme {label} modes.{mode}"))?;
    walk_composites(mode_body, &[], &mut |path, ty, members| {
        let required = required_composite_members(ty);
        if required.is_empty() {
            return Ok(());
        }
        // Whole-composite alias: members come from the target; skip local list.
        let Some(have) = members else {
            return Ok(());
        };
        let mut missing = Vec::new();
        for req in required {
            if !have.iter().any(|h| h == req) {
                missing.push(*req);
            }
        }
        if !missing.is_empty() {
            bail!(
                "theme {label} modes.{mode}: composite {path} missing member {} \
                 (have {have:?}; required {required:?})",
                missing.join(", ")
            );
        }
        Ok(())
    })
}

fn walk_composites(
    node: &Value,
    prefix: &[String],
    visit: &mut impl FnMut(&str, &str, Option<Vec<String>>) -> Result<()>,
) -> Result<()> {
    let Some(obj) = node.as_object() else {
        return Ok(());
    };
    if let Some(ty) = obj.get("$type").and_then(|v| v.as_str()) {
        if is_composite_type(ty) && obj.contains_key("$value") {
            let path = prefix.join(".");
            let members = match obj.get("$value") {
                Some(Value::Object(m)) => Some(
                    m.keys()
                        .filter(|k| !k.starts_with('$'))
                        .cloned()
                        .collect::<Vec<_>>(),
                ),
                Some(Value::String(_)) => None,
                _ => Some(Vec::new()),
            };
            visit(&path, ty, members)?;
            return Ok(());
        }
    }
    if obj.contains_key("$value") {
        return Ok(());
    }
    for (k, child) in obj {
        if k.starts_with('$') || !child.is_object() {
            continue;
        }
        let mut next = prefix.to_vec();
        next.push(k.clone());
        walk_composites(child, &next, visit)?;
    }
    Ok(())
}

/// Color leaves plus every composite member (any type) — mode completeness
/// is per member, not per composite blob.
fn flatten_parity_paths(mode_body: &Value) -> std::collections::BTreeSet<String> {
    let mut set: std::collections::BTreeSet<String> = flatten_color_leaves(mode_body)
        .into_iter()
        .map(|l| l.path)
        .collect();
    for l in flatten_composite_member_leaves(mode_body) {
        set.insert(l.path);
    }
    set
}

fn flatten_composite_member_leaves(mode_body: &Value) -> Vec<ColorLeaf> {
    collect_composite_member_leaves(mode_body, &[])
}

fn collect_composite_member_leaves(node: &Value, prefix: &[String]) -> Vec<ColorLeaf> {
    let Some(obj) = node.as_object() else {
        return Vec::new();
    };
    if let Some(ty) = obj.get("$type").and_then(|v| v.as_str()) {
        if is_composite_type(ty) && obj.get("$value").is_some_and(Value::is_object) {
            return expand_composite_leaves(obj, prefix, ty);
        }
    }
    if obj.contains_key("$value") {
        return Vec::new();
    }
    let mut out = Vec::new();
    for (k, child) in obj {
        if k.starts_with('$') || !child.is_object() {
            continue;
        }
        let mut next = prefix.to_vec();
        next.push(k.clone());
        out.extend(collect_composite_member_leaves(child, &next));
    }
    out
}

fn paint_map(mode_body: &Value) -> std::collections::BTreeMap<String, ColorLeaf> {
    let mut map = std::collections::BTreeMap::new();
    for leaf in flatten_color_leaves(mode_body) {
        if leaf.ty != "color" {
            continue;
        }
        map.insert(leaf.css_name.clone(), leaf);
    }
    map
}

fn validate_required_slots(mode_body: &Value, label: &str, mode: &str) -> Result<()> {
    let map = paint_map(mode_body);
    let mut missing = Vec::new();
    for slot in REQUIRED_SLOTS {
        if !map.contains_key(*slot) {
            missing.push(*slot);
        }
    }
    if !missing.is_empty() {
        let have: Vec<&str> = map.keys().map(String::as_str).collect();
        bail!(
            "theme {label} modes.{mode}: missing required semantic slot(s) {missing:?} \
             (have {have:?}) — every declared mode must fill {}",
            REQUIRED_SLOTS.join(", ")
        );
    }
    Ok(())
}

fn parse_alias(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    if bytes.len() < 3 || bytes[0] != b'{' || bytes[bytes.len() - 1] != b'}' {
        return None;
    }
    let inner = &value[1..value.len() - 1];
    if inner.is_empty() {
        return None;
    }
    if !inner
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'.' || b == b'-')
    {
        return None;
    }
    Some(inner.to_string())
}

fn validate_aliases(mode_body: &Value, label: &str, mode: &str) -> Result<()> {
    let mut leaves = flatten_color_leaves(mode_body);
    leaves.extend(flatten_composite_member_leaves(mode_body));
    let mut paths: std::collections::HashSet<&str> = std::collections::HashSet::new();
    let mut css: std::collections::HashSet<&str> = std::collections::HashSet::new();
    let mut composites: std::collections::HashSet<String> = std::collections::HashSet::new();
    for l in &leaves {
        paths.insert(l.path.as_str());
        css.insert(l.css_name.as_str());
        if let Some((parent, _)) = l.path.rsplit_once('.') {
            // Member path `border.subtle.color` implies composite `border.subtle`.
            if l.path.ends_with(".color")
                || l.path.ends_with(".width")
                || l.path.ends_with(".style")
            {
                composites.insert(parent.to_string());
            }
        }
    }
    for l in &leaves {
        let Some(alias) = parse_alias(&l.value) else {
            continue;
        };
        if paths.contains(alias.as_str())
            || css.contains(alias.as_str())
            || composites.contains(alias.as_str())
        {
            continue;
        }
        bail!(
            "theme {label} modes.{mode}: dangling alias `{{{alias}}}` on {} \
             — target is not a token in this mode",
            l.css_name
        );
    }
    // Cycle: follow alias edges inside this mode.
    let mut edges: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for l in &leaves {
        let Some(alias) = parse_alias(&l.value) else {
            continue;
        };
        let target = if paths.contains(alias.as_str()) {
            alias
        } else if css.contains(alias.as_str()) {
            leaves
                .iter()
                .find(|x| x.css_name == alias)
                .map(|x| x.path.clone())
                .unwrap_or(alias)
        } else {
            continue;
        };
        edges.insert(l.path.clone(), target);
    }
    for start in edges.keys() {
        let mut seen = std::collections::HashSet::new();
        let mut cur = start.as_str();
        while let Some(next) = edges.get(cur) {
            if !seen.insert(cur.to_string()) {
                bail!("theme {label} modes.{mode}: alias cycle involving {start}");
            }
            cur = next.as_str();
        }
    }
    Ok(())
}

fn validate_contrast_floors(mode_body: &Value, label: &str, mode: &str) -> Result<()> {
    let map = paint_map(mode_body);
    for (fg_name, bg_name) in CONTRAST_PAIRS {
        let Some(fg) = map.get(*fg_name) else {
            continue;
        };
        let Some(bg) = map.get(*bg_name) else {
            continue;
        };
        let fg_c = parse_oklch(&fg.value).ok_or_else(|| {
            anyhow::anyhow!(
                "theme {label} modes.{mode}: contrast pair {fg_name} on {bg_name} — \
                 {fg_name} is not parseable oklch (got {:?})",
                fg.value
            )
        })?;
        let bg_c = parse_oklch(&bg.value).ok_or_else(|| {
            anyhow::anyhow!(
                "theme {label} modes.{mode}: contrast pair {fg_name} on {bg_name} — \
                 {bg_name} is not parseable oklch (got {:?})",
                bg.value
            )
        })?;
        if fg_c.alpha < 0.99 || bg_c.alpha < 0.99 {
            bail!(
                "theme {label} modes.{mode}: contrast pair {fg_name} on {bg_name} — \
                 translucent color (alpha {fg_a:.2}/{bg_a:.2}); required pairs must be opaque oklch",
                fg_a = fg_c.alpha,
                bg_a = bg_c.alpha
            );
        }
        let ratio = contrast_ratio(relative_luminance(&fg_c), relative_luminance(&bg_c));
        if ratio + 1e-6 < CONTRAST_FLOOR {
            bail!(
                "theme {label} modes.{mode}: {fg_name} on {bg_name} is {ratio:.2}:1, \
                 floor {CONTRAST_FLOOR:.1}:1"
            );
        }
    }
    validate_composite_color_contrast(mode_body, label, mode, &map)?;
    Ok(())
}

/// Opaque color members of composites are checked against `background`.
/// Translucent catalog borders skip (same alpha rule as named pairs).
fn validate_composite_color_contrast(
    mode_body: &Value,
    label: &str,
    mode: &str,
    map: &std::collections::BTreeMap<String, ColorLeaf>,
) -> Result<()> {
    let Some(bg) = map.get("background") else {
        return Ok(());
    };
    let Some(bg_c) = parse_oklch(&bg.value) else {
        return Ok(());
    };
    if bg_c.alpha < 0.99 {
        return Ok(());
    }
    for leaf in flatten_composite_member_leaves(mode_body) {
        if leaf.ty != "color" {
            continue;
        }
        let Some((parent, member)) = leaf.path.rsplit_once('.') else {
            continue;
        };
        if member != "color" && member != "fg" && member != "foreground" {
            continue;
        }
        let Some(fg_c) = parse_oklch(&leaf.value) else {
            continue;
        };
        if fg_c.alpha < 0.99 {
            continue;
        }
        let ratio = contrast_ratio(relative_luminance(&fg_c), relative_luminance(&bg_c));
        if ratio + 1e-6 < CONTRAST_FLOOR {
            bail!(
                "theme {label} modes.{mode}: {parent}.{member} on background is {ratio:.2}:1, \
                 floor {CONTRAST_FLOOR:.1}:1"
            );
        }
    }
    Ok(())
}

fn validate_tone_ramp(mode_body: &Value, label: &str, mode: &str) -> Result<()> {
    let Some(tone) = mode_body.pointer("/color/tone") else {
        return Ok(());
    };
    let Some(obj) = tone.as_object() else {
        return Ok(());
    };
    for (name, ramp) in obj {
        if name.starts_with('$') {
            continue;
        }
        let Some(ramp_obj) = ramp.as_object() else {
            continue;
        };
        let mut steps: Vec<(String, f64)> = Vec::new();
        for (step, node) in ramp_obj {
            if step.starts_with('$') {
                continue;
            }
            let value = node.get("$value").and_then(|v| v.as_str()).or_else(|| {
                node.get("bg")
                    .and_then(|b| b.get("$value"))
                    .and_then(|v| v.as_str())
            });
            let Some(value) = value else {
                continue;
            };
            let Some(c) = parse_oklch(value) else {
                continue;
            };
            steps.push((step.clone(), c.l));
        }
        if steps.len() < 2 {
            continue;
        }
        steps.sort_by(|a, b| a.0.cmp(&b.0));
        let inc = steps.windows(2).all(|w| w[1].1 + 1e-6 >= w[0].1);
        let dec = steps.windows(2).all(|w| w[1].1 <= w[0].1 + 1e-6);
        if !inc && !dec {
            bail!(
                "theme {label} modes.{mode}: tone ramp {name} is not monotonic in L \
                 (steps {steps:?})"
            );
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct Oklch {
    l: f64,
    c: f64,
    h: f64,
    alpha: f64,
}

fn parse_oklch(raw: &str) -> Option<Oklch> {
    let s = raw.trim();
    let rest = s.strip_prefix("oklch(")?.strip_suffix(')')?.trim();
    // oklch(L C H) or oklch(L C H / A)
    let (body, alpha) = if let Some((lhs, rhs)) = rest.split_once('/') {
        (lhs.trim(), parse_alpha(rhs.trim())?)
    } else {
        (rest, 1.0)
    };
    let parts: Vec<&str> = body.split_whitespace().collect();
    if parts.len() < 3 {
        return None;
    }
    let l = parse_l(parts[0])?;
    let c: f64 = parts[1].parse().ok()?;
    let h = parse_hue(parts[2..].join(" ").trim())?;
    Some(Oklch { l, c, h, alpha })
}

fn parse_l(s: &str) -> Option<f64> {
    if let Some(pct) = s.strip_suffix('%') {
        return Some(pct.parse::<f64>().ok()? / 100.0);
    }
    s.parse().ok()
}

fn parse_alpha(s: &str) -> Option<f64> {
    if let Some(pct) = s.strip_suffix('%') {
        return Some(pct.parse::<f64>().ok()? / 100.0);
    }
    s.parse().ok()
}

fn parse_hue(s: &str) -> Option<f64> {
    if let Ok(h) = s.parse::<f64>() {
        return Some(h);
    }
    // var(--brand-hue, 30) or var(--brand-hue, 30))
    let inner = s.strip_prefix("var(")?.trim_end_matches(')').trim();
    let fallback = inner.split(',').nth(1)?.trim();
    fallback.parse().ok()
}

fn relative_luminance(c: &Oklch) -> f64 {
    let (r, g, b) = oklch_to_linear_srgb(c.l, c.c, c.h);
    0.2126 * r.max(0.0) + 0.7152 * g.max(0.0) + 0.0722 * b.max(0.0)
}

fn oklch_to_linear_srgb(l: f64, c: f64, h_deg: f64) -> (f64, f64, f64) {
    let h = h_deg.to_radians();
    let a = c * h.cos();
    let b = c * h.sin();
    let l_ = l + 0.3963377774 * a + 0.2158037573 * b;
    let m_ = l - 0.1055613458 * a - 0.0638541728 * b;
    let s_ = l - 0.0894841775 * a - 1.2914855480 * b;
    let l3 = l_ * l_ * l_;
    let m3 = m_ * m_ * m_;
    let s3 = s_ * s_ * s_;
    let r = 4.0767416621 * l3 - 3.3077115913 * m3 + 0.2309699292 * s3;
    let g = -1.2684380046 * l3 + 2.6097574011 * m3 - 0.3413193965 * s3;
    let bch = -0.0041960863 * l3 - 0.7034186147 * m3 + 1.7076147010 * s3;
    (r, g, bch)
}

fn contrast_ratio(l1: f64, l2: f64) -> f64 {
    let (hi, lo) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
    (hi + 0.05) / (lo + 0.05)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn mode_color(extra: serde_json::Map<String, Value>) -> Value {
        let mut color = serde_json::Map::new();
        color.insert(
            "background".into(),
            json!({ "$value": "oklch(0.98 0 0)", "$type": "color" }),
        );
        color.insert(
            "foreground".into(),
            json!({ "$value": "oklch(0.20 0 0)", "$type": "color" }),
        );
        color.insert(
            "primary".into(),
            json!({
                "default": { "$value": "oklch(0.45 0.12 250)", "$type": "color" },
                "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
            }),
        );
        color.insert(
            "muted".into(),
            json!({
                "default": { "$value": "oklch(0.92 0 0)", "$type": "color" },
                "foreground": { "$value": "oklch(0.35 0 0)", "$type": "color" }
            }),
        );
        color.insert(
            "accent".into(),
            json!({
                "default": { "$value": "oklch(0.45 0.16 300)", "$type": "color" },
                "foreground": { "$value": "oklch(0.98 0 0)", "$type": "color" }
            }),
        );
        for (k, v) in extra {
            color.insert(k, v);
        }
        json!({
            "$extensions": { "sail": { "layer": "theme", "themeId": "probe", "defaultMode": "light" } },
            "modes": { "light": { "color": color } }
        })
    }

    #[test]
    fn happy_minimal_theme_passes() {
        validate_theme_gate(&mode_color(serde_json::Map::new()), "probe").unwrap();
    }

    #[test]
    fn missing_foreground_names_slot() {
        let mut doc = mode_color(serde_json::Map::new());
        doc["modes"]["light"]["color"]
            .as_object_mut()
            .unwrap()
            .remove("foreground");
        let err = validate_theme_gate(&doc, "probe").unwrap_err().to_string();
        assert!(err.contains("foreground"), "{err}");
        assert!(err.contains("missing required"), "{err}");
    }

    #[test]
    fn dangling_alias_is_named() {
        let mut extra = serde_json::Map::new();
        extra.insert(
            "secondary".into(),
            json!({ "$value": "{no-such-token}", "$type": "color" }),
        );
        let err = validate_theme_gate(&mode_color(extra), "probe")
            .unwrap_err()
            .to_string();
        assert!(err.contains("dangling alias"), "{err}");
        assert!(err.contains("no-such-token"), "{err}");
    }

    #[test]
    fn missing_supported_mode_is_named() {
        let mut doc = mode_color(serde_json::Map::new());
        doc["$extensions"]["sail"]["supportedModes"] = json!(["light", "dark"]);
        let err = validate_theme_gate(&doc, "probe").unwrap_err().to_string();
        assert!(
            err.contains("missing mode \"dark\"") || err.contains("missing mode 'dark'"),
            "{err}"
        );
    }

    #[test]
    fn contrast_floor_names_pair_and_ratios() {
        let mut extra = serde_json::Map::new();
        extra.insert(
            "muted".into(),
            json!({
                "default": { "$value": "oklch(0.92 0 0)", "$type": "color" },
                "foreground": { "$value": "oklch(0.85 0 0)", "$type": "color" }
            }),
        );
        let err = validate_theme_gate(&mode_color(extra), "probe")
            .unwrap_err()
            .to_string();
        assert!(err.contains("muted-foreground"), "{err}");
        assert!(err.contains("muted"), "{err}");
        assert!(err.contains("floor 4.5:1"), "{err}");
        assert!(err.contains(":1"), "{err}");
    }

    fn border_composite(color: &str) -> Value {
        json!({
            "$type": "border",
            "$value": {
                "color": { "$value": color, "$type": "color" },
                "width": { "$value": "1px", "$type": "dimension" },
                "style": { "$value": "solid", "$type": "string" }
            }
        })
    }

    #[test]
    fn composite_dangling_alias_member_is_named() {
        let mut doc = mode_color(serde_json::Map::new());
        doc["modes"]["light"]["border"] = json!({
            "subtle": {
                "$type": "border",
                "$value": {
                    "color": "{no-such-member}",
                    "width": "1px",
                    "style": "solid"
                }
            }
        });
        let err = validate_theme_gate(&doc, "probe").unwrap_err().to_string();
        assert!(err.contains("dangling alias"), "{err}");
        assert!(err.contains("no-such-member"), "{err}");
        assert!(
            err.contains("border-subtle-color") || err.contains("border.subtle"),
            "{err}"
        );
    }

    #[test]
    fn composite_missing_member_in_mode_is_named() {
        let mut doc = mode_color(serde_json::Map::new());
        doc["$extensions"]["sail"]["supportedModes"] = json!(["light", "dark"]);
        doc["modes"]["light"]["border"] = json!({
            "subtle": border_composite("oklch(0.20 0 0)")
        });
        doc["modes"]["dark"] = doc["modes"]["light"].clone();
        doc["modes"]["dark"]["$extensions"] = json!({ "sail": { "dark": true } });
        doc["modes"]["dark"]["border"]["subtle"]["$value"]
            .as_object_mut()
            .unwrap()
            .remove("width");
        let err = validate_theme_gate(&doc, "probe").unwrap_err().to_string();
        assert!(
            err.contains("missing member") && err.contains("width"),
            "{err}"
        );
        assert!(err.contains("border.subtle"), "{err}");
    }

    #[test]
    fn composite_contrast_fail_names_the_pair() {
        let mut doc = mode_color(serde_json::Map::new());
        doc["modes"]["light"]["border"] = json!({
            "subtle": border_composite("oklch(0.90 0 0)")
        });
        let err = validate_theme_gate(&doc, "probe").unwrap_err().to_string();
        assert!(err.contains("border.subtle.color"), "{err}");
        assert!(err.contains("background"), "{err}");
        assert!(err.contains("floor 4.5:1"), "{err}");
        assert!(err.contains(":1"), "{err}");
    }

    #[test]
    fn composite_happy_translucent_border_skips_contrast() {
        let mut doc = mode_color(serde_json::Map::new());
        doc["modes"]["light"]["border"] = json!({
            "subtle": border_composite("oklch(0.20 0 0 / 25%)")
        });
        validate_theme_gate(&doc, "probe").unwrap();
    }

    #[test]
    fn size_composite_with_pad_is_schema_error() {
        let mut doc = mode_color(serde_json::Map::new());
        doc["modes"]["light"]["size"] = json!({
            "control-md": {
                "$type": "size",
                "$value": { "h": "34px", "pad": "8px" }
            }
        });
        let err = validate_theme_gate(&doc, "probe").unwrap_err().to_string();
        assert!(
            err.contains("schema error") && err.contains("pad") && err.contains("size.control-md"),
            "expected named size-pad schema error, got: {err}"
        );
    }

    #[test]
    fn catalog_themes_pass_gate() {
        // CARGO_MANIFEST_DIR = crates/sail/compiler
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../packages/sail/tokens/model/themes");
        assert!(
            dir.is_dir(),
            "catalog themes dir missing: {}",
            dir.display()
        );
        for entry in std::fs::read_dir(&dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let text = std::fs::read_to_string(&path).unwrap();
            let mut doc: Value = serde_json::from_str(&text).unwrap();
            let label = path.file_name().unwrap().to_string_lossy();
            crate::derive::apply_theme_derivation(&mut doc).unwrap();
            validate_theme_gate(&doc, &label).unwrap_or_else(|e| {
                panic!("{}: {e}", path.display());
            });
        }
    }
}
