//! Compile-time semantic checks that run after the IR has been built.
//!
//! These passes are additive — they do not mutate the IR, they only
//! emit diagnostics. The first pass (and currently only one) is the
//! renderable-expression subset enforced on every interpolation
//! expression in the program.

pub mod controller_parse;
#[cfg(test)]
mod controller_parse_tests;
pub mod controller_purity;
pub mod light_shadow_root;
pub mod paired_controller;
pub mod css_token;
pub mod sx_vars;
pub mod lint_overrides;
pub mod policy;
pub mod expr_subset;
pub mod reserved_names;
pub mod tscheck;
pub mod handler_complexity;
pub(crate) mod js_scan;
pub mod pure_derived;
#[cfg(test)]
mod pure_derived_tests;
pub mod scope_bound;
mod scope_bound_expr;
#[cfg(test)]
mod scope_bound_expr_tests;
pub mod symbol_table;
pub mod scope_member;
pub mod key_identity;
pub mod bind_this_target;
pub mod on_callable;
pub mod for_iterable;

use std::path::Path;

use crate::diagnostics::{CompileError, DiagSeverity, Diagnostic};
use crate::ir::*;

/// Run every post-parse check. Returns the first error, or `Ok(())`.
///
/// Event handlers (`allow_arrow == true`) are excluded from the renderable
/// expr-subset check; they are checked separately by `handler_complexity`.
pub fn run(
    program: &Program,
    source: &str,
    source_path: Option<&Path>,
    controller: Option<&controller_parse::ParsedController>,
    overrides: &[crate::check::lint_overrides::LintOverride],
    route_params: Option<&[String]>,
) -> Result<(), CompileError> {
    visit_program(program, &mut |e| {
        if e.allow_arrow {
            return Ok(());
        }
        expr_subset::check(e)
    })?;
    check_mutations(program)?;
    // Bare names are the language's namespace — reject unknown bare tags
    // (typos / not-yet-language elements) so L1 growth is safe.
    reserved_names::check(program)?;
    handler_complexity::check(program, controller.is_some())?;
    controller_purity::check(controller, overrides)?;
    paired_controller::check(source, source_path)?;
    css_token::check(source, source_path, overrides)?;
    sx_vars::resolve(source, program, controller).map(|_| ())?;
    // `controller` is resolved by the caller: the inline `<controller>` block
    // (SFC) wins, else the paired `.controller.ts`.
    if let Some(c) = controller {
        pure_derived::check(c)?;
        light_shadow_root::check(Some(c))?;
    }
    // Scope-bound checks need the controller's declared names (`scope` /
    // `derived` / `use`). We have them whenever a controller is resolved —
    // the inline `<controller>` block (SFC) OR the paired file — so these run
    // regardless of `source_path` (the checks read the passed `controller`,
    // never the path). Only a bare fragment with NEITHER a controller nor a
    // source_path is skipped: nothing defines its scope to check against, so
    // every free name would false-positive as undeclared.
    let has_scope_context = source_path.is_some() || controller.is_some();
    if has_scope_context {
        scope_bound::check(program, controller, route_params)?;
    }
    // Type-strengthening passes — see crates/sail/compiler/EXPRESSIONS.md.
    // These run after scope_bound so the user gets the "name not declared"
    // error first (it's the more common authoring mistake) before the
    // shape mismatches surface.
    key_identity::check(program)?;
    for_iterable::check(program)?;
    if has_scope_context {
        scope_member::check(program, controller)?;
        bind_this_target::check(program, controller)?;
        on_callable::check(program, controller)?;
    }
    Ok(())
}

/// Gate compilation on the **full Error-severity diagnostic stream**, not just
/// the structural passes [`run`] invokes. `run` covers the expr-subset /
/// scope-bound / type-strengthening passes; it does NOT invoke the
/// Error-severity `sail-lint` rules (`sail-prop-value`, `sail-attr-bool-prop`,
/// `sail-css-content-text`, `controller-*`). Before #255 those rules rejected in
/// the CLI bin's `fatal_diagnostics_gate` but passed silently through library
/// `compile()` — which the daemon (`deck-core/src/view.rs`) and the napi test
/// harness both call. One contract per boundary: a `Severity::Error` finding is
/// a compile error in ALL three artifacts (CLI, daemon, harness).
///
/// Returns the first Error-severity diagnostic as a span-bearing
/// [`CompileError`] (its line/col mapped back to a byte offset so the
/// napi/wasm surface can render coordinates), or `Ok(())` when the stream is
/// clean. Runs after [`run`] in [`crate::compile`], so the structural passes
/// short-circuit first and this catches only the lint Error rules they skip.
pub fn gate(
    program: &Program,
    source: &str,
    source_path: Option<&Path>,
    controller: Option<&controller_parse::ParsedController>,
    overrides: &[crate::check::lint_overrides::LintOverride],
    route_params: Option<&[String]>,
) -> Result<(), CompileError> {
    let fatal = collect(program, source, source_path, controller, overrides, route_params)
        .into_iter()
        .find(|d| d.severity == DiagSeverity::Error);
    let Some(d) = fatal else {
        return Ok(());
    };
    let offset = offset_for_line_col(source, d.line, d.col);
    let mut err = CompileError::new(d.message).with_span(crate::diagnostics::Span::point(offset));
    if let Some(rule) = d.rule {
        err = err.with_category(rule);
    }
    Err(err)
}

/// Map a 1-based (line, col) back to a byte offset into `source`. Inverse of
/// [`crate::diagnostics::line_col`]; used by [`gate`] so a lint finding (which
/// carries line/col, not a span) produces a [`CompileError`] whose span the
/// napi/wasm surface can render. Clamps past-end positions to `source.len()`.
fn offset_for_line_col(source: &str, line: usize, col: usize) -> usize {
    let mut cur_line = 1usize;
    let mut cur_col = 1usize;
    for (i, ch) in source.char_indices() {
        if cur_line == line && cur_col == col {
            return i;
        }
        if ch == '\n' {
            cur_line += 1;
            cur_col = 1;
        } else {
            cur_col += 1;
        }
    }
    source.len()
}

/// Run every post-parse check **collecting all diagnostics** instead of
/// stopping at the first. This is the intelligence behind the LSP + agent
/// tooling: a `.sui` author (or agent) sees every issue in one pass.
///
/// `run()` (above) stays the build-time gate — it short-circuits because a
/// failed build only needs one reason. `collect()` is the editor/agent view.
///
/// Coverage today: every `expr-subset` violation (the common class) plus the
/// first error from each remaining structural/semantic pass, merged with the
/// full multi-finding `sail-lint` stream. `css_token` / `controller_purity`
/// are not run here — they delegate to `sail-lint`, collected below, so
/// running them too would double-report. Per-pass full collection (e.g. all
/// unbound identifiers, not just the first) is a future refinement; the
/// structure here makes it additive.
pub fn collect(
    program: &Program,
    source: &str,
    source_path: Option<&Path>,
    controller: Option<&controller_parse::ParsedController>,
    overrides: &[crate::check::lint_overrides::LintOverride],
    route_params: Option<&[String]>,
) -> Vec<Diagnostic> {
    let mut diags: Vec<Diagnostic> = Vec::new();

    // expr-subset — collect EVERY offending interpolation, not just the first.
    // Closure always returns Ok, so `visit_program` walks the whole tree.
    let _ = visit_program(program, &mut |e: &Expr| {
        if !e.allow_arrow
            && let Err(err) = expr_subset::check(e)
        {
            diags.push(Diagnostic::from_compile_error(&err, source));
        }
        Ok::<(), CompileError>(())
    });

    // Structural / semantic passes — first error from each, continuing past
    // failures so one bad pass doesn't mask the rest.
    // `controller` is resolved by the caller from the inline `<controller>`
    // block (SFC) — the one source, no paired-file fallback.
    push(check_mutations(program), source, &mut diags);
    push(paired_controller::check(source, source_path), source, &mut diags);
    push(sx_vars::resolve(source, program, controller).map(|_| ()), source, &mut diags);
    push(handler_complexity::check(program, controller.is_some()), source, &mut diags);
    if let Some(c) = controller {
        push(pure_derived::check(c), source, &mut diags);
        push(light_shadow_root::check(Some(c)), source, &mut diags);
    }
    // Scope checks run whenever a controller is resolved (inline SFC OR paired
    // file) — not only when a `source_path` is present. See `run` above: the
    // checks read the passed `controller`, never the path.
    let has_scope_context = source_path.is_some() || controller.is_some();
    if has_scope_context {
        // Collect ALL out-of-scope references (one per expression), not just the
        // first — the common "I have three typos" case.
        push_all(scope_bound::collect(program, controller, route_params), source, &mut diags);
    }
    push(key_identity::check(program), source, &mut diags);
    push(for_iterable::check(program), source, &mut diags);
    if has_scope_context {
        push_all(scope_member::collect(program, controller), source, &mut diags);
        push(bind_this_target::check(program, controller), source, &mut diags);
        push(on_callable::check(program, controller), source, &mut diags);
    }

    // Full `sail-lint` pass — the multi-finding source of truth for
    // css_token / css_literal / for_key / event_name / prop_value (template)
    // and controller_purity (the paired `.ts`).
    //
    // `apply_overrides` runs after the lint emits its findings at the
    // rule's natural severity (Error for `sail-css-token`, etc.). The
    // override only downgrades; the rule itself stays strict. This is
    // the single chokepoint the build host uses to opt out of a rule.
    //
    // THE FLOOR is enforced here unconditionally: `enforce_floor` promotes every
    // floor-rule finding to Error regardless of `overrides` (and `apply_overrides`
    // refuses to soften a floor rule), so authored `<style>` / `style=` / `class`
    // / an undeclared stylesheet is a hard Error in every `.sui` — on every host,
    // including one that passes no overrides (wasm/playground). This `collect` is
    // the single path both the build gate (`gate` → here) and the LSP/agent view
    // (`check_all` → here) flow through, so the floor is a compiler property, not
    // opt-in host wiring. L1 element sources bypass this entirely (`compile_element`).
    let lint_path = source_path.unwrap_or_else(|| Path::new("<synthetic>.sui"));
    let template_findings =
        crate::check::lint_overrides::enforce_floor(
            crate::check::lint_overrides::apply_overrides(
                sail_lint::lint_sui_template_source(lint_path, source),
                overrides,
            ),
        );
    for f in template_findings {
        diags.push(finding_to_diag(&f));
    }
    if let Some(c) = controller {
        let mut controller_findings =
            crate::check::lint_overrides::apply_overrides(
                sail_lint::lint_controller_source(&c.path, &c.source),
                overrides,
            );
        // `sail-controller-no-ambient-authority` is `sail.config.json`
        // "network"-aware, but the generic `CONTROLLER_RULES` table
        // (`lint_controller_source`, above) is config-blind — it always ran
        // the rule's default (empty-allowlist) entry. Drop those findings and
        // re-run the network-aware `check_confined` with the resolved
        // allowlist (`Policy::network_for`), same "specially invoked" pattern
        // `css_token` uses in `run()`. This is not a second `sail.config.json`
        // reader: `network_for` re-derives from the one resolver
        // (`sail_lint::config::resolve`) via `Policy::resolve_outcome`.
        controller_findings.retain(|f| {
            f.rule != sail_lint::rules::controller_no_ambient_authority::RULE_ID
        });
        let network = crate::check::policy::Policy::network_for(&c.path);
        let ambient_findings = crate::check::lint_overrides::apply_overrides(
            sail_lint::rules::controller_no_ambient_authority::check_confined(
                &c.path, &c.source, &network,
            ),
            overrides,
        );
        controller_findings.extend(ambient_findings);
        for f in controller_findings {
            diags.push(finding_to_diag(&f));
        }
    }

    diags.sort_by(|a, b| a.line.cmp(&b.line).then(a.col.cmp(&b.col)));
    diags.dedup_by(|a, b| a.line == b.line && a.col == b.col && a.message == b.message);
    diags
}

/// Push a check pass's `Err` (if any) onto `diags` as a [`Diagnostic`].
/// Push every collected [`CompileError`] from a multi-error pass.
fn push_all(errs: Vec<CompileError>, source: &str, diags: &mut Vec<Diagnostic>) {
    for e in errs {
        diags.push(Diagnostic::from_compile_error(&e, source));
    }
}

fn push(r: Result<(), CompileError>, source: &str, diags: &mut Vec<Diagnostic>) {
    if let Err(e) = r {
        diags.push(Diagnostic::from_compile_error(&e, source));
    }
}

/// Convert a `sail-lint` finding to a [`Diagnostic`], folding any `suggest`
/// hint into the message (same rendering the CLI/MCP used).
fn finding_to_diag(f: &sail_lint::Finding) -> Diagnostic {
    let mut message = f.message.clone();
    if let Some(s) = &f.suggest {
        message.push_str("\n  suggest: ");
        message.push_str(s);
    }
    Diagnostic {
        line: f.line,
        col: f.col,
        end_line: f.line,
        end_col: f.col,
        severity: match f.severity {
            sail_lint::Severity::Error => DiagSeverity::Error,
            sail_lint::Severity::Warn => DiagSeverity::Warning,
        },
        message,
        rule: Some(f.rule.to_string()),
    }
}

/// Validate that every `mutation=` attribute contains a valid `ship.X.Y` chain.
fn check_mutations(program: &Program) -> Result<(), CompileError> {
    check_mutations_instrs(&program.template)
}

fn check_mutations_instrs(instrs: &[Instr]) -> Result<(), CompileError> {
    for instr in instrs {
        if let Instr::Element(el) = instr {
            for attr in &el.attrs {
                if let AttrInstr::Mutation { expr, .. } = attr {
                    validate_mutation_expr(&expr.source, expr.span)?;
                }
            }
            check_mutations_instrs(&el.children)?;
        }
        match instr {
            Instr::If(i) => {
                for b in &i.branches { check_mutations_instrs(&b.body)?; }
                if let Some(e) = &i.else_body { check_mutations_instrs(e)?; }
            }
            Instr::For(f) => {
                check_mutations_instrs(&f.body)?;
                if let Some(e) = &f.else_body { check_mutations_instrs(e)?; }
            }
            Instr::Match(m) => {
                for c in &m.cases { check_mutations_instrs(&c.body)?; }
                if let Some(d) = &m.default { check_mutations_instrs(d)?; }
            }
            Instr::WhileLoading(wl) => check_mutations_instrs(&wl.body)?,
            Instr::WhileError(we) => check_mutations_instrs(&we.body)?,
            _ => {}
        }
    }
    Ok(())
}

/// `ship.X.Y` — must have ≥3 dot-separated segments, first is `ship`,
/// each segment is a non-empty ASCII identifier. No parens or args.
fn validate_mutation_expr(expr: &str, span: crate::diagnostics::Span) -> Result<(), CompileError> {
    let parts: Vec<&str> = expr.split('.').collect();
    let valid = parts.len() >= 3
        && parts[0] == "ship"
        && parts.iter().all(|p| {
            !p.is_empty()
                && p.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        });
    if !valid {
        return Err(CompileError::new(format!(
            "mutation= must be ship.<namespace>.<method> (got `{expr}`)"
        ))
        .with_span(span));
    }
    Ok(())
}

/// Walk every `Expr` in the program and invoke `f` on each. Public so
/// the type-strengthening checks (`scope_member`, `key_identity`, ...)
/// can share the traversal instead of re-implementing it five times.
pub fn visit_exprs<F>(program: &Program, f: &mut F) -> Result<(), CompileError>
where
    F: FnMut(&Expr) -> Result<(), CompileError>,
{
    visit_program(program, f)
}

/// Walk every `Expr` in the program and invoke `f` on each.
fn visit_program<F>(program: &Program, f: &mut F) -> Result<(), CompileError>
where
    F: FnMut(&Expr) -> Result<(), CompileError>,
{
    visit_instrs(&program.template, f)?;
    Ok(())
}

fn visit_instrs<F>(instrs: &[Instr], f: &mut F) -> Result<(), CompileError>
where
    F: FnMut(&Expr) -> Result<(), CompileError>,
{
    for instr in instrs {
        visit_instr(instr, f)?;
    }
    Ok(())
}

fn visit_instr<F>(instr: &Instr, f: &mut F) -> Result<(), CompileError>
where
    F: FnMut(&Expr) -> Result<(), CompileError>,
{
    match instr {
        Instr::StaticText { .. } | Instr::Comment { .. } => {}
        Instr::DynamicText { expr } => f(expr)?,
        Instr::Element(el) => {
            for a in &el.attrs {
                visit_attr(a, f)?;
            }
            visit_instrs(&el.children, f)?;
        }
        Instr::If(IfInstr { branches, else_body, .. }) => {
            for b in branches {
                f(&b.cond)?;
                visit_instrs(&b.body, f)?;
            }
            if let Some(eb) = else_body {
                visit_instrs(eb, f)?;
            }
        }
        Instr::For(ForInstr { iterable, key, body, else_body, .. }) => {
            f(iterable)?;
            if let Some(k) = key {
                f(k)?;
            }
            visit_instrs(body, f)?;
            if let Some(eb) = else_body {
                visit_instrs(eb, f)?;
            }
        }
        Instr::Match(MatchInstr { discriminant, cases, default, .. }) => {
            f(discriminant)?;
            for c in cases {
                f(&c.test)?;
                visit_instrs(&c.body, f)?;
            }
            if let Some(d) = default {
                visit_instrs(d, f)?;
            }
        }
        Instr::WhileLoading(wl) => visit_instrs(&wl.body, f)?,
        Instr::WhileError(we) => visit_instrs(&we.body, f)?,
    }
    Ok(())
}

fn visit_attr<F>(attr: &AttrInstr, f: &mut F) -> Result<(), CompileError>
where
    F: FnMut(&Expr) -> Result<(), CompileError>,
{
    match attr {
        AttrInstr::Static { .. } => {}
        AttrInstr::Expr { expr, .. } => f(expr)?,
        AttrInstr::DynamicAttr { name_expr, expr, .. } => {
            f(name_expr)?;
            f(expr)?;
        }
        AttrInstr::BindValue { expr, .. } | AttrInstr::BindChecked { expr, .. } => f(expr)?,
        AttrInstr::BindThis { .. } | AttrInstr::SlotReflectEmpty { .. } => {}
        AttrInstr::On { handler, .. } => f(handler)?,
        AttrInstr::Class { expr, .. } | AttrInstr::Prop { expr, .. } => f(expr)?,
        AttrInstr::Mutation { .. } => {}
        AttrInstr::Use { param, .. } => {
            if let Some(e) = param {
                f(e)?;
            }
        }
        // `motion:` config is statically parsed in the attr
        // compiler — no JS expression body to type-check. Future
        // milestones that allow dynamic configs will need an arm here.
        AttrInstr::Motion { .. } => {}
    }
    Ok(())
}

#[cfg(test)]
mod gate_tests {
    use crate::diagnostics::line_col;
    use crate::parse;

    /// `gate` must turn an Error-severity lint rule that `run` does NOT invoke
    /// (`sail-prop-value`) into a `CompileError` — the #255 contract. Returns a
    /// clean `Ok` for valid sources.
    #[test]
    fn gate_rejects_lint_error_rule_run_skips() {
        let src = r#"<badge size="huge">x</badge>"#;
        let program = parse::parse(src).unwrap();
        let err = super::gate(&program, src, None, None, &[], None).unwrap_err();
        assert_eq!(err.category.as_deref(), Some("sail-prop-value"), "{err:?}");
    }

    #[test]
    fn gate_passes_valid_source() {
        let src = r#"<badge size="md">x</badge>"#;
        let program = parse::parse(src).unwrap();
        super::gate(&program, src, None, None, &[], None).unwrap();
    }

    /// The gate's span must map back to the offending line/col so the
    /// napi/wasm surface renders coordinates. `variant="primary"` sits on
    /// line 1; the round-trip offset → line_col lands there.
    #[test]
    fn gate_error_span_round_trips_to_finding_location() {
        let src = "<div>\n  <badge size=\"huge\">x</badge>\n</div>";
        let program = parse::parse(src).unwrap();
        let err = super::gate(&program, src, None, None, &[], None).unwrap_err();
        let span = err.span.expect("gate error carries a span");
        let (line, _col) = line_col(src, span.start);
        assert_eq!(line, 2, "the bad size is on line 2");
    }

    #[test]
    fn offset_for_line_col_inverts_line_col() {
        let src = "abc\ndef\nghi";
        // Round-trip every offset through line_col → offset_for_line_col.
        for off in 0..=src.len() {
            let (l, c) = line_col(src, off);
            assert_eq!(super::offset_for_line_col(src, l, c), off.min(src.len()));
        }
    }
}
