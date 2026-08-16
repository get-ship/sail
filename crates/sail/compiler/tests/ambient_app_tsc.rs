//! End-to-end tsc proof on a sail-app-shaped consumer fixture.
//!
//! Proves the acceptance path ambient smoke alone does not cover:
//!   1. emit ambient pack into `.sail/types.d.ts`
//!   2. project tsconfig extends `@ship/sail-tsconfig` base + includes the pack
//!   3. `tsc --noEmit` EXIT 0 with **zero** hand ambient shims in app source
//!   4. authored controller uses `signal` / `controller` / `style`
//!
//! Fixture lives at `tests/fixtures/ambient-app/` (committed project shape;
//! pack is generated, never committed — under `.sail/`).
//!
//! Requires `node_modules/typescript/bin/tsc` (bun install). Missing wrapper panics.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use sail_compiler::ambients;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap()
}

fn require_tsc(repo: &Path) -> PathBuf {
    let p = repo.join("node_modules/typescript/bin/tsc");
    assert!(
        p.is_file(),
        "repo TypeScript wrapper missing at {} — bun install",
        p.display()
    );
    p
}

fn fixture_dir(repo: &Path) -> PathBuf {
    repo.join("tests/fixtures/ambient-app")
}

/// App sources must not smuggle in hand-written ambient declarations.
fn assert_no_hand_ambient_shims(project: &Path) {
    let mut stack = vec![project.join("routes"), project.join("src")];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            // Only authored .ts under routes/src — not generated packs.
            if !name.ends_with(".ts") {
                continue;
            }
            let text = fs::read_to_string(&path).unwrap_or_default();
            assert!(
                !text.contains("declare global")
                    && !text.contains("declare const signal")
                    && !text.contains("declare const controller")
                    && !text.contains("declare const style")
                    && !text.contains("declare function signal")
                    && !text.contains("declare function controller"),
                "hand ambient shim forbidden in {}: remove declare* for sail globals",
                path.display()
            );
            assert!(
                !name.ends_with(".d.ts"),
                "hand .d.ts under app source is forbidden (got {})",
                path.display()
            );
        }
    }
}

#[test]
fn ambient_app_fixture_tsc_no_emit_green() {
    let repo = repo_root();
    let tsc = require_tsc(&repo);

    let project = fixture_dir(&repo);
    assert!(
        project.join("sail.config.jsonc").is_file(),
        "fixture missing sail.config.jsonc at {}",
        project.display()
    );
    assert!(
        project.join("tsconfig.json").is_file(),
        "fixture missing tsconfig.json at {}",
        project.display()
    );
    assert!(
        project.join("routes/home/controller.ts").is_file(),
        "fixture missing routes/home/controller.ts"
    );

    // Project must extend the shared sail base (not a one-off options dump).
    let tsconfig = fs::read_to_string(project.join("tsconfig.json")).unwrap();
    assert!(
        tsconfig.contains("packages/sail/tsconfig/tsconfig.base.json")
            || tsconfig.contains("@ship/sail-tsconfig"),
        "fixture tsconfig must extend sail base defaults:\n{tsconfig}"
    );
    assert!(
        tsconfig.contains(".sail/types.d.ts") || tsconfig.contains(ambients::PROJECT_TYPES_REL),
        "fixture tsconfig must include generated ambient pack:\n{tsconfig}"
    );

    // TOOLING-1 WI2: base carries the strict flag set (fixture extends it).
    let base = fs::read_to_string(repo.join("packages/sail/tsconfig/tsconfig.base.json")).unwrap();
    for flag in ambients::SAIL_TSCONFIG_REQUIRED_FLAGS {
        assert!(
            base.contains(flag),
            "tsconfig.base missing WI2 flag {flag}:\n{base}"
        );
    }
    // Oxlint defaults present (recommended + typescript for TS regions).
    let oxlint = project.join(".oxlintrc.json");
    assert!(
        oxlint.is_file(),
        "fixture missing .oxlintrc.json at {}",
        oxlint.display()
    );
    let oxlint_body = fs::read_to_string(&oxlint).unwrap();
    assert!(
        oxlint_body.contains("typescript") && oxlint_body.contains("correctness"),
        "fixture oxlint must enable typescript + recommended correctness:\n{oxlint_body}"
    );

    assert_no_hand_ambient_shims(&project);

    // Emit path consumers use: `sail types` / `write_project_types` →
    // `.sail/types.d.ts` (project generate).
    let pack = ambients::write_project_types(&project).expect("write ambient pack");
    assert!(
        pack.ends_with(ambients::PROJECT_TYPES_REL.replace('/', std::path::MAIN_SEPARATOR_STR))
            || pack.ends_with(".sail/types.d.ts")
            || pack.ends_with(".sail\\types.d.ts"),
        "pack path should be project .sail/types.d.ts, got {}",
        pack.display()
    );
    let pack_text = fs::read_to_string(&pack).expect("read pack");
    assert!(
        pack_text.contains("const style:") || pack_text.contains("const style "),
        "pack must declare style ambient"
    );
    assert!(
        pack_text.contains("function signal") || pack_text.contains("const signal"),
        "pack must declare signal ambient"
    );
    assert!(
        pack_text.contains("function controller") || pack_text.contains("const controller"),
        "pack must declare controller ambient"
    );

    let out = Command::new(&tsc)
        .args(["-p", "tsconfig.json", "--noEmit"])
        .current_dir(&project)
        .output()
        .expect("run tsc on ambient-app fixture");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        out.status.success(),
        "sail-app fixture must typecheck with generated pack + sail base tsconfig \
         (zero hand shims). tsc exit {:?}:\n{combined}",
        out.status.code()
    );
}
