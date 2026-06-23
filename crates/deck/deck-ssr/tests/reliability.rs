//! Slice-1 reliability sweep: run a representative, feature-diverse set of
//! REAL views + components through the synthetic emit → Rolldown bundle →
//! ship-js (deno_core/V8) SSR, and catalog which render correctly + reliably.
//!
//! This is the ship-js (SSR) half of the slice's reliability proof. The CSR /
//! hydration half (the client `__mount` + `hydrate` contract, jsdom) lives in
//! `crates/sui/compiler/tests/synthetic_client_jsdom.rs`; the tsgo-green half
//! in `synthetic_client_tsc.rs`. Together they answer the load-bearing
//! question: do real-world surfaces render reliably through the JS engine?
//!
//! Honest accounting: every target is rendered; a failure records the precise
//! gap (the input that scopes the full sweep), never a silent exclusion. The
//! pass/fail table is written to `.artifacts/synthetic-slice1-ssr-table.md`.
//!
//! Heavy (V8 + a Rolldown bundle per target). Opt-in via the normal test gate;
//! run with `cargo test -p deck-ssr --test reliability -- --nocapture`.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use deck_ssr::build::{repo_root, synthesize_and_bundle_with, ClientAlias};
use deck_ssr::host::SsrDispatch;
use deck_ssr::run::render;
use deck_core::DataPort;
use serde_json::{json, Value};
use sui_compiler::emit::synthetic::{emit_synthetic, AppManifest};
use sui_compiler::parse::parse;
use sui_compiler::CompileMode;

const CONF: &str = "crates/deck/deck-conformance/app";
// `job/cut-app-manifest` deleted the per-app `app.manifest.jsonc` files for
// the `hub /` and `views/showcase` targets — `route` / `navigate` now resolve
// by convention (`sui_compiler::emit::synthetic::convention_client`) so
// product apps no longer need one. The conformance manifest is the one
// remaining checked-in manifest; it covers the `hub /` and `views/showcase`
// cases (its `ship` / `api` declarations are tree-shaken when unused, and
// the convention covers the `route` reads the two views carry).
const CONF_MANIFEST: &str = "crates/deck/deck-conformance/app/app.manifest.jsonc";
const HUB_MANIFEST: &str = "crates/deck/deck-conformance/app/app.manifest.jsonc";
const VIEWS_MANIFEST: &str = "crates/deck/deck-conformance/app/app.manifest.jsonc";
const DEMO_SERVER: &str = "crates/deck/deck-conformance/app/clients/demo.server.ts";

/// One reliability target.
struct Target {
    /// Display name in the table.
    name: &'static str,
    /// Feature surface this target exercises (for the table).
    features: &'static str,
    /// `.sui` source, repo-relative.
    source: String,
    /// App manifest, repo-relative.
    manifest: &'static str,
    /// Sibling authored controller, repo-relative (when present).
    controller: Option<String>,
    /// Does it read `<data>`/`<client>` ops (needs the demo client alias)?
    needs_data: bool,
    /// Request pathname + route params (for `[id]` routes).
    pathname: &'static str,
    params: Value,
    /// A substring the first-paint HTML MUST contain to count as "renders
    /// correctly" (the fidelity check). Empty → "rendered without error".
    expect: &'static str,
}

/// Per-target outcome.
struct Outcome {
    name: String,
    features: String,
    bundled: bool,
    rendered: bool,
    correct: bool,
    bytes: usize,
    note: String,
}

/// The demo data surface: answers the ops the conformance data-views call.
/// `ship.demo.boom` is deliberately absent — boom's `<data>` thunk must throw
/// (the error-boundary fixture), proving the error branch renders.
struct DemoData;
impl DataPort for DemoData {
    fn resolve(&self, domain: &str, op: &str, _args: &Value) -> Option<Value> {
        match (domain, op) {
            ("ship", "demo.todos") => Some(json!({
                "items": [
                    { "id": "1", "text": "ship it", "done": false },
                    { "id": "2", "text": "prove it", "done": true }
                ]
            })),
            ("ship", "demo.config") => Some(json!({ "title": "Demo" })),
            ("api", "demo.clock") => Some(json!({ "now": "12:34:56" })),
            _ => None,
        }
    }
}

fn conf(view: &str) -> String {
    format!("{CONF}/views/{view}/page.sui")
}
fn conf_ctrl(view: &str) -> Option<String> {
    Some(format!("{CONF}/views/{view}/controller.ts"))
}

fn targets() -> Vec<Target> {
    let v = |name, features, view: &str, ctrl: Option<String>, needs_data, expect| Target {
        name,
        features,
        source: conf(view),
        manifest: CONF_MANIFEST,
        controller: ctrl,
        needs_data,
        pathname: "/",
        params: json!({}),
        expect,
    };
    vec![
        // ── Launch-relevant: hub / and a real view-host view ───────────────
        Target {
            name: "hub /",
            features: "text-binding, slots, custom-element chrome, authored controller",
            source: "apps/hub/routes/page.sui".to_string(),
            manifest: HUB_MANIFEST,
            controller: Some("apps/hub/routes/controller.ts".to_string()),
            needs_data: false,
            pathname: "/",
            params: json!({}),
            // The hub home was redesigned (greeting + workspace layout) — the
            // static `home-sub` copy is the stable fidelity marker now.
            expect: "Your Ship workspace",
        },
        Target {
            name: "views/showcase",
            features: "static gallery, attribute props, custom-element charts, nesting",
            source: "views/showcase/page.sui".to_string(),
            manifest: VIEWS_MANIFEST,
            controller: None,
            needs_data: false,
            pathname: "/showcase",
            params: json!({}),
            expect: "sui-area-chart",
        },
        // ── Control flow + bindings (conformance views) ────────────────────
        v("board", "{#if}, {#for} keyed, class:, on:, nested component", "board", conf_ctrl("board"), false, "Alpha"),
        v("counter", "on:, signal reactivity", "counter", conf_ctrl("counter"), false, "<button"),
        v("todos", "{#for} keyed, {#if}, {#while}, class:, <data live>, form mutation", "todos", None, true, "ship it"),
        v("feed", "{#while}, <data live>, <client>", "feed", None, true, "state"),
        v("boom", "{#if}, {#while}, <data live> error branch", "boom", None, true, ""),
        v("clock", "{#if}, {#while}, <data live>", "clock", None, true, ""),
        v("glyph", "inline SVG, namespaced serialization", "glyph", None, false, "<svg"),
        Target {
            name: "greet/[id]",
            features: "dynamic route params (route.params.id)",
            source: conf("greet/[id]"),
            manifest: CONF_MANIFEST,
            controller: None,
            needs_data: false,
            pathname: "/greet/zelda",
            params: json!({ "id": "zelda" }),
            expect: "zelda",
        },
        v("hello", "constant folding, static markup", "hello", None, false, ""),
        v("widgets", "repeated nested component (shared-chunk)", "widgets", None, false, "demo-chip"),
        v("gallery", "on:, class:, authored controller, nested", "gallery", conf_ctrl("gallery"), false, ""),
        // ── Component-gallery views (render real component tags) ────────────
        v("gallery-sui-primitives", "{#if}, bind:, on:, style:, transitions, nested", "gallery-sui-primitives", conf_ctrl("gallery-sui-primitives"), false, ""),
        v("gallery-sui-charts", "static chart components", "gallery-sui-charts", None, false, ""),
        v("gallery-sui-media", "static media components", "gallery-sui-media", None, false, ""),
        v("gallery-sui-viewer", "static viewer component", "gallery-sui-viewer", None, false, ""),
        v("gallery-ship-markdown", "static markdown component", "gallery-ship-markdown", None, false, ""),
        v("gallery-ship-pdf", "style:, transitions", "gallery-ship-pdf", None, false, ""),
        v("gallery-ship-arcade", "on:, transitions", "gallery-ship-arcade", None, false, ""),
        v("gallery-ship-code", "on:, code viewer component", "gallery-ship-code", None, false, ""),
    ]
}

fn run_target(repo: &Path, t: &Target) -> Outcome {
    let mut o = Outcome {
        name: t.name.to_string(),
        features: t.features.to_string(),
        bundled: false,
        rendered: false,
        correct: false,
        bytes: 0,
        note: String::new(),
    };

    let build_dir = repo.join("target/deck-ssr-reliability").join(sanitize(t.name));
    // Always alias the demo client. A view-host always has its clients
    // available; aliasing an UNused client is harmless (tree-shaken). This
    // also absorbs the synthetic scan's string-literal false positive —
    // `class:active={x === 'ship-light'}` trips the free-identifier scan on
    // `ship`, importing the client spuriously; with the alias present that
    // import resolves instead of denying the bundle. (The scan over-
    // approximation itself is a catalogued gap — see the assessment notes.)
    let aliases: Vec<ClientAlias> = vec![ClientAlias {
        specifier: "@app/clients/demo".to_string(),
        server_entry: repo.join(DEMO_SERVER),
    }];
    let ctrl_abs = t.controller.as_ref().map(|c| repo.join(c));

    // Synthesize + bundle (panics → recorded as a bundle failure, not abort).
    let bundle = catch_unwind(AssertUnwindSafe(|| {
        let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
        rt.block_on(synthesize_and_bundle_with(
            repo,
            &t.source,
            t.manifest,
            CompileMode::View,
            ctrl_abs.as_deref(),
            &aliases,
            &build_dir,
        ))
    }));
    let bundle = match bundle {
        Ok(Ok(b)) => b,
        Ok(Err(e)) => {
            o.note = format!("synth/bundle: {}", first_line(&format!("{e:#}")));
            return o;
        }
        Err(_) => {
            o.note = "synth/bundle PANICKED".to_string();
            return o;
        }
    };
    o.bundled = true;
    o.bytes = bundle.len();

    // Render through ship-js (V8).
    let mut dispatch = SsrDispatch::for_request(t.pathname, t.params.clone(), json!({}));
    if t.needs_data {
        dispatch = dispatch.with_data(Arc::new(DemoData));
    }
    let dispatch = Arc::new(dispatch);
    let rendered = catch_unwind(AssertUnwindSafe(|| render(&bundle, dispatch)));
    let out = match rendered {
        Ok(Ok(out)) => out,
        Ok(Err(e)) => {
            o.note = format!("ssr: {}", first_line(&format!("{e:#}")));
            return o;
        }
        Err(_) => {
            o.note = "ssr render PANICKED".to_string();
            return o;
        }
    };
    o.rendered = true;

    // Fidelity: HTML present + the expected substring (when one is set).
    if out.html.trim().is_empty() {
        o.note = "rendered empty HTML".to_string();
        return o;
    }
    if !t.expect.is_empty() && !out.html.contains(t.expect) {
        o.note = format!("missing expected `{}` in first paint", t.expect);
        return o;
    }
    o.correct = true;
    o.note = format!("{} bytes html", out.html.len());
    o
}

#[test]
fn slice1_reliability_sweep() {
    let repo = repo_root().expect("repo root");
    let targets = targets();
    let total = targets.len();
    let mut outcomes: Vec<Outcome> = Vec::with_capacity(total);
    for t in &targets {
        let o = run_target(&repo, t);
        eprintln!(
            "[{}] bundle={} render={} correct={} — {}",
            o.name, o.bundled, o.rendered, o.correct, o.note
        );
        outcomes.push(o);
    }

    let correct = outcomes.iter().filter(|o| o.correct).count();
    let table = render_table(&outcomes, correct, total);
    let out_path = repo.join(".artifacts/synthetic-slice1-ssr-table.md");
    let _ = std::fs::create_dir_all(out_path.parent().unwrap());
    std::fs::write(&out_path, &table).expect("write table");
    eprintln!("\n{table}\nwrote {}", out_path.display());

    // The receipt is the TABLE, not a green/red gate: this slice catalogs
    // reliability, it does not assert universality. We require only that the
    // launch-relevant core (hub /, showcase, board, counter, glyph,
    // greet/[id]) renders correctly — a regression in those is a real break.
    let core = ["hub /", "views/showcase", "board", "counter", "glyph", "greet/[id]"];
    for name in core {
        let o = outcomes.iter().find(|o| o.name == name).expect("core target present");
        assert!(o.correct, "core target `{}` must render correctly: {}", name, o.note);
    }
}

#[test]
fn slice1_warm_ssr_latency() {
    // Warm ship-js SSR latency: bundle ONCE (cached), then render N times and
    // take the median of the warm runs (drop the first, cold-isolate run).
    // Each render() is a fresh current-thread isolate evaluating the cached
    // bundle — the serve path pools isolates, so this is an upper bound on the
    // pooled warm latency, not the floor.
    let repo = repo_root().expect("repo root");
    let samples: Vec<(&str, String, &str, Option<String>, &str)> = vec![
        ("hub /", "apps/hub/routes/page.sui".to_string(), HUB_MANIFEST, Some("apps/hub/routes/controller.ts".to_string()), "/"),
        ("board", conf("board"), CONF_MANIFEST, Some(format!("{CONF}/views/board/controller.ts")), "/"),
        ("glyph", conf("glyph"), CONF_MANIFEST, None, "/"),
    ];
    const N: usize = 9;
    let mut lines = vec!["## Warm ship-js SSR latency (median of warm runs)\n".to_string(), format!("N = {N} renders per target (first dropped as cold), fresh isolate per render.\n")];
    lines.push("| target | warm median | min | max |".to_string());
    lines.push("|---|---|---|---|".to_string());
    for (name, source, manifest, ctrl, pathname) in &samples {
        let build_dir = repo.join("target/deck-ssr-latency").join(sanitize(name));
        let ctrl_abs = ctrl.as_ref().map(|c| repo.join(c));
        let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
        let bundle = rt
            .block_on(synthesize_and_bundle_with(
                &repo, source, manifest, CompileMode::View, ctrl_abs.as_deref(), &[], &build_dir,
            ))
            .expect("bundle latency sample");
        let mut times = Vec::new();
        for i in 0..N {
            let dispatch = Arc::new(SsrDispatch::for_request(pathname, json!({}), json!({})));
            let t0 = Instant::now();
            let _ = render(&bundle, dispatch).expect("render latency sample");
            let dt = t0.elapsed();
            if i > 0 {
                times.push(dt);
            }
        }
        times.sort();
        let median = times[times.len() / 2];
        let min = *times.first().unwrap();
        let max = *times.last().unwrap();
        let line = format!(
            "| {} | {:.1} ms | {:.1} ms | {:.1} ms |",
            name,
            median.as_secs_f64() * 1e3,
            min.as_secs_f64() * 1e3,
            max.as_secs_f64() * 1e3,
        );
        eprintln!("{line}");
        lines.push(line);
    }
    let out = lines.join("\n");
    let out_path = repo.join(".artifacts/synthetic-slice1-latency.md");
    let _ = std::fs::create_dir_all(out_path.parent().unwrap());
    std::fs::write(&out_path, format!("{out}\n")).expect("write latency");
    eprintln!("\nwrote {}", out_path.display());
}

/// The hub `/` launch closure is built from `@ship/sui-primitives` chrome
/// components — and they are PROPFUL, which the synthetic emit rejects today
/// (gap G1: props flow in at mount; the standalone synthetic controller can't
/// model them). This catalogs that concretely on the ACTUAL closure, so the
/// dominant sweep gap has honest, measurable counts rather than prose. The
/// view hub `/` still renders (its chrome SSRs as custom-element tags); it is
/// the components THEMSELVES that the synthetic emit can't yet synthesize.
#[test]
fn slice1_hub_closure_component_catalog() {
    let repo = repo_root().expect("repo root");
    let manifest = AppManifest::parse("{}").expect("empty manifest");
    let closure = [
        "packages/components/sui/primitives/src/sui-nav-top/sui-nav-top.sui",
        "packages/components/sui/primitives/src/sui-brand-mark/sui-brand-mark.sui",
        "packages/components/sui/primitives/src/sui-theme-picker/sui-theme-picker.sui",
        "packages/components/sui/primitives/src/sui-account-chip/sui-account-chip.sui",
        "packages/components/sui/primitives/src/sui-icon/sui-icon.sui",
        "packages/components/sui/primitives/src/sui-popover/sui-popover.sui",
    ];
    let mut rows = vec![
        "# Slice 1 — hub `/` component closure (synthetic emit)\n".to_string(),
        "The launch chrome is propful `@ship/sui-primitives`; the synthetic emit rejects \
         propful components today (gap G1). Honest count:\n"
            .to_string(),
        "| component | props | synthesizes | reason |".to_string(),
        "|---|--:|:--:|---|".to_string(),
    ];
    let mut rejected_propful = 0;
    for rel in &closure {
        let src = std::fs::read_to_string(repo.join(rel)).expect("read component");
        let props = src.matches("<prop").count();
        let name = Path::new(rel).file_stem().and_then(|s| s.to_str()).unwrap_or(rel);
        let (ok, reason) = match parse(&src)
            .map_err(|e| e.render(&src))
            .and_then(|p| {
                emit_synthetic(&p, &manifest, None, "./c.ts", CompileMode::Component, None)
                    .map_err(|e| e.render(&src))
            }) {
            Ok(_) => (true, "synthesizes (propless)".to_string()),
            Err(e) => {
                let first = e.lines().next().unwrap_or("").to_string();
                if first.contains("prop") {
                    rejected_propful += 1;
                    (false, "rejected: propful (gap G1)".to_string())
                } else {
                    (false, format!("rejected: {}", first.chars().take(80).collect::<String>()))
                }
            }
        };
        rows.push(format!("| {name} | {props} | {} | {reason} |", yn(ok)));
    }
    rows.push(format!(
        "\n**{rejected_propful} of {} hub-closure chrome components are blocked by gap G1 \
         (propful components unsupported).** This is the dominant Slice-2 item.",
        closure.len()
    ));
    let out = rows.join("\n");
    let out_path = repo.join(".artifacts/synthetic-slice1-hub-closure.md");
    let _ = std::fs::create_dir_all(out_path.parent().unwrap());
    std::fs::write(&out_path, format!("{out}\n")).expect("write closure table");
    eprintln!("\n{out}\nwrote {}", out_path.display());

    // The finding, regression-guarded: the propful launch chrome is uniformly
    // rejected with the props gap (G1) — not some other error, not silently.
    assert!(
        rejected_propful >= 5,
        "expected the propful hub-closure chrome to be rejected by the props gap (G1); \
         only {rejected_propful} were"
    );
}

fn render_table(outcomes: &[Outcome], correct: usize, total: usize) -> String {
    let mut s = String::new();
    s.push_str("# Slice 1 — ship-js SSR reliability table\n\n");
    s.push_str(&format!(
        "**{correct} of {total} real targets render correctly through synthesize → bundle → ship-js (V8) SSR.**\n\n",
    ));
    s.push_str("| target | features | bundled | ssr | correct | note |\n");
    s.push_str("|---|---|:--:|:--:|:--:|---|\n");
    for o in outcomes {
        s.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} |\n",
            o.name,
            o.features,
            yn(o.bundled),
            yn(o.rendered),
            yn(o.correct),
            o.note.replace('|', "\\|"),
        ));
    }
    s
}

fn yn(b: bool) -> &'static str {
    if b { "✅" } else { "❌" }
}

fn first_line(s: &str) -> String {
    s.lines().next().unwrap_or("").chars().take(160).collect()
}

fn sanitize(s: &str) -> String {
    s.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect()
}
