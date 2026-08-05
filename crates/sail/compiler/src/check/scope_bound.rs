//! Scope-bound expressions rule: every free identifier in a template
//! interpolation or `on:` handler must resolve to a name the runtime
//! actually has. Scope sources, in resolution order:
//!
//!  * `<scope>` prop declarations on the program.
//!  * `setup` factory return keys, `derived` keys, and `on` keys from
//!    the paired controller (when one exists).
//!  * `route` — ONLY when the host GRANTED it (a route file; deck routes
//!    by file location and passes the params at compile time).
//!  * JS / browser globals from a small fixed allowlist.
//!  * Built-in `use:` actions (`action_manifest::BUILTIN_ACTION_NAMES`),
//!    auto-imported from `@ship/sail-runtime` when a template references
//!    them — a bare ESM binding, not a scope read.
//!  * Locals introduced by an arrow's parameter list (handlers can
//!    write `(e) => …`; `e` is in scope inside the body).
//!
//! Anything else triggers a compile error pointing at the unknown name.
//!
//! ## Precedence: scope vs. a built-in action import
//!
//! A scope member (prop, `setup`-returned key, …) and a built-in `use:`
//! action can be given the SAME name — the parser doesn't forbid it. The
//! two live in different namespaces at the type level (one is a scope
//! read, the other an ESM import), but a template expression is flat
//! source text: `worldTransform` in `{worldTransform()}` has to resolve to
//! ONE of them.
//!
//! The precedence is: **the action import always wins.** `emit::expr_rewrite`
//! must leave a built-in action name bare so `runAction(el, worldTransform,
//! …)` receives the real imported function — qualifying it as `$scope.
//! worldTransform` would pass `runAction` a scope value that is almost
//! certainly not a valid action, and `use:` couldn't work at all otherwise
//! (the directive name itself has nowhere else to resolve from). That part
//! of the rule is not new here and is not changing.
//!
//! What IS new: this precedence made a same-named scope member silently,
//! permanently unreachable from every template expression while the
//! collision existed — no diagnostic, just a value that always read as
//! something else. That is the canvas-detail-crash shape: `<sail-canvas>`
//! named a `derived()` value `worldTransform`, identical to the
//! `use:worldTransform` action it fed via
//! `use:worldTransform={worldTransform()}`; the compiler resolved the bare
//! `worldTransform()` call to the ACTION import (zero args) instead of the
//! scope value, and the action crashed reading properties off the
//! `undefined` node it never received. Three composing components broke
//! on mount for weeks with no compile-time signal.
//!
//! So: the compiler now REFUSES the collision outright rather than
//! silently applying the action-wins precedence — see the two checks in
//! `scope_bound_expr::check_expr_inner` for the enforcement:
//!
//!  1. General rule — any template expression that references a name
//!     which is BOTH a declared scope member AND a built-in action name is
//!     a compile error naming both bindings, everywhere that name is read
//!     (not only inside its own `use:` directive).
//!  2. Special case — inside `use:X={expr}`, a bare reference to `X`
//!     itself inside `expr` is ALWAYS an error, even on the (currently
//!     impossible in practice, but not worth trusting) chance there's no
//!     separate scope collision: calling the action you're attaching,
//!     zero-arg, from within its own param thunk, is never a valid
//!     program.
//!
//! A same name that is declared in scope but never referenced from any
//! template expression is not observable by this pass and is not an
//! error — the compiler only refuses ambiguity it can actually see
//! materialize.

use std::collections::BTreeSet;

use crate::check::controller_parse::ParsedController;
use crate::check::js_scan::{
    is_ident_cont, is_ident_start, next_top_separator, skip_balanced, skip_trivia,
};
use crate::diagnostics::CompileError;
use crate::ir::*;

use super::scope_bound_expr::{check_expr, check_use_value_expr};

/// JS/browser globals + framework-known ctx names that are always
/// available without an explicit declaration.
const BUILTINS: &[&str] = &[
    // Setup context. `route` is HOST-GRANTED per file (never ambient), and
    // `navigate` is gone with it.
    "ctx",
    // Namespace head — the `check::scope_member` pass type-checks member
    // access through this so typos are flagged with a useful "did you
    // mean…" message instead of "scope is not declared". `setup`/`derived`
    // are retired: the controller's setup function returns ONE flat scope
    // object now, so there's no section-namespace left to dot into.
    "props",
    // JS standard objects.
    "Math",
    "JSON",
    "Object",
    "Array",
    "String",
    "Number",
    "Boolean",
    "Date",
    "Symbol",
    "Map",
    "Set",
    "WeakMap",
    "WeakSet",
    "Error",
    "RegExp",
    "Promise",
    "BigInt",
    "ArrayBuffer",
    "DataView",
    "Int8Array",
    "Uint8Array",
    "Uint8ClampedArray",
    "Int16Array",
    "Uint16Array",
    "Int32Array",
    "Uint32Array",
    "Float32Array",
    "Float64Array",
    // Standard literals/keywords-as-identifiers.
    "null",
    "undefined",
    "true",
    "false",
    "NaN",
    "Infinity",
    // Standard fns.
    "parseInt",
    "parseFloat",
    "isNaN",
    "isFinite",
    "encodeURIComponent",
    "decodeURIComponent",
    "encodeURI",
    "decodeURI",
    // Browser globals commonly used in handlers.
    "document",
    "window",
    "console",
    "globalThis",
    // Runtime helpers (unwrap variants are imported by the compiled
    // module; templates may also reference them via `use:` results).
    "unwrap",
    "unwrapOr",
    // Built-in `use:` actions — auto-imported when referenced by a `use:`
    // directive (see `action_manifest::BUILTIN_ACTION_NAMES`), so `use:anchor`
    // &c resolve without an explicit controller import. This is the
    // bare-ident subset: the actions a template expression might
    // reference by name. Drift from `BARE_REFERENCED_ACTION_NAMES` is
    // caught by `action_manifest::tests` — the table is the source.
    "anchor",
    "autosize",
    "cellPad",
    "clickOutside",
    "dropzone",
    "eventSpan",
    "gridTracks",
    "maxWidth",
    "pin",
    "portEdge",
    "rect",
    "spotlight",
    "swatch",
    "targetCursor",
    "virtualRow",
    "virtualSpacer",
    "worldTransform",
    "zoomPan",
];

pub fn check(
    program: &Program,
    controller: Option<&ParsedController>,
    route_params: Option<&[String]>,
) -> Result<(), CompileError> {
    let scope = build_scope(program, controller, route_params);
    let mut visitor = Visitor {
        scope,
        errors: Vec::new(),
    };
    visit_program(program, &mut visitor);
    // Errors are pushed in traversal order, so the first is the same one the
    // old short-circuiting visitor returned — the build gate's behaviour is
    // unchanged.
    visitor.errors.into_iter().next().map_or(Ok(()), Err)
}

/// Collect EVERY out-of-scope reference (one diagnostic per expression), for
/// the LSP / agent multi-diagnostic surface. Unlike [`check`] (the build gate,
/// which needs only the first reason), this walks the whole template.
pub fn collect(
    program: &Program,
    controller: Option<&ParsedController>,
    route_params: Option<&[String]>,
) -> Vec<CompileError> {
    let scope = build_scope(program, controller, route_params);
    let mut visitor = Visitor {
        scope,
        errors: Vec::new(),
    };
    visit_program(program, &mut visitor);
    visitor.errors
}

#[derive(Debug)]
pub(super) struct Scope {
    pub(super) names: BTreeSet<String>,
}

impl Scope {
    pub(super) fn contains(&self, name: &str) -> bool {
        self.names.contains(name)
            || BUILTINS.contains(&name)
            // Read from `lower::bindings`, not copied: an auto-imported `use:`
            // action resolves as an ESM binding, so it is in scope by definition.
            || crate::lower::bindings::BUILTIN_ACTIONS.contains(&name)
    }

    pub(super) fn list_for_diagnostic(&self) -> String {
        let mut declared: Vec<&str> = self.names.iter().map(|s| s.as_str()).collect();
        declared.sort();
        if declared.is_empty() {
            "(none — only built-ins like Math and JSON are available)".into()
        } else {
            declared.join(", ")
        }
    }
}

fn build_scope(
    program: &Program,
    controller: Option<&ParsedController>,
    route_params: Option<&[String]>,
) -> Scope {
    let mut names = BTreeSet::new();
    // POSITION GRANT: the host told us this file is a route, so `route` is a
    // legal root (its params arrive from the router at mount). No grant, no
    // name — a component reading `route` gets the teaching refusal.
    if route_params.is_some() {
        names.insert("route".to_string());
    }
    for p in &program.props {
        names.insert(super::super::emit::util::camel_of(&p.name));
    }
    if let Some(c) = controller {
        for member in &c.returned {
            names.insert(member.name.clone());
        }
        names.extend(bound_names_of_param(&c.ctx_param));
        if let Some(host) = &c.host_param {
            names.extend(bound_names_of_param(host));
        }
    }
    Scope { names }
}

/// Names a setup-function parameter binds into its body. A bare
/// identifier (`ctx`) binds itself; a destructuring object pattern
/// (`{ ship, route: r, ...rest }`) binds its LOCAL names (`ship`, `r`,
/// `rest` — not the source property `route`). Array patterns and nested
/// destructuring aren't modeled (rare in a setup param) — their outer
/// bound names are still collected on a best-effort basis.
fn bound_names_of_param(pattern: &str) -> BTreeSet<String> {
    let bytes = pattern.as_bytes();
    let mut out = BTreeSet::new();
    if pattern.is_empty() {
        return out;
    }
    let mut i = skip_trivia(bytes, 0);
    if i < bytes.len() && is_ident_start(bytes[i]) {
        // Bare identifier param.
        let s = i;
        while i < bytes.len() && is_ident_cont(bytes[i]) {
            i += 1;
        }
        out.insert(pattern[s..i].to_string());
        return out;
    }
    if i >= bytes.len() || bytes[i] != b'{' {
        return out;
    }
    let end = skip_balanced(bytes, i, bytes.len())
        .saturating_sub(1)
        .min(bytes.len());
    let mut k = i + 1;
    while k < end {
        k = skip_trivia(bytes, k);
        if k >= end {
            break;
        }
        if bytes[k] == b',' {
            k += 1;
            continue;
        }
        if bytes[k..].starts_with(b"...") {
            k += 3;
            let s = skip_trivia(bytes, k);
            let mut e = s;
            while e < end && is_ident_cont(bytes[e]) {
                e += 1;
            }
            if e > s {
                out.insert(pattern[s..e].to_string());
            }
            k = e;
            continue;
        }
        let key_start = k;
        if !is_ident_start(bytes[k]) {
            k = next_top_separator(bytes, k, end);
            if k < end && bytes[k] == b',' {
                k += 1;
            }
            continue;
        }
        while k < end && is_ident_cont(bytes[k]) {
            k += 1;
        }
        let key = &pattern[key_start..k];
        let after = skip_trivia(bytes, k);
        if after < end && bytes[after] == b':' {
            // Aliased or nested — bind the local name after `:`. A
            // nested destructure (`route: { id }`) doesn't have a single
            // local ident here; best-effort: skip it (rare in practice).
            let s = skip_trivia(bytes, after + 1);
            let mut e = s;
            while e < end && is_ident_cont(bytes[e]) {
                e += 1;
            }
            if e > s {
                out.insert(pattern[s..e].to_string());
            }
            k = next_top_separator(bytes, after, end);
        } else {
            out.insert(key.to_string());
            k = next_top_separator(bytes, k, end);
        }
        if k < end && bytes[k] == b',' {
            k += 1;
        }
    }
    out
}

pub(super) struct Visitor {
    pub(super) scope: Scope,
    /// Every out-of-scope reference (one per expression). `check()` returns
    /// the first; `collect()` returns all.
    pub(super) errors: Vec<CompileError>,
}

// The walk no longer short-circuits on the first error — `check_expr` pushes
// one diagnostic per offending expression and the traversal continues, so
// `collect()` sees every out-of-scope reference. `<for as="}` / `<while-error>`
// locals are still added before the body and removed after, so the scope
// stays balanced regardless of how many errors were found.
fn visit_program(program: &Program, v: &mut Visitor) {
    visit_instrs(&program.template, v);
}

fn visit_instrs(instrs: &[Instr], v: &mut Visitor) {
    for instr in instrs {
        visit_instr(instr, v);
    }
}

fn visit_instr(instr: &Instr, v: &mut Visitor) {
    match instr {
        Instr::StaticText { .. } | Instr::Comment { .. } => {}
        Instr::DynamicText { expr } => check_expr(expr, v, &[]),
        Instr::Element(el) => {
            for a in &el.attrs {
                visit_attr(a, v);
            }
            visit_instrs(&el.children, v);
        }
        Instr::If(IfInstr {
            branches,
            else_body,
            ..
        }) => {
            for b in branches {
                check_expr(&b.cond, v, &[]);
                visit_instrs(&b.body, v);
            }
            if let Some(eb) = else_body {
                visit_instrs(eb, v);
            }
        }
        Instr::For(ForInstr {
            iterable,
            key,
            body,
            else_body,
            var_name,
            index_name,
            ..
        }) => {
            check_expr(iterable, v, &[]);
            // Inside the loop body, the iteration var (and optional
            // index var) are bound.
            let mut locals: Vec<String> = vec![var_name.clone()];
            if let Some(idx) = index_name {
                locals.push(idx.clone());
            }
            if let Some(k) = key {
                check_expr(
                    k,
                    v,
                    locals
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .as_slice(),
                );
            }
            v.scope.names.extend(locals.iter().cloned());
            visit_instrs(body, v);
            for l in &locals {
                v.scope.names.remove(l);
            }
            if let Some(eb) = else_body {
                visit_instrs(eb, v);
            }
        }
        Instr::Match(MatchInstr {
            discriminant,
            cases,
            default,
            ..
        }) => {
            check_expr(discriminant, v, &[]);
            for c in cases {
                check_expr(&c.test, v, &[]);
                visit_instrs(&c.body, v);
            }
            if let Some(d) = default {
                visit_instrs(d, v);
            }
        }
        Instr::When(WhenInstr {
            body,
            else_body,
            ..
        }) => {
            visit_instrs(body, v);
            if let Some(eb) = else_body {
                visit_instrs(eb, v);
            }
        }
        Instr::WhileLoading(wl) => visit_instrs(&wl.body, v),
        Instr::WhileError(we) => {
            // `{#while-error err" of={agents}>` binds `err` inside the body.
            v.scope.names.insert(we.error_var.clone());
            visit_instrs(&we.body, v);
            v.scope.names.remove(&we.error_var);
        }
    }
}

fn visit_attr(attr: &AttrInstr, v: &mut Visitor) {
    match attr {
        AttrInstr::Static { .. } | AttrInstr::SlotReflectEmpty { .. } => {}
        AttrInstr::Expr { expr, .. }
        | AttrInstr::BindValue { expr, .. }
        | AttrInstr::BindChecked { expr, .. } => check_expr(expr, v, &[]),
        AttrInstr::On { handler, .. } => check_expr(handler, v, &[]),
        AttrInstr::Mutation { .. } => {}
        AttrInstr::Use { action, param, .. } => {
            if let Some(e) = param {
                check_use_value_expr(action, e, v);
            }
        }
        // A signal in a `fx:` config field must resolve against scope
        // like any other expression — an undefined name there should read as
        // the same mistake it is anywhere else.
        AttrInstr::Motion { props, timing, .. } => {
            for e in timing.exprs() {
                check_expr(e, v, &[]);
            }
            for p in props {
                for e in p.exprs() {
                    check_expr(e, v, &[]);
                }
            }
        }
    }
}

#[cfg(test)]
mod collect_tests {
    use super::{check, collect};

    #[test]
    fn ship_is_no_longer_ambient() {
        // `ship` is no longer an ambient BUILTIN — it's simply not a name,
        // same as any other undeclared identifier.
        let program = crate::parse::parse("<div>{ship.git.overview()}</div>").expect("parses");
        assert!(
            check(&program, None, None).is_err(),
            "ship must be rejected as an unbound identifier"
        );
    }

    #[test]
    fn route_is_host_granted_never_ambient() {
        let program = crate::parse::parse("<h1>Hello, {route.params.id()}</h1>").expect("parses");
        // Granted (deck passed the params): legal.
        let params = vec!["id".to_string()];
        assert!(
            check(&program, None, Some(&params)).is_ok(),
            "a host-granted route file reads `route` freely"
        );
        // No grant: refused, and the error names the real cause.
        let err = check(&program, None, None).unwrap_err();
        assert!(
            err.message.contains("not registered as a route"),
            "the refusal must teach the file-location convention: {}",
            err.message
        );
    }

    #[test]
    fn navigate_is_no_longer_ambient() {
        let program = crate::parse::parse("<button on:click={() => navigate('/x')}>go</button>")
            .expect("parses");
        assert!(
            check(&program, None, Some(&["id".to_string()])).is_err(),
            "navigate is not granted by route-ness — it is simply not a name"
        );
    }

    #[test]
    fn collect_reports_every_unbound_identifier() {
        // Three unbound idents across three expressions. `collect()` returns
        // all three (the LSP/agent view); `check()` returns only the first
        // (the build gate needs one reason).
        let src = "<p>{alpha}</p><p>{beta}</p><p>{gamma}</p>";
        let program = crate::parse::parse(src).expect("parses");
        let all = collect(&program, None, None);
        assert_eq!(all.len(), 3, "want 3 unbound, got {}: {all:?}", all.len());
        assert!(
            check(&program, None, None).is_err(),
            "check() still flags the first"
        );
    }
}
