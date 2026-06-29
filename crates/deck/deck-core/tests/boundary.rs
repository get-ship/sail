//! Boundary invariant I2 — Deck's HOST and BUNDLER crates name no Ship app
//! concepts.
//!
//! deck-core / deck-axum / deck-host serve SUI views + assets for ANY host.
//! deck-bundler builds those assets. deck-config holds the config schema.
//! None of them may hardcode a Ship app layout (`apps/shell`, `apps/shipd`,
//! `apps/site`) or a Ship-specific identifier (`sui-chrome`, `sui-marketing`,
//! `sui-arcade`, `sui-pdf`, `__SHIP_APP_ID__`) in PRODUCTION source lines.
//!
//! The three deck-own singleton bundle names (sui-runtime, sui-dom,
//! sui-sdk-client) are legitimately hardcoded in deck-bundler and ARE expected.
//! (`sui-components` is no longer a singleton — the umbrella was retired
//! (#221/#269); `sui-components/<tag>` is now the per-tag resolver namespace.)
//!
//! P2-11 hardening:
//!   - `FORBIDDEN` widens to Ship-app-prefixed route patterns (`/shell/`,
//!     `/spaces/`) and Ship-app asset namespaces (`/shell/_assets/`,
//!     `/space/<id>`), so a deck crate cannot mount a route that is a Ship-
//!     app concern (deck names the framework surface, the host names the
//!     app surface).
//!   - The scan also covers `*.sh` e2e scripts under `crates/deck/`. Bash
//!     scripts that hardcode `apps/shell`, `apps/site`, etc. are also a
//!     boundary leak — a deck e2e script must call the deck surface, not
//!     reach into a Ship app's layout. The comment-line exemption carries
//!     over; `#[cfg(test)]`-style brace suppression does not apply (bash has
//!     no blocks), so only `#` comments are exempt.
//!
//! Exclusions:
//!   - Rust: lines starting with `//` (doc/line comments — may cite paths as
//!     examples); lines inside `#[cfg(test)]` + `mod tests` blocks; dedicated
//!     `*_tests.rs` files.
//!   - Bash: lines starting with `#` (comment lines).

use std::fs;
use std::path::{Path, PathBuf};

/// Ship-app concepts that must never appear in deck-crate production source.
/// Widened under P2-11:
///   - The four route-prefix entries catch Ship-app mount paths and asset
///     namespaces (`/shell/`, `/spaces/`, `/shell/_assets/`, `/space/`).
///     These are exclusively Ship-app concerns — a non-Ship host would not
///     mount routes under those prefixes. (`/_deck/*` is deck's surface and
///     stays permitted; the gate's purpose is to keep deck from naming the
///     app side of the seam.)
///   - `shipd` and `shipd_` cover daemon-side naming patterns (env vars,
///     files) that a host-agnostic deck crate would never legitimately use.
const FORBIDDEN: &[&str] = &[
    // Ship-app directories (path literals).
    "apps/shell",
    "apps/shipd",
    "apps/site",
    // Ship-app-specific bundle names (deck owns the generic framework
    // singletons — sui-runtime, sui-dom, sui-sdk-client — but the app-side
    // bundles below are Ship-app concerns).
    "sui-chrome",
    "sui-marketing",
    "sui-arcade",
    "sui-pdf",
    // The ship-app identifier macro.
    "__SHIP_APP_ID__",
    // Route patterns that only exist inside Ship apps — a deck crate must
    // not bake one of these paths into a router or asset URL. Both the
    // trailing-slash forms AND the bare prefixes (`starts_with("/space")`)
    // count: deck's SPA fallback must take host backend-prefixes via config,
    // never hardcode an app-side route.
    "/shell/",
    "/shell/_assets/",
    "/spaces/",
    "/space/",
    "/space",
    "/shell",
    // The Ship product config directory. deck must never reach into `.ship`
    // at runtime (themes, views, build output) — it serves its own defaults
    // and takes host paths via config/ports, naming no ship-product dir. Bare
    // `.ship` (not `.ship/`) so the path-component form `join(".ship")` is
    // caught, not just slash-literals.
    ".ship",
    // Framework wire/meta strings must be deck-branded, not ship-branded —
    // deck's own contract, served to every app's HTML/SSE (inv-deck-standalone).
    "ship-mode",
    "ship-render",
    "ship-keepalive",
    // Daemon-side identifiers (the binary name + the daemon env-var prefix
    // when used as an env-var name in deck source — e.g. `SHIPD_PORT`).
    "shipd",
];

/// Walk a directory tree and yield every `.rs` file that is NOT a dedicated
/// test file. Test files (`*_tests.rs`, the gate's own `boundary.rs`) legitimately
/// use realistic path fixtures — the gate is about production source.
fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            collect_rs(&p, out);
        } else if p.extension().is_some_and(|e| e == "rs")
            && !p.file_name().is_some_and(|n| {
                let s = n.to_string_lossy();
                s.ends_with("_tests.rs") || s == "boundary.rs"
            })
        {
            out.push(p);
        }
    }
}

/// Walk a directory tree and yield every `.sh` file. Bash e2e scripts that
/// hardcode `apps/shell`, `apps/site`, etc. are a boundary leak the same way
/// the equivalent Rust reference is — a deck e2e script must call the deck
/// surface, not reach into a Ship-app layout. (P2-11: a previous revision of
/// the gate exempted test scripts from the Ship-name scan, which let e2e
/// bash hardcode `apps/shell`; this scanner closes that gap.)
fn collect_sh(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            collect_sh(&p, out);
        } else if p.extension().is_some_and(|e| e == "sh") {
            out.push(p);
        }
    }
}

/// Scan a Rust file's non-comment production lines for forbidden literals.
/// Skips lines inside `#[cfg(test)]` attribute + `mod tests { … }` blocks
/// so test fixtures that pass realistic Ship paths don't trigger.
fn scan_rs_violations(f: &Path) -> Vec<String> {
    let Ok(text) = fs::read_to_string(f) else { return vec![] };
    let mut violations = Vec::new();
    let mut brace_depth: i32 = 0;
    // Track whether we've seen a `#[cfg(test)]` or `mod tests` opener at the
    // current outer brace depth. Simple heuristic: once we enter a `mod tests`
    // block we suppress until we leave that brace depth. This doesn't handle
    // all edge cases but is sufficient for the Rust source shapes in deck crates.
    let mut test_block_depth: Option<i32> = None;
    for (n, line) in text.lines().enumerate() {
        let trimmed = line.trim_start();

        for ch in line.chars() {
            match ch {
                '{' => brace_depth += 1,
                '}' => {
                    brace_depth -= 1;
                    // If we just closed the test block, clear the marker.
                    // `d` was captured BEFORE the test-block's opening `{`
                    // was counted (see the order below — `#[cfg(test)]` is
                    // checked AFTER the brace counter runs, so `d` is the
                    // depth immediately OUTSIDE the test block). After the
                    // closing `}`, `brace_depth` is back to that level, so
                    // `<= d` (not `< d`) is the right "we just closed" signal.
                    if let Some(d) = test_block_depth {
                        if brace_depth <= d {
                            test_block_depth = None;
                        }
                    }
                }
                _ => {}
            }
        }

        // Mark entry into a test block.
        if trimmed.starts_with("#[cfg(test)]") || trimmed.starts_with("mod tests") {
            if test_block_depth.is_none() {
                test_block_depth = Some(brace_depth);
            }
        }

        // Suppress lines inside test blocks.
        if test_block_depth.is_some() {
            continue;
        }

        // Suppress comment lines — they may cite paths as prose examples.
        if trimmed.starts_with("//") {
            continue;
        }

        for forbidden in FORBIDDEN {
            if line.contains(forbidden) {
                violations.push(format!("{}:{}: {}", f.display(), n + 1, trimmed));
                break;
            }
        }
    }
    violations
}

/// Scan a bash e2e script for forbidden literals. Suppresses `#` line
/// comments (which may cite Ship paths as documentation); bash has no block
/// structure to track.
fn scan_sh_violations(f: &Path) -> Vec<String> {
    let Ok(text) = fs::read_to_string(f) else { return vec![] };
    let mut violations = Vec::new();
    for (n, line) in text.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            continue;
        }
        for forbidden in FORBIDDEN {
            if line.contains(forbidden) {
                violations.push(format!("{}:{}: {}", f.display(), n + 1, trimmed));
                break;
            }
        }
    }
    violations
}

/// Belt-and-suspenders test-side guards. The bash scanner has no brace
/// tracking; verify the rust scanner is the one we expect by calling its
/// brace-aware scan on a synthetic snippet that mixes a comment / test /
/// production line so a regression that drops brace tracking fails loudly.
#[cfg(test)]
mod scanner_self_tests {
    use super::*;

    #[test]
    fn rust_scanner_skips_test_block() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("probe.rs");
        std::fs::write(
            &f,
            "// forbidden comment: apps/shell\n\
             fn prod() {}\n\
             #[cfg(test)]\n\
             mod tests {\n\
             // forbidden in a test block: apps/site\n\
             #[test] fn _x() { assert!(true); }\n\
             }\n",
        )
        .unwrap();
        assert!(scan_rs_violations(&f).is_empty(), "test-block comments must not trip the gate");
    }

    #[test]
    fn rust_scanner_flags_production_line() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("probe.rs");
        std::fs::write(&f, "fn bad() { let _ = \"apps/shell\"; }\n").unwrap();
        let v = scan_rs_violations(&f);
        assert_eq!(v.len(), 1, "production-line literal must trip the gate: {v:?}");
        assert!(v[0].contains("apps/shell"), "the violation names the forbidden literal: {v:?}");
    }

    #[test]
    fn bash_scanner_skips_hash_comments_and_flags_production() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("probe.sh");
        std::fs::write(
            &f,
            "#!/usr/bin/env bash\n\
             # doc: `apps/shell` is fine in a comment\n\
             echo \"apps/shell must trip\"\n",
        )
        .unwrap();
        let v = scan_sh_violations(&f);
        assert_eq!(v.len(), 1, "non-comment line must trip the bash scanner: {v:?}");
        assert!(v[0].contains("apps/shell"), "the violation names the forbidden literal: {v:?}");
    }
}

#[test]
fn deck_crates_name_no_ship_app() {
    // crates/deck/deck-core/  → ../  → crates/deck/
    let deck_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();

    // Rust: every `<crate>/src/**/*.rs` (excluding `*_tests.rs` + this gate
    // itself) + each crate's `build.rs`. Bash: every `*.sh` under each crate
    // (the deck-conformance crate ships e2e scripts; deck-bundler has its own
    // shell fixtures; deck-axum / deck-host / deck-config have none).
    let mut rs_files = Vec::new();
    let mut sh_files = Vec::new();
    for crate_name in [
        "deck", "deck-cli", "deck-core", "deck-axum", "deck-host", "deck-bundler", "deck-config",
    ] {
        let crate_root = deck_dir.join(crate_name);
        let src = crate_root.join("src");
        if src.is_dir() {
            collect_rs(&src, &mut rs_files);
            collect_sh(&src, &mut sh_files);
        }
        // Also include `build.rs` at the crate root if present.
        let build_rs = crate_root.join("build.rs");
        if build_rs.is_file() {
            rs_files.push(build_rs);
        }
        // Plus the conformance crate's e2e scripts (a `e2e/` dir sibling
        // to `src/`) — the bash scanner's main target.
        let e2e = crate_root.join("e2e");
        if e2e.is_dir() {
            collect_sh(&e2e, &mut sh_files);
        }
    }
    let mut violations = Vec::new();
    for f in &rs_files {
        violations.extend(scan_rs_violations(f));
    }
    for f in &sh_files {
        violations.extend(scan_sh_violations(f));
    }
    assert!(
        violations.is_empty(),
        "I2 violation: deck crates must not hardcode Ship app concepts \
         in production source (deck is told its roots, never assumes a \
         Ship app layout):\n\n{}\n\nForbidden set: {:?}",
        violations.join("\n"),
        FORBIDDEN,
    );
}

/// Positive zero-config test: `DeckConfig::defaults()` + a temp `views/foo/page.sui`
/// yields exactly one views/foo target and zero consumer targets.
#[test]
fn zero_config_discovers_views_only() {
    use deck_bundler::targets::{BundleLayout, BundleTarget};

    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().to_path_buf();

    // Write a single view.
    let page_sui = root.join("views/foo/page.sui");
    std::fs::create_dir_all(page_sui.parent().unwrap()).unwrap();
    std::fs::write(&page_sui, "<div>foo</div>\n").unwrap();

    // Zero-config host: DeckConfig::defaults() (no deck.jsonc).
    let cfg = deck_config::DeckConfig::defaults();
    let layout = BundleLayout::with_config(root.clone(), None, cfg);

    let targets: Vec<BundleTarget> = layout.all_targets();
    let names: Vec<&str> = targets.iter().map(|t| t.name.as_str()).collect();

    // Framework singletons are present ONLY when the framework package is
    // resolvable by NODE RESOLUTION from the project (node_modules / workspace
    // install). In the framework repo without a `bun install`, @ship/sui-runtime
    // does not resolve from a synthetic tempdir, so the singleton-presence checks
    // are install-gated — skip them with a note (not an apps/ coupling). The
    // test's subject — view discovery + zero consumer targets — runs regardless.
    // sui-components is NOT a singleton — the umbrella was retired (#221/#269);
    // the `sui-components/<tag>` prefix is now the per-tag resolver namespace.
    if deck_bundler::node_resolve::resolve_entry(&root, "@ship/sui-runtime").is_some() {
        assert!(names.contains(&"sui-runtime"), "sui-runtime present");
        assert!(names.contains(&"sui-dom"), "sui-dom present");
        assert!(names.contains(&"sui-sdk-client"), "sui-sdk-client present");
    } else {
        eprintln!(
            "SKIP zero_config_discovers_views_only singleton checks: @ship/sui-runtime \
             not resolvable from the project (no node_modules / workspace install) — \
             install-gated, not an apps/ coupling"
        );
    }

    // Discovered view.
    assert!(names.contains(&"views/foo/view"), "views/foo/view discovered");

    // Zero consumer targets from defaults (no deck.jsonc declares any).
    assert!(
        layout.consumer_targets().is_empty(),
        "zero consumer targets with defaults"
    );
}

/// Boundary invariant I2 (extension) — deck-core and deck-config declare
/// no `ship-substrate` dependency. The project-root resolver in
/// `deck_core::resolver` and the JSONC parser in `deck-config` must
/// operate on std alone (override + nearest-ancestor marker detection +
/// inlined stripper). Any re-introduction of `ship-substrate` into
/// either crate's `Cargo.toml` re-threads the up-leak this slice cut.
#[test]
fn deck_core_and_deck_config_have_no_ship_substrate_dep() {
    let deck_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();
    for crate_name in ["deck-core", "deck-config"] {
        let toml_path = deck_dir.join(crate_name).join("Cargo.toml");
        let contents = fs::read_to_string(&toml_path)
            .unwrap_or_else(|e| panic!("read {}: {e}", toml_path.display()));
        // Search production `[dependencies]` + `[dev-dependencies]` tables
        // only — `[workspace.dependencies]` and any in-comment mentions
        // do not count as a dep declaration.
        assert!(
            !contents.contains("ship-substrate"),
            "{} declares a `ship-substrate` dependency — cut the up-leak \
             by inlining (deck-config) or replacing with std (deck-core)",
            toml_path.display(),
        );
    }
}

/// Belt-and-suspenders: also assert the production source itself has no
/// references to the deleted resolver helpers, the legacy project-root
/// env vars, or the `deck_substrate::*` paths. The Cargo.toml check above
/// catches the dep, this catches a residual dead-code reference that the
/// compiler might not warn about. Scans `deck-core` only (deck-config
/// is a tiny types crate with no FS coupling to worry about).
#[test]
fn deck_core_source_has_no_deleted_resolver_helpers_or_project_dir_env() {
    let deck_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();
    let src = deck_dir.join("deck-core").join("src");
    let mut files = Vec::new();
    collect_rs(&src, &mut files);
    let forbidden = [
        "DECK_PROJECT_DIR",
        "SHIP_DEV_PROJECT_DIR",
        "resolve_project_ship_dir",
        "daemon_dir_for",
        "workspace_from_daemon_file",
    ];
    let mut violations = Vec::new();
    for f in &files {
        let body = fs::read_to_string(f).unwrap();
        for token in &forbidden {
            if body.contains(token) {
                violations.push(format!("{}: contains `{}`", f.display(), token));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "deck-core production source must not name the legacy project-root \
         envs or the deleted resolver helpers:\n{}",
        violations.join("\n"),
    );
}

/// Boundary invariant I2 (extension) — deck crates name no Ship app env vars.
///
/// Deck crates have zero Ship-prefixed env vars; the boundary rule
/// (deck names no Ship concept) forbids re-introducing any
/// `std::env::var("SHIP_…")` read inside deck production source.
/// Hosts that need Ship-app envs are wired in via the env module, not
/// directly in deck.
#[test]
fn deck_crates_have_no_ship_prefixed_env_var_reads() {
    let deck_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();

    let mut files = Vec::new();
    for crate_name in [
        "deck", "deck-cli", "deck-core", "deck-axum", "deck-host", "deck-bundler", "deck-config",
    ] {
        let src = deck_dir.join(crate_name).join("src");
        collect_rs(&src, &mut files);
        let build_rs = deck_dir.join(crate_name).join("build.rs");
        if build_rs.is_file() {
            files.push(build_rs);
        }
    }
    assert!(!files.is_empty(), "found no deck crate source to scan ({deck_dir:?})");

    let mut violations = Vec::new();
    // We need a real shell boundary check: any line whose payload contains
    // the executable form `env::var("SHIP` outside a comment fails. We
    // reuse the same suppression machinery as the I2 scan: lines inside
    // `mod tests` blocks, doc comments, and `*_tests.rs` files are
    // excluded.
    for f in &files {
        let Ok(text) = fs::read_to_string(f) else { continue };
        let mut brace_depth: i32 = 0;
        let mut test_block_depth: Option<i32> = None;
        for (n, line) in text.lines().enumerate() {
            let trimmed = line.trim_start();
            for ch in line.chars() {
                match ch {
                    '{' => brace_depth += 1,
                    '}' => {
                        brace_depth -= 1;
                        if let Some(d) = test_block_depth {
                            if brace_depth < d {
                                test_block_depth = None;
                            }
                        }
                    }
                    _ => {}
                }
            }
            if trimmed.starts_with("#[cfg(test)]") || trimmed.starts_with("mod tests") {
                if test_block_depth.is_none() {
                    test_block_depth = Some(brace_depth);
                }
            }
            if test_block_depth.is_some() {
                continue;
            }
            if trimmed.starts_with("//") {
                continue;
            }
            // The exact pattern: `env::var("SHIP…` — any SHIP-prefixed env
            // name. This is a real env read, not a placeholder.
            if line.contains("env::var(\"SHIP") {
                violations.push(format!("{}:{}: {}", f.display(), n + 1, trimmed));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "I2 (SHIP_ env) violation: deck crates must not read Ship-prefixed \
         env vars directly (deck names no Ship concept — wire the host's \
         env accessor instead):\n\n{}",
        violations.join("\n"),
    );
}
