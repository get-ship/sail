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
    let mut template_nodes: Vec<Node> = Vec::with_capacity(nodes.len());

    for node in nodes {
        match &node {
            Node::Comment(c) => {
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
                "component" => {
                    // The single declaration surface. Props are props-as-ts in
                    // this block → `PropDecl` directly (no `<prop>` tag, no
                    // desugar). `form:` names the form-associated props. The
                    // `variants` field is lowered in preprocess; ignore it here.
                    let body: String = el
                        .children
                        .iter()
                        .filter_map(|c| match c {
                            Node::Text(t) => Some(t.data.clone()),
                            _ => None,
                        })
                        .collect();
                    let fields = crate::parse::preprocess::parse_object_fields(&body);
                    // The block holds a fixed, small field set. Anything else —
                    // a retired construct (`light`, `variants`), a typo — is
                    // rejected generically: the compiler names what IS valid and
                    // carries no memory of the old forms (no fossilization).
                    const COMPONENT_FIELDS: &[&str] = &["props", "form", "compound", "shadow"];
                    if let Some((k, _)) =
                        fields.iter().find(|(k, _)| !COMPONENT_FIELDS.contains(&k.as_str()))
                    {
                        return Err(CompileError::new(format!(
                            "unknown `<component>` field `{k}` \u{2014} the block holds \
                             `props`, `form`, `compound`, and `shadow` only."
                        ))
                        .with_span(el.span));
                    }
                    let (form_value, form_disabled) = fields
                        .iter()
                        .find(|(k, _)| k == "form")
                        .map(|(_, v)| crate::parse::preprocess::parse_form_field(v))
                        .unwrap_or((None, None));
                    let props_obj = fields
                        .iter()
                        .find(|(k, _)| k == "props")
                        .map(|(_, v)| v.clone())
                        .unwrap_or_default();
                    let compound = fields.iter().find(|(k, _)| k == "compound").map(|(_, v)| v.clone());
                    // Strict, one-way variant syntax — no parallel paths, teaching
                    // errors so a fresh agent is corrected rather than left to guess.
                    for (k, v) in crate::parse::preprocess::parse_object_fields(&props_obj) {
                        let v = v.trim();
                        if k == "variant" && v.starts_with("variant(") {
                            return Err(CompileError::new(
                                "`variant` is the reserved variant key \u{2014} drop the `variant()` wrapper (write `variant: { \u{2026} }`)",
                            )
                            .with_span(el.span));
                        }
                    }
                    let decls = crate::parse::preprocess::component_props_to_decls(
                        &props_obj,
                        form_value.as_deref(),
                        form_disabled.as_deref(),
                    );

                    // `form: { value, disabled }` names props that get wired to the
                    // PLATFORM's form APIs, so their types are not ours to choose:
                    // `ElementInternals.setFormValue` takes a serializable value, and
                    // `formDisabledCallback` takes a bool. The old `<prop type="…">`
                    // tag checked this; props-as-ts dropped the check with the tag,
                    // and nothing replaced it — so `form: { disabled: 'label' }` on a
                    // string silently produced a component that lies to the browser.
                    // (`form` naming a prop that doesn't exist is caught here too;
                    // "at most one form-value" is gone because the object shape makes
                    // it unrepresentable — one key, one value.)
                    let find = |n: &str| decls.iter().find(|p| p.name == n);
                    if let Some(name) = form_value.as_deref() {
                        match find(name).map(|p| &p.kind) {
                            None => {
                                return Err(CompileError::new(format!(
                                    "`form: {{ value: '{name}' }}` names no prop — form-value must \
                                     name a prop declared in the same block."
                                ))
                                .with_span(el.span));
                            }
                            // A bool IS a legitimate form value — that is a checkbox
                            // (`checked`). What cannot be submitted is an arbitrary TS
                            // type: `Identity` is an identity property, never reflected
                            // and never serialized.
                            Some(PropKind::Identity(_)) => {
                                return Err(CompileError::new(format!(
                                    "prop `{name}` cannot be the form-value — the platform submits it \
                                     via `setFormValue`, which takes a serializable value, and `{name}` \
                                     is a non-primitive type (an identity property, never reflected). \
                                     Declare it as a string, number, bool, or union."
                                ))
                                .with_span(el.span));
                            }
                            _ => {}
                        }
                    }
                    if let Some(name) = form_disabled.as_deref() {
                        match find(name).map(|p| &p.kind) {
                            None => {
                                return Err(CompileError::new(format!(
                                    "`form: {{ disabled: '{name}' }}` names no prop — form-disabled \
                                     must name a prop declared in the same block."
                                ))
                                .with_span(el.span));
                            }
                            Some(PropKind::Bool) => {}
                            Some(_) => {
                                return Err(CompileError::new(format!(
                                    "prop `{name}` cannot be the form-disabled flag — the platform \
                                     calls `formDisabledCallback(disabled)` with a boolean. Declare \
                                     it as `{name}: false`."
                                ))
                                .with_span(el.span));
                            }
                        }
                    }

                    props.extend(decls);
                    // Variant styles (from the `variant` props + a sibling
                    // `compound`) lower straight to CSS — no `<variants>` block.
                    let variant_css = crate::parse::preprocess::component_variants_to_css(
                        &props_obj,
                        compound.as_deref(),
                    );
                    if !variant_css.is_empty() {
                        style_parts.push(variant_css);
                    }
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
        template,
        source_len: source.len(),
        span: Span::new(0, source.len()),
    })
}

/// Reject author-written `:host` in a light component's `<style>` block. Model
/// A: an element styles its own template nodes by class — a spliced element's
/// root is `.sui-<name>`, a native-kind element's is the bare tag; variants are
/// attribute-keyed. Shadow components (the `shadow: true` opt-in) keep `:host` —
/// shadow DOM has a real host. Scans the RAW source, before preprocess prepends
/// the compiler's own scoped CSS.
fn reject_retired_host_selector(source: &str) -> Result<(), CompileError> {
    if preprocess::declares_shadow_dom(source) {
        return Ok(());
    }
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
