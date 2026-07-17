//! THE SEAM-COHERENCE GATE for the `"network"` capability (fetch lever, item
//! 4). A `sui.config.json` declaring `{"network": [...]}` must drive BOTH the
//! compiler's static lint AND deck's served CSP `connect-src` header from the
//! SAME declaration — drift between the two is a test failure here, not a
//! discovery in production.
//!
//! Both assertions below derive from ONE Rust-level origin list
//! (`GRANTED_ORIGIN`), used to:
//!   (a) build the fixture `sui.config.json` body that the real compiler
//!       resolver (`sui_compiler::check::policy::Policy`) reads, driving the
//!       `sui-controller-no-ambient-authority` lint through the real
//!       `compile()` build gate — same path `config_loud_strict.rs` exercises;
//!   (b) build the `connect-src` header via `deck_core::connect_src_directive`
//!       — the same list, not two independently hardcoded origin strings.
//!
//! If a follow-up threads `connect_src_directive` into `DeckResponse::html_view`
//! (currently NOT wired — see `response.rs`'s `connect_src_directive` doc
//! comment for the exact missing seam), this test's part (b) already proves
//! the function itself is correct; only the plumbing remains.

use std::fs;
use std::path::PathBuf;

use sui_compiler::check::policy::Policy;
use sui_compiler::{CompileMode, CompileOpts, compile};

/// The ONE declaration both halves of this test derive from.
const GRANTED_ORIGIN: &str = "https://api.example.com";
const EVIL_ORIGIN: &str = "https://evil.com";

fn write_fixture(fetch_target: &str) -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
    // `views/` in the path is required for the ambient-authority rule's
    // confined-surface scope (`is_confined_surface`); the config sits at the
    // views root so the walk-up from any file under it finds it.
    let views_dir = tmp.path().join("views").join("git");
    fs::create_dir_all(&views_dir).unwrap();
    fs::write(
        tmp.path().join("views").join("sui.config.json"),
        format!(r#"{{"network": ["{GRANTED_ORIGIN}"]}}"#),
    )
    .unwrap();
    let sui = views_dir.join("w.sui");
    let body = format!(
        "<controller>\nexport default controller((ctx) => {{\n  fetch('{fetch_target}')\n  return {{}}\n}})\n</controller>\n<p>hi</p>"
    );
    fs::write(&sui, &body).unwrap();
    (tmp, sui)
}

fn opts_for(path: &PathBuf) -> CompileOpts {
    CompileOpts {
        route_params: None,
        setup_url: None,
        mode: CompileMode::Auto,
        source_path: Some(path.clone()),
        component_meta: None,
        emit_dts: false,
        emit_component_imports: false,
        lint_overrides: Policy::overrides_for(path),
    }
}

#[test]
fn granted_origin_passes_lint_evil_origin_is_refused_same_config() {
    // (a) the granted origin compiles clean.
    let (_tmp, sui) = write_fixture(GRANTED_ORIGIN);
    let src = fs::read_to_string(&sui).unwrap();
    compile(&src, opts_for(&sui)).expect("a fetch to the granted origin must compile clean");

    // (a) an origin NOT on the same config's allowlist is refused.
    let (_tmp2, sui2) = write_fixture(EVIL_ORIGIN);
    let src2 = fs::read_to_string(&sui2).unwrap();
    let err = compile(&src2, opts_for(&sui2))
        .expect_err("a fetch to an origin outside the allowlist must be refused");
    assert_eq!(err.category.as_deref(), Some("sui-controller-no-ambient-authority"));
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
/// `sui.config.json` through the real config resolver
/// (`Policy::network_for`) and feeding THAT resolved list into the CSP
/// builder — no hardcoded origin at all on this path.
#[test]
fn resolved_config_network_list_drives_the_csp_header_directly() {
    let (_tmp, sui) = write_fixture(GRANTED_ORIGIN);
    let resolved = Policy::network_for(&sui);
    assert_eq!(resolved, vec![GRANTED_ORIGIN.to_string()], "sanity: fixture resolves as declared");

    let headers = deck_core::html_security_headers_for(&resolved);
    let csp = headers
        .iter()
        .find(|(k, _)| *k == "Content-Security-Policy")
        .map(|(_, v)| v.clone())
        .unwrap();
    assert!(csp.contains(&format!("connect-src {GRANTED_ORIGIN}")), "{csp}");
}
