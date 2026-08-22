//! Tests for SAIL lint rules.

use std::path::Path;

use crate::{
    is_app_source_ts, is_controller_path, lint_app_source_source, lint_controller_source,
    lint_pack_source_source, lint_path,
    rules::{controller_purity, controller_state_cast},
};

fn lint(source: &str) -> Vec<crate::Finding> {
    lint_controller_source(Path::new("/fake/foo.controller.ts"), source)
}

#[test]
fn clean_controller_passes() {
    // Ambient idents (controller, signal) need no import — a clean controller
    // is one that relies on ambient injection and only imports non-ambient /
    // type-only surface when needed.
    let src = r#"
        export default controller(() => {
          const count = signal(0)
          mount((host) => {
            host.addEventListener('click', () => count.set(count() + 1))
          })
          return { count }
        })
    "#;
    let findings = lint(src);
    assert!(
        findings.is_empty(),
        "clean controller should pass: {findings:?}"
    );
}

#[test]
fn inner_html_assignment_reports() {
    let src = r#"
        export default {
          mount: (_state, host) => {
            host.innerHTML = '<p>hi</p>'
            return () => {}
          },
        }
    "#;
    let findings = lint(src);
    assert_eq!(findings.len(), 1, "expected one finding, got {findings:?}");
    assert_eq!(findings[0].rule, controller_purity::RULE_ID);
    assert!(
        findings[0].message.contains(".innerHTML"),
        "msg: {}",
        findings[0].message
    );
    assert_eq!(findings[0].line, 4);
    assert!(findings[0].suggest.is_some(), "should suggest bindHtml");
}

#[test]
fn outer_html_assignment_reports() {
    let src = r#"
        export default {
          mount: (_state, host) => {
            host.outerHTML = '<p>hi</p>'
          },
        }
    "#;
    let findings = lint(src);
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains(".outerHTML"));
}

#[test]
fn insert_adjacent_html_reports() {
    let src = r#"
        export default {
          mount: (_s, host) => {
            host.insertAdjacentHTML('beforeend', '<p>hi</p>')
          },
        }
    "#;
    let findings = lint(src);
    assert_eq!(findings.len(), 1, "got {findings:?}");
    assert!(findings[0].message.contains("insertAdjacentHTML"));
}

#[test]
fn nested_member_chain_reports() {
    // host.shadowRoot.innerHTML = … is the common shadow-DOM slop.
    let src = r#"
        export default {
          mount: (_s, host) => {
            host.shadowRoot.innerHTML = '<div></div>'
          },
        }
    "#;
    let findings = lint(src);
    assert_eq!(findings.len(), 1);
}

#[test]
fn text_content_passes() {
    // Setting textContent is fine — it's not HTML, and it's how
    // controllers should update text from outside the template.
    let src = r#"
        export default {
          mount: (_s, host) => {
            host.textContent = 'hello'
          },
        }
    "#;
    let findings = lint(src);
    assert!(
        findings.is_empty(),
        "textContent must be allowed: {findings:?}"
    );
}

#[test]
fn computed_member_with_innerhtml_string_passes() {
    // host['inner' + 'HTML'] = … is computed access — we can't
    // statically know it's innerHTML. We don't report; if someone is
    // doing this they're already pathologically working around the
    // rule and tsc/runtime will catch it elsewhere.
    let src = r#"
        export default {
          mount: (_s, host) => {
            host['innerHTML'] = '<p>hi</p>'
          },
        }
    "#;
    // Computed member with a literal — rule doesn't fire (we only
    // match StaticMemberExpression). Acceptable for a hard ban now;
    // can tighten later if needed.
    let findings = lint(src);
    assert!(
        findings.is_empty(),
        "computed access not currently inspected"
    );
}

#[test]
fn parse_error_returns_empty() {
    // Invalid TS: parse error → rule swallows, returns no findings.
    let findings = lint("export default { mount: (host =>");
    assert!(findings.is_empty());
}

#[test]
fn is_controller_path_recognises_suffix() {
    assert!(is_controller_path(Path::new("foo/bar.controller.ts")));
    assert!(is_controller_path(Path::new(
        "/abs/sail-button.controller.ts"
    )));
    assert!(!is_controller_path(Path::new("foo/bar.ts")));
    assert!(!is_controller_path(Path::new("foo/bar.controller.tsx")));
    assert!(!is_controller_path(Path::new("foo/bar.sail")));
    assert!(!is_controller_path(Path::new("foo/render.ts")));
}

#[test]
fn flags_synthetic_innerhtml_offender() {
    // Synthetic source matching the pattern Phase 1.5 cleaned up. Real
    // controllers in the repo are now all clean, so a smoke against any
    // specific file would pass for the wrong reason. This keeps the rule
    // exercised end-to-end against the kind of code it's meant to catch.
    let path = std::path::PathBuf::from("synthetic/sail-foo/sail-foo.controller.ts");
    let source = r#"
export default controller({
  mount(state, host) {
    const root = host.shadowRoot
    if (root) root.innerHTML = '<div></div>'
  },
})
"#;
    let findings = lint_controller_source(&path, source);
    assert!(
        !findings.is_empty(),
        "expected synthetic .innerHTML write to report a violation; got none — rule regressed."
    );
}

#[test]
fn state_cast_unknown_as_scope_reports() {
    let src = r#"
        import type { Scope } from './foo.sail'
        export default controller<Scope>({
          mount(state, host) {
            const scope = state as unknown as Scope
            return () => { scope.open.set(false) }
          },
        })
    "#;
    let findings = lint(src);
    assert_eq!(
        findings.len(),
        1,
        "expected one state-cast finding, got {findings:?}"
    );
    assert_eq!(findings[0].rule, controller_state_cast::RULE_ID);
    assert!(
        findings[0].message.contains("Scope"),
        "msg: {}",
        findings[0].message
    );
    assert!(
        findings[0].suggest.is_some(),
        "should suggest $signals access"
    );
}

#[test]
fn state_cast_scope_intersection_reports() {
    // `state as unknown as Scope & { extensions: Signal<X[]> }` (factory form).
    let src = r#"
        export default {
          mount(state) {
            const scope = state as unknown as Scope & { extensions: () => string[] }
            return scope.extensions()
          },
        }
    "#;
    let findings = lint(src);
    assert_eq!(
        findings.len(),
        1,
        "intersection cast should report once, got {findings:?}"
    );
    assert_eq!(findings[0].rule, controller_state_cast::RULE_ID);
}

#[test]
fn state_cast_scopeview_alias_reports() {
    let src = r#"
        type ScopeView = Scope
        export default {
          mount(state) {
            const s = state as unknown as ScopeView
            return s.value()
          },
        }
    "#;
    let findings = lint(src);
    assert!(
        findings
            .iter()
            .any(|f| f.rule == controller_state_cast::RULE_ID),
        "ScopeView alias cast should report, got {findings:?}"
    );
}

#[test]
fn typed_signals_access_passes_state_cast() {
    // The post-#121 pattern: no cast, read via $signals — must NOT report.
    let src = r#"
        export default controller({
          mount(state, host) {
            const el = state.$signals.dialogEl()
            host.appendChild(el)
            return () => { state.$signals.open.set(false) }
          },
        })
    "#;
    let findings: Vec<_> = lint(src)
        .into_iter()
        .filter(|f| f.rule == controller_state_cast::RULE_ID)
        .collect();
    assert!(
        findings.is_empty(),
        "typed $signals access must pass: {findings:?}"
    );
}

fn lint_tpl(source: &str) -> Vec<crate::Finding> {
    crate::lint_sail_template_source(Path::new("/fake/page.sail"), source)
        .into_iter()
        .filter(|f| f.rule == crate::rules::attr_bool_prop::RULE_ID)
        .collect()
}

#[test]
fn attr_bool_rule_is_noop_after_prefix_removal() {
    // attr:/prop: are parser-refused; this lint no longer fires on plain fields.
    let f = lint_tpl(r#"<alert dismissible={busy()}>Go</alert>"#);
    assert!(f.is_empty(), "plain bool field must not be flagged: {f:?}");
    let f = lint_tpl(r#"<option value="x" selected={sel()}>x</option>"#);
    assert!(f.is_empty(), "{f:?}");
}

#[test]
fn static_bool_flag_passes() {
    // Bare flag / static attr (no brace binding) is not this rule's concern.
    let f = lint_tpl(r#"<sail-dialog open></sail-dialog>"#);
    assert!(f.is_empty(), "bare flag attr not flagged: {f:?}");
}

#[test]
fn pack_source_path_recognises_components_dir() {
    use crate::is_pack_source_ts;
    // Pack src files under components/sail → gate.
    assert!(is_pack_source_ts(Path::new(
        "/repo/packages/components/sail/primitives/src/sail-tree/engine.ts"
    )));
    // Framework runtime → skip (the seam lives here, exempt).
    assert!(!is_pack_source_ts(Path::new(
        "/repo/packages/sail/runtime/src/actions/range-fill.ts"
    )));
    // .controller.ts → not this category (it's a controller).
    assert!(!is_pack_source_ts(Path::new(
        "/repo/packages/components/sail/primitives/src/sail-tree.controller.ts"
    )));
    // Not a .ts file.
    assert!(!is_pack_source_ts(Path::new(
        "/repo/packages/components/sail/primitives/src/sail-tree/sail-tree.sail"
    )));
    // Not a pack source (no /src/ under components/sail).
    assert!(!is_pack_source_ts(Path::new(
        "/repo/packages/components/sail/primitives/package.json"
    )));
}

#[test]
fn pack_source_lints_imperative_style() {
    // A pack glue file with a `.style.x =` write — the floor rule fires.
    let findings = lint_pack_source_source(
        Path::new("/repo/packages/components/sail/primitives/src/widget/glue.ts"),
        "el.style.opacity = '0.5'",
    );
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].rule, "sail-no-imperative-style");
}

#[test]
fn app_source_path_recognises_apps_src() {
    assert!(is_app_source_ts(Path::new(
        "/repo/apps/studio/src/edit/region-overlay.ts"
    )));
    assert!(is_app_source_ts(Path::new(
        "apps/studio/src/edit/region-overlay.ts"
    )));
    // Controllers are not this category.
    assert!(!is_app_source_ts(Path::new(
        "/repo/apps/studio/src/shell.controller.ts"
    )));
    // Tests are exempt.
    assert!(!is_app_source_ts(Path::new(
        "/repo/apps/studio/src/edit/region-overlay.test.ts"
    )));
    // Pack / runtime not app source.
    assert!(!is_app_source_ts(Path::new(
        "/repo/packages/components/sail/primitives/src/widget/glue.ts"
    )));
}

#[test]
fn app_source_path_recognises_created_app_src() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src/stores");
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(dir.path().join("sail.config.jsonc"), "{}\n").unwrap();
    let leak = src.join("leak.ts");
    std::fs::write(&leak, "export const x = 1;\n").unwrap();
    assert!(
        is_app_source_ts(&leak),
        "created-app src/stores must be app source: {}",
        leak.display()
    );
    assert!(!is_app_source_ts(
        &dir.path().join("node_modules/foo/src/index.ts")
    ));
}

#[test]
fn app_source_lints_csstext_floor() {
    // The hole this job closed: plain studio modules were never dispatched.
    // CSSOM still dual-linted; ambient document.* is TypeEnv::Sail (S2.3).
    let path = Path::new("/repo/apps/studio/src/edit/region-overlay.ts");
    let findings = lint_app_source_source(
        path,
        "el.style.cssText = 'x'; document.createElement('div')",
    );
    let rules: Vec<_> = findings.iter().map(|f| f.rule).collect();
    assert!(
        rules.iter().any(|r| *r == "sail-no-imperative-style"),
        "cssText / .style must fire floor: {findings:?}"
    );
    assert!(
        !rules.iter().any(|r| *r == "sail-no-ambient-dom"),
        "S2.3: createElement must not dual-fire ambient-dom (TypeEnv owns it): {findings:?}"
    );
}

#[test]
fn lint_path_dispatches_app_source_ts() {
    // End-to-end path gate: a synthetic on-disk path shape (no read — we call
    // lint_app_source_source via the same predicate lint_path uses).
    let path = Path::new("/repo/apps/studio/src/foo.ts");
    assert!(is_app_source_ts(path));
    // Ambient DOM dual floor retired (S2.3) — document.querySelector is a
    // TypeEnv error, not a lint finding. CSSOM still hits the style floor.
    let findings = lint_app_source_source(path, "el.style.cssText = 'x'");
    assert!(
        findings
            .iter()
            .any(|f| f.rule == "sail-no-imperative-style"),
        "app-source dispatch must still run imperative-style: {findings:?}"
    );
    // Silence unused import if lint_path is only referenced for docs.
    let _ = lint_path as fn(&Path) -> Vec<crate::Finding>;
}

#[test]
fn author_action_path_and_floor_regime() {
    use crate::{
        is_action_source_path, is_controller_path, is_pack_source_ts, lint_action_source,
        lint_controller_source,
    };
    // Controllers: `el.style.setProperty('--x', v)` still REFUSES (2026-07-18).
    let controller = lint_controller_source(
        Path::new("/repo/packages/components/sail/primitives/src/widget.controller.ts"),
        "el.style.setProperty('--row-y', y)",
    );
    assert_eq!(controller.len(), 1);
    assert_eq!(controller[0].rule, "sail-no-imperative-style");

    // Job 0 — same law everywhere. Catalog stamps are an action surface,
    // not a trusted caste. Path classify + dispatch must lint them.
    let catalog = Path::new("/repo/packages/sail/runtime/src/actions/grid-tracks.ts");
    assert!(
        !is_controller_path(catalog),
        "catalog stamp is not a controller"
    );
    assert!(
        !is_pack_source_ts(catalog),
        "catalog stamp is not pack glue"
    );
    assert!(
        is_action_source_path(catalog),
        "catalog stamp path must be an action surface (no trusted caste)"
    );

    // Author hatch: apps/*/src/actions/**
    let action = Path::new("/repo/apps/demo/src/actions/autofocus.ts");
    assert!(!is_controller_path(action));
    assert!(!is_pack_source_ts(action));
    assert!(is_action_source_path(action));

    // Same CSSOM floor on author and catalog: sail-no-imperative-style.
    // sail-action-no-style may still fire on authors for `.style`, but
    // catalog must not get a weaker sibling.
    let action_style = lint_action_source(action, "el.style.setProperty('--row-y', y)");
    assert!(
        action_style
            .iter()
            .any(|f| f.rule == "sail-no-imperative-style"),
        "author `.style` must fail sail-no-imperative-style: {action_style:?}"
    );
    let catalog_style = lint_action_source(catalog, "el.style.setProperty('--row-y', y)");
    assert!(
        catalog_style
            .iter()
            .any(|f| f.rule == "sail-no-imperative-style"),
        "catalog stamp `.style` must fail sail-no-imperative-style (not a caste): {catalog_style:?}"
    );

    // Legal author action: host listener only.
    let legal = r#"
export const autofocus = (node: Element) => {
  (node as HTMLElement).focus()
}
"#;
    let ok = lint_action_source(action, legal);
    assert!(ok.is_empty(), "{ok:?}");

    // Bench harness path is exempt.
    assert!(!is_action_source_path(Path::new(
        "/repo/tests/perf/frameworks/keyed/sail/src/main.ts"
    )));
}
