//! Gate tests for SPEC-AMBIENT-SURFACE-1 S5.2 — ambient names required so
//! the app corpus can delete `@ship/*` value imports.
//!
//! S5.1 locked the core table. S5.2 adds corpus-facing ambient inject names
//! that real apps already call without a package import:
//! - `navigate` (deck/router surface, SPEC §1)
//! - theme writers used by product chrome: `applyTheme`, `persistTheme`,
//!   `readStoredTheme` (theme API → runtime ambient, SPEC §3)
//!
//! Implementer must not modify, skip, or weaken these tests.
//! Product: extend RUNTIME_AMBIENT + re-export from runtime/internal inject.

use sail_compiler::runtime_ambient::{RUNTIME_AMBIENT, RUNTIME_SPECIFIER};

fn ambient_names() -> Vec<&'static str> {
    RUNTIME_AMBIENT.iter().map(|(a, _, _)| *a).collect()
}

#[test]
fn inject_specifier_is_runtime_internal() {
    assert_eq!(RUNTIME_SPECIFIER, "@ship/sail-runtime/internal");
}

#[test]
fn navigate_is_ambient() {
    assert!(
        ambient_names().contains(&"navigate"),
        "navigate must be RUNTIME_AMBIENT so app controllers drop @ship/deck/router imports"
    );
}

#[test]
fn theme_session_writers_are_ambient() {
    let names = ambient_names();
    for required in ["applyTheme", "persistTheme", "readStoredTheme"] {
        assert!(
            names.contains(&required),
            "{required} must be RUNTIME_AMBIENT (theme API → runtime; delete @ship/tokens/themes imports)"
        );
    }
}

#[test]
fn s5_1_ambient_table_still_present() {
    let names = ambient_names();
    for required in [
        "signal",
        "derived",
        "effect",
        "controller",
        "interval",
        "debounce",
        "theme",
        "defineStore",
        "provideStore",
        "useStore",
        "query",
        "mutation",
        "mount",
        "onCleanup",
    ] {
        assert!(
            names.contains(&required),
            "S5.1 ambient name missing: {required}"
        );
    }
}
