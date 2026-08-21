//! Controller-bearing L1 is mount-form: `mountComponent` the compiled
//! `.sail`. Controller-less L1 still splices to HTML in preprocess.

use super::super::util::camel_of;
use super::super::writer::{Writer, js_string};
use super::TemplateEmitter;
use crate::ir::*;
use std::collections::{HashMap, HashSet};

pub(in crate::emit) struct SlotProj {
    default: Vec<Instr>,
    named: HashMap<String, Vec<Instr>>,
    outer_prefix: String,
}

pub(in crate::emit) struct L1Frame {
    pending: Vec<L1Pending>,
}

struct L1Pending {
    ident: String,
    host: String,
    props: String,
    inherit: Vec<String>,
    bind_value: Option<String>,
}

impl TemplateEmitter {
    /// If `el` is a controller-bearing stdlib tag, emit it as an inlined
    /// component and return the root element var. Otherwise `None`.
    pub(super) fn emit_inlined_stdlib(
        &mut self,
        w: &mut Writer,
        el: &ElementInstr,
        parent_var: &str,
        cleanups_var: &str,
    ) -> Option<String> {
        // `[anatomy]` is closed widget internals — native HTML, not an L1 host.
        if el.attrs.iter().any(|a| attr_named(a, "anatomy")) {
            return None;
        }
        if skip_foreign_input(el) {
            return None;
        }
        let std = crate::stdlib::element(&el.tag)?;
        if std.out.controller_js.is_none() {
            if std.out.native {
                return None;
            }
            return self.emit_inlined_static(w, el, std, parent_var, cleanups_var);
        }
        Some(self.emit_l1_component(w, el, std, parent_var, cleanups_var))
    }

    fn emit_l1_component(
        &mut self,
        w: &mut Writer,
        el: &ElementInstr,
        std: &crate::stdlib::StdlibElement,
        parent_var: &str,
        cleanups_var: &str,
    ) -> String {
        let ident = crate::emit::util::l1_controller_ident(&el.tag);
        let host = self.fresh("h");
        let props = self.fresh("p");
        w.line(&format!("const {host} = document.createElement(\"div\");"));
        w.line(&format!("{host}.style.display = \"contents\";"));
        w.line(&format!(
            "{host}.setAttribute(\"data-component\", {});",
            js_string(&el.tag)
        ));
        w.line(&format!("const {props} = Object.create(null);"));
        for (name, value) in &std.out.default_attrs {
            if skip_seed_attr(name) {
                continue;
            }
            let key = camel_of(name);
            if el.attrs.iter().any(|a| attr_named(a, name)) {
                continue;
            }
            w.line(&format!("{props}.{key} = {};", js_string(value)));
        }
        let mut bind_value = None;
        for a in &el.attrs {
            match a {
                AttrInstr::Static { name, value, .. } if !skip_seed_attr(name) => {
                    w.line(&format!(
                        "{props}.{} = {};",
                        camel_of(name),
                        js_string(value)
                    ));
                }
                AttrInstr::Expr { name, expr, .. } if !skip_seed_attr(name) => {
                    let rw = self.rw(&expr.source, &[]);
                    w.line(&format!("{props}.{} = ({rw});", camel_of(name)));
                }
                AttrInstr::BindValue { expr, .. } => {
                    let rw = self.rw(&expr.source, &[]);
                    w.line(&format!("{props}.value = ({rw});"));
                    bind_value = Some(rw);
                }
                _ => {}
            }
        }
        let (names, labels) = collect_child_names(&el.children);
        if !names.is_empty() {
            let list = names
                .iter()
                .map(|n| js_string(n))
                .collect::<Vec<_>>()
                .join(", ");
            w.line(&format!("{props}.names = [{list}];"));
        }
        if !labels.is_empty() {
            let list = labels
                .iter()
                .map(|n| js_string(n))
                .collect::<Vec<_>>()
                .join(", ");
            w.line(&format!("{props}.labels = [{list}];"));
        }

        let inherit = inherited_fields(std, el);
        if !self.l1_frames.is_empty() {
            self.emit_children(w, &el.children, &host, cleanups_var);
            if !parent_var.is_empty() {
                w.line(&format!("{parent_var}.appendChild({host});"));
            }
            self.l1_frames.last_mut().unwrap().pending.push(L1Pending {
                ident,
                host: host.clone(),
                props,
                inherit,
                bind_value,
            });
            return host;
        }

        self.l1_frames.push(L1Frame {
            pending: Vec::new(),
        });
        self.emit_children(w, &el.children, &host, cleanups_var);
        let ctx = self.fresh("x");
        w.line(&format!("const {ctx} = {{ props: {props} }};"));
        if !names.is_empty() {
            w.line(&format!("{ctx}.names = {props}.names;"));
        }
        if !labels.is_empty() {
            w.line(&format!("{ctx}.labels = {props}.labels;"));
        }
        let inst = self.fresh("m");
        w.line(&format!(
            "const {inst} = mountComponent({ident}, {host}, {ctx});"
        ));
        w.line(&format!("{cleanups_var}.push({inst});"));
        emit_inner_bind_value(w, &host, bind_value.as_deref(), cleanups_var);
        if !parent_var.is_empty() {
            w.line(&format!("{parent_var}.appendChild({host});"));
        }
        self.flush_l1_pending(w, &inst, cleanups_var);
        self.l1_frames.pop();
        host
    }

    fn flush_l1_pending(&mut self, w: &mut Writer, parent_inst: &str, cleanups_var: &str) {
        let pending = self
            .l1_frames
            .last_mut()
            .map(|f| std::mem::take(&mut f.pending))
            .unwrap_or_default();
        for p in pending {
            let ctx = self.fresh("x");
            w.line(&format!("const {ctx} = {{ props: {} }};", p.props));
            w.line(&format!("if ({0}.names) {ctx}.names = {0}.names;", p.props));
            w.line(&format!(
                "if ({0}.labels) {ctx}.labels = {0}.labels;",
                p.props
            ));
            for field in &p.inherit {
                w.line(&format!(
                    "{{ const __v = {parent_inst}.instance.{field}; {ctx}.{field} = __v; {props}.\
                     {field} = __v; }}",
                    props = p.props,
                ));
            }
            let inst = self.fresh("m");
            w.line(&format!(
                "const {inst} = mountComponent({}, {}, {ctx});",
                p.ident, p.host
            ));
            w.line(&format!("{cleanups_var}.push({inst});"));
            emit_inner_bind_value(w, &p.host, p.bind_value.as_deref(), cleanups_var);
        }
    }

    fn emit_inlined_static(
        &mut self,
        w: &mut Writer,
        el: &ElementInstr,
        std: &crate::stdlib::StdlibElement,
        parent_var: &str,
        cleanups_var: &str,
    ) -> Option<String> {
        // Only when we are already inside an inlined component — page
        // source still splices controller-less L1 in preprocess.
        if self.slot_stack.is_empty() || std.out.native {
            return None;
        }
        let root = std.out.template.iter().find_map(|i| match i {
            Instr::Element(e) => Some(e),
            _ => None,
        })?;
        let (default, named) = partition_slots(&el.children);
        self.slot_stack.push(SlotProj {
            default,
            named,
            outer_prefix: self.expr_prefix.clone(),
        });
        let mut fused = root.clone();
        merge_host_class(&mut fused, &std.out.host_classes);
        merge_static_attrs(&mut fused, &el.attrs);
        let v = self.emit_element(w, &fused, parent_var, cleanups_var);
        self.slot_stack.pop();
        Some(v)
    }

    pub(super) fn emit_projected_slot(
        &mut self,
        w: &mut Writer,
        el: &ElementInstr,
        parent_var: &str,
        cleanups_var: &str,
    ) -> bool {
        let Some(proj) = self.slot_stack.last() else {
            return false;
        };
        let name = slot_attr_name(&el.attrs).unwrap_or_default();
        let kids: Vec<Instr> = if name.is_empty() {
            proj.default.clone()
        } else {
            proj.named.get(&name).cloned().unwrap_or_default()
        };
        let outer = proj.outer_prefix.clone();
        let saved = self.expr_prefix.clone();
        self.expr_prefix = outer;
        self.emit_children(w, &kids, parent_var, cleanups_var);
        self.expr_prefix = saved;
        true
    }
}

fn emit_inner_bind_value(w: &mut Writer, host: &str, rw: Option<&str>, cleanups_var: &str) {
    let Some(rw) = rw else {
        return;
    };
    w.line(&format!(
        "{{ const __n = {host}.querySelector(\"input\"); if (__n) {cleanups_var}.push(bindValue(__n, ({rw}))); }}"
    ));
}

/// Spliced widgets whose root is a native `<input>` (checkbox, switch, color)
/// must not be remounted as L1 `<input>`. Closed types have own tags.
fn skip_foreign_input(el: &ElementInstr) -> bool {
    if el.tag != "input" {
        return false;
    }
    if let Some(host) = static_attr(el, "data-sail-host") {
        let host = host.trim_matches(['"', '\'']);
        if host != "input" {
            return true;
        }
    }
    if let Some(ty) = static_attr(el, "type") {
        let ty = ty.trim_matches(['"', '\'']);
        if crate::authoring_bans::banned_input_type(ty).is_some() {
            return true;
        }
    }
    false
}

fn inherited_fields(std: &crate::stdlib::StdlibElement, el: &ElementInstr) -> Vec<String> {
    let Some(js) = std.out.controller_js.as_deref() else {
        return Vec::new();
    };
    let mut own = HashSet::new();
    for (name, _) in &std.out.default_attrs {
        own.insert(camel_of(name));
    }
    for a in &el.attrs {
        match a {
            AttrInstr::Static { name, .. } | AttrInstr::Expr { name, .. }
                if !skip_seed_attr(name) =>
            {
                own.insert(camel_of(name));
            }
            AttrInstr::BindValue { .. } => {
                own.insert("value".into());
            }
            _ => {}
        }
    }
    init_fields(js)
        .into_iter()
        .filter(|f| !own.contains(f) && f != "names" && f != "labels")
        .collect()
}

fn init_fields(js: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut i = 0;
    while let Some(pos) = js[i..].find("init.") {
        let start = i + pos + 5;
        if pos > 0 {
            let before = js[..i + pos].chars().next_back().unwrap_or('\0');
            if before.is_ascii_alphanumeric() || before == '_' || before == '$' {
                i = start;
                continue;
            }
        }
        let ident: String = js[start..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if !ident.is_empty() && !out.iter().any(|s| s == &ident) {
            out.push(ident);
        }
        i = start;
    }
    out
}

fn skip_seed_attr(name: &str) -> bool {
    matches!(
        name,
        "class" | "style" | "role" | "slot" | "hidden" | "anatomy"
    ) || name.starts_with("data-")
        || name.starts_with("aria-")
        || name.starts_with("use:")
        || name.starts_with("_sail")
        || name.contains(':')
}

fn attr_named(a: &AttrInstr, name: &str) -> bool {
    match a {
        AttrInstr::Static { name: n, .. } | AttrInstr::Expr { name: n, .. } => n == name,
        _ => false,
    }
}

fn slot_attr_name(attrs: &[AttrInstr]) -> Option<String> {
    attrs.iter().find_map(|a| match a {
        AttrInstr::Static { name, value, .. } if name == "slot" => Some(value.clone()),
        _ => None,
    })
}

fn static_attr(el: &ElementInstr, name: &str) -> Option<String> {
    el.attrs.iter().find_map(|a| match a {
        AttrInstr::Static { name: n, value, .. } if n == name => Some(value.clone()),
        _ => None,
    })
}

fn collect_static_text(instrs: &[Instr]) -> Option<String> {
    let mut s = String::new();
    collect_static_text_into(instrs, &mut s);
    let t = s.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

fn collect_static_text_into(instrs: &[Instr], out: &mut String) {
    for i in instrs {
        match i {
            Instr::StaticText { text, .. } => out.push_str(text),
            Instr::Element(el) => collect_static_text_into(&el.children, out),
            _ => {}
        }
    }
}

fn collect_child_names(instrs: &[Instr]) -> (Vec<String>, Vec<String>) {
    let mut names = Vec::new();
    let mut labels = Vec::new();
    collect_child_names_into(instrs, &mut names, &mut labels);
    (names, labels)
}

fn collect_child_names_into(instrs: &[Instr], names: &mut Vec<String>, labels: &mut Vec<String>) {
    for i in instrs {
        match i {
            Instr::Element(el) => {
                if el.tag == "option" {
                    let label = collect_static_text(&el.children);
                    let value = static_attr(el, "value")
                        .filter(|s| !s.is_empty())
                        .or_else(|| label.clone());
                    if let Some(v) = value {
                        let lab = label.unwrap_or_else(|| v.clone());
                        names.push(v);
                        labels.push(lab);
                    }
                } else if el.tag == "toggle" {
                    let label = collect_static_text(&el.children);
                    let value = static_attr(el, "value")
                        .filter(|s| !s.is_empty())
                        .or_else(|| label.clone());
                    if let Some(v) = value {
                        names.push(v);
                    }
                } else if el.tag == "tab" || el.tag == "tab-panel" {
                    if let Some(n) = static_attr(el, "name").filter(|s| !s.is_empty()) {
                        names.push(n);
                    } else if let Some(n) = static_attr(el, "title").filter(|s| !s.is_empty()) {
                        names.push(n);
                    }
                }
                collect_child_names_into(&el.children, names, labels);
            }
            Instr::If(v) => {
                for b in &v.branches {
                    collect_child_names_into(&b.body, names, labels);
                }
                if let Some(eb) = &v.else_body {
                    collect_child_names_into(eb, names, labels);
                }
            }
            Instr::For(v) => collect_child_names_into(&v.body, names, labels),
            Instr::Match(v) => {
                for c in &v.cases {
                    collect_child_names_into(&c.body, names, labels);
                }
                if let Some(d) = &v.default {
                    collect_child_names_into(d, names, labels);
                }
            }
            _ => {}
        }
    }
}

fn partition_slots(children: &[Instr]) -> (Vec<Instr>, HashMap<String, Vec<Instr>>) {
    let mut default = Vec::new();
    let mut named: HashMap<String, Vec<Instr>> = HashMap::new();
    for c in children {
        if let Instr::Element(el) = c {
            if let Some(name) = slot_attr_name(&el.attrs) {
                named.entry(name).or_default().push(c.clone());
                continue;
            }
        }
        default.push(c.clone());
    }
    (default, named)
}

fn merge_host_class(root: &mut ElementInstr, host_classes: &str) {
    if host_classes.is_empty() {
        return;
    }
    for a in &mut root.attrs {
        if let AttrInstr::Static { name, value, .. } = a {
            if name.eq_ignore_ascii_case("class") {
                *value = format!("{host_classes} {value}");
                return;
            }
        }
    }
    root.attrs.insert(
        0,
        AttrInstr::Static {
            name: "class".into(),
            value: host_classes.to_string(),
            span: crate::diagnostics::Span::point(0),
        },
    );
}

fn merge_static_attrs(root: &mut ElementInstr, use_attrs: &[AttrInstr]) {
    for a in use_attrs {
        let AttrInstr::Static { name, value, span } = a else {
            continue;
        };
        if skip_seed_attr(name) && name != "class" {
            continue;
        }
        if let Some(existing) = root.attrs.iter_mut().find_map(|x| match x {
            AttrInstr::Static {
                name: n, value: v, ..
            } if n == name => Some(v),
            _ => None,
        }) {
            if name.eq_ignore_ascii_case("class") {
                *existing = format!("{existing} {value}");
            } else {
                *existing = value.clone();
            }
        } else {
            root.attrs.push(AttrInstr::Static {
                name: name.clone(),
                value: value.clone(),
                span: *span,
            });
        }
    }
}
