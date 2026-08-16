//! JOB-EDITOR-1 hover matrix — local fixture only (no studio walk).
//!
//! Structure probes always run. TS probes are red gates.

use std::path::PathBuf;
use std::time::Duration;

use sail_lsp::{Intelligence, doc_key};

/// Inline fixture (not a `.sail` on disk — the unknown-vocab probe is
/// intentionally illegal product and must not trip `sail check` hooks).
const FIXTURE: &str = r#"<controller>
import { controller, signal } from '@ship/sail-runtime'

export default controller((ctx) => ({ count: signal(0) }))
</controller>
<column style({ bg(primary), padding(4) })>
  {style.bg(primary)}
</column>
"#;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("repo root")
}

/// 1-based line/column of the first `needle` in `src`.
fn pos_of(src: &str, needle: &str) -> (u32, u32) {
    pos_of_nth(src, needle, 0)
}

fn pos_of_nth(src: &str, needle: &str, n: usize) -> (u32, u32) {
    let mut from = 0usize;
    let mut off = 0usize;
    for i in 0..=n {
        off = src[from..]
            .find(needle)
            .unwrap_or_else(|| panic!("needle `{needle}` #{i}"))
            + from;
        from = off + needle.len();
    }
    let mut line = 1u32;
    let mut col = 1u32;
    for (i, ch) in src.char_indices() {
        if i >= off {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

fn hover_text(intel: &mut Intelligence, path: &str, line: u32, col: u32) -> String {
    intel
        .hover_sail(FIXTURE, Some(path), line, col)
        .map(|h| h.contents)
        .unwrap_or_default()
}

fn settle_hover(intel: &mut Intelligence, path: &str, line: u32, col: u32) -> String {
    let mut text = hover_text(intel, path, line, col);
    if text.trim().is_empty() {
        std::thread::sleep(Duration::from_millis(300));
        text = hover_text(intel, path, line, col);
    }
    text
}

fn looks_typed_signal_or_fn(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    (lower.contains("signal") || lower.contains("function") || lower.contains("controllerdef"))
        && !text.trim().is_empty()
}

fn is_empty_object_type(text: &str) -> bool {
    let stripped: String = text
        .lines()
        .filter(|l| !l.trim().starts_with("```"))
        .collect::<Vec<_>>()
        .join("\n");
    let compact: String = stripped.chars().filter(|c| !c.is_whitespace()).collect();
    let compact = compact.replace(['`', '*'], "");
    let tail = compact.rsplit(':').next().unwrap_or(compact.as_str());
    tail == "{}"
}

#[test]
fn hover_matrix_records_probes() {
    let path = "crates/sail/lsp/tests/fixtures/hover-matrix.sail";
    let mut intel = Intelligence::start(Some(&repo_root()));
    intel.open_or_update(&doc_key(Some(path), "hover.sail"), FIXTURE);

    if sail_lsp::tsgo_test::discover_tsgo(None).is_none() {
        panic!("{}", sail_compiler::ts_native::missing_sidecar_hint());
    }
    assert!(
        intel.is_tsgo_live(),
        "sidecar present but Intelligence session is dead"
    );
    std::thread::sleep(Duration::from_millis(200));

    // ── probes (1-based) ──────────────────────────────────────────────
    let (theme_l, theme_c) = pos_of(FIXTURE, "primary");
    let (signal_l, signal_c) = pos_of(FIXTURE, "signal(0)");
    let (ctrl_l, ctrl_c) = pos_of(FIXTURE, "controller((ctx)");
    let (col_l, col_c) = pos_of(FIXTURE, "column");
    let (pad_l, pad_c) = pos_of(FIXTURE, "padding");
    let (foo_l, foo_c) = pos_of(FIXTURE, "style.bg");
    // Land on `bg` of `style.bg`, not on `style`.
    let foo_c = foo_c + "style.".len() as u32;

    let theme = settle_hover(&mut intel, path, theme_l, theme_c);
    let layout = settle_hover(&mut intel, path, col_l, col_c);
    let unknown = settle_hover(&mut intel, path, pad_l, pad_c);

    let signal = settle_hover(&mut intel, path, signal_l, signal_c);
    let controller = settle_hover(&mut intel, path, ctrl_l, ctrl_c);
    let style_foo = settle_hover(&mut intel, path, foo_l, foo_c);

    eprintln!(
        "\nHOVER MATRIX (1-based)\n\
         | Probe | Line | Col | Contents |\n\
         |---|---:|---:|---|\n\
         | theme token `primary` | {theme_l} | {theme_c} | {} |\n\
         | `signal` | {signal_l} | {signal_c} | {} |\n\
         | `controller` | {ctrl_l} | {ctrl_c} | {} |\n\
         | layout `<column>` | {col_l} | {col_c} | {} |\n\
         | unknown vocab `padding` | {pad_l} | {pad_c} | {} |\n\
         | `style.bg` | {foo_l} | {foo_c} | {} |\n",
        theme.replace('|', "\\|").replace('\n', " / "),
        signal.replace('|', "\\|").replace('\n', " / "),
        controller.replace('|', "\\|").replace('\n', " / "),
        layout.replace('|', "\\|").replace('\n', " / "),
        unknown.replace('|', "\\|").replace('\n', " / "),
        style_foo.replace('|', "\\|").replace('\n', " / "),
    );

    // theme token under bg(primary)
    assert!(
        theme.contains("theme") && (theme.contains("oklch") || theme.contains("--")),
        "theme hover must show css name / resolved value / theme/mode; got {theme}"
    );
    assert!(!theme.contains("parse error"), "theme: {theme}");

    // layout keyword
    assert!(
        !layout.trim().is_empty(),
        "layout keyword hover must not be empty"
    );
    assert!(
        layout.to_ascii_lowercase().contains("layout")
            || layout.contains("column")
            || layout.contains("sail-participation-style"),
        "layout hover must be teaching / symbol, not empty; got {layout}"
    );
    assert!(
        !layout.to_ascii_lowercase().contains("parse error"),
        "layout must not be parse error: {layout}"
    );
    assert!(!layout.contains("Pure."), "do not say Pure.: {layout}");

    // unknown vocab
    assert!(
        unknown.contains("unknown vocabulary"),
        "unknown vocab must be a teaching sentence; got {unknown}"
    );
    assert!(
        !unknown.to_ascii_lowercase().contains("parse error"),
        "unknown vocab must not be a raw parse error: {unknown}"
    );
    assert!(!unknown.contains("Pure."), "do not say Pure.: {unknown}");

    assert!(
        !signal.trim().is_empty(),
        "empty hover on `signal` is a fail"
    );
    assert!(
        looks_typed_signal_or_fn(&signal),
        "expected typed Signal<…> / function hover for signal; got {signal}"
    );
    assert!(
        !controller.trim().is_empty(),
        "empty hover on `controller` is a fail"
    );
    assert!(
        looks_typed_signal_or_fn(&controller),
        "expected typed controller / function hover; got {controller}"
    );

    // style.foo must not be typed as {} if we claim types
    assert!(
        !style_foo.trim().is_empty(),
        "empty hover on style.bg is a fail"
    );
    assert!(
        !is_empty_object_type(&style_foo),
        "style.foo must not be typed as {{}}; got {style_foo}"
    );
    let foo_lwr = style_foo.to_ascii_lowercase();
    assert!(
        foo_lwr.contains("bg")
            || foo_lwr.contains("sailstyle")
            || foo_lwr.contains("function")
            || foo_lwr.contains("("),
        "style.bg must claim a real type; got {style_foo}"
    );
}
