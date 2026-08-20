//! Job C door: parameterized `use:` / `on:` / live attrs on an L1 element
//! with a `<controller>` attach in **controller scope**, not the page `$scope`.
//!
//! Controller-less static splice still refuses those constructs. With a
//! block they are dropped from `structure_html`. The **page compiler**
//! (not the author controller, not deck) emits the adaptor:
//!
//! ```js
//! const inst = __ctrl_tabs.setup({ title: "Overview" })
//! runAction(el, hotkeys, () => inst.keys)
//! ```

use crate::CompileError;

/// One host- or descendant-node attach compiled from the L1 template.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControllerAttach {
    /// `use:action={param}` — `param` is an ident on the controller return.
    Use {
        action: String,
        param: String,
        /// Empty = the spliced host. Otherwise a `:scope > …` query from
        /// the compiler's static path — page emit, not the controller.
        selector: String,
    },
    /// `on:event={handler}` — `handler` is an ident on the controller return.
    On {
        event: String,
        handler: String,
        selector: String,
    },
    /// `name={expr}` — `expr` is `ident` or `ident()` on the controller return.
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

/// A controller binding used as a `use:` param / `on:` handler / attr expr.
/// Only a bare ident or `ident()` — anything richer is page `$scope` again.
pub fn inst_field(source: &str) -> Result<String, CompileError> {
    let s = source.trim();
    let ident = s.strip_suffix("()").unwrap_or(s).trim();
    let mut chars = ident.chars();
    let Some(first) = chars.next() else {
        return Err(CompileError::new(
            "element mode: empty controller binding on a parameterized use:/on:/attr",
        ));
    };
    let ok = (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_');
    if !ok {
        return Err(CompileError::new(format!(
            "element mode: `{s}` is not a controller binding ident — parameterized \
             use:/on:/attr resolve against the controller return, not the page"
        )));
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

/// Page-emit adaptor for one attach. The author controller never sees this.
pub fn emit_attach_js(el_var: &str, inst_var: &str, attach: &ControllerAttach) -> String {
    match attach {
        ControllerAttach::Use {
            action,
            param,
            selector,
        } => {
            let node = node_js(el_var, selector);
            if selector.is_empty() {
                format!("runAction({node}, {action}, () => {inst_var}.{param});")
            } else {
                format!(
                    "{{ const __n = {node}; if (__n) runAction(__n, {action}, () => {inst_var}.{param}); }}"
                )
            }
        }
        ControllerAttach::On {
            event,
            handler,
            selector,
        } => {
            let node = node_js(el_var, selector);
            format!(
                "{{ const __n = {node}; if (__n) on(__n, \"{event}\", (e) => {{ const __h = {inst_var}.{handler}; if (typeof __h === \"function\") __h(e); }}); }}"
            )
        }
        ControllerAttach::Bind {
            name,
            expr,
            selector,
        } => {
            let node = node_js(el_var, selector);
            format!(
                "{{ const __n = {node}; if (__n) bindAttr(__n, \"{name}\", () => ({inst_var}.{expr})); }}"
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attach_js_is_instance_not_page_scope() {
        let js = emit_attach_js(
            "el",
            "inst",
            &ControllerAttach::Use {
                action: "hotkeys".into(),
                param: "keys".into(),
                selector: String::new(),
            },
        );
        assert_eq!(js, "runAction(el, hotkeys, () => inst.keys);");
        assert!(!js.contains("$scope"));
        assert!(!js.contains("ctx.host"));
    }
}
