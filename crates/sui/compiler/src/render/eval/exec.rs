//! Walks the parsed expression AST and produces a `serde_json::Value`.
//! All coercion helpers live in `coerce`; method dispatch in `methods`.

use serde_json::{Map, Value};

use super::EvalError;
use super::coerce::{
    format_num, loose_eq, num_to_value, strict_eq, to_number, to_string, truthy,
};
use super::methods;
use super::parser::{BinOp, LogicalOp, Node, TplPart, UnaryOp};

type Scope = std::collections::HashMap<String, Value>;

pub(super) fn execute(node: &Node, scope: &Scope) -> Result<Value, EvalError> {
    match node {
        Node::Null | Node::Undefined => Ok(Value::Null),
        Node::Bool(b) => Ok(Value::Bool(*b)),
        Node::Num(n) => Ok(num_to_value(*n)),
        Node::Str(s) => Ok(Value::String(s.clone())),
        Node::Template(parts) => eval_template(parts, scope),
        Node::Ident(name) => scope
            .get(name)
            .cloned()
            .ok_or_else(|| EvalError::Unbound(name.clone())),
        Node::Array(items) => {
            let mut out = Vec::with_capacity(items.len());
            for it in items {
                out.push(execute(it, scope)?);
            }
            Ok(Value::Array(out))
        }
        Node::Object(props) => {
            let mut map = Map::with_capacity(props.len());
            for (k, v) in props {
                map.insert(k.clone(), execute(v, scope)?);
            }
            Ok(Value::Object(map))
        }
        Node::Member { obj, prop, optional } => {
            let o = execute(obj, scope)?;
            member_access(&o, prop, *optional)
        }
        Node::Index { obj, idx, optional } => {
            let o = execute(obj, scope)?;
            if o.is_null() {
                return if *optional {
                    Ok(Value::Null)
                } else {
                    Err(EvalError::TypeMismatch("index access on null".into()))
                };
            }
            let i = execute(idx, scope)?;
            index_access(&o, &i)
        }
        Node::Call { callee, args } => call_expr(callee, args, scope),
        Node::Unary { op, expr } => {
            let v = execute(expr, scope)?;
            Ok(unary(*op, &v))
        }
        Node::Binary { op, l, r } => {
            let lv = execute(l, scope)?;
            let rv = execute(r, scope)?;
            Ok(binary(*op, &lv, &rv))
        }
        Node::Logical { op, l, r } => logical(*op, l, r, scope),
        Node::Conditional { cond, then, els } => {
            let c = execute(cond, scope)?;
            if truthy(&c) { execute(then, scope) } else { execute(els, scope) }
        }
    }
}

fn eval_template(parts: &[TplPart], scope: &Scope) -> Result<Value, EvalError> {
    let mut out = String::new();
    for part in parts {
        match part {
            TplPart::Str(s) => out.push_str(s),
            TplPart::Expr(node) => {
                let v = execute(node, scope)?;
                out.push_str(&to_string(&v));
            }
        }
    }
    Ok(Value::String(out))
}

fn member_access(obj: &Value, prop: &str, optional: bool) -> Result<Value, EvalError> {
    if obj.is_null() {
        return if optional {
            Ok(Value::Null)
        } else {
            Err(EvalError::TypeMismatch(format!("property `.{prop}` on null")))
        };
    }
    match obj {
        Value::Object(map) => Ok(map.get(prop).cloned().unwrap_or(Value::Null)),
        Value::Array(arr) if prop == "length" => Ok(num_to_value(arr.len() as f64)),
        Value::String(s) if prop == "length" => Ok(num_to_value(s.chars().count() as f64)),
        Value::Array(_) | Value::String(_) => Ok(Value::Null),
        _ => Err(EvalError::TypeMismatch(format!(
            "property `.{prop}` on non-object"
        ))),
    }
}

fn index_access(obj: &Value, idx: &Value) -> Result<Value, EvalError> {
    match (obj, idx) {
        (Value::Array(arr), Value::Number(n)) => {
            let i = n.as_f64().unwrap_or(f64::NAN) as isize;
            if i < 0 || (i as usize) >= arr.len() {
                Ok(Value::Null)
            } else {
                Ok(arr[i as usize].clone())
            }
        }
        (Value::String(s), Value::Number(n)) => {
            let i = n.as_f64().unwrap_or(f64::NAN) as isize;
            if i < 0 {
                return Ok(Value::Null);
            }
            Ok(s.chars()
                .nth(i as usize)
                .map(|c| Value::String(c.to_string()))
                .unwrap_or(Value::Null))
        }
        (Value::Object(map), Value::String(k)) => {
            Ok(map.get(k.as_str()).cloned().unwrap_or(Value::Null))
        }
        (Value::Object(map), Value::Number(n)) => Ok(map
            .get(&format_num(n.as_f64().unwrap_or(f64::NAN)))
            .cloned()
            .unwrap_or(Value::Null)),
        _ => Err(EvalError::TypeMismatch("invalid index access".into())),
    }
}

fn call_expr(callee: &Node, args: &[Node], scope: &Scope) -> Result<Value, EvalError> {
    if let Node::Member { obj, prop, optional } = callee {
        let obj_val = execute(obj, scope)?;
        if obj_val.is_null() {
            return if *optional {
                Ok(Value::Null)
            } else {
                Err(EvalError::TypeMismatch(format!(
                    "method `.{prop}()` on null"
                )))
            };
        }
        return methods::dispatch(&obj_val, prop, args, scope);
    }
    if args.is_empty() {
        return execute(callee, scope);
    }
    Err(EvalError::NonEvaluable(
        "non-allowlisted call with arguments".into(),
    ))
}

fn unary(op: UnaryOp, v: &Value) -> Value {
    match op {
        UnaryOp::Not => Value::Bool(!truthy(v)),
        UnaryOp::Neg => num_to_value(-to_number(v)),
        UnaryOp::Pos => num_to_value(to_number(v)),
    }
}

fn binary(op: BinOp, l: &Value, r: &Value) -> Value {
    match op {
        BinOp::Add => add(l, r),
        BinOp::Sub => num_to_value(to_number(l) - to_number(r)),
        BinOp::Mul => num_to_value(to_number(l) * to_number(r)),
        BinOp::Div => num_to_value(to_number(l) / to_number(r)),
        BinOp::Mod => num_to_value(to_number(l) % to_number(r)),
        BinOp::Lt => compare(l, r, |a, b| a < b, |a, b| a < b),
        BinOp::Le => compare(l, r, |a, b| a <= b, |a, b| a <= b),
        BinOp::Gt => compare(l, r, |a, b| a > b, |a, b| a > b),
        BinOp::Ge => compare(l, r, |a, b| a >= b, |a, b| a >= b),
        BinOp::Eq => Value::Bool(loose_eq(l, r)),
        BinOp::Neq => Value::Bool(!loose_eq(l, r)),
        BinOp::EqStrict => Value::Bool(strict_eq(l, r)),
        BinOp::NeqStrict => Value::Bool(!strict_eq(l, r)),
    }
}

fn add(l: &Value, r: &Value) -> Value {
    if l.is_string() || r.is_string() {
        Value::String(format!("{}{}", to_string(l), to_string(r)))
    } else {
        num_to_value(to_number(l) + to_number(r))
    }
}

fn compare<N, S>(l: &Value, r: &Value, num: N, str_cmp: S) -> Value
where
    N: Fn(f64, f64) -> bool,
    S: Fn(&str, &str) -> bool,
{
    if let (Value::String(a), Value::String(b)) = (l, r) {
        return Value::Bool(str_cmp(a.as_str(), b.as_str()));
    }
    let a = to_number(l);
    let b = to_number(r);
    if a.is_nan() || b.is_nan() {
        return Value::Bool(false);
    }
    Value::Bool(num(a, b))
}

fn logical(op: LogicalOp, l: &Node, r: &Node, scope: &Scope) -> Result<Value, EvalError> {
    let lv = execute(l, scope)?;
    match op {
        LogicalOp::And => {
            if !truthy(&lv) { Ok(lv) } else { execute(r, scope) }
        }
        LogicalOp::Or => {
            if truthy(&lv) { Ok(lv) } else { execute(r, scope) }
        }
        LogicalOp::Coalesce => {
            if lv.is_null() { execute(r, scope) } else { Ok(lv) }
        }
    }
}
