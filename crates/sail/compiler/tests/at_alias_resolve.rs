//! Gate tests for SPEC-PROJECT-STRUCTURE-1 §3 residual — deck/`@/` resolve via SSOT.
//!
//! Spike S1.3 red gate. Calls product APIs only.
//! Implementer must not modify, skip, or weaken these tests.
//!
//! Expected product surface:
//! - `project_map::resolve_at_alias(project_root, specifier) -> Option<PathBuf>`
//!   maps `@/stores/…` and `@/themes/…` (and the other fixed roots) using
//!   **only** [`project_map::at_alias_paths`] — same table as tsconfig emit.
//! - Pure unit: does not require the target file to exist.
//! - Deck bundler `resolve_id` must call this (or an equivalent thin wrapper
//!   over the same SSOT); tests pin the shared function so paths cannot drift.
//!
//! Optional corpus gate:
//! - sail-site `counter-store` lives under `src/stores/` (defineStore home).

use std::path::{Path, PathBuf};

use sail_compiler::project_map::{
    AtAliasMapping, FileKind, at_alias_paths, classify, resolve_at_alias,
};

fn fixture_app_root() -> PathBuf {
    // Synthetic app root — resolve is pure path math; files need not exist.
    PathBuf::from("/tmp/sail-s1-3-fixture-app")
}

/// Normalize for assertions (drop `./`, unify separators).
fn norm(p: &Path) -> String {
    p.to_string_lossy().replace('\\', "/")
}

/// Whether `resolved` is under `project_root` and ends with `expected_rel`
/// (e.g. `src/stores/tally` or `themes/ship.json`).
fn ends_with_home(project_root: &Path, resolved: &Path, expected_rel: &str) -> bool {
    let root = norm(project_root).trim_end_matches('/').to_string();
    let got = norm(resolved);
    let rel = expected_rel
        .trim_start_matches("./")
        .trim_start_matches('/');
    if !got.starts_with(&root) {
        return false;
    }
    let suffix = got[root.len()..].trim_start_matches('/');
    suffix == rel
        || suffix == format!("{rel}.ts")
        || suffix == format!("{rel}.js")
        || suffix.starts_with(&format!("{rel}/"))
        || suffix.starts_with(&format!("{rel}."))
}

/// Mapping table must stay the resolve SSOT (S1.2 + S1.3).
fn mapping_for(alias_root: &str) -> Option<&'static AtAliasMapping> {
    let specific = format!("@/{alias_root}/*");
    at_alias_paths()
        .iter()
        .find(|m| m.pattern == specific || m.pattern == "@/*" || m.pattern == "@/")
}

#[test]
fn resolve_at_alias_stores_under_src_stores() {
    let root = fixture_app_root();
    let resolved = resolve_at_alias(&root, "@/stores/tally")
        .expect("resolve_at_alias must map @/stores/… via project_map SSOT");
    assert!(
        ends_with_home(&root, &resolved, "src/stores/tally"),
        "@/stores/tally must resolve under src/stores/; got {}",
        norm(&resolved)
    );
    // Agreement with SSOT table target fragment.
    let map = mapping_for("stores").expect("at_alias_paths must cover stores");
    assert!(
        map.target.contains("stores") || map.target.contains("src"),
        "SSOT stores target unexpected: {:?}",
        map.target
    );
}

#[test]
fn resolve_at_alias_themes_under_themes() {
    let root = fixture_app_root();
    let resolved = resolve_at_alias(&root, "@/themes/ship")
        .expect("resolve_at_alias must map @/themes/… via project_map SSOT");
    assert!(
        ends_with_home(&root, &resolved, "themes/ship"),
        "@/themes/ship must resolve under themes/ (not src/); got {}",
        norm(&resolved)
    );
}

#[test]
fn resolve_at_alias_rejects_non_at_specifiers() {
    let root = fixture_app_root();
    assert_eq!(
        resolve_at_alias(&root, "./src/stores/tally"),
        None,
        "relative imports are not @/ aliases"
    );
    assert_eq!(
        resolve_at_alias(&root, "@ship/sail-runtime"),
        None,
        "framework packages are not project @/ aliases"
    );
}

#[test]
fn resolve_at_alias_uses_same_ssot_as_at_alias_paths() {
    // Every fixed root that at_alias_paths covers must resolve a probe import.
    let root = fixture_app_root();
    let probes: &[(&str, &str)] = &[
        ("@/components/widget/widget", "src/components/widget/widget"),
        ("@/compositions/use-foo", "src/compositions/use-foo"),
        ("@/stores/tally", "src/stores/tally"),
        ("@/actions/focus", "src/actions/focus"),
        ("@/themes/ship", "themes/ship"),
    ];
    let map = at_alias_paths();
    assert!(
        !map.is_empty(),
        "at_alias_paths SSOT must be non-empty before resolve agreement"
    );
    for (spec, home_rel) in probes {
        let resolved = resolve_at_alias(&root, spec)
            .unwrap_or_else(|| panic!("resolve_at_alias({spec}) must hit SSOT"));
        assert!(
            ends_with_home(&root, &resolved, home_rel),
            "{spec} → expected …/{home_rel}, got {}",
            norm(&resolved)
        );
    }
}

/// S1.3 corpus cutover: defineStore home is `src/stores/`, not compositions.
#[test]
fn sail_site_counter_store_lives_under_src_stores() {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("repo root");
    let under_stores = repo.join("apps/sail-site/src/stores/counter-store.ts");
    let under_compositions = repo.join("apps/sail-site/src/compositions/counter-store.ts");

    assert!(
        under_stores.is_file(),
        "S1.3 cutover: counter-store must live at apps/sail-site/src/stores/counter-store.ts \
         (got missing file; still under compositions? {})",
        under_compositions.exists()
    );
    assert!(
        !under_compositions.exists(),
        "counter-store must not remain under compositions/ after the store home move"
    );

    let src = std::fs::read_to_string(&under_stores).expect("read counter-store");
    let c = classify(&under_stores, Some(&src));
    assert_eq!(
        c.kind,
        FileKind::Store,
        "counter-store shape is defineStore → Store"
    );
    assert!(
        c.ok,
        "store under src/stores/ must classify ok; home={:?}",
        c.home
    );
}
