//! Attribute / element emission. Split from `walker.rs` to keep the
//! state-bearing core (struct + walk loop + Phase E-2 fragment hooks)
//! tight enough to read in one pass.

use crate::ir::{AttrInstr, ElementInstr};

use crate::escape::escape_attr_value;
use super::eval::{eval, stringify, truthy};
use super::walker::Walker;
use super::{EmitDecision, decide_element};

impl<'r> Walker<'r> {
    pub(super) fn walk_element(&mut self, el: &ElementInstr) {
        self.walk_element_with_extra(el, &[]);
    }

    pub(super) fn walk_element_with_extra(
        &mut self,
        el: &ElementInstr,
        extra: &[(&str, &str)],
    ) {
        match decide_element(el) {
            EmitDecision::Skip => {}
            EmitDecision::Void => self.emit_open_tag(el, extra),
            EmitDecision::Normal => {
                self.emit_open_tag(el, extra);
                self.walk_block(&el.children);
                self.out.push_str(&format!("</{}>", el.tag));
            }
        }
    }

    pub(super) fn push_raw(&mut self, s: &str) {
        self.out.push_str(s);
    }

    fn emit_open_tag(&mut self, el: &ElementInstr, extra: &[(&str, &str)]) {
        self.out.push('<');
        self.out.push_str(&el.tag);

        let class_text = self.collect_classes(&el.attrs);
        let style_text = self.collect_styles(&el.attrs);
        let first_class_pos = el.attrs.iter().position(is_class_related);
        let first_style_pos = el.attrs.iter().position(is_style_related);

        for (i, attr) in el.attrs.iter().enumerate() {
            if Some(i) == first_class_pos && !class_text.is_empty() {
                self.push_attr("class", &class_text);
            }
            if Some(i) == first_style_pos && !style_text.is_empty() {
                self.push_attr("style", &style_text);
            }
            self.emit_attr_marker(attr);
        }
        for (name, value) in extra {
            self.push_attr(name, value);
        }
        self.out.push('>');
    }

    /// Walk attributes once and gather every class fragment we want in
    /// the merged `class="..."` attribute — static `class="..."` values
    /// plus any `class:NAME={truthy expr}` toggles.
    fn collect_classes(&self, attrs: &[AttrInstr]) -> String {
        let mut classes: Vec<String> = Vec::new();
        for attr in attrs {
            match attr {
                AttrInstr::Static { name, value, .. } if name == "class" => {
                    if !value.is_empty() {
                        classes.push(value.clone());
                    }
                }
                AttrInstr::Class { name, expr, .. } => {
                    if let Ok(v) = eval(&expr.source, &self.scope)
                        && truthy(&v)
                    {
                        classes.push(name.clone());
                    }
                }
                _ => {}
            }
        }
        classes.join(" ")
    }

    fn collect_styles(&self, attrs: &[AttrInstr]) -> String {
        let mut styles: Vec<String> = Vec::new();
        for attr in attrs {
            match attr {
                AttrInstr::Static { name, value, .. } if name == "style" => {
                    if !value.is_empty() {
                        styles.push(trim_trailing_semi(value));
                    }
                }
                AttrInstr::Style { name, expr, .. } => {
                    if let Ok(v) = eval(&expr.source, &self.scope)
                        && truthy(&v)
                    {
                        styles.push(format!("{name}: {}", stringify(&v)));
                    }
                }
                _ => {}
            }
        }
        styles.join("; ")
    }

    fn emit_attr_marker(&mut self, attr: &AttrInstr) {
        match attr {
            AttrInstr::Static { name, value, .. } => {
                if name == "class" || name == "style" {
                    return;
                }
                self.push_attr(name, value);
            }
            AttrInstr::Expr { name, expr, .. } => {
                let key = self.next_key();
                if let Ok(v) = eval(&expr.source, &self.scope) {
                    self.push_attr(name, &stringify(&v));
                }
                self.push_marker(&format!("attr:{name}"), &key);
            }
            AttrInstr::Class { name, .. } => {
                let key = self.next_key();
                self.push_marker(&format!("class:{name}"), &key);
            }
            AttrInstr::Style { name, .. } => {
                let key = self.next_key();
                self.push_marker(&format!("style:{name}"), &key);
            }
            AttrInstr::Prop { name, .. } => {
                let key = self.next_key();
                self.push_marker(&format!("prop:{name}"), &key);
            }
            AttrInstr::BindValue { expr, .. } => {
                let key = self.next_key();
                if let Ok(v) = eval(&expr.source, &self.scope) {
                    self.push_attr("value", &stringify(&v));
                }
                self.push_marker("bind:value", &key);
            }
            AttrInstr::BindChecked { expr, .. } => {
                let key = self.next_key();
                if let Ok(v) = eval(&expr.source, &self.scope)
                    && truthy(&v)
                {
                    self.out.push(' ');
                    self.out.push_str("checked");
                }
                self.push_marker("bind:checked", &key);
            }
            AttrInstr::BindThis { .. } => {
                let key = self.next_key();
                self.push_marker("bind:this", &key);
            }
            AttrInstr::SlotReflectEmpty { .. } => {
                // No SSR marker — the compiled component setup body
                // wires this declaratively at attach time. Slots have no
                // SSR side-effect on the host attribute set.
            }
            AttrInstr::On { event, .. } => {
                let key = self.next_key();
                self.push_marker(&format!("on:{event}"), &key);
            }
            AttrInstr::DynamicAttr { .. } => {
                let key = self.next_key();
                self.push_marker("attr-dynamic", &key);
            }
            AttrInstr::Mutation { .. } => {
                let key = self.next_key();
                self.push_marker("mutation", &key);
                // Action/method are injected by the host (deck-core), not the
                // compiler. The host owns the /_deck/ URL scheme.
            }
            AttrInstr::Transition { .. } => {
                // Transitions don't surface in SSR output. They register
                // on element construction (CSR / hydrate paths) and
                // consume no SSR marker key.
            }
        }
    }

    fn push_attr(&mut self, name: &str, value: &str) {
        self.out.push(' ');
        self.out.push_str(name);
        self.out.push_str("=\"");
        self.out.push_str(&escape_attr_value(value));
        self.out.push('"');
    }

    fn push_marker(&mut self, name: &str, key: &str) {
        self.out.push(' ');
        self.out.push_str(name);
        self.out.push_str("=\"");
        self.out.push_str(key);
        self.out.push('"');
    }
}

fn is_class_related(attr: &AttrInstr) -> bool {
    matches!(attr, AttrInstr::Static { name, .. } if name == "class")
        || matches!(attr, AttrInstr::Class { .. })
}

fn is_style_related(attr: &AttrInstr) -> bool {
    matches!(attr, AttrInstr::Static { name, .. } if name == "style")
        || matches!(attr, AttrInstr::Style { .. })
}

fn trim_trailing_semi(s: &str) -> String {
    s.trim_end().trim_end_matches(';').trim_end().to_string()
}
