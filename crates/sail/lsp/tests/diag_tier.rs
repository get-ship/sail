//! SPEC-TOOLING-1 §WI6 — diagnostic tiering.
//!
//! Acceptance: typing in a ~500-line `.sail` never triggers a layout pass.
//! On-save / full check does.

use std::sync::{Mutex, OnceLock};

use sail_compiler::{DiagnosticTier, check_all_with_tier};
use sail_lsp::{Intelligence, diagnostics_sail_tier};
use spinnaker::layout::{layout_pass_count, reset_layout_pass_count};

/// Serialize tests that observe the process-wide layout-pass counter.
fn layout_counter_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

/// Build a ~500-line valid-ish `.sail` buffer with enough structure that a
/// full layout pass would actually compute something (nested columns/rows).
fn five_hundred_line_sail() -> String {
    let mut out = String::with_capacity(16 * 1024);
    out.push_str(
        r#"<controller>
import { signal, controller } from "@ship/sail-runtime";
export default controller({
  scope: {
    count: signal(0),
    label: signal("hello"),
  },
  on: {
    bump() { this.scope.count.update((n) => n + 1); },
  },
});
</controller>
"#,
    );
    out.push_str("<column gap={4} pad={4}>\n");
    // ~480 body lines → total ~500 with controller shell.
    for i in 0..96 {
        out.push_str("  <row gap={2} align=\"center\">\n");
        out.push_str(&format!("    <text>row {i} a · {{label()}}</text>\n"));
        out.push_str(&format!("    <text>row {i} b · {{count()}}</text>\n"));
        out.push_str("    <button on:click={bump}>Bump</button>\n");
        out.push_str("  </row>\n");
    }
    out.push_str("</column>\n");
    // Ensure we are in the ~500-line ballpark the acceptance cites.
    let lines = out.lines().count();
    assert!(
        (450..=550).contains(&lines),
        "fixture should be ~500 lines, got {lines}"
    );
    out
}

/// Simulate a burst of keystrokes: each change re-runs OnType diagnostics.
/// The taffy layout-pass counter must stay at zero for the whole burst.
#[test]
fn typing_in_500_line_sail_never_triggers_layout_pass() {
    let _guard = layout_counter_lock();
    let base = five_hundred_line_sail();
    reset_layout_pass_count();
    assert_eq!(layout_pass_count(), 0);

    // ~N keystroke-sized edits (append a space, then a char, etc.).
    let mut text = base.clone();
    for step in 0..40 {
        // Mutate near the end so the buffer stays large.
        text.push(if step % 2 == 0 { ' ' } else { 'x' });
        let _diags = check_all_with_tier(&text, None, &[], None, DiagnosticTier::OnType);
        assert_eq!(
            layout_pass_count(),
            0,
            "layout pass fired during on-type step {step} (count={})",
            layout_pass_count()
        );
    }

    // Library intelligence path used by studio bridge — same contract.
    let mut intel = Intelligence::start(None);
    let _ = intel.diagnostics_sail_tier(&base, Some("big.sail"), DiagnosticTier::OnType);
    assert_eq!(
        layout_pass_count(),
        0,
        "intelligence OnType must not run layout"
    );

    // Free-function mirror.
    let _ = diagnostics_sail_tier(&base, Some("big.sail"), DiagnosticTier::OnType);
    assert_eq!(layout_pass_count(), 0);
}

/// On-save / full tier must actually exercise taffy (counter increments).
#[test]
fn on_save_tier_runs_layout_pass() {
    let _guard = layout_counter_lock();
    let src = five_hundred_line_sail();
    reset_layout_pass_count();
    let _ = check_all_with_tier(&src, None, &[], None, DiagnosticTier::OnSave);
    assert!(
        layout_pass_count() > 0,
        "OnSave must run at least one taffy layout pass"
    );
}

/// On-type still surfaces parse / vocabulary findings (not a no-op tier).
#[test]
fn on_type_still_reports_vocabulary_diagnostics() {
    let _guard = layout_counter_lock();
    // Bare participation attr is a structure/vocabulary lint (no taffy needed).
    let bad = r#"<column grow>x</column>"#;
    reset_layout_pass_count();
    let diags = check_all_with_tier(bad, None, &[], None, DiagnosticTier::OnType);
    assert_eq!(layout_pass_count(), 0);
    assert!(
        diags
            .iter()
            .any(|d| d.rule.as_deref() == Some("sail-participation-style")),
        "expected vocabulary diag on-type: {diags:?}"
    );
}
