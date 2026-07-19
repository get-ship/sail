//! Binding-manifest pre-pass.
//!
//! Walks the IR depth-first in the same source order the SSR render
//! walker uses today, allocating one `b<N>` key per binding-bearing
//! source position and recording a `Binding` entry that describes it.
//!
//! Two consumers downstream:
//! 1. `emit::js` writes the manifest into the compiled ESM as
//!    `export const bindings = [...]`. Phase E's attach-only client
//!    runtime imports it and resolves author-syntax markers by
//!    indexing into the array.
//! 2. The render walker (`render::walker`) shares the same allocator —
//!    its emitted marker keys are exactly the registry's keys, so the
//!    SSR HTML and the manifest agree on what `b<N>` refers to.
//!
//! ## Branch handling
//!
//! Control-flow bodies (`{#if}`, `{#match}`) are walked across **every**
//! branch (taken or not). The registry has no SSR scope, so it can't
//! tell which branch will render at SSR time — the marker contract
//! (`docs/ship-adrs/.../sail-marker-contract.md`) takes the position
//! that the manifest is the union of all bindings the IR could emit;
//! markers in the SSR'd HTML reference a subset.
//!
//! `{#for}` bodies are walked **once** — the walker's snapshot/restore
//! makes every iteration share the same key range, so the registry
//! mirrors that with a single body walk.

use crate::ir::*;
use crate::keys::{Binding, BindingKind, BindingRegistry, KeyAllocator};
use crate::parse::expr::collect_idents;
use crate::escape::is_void_element;

#[cfg(test)]
mod tests;

/// Build the binding manifest for a parsed program.
pub fn build(program: &Program) -> BindingRegistry {
    let mut b = Builder::default();
    b.walk_block(&program.template);
    BindingRegistry::from_bindings(b.bindings)
}

#[derive(Default)]
struct Builder {
    keys: KeyAllocator,
    bindings: Vec<Binding>,
    /// When > 0, keys are allocated (for render-walker parity) but entries
    /// are marked `suppressed` — they appear in the registry for debug-assert
    /// alignment but are excluded from the emitted JS manifest.
    suppress_depth: usize,
    /// Sequential counters for while-loading/while-error render-callback names.
    /// Must stay in lockstep with `TemplateEmitter::while_loading_idx` /
    /// `while_error_idx` so both passes generate the same render-key strings.
    while_loading_idx: usize,
    while_error_idx: usize,
}

impl Builder {
    fn push(&mut self, kind: BindingKind, expr_src: &str, allow_arrow: bool) {
        let key = self.keys.alloc();
        self.bindings.push(Binding {
            key,
            kind,
            expr: expr_src.to_string(),
            deps: collect_idents(expr_src, allow_arrow),
            render_key: None,
            suppressed: self.suppress_depth > 0,
        });
    }

    fn walk_block(&mut self, instrs: &[Instr]) {
        for i in instrs {
            self.walk_instr(i);
        }
    }

    fn walk_instr(&mut self, instr: &Instr) {
        match instr {
            Instr::StaticText { .. } | Instr::Comment { .. } => {}
            Instr::DynamicText { expr } => {
                self.push(BindingKind::Text, &expr.source, expr.allow_arrow);
            }
            Instr::Element(el) => self.walk_element(el),
            Instr::If(if_instr) => self.walk_if(if_instr),
            Instr::For(for_instr) => self.walk_for(for_instr),
            Instr::Match(match_instr) => self.walk_match(match_instr),
            Instr::WhileLoading(wl) => self.walk_while_loading(wl),
            Instr::WhileError(we) => self.walk_while_error(we),
        }
    }

    fn walk_element(&mut self, el: &ElementInstr) {
        let lower = el.tag.to_ascii_lowercase();
        for attr in &el.attrs {
            self.walk_attr(attr);
        }
        if !is_void_element(&lower) {
            self.walk_block(&el.children);
        }
    }

    fn walk_attr(&mut self, attr: &AttrInstr) {
        match attr {
            AttrInstr::Static { .. } => {}
            AttrInstr::Expr { name, expr, .. } => {
                self.push(BindingKind::Attr(name.clone()), &expr.source, expr.allow_arrow);
            }
            AttrInstr::Class { name, expr, .. } => {
                self.push(BindingKind::Class(name.clone()), &expr.source, expr.allow_arrow);
            }
            AttrInstr::Prop { name, expr, .. } => {
                self.push(BindingKind::Prop(name.clone()), &expr.source, expr.allow_arrow);
            }
            AttrInstr::BindValue { expr, .. } => {
                self.push(BindingKind::BindValue, &expr.source, expr.allow_arrow);
            }
            AttrInstr::BindChecked { expr, .. } => {
                self.push(BindingKind::BindChecked, &expr.source, expr.allow_arrow);
            }
            AttrInstr::BindThis { var, .. } => {
                // The "expr" carried by the manifest is the bare scope
                // variable name; deps is the same single identifier so the
                // attach runtime knows which scope key to write into.
                self.push(BindingKind::BindThis, var, false);
            }
            AttrInstr::SlotReflectEmpty { .. } => {
                // Side-effect wiring on the host's attribute set. No scope
                // entry, no marker — the compiled component setup body
                // emits the runtime call inline; routes never see this kind.
            }
            AttrInstr::On { event, handler, .. } => {
                self.push(BindingKind::On(event.clone()), &handler.source, handler.allow_arrow);
            }
            AttrInstr::DynamicAttr { expr, .. } => {
                self.push(BindingKind::AttrDynamic, &expr.source, expr.allow_arrow);
            }
            AttrInstr::Mutation { expr, .. } => {
                self.push(BindingKind::Mutation, &expr.source, expr.allow_arrow);
            }
AttrInstr::Use { .. } => {
                // Actions register on element construction, same as
                // transitions — no SSR marker, no key, no manifest entry.
            }
            // `motion:` produces pre-baked `element.animate(...)` calls
            // — no binding-key allocation, no SSR marker, no manifest
            // entry. Same shape as transitions: registration happens at
            // element construction in `template_emit/dom_emit.rs`.
            AttrInstr::Motion { .. } => {}
        }
    }

    /// `{#if}`: one binding for the boundary, then walk every branch
    /// body and the optional else body. The boundary's `expr` is the
    /// first branch's condition (the canonical discriminator);
    /// alternate-branch conditions live in the IR but the manifest's
    /// shape is one row per allocated key.
    fn walk_if(&mut self, instr: &IfInstr) {
        let cond_src = instr
            .branches
            .first()
            .map(|b| b.cond.source.as_str())
            .unwrap_or("");
        self.push(BindingKind::If, cond_src, false);
        for branch in &instr.branches {
            self.walk_block(&branch.body);
        }
        if let Some(else_body) = &instr.else_body {
            self.walk_block(else_body);
        }
    }

    /// `{#for}`: one binding for the boundary (expr = iterable source),
    /// body walked once, else body walked once.
    fn walk_for(&mut self, instr: &ForInstr) {
        self.push(BindingKind::For, &instr.iterable.source, false);
        self.walk_block(&instr.body);
        if let Some(else_body) = &instr.else_body {
            self.walk_block(else_body);
        }
    }

    /// `{#match}`: one binding for the boundary (expr = discriminant),
    /// each case body walked, default walked.
    fn walk_match(&mut self, instr: &MatchInstr) {
        self.push(BindingKind::Match, &instr.discriminant.source, false);
        for case in &instr.cases {
            self.walk_block(&case.body);
        }
        if let Some(default) = &instr.default {
            self.walk_block(default);
        }
    }

    /// `{#while-loading of <sig>}`: one boundary entry (with `renderKey`);
    /// body walked under suppression so the render walker's key counter
    /// stays in sync but no flat manifest entries are emitted for body content.
    fn walk_while_loading(&mut self, instr: &WhileLoadingInstr) {
        let idx = self.while_loading_idx;
        self.while_loading_idx += 1;
        let render_key = format!("__renderWhileLoading_{idx}");
        let boundary_key = self.keys.alloc();
        self.bindings.push(Binding {
            key: boundary_key,
            kind: BindingKind::WhileLoading,
            expr: instr.bind.clone(),
            deps: vec![instr.bind.clone()],
            render_key: Some(render_key),
            suppressed: self.suppress_depth > 0,
        });
        self.suppress_depth += 1;
        self.walk_block(&instr.body);
        self.suppress_depth -= 1;
    }

    /// `{#while-error <errVar> of <sig>}`: one boundary entry (with `renderKey`);
    /// body walked under suppression for key-counter parity.
    fn walk_while_error(&mut self, instr: &WhileErrorInstr) {
        let idx = self.while_error_idx;
        self.while_error_idx += 1;
        let render_key = format!("__renderWhileError_{idx}");
        let boundary_key = self.keys.alloc();
        self.bindings.push(Binding {
            key: boundary_key,
            kind: BindingKind::WhileError,
            expr: instr.bind.clone(),
            deps: vec![instr.bind.clone()],
            render_key: Some(render_key),
            suppressed: self.suppress_depth > 0,
        });
        self.suppress_depth += 1;
        self.walk_block(&instr.body);
        self.suppress_depth -= 1;
    }
}
