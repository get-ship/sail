// WASM-bindgen surface — BROWSER ONLY (docs playground + LSP browser worker).
// The vitest test harness does NOT use this; it loads the native compiler as a
// Node-API `.node` addon (crates/sail/compiler-napi) so one `cargo build` feeds
// both the daemon and the harness, with no second artifact to go stale (#255).
// Keep this surface + `just sail-wasm` + @ship/sail-compiler-wasm for the only
// legit browser consumer: apps/site / docs (and the browser LSP worker).
//
// WASM-bindgen surface for the docs-site playground.
//
// Exposes a single `compile(source, optsJSON) -> resultJSON` function
// callable from JS — the playground passes the editor's current .sail
// text + an opts JSON (mode, component name, …); the
// returned result holds the emitted JS module plus any compile error
// in a discriminated shape JS can branch on without try/catch.
//
// Kept deliberately thin: JSON in / JSON out. No bindgen-shaped
// structs leaking the native enums / paths — those are awkward to
// keep in sync between Rust and the playground's TS, and the JS
// caller only cares about source text + a small bag of options.

#![cfg(target_arch = "wasm32")]

use crate::{CompileMode, CompileOpts, ComponentMeta, compile as native_compile};
use crate::{block_manifest, directive_manifest};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WasmOpts {
    /// "view" | "component" | "auto" (default).
    mode: Option<String>,
    /// Component name. When set, forces component mode AND seeds
    /// `ComponentMeta { name }`. The playground passes this
    /// whenever the user's source is a component.
    component_name: Option<String>,
    /// Shadow flag for component mode. Defaults to false to match the
    /// standalone-example shape — playground rendering uses light DOM
    /// so the page's stylesheet can target the rendered tree.
    #[serde(default)]
    /// Setup URL relative to the emitted JS. When `None`, no
    /// `import setup …` is emitted (template-only modules).
    setup_url: Option<String>,
    /// Host grant: route params when the host's routing convention names
    /// this file a route. `None`/absent = not a route.
    #[serde(default)]
    route_params: Option<Vec<String>>,
    /// Optional CSS to prepend as a `<style>` block — same semantics
    /// as the native CLI's `--styles` flag.
    styles: Option<String>,
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
enum WasmResult {
    Ok {
        js: String,
        css: Option<String>,
        dts: Option<String>,
    },
    Err {
        message: String,
        line: Option<usize>,
        col: Option<usize>,
        category: Option<String>,
    },
}

#[wasm_bindgen]
pub fn compile(source: &str, opts_json: &str) -> Result<JsValue, JsValue> {
    console_error_panic_hook::set_once();
    // Best-effort: empty opts is valid (use all defaults). A JSON
    // parse error surfaces as a thrown JS Error rather than a result
    // — the playground would treat this as a wiring bug rather than
    // user-typed source rejection.
    let parsed: WasmOpts = if opts_json.trim().is_empty() {
        WasmOpts::default()
    } else {
        serde_json::from_str(opts_json)
            .map_err(|e| JsValue::from_str(&format!("opts JSON parse: {e}")))?
    };

    let mode = match parsed.mode.as_deref() {
        Some("auto") | None => CompileMode::Auto,
        Some("component") => CompileMode::Component,
        Some(other) => {
            return Err(JsValue::from_str(&format!(
                "unknown mode {other:?} — expected component / auto"
            )));
        }
    };

    let component_meta = parsed
        .component_name
        .as_ref()
        .map(|name| ComponentMeta { name: name.clone() });

    let opts = CompileOpts {
        route_params: parsed.route_params.clone(),
        setup_url: parsed.setup_url,
        mode,
        source_path: None,
            element_prefix: "sail".into(),
            host_data_attr: "data-sail-host".into(),
        // The optional styles-injection flag was removed from
        // `CompileOpts` (the CLI no longer accepts `--styles`); the
        // wasm surface was orphaned by that refactor and the
        // playground never sent it. Drop the field reference; the
        // `WasmOpts.styles` deserialization key stays (forward-
        // compatible — ignored on this side).
        component_meta,
        emit_dts: false,
        scope_capabilities: Vec::new(),
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    };

    let result = match native_compile(source, opts) {
        Ok(out) => WasmResult::Ok {
            js: out.js,
            css: out.css,
            dts: out.dts,
        },
        Err(err) => WasmResult::Err {
            message: err.message.clone(),
            line: err.span.map(|s| line_for_offset(source, s.start)),
            col: err.span.map(|s| col_for_offset(source, s.start)),
            category: err.category.clone(),
        },
    };

    serde_wasm_bindgen::to_value(&result)
        .map_err(|e| JsValue::from_str(&format!("serialize result: {e}")))
}

/// Browser analysis surface for the docs-playground LSP worker: the full
/// multi-diagnostic stream + scope symbols + tag/event dependencies from one
/// `.sail` source, off the same Rust grammar the compiler + native LSP use.
/// `source_path` is `None` (the browser has no paired `.controller.ts` on
/// disk), so controller-aware passes are skipped — template parse/semantic
/// checks + lint still run. Mirrors `sail_compiler::analyze`.
#[wasm_bindgen]
pub fn analyze(source: &str) -> Result<JsValue, JsValue> {
    console_error_panic_hook::set_once();
    let analysis = crate::analyze(source, None, &[]);
    serde_wasm_bindgen::to_value(&analysis)
        .map_err(|e| JsValue::from_str(&format!("serialize analysis: {e}")))
}

/// Browser completion surface: every name in scope for a `.sail` template,
/// with binding kind and origin. Mirrors `sail_compiler::completions_at` as
/// called by the native LSP; the browser has no source path, so pass `None`.
#[wasm_bindgen]
pub fn completions_at(source: &str) -> Result<JsValue, JsValue> {
    let items = crate::completions_at(source, None);
    serde_wasm_bindgen::to_value(&items)
        .map_err(|e| JsValue::from_str(&format!("serialize completions: {e}")))
}

/// Browser hover surface: resolve the symbol under a 1-based `(line, col)`
/// cursor. Mirrors `sail_compiler::hover_at` as called by the native LSP;
/// returns `undefined` when the cursor is not over a known symbol.
#[wasm_bindgen]
pub fn hover_at(source: &str, line: usize, col: usize) -> Result<JsValue, JsValue> {
    let h = crate::hover_at(source, None, line, col);
    serde_wasm_bindgen::to_value(&h)
        .map_err(|e| JsValue::from_str(&format!("serialize hover: {e}")))
}

/// Closed vocabulary the browser LSP needs for trigger-based completions and
/// hover docs: event names, modifiers, bind targets, blocks, directives, and
/// foreign idioms. All tables are owned by the compiler; this export replaces
/// every hand-maintained TypeScript copy in the browser LSP.
#[derive(Serialize)]
struct CompletionVocabulary {
    events: Vec<&'static str>,
    modifiers: &'static [directive_manifest::ModifierEntry],
    bind_targets: &'static [directive_manifest::BindTargetEntry],
    blocks: &'static [block_manifest::BlockEntry],
    directives: &'static [directive_manifest::DirectiveEntry],
    foreign_idioms: &'static [directive_manifest::ForeignIdiomEntry],
    motion_value_forms: &'static [directive_manifest::MotionValueForm],
}

#[wasm_bindgen]
pub fn completion_vocabulary() -> Result<JsValue, JsValue> {
    let vocab = CompletionVocabulary {
        events: crate::directive_manifest::EVENT_NAMES
            .iter()
            .map(|e| e.name)
            .collect(),
        modifiers: directive_manifest::EVENT_MODIFIERS,
        bind_targets: directive_manifest::BIND_TARGETS,
        blocks: block_manifest::BLOCKS,
        directives: directive_manifest::DIRECTIVES,
        foreign_idioms: directive_manifest::FOREIGN_IDIOMS,
        motion_value_forms: directive_manifest::MOTION_VALUE_FORMS,
    };
    serde_wasm_bindgen::to_value(&vocab)
        .map_err(|e| JsValue::from_str(&format!("serialize vocabulary: {e}")))
}

/// Convert a byte offset into a 1-based line number. Mirrors the
/// formatting the CLI uses for error messages so the playground can
/// surface the same coordinates.
fn line_for_offset(source: &str, offset: usize) -> usize {
    let clamped = offset.min(source.len());
    source[..clamped].matches('\n').count() + 1
}

/// 1-based column for `offset` (chars since the last newline + 1).
fn col_for_offset(source: &str, offset: usize) -> usize {
    let clamped = offset.min(source.len());
    let last_nl = source[..clamped].rfind('\n').map(|i| i + 1).unwrap_or(0);
    source[last_nl..clamped].chars().count() + 1
}
