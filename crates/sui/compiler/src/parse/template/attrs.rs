//! Attribute classifier. Maps each attribute on an element to an
//! `AttrInstr`. Mirrors `packages/sui/dom/src/bindings/attr-compile.ts`.

use crate::diagnostics::{CompileError, Span};
use crate::ir::*;
use crate::parse::expr;
use crate::parse::html;

pub fn compile_attrs(
    el: &html::Element,
    loc_prefix: &str,
) -> Result<Vec<AttrInstr>, CompileError> {
    let mut out = Vec::new();
    let is_slot = el.tag.eq_ignore_ascii_case("slot");
    for a in &el.attributes {
        out.push(compile_attr(
            &a.name,
            &a.value,
            a.value_span,
            &format!("{loc_prefix} {}", a.name),
            is_slot,
        )?);
    }
    Ok(out)
}

fn compile_attr(
    name: &str,
    value: &str,
    value_span: Span,
    loc: &str,
    on_slot: bool,
) -> Result<AttrInstr, CompileError> {
    if name == "reflect-empty" {
        return compile_reflect_empty(value, value_span, loc, on_slot);
    }
    if name == "mutation" {
        if value.is_empty() {
            return Err(CompileError::new("mutation= requires a value (e.g. \"ship.agents.create\")")
                .with_span(value_span)
                .with_category(loc.to_string()));
        }
        return Ok(AttrInstr::Mutation {
            expr: Expr::new(value.to_string(), value_span, false),
            span: value_span,
        });
    }
    if let Some(rest) = name.strip_prefix("on:") {
        return compile_on(rest, value, value_span, loc);
    }
    if let Some(rest) = name.strip_prefix("bind:") {
        return compile_bind(rest, value, value_span, loc);
    }
    if let Some(rest) = name.strip_prefix("class:") {
        return compile_class(rest, value, value_span, loc);
    }
    if let Some(rest) = name.strip_prefix("style:") {
        return compile_style(rest, value, value_span, loc);
    }
    if let Some(rest) = name.strip_prefix("attr:") {
        return compile_dynamic_attr(rest, value, value_span, loc);
    }
    if let Some(rest) = name.strip_prefix("prop:") {
        return compile_prop(rest, value, value_span, loc);
    }
    if let Some(rest) = name.strip_prefix("transition:") {
        return compile_transition(rest, value, value_span, loc, TransitionDir::Both);
    }
    if let Some(rest) = name.strip_prefix("in:") {
        return compile_transition(rest, value, value_span, loc, TransitionDir::In);
    }
    if let Some(rest) = name.strip_prefix("out:") {
        return compile_transition(rest, value, value_span, loc, TransitionDir::Out);
    }
    if is_expr_value(value) {
        let inner = strip_braces(value).to_string();
        expr::validate_at(&inner, false, loc, value_span)?;
        return Ok(AttrInstr::Expr {
            name: name.to_string(),
            expr: Expr::new(inner, value_span, false),
            span: value_span,
        });
    }
    Ok(AttrInstr::Static {
        name: name.to_string(),
        value: value.to_string(),
        span: value_span,
    })
}

fn is_expr_value(v: &str) -> bool {
    let t = v.trim();
    t.starts_with('{') && t.ends_with('}') && t.len() >= 2
}

fn strip_braces(v: &str) -> &str {
    let t = v.trim();
    if t.starts_with('{') && t.ends_with('}') && t.len() >= 2 {
        t[1..t.len() - 1].trim()
    } else {
        v
    }
}

fn compile_on(
    suffix: &str,
    value: &str,
    value_span: Span,
    loc: &str,
) -> Result<AttrInstr, CompileError> {
    let mut parts = suffix.split('|');
    let event = parts.next().unwrap_or("").to_string();
    if event.is_empty() {
        return Err(CompileError::new("empty event name in on: binding")
            .with_span(value_span)
            .with_category(loc.to_string()));
    }
    let mut modifiers: Vec<Modifier> = Vec::new();
    for raw in parts {
        if raw.is_empty() {
            continue;
        }
        modifiers.push(parse_modifier(raw, value_span, loc)?);
    }
    if !is_expr_value(value) {
        return Err(CompileError::new("on: handler must be {expression}")
            .with_span(value_span)
            .with_category(loc.to_string()));
    }
    let inner = strip_braces(value).to_string();
    expr::validate_at(&inner, true, loc, value_span)?;
    Ok(AttrInstr::On {
        event,
        handler: Expr::new(inner, value_span, true),
        modifiers,
        span: value_span,
    })
}

fn parse_modifier(raw: &str, span: Span, loc: &str) -> Result<Modifier, CompileError> {
    let (head, arg) = match raw.find(':') {
        Some(idx) => (&raw[..idx], Some(raw[idx + 1..].to_string())),
        None => (raw, None),
    };
    let key = head.to_ascii_lowercase();
    Ok(match key.as_str() {
        "preventdefault" | "prevent" => Modifier::PreventDefault,
        "stoppropagation" | "stop" => Modifier::StopPropagation,
        "stopimmediatepropagation" => Modifier::StopImmediatePropagation,
        "self" => Modifier::SelfMod,
        "trusted" => Modifier::Trusted,
        "key" => Modifier::Key(arg),
        "debounce" => Modifier::Debounce(arg),
        "throttle" => Modifier::Throttle(arg),
        _ => {
            return Err(CompileError::new(format!("unknown modifier \"{head}\""))
                .with_span(span)
                .with_category(loc.to_string()));
        }
    })
}

fn compile_bind(
    target: &str,
    value: &str,
    value_span: Span,
    loc: &str,
) -> Result<AttrInstr, CompileError> {
    if !is_expr_value(value) {
        return Err(CompileError::new(format!("bind:{target} expects {{signal}}"))
            .with_span(value_span)
            .with_category(loc.to_string()));
    }
    let inner = strip_braces(value).to_string();
    if target == "this" {
        if !is_bare_identifier(&inner) {
            return Err(CompileError::new(
                "bind:this={var} expects a bare identifier (refs are write-once, not derivable)",
            )
            .with_span(value_span)
            .with_category(loc.to_string()));
        }
        return Ok(AttrInstr::BindThis { var: inner, span: value_span });
    }
    expr::validate_at(&inner, false, loc, value_span)?;
    let expr = Expr::new(inner, value_span, false);
    match target {
        "value" => Ok(AttrInstr::BindValue { expr, span: value_span }),
        "checked" => Ok(AttrInstr::BindChecked { expr, span: value_span }),
        other => Err(CompileError::new(format!("unknown bind:{other}"))
            .with_span(value_span)
            .with_category(loc.to_string())),
    }
}

fn compile_reflect_empty(
    value: &str,
    value_span: Span,
    loc: &str,
    on_slot: bool,
) -> Result<AttrInstr, CompileError> {
    if !on_slot {
        return Err(CompileError::new(
            "reflect-empty=\"...\" is only valid on <slot> elements",
        )
        .with_span(value_span)
        .with_category(loc.to_string()));
    }
    if is_expr_value(value) {
        return Err(CompileError::new(
            "reflect-empty must be a static attribute name (e.g. \"data-icon-only\"), not an expression",
        )
        .with_span(value_span)
        .with_category(loc.to_string()));
    }
    if value.is_empty() {
        return Err(CompileError::new(
            "reflect-empty=\"<attr>\" requires a target attribute name",
        )
        .with_span(value_span)
        .with_category(loc.to_string()));
    }
    if !is_valid_attr_name(value) {
        return Err(CompileError::new(format!(
            "reflect-empty target \"{value}\" must match [a-z][a-z0-9-]*"
        ))
        .with_span(value_span)
        .with_category(loc.to_string()));
    }
    Ok(AttrInstr::SlotReflectEmpty {
        attr: value.to_string(),
        span: value_span,
    })
}

fn is_bare_identifier(s: &str) -> bool {
    let s = s.trim();
    let mut chars = s.chars();
    let Some(first) = chars.next() else { return false };
    if !(first.is_ascii_alphabetic() || first == '_' || first == '$') {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

fn is_valid_attr_name(s: &str) -> bool {
    let mut chars = s.chars();
    let Some(first) = chars.next() else { return false };
    if !first.is_ascii_lowercase() {
        return false;
    }
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

fn compile_class(
    name: &str,
    value: &str,
    value_span: Span,
    loc: &str,
) -> Result<AttrInstr, CompileError> {
    if name.is_empty() {
        return Err(CompileError::new("class: requires a class name")
            .with_span(value_span)
            .with_category(loc.to_string()));
    }
    if !is_expr_value(value) {
        return Err(CompileError::new(format!("class:{name} expects {{expr}}"))
            .with_span(value_span)
            .with_category(loc.to_string()));
    }
    let inner = strip_braces(value).to_string();
    expr::validate_at(&inner, false, loc, value_span)?;
    Ok(AttrInstr::Class {
        name: name.to_string(),
        expr: Expr::new(inner, value_span, false),
        span: value_span,
    })
}

fn compile_style(
    name: &str,
    value: &str,
    value_span: Span,
    loc: &str,
) -> Result<AttrInstr, CompileError> {
    if name.is_empty() {
        return Err(CompileError::new("style: requires a property name")
            .with_span(value_span)
            .with_category(loc.to_string()));
    }
    if !is_expr_value(value) {
        return Err(CompileError::new(format!("style:{name} expects {{expr}}"))
            .with_span(value_span)
            .with_category(loc.to_string()));
    }
    let inner = strip_braces(value).to_string();
    expr::validate_at(&inner, false, loc, value_span)?;
    Ok(AttrInstr::Style {
        name: name.to_string(),
        expr: Expr::new(inner, value_span, false),
        span: value_span,
    })
}

fn compile_prop(
    name: &str,
    value: &str,
    value_span: Span,
    loc: &str,
) -> Result<AttrInstr, CompileError> {
    if name.is_empty() {
        return Err(CompileError::new("prop: requires a property name")
            .with_span(value_span)
            .with_category(loc.to_string()));
    }
    if !is_expr_value(value) {
        return Err(CompileError::new(format!("prop:{name} expects {{expr}}"))
            .with_span(value_span)
            .with_category(loc.to_string()));
    }
    let inner = strip_braces(value).to_string();
    expr::validate_at(&inner, false, loc, value_span)?;
    Ok(AttrInstr::Prop {
        name: name.to_string(),
        expr: Expr::new(inner, value_span, false),
        span: value_span,
    })
}

fn compile_transition(
    name: &str,
    value: &str,
    value_span: Span,
    loc: &str,
    dir: TransitionDir,
) -> Result<AttrInstr, CompileError> {
    let err = |msg: String| {
        CompileError::new(msg).with_span(value_span).with_category(loc.to_string())
    };
    if name.is_empty() {
        return Err(err(format!("{}:requires a transition name", dir.as_str())));
    }
    if !is_bare_identifier(name) {
        return Err(err(format!("transition name \"{name}\" must be a bare identifier")));
    }
    let opts = if value.is_empty() {
        None
    } else if is_expr_value(value) {
        let inner = strip_braces(value).to_string();
        expr::validate_at(&inner, false, loc, value_span)?;
        Some(Expr::new(inner, value_span, false))
    } else {
        return Err(err("transition options must be an {expression}".into()));
    };
    Ok(AttrInstr::Transition { name: name.to_string(), opts, dir, span: value_span })
}

fn compile_dynamic_attr(
    raw_name: &str,
    value: &str,
    value_span: Span,
    loc: &str,
) -> Result<AttrInstr, CompileError> {
    if !is_expr_value(value) {
        // `attr:foo="bar"` is the most common mistake — author reached
        // for `attr:` to bind a *static* value, when plain `foo="bar"`
        // is what they wanted. Surface the fix in the error so the
        // author doesn't have to know that `attr:` is reserved for
        // expression bindings (`attr:foo={expr}`). Hit five+ times
        // during the harness build with shapes like `attr:min="0"`.
        return Err(CompileError::new(format!(
            "attr:{raw_name}=\"{value}\" — `attr:` expects an expression binding (attr:{raw_name}={{expr}}). For a static value, drop the `attr:` prefix: {raw_name}=\"{value}\"."
        ))
        .with_span(value_span)
        .with_category(loc.to_string()));
    }
    let value_inner = strip_braces(value).to_string();
    expr::validate_at(&value_inner, false, loc, value_span)?;
    if is_expr_value(raw_name) {
        let name_inner = strip_braces(raw_name).to_string();
        expr::validate_at(&name_inner, false, loc, value_span)?;
        return Ok(AttrInstr::DynamicAttr {
            name_expr: Expr::new(name_inner, value_span, false),
            expr: Expr::new(value_inner, value_span, false),
            span: value_span,
        });
    }
    if !is_valid_attr_name(raw_name) {
        return Err(CompileError::new(
            "attr: expects attr:<name>={valueExpr} or attr:{nameExpr}={valueExpr}",
        )
        .with_span(value_span)
        .with_category(loc.to_string()));
    }
    Ok(AttrInstr::Expr {
        name: raw_name.to_string(),
        expr: Expr::new(value_inner, value_span, false),
        span: value_span,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attr_with_literal_value_suggests_dropping_prefix() {
        // Pre-fix the error read "attr: expects ={valueExpr}", which is
        // technically correct but doesn't tell the author what to do.
        // Most of the time the fix is "drop attr: and write min=\"0\""
        // — surface that in the message.
        // The HTML attribute parser strips quotes before handing us
        // the value, so we re-quote in the suggestion.
        let err = compile_dynamic_attr("min", "0", Span::new(0, 1), "[<sui-slider> attr:min]")
            .unwrap_err();
        assert!(
            err.message.contains("drop the `attr:` prefix"),
            "expected suggestion, got: {}",
            err.message
        );
        assert!(err.message.contains("min=\"0\""), "expected suggested form, got: {}", err.message);
    }

    #[test]
    fn attr_with_expr_value_still_compiles() {
        // Don't regress the happy path.
        compile_dynamic_attr("min", "{n()}", Span::new(0, 5), "[<sui-slider> attr:min]")
            .expect("expression-value attr: should compile");
    }
}
