//! `sui-layout` — compile-time box-tree computation for `.sui` templates.
//!
//! SPIKE for **Job H — taffy: the compiler computes the box tree, with no
//! browser**. The thesis is that the closed vocabulary pays for static layout:
//! no cascade, tokens resolve to pixels, layout intent is declared.
//!
//! This crate is a **diagnostic engine**, not a renderer. "This box is 0px tall
//! with two children" is a safe report. "This looks right" is not — taffy's
//! pixel arithmetic differs from every browser at the edges.
//!
//! ## Pipeline
//!
//! ```text
//!   .sui source
//!        │
//!        ├── sui_compiler::parse::preprocess::preprocess  (sx→class, layout→div+class)
//!        ▼
//!   lowered HTML ──► sui_layout::html::parse  ──►  Vec<Element>
//!        │
//!        ├── sui_layout::style::resolve  (class → taffy Style)
//!        ▼
//!   taffy::TaffyTree ──► compute_layout at viewport
//!        │
//!        ├── sui_layout::diagnostics::scan
//!        ▼
//!   Vec<Diagnostic>
//! ```
//!
//! ## What is NOT in this spike
//!
//! - **Text shaping.** Without a font shaper (cosmic-text, fontdue), a leaf
//!   `<p>` measures `0×0`. We supply a placeholder intrinsic size and call
//!   that gap out explicitly in the verdict.
//! - **Multiple viewports / breakpoints.** One fixed viewport per call.
//! - **`if` / `for` / dynamic content.** The static-only path catches most
//!   real-world layout bugs.

pub mod html;
pub mod style;
pub mod taffy_tree;
pub mod diagnostics;

pub use diagnostics::{Diagnostic, ScanResult, ScanInput, Severity};
pub use html::{Element, parse_source, source_viewport};
pub use style::ResolvedStyle;
pub use taffy_tree::Layout;

/// Re-export the engine we wrap. External callers (the bin, future LSP/seam
/// hooks) build trees and call compute_layout through this alias.
pub use taffy;

/// Compute a layout for a `.sui` source string at `viewport` px and emit
/// diagnostics. The single entry API: every other surface lowers into this.
pub fn check(source: &str, viewport: (f32, f32)) -> anyhow::Result<ScanResult> {
    // 1. Lower sx + layout keywords to atomic CSS the same way the compiler
    let preprocessed = sui_compiler::parse::preprocess::preprocess(source);

    // 2. Parse the lowered HTML (controller-style block already extracted).
    let elements = parse_source(&preprocessed);

    // 3. Resolve every element to a taffy style and compute the tree.
    let tree = taffy_tree::build(elements, viewport)?;

    // 4. Walk the computed tree and emit diagnostics.
    diagnostics::scan(&tree, viewport)
}
