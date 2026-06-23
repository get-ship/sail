//! IR walker. Owns the output buffer + key allocator + scope and emits
//! HTML one instruction at a time. Phase D-3 fills in control-flow
//! bodies against the initial scope; D-2 brought renderable-subset
//! evaluation into text/attribute output, and D-1 nailed the marker
//! shapes. On evaluator failure we fall back to the marker-only shape
//! so client-side attachment still works.
//!
//! ## Phase E-prep — registry parity
//!
//! Key allocation is shared with `bindings::BindingRegistry`. The
//! registry pre-walks the IR (every branch, taken or not) and assigns
//! `b<N>` keys; the walker walks the IR again here, emitting HTML, and
//! its `b<N>` keys must match the registry index-for-index. To keep
//! the counter aligned across un-rendered branches, the walker walks
//! every branch — but suppresses HTML output for the un-taken ones via
//! the `Out` wrapper's silent mode.
//!
//! Phase 2 deleted the `/_deck/render-fragment/` POST endpoint, so the
//! walker no longer carries the `FragmentState` plumbing for
//! single-boundary fragment renders. Control-flow swap is now
//! client-side; the SSR walker only emits the active-branch HTML at
//! first paint, and `attach()` takes over from there.
//!
//! Element / attribute emission lives in `walker_attrs.rs` to keep
//! this file under the 300-line cap.

use crate::ir::Instr;
use crate::keys::{BindingRegistry, KeyAllocator};

use super::control_flow;
use super::escape::escape_text_value;
use super::eval::{Scope, eval, stringify};
use super::out::Out;

pub(crate) struct Walker<'r> {
    pub(super) out: Out,
    pub(super) keys: KeyAllocator,
    pub(super) scope: Scope,
    pub(super) registry: &'r BindingRegistry,
    /// Counter for parity with the registry. Increments each time the
    /// walker would have allocated a key. Used by `next_key` to assert
    /// the registry's `b<N>` matches.
    pub(super) reg_idx: usize,
}

impl<'r> Walker<'r> {
    pub(crate) fn new_named(
        scope: Scope,
        registry: &'r BindingRegistry,
        _view_name: String,
    ) -> Self {
        Self {
            out: Out::new(),
            keys: KeyAllocator::default(),
            scope,
            registry,
            reg_idx: 0,
        }
    }

    pub(crate) fn into_html(self) -> String {
        self.out.into_string()
    }

    /// Allocate the next `b<N>` key. Asserts (in debug builds) that
    /// the registry's entry at the matching index has the same key
    /// string — drift means the two traversals fell out of sync.
    pub(super) fn next_key(&mut self) -> String {
        let key = self.keys.alloc();
        debug_assert_eq!(
            self.registry.get(self.reg_idx).map(|b| b.key.as_str()),
            Some(key.as_str()),
            "walker/registry key drift at index {}",
            self.reg_idx
        );
        self.reg_idx += 1;
        key
    }

    /// Walk a block of instructions with HTML output suppressed.
    /// Key allocation continues so the registry-parity counter stays
    /// in lockstep with un-taken branches.
    pub(super) fn walk_block_silent(&mut self, instrs: &[Instr]) {
        self.out.enter_silent();
        self.walk_block(instrs);
        self.out.exit_silent();
    }

    pub(crate) fn walk_block(&mut self, instrs: &[Instr]) {
        for instr in instrs {
            self.walk_instr(instr);
        }
    }

    pub(super) fn walk_instr(&mut self, instr: &Instr) {
        match instr {
            Instr::StaticText { text, .. } => self.out.push_str(text),
            Instr::Comment { data, .. } => {
                self.out.push_str("<!--");
                self.out.push_str(data);
                self.out.push_str("-->");
            }
            Instr::DynamicText { expr } => {
                let key = self.next_key();
                let inner = eval(&expr.source, &self.scope)
                    .map(|v| escape_text_value(&stringify(&v)))
                    .unwrap_or_default();
                self.out.push_str(&format!("<!--sui:text:{key}-->{inner}<!--/sui-->"));
            }
            Instr::Element(el) => self.walk_element(el),
            Instr::If(if_instr) => control_flow::render_if(self, if_instr),
            Instr::For(for_instr) => control_flow::render_for(self, for_instr),
            Instr::Match(match_instr) => control_flow::render_match(self, match_instr),
            Instr::WhileLoading(wl) => control_flow::render_while_loading(self, wl),
            Instr::WhileError(we) => control_flow::render_while_error(self, we),
            Instr::Let(l) => control_flow::render_let(self, l),
        }
    }
}
