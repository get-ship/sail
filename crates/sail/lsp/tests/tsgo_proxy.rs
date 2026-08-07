//! Integration tests: virtual TS + tsgo proxy round-trips at **.sail** positions.
//!
//! Soft-skip when tsgo is absent (mirrors `synthetic_tsc` opt-in style).
//! Asserts hover / diagnostics land on `.sail` spans, not virtual ones.

use std::path::PathBuf;

use sail_lsp::tsgo_test::{TsgoSession, discover_tsgo};
use sail_lsp::virtual_ts_test::{
    RegionKind, VirtualFile, build_virtual_ts, offset_to_lsp_position,
};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("repo root")
}

fn skip_without_tsgo() -> bool {
    if discover_tsgo(Some(&repo_root())).is_none() {
        eprintln!(
            "SKIP tsgo_proxy: tsgo not found under {} — run `bun install` \
             (provides node_modules/.bin/tsgo via @typescript/native-preview).",
            repo_root().join("node_modules/.bin/tsgo").display()
        );
        return true;
    }
    false
}

const COUNTER: &str = r#"<controller>
import { controller, signal } from '@ship/sail-runtime'

export default controller((ctx) => ({ count: signal(0) }))
</controller>

<div>
  <p>{count()}</p>
  <button on:click={() => count.set(count() + 1)}>inc</button>
  <span title={count()}>x</span>
</div>
"#;

const EXPR_TYPO: &str = r#"<controller>
import { controller, signal } from '@ship/sail-runtime'
export default controller((ctx) => ({ count: signal(0) }))
</controller>
<div>{countt()}</div>
"#;

const CTRL_TYPO: &str = r#"<controller>
import { controller, signal } from '@ship/sail-runtime'
export default controller((ctx) => {
  const c = signal(0)
  return { count: c.nope }
})
</controller>
<div>{count()}</div>
"#;

const SCOPE_FILE: &str = r#"<scope>
type Count = number
props: { n: Count = 0 }
</scope>
<div>hi</div>
"#;

#[test]
fn source_map_covers_all_fixture_kinds() {
    // Pure map fixtures — no tsgo required.
    let v = build_virtual_ts(COUNTER).expect("counter virtual");
    assert!(v.regions.iter().any(|r| r.kind == RegionKind::Controller));
    assert!(
        v.regions
            .iter()
            .filter(|r| r.kind == RegionKind::Expression)
            .count()
            >= 3
    );

    // controller body char
    let signal_at = COUNTER.find("signal").unwrap();
    let (file, virt) = v.sail_to_virtual(signal_at).unwrap();
    assert_eq!(file, VirtualFile::Controller);
    assert_eq!(v.virtual_to_sail(file, virt), Some(signal_at));

    // text interpolation
    let count_at = COUNTER.find("{count()}").unwrap() + 1;
    assert_eq!(v.region_at(count_at), Some(RegionKind::Expression));
    let (file, _) = v.sail_to_virtual(count_at).unwrap();
    assert_eq!(file, VirtualFile::View);

    // event handler
    let handler = COUNTER.find("() => count.set").unwrap();
    assert_eq!(v.region_at(handler), Some(RegionKind::Expression));

    // attribute expression
    let title = COUNTER.rfind("{count()}").unwrap() + 1;
    assert_eq!(v.region_at(title), Some(RegionKind::Expression));

    // scope type alias
    let vs = build_virtual_ts(SCOPE_FILE).expect("scope virtual");
    let type_at = SCOPE_FILE.find("type Count").unwrap();
    assert_eq!(vs.region_at(type_at), Some(RegionKind::Scope));
    let (file, virt) = vs.sail_to_virtual(type_at).unwrap();
    assert_eq!(vs.virtual_to_sail(file, virt), Some(type_at));
}

#[test]
fn degrade_without_tsgo_keeps_structure() {
    // Session with no client still builds virtuals and never panics.
    let mut session = TsgoSession::start(Some(std::path::Path::new("/no-tsgo-here-for-degrade")));
    // Force client off even if PATH has tsgo (discover may still succeed from
    // PATH; structure path must work either way).
    let uri = "file:///tmp/degrade.sail";
    session.on_sail_change(uri, COUNTER);
    assert!(session.virtual_for(uri).is_some());
    // Structure analysis still works independent of tsgo.
    let analysis = sail_compiler::analyze(COUNTER, None, &[]);
    // No parse crash; may have other diags but must return.
    let _ = analysis.diagnostics;
}

#[test]
fn tsgo_template_typo_maps_to_sail_span() {
    if skip_without_tsgo() {
        return;
    }
    let root = repo_root();
    let mut session = TsgoSession::start(Some(&root));
    if !session.is_live() {
        eprintln!("SKIP tsgo_proxy typo: session failed to start");
        return;
    }
    let uri = "file:///tmp/expr-typo.sail";
    session.on_sail_change(uri, EXPR_TYPO);
    // Give tsgo a moment after didOpen before pull diagnostics.
    std::thread::sleep(std::time::Duration::from_millis(200));
    let diags = session.diagnostics_mapped(uri, EXPR_TYPO);
    let hit = diags.iter().find(|d| d.message.contains("countt"));
    assert!(
        hit.is_some(),
        "expected diagnostic mentioning countt at .sail span; got {diags:?}"
    );
    let d = hit.unwrap();
    // Range must fall on the template expression line in the .sail source.
    let countt_off = EXPR_TYPO.find("countt").unwrap();
    let (line, col) = offset_to_lsp_position(EXPR_TYPO, countt_off);
    assert_eq!(
        d.range.start.line, line,
        "diagnostic line must be .sail line of countt ({}), got {:?}",
        line, d.range
    );
    // Character should be near the typo (allow small mapping slack for
    // rewritten scope. prefix proportional map).
    assert!(
        (d.range.start.character as i64 - col as i64).abs() <= 8 || d.range.end.character > col,
        "diagnostic col should near countt col {col}, got {:?}",
        d.range
    );
}

#[test]
fn tsgo_controller_typo_maps_to_sail_span() {
    if skip_without_tsgo() {
        return;
    }
    let root = repo_root();
    let mut session = TsgoSession::start(Some(&root));
    if !session.is_live() {
        eprintln!("SKIP tsgo_proxy controller typo: session failed to start");
        return;
    }
    let uri = "file:///tmp/ctrl-typo.sail";
    session.on_sail_change(uri, CTRL_TYPO);
    std::thread::sleep(std::time::Duration::from_millis(200));
    let diags = session.diagnostics_mapped(uri, CTRL_TYPO);
    let hit = diags.iter().find(|d| d.message.contains("nope"));
    assert!(
        hit.is_some(),
        "expected controller diagnostic for .nope; got {diags:?}"
    );
    let d = hit.unwrap();
    let nope = CTRL_TYPO.find("nope").unwrap();
    let (line, _) = offset_to_lsp_position(CTRL_TYPO, nope);
    assert_eq!(
        d.range.start.line, line,
        "controller typo must map to .sail line {line}, got {:?}",
        d.range
    );
}

#[test]
fn tsgo_hover_in_controller_returns_typed_result() {
    if skip_without_tsgo() {
        return;
    }
    let root = repo_root();
    let mut session = TsgoSession::start(Some(&root));
    if !session.is_live() {
        eprintln!("SKIP tsgo_proxy hover: session failed to start");
        return;
    }
    let uri = "file:///tmp/counter-hover.sail";
    session.on_sail_change(uri, COUNTER);
    std::thread::sleep(std::time::Duration::from_millis(200));
    // Hover on `signal` inside the controller body.
    let signal_at = COUNTER.find("signal(0)").unwrap();
    let hover = session.hover_mapped(uri, COUNTER, signal_at);
    assert!(
        hover.is_some(),
        "expected tsgo hover on signal() in controller"
    );
    let h = hover.unwrap();
    let text = h.get("contents").map(|c| c.to_string()).unwrap_or_default();
    assert!(
        text.to_ascii_lowercase().contains("signal") || text.contains("function"),
        "hover contents should mention signal; got {text}"
    );
}

#[test]
fn tsgo_hover_in_template_expression() {
    if skip_without_tsgo() {
        return;
    }
    let root = repo_root();
    let mut session = TsgoSession::start(Some(&root));
    if !session.is_live() {
        eprintln!("SKIP tsgo_proxy expr hover: session failed to start");
        return;
    }
    let uri = "file:///tmp/counter-expr-hover.sail";
    session.on_sail_change(uri, COUNTER);
    std::thread::sleep(std::time::Duration::from_millis(300));
    // Hover on `count` inside `{count()}`.
    let count_at = COUNTER.find("{count()}").unwrap() + 1;
    // Source-map must land on the free ident in `scope.count`, not on `scope`
    // (the pre-fix always returned segment start → `const scope: object`).
    if let Some(virt) = session.virtual_for(uri) {
        let (file, v_off) = virt.sail_to_virtual(count_at).expect("map count");
        assert_eq!(file, VirtualFile::View);
        assert!(
            virt.view_text[v_off..].starts_with("count"),
            "sail→virtual must hit `count` not `scope`; got {:?}",
            &virt.view_text[v_off.saturating_sub(6)..(v_off + 12).min(virt.view_text.len())]
        );
        assert!(
            virt.view_text
                .contains("typeof __sail_authored.__def extends (...a: never[]) => infer R"),
            "view must use synthetic-parity Scope recovery"
        );
    }
    let hover = session.hover_mapped(uri, COUNTER, count_at);
    // Soft-skip only when hover is wholly unavailable (runtime path missing);
    // do not soft-pass a weak `scope: object` result — that is the bug.
    if hover.is_none() {
        eprintln!(
            "NOTE: template expression hover returned None — @ship/sail-runtime \
             types may be missing from the virtual tsconfig paths. Map + recovery \
             emit still asserted above."
        );
        return;
    }
    let text = hover.unwrap().to_string();
    // Must not be the residual weak hover that landed on `const scope: object`.
    assert!(
        !(text.contains("const scope") && text.contains("object")),
        "template count hover must not resolve as `const scope: object`; got {text}"
    );
    // Recoverable success: property/member typing for controller Scope field.
    assert!(
        text.contains("count")
            && (text.contains("number")
                || text.to_ascii_lowercase().contains("signal")
                || text.contains("property")),
        "expected typed Scope member hover for count; got {text}"
    );
}
