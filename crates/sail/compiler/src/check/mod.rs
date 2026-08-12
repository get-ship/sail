//! Compile-time semantic checks that run after the IR has been built.
//!
//! These passes are additive — they do not mutate the IR, they only
//! emit diagnostics. The first pass (and currently only one) is the
//! renderable-expression subset enforced on every interpolation
//! expression in the program.

pub mod binding;
pub mod compose;
pub mod controller_parse;
#[cfg(test)]
mod controller_parse_tests;
pub mod controller_purity;
pub mod css_token;
pub mod expr_subset;
pub mod for_iterable;
pub mod handler_complexity;
pub(crate) mod js_scan;
pub mod key_identity;
pub mod layout;
#[cfg(not(target_arch = "wasm32"))]
pub mod layout_corpus;
pub mod light_shadow_root;
pub mod lint_overrides;
pub mod match_exhaustiveness;
pub mod mount_ambient_shadow;
pub mod on_callable;
pub mod paired_controller;
pub mod partials;
pub mod policy;
pub mod pure_derived;
#[cfg(test)]
mod pure_derived_tests;
pub mod query_keys;
pub mod reserved_names;
pub mod scope_bound;
mod scope_bound_expr;
#[cfg(test)]
mod scope_bound_expr_tests;
pub mod scope_member;
pub mod semantic_a11y;
pub mod signal_called;
pub mod symbol_table;
pub mod tscheck;

use std::path::Path;

use crate::diagnostics::{CompileError, DiagSeverity, Diagnostic, DiagnosticTier};
use crate::ir::*;

const RULE_CHECK_GATE: &str = "sail-internal-check-gate";
const RULE_MUTATION: &str = "sail-mutation";

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
    let profile = std::env::var_os("SAIL_LINT_PROFILE").is_some();
    macro_rules! pass {
        ($name:literal, $body:expr) => {{
            // Instant::now panics on wasm32 (no platform clock) — profiling
            // is a native-only affordance.
            #[cfg(not(target_arch = "wasm32"))]
            let __t = std::time::Instant::now();
            let __r = $body;
            #[cfg(not(target_arch = "wasm32"))]
            if profile {
                eprintln!("sail-check: {} {:?}", $name, __t.elapsed());
            }
            #[cfg(target_arch = "wasm32")]
            let _ = profile;
            __r
        }};
    }
    pass!(
        "expr-subset",
        visit_program(program, &mut |e| {
            if e.allow_arrow {
                return Ok(());
            }
            expr_subset::check(e)
        })
    )?;
    // SPEC-PARTIAL-1: author-source checks (leakage, attrs-vs-type, collision).
    // Expansion already ran in preprocess; this inspects the original text.
    pass!("partials", partials::check(source, controller, source_path))?;
    pass!("mutations", check_mutations(program))?;
    // Bare names are the language's namespace — reject unknown bare tags
    // (typos / not-yet-language elements) so L1 growth is safe.
    pass!("reserved-names", reserved_names::check(program))?;
    pass!(
        "handler-complexity",
        handler_complexity::check(program, controller.is_some())
    )?;
    pass!(
        "controller-purity",
        controller_purity::check(controller, overrides)
    )?;
    pass!(
        "paired-controller",
        paired_controller::check(source, source_path)
    )?;
    pass!(
        "css-token",
        css_token::check(source, source_path, overrides)
    )?;
    pass!(
        "style-bind",
        binding::resolve(source, program, controller).map(|_| ())
    )?;
    // `controller` is resolved by the caller: the inline `<controller>` block
    // (SFC) wins, else the paired `.controller.ts`.
    if let Some(c) = controller {
        pass!("pure-derived", pure_derived::check(c))?;
        pass!("light-shadow-root", light_shadow_root::check(Some(c)))?;
        pass!("query-keys", query_keys::check(&c.source))?;
        pass!(
            "mount-ambient-shadow",
            mount_ambient_shadow::check(&c.source)
        )?;
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
        pass!(
            "scope-bound",
            scope_bound::check(program, controller, route_params)
        )?;
    }
    // Type-strengthening passes. These run after scope_bound so the user
    // gets the "name not declared" error first (more common authoring
    // mistake) before shape mismatches surface.
    pass!("key-identity", key_identity::check(program))?;
    pass!("for-iterable", for_iterable::check(program))?;
    if has_scope_context {
        pass!("scope-member", scope_member::check(program, controller))?;
        // A reactive value rendered without its `()` — the silent one: it
        // emitted the getter's SOURCE into the DOM and warned about nothing.
        pass!("signal-called", signal_called::check(program, controller))?;
        pass!("on-callable", on_callable::check(program, controller))?;
    }
    // SPEC-ARIA-1: icon-only buttons require semantic prop `label`.
    pass!("semantic-a11y", semantic_a11y::check(program))?;
    Ok(())
}

/// Gate compilation on the **full Error-severity diagnostic stream**, not just
/// the structural passes [`run`] invokes. `run` covers the expr-subset /
/// scope-bound / type-strengthening passes; it does NOT invoke the
/// Error-severity `sail-lint` rules (`sail-prop-value`, `sail-attr-bool-prop`,
/// `sail-css-content-text`, `controller-*`). Before #255 those rules rejected in
/// the CLI bin's `fatal_diagnostics_gate` but passed silently through library
/// `compile()` — which the daemon (`deck-core/src/route.rs`) and the napi test
/// harness both call. One contract per boundary: a `Severity::Error` finding is
/// a compile error in ALL three artifacts (CLI, daemon, harness).
///
/// The type gate (JOB-E1, `tscheck`) joins this same stream — DOM globals in
/// Pure regions, action-surface law, `@ts-ignore` bans — so `compile()` hard-
/// fails identically to `check_all` / MCP / LSP OnSave.
///
/// Returns the first Error-severity diagnostic as a span-bearing
/// [`CompileError`] (its line/col mapped back to a byte offset so the
/// napi/wasm surface can render coordinates), or `Ok(())` when the stream is
/// clean. Runs after [`run`] in [`crate::compile`], so the structural passes
/// short-circuit first and this catches only the lint Error rules they skip.
///
/// `author_source` is the original `.sail` text (controller still present) so
/// the type gate can map spans onto author lines. `source` remains the
/// stripped template text used by structural passes.
pub fn gate(
    program: &Program,
    source: &str,
    author_source: &str,
    source_path: Option<&Path>,
    controller: Option<&controller_parse::ParsedController>,
    overrides: &[crate::check::lint_overrides::LintOverride],
    route_params: Option<&[String]>,
) -> Result<(), CompileError> {
    let fatal = collect(
        program,
        source,
        author_source,
        source_path,
        controller,
        overrides,
        route_params,
        DiagnosticTier::OnSave,
    )
    .into_iter()
    .find(|d| d.severity == DiagSeverity::Error);
    let Some(d) = fatal else {
        return Ok(());
    };
    // Prefer author_source for offset mapping so type-gate lines in the
    // controller region land on the right byte (stripped blanks the block).
    let offset = offset_for_line_col(author_source, d.line, d.col);
    let mut err = CompileError::new(d.message)
        .with_span(crate::diagnostics::Span::point(offset))
        .with_category(RULE_CHECK_GATE);
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
/// tooling: a `.sail` author (or agent) sees every issue in one pass.
///
/// `run()` (above) stays the build-time gate — it short-circuits because a
/// failed build only needs one reason. `collect()` is the editor/agent view.
///
/// `tier` (SPEC-TOOLING-1 §WI6) selects the stream:
/// - [`DiagnosticTier::OnType`] — parse / vocabulary / semantic; **no taffy**,
///   **no type gate** (LSP keystrokes stay fast)
/// - [`DiagnosticTier::OnSave`] — full stream including layout + compose +
///   type gate (JOB-E1)
///
/// `author_source` is the original `.sail` text (inline `<controller>` still
/// present). Structural passes use `source` (controller blanked); the type
/// gate re-extracts the controller region from `author_source` so spans land
/// on author lines.
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
    author_source: &str,
    source_path: Option<&Path>,
    controller: Option<&controller_parse::ParsedController>,
    overrides: &[crate::check::lint_overrides::LintOverride],
    route_params: Option<&[String]>,
    tier: DiagnosticTier,
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
    diags.extend(partials::collect(source, controller, source_path));
    push(
        paired_controller::check(source, source_path),
        source,
        &mut diags,
    );
    push(
        binding::resolve(source, program, controller).map(|_| ()),
        source,
        &mut diags,
    );
    push(
        handler_complexity::check(program, controller.is_some()),
        source,
        &mut diags,
    );
    if let Some(c) = controller {
        push(pure_derived::check(c), source, &mut diags);
        push(light_shadow_root::check(Some(c)), source, &mut diags);
        push(query_keys::check(&c.source), source, &mut diags);
        push(mount_ambient_shadow::check(&c.source), source, &mut diags);
    }
    // Scope checks run whenever a controller is resolved (inline SFC OR paired
    // file) — not only when a `source_path` is present. See `run` above: the
    // checks read the passed `controller`, never the path.
    let has_scope_context = source_path.is_some() || controller.is_some();
    if has_scope_context {
        // Collect ALL out-of-scope references (one per expression), not just the
        // first — the common "I have three typos" case.
        push_all(
            scope_bound::collect(program, controller, route_params),
            source,
            &mut diags,
        );
    }
    push(key_identity::check(program), source, &mut diags);
    push(for_iterable::check(program), source, &mut diags);
    push_all(semantic_a11y::collect(program), source, &mut diags);
    if has_scope_context {
        push_all(
            scope_member::collect(program, controller),
            source,
            &mut diags,
        );
        push_all(
            signal_called::collect(program, controller),
            source,
            &mut diags,
        );
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
    // / an undeclared stylesheet is a hard Error in every `.sail` — on every host,
    // including one that passes no overrides (wasm/playground). This `collect` is
    // the single path both the build gate (`gate` → here) and the LSP/agent view
    // (`check_all` → here) flow through, so the floor is a compiler property, not
    // opt-in host wiring. L1 element sources bypass this entirely (`compile_element`).
    let lint_path = source_path.unwrap_or_else(|| Path::new("<synthetic>.sail"));
    // ONE answer about the capability tier, for the build gate and the editor
    // alike. Both `compile()` (via `gate`) and `analyze()` reach the lint
    // stream through this function, so resolving the package's declared
    // capabilities HERE is what stops `analyze` from squiggling a policy the
    // build never applied. Host-supplied overrides win; see
    // `Policy::capability_defaults_for`.
    let defaults = crate::check::policy::Policy::capability_defaults_for(lint_path, overrides);
    let mut resolved = overrides.to_vec();
    resolved.extend(defaults);
    let overrides = &resolved[..];
    // Order: natural findings → per-rule overrides (incl. off kill switch) →
    // unconditional styling floor → SPEC-LINTS-1 strict / lint.severity floor
    // (Warn→Error). Off must run before the global floor so the escape hatch
    // still silences under strict.
    let template_findings = crate::check::lint_overrides::apply_strict_or_severity_floor(
        crate::check::lint_overrides::enforce_floor(crate::check::lint_overrides::apply_overrides(
            sail_lint::lint_sail_template_source(lint_path, source),
            overrides,
        )),
        lint_path,
    );
    for f in template_findings {
        diags.push(finding_to_diag(&f));
    }
    if let Some(c) = controller {
        let mut controller_findings = crate::check::lint_overrides::apply_overrides(
            sail_lint::lint_controller_source(&c.path, &c.source),
            overrides,
        );
        // `sail-controller-no-ambient-authority` is `sail.config.json`
        // "network"/"unconfined"-aware, but the generic `CONTROLLER_RULES`
        // table (`lint_controller_source`, above) is config-blind — it always
        // ran the rule's default (confined, empty-allowlist) entry. Drop
        // those findings and re-run the config-aware `check_confined` with
        // the resolved allowlist (`Policy::network_for`) and capability
        // (`Policy::unconfined_for`), same "specially invoked" pattern
        // `css_token` uses in `run()`. This is not a second
        // `sail.config.json` reader: both re-derive from the one resolver
        // (`sail_lint::config::resolve`) via `Policy::resolve_outcome`.
        controller_findings
            .retain(|f| f.rule != sail_lint::rules::controller_no_ambient_authority::RULE_ID);
        let network = crate::check::policy::Policy::network_for(&c.path);
        let unconfined = crate::check::policy::Policy::unconfined_for(&c.path);
        let ambient_findings = crate::check::lint_overrides::apply_overrides(
            sail_lint::rules::controller_no_ambient_authority::check_confined(
                &c.path, &c.source, &network, unconfined,
            ),
            overrides,
        );
        controller_findings.extend(ambient_findings);
        let controller_findings = crate::check::lint_overrides::apply_strict_or_severity_floor(
            controller_findings,
            &c.path,
        );
        for f in controller_findings {
            diags.push(finding_to_diag(&f));
        }
    }

    // Layout + compose (taffy) — SPEC-TOOLING-1 §WI6: OnSave only.
    // On-type must never compute a box tree; typing a 500-line buffer must
    // not pay for multi-viewport layout. Compose rebuilds taffy today, so
    // it rides the same gate (not a separate "reuse" of layout's tree).
    if tier.includes_layout() {
        // Layout diagnostics — build the same box tree the runtime will build
        // and report collapsed / overflow findings. Soft findings stay
        // warnings unless the package sets `"layout": "error"` (studio
        // profile) — then gate-gap kinds promote to Error and fail
        // `compile` / `sail check`. Must run on the preprocessed source so
        // class tokens line up with the CSS the compiler emits; spans map
        // back to the author's original source.
        diags.extend(layout::collect_for(source, source_path));

        // Compose auto-fix — sibling rhythm (spacing) consistency. Emits
        // `Fix` severity diagnostics that the `sail_edit` path turns into
        // source rewrites.
        diags.extend(compose::collect(source, source_path, overrides));
    }

    // Type gate (JOB-E1) — OnSave only. Never OnType: tsgo is too heavy for
    // the LSP keystroke path. Joins the same Error stream as lint (no second
    // gate). One tsgo invocation per call over this file's controller region.
    if matches!(tier, DiagnosticTier::OnSave) {
        diags.extend(tscheck::collect_sail_type_gate(author_source, source_path));
    }

    // Style match exhaustiveness — a `match` on a closed-union scope prop
    // that leaves members arm-less warns (never gates; partial matches are a
    // legitimate way to leave a variant unstyled). Cheap; runs on-type too.
    diags.extend(match_exhaustiveness::collect(program, source));

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
                for b in &i.branches {
                    check_mutations_instrs(&b.body)?;
                }
                if let Some(e) = &i.else_body {
                    check_mutations_instrs(e)?;
                }
            }
            Instr::For(f) => {
                check_mutations_instrs(&f.body)?;
                if let Some(e) = &f.else_body {
                    check_mutations_instrs(e)?;
                }
            }
            Instr::Match(m) => {
                for c in &m.cases {
                    check_mutations_instrs(&c.body)?;
                }
                if let Some(d) = &m.default {
                    check_mutations_instrs(d)?;
                }
            }
            Instr::When(w) => {
                check_mutations_instrs(&w.body)?;
                if let Some(e) = &w.else_body {
                    check_mutations_instrs(e)?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

/// `<client>.<method>` (optionally nested) — ≥2 dot-separated segments,
/// first segment is a host-injected client name, each segment a non-empty
/// ASCII identifier. No `ship.` framework prefix; no parens or args.
fn validate_mutation_expr(expr: &str, span: crate::diagnostics::Span) -> Result<(), CompileError> {
    let parts: Vec<&str> = expr.split('.').collect();
    let valid = parts.len() >= 2
        && parts.iter().all(|p| {
            !p.is_empty()
                && p.chars()
                    .next()
                    .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
                && p.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        });
    if !valid {
        return Err(CompileError::new(format!(
            "mutation= must be <client>.<method> (got `{expr}`)"
        ))
        .with_span(span)
        .with_category(RULE_MUTATION));
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
        Instr::If(IfInstr {
            branches,
            else_body,
            ..
        }) => {
            for b in branches {
                f(&b.cond)?;
                visit_instrs(&b.body, f)?;
            }
            if let Some(eb) = else_body {
                visit_instrs(eb, f)?;
            }
        }
        Instr::For(ForInstr {
            iterable,
            key,
            body,
            else_body,
            ..
        }) => {
            f(iterable)?;
            if let Some(k) = key {
                f(k)?;
            }
            visit_instrs(body, f)?;
            if let Some(eb) = else_body {
                visit_instrs(eb, f)?;
            }
        }
        Instr::Match(MatchInstr {
            discriminant,
            cases,
            default,
            ..
        }) => {
            f(discriminant)?;
            for c in cases {
                if let MatchTest::Expr(e) = &c.test {
                    f(e)?;
                }
                visit_instrs(&c.body, f)?;
            }
            if let Some(d) = default {
                visit_instrs(d, f)?;
            }
        }
        Instr::When(w) => {
            visit_instrs(&w.body, f)?;
            if let Some(e) = &w.else_body {
                visit_instrs(e, f)?;
            }
        }
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
        AttrInstr::BindValue { expr, .. } | AttrInstr::BindChecked { expr, .. } => f(expr)?,
        AttrInstr::SlotReflectEmpty { .. } => {}
        AttrInstr::On { handler, .. } => f(handler)?,
        AttrInstr::Mutation { .. } => {}
        AttrInstr::Use { param, .. } => {
            if let Some(e) = param {
                f(e)?;
            }
        }
        // A `fx:` config is a statically-parsed object, but any of its
        // fields may hold a signal (`duration: speed()`). Those are ordinary
        // expressions and get the ordinary treatment.
        AttrInstr::Motion { props, timing, .. } => {
            for e in timing.exprs() {
                f(e)?;
            }
            for p in props {
                for e in p.exprs() {
                    f(e)?;
                }
            }
        }
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
        let err = super::gate(&program, src, src, None, None, &[], None).unwrap_err();
        assert_eq!(err.category.as_deref(), Some("sail-prop-value"), "{err:?}");
    }

    #[test]
    fn gate_passes_valid_source() {
        let src = r#"<badge size="md">x</badge>"#;
        let program = parse::parse(src).unwrap();
        super::gate(&program, src, src, None, None, &[], None).unwrap();
    }

    /// The gate's span must map back to the offending line/col so the
    /// napi/wasm surface renders coordinates. `variant="primary"` sits on
    /// line 1; the round-trip offset → line_col lands there.
    #[test]
    fn gate_error_span_round_trips_to_finding_location() {
        let src = "<div>\n  <badge size=\"huge\">x</badge>\n</div>";
        let program = parse::parse(src).unwrap();
        let err = super::gate(&program, src, src, None, None, &[], None).unwrap_err();
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
