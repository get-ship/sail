//! Tests for SUI lint rules.

use std::path::Path;

use crate::{
    is_controller_path, lint_controller_source,
    rules::{controller_purity, controller_state_cast, dead_affordance},
};

fn lint(source: &str) -> Vec<crate::Finding> {
    lint_controller_source(Path::new("/fake/foo.controller.ts"), source)
}

#[test]
fn clean_controller_passes() {
    let src = r#"
        import { controller, signal } from '@ship/sui-runtime'
        export default controller({
          mount: (state, host) => {
            const count = signal(0)
            host.addEventListener('click', () => count.set(count() + 1))
            return () => {}
          },
        })
    "#;
    let findings = lint(src);
    assert!(findings.is_empty(), "clean controller should pass: {findings:?}");
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
    assert!(findings[0].message.contains(".innerHTML"), "msg: {}", findings[0].message);
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
    assert!(findings.is_empty(), "textContent must be allowed: {findings:?}");
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
    assert!(findings.is_empty(), "computed access not currently inspected");
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
    assert!(is_controller_path(Path::new("/abs/sui-button.controller.ts")));
    assert!(!is_controller_path(Path::new("foo/bar.ts")));
    assert!(!is_controller_path(Path::new("foo/bar.controller.tsx")));
    assert!(!is_controller_path(Path::new("foo/bar.sui")));
    assert!(!is_controller_path(Path::new("foo/render.ts")));
}

#[test]
fn flags_synthetic_innerhtml_offender() {
    // Synthetic source matching the pattern Phase 1.5 cleaned up. Real
    // controllers in the repo are now all clean, so a smoke against any
    // specific file would pass for the wrong reason. This keeps the rule
    // exercised end-to-end against the kind of code it's meant to catch.
    let path = std::path::PathBuf::from("synthetic/sui-foo/sui-foo.controller.ts");
    let source = r#"
import { controller } from '@ship/sui-runtime'
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
        import { controller } from '@ship/sui-runtime'
        import type { Scope } from './foo.sui'
        export default controller<Scope>({
          mount(state, host) {
            const scope = state as unknown as Scope
            return () => { scope.open.set(false) }
          },
        })
    "#;
    let findings = lint(src);
    assert_eq!(findings.len(), 1, "expected one state-cast finding, got {findings:?}");
    assert_eq!(findings[0].rule, controller_state_cast::RULE_ID);
    assert!(findings[0].message.contains("Scope"), "msg: {}", findings[0].message);
    assert!(findings[0].suggest.is_some(), "should suggest $signals access");
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
    assert_eq!(findings.len(), 1, "intersection cast should report once, got {findings:?}");
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
        findings.iter().any(|f| f.rule == controller_state_cast::RULE_ID),
        "ScopeView alias cast should report, got {findings:?}"
    );
}

#[test]
fn typed_signals_access_passes_state_cast() {
    // The post-#121 pattern: no cast, read via $signals — must NOT report.
    let src = r#"
        import { controller } from '@ship/sui-runtime'
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
    assert!(findings.is_empty(), "typed $signals access must pass: {findings:?}");
}

fn lint_tpl(source: &str) -> Vec<crate::Finding> {
    crate::lint_sui_template_source(Path::new("/fake/page.sui"), source)
        .into_iter()
        .filter(|f| f.rule == crate::rules::attr_bool_prop::RULE_ID)
        .collect()
}

#[test]
fn attr_bool_on_sui_component_reports() {
    let f = lint_tpl(r#"<alert attr:dismissible={busy()}>Go</alert>"#);
    assert_eq!(f.len(), 1, "attr:dismissible on alert is a bool prop: {f:?}");
    assert!(f[0].message.contains("prop:dismissible"), "msg: {}", f[0].message);
}

#[test]
fn attr_bool_multiline_tag_reports() {
    let f = lint_tpl("<badge\n  tone=\"primary\"\n  attr:pill={busy()}>x</badge>");
    assert_eq!(f.len(), 1, "multi-line tag bool attr should report: {f:?}");
}

#[test]
fn prop_bool_passes() {
    let f = lint_tpl(r#"<alert prop:dismissible={busy()}>Go</alert>"#);
    assert!(f.is_empty(), "prop:dismissible is correct: {f:?}");
}

#[test]
fn attr_on_native_option_passes() {
    // <option> isn't in the registry → not flagged (attr:selected is correct there).
    let f = lint_tpl(r#"<option value="x" attr:selected={sel()}>x</option>"#);
    assert!(f.is_empty(), "native element must not be flagged: {f:?}");
}

#[test]
fn attr_on_string_prop_passes() {
    // variant is a string/enum prop, not bool → not flagged.
    let f = lint_tpl(r#"<sui-button attr:variant={v()}>x</sui-button>"#);
    assert!(f.is_empty(), "string prop attr not flagged by this rule: {f:?}");
}

#[test]
fn static_bool_flag_passes() {
    // Bare flag / static attr (no brace binding) is not this rule's concern.
    let f = lint_tpl(r#"<sui-dialog open></sui-dialog>"#);
    assert!(f.is_empty(), "bare flag attr not flagged: {f:?}");
}

// ── sui-dead-affordance ────────────────────────────────────────────────────
// Driven through the rule's `check` directly with a synthetic path (no paired
// controller on disk), so unrelated css/space/dimension warnings don't muddy
// the assertions.

fn lint_dead(src: &str) -> Vec<crate::Finding> {
    dead_affordance::check(Path::new("/fake/sui-x/sui-x.sui"), src)
}

#[test]
fn dead_affordance_unread_prop_warns() {
    let f = lint_dead("<prop name=\"ghost\" type=\"bool\" default=\"false\" />\n<div>hi</div>");
    assert_eq!(f.len(), 1, "{f:?}");
    assert_eq!(f[0].rule, dead_affordance::RULE_ID);
    assert!(f[0].message.contains("`ghost`"));
}

#[test]
fn dead_affordance_interpolated_prop_passes() {
    let f = lint_dead("<prop name=\"label\" type=\"string\" default=\"\" />\n<span>{label()}</span>");
    assert!(f.is_empty(), "{f:?}");
}

#[test]
fn dead_affordance_directive_bound_prop_passes() {
    let f = lint_dead(
        "<prop name=\"disabled\" type=\"bool\" default=\"false\" />\n\
         <button attr:disabled={disabled()}>go</button>",
    );
    assert!(f.is_empty(), "{f:?}");
}

#[test]
fn dead_affordance_reflect_prop_styled_via_host_selector_passes() {
    // sui-skeleton idiom: prop read purely through `:host([shape=…])` CSS, no
    // interpolation, no controller. Must NOT be flagged.
    let f = lint_dead(
        "<prop name=\"shape\" values=\"text|line\" default=\"text\" reflect />\n\
         <style>:host([shape='block']) { display: block; }</style>\n<div></div>",
    );
    assert!(f.is_empty(), "reflect-via-CSS is a real read: {f:?}");
}

#[test]
fn dead_affordance_docblock_only_is_dead() {
    // A comment mention must not count as a read.
    let f = lint_dead(
        "<!-- {size()} and attr:size= are described here -->\n\
         <prop name=\"size\" values=\"sm|md\" default=\"md\" reflect />\n<div>x</div>",
    );
    assert_eq!(f.len(), 1, "comment mentions are not reads: {f:?}");
}

#[test]
fn dead_affordance_substring_does_not_count() {
    // `loading` must not satisfy a `load` prop.
    let f = lint_dead("<prop name=\"load\" type=\"bool\" default=\"false\" />\n<span>{loading()}</span>");
    assert_eq!(f.len(), 1, "{f:?}");
}

#[test]
fn dead_affordance_accessor_in_handler_passes() {
    let f = lint_dead(
        "<prop name=\"open\" type=\"bool\" default=\"false\" />\n\
         <button on:click={() => toggle(open())}>x</button>",
    );
    assert!(f.is_empty(), "{f:?}");
}

#[test]
fn dead_affordance_event_declaration_unread_warns() {
    let f = lint_dead("<event name=\"ping\" />\n<div>x</div>");
    assert_eq!(f.len(), 1, "{f:?}");
    assert!(f[0].message.contains("event `ping`"));
}

#[test]
fn dead_affordance_severity_is_warn() {
    let f = lint_dead("<prop name=\"ghost\" type=\"bool\" default=\"false\" />\n<div>hi</div>");
    assert_eq!(f.len(), 1);
    assert_eq!(f[0].severity, crate::Severity::Warn);
}
