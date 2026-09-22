//! `spinnaker::layout` — compile-time box-tree computation for `.sail` templates.
//!
//! SPIKE for **Job H — taffy: the compiler computes the box tree, with no
//! browser**. The thesis is that the closed vocabulary pays for static layout:
//! no cascade, tokens resolve to pixels, layout intent is declared.
//!
//! This module is a **diagnostic engine**, not a renderer. "This box is 0px tall
//! with two children" is a safe report. "This looks right" is not — taffy's
//! pixel arithmetic differs from every browser at the edges.
//!
//! ## Pipeline
//!
//! ```text
//!   .sail source
//!        │
//!        ├── caller-supplied preprocess  (Style IR → classes + TypeHost + BoxHost)
//!        ▼
//!   lowered HTML ──► spinnaker::layout::html::parse  ──►  Vec<Element>
//!        │
//!        ├── apply_resolved_types (Style tokens + theme tag defaults)
//!        ├── apply_resolved_boxes (Style box tokens)
//!        ├── spinnaker::layout::style::resolve  (Style IR + layout primitives → taffy)
//!        ▼
//!   taffy::TaffyTree<NodeContext> ──► compute_layout_with_measure
//!        │                              (text leaves use the metrics table)
//!        ├── spinnaker::layout::diagnostics::scan
//!        ▼
//!   Vec<Diagnostic>
//! ```
//!
//! ## What is measured and what is not
//!
//! - **Text leaves.** The parser captures text content onto elements; the
//!   taffy measure function reads Style-resolved type tokens
//!   ([`Element::type_size`] / [`Element::type_family`]) and looks up the
//!   baked metrics table generated from the framework's own woff2 fonts in
//!   `packages/sail/tokens/src/fonts/`. See [`metrics`] for the contract
//!   and the list of deliberate cuts (kerning, ligatures, BiDi).
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

pub mod box_ir;
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

pub use box_ir::{BoxHost, ResolvedBox, apply_resolved_boxes, resolve_box};
pub use diagnostics::{
    BoxExtents, BoxReport, Diagnostic, MainAxis, OverflowGeometry, ScanInput, ScanResult, Severity,
};

/// Style-resolved hosts collected at preprocess (type + box).
#[derive(Debug, Clone, Default)]
pub struct StyleHosts {
    pub types: Vec<TypeHost>,
    pub boxes: Vec<BoxHost>,
}
pub use html::{Element, parse_source, source_viewport};
pub use metrics::{
    Family, Size as TextSize, TypeHost, apply_resolved_types, apply_tag_default_types,
    table as metrics_table,
};
pub use style::ResolvedStyle;
pub use taffy_tree::Layout;

/// Re-export the engine we wrap. External callers (the bin, future LSP/seam
/// hooks) build trees and call compute_layout through this alias.
pub use taffy;

use std::cell::Cell;

// Per-thread, not process-global: parallel cargo-test threads must not
// race reset / increment / assert on this hook.
thread_local! {
    static LAYOUT_PASS_COUNT: Cell<u64> = const { Cell::new(0) };
}

/// Number of taffy layout passes on this thread since start (or last
/// [`reset_layout_pass_count`]).
///
/// Test/hook surface for SPEC-TOOLING-1 §WI6: LSP on-type must not
/// increment this while typing.
pub fn layout_pass_count() -> u64 {
    LAYOUT_PASS_COUNT.with(Cell::get)
}

/// Reset the layout-pass counter for this thread (tests).
pub fn reset_layout_pass_count() {
    LAYOUT_PASS_COUNT.with(|c| c.set(0));
}

pub(crate) fn record_layout_pass() {
    LAYOUT_PASS_COUNT.with(|c| c.set(c.get() + 1));
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
/// [`Self::text_metrics_enabled`] (default on). `sub-min-tap-target` stays
/// behind [`Self::experimental`].
#[derive(Debug, Clone, Copy)]
pub struct CheckOptions {
    /// Emit remaining experimental kinds (`sub-min-tap-target`).
    pub experimental: bool,
    /// Text-metrics-dependent `clipped-text`. `None` means the promoted
    /// default (on). `Some(false)` is the kill switch.
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
    /// Strict / experimental rollout: tap + text-metrics on.
    pub fn experimental() -> Self {
        Self {
            experimental: true,
            text_metrics: Some(true),
        }
    }

    /// Whether text-metrics kinds should run.
    pub fn text_metrics_enabled(self) -> bool {
        self.text_metrics.unwrap_or(true)
    }
}

/// Compute a layout for an already-preprocessed `.sail` template at `viewport`
/// px and emit diagnostics. This is the primary entry point for callers that
/// already have a preprocessor (e.g. `sail-compiler`).
///
/// Promoted kinds emit; tap stays off (see [`check_preprocessed_with`]).
pub fn check_preprocessed(preprocessed: &str, viewport: (f32, f32)) -> anyhow::Result<ScanResult> {
    check_preprocessed_with(preprocessed, viewport, CheckOptions::default())
}

/// Like [`check_preprocessed`], with explicit experimental-kind options.
///
/// `unintended-overflow` always emits. `clipped-text` follows
/// [`CheckOptions::text_metrics_enabled`]. `sub-min-tap-target` needs
/// [`CheckOptions::experimental`].
pub fn check_preprocessed_with(
    preprocessed: &str,
    viewport: (f32, f32),
    options: CheckOptions,
) -> anyhow::Result<ScanResult> {
    check_preprocessed_with_types(preprocessed, viewport, options, &[])
}

/// Like [`check_preprocessed_with`], overlaying Style-resolved type tokens
/// collected at preprocess (not scraped from class names).
pub fn check_preprocessed_with_types(
    preprocessed: &str,
    viewport: (f32, f32),
    options: CheckOptions,
    hosts: &[TypeHost],
) -> anyhow::Result<ScanResult> {
    check_preprocessed_with_hosts(preprocessed, viewport, options, hosts, &[])
}

/// Like [`check_preprocessed_with_types`], also overlaying Style-resolved box
/// tokens (display / grow / overflow / position / pad / size).
pub fn check_preprocessed_with_hosts(
    preprocessed: &str,
    viewport: (f32, f32),
    options: CheckOptions,
    type_hosts: &[TypeHost],
    box_hosts: &[BoxHost],
) -> anyhow::Result<ScanResult> {
    let mut elements = parse_source(preprocessed);
    apply_resolved_types(&mut elements, type_hosts);
    apply_resolved_boxes(&mut elements, box_hosts);
    let tree = taffy_tree::build(elements, viewport)?;
    check_tree(&tree, options)
}

/// Scan an already-computed box tree. Callers that share one HTML forest
/// across viewports use this so they do not re-parse or rebuild taffy.
pub fn check_tree(layout: &Layout, options: CheckOptions) -> anyhow::Result<ScanResult> {
    diagnostics::scan_with(layout, layout.viewport, options)
}

/// Convenience for standalone callers that have their own preprocessor.
/// `preprocess` must perform the same Style + layout lowering the compiler does.
pub fn check_with_preprocessor(
    source: &str,
    viewport: (f32, f32),
    preprocess: impl FnOnce(&str) -> String,
) -> anyhow::Result<ScanResult> {
    check_preprocessed(&preprocess(source), viewport)
}
