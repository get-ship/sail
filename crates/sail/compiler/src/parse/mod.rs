//! Parse pipeline. Walks `.sail` source and produces an IR.
//!
//! Stages, in order:
//!  1. Preprocess — same two transforms the interpreter does
//!     (quote unquoted brace attrs, expand non-void self-closers).
//!  2. HTML parse — produces a span-bearing DOM-ish tree.
//!  3. Directive lift — pluck `<scope>` (props/variants/types) and
//!     top-level `<style>` out of the top-level child list into the
//!     IR. A top-level `<state>` directive is rejected with a parse
//!     error pointing authors at the controller's `setup:` factory.
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

/// Parse a `.sail` source string into an IR program.
pub fn parse(source: &str) -> Result<Program, CompileError> {
    parse_inner(source, true)
}

/// Author-facing template IR for tooling: no stdlib splice, no sx/layout
/// rewrite. Tags stay as authored (`column`, `ticker`, `my-widget`); spans
/// land in a lightly normalized string — callers remap to author bytes via
/// tag ordinal when needed (SailInspect).
pub fn parse_author(source: &str) -> Result<Program, CompileError> {
    reject_retired_host_selector(source)?;
    reject_part_selector(source)?;
    let prepared = preprocess::preprocess_author(source)?;
    parse_prepared(source, &prepared)
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
    let prepared = preprocess::preprocess_impl(source, expand_elements)?;
    parse_prepared(source, &prepared)
}

fn parse_prepared(source: &str, prepared: &str) -> Result<Program, CompileError> {
    let nodes = html::parse_fragment(prepared)?;

    let mut props: Vec<PropDecl> = Vec::new();
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
                "state" => {
                    return Err(state_directive_retired(el));
                }
                "scope" => {
                    // The component's static surface: `type` aliases +
                    // `props:` (union props only) + optional `compound:`,
                    // in TypeScript field syntax with literal values only
                    // (see parse::scope_block). Paint lives in host `sx=`
                    // via the Style product (`match`, `when`, `compound`).
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
                // Inline `<controller>` is stripped by `extract_controller_block`
                // before parse in `compile` / `compile_element`. If one still
                // lands here, drop it — it is not template content.
                "controller" => continue,
                _ => {}
            },
            _ => {}
        }
        template_nodes.push(node);
    }

    let styles = style_parts.join("\n");
    let template = template::compile_children(&template_nodes)?;

    Ok(Program {
        props,
        styles,
        scope_types,
        template,
        source_len: source.len(),
        span: Span::new(0, source.len()),
    })
}

/// Reject author-written `:host` in a component's `<style>` block. Model A:
/// an element styles its own template nodes by class — a spliced element's
/// root is `.sail-<name>`, a native-kind element's is the bare tag; variants
/// are attribute-keyed. Components are light DOM; there is no host to select.
/// Scans the RAW source, before preprocess prepends the compiler's own
/// scoped CSS.
fn reject_retired_host_selector(source: &str) -> Result<(), CompileError> {
    let mut i = 0;
    while let Some(rel) = source[i..].find("<style") {
        let open = i + rel;
        let Some(gt) = source[open..].find('>') else {
            break;
        };
        let cs = open + gt + 1;
        let Some(close) = source[cs..].find("</style>") else {
            break;
        };
        let content = &source[cs..cs + close];
        if let Some(pos) = content.find(":host") {
            let at = cs + pos;
            return Err(CompileError::new(
                "`:host` is a shadow-DOM idiom — light DOM has no host to select. \
                 Style the element with sx.",
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
        let Some(gt) = source[open..].find('>') else {
            break;
        };
        let cs = open + gt + 1;
        let Some(close) = source[cs..].find("</style>") else {
            break;
        };
        let content = &source[cs..cs + close];
        if let Some(pos) = content.find("::part") {
            let at = cs + pos;
            return Err(CompileError::new(
                "the `::part()` selector is retired — light DOM has no shadow \
                 parts. Style the target with sx.",
            )
            .with_span(Span::new(at, at + "::part".len())));
        }
        i = cs + close + "</style>".len();
    }
    Ok(())
}

/// Build the parse error returned when an author writes a top-level
/// `<state>` element. The directive was retired in favour of
/// declaring reactive state in the controller's setup function; the
/// message names the migration target so authors can fix the file
/// without consulting the changelog.
fn state_directive_retired(el: &html::Element) -> CompileError {
    let name = el.get_attribute("name").unwrap_or_default();
    let label = if name.is_empty() {
        "<state>".to_string()
    } else {
        format!("<state name=\"{name}\">")
    };
    CompileError::new(format!(
        "{label} directive is no longer supported. Declare reactive state \
         in the controller's setup function \
         (e.g. `controller((ctx) => {{ ...; return {{ {} }} }})`).",
        if name.is_empty() {
            "x: signal(0)".to_string()
        } else {
            format!("{name}: signal(...)")
        }
    ))
    .with_span(el.span)
    .with_category(label)
}
