//! SPIKE S5.3 — product typecheck must default to Pure (SPEC-PLATFORM-NEUTRALITY-1).
//!
//! Law: FileKind Route/Component/Composition/Store → TypeEnv::Pure → lib ESNext only.
//! Default `emit_project_tsconfig_json` is what projects actually run — it must not
//! ship DOM lib (that is the Framework escape hatch).
//!
//! RED on tip while default emit is still TypeEnv::Framework.
//! Implementer must not modify this file.

use sail_compiler::ambients::{
    TypeEnv, emit_project_tsconfig_json, lib_for_type_env, type_env_for_kind,
};
use sail_compiler::project_map::FileKind;

#[test]
fn product_file_kinds_map_to_pure() {
    for kind in [
        FileKind::Route,
        FileKind::Component,
        FileKind::Composition,
        FileKind::Store,
    ] {
        assert_eq!(
            type_env_for_kind(kind),
            TypeEnv::Pure,
            "{kind:?} must be Pure"
        );
    }
}

#[test]
fn pure_lib_excludes_dom() {
    let lib = lib_for_type_env(TypeEnv::Pure);
    assert!(lib.contains(&"ESNext"));
    assert!(
        !lib.iter().any(|e| e.contains("DOM")),
        "Pure lib must not include DOM: {lib:?}"
    );
}

#[test]
fn default_project_tsconfig_emit_is_pure_not_framework() {
    let body = emit_project_tsconfig_json(&["src/**/*.ts", "routes/**/*.ts"]);
    // Product default must match Pure — no DOM in compilerOptions.lib.
    assert!(
        !body.contains("DOM"),
        "default emit_project_tsconfig_json still Framework (DOM present) — \
         product FileKinds cannot enforce unnameable document/window:\n{body}"
    );
    assert!(
        body.contains("ESNext"),
        "default emit must include ESNext:\n{body}"
    );
}
