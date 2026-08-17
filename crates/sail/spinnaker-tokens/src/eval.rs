//! Structured-value evaluation: `{ref}`, `{op: mix}`, gradients, fluid, breakpoint-keyed.

use std::collections::BTreeMap;

use anyhow::{Result, bail};
use serde_json::Value;

use crate::color::{parse_color, parse_structured_color};

/// Resolved CSS plus optional per-breakpoint values.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct EvalResult {
    pub css: String,
    pub breakpoints: BTreeMap<String, String>,
}

impl EvalResult {
    pub fn scalar(css: impl Into<String>) -> Self {
        Self {
            css: css.into(),
            breakpoints: BTreeMap::new(),
        }
    }
}

pub type RefResolver<'a> = dyn Fn(&str) -> Option<String> + 'a;

/// Evaluate a DTCG `$value` (string, number, or structured object) to CSS.
pub fn evaluate(value: &Value, resolve: &RefResolver<'_>) -> Result<EvalResult> {
    match value {
        Value::String(s) => evaluate_string(s, resolve),
        Value::Number(n) => Ok(EvalResult::scalar(n.to_string())),
        Value::Bool(b) => Ok(EvalResult::scalar(b.to_string())),
        Value::Null => Ok(EvalResult::scalar("null")),
        Value::Array(items) => {
            // Multi-layer shadow: [{offset, blur, ...}, ...]
            if !items.is_empty()
                && items.iter().all(|i| {
                    i.get("blur").is_some()
                        || i.get("offset").is_some()
                        || i.get("offsetX").is_some()
                        || i.get("offsetY").is_some()
                })
            {
                let mut parts = Vec::new();
                for item in items {
                    parts.push(evaluate(item, resolve)?.css);
                }
                return Ok(EvalResult::scalar(parts.join(", ")));
            }
            bail!(
                "structured value: bare array is not a token value ({} items) — use {{kind, stops}} or a shadow layer list",
                items.len()
            )
        }
        Value::Object(obj) => evaluate_object(obj, resolve),
    }
}

fn evaluate_string(s: &str, resolve: &RefResolver<'_>) -> Result<EvalResult> {
    if let Some(inner) = parse_ref_token(s) {
        let css = resolve_ref(&inner, resolve)?;
        return Ok(EvalResult::scalar(css));
    }
    Ok(EvalResult::scalar(s.to_string()))
}

fn parse_ref_token(s: &str) -> Option<String> {
    let bytes = s.as_bytes();
    if bytes.len() < 3 || bytes[0] != b'{' || bytes[bytes.len() - 1] != b'}' {
        return None;
    }
    let inner = &s[1..s.len() - 1];
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

fn resolve_ref(name: &str, resolve: &RefResolver<'_>) -> Result<String> {
    resolve(name)
        .map(|css| {
            if css.starts_with("var(--") || css.starts_with('#') || css.starts_with("oklch(") {
                css
            } else if css.starts_with("--") {
                format!("var({css})")
            } else {
                format!("var(--{css})")
            }
        })
        .ok_or_else(|| {
            anyhow::anyhow!("dangling ref `{{{name}}}` — target identity does not exist")
        })
}

fn evaluate_object(
    obj: &serde_json::Map<String, Value>,
    resolve: &RefResolver<'_>,
) -> Result<EvalResult> {
    if obj.contains_key("ref") {
        let Some(name) = obj.get("ref").and_then(|v| v.as_str()) else {
            bail!("structured value: {{ref}} must be a string");
        };
        return Ok(EvalResult::scalar(resolve_ref(name, resolve)?));
    }
    if let Some(op) = obj.get("op").and_then(|v| v.as_str()) {
        return evaluate_op(op, obj, resolve);
    }
    if obj.contains_key("space") || obj.contains_key("hex") {
        let Some(c) = parse_structured_color(&Value::Object(obj.clone())) else {
            bail!("structured value: color object is not a parseable oklch/hex");
        };
        return Ok(EvalResult::scalar(c.to_css()));
    }
    if obj.contains_key("kind") && obj.contains_key("stops") {
        return evaluate_gradient(obj, resolve);
    }
    if obj.contains_key("fluid")
        || (obj.contains_key("min") && obj.contains_key("pref") && obj.contains_key("max"))
    {
        return evaluate_fluid(obj, resolve);
    }
    if obj.contains_key("offset")
        || obj.contains_key("offsetX")
        || obj.contains_key("blur")
        || obj.contains_key("spread")
    {
        return evaluate_shadow(obj, resolve);
    }
    if is_breakpoint_map(obj) {
        return evaluate_breakpoints(obj, resolve);
    }
    bail!(
        "structured value: unrecognized object keys {:?} — expected ref / op / color / gradient / fluid / shadow / breakpoint map",
        obj.keys().cloned().collect::<Vec<_>>()
    )
}

fn is_breakpoint_map(obj: &serde_json::Map<String, Value>) -> bool {
    const KEYS: &[&str] = &["base", "mobile", "tablet", "desktop"];
    obj.keys().any(|k| KEYS.contains(&k.as_str()))
        && obj
            .keys()
            .all(|k| KEYS.contains(&k.as_str()) || k.starts_with('$'))
}

fn evaluate_breakpoints(
    obj: &serde_json::Map<String, Value>,
    resolve: &RefResolver<'_>,
) -> Result<EvalResult> {
    let mut bp = BTreeMap::new();
    for (k, v) in obj {
        if k.starts_with('$') {
            continue;
        }
        let ev = evaluate(v, resolve)?;
        bp.insert(k.clone(), ev.css);
    }
    let css = bp
        .get("base")
        .or_else(|| bp.get("mobile"))
        .cloned()
        .or_else(|| bp.values().next().cloned())
        .unwrap_or_default();
    Ok(EvalResult {
        css,
        breakpoints: bp,
    })
}

fn evaluate_op(
    op: &str,
    obj: &serde_json::Map<String, Value>,
    resolve: &RefResolver<'_>,
) -> Result<EvalResult> {
    match op {
        "mix" => evaluate_mix(obj, resolve),
        other => bail!("structured value: unknown op {other:?}"),
    }
}

fn evaluate_mix(
    obj: &serde_json::Map<String, Value>,
    resolve: &RefResolver<'_>,
) -> Result<EvalResult> {
    let amount = obj
        .get("amount")
        .or_else(|| obj.get("pct"))
        .and_then(|v| match v {
            Value::Number(n) => n.as_f64(),
            Value::String(s) => s.trim_end_matches('%').parse().ok(),
            _ => None,
        })
        .unwrap_or(0.5);
    let from = obj
        .get("from")
        .or_else(|| obj.get("color"))
        .ok_or_else(|| anyhow::anyhow!("mix op requires `from`"))?;
    let onto = obj
        .get("onto")
        .or_else(|| obj.get("with"))
        .ok_or_else(|| anyhow::anyhow!("mix op requires `onto`"))?;
    let from_css = evaluate(from, resolve)?.css;
    let onto_css = evaluate(onto, resolve)?.css;
    // Prefer computed oklch when both sides parse; otherwise color-mix.
    let from_c = parse_color(&strip_var_fallback(&from_css));
    let onto_c = parse_color(&strip_var_fallback(&onto_css));
    if let (Some(a), Some(b)) = (from_c, onto_c) {
        return Ok(EvalResult::scalar(a.mix(b, amount).to_css()));
    }
    let pct = if amount <= 1.0 {
        amount * 100.0
    } else {
        amount
    };
    Ok(EvalResult::scalar(format!(
        "color-mix(in oklch, {from_css} {pct:.2}%, {onto_css})"
    )))
}

fn strip_var_fallback(s: &str) -> String {
    s.trim().to_string()
}

fn evaluate_gradient(
    obj: &serde_json::Map<String, Value>,
    resolve: &RefResolver<'_>,
) -> Result<EvalResult> {
    let kind = obj.get("kind").and_then(|v| v.as_str()).unwrap_or("linear");
    let stops = obj
        .get("stops")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("gradient requires `stops` array"))?;
    let mut rendered = Vec::new();
    for stop in stops {
        rendered.push(render_stop(stop, resolve)?);
    }
    let body = rendered.join(", ");
    let css = match kind {
        "radial" => format!("radial-gradient({body})"),
        "conic" => format!("conic-gradient({body})"),
        _ => {
            let angle = obj
                .get("angle")
                .map(|v| match v {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => format!("{n}deg"),
                    _ => "180deg".into(),
                })
                .unwrap_or_else(|| "180deg".into());
            format!("linear-gradient({angle}, {body})")
        }
    };
    Ok(EvalResult::scalar(css))
}

fn render_stop(stop: &Value, resolve: &RefResolver<'_>) -> Result<String> {
    match stop {
        Value::String(s) => Ok(evaluate_string(s, resolve)?.css),
        Value::Object(o) => {
            let color = o
                .get("color")
                .ok_or_else(|| anyhow::anyhow!("gradient stop needs `color`"))?;
            let color_css = evaluate(color, resolve)?.css;
            if let Some(at) = o.get("at").or_else(|| o.get("pos")) {
                let pos = match at {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => format!("{n}%"),
                    _ => String::new(),
                };
                if pos.is_empty() {
                    Ok(color_css)
                } else {
                    Ok(format!("{color_css} {pos}"))
                }
            } else {
                Ok(color_css)
            }
        }
        other => bail!("gradient stop must be string or object, got {other}"),
    }
}

fn evaluate_fluid(
    obj: &serde_json::Map<String, Value>,
    resolve: &RefResolver<'_>,
) -> Result<EvalResult> {
    let src = obj.get("fluid").and_then(|v| v.as_object()).unwrap_or(obj);
    let min = scalar_field(src, "min", resolve)?;
    let pref =
        scalar_field(src, "pref", resolve).or_else(|_| scalar_field(src, "preferred", resolve))?;
    let max = scalar_field(src, "max", resolve)?;
    Ok(EvalResult::scalar(format!("clamp({min}, {pref}, {max})")))
}

fn scalar_field(
    obj: &serde_json::Map<String, Value>,
    key: &str,
    resolve: &RefResolver<'_>,
) -> Result<String> {
    let v = obj
        .get(key)
        .ok_or_else(|| anyhow::anyhow!("fluid value missing `{key}`"))?;
    Ok(evaluate(v, resolve)?.css)
}

fn evaluate_shadow(
    obj: &serde_json::Map<String, Value>,
    resolve: &RefResolver<'_>,
) -> Result<EvalResult> {
    let ox = dim_field(obj, &["offsetX", "x", "offset"], "0")?;
    let oy = dim_field(obj, &["offsetY", "y"], "0")?;
    let blur = dim_field(obj, &["blur"], "0")?;
    let spread = dim_field(obj, &["spread"], "0")?;
    let color = if let Some(c) = obj.get("color") {
        evaluate(c, resolve)?.css
    } else {
        "currentColor".into()
    };
    let inset = obj.get("inset").and_then(|v| v.as_bool()).unwrap_or(false);
    let prefix = if inset { "inset " } else { "" };
    Ok(EvalResult::scalar(format!(
        "{prefix}{ox} {oy} {blur} {spread} {color}"
    )))
}

fn dim_field(obj: &serde_json::Map<String, Value>, keys: &[&str], default: &str) -> Result<String> {
    for k in keys {
        if let Some(v) = obj.get(*k) {
            return Ok(match v {
                Value::String(s) => s.clone(),
                Value::Number(n) => format!("{n}px"),
                other => other.to_string(),
            });
        }
    }
    Ok(default.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn resolve(name: &str) -> Option<String> {
        match name {
            "primary" | "color.primary.solid" => Some("primary".into()),
            "background" => Some("background".into()),
            _ => None,
        }
    }

    #[test]
    fn ref_object() {
        let v = json!({ "ref": "primary" });
        let ev = evaluate(&v, &resolve).unwrap();
        assert_eq!(ev.css, "var(--primary)");
    }

    #[test]
    fn dangling_ref_is_named() {
        let v = json!({ "ref": "no-such" });
        let err = evaluate(&v, &resolve).unwrap_err().to_string();
        assert!(err.contains("dangling ref"));
        assert!(err.contains("no-such"));
    }

    #[test]
    fn color_object() {
        let v = json!({ "space": "oklch", "l": 0.5, "c": 0.1, "h": 40 });
        let ev = evaluate(&v, &resolve).unwrap();
        assert!(ev.css.starts_with("oklch("));
    }

    #[test]
    fn mix_computed_when_both_parse() {
        let v = json!({
            "op": "mix",
            "from": { "space": "oklch", "l": 0.4, "c": 0.1, "h": 40 },
            "onto": { "space": "oklch", "l": 0.8, "c": 0.0, "h": 40 },
            "amount": 0.25
        });
        let ev = evaluate(&v, &resolve).unwrap();
        assert!(ev.css.starts_with("oklch("));
    }

    #[test]
    fn fluid_clamp() {
        let v = json!({ "min": "3rem", "pref": "9vw", "max": "8.5rem" });
        let ev = evaluate(&v, &resolve).unwrap();
        assert_eq!(ev.css, "clamp(3rem, 9vw, 8.5rem)");
    }

    #[test]
    fn breakpoint_keyed() {
        let v = json!({ "base": "16px", "tablet": "24px", "desktop": "32px" });
        let ev = evaluate(&v, &resolve).unwrap();
        assert_eq!(ev.css, "16px");
        assert_eq!(ev.breakpoints.get("tablet").unwrap(), "24px");
    }

    #[test]
    fn gradient_linear() {
        let v = json!({
            "kind": "linear",
            "angle": "180deg",
            "stops": [
                { "color": { "space": "oklch", "l": 0.6, "c": 0.1, "h": 40 }, "at": "0%" },
                { "color": "transparent", "at": "92%" }
            ]
        });
        let ev = evaluate(&v, &resolve).unwrap();
        assert!(ev.css.starts_with("linear-gradient(180deg,"));
        assert!(ev.css.contains("transparent 92%"));
    }

    #[test]
    fn structured_shadow() {
        let v = json!({
            "offsetX": "0",
            "offsetY": "4px",
            "blur": "12px",
            "spread": "0",
            "color": { "ref": "primary" }
        });
        let ev = evaluate(&v, &resolve).unwrap();
        assert!(ev.css.contains("4px 12px 0 var(--primary)"));
    }
}
