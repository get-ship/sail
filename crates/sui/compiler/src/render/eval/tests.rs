//! Unit tests for the expression evaluator. Cover the renderable
//! subset rule-by-rule (positive + negative).

use serde_json::{Value, json};

use super::{EvalError, Scope, eval};

fn s(pairs: &[(&str, Value)]) -> Scope {
    pairs.iter().cloned().map(|(k, v)| (k.to_string(), v)).collect()
}

#[test]
fn literals_and_identifier_reads() {
    let sc = s(&[("name", json!("Ada"))]);
    assert_eq!(eval("42", &sc).unwrap(), json!(42));
    assert_eq!(eval("'hi'", &sc).unwrap(), json!("hi"));
    assert_eq!(eval("true", &sc).unwrap(), json!(true));
    assert_eq!(eval("null", &sc).unwrap(), Value::Null);
    assert_eq!(eval("undefined", &sc).unwrap(), Value::Null);
    assert_eq!(eval("name", &sc).unwrap(), json!("Ada"));
    assert_eq!(eval("missing", &sc), Err(EvalError::Unbound("missing".into())));
}

#[test]
fn property_access_with_optional_chain() {
    let sc = s(&[("u", json!({"profile": {"name": "Ada"}}))]);
    assert_eq!(eval("u.profile.name", &sc).unwrap(), json!("Ada"));
    assert_eq!(eval("u.missing", &sc).unwrap(), Value::Null);
    assert!(matches!(eval("u.x.y", &sc), Err(EvalError::TypeMismatch(_))));
    assert_eq!(eval("u.x?.y", &sc).unwrap(), Value::Null);
}

#[test]
fn nullish_coalescing_returns_right_only_for_null() {
    let sc = s(&[("a", Value::Null), ("b", json!(0)), ("c", json!("x"))]);
    assert_eq!(eval("a ?? 'fallback'", &sc).unwrap(), json!("fallback"));
    assert_eq!(eval("b ?? 'fallback'", &sc).unwrap(), json!(0));
    assert_eq!(eval("c ?? 'fallback'", &sc).unwrap(), json!("x"));
}

#[test]
fn arithmetic_and_string_concat() {
    let sc = s(&[("n", json!(5)), ("name", json!("Ada"))]);
    assert_eq!(eval("n + 3", &sc).unwrap(), json!(8));
    assert_eq!(eval("n - 1", &sc).unwrap(), json!(4));
    assert_eq!(eval("n * 2", &sc).unwrap(), json!(10));
    assert_eq!(eval("n / 2", &sc).unwrap(), json!(2.5));
    assert_eq!(eval("n % 2", &sc).unwrap(), json!(1));
    assert_eq!(eval("'/u/' + name", &sc).unwrap(), json!("/u/Ada"));
    assert_eq!(eval("'a' + 1", &sc).unwrap(), json!("a1"));
}

#[test]
fn comparisons_strict_and_loose() {
    let sc = s(&[("a", json!(1))]);
    assert_eq!(eval("a < 2", &sc).unwrap(), json!(true));
    assert_eq!(eval("a >= 1", &sc).unwrap(), json!(true));
    assert_eq!(eval("a === 1", &sc).unwrap(), json!(true));
    assert_eq!(eval("a === '1'", &sc).unwrap(), json!(false));
    assert_eq!(eval("a == '1'", &sc).unwrap(), json!(true));
    assert_eq!(eval("a !== 2", &sc).unwrap(), json!(true));
}

#[test]
fn boolean_short_circuit() {
    let sc = s(&[("flag", json!(true))]);
    assert_eq!(eval("flag && 'yes'", &sc).unwrap(), json!("yes"));
    assert_eq!(eval("false && missing", &sc).unwrap(), json!(false));
    assert_eq!(eval("flag || missing", &sc).unwrap(), json!(true));
    assert_eq!(eval("!flag", &sc).unwrap(), json!(false));
}

#[test]
fn ternary_picks_branch() {
    let sc = s(&[("n", json!(2))]);
    assert_eq!(eval("n === 1 ? 'one' : 'many'", &sc).unwrap(), json!("many"));
}

#[test]
fn template_literal_concatenates() {
    let sc = s(&[("n", json!(3))]);
    assert_eq!(eval("`count: ${n}`", &sc).unwrap(), json!("count: 3"));
}

#[test]
fn array_and_object_literals() {
    let sc = s(&[("x", json!(2))]);
    assert_eq!(eval("[1, x, 3]", &sc).unwrap(), json!([1, 2, 3]));
    assert_eq!(eval("{ a: 1, b: x }", &sc).unwrap(), json!({"a": 1, "b": 2}));
}

#[test]
fn zero_arg_call_is_signal_read() {
    let sc = s(&[("count", json!(7))]);
    assert_eq!(eval("count()", &sc).unwrap(), json!(7));
}

#[test]
fn allowlisted_methods_evaluate() {
    let sc = s(&[
        ("xs", json!([1, 2, 3, 4])),
        ("name", json!("Ada")),
        ("o", json!({"a": 1, "b": 2})),
    ]);
    assert_eq!(eval("xs.length", &sc).unwrap(), json!(4));
    assert_eq!(eval("name.length", &sc).unwrap(), json!(3));
    assert_eq!(eval("xs.slice(1, 3)", &sc).unwrap(), json!([2, 3]));
    assert_eq!(eval("name.slice(0, 2)", &sc).unwrap(), json!("Ad"));
    assert_eq!(eval("xs.includes(2)", &sc).unwrap(), json!(true));
    assert_eq!(eval("xs.includes(99)", &sc).unwrap(), json!(false));
    assert_eq!(eval("o.keys()", &sc).unwrap(), json!(["a", "b"]));
    assert_eq!(eval("o.values()", &sc).unwrap(), json!([1, 2]));
    assert_eq!(eval("(42).toString()", &sc).unwrap(), json!("42"));
}

#[test]
fn to_fixed_formats_numbers() {
    let sc = s(&[
        ("n", json!(3.14159)),
        ("zero", json!(0)),
        ("neg", json!(-1.5)),
        ("nan", json!("not-a-number")),
    ]);
    assert_eq!(eval("n.toFixed(2)", &sc).unwrap(), json!("3.14"));
    assert_eq!(eval("n.toFixed(0)", &sc).unwrap(), json!("3"));
    assert_eq!(eval("n.toFixed(4)", &sc).unwrap(), json!("3.1416"));
    assert_eq!(eval("zero.toFixed(3)", &sc).unwrap(), json!("0.000"));
    assert_eq!(eval("neg.toFixed(1)", &sc).unwrap(), json!("-1.5"));
    // Coerce-to-number on string: matches JS — "not-a-number" → NaN → "NaN".
    assert_eq!(eval("nan.toFixed(2)", &sc).unwrap(), json!("NaN"));
}

#[test]
fn map_callback_is_non_evaluable() {
    // `.map`/`.filter` always return NonEvaluable — the parser doesn't
    // accept arrow callbacks (no `=>` token), and we can't model JS
    // callbacks against the IR scope. A bare identifier arg parses but
    // the method handler short-circuits without evaluating args.
    let sc = s(&[("xs", json!([1, 2, 3])), ("fn", json!(null))]);
    assert!(matches!(
        eval("xs.map(fn)", &sc),
        Err(EvalError::NonEvaluable(_))
    ));
    assert!(matches!(
        eval("xs.filter(fn)", &sc),
        Err(EvalError::NonEvaluable(_))
    ));
    assert!(matches!(
        eval("xs.foo()", &sc),
        Err(EvalError::NonEvaluable(_))
    ));
}

#[test]
fn parse_error_for_garbage() {
    let sc = Scope::new();
    assert!(matches!(eval("1 +", &sc), Err(EvalError::ParseError(_))));
    assert!(matches!(eval("", &sc), Err(EvalError::ParseError(_))));
    assert!(matches!(eval("@", &sc), Err(EvalError::ParseError(_))));
}
