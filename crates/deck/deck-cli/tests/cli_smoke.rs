//! Behavior smoke tests for the `sail` + `deck-bake` CLIs (`deck-cli` package).
//!
//! Behavior coverage for `deck-bake emit` / `deck-bake build-assets`, plus
//! product `sail` verbs. Bake subcommands run via
//! `Command::new(env!("CARGO_BIN_EXE_deck-bake"))` and assert on the PRODUCED
//! ARTIFACTS (not internal logs).
//!
//! Design notes:
//!
//! - The CLI discovers the workspace root from cwd via
//!   `deck_bundler::paths::topmost_workspace_root`. Tests set the child's
//!   cwd to the repo root so the walk is deterministic.
//! - Create/theme scaffolds bail when the target dir already exists, so
//!   each test picks a unique timestamped name (tempdirs for create).
//! - `build-assets` may exit non-zero when a consumer bundle fails on
//!   this worktree (missing consumer tsconfigs / strict miss). Framework
//!   singletons bake independently of
//!   consumer targets — `build_all` writes each framework bundle to
//!   `.sail/framework/` before the consumer guard runs — so the test
//!   asserts the framework artifacts regardless of overall exit code.
//! - `emit` requires its targets to be already baked. The emit test
//!   runs `build-assets` first (accepting non-zero exit), then asserts
//!   the requested target lands in the out dir.

use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use tempfile::TempDir;

// Serialize tests that invoke `deck-bake build-assets`. The command writes
// framework bundles into the shared repo-root bake directory; without this
// lock, parallel test threads can truncate a file another thread is reading or
// emitting. See claim c-2026-07-19T18-12-45-685Z-f499ce4b.
static BUILD_ASSETS_LOCK: Mutex<()> = Mutex::new(());

// ── Repo + binary discovery ─────────────────────────────────────────────────

fn repo_root() -> PathBuf {
    let start = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    deck_bundler::paths::topmost_cargo_root(&start)
        .expect("could not find repo root from CARGO_MANIFEST_DIR")
}

fn sail_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_sail"))
}

fn bake_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_deck-bake"))
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

/// Run `sail <args...>` with cwd = repo root. Returns the `Output`
/// (stdout, stderr, status) for the caller to assert on.
fn run_sail(args: &[&str]) -> std::process::Output {
    Command::new(sail_bin())
        .args(args)
        .current_dir(repo_root())
        .output()
        .expect("spawn sail binary")
}

/// Run `deck-bake <args...>` with cwd = repo root.
fn run_bake(args: &[&str]) -> std::process::Output {
    Command::new(bake_bin())
        .args(args)
        .current_dir(repo_root())
        .output()
        .expect("spawn deck-bake binary")
}

fn help_has_subcommand(help: &str, cmd: &str) -> bool {
    help.lines()
        .any(|line| line.split_whitespace().next() == Some(cmd))
}

// ── build-assets ────────────────────────────────────────────────────────────

/// `deck-bake build-assets` must materialize the framework singletons
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

    let out = run_bake(&["build-assets", "--no-stamp"]);
    let stderr = String::from_utf8_lossy(&out.stderr);
    // Don't assert status — consumer failures (sail-marketing) are a
    // known-bad on this worktree. Framework bundles are written
    // before build_all returns; their absence is the real signal.
    let fw_dir = repo_root().join(".sail").join("framework");
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

/// `deck-bake emit --target <name> --out <dir>` must place the named target
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
    let _ = run_bake(&["build-assets", "--no-stamp"]);

    let out = TempDir::new().expect("tempdir");
    let status = run_bake(&[
        "emit",
        "--target",
        "sail-runtime",
        "--out",
        out.path().to_str().unwrap(),
    ])
    .status;
    assert!(
        status.success(),
        "deck-bake emit --target sail-runtime exited {status:?}"
    );
    let entry = out.path().join("sail-runtime.js");
    assert!(entry.exists(), "emit did not write {}", entry.display());
    let len = std::fs::metadata(&entry).map(|m| m.len()).unwrap_or(0);
    assert!(len > 0, "emit wrote empty {}", entry.display());
}

// ── scaffold theme (C5) ─────────────────────────────────────────────────────

/// `deck scaffold theme --project <tmp>` copies one catalog theme into
/// `themes/` and writes a thin `sail.config.jsonc` (theme + nested deck).
#[test]
fn scaffold_theme_copies_default_and_writes_config() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let project = tmp.path().join("app");
    std::fs::create_dir_all(&project).expect("mkdir project");

    let status = run_sail(&[
        "scaffold",
        "theme",
        "--project",
        project.to_str().expect("utf8 path"),
    ])
    .status;
    assert!(status.success(), "deck scaffold theme exited {status:?}");

    // Modern external layout: <project>/themes/<id>.json (EXT-1 E4).
    let theme = project.join("themes/sail.json");
    assert!(
        theme.is_file(),
        "missing {} after scaffold theme",
        theme.display()
    );
    let len = std::fs::metadata(&theme).map(|m| m.len()).unwrap_or(0);
    assert!(len > 0, "empty theme file after scaffold theme");
    let theme_body = std::fs::read_to_string(&theme).expect("read theme");
    assert!(
        theme_body.contains("\"modes\"") || theme_body.contains("themeId"),
        "scaffold theme should carry modes-in-file or themeId meta: {theme_body}"
    );

    let cfg = project.join("sail.config.jsonc");
    assert!(
        cfg.is_file(),
        "missing sail.config.jsonc after scaffold theme"
    );
    let body = std::fs::read_to_string(&cfg).expect("read config");
    assert!(
        body.contains("\"default\": \"sail\""),
        "config missing default theme: {body}"
    );
    // Thin single-id allowlist (C5) — not a full catalog list / null.
    assert!(
        body.contains("\"enabled\": [\"sail\"]"),
        "config must thin-enable only sail: {body}"
    );
    assert!(
        body.contains("\"deck\"") && body.contains("\"render\": \"csr\""),
        "scaffold must emit nested deck.render csr: {body}"
    );
    assert!(
        !body.contains("paper") && !body.contains("glass"),
        "scaffold must not dump full catalog into enabled: {body}"
    );

    // S0: theme scaffold ensures gitignore + .sail/README (tasks never commit).
    let gi = std::fs::read_to_string(project.join(".gitignore")).expect("read .gitignore");
    for line in [
        "dist/",
        ".sail/",
        "routes.gen.ts",
        "entry.gen.ts",
        "components.gen.ts",
        "node_modules/",
        ".env",
    ] {
        assert!(
            gi.lines().any(|l| {
                let t = l.trim().trim_start_matches('/');
                t == line.trim_start_matches('/')
            }),
            "gitignore missing critical {line:?}:\n{gi}"
        );
    }
    assert!(
        !gi.contains("tokens/themes") && !gi.contains("sail.config"),
        "must not ignore theme SSOT: {gi}"
    );
    let readme = project.join(".sail/README.md");
    assert!(
        readme.is_file(),
        "missing {} after scaffold theme",
        readme.display()
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
    // A Project is nearest sail.config — routes/ alone is deck detect
    // zero-config, not `sail build`.
    std::fs::write(root.join("sail.config.jsonc"), "{}\n").expect("sail.config");
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

    let out = run_sail(&["build", "--project", root.to_str().unwrap()]);
    assert!(
        out.status.success(),
        "deck build of a zero-ship-product app failed (status {:?}):\n{}",
        out.status,
        String::from_utf8_lossy(&out.stderr),
    );

    // The built SPA shell exists and is wired (importmap + a mount root).
    let index = root.join("dist").join("index.html");
    assert!(index.exists(), "deck build did not emit dist/index.html");
    let html = std::fs::read_to_string(&index).expect("read index.html");
    assert!(
        html.contains("importmap"),
        "built index.html missing importmap:\n{html}"
    );
    assert!(
        html.contains("root"),
        "built index.html missing the #root mount point:\n{html}"
    );

    // The entry chunk was bundled (the routes/ page compiled into the SPA).
    let assets = root.join("dist").join("assets");
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
        panic!(
            "deck build did not emit an entry-*.js chunk in {}",
            assets.display()
        )
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

// ── sail create (EXT-1 E8) ──────────────────────────────────────────────────

/// `sail create my-app` writes the external-app 4-entry tree + gitignore
/// (with `dist/` + `.sail/`), and `sail check` on the result is green.
#[test]
fn create_writes_four_entry_tree_and_check_is_green() {
    let tmp = TempDir::new().expect("tempdir");
    let name = unique_stem("create-app");
    let app = tmp.path().join(&name);

    let out = Command::new(sail_bin())
        .args(["create", app.to_str().unwrap()])
        .current_dir(tmp.path())
        .output()
        .expect("spawn sail create");
    assert!(
        out.status.success(),
        "sail create failed (status {:?}):\nstdout:{}\nstderr:{}",
        out.status,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );

    // 4-entry tree: package.json, sail.config.jsonc, routes/, themes/
    for rel in [
        "package.json",
        "sail.config.jsonc",
        "routes/page.sail",
        "themes/sail.json",
    ] {
        let p = app.join(rel);
        assert!(
            p.exists(),
            "missing {rel} after sail create: {}",
            p.display()
        );
        let len = std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0);
        assert!(len > 0, "empty {rel} after sail create");
    }

    // .gitignore enforces dist/ never committed (deploy out)
    let gi = std::fs::read_to_string(app.join(".gitignore")).expect("read .gitignore");
    assert!(
        gi.lines().any(|l| {
            let t = l.trim().trim_start_matches('/');
            t == "dist/" || t == "dist"
        }),
        "gitignore missing dist/:\n{gi}"
    );

    // Registry-style deps only (no file: / workspace:*)
    let pkg = std::fs::read_to_string(app.join("package.json")).expect("package.json");
    assert!(
        !pkg.contains("file:") && !pkg.contains("workspace:"),
        "create package.json must use registry deps, not file:/workspace:\n{pkg}"
    );
    assert!(
        pkg.contains("@ship/sail-runtime"),
        "package.json missing @ship/sail-runtime:\n{pkg}"
    );
    // TOOLING-1 WI2: typecheck/lint scripts + tooling devDeps
    assert!(
        pkg.contains("\"typecheck\"") && pkg.contains("\"lint\""),
        "create package.json missing typecheck/lint scripts:\n{pkg}"
    );
    assert!(
        pkg.contains("typescript") && pkg.contains("oxlint"),
        "create package.json missing typescript/oxlint devDeps:\n{pkg}"
    );
    assert!(
        pkg.contains("sail build --bundle-framework"),
        "create build script must emit a self-contained bundle:\n{pkg}"
    );

    // G2c / TOOLING-1 WI2: solution root + kind configs under .sail/
    // + ambient pack + oxlint. Root is references-only; flags live on pure/action.
    let tsconfig = std::fs::read_to_string(app.join("tsconfig.json")).expect("tsconfig.json");
    assert!(
        tsconfig.contains("references")
            && tsconfig.contains(".sail/tsconfig.sail.json")
            && tsconfig.contains(".sail/tsconfig.action.json"),
        "create root tsconfig must be a solution referencing pure+action under .sail/:\n{tsconfig}"
    );
    assert!(
        !app.join("tsconfig.sail.json").exists() && !app.join("tsconfig.action.json").exists(),
        "create must not emit root-level tsconfig.pure/action.json (G2c)"
    );
    let pure = std::fs::read_to_string(app.join(".sail/tsconfig.sail.json"))
        .expect(".sail/tsconfig.sail.json");
    for flag in [
        "\"strict\": true",
        "\"noUncheckedIndexedAccess\": true",
        "\"exactOptionalPropertyTypes\": true",
        "\"noImplicitOverride\": true",
        "\"verbatimModuleSyntax\": true",
        "\"erasableSyntaxOnly\": true",
        "\"isolatedModules\": true",
        "\"moduleResolution\": \"Bundler\"",
        "\"target\": \"ESNext\"",
        "\"skipLibCheck\": true",
        "types.d.ts",
    ] {
        assert!(
            pure.contains(flag),
            "create pure kind tsconfig missing WI2 flag/pack {flag}:\n{pure}"
        );
    }
    assert!(
        app.join(".sail/types.d.ts").is_file(),
        "create must emit ambient pack .sail/types.d.ts"
    );
    assert!(
        app.join(".sail/tsconfig.action.json").is_file(),
        "create must emit action kind tsconfig under .sail/"
    );
    let oxlint = std::fs::read_to_string(app.join(".oxlintrc.json")).expect(".oxlintrc.json");
    assert!(
        oxlint.contains("typescript") && oxlint.contains("correctness"),
        "create .oxlintrc.json must enable typescript + correctness:\n{oxlint}"
    );
    let cfg = std::fs::read_to_string(app.join("sail.config.jsonc")).expect("sail.config.jsonc");
    assert!(
        cfg.contains("\"default\": \"sail\"") && cfg.contains("\"preset\": \"strict\""),
        "create config must default to sail + explicit strict preset:\n{cfg}"
    );

    // create → check path green
    let check = Command::new(sail_bin())
        .args(["check", app.to_str().unwrap()])
        .current_dir(tmp.path())
        .output()
        .expect("spawn sail check");
    assert!(
        check.status.success(),
        "sail check after create failed (status {:?}):\nstdout:{}\nstderr:{}",
        check.status,
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr),
    );
    let stderr = String::from_utf8_lossy(&check.stderr);
    assert!(
        !stderr.contains("dist/ is tracked"),
        "fresh create must not warn about tracked dist/:\n{stderr}"
    );
    assert!(
        !stderr.contains("carries no capability signal"),
        "fresh create must not dump a NoSignal policy warning:\n{stderr}"
    );
}

/// FIRST-RUN: create → `sail build` with no `bun install` and no monorepo cwd.
/// `--bundle-framework` still needs `@ship/*` on disk; the no-flag build must
/// not. Hello-world output has no `.wasm` (EXT-1 E5).
#[test]
fn create_then_build_without_install() {
    let tmp = TempDir::new().expect("tempdir");
    let app = tmp.path().join(unique_stem("create-build"));
    let create = Command::new(sail_bin())
        .args(["create", app.to_str().unwrap()])
        .current_dir(tmp.path())
        .output()
        .expect("spawn sail create");
    assert!(
        create.status.success(),
        "create failed: {}",
        String::from_utf8_lossy(&create.stderr)
    );
    assert!(
        !app.join("node_modules").exists(),
        "create must not write node_modules"
    );

    let build = Command::new(sail_bin())
        .args(["build", "--project", app.to_str().unwrap()])
        .current_dir(tmp.path())
        .output()
        .expect("spawn sail build");
    assert!(
        build.status.success(),
        "sail build after create (no install) failed (status {:?}):\nstdout:{}\nstderr:{}",
        build.status,
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr),
    );
    let index = app.join("dist/index.html");
    assert!(index.is_file(), "build must emit dist/index.html");
    let html = std::fs::read_to_string(&index).expect("index.html");
    assert!(
        html.contains("importmap") && html.contains("root"),
        "built index missing importmap/#root:\n{html}"
    );
    assert!(
        html.contains("sail-default-theme") && html.contains("sail"),
        "built index must stamp the sail brand default:\n{html}"
    );
    let wasm = walk_ext(&app.join("dist"), "wasm");
    assert!(
        wasm.is_empty(),
        "hello-world build must not ship wasm: {wasm:?}"
    );
}

/// FIRST-RUN: `sail dev` on a created app outside the monorepo serves HTML.
#[test]
fn create_then_dev_serves() {
    let tmp = TempDir::new().expect("tempdir");
    let app = tmp.path().join(unique_stem("create-dev"));
    let create = Command::new(sail_bin())
        .args(["create", app.to_str().unwrap()])
        .current_dir(tmp.path())
        .output()
        .expect("spawn sail create");
    assert!(
        create.status.success(),
        "create failed: {}",
        String::from_utf8_lossy(&create.stderr)
    );

    let port = free_port();
    let addr = format!("127.0.0.1:{port}");
    let mut child = Command::new(sail_bin())
        .args(["dev", "--project", app.to_str().unwrap(), "--addr", &addr])
        .current_dir(tmp.path())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn sail dev");

    let body = match wait_http_get(&addr, "/", std::time::Duration::from_secs(30)) {
        Ok(b) => b,
        Err(e) => {
            let _ = child.kill();
            let (stdout, stderr) = match child.wait_with_output() {
                Ok(o) => (
                    String::from_utf8_lossy(&o.stdout).into_owned(),
                    String::from_utf8_lossy(&o.stderr).into_owned(),
                ),
                Err(wait_err) => (String::new(), wait_err.to_string()),
            };
            panic!("sail dev did not serve {addr}: {e}\nstdout:{stdout}\nstderr:{stderr}");
        }
    };
    let _ = child.kill();
    let _ = child.wait();

    assert!(
        body.contains("<html") && body.contains("sail-default-theme"),
        "dev index missing html/theme meta:\n{body}"
    );
    assert!(
        body.contains("content=\"sail\""),
        "dev must paint the sail brand default:\n{body}"
    );
}

fn walk_ext(dir: &std::path::Path, ext: &str) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(dir) else {
        return out;
    };
    for ent in rd.flatten() {
        let p = ent.path();
        if p.is_dir() {
            out.extend(walk_ext(&p, ext));
        } else if p.extension().and_then(|e| e.to_str()) == Some(ext) {
            out.push(p);
        }
    }
    out
}

fn free_port() -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind :0");
    let port = listener.local_addr().expect("local_addr").port();
    drop(listener);
    port
}

fn wait_http_get(addr: &str, path: &str, budget: std::time::Duration) -> Result<String, String> {
    let start = std::time::Instant::now();
    let mut last = String::from("no attempt");
    while start.elapsed() < budget {
        match http_get(addr, path) {
            Ok(body) => return Ok(body),
            Err(e) => last = e,
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
    Err(last)
}

fn http_get(addr: &str, path: &str) -> Result<String, String> {
    use std::io::{Read, Write};
    let mut stream = std::net::TcpStream::connect(addr).map_err(|e| e.to_string())?;
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(2)))
        .ok();
    let req = format!("GET {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n");
    stream
        .write_all(req.as_bytes())
        .map_err(|e| e.to_string())?;
    let mut buf = String::new();
    stream.read_to_string(&mut buf).map_err(|e| e.to_string())?;
    if !buf.starts_with("HTTP/1.1 200") && !buf.starts_with("HTTP/1.0 200") {
        return Err(format!("not 200: {}", buf.lines().next().unwrap_or("")));
    }
    Ok(buf)
}

/// `sail check` warns (does not fail) when `dist/` is git-tracked.
#[test]
fn check_warns_when_dist_is_tracked() {
    let tmp = TempDir::new().expect("tempdir");
    let name = unique_stem("dist-tracked");
    let app = tmp.path().join(&name);

    let create = Command::new(sail_bin())
        .args(["create", app.to_str().unwrap()])
        .current_dir(tmp.path())
        .output()
        .expect("spawn sail create");
    assert!(
        create.status.success(),
        "sail create failed: {}",
        String::from_utf8_lossy(&create.stderr)
    );

    // Init a git repo in the app and force-add a dist artifact despite gitignore.
    let git = |args: &[&str]| {
        let st = Command::new("git")
            .args(args)
            .current_dir(&app)
            .output()
            .expect("git");
        assert!(
            st.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&st.stderr)
        );
    };
    git(&["init"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "test"]);
    std::fs::create_dir_all(app.join("dist")).expect("mkdir dist");
    std::fs::write(app.join("dist").join("index.html"), "<html></html>\n").expect("write");
    // -f forces through .gitignore so we can simulate a bad commit of deploy output.
    git(&["add", "-f", "dist/index.html"]);
    git(&["commit", "-m", "track dist by mistake"]);

    let check = Command::new(sail_bin())
        .args(["check", "."])
        .current_dir(&app)
        .output()
        .expect("spawn sail check");
    assert!(
        check.status.success(),
        "sail check must still succeed (warning only); status {:?}:\nstdout:{}\nstderr:{}",
        check.status,
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr),
    );
    let stderr = String::from_utf8_lossy(&check.stderr);
    assert!(
        stderr.contains("dist/ is tracked") || stderr.contains("dist is tracked"),
        "expected dist/ tracked warning, got:\n{stderr}"
    );
}

// ── sail surface (C-pack) ───────────────────────────────────────────────────

#[test]
fn sail_version_and_doctor_smoke() {
    let out = Command::new(sail_bin())
        .args(["version"])
        .current_dir(repo_root())
        .output()
        .expect("spawn sail");
    assert!(out.status.success(), "sail version failed: {:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("sail") || stdout.contains("0.1.0"),
        "unexpected version output: {stdout}"
    );

    let doc = Command::new(sail_bin())
        .args(["doctor"])
        .current_dir(repo_root())
        .output()
        .expect("spawn sail doctor");
    assert!(doc.status.success(), "sail doctor failed: {:?}", doc);
    let d = String::from_utf8_lossy(&doc.stdout);
    assert!(d.contains("status: ok"), "doctor: {d}");
    assert!(
        d.contains("sail mcp --list-tools") && d.contains("tools (ok)"),
        "doctor must probe in-process list-tools: {d}"
    );
}

/// Empty probe dirs still fail on bun/tsc/types, but list-tools must not
/// require a project MCP config (in-process sail-mcp).
#[test]
fn doctor_list_tools_does_not_require_mcp_config() {
    let tmp = TempDir::new().expect("tempdir");
    let doc = Command::new(sail_bin())
        .args(["doctor"])
        .current_dir(tmp.path())
        .output()
        .expect("spawn sail doctor");
    let stdout = String::from_utf8_lossy(&doc.stdout);
    let stderr = String::from_utf8_lossy(&doc.stderr);
    assert!(
        stdout.contains("sail mcp --list-tools") && stdout.contains("tools (ok)"),
        "list-tools must pass without .mcp.json:\nstdout:{stdout}\nstderr:{stderr}"
    );
    assert!(
        !doc.status.success(),
        "empty probe must still fail (types/bun/tsc), got success:\n{stdout}"
    );
    assert!(
        stdout.contains("status: fail"),
        "empty probe must print status: fail:\n{stdout}"
    );
    assert!(
        stdout.contains("tokens model: baked catalog (ok)"),
        "standalone doctor must not demand a monorepo tokens tree:\n{stdout}"
    );
    assert!(
        !stderr.contains("sail mcp --list-tools"),
        "list-tools must not be the doctor failure:\n{stderr}"
    );
}

#[test]
fn sail_help_lists_mvp_subcommands() {
    let out = Command::new(sail_bin())
        .args(["--help"])
        .current_dir(repo_root())
        .output()
        .expect("spawn sail --help");
    assert!(out.status.success());
    let help = String::from_utf8_lossy(&out.stdout);
    for cmd in [
        "create", "dev", "build", "scaffold", "tokens", "compile", "check", "fmt", "lsp", "mcp",
        "editor", "op", "layout", "doctor", "version", "types",
    ] {
        assert!(
            help_has_subcommand(&help, cmd),
            "help missing subcommand `{cmd}`:\n{help}"
        );
    }
    for cmd in ["build-assets", "emit", "routes"] {
        assert!(
            !help_has_subcommand(&help, cmd),
            "product sail --help must not list bake verb `{cmd}`:\n{help}"
        );
    }
}

#[test]
fn deck_bake_help_lists_bake_verbs() {
    let out = Command::new(bake_bin())
        .args(["--help"])
        .current_dir(repo_root())
        .output()
        .expect("spawn deck-bake --help");
    assert!(out.status.success());
    let help = String::from_utf8_lossy(&out.stdout);
    for cmd in ["build-assets", "emit"] {
        assert!(
            help_has_subcommand(&help, cmd),
            "deck-bake --help missing `{cmd}`:\n{help}"
        );
    }
    for cmd in ["create", "dev", "build", "routes"] {
        assert!(
            !help_has_subcommand(&help, cmd),
            "deck-bake --help must not list product verb `{cmd}`:\n{help}"
        );
    }
}

#[test]
fn sail_rejects_bake_verbs() {
    for cmd in ["build-assets", "emit", "routes"] {
        let out = run_sail(&[cmd]);
        assert!(
            !out.status.success(),
            "product sail must reject `{cmd}` (status {:?})",
            out.status
        );
    }
}

#[test]
fn tokens_without_project_or_bake_flags_refuses() {
    let tmp = TempDir::new().expect("tempdir");
    let out = Command::new(sail_bin())
        .args(["tokens"])
        .current_dir(tmp.path())
        .output()
        .expect("spawn sail tokens");
    assert!(
        !out.status.success(),
        "sail tokens must refuse without Project or --model/--out"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("pass --model/--out (bake)"),
        "expected bake-flag refuse, got: {stderr}"
    );
}
