//! Attribute classifier. Maps each attribute on an element to an
//! `AttrInstr`.

use crate::diagnostics::{CompileError, Span};
use crate::ir::*;
use crate::parse::expr;
use crate::parse::html;

const RULE_ID: &str = "sail-template-attr";
pub fn compile_attrs(el: &html::Element, loc_prefix: &str) -> Result<Vec<AttrInstr>, CompileError> {
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
        return Err(foreign_idiom_error("part=\"…\"", value_span, loc));
    }
    if name == "reflect-empty" {
        return compile_reflect_empty(value, value_span, loc, on_slot);
    }
    if name == "mutation" {
        if value.is_empty() {
            return Err(
                CompileError::new("mutation= requires a value (e.g. \"demo.addTodo\")")
                    .with_span(value_span)
                    .with_category(RULE_ID),
            );
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
    // Compiler-private Style motion handoff (preprocess inject). Not an
    // author attribute — authored `motion:` / `fx:` fall through as unknown.
    if let Some(rest) = name.strip_prefix("_sail_m_") {
        return compile_motion(rest, value, value_span, loc);
    }
    if name.starts_with("class:") {
        return Err(foreign_idiom_error(
            "class= / class:<name>",
            value_span,
            loc,
        ));
    }
    if name.starts_with("style:") {
        return Err(foreign_idiom_error(
            "style= / style:<prop>",
            value_span,
            loc,
        ));
    }
    if let Some(rest) = name.strip_prefix("use:") {
        return compile_use(rest, value, value_span, loc);
    }
    // Unknown colon-prefix: not an active directive. Foreign idioms already
    // handled above; everything else is generic unknown vocabulary.
    if let Some((prefix, _)) = name.split_once(':') {
        if !matches!(prefix, "on" | "bind" | "use") && !prefix.is_empty() {
            return Err(unknown_directive_error(name, value_span, loc));
        }
    }
    // Own-API ghost: no special tombstone — generic unknown attribute.
    if name == "sx" {
        return Err(CompileError::new("unknown attribute `sx`")
            .with_span(value_span)
            .with_category(RULE_ID));
    }
    if is_expr_value(value) {
        let inner = strip_braces(value).to_string();
        // Callback props (`on-select={fn}`, `on-success={…}`) accept the same
        // expression subset as `on:` handlers — including arrow functions.
        // They bind as identity properties (bindProp), not event listeners.
        if name.starts_with("on-") {
            expr::validate_at(&inner, true, loc, value_span)?;
            return Ok(AttrInstr::Expr {
                name: name.to_string(),
                expr: Expr::new(inner, value_span, true),
                span: value_span,
            });
        }
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
                         for a DOM event use `on:event={{...}}`; for a component moment \
                         use a typed callback prop `on-select={{...}}`"
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
                return Err(CompileError::new(msg)
                    .with_span(value_span)
                    .with_category(RULE_ID));
            }
        };
        // Object AST with an explicit `key: value` entry is the
        // JSON-as-string mistake (`data='{"a":1}'`) — fail loud
        // with a fix hint. Bare shorthand entries (`{count}`,
        // `{...spread}`, `{"hello"}`) are normal JSX-style
        // JSX-style bindings and pass through.
        if let crate::parse::expr::AstKind::Object(entries) = &ast.kind {
            let has_key_value = entries.iter().any(|e| {
                matches!(
                    e.key,
                    crate::parse::expr::ObjKey::Ident(_) | crate::parse::expr::ObjKey::String(_)
                ) && e.value.is_some()
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
                    .with_category(RULE_ID));
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

/// Accept `sx={…}` / `sx="…"` without JS grammar. Style is lowered later
/// (`preprocess` / `lower_sx`); author IR only needs an opaque body.
///
/// Stored as [`AttrInstr::Expr`] (name `"sx"`) so existing author-tree walkers
/// keep seeing a dynamic attr without a new IR variant. Body is **not** JS —
/// consumers must not assume expression subset validity for `name == "sx"`.
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
            .with_category(RULE_ID));
    }
    if !crate::directive_manifest::is_known_event(&event) {
        let mut msg = format!("unknown event {event}");
        if let Some(cb) = crate::directive_manifest::custom_event_callback_hint(&event) {
            msg.push_str(&format!(
                " — custom events are retired; use the callback prop `{cb}`. Props flow down, callbacks report up; there is no event bus."
            ));
        } else if let Some(hint) = crate::directive_manifest::suggest_event(&event) {
            msg.push_str(&format!(" — did you mean on:{hint}?"));
        } else {
            msg.push_str(
                " — on: accepts standard DOM event names only. Component moments use on-* callback props (e.g. on-select={{fn}}); form mutation uses on-success/on-failure.",
            );
        }
        return Err(CompileError::new(msg)
            .with_span(value_span)
            .with_category(RULE_ID));
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
            .with_category(RULE_ID));
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

fn parse_modifier(raw: &str, span: Span, _loc: &str) -> Result<Modifier, CompileError> {
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
            let known = crate::directive_manifest::EVENT_MODIFIERS
                .iter()
                .map(|m| match m.alias {
                    Some(a) => format!("{} ({a})", m.canonical),
                    None => m.canonical.to_string(),
                })
                .collect::<Vec<_>>()
                .join(", ");
            return Err(CompileError::new(format!(
                "unknown modifier \"{head}\" — the modifiers are: {known}"
            ))
            .with_span(span)
            .with_category(RULE_ID));
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
        return Err(
            CompileError::new(format!("bind:{target} expects {{signal}}"))
                .with_span(value_span)
                .with_category(RULE_ID),
        );
    }
    let inner = strip_braces(value).to_string();
    expr::validate_at(&inner, false, loc, value_span)?;
    let expr = Expr::new(inner, value_span, false);
    match target {
        "value" => Ok(AttrInstr::BindValue {
            expr,
            span: value_span,
        }),
        "checked" => Ok(AttrInstr::BindChecked {
            expr,
            span: value_span,
        }),
        other => {
            let known = crate::directive_manifest::BIND_TARGETS
                .iter()
                .map(|b| b.target)
                .collect::<Vec<_>>()
                .join(", ");
            Err(
                CompileError::new(format!("unknown bind:{other} — the targets are: {known}"))
                    .with_span(value_span)
                    .with_category(RULE_ID),
            )
        }
    }
}

fn foreign_idiom_error(syntax_key: &str, value_span: Span, _loc: &str) -> CompileError {
    let instead = crate::directive_manifest::FOREIGN_IDIOMS
        .iter()
        .find(|e| e.syntax == syntax_key)
        .map(|e| e.instead)
        .unwrap_or("use the sail form from the language surface");
    CompileError::new(format!("`{syntax_key}` is a foreign idiom — {instead}"))
        .with_span(value_span)
        .with_category(RULE_ID)
}

fn unknown_directive_error(name: &str, value_span: Span, _loc: &str) -> CompileError {
    let known = crate::directive_manifest::DIRECTIVE_PREFIXES.join(":, ");
    CompileError::new(format!(
        "unknown directive `{name}` — attribute directives use the prefixes: {known}:… \
         (plus sx=, mutation=, slot=). Motion is Style: sx={{style.<trigger>({{…}})}}."
    ))
    .with_span(value_span)
    .with_category(RULE_ID)
}

fn compile_reflect_empty(
    value: &str,
    value_span: Span,
    _loc: &str,
    on_slot: bool,
) -> Result<AttrInstr, CompileError> {
    if !on_slot {
        return Err(
            CompileError::new("reflect-empty=\"...\" is only valid on <slot> elements")
                .with_span(value_span)
                .with_category(RULE_ID),
        );
    }
    if is_expr_value(value) {
        return Err(CompileError::new(
            "reflect-empty must be a static attribute name (e.g. \"data-icon-only\"), not an expression",
        )
        .with_span(value_span)
        .with_category(RULE_ID));
    }
    if value.is_empty() {
        return Err(
            CompileError::new("reflect-empty=\"<attr>\" requires a target attribute name")
                .with_span(value_span)
                .with_category(RULE_ID),
        );
    }
    if !is_valid_attr_name(value) {
        return Err(CompileError::new(format!(
            "reflect-empty target \"{value}\" must match [a-z][a-z0-9-]*"
        ))
        .with_span(value_span)
        .with_category(RULE_ID));
    }
    Ok(AttrInstr::SlotReflectEmpty {
        attr: value.to_string(),
        span: value_span,
    })
}

fn is_bare_identifier(s: &str) -> bool {
    let s = s.trim();
    let mut chars = s.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_' || first == '$') {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

fn is_valid_attr_name(s: &str) -> bool {
    let mut chars = s.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_lowercase() {
        return false;
    }
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
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
        CompileError::new(msg)
            .with_span(value_span)
            .with_category(RULE_ID)
    };
    if name.is_empty() {
        return Err(err("use: requires an action name".into()));
    }
    if !is_bare_identifier(name) {
        return Err(err(format!(
            "action name \"{name}\" must be a bare identifier"
        )));
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
    Ok(AttrInstr::Use {
        action: name.to_string(),
        param,
        span: value_span,
    })
}
/// Compile a motion attach config from the compiler-private `_sail_m_<trigger>`
/// handoff produced by Style `.enter` / `.exit` / `.loop` / `.reveal` /
/// `.scroll({…})` lowering. Authors write Style methods, never this attr.
///
/// Accepts only a static object literal; dynamic expressions are rejected so
/// the runtime never has to evaluate an animation config.
fn compile_motion(
    trigger: &str,
    value: &str,
    value_span: Span,
    loc: &str,
) -> Result<AttrInstr, CompileError> {
    let err = |msg: String| {
        CompileError::new(msg)
            .with_span(value_span)
            .with_category(RULE_ID)
    };
    if trigger.is_empty() {
        return Err(err(
            "motion attach requires a trigger name (enter, exit, loop, hover, press, …)".into(),
        ));
    }
    if !MOTION_TRIGGERS.contains(&trigger) {
        return Err(err(format!(
            "motion trigger `{trigger}` is not supported. Triggers: {}.",
            MOTION_TRIGGERS.join(", ")
        )));
    }
    if value.is_empty() {
        return Err(err("motion attach requires a config object".into()));
    }

    fn has_static_shape(body: &str) -> bool {
        let s = body.trim();
        s.starts_with('{') && s.contains(':') && s.ends_with('}')
    }
    let trimmed = value.trim_start();
    let mut base = value.len() - trimmed.len();
    let stripped = trimmed.trim_end();
    let stripped = match stripped.strip_prefix('{') {
        Some(rest) => {
            base += 1;
            rest
        }
        None => stripped,
    };
    let stripped = stripped.strip_suffix('}').unwrap_or(stripped);
    let (props, stagger_ms, timing) =
        match parse_motion_static_config(stripped, value_span, base, loc) {
            Ok(parsed) => parsed,
            Err(_) if !has_static_shape(value) => {
                return Err(err(format!(
                    "style.{trigger}({{...}}) — the value must be a config object like \
                     `style.{trigger}({{ scale: 'pop' }})`. Fields may be inline values \
                     (`duration: 3600`) or signals (`duration: speed()`), but the config \
                     itself is written out, not passed in."
                )));
            }
            Err(parse_err) => return Err(parse_err),
        };
    Ok(AttrInstr::Motion {
        trigger: trigger.to_string(),
        props,
        stagger_ms,
        timing,
        span: value_span,
    })
}

// The property vocabulary lives in sail-lint — ONE list, two readers.
use sail_lint::rules::animate_token_only::{
    MOTION_CHANNEL_KEYS, MOTION_PROPERTIES, MOTION_STAGGER_KEY, MOTION_TIMING_KEYS, MOTION_TRIGGERS,
};

/// Parse the static object body of a Style motion config (`style.<trigger>({…})`).
///
/// The grammar, in one sentence: **quoted is a literal, unquoted is code.**
///
/// ```text
/// config  := '{' (entry (',' entry)* ','?)? '}'
/// entry   := property ':' (preset | channel)
///          | timing-key ':' value
///          | 'stagger' ':' (duration-string | number)
/// channel := '{' (('from'|'to'|'preset'|timing-key) ':' value ...) '}'
/// value   := string | number | bezier | expression
/// bezier  := '[' num ',' num ',' num ',' num ']'
/// ```
///
/// A bare number in a time field is MILLISECONDS; in a magnitude field it
/// is the channel's native unit (px / deg / unitless). A quoted string may
/// carry its own unit (`'120px'`, `'-12%'`). Anything unquoted that is not
/// a number or a bezier is an expression — that is how a signal gets in.
///
/// Why hand-rolled: the compiler crate is WASM-safe and cannot pull in
/// `serde_json`. We already have a similar hand-rolled parser in
/// `lower/presets.rs`.
fn parse_motion_static_config(
    raw: &str,
    value_span: Span,
    base: usize,
    loc: &str,
) -> Result<(Vec<MotionProp>, Option<u32>, MotionTiming), CompileError> {
    let err = |msg: String| -> CompileError {
        CompileError::new(msg)
            .with_span(value_span)
            .with_category(RULE_ID)
    };
    let leading = raw.len() - raw.trim_start().len();
    let s = raw.trim();
    let base = base + leading;
    let bytes = s.as_bytes();
    let mut i = 0usize;
    if bytes.first() != Some(&b'{') {
        return Err(err(format!(
            "motion config must be an object literal, got `{raw}`"
        )));
    }
    i += 1;
    let mut props: Vec<MotionProp> = Vec::new();
    let mut stagger_ms: Option<u32> = None;
    let mut timing = MotionTiming::default();
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
            return Err(err(format!(
                "unexpected `,` in motion config near `{}`",
                context_window(s, i)
            )));
        }
        let key_start = i;
        let key_end = scan_ident(bytes, i).ok_or_else(|| {
            err(format!(
                "expected a property name (scale, opacity, …) near `{}`",
                context_window(s, key_start)
            ))
        })?;
        let key = &s[key_start..key_end];
        // `stagger` is a RESERVED key, not a CSS property: it describes the
        // relationship BETWEEN sibling elements (the delay step between them), not
        // anything about any one of them. This is the orchestration primitive — the
        // difference between a list that appears and a list that arrives.
        let is_stagger = key == MOTION_STAGGER_KEY;
        let is_timing = MOTION_TIMING_KEYS.contains(&key);
        let is_property = MOTION_PROPERTIES.contains(&key);
        if !is_stagger && !is_timing && !is_property {
            return Err(err(format!(
                "motion config key `{key}` is not supported. Animated properties: {}. \
                 Config-level timing: {}. Plus the reserved `stagger` (e.g. `stagger: '40ms'`).",
                MOTION_PROPERTIES.join(", "),
                MOTION_TIMING_KEYS.join(", "),
            )));
        }
        i = skip_ws(bytes, key_end);
        if i >= bytes.len() || bytes[i] != b':' {
            return Err(err(format!("expected `:` after `{key}` in motion config")));
        }
        i = skip_ws(bytes, i + 1);
        if i >= bytes.len() {
            return Err(err(format!("expected a value after `{key}:`")));
        }

        if is_property && bytes[i] == b'{' {
            // A per-channel object: `translate-x: { from: 0, to: '120px' }`.
            let (prop, next) = parse_motion_channel(key, s, bytes, i, base, value_span, loc)?;
            props.push(prop);
            i = next;
        } else {
            let (val, next) = parse_motion_value(key, s, bytes, i, base, value_span, loc)?;
            i = next;
            if is_stagger {
                stagger_ms = Some(motion_value_to_stagger_ms(&val).ok_or_else(|| {
                    err(format!(
                        "stagger must be a duration like '40ms' / '0.04s' or a number of \
                         milliseconds (e.g. `stagger: 450`), got `{}`",
                        describe_motion_value(&val)
                    ))
                })?);
            } else if is_timing {
                assign_timing(&mut timing, key, val, &err)?;
            } else {
                // `scale: 'pop'` — a preset name, or an expression naming one.
                match val {
                    MotionValue::Str(preset) => props.push(MotionProp {
                        property: key.to_string(),
                        preset: Some(preset),
                        ..MotionProp::default()
                    }),
                    other => {
                        return Err(err(format!(
                            "fx property `{key}` takes a preset name (`{key}: 'fade'`) or an \
                             inline object (`{key}: {{ from: 0, to: 1 }}`), got `{}`. \
                             A bare number is a timing value, not a magnitude — put it in \
                             `from`/`to`.",
                            describe_motion_value(&other)
                        )));
                    }
                }
            }
        }

        i = skip_ws(bytes, i);
        if i < bytes.len() && bytes[i] == b',' {
            i += 1;
            continue;
        }
        if i < bytes.len() && bytes[i] == b'}' {
            break;
        }
        return Err(err(format!(
            "expected `,` or `}}` after the value for `{key}`"
        )));
    }
    if props.is_empty() {
        return Err(err(
            "motion config must declare at least one animated property \
             (e.g. `scale: 'pop'` or `scale: { to: 1.75 }`)"
                .into(),
        ));
    }
    Ok((props, stagger_ms, timing))
}

/// Parse `{ from: 0, to: '120px', duration: 3600 }` — the per-channel
/// object form. Returns the channel and the offset just past its `}`.
#[allow(clippy::too_many_arguments)]
fn parse_motion_channel(
    property: &str,
    s: &str,
    bytes: &[u8],
    start: usize,
    base: usize,
    value_span: Span,
    loc: &str,
) -> Result<(MotionProp, usize), CompileError> {
    let err = |msg: String| -> CompileError {
        CompileError::new(msg)
            .with_span(value_span)
            .with_category(RULE_ID)
    };
    let mut prop = MotionProp {
        property: property.to_string(),
        ..MotionProp::default()
    };
    let mut i = start + 1; // consume `{`
    loop {
        i = skip_ws(bytes, i);
        if i >= bytes.len() {
            return Err(err(format!(
                "unterminated inline object for `{property}` — expected `}}`"
            )));
        }
        if bytes[i] == b'}' {
            i += 1;
            break;
        }
        if bytes[i] == b',' {
            i += 1;
            i = skip_ws(bytes, i);
            if i < bytes.len() && bytes[i] == b'}' {
                i += 1;
                break;
            }
            return Err(err(format!(
                "unexpected `,` in the object for `{property}`"
            )));
        }
        let key_start = i;
        let key_end = scan_ident(bytes, i).ok_or_else(|| {
            err(format!(
                "expected `from`, `to`, `preset`, or a timing key inside `{property}`, near `{}`",
                context_window(s, key_start)
            ))
        })?;
        let key = &s[key_start..key_end];
        if !MOTION_CHANNEL_KEYS.contains(&key) && !MOTION_TIMING_KEYS.contains(&key) {
            return Err(err(format!(
                "`{key}` is not a field of an inline motion object. Fields: {}, {}.",
                MOTION_CHANNEL_KEYS.join(", "),
                MOTION_TIMING_KEYS.join(", "),
            )));
        }
        i = skip_ws(bytes, key_end);
        if i >= bytes.len() || bytes[i] != b':' {
            return Err(err(format!("expected `:` after `{key}` in `{property}`")));
        }
        i = skip_ws(bytes, i + 1);
        let (val, next) = parse_motion_value(key, s, bytes, i, base, value_span, loc)?;
        i = next;
        match key {
            "from" => prop.from = Some(val),
            "to" => prop.to = Some(val),
            "preset" => match val {
                MotionValue::Str(p) => prop.preset = Some(p),
                other => {
                    return Err(err(format!(
                        "`preset` inside `{property}` must be a quoted preset name \
                         (e.g. `preset: 'pop'`), got `{}`",
                        describe_motion_value(&other)
                    )));
                }
            },
            _ => assign_timing(&mut prop.timing, key, val, &err)?,
        }
        i = skip_ws(bytes, i);
        if i < bytes.len() && bytes[i] == b',' {
            i += 1;
            continue;
        }
        if i < bytes.len() && bytes[i] == b'}' {
            i += 1;
            break;
        }
        return Err(err(format!(
            "expected `,` or `}}` after `{key}` in `{property}`"
        )));
    }
    if prop.preset.is_none() && prop.from.is_none() && prop.to.is_none() {
        return Err(err(format!(
            "`{property}: {{ … }}` declares no motion — give it a `to` \
             (e.g. `{property}: {{ to: 1.75 }}`) or a `preset`."
        )));
    }
    Ok((prop, i))
}

fn assign_timing(
    timing: &mut MotionTiming,
    key: &str,
    val: MotionValue,
    err: &dyn Fn(String) -> CompileError,
) -> Result<(), CompileError> {
    // Reject the shapes that would silently mean the wrong thing, and name
    // the right form in the message — the rejection IS the documentation.
    match key {
        "duration" | "delay" => match &val {
            MotionValue::Num(n) if *n < 0.0 => {
                return Err(err(format!("`{key}` must not be negative, got {n}")));
            }
            MotionValue::Bezier(_) => {
                return Err(err(format!(
                    "`{key}` is a number of milliseconds (e.g. `{key}: 3600`), not a bezier"
                )));
            }
            _ => {}
        },
        "easing" => match &val {
            MotionValue::Num(_) => {
                return Err(err(
                    "`easing` is a keyword (`'in-out'`, `'linear'`) or an inline bezier \
                     (`[0.6, 0, 0.2, 1]`), not a number"
                        .into(),
                ));
            }
            _ => {}
        },
        _ => {}
    }
    let slot = match key {
        "duration" => &mut timing.duration,
        "delay" => &mut timing.delay,
        "easing" => &mut timing.easing,
        "iterations" => &mut timing.iterations,
        "direction" => &mut timing.direction,
        _ => return Err(err(format!("unknown timing key `{key}`"))),
    };
    if slot.is_some() {
        return Err(err(format!(
            "`{key}` is set twice in the same motion config"
        )));
    }
    *slot = Some(val);
    Ok(())
}

/// Parse one value. Returns the value and the offset just past it.
#[allow(clippy::too_many_arguments)]
fn parse_motion_value(
    key: &str,
    s: &str,
    bytes: &[u8],
    start: usize,
    base: usize,
    value_span: Span,
    loc: &str,
) -> Result<(MotionValue, usize), CompileError> {
    let err = |msg: String| -> CompileError {
        CompileError::new(msg)
            .with_span(value_span)
            .with_category(RULE_ID)
    };
    let mut i = start;
    let c = *bytes
        .get(i)
        .ok_or_else(|| err(format!("expected a value after `{key}:`")))?;

    // 1. Quoted string — a literal.
    if c == b'"' || c == b'\'' {
        let q = c;
        i += 1;
        let val_start = i;
        while i < bytes.len() && bytes[i] != q {
            if bytes[i] == b'\\' && i + 1 < bytes.len() {
                i += 2;
                continue;
            }
            i += 1;
        }
        if i >= bytes.len() {
            return Err(err(format!(
                "unterminated string literal in motion config for `{key}`"
            )));
        }
        let text = s[val_start..i].to_string();
        return Ok((MotionValue::Str(text), i + 1));
    }

    // 2. Inline cubic-bezier — `[0.6, 0, 0.2, 1]`.
    if c == b'[' {
        let close = find_matching(bytes, i, b'[', b']')
            .ok_or_else(|| err(format!("unterminated `[` in motion config for `{key}`")))?;
        let inner = &s[i + 1..close];
        let nums: Vec<&str> = inner
            .split(',')
            .map(str::trim)
            .filter(|p| !p.is_empty())
            .collect();
        if nums.len() != 4 {
            return Err(err(format!(
                "`{key}` as an inline easing must be a cubic-bezier of exactly four numbers \
                 (e.g. `{key}: [0.6, 0, 0.2, 1]`), got {} value(s)",
                nums.len()
            )));
        }
        let mut out = [0.0f64; 4];
        for (slot, raw) in out.iter_mut().zip(nums.iter()) {
            *slot = raw.parse::<f64>().map_err(|_| {
                err(format!(
                    "`{key}` bezier control points must be numbers, got `{raw}`"
                ))
            })?;
        }
        // x1/x2 outside 0..=1 is not a curve CSS will accept — catching it here
        // beats a silent no-op animation in the browser.
        if !(0.0..=1.0).contains(&out[0]) || !(0.0..=1.0).contains(&out[2]) {
            return Err(err(format!(
                "cubic-bezier x control points must be within 0..1 \
                 (got {} and {}); y may overshoot",
                out[0], out[2]
            )));
        }
        return Ok((MotionValue::Bezier(out), close + 1));
    }

    // 3. A bare number.
    let num_end = scan_number(bytes, i);
    if let Some(end) = num_end {
        // Only a number if the token ENDS here — `3600ms` unquoted is a
        // mistake worth naming rather than silently reading as `3600`.
        let after = skip_ws(bytes, end);
        if after >= bytes.len() || bytes[after] == b',' || bytes[after] == b'}' {
            let n = s[i..end]
                .parse::<f64>()
                .map_err(|_| err(format!("could not read `{}` as a number", &s[i..end])))?;
            return Ok((MotionValue::Num(n), end));
        }
        if bytes[end].is_ascii_alphabetic() || bytes[end] == b'%' {
            let unit_end = scan_value_end(bytes, end);
            let written = s[i..unit_end].trim_end();
            return Err(err(format!(
                "`{key}: {written}` — a unit-bearing value must be quoted \
                 (`{key}: '{written}'`); an unquoted number is milliseconds for time \
                 and the channel's native unit for a magnitude."
            )));
        }
    }

    // 4. Anything else is an expression — this is how a signal gets in.
    let end = scan_value_end(bytes, i);
    let src = s[i..end].trim_end();
    if src.is_empty() {
        return Err(err(format!("expected a value after `{key}:`")));
    }
    let span_start = value_span.start + base + i;
    let expr_span = Span {
        start: span_start,
        end: span_start + src.len(),
    };
    // Validate through the same expression machinery every other reactive
    // attribute uses, so a typo inside a motion config reads like a typo
    // anywhere else.
    expr::grammar::parse_expression(src, span_start).map_err(|e| {
        CompileError::new(format!(
            "motion config value for `{key}` is neither a literal nor a valid expression: {}. \
             Quote it for a literal (`{key}: 'fade'`), or write a signal call \
             (`{key}: speed()`).",
            e.message
        ))
        .with_span(expr_span)
        .with_category(RULE_ID)
    })?;
    expr::validate_at(src, false, loc, expr_span)?;
    Ok((
        MotionValue::Expr(Expr::new(src.to_string(), expr_span, false)),
        end,
    ))
}

/// Scan to the end of a value: the next top-level `,` or `}`, respecting
/// nesting and quotes so `fn(a, b)` and `x['a,b']` survive intact.
fn scan_value_end(bytes: &[u8], mut i: usize) -> usize {
    let mut depth = 0i32;
    while i < bytes.len() {
        match bytes[i] {
            b'"' | b'\'' | b'`' => {
                let q = bytes[i];
                i += 1;
                while i < bytes.len() && bytes[i] != q {
                    if bytes[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
            }
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' => depth -= 1,
            b'}' if depth == 0 => return i,
            b'}' => depth -= 1,
            b',' if depth == 0 => return i,
            _ => {}
        }
        if i < bytes.len() {
            i += 1;
        }
    }
    i
}

fn find_matching(bytes: &[u8], start: usize, open: u8, close: u8) -> Option<usize> {
    let mut depth = 0i32;
    let mut i = start;
    while i < bytes.len() {
        let c = bytes[i];
        if c == open {
            depth += 1;
        } else if c == close {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

/// A JS number literal: optional sign, digits, optional fraction.
fn scan_number(bytes: &[u8], mut i: usize) -> Option<usize> {
    let start = i;
    if i < bytes.len() && (bytes[i] == b'-' || bytes[i] == b'+') {
        i += 1;
    }
    let digits_start = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i < bytes.len() && bytes[i] == b'.' {
        i += 1;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
    }
    if i == digits_start {
        return None;
    }
    let _ = start;
    Some(i)
}

fn describe_motion_value(v: &MotionValue) -> String {
    match v {
        MotionValue::Num(n) => format!("the number {n}"),
        MotionValue::Str(s) => format!("'{s}'"),
        MotionValue::Bezier(b) => format!("[{}, {}, {}, {}]", b[0], b[1], b[2], b[3]),
        MotionValue::Expr(e) => format!("the expression `{}`", e.source),
    }
}

/// `stagger` accepts the historical duration string AND a bare number of
/// milliseconds — `stagger: '450ms'` was already inline time, so the
/// number form is the same idea with one less ceremony.
fn motion_value_to_stagger_ms(v: &MotionValue) -> Option<u32> {
    match v {
        MotionValue::Str(s) => parse_duration_ms(s),
        MotionValue::Num(n) => {
            if !(0.0..=10_000.0).contains(n) {
                return None;
            }
            Some(n.round() as u32)
        }
        _ => None,
    }
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

/// Up to 20 chars of source starting at `i`, for "near `…`" diagnostics.
/// Char-counted, not byte-truncated — a fixed byte window (`i + 20`) can cut
/// a multibyte char in half and panic the slice.
fn context_window(s: &str, i: usize) -> &str {
    let rest = &s[i..];
    match rest.char_indices().nth(20) {
        Some((end, _)) => &rest[..end],
        None => rest,
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prop_prefix_is_unknown_directive() {
        let err = compile_attr(
            "prop:items",
            "{rows()}",
            Span::new(0, 8),
            "[grid prop:items]",
            false,
        )
        .unwrap_err();
        assert!(
            err.message.contains("unknown directive") && err.message.contains("prop:items"),
            "expected unknown directive, got: {}",
            err.message
        );
        assert!(!err.message.to_ascii_lowercase().contains("retired"));
    }

    #[test]
    fn attr_prefix_is_unknown_directive() {
        let err = compile_attr(
            "attr:hidden",
            "{h()}",
            Span::new(0, 5),
            "[div attr:hidden]",
            false,
        )
        .unwrap_err();
        assert!(
            err.message.contains("unknown directive") && err.message.contains("attr:hidden"),
            "expected unknown directive, got: {}",
            err.message
        );
        assert!(!err.message.to_ascii_lowercase().contains("retired"));
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
    fn sx_attr_host_is_unknown() {
        // STYLE-TAG-1: `sx=` special tombstone deleted — generic unknown attr.
        let err = compile_attr(
            "sx",
            "{style.fontSize(2xl)}",
            Span::new(0, 20),
            "[h1 sx]",
            false,
        )
        .expect_err("sx= is unknown attribute");
        assert!(
            err.message.contains("unknown attribute") && err.message.contains("sx"),
            "must be generic unknown attr (no retired tombstone), got: {}",
            err.message
        );
        assert!(
            !err.message.contains("retired") && !err.message.contains("block Style"),
            "tombstone teaching deleted, got: {}",
            err.message
        );
    }

    #[test]
    fn parse_author_keeps_tag_style_chain_with_scale_tokens() {
        // Tag-position object Style with when: lowers to classes before author
        // HTML parse — scale tokens must survive product Style parse.
        let src = r#"
            <h1 style({ box: padX(2_5), paint: fg(sidebar-foreground), text: fontSize(2xl), when: { hover: { paint: fg(foreground) } } })>              T
            </h1>
        "#;
        let pre = crate::parse::preprocess::preprocess(src).expect("preprocess tag Style");
        assert!(
            pre.contains("s-") || pre.contains("fontSize") || pre.contains("2xl"),
            "tag Style with scale tokens must lower:\n{pre}"
        );
        let _prog = crate::parse::parse_author(src)
            .expect("parse_author must accept tag-position Style product");
        assert!(
            pre.contains(":hover") || pre.contains("hover"),
            "object when: hover must lower from tag Style:\n{pre}"
        );
    }

    #[test]
    fn parse_author_still_rejects_leading_digit_in_non_sx_expr_attr() {
        // Opaque sx must not loosen ordinary expression attrs.
        let src = r#"<h1 title={fontSize(2xl)}>T</h1>"#;
        let err = crate::parse::parse_author(src)
            .expect_err("non-sx expr with Style-shaped body must still fail JS parse");
        assert!(
            !err.message.is_empty(),
            "must surface a compile error for title={{fontSize(2xl)}}"
        );
    }

    #[test]
    fn attr_value_valid_expression_unchanged() {
        // Happy path: `attr={count}` keeps compiling as an expression
        // binding. Don't regress.
        let instr = compile_attr(
            "disabled",
            "{count}",
            Span::new(0, 7),
            "[<el> disabled]",
            false,
        )
        .expect("valid expression attr should compile");
        match instr {
            AttrInstr::Expr {
                ref name, ref expr, ..
            } => {
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
        let instr = compile_attr(
            "value",
            "{signal(0)}",
            Span::new(0, 10),
            "[<el> value]",
            false,
        )
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
    fn context_window_never_cuts_a_multibyte_char() {
        // 7 × `…` = 21 bytes; the old fixed 20-BYTE window (`i + 20`) landed
        // mid-char and panicked the diagnostic that was trying to help.
        let s = "………………………";
        assert_eq!(context_window(s, 0), s, "under 20 chars: whole rest");
        let long = "…".repeat(30);
        assert_eq!(context_window(&long, 0), "…".repeat(20));
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

    // ── directive-manifest parity: the docs tables ARE the parser's tables ──

    #[test]
    fn every_manifest_modifier_parses_and_no_others() {
        let span = Span::new(0, 0);
        for m in crate::directive_manifest::EVENT_MODIFIERS {
            let raw = match m.arg {
                Some(_) => format!("{}:x", m.canonical),
                None => m.canonical.to_string(),
            };
            assert!(
                parse_modifier(&raw, span, "t").is_ok(),
                "manifest modifier `{}` is not accepted by the parser",
                m.canonical
            );
            if let Some(alias) = m.alias {
                assert!(
                    parse_modifier(alias, span, "t").is_ok(),
                    "manifest alias `{alias}` is not accepted by the parser"
                );
            }
        }
        let err = parse_modifier("bogus", span, "t").unwrap_err();
        for m in crate::directive_manifest::EVENT_MODIFIERS {
            assert!(
                err.message.contains(m.canonical),
                "unknown-modifier error must render the manifest list; missing `{}` in: {}",
                m.canonical,
                err.message
            );
        }
    }

    #[test]
    fn every_manifest_bind_target_parses_and_no_others() {
        let span = Span::new(0, 0);
        for b in crate::directive_manifest::BIND_TARGETS {
            assert!(
                compile_bind(b.target, "{sig}", span, "t").is_ok(),
                "manifest bind target `{}` is not accepted by the parser",
                b.target
            );
        }
        let err = compile_bind("bogus", "{sig}", span, "t").unwrap_err();
        for b in crate::directive_manifest::BIND_TARGETS {
            assert!(
                err.message.contains(b.target),
                "unknown-bind error must render the manifest list; missing `{}` in: {}",
                b.target,
                err.message
            );
        }
        // bind:this is not a bind target — generic unknown-bind (no tombstone).
        let unknown = compile_bind("this", "{el}", span, "t").unwrap_err();
        assert!(
            unknown.message.contains("unknown bind:this") && unknown.message.contains("value"),
            "bind:this must be generic unknown-bind, got: {}",
            unknown.message
        );
        assert!(
            !unknown.message.to_ascii_lowercase().contains("retired"),
            "must not embalm bind:this: {}",
            unknown.message
        );
    }

    #[test]
    fn every_manifest_event_compiles_and_unknown_suggests() {
        let span = Span::new(0, 0);
        for e in crate::directive_manifest::EVENT_NAMES {
            assert!(
                compile_on(e.name, "{handler}", span, "t").is_ok(),
                "manifest event `{}` is not accepted by the parser",
                e.name
            );
        }
        // Custom-event bus is gone (CALLBACKS-1) — DOM events only; sail-*/ship-*
        // names refuse with a callback-prop hint, not an escape hatch.
        let retired = compile_on("sail-rail:toggle", "{handler}", span, "t").unwrap_err();
        assert!(
            retired.message.contains("custom events are retired")
                || retired.message.contains("on-"),
            "sail-* events must refuse with callback hint, got: {}",
            retired.message
        );
        let retired_ship = compile_on("ship-thing:done", "{handler}", span, "t").unwrap_err();
        assert!(
            retired_ship.message.contains("custom events are retired")
                || retired_ship.message.contains("on-")
                || retired_ship.message.contains("unknown event"),
            "ship-* events must refuse, got: {}",
            retired_ship.message
        );

        let err = compile_on("clik", "{handler}", span, "t").unwrap_err();
        assert!(
            err.message.contains("unknown event clik"),
            "must name the bad event, got: {}",
            err.message
        );
        assert!(
            err.message.contains("did you mean")
                && (err.message.contains("click") || err.message.contains("on:click")),
            "must suggest click, got: {}",
            err.message
        );

        let err = compile_on("Click", "{handler}", span, "t").unwrap_err();
        assert!(
            err.message.contains("did you mean")
                && (err.message.contains("click") || err.message.contains("on:click")),
            "capitalization typo must suggest click, got: {}",
            err.message
        );
    }
}
