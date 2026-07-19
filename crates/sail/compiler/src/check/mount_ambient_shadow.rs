//! `sail-mount-ambient-shadow` — refuse an explicit `import { mount } from
//! '@ship/sail-runtime'` (or `/internal`) inside a controller.
//!
//! `mount` is a hazardous, doubly-loaded name. `@ship/sail-runtime` exports
//! the F1 HOST entry point `mount(target, payload, opts)` under the bare
//! name `mount` (`packages/sail/runtime/src/mount.ts`). The controller's
//! AMBIENT lifecycle hook is a completely different function — `(fn: (el) =>
//! cleanup) => void` — exported only under the alias `mountAmbient`
//! (`packages/sail/runtime/src/effect-scope.ts`). The compiler auto-injects
//! `import { mountAmbient as mount } from '@ship/sail-runtime/internal'`
//! into a controller that references the bare identifier `mount`
//! (`runtime_ambient.rs`), but ONLY when the module has not already bound
//! that name itself (`preprocess::inject_ambient_runtime_imports`).
//!
//! Every other ambient primitive (`signal`, `effect`, `controller`,
//! `onCleanup`) is safe to import explicitly from the same barrel — this
//! codebase does it routinely (e.g. `apps/sail-site/routes/showcase/page.sail`
//! imports `effect` explicitly and gets the real one). `mount` is the ONE
//! exception: an author who follows that same, otherwise-correct convention
//! and writes `import { controller, signal, mount } from '@ship/sail-runtime'`
//! silently wins over the ambient injection — the controller's `mount(fn)`
//! call now invokes the HOST's route-mounting entry with `fn` as its `target`
//! argument. It compiles clean and throws at runtime the first time the route
//! mounts: `Cannot read properties of undefined (reading 'params')`, deep
//! inside `mount.ts`'s `seedRouteParams`. Catch the collision here instead,
//! with text that names both `mount`s and says what to do.

use crate::diagnostics::CompileError;

pub const RULE_ID: &str = "sail-mount-ambient-shadow";

const RUNTIME_SPECIFIERS: &[&str] = &["@ship/sail-runtime", "@ship/sail-runtime/internal"];

/// Scan a controller's source (the inline `<controller>` block, or a paired
/// `.controller.ts`) for an import specifier that binds the local name
/// `mount` to the runtime's real, exported `mount` — as opposed to the
/// `mountAmbient as mount` alias the compiler itself injects, which is fine.
pub fn check(source: &str) -> Result<(), CompileError> {
    for line in source.lines() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("import") {
            continue;
        }
        let Some(brace_start) = trimmed.find('{') else { continue };
        let Some(brace_end) = trimmed.find('}') else { continue };
        if brace_end < brace_start {
            continue;
        }
        let after_brace = &trimmed[brace_end + 1..];
        let Some(from_at) = after_brace.find("from") else { continue };
        let from_clause = after_brace[from_at + 4..].trim();
        let is_runtime_module = RUNTIME_SPECIFIERS
            .iter()
            .any(|spec| from_clause.contains(&format!("'{spec}'")) || from_clause.contains(&format!("\"{spec}\"")));
        if !is_runtime_module {
            continue;
        }
        let specifiers = &trimmed[brace_start + 1..brace_end];
        for part in specifiers.split(',') {
            let part = part.trim().trim_start_matches("type ").trim();
            if part.is_empty() {
                continue;
            }
            let exported = part.split(" as ").next().unwrap_or(part).trim();
            let local = part.rsplit(" as ").next().unwrap_or(part).trim();
            // The hazard is entirely about what LOCAL name gets bound: only a
            // local `mount` can shadow the ambient identifier a controller
            // body calls. `mount as mountView` frees `mount` right back up
            // for the ambient injector — that's the escape hatch this rule's
            // own message recommends, not a second violation.
            if local != "mount" {
                continue;
            }
            // `mountAmbient as mount` is exactly what the compiler's own
            // ambient injector would have written — the one binding under
            // the local name `mount` that is NOT a collision.
            if exported == "mountAmbient" {
                continue;
            }
            let import_text = if exported == local {
                "`import { mount }`".to_string()
            } else {
                format!("`import {{ {exported} as mount }}`")
            };
            return Err(CompileError::new(format!(
                "{import_text} from '@ship/sail-runtime' binds the HOST's \
                 `mount(target, payload, opts)` view-mounting entry point, not the \
                 controller's ambient lifecycle hook — the two functions share the \
                 name `mount` but do completely different things. A controller never \
                 imports `mount`: call `mount(fn)` directly in the setup body and the \
                 compiler wires it to the lifecycle hook for you (internally aliased \
                 as `mountAmbient`). Remove this import. If you genuinely need the \
                 host's view-mounting entry point inside a controller, import it under \
                 a different local name, e.g. `import {{ mount as mountView }}`."
            ))
            .with_category(RULE_ID.to_string()));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bare_mount_import_is_rejected() {
        let src = "import { controller, signal, mount } from '@ship/sail-runtime'\n\
                   export default controller((ctx) => { mount(() => {}); return {} })";
        let err = check(src).unwrap_err();
        assert_eq!(err.category.as_deref(), Some(RULE_ID));
    }

    #[test]
    fn aliased_mount_import_is_rejected_too() {
        let src = "import { mount as mount } from \"@ship/sail-runtime/internal\"\n\
                   export default controller((ctx) => ({}))";
        check(src).unwrap_err();
    }

    #[test]
    fn correct_ambient_alias_is_fine() {
        let src = "import { mountAmbient as mount } from '@ship/sail-runtime/internal'\n\
                   export default controller((ctx) => ({}))";
        check(src).unwrap();
    }

    #[test]
    fn no_mount_import_is_fine() {
        let src = "import { controller, signal, effect } from '@ship/sail-runtime'\n\
                   export default controller((ctx) => { mount(() => {}); return {} })";
        check(src).unwrap();
    }

    #[test]
    fn renamed_import_is_fine() {
        let src = "import { mount as mountView } from '@ship/sail-runtime'\n\
                   export default controller((ctx) => ({}))";
        check(src).unwrap();
    }

    #[test]
    fn unrelated_module_import_named_mount_is_fine() {
        let src = "import { mount } from './local-helpers'\n\
                   export default controller((ctx) => ({}))";
        check(src).unwrap();
    }
}
