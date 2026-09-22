//! `sail-light-shadow-root` — Sail L1 components are light DOM.
//!
//! Controllers must not wire against bare `.shadowRoot` (null for light
//! components). Prefer the host element. The residual form
//! `host.shadowRoot ?? host` is allowed for rare foreign-content
//! containment wrappers that still attach a shadow root — it is not the
//! L1 product model.

use crate::check::controller_parse::ParsedController;
use crate::diagnostics::CompileError;

pub const RULE_ID: &str = "sail-light-shadow-root";

pub fn check(controller: Option<&ParsedController>) -> Result<(), CompileError> {
    let Some(c) = controller else {
        return Ok(());
    };
    let needle = ".shadowRoot";
    let mut from = 0;
    while let Some(rel) = c.source[from..].find(needle) {
        let at = from + rel;
        let after = c.source[at + needle.len()..].trim_start();
        if !after.starts_with("??") {
            let line = c.source[..at].bytes().filter(|&b| b == b'\n').count() + 1;
            return Err(CompileError::new(format!(
                "`{}` line {line} reads `.shadowRoot`, but Sail components are light DOM \
                 — the shadow root is null at runtime, so this wiring is dead. Use the \
                 host element (or residual `host.shadowRoot ?? host` only for foreign-content wrappers).",
                c.path.display()
            ))
            .with_category(RULE_ID.to_string()));
        }
        from = at + needle.len();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::controller_parse;
    use std::path::PathBuf;

    fn ctrl(src: &str) -> ParsedController {
        controller_parse::parse(PathBuf::from("t.controller.ts"), src.to_string()).expect("parses")
    }

    #[test]
    fn light_component_with_bare_shadow_root_errors() {
        let c = ctrl(
            "export default controller((ctx, host) => { mount((el) => { host.shadowRoot.addEventListener('click', () => {}) }); return {} })",
        );
        let err = check(Some(&c)).unwrap_err();
        assert_eq!(err.category.as_deref(), Some(RULE_ID));
    }

    #[test]
    fn optional_chain_is_still_dead_wiring() {
        let c = ctrl(
            "export default controller((ctx, host) => { mount((el) => { host.shadowRoot?.addEventListener('click', () => {}) }); return {} })",
        );
        assert!(check(Some(&c)).is_err());
    }

    #[test]
    fn root_agnostic_fallback_passes() {
        let c = ctrl(
            "export default controller((ctx, host) => { mount((el) => { const root = host.shadowRoot ?? host; root.addEventListener('click', () => {}) }); return {} })",
        );
        check(Some(&c)).unwrap();
    }
}
