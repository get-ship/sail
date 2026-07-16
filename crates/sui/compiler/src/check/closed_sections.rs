//! Closed-sections rule: a `controller(def)` def object literal may
//! only contain keys from a fixed allowlist. Anything else is a
//! compile error — making typos like `effects` (vs `derived`) fail
//! loudly instead of silently doing nothing at runtime.
//!
//! See `.artifacts/job-spec.md` (phase 3, "Compiler validations").

use crate::check::controller_parse::ParsedController;
use crate::diagnostics::CompileError;

const ALLOWED: &[&str] = &["setup", "derived", "on", "mount", "unmount"];

pub fn check(controller: &ParsedController) -> Result<(), CompileError> {
    for section in &controller.sections {
        if ALLOWED.contains(&section.name.as_str()) {
            continue;
        }
        return Err(CompileError::new(format!(
            "Unknown controller section '{}'. Valid sections are: {}.",
            section.name,
            ALLOWED.join(", ")
        ))
        .with_span(section.key_span)
        .with_path(controller.path.clone())
        .with_category("controller(def)"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::controller_parse::parse;
    use std::path::PathBuf;

    fn parse_src(src: &str) -> ParsedController {
        parse(PathBuf::from("t.ts"), src.into()).expect("controller call expected")
    }

    #[test]
    fn allows_full_section_set() {
        let src = r#"
export default controller({
  setup: () => ({}),
  derived: {},
  on: {},
  mount: () => {},
  unmount: () => {},
})
"#;
        check(&parse_src(src)).expect("all five sections should be allowed");
    }

    #[test]
    fn rejects_unknown_section_with_message() {
        let src = r#"
export default controller({
  derived: {},
  effects: {},
})
"#;
        let err = check(&parse_src(src)).expect_err("effects should fail");
        assert!(err.message.contains("Unknown controller section 'effects'"));
        assert!(err.message.contains("setup, derived, on, mount, unmount"));
    }

    #[test]
    fn empty_def_is_legal() {
        let src = r#"export default controller({})"#;
        check(&parse_src(src)).expect("empty def is fine");
    }
}
