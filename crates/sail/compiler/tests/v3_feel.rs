//! Red-first v3 feel-test fixtures (SPEC-SPINNAKER-3 §10).
//!
//! The `.sail` files are valid-looking v3 authoring: a flat chain of
//! calls, components first (`<card>` / `<h2>` / `<a>`), no author-facing
//! twins (`card-chrome`, `heading-2`, `navLink`). They will not compile
//! until the v3 parse lands. Default suite: fixtures exist.
//! `v3_feel_compiles` stays `#[ignore]` until that PR un-ignores it.

use std::fs;
use std::path::PathBuf;

use sail_compiler::{compile, CompileOpts};

const FIXTURES: &[&str] = &["form", "overlay", "nav", "data", "motion"];

/// Author-facing twins SPEC-SPINNAKER-3 §10 forbids beside the tags.
const BANNED_TWINS: &[&str] = &["card-chrome", "heading-2", "heading-3", "navLink"];

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/v3_feel")
}

fn fixture_src(name: &str) -> (PathBuf, String) {
    let path = fixtures_dir().join(format!("{name}.sail"));
    let src = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    (path, src)
}

#[test]
fn v3_feel_fixtures_exist() {
    let dir = fixtures_dir();
    assert!(dir.is_dir(), "missing fixtures dir {}", dir.display());

    for name in FIXTURES {
        let (path, src) = fixture_src(name);
        assert!(path.is_file(), "missing fixture {}", path.display());
        assert!(!src.trim().is_empty(), "{name}.sail is empty");
        assert!(
            src.contains("style(") || src.contains("style "),
            "{name}.sail must be v3 authoring (style chain / declaration)"
        );
        for banned in BANNED_TWINS {
            assert!(
                !src.contains(banned),
                "{name}.sail must not seed `{banned}` (SPEC-SPINNAKER-3 §10)"
            );
        }
    }

    let overlay = fixture_src("overlay").1;
    assert!(
        !overlay.contains("style(muted)"),
        "overlay muted must be inline, not a missing const"
    );
    assert!(
        overlay.contains("type(body-sm).fg(muted-foreground)"),
        "overlay muted must be the inline v3 chain"
    );
    assert!(
        overlay.contains("play("),
        "overlay must keep play(…) as in the feel test"
    );
    assert!(
        overlay.contains("style(fg("),
        "overlay must keep style(fg(…)) as in the feel test"
    );

    let form = fixture_src("form").1;
    assert!(
        form.contains("<card") && form.contains("<h2>"),
        "form must compose <card> / <h2>, not twins"
    );
    assert!(
        form.contains("style(pad("),
        "form must keep style(pad(…)) as in the feel test"
    );

    let nav = fixture_src("nav").1;
    assert!(
        nav.contains("<a ") && nav.contains("style(pad("),
        "nav must style <a> with pad(…), not a navLink twin"
    );
    assert!(
        nav.contains("play("),
        "nav must keep play(…) as in the feel test"
    );
}

#[test]
#[ignore]
fn v3_feel_compiles() {
    for name in FIXTURES {
        let (path, src) = fixture_src(name);
        compile(
            &src,
            CompileOpts {
                source_path: Some(path.clone()),
                ..CompileOpts::default()
            },
        )
        .unwrap_or_else(|e| panic!("compile {}: {e}", path.display()));
    }
}
