//! Emit scan + purity call for CSSOM in used `use:` actions.
//!
//! OnSave / compile / `sail check` only. LSP OnType skips this pass.
//! Fail → [`sail-no-imperative-style`]. No author `<test>`. No SES.

use std::path::{Path, PathBuf};

use crate::diagnostics::{CompileError, Diagnostic};
use crate::ir::{AttrInstr, Instr, Program};

const STYLE_FLOOR: &str = "sail-no-imperative-style";

/// Build-gate: first CSSOM finding in a used action is a compile error.
pub fn check(program: &Program, source_path: Option<&Path>) -> Result<(), CompileError> {
    let Some(path) = source_path else {
        return Ok(());
    };
    for (action_path, source) in used_action_sources(program, path) {
        let findings = sail_lint::lint_action_source(&action_path, &source);
        if let Some(f) = findings.iter().find(|f| f.rule == STYLE_FLOOR) {
            return Err(CompileError::new(f.message.clone())
                .with_category(STYLE_FLOOR)
                .with_path(action_path));
        }
    }
    Ok(())
}

/// Check-stream: every CSSOM finding on used actions.
pub fn collect(program: &Program, source_path: Option<&Path>) -> Vec<Diagnostic> {
    let Some(path) = source_path else {
        return Vec::new();
    };
    let mut diags = Vec::new();
    for (action_path, source) in used_action_sources(program, path) {
        for f in sail_lint::lint_action_source(&action_path, &source) {
            if f.rule != STYLE_FLOOR {
                continue;
            }
            diags.push(Diagnostic {
                line: f.line,
                col: f.col,
                end_line: f.line,
                end_col: f.col,
                severity: crate::diagnostics::DiagSeverity::Error,
                message: f.message,
                rule: Some(STYLE_FLOOR.to_string()),
            });
        }
    }
    diags
}

fn used_action_sources(program: &Program, sail_path: &Path) -> Vec<(PathBuf, String)> {
    let mut names = Vec::new();
    collect_use_names(&program.template, &mut names);
    names.sort();
    names.dedup();
    let mut out = Vec::new();
    for name in names {
        if let Some(action_path) = resolve_action_file(sail_path, &name)
            && let Ok(source) = std::fs::read_to_string(&action_path)
        {
            out.push((action_path, source));
        }
    }
    out
}

fn collect_use_names(instrs: &[Instr], out: &mut Vec<String>) {
    for instr in instrs {
        match instr {
            Instr::Element(el) => {
                for attr in &el.attrs {
                    if let AttrInstr::Use { action, .. } = attr {
                        out.push(action.clone());
                    }
                }
                collect_use_names(&el.children, out);
            }
            Instr::If(i) => {
                for b in &i.branches {
                    collect_use_names(&b.body, out);
                }
                if let Some(eb) = &i.else_body {
                    collect_use_names(eb, out);
                }
            }
            Instr::For(f) => {
                collect_use_names(&f.body, out);
                if let Some(eb) = &f.else_body {
                    collect_use_names(eb, out);
                }
            }
            Instr::Match(m) => {
                for c in &m.cases {
                    collect_use_names(&c.body, out);
                }
                if let Some(d) = &m.default {
                    collect_use_names(d, out);
                }
            }
            Instr::When(w) => {
                collect_use_names(&w.body, out);
                if let Some(eb) = &w.else_body {
                    collect_use_names(eb, out);
                }
            }
            Instr::StaticText { .. } | Instr::DynamicText { .. } | Instr::Comment { .. } => {}
        }
    }
}

fn resolve_action_file(sail_path: &Path, name: &str) -> Option<PathBuf> {
    let file = format!("{name}.ts");
    let parent = sail_path.parent()?;
    let candidates = [
        parent.join("actions").join(&file),
        parent.join("src").join("actions").join(&file),
        parent
            .parent()
            .map(|p| p.join("src").join("actions").join(&file))
            .unwrap_or_default(),
        parent
            .parent()
            .map(|p| p.join("actions").join(&file))
            .unwrap_or_default(),
    ];
    candidates
        .into_iter()
        .find(|p| !p.as_os_str().is_empty() && p.is_file())
}
