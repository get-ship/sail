//! SAIL authoring lint engine.
//!
//! Lints `*.controller.ts` (paired SAIL controllers), `*.sail` templates,
//! the `.ts` composition modules under a component pack's `src/`, plain app
//! TypeScript under `apps/*/src/**`, and **author** `use:` Action bodies under
//! app `src/actions/**` (ACTIONS-1 regime). Rule IDs are prefixed
//! `sail-<descriptor>`.
//!
//! The framework catalog (`packages/sail/runtime/src/actions/*`) is trusted and
//! is not floor-linted this cycle.

pub mod config;
pub mod prop_registry;
pub mod report;
pub mod rules;
pub mod source;

// CHANNELS-1: Style pipeline lives in `sail-style`. Re-export under the
// historical `sail_lint::{style,sx_ir,sx_vocab,scales,tokens}` paths so
// compiler/layout/ops/lsp keep compiling without a fan-out import thrash.
pub use sail_style::{component_surfaces, host_ctx, scales, style_ir, style_vocab, token_groups, tokens};

/// Style product surface — re-export of `sail-style` (parse / IR / emit / clash).
pub mod style {
    pub use sail_style::{
        CONDITION_PREFIXES, Channel, CssRule, OBJECT_KEYS, PaintArg, PaintOp, Style,
        StyleCompleteContext, StyleCompleteItem, StyleCompleteKind, StyleDiagnostic, StyleEnv,
        TEXT_BEARING_ELEMENTS, channel_for_setter, clash, collect_motion_attaches,
        collect_style_const_names, color_paint_vocabulary, condition_media, conditions, emit,
        emit_style, ir, is_geometry_setter, is_interaction_when_key, is_text_bearing,
        looks_like_utility_string_product, lower_style, lower_style_source,
        lower_style_source_with_motion, parse, parse_style_const_rhs, prefixes, setters,
        style_complete_at, style_complete_context_at, style_complete_for_context,
        style_contains_geometry, style_uses_text_channel, style_uses_text_channel_env,
        surface_pack_for_host, text_channel_host_error, token_groups, when_keys, with_host_surface,
    };
}

#[cfg(test)]
mod tests;

pub use report::{Category, Finding, Severity};

/// Rule IDs that remain forever after the rule body was scrubbed.
/// HARDENING-1 WI6: IDs are never deleted — only superseded. SCRUB may
/// remove machinery; the id string stays pinned so drift tests catch
/// accidental loss of history.
pub const RETIRED_RULE_IDS: &[&str] = &[
    // Superseded by language-level layout keywords (`row`/`column`/`grid`/`center`).
    "sail-legacy-layout",
];

/// All rule IDs defined by this crate (live rules + retired forever-ids).
pub fn all_rule_ids() -> Vec<&'static str> {
    rules::CONTROLLER_RULES
        .iter()
        .chain(rules::SUI_TEMPLATE_RULES.iter())
        .chain(rules::PACK_SOURCE_RULES.iter())
        .chain(rules::APP_SOURCE_RULES.iter())
        .chain(rules::ACTION_RULES.iter())
        .map(|(id, _)| *id)
        .chain(RETIRED_RULE_IDS.iter().copied())
        .collect()
}

/// True iff `path` is a paired SAIL controller (`<stem>.controller.ts`).
///
/// The `.controller.ts` suffix is the SAIL convention for paired
/// controllers. Plain `.ts` files (helpers, types, tests) are never
/// linted as controllers regardless of co-location.
pub fn is_controller_path(path: &std::path::Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.ends_with(".controller.ts"))
        .unwrap_or(false)
}

/// True iff `path` is a SAIL template (`*.sail`).
pub fn is_sail_template_path(path: &std::path::Path) -> bool {
    path.extension().and_then(|e| e.to_str()) == Some("sail")
}

/// True iff `path` is a pack source `.ts` (a composition module a component ships).
///
/// The set is "any `.ts` file a component pack ships, not under the framework
/// runtime". Author action modules are a separate category
/// (`is_action_source_path`) with the ACTIONS-1 floor. Controllers stay on
/// `lint_controller_source`.
///
/// Detection walks the path's segments: a path under `packages/components/sail/**/src/`
/// (or the repo-local equivalent) is a pack source. The `*.controller.ts`
/// files are matched by `is_controller_path` first, so this never overlaps.
pub fn is_pack_source_ts(path: &std::path::Path) -> bool {
    if path.extension().and_then(|e| e.to_str()) != Some("ts") {
        return false;
    }
    // Controllers go through first in lint_path; the
    // caller that bypasses the dispatch and asks this directly won't see a
    // controller file as a pack source.
    if is_controller_path(path) {
        return false;
    }
    if is_action_source_path(path) {
        return false;
    }
    let p = path.to_string_lossy();
    let components_hit =
        p.contains("/packages/components/sail/") && (p.contains("/src/") || p.ends_with("/src"));
    if !components_hit {
        return false;
    }
    // Skip test files (they re-implement focus traps for assertion convenience
    // and the lint surface is irrelevant). The pack's own test directory
    // (often `test/` next to `src/`) is also out of scope.
    p.contains("/src/") || p.ends_with("/src")
}

/// True iff `path` is plain app TypeScript under `apps/*/src/**` or
/// `apps/*/routes/**` (SPEC-DOMDEC-1).
///
/// Controllers (`.controller.ts`) are matched first by `is_controller_path`
/// and never count as app sources. Test files (`*.test.ts`, `*.spec.ts`) are
/// out of scope — vitest harnesses create DOM freely.
///
/// This is the gate that closed the studio escape hatch: a side module under
/// `apps/studio/src/edit/region-overlay.ts` used to skip every style floor
/// because it was neither a controller nor a pack source.
pub fn is_app_source_ts(path: &std::path::Path) -> bool {
    if path.extension().and_then(|e| e.to_str()) != Some("ts") {
        return false;
    }
    if is_controller_path(path) {
        return false;
    }
    // Author actions get the stricter ACTION_RULES floor via
    // `is_action_source_path` — not the app-source ambient set.
    if is_action_source_path(path) {
        return false;
    }
    let p = path.to_string_lossy().replace('\\', "/");
    if p.contains(".test.ts")
        || p.contains(".spec.ts")
        || p.contains("/__tests__/")
        || p.contains("/test/")
        || p.contains("/qa/")
        || p.contains("/scripts/")
    {
        return false;
    }
    // Workspace-relative (`apps/studio/src/…`) or absolute (`…/apps/studio/src/…`).
    let under_apps = p.contains("/apps/") || p.starts_with("apps/");
    if !under_apps {
        return false;
    }
    p.contains("/src/") || p.contains("/routes/")
}

/// True iff `path` is an **author** `use:` Action module the regime floor lints.
///
/// Convention (fixed, no config knob): modules under `src/actions/**/*.ts`.
///
/// **Exempt (trusted):** framework catalog
/// `packages/sail/runtime/src/actions/**` — not subject to the author floor
/// this cycle (CSS-var bridges, document listeners, etc. by design).
///
/// **Exempt:** tests, benches, generated files.
pub fn is_action_source_path(path: &std::path::Path) -> bool {
    if path.extension().and_then(|e| e.to_str()) != Some("ts") {
        return false;
    }
    let p = path.to_string_lossy().replace('\\', "/");
    if p.contains("/tests/perf/")
        || p.contains(".test.ts")
        || p.contains(".spec.ts")
        || p.contains("/__tests__/")
        || p.contains("/test/")
        || p.contains(".gen.")
    {
        return false;
    }
    // Framework catalog is TRUSTED — never author-floor.
    if p.contains("/packages/sail/runtime/") {
        return false;
    }
    // Fixed convention: src/actions/**
    p.contains("/src/actions/") || p.starts_with("src/actions/")
}

/// Lint an **author** `use:` Action body from its source. Returns findings
/// sorted by `(line, col)`. No filesystem access.
///
/// Applies the ACTIONS-1 floor (ERROR): style/class/layout, state primitives,
/// network, host-only, signature.
pub fn lint_action_source(path: &std::path::Path, source: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (_id, check) in rules::ACTION_RULES {
        findings.extend(check(path, source));
    }
    findings.sort_by(|a, b| a.line.cmp(&b.line).then_with(|| a.col.cmp(&b.col)));
    findings
}

/// Lint a SAIL controller from its source. Returns findings sorted by
/// `(line, col)`.
///
/// Caller-supplied `path` is used in findings' `file` field; reads
/// happen elsewhere. No filesystem access.
pub fn lint_controller_source(path: &std::path::Path, source: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    let profile = std::env::var_os("SAIL_LINT_PROFILE").is_some();
    for (id, check) in rules::CONTROLLER_RULES {
        let t = std::time::Instant::now();
        findings.extend(check(path, source));
        if profile {
            eprintln!("sail-lint: {id} {:?}", t.elapsed());
        }
    }
    findings.sort_by(|a, b| a.line.cmp(&b.line).then_with(|| a.col.cmp(&b.col)));
    findings
}

/// Lint a pack source `.ts` file from its source. Returns findings
/// sorted by `(line, col)`. No filesystem access.
///
/// The composition modules a component ships — helpers, glue code, the
/// packed controllers in their on-disk form (when not inline — the SFC
/// path uses `lint_controller_source` instead). The framework layer
/// (`packages/sail/runtime/src/**`) is exempt; this dispatches only on
/// paths under `packages/components/sail/**/src/**`.
pub fn lint_pack_source_source(path: &std::path::Path, source: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (_id, check) in rules::PACK_SOURCE_RULES {
        findings.extend(check(path, source));
    }
    findings.sort_by(|a, b| a.line.cmp(&b.line).then_with(|| a.col.cmp(&b.col)));
    findings
}

/// Lint a plain app `.ts` module under `apps/*/src/**`. Returns findings
/// sorted by `(line, col)`. No filesystem access.
///
/// Runs the floor (`sail-no-imperative-style`) plus `sail-no-ambient-dom`
/// so a side module cannot reintroduce CSSOM / createElement / querySelector
/// host mutation by living outside `.controller.ts`.
pub fn lint_app_source_source(path: &std::path::Path, source: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (_id, check) in rules::APP_SOURCE_RULES {
        findings.extend(check(path, source));
    }
    findings.sort_by(|a, b| a.line.cmp(&b.line).then_with(|| a.col.cmp(&b.col)));
    findings
}

/// Lint a SAIL template (`*.sail`) from its source. Returns findings
/// sorted by `(line, col)`. No filesystem access.
pub fn lint_sail_template_source(path: &std::path::Path, source: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    let profile = std::env::var_os("SAIL_LINT_PROFILE").is_some();
    for (id, check) in rules::SUI_TEMPLATE_RULES {
        let t = std::time::Instant::now();
        findings.extend(check(path, source));
        if profile {
            eprintln!("sail-lint: {id} {:?}", t.elapsed());
        }
    }
    findings.sort_by(|a, b| a.line.cmp(&b.line).then_with(|| a.col.cmp(&b.col)));
    findings
}

/// Lint a single file path. Reads from disk, dispatches by extension.
/// Returns empty `Vec` for files outside SAIL's authoring scope.
pub fn lint_path(path: &std::path::Path) -> Vec<Finding> {
    if is_controller_path(path) {
        let Ok(source) = std::fs::read_to_string(path) else {
            return Vec::new();
        };
        return lint_controller_source(path, &source);
    }
    if is_sail_template_path(path) {
        let Ok(source) = std::fs::read_to_string(path) else {
            return Vec::new();
        };
        return lint_sail_template_source(path, &source);
    }
    if is_action_source_path(path) {
        let Ok(source) = std::fs::read_to_string(path) else {
            return Vec::new();
        };
        return lint_action_source(path, &source);
    }
    if is_pack_source_ts(path) {
        let Ok(source) = std::fs::read_to_string(path) else {
            return Vec::new();
        };
        return lint_pack_source_source(path, &source);
    }
    if is_app_source_ts(path) {
        let Ok(source) = std::fs::read_to_string(path) else {
            return Vec::new();
        };
        return lint_app_source_source(path, &source);
    }
    Vec::new()
}
