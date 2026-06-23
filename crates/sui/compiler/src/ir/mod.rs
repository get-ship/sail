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
    pub data: Vec<DataDecl>,
    /// Props declared by top-level `<prop>` directives (component files only).
    pub props: Vec<PropDecl>,
    /// Host-injected data/ops clients declared by top-level `<client>`
    /// directives. Each name is legal in `<data source>` + handler scope and
    /// is provided by the host at mount — the implementor-named replacement
    /// for the hardcoded ambient `ship`.
    pub clients: Vec<ClientDecl>,
    /// CSS lifted from top-level `<style>` elements (component files only).
    pub styles: String,
    pub template: Vec<Instr>,
    /// Total source length — useful for serialization tools.
    pub source_len: usize,
    /// Span of the whole input, for top-level diagnostics.
    pub span: Span,
}
