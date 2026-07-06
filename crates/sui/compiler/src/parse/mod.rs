//! Parse pipeline. Walks `.sui` source and produces an IR.
//!
//! Stages, in order:
//!  1. Preprocess — same two transforms the interpreter does
//!     (quote unquoted brace attrs, expand non-void self-closers).
//!  2. HTML parse — produces a span-bearing DOM-ish tree.
//!  3. Directive lift — pluck `<data>`, `<prop>`, and top-level
//!     `<style>` out of the top-level child list into the IR. A
//!     top-level `<state>` directive is rejected with a parse error
//!     pointing authors at the controller's `scope:` factory.
//!  4. Template grammar scan — text-node `{...}` markers become
//!     template instructions; element children recurse.
//!  5. Expression validate — every brace expression goes through the
//!     same forbidden-syntax checks the interpreter applies.

use crate::diagnostics::{CompileError, Span};
use crate::ir::*;

pub mod expr;
pub mod html;
pub mod preprocess;
pub mod template;

pub use html::{Attr, Node};

/// Parse a `.sui` source string into an IR program.
pub fn parse(source: &str) -> Result<Program, CompileError> {
    parse_inner(source, true)
}

/// Parse an ELEMENT source (element mode / the stdlib registry): identical
/// pipeline minus the stdlib-element expansion pass — an element source may
/// not use stdlib elements (no recursion, no element-in-element).
pub(crate) fn parse_element_source(source: &str) -> Result<Program, CompileError> {
    parse_inner(source, false)
}

fn parse_inner(source: &str, expand_elements: bool) -> Result<Program, CompileError> {
    let prepared = preprocess::preprocess_impl(source, expand_elements);
    let nodes = html::parse_fragment(&prepared)?;

    let mut data: Vec<DataDecl> = Vec::new();
    let mut props: Vec<PropDecl> = Vec::new();
    let mut clients: Vec<ClientDecl> = Vec::new();
    let mut style_parts: Vec<String> = Vec::new();
    let mut template_nodes: Vec<Node> = Vec::with_capacity(nodes.len());
    // Holds the most-recent JSDoc-style HTML comment so the next
    // `<prop>` can adopt it as its `description`. Reset by any
    // intervening non-whitespace node.
    let mut pending_jsdoc: Option<String> = None;

    for node in nodes {
        match &node {
            Node::Comment(c) => {
                if let Some(text) = extract_jsdoc(&c.data) {
                    pending_jsdoc = Some(text);
                }
                template_nodes.push(node);
                continue;
            }
            Node::Text(t) if t.data.trim().is_empty() => {
                template_nodes.push(node);
                continue;
            }
            Node::Element(el) => match el.tag.as_str() {
                "data" => {
                    data.push(parse_data(el)?);
                    pending_jsdoc = None;
                    continue;
                }
                "client" => {
                    clients.push(parse_client(el, &clients)?);
                    pending_jsdoc = None;
                    continue;
                }
                "state" => {
                    return Err(state_directive_retired(el));
                }
                "prop" => {
                    let mut decl = parse_prop(el)?;
                    decl.description = pending_jsdoc.take();
                    if decl.form_value && props.iter().any(|p| p.form_value) {
                        return Err(CompileError::new(format!(
                            "<prop name=\"{}\"> form-value: a component may declare \
                             at most one form-value prop (another already exists)",
                            decl.name,
                        ))
                        .with_span(el.span));
                    }
                    props.push(decl);
                    continue;
                }
                "style" => {
                    for child in &el.children {
                        if let Node::Text(t) = child {
                            style_parts.push(t.data.clone());
                        }
                    }
                    pending_jsdoc = None;
                    continue;
                }
                _ => {}
            },
            _ => {}
        }
        pending_jsdoc = None;
        template_nodes.push(node);
    }

    let styles = style_parts.join("\n");
    let template = template::compile_children(&template_nodes)?;

    Ok(Program {
        data,
        props,
        clients,
        styles,
        template,
        source_len: source.len(),
        span: Span::new(0, source.len()),
    })
}

/// Build the parse error returned when an author writes a top-level
/// `<state>` element. The directive was retired in favour of
/// declaring reactive state in the paired controller's `scope:`
/// factory; the message names the migration target so authors can
/// fix the file without consulting the changelog.
fn state_directive_retired(el: &html::Element) -> CompileError {
    let name = el.get_attribute("name").unwrap_or_default();
    let label = if name.is_empty() {
        "<state>".to_string()
    } else {
        format!("<state name=\"{name}\">")
    };
    CompileError::new(format!(
        "{label} directive is no longer supported. Declare reactive state \
         in the paired controller's `scope:` factory \
         (e.g. `scope: () => ({{ {} }})`).",
        if name.is_empty() {
            "x: signal(0)".to_string()
        } else {
            format!("{name}: signal(...)")
        }
    ))
    .with_span(el.span)
    .with_category(label)
}

fn parse_data(el: &html::Element) -> Result<DataDecl, CompileError> {
    let name = required_attr(el, "name", "<data>")?;
    let source_attr = required_attr(el, "source", "<data>")?;
    let live = el.has_attribute("live");
    // The `source=` attribute is a JS expression evaluated against
    // (ship, route). We validate it as a normal expression.
    let span = attr_span(el, "source").unwrap_or(el.span);
    let expr_text = strip_brace_attr(&source_attr);
    expr::validate_at(&expr_text, false, &format!("<data name=\"{name}\"> source"), span)?;
    Ok(DataDecl {
        name,
        source: Expr::new(expr_text, span, false),
        live,
        span: el.span,
    })
}

/// Setup-context names a `<client>` may not shadow. `ship` is deliberately
/// NOT here — declaring `<client name="ship" />` is the migration path off
/// the ambient default, routing the same key through `ctx.clients`.
const RESERVED_CLIENT_NAMES: &[&str] = &["route", "navigate", "ctx", "scope", "props", "derived"];

fn parse_client(el: &html::Element, existing: &[ClientDecl]) -> Result<ClientDecl, CompileError> {
    let name = required_attr(el, "name", "<client>")?;
    if !is_js_identifier(&name) {
        return Err(CompileError::new(format!(
            "<client name=\"{name}\">: name must be a JS identifier — it becomes \
             the scope name the template resolves (e.g. `{name}.domain.op()`)"
        ))
        .with_span(el.span)
        .with_category("<client>"));
    }
    if RESERVED_CLIENT_NAMES.contains(&name.as_str()) {
        return Err(CompileError::new(format!(
            "<client name=\"{name}\">: `{name}` is a reserved setup-context name \
             and cannot be used as a client key"
        ))
        .with_span(el.span)
        .with_category("<client>"));
    }
    if existing.iter().any(|c| c.name == name) {
        return Err(CompileError::new(format!(
            "<client name=\"{name}\"> is declared more than once"
        ))
        .with_span(el.span)
        .with_category("<client>"));
    }
    Ok(ClientDecl { name, span: el.span })
}

fn is_js_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' || c == '$' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

fn parse_prop(el: &html::Element) -> Result<PropDecl, CompileError> {
    let name = required_attr(el, "name", "<prop>")?;
    let values_attr = el.get_attribute("values");
    let type_attr = el.get_attribute("type");
    let default = el.get_attribute("default");
    let reflect = el.has_attribute("reflect");
    let bindable = el.has_attribute("bindable");
    let form_value = el.has_attribute("form-value");
    let form_disabled = el.has_attribute("form-disabled");

    let kind = if let Some(values_raw) = values_attr {
        let values = values_raw
            .split('|')
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
            .collect::<Vec<_>>();
        PropKind::Union(values)
    } else {
        match type_attr.as_deref() {
            Some("bool") => PropKind::Bool,
            Some("number") => PropKind::Number,
            None | Some("string") => PropKind::String,
            Some(t @ ("object" | "array")) if form_value => {
                // form-value only supports primitive-coercible kinds.
                return Err(CompileError::new(format!(
                    "<prop name=\"{name}\"> form-value cannot be used on \
                     type=\"{t}\" (only string, number, bool, or union props \
                     can participate in form value)"
                ))
                .with_span(el.span));
            }
            Some(other) => {
                return Err(CompileError::new(format!(
                    "<prop name=\"{name}\"> unknown type: {other}"
                ))
                .with_span(el.span));
            }
        }
    };

    if form_disabled && kind != PropKind::Bool {
        return Err(CompileError::new(format!(
            "<prop name=\"{name}\"> form-disabled requires type=\"bool\" \
             (the browser passes a boolean to formDisabledCallback)"
        ))
        .with_span(el.span));
    }

    Ok(PropDecl {
        name,
        kind,
        default,
        reflect,
        bindable,
        form_value,
        form_disabled,
        description: None,
        span: el.span,
    })
}

/// Recognise an HTML comment whose body is a JSDoc block (`/** ... */`)
/// and return the description text. Per-line `*` margins are stripped
/// the way TypeScript / VS Code render JSDoc, so multi-line comments
/// produce clean, contiguous text.
fn extract_jsdoc(comment_data: &str) -> Option<String> {
    let trimmed = comment_data.trim();
    let body = trimmed.strip_prefix("/**")?.strip_suffix("*/")?;
    let mut out_lines: Vec<String> = Vec::new();
    for line in body.lines() {
        let l = line.trim();
        let cleaned = l
            .strip_prefix("* ")
            .or_else(|| l.strip_prefix('*'))
            .unwrap_or(l)
            .trim_end()
            .to_string();
        out_lines.push(cleaned);
    }
    let joined = out_lines.join("\n").trim().to_string();
    if joined.is_empty() { None } else { Some(joined) }
}

fn required_attr(
    el: &html::Element,
    name: &str,
    label: &str,
) -> Result<String, CompileError> {
    el.get_attribute(name).ok_or_else(|| {
        CompileError::new(format!("{label} missing required attribute: {name}"))
            .with_span(el.span)
            .with_category(label.to_string())
    })
}

fn attr_span(el: &html::Element, name: &str) -> Option<Span> {
    el.attributes.iter().find(|a| a.name == name).map(|a| a.value_span)
}

/// Authors may write `value={0}` (brace form) or `value="0"` (literal
/// expression string). Both compile to the same code.
pub fn strip_brace_attr(s: &str) -> String {
    let trimmed = s.trim();
    if trimmed.starts_with('{') && trimmed.ends_with('}') && trimmed.len() >= 2 {
        trimmed[1..trimmed.len() - 1].trim().to_string()
    } else {
        trimmed.to_string()
    }
}
