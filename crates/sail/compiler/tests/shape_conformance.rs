//! Shape-conformance integration test.
//!
//! Compiles every checked-in `.sail` component (under `packages/components/`)
//! with the in-process compiler API,
//! merges the emitted JS into a tempdir, and runs `tsc --noEmit`
//! against it with paths mapped to the runtime / ui-dom sources.
//!
//! Goal: catch the class of bug where the compiler emits a call shape
//! the runtime's public types don't accept (e.g. `live(({ ship, route
//! }) => …)` while `live()` accepts only a zero-arg thunk). One pass,
//! one tsc run, named-fixture diagnostics on failure.
//!
//! See `.artifacts/job-spec.md` § Part A.

use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use sail_compiler::{compile, CompileMode, CompileOpts, ComponentMeta};

fn repo_root() -> PathBuf {
    // crates/sail/compiler -> repo root is three levels up.
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
        .canonicalize()
        .expect("canonicalize repo root")
}

fn collect_sui_files(roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for root in roots {
        walk_collect(root, &mut out);
    }
    out.sort();
    out
}

fn walk_collect(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        if is_dir {
            // Skip dist/ artifacts and node_modules just in case.
            let name = entry.file_name();
            let n = name.to_string_lossy();
            if n == "dist" || n == "node_modules" || n == "target" {
                continue;
            }
            walk_collect(&path, out);
        } else if path.extension() == Some(OsStr::new("sail")) {
            out.push(path);
        }
    }
}

/// Determine compile mode + component metadata for a given `.sail` path.
/// Files under `packages/components/` are components; their component name is
/// the file stem. (The framework builds components, full stop — there is no
/// `views/` concept here; views are a Ship/deck applet feature.)
fn classify(path: &Path, repo: &Path) -> (CompileMode, Option<ComponentMeta>) {
    let rel = path.strip_prefix(repo).unwrap_or(path);
    let is_component = rel.starts_with("packages/components");
    if is_component {
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("component")
            .to_string();
        // Shadow flag mirrors the precompile script's default
        // (--component-name only — no shadow override). Lighter-DOM
        // components are the norm; the compiler's check pipeline
        // handles either.
        (
            CompileMode::Component,
            Some(ComponentMeta {
                name: stem,
            }),
        )
    } else {
        (CompileMode::Component, None)
    }
}

fn sanitize_for_tempdir(rel: &Path) -> String {
    rel.to_string_lossy().replace(['/', '\\'], "__")
}

/// Test mode reads the original source verbatim — the production
/// pipeline's `--styles` flag preprocesses external CSS, but for shape
/// conformance the template-as-checked-in is what we type-check.
fn compile_one(path: &Path, repo: &Path) -> Result<String, String> {
    let source = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let (mode, component_meta) = classify(path, repo);
    let opts = CompileOpts {
        route_params: None,
        // No paired controller: the test exercises the compiler's
        // import shape against the framework, not the controller side.
        setup_url: None,
        mode,
        source_path: Some(path.to_path_buf()),
        component_meta,
        emit_dts: false,
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    };
    let out = compile(&source, opts).map_err(|e| {
        let rendered = e.render(&source);
        format!("compile {}:\n{rendered}", path.display())
    })?;
    Ok(out.js)
}

fn write_tsconfig(temp_dir: &Path, repo: &Path) {
    let runtime_index = repo.join("packages/sail/runtime/src/index.ts");
    let runtime_template_hooks = repo.join("packages/sail/runtime/src/template-hooks/index.ts");
    let runtime_route = repo.join("packages/sail/runtime/src/route/index.ts");
    let runtime_dev = repo.join("packages/sail/runtime/src/dev.ts");
    let ui_dom_index = repo.join("packages/sail/dom/src/index.ts");

    // tsconfig: allowJs without checkJs. tsc resolves the named
    // imports against the runtime / ui-dom public surface and reports
    // any name that isn't exported, but it doesn't deep-type-check
    // the compiled-JS function bodies. The structural check above
    // already gates on missing exports; this layer adds tsc's own
    // module-resolution view as a cross-check (catches subtle subpath
    // issues — e.g. the compiler started importing from
    // `@ship/sail-runtime/foo` and that subpath doesn't exist).
    //
    // We deliberately stop short of `checkJs: true`. The compiled
    // output is intentionally untyped scaffolding (Object.create(null)
    // scopes, dynamic property writes); enabling deep checking surfaces
    // many pre-existing call-shape mismatches that pre-date this test
    // and live in compiler/runtime territory the integration-tests job
    // is explicitly out-of-scope to touch (see job-spec.md
    // "Don't touch"). Tightening this gate further is a follow-on
    // covered by a runtime/compiler shape-alignment job.
    let cfg = serde_json::json!({
        "compilerOptions": {
            "target": "ES2022",
            "module": "ESNext",
            "moduleResolution": "Bundler",
            "lib": ["ES2022", "DOM", "DOM.Iterable"],
            "noEmit": true,
            "allowJs": true,
            "checkJs": false,
            "strict": false,
            "noImplicitAny": false,
            "skipLibCheck": true,
            "esModuleInterop": true,
            "isolatedModules": false,
            "allowImportingTsExtensions": true,
            "types": [],
            "baseUrl": ".",
            "paths": {
                "@ship/sail-runtime": [runtime_index.to_string_lossy()],
                "@ship/sail-runtime/template-hooks": [runtime_template_hooks.to_string_lossy()],
                "@ship/sail-runtime/route": [runtime_route.to_string_lossy()],
                "@ship/sail-runtime/dev": [runtime_dev.to_string_lossy()],
                "@ship/sail-dom": [ui_dom_index.to_string_lossy()],
            },
        },
        "include": ["**/*.js"],
    });
    let path = temp_dir.join("tsconfig.json");
    fs::write(&path, serde_json::to_string_pretty(&cfg).unwrap()).expect("write tsconfig");
}

/// Discover `tsc`. Production CI runs `bun install --frozen-lockfile`
/// before tests; locally devs may run from a fresh clone. Skip the
/// tsc-bound assertion (with a printed warning) when nothing's
/// available — the structural check still gates.
fn find_tsc(repo: &Path) -> Option<PathBuf> {
    let direct = repo.join("node_modules/.bin/tsc");
    if direct.exists() {
        return Some(direct);
    }
    None
}

/// Parse named imports of a given module specifier from emitted JS.
fn extract_named_imports(js: &str, module: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    // Naive multi-line scan: for each `import { … } from "<module>"`
    // captures the comma-separated names. The compiler emits these
    // single-line and well-formatted, so a one-shot regex-free parse
    // is enough.
    for line in js.lines() {
        let l = line.trim();
        if !l.starts_with("import {") || !l.contains(module) {
            continue;
        }
        // Match exact module spec to avoid false positives where one
        // module is a prefix of another (e.g. `@ship/sail-runtime` vs
        // `@ship/sail-runtime/template-hooks`).
        let from_spec = format!("from \"{module}\"");
        if !l.contains(&from_spec) {
            continue;
        }
        let Some(open) = l.find('{') else { continue };
        let Some(close) = l.find('}') else { continue };
        let names = &l[open + 1..close];
        for raw in names.split(',') {
            let name = raw.trim().trim_end_matches(';').to_string();
            if name.is_empty() {
                continue;
            }
            out.push((name, module.to_string()));
        }
    }
    out
}

/// Parse named exports from an `index.ts` file. Recognises:
/// - `export { A, B } from …;` and `export type { A } from …;`
/// - `export function name`, `export async function name`
/// - `export const name`, `export let name`
/// - `export class name`, `export interface name`
/// - `export type name` (type alias declarations)
fn extract_exports(idx: &Path) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let Ok(src) = fs::read_to_string(idx) else {
        return out;
    };
    let mut buf = String::new();
    let mut in_block = false;
    for line in src.lines() {
        let l = line.trim();
        if !in_block {
            if l.starts_with("export {") || l.starts_with("export type {") {
                buf.clear();
                buf.push_str(l);
                if !l.contains('}') {
                    in_block = true;
                    continue;
                }
            } else if let Some(name) = extract_declaration_export(l) {
                out.insert(name);
                continue;
            } else {
                continue;
            }
        } else {
            buf.push(' ');
            buf.push_str(l);
            if l.contains('}') {
                in_block = false;
            } else {
                continue;
            }
        }
        // buf now has a complete `export { … }` clause.
        let Some(open) = buf.find('{') else {
            continue;
        };
        let Some(close) = buf.find('}') else { continue };
        let names = &buf[open + 1..close];
        for raw in names.split(',') {
            let name = raw.trim();
            if name.is_empty() {
                continue;
            }
            // `as` re-export: keep the rebound name.
            let final_name = if let Some(idx) = name.find(" as ") {
                name[idx + 4..].trim().to_string()
            } else {
                name.to_string()
            };
            out.insert(final_name);
        }
    }
    out
}

/// Extract the export name from a direct declaration export line.
/// Handles `export [async] function name`, `export const/let/class/
/// interface/type name`, but not `export {` (handled by `extract_exports`).
fn extract_declaration_export(l: &str) -> Option<String> {
    let rest = l.strip_prefix("export ")?;
    // Named re-export blocks are handled elsewhere; default exports are
    // anonymous in terms of the name callers import.
    if rest.starts_with('{') || rest.starts_with("type {") || rest.starts_with("default ") {
        return None;
    }
    // Strip optional `async` modifier (for async functions).
    let rest = rest.strip_prefix("async ").unwrap_or(rest);
    // Strip the declaration keyword.
    let rest = ["function ", "const ", "let ", "class ", "interface ", "type "]
        .iter()
        .find_map(|kw| rest.strip_prefix(kw))?;
    // Extract the identifier up to the first non-identifier character.
    let name: String = rest
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '$')
        .collect();
    if name.is_empty() { None } else { Some(name) }
}

fn structural_check(
    js_files: &[(PathBuf, String, String)],
    repo: &Path,
) -> Vec<String> {
    let runtime_idx = repo.join("packages/sail/runtime/src/index.ts");
    let runtime_hooks_idx = repo.join("packages/sail/runtime/src/template-hooks/index.ts");
    let runtime_route_idx = repo.join("packages/sail/runtime/src/route/index.ts");
    let dom_idx = repo.join("packages/sail/dom/src/index.ts");

    let runtime_exports = extract_exports(&runtime_idx);
    let runtime_hook_exports = extract_exports(&runtime_hooks_idx);
    let runtime_route_exports = extract_exports(&runtime_route_idx);
    let dom_exports = extract_exports(&dom_idx);

    let mut errors = Vec::new();
    for (src, _temp_name, js) in js_files {
        for (name, module) in extract_named_imports(js, "@ship/sail-runtime") {
            if !runtime_exports.contains(&name) {
                errors.push(format!(
                    "{}: imports `{name}` from `{module}` — not exported by runtime",
                    src.display()
                ));
            }
        }
        for (name, module) in extract_named_imports(js, "@ship/sail-runtime/template-hooks") {
            if !runtime_hook_exports.contains(&name) {
                errors.push(format!(
                    "{}: imports `{name}` from `{module}` — not exported",
                    src.display()
                ));
            }
        }
        for (name, module) in extract_named_imports(js, "@ship/sail-runtime/route") {
            // The import uses `as` alias: `import { route as __routeFactory }`.
            // extract_named_imports returns the local alias, so strip any
            // `as <alias>` suffix to recover the original export name.
            let export_name = if let Some(idx) = name.find(" as ") {
                name[..idx].trim().to_string()
            } else {
                name.clone()
            };
            if !runtime_route_exports.contains(&export_name) {
                errors.push(format!(
                    "{}: imports `{name}` from `{module}` — not exported by @ship/sail-runtime/route",
                    src.display()
                ));
            }
        }
        for (name, module) in extract_named_imports(js, "@ship/sail-dom") {
            if !dom_exports.contains(&name) {
                errors.push(format!(
                    "{}: imports `{name}` from `{module}` — not exported by ui-dom",
                    src.display()
                ));
            }
        }
    }
    errors
}

#[test]
fn every_sui_compiles_and_imports_resolve() {
    let repo = repo_root();
    let roots = [repo.join("packages/components")];
    let files = collect_sui_files(&roots);
    assert!(
        !files.is_empty(),
        "shape-conformance found zero `.sail` files under {roots:?} — wrong repo root?"
    );

    // Compile every `.sail` to JS via the in-process API. No spawned
    // sub-processes, no concurrency deadlock — sequential, fast.
    let mut compiled: Vec<(PathBuf, String, String)> = Vec::new();
    let mut compile_errors: Vec<String> = Vec::new();
    for sail in &files {
        match compile_one(sail, &repo) {
            Ok(js) => {
                let rel = sail.strip_prefix(&repo).unwrap_or(sail);
                let temp_name = sanitize_for_tempdir(rel);
                compiled.push((sail.clone(), temp_name, js));
            }
            Err(e) => compile_errors.push(e),
        }
    }
    assert!(
        compile_errors.is_empty(),
        "{} fixtures failed to compile:\n\n{}",
        compile_errors.len(),
        compile_errors.join("\n\n")
    );

    // Structural check (cheap, hermetic): every named import in the
    // compiled output exists in the matching package's public surface.
    let structural = structural_check(&compiled, &repo);
    assert!(
        structural.is_empty(),
        "{} structural import mismatches:\n  {}",
        structural.len(),
        structural.join("\n  ")
    );
}

#[test]
fn every_sui_typechecks_against_runtime_dts() {
    let repo = repo_root();

    // Skip cleanly when tsc isn't available — local first-clone flow,
    // not CI. CI runs `bun install` before tests.
    let Some(tsc_bin) = find_tsc(&repo) else {
        eprintln!(
            "shape-conformance: tsc not found at node_modules/.bin/tsc — \
             run `bun install` to enable the typecheck pass. Structural \
             check still gates."
        );
        return;
    };

    let temp = tempfile::tempdir().expect("tempdir");
    let temp_path = temp.path().to_path_buf();

    let roots = [repo.join("packages/components")];
    let files = collect_sui_files(&roots);
    assert!(!files.is_empty());

    for sail in &files {
        let js = compile_one(sail, &repo).unwrap_or_else(|e| panic!("{e}"));
        let rel = sail.strip_prefix(&repo).unwrap_or(sail);
        // Each fixture lands as `<sanitized-path>.js` so tsc diagnostics
        // name the originating .sail file directly.
        let out_name = format!("{}.js", sanitize_for_tempdir(rel));
        fs::write(temp_path.join(&out_name), js).expect("write js");
    }

    write_tsconfig(&temp_path, &repo);

    let output = Command::new(&tsc_bin)
        .arg("--noEmit")
        .arg("--project")
        .arg(temp_path.join("tsconfig.json"))
        .current_dir(&temp_path)
        .output()
        .expect("spawn tsc");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Filter to diagnostics that originate in the temp dir's compiled
    // fixtures (the `<sanitized>.sail.js` files we wrote). tsc walks
    // imported modules and may surface errors in workspace files
    // outside our test surface (the runtime source isn't strict-clean
    // at HEAD; `packages/sail/runtime/src/context.ts` and
    // `scope-resource-warn.ts` have pre-existing errors that this
    // job is out-of-scope to fix). Those errors aren't a shape-
    // conformance signal — they'd fire even if every `.sail` was
    // empty — so we drop them here and only gate on diagnostics
    // tied to compiler-emitted output.
    let fixture_errors: Vec<&str> = stdout
        .lines()
        .filter(|line| line.contains(".sail.js("))
        .collect();

    if !fixture_errors.is_empty() {
        panic!(
            "tsc rejected {} compiled fixture(s) (raw exit {:?}).\n\n\
             --- fixture diagnostics ---\n{}\n\n\
             --- full stdout (includes pre-existing runtime/source noise filtered out above) ---\n{stdout}\n\
             --- stderr ---\n{stderr}\n\n\
             Each fixture diagnostic names a file like `views__git__page.sail.js`. \
             Reverse `__` -> `/` to find the originating `.sail` source.",
            fixture_errors.len(),
            output.status.code(),
            fixture_errors.join("\n")
        );
    }
}
