//! TOOLING-1 WI1 — `sail fmt` acceptance fixtures.
//!
//! 1. Hand-mangled source + format → matches golden (canonical).
//! 2. format ∘ format = format (idempotent).
//! 3. Controller + template regions covered.

use std::fs;
use std::path::PathBuf;

use sail_compiler::{FormatOpts, format_source_with};

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fmt_fixtures")
}

fn opts() -> FormatOpts {
    // Deterministic without host oxfmt; oxfmt path covered by unit discover +
    // optional env-gated test below.
    FormatOpts {
        use_oxfmt: false,
        ..Default::default()
    }
}

fn format(src: &str) -> String {
    format_source_with(src, &opts()).expect("format").source
}

#[test]
fn mangled_template_matches_golden() {
    let mangled = fs::read_to_string(fixture_dir().join("template.mangled.sail")).unwrap();
    let golden_path = fixture_dir().join("template.golden.sail");
    let got = format(&mangled);
    if std::env::var_os("UPDATE_FMT_GOLDEN").is_some() {
        fs::write(&golden_path, &got).unwrap();
    }
    let golden = fs::read_to_string(&golden_path).expect("template.golden.sail — run with UPDATE_FMT_GOLDEN=1 once");
    assert_eq!(got, golden, "mangled template must format to golden");
    assert_eq!(format(&got), got, "golden must be idempotent");
}

#[test]
fn mangled_counter_matches_golden() {
    let mangled = fs::read_to_string(fixture_dir().join("counter.mangled.sail")).unwrap();
    let golden_path = fixture_dir().join("counter.golden.sail");
    let got = format(&mangled);
    if std::env::var_os("UPDATE_FMT_GOLDEN").is_some() {
        fs::write(&golden_path, &got).unwrap();
    }
    let golden = fs::read_to_string(&golden_path).expect("counter.golden.sail — run with UPDATE_FMT_GOLDEN=1 once");
    assert_eq!(got, golden, "mangled counter must format to golden");
    assert_eq!(format(&got), got, "golden must be idempotent");
}

#[test]
fn corpus_idempotent_on_fixtures() {
    for name in ["template.mangled.sail", "counter.mangled.sail"] {
        let src = fs::read_to_string(fixture_dir().join(name)).unwrap();
        let once = format(&src);
        let twice = format(&once);
        assert_eq!(once, twice, "{name}: fmt ∘ fmt = fmt");
    }
}

#[test]
fn is_formatted_false_on_mangled() {
    let mangled = fs::read_to_string(fixture_dir().join("template.mangled.sail")).unwrap();
    assert!(!sail_compiler::is_formatted(&mangled, &opts()).unwrap());
    let gold = format(&mangled);
    assert!(sail_compiler::is_formatted(&gold, &opts()).unwrap());
}

/// When oxfmt is on PATH / node_modules, controller body is handed through it
/// and remains idempotent.
#[test]
fn oxfmt_controller_region_when_available() {
    if matches!(
        sail_compiler::fmt::discover_oxfmt(None),
        sail_compiler::fmt::OxfmtStatus::Missing
    ) {
        eprintln!("oxfmt missing — skip live oxfmt region test");
        return;
    }
    let src = r#"<controller>
export default controller(()=>({x:signal(0)}))
</controller>

<p>{x()}</p>
"#;
    let opts = FormatOpts {
        use_oxfmt: true,
        ..Default::default()
    };
    let once = format_source_with(src, &opts).expect("fmt").source;
    let twice = format_source_with(&once, &opts).expect("fmt2").source;
    assert_eq!(once, twice, "oxfmt path must be idempotent:\n{once}");
    assert!(once.contains("<controller>"), "{once}");
    assert!(once.contains("export default"), "{once}");
}
