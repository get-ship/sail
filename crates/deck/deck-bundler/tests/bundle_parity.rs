//! G3 — bundle-parity regression gate.
//!
//! Spec: `.artifacts/sail-arc/integrity/report.md` §4 (gate G3) + the
//! bundling-robustness audit. This gate proves two things the default test
//! sweep does not, on a pure-transformation surface (no daemon boot, no
//! browser, no network):
//!
//! ## Part 1 — ALL-TARGETS build with a REAL exit check
//!
//! Enumerate `BundleLayout::all_targets()` — the static framework singletons
//! (`sail-runtime`, `sail-dom`, the umbrella, the heavy lazy packs) PLUS every
//! discovered per-component island AND every discovered view bundle —
//! and build each one through the deck Rolldown path. Every target must return
//! `Ok`.
//!
//! The honest-exit point is the crux (a #255-class dishonest-success hole):
//! `Bundler::build(&name)` propagates a failed bundle as `Err(BundlerError)`,
//! whereas `Bundler::build_all()` logs `error!` per failed target and returns
//! `Ok(())` regardless — a target that compiles to nothing or errors is
//! INVISIBLE through `build_all`. So this gate drives the SAME Rolldown path
//! `Bundler::build` funnels into (`build_single_flight` → `bundle_target`) and
//! asserts the `Result` per target. Artifact presence is necessary-not-
//! sufficient; the propagated exit is the gate.
//!
//! We call `bundle_target` directly (not `Bundler::build`) for ONE reason: it
//! takes a `BundleLayout`, so we can redirect output under a scratch
//! `out_root` (`with_out_root`) and never clobber the committed
//! `crates/deck/deck-core/assets`, `.deck/`, or `apps/shipd/assets` trees.
//! `Bundler::build` hardcodes `out_root = repo_root` (no redirect seam) and
//! would write into the source tree. `bundle_target` IS the function
//! `Bundler::build` ends up awaiting inside `build_single_flight`, with the
//! same `Result` propagation — so the exit semantics under test are identical,
//! and the test additionally asserts at the type level (below) that
//! `Bundler::build`'s signature returns `Result<_, BundlerError>` while
//! `build_all` returns `Result<(), _>` (the swallow). See FOUND BUG note.
//!
//! ## Part 2 — CROSS-PRODUCER parity (native vs wasm, #160)
//!
//! For each compiler fixture (`crates/sail/compiler/tests/fixtures/*.sui`),
//! compile via the native `sail_compiler::compile` (in-process, x86_64) AND via
//! the wasm `compile` export (the artifact `just sail-wasm` builds into
//! `packages/sail/compiler-wasm/pkg`, the exact surface the docs playground +
//! browser LSP ship). Assert the emitted `.js` is byte-identical (modulo the
//! build-stamp comment — see below). One Rust grammar compiled to two targets
//! must emit the same module; a divergence is the #160 native-vs-wasm class.
//!
//! The wasm `compile` is `#![cfg(target_arch = "wasm32")]`, so it is NOT
//! callable in this native test process. We drive it the same way
//! `parity.rs`/`parity_runner.mjs` drive their JS side: a sibling node script
//! (`g3_wasm_compile.mjs`) instantiates the local `.wasm` bytes
//! (`WebAssembly.instantiate`, no fetch/URL) and returns each fixture's `.js`.
//! The native side runs the SAME `CompileOpts` the wasm export builds
//! internally (`source_path: None`, `emit_dts/ssr: false`) so the `.js` body is
//! the only variable.
//!
//! ### Build-stamp normalization
//! Every emitted module carries a leading + export-attached
//! `_compilerVersion {…}` stamp (git sha + dirty + builtAt — `emit/js.rs:81`).
//! The committed `.wasm` was built at one commit; the in-tree native build is
//! at HEAD, so the STAMP legitimately differs while the compiled body must not.
//! We strip every `_compilerVersion {…}` object from both sides before the
//! byte-diff so the gate fails on real body divergence (#160) and tolerates the
//! unavoidable cross-build stamp difference. If the stamps were identical the
//! diff would be exact; normalizing only widens what we accept, never narrows.
//!
//! ## SEQUENTIAL + CAPPED (the safety note — #256)
//!
//! All-targets-at-once is the #256 memory shape: N concurrent oxc module graphs
//! live simultaneously. This gate builds targets STRICTLY SEQUENTIALLY (a plain
//! `for` loop awaiting one `bundle_target` at a time — never `join_all` /
//! spawn-per-target), so exactly ONE module graph is live at any moment.
//! `bundle_target` no longer calls malloc_trim(0) per build — cut 4
//! dropped crate::arena::trim_arenas() in favor of jemalloc (shipd's
//! global allocator, which returns freed arenas to the OS by default).
//! Each target build is wrapped in a per-target wall-clock
//! `tokio::time::timeout`; a target that wedges aborts the gate rather
//! than hanging. All scratch output + staging dirs are `TempDir`s
//! dropped at end of scope (teardown on panic/early-return), so
//! nothing leaks into the source tree or `/tmp`.
//!
//! ## Gating
//!
//! The whole file is behind `#![cfg(feature = "g3")]`. With the feature off
//! (the default for `cargo test -p deck-bundler` / `just build`) it compiles to
//! nothing and runs nothing. Invoke explicitly:
//!
//! ```text
//! cargo test -p deck-bundler --features g3 --test bundle_parity -- --nocapture
//! ```
//!
//! Prerequisites (the gate skips with a loud message, never silently passes, if
//! a tool is absent so a misconfigured CI cannot masquerade as green):
//!   * `sail-compiler` built under `target/` (the bundler's Rolldown `load` hook
//!     spawns it) — `cargo build -p sail-compiler`.
//!   * `packages/sail/compiler-wasm/pkg/sail_compiler_bg.wasm` present —
//!     `just sail-wasm`.
//!   * `node` on PATH (drives the wasm side).

#![cfg(feature = "g3")]
// FILE_CAP_OVERRIDE: single self-contained parity gate. The all-targets
// sequential build (with its per-target timeout + temp teardown safety), the
// build/build_all exit-honesty assertion, and the cross-producer (native vs
// wasm) byte-diff are one auditable unit — splitting them would scatter the
// "is this gate honest + safe?" review across files. ~45% of the length is the
// load-bearing doc comment documenting the exit-check, the stamp-normalization
// rationale, and the sequential+capped safety story.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use deck_bundler::rolldown_bundle::bundle_target;
use deck_bundler::targets::BundleLayout;
use deck_config::DeckConfig;
use serde_json::json;
use sail_compiler::{compile, CompileMode, CompileOpts, ComponentMeta};

// ── Safety constants (the founder OOM'd this box twice; a runaway here is
// unacceptable — see #256) ───────────────────────────────────────────────────

/// Per-target wall-clock cap. A single Rolldown build of even the heaviest pack
/// (arcade/pdf, ~800 KB) finishes in a few seconds on a cold cache; 180s is a
/// generous ceiling that still guarantees a wedged target aborts the gate
/// rather than hanging CI. Applied PER TARGET (not to the whole sweep) so the
/// bound is tight on each one-graph-at-a-time build.
const PER_TARGET_TIMEOUT: Duration = Duration::from_secs(180);

/// Wall-clock cap for one node invocation of the wasm side (instantiate +
/// compile the whole fixture corpus once). Bounded so a hung node can't stall
/// the gate.
const WASM_RUNNER_TIMEOUT: Duration = Duration::from_secs(120);

// ── Repo discovery ───────────────────────────────────────────────────────────

fn repo_root() -> PathBuf {
    let mut d = Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();
    loop {
        if d.join("package.json").exists() && d.join("packages").is_dir() {
            return d;
        }
        if !d.pop() {
            panic!("repo root not found");
        }
    }
}

fn fixtures_dir(repo: &Path) -> PathBuf {
    repo.join("crates/sail/compiler/tests/fixtures")
}

fn wasm_pkg_dir(repo: &Path) -> PathBuf {
    repo.join("packages/sail/compiler-wasm/pkg")
}

// ── Part 1: all-targets sequential build with real exit check ────────────────

#[tokio::test(flavor = "multi_thread")]
async fn gate_bundle_parity_all_targets_build() {
    let repo = repo_root();
    // The compiler is the linked crate — compiled in-process (#255), no binary
    // prereq. Scratch output + staging so we never clobber the committed trees.
    // Dropped at end of scope (teardown on panic/early-return).
    let scratch = tempfile::tempdir().expect("scratch out_root");
    let staging = tempfile::tempdir().expect("staging dir");

    let layout = BundleLayout::with_config(repo.clone(), None, DeckConfig::defaults()).with_out_root(scratch.path().to_path_buf());

    // The full target set: static framework singletons + discovered consumer
    // bundles (per-package `deck.bundles`) + discovered view bundles. This is
    // the EXACT set `Bundler::build_all` would iterate.
    let targets = layout.all_targets();
    assert!(
        !targets.is_empty(),
        "all_targets() returned nothing — discovery is broken"
    );

    let mut built = Vec::new();
    let mut failures = Vec::new();

    // STRICTLY SEQUENTIAL: one `bundle_target` awaited at a time. Exactly one
    // oxc module graph is ever live; `bundle_target` trims arenas after each.
    // NEVER join_all / spawn-per-target (that is the #256 memory shape).
    for t in targets.iter() {
        let fut = bundle_target(&layout, t, staging.path(), false);
        match tokio::time::timeout(PER_TARGET_TIMEOUT, fut).await {
            Err(_elapsed) => {
                failures.push(format!(
                    "{}: TIMEOUT after {}s (wedged build)",
                    t.name,
                    PER_TARGET_TIMEOUT.as_secs()
                ));
            }
            // The REAL exit check: `bundle_target` propagates a failed bundle as
            // Err — the same Err `Bundler::build` re-wraps as `BundlerError`.
            // `Bundler::build_all` would have swallowed this (logged + Ok). We do
            // not.
            Ok(Err(e)) => {
                failures.push(format!("{}: {e}", t.name));
            }
            Ok(Ok(manifest)) => {
                // Artifact presence is necessary-not-sufficient, but a target
                // that "succeeds" yet wrote zero bytes is the compile-to-nothing
                // case the exit check exists to catch alongside the Err path.
                assert!(
                    manifest.bytes > 0,
                    "{} reported Ok but emitted an empty artifact at {}",
                    t.name,
                    manifest.out_path
                );
                built.push(t.name.clone());
            }
        }
    }

    eprintln!(
        "G3 all-targets: {} built OK, {} failed",
        built.len(),
        failures.len()
    );
    for name in &built {
        eprintln!("  ok  {name}");
    }
    for f in &failures {
        eprintln!("  ERR {f}");
    }

    assert!(
        failures.is_empty(),
        "G3: {} target(s) failed to build through the deck Rolldown path \
         (each propagated Err — `build_all` would have swallowed these):\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// Compile-time + behavioral proof of the build vs build_all exit asymmetry —
/// the #255-class dishonest-success hole this gate guards against. `build`
/// hands back a `BundlerError` on failure (the caller can react); `build_all`
/// erases per-target failure into a bare `Ok(())`. We bind both return types so
/// a refactor that changes either signature breaks here, and document why the
/// gate deliberately drives `build`'s path, not `build_all`'s.
#[test]
fn build_exit_is_honest_unlike_build_all() {
    // These are type-level assertions: `build` -> Result<BundleManifest,
    // BundlerError>; `build_all` -> Result<(), BundlerError>. The `()` success
    // type is the swallow: build_all cannot tell the caller WHICH (or whether
    // any) target failed — it logs `error!` and returns Ok. If a future change
    // made `build_all` return `Result<Vec<Result<_, _>>>` or similar, this
    // binding would fail to compile, flagging that the gate's rationale moved.
    fn _assert_build_sig(
        b: &deck_bundler::Bundler,
    ) -> impl std::future::Future<Output = Result<deck_bundler::BundleManifest, deck_bundler::BundlerError>> + '_
    {
        b.build("noop")
    }
    fn _assert_build_all_sig(
        b: &deck_bundler::Bundler,
    ) -> impl std::future::Future<Output = Result<(), deck_bundler::BundlerError>> + '_ {
        b.build_all()
    }
    let _ = _assert_build_sig;
    let _ = _assert_build_all_sig;
}

// ── Part 2: cross-producer parity (native vs wasm) ───────────────────────────

/// A fixture + the `CompileOpts` driven IDENTICALLY into the native and wasm
/// producers. `setup_url` / `component_name` affect the emitted
/// `.js`, so both sides must receive the same values — that is the whole point.
struct Fixture {
    name: &'static str,
    mode: CompileMode,
    setup_url: Option<&'static str>,
    component_name: Option<&'static str>,
}

/// The fixture corpus. Views, components, and the
/// control-flow / binding edge cases. Components carry a `component_name` so the
/// component emit path is exercised on both producers, not just views.
fn fixtures() -> Vec<Fixture> {
    use CompileMode::{Component, View};
    vec![
        Fixture { name: "hello", mode: View, setup_url: None, component_name: None },
        Fixture { name: "counter", mode: View, setup_url: Some("./counter.controller.ts"), component_name: None },
        Fixture { name: "agents", mode: View, setup_url: Some("./agents.controller.ts"), component_name: None },
        Fixture { name: "comment-and-whitespace", mode: View, setup_url: None, component_name: None },
        Fixture { name: "for-static", mode: View, setup_url: None, component_name: None },
        Fixture { name: "for-static-text", mode: View, setup_url: None, component_name: None },
        Fixture { name: "for-static-attr", mode: View, setup_url: None, component_name: None },
        Fixture { name: "for-dynamic-mixed", mode: View, setup_url: None, component_name: None },
        Fixture { name: "for-update", mode: View, setup_url: None, component_name: None },
        Fixture { name: "route-view", mode: View, setup_url: None, component_name: None },
        Fixture { name: "ship-button", mode: Component, setup_url: None, component_name: Some("ship-button-test") },
        Fixture { name: "reflect-empty-slot", mode: Component, setup_url: None, component_name: Some("reflect-empty-slot") },
        Fixture { name: "shadow-tones", mode: Component, setup_url: None, component_name: Some("shadow-tones") },
        Fixture { name: "bind-this-button", mode: Component, setup_url: None, component_name: Some("bind-this-button") },
    ]
}

/// Strip every `_compilerVersion …{…}` JSON object so the cross-build stamp
/// (git sha / dirty / builtAt) does not masquerade as a body divergence. The
/// stamp appears in TWO syntactic forms (`emit/js.rs`):
///   * leading legal comment: `/*! _compilerVersion {…} */`
///   * export-attached property: `_compilerVersion: { … }`
/// Both start with the literal `_compilerVersion` followed (after an optional
/// `:` and whitespace) by a single brace-balanced JSON object. We drop from the
/// marker through its matching close brace in both forms.
fn strip_version_stamp(js: &str) -> String {
    const MARKER: &str = "_compilerVersion";
    let mut out = String::with_capacity(js.len());
    let mut rest = js;
    while let Some(idx) = rest.find(MARKER) {
        out.push_str(&rest[..idx]);
        let after = &rest[idx + MARKER.len()..];
        // After the marker: an optional `:` (export form) then whitespace then
        // the opening brace of the JSON object. Skip the brace-balanced object.
        if let Some(stripped) = skip_balanced_object(after) {
            rest = stripped;
        } else {
            // Defensive: no object follows — keep the marker text and move on so
            // we never loop forever.
            out.push_str(MARKER);
            rest = after;
        }
    }
    out.push_str(rest);
    out
}

/// Given a slice that begins at an optional `:` then optional whitespace then a
/// `{`, return the slice AFTER the brace-balanced object, or `None` if no `{`
/// opens it.
fn skip_balanced_object(s: &str) -> Option<&str> {
    let bytes = s.as_bytes();
    let mut i = 0;
    // Optional `:` separator (the export-attached `_compilerVersion: {…}` form).
    if i < bytes.len() && bytes[i] == b':' {
        i += 1;
    }
    while i < bytes.len() && (bytes[i] as char).is_whitespace() {
        i += 1;
    }
    if i >= bytes.len() || bytes[i] != b'{' {
        return None;
    }
    let mut depth = 0i32;
    let mut in_str = false;
    let mut escaped = false;
    while i < bytes.len() {
        let c = bytes[i];
        if in_str {
            if escaped {
                escaped = false;
            } else if c == b'\\' {
                escaped = true;
            } else if c == b'"' {
                in_str = false;
            }
        } else {
            match c {
                b'"' => in_str = true,
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(&s[i + 1..]);
                    }
                }
                _ => {}
            }
        }
        i += 1;
    }
    None
}

/// Native producer: compile in-process with the SAME opts the wasm export uses
/// internally (`source_path: None`, `emit_dts/ssr: false`). Returns
/// `Ok(js)` / `Err(message)` so we can assert kind-parity with the wasm side.
fn native_compile(repo: &Path, f: &Fixture) -> Result<String, String> {
    let path = fixtures_dir(repo).join(format!("{}.sui", f.name));
    let source = std::fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", f.name))?;
    let component_meta = f.component_name.map(|name| ComponentMeta {
        name: name.to_string(),
    });
    let opts = CompileOpts {
        route_params: None,
        setup_url: f.setup_url.map(String::from),
        mode: f.mode,
        // Matches the wasm export's `CompileOpts` exactly (wasm.rs).
        source_path: None,
        component_meta,
        emit_dts: false,
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    };
    compile(&source, opts)
        .map(|out| out.js)
        .map_err(|e| e.message)
}

/// One result row from the node/wasm runner.
#[derive(serde::Deserialize)]
struct WasmRow {
    name: String,
    kind: String,
    #[serde(default)]
    js: Option<String>,
    #[serde(default)]
    message: Option<String>,
}

#[derive(serde::Deserialize)]
struct WasmRunnerOut {
    ok: bool,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    results: Vec<WasmRow>,
}

/// Drive the wasm `compile` export over the whole corpus via node. Identical
/// opts to `native_compile` — the wasm `WasmOpts` JSON-in shape mirrors them.
fn wasm_compile_all(repo: &Path, fixtures: &[Fixture]) -> Vec<WasmRow> {
    let pkg = wasm_pkg_dir(repo);
    if !pkg.join("sail_compiler_bg.wasm").exists() {
        panic!(
            "G3 PREREQ MISSING: wasm artifact not built at {}. Run `just sail-wasm`.",
            pkg.display()
        );
    }
    let runner = repo.join("crates/deck/deck-bundler/tests/g3_wasm_compile.mjs");
    assert!(runner.exists(), "g3_wasm_compile.mjs missing at {}", runner.display());

    let jobs: Vec<_> = fixtures
        .iter()
        .map(|f| {
            let path = fixtures_dir(repo).join(format!("{}.sui", f.name));
            let source = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("read {} for wasm side: {e}", f.name));
            let mode = match f.mode {
                CompileMode::View => "view",
                CompileMode::Component => "component",
                CompileMode::Auto => "auto",
            };
            json!({
                "name": f.name,
                "source": source,
                "opts": {
                    "mode": mode,
                    "componentName": f.component_name,
                    "setupUrl": f.setup_url,
                },
            })
        })
        .collect();
    let input = json!({ "pkgDir": pkg.to_string_lossy(), "jobs": jobs }).to_string();

    // Spawn node, feed the corpus on stdin, read JSON on stdout, with a bounded
    // wall-clock wait so a hung node aborts the gate.
    let mut child = Command::new("node")
        .arg(&runner)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap_or_else(|e| panic!("G3 PREREQ MISSING: cannot spawn `node` ({e}). Install node."));

    use std::io::Write;
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(input.as_bytes())
        .expect("write jobs to node stdin");

    let out = wait_with_timeout(child, WASM_RUNNER_TIMEOUT);
    let stdout = String::from_utf8_lossy(&out);
    let parsed: WasmRunnerOut = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("wasm runner emitted unparseable stdout ({e}):\n{stdout}"));
    assert!(
        parsed.ok,
        "wasm runner failed at harness level: {}",
        parsed.error.unwrap_or_default()
    );
    parsed.results
}

/// Wait for `child` to finish, killing it (and failing the gate) if it exceeds
/// `timeout`. Returns captured stdout bytes.
fn wait_with_timeout(mut child: std::process::Child, timeout: Duration) -> Vec<u8> {
    use std::io::Read;
    let start = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let mut buf = Vec::new();
                if let Some(mut so) = child.stdout.take() {
                    let _ = so.read_to_end(&mut buf);
                }
                assert!(status.success(), "wasm runner exited non-zero: {status:?}");
                return buf;
            }
            Ok(None) => {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    panic!("wasm runner TIMEOUT after {}s", timeout.as_secs());
                }
                std::thread::sleep(Duration::from_millis(25));
            }
            Err(e) => panic!("wasm runner wait failed: {e}"),
        }
    }
}

#[test]
fn gate_bundle_parity_native_vs_wasm() {
    let repo = repo_root();
    let fixtures = fixtures();

    let wasm_rows = wasm_compile_all(&repo, &fixtures);
    assert_eq!(
        wasm_rows.len(),
        fixtures.len(),
        "wasm runner returned a different number of rows than fixtures"
    );

    let mut diffs = Vec::new();
    let mut checked = 0usize;
    for f in &fixtures {
        let row = wasm_rows
            .iter()
            .find(|r| r.name == f.name)
            .unwrap_or_else(|| panic!("wasm side missing result for fixture {}", f.name));
        let native = native_compile(&repo, f);

        match (&native, row.kind.as_str()) {
            // Both succeeded — the `.js` body must be byte-identical after
            // stamp normalization (the #160 check).
            (Ok(njs), "ok") => {
                let wjs = row.js.as_deref().unwrap_or_default();
                let n = strip_version_stamp(njs);
                let w = strip_version_stamp(wjs);
                if n != w {
                    diffs.push(format!(
                        "{}: native vs wasm .js DIVERGED\n--- native (len {}) ---\n{}\n--- wasm (len {}) ---\n{}",
                        f.name,
                        n.len(),
                        head(&n),
                        w.len(),
                        head(&w),
                    ));
                }
                checked += 1;
            }
            // Both rejected the source — kind parity holds; we don't require
            // identical error text (spans/formatting may differ across the two
            // builds), only that neither producer silently accepts what the
            // other rejects.
            (Err(_), "err") | (Err(_), "throw") => {
                checked += 1;
            }
            // KIND DIVERGENCE — one producer compiled, the other didn't. This is
            // exactly the dangerous #160 split: a `.sui` one toolchain accepts
            // and the other rejects.
            (Ok(_), other) => diffs.push(format!(
                "{}: native OK but wasm produced kind={other:?} (msg: {:?}) — producers disagree on acceptance",
                f.name, row.message
            )),
            (Err(msg), "ok") => diffs.push(format!(
                "{}: wasm OK but native rejected ({msg}) — producers disagree on acceptance",
                f.name
            )),
            (Err(_), other) => diffs.push(format!(
                "{}: unexpected wasm kind={other:?}",
                f.name
            )),
        }
    }

    eprintln!("G3 cross-producer parity: {checked}/{} fixtures agreed", fixtures.len());
    assert!(
        diffs.is_empty(),
        "G3: {} fixture(s) diverged between native and wasm producers (#160):\n\n{}",
        diffs.len(),
        diffs.join("\n\n")
    );
    assert_eq!(
        checked,
        fixtures.len(),
        "not every fixture was compared — coverage hole"
    );
}

/// First ~600 chars of a string, for readable diff output.
fn head(s: &str) -> String {
    if s.len() <= 600 {
        s.to_string()
    } else {
        format!("{}…(+{} more)", &s[..600], s.len() - 600)
    }
}
