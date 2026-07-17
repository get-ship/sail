//! Intermediate representation. Built by `parse`, consumed by
//! `lower` + `emit`. Nodes carry the source `Span` so diagnostics
//! generated downstream still point at the user's file.

pub mod directive;
pub mod expr;
pub mod template;

pub use directive::*;
pub use expr::*;
pub use template::*;

use crate::diagnostics::Span;

/// The full IR for a single `.sui` source file.
#[derive(Debug, Clone)]
pub struct Program {
    /// Props declared by the `<scope>` block (`props:` fields + variant axes).
    pub props: Vec<PropDecl>,
    /// CSS lifted from top-level `<style>` elements (component files only).
    pub styles: String,
    /// Raw `type` / `export type` alias text from the `<scope>` block,
    /// appended to the controller module so aliases are reusable there
    /// (type aliases hoist — appending preserves controller spans).
    pub scope_types: String,
    pub template: Vec<Instr>,
    /// Total source length — useful for serialization tools.
    pub source_len: usize,
    /// Span of the whole input, for top-level diagnostics.
    pub span: Span,
}
