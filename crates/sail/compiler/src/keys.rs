//! Stable opaque marker keys (`b0`, `b1`, …) and the binding manifest
//! they index into.
//!
//! Phase E-prep lifts the key allocator out of the SSR walker into a
//! shared module so the render pass and the `bindings` pre-pass agree
//! on what `b<N>` refers to. Both passes traverse the IR in identical
//! source order; the registry pass records one `Binding` per key, the
//! render pass emits the markers and consumes the registry by index.
//!
//! See `crates/sail/compiler/src/bindings/mod.rs` for the
//! traversal that builds the registry, and the `sail-marker-contract`
//! ADR for the marker shapes that every kind here corresponds to.

/// Allocates stable opaque marker keys (`b0`, `b1`, …) in source order.
#[derive(Debug, Default)]
pub struct KeyAllocator {
    next: usize,
}

impl KeyAllocator {
    pub fn alloc(&mut self) -> String {
        let key = format!("b{}", self.next);
        self.next += 1;
        key
    }

    /// Snapshot the next-key counter. Pairs with `restore` to give
    /// `{#for}` iterations identical key sequences inside their bodies
    /// (Phase E's attach runtime needs one binding per source-position,
    /// not per DOM node).
    pub fn snapshot(&self) -> usize {
        self.next
    }

    /// Restore the next-key counter to a prior snapshot value.
    pub fn restore(&mut self, n: usize) {
        self.next = n;
    }
}

/// One entry in the binding manifest. Index in the registry === the
/// numeric portion of the marker key (`bindings[3]` matches `b3`).
#[derive(Debug, Clone)]
pub struct Binding {
    /// `"b0"`, `"b1"`, … — the opaque counter-allocated binding key.
    pub key: String,
    pub kind: BindingKind,
    /// Raw expression source from the IR (or a discriminator like the
    /// `for` iterable / `match` discriminant). Empty when the binding
    /// has no associated expression.
    pub expr: String,
    /// Identifier names read in the expression. De-duplicated, in
    /// first-encounter order. Property accesses chain to the root.
    pub deps: Vec<String>,
    /// For `WhileLoading` / `WhileError` boundaries: the name of the
    /// render-callback export on the compiled module (e.g.
    /// `"__renderWhileLoading_b3"`). `None` for all other kinds.
    pub render_key: Option<String>,
    /// When `true`, this entry was allocated for key-counter parity
    /// (inside a while-loading/while-error body) but must not appear
    /// in the emitted JS manifest — the content is rendered imperatively
    /// by the boundary's render callback.
    pub suppressed: bool,
}

/// Mirrors the marker `kind` used by the attach runtime (Phase E)
/// to decide how to wire each binding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BindingKind {
    /// `<!--sail:text:b<N>-->{value}<!--/sail-->` interpolation.
    Text,
    /// `on:event` handler. Carries the event name (`"click"`, …).
    On(String),
    /// `bind:value`.
    BindValue,
    /// `bind:checked`.
    BindChecked,
    /// Dynamic plain-field binding. Marker name form is still `attr:NAME`
    /// (SSR mechanism string, not an author `attr:` directive).
    Attr(String),
    /// `{#if}` boundary.
    If,
    /// `{#for}` boundary.
    For,
    /// `{#match}` boundary.
    Match,
    /// `<form mutation="client.method">` — form mutation directive.
    Mutation,
    /// `{#while-loading of <signal>}` boundary.
    WhileLoading,
    /// `{#while-error <errVar> of <signal>}` boundary.
    WhileError,
}

impl BindingKind {
    /// String form of the binding kind.
    /// Used to emit the `kind` field of each binding object literal.
    pub fn as_marker_str(&self) -> String {
        match self {
            BindingKind::Text => "text".into(),
            BindingKind::On(ev) => format!("on:{ev}"),
            BindingKind::BindValue => "bind:value".into(),
            BindingKind::BindChecked => "bind:checked".into(),
            BindingKind::Attr(n) => format!("attr:{n}"),
            BindingKind::If => "if".into(),
            BindingKind::For => "for".into(),
            BindingKind::Match => "match".into(),
            BindingKind::Mutation => "mutation".into(),
            BindingKind::WhileLoading => "while-loading".into(),
            BindingKind::WhileError => "while-error".into(),
        }
    }
}

/// The full per-module binding manifest. Built once by
/// `bindings::BindingRegistry::build`, consumed by both the render
/// walker (which checks parity) and the JS emitter (which writes
/// `export const bindings = [...]`).
#[derive(Debug, Default, Clone)]
pub struct BindingRegistry {
    bindings: Vec<Binding>,
}

impl BindingRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_bindings(bindings: Vec<Binding>) -> Self {
        Self { bindings }
    }

    pub fn push(&mut self, binding: Binding) {
        self.bindings.push(binding);
    }

    pub fn get(&self, index: usize) -> Option<&Binding> {
        self.bindings.get(index)
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Binding> {
        self.bindings.iter()
    }

    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_allocator_yields_sequential_b_keys() {
        let mut k = KeyAllocator::default();
        assert_eq!(k.alloc(), "b0");
        assert_eq!(k.alloc(), "b1");
        assert_eq!(k.alloc(), "b2");
    }

    #[test]
    fn snapshot_restore_reuses_keys() {
        let mut k = KeyAllocator::default();
        let _ = k.alloc();
        let snap = k.snapshot();
        assert_eq!(k.alloc(), "b1");
        assert_eq!(k.alloc(), "b2");
        k.restore(snap);
        assert_eq!(k.alloc(), "b1");
    }

    #[test]
    fn binding_kind_marker_strings() {
        assert_eq!(BindingKind::Text.as_marker_str(), "text");
        assert_eq!(BindingKind::On("click".into()).as_marker_str(), "on:click");
        assert_eq!(BindingKind::BindValue.as_marker_str(), "bind:value");
        assert_eq!(BindingKind::BindChecked.as_marker_str(), "bind:checked");
        assert_eq!(
            BindingKind::Attr("href".into()).as_marker_str(),
            "attr:href"
        );
        assert_eq!(BindingKind::If.as_marker_str(), "if");
        assert_eq!(BindingKind::For.as_marker_str(), "for");
        assert_eq!(BindingKind::Match.as_marker_str(), "match");
    }
}
