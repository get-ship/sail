//! ELEMENT MODE — compile a `.sail` source as an L1 language element.
//!
//! One authoring technology per layer: an element source is a normal `.sail`
//! file (scope for the contract, template for the structure, variants/sx for
//! the skin), but the outputs differ from a component:
//!
//!   - the template becomes a STRUCTURE snippet for use-site lowering (like
//!     the layout keywords) — no registered module, no runtime identity;
//!   - the skin routes to the FRAMEWORK SHEET, keyed on the element's root
//!     class (`.sail-<name>`) instead of `:host`;
//!   - the props block feeds the element MANIFEST (and, downstream, the
//!     bare-shorthand vocabulary).
//!
//! The mode enforces the seam test mechanically: element *templates* are
//! static structure + slots. Control flow, dynamic attributes, or dynamic
//! text mean the source is a COMPONENT — the error says so. JS enhancement
//! rides stacked `use:<behavior>` stamps on the static structure (lowered
//! to `runAction` at the use site); an inline `<controller>` block is an
//! L2-component facility and is REJECTED here (see `compile_element`).

use crate::diagnostics::Span;
use crate::ir::*;
use crate::CompileError;
use serde_json::{json, Value};

/// Spans on `Program`/`Instr` are byte offsets into the PREPROCESSED source
/// (`parse::preprocess::preprocess_impl`): brace-attr quoting, self-close
/// expansion, `sx=`/layout lowering, and any injected `<style>` block can
/// all change the text's length ahead of a given position. Every other
/// diagnostic renderer (`CompileError::render`, `Diagnostic::from_compile_error`,
/// the wasm surface) is handed the AUTHOR'S ORIGINAL source, not that
/// preprocessed copy — so reusing an IR span as-is can point past its end.
///
/// The root-element structural checks below therefore locate their span by
/// scanning the ORIGINAL source text directly, rather than trusting the
/// (possibly-shifted) IR span. This sidesteps the mismatch entirely instead
/// of trying to reverse-map an offset.
///
/// Byte offset just past the top-level `<scope>…</scope>` / `<style>…</style>`
/// blocks (in whatever order/repetition they appear, leading the source) —
/// where an element's template body is expected to start. Falls back to 0
/// when neither is present, which is always in-bounds.
fn skip_scope_and_style(source: &str) -> usize {
    let mut i = 0usize;
    loop {
        let rest = source[i..].trim_start();
        let ws = source[i..].len() - rest.len();
        let close = if rest.starts_with("<scope") {
            "</scope>"
        } else if rest.starts_with("<style") {
            "</style>"
        } else {
            return i + ws;
        };
        match source[i + ws..].find(close) {
            Some(off) => i = i + ws + off + close.len(),
            None => return i + ws,
        }
    }
}

/// The tag-name span (just `span` in `<span …>`) and the byte offset just
/// past the element's matching close tag, for every element found at the
/// top level of `source` starting at `from` — comments and whitespace text
/// are skipped, nested content is walked past as one unit via a depth
/// count. A pragmatic scan (not a full HTML parser): good enough for the
/// flat, static shape element-mode templates are required to have.
fn top_level_element_spans(source: &str, from: usize) -> Vec<(Span, usize)> {
    let mut out = Vec::new();
    let len = source.len();
    let mut i = from.min(len);
    while i < len {
        let rest = &source[i..];
        let ws = rest.len() - rest.trim_start().len();
        i += ws;
        if i >= len {
            break;
        }
        if source.as_bytes()[i] != b'<' {
            // Stray top-level text — not an element; skip to the next tag.
            match source[i..].find('<') {
                Some(off) => i += off,
                None => break,
            }
            continue;
        }
        if source[i..].starts_with("<!--") {
            match source[i..].find("-->") {
                Some(off) => i += off + 3,
                None => break,
            }
            continue;
        }
        let Some(name_end_off) = source[i + 1..]
            .find(|c: char| c.is_whitespace() || c == '>' || c == '/')
        else {
            break;
        };
        let name_start = i + 1;
        let name_end = name_start + name_end_off;
        let tag = &source[name_start..name_end];
        let Some(gt_off) = source[i..].find('>') else {
            break;
        };
        let open_end = i + gt_off + 1;
        let self_closed = source[..open_end].ends_with("/>") || is_void_element(tag);
        let name_span = Span::new(name_start, name_end);
        if self_closed {
            out.push((name_span, open_end));
            i = open_end;
            continue;
        }
        let open_prefix = format!("<{tag}");
        let close_tag = format!("</{tag}>");
        let mut depth = 1usize;
        let mut j = open_end;
        let end = loop {
            let next_open = source[j..].find(&open_prefix).map(|o| j + o).filter(|&o| {
                source.as_bytes().get(o + open_prefix.len()).is_some_and(|b| {
                    matches!(b, b' ' | b'\t' | b'\n' | b'\r' | b'>' | b'/')
                })
            });
            let next_close = source[j..].find(&close_tag).map(|o| j + o);
            match (next_open, next_close) {
                (Some(no), Some(nc)) if no < nc => {
                    depth += 1;
                    j = no + open_prefix.len();
                }
                (_, Some(nc)) => {
                    depth -= 1;
                    j = nc + close_tag.len();
                    if depth == 0 {
                        break j;
                    }
                }
                _ => break len,
            }
        };
        out.push((name_span, end));
        i = end;
    }
    out
}

/// The element-mode artifacts.
#[derive(Debug, Clone)]
pub struct ElementOutput {
    /// Root class the lowering stamps and the skin keys on: `sail-<name>`.
    /// (Unused for native-kind elements — their skin keys on the tag.)
    pub root_class: String,
    /// NATIVE-KIND: the template root IS the element tag (label → <label>).
    /// The tag already exists in HTML; there is nothing to splice — the
    /// element is skin + manifest only, and rewriting real uses would
    /// clobber every ordinary occurrence of the tag.
    pub native: bool,
    /// Static structure snippet (single root element, slots preserved).
    pub structure_html: String,
    /// The class string `serialize_static` stamped on the structure root:
    /// `sail-<name>` + the atomic sx classes the host `sx` lowered to.
    /// Spliced elements carry it on the snippet root; NATIVE-kind elements
    /// have no splice, so the use-site lowering stamps this same string onto
    /// the real tag (preprocess `lower_into`) — otherwise the host paint
    /// keyed on those classes never reaches a single element.
    pub host_classes: String,
    /// Skin CSS — keyed on `.sail-<name>` for spliced elements, on the bare
    /// tag selector for native-kind (from the source's `:host` idioms).
    pub skin_css: String,
    /// The element manifest (contract): tag, attributes, anatomy.
    pub manifest: Value,
}

pub fn emit_element(
    program: &Program,
    source: &str,
    name: &str,
) -> Result<ElementOutput, CompileError> {
    // L1 element names are lowercase words, optionally hyphen-joined
    // (`badge`, `empty-state`, `date-picker`). The naming seam is the STDLIB
    // REGISTRY, not the hyphen: a tag in the registry compiles away (L1); a
    // hyphenated tag NOT in the registry is an ordinary user custom element —
    // `lower_stdlib_elements` only ever splices registered tags, so the two
    // never collide.
    let valid = !name.is_empty()
        && name.chars().next().is_some_and(|c| c.is_ascii_lowercase())
        && name.chars().all(|c| c.is_ascii_lowercase() || c == '-')
        && !name.ends_with('-')
        && !name.contains("--");
    if !valid {
        return Err(CompileError::new(format!(
            "element name `{name}` must be lowercase words joined by single \
             hyphens (e.g. `badge`, `empty-state`) — the stdlib registry is what \
             makes it an L1 element"
        )));
    }
    let root_class = format!("sail-{name}");

    // Exactly one root element; whitespace text and comments tolerated.
    let mut roots = program.template.iter().filter_map(|i| match i {
        Instr::Element(el) => Some(el),
        _ => None,
    });
    // The template body starts right after any leading `<scope>`/`<style>`
    // blocks — the sensible location to point at when there's no root
    // element there to point at instead.
    let body_start = skip_scope_and_style(source);
    let Some(root) = roots.next() else {
        return Err(CompileError::new(
            "element mode: the template needs exactly one root element",
        )
        .with_span(Span::point(body_start)));
    };
    if roots.next().is_some()
        || program.template.iter().any(|i| match i {
            Instr::StaticText { text, .. } => !text.trim().is_empty(),
            Instr::Element(_) | Instr::Comment { .. } => false,
            _ => true,
        })
    {
        // Point at whatever comes after the (valid) first root — a second
        // root element if there is one, else the end of the first root's
        // subtree (where the offending stray text/control-flow begins).
        // Located by scanning the ORIGINAL source text rather than reusing
        // an IR span, which is relative to the PREPROCESSED source and can
        // land past this source's end (see the module-level span note).
        let elements = top_level_element_spans(source, body_start);
        let span = match elements.as_slice() {
            [_, (second, _), ..] => *second,
            [(_, first_end)] => Span::point(*first_end),
            [] => Span::point(body_start),
        };
        return Err(CompileError::new(
            "element mode: the template must be ONE static root element \
             (control flow at the top level means this is a component)",
        )
        .with_span(span));
    }

    let native = root.tag == name;
    let mut structure = String::new();
    serialize_static(root, &root_class, true, &mut structure)?;

    // Spliced elements key their skin on the stamped root class; native-kind
    // elements key on the tag itself — the tag IS the element.
    let skin_root = if native { name.to_string() } else { format!(".{root_class}") };
    let mut skin_css = element_skin_selectors(&program.styles, &skin_root);
    crate::stdlib::append_descendant_skin(name, &mut skin_css);

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

    let host_classes = root_class_attr(&structure);

    Ok(ElementOutput {
        root_class,
        native,
        structure_html: structure,
        host_classes,
        skin_css,
        manifest,
    })
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
    let mut host_attr_written = false;
    for a in &el.attrs {
        match a {
            AttrInstr::Static { name, value, .. } => {
                let mut value = value.clone();
                if is_root && name == "class" {
                    class_written = true;
                    value = format!("{root_class} {value}");
                }
                if is_root && name == "data-sail-host" {
                    host_attr_written = true;
                }
                out.push(' ');
                out.push_str(name);
                if !value.is_empty() {
                    out.push_str("=\"");
                    out.push_str(&value);
                    out.push('"');
                }
            }
            // `use:action` — a reusable BEHAVIOR (a toolchain action) bundled
            // with the element. It is NOT per-instance authored state, so it
            // doesn't break the compiles-away seam: the action is written once
            // in the toolchain, and here it rides back into the static structure
            // string as a `use:` attribute. When the element splices at a
            // use-site, the normal pipeline lowers that `use:` to a
            // `runAction(el, action)` call — behavior attaches where the runtime
            // already lives, the structure still inlines with zero registration.
            AttrInstr::Use { action, param, .. } => {
                if param.is_some() {
                    return Err(CompileError::new(format!(
                        "element mode: use:{action}={{param}} — a reactive action \
                         parameter is per-instance state and can't ride the static \
                         splice yet; use a param-less action (its config lives in \
                         the action) or author the element as a component",
                    )));
                }
                out.push_str(" use:");
                out.push_str(action);
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
    // Compiler-owned host identity for runtime part discovery (not CSS).
    if is_root && !host_attr_written {
        let host = root_class.strip_prefix("sail-").unwrap_or(root_class);
        out.push_str(&format!(" data-sail-host=\"{host}\""));
    }
    out.push('>');
    // Void elements (hr, br, img, input, …) take NO closing tag — emitting
    // `</hr>` is invalid HTML the template parser rejects when the lowered
    // structure is spliced back into a page. A void element with children
    // is an authoring error (they can't hold content).
    if is_void_element(&el.tag) {
        if !el.children.is_empty() {
            return Err(CompileError::new(format!(
                "element mode: <{}> is a void element and cannot have children",
                el.tag
            )));
        }
        return Ok(());
    }
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

/// The root element's `class` value out of a serialized structure snippet —
/// the exact string `serialize_static` stamped there (`sail-<name>` first,
/// then the author's/sx classes, in order). `serialize_static` always writes
/// the attribute as ` class="…"` on the root open tag.
fn root_class_attr(structure_html: &str) -> String {
    let Some(gt) = structure_html.find('>') else {
        return String::new();
    };
    let open = &structure_html[..gt];
    let Some(p) = open.find(" class=\"") else {
        return String::new();
    };
    let rest = &open[p + " class=\"".len()..];
    let Some(end) = rest.find('"') else {
        return String::new();
    };
    rest[..end].to_string()
}

/// Rewrite the source's `:host`-keyed skin for the element's root class:
/// `:host` → `.sail-<name>`, `:host(<sel>)` → `.sail-<name><sel>`.
///
/// `::slotted(<sel>)` is subtle for L1: unlike a runtime light-DOM component
/// (which keeps a real `<slot>` wrapper, so `slot > X` is right), an element's
/// `<slot>` is REPLACED by the projected children at COMPILE time — the
/// slotted node ends up a descendant of the ROOT. So `::slotted(X)` becomes a
/// root-scoped descendant: `{root} X` when it opens a selector (bare), or just
/// ` X` (descendant of the already-emitted `:host` compound) mid-selector.
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
                // At selector start (last non-ws is a delimiter) the slotted
                // rule has no scope of its own — anchor it to the root.
                let at_selector_start = out
                    .trim_end()
                    .chars()
                    .last()
                    .map(|c| matches!(c, '{' | '}' | ','))
                    .unwrap_or(true);
                if at_selector_start {
                    out.push_str(&root);
                    out.push(' ');
                }
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

/// The HTML void elements — self-closing, no end tag.
pub(crate) fn is_void_element(tag: &str) -> bool {
    matches!(
        tag,
        "area" | "base" | "br" | "col" | "embed" | "hr" | "img" | "input"
            | "link" | "meta" | "param" | "source" | "track" | "wbr"
    )
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
