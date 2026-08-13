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

pub mod canary;
pub mod diagnostics;
pub mod html;
pub mod mapping;
pub mod metrics;
pub mod style;
mod style_scale;
pub mod taffy_tree;
pub mod tap_targets;
mod unintended_overflow;

pub use diagnostics::{
    BoxExtents, BoxReport, Diagnostic, MainAxis, OverflowGeometry, ScanInput, ScanResult, Severity,
};
pub use html::{Element, parse_source, source_viewport};
pub use metrics::{Family, Size as TextSize, table as metrics_table};
pub use style::ResolvedStyle;
pub use taffy_tree::Layout;

use std::sync::Arc;

/// Parsed HTML forest. One parse, many viewport layouts.
#[derive(Debug, Clone)]
pub struct Forest {
    elements: Arc<[Element]>,
}

impl Forest {
    /// Parse already-preprocessed HTML once.
    pub fn parse(preprocessed: &str) -> Self {
        Self {
            elements: Arc::from(parse_source(preprocessed)),
        }
    }

    pub fn elements(&self) -> &[Element] {
        &self.elements
    }

    /// Compute a box tree at `viewport`. Shares this forest's elements.
    pub fn layout(&self, viewport: (f32, f32)) -> anyhow::Result<Layout> {
        taffy_tree::build_arc(Arc::clone(&self.elements), viewport)
    }
}

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

/// Options for layout finding kinds (SPEC-LAYOUT-DIAG-1 / LAYOUT-5).
///
/// `unintended-overflow` always emits. `clipped-text` follows
/// [`Self::text_metrics_enabled`] (on unless explicitly disabled).
/// `sub-min-tap-target` stays behind [`Self::experimental`].
#[derive(Debug, Clone, Copy)]
pub struct CheckOptions {
    /// Emit still-experimental kinds (`sub-min-tap-target`).
    pub experimental: bool,
    /// Text-metrics-dependent kinds (`clipped-text`). `None` means on
    /// (promoted default). Set `Some(false)` to disable.
    pub text_metrics: Option<bool>,
}

impl Default for CheckOptions {
    fn default() -> Self {
        Self {
            experimental: false,
            text_metrics: Some(true),
        }
    }
}

impl CheckOptions {
    /// Calibration / canary: experimental kinds on, including text-metrics.
    pub fn experimental() -> Self {
        Self {
            experimental: true,
            text_metrics: Some(true),
        }
    }

    /// Whether text-metrics kinds should run. Promoted default is on.
    pub fn text_metrics_enabled(self) -> bool {
        self.text_metrics.unwrap_or(true)
    }
}

/// Compute a layout for an already-preprocessed `.sail` template at `viewport`
/// px and emit diagnostics. This is the primary entry point for callers that
/// already have a preprocessor (e.g. `sail-compiler`).
///
/// Promoted kinds emit (`unintended-overflow`, `clipped-text`). Tap stays
/// experimental (see [`check_preprocessed_with`]).
pub fn check_preprocessed(preprocessed: &str, viewport: (f32, f32)) -> anyhow::Result<ScanResult> {
    check_preprocessed_with(preprocessed, viewport, CheckOptions::default())
}

/// Like [`check_preprocessed`], with explicit experimental-kind options.
///
/// `unintended-overflow` always emits. `clipped-text` requires
/// [`CheckOptions::text_metrics_enabled`]. `sub-min-tap-target` requires
/// [`CheckOptions::experimental`].
pub fn check_preprocessed_with(
    preprocessed: &str,
    viewport: (f32, f32),
    options: CheckOptions,
) -> anyhow::Result<ScanResult> {
    let forest = Forest::parse(preprocessed);
    check_tree(&forest.layout(viewport)?, options)
}

/// Scan an already-computed box tree. Callers that share a [`Forest`] across
/// viewports use this so they do not re-parse HTML.
pub fn check_tree(layout: &Layout, options: CheckOptions) -> anyhow::Result<ScanResult> {
    diagnostics::scan_with(layout, layout.viewport, options)
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
