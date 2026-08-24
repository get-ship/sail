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
//! Element templates may include `{#if}` / `{#for}` / `{#match}` (legal in
//! L1). Those compile with the render machinery — same tag, same registry.
//! Dynamic attributes still refuse on a **controller-less** splice. An
//! inline `<controller>` is Job C: parameterized `use:` / `on:` / live
//! attrs attach from the controller instance, not the page `$scope`.
//! The attach type lives in `crate::check::controller_attach`; `emit`
//! only interprets it. The host stays an opaque splice stamped
//! `data-sail-ctrl`.

use crate::CompileError;
use crate::diagnostics::Span;
use crate::ir::*;
use serde_json::{Value, json};

const RULE_ID: &str = "sail-element-mode";
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
        let Some(name_end_off) =
            source[i + 1..].find(|c: char| c.is_whitespace() || c == '>' || c == '/')
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
                source
                    .as_bytes()
                    .get(o + open_prefix.len())
                    .is_some_and(|b| matches!(b, b' ' | b'\t' | b'\n' | b'\r' | b'>' | b'/'))
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

/// Native inner pack: if the host's only element child wraps the default
/// `<slot>` (`<button><row cluster gap="2"><slot></slot></row></button>`),
/// return that child's open tag. Slot-as-direct-child (label) or extra
/// siblings (req mark) return None so stamp-only stays stamp-only.
fn extract_native_inner_pack(source: &str, host: &str) -> Option<String> {
    let body_start = skip_preamble(source);
    let roots = top_level_element_spans(source, body_start);
    let (name_span, host_end) = roots.first().copied()?;
    if &source[name_span.start..name_span.end] != host {
        return None;
    }
    let host_open_start = name_span.start.saturating_sub(1);
    let rel = source.get(host_open_start..host_end)?.find('>')?;
    let inner_start = host_open_start + rel + 1;
    let close = format!("</{host}>");
    if host_end < close.len() {
        return None;
    }
    let inner_end = host_end - close.len();
    if inner_start >= inner_end {
        return None;
    }
    let kids: Vec<_> = top_level_element_spans(source, inner_start)
        .into_iter()
        .filter(|(span, _)| span.start < inner_end)
        .collect();
    if kids.len() != 1 {
        return None;
    }
    let (kid_name, kid_end) = kids[0];
    let kid_tag = &source[kid_name.start..kid_name.end];
    if kid_tag == "slot" {
        return None;
    }
    let kid_start = kid_name.start.saturating_sub(1);
    if kid_start >= source.len() || !source[kid_start..].starts_with('<') {
        return None;
    }
    let kid_html = source.get(kid_start..kid_end.min(inner_end))?;
    if !contains_default_slot(kid_html) {
        return None;
    }
    let gt = kid_html.find('>')?;
    Some(kid_html[..=gt].to_string())
}

/// Skip leading comments plus `<scope>` / `<style>` so the template root
/// is the first real host tag (button.sail opens with a file comment).
fn skip_preamble(source: &str) -> usize {
    let mut i = 0usize;
    loop {
        let rest = source[i..].trim_start();
        i += source[i..].len() - rest.len();
        if source[i..].starts_with("<!--") {
            match source[i..].find("-->") {
                Some(off) => i += off + 3,
                None => return i,
            }
            continue;
        }
        if rest.starts_with("<scope") {
            match source[i..].find("</scope>") {
                Some(off) => i += off + "</scope>".len(),
                None => return i,
            }
            continue;
        }
        if rest.starts_with("<style") {
            match source[i..].find("</style>") {
                Some(off) => i += off + "</style>".len(),
                None => return i,
            }
            continue;
        }
        return i;
    }
}

fn contains_default_slot(html: &str) -> bool {
    let bytes = html.as_bytes();
    let mut i = 0;
    while i + 5 <= bytes.len() {
        if html[i..].starts_with("<slot")
            && bytes
                .get(i + 5)
                .is_some_and(|&b| b.is_ascii_whitespace() || b == b'>' || b == b'/')
        {
            let mut k = i + 5;
            let mut quote: Option<u8> = None;
            while k < bytes.len() {
                match (quote, bytes[k]) {
                    (Some(q), b) if b == q => quote = None,
                    (None, b'"') | (None, b'\'') => quote = Some(bytes[k]),
                    (None, b'>') => break,
                    _ => {}
                }
                k += 1;
            }
            if k >= bytes.len() {
                return false;
            }
            let open = &html[i..=k];
            if open_tag_has_name(open).is_none() {
                return true;
            }
        }
        i += 1;
    }
    false
}

fn open_tag_has_name(open: &str) -> Option<()> {
    let inner = open
        .trim_start_matches('<')
        .trim_end_matches('>')
        .trim_end_matches('/');
    let after = inner.split_once(char::is_whitespace)?.1;
    for part in after.split_whitespace() {
        let name = part.split('=').next().unwrap_or("");
        if name == "name" {
            return Some(());
        }
    }
    None
}

/// The element-mode artifacts.
#[derive(Debug, Clone)]
pub struct ElementOutput {
    /// Root class the lowering stamps and the skin keys on: `sail-<name>`.
    /// (Unused for native-kind elements — their skin keys on the tag.)
    pub root_class: String,
    /// NATIVE-KIND: the template root IS the element tag (label → <label>).
    /// The tag already exists in HTML; host paint is stamped, not spliced —
    /// rewriting the host would clobber every real HTML use. An inner pack
    /// around the default slot (button's `<row cluster>`) still wraps
    /// use-site children; siblings of the slot (label's req mark) do not.
    pub native: bool,
    /// Authored open tag of the single child that wraps the default slot
    /// (`<row cluster gap="2">`). Native lowering wraps use-site children
    /// with this; `None` when the slot is a direct host child.
    pub inner_pack: Option<String>,
    /// Static structure snippet (single root element, slots preserved).
    pub structure_html: String,
    /// The class string `serialize_static` stamped on the structure root:
    /// `sail-<name>` + the atomic sx classes the host `sx` lowered to.
    /// Spliced elements carry it on the snippet root; NATIVE-kind elements
    /// have no splice, so the use-site lowering stamps this same string onto
    /// the real tag (preprocess `lower_into`) — otherwise the host paint
    /// keyed on those classes never reaches a single element.
    pub host_classes: String,
    /// Prop defaults to stamp as host attributes when the use site omits them
    /// (`size="md"`, …) so Style match/compound attr selectors match.
    /// Tone has no default — rest paint is omit. Bool false is omitted; bool
    /// true is presence (`name` with empty value).
    pub default_attrs: Vec<(String, String)>,
    /// Prop names bound via `bind(name)` / Tone group paths with their unit
    /// realization — use-site splice inlines static first-paint custom
    /// properties (`offset="16"` → `style="--offset: 16px"`; `tone="primary"`
    /// → `style="--tone: var(--primary); …"`).
    pub bound_props: Vec<(String, crate::check::binding::StyleBindingUnit)>,
    /// Skin CSS — keyed on `.sail-<name>` for spliced elements, on the bare
    /// tag selector for native-kind (from the source's `:host` idioms).
    pub skin_css: String,
    /// The element manifest (contract): tag, attributes, anatomy.
    pub manifest: Value,
    /// Root `_sail_m_<trigger>` stamps (enter/exit play). Spliced hosts
    /// already carry them in `structure_html`; native-kind lowering copies
    /// them onto the real tag so `/motion` still attaches.
    pub motion_stamps: Vec<(String, String)>,
    /// Parameterized `use:` / `on:` / live attrs collected from a
    /// controller-bearing template. Not serialized into `structure_html`.
    /// Single source — `crate::check::controller_attach::ControllerAttach`.
    pub controller_attaches: Vec<crate::check::controller_attach::ControllerAttach>,
    /// Wrapped `<controller>` module (`{tag}.sail.controller.ts`). `None` when the
    /// element has no block.
    pub controller_js: Option<String>,
    /// Template IR. Controller-bearing L1 is inlined through the same
    /// template emit as app components (nested `$scope` + `setup()`), not
    /// a second attach runner.
    pub template: Vec<crate::ir::Instr>,
}

pub fn emit_element(
    program: &Program,
    source: &str,
    name: &str,
) -> Result<ElementOutput, CompileError> {
    emit_element_with_bindings(
        program,
        source,
        name,
        &[],
        false,
        &crate::CompileOpts::default(),
    )
}

pub fn emit_element_with_bindings(
    program: &Program,
    source: &str,
    name: &str,
    style_bindings: &[crate::check::binding::StyleBinding],
    has_controller: bool,
    opts: &crate::CompileOpts,
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
        ))
        .with_category(RULE_ID));
    }
    let root_class = format!("{}-{name}", opts.element_prefix);

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
        return Err(
            CompileError::new("element mode: the template needs exactly one root element")
                .with_span(Span::point(body_start))
                .with_category(RULE_ID),
        );
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
        .with_span(span)
        .with_category(RULE_ID));
    }

    let native = root.tag == name;
    let mut structure = String::new();
    let mut controller_attaches = Vec::new();
    serialize_static(
        root,
        &root_class,
        true,
        &[],
        has_controller,
        true,
        &mut controller_attaches,
        &mut structure,
    )?;
    // Fixed host role/live/label from the anatomy table (authors don't stamp).
    structure = crate::semantic_props::stamp_host_anatomy(name, &structure);
    // Prop defaults must land as DOM attributes on the structure root:
    // variant-scoped skin CSS is attribute-gated (`[variant='drop']
    // .s-variant-drop-… { position: relative; … }`), and match-arm classes
    // alone never satisfy the attribute selector. Without this the host is
    // unstyled — file's invisible input then resolves `inset: 0` against
    // the viewport and swallows every click on the page.
    let default_attrs = default_prop_attrs(name, &program.props);
    let open_end = structure.find('>').map(|p| p + 1).unwrap_or(0);
    for (attr_name, attr_value) in &default_attrs {
        if !structure[..open_end].contains(&format!("{attr_name}=")) {
            stamp_attr_on_first_tag(&mut structure, name, attr_name, attr_value);
        }
    }

    // Spliced elements key their skin on the stamped root class; native-kind
    // elements key on the tag itself — the tag IS the element.
    let skin_root = if native {
        name.to_string()
    } else {
        format!(".{root_class}")
    };
    let skin_css = element_skin_selectors(&program.styles, &skin_root);

    let attributes: Vec<Value> = program
        .props
        .iter()
        .map(|p| {
            let mut e = json!({ "name": p.name });
            if let PropKind::Union(values) = &p.kind {
                e.as_object_mut()
                    .unwrap()
                    .insert("values".into(), json!(values));
            }
            if let Some(d) = p.default.as_ref() {
                e.as_object_mut()
                    .unwrap()
                    .insert("default".into(), json!(d));
            }
            e
        })
        .collect();
    let (classes, slots) = super::cem::collect_anatomy(&program.template);
    // Surface fixed host semantics in the element contract (skill / CEM).
    let host_sem = crate::semantic_props::host_anatomy(name).map(|a| {
        let mut m = serde_json::Map::new();
        if let Some(r) = a.role {
            m.insert("role".into(), json!(r));
        }
        if let Some(l) = a.live {
            m.insert("live".into(), json!(l));
        }
        if let Some(at) = a.atomic {
            m.insert("atomic".into(), json!(at));
        }
        if let Some(lab) = a.label {
            m.insert("label".into(), json!(lab));
        }
        Value::Object(m)
    });
    let mut anatomy = json!({ "dom": "light", "classes": classes, "slots": slots });
    if let Some(hs) = host_sem {
        anatomy.as_object_mut().unwrap().insert("host".into(), hs);
    }
    let manifest = json!({
        "kind": "element",
        "tag": name,
        "native": native,
        "rootClass": root_class,
        "attributes": attributes,
        "anatomy": anatomy,
    });

    let host_classes = root_class_attr(&structure);
    let bound_props: Vec<(String, crate::check::binding::StyleBindingUnit)> = style_bindings
        .iter()
        .filter(|b| b.source == crate::check::binding::StyleBindingSource::Prop)
        .map(|b| (b.scope_name.clone(), b.unit))
        .collect();

    let inner_pack = if native {
        extract_native_inner_pack(source, name)
    } else {
        None
    };
    let motion_stamps = collect_root_motion_stamps(&structure);

    Ok(ElementOutput {
        root_class,
        native,
        inner_pack,
        structure_html: structure,
        host_classes,
        default_attrs,
        bound_props,
        skin_css,
        manifest,
        motion_stamps,
        controller_attaches,
        controller_js: None,
        template: program.template.clone(),
    })
}

/// Insert `attr="value"` on the first `<tag …>` if that attr is absent.
fn stamp_attr_on_first_tag(html: &mut String, tag: &str, attr: &str, value: &str) {
    let open = format!("<{tag}");
    let Some(pos) = html.find(&open) else {
        return;
    };
    let Some(gt) = html[pos..].find('>') else {
        return;
    };
    let open_tag = &html[pos..pos + gt];
    let needle = format!("{attr}=");
    if open_tag.contains(&needle) {
        return;
    }
    html.insert_str(pos + gt, &format!(" {attr}=\"{value}\""));
}

/// Reconstruct the `_sail_m_*` config object parse_motion accepts.
fn motion_config_literal(
    props: &[crate::ir::MotionProp],
    stagger_ms: Option<u32>,
    timing: &crate::ir::MotionTiming,
) -> String {
    let mut parts: Vec<String> = Vec::new();
    for p in props {
        let mut fields: Vec<String> = Vec::new();
        if let Some(v) = &p.from {
            fields.push(format!("from: {}", motion_value_literal(v)));
        }
        if let Some(v) = &p.to {
            fields.push(format!("to: {}", motion_value_literal(v)));
        }
        if let Some(preset) = &p.preset
            && preset != "inline"
            && fields.is_empty()
        {
            parts.push(format!("{}: '{preset}'", p.property));
            continue;
        }
        if fields.is_empty() {
            continue;
        }
        parts.push(format!("{}: {{ {} }}", p.property, fields.join(", ")));
    }
    if let Some(ms) = stagger_ms {
        parts.push(format!("stagger: {ms}"));
    }
    if let Some(v) = &timing.duration {
        parts.push(format!("duration: {}", motion_value_literal(v)));
    }
    if let Some(v) = &timing.delay {
        parts.push(format!("delay: {}", motion_value_literal(v)));
    }
    if let Some(v) = &timing.easing {
        parts.push(format!("easing: {}", motion_value_literal(v)));
    }
    if let Some(v) = &timing.iterations {
        parts.push(format!("iterations: {}", motion_value_literal(v)));
    }
    if let Some(v) = &timing.direction {
        parts.push(format!("direction: {}", motion_value_literal(v)));
    }
    format!("{{ {} }}", parts.join(", "))
}

fn motion_value_literal(v: &crate::ir::MotionValue) -> String {
    match v {
        crate::ir::MotionValue::Num(n) if *n == n.trunc() => format!("{}", *n as i64),
        crate::ir::MotionValue::Num(n) => format!("{n}"),
        crate::ir::MotionValue::Str(s) => format!("'{s}'"),
        crate::ir::MotionValue::Bezier([a, b, c, d]) => format!("[{a}, {b}, {c}, {d}]"),
        crate::ir::MotionValue::Expr(e) => e.source.clone(),
    }
}

/// `_sail_m_<trigger>="…"` on the structure root — native lowering copies these.
fn collect_root_motion_stamps(structure: &str) -> Vec<(String, String)> {
    let Some(gt) = structure.find('>') else {
        return Vec::new();
    };
    let open = &structure[..gt];
    let mut out = Vec::new();
    let mut rest = open;
    while let Some(idx) = rest.find("_sail_m_") {
        rest = &rest[idx + "_sail_m_".len()..];
        let Some(eq) = rest.find('=') else {
            break;
        };
        let trigger = rest[..eq].trim().to_string();
        rest = rest[eq + 1..].trim_start();
        let Some(q) = rest.chars().next().filter(|c| *c == '"' || *c == '\'') else {
            continue;
        };
        rest = &rest[1..];
        let Some(end) = rest.find(q) else {
            break;
        };
        let value = rest[..end].replace("&quot;", "\"");
        out.push((trigger, value));
        rest = &rest[end + 1..];
    }
    out
}

/// Retained for test-compat — element stdlib attach now rides `path` via
/// controller wiring, not DOM `data-sail-ctrl` discovery. No-op.
pub fn stamp_ctrl(_structure: &mut String, _tag: &str) {}

/// Attribute defaults for freeform hosts. Skips Identity props and bool false.
///
/// `<a>` is dual-mode: bare = text link (sail-base `a:not([variant])`), with
/// `variant` = button link. Stamping the prop defaults would force every freeform
/// href into button chrome and kill site-wide text links — so skip variant/tone.
fn default_prop_attrs(tag: &str, props: &[crate::ir::PropDecl]) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for p in props {
        if tag == "a" && (p.name == "variant" || p.name == "tone") {
            continue;
        }
        match &p.kind {
            crate::ir::PropKind::Identity(_) => continue,
            crate::ir::PropKind::Bool => {
                if p.default.as_deref() == Some("true") {
                    out.push((p.name.clone(), String::new()));
                }
            }
            _ => {
                if let Some(d) = p.default.as_ref() {
                    if !d.is_empty() {
                        out.push((p.name.clone(), d.clone()));
                    }
                }
            }
        }
    }
    out
}

/// Serialize a static element tree. `is_root` stamps the root class so the
/// skin and the use-site lowering agree on the key. Controller-less
/// templates still refuse live attrs; with a controller they become
/// [`ControllerAttach`] entries instead of splice HTML.
fn serialize_static(
    el: &ElementInstr,
    root_class: &str,
    is_root: bool,
    path: &[String],
    has_controller: bool,
    allow_attach: bool,
    attaches: &mut Vec<crate::check::controller_attach::ControllerAttach>,
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
                // Semantic props (label/current/…) lower to aria-* in structure.
                let emit_name =
                    crate::semantic_props::lower_attr_name_str_valued(&el.tag, name, Some(&value));
                let mut stamp_value = value.clone();
                if crate::semantic_props::is_aria_bool_attr(emit_name) {
                    stamp_value =
                        crate::semantic_props::coerce_aria_bool_static(&stamp_value).to_string();
                }
                out.push(' ');
                out.push_str(emit_name);
                if !stamp_value.is_empty() {
                    out.push_str("=\"");
                    out.push_str(&stamp_value);
                    out.push('"');
                }
                if let Some(extra) = crate::semantic_props::dual_stamp_aria(&el.tag, name) {
                    let extra_val = if crate::semantic_props::is_aria_bool_attr(extra) {
                        crate::semantic_props::coerce_aria_bool_static(&value).to_string()
                    } else {
                        stamp_value.clone()
                    };
                    out.push(' ');
                    out.push_str(extra);
                    if !extra_val.is_empty() {
                        out.push_str("=\"");
                        out.push_str(&extra_val);
                        out.push('"');
                    }
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
                if let Some(p) = param {
                    if !has_controller || !allow_attach {
                        return Err(CompileError::new(format!(
                            "element mode: use:{action}={{param}} — a reactive action \
                             parameter is per-instance state and can't ride the static \
                             splice; add a <controller> so it attaches in controller \
                             scope, or use a param-less action",
                        ))
                        .with_category(RULE_ID));
                    }
                    let field = crate::check::controller_attach::inst_field(&p.source)?;
                    attaches.push(crate::check::controller_attach::ControllerAttach::Use {
                        action: action.clone(),
                        param: field,
                        path: path.to_vec(),
                    });
                    continue;
                }
                out.push_str(" use:");
                out.push_str(action);
            }
            AttrInstr::On { event, handler, .. } => {
                if !has_controller || !allow_attach {
                    return Err(CompileError::new(format!(
                        "element mode: dynamic attribute on <{}> — elements are static \
                         structure; per-instance behavior belongs to the use site, a \
                         <controller>, or a component",
                        el.tag
                    ))
                    .with_category(RULE_ID));
                }
                let field = crate::check::controller_attach::inst_field(&handler.source)?;
                attaches.push(crate::check::controller_attach::ControllerAttach::On {
                    event: event.clone(),
                    handler: field,
                    path: path.to_vec(),
                });
            }
            AttrInstr::Expr { name, expr, .. } => {
                if !has_controller || !allow_attach {
                    return Err(CompileError::new(format!(
                        "element mode: dynamic attribute on <{}> — elements are static \
                         structure; per-instance behavior belongs to the use site, a \
                         <controller>, or a component",
                        el.tag
                    ))
                    .with_category(RULE_ID));
                }
                let field = crate::check::controller_attach::inst_field(&expr.source)?;
                let path_vec = path.to_vec();
                attaches.push(crate::check::controller_attach::ControllerAttach::Bind {
                    name: name.clone(),
                    expr: field.clone(),
                    path: path_vec.clone(),
                });
                if let Some(extra) = crate::semantic_props::dual_stamp_aria(&el.tag, name) {
                    attaches.push(crate::check::controller_attach::ControllerAttach::Bind {
                        name: extra.to_string(),
                        expr: field,
                        path: path_vec,
                    });
                }
            }
            AttrInstr::Controller { .. } => {
                // `data-sail-ctrl` retired — controller attach rides `ControllerAttach::path`,
                // not DOM discovery. No splice attr.
            }
            // `animate: { enter: play(fade) }` — same as `use:`: toolchain
            // behavior that rides the static splice as `_sail_m_*`. The page
            // parse lowers it to `__sail_motion_attach`. Dynamic magnitudes
            // (signal from/to) still refuse.
            AttrInstr::Motion {
                trigger,
                props,
                stagger_ms,
                timing,
                ..
            } => {
                if props.iter().any(|p| !p.exprs().is_empty()) || !timing.exprs().is_empty() {
                    return Err(CompileError::new(format!(
                        "element mode: dynamic motion on <{}> — enter/exit play() \
                         must be a static play, not a per-instance signal",
                        el.tag
                    ))
                    .with_category(RULE_ID));
                }
                let config = motion_config_literal(props, *stagger_ms, timing);
                let value = format!("{{{config}}}");
                let escaped = value.replace('"', "&quot;");
                out.push_str(" _sail_m_");
                out.push_str(trigger);
                out.push_str("=\"");
                out.push_str(&escaped);
                out.push('"');
            }
            _ => {
                return Err(CompileError::new(format!(
                    "element mode: dynamic attribute on <{}> — elements are static \
                     structure; per-instance behavior belongs to the use site or a \
                     component",
                    el.tag
                ))
                .with_category(RULE_ID));
            }
        }
    }
    if is_root && !class_written {
        out.push_str(&format!(" class=\"{root_class}\""));
    }
    // Compiler-owned host identity for runtime part discovery (not CSS).
    // Prefix-aware: strip "{prefix}-" not hardcoded "sail-".
    if is_root && !host_attr_written {
        let host = root_class
            .strip_prefix(&format!(
                "{}-",
                crate::CompileOpts::default().element_prefix
            ))
            .unwrap_or(root_class.as_ref() as &str)
            .to_string();
        // Attr name is opts.host_data_attr in next plumbing; today web default is data-sail-host.
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
            ))
            .with_category(RULE_ID));
        }
        return Ok(());
    }
    serialize_static_children(
        &el.children,
        root_class,
        path,
        has_controller,
        allow_attach,
        attaches,
        out,
        &el.tag,
    )?;
    out.push_str(&format!("</{}>", el.tag));
    Ok(())
}

fn serialize_static_children(
    children: &[Instr],
    root_class: &str,
    path: &[String],
    has_controller: bool,
    allow_attach: bool,
    attaches: &mut Vec<crate::check::controller_attach::ControllerAttach>,
    out: &mut String,
    parent_tag: &str,
) -> Result<(), CompileError> {
    let mut tag_nth: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for child in children {
        match child {
            Instr::Element(c) => {
                let n = tag_nth.entry(c.tag.as_str()).or_insert(0);
                *n += 1;
                let mut child_path = path.to_vec();
                child_path.push(format!("{}:nth-of-type({n})", c.tag));
                serialize_static(
                    c,
                    root_class,
                    false,
                    &child_path,
                    has_controller,
                    allow_attach,
                    attaches,
                    out,
                )?;
            }
            Instr::StaticText { text, .. } => out.push_str(text),
            Instr::Comment { .. } => {}
            Instr::DynamicText { expr } => {
                if has_controller {
                    if let Some(field) = simple_prop_text(&expr.source) {
                        out.push_str("<!--sail:prop:");
                        out.push_str(field);
                        out.push_str("-->");
                    } else {
                        let field = crate::check::controller_attach::inst_field(&expr.source)?;
                        out.push_str("<!--sail:ctrl-text:");
                        out.push_str(&field);
                        out.push_str("-->");
                    }
                } else {
                    let Some(prop) = simple_prop_text(&expr.source) else {
                        return Err(CompileError::new(format!(
                            "element mode: <{parent_tag}> contains dynamic content that is not a prop — \
                             bind a prop or use control flow"
                        ))
                        .with_category(RULE_ID));
                    };
                    out.push_str("<!--sail:prop:");
                    out.push_str(prop);
                    out.push_str("-->");
                }
            }
            Instr::If(i) => {
                out.push_str("<!--sail:if-->");
                for b in &i.branches {
                    serialize_static_instrs(
                        &b.body,
                        root_class,
                        path,
                        has_controller,
                        allow_attach,
                        attaches,
                        out,
                    )?;
                }
                if let Some(eb) = &i.else_body {
                    serialize_static_instrs(
                        eb,
                        root_class,
                        path,
                        has_controller,
                        allow_attach,
                        attaches,
                        out,
                    )?;
                }
            }
            Instr::For(f) => {
                out.push_str("<!--sail:for-->");
                serialize_static_instrs(
                    &f.body,
                    root_class,
                    path,
                    has_controller,
                    allow_attach,
                    attaches,
                    out,
                )?;
            }
            Instr::Match(m) => {
                out.push_str("<!--sail:match-->");
                for c in &m.cases {
                    serialize_static_instrs(
                        &c.body,
                        root_class,
                        path,
                        has_controller,
                        allow_attach,
                        attaches,
                        out,
                    )?;
                }
                if let Some(d) = &m.default {
                    serialize_static_instrs(
                        d,
                        root_class,
                        path,
                        has_controller,
                        allow_attach,
                        attaches,
                        out,
                    )?;
                }
            }
            Instr::When(w) => {
                serialize_static_instrs(
                    &w.body,
                    root_class,
                    path,
                    has_controller,
                    allow_attach,
                    attaches,
                    out,
                )?;
                if let Some(eb) = &w.else_body {
                    serialize_static_instrs(
                        eb,
                        root_class,
                        path,
                        has_controller,
                        allow_attach,
                        attaches,
                        out,
                    )?;
                }
            }
        }
    }
    Ok(())
}

fn serialize_static_instrs(
    instrs: &[Instr],
    root_class: &str,
    path: &[String],
    has_controller: bool,
    allow_attach: bool,
    attaches: &mut Vec<crate::check::controller_attach::ControllerAttach>,
    out: &mut String,
) -> Result<(), CompileError> {
    serialize_static_children(
        instrs,
        root_class,
        path,
        has_controller,
        allow_attach,
        attaches,
        out,
        "element",
    )
}

/// Bare prop identifier, optionally called (`title` / `title()`). Anything
/// with operators, member access, or literals is a component expression.
fn simple_prop_text(src: &str) -> Option<&str> {
    let s = src.trim();
    let s = s.strip_suffix("()").unwrap_or(s).trim();
    if s.is_empty() {
        return None;
    }
    let mut chars = s.chars();
    let first = chars.next()?;
    if !first.is_ascii_alphabetic() && first != '_' {
        return None;
    }
    if chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
        Some(s)
    } else {
        None
    }
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

/// HTML serialize-void (WHATWG). Not author-void.
pub(crate) fn is_void_element(tag: &str) -> bool {
    crate::escape::is_void_element(tag)
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
