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
    if name == "part" {
        return Err(CompileError::new(
            "the `part` attribute is retired — light DOM has no shadow parts. \
             Anatomy is classes + attributes; the element's `class` is the \
             styling hook (published as CEM).",
        )
        .with_span(value_span)
        .with_category(loc.to_string()));
    }
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
    // `transition:` / `in:` / `out:` are RETIRED. They were a second animation
    // system — a second preset catalog (fade/slide/scale), a second registry
    // (`registerTransition`), a second runtime — sitting beside `motion:`, which
    // already bakes to `el.animate()` with compile-time-validated spring presets and
    // no runtime library at all. Two systems meant two vocabularies to learn, two
    // things to teach an agent, and two places for a bug to live.
    //
    // The CAPABILITY survives intact: holding an element in the DOM until its exit
    // animation finishes is real, and CSS cannot do it alone. It is `motion:exit`
    // now, running on the same lifecycle seam, from the one catalog.
    for (retired, hint) in [
        ("transition:", "motion:enter + motion:exit"),
        ("in:", "motion:enter"),
        ("out:", "motion:exit"),
    ] {
        if let Some(rest) = name.strip_prefix(retired) {
            return Err(CompileError::new(format!(
                "`{retired}{rest}` is retired — there is one animation system. \
                 Use `{hint}` with a preset from the motion catalog \
                 (e.g. `motion:exit={{ opacity: 'fade-out' }}`)."
            ))
            .with_span(value_span)
            .with_category(loc.to_string()));
        }
    }
    if let Some(rest) = name.strip_prefix("motion:") {
        return compile_motion(rest, value, value_span, loc);
    }
    if let Some(rest) = name.strip_prefix("use:") {
        return compile_use(rest, value, value_span, loc);
    }
    if is_expr_value(value) {
        let inner = strip_braces(value).to_string();
        // Parse the STRIPPED inner expression — `signal(0)`,
        // `state.x`, `"hello"` all parse fine; `count` parses as an
        // ident. Parse failures then fall into a first-char-based
        // classifier:
        //  * first non-trivia char `"` or `{` → author almost
        //    certainly meant a JSON-string or nested object, not an
        //    expression binding. Fail loud with a fix hint.
        //  * inner matches a renderable-subset forbidden-keyword
        //    pattern (`new`/`await`/`yield`/... as a standalone
        //    ident, or a tagged-template shape `ident\\`…\\``) →
        //    defer to the downstream subset check (it produces the
        //    sharper message).
        //  * otherwise → genuine structural mistake; fail loud.
        let base = value_span.start + value.len().saturating_sub(inner.len() + 2);
        let ast = match expr::grammar::parse_expression(&inner, base) {
            Ok(ast) => ast,
            Err(e) => {
                let trimmed = inner.trim();
                let first = trimmed.as_bytes().first().copied().unwrap_or(0);
                // Inner starts with `"` (JSON-shaped) or `{` (nested
                // object) — author mistake, not a deferred subset
                // rejection.
                let looks_json_like = first == b'"' || first == b'\'' || first == b'{';
                if !looks_json_like && defer_to_renderable_subset(&inner) {
                    return Ok(AttrInstr::Expr {
                        name: name.to_string(),
                        expr: Expr::new(inner, value_span, false),
                        span: value_span,
                    });
                }
                let is_arrow = inner.contains("=>");
                let msg = if is_arrow {
                    format!(
                        "attribute `{name}` value `{value}` is an arrow function — \
                         attribute expressions must be a plain JS value, not an arrow; \
                         for an event handler use `on:event={{...}}` instead"
                    )
                } else {
                    format!(
                        "attribute `{name}` has malformed `{{…}}` value `{value}` \
                         ({}); attribute expression bodies must be a valid JS \
                         expression — to pass a literal value that begins with `{{`, \
                         write it as a JS string (`attr=\"{{…}}\"`) or bind a controller \
                         value (`attr={{myValue}}`)",
                        e.message,
                    )
                };
                return Err(
                    CompileError::new(msg)
                        .with_span(value_span)
                        .with_category(loc.to_string()),
                );
            }
        };
        // Object AST with an explicit `key: value` entry is the
        // JSON-as-string mistake (`data='{"a":1}'`) — fail loud
        // with a fix hint. Bare shorthand entries (`{count}`,
        // `{...spread}`, `{"hello"}`) are normal JSX-style
        // JSX-style bindings and pass through.
        if let crate::parse::expr::AstKind::Object(entries) = &ast.kind {
            let has_key_value = entries
                .iter()
                .any(|e| {
                    matches!(e.key, crate::parse::expr::ObjKey::Ident(_) | crate::parse::expr::ObjKey::String(_))
                        && e.value.is_some()
                });
            if has_key_value {
                let msg = format!(
                    "attribute `{name}` value `{value}` is an object literal — \
                     attribute bindings must be a JS expression, not an object. \
                     To pass a JSON-shaped string, quote it: `attr=\"{value}\"` \
                     or escape the braces (`attr=\"{{…}}\"`); to bind a value, \
                     reference it: `attr={{myValue}}`.",
                );
                return Err(CompileError::new(msg)
                    .with_span(value_span)
                    .with_category(loc.to_string()));
            }
        }
        // Use the validated inner as the emit source so downstream
        // passes see the same expression we just parsed.
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
/// True when a parse failure on the brace body is best deferred to
/// the renderable-subset check downstream rather than surfaced here
/// as a "malformed brace body" error. Used when the grammar parser
/// rejects an inner shape the subset check rejects more precisely:
///
///  * any of the subset's forbidden keywords (`new`, `await`,
///    `yield`, `this`, `function`, `delete`, `void`) appears as a
///    standalone identifier — the subset produces a sharper
///    `"\`new\` is not allowed in renderable expressions"` etc.
///  * the body is a tagged template literal (`html\`…\``) — subset
///    says "tagged template literals are not allowed…".
///  * the body is an arrow function — the existing `arrow
///    functions are not allowed` message is sharper.
///
/// Anything else (`{@!#}`, trailing-comma object, empty `{}`, …) is
/// treated as a structural mistake and reported from
/// `compile_attr` itself.
fn defer_to_renderable_subset(inner: &str) -> bool {
    if inner.contains("=>") {
        return true;
    }
    const FORBIDDEN: &[&str] = &[
        "new", "await", "yield", "this", "function", "delete", "void",
    ];
    let bytes = inner.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // Skip strings / template literals — the forbidden-keyword
        // search must not match inside literal text.
        let c = bytes[i];
        if c == b'"' || c == b'\'' || c == b'`' {
            let q = c;
            i += 1;
            while i < bytes.len() && bytes[i] != q {
                if bytes[i] == b'\\' && i + 1 < bytes.len() {
                    i += 2;
                    continue;
                }
                // Template substitution: ${expr} re-enters JS.
                if q == b'`' && bytes[i] == b'$' && i + 1 < bytes.len() && bytes[i + 1] == b'{' {
                    let mut depth = 0i32;
                    i += 2;
                    while i < bytes.len() {
                        match bytes[i] {
                            b'{' => depth += 1,
                            b'}' => {
                                depth -= 1;
                                if depth == 0 {
                                    i += 1;
                                    break;
                                }
                            }
                            _ => {}
                        }
                        i += 1;
                    }
                    continue;
                }
                i += 1;
            }
            if i < bytes.len() {
                i += 1;
            }
            continue;
        }
        if c.is_ascii_alphabetic() || c == b'_' || c == b'$' {
            let start = i;
            while i < bytes.len()
                && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'$')
            {
                i += 1;
            }
            let word = &inner[start..i];
            if FORBIDDEN.contains(&word) {
                return true;
            }
            // Tagged template: an identifier immediately followed by
            // a backtick (after optional whitespace).
            let mut k = i;
            while k < bytes.len() && bytes[k].is_ascii_whitespace() {
                k += 1;
            }
            if k < bytes.len() && bytes[k] == b'`' {
                return true;
            }
            continue;
        }
        i += 1;
    }
    false
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


/// Compile a `use:action` / `use:action={param}` directive — attach a
/// reusable behavior (an "action") to the rendered element. `action` is a
/// bare identifier resolving to a scope value (a use-able hook handle or an
/// imported action fn); the optional `{param}` is its reactive parameter.
fn compile_use(
    name: &str,
    value: &str,
    value_span: Span,
    loc: &str,
) -> Result<AttrInstr, CompileError> {
    let err = |msg: String| {
        CompileError::new(msg).with_span(value_span).with_category(loc.to_string())
    };
    if name.is_empty() {
        return Err(err("use: requires an action name".into()));
    }
    if !is_bare_identifier(name) {
        return Err(err(format!("action name \"{name}\" must be a bare identifier")));
    }
    // Allow arrows in the param: an action's parameter is frequently a callback
    // (`use:clickOutside={() => menu.close()}`), unlike a plain attribute value.
    // The param is handler-capable so its arrow body rewrites against scope.
    let param = if value.is_empty() {
        None
    } else if is_expr_value(value) {
        let inner = strip_braces(value).to_string();
        expr::validate_at(&inner, true, loc, value_span)?;
        Some(Expr::new(inner, value_span, true))
    } else {
        return Err(err("use: parameter must be an {expression}".into()));
    };
    Ok(AttrInstr::Use { action: name.to_string(), param, span: value_span })
}
/// Compile a `motion:<trigger>=…` directive. Milestone 1 accepts only a
/// static object literal (`motion:hover={{ scale: 'sm' }}`); dynamic
/// expressions (`motion:hover={someExpr}`) are explicitly rejected so
/// the runtime never has to evaluate an animation config.
///
/// Returns an [`AttrInstr::Motion`] carrying the resolved property ->
/// preset pairs (validated in the lower layer against
/// `animation-presets.json`).
fn compile_motion(
    trigger: &str,
    value: &str,
    value_span: Span,
    loc: &str,
) -> Result<AttrInstr, CompileError> {
    let err = |msg: String| {
        CompileError::new(msg).with_span(value_span).with_category(loc.to_string())
    };
    if trigger.is_empty() {
        return Err(err(
            "motion: requires a trigger name (hover, press, focus, focus-visible, …)".into(),
        ));
    }
    // NO `is_bare_identifier` GATE HERE. That is a JAVASCRIPT identifier check, and
    // it forbids hyphens — so `motion:focus-visible` was rejected as "must be a bare
    // identifier" even though `focus-visible` is one of the eight triggers, is named
    // in MOTION_TRIGGERS, and is advertised by the error message four lines above.
    // One of the language's own triggers could not be written, and the diagnostic
    // told you to write the exact thing it refused.
    //
    // A trigger is not a JS identifier; it is a kebab-case name mirroring the CSS
    // pseudo-class. The allowlist below is strictly stronger than any shape check
    // could be — an unknown trigger fails it, garbage fails it — so the shape check
    // was only ever able to reject things the allowlist would have accepted.
    //
    // (An unknown trigger used to COMPILE and then silently do nothing — the runtime
    // no-ops a trigger it has no dispatch for, so `motion:hovr={…}` was a typo you
    // could ship. Naming the set is what fixed that; keep it.)
    if !MOTION_TRIGGERS.contains(&trigger) {
        return Err(err(format!(
            "motion trigger `{trigger}` is not supported. Triggers: {}.",
            MOTION_TRIGGERS.join(", ")
        )));
    }
    if value.is_empty() {
        return Err(err("motion: requires a config object".into()));
    }

    // Disambiguate `motion:hover={{...}}` (static) from
    // first; only reject as dynamic when the body is not a static
    // key/value list AND looks like an expression. This keeps the
    // common author form working while still flagging actual
    // Disambiguate `motion:hover={{ prop: 'value' }}` (static) from
    // `motion:hover={someExpr}` (dynamic). The pre-pass delivers the
    // value with both outer and inner braces intact for brace-attrs;
    // strip the outermost pair before the static parser. Treat as
    // dynamic ONLY when the body has no `:` separator (a key/value
    // shape). If the body has a `:` but the static parser still
    // rejects it (unknown property, malformed value, etc.), surface
    // the static-shape error verbatim — the author clearly meant
    // a static config.
    fn has_static_shape(body: &str) -> bool {
        let s = body.trim();
        s.starts_with('{') && s.contains(':') && s.ends_with('}')
    }
    let stripped = value.trim();
    let stripped = stripped.strip_prefix('{').unwrap_or(stripped);
    let stripped = stripped.strip_suffix('}').unwrap_or(stripped);
    let (props, stagger_ms) = match parse_motion_static_config(stripped, value_span, loc) {
        Ok(pair) => pair,
        Err(_) if !has_static_shape(value) => {
            return Err(err(format!(
                "motion:{trigger}={{...}} — dynamic configs are not yet supported; \
                 the value must be a static object literal like \
                 `motion:{trigger}={{ scale: 'sm' }}`"
            )));
        }
        Err(parse_err) => return Err(parse_err),
    };
    Ok(AttrInstr::Motion {
        trigger: trigger.to_string(),
        props,
        stagger_ms,
        span: value_span,
    })
}

// The property vocabulary lives in sui-lint — ONE list, two readers. The compiler
// kept its own copy, and the copies drifted the first time a key was added.
use sui_lint::rules::animate_token_only::{MOTION_PROPERTIES, MOTION_STAGGER_KEY, MOTION_TRIGGERS};

/// Parse the static object body of a `motion:trigger="…"` directive.
/// Accepted shape: `{ <bare-identifier>: '<string-literal>' (, …) }`.
/// Tolerant of whitespace and trailing commas. Anything more complex
/// (numeric / boolean / nested object values, quoted keys, etc.) is a
/// hard error with a byte-offset span.
///
/// Why hand-rolled: the compiler crate is WASM-safe and cannot pull in
/// `serde_json`. The grammar is small and stable; we already have a
/// similar hand-rolled parser in `lower/presets.rs`.
fn parse_motion_static_config(
    raw: &str,
    value_span: Span,
    loc: &str,
) -> Result<(Vec<MotionProp>, Option<u32>), CompileError> {
    let err = |msg: String| -> CompileError {
        CompileError::new(msg).with_span(value_span).with_category(loc.to_string())
    };
    let s = raw.trim();
    let bytes = s.as_bytes();
    let mut i = 0usize;
    // Skip leading whitespace (already done by `trim`).
    if bytes.first() != Some(&b'{') {
        return Err(err(format!(
            "motion config must be a static object literal, got `{raw}`"
        )));
    }
    i += 1;
    // Future improvement: translate any inner parse offset into a
    // Span pointing at the offending byte. We don't render that
    // today — the directive's `value_span` is precise enough.
    let mut props: Vec<MotionProp> = Vec::new();
    let mut stagger_ms: Option<u32> = None;
    loop {
        i = skip_ws(bytes, i);
        if i >= bytes.len() {
            return Err(err("unterminated motion config — expected `}`".into()));
        }
        if bytes[i] == b'}' {
            break;
        }
        // Allow trailing comma before `}`.
        if bytes[i] == b',' {
            i += 1;
            i = skip_ws(bytes, i);
            if i < bytes.len() && bytes[i] == b'}' {
                break;
            }
            // Otherwise it's a leading-comma mistake; surface it.
            return Err(err(format!(
                "unexpected `,` in motion config near `{}`",
                &s[i..s.len().min(i + 20)]
            )));
        }
        // Parse key — bare identifier only.
        let key_start = i;
        let key_end = scan_ident(bytes, i).ok_or_else(|| {
            err(format!(
                "expected a property name (scale, opacity, …) near `{}`",
                &s[key_start..s.len().min(key_start + 20)]
            ))
        })?;
        let property = &s[key_start..key_end];
        // `stagger` is a RESERVED key, not a CSS property: it describes the
        // relationship BETWEEN sibling elements (the delay step between them), not
        // anything about any one of them. This is the orchestration primitive — the
        // difference between a list that appears and a list that arrives.
        let is_stagger = property == MOTION_STAGGER_KEY;
        if !is_stagger && !MOTION_PROPERTIES.contains(&property) {
            return Err(err(format!(
                "motion property `{property}` is not supported. Supported properties: {}, \
                 plus the reserved `stagger` (e.g. `stagger: '40ms'`).",
                MOTION_PROPERTIES.join(", ")
            )));
        }
        i = skip_ws(bytes, key_end);
        if i >= bytes.len() || bytes[i] != b':' {
            return Err(err(format!(
                "expected `:` after `{property}` in motion config"
            )));
        }
        i = skip_ws(bytes, i + 1);
        // Parse value — single- or double-quoted string literal.
        let q = *bytes.get(i).ok_or_else(|| {
            err(format!(
                "expected a preset name (e.g. 'sm') after `{property}:`"
            ))
        })?;
        if q != b'"' && q != b'\'' {
            return Err(err(format!(
                "motion config value for `{property}` must be a quoted string literal \
                 (e.g. `{property}: 'sm'`); got `{raw}`"
            )));
        }
        i += 1;
        let val_start = i;
        while i < bytes.len() && bytes[i] != q {
            if bytes[i] == b'\\' && i + 1 < bytes.len() {
                // Skip the escape — we don't try to interpret it; the
                // preset name is consumed as raw text and resolved
                // later against the presets JSON.
                i += 2;
                continue;
            }
            i += 1;
        }
        if i >= bytes.len() {
            return Err(err(format!(
                "unterminated string literal in motion config for `{property}`"
            )));
        }
        let preset = &s[val_start..i];
        i += 1; // consume the closing quote
        if is_stagger {
            stagger_ms = Some(parse_duration_ms(preset).ok_or_else(|| {
                err(format!(
                    "stagger must be a duration like '40ms' or '0.04s', got `{preset}`"
                ))
            })?);
        } else {
            props.push(MotionProp {
                property: property.to_string(),
                preset: preset.to_string(),
            });
        }
        i = skip_ws(bytes, i);
        // Expect `,` or `}`.
        if i < bytes.len() && bytes[i] == b',' {
            i += 1;
            continue;
        }
        if i < bytes.len() && bytes[i] == b'}' {
            break;
        }
        return Err(err(format!(
            "expected `,` or `}}` after `{property}: '{preset}'`"
        )));
    }
    // Allow trailing whitespace + optional closing brace + trailing
    // whitespace — already consumed.
    if props.is_empty() {
        return Err(err("motion config must declare at least one property".into()));
    }
    Ok((props, stagger_ms))
}

/// `'40ms'` / `'0.04s'` → milliseconds. The only two units that mean anything for a
/// stagger step; anything else is a typo, so it fails rather than guessing.
fn parse_duration_ms(raw: &str) -> Option<u32> {
    let t = raw.trim();
    let (num, mult) = if let Some(n) = t.strip_suffix("ms") {
        (n, 1.0)
    } else if let Some(n) = t.strip_suffix('s') {
        (n, 1000.0)
    } else {
        return None;
    };
    let v: f64 = num.trim().parse().ok()?;
    let ms = v * mult;
    if !(0.0..=10_000.0).contains(&ms) {
        return None;
    }
    Some(ms.round() as u32)
}

fn skip_ws(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() && (bytes[i] as char).is_whitespace() {
        i += 1;
    }
    i
}

fn scan_ident(bytes: &[u8], mut i: usize) -> Option<usize> {
    let first = *bytes.get(i)?;
    if !(first.is_ascii_alphabetic() || first == b'_' || first == b'$') {
        return None;
    }
    i += 1;
    while i < bytes.len()
        && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'-')
    {
        i += 1;
    }
    Some(i)
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

    // ── Fix 1: fail-loud on malformed `{…}` attribute bodies ─────────────
    //
    // An attribute value is classified as an expression binding iff it
    // `trim().starts_with('{') && ends_with('}')`. When the brace body
    // is not actually a JS expression — or when it's an object literal
    // the author almost certainly intended as a JSON-string literal —
    // we MUST raise a CompileError with a span and a helpful hint.
    // Silent bad output (invalid JS at runtime) is the bug.

    #[test]
    fn attr_value_json_object_literal_errors_with_span_and_hint() {
        // `data='{"a":1}'` — author meant a string, compiler would
        // otherwise emit `bindAttr(el, "data", () => ({"a":1}))` and
        // the runtime would assign an object to the attribute. The
        // brace body parses as a JS expression (object literal), but
        // the author's intent is the literal string. Fail loud.
        let err = compile_attr("data", "{\"a\":1}", Span::new(0, 9), "[<el> data]", false)
            .expect_err("JSON-object attribute value must error, not silently emit");
        let msg = err.message.as_str();
        // Names the offending attribute.
        assert!(
            msg.contains("data"),
            "error must name the attribute `data`, got: {msg}"
        );
        // Shows the offending source so the author can spot it.
        assert!(
            msg.contains("{\"a\":1}") || msg.contains("{"),
            "error must echo the offending source, got: {msg}"
        );
        // Points at the author's likely fix.
        assert!(
            msg.contains("string") || msg.contains("bind") || msg.contains("escape"),
            "error must hint at the fix, got: {msg}"
        );
        // Has a span pointing at the value.
        assert!(err.span.is_some(), "error must carry a span");
        assert_eq!(err.span.unwrap(), Span::new(0, 9));
    }

    #[test]
    fn attr_value_garbage_in_braces_errors_with_span_and_hint() {
        // `attr={`"a":1,}` — trailing comma — not a valid object
        // literal. The expression parser will reject it. The error
        // must surface at this site (not as a cryptic downstream
        // parse error) with the attribute name and a fix hint.
        let err = compile_attr("attr", "{\"a\":1,}", Span::new(0, 10), "[<el> attr]", false)
            .expect_err("malformed object literal in attribute must error");
        let msg = err.message.as_str();
        assert!(msg.contains("attr"), "must name the attribute, got: {msg}");
        assert!(
            msg.contains("string") || msg.contains("bind") || msg.contains("escape"),
            "must hint at the fix, got: {msg}"
        );
        assert!(err.span.is_some(), "must carry a span");
    }

    #[test]
    fn attr_value_unparseable_braces_errors_not_panics() {
        // `{@!#}` — gibberish. Must produce a CompileError, never panic.
        let err = compile_attr("aria", "{@!#}", Span::new(0, 5), "[<el> aria]", false)
            .expect_err("gibberish in braces must error");
        let msg = err.message.as_str();
        assert!(msg.contains("aria"), "must name the attribute, got: {msg}");
        assert!(err.span.is_some(), "must carry a span");
    }

    #[test]
    fn attr_value_valid_expression_unchanged() {
        // Happy path: `attr={count}` keeps compiling as an expression
        // binding. Don't regress.
        let instr =
            compile_attr("disabled", "{count}", Span::new(0, 7), "[<el> disabled]", false)
                .expect("valid expression attr should compile");
        match instr {
            AttrInstr::Expr { ref name, ref expr, .. } => {
                assert_eq!(name, "disabled");
                assert_eq!(expr.source, "count");
            }
            other => panic!("expected AttrInstr::Expr, got {other:?}"),
        }
    }

    #[test]
    fn attr_value_call_expression_unchanged() {
        // `attr={signal(0)}` — call expression, not an object literal.
        // Must compile as expression binding (this is the intended
        // Svelte/JSX-like shape).
        let instr =
            compile_attr("value", "{signal(0)}", Span::new(0, 10), "[<el> value]", false)
                .expect("call-expression attr should compile");
        match instr {
            AttrInstr::Expr { ref expr, .. } => assert_eq!(expr.source, "signal(0)"),
            other => panic!("expected AttrInstr::Expr, got {other:?}"),
        }
    }

    #[test]
    fn attr_value_member_expression_unchanged() {
        // `attr={state.user.name}` — member access. Must compile.
        let instr = compile_attr(
            "title",
            "{state.user.name}",
            Span::new(0, 17),
            "[<el> title]",
            false,
        )
        .expect("member-expression attr should compile");
        match instr {
            AttrInstr::Expr { ref expr, .. } => assert_eq!(expr.source, "state.user.name"),
            other => panic!("expected AttrInstr::Expr, got {other:?}"),
        }
    }

    #[test]
    fn attr_value_string_literal_inside_braces_still_compiles() {
        // `attr={"hello"}` — a string-literal expression. The braces
        // wrap an expression, not an object literal — must compile.
        let instr = compile_attr(
            "placeholder",
            "{\"hello\"}",
            Span::new(0, 9),
            "[<el> placeholder]",
            false,
        )
        .expect("string-literal expression attr should compile");
        match instr {
            AttrInstr::Expr { ref expr, .. } => assert_eq!(expr.source, "\"hello\""),
            other => panic!("expected AttrInstr::Expr, got {other:?}"),
        }
    }

    #[test]
    fn attr_value_empty_braces_errors_with_hint() {
        // `attr={}` — empty brace body. Must error.
        let err = compile_attr("data", "{}", Span::new(0, 2), "[<el> data]", false)
            .expect_err("empty brace body must error");
        let msg = err.message.as_str();
        assert!(msg.contains("data"), "must name the attribute, got: {msg}");
        assert!(err.span.is_some(), "must carry a span");
    }
}
