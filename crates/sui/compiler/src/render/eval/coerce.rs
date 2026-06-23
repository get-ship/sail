//! JS-flavoured coercion + equality helpers shared across the
//! evaluator. The renderable subset never produces `undefined`
//! distinctly from `null`, so coercion treats both as `Value::Null`.

use serde_json::Value;

pub fn truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().map(|f| f != 0.0 && !f.is_nan()).unwrap_or(false),
        Value::String(s) => !s.is_empty(),
        Value::Array(_) | Value::Object(_) => true,
    }
}

pub fn to_string(v: &Value) -> String {
    match v {
        Value::Null => "null".into(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => format_num(n.as_f64().unwrap_or(f64::NAN)),
        Value::String(s) => s.clone(),
        Value::Array(arr) => arr.iter().map(to_string).collect::<Vec<_>>().join(","),
        Value::Object(_) => "[object Object]".into(),
    }
}

pub fn to_number(v: &Value) -> f64 {
    match v {
        Value::Null => 0.0,
        Value::Bool(true) => 1.0,
        Value::Bool(false) => 0.0,
        Value::Number(n) => n.as_f64().unwrap_or(f64::NAN),
        Value::String(s) => {
            let t = s.trim();
            if t.is_empty() { 0.0 } else { t.parse::<f64>().unwrap_or(f64::NAN) }
        }
        Value::Array(arr) if arr.is_empty() => 0.0,
        Value::Array(arr) if arr.len() == 1 => to_number(&arr[0]),
        _ => f64::NAN,
    }
}

pub fn num_to_value(n: f64) -> Value {
    if !n.is_finite() {
        return Value::Null;
    }
    if n.fract() == 0.0 && n.abs() < 9.007e15 {
        return Value::from(n as i64);
    }
    serde_json::Number::from_f64(n)
        .map(Value::Number)
        .unwrap_or(Value::Null)
}

pub fn format_num(n: f64) -> String {
    if n.is_nan() {
        "NaN".into()
    } else if n.is_infinite() {
        if n.is_sign_negative() { "-Infinity".into() } else { "Infinity".into() }
    } else if n == n.trunc() && n.abs() < 1e21 {
        format!("{}", n as i64)
    } else {
        format!("{n}")
    }
}

pub fn strict_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Null, Value::Null) => true,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Number(x), Value::Number(y)) => match (x.as_f64(), y.as_f64()) {
            (Some(a), Some(b)) => a == b,
            _ => false,
        },
        (Value::String(x), Value::String(y)) => x == y,
        (Value::Array(_), Value::Array(_)) | (Value::Object(_), Value::Object(_)) => {
            std::ptr::eq(a, b)
        }
        _ => false,
    }
}

pub fn loose_eq(a: &Value, b: &Value) -> bool {
    if std::mem::discriminant(a) == std::mem::discriminant(b) {
        return strict_eq(a, b);
    }
    if a.is_null() && b.is_null() {
        return true;
    }
    if a.is_null() || b.is_null() {
        return false;
    }
    match (a, b) {
        (Value::Number(_), Value::String(_)) | (Value::String(_), Value::Number(_)) => {
            to_number(a) == to_number(b)
        }
        (Value::Bool(_), _) | (_, Value::Bool(_)) => to_number(a) == to_number(b),
        _ => false,
    }
}
