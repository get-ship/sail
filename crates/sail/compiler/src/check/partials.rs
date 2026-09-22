//! Semantic checks for in-file `<partial>` (SPEC-PARTIAL-1).
//!
//! Expansion lives in `parse::partials`; this pass validates on the **author**
//! source (pre-expansion): leaked controller idents, missing/excess attrs vs
//! the props type, and tag collision with `src/<tag>/<tag>.sail`.

use std::collections::BTreeSet;
use std::path::Path;

use crate::check::controller_parse::ParsedController;
use crate::diagnostics::{CompileError, Diagnostic};
use crate::parse::partials;

pub fn check(
    source: &str,
    controller: Option<&ParsedController>,
    source_path: Option<&Path>,
) -> Result<(), CompileError> {
    let errs = collect_errors(source, controller, source_path);
    if let Some(e) = errs.into_iter().next() {
        return Err(e);
    }
    Ok(())
}

pub fn collect(
    source: &str,
    controller: Option<&ParsedController>,
    source_path: Option<&Path>,
) -> Vec<Diagnostic> {
    collect_errors(source, controller, source_path)
        .into_iter()
        .map(|e| Diagnostic::from_compile_error(&e, source))
        .collect()
}

fn collect_errors(
    source: &str,
    controller: Option<&ParsedController>,
    source_path: Option<&Path>,
) -> Vec<CompileError> {
    let mut controller_names = BTreeSet::new();
    if let Some(c) = controller {
        for m in &c.returned {
            controller_names.insert(m.name.clone());
        }
    }
    let scope_consts = partials::scope_const_names(source);
    partials::check_partials(source, &controller_names, &scope_consts, source_path)
}
