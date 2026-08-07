//! `sail-layout` — compile-time box-tree computation for `.sail` templates.
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
//!   .sail source
//!        │
//!        ├── caller-supplied preprocess  (sx→class, layout→div+class)
//!        ▼
//!   lowered HTML ──► sail_layout::html::parse  ──►  Vec<Element>
//!        │
//!        ├── sail_layout::style::resolve  (class → taffy Style)
//!        ▼
//!   taffy::TaffyTree<NodeContext> ──► compute_layout_with_measure
//!        │                              (text leaves use the metrics table)
//!        ├── sail_layout::diagnostics::scan
//!        ▼
//!   Vec<Diagnostic>
//! ```
//!
//! ## What is measured and what is not
//!
//! - **Text leaves.** The parser captures text content onto elements; the
//!   taffy measure function resolves `font-*` and `text-*` classes (and tag
//!   defaults) to a baked metrics table generated from the framework's own
//!   woff2 fonts in `packages/sail/tokens/src/fonts/`. See [`metrics`] for the
//!   contract and the list of deliberate cuts (kerning, ligatures, BiDi).
//! - **Custom / replaced elements.** Tags the layout pass cannot size (custom
//!   elements containing a hyphen, plus replaced elements like `<iframe>` that
//!   taffy does not model) are marked `unknown_subtree`. Any collapse or
//!   zero-size diagnostic whose subtree contains an unknown node is
//!   suppressed, because reporting a parent as "collapsed" when its only
//!   child is an unmeasurable web component would be confidently wrong.
//! - **Multiple viewports / breakpoints.** One fixed viewport per
//!   [`check_preprocessed`] call; the compiler's layout check pass runs at
//!   least two (mobile + desktop) and dedups.
//! - **`if` / `for` / dynamic content.** The static-only path catches most
//!   real-world layout bugs.

pub mod diagnostics;
pub mod html;
pub mod mapping;
pub mod metrics;
pub mod style;
mod style_scale;
pub mod taffy_tree;

pub use diagnostics::{BoxReport, Diagnostic, ScanInput, ScanResult, Severity};
pub use html::{Element, parse_source, source_viewport};
pub use metrics::{Family, Size as TextSize, table as metrics_table};
pub use style::ResolvedStyle;
pub use taffy_tree::Layout;

/// Re-export the engine we wrap. External callers (the bin, future LSP/seam
/// hooks) build trees and call compute_layout through this alias.
pub use taffy;

use std::sync::atomic::{AtomicU64, Ordering};

/// Process-wide count of taffy layout computations (each `taffy_tree::build`).
/// Test/hook surface for SPEC-TOOLING-1 §WI6: LSP on-type must not increment
/// this while typing.
static LAYOUT_PASS_COUNT: AtomicU64 = AtomicU64::new(0);

/// Number of taffy layout passes since process start (or last
/// [`reset_layout_pass_count`]).
pub fn layout_pass_count() -> u64 {
    LAYOUT_PASS_COUNT.load(Ordering::SeqCst)
}

/// Reset the layout-pass counter (tests).
pub fn reset_layout_pass_count() {
    LAYOUT_PASS_COUNT.store(0, Ordering::SeqCst);
}

pub(crate) fn record_layout_pass() {
    LAYOUT_PASS_COUNT.fetch_add(1, Ordering::SeqCst);
}

#[cfg(test)]
mod layout_pass_counter_tests {
    use super::*;

    #[test]
    fn build_increments_layout_pass_count() {
        reset_layout_pass_count();
        let before = layout_pass_count();
        let elems = parse_source("<div>hi</div>");
        let _ = taffy_tree::build(elems, (400.0, 300.0));
        assert!(layout_pass_count() > before);
    }
}

/// Compute a layout for an already-preprocessed `.sail` template at `viewport`
/// px and emit diagnostics. This is the primary entry point for callers that
/// already have a preprocessor (e.g. `sail-compiler`).
pub fn check_preprocessed(preprocessed: &str, viewport: (f32, f32)) -> anyhow::Result<ScanResult> {
    // 1. Parse the lowered HTML (controller-style block already extracted).
    let elements = parse_source(preprocessed);

    // 2. Resolve every element to a taffy style and compute the tree.
    let tree = taffy_tree::build(elements, viewport)?;

    // 3. Walk the computed tree and emit diagnostics.
    diagnostics::scan(&tree, viewport)
}

/// Convenience for standalone callers that have their own preprocessor.
/// `preprocess` must perform the same sx + layout lowering the compiler does.
pub fn check_with_preprocessor(
    source: &str,
    viewport: (f32, f32),
    preprocess: impl FnOnce(&str) -> String,
) -> anyhow::Result<ScanResult> {
    check_preprocessed(&preprocess(source), viewport)
}
