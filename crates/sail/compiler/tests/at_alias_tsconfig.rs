//! Gate tests for SPEC-PROJECT-STRUCTURE-1 §3 — `@/` alias in project tsconfig.
//!
//! Spike S1.2 red gate. Calls product APIs only.
//! Implementer must not modify, skip, or weaken these tests.
//!
//! Expected product surface:
//! - `project_map::at_alias_paths` → SSOT `AtAliasMapping` table
//! - `ambients::emit_project_tsconfig_json` embeds `paths` from that table
//!   (targets rewritten relative to `.sail/`; no `baseUrl` —
//!   tsgo removes it with TS5102)
//! - Deck resolve (out of this gate file) must use the same SSOT

use std::path::Path;

use sail_compiler::ambients::emit_project_tsconfig_json;
use sail_compiler::project_map::{AtAliasMapping, FileKind, at_alias_paths, classify};

/// SPEC §3 fixed import roots (`@/<root>/…`).
const FIXED_ALIAS_ROOTS: &[(&str, &str)] = &[
    // (alias root after `@/`, filesystem home fragment)
    ("components", "src/components"),
    ("compositions", "src/compositions"),
    ("stores", "src/stores"),
    ("actions", "src/actions"),
    ("themes", "themes"),
];

/// Whether one mapping would let TS resolve `from '@/<alias_root>/…'` to `fs_home`.
fn mapping_covers(m: &AtAliasMapping, alias_root: &str, fs_home: &str) -> bool {
    let home = fs_home.trim_end_matches('/');
    let target = m.target.replace('\\', "/");
    let target_has_home = target.contains(home)
        || (alias_root != "themes" && target.trim_end_matches('/').ends_with("src"))
        || target.trim_end_matches('/').ends_with('.')
        || target == "./*"
        || target == "*";

    let pat = m.pattern.replace('\\', "/");
    let specific = format!("@/{alias_root}/*");
    let specific_no_star = format!("@/{alias_root}/");
    let specific_bare = format!("@/{alias_root}");

    let pattern_matches = pat == specific
        || pat == specific_no_star
        || pat == specific_bare
        || pat == "@/*"
        || pat == "@/";

    pattern_matches && target_has_home
}

#[test]
fn at_alias_paths_ssot_is_nonempty() {
    let map = at_alias_paths();
    assert!(
        !map.is_empty(),
        "project_map::at_alias_paths must return the SSOT alias table (SPEC §3)"
    );
}

#[test]
fn at_alias_paths_covers_fixed_homes() {
    let map = at_alias_paths();
    for (root, fs_home) in FIXED_ALIAS_ROOTS {
        let ok = map.iter().any(|m| mapping_covers(m, root, fs_home));
        assert!(
            ok,
            "at_alias_paths must cover @/{root}/… → {fs_home}; got {map:?}"
        );
    }
}

#[test]
fn at_alias_paths_patterns_are_at_slash() {
    let map = at_alias_paths();
    assert!(!map.is_empty(), "SSOT must not be empty");
    for m in map {
        assert!(
            m.pattern.starts_with("@/"),
            "alias pattern must start with `@/`, got {:?}",
            m.pattern
        );
        assert!(
            !m.target.is_empty(),
            "alias target must not be empty for {:?}",
            m.pattern
        );
    }
}

#[test]
fn emit_project_tsconfig_includes_paths_without_base_url() {
    let body = emit_project_tsconfig_json(&["src/**/*.ts", "routes/**/*.ts"]);
    // tsgo removes baseUrl (TS5102); paths alone must resolve `@/` from .sail/.
    assert!(
        !body.contains("\"baseUrl\""),
        "project tsconfig must not emit baseUrl (tsgo TS5102):\n{body}"
    );
    assert!(
        body.contains("\"paths\""),
        "project tsconfig must set compilerOptions.paths:\n{body}"
    );
    assert!(
        body.contains("\"@/*\"") || body.contains("@/"),
        "project tsconfig paths must include `@/*` (or fixed `@/…` keys):\n{body}"
    );
    assert!(
        body.contains("../src/stores/*") || body.contains("\"../src/stores/*\""),
        "paths targets must be relative to .sail/:\n{body}"
    );
}

#[test]
fn emit_project_tsconfig_paths_cover_fixed_homes() {
    let body = emit_project_tsconfig_json(&["src/**/*.ts", "routes/**/*.ts"]);
    // Prefer explicit `@/<root>` keys; accept a single `@/*` only when targets
    // for each home also appear in the emitted JSON (via SSOT agreement test).
    for (root, fs_home) in FIXED_ALIAS_ROOTS {
        let explicit =
            body.contains(&format!("@/{root}")) || body.contains(&format!("\"@/{root}/*\""));
        let catch_all = body.contains("\"@/*\"");
        let home_in_body = body.contains(fs_home) || body.contains(&format!("./{fs_home}"));
        assert!(
            explicit || (catch_all && home_in_body) || (catch_all && root != &"themes"),
            "tsconfig paths must resolve @/{root}/… (home {fs_home}):\n{body}"
        );
    }
    // themes is not under src/ — catch-all `@/*` → `./src/*` alone is insufficient.
    let themes_ok = body.contains("@/themes")
        || body.contains("\"@/themes/*\"")
        || (body.contains("\"@/*\"")
            && (body.contains("\"./themes")
                || body.contains("\"themes/")
                || body.contains("\"./*\"")
                || body.contains("\"*\"")
                || body.contains("\"./\"")));
    assert!(
        themes_ok,
        "tsconfig must map @/themes/… to themes/ (not only src/*):\n{body}"
    );
}

#[test]
fn emit_tsconfig_paths_agree_with_at_alias_paths_ssot() {
    let map = at_alias_paths();
    assert!(
        !map.is_empty(),
        "SSOT must not be empty before emit agreement check"
    );
    let body = emit_project_tsconfig_json(&["src/**/*.ts", "routes/**/*.ts"]);
    for m in map {
        assert!(
            body.contains(m.pattern),
            "emit_project_tsconfig_json must include SSOT pattern {:?}:\n{body}",
            m.pattern
        );
        // Target may be written with or without leading `./`.
        let stripped = m.target.trim_start_matches("./");
        assert!(
            body.contains(m.target) || body.contains(stripped),
            "emit_project_tsconfig_json must include SSOT target {:?} (or {stripped}):\n{body}",
            m.target
        );
    }
}

#[test]
fn classify_homes_align_with_alias_targets() {
    // Smoke: fixed homes from classify stay consistent with alias map targets.
    let cases: &[(&str, Option<&str>, FileKind, &str)] = &[
        (
            "app/src/components/widget/widget.sail",
            None,
            FileKind::Component,
            "components",
        ),
        (
            "app/src/compositions/use-foo.ts",
            Some("export function useFoo() { return 1 }\n"),
            FileKind::Composition,
            "compositions",
        ),
        (
            "app/src/stores/tally.ts",
            Some("export const tally = defineStore('tally', () => ({}))\n"),
            FileKind::Store,
            "stores",
        ),
        (
            "app/src/actions/focus.ts",
            Some("export function focus(node) { return {} }\n"),
            FileKind::Action,
            "actions",
        ),
        ("app/themes/acme.json", None, FileKind::Theme, "themes"),
    ];

    let map = at_alias_paths();
    assert!(!map.is_empty(), "SSOT must not be empty");

    for (path, src, kind, alias_root) in cases {
        let c = classify(Path::new(path), *src);
        assert_eq!(c.kind, *kind, "classify({path}) kind");
        let home = c.home.trim_end_matches('/');
        let covered = map.iter().any(|m| {
            mapping_covers(m, alias_root, home)
                || mapping_covers(m, alias_root, &c.home)
                || m.target.contains(home)
        });
        assert!(
            covered,
            "alias SSOT must target classify home {:?} for {path} (@/{alias_root}); map={map:?}",
            c.home
        );
    }
}
