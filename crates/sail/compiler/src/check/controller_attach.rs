//! Single source for L1 controller attach validation + type.
//!
//! Parameterized `use:` / `on:` / live `name={expr}` on an L1 element with
//! a `<controller>` attach in controller scope. One ident rule for all
//! three — previously duplicated in `emit::l1_controller::inst_field`.

use crate::CompileError;

/// One host- or descendant-node attach compiled from the L1 template.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControllerAttach {
    Use {
        action: String,
        param: String,
        selector: String,
    },
    On {
        event: String,
        handler: String,
        selector: String,
    },
    Bind {
        name: String,
        expr: String,
        selector: String,
    },
}

/// CSS selector for a descendant of the L1 host, or `""` for the host itself.
pub fn selector_from_path(path: &[String]) -> String {
    if path.is_empty() {
        String::new()
    } else {
        format!(":scope > {}", path.join(" > "))
    }
}

/// A controller binding used as `use:` param / `on:` handler / attr expr.
/// Only a bare ident or `ident()` — anything richer is page `$scope` again.
pub fn inst_field(source: &str) -> Result<String, CompileError> {
    let s = source.trim();
    let ident = s.strip_suffix("()").unwrap_or(s).trim();
    let mut chars = ident.chars();
    let Some(first) = chars.next() else {
        return Err(CompileError::new(
            "element mode: empty controller binding on a parameterized use:/on:/attr",
        )
        .with_category("sail-element-controller"));
    };
    let ok = (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_');
    if !ok {
        return Err(CompileError::new(format!(
            "element mode: `{s}` is not a controller binding ident — parameterized \
             use:/on:/attr resolve against the controller return, not the page"
        ))
        .with_category("sail-element-controller"));
    }
    Ok(s.to_string())
}

fn node_js(el_var: &str, selector: &str) -> String {
    if selector.is_empty() {
        el_var.to_string()
    } else {
        format!(
            "{el_var}.querySelector(\"{}\")",
            selector.replace('\\', "\\\\").replace('"', "\\\"")
        )
    }
}

pub fn emit_attach_js(el_var: &str, inst_var: &str, attach: &ControllerAttach) -> String {
    match attach {
        ControllerAttach::Use { action, param, selector } => {
            let node = node_js(el_var, selector);
            if selector.is_empty() {
                format!("runAction({node}, {action}, () => {inst_var}.{param});")
            } else {
                format!("{{ const __n = {node}; if (__n) runAction(__n, {action}, () => {inst_var}.{param}); }}")
            }
        }
        ControllerAttach::On { event, handler, selector } => {
            let node = node_js(el_var, selector);
            format!("{{ const __n = {node}; if (__n) on(__n, \"{event}\", (e) => {{ const __h = {inst_var}.{handler}; if (typeof __h === \"function\") __h(e); }}); }}")
        }
        ControllerAttach::Bind { name, expr, selector } => {
            let node = node_js(el_var, selector);
            format!("{{ const __n = {node}; if (__n) bindAttr(__n, \"{name}\", () => ({inst_var}.{expr})); }}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn attach_js_is_instance_not_page_scope() {
        let js = emit_attach_js("el", "inst", &ControllerAttach::Use { action: "hotkeys".into(), param: "keys".into(), selector: String::new() });
        assert_eq!(js, "runAction(el, hotkeys, () => inst.keys);");
        assert!(!js.contains("$scope"));
        assert!(!js.contains("ctx.host"));
    }
}
