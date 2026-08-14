//! THE SEAM-COHERENCE GATE for the `"network"` capability (fetch lever, item
//! 4). A `sail.config.json` declaring `{"network": [...]}` must drive BOTH the
//! compiler's static lint AND deck's served CSP `connect-src` header from the
//! SAME declaration — drift between the two is a test failure here, not a
//! discovery in production.
//!
//! Assertions derive from ONE Rust-level origin list (`GRANTED_ORIGIN`), used to:
//!   (a) build the fixture `sail.config.json` body that the real compiler
//!       resolver (`sail_compiler::check::policy::Policy`) reads, driving the
//!       `sail-controller-no-ambient-authority` lint through the real
//!       `compile()` build gate — same path `config_loud_strict.rs` exercises;
//!   (b) build the `connect-src` header via `deck_core::html_security_headers_for`
//!       — the same list, not two independently hardcoded origin strings;
//!   (c) serve a live SPA shell via `serve_scaffold` with that config on disk
//!       and assert the response CSP carries the granted origin (EXT-1 E6 wire).

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use deck_config::{AppServeConfig, RenderMode};
use deck_core::{DaemonMode, HTML_SECURITY_HEADERS, serve_scaffold};
use sail_compiler::check::policy::Policy;
use sail_compiler::{CompileMode, CompileOpts, compile};

/// The ONE declaration both halves of this test derive from.
const GRANTED_ORIGIN: &str = "https://api.example.com";
const EVIL_ORIGIN: &str = "https://evil.com";

fn write_fixture(fetch_target: &str) -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
    // Confined by default, path-independent — the ambient-authority rule
    // governs every inline controller regardless of directory. `views/` here
    // is just fixture layout; the config sits at its root so the walk-up
    // from any file under it finds it.
    let routes_dir = tmp.path().join("views").join("git");
    fs::create_dir_all(&routes_dir).unwrap();
    fs::write(
        tmp.path().join("views").join("sail.config.json"),
        format!(r#"{{"network": ["{GRANTED_ORIGIN}"]}}"#),
    )
    .unwrap();
    let sail = routes_dir.join("w.sail");
    let body = format!(
        "<controller>\nexport default controller((ctx) => {{\n  fetch('{fetch_target}')\n  return {{}}\n}})\n</controller>\n<p>hi</p>"
    );
    fs::write(&sail, &body).unwrap();
    (tmp, sail)
}

fn opts_for(path: &PathBuf) -> CompileOpts {
    CompileOpts {
        route_params: None,
        setup_url: None,
        mode: CompileMode::Auto,
        source_path: Some(path.clone()),
        component_meta: None,
        emit_dts: false,
        scope_capabilities: Vec::new(),
        emit_component_imports: false,
        lint_overrides: Policy::overrides_for(path),
    }
}

#[test]
fn fetch_in_controller_is_sail_pure_region_regardless_of_network_grant() {
    // Controllers are the sail typecheck (no DOM). `fetch` is unnameable
    // even when `"network"` grants the origin — the grant drives CSP, not
    // the controller surface. Both granted and evil fail the same way.
    for target in [GRANTED_ORIGIN, EVIL_ORIGIN] {
        let (_tmp, sail) = write_fixture(target);
        let src = fs::read_to_string(&sail).unwrap();
        let err = compile(&src, opts_for(&sail)).expect_err("fetch in a controller");
        assert!(
            matches!(
                err.category.as_deref(),
                Some("sail-pure-region") | Some("sail-controller-no-ambient-authority")
            ),
            "target {target}: {err:?}"
        );
    }
}

#[test]
fn served_csp_names_exactly_the_same_granted_origin() {
    // (b) the SAME `GRANTED_ORIGIN` constant, fed to deck-core's CSP builder —
    // not a second hardcoded string that happens to match (a)'s fixture.
    let network = vec![GRANTED_ORIGIN.to_string()];
    let headers = deck_core::html_security_headers_for(&network);
    let csp = headers
        .iter()
        .find(|(k, _)| *k == "Content-Security-Policy")
        .map(|(_, v)| v.clone())
        .expect("CSP header present");
    assert!(
        csp.contains(&format!("connect-src {GRANTED_ORIGIN}")),
        "connect-src must name exactly the granted origin: {csp}"
    );
    assert!(
        !csp.contains(EVIL_ORIGIN),
        "the refused origin from (a) must never appear in the served CSP: {csp}"
    );
}

/// Belt-and-suspenders: prove BOTH assertions really do read `GRANTED_ORIGIN`
/// from the one fixture-building path, by round-tripping the fixture's
/// `sail.config.json` through the real config resolver
/// (`Policy::network_for`) and feeding THAT resolved list into the CSP
/// builder — no hardcoded origin at all on this path.
#[test]
fn resolved_config_network_list_drives_the_csp_header_directly() {
    let (_tmp, sail) = write_fixture(GRANTED_ORIGIN);
    let resolved = Policy::network_for(&sail);
    assert_eq!(
        resolved,
        vec![GRANTED_ORIGIN.to_string()],
        "sanity: fixture resolves as declared"
    );

    let headers = deck_core::html_security_headers_for(&resolved);
    let csp = headers
        .iter()
        .find(|(k, _)| *k == "Content-Security-Policy")
        .map(|(_, v)| v.clone())
        .unwrap();
    assert!(
        csp.contains(&format!("connect-src {GRANTED_ORIGIN}")),
        "{csp}"
    );
}

/// Live SPA shell path (EXT-1 E6): `serve_scaffold` must resolve the project's
/// `sail.config` `"network"` and emit it on the response CSP. If the wire is
/// broken (shell still uses the constant `'self'` headers only), this fails.
#[test]
fn serve_scaffold_csp_honors_package_network() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    fs::write(
        root.join("sail.config.json"),
        format!(r#"{{"network": ["{GRANTED_ORIGIN}"]}}"#),
    )
    .unwrap();
    // Built shell the serve path reads.
    fs::create_dir_all(root.join(".deck")).unwrap();
    fs::write(
        root.join(".deck/index.html"),
        "<!doctype html><html><head></head><body><div id=\"root\"></div></body></html>",
    )
    .unwrap();

    let scaffold = AppServeConfig {
        name: "app".to_string(),
        route: "/".to_string(),
        source: ".deck".to_string(),
        inject_globals: BTreeMap::new(),
        render: RenderMode::Csr,
        entry: None,
        ops_project: None,
    };
    let resolution = deck_core::Resolution::new().with_root(root);
    let resp = serve_scaffold(
        DaemonMode::Development,
        &scaffold,
        &[],
        None,
        Some(&resolution),
        None,
    );
    assert_eq!(resp.status, 200, "shell must serve");

    let csp = resp
        .extra_headers
        .iter()
        .find(|(k, _)| *k == "Content-Security-Policy")
        .map(|(_, v)| v.as_str())
        .expect("CSP on SPA shell");
    assert!(
        csp.contains(&format!("connect-src {GRANTED_ORIGIN}")),
        "live shell CSP must name the package network grant; got: {csp}"
    );
    assert!(
        !csp.contains(EVIL_ORIGIN),
        "refused origin must never appear: {csp}"
    );
}

/// Empty / absent network on the live path must stay byte-identical to the
/// historical default security header set (no silent connect-src change).
#[test]
fn serve_scaffold_empty_network_matches_default_security_headers() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    // No sail.config at all.
    fs::create_dir_all(root.join(".deck")).unwrap();
    fs::write(
        root.join(".deck/index.html"),
        "<!doctype html><html><head></head><body><div id=\"root\"></div></body></html>",
    )
    .unwrap();

    let scaffold = AppServeConfig {
        name: "app".to_string(),
        route: "/".to_string(),
        source: ".deck".to_string(),
        inject_globals: BTreeMap::new(),
        render: RenderMode::Csr,
        entry: None,
        ops_project: None,
    };
    let resolution = deck_core::Resolution::new().with_root(root);
    let resp = serve_scaffold(
        DaemonMode::Development,
        &scaffold,
        &[],
        None,
        Some(&resolution),
        None,
    );
    assert_eq!(resp.status, 200);
    assert_eq!(
        resp.extra_headers.len(),
        HTML_SECURITY_HEADERS.len(),
        "empty network must emit the full default set"
    );
    for (i, (name, value)) in HTML_SECURITY_HEADERS.iter().enumerate() {
        let (got_name, got_value) = &resp.extra_headers[i];
        assert_eq!(got_name, name);
        assert_eq!(
            got_value, value,
            "empty network must be byte-identical to HTML_SECURITY_HEADERS for `{name}`"
        );
    }
}
