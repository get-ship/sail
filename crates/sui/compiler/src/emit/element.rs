//! ELEMENT MODE — compile a `.sui` source as an L1 language element.
//!
//! One authoring technology per layer: an element source is a normal `.sui`
//! file (component block for the contract, template for the structure,
//! variants/sx for the skin), but the outputs differ from a component:
//!
//!   - the template becomes a STRUCTURE snippet for use-site lowering (like
//!     the layout keywords) — no registered module, no runtime identity;
//!   - the skin routes to the FRAMEWORK SHEET, keyed on the element's root
//!     class (`.sui-<name>`) instead of `:host`;
//!   - the props block feeds the element MANIFEST (and, downstream, the
//!     bare-shorthand vocabulary).
//!
//! The mode enforces the seam test mechanically: element templates are
//! static structure + slots. Control flow, dynamic attributes, dynamic
//! text, `<data>`, or a controller mean the source is a COMPONENT — the
//! error says so.

use crate::ir::*;
use crate::CompileError;
use serde_json::{json, Value};

/// The element-mode artifacts.
#[derive(Debug, Clone)]
pub struct ElementOutput {
    /// Root class the lowering stamps and the skin keys on: `sui-<name>`.
    /// (Unused for native-kind elements — their skin keys on the tag.)
    pub root_class: String,
    /// NATIVE-KIND: the template root IS the element tag (label → <label>).
    /// The tag already exists in HTML; there is nothing to splice — the
    /// element is skin + manifest only, and rewriting real uses would
    /// clobber every ordinary occurrence of the tag.
    pub native: bool,
    /// Static structure snippet (single root element, slots preserved).
    pub structure_html: String,
    /// Skin CSS — keyed on `.sui-<name>` for spliced elements, on the bare
    /// tag selector for native-kind (from the source's `:host` idioms).
    pub skin_css: String,
    /// The element manifest (contract): tag, attributes, anatomy.
    pub manifest: Value,
}

pub fn emit_element(program: &Program, name: &str) -> Result<ElementOutput, CompileError> {
    // L1 tags are BARE names — the physical naming seam: hyphenated tags are
    // custom elements, bare tags compile away.
    if name.is_empty()
        || !name.chars().all(|c| c.is_ascii_lowercase())
        || name.contains('-')
    {
        return Err(CompileError::new(format!(
            "element name `{name}` must be a bare lowercase word (no hyphen) — \
             a hyphenated tag is a custom element, i.e. a component"
        )));
    }
    if !program.data.is_empty() || !program.clients.is_empty() {
        return Err(CompileError::new(
            "element mode: <data>/<client> need runtime state — this source is a \
             component; compile it with --component-name instead",
        ));
    }
    let root_class = format!("sui-{name}");

    // Exactly one root element; whitespace text and comments tolerated.
    let mut roots = program.template.iter().filter_map(|i| match i {
        Instr::Element(el) => Some(el),
        _ => None,
    });
    let Some(root) = roots.next() else {
        return Err(CompileError::new(
            "element mode: the template needs exactly one root element",
        ));
    };
    if roots.next().is_some()
        || program.template.iter().any(|i| match i {
            Instr::StaticText { text, .. } => !text.trim().is_empty(),
            Instr::Element(_) | Instr::Comment { .. } => false,
            _ => true,
        })
    {
        return Err(CompileError::new(
            "element mode: the template must be ONE static root element \
             (control flow at the top level means this is a component)",
        ));
    }

    let native = root.tag == name;
    let mut structure = String::new();
    serialize_static(root, &root_class, true, &mut structure)?;

    // Spliced elements key their skin on the stamped root class; native-kind
    // elements key on the tag itself — the tag IS the element.
    let skin_root = if native { name.to_string() } else { format!(".{root_class}") };
    let skin_css = element_skin_selectors(&program.styles, &skin_root);

    let attributes: Vec<Value> = program
        .props
        .iter()
        .map(|p| {
            let mut e = json!({ "name": p.name });
            if let PropKind::Union(values) = &p.kind {
                e.as_object_mut().unwrap().insert("values".into(), json!(values));
            }
            if let Some(d) = p.default.as_ref() {
                e.as_object_mut().unwrap().insert("default".into(), json!(d));
            }
            e
        })
        .collect();
    let (classes, slots) = super::cem::collect_anatomy(&program.template);
    let manifest = json!({
        "kind": "element",
        "tag": name,
        "native": native,
        "rootClass": root_class,
        "attributes": attributes,
        "anatomy": { "dom": "light", "classes": classes, "slots": slots },
    });

    Ok(ElementOutput { root_class, native, structure_html: structure, skin_css, manifest })
}

/// Serialize a static element tree. `is_root` stamps the root class so the
/// skin and the use-site lowering agree on the key. Any dynamic construct is
/// the seam-test error.
fn serialize_static(
    el: &ElementInstr,
    root_class: &str,
    is_root: bool,
    out: &mut String,
) -> Result<(), CompileError> {
    out.push('<');
    out.push_str(&el.tag);
    let mut class_written = false;
    for a in &el.attrs {
        match a {
            AttrInstr::Static { name, value, .. } => {
                let mut value = value.clone();
                if is_root && name == "class" {
                    class_written = true;
                    value = format!("{root_class} {value}");
                }
                out.push(' ');
                out.push_str(name);
                if !value.is_empty() {
                    out.push_str("=\"");
                    out.push_str(&value);
                    out.push('"');
                }
            }
            _ => {
                return Err(CompileError::new(format!(
                    "element mode: dynamic attribute on <{}> — elements are static \
                     structure; per-instance behavior belongs to the use site or a \
                     component",
                    el.tag
                )));
            }
        }
    }
    if is_root && !class_written {
        out.push_str(&format!(" class=\"{root_class}\""));
    }
    out.push('>');
    for child in &el.children {
        match child {
            Instr::Element(c) => serialize_static(c, root_class, false, out)?,
            Instr::StaticText { text, .. } => out.push_str(text),
            Instr::Comment { .. } => {}
            _ => {
                return Err(CompileError::new(format!(
                    "element mode: <{}> contains control flow or dynamic content — \
                     this is a component; compile it with --component-name",
                    el.tag
                )));
            }
        }
    }
    out.push_str(&format!("</{}>", el.tag));
    Ok(())
}

/// Rewrite the source's `:host`-keyed skin for the element's root class:
/// `:host` → `.sui-<name>`, `:host(<sel>)` → `.sui-<name><sel>`,
/// `::slotted(<sel>)` → `slot > <sel>`.
fn element_skin_selectors(css: &str, root_selector: &str) -> String {
    let root = root_selector.to_string();
    let mut out = String::with_capacity(css.len());
    let bytes = css.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if css[i..].starts_with(":host") {
            let after = i + ":host".len();
            if bytes.get(after) == Some(&b'(') {
                if let Some(close) = balanced(bytes, after) {
                    out.push_str(&root);
                    out.push_str(&css[after + 1..close]);
                    i = close + 1;
                    continue;
                }
            }
            out.push_str(&root);
            i = after;
            continue;
        }
        if css[i..].starts_with("::slotted(") {
            let open = i + "::slotted".len();
            if let Some(close) = balanced(bytes, open) {
                out.push_str("slot > ");
                out.push_str(&css[open + 1..close]);
                i = close + 1;
                continue;
            }
        }
        let mut n = i + 1;
        while n < bytes.len() && (bytes[n] & 0xC0) == 0x80 {
            n += 1;
        }
        out.push_str(&css[i..n]);
        i = n;
    }
    out
}

fn balanced(bytes: &[u8], open: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut i = open;
    while i < bytes.len() {
        match bytes[i] {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}
