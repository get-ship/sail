//! Parse pipeline. Walks `.sui` source and produces an IR.
//!
//! Stages, in order:
//!  1. Preprocess — same two transforms the interpreter does
//!     (quote unquoted brace attrs, expand non-void self-closers).
//!  2. HTML parse — produces a span-bearing DOM-ish tree.
//!  3. Directive lift — pluck `<data>`, `<prop>`, and top-level
//!     `<style>` out of the top-level child list into the IR. A
//!     top-level `<state>` directive is rejected with a parse error
//!     pointing authors at the controller's `setup:` factory.
//!  4. Template grammar scan — text-node `{...}` markers become
//!     template instructions; element children recurse.
//!  5. Expression validate — every brace expression goes through the
//!     same forbidden-syntax checks the interpreter applies.

use crate::diagnostics::{CompileError, Span};
use crate::ir::*;

pub mod expr;
pub mod html;
pub mod preprocess;
pub(crate) mod scope_block;
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
    // Reject retired shadow-DOM CSS idioms in author `<style>` BEFORE preprocess
    // injects the compiler's own scoped CSS (which uses `:host`/`:scope` as
    // internal dual-target plumbing). Only author `<style>` is inspected.
    reject_retired_host_selector(source)?;
    reject_part_selector(source)?;
    let prepared = preprocess::preprocess_impl(source, expand_elements);
    let nodes = html::parse_fragment(&prepared)?;

    let mut data: Vec<DataDecl> = Vec::new();
    let mut props: Vec<PropDecl> = Vec::new();
    let mut clients: Vec<ClientDecl> = Vec::new();
    let mut style_parts: Vec<String> = Vec::new();
    let mut scope_types = String::new();
    let mut template_nodes: Vec<Node> = Vec::with_capacity(nodes.len());

    for node in nodes {
        match &node {
            Node::Comment(_c) => {
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
                    continue;
                }
                "client" => {
                    clients.push(parse_client(el, &clients)?);
                    continue;
                }
                "state" => {
                    return Err(state_directive_retired(el));
                }
                "scope" => {
                    // The component's static surface: `type` aliases +
                    // `props:` + `variants:`, in TypeScript field syntax with
                    // literal values only (see parse::scope_block). Types
                    // thread to the controller module; variants lower to
                    // attribute-keyed CSS + reflected union props.
                    let body: String = el
                        .children
                        .iter()
                        .filter_map(|c| match c {
                            Node::Text(t) => Some(t.data.clone()),
                            _ => None,
                        })
                        .collect();
                    let block = scope_block::parse_scope_block(&body, el.span)?;
                    props.extend(block.props);
                    if !block.variants_css.is_empty() {
                        style_parts.push(block.variants_css);
                    }
                    scope_types.push_str(&block.types);
                    continue;
                }
                "style" => {
                    for child in &el.children {
                        if let Node::Text(t) = child {
                            style_parts.push(t.data.clone());
                        }
                    }
                    continue;
                }
                _ => {}
            },
            _ => {}
        }
        template_nodes.push(node);
    }

    let styles = style_parts.join("\n");
    let template = template::compile_children(&template_nodes)?;

    Ok(Program {
        data,
        props,
        clients,
        styles,
        scope_types,
        template,
        source_len: source.len(),
        span: Span::new(0, source.len()),
    })
}

/// Reject author-written `:host` in a component's `<style>` block. Model A:
/// an element styles its own template nodes by class — a spliced element's
/// root is `.sui-<name>`, a native-kind element's is the bare tag; variants
/// are attribute-keyed. Components are light DOM; there is no host to select.
/// Scans the RAW source, before preprocess prepends the compiler's own
/// scoped CSS.
fn reject_retired_host_selector(source: &str) -> Result<(), CompileError> {
    let mut i = 0;
    while let Some(rel) = source[i..].find("<style") {
        let open = i + rel;
        let Some(gt) = source[open..].find('>') else { break };
        let cs = open + gt + 1;
        let Some(close) = source[cs..].find("</style>") else { break };
        let content = &source[cs..cs + close];
        if let Some(pos) = content.find(":host") {
            let at = cs + pos;
            return Err(CompileError::new(
                "`:host` is a shadow-DOM idiom — light DOM has no host to select. \
                 Style the element's root by its class (`.sui-<name>`, or the bare \
                 tag for a native-kind element); variants are attribute-keyed.",
            )
            .with_span(Span::new(at, at + ":host".len())));
        }
        i = cs + close + "</style>".len();
    }
    Ok(())
}

/// Reject a `::part()` selector in an author `<style>` block. Light DOM has no
/// shadow parts — the target element's class is the styling hook (anatomy =
/// classes + attributes). Scans the RAW source, before preprocess prepends the
/// compiler's own scoped CSS (which never uses `::part`).
fn reject_part_selector(source: &str) -> Result<(), CompileError> {
    let mut i = 0;
    while let Some(rel) = source[i..].find("<style") {
        let open = i + rel;
        let Some(gt) = source[open..].find('>') else { break };
        let cs = open + gt + 1;
        let Some(close) = source[cs..].find("</style>") else { break };
        let content = &source[cs..cs + close];
        if let Some(pos) = content.find("::part") {
            let at = cs + pos;
            return Err(CompileError::new(
                "the `::part()` selector is retired — light DOM has no shadow \
                 parts. Style the target's class directly (`.name`); anatomy is \
                 classes + attributes.",
            )
            .with_span(Span::new(at, at + "::part".len())));
        }
        i = cs + close + "</style>".len();
    }
    Ok(())
}

/// Build the parse error returned when an author writes a top-level
/// `<state>` element. The directive was retired in favour of
/// declaring reactive state in the paired controller's `setup:`
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
         in the paired controller's `setup:` factory \
         (e.g. `setup: () => ({{ {} }})`).",
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
const RESERVED_CLIENT_NAMES: &[&str] = &["route", "navigate", "ctx", "scope", "setup", "props", "derived"];

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
