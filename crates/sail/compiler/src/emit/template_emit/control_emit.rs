// FILE_CAP_OVERRIDE: five sibling control-flow blocks ({#if}, {#for},
// {#match}, {#while-loading}, {#while-error}) all share a hoist /
// hydrate / boundary-key plumbing pattern that's only legible
// side-by-side. Splitting each into its own file would force the
// shared `emit_branch` + `emit_boundary_hydrate` helpers behind a
// `pub(super)` boundary and obscure the structural similarity.

//! Control-flow emit: `{#if}`, `{#for}`, `{#match}`, `{#while-loading}`,
//! and `{#while-error}`.
//!
//! Phase 3b.5 (gate H): control-flow boundaries are direct-emit through
//! the unified `hydrate(root, ctx)` function on the route's setup return.
//! Each `{#if}` / `{#for}` / `{#match}` boundary still hoists a
//! `__wireBoundary_b<N>` const inside `setup(ctx)` so the legacy
//! `__mount` path can call it inline; the hydrate function looks up the
//! SSR'd `<sail-{kind} key="b<N>">` element via `collectElementMarkers`,
//! swaps it for a Comment anchor, and invokes the same hoisted helper.
//! `__b<N>_attach` named properties are no longer emitted — `attach()`
//! drives boundaries through hydrate now, not through `wireBoundaries`.
//!
//! `{#while-loading}` / `{#while-error}` follow the same shape via
//! `hydrateWhileLoading` / `hydrateWhileError` (gate G).

use super::super::writer::{js_string, Writer};
use super::TemplateEmitter;
use crate::ir::*;
use crate::lower::for_uses_row::body_references_for_vars;
use crate::lower::row_field_uses::RowUse;

/// Partition the for-loop's inline-update statements into per-field
/// diff-guarded groups. Returns:
///
/// - `(Some(body), _)` when every stmt's row-use is `FieldsOnly` or
///   `None`. Body groups stmts by their (sorted) field set and wraps
///   each group in `if (__old.f !== $scope.var.f) {…}`. `None` stmts
///   are dropped entirely from `update()` — a constant expression's
///   textContent assignment is idempotent and never needs re-firing.
/// - `(None, body)` when any stmt uses the row in a non-decomposable
///   shape (`row` bare, `row[…]`, `row?.id`, spread, fn-call arg) —
///   we can't prove a particular field change is sufficient, so the
///   for-emitter falls back to the unguarded join (always re-fire).
///
/// The bench's hot path matches the gated arm: every D5 inline stmt
/// reads `row.id` or `row.label` cleanly, so the bench's
/// `update()`-every-10th-row benchmark skips the id-bound textContent
/// and setAttribute calls when the id is unchanged.
pub(super) fn partition_inline_updates(
    updates: &[(String, RowUse)],
    var: &str,
) -> (Option<String>, String) {
    let fallback = updates
        .iter()
        .map(|(s, _)| s.as_str())
        .collect::<Vec<_>>()
        .join(" ");

    if updates.iter().any(|(_, u)| matches!(u, RowUse::Whole)) {
        return (None, fallback);
    }

    // Group stmts by their sorted field set. `None`-row stmts (constants
    // like `{42}` that classify as static-row-expr but don't touch the
    // row binder) are dropped — they're idempotent on update.
    let mut groups: Vec<(Vec<String>, Vec<&str>)> = Vec::new();
    for (stmt, use_) in updates {
        match use_ {
            RowUse::None => continue,
            RowUse::Whole => unreachable!("filtered above"),
            RowUse::FieldsOnly(fs) => {
                let mut key = fs.clone();
                key.sort();
                if let Some(slot) = groups.iter_mut().find(|(k, _)| k == &key) {
                    slot.1.push(stmt.as_str());
                } else {
                    groups.push((key, vec![stmt.as_str()]));
                }
            }
        }
    }

    if groups.is_empty() {
        // All stmts were None-row constants. Nothing to re-fire in update().
        return (Some(String::new()), fallback);
    }

    let body = groups
        .into_iter()
        .map(|(fields, stmts)| {
            let cond = fields
                .iter()
                .map(|f| format!("__old.{f} !== $scope.{var}.{f}"))
                .collect::<Vec<_>>()
                .join(" || ");
            let stmts_joined = stmts.join(" ");
            format!("if ({cond}) {{ {stmts_joined} }}")
        })
        .collect::<Vec<_>>()
        .join(" ");
    (Some(body), fallback)
}

/// Does any instruction in this subtree carry a `<slot reflect-empty>`?
///
/// A `<slot reflect-empty>` emits `bindSlotEmpty(slot, container, attr)`, which
/// reflects an attr on the HOST. Inside a hoisted `__wireBoundary_b<N>` helper
/// (the `{#if}`/`{#for}`/`{#match}` branch closures live at factory scope, NOT
/// in `__mount(container)`), `container` is unbound — so when a boundary's
/// subtree contains such a slot, the helper must take `container` as a param
/// and every call site must pass it. This walk gates that threading so
/// boundaries WITHOUT a reflect-empty slot stay byte-identical (no churn).
///
/// Recurses through nested boundaries: a slot in a nested `{#if}` makes the
/// nested helper take `container` AND the outer helper (whose branch holds the
/// nested call site, which now passes `container`) take it too.
fn instrs_have_reflect_empty_slot(instrs: &[Instr]) -> bool {
    instrs.iter().any(instr_has_reflect_empty_slot)
}

fn instr_has_reflect_empty_slot(instr: &Instr) -> bool {
    match instr {
        Instr::Element(e) => {
            e.attrs
                .iter()
                .any(|a| matches!(a, AttrInstr::SlotReflectEmpty { .. }))
                || instrs_have_reflect_empty_slot(&e.children)
        }
        Instr::If(i) => {
            i.branches
                .iter()
                .any(|b| instrs_have_reflect_empty_slot(&b.body))
                || i.else_body
                    .as_deref()
                    .is_some_and(instrs_have_reflect_empty_slot)
        }
        Instr::For(f) => {
            instrs_have_reflect_empty_slot(&f.body)
                || f.else_body
                    .as_deref()
                    .is_some_and(instrs_have_reflect_empty_slot)
        }
        Instr::Match(m) => {
            m.cases
                .iter()
                .any(|c| instrs_have_reflect_empty_slot(&c.body))
                || m.default
                    .as_deref()
                    .is_some_and(instrs_have_reflect_empty_slot)
        }
        Instr::WhileLoading(w) => instrs_have_reflect_empty_slot(&w.body),
        Instr::WhileError(w) => instrs_have_reflect_empty_slot(&w.body),
        Instr::StaticText { .. } | Instr::DynamicText { .. } | Instr::Comment { .. } => false,
    }
}

impl TemplateEmitter {
    /// The `container` arg appended to a `__wireBoundary` helper signature /
    /// call when its subtree reflects a slot (see `instrs_have_reflect_empty_slot`).
    /// Empty when not needed — so non-slot boundaries emit unchanged.
    fn boundary_host_param(&self, needs_host: bool) -> String {
        if needs_host {
            format!(", {}", self.machinery_param("container", "HTMLElement"))
        } else {
            String::new()
        }
    }
    fn boundary_host_arg(needs_host: bool) -> &'static str {
        if needs_host {
            ", container"
        } else {
            ""
        }
    }
    pub(in crate::emit) fn emit_if(
        &mut self,
        w: &mut Writer,
        instr: &IfInstr,
        parent_var: &str,
        cleanups_var: &str,
    ) {
        let key = self.next_key();
        let anchor = self.fresh("ifAnchor");

        // A `<slot reflect-empty>` anywhere in this boundary's subtree means the
        // hoisted helper reflects on the host → it must take `container`.
        let needs_host = instr
            .branches
            .iter()
            .any(|b| instrs_have_reflect_empty_slot(&b.body))
            || instr
                .else_body
                .as_deref()
                .is_some_and(instrs_have_reflect_empty_slot);

        // __mount-side: anchor + invoke hoisted handler.
        w.line(&format!(
            "const {anchor} = document.createComment(\"if\");"
        ));
        w.line(&format!("{parent_var}.appendChild({anchor});"));
        w.line(&format!(
            "{cleanups_var}.push(__wireBoundary_{key}({anchor}{host}));",
            host = Self::boundary_host_arg(needs_host)
        ));

        // Direct-emit hydrate dispatch (parent before children — outer
        // boundary swaps detach inner SSR'd nodes, so inner hydrate
        // calls become no-ops via the `__m[k]` guard).
        if self.is_route && self.boundary_depth == 0 {
            self.hydrate_uses_markers = true;
            self.emit_boundary_hydrate(&key, "if", needs_host);
        }

        // Hoisted handler — emitted into `boundary_hoists`, drained
        // into the setup body before the `return`.
        let mut h = Writer::new();
        let ap = self.machinery_param("anchor", "Comment");
        h.line(&format!(
            "const __wireBoundary_{key} = ({ap}{host}) => bindIf({{",
            host = self.boundary_host_param(needs_host)
        ));
        h.enter();
        h.line("anchor,");
        h.line("test: () => {");
        h.enter();
        // D5: subscribe to every branch condition's row-field deps so a
        // keyed `{#for}` row reuse (which reassigns `$scope.<var>` and
        // bumps the row's `__v_<field>` signals rather than re-running
        // this closure's caller) still re-fires this `bindIf` test and
        // re-selects the matching branch. Without this the discriminant
        // reads the row directly with no signal subscription — see
        // `for_version_stmt` doc comment.
        let combined_source = instr
            .branches
            .iter()
            .map(|b| b.cond.source.as_str())
            .collect::<Vec<_>>()
            .join(" ; ");
        let fvs = self.for_version_stmt(&combined_source);
        if !fvs.is_empty() {
            h.line(&fvs);
        }
        for (i, b) in instr.branches.iter().enumerate() {
            let rw = self.rw(&b.cond.source, &[]);
            h.line(&format!("if ({rw}) return {i};"));
        }
        let fallback = if instr.else_body.is_some() {
            instr.branches.len() as i32
        } else {
            -1
        };
        h.line(&format!("return {fallback};"));
        h.leave();
        h.line("},");
        h.line("branches: [");
        h.enter();
        for b in &instr.branches {
            self.emit_branch(&mut h, &b.body);
        }
        if let Some(eb) = &instr.else_body {
            self.emit_branch(&mut h, eb);
        }
        h.leave();
        h.line("],");
        h.leave();
        h.line("});");
        self.boundary_hoists.push_str(&h.into_string());
    }

    pub(in crate::emit) fn emit_for(
        &mut self,
        w: &mut Writer,
        instr: &ForInstr,
        parent_var: &str,
        cleanups_var: &str,
    ) {
        let key = self.next_key();
        let anchor = self.fresh("forAnchor");

        let needs_host = instrs_have_reflect_empty_slot(&instr.body)
            || instr
                .else_body
                .as_deref()
                .is_some_and(instrs_have_reflect_empty_slot);

        // __mount-side: anchor + invoke hoisted handler.
        w.line(&format!(
            "const {anchor} = document.createComment(\"for\");"
        ));
        w.line(&format!("{parent_var}.appendChild({anchor});"));
        w.line(&format!(
            "{cleanups_var}.push(__wireBoundary_{key}({anchor}{host}));",
            host = Self::boundary_host_arg(needs_host)
        ));

        if self.is_route && self.boundary_depth == 0 {
            self.hydrate_uses_markers = true;
            self.emit_boundary_hydrate(&key, "for", needs_host);
        }

        let it_rw = self.rw(&instr.iterable.source, &[]);
        let var = &instr.var_name;
        let idx_name = instr.index_name.as_deref().unwrap_or("__i");
        // D3: skip the per-row `__v` version signal when no binding in
        // the body would observe the update. The detector checks every
        // reachable Expr for an identifier-shaped reference to the row
        // binder (or the named index); on a hit we keep the sentinel
        // and the matching `void $scope.__v()` / `__v.set(c => c + 1)`
        // wiring. On a miss the row body is purely static (or reads
        // only outer-scope signals) and the per-row reactive plumbing
        // is dead weight — ~4 allocations × every row × every render.
        let body_needs_v = body_references_for_vars(
            &instr.body,
            var,
            instr.index_name.as_deref(),
        );

        let mut h = Writer::new();
        let ap = self.machinery_param("anchor", "Comment");
        h.line(&format!(
            "const __wireBoundary_{key} = ({ap}{host}) => bindFor({{",
            host = self.boundary_host_param(needs_host)
        ));
        h.enter();
        h.line("anchor,");
        h.line(&format!("items: () => ({it_rw}),"));
        if let Some(k) = &instr.key {
            let k_rw = self.rw(&k.source, &[var.as_str(), idx_name]);
            // `rewrite_for_scope` qualifies every free identifier that
            // isn't a loop binding, an expression-local, or a builtin
            // with `$scope.` — so a rewritten key with no `$scope.`
            // reference provably reads only its own parameters and
            // globals. Emit the flat closure: bindFor calls the key fn
            // once per row per render, and the child-scope form below
            // pays one dead `Object.create($scope)` per call (~1000
            // allocations on a keyed 1k-row swap). False positives
            // (e.g. a string literal containing "$scope.") only fall
            // back to the scoped form, which is always correct.
            if k_rw.contains("$scope.") {
                h.line(&format!(
                    "key: ({var}, {idx_name}) => {{ const __child = Object.create($scope); __child.{var} = {var}; __child.{idx_name} = {idx_name}; return (($scope) => ({k_rw}))(__child); }},"
                ));
            } else {
                h.line(&format!("key: ({var}, {idx_name}) => ({k_rw}),"));
            }
        }
        h.line(&format!("render: ({var}, {idx_name}) => {{"));
        h.enter();
        let child_scope = self.fresh("childScope");
        h.line(&format!("const {child_scope} = Object.create($scope);"));
        h.line(&format!("{child_scope}.{var} = {var};"));
        h.line(&format!("{child_scope}.{idx_name} = {idx_name};"));
        // Per-row version signals are emitted AFTER the body walk —
        // we need to know which fields reactive bindings touched before
        // we can decide whether to allocate `__v` (legacy whole-row),
        // per-field `__v_<f>`, or no signal at all.
        // Wrap the body in an IIFE that takes `$scope` as a parameter so
        // the iteration's child scope shadows the outer `$scope` without
        // a same-block `const $scope` declaration (which would TDZ on
        // the `Object.create($scope)` above).
        h.line("return (($scope) => {");
        h.enter();
        let frag = self.fresh("frag");
        let cl = self.fresh("c");
        // Walk body into a separate writer so we can inject any nested
        // boundary hoists (from {#if}/{#for}/{#match} inside the for
        // body) BEFORE the body consumes them. Without this, nested
        // hoists would land in `boundary_hoists` and emit at top-level —
        // closing over the outer `$scope` instead of the iteration's
        // IIFE-bound one. References like `$scope.item` would then
        // resolve to undefined inside nested test/render closures.
        // A row is almost always ONE element (`<tr>`, `<li>`, a card). Building it into
        // a DocumentFragment and then reading it back out with
        // `Array.from(frag.childNodes)` allocates two objects per row to carry one
        // element — 20,000 pieces of garbage for a 10,000-row list, and GC was already
        // eating 23.8% of our JS time on that benchmark. When the body is a single
        // root element we emit it DETACHED and hand `bindFor` the element directly.
        let node_producing: Vec<&Instr> = instr
            .body
            .iter()
            .filter(|i| !matches!(i, Instr::Comment { .. }))
            .collect();
        let single_root: Option<&ElementInstr> = match node_producing[..] {
            [Instr::Element(el)] => Some(el),
            _ => None,
        };

        let mut body_w = Writer::new();
        if single_root.is_none() {
            body_w.line(&format!(
                "const {frag} = document.createDocumentFragment();"
            ));
        }
        body_w.line(&self.decl_cleanups(&cl));
        let hoist_snapshot = self.boundary_hoists.len();
        self.boundary_depth += 1;
        let outer_for = std::mem::replace(&mut self.for_in_scope, body_needs_v);
        let outer_var = self.for_var_name.replace(var.clone());
        let outer_idx = self.for_idx_name.replace(idx_name.to_string());
        // D5: scope inline-update statements per for-loop so a nested
        // for inside this one collects into its own bucket, not ours.
        let outer_inline = std::mem::take(&mut self.inline_update_stmts);
        // Reactive uses also scope per-loop. Bindings inside the body
        // call `for_version_comma`/`for_version_stmt` which unions into
        // self.reactive_row_use; we read the union back AFTER the walk
        // to decide what signals to allocate at the top of the IIFE.
        let outer_reactive = std::mem::replace(
            &mut self.reactive_row_use,
            RowUse::None,
        );
        // The row + index binders live on the runtime `$scope` child scope
        // — qualify author references to them with `$scope.` (synthetic
        // mode), not the typed `scope.` prefix. Push for the body walk; the
        // Vec is truncated back after so nested loops stack correctly.
        let machinery_mark = self.machinery_locals.len();
        self.machinery_locals.push(var.clone());
        self.machinery_locals.push(idx_name.to_string());
        let root_var = if let Some(el) = single_root {
            // Detached: no fragment, no appendChild — the element IS the row.
            self.emit_element(&mut body_w, el, "", &cl)
        } else {
            self.emit_children(&mut body_w, &instr.body, &frag, &cl);
            String::new()
        };
        self.machinery_locals.truncate(machinery_mark);
        self.for_in_scope = outer_for;
        self.for_var_name = outer_var;
        self.for_idx_name = outer_idx;
        let inline_updates = std::mem::replace(&mut self.inline_update_stmts, outer_inline);
        let reactive_use = std::mem::replace(&mut self.reactive_row_use, outer_reactive);
        self.boundary_depth -= 1;
        let captured_hoists = self.boundary_hoists.split_off_from(hoist_snapshot);
        // Emit per-row version-signal allocations based on the union of
        // reactive subscriptions the body actually requested. Three
        // shapes:
        //   - RowUse::None    — no reactive subscription touches the
        //                       row binder (bindings read only outer
        //                       signals like `selected()`). No signal
        //                       needed; nothing to bump on update.
        //   - RowUse::Whole   — at least one reactive binding can't be
        //                       decomposed into a field set (bare `row`,
        //                       computed access, index read, etc.). Fall
        //                       back to the legacy `__v` whole-row
        //                       signal, bumped unconditionally on update.
        //   - RowUse::FieldsOnly(fs) — every reactive binding reads
        //                       only `row.<field>` patterns. Allocate
        //                       one `__v_<field>` signal per field;
        //                       each binding closure subscribes to just
        //                       the signals for the fields it reads;
        //                       `update()` bumps only the per-field
        //                       signals whose value actually changed.
        //                       This is the path that drops Krausest's
        //                       update-every-10th cost — the `class:
        //                       danger={row.id === selected()}` binding
        //                       subscribes to `__v_id`, which never
        //                       fires when only `label` changes.
        // Splice signal allocs at the current indent (inside the IIFE,
        // before captured hoists / body emit) so any nested boundary's
        // render closure can see them via `$scope.__v_<f>`. They run
        // before any binding closure fires because closures only
        // execute on their own events, not at IIFE-init time.
        match &reactive_use {
            RowUse::None => {}
            RowUse::Whole => {
                h.line(&format!("{child_scope}.__v = signal(0);"));
            }
            RowUse::FieldsOnly(fields) => {
                let mut sorted = fields.clone();
                sorted.sort();
                for f in &sorted {
                    h.line(&format!("{child_scope}.__v_{f} = signal(0);"));
                }
            }
        }
        if !captured_hoists.is_empty() {
            // Captured hoist text was generated at indent=0; placing it
            // inside the IIFE leaves it left-aligned (ugly but valid JS).
            h.push_str(&captured_hoists);
        }
        h.push_str(&body_w.into_string());
        let reactive_needs_update = !matches!(reactive_use, RowUse::None);
        let needs_update = reactive_needs_update || !inline_updates.is_empty();
        if needs_update {
            let new_var = self.fresh("new");
            let new_idx = self.fresh("newI");
            // The update body re-runs each inlined static assignment so
            // keyed-row reuse refreshes the DOM when bindFor reassigns
            // the row instance, and bumps the per-row version signal(s)
            // so reactive bindings re-evaluate against the new item.
            //
            // Per-field diff gating: when every inline stmt's row-use
            // is `FieldsOnly`, partition stmts by field and wrap each
            // group in `if (oldRow.<field> !== newRow.<field>) {…}`.
            // For the Krausest update benchmark (only `label` changes
            // across 100 rows), this skips the id-bound textContent +
            // setAttribute calls entirely. Falls back to the unguarded
            // join when any stmt has `Whole` or `None` usage.
            let (gated, fallback) = partition_inline_updates(&inline_updates, var);
            let inline_run = if let Some(body) = gated {
                body
            } else {
                fallback
            };
            // Reactive bumps: same three-way split as signal alloc.
            let v_bump: String = match &reactive_use {
                RowUse::None => String::new(),
                RowUse::Whole => " $scope.__v.set(c => c + 1);".to_string(),
                RowUse::FieldsOnly(fields) => {
                    let mut sorted = fields.clone();
                    sorted.sort();
                    sorted
                        .iter()
                        .map(|f| {
                            format!(
                                " if (__old.{f} !== $scope.{var}.{f}) {{ $scope.__v_{f}.set(c => c + 1); }}"
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("")
                }
            };
            // `const __old = ...` is needed when EITHER inline gating
            // references __old.f OR reactive bumps gate per-field.
            // Whole-row bumps and fallback inline-runs both work
            // without __old, so we skip the capture there to keep the
            // emit byte-identical to pre-spike output for those rows.
            let inline_uses_old = inline_run.contains("__old.");
            let reactive_uses_old = matches!(reactive_use, RowUse::FieldsOnly(_));
            let old_capture = if inline_uses_old || reactive_uses_old {
                format!("const __old = $scope.{var}; ")
            } else {
                String::new()
            };
            let nodes = if root_var.is_empty() {
                format!("Array.from({frag}.childNodes)")
            } else {
                format!("[{root_var}]")
            };
            h.line(&format!(
                "return {{ nodes: {nodes}, cleanup: () => {{ for (const c of {cl}) try {{ c(); }} catch (e) {{}} }}, update: ({new_var}, {new_idx}) => {{ {old_capture}$scope.{var} = {new_var}; $scope.{idx_name} = {new_idx}; {inline_run}{v_bump} }} }};"
            ));
        } else {
            // No body expression reads the row/index — bindFor's
            // optional `update` hook would have nothing to refresh.
            // Omit it entirely so key-reuse skips a no-op call too.
            let nodes = if root_var.is_empty() {
                format!("Array.from({frag}.childNodes)")
            } else {
                format!("[{root_var}]")
            };
            h.line(&format!(
                "return {{ nodes: {nodes}, cleanup: () => {{ for (const c of {cl}) try {{ c(); }} catch (e) {{}} }} }};"
            ));
        }
        h.leave();
        h.line(&format!("}})({child_scope});"));
        h.leave();
        h.line("},");
        if let Some(eb) = &instr.else_body {
            h.line("emptyRender: () => {");
            h.enter();
            let frag = self.fresh("frag");
            let cl = self.fresh("c");
            h.line(&format!(
                "const {frag} = document.createDocumentFragment();"
            ));
            h.line(&self.decl_cleanups(&cl));
            self.boundary_depth += 1;
            self.emit_children(&mut h, eb, &frag, &cl);
            self.boundary_depth -= 1;
            let nodes = if root_var.is_empty() {
                format!("Array.from({frag}.childNodes)")
            } else {
                format!("[{root_var}]")
            };
            h.line(&format!(
                "return {{ nodes: {nodes}, cleanup: () => {{ for (const c of {cl}) try {{ c(); }} catch (e) {{}} }} }};"
            ));
            h.leave();
            h.line("},");
        }
        h.leave();
        h.line("});");
        self.boundary_hoists.push_str(&h.into_string());
    }

    pub(in crate::emit) fn emit_match(
        &mut self,
        w: &mut Writer,
        instr: &MatchInstr,
        parent_var: &str,
        cleanups_var: &str,
    ) {
        let key = self.next_key();
        let anchor = self.fresh("matchAnchor");

        let needs_host = instr
            .cases
            .iter()
            .any(|c| instrs_have_reflect_empty_slot(&c.body))
            || instr
                .default
                .as_deref()
                .is_some_and(instrs_have_reflect_empty_slot);

        w.line(&format!(
            "const {anchor} = document.createComment(\"match\");"
        ));
        w.line(&format!("{parent_var}.appendChild({anchor});"));
        w.line(&format!(
            "{cleanups_var}.push(__wireBoundary_{key}({anchor}{host}));",
            host = Self::boundary_host_arg(needs_host)
        ));

        if self.is_route && self.boundary_depth == 0 {
            self.hydrate_uses_markers = true;
            self.emit_boundary_hydrate(&key, "match", needs_host);
        }

        let disc_rw = self.rw(&instr.discriminant.source, &[]);
        let mut h = Writer::new();
        let ap = self.machinery_param("anchor", "Comment");
        h.line(&format!(
            "const __wireBoundary_{key} = ({ap}{host}) => bindIf({{",
            host = self.boundary_host_param(needs_host)
        ));
        h.enter();
        h.line("anchor,");
        h.line("test: () => {");
        h.enter();
        // D5: subscribe to the discriminant's (and each case test's)
        // row-field deps — same rationale as `emit_if` above. A keyed
        // `{#for}` row reuse only bumps `__v_<field>` signals; without
        // reading them here this `bindIf` test never re-fires and the
        // stale branch stays mounted.
        let mut combined_parts: Vec<&str> = vec![instr.discriminant.source.as_str()];
        combined_parts.extend(instr.cases.iter().map(|c| c.test.source.as_str()));
        let combined_source = combined_parts.join(" ; ");
        let fvs = self.for_version_stmt(&combined_source);
        if !fvs.is_empty() {
            h.line(&fvs);
        }
        h.line(&format!("const __d = ({disc_rw});"));
        for (i, c) in instr.cases.iter().enumerate() {
            let c_rw = self.rw(&c.test.source, &[]);
            h.line(&format!(
                "if (__d === ({c_rw})) return {i};"
            ));
        }
        let fallback = if instr.default.is_some() {
            instr.cases.len() as i32
        } else {
            -1
        };
        h.line(&format!("return {fallback};"));
        h.leave();
        h.line("},");
        h.line("branches: [");
        h.enter();
        for c in &instr.cases {
            self.emit_branch(&mut h, &c.body);
        }
        if let Some(d) = &instr.default {
            self.emit_branch(&mut h, d);
        }
        h.leave();
        h.line("],");
        h.leave();
        h.line("});");
        self.boundary_hoists.push_str(&h.into_string());
    }

    /// Emit a hydrate-side dispatch for an `if` / `for` / `match`
    /// boundary. The compiled `hydrate(root, ctx)` body looks up the
    /// SSR'd `<sail-{kind} key="b<N>">` element via `__m`, swaps it for
    /// a Comment anchor, and invokes the hoisted `__wireBoundary_b<N>`
    /// helper — same shape the legacy `__bN_attach` named property used
    /// to expose, just inlined into the unified hydrate function.
    fn emit_boundary_hydrate(&mut self, key: &str, kind: &str, needs_host: bool) {
        let k = js_string(key);
        self.hydrate_calls.line(&format!("if (__m[{k}]) {{"));
        self.hydrate_calls.line(&format!("  const __t = __m[{k}];"));
        self.hydrate_calls.line("  const __doc = __t.ownerDocument ?? document;");
        self.hydrate_calls.line(&format!(
            "  const __anchor = (__t.nodeType === 1) ? __doc.createComment({}) : __t;",
            js_string(kind),
        ));
        self.hydrate_calls.line(
            "  if (__t.nodeType === 1 && __t.parentNode) __t.parentNode.replaceChild(__anchor, __t);",
        );
        // `__anchor` is `Comment | Element` (the marker is an Element until the
        // replace above swaps in a Comment); `__wireBoundary` takes a `Comment`.
        // In the typed synthetic render, narrow with a cast — the marker is
        // always an element boundary, so the Comment branch always wins at
        // runtime. Production emits untyped JS, so no cast there.
        let anchor_arg = if self.is_synthetic() { "__anchor as Comment" } else { "__anchor" };
        // When the boundary reflects a slot, pass the host. In hydrate the
        // mount target `root` IS the host (a route hydrates into its container);
        // synthetic mode narrows since `root`'s param type is broader.
        let host_arg = if needs_host {
            if self.is_synthetic() { ", root as HTMLElement" } else { ", root" }
        } else {
            ""
        };
        self.hydrate_calls.line(&format!(
            "  __cleanups.push(__wireBoundary_{key}({anchor_arg}{host_arg}));"
        ));
        self.hydrate_calls.line("}");
    }

    /// Emit `{#while-loading of <bind>}…{/while-loading}`.
    ///
    /// Hoists `const __renderWhileLoading_<N> = () => …` inside
    /// `setup(ctx)` so the body is walked exactly once. Both `__mount`
    /// (legacy) and `hydrate(root, ctx)` reference the same hoisted const.
    /// Single walk = single registry-key consumption per body.
    pub(super) fn emit_while_loading_block(
        &mut self,
        w: &mut Writer,
        instr: &WhileLoadingInstr,
        parent_var: &str,
        cleanups_var: &str,
    ) {
        // Registry parity: while-loading consumes one b<N> for the
        // boundary itself. Body keys are consumed during the (single)
        // emit walk below.
        let key = self.next_key();
        let idx = self.while_loading_idx;
        self.while_loading_idx += 1;
        let render_key = format!("__renderWhileLoading_{idx}");
        let bind = &instr.bind;

        // Hoist the render fn so it's a single-source-of-truth.
        self.emit_wl_render_hoist(&render_key, &instr.body);

        // __mount inline call references the hoisted const.
        let anchor = self.fresh("wlAnchor");
        w.line(&format!(
            "const {anchor} = document.createComment(\"while-loading\");"
        ));
        w.line(&format!("{parent_var}.appendChild({anchor});"));
        w.line(&format!(
            "{cleanups_var}.push(bindWhileLoading({{ anchor: {anchor}, signal: $scope.{bind}, render: {render_key} }}));"
        ));

        // Phase 3b.5 gate G: direct-emit hydrate call. The render fn is
        // already in the setup closure (no need to expose it as a named
        // property anymore).
        if self.is_route && self.boundary_depth == 0 {
            self.hydrate_uses_markers = true;
            self.hydrate_calls.line(&format!(
                "if (__m[{k}]) __cleanups.push(hydrateWhileLoading(__m[{k}], {k}, () => $scope.{bind}(), {render_key}));",
                k = js_string(&key),
            ));
        }
    }

    /// Emit `{#while-error <errVar> of <bind>}…{/while-error}`.
    pub(super) fn emit_while_error_block(
        &mut self,
        w: &mut Writer,
        instr: &WhileErrorInstr,
        parent_var: &str,
        cleanups_var: &str,
    ) {
        let key = self.next_key();
        let idx = self.while_error_idx;
        self.while_error_idx += 1;
        let render_key = format!("__renderWhileError_{idx}");
        let bind = &instr.bind;
        let var = &instr.error_var;

        self.emit_we_render_hoist(&render_key, var, &instr.body);

        let anchor = self.fresh("weAnchor");
        w.line(&format!(
            "const {anchor} = document.createComment(\"while-error\");"
        ));
        w.line(&format!("{parent_var}.appendChild({anchor});"));
        w.line(&format!(
            "{cleanups_var}.push(bindWhileError({{ anchor: {anchor}, signal: $scope.{bind}, render: {render_key} }}));"
        ));

        if self.is_route && self.boundary_depth == 0 {
            self.hydrate_uses_markers = true;
            // The `{#while-error}` render binds its error as `Error` (so the
            // author's `err.message` type-checks), matching `bindWhileError`.
            // But `hydrateWhileError` types its render param `unknown`, so in
            // the typed synthetic render the narrower `(err: Error)` fn is not
            // directly assignable — cast it to the helper's own param type.
            // Production emits untyped JS (no cast). This surfaces an upstream
            // runtime-types inconsistency between the bind/hydrate while-error
            // helpers (see the slice-1 assessment notes).
            let render_arg = if self.is_synthetic() {
                format!("({render_key} as unknown as Parameters<typeof hydrateWhileError>[3])")
            } else {
                render_key.clone()
            };
            self.hydrate_calls.line(&format!(
                "if (__m[{k}]) __cleanups.push(hydrateWhileError(__m[{k}], {k}, () => $scope.{bind}(), {render_arg}));",
                k = js_string(&key),
            ));
        }
    }

    fn emit_wl_render_hoist(&mut self, render_key: &str, body: &[Instr]) {
        let frag = self.fresh("frag");
        let cl = self.fresh("c");
        let mut h = Writer::new();
        h.line(&format!("const {render_key} = () => {{"));
        h.enter();
        h.line(&format!("const {frag} = document.createDocumentFragment();"));
        h.line(&self.decl_cleanups(&cl));
        self.boundary_depth += 1;
        self.emit_children(&mut h, body, &frag, &cl);
        self.boundary_depth -= 1;
        h.line(&format!(
            "return {{ nodes: Array.from({frag}.childNodes), cleanup: () => {{ for (const c of {cl}) try {{ c(); }} catch (e) {{}} }} }};"
        ));
        h.leave();
        h.line("};");
        self.boundary_hoists.push_str(&h.into_string());
    }

    fn emit_we_render_hoist(&mut self, render_key: &str, error_var: &str, body: &[Instr]) {
        let child = self.fresh("childScope");
        let var = error_var;
        let mut h = Writer::new();
        // The `{#while-error}` binder is an `Error` (the runtime
        // `bindWhileError` contract); annotate it in synthetic mode so the
        // render type-checks strict.
        let ep = self.machinery_param(var, "Error");
        h.line(&format!("const {render_key} = ({ep}) => {{"));
        h.enter();
        h.line(&format!("const {child} = Object.create($scope);"));
        h.line(&format!("{child}.{var} = {var};"));
        // IIFE-shadow `$scope` to avoid the TDZ that would fire if we
        // declared `const $scope = {child}` in the same block as the
        // `Object.create($scope)` above.
        h.line("return (($scope) => {");
        h.enter();
        let frag = self.fresh("frag");
        let cl = self.fresh("c");
        h.line(&format!("const {frag} = document.createDocumentFragment();"));
        h.line(&self.decl_cleanups(&cl));
        self.boundary_depth += 1;
        // The error binder lives on the runtime `$scope` child scope; qualify
        // author references to it with `$scope.` (synthetic mode).
        let machinery_mark = self.machinery_locals.len();
        self.machinery_locals.push(var.to_string());
        self.emit_children(&mut h, body, &frag, &cl);
        self.machinery_locals.truncate(machinery_mark);
        self.boundary_depth -= 1;
        h.line(&format!(
            "return {{ nodes: Array.from({frag}.childNodes), cleanup: () => {{ for (const c of {cl}) try {{ c(); }} catch (e) {{}} }} }};"
        ));
        h.leave();
        h.line(&format!("}})({child});"));
        h.leave();
        h.line("};");
        self.boundary_hoists.push_str(&h.into_string());
    }
}

#[cfg(test)]
mod partition_tests {
    use super::*;

    #[test]
    fn whole_row_falls_back() {
        let u = vec![
            (
                "t1".into(),
                RowUse::FieldsOnly(vec!["id".into()]),
            ),
            ("t2".into(), RowUse::Whole),
        ];
        let (gated, fallback) = partition_inline_updates(&u, "row");
        assert!(gated.is_none());
        assert!(fallback.contains("t1") && fallback.contains("t2"));
    }

    #[test]
    fn fields_only_groups_by_field_set() {
        let u = vec![
            ("set_id_text".into(), RowUse::FieldsOnly(vec!["id".into()])),
            (
                "set_label_text".into(),
                RowUse::FieldsOnly(vec!["label".into()]),
            ),
            (
                "set_id_attr".into(),
                RowUse::FieldsOnly(vec!["id".into()]),
            ),
        ];
        let (gated, _) = partition_inline_updates(&u, "row");
        let body = gated.unwrap();
        // id-group fires id stmts together
        assert!(body.contains("__old.id !== $scope.row.id"));
        assert!(body.contains("__old.label !== $scope.row.label"));
        // id stmts share their guard
        let id_section = body
            .split("__old.id !== $scope.row.id")
            .nth(1)
            .unwrap_or("");
        assert!(id_section.contains("set_id_text"));
        assert!(id_section.contains("set_id_attr"));
    }

    #[test]
    fn multi_field_stmt_disjunction() {
        let u = vec![(
            "concat".into(),
            RowUse::FieldsOnly(vec!["id".into(), "label".into()]),
        )];
        let body = partition_inline_updates(&u, "row").0.unwrap();
        assert!(body.contains("__old.id !== $scope.row.id || __old.label !== $scope.row.label"));
    }

    #[test]
    fn none_stmts_dropped() {
        // A `{42}` constant inside a for body classifies as None.
        // It's idempotent on update — never needs re-firing.
        let u = vec![("set_const".into(), RowUse::None)];
        let body = partition_inline_updates(&u, "row").0.unwrap();
        assert!(body.is_empty());
    }
}
