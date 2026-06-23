//! Walk template `Instr`s and emit DOM-construction JS into a writer.
//!
//! Each instruction maps to a small chunk of JS that creates DOM
//! nodes, registers reactive bindings, and pushes their disposers
//! onto a `__cleanups` array.
//!
//! Expressions are pasted inline. The compiler rewrites each
//! template-expression source so free identifiers are qualified with
//! `$scope.` (see `expr_rewrite`), then the rewritten string is
//! emitted directly into a closure body — no `with($scope)`, no
//! `new Function` eval, no shared expression bank.
//!
//! ## Phase 2 — client-side control flow
//!
//! `{#if}` / `{#for}` / `{#match}` boundaries no longer round-trip
//! through `/_deck/render-fragment/`. Each boundary emits two pieces:
//!
//!   1. A hoisted `const __wireBoundary_b<N> = (anchor) => bindIf({...})`
//!      defined inside `setup(ctx)`. Captures `$scope`. Used by
//!      `__mount`'s inline path AND by the public attach handler below.
//!   2. (Views only) A public property `__b<N>_attach: (target) => …`
//!      on the setup return. `attach()` walks SSR'd
//!      `<sui-{if|for|match|await} key="b<N>">` elements, finds the
//!      handler by key, swaps the SSR wrapper for a Comment anchor, and
//!      invokes the hoisted helper. The first effect-run mounts the
//!      active branch synchronously, replacing the SSR'd content.
//!
//! The emit logic is split across three modules:
//!  * this file — orchestration + the `emit_one` dispatcher
//!  * `dom_emit` — element + attribute emit
//!  * `control_emit` — `{#if}`, `{#for}`, `{#match}`, `{#while-*}`

// FILE_CAP_OVERRIDE: one cohesive concern — the Instr→JS emit orchestrator.
// It already delegates element/attr emit to `dom_emit` and control flow to
// `control_emit` (submodules); what remains is the single walk that dispatches
// instructions to them. Splitting the dispatcher scatters one loop.
use super::expr_rewrite::rewrite_scoped;
use super::writer::{js_string, Writer};
use crate::ir::*;
use crate::keys::{BindingRegistry, KeyAllocator};

mod control_emit;
mod dom_emit;

pub struct TemplateEmitter {
    next_id: usize,
    /// Sequential counters for while-loading/while-error render-callback names.
    /// Both the bindings pass and this emit pass walk the IR in the same source
    /// order, so the N-th while-loading gets the same `__renderWhileLoading_<N>`
    /// name in both.
    pub(super) while_loading_idx: usize,
    pub(super) while_error_idx: usize,
    /// Render callbacks for `{#while-loading}` / `{#while-error}` blocks,
    /// emitted as named properties on the setup return object (outside
    /// `__mount`) so `attach()` can look them up by `renderKey`.
    pub(super) render_callbacks: Writer,
    /// Hoisted `const __wireBoundary_b<N> = (anchor) => bindIf(...)`
    /// declarations for each `{#if}` / `{#for}` / `{#match}` boundary.
    /// Drained into the setup body before the return statement so the
    /// helpers are in scope for both `__mount` and the hydrate-side
    /// boundary dispatch emitted into `hydrate_calls`.
    pub(super) boundary_hoists: Writer,
    /// Direct-emit hydrate calls for migrated binding kinds (Phase 3b.3).
    /// Each line is one statement inside the view's `hydrate(root, ctx)`
    /// function — e.g. `if (__t["b0"]) __cleanups.push(hydrateText(...));`.
    /// Only populated in view mode; components stay manifest-driven this
    /// phase. The corresponding manifest entries are skipped in
    /// `js.rs::emit_bindings`, so each migrated kind moves out of the
    /// manifest and into hydrate atomically.
    pub(super) hydrate_calls: Writer,
    /// True iff `hydrate_calls` includes any text-binding wiring — drives
    /// the `collectTextOpens(root)` preamble emit in `js.rs`.
    pub(super) hydrate_uses_text: bool,
    /// True iff `hydrate_calls` includes any element-binding wiring
    /// (`on:*`, `attr:*`, `class:*`, `style:*`, `bind:value`, `bind:this`)
    /// — drives the `collectElementMarkers(root)` preamble emit in
    /// `js.rs`.
    pub(super) hydrate_uses_markers: bool,
    /// Depth counter for emits inside `{#if}` / `{#for}` / `{#match}` /
    /// `{#while-*}` bodies. Hydrate calls fire only at depth 0 — bindings
    /// inside boundary subtrees are wired by the boundary's own render
    /// closure (which builds DOM client-side and calls `bindText` etc.
    /// directly). The SSR'd markers inside boundary subtrees are detached
    /// when `wireBoundaries` swaps the wrapper, so `collectTextOpens`
    /// would not see them anyway.
    pub(super) boundary_depth: usize,
    /// Key allocator that mirrors `bindings::Builder` exactly. Bumped at
    /// every IR position the registry pre-pass would have allocated a
    /// key — so the `b<N>` we read here for control-flow boundaries
    /// matches the SSR walker's emitted `<sui-{kind} key="b<N>">`.
    pub(super) keys: KeyAllocator,
    /// Whether the current compile mode is `View`. Components don't
    /// emit `hydrate(root, ctx)` (no SSR/attach() path).
    pub(super) is_view: bool,
    /// True while emitting the per-item IIFE body of a `{#for}` loop.
    /// When set, reactive binding closures void-read `$scope.__v` to
    /// track the per-row version signal and re-run when `update` fires.
    pub(super) for_in_scope: bool,
    /// The row binder of the innermost enclosing `{#for}`. Used by
    /// per-field diff gating to attribute `<row>.<field>` reads in
    /// inline-update statements to a specific row binder. None when
    /// not inside any for-loop body.
    pub(super) for_var_name: Option<String>,
    /// The index binder of the innermost enclosing `{#for}` (default
    /// `__i`, or whatever the author named it). An inline-update stmt
    /// that references the index can't be gated on row-field diffs —
    /// the index may have shifted while every row field stayed put —
    /// so a hit here escalates the row-use to `Whole` (always re-fire).
    pub(super) for_idx_name: Option<String>,
    /// D5: statements that re-execute an inlined static assignment
    /// (`__t.textContent = $scope.row.label;`, …). Accumulated while
    /// emitting the current `{#for}` row body; spliced into that for's
    /// `update()` callback so keyed-row reuse refreshes the inlined
    /// DOM. Each entry carries the row-field usage of its source
    /// expression so the for-emitter can wrap mutually-independent
    /// stmts in per-field `if (oldRow.f !== newRow.f) {…}` guards —
    /// the bench's `update` (label-only changes) saves ~3 writes per
    /// row by skipping id-bound stmts entirely. `emit_for` saves /
    /// restores this list around its body walk to scope it per for-loop.
    pub(super) inline_update_stmts: Vec<(String, crate::lower::row_field_uses::RowUse)>,
    /// Accumulated row-field usage across REACTIVE binding closures
    /// in the current `{#for}` row body (bindClass / bindAttr-effect
    /// / bindProp / etc. — anything that subscribes via for_version_*
    /// prefixes). emit_for reads this to decide which per-field `__v_<f>`
    /// signals to allocate and which to bump in `update()`. Defaults
    /// to `RowUse::None`; a single Whole-row binding escalates the
    /// whole loop to the legacy `__v` whole-row signal so we never
    /// silently miss a reactive update. Save/restored per for-loop.
    pub(super) reactive_row_use: crate::lower::row_field_uses::RowUse,
    /// Depth into an `<svg>` subtree. Bumped by `emit_element` around
    /// its child walk when the tag is `svg` (or already inside one).
    /// While > 0, `emit_element` emits `createElementNS(SVG_NS, tag)`
    /// instead of `createElement(tag)` so the resulting nodes are in
    /// the SVG namespace — without this, `<svg>` / `<rect>` etc. from
    /// `.sui` templates resolve as HTML elements and render blank.
    pub(super) svg_depth: usize,
    /// Prefix applied to free identifiers in author expressions. The
    /// default `"$scope."` is the production runtime-proxy path (output
    /// byte-identical to before this field existed). The synthetic emit
    /// (`emit::synthetic`) sets `"scope."` so author expressions resolve
    /// against the typed `scope` export — the runtime machinery this
    /// emitter generates (child scopes, version signals) keeps using the
    /// literal `$scope` runtime variable. See `emit::synthetic`.
    pub(super) expr_prefix: String,
    /// Stack of row/error binders introduced by enclosing `{#for}` /
    /// `{#while-error}` blocks. These live on the runtime-mutable `$scope`
    /// child scope (which keyed-reuse `update` reassigns), so author
    /// expressions referencing them are qualified with `$scope.` even when
    /// `expr_prefix` is `"scope."`. Pushed entering a body, popped exiting;
    /// the stack handles nesting. In production (`expr_prefix == "$scope."`)
    /// this changes nothing — see `expr_rewrite::rewrite_scoped`.
    pub(super) machinery_locals: Vec<String>,
}

impl TemplateEmitter {
    pub fn new(_registry: &BindingRegistry, is_view: bool) -> Self {
        Self {
            next_id: 0,
            while_loading_idx: 0,
            while_error_idx: 0,
            render_callbacks: Writer::new(),
            boundary_hoists: Writer::new(),
            hydrate_calls: Writer::new(),
            hydrate_uses_text: false,
            hydrate_uses_markers: false,
            boundary_depth: 0,
            keys: KeyAllocator::default(),
            is_view,
            for_in_scope: false,
            for_var_name: None,
            for_idx_name: None,
            inline_update_stmts: Vec::new(),
            reactive_row_use: crate::lower::row_field_uses::RowUse::None,
            svg_depth: 0,
            expr_prefix: "$scope.".to_string(),
            machinery_locals: Vec::new(),
        }
    }

    /// Set the author-expression prefix (default `"$scope."`). Returns
    /// `self` for builder-style use. The synthetic emit passes `"scope."`
    /// so author expressions type-check against the controller's typed
    /// `scope` export while the generated runtime machinery still drives
    /// the literal `$scope` variable.
    pub fn with_expr_prefix(mut self, prefix: &str) -> Self {
        self.expr_prefix = prefix.to_string();
        self
    }

    /// Rewrite an author-expression source, qualifying free identifiers
    /// with the configured [`Self::expr_prefix`]. All template-feature
    /// emit goes through this rather than `rewrite_for_scope` directly so
    /// the synthetic and production paths share one walk.
    pub(super) fn rw(&self, src: &str, extra_locals: &[&str]) -> String {
        let machinery: Vec<&str> = self.machinery_locals.iter().map(|s| s.as_str()).collect();
        rewrite_scoped(src, extra_locals, &self.expr_prefix, &machinery)
    }

    /// `true` when emitting the synthetic (typed TS) render rather than the
    /// production runtime module — i.e. the author-expression prefix is
    /// `scope.` not `$scope.`.
    pub(super) fn is_synthetic(&self) -> bool {
        self.expr_prefix != "$scope."
    }

    /// Declare a per-boundary disposer array. In synthetic (TS) mode the
    /// empty array literal is annotated so it doesn't infer `never[]` under
    /// strict null checks; production stays byte-identical (`const v = [];`,
    /// valid JS — TS annotations would be invalid there).
    pub(super) fn decl_cleanups(&self, var: &str) -> String {
        if self.is_synthetic() {
            format!("const {var}: Array<() => void> = [];")
        } else {
            format!("const {var} = [];")
        }
    }

    /// A machinery callback parameter, annotated in synthetic (TS) mode so the
    /// render type-checks under `noImplicitAny`. Production stays byte-identical
    /// (bare `name`, valid JS — a TS annotation would be a syntax error there).
    /// The generated machinery types are FIXED by the runtime contract (the
    /// boundary anchor is a `Comment`, the `{#while-error}` binder is an
    /// `Error`), so the annotation is precise, not a widening to `any`.
    pub(super) fn machinery_param(&self, name: &str, ty: &str) -> String {
        if self.is_synthetic() {
            format!("{name}: {ty}")
        } else {
            name.to_string()
        }
    }

    /// Drain the render-callback buffer. Called by `emit_setup_body`
    /// after `emit_top_level` finishes writing `__mount`.
    pub fn take_render_callbacks(&mut self) -> String {
        std::mem::replace(&mut self.render_callbacks, Writer::new()).into_string()
    }

    /// Drain the hoisted-handler buffer. Called by `emit_setup_body`
    /// before the `return` statement so the consts are in scope.
    pub fn take_boundary_hoists(&mut self) -> String {
        std::mem::replace(&mut self.boundary_hoists, Writer::new()).into_string()
    }

    /// Drain the hydrate-call buffer. Returns the body of the view's
    /// `hydrate(root, ctx)` named property — empty when no leaf binding
    /// kinds have been migrated to direct-emit yet.
    pub fn take_hydrate_calls(&mut self) -> String {
        std::mem::replace(&mut self.hydrate_calls, Writer::new()).into_string()
    }

    /// Whether `take_hydrate_calls` will need the `collectTextOpens(root)`
    /// preamble (i.e. at least one text binding migrated).
    pub fn hydrate_uses_text(&self) -> bool {
        self.hydrate_uses_text
    }

    /// Whether `take_hydrate_calls` will need the
    /// `collectElementMarkers(root)` preamble (i.e. at least one
    /// element-bound leaf kind migrated).
    pub fn hydrate_uses_markers(&self) -> bool {
        self.hydrate_uses_markers
    }

    /// Comma-form subscription prefix for a reactive binding closure
    /// inside a `{#for}` row body. Returns the empty string when not
    /// in a for-loop. When `source` reads only specific row fields,
    /// emits a `void $scope.__v_<field>()` per field — narrowing the
    /// effect's per-row dep so an `update()` that only changes a
    /// different field doesn't re-fire it. Whole-row reads escalate
    /// to the legacy `void $scope.__v()` whole-row signal.
    ///
    /// Side effect: unions the source's row-use into
    /// `self.reactive_row_use` so `emit_for` knows which per-field
    /// signals to allocate and bump.
    pub(super) fn for_version_comma(&mut self, source: &str) -> String {
        self.for_version_subscription(source, ", ")
    }

    /// Statement-form ("`...; `") version for use inside
    /// `effect(() => { … })` bodies. Same semantics as the
    /// comma form; suffix differs so the emitted code parses.
    pub(super) fn for_version_stmt(&mut self, source: &str) -> String {
        self.for_version_subscription(source, "; ")
    }

    fn for_version_subscription(&mut self, source: &str, sep: &str) -> String {
        if !self.for_in_scope {
            return String::new();
        }
        let use_ = self.row_use_for_source(source);
        use crate::lower::row_field_uses::RowUse;
        match &use_ {
            RowUse::None => {
                // Binding doesn't read the row at all (e.g. reads only
                // outer-scope signals like `selected()`). No per-row
                // re-fire needed — the outer-scope sub keeps it live.
                self.reactive_row_use = std::mem::replace(
                    &mut self.reactive_row_use,
                    RowUse::None,
                )
                .union(use_);
                String::new()
            }
            RowUse::Whole => {
                // Escalates the whole loop to the legacy __v signal.
                self.reactive_row_use = std::mem::replace(
                    &mut self.reactive_row_use,
                    RowUse::None,
                )
                .union(use_);
                format!("void $scope.__v(){sep}")
            }
            RowUse::FieldsOnly(fields) => {
                let prefix = fields
                    .iter()
                    .map(|f| format!("void $scope.__v_{f}()"))
                    .collect::<Vec<_>>()
                    .join(", ");
                self.reactive_row_use = std::mem::replace(
                    &mut self.reactive_row_use,
                    RowUse::None,
                )
                .union(use_);
                format!("{prefix}{sep}")
            }
        }
    }

    /// Classify how `source` references the current `{#for}` row
    /// binder, for per-field diff gating in the for's `update()` body.
    /// Returns `RowUse::None` when not inside a for-loop (gating
    /// doesn't apply outside).
    ///
    /// Index reads escalate to `Whole`: a `__i % 2 === 0` class
    /// binding has no row-field dep at all, but the index may still
    /// have shifted on reorder, so the stmt must re-fire — gating it
    /// would leave stale DOM after a swap.
    pub(super) fn row_use_for_source(
        &self,
        source: &str,
    ) -> crate::lower::row_field_uses::RowUse {
        let var = match &self.for_var_name {
            Some(v) => v,
            None => return crate::lower::row_field_uses::RowUse::None,
        };
        let base = crate::lower::row_field_uses::analyze_source(source, var);
        if let Some(idx) = &self.for_idx_name
            && crate::lower::row_field_uses::analyze_source(source, idx)
                != crate::lower::row_field_uses::RowUse::None
        {
            return crate::lower::row_field_uses::RowUse::Whole;
        }
        base
    }

    /// D5: classify a binding-expression source as "static enough to
    /// inline as a direct DOM assignment". Conservative — any `()`
    /// call, optional-chain `?.`, or nullish-coalescing `??` defers
    /// back to the `bind*` effect path so reactivity / null semantics
    /// stay correct. Only meaningful inside a `{#for}` row body, where
    /// each effect costs a closure + cleanup slot × every row × every
    /// render.
    pub(super) fn is_static_row_expr(&self, source: &str) -> bool {
        // TEMP-DISABLE for A/B bench measurement; revert before commit.
        if std::env::var_os("SUI_D5_DISABLE").is_some() {
            return false;
        }
        self.for_in_scope
            && !source.contains('(')
            && !source.contains("?.")
            && !source.contains("??")
    }

    pub(super) fn fresh(&mut self, prefix: &str) -> String {
        let id = self.next_id;
        self.next_id += 1;
        format!("__{prefix}{id}")
    }

    /// Allocate the next `b<N>` key. Mirrors `bindings::Builder::push`
    /// — call sites must match the registry pre-pass exactly.
    pub(super) fn next_key(&mut self) -> String {
        self.keys.alloc()
    }

    pub fn emit_top_level(
        &mut self,
        w: &mut Writer,
        instrs: &[Instr],
        parent_var: &str,
        cleanups_var: &str,
    ) {
        self.emit_children(w, instrs, parent_var, cleanups_var);
    }

    pub(super) fn emit_children(
        &mut self,
        w: &mut Writer,
        instrs: &[Instr],
        parent_var: &str,
        cleanups_var: &str,
    ) {
        for instr in instrs {
            self.emit_one(w, instr, parent_var, cleanups_var);
        }
    }

    pub(super) fn emit_one(
        &mut self,
        w: &mut Writer,
        instr: &Instr,
        parent_var: &str,
        cleanups_var: &str,
    ) {
        match instr {
            Instr::StaticText { text, .. } => {
                let v = self.fresh("t");
                w.line(&format!(
                    "const {v} = document.createTextNode({});",
                    js_string(text)
                ));
                w.line(&format!("{parent_var}.appendChild({v});"));
            }
            // HTML comments are developer documentation; drop from compiled output.
            Instr::Comment { .. } => {}
            Instr::DynamicText { expr } => {
                // Registry parity: text bindings consume one b<N>.
                let key = self.next_key();
                let v = self.fresh("t");
                let rw = self.rw(&expr.source, &[]);
                w.line(&format!("const {v} = document.createTextNode(\"\");"));
                // Slice 2 — record the hydration key on the text node so
                // the ship-js SSR path's serialization wraps it in
                // `<!--sui:text:bN-->...<!--/sui-->`. The browser DOM
                // has no `__hydrateKey` field so this call is a no-op;
                // the server-DOM reads the field at toHTML time.
                w.line(&format!(
                    "markHydrateText({v}, {k});",
                    k = js_string(&key),
                ));
                w.line(&format!("{parent_var}.appendChild({v});"));
                if self.is_static_row_expr(&expr.source) {
                    // D5: row-scope static read — skip the bindText
                    // effect and write textContent directly. Saves a
                    // closure + cleanup slot per row per binding. The
                    // assignment is also re-run by the for's
                    // `update()` callback so keyed-row reuse refreshes
                    // the DOM when the row instance changes.
                    let stmt = format!("{v}.textContent = ({rw});");
                    w.line(&stmt);
                    let use_ = self.row_use_for_source(&expr.source);
                    self.inline_update_stmts.push((stmt, use_));
                } else {
                    let fv = self.for_version_comma(&expr.source);
                    w.line(&format!(
                        "{cleanups_var}.push(bindText({v}, () => ({fv}{rw})));"
                    ));
                }
                // Phase 3b.3: at depth 0 in views, also emit a direct
                // hydrate call so SSR'd `<!--sui:text:bN-->…<!--/sui-->`
                // markers wire up without going through the runtime
                // walker / `bindings` manifest. The matching `text`
                // manifest entry is suppressed in `js.rs::emit_bindings`.
                if self.is_view && self.boundary_depth == 0 {
                    self.hydrate_uses_text = true;
                    self.hydrate_calls.line(&format!(
                        "if (__t[{k}]) __cleanups.push(hydrateText(__t[{k}], () => ({rw})));",
                        k = js_string(&key),
                    ));
                }
            }
            Instr::Element(el) => self.emit_element(w, el, parent_var, cleanups_var),
            Instr::If(if_instr) => self.emit_if(w, if_instr, parent_var, cleanups_var),
            Instr::For(for_instr) => self.emit_for(w, for_instr, parent_var, cleanups_var),
            Instr::Match(match_instr) => {
                self.emit_match(w, match_instr, parent_var, cleanups_var)
            }
            // While-loading/while-error: the render callback is emitted as a
            // named property on the setup return object (via `render_callbacks`
            // buffer). `__mount` emits an inline call for the legacy path.
            Instr::WhileLoading(wl) => {
                self.emit_while_loading_block(w, wl, parent_var, cleanups_var)
            }
            Instr::WhileError(we) => {
                self.emit_while_error_block(w, we, parent_var, cleanups_var)
            }
            Instr::Let(l) => self.emit_let_block(w, l, parent_var, cleanups_var),
        }
    }

    /// `Instr::Let` — snapshot a value expression into `$scope.<name>`
    /// before emitting the body, so body references like `<name>.field`
    /// (rewritten to `$scope.<name>.field` by `rewrite_for_scope`)
    /// resolve correctly. Only emitted by `{#async X as v}`.
    fn emit_let_block(
        &mut self,
        w: &mut Writer,
        instr: &crate::ir::LetInstr,
        parent_var: &str,
        cleanups_var: &str,
    ) {
        let value_rw = self.rw(&instr.value.source, &[]);
        w.line(&format!("$scope.{name} = ({value_rw});", name = instr.name));
        // The `as` binder lives on the runtime `$scope`; qualify body
        // references to it with `$scope.` (synthetic mode). (The synthetic
        // render currently rejects `{#async}` upstream; this keeps the emit
        // internally consistent and is a no-op in production.)
        let machinery_mark = self.machinery_locals.len();
        self.machinery_locals.push(instr.name.clone());
        for child in &instr.body {
            self.emit_one(w, child, parent_var, cleanups_var);
        }
        self.machinery_locals.truncate(machinery_mark);
    }

    /// Emit a `{ nodes, cleanup }` branch result. Used by `bindIf`,
    /// `bindFor`, and the `<while-*>` helpers.
    ///
    /// D5's static-inline path collects per-row update statements into
    /// `self.inline_update_stmts` so the surrounding `{#for}`'s
    /// `update(new, idx)` callback can re-run them on row reuse. But
    /// the vars they reference (text-node consts like `__t12`) are
    /// declared INSIDE this branch's closure — invisible to the outer
    /// `update`. Leaving the pushes in place crashes every navigation
    /// with `ReferenceError: __t12 is not defined`. Save + restore the
    /// stack so branch-local static bindings stay branch-local; their
    /// initial values are still set inline during branch render.
    pub(super) fn emit_branch(&mut self, w: &mut Writer, body: &[Instr]) {
        let outer_inline = std::mem::take(&mut self.inline_update_stmts);
        w.line("() => {");
        w.enter();
        let frag = self.fresh("frag");
        let cl = self.fresh("c");
        w.line(&format!(
            "const {frag} = document.createDocumentFragment();"
        ));
        w.line(&self.decl_cleanups(&cl));
        self.boundary_depth += 1;
        self.emit_children(w, body, &frag, &cl);
        self.boundary_depth -= 1;
        w.line(&format!(
            "return {{ nodes: Array.from({frag}.childNodes), cleanup: () => {{ for (const c of {cl}) try {{ c(); }} catch (e) {{}} }} }};"
        ));
        w.leave();
        w.line("},");
        self.inline_update_stmts = outer_inline;
    }
}
