//! Behavior smoke tests for the `deck` CLI.
//!
//! Spec: `.artifacts/job-spec.md` (job/deck-cli-tests) — top-3 fix #3 of
//! the 2026-06-19 deck-readiness punch-list (P2-10). `crates/deck/deck-cli/`
//! shipped zero tests; `emit`, `scaffold {route,component,pack}`, and
//! `build-assets` were unproven. These tests exercise each subcommand
//! against a temp dir via `Command::new(env!("CARGO_BIN_EXE_deck"))` and
//! assert on the PRODUCED ARTIFACTS (not internal logs).
//!
//! Design notes:
//!
//! - The CLI discovers the workspace root from cwd via
//!   `topmost_cargo_root` (deck-bundler/paths.rs). Tests set the child's
//!   cwd to the repo root so the walk is deterministic.
//! - Scaffolds bail when the target dir already exists, so each test
//!   picks a unique timestamped name. A `Drop` guard removes the dirs
//!   the test wrote so reruns stay clean.
//! - `build-assets` may exit non-zero when a consumer bundle fails on
//!   this worktree (sail-marketing astro/tsconfigs/strict miss, see
//!   job-spec.md Scope). Framework singletons bake independently of
//!   consumer targets — `build_all` writes each framework bundle to
//!   `.deck/framework/` before the consumer guard runs — so the test
//!   asserts the framework artifacts regardless of overall exit code.
//! - `emit` requires its targets to be already baked. The emit test
//!   runs `build-assets` first (accepting non-zero exit), then asserts
//!   the requested target lands in the out dir.
//! - The component validator is tested by running the CLI with a
//!   non-`sail-` tag and asserting non-zero exit — no artifacts on
//!   disk means the validator was the failure point.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use tempfile::TempDir;

// Serialize tests that invoke `deck build-assets`. The command writes framework
// bundles into the shared repo-root `.deck/framework/` directory; without this
// lock, parallel test threads can truncate a file another thread is reading or
// emitting. See claim c-2026-07-19T18-12-45-685Z-f499ce4b.
static BUILD_ASSETS_LOCK: Mutex<()> = Mutex::new(());

// ── Repo + binary discovery ─────────────────────────────────────────────────

/// Resolve the cargo workspace root from `start`. Mirrors
/// `deck_bundler::paths::topmost_cargo_root` semantics (topmost, not
/// nearest) so we land on the same root the CLI will discover.
fn topmost_cargo_root(start: &Path) -> PathBuf {
    let mut cur = start;
    let mut topmost: Option<PathBuf> = None;
    for _ in 0..16 {
        if cur.join("Cargo.toml").exists() {
            topmost = Some(cur.to_path_buf());
        }
        match cur.parent() {
            Some(p) => cur = p,
            None => break,
        }
    }
    topmost.expect("could not find repo root from CARGO_MANIFEST_DIR")
}

fn repo_root() -> PathBuf {
    let start = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    topmost_cargo_root(&start)
}

fn deck_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_deck"))
}

/// Build a unique lowercase-hyphen stem from the unix epoch nanos.
/// Satisfies `/^[a-z][a-z0-9-]*$/` (the validators enforce this) while
/// avoiding collisions across concurrent reruns.
fn unique_stem(label: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{label}-{nanos:x}")
}

/// Run `deck <args...>` with cwd = repo root. Returns the `Output`
/// (stdout, stderr, status) for the caller to assert on.
fn run_deck(args: &[&str]) -> std::process::Output {
    Command::new(deck_bin())
        .args(args)
        .current_dir(repo_root())
        .output()
        .expect("spawn deck binary")
}

// ── Scaffold cleanup guard ──────────────────────────────────────────────────

/// Removes scaffold dirs on drop so reruns / panics don't leave residue.
struct ScaffoldGuard {
    paths: Vec<PathBuf>,
}

impl ScaffoldGuard {
    fn new() -> Self {
        Self { paths: Vec::new() }
    }
    fn track(&mut self, p: PathBuf) {
        self.paths.push(p);
    }
}

impl Drop for ScaffoldGuard {
    fn drop(&mut self) {
        for p in &self.paths {
            let _ = std::fs::remove_dir_all(p);
        }
    }
}

// ── build-assets ────────────────────────────────────────────────────────────

/// `deck build-assets` must materialize the framework singletons
/// (the bundles deck-core's build.rs stages into `assets/` for
/// `include_str!`). Framework targets bake independently of consumer
/// targets in `build_all`, so their on-disk artifacts are present
/// even when a consumer target (e.g. sail-marketing) fails the bake.
///
/// Only the singletons this repo actually PROVIDES are asserted. Some
/// framework singletons are host-provided and node-resolved from the
/// project being built (`@ship/sdk-client` → `sail-sdk-client.js`,
/// `@ship/sdk` → `sdk.js`): the bundler skips them when the package is
/// absent (this repo does not vend `@ship/sdk-client`), and deck-core's
/// build.rs stages a bootstrap stub for the missing embed rather than
/// failing the build. Asserting `sail-sdk-client.js` here would demand a
/// package this repo doesn't ship.
#[test]
fn build_assets_writes_framework_bundles() {
    let _guard = BUILD_ASSETS_LOCK.lock().unwrap();

    let out = run_deck(&["build-assets", "--no-stamp"]);
    let stderr = String::from_utf8_lossy(&out.stderr);
    // Don't assert status — consumer failures (sail-marketing) are a
    // known-bad on this worktree. Framework bundles are written
    // before build_all returns; their absence is the real signal.
    let fw_dir = repo_root().join(".deck").join("framework");
    for name in ["sail-runtime.js", "sail-dom.js", "deck-route.js"] {
        let path = fw_dir.join(name);
        assert!(
            path.exists(),
            "framework bundle missing after build-assets: {} (stderr: {stderr})",
            path.display()
        );
        let len = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        assert!(
            len > 0,
            "framework bundle empty after build-assets: {} (stderr: {stderr})",
            path.display()
        );
    }
}

// ── emit ────────────────────────────────────────────────────────────────────

/// `deck emit --target <name> --out <dir>` must place the named target
/// (and its lazy-chunk closure) into `<dir>`. Uses the framework
/// singleton `sail-runtime` because it bundles deterministically on
/// this worktree (no consumer-bundle dependency).
///
/// Runs `build-assets` first so the target is on disk; accepts a
/// non-zero build-assets exit (see `build_assets_writes_framework_bundles`).
#[test]
fn emit_places_target_in_out_dir() {
    let _guard = BUILD_ASSETS_LOCK.lock().unwrap();

    // Ensure the target is baked. If the framework dir is already
    // populated, this is a fast no-op rewrite.
    let _ = run_deck(&["build-assets", "--no-stamp"]);

    let out = TempDir::new().expect("tempdir");
    let status = run_deck(&[
        "emit",
        "--target",
        "sail-runtime",
        "--out",
        out.path().to_str().unwrap(),
    ])
    .status;
    assert!(
        status.success(),
        "deck emit --target sail-runtime exited {status:?}"
    );
    let entry = out.path().join("sail-runtime.js");
    assert!(
        entry.exists(),
        "emit did not write {}",
        entry.display()
    );
    let len = std::fs::metadata(&entry).map(|m| m.len()).unwrap_or(0);
    assert!(len > 0, "emit wrote empty {}", entry.display());
}

// ── scaffold route ──────────────────────────────────────────────────────────

/// `deck scaffold route <name>` writes a generic `routes/<name>/page.sail`.
/// Deck derives the URL from the directory tree — no `manifest.jsonc`, no
/// host metadata (`auth`/`rail`/`layout`/`title` are a Ship-shell concern,
/// excised from deck). The retired sibling files (`controller.ts`,
/// `style.css`) and the Ship manifest must NOT be written.
#[test]
fn scaffold_route_writes_page_and_no_manifest() {
    let mut guard = ScaffoldGuard::new();
    let name = unique_stem("testroute");
    let route_dir = repo_root().join("routes").join(&name);
    guard.track(route_dir.clone());

    let status = run_deck(&["scaffold", "route", &name]).status;
    assert!(status.success(), "deck scaffold route {name} exited {status:?}");

    let page = route_dir.join("page.sail");
    assert!(page.exists(), "missing {} after scaffold route", page.display());
    let len = std::fs::metadata(&page).map(|m| m.len()).unwrap_or(0);
    assert!(len > 0, "empty {} after scaffold route", page.display());

    // The Ship-Views manifest + retired siblings must NOT be written.
    for rel in ["manifest.jsonc", "controller.ts", "style.css"] {
        assert!(
            !route_dir.join(rel).exists(),
            "deck must not write {rel} — that is a Ship-Views concern"
        );
    }
}

/// `deck scaffold route --component <tag>` writes the route-private
/// widget under `routes/<name>/_components/<tag>/`.
#[test]
fn scaffold_route_with_component_writes_widget() {
    let mut guard = ScaffoldGuard::new();
    let name = unique_stem("testroute");
    let route_dir = repo_root().join("routes").join(&name);
    let tag = format!("{name}-widget");
    let widget_dir = route_dir.join("_components").join(&tag);
    guard.track(route_dir);

    let status = run_deck(&[
        "scaffold",
        "route",
        &name,
        "--component",
        &tag,
    ])
    .status;
    assert!(
        status.success(),
        "deck scaffold route {name} --component {tag} exited {status:?}"
    );

    let p = widget_dir.join(format!("{tag}.sail"));
    assert!(p.exists(), "missing {} after scaffold route --component", p.display());
    let len = std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0);
    assert!(len > 0, "empty {} after scaffold route --component", p.display());
    assert!(
        !widget_dir.join(format!("{tag}.controller.ts")).exists(),
        "retired sibling controller written by scaffold route --component"
    );
}

// ── scaffold component ──────────────────────────────────────────────────────

/// `deck scaffold component <tag>` writes the `.sail` (and optional
/// controller) under the SAIL primitives package:
/// `packages/components/sail/primitives/src/<tag>/<tag>.sail`.
#[test]
fn scaffold_component_writes_sui_under_primitives_package() {
    let mut guard = ScaffoldGuard::new();
    let tag = unique_stem("sail-testcomp");
    let base = repo_root()
        .join("packages/components/sail/primitives/src")
        .join(&tag);
    guard.track(base.clone());

    let status = run_deck(&["scaffold", "component", &tag]).status;
    assert!(
        status.success(),
        "deck scaffold component {tag} exited {status:?}"
    );

    let sail = base.join(format!("{tag}.sail"));
    assert!(sail.exists(), "missing {} after scaffold component", sail.display());
    let len = std::fs::metadata(&sail).map(|m| m.len()).unwrap_or(0);
    assert!(len > 0, "empty {} after scaffold component", sail.display());

    // Without --controller, no controller file should be written.
    assert!(
        !base.join(format!("{tag}.controller.ts")).exists(),
        "controller file written without --controller"
    );
}

/// The component validator rejects tags that don't start with `sail-`.
/// Tested by running the CLI with a non-`sail-` tag and asserting
/// non-zero exit; the validator runs before any directory is created,
/// so the absence of artifacts on disk is the failure-mode proof.
#[test]
fn scaffold_component_rejects_non_sui_tag() {
    // `tag` must pass `validate_route_name` (lowercase ASCII + hyphens
    // + digits) but violate the `sail-` prefix requirement.
    let bad_tag = unique_stem("not-sail-widget");
    let base = repo_root()
        .join("packages/components/sail/primitives/src")
        .join(&bad_tag);
    // Cleanup guard in case validation is bypassed (defense in depth).
    let mut guard = ScaffoldGuard::new();
    guard.track(base.clone());

    let output = run_deck(&["scaffold", "component", &bad_tag]);
    assert!(
        !output.status.success(),
        "deck scaffold component accepted non-sail tag (status: {:?})",
        output.status
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("sail-"),
        "expected validator message mentioning `sail-`, got: {stderr}"
    );
    assert!(
        !base.exists(),
        "validator rejected the tag but directory was created: {}",
        base.display()
    );
}

// ── scaffold pack ───────────────────────────────────────────────────────────

/// `deck scaffold pack <name>` writes `package.json`, `tsconfig.json`,
/// and the `src/` tree under `packages/components/<name>/`. The
/// package.json's `name` field is `@ship/<name>`.
#[test]
fn scaffold_pack_writes_package_json_tsconfig_and_src() {
    let mut guard = ScaffoldGuard::new();
    let name = unique_stem("testpack");
    let base = repo_root().join("packages/components").join(&name);
    let example_dir = base.join("src").join(format!("{name}-example"));
    guard.track(base.clone());

    let status = run_deck(&["scaffold", "pack", &name]).status;
    assert!(status.success(), "deck scaffold pack {name} exited {status:?}");

    for rel in ["package.json", "tsconfig.json", "src/index.ts"] {
        let p = base.join(rel);
        assert!(p.exists(), "missing {} after scaffold pack", p.display());
        let len = std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0);
        assert!(len > 0, "empty {} after scaffold pack", p.display());
    }
    // Placeholder example primitive under src/<name>-example/.
    let p = example_dir.join(format!("{name}-example.sail"));
    assert!(p.exists(), "missing {} after scaffold pack", p.display());
    let len = std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0);
    assert!(len > 0, "empty {} after scaffold pack", p.display());
    assert!(
        !example_dir
            .join(format!("{name}-example.controller.ts"))
            .exists(),
        "retired sibling controller written by scaffold pack"
    );

    // package.json `name` field must be `@ship/<name>`.
    let pkg = std::fs::read_to_string(base.join("package.json")).expect("read package.json");
    let expected = format!("\"name\": \"@ship/{name}\"");
    assert!(
        pkg.contains(&expected),
        "package.json missing {expected:?}; contents:\n{pkg}"
    );
}

// ── inv-deck-standalone (behavioral proof) ───────────────────────────────────

/// The REAL standalone test (the boundary lint in deck-conformance is only the
/// backstop): deck BUILDS a zero-ship-PRODUCT app — a plain `routes/` page that
/// depends on the SAIL framework and nothing of the ship product — into a served
/// SPA. If deck can compile + emit a runnable app that names nothing ship, it
/// stands on its own. This serves development (it proves the capability) and
/// doesn't break when a path is renamed, unlike a forbidden-string scan.
#[test]
fn builds_a_zero_ship_product_app() {
    let proj = TempDir::new().expect("tempdir");
    let root = proj.path();
    std::fs::create_dir_all(root.join("routes")).expect("routes dir");
    // Framework deps only (@ship/* is the org npm SCOPE — sail-runtime/dom are
    // framework packages, not the ship product). No @ship/sdk-client, no
    // ship.* client, no .ship/ — a genuinely host-free app.
    std::fs::write(
        root.join("package.json"),
        r#"{"name":"zeroship","type":"module","dependencies":{"@ship/sail-runtime":"workspace:*","@ship/sail-dom":"workspace:*"}}"#,
    )
    .expect("package.json");
    std::fs::write(
        root.join("routes").join("page.sail"),
        "<main>\n  <h1>zero-ship</h1>\n  <p>sum is {1 + 1}</p>\n</main>\n",
    )
    .expect("page.sail");

    let out = run_deck(&["build", "--project", root.to_str().unwrap()]);
    assert!(
        out.status.success(),
        "deck build of a zero-ship-product app failed (status {:?}):\n{}",
        out.status,
        String::from_utf8_lossy(&out.stderr),
    );

    // The built SPA shell exists and is wired (importmap + a mount root).
    let index = root.join(".deck").join("index.html");
    assert!(index.exists(), "deck build did not emit .deck/index.html");
    let html = std::fs::read_to_string(&index).expect("read index.html");
    assert!(html.contains("importmap"), "built index.html missing importmap:\n{html}");
    assert!(html.contains("root"), "built index.html missing the #root mount point:\n{html}");

    // The entry chunk was bundled (the routes/ page compiled into the SPA).
    let assets = root.join(".deck").join("assets");
    let entry_js = std::fs::read_dir(&assets)
        .ok()
        .and_then(|d| {
            d.filter_map(|e| e.ok()).find(|e| {
                let n = e.file_name().to_string_lossy().into_owned();
                n.starts_with("entry-") && n.ends_with(".js")
            })
        })
        .map(|e| e.path());
    let entry_js = entry_js.unwrap_or_else(|| {
        panic!("deck build did not emit an entry-*.js chunk in {}", assets.display())
    });

    // The entry must BOOTSTRAP, not merely export data: a standalone app's
    // entry chunk has to call `mountRouter` so it actually mounts into
    // #root. (Build-success alone is not proof the app runs — deck once
    // emitted a data-only entry that never mounted; this guards that
    // regression at build time, the browser e2e proves it end-to-end.)
    let entry_src = std::fs::read_to_string(&entry_js).expect("read entry chunk");
    assert!(
        entry_src.contains("mountRouter"),
        "entry chunk must call mountRouter (the SPA bootstrap), else nothing mounts:\n{}",
        &entry_src[..entry_src.len().min(600)],
    );
}
