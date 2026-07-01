//! Node-API surface for the SUI test harness.
//!
//! Mirrors `crates/sui/compiler/src/wasm.rs` field-for-field — same
//! `WasmOpts -> CompileOpts` mapping (`emit_dts:false`, `emit_ssr:false`,
//! `source_path:None`; `setupUrl` / `componentName` / `mode` / `styles`
//! passthrough) and the same discriminated result shape
//! (`{kind:'ok',js,css,dts,ssrHtml}` | `{kind:'err',message,line,col,
//! category}`). The wasm surface stays the browser-only path (docs
//! playground + LSP worker); this is the Node path the vitest harness loads
//! once per worker.
//!
//! Why a separate crate, not a `napi` feature on `sui-compiler`: the
//! compiler crate must keep building for wasm + the CLI without dragging in
//! the node-API host. This is a thin dependent, exactly like `wasm.rs` is a
//! feature-gated surface.
//!
//! Return contract: a JSON **string**. The harness does one `JSON.parse`.
//! Keeping the boundary string-in/string-out (rather than reaching for
//! napi's serde object mapping) keeps this binding identical in spirit to
//! the wasm `compile(source, optsJson) -> resultJSON` shape and avoids a
//! second serialization contract to keep in sync.

use napi_derive::napi;
use serde::{Deserialize, Serialize};
use sui_compiler::{compile as native_compile, CompileMode, CompileOpts, ComponentMeta};

/// JS-facing opts — identical field set + camelCase mapping to
/// `wasm.rs::WasmOpts`. The harness passes
/// `JSON.stringify({ mode, componentName, setupUrl })`; `shadow` / `styles`
/// default and round-trip the same way the playground's do.
#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NapiOpts {
    /// "view" | "component" | "auto" (default).
    mode: Option<String>,
    /// Component name. When set, forces component mode AND seeds
    /// `ComponentMeta { name, shadow }`.
    component_name: Option<String>,
    /// Shadow flag for component mode. Defaults to false — light DOM, same
    /// default as the playground.
    #[serde(default)]
    shadow: bool,
    /// Setup URL relative to the emitted JS. `None` => no `import setup …`.
    setup_url: Option<String>,
    /// Optional CSS prepended as a `<style>` block (component sources only).
    styles: Option<String>,
    /// On-disk path of the `.sui` source. When set, locates the paired
    /// `<stem>.controller.ts` for the four controller-aware check passes
    /// (`scope_bound`, `scope_member`, `bind_this_target`, `on_callable`)
    /// gated on `source_path.is_some()` in `check::run`. The harness passes
    /// the real path for tests that `readFileSync` a checked-in `.sui` so the
    /// napi path rejects exactly what the daemon + CLI do (#263-A). `None` for
    /// synthetic in-memory sources (no controller to pair with).
    source_path: Option<String>,
}

/// Discriminated result — byte-for-byte the same JSON the wasm surface
/// emits (`serde(tag = "kind")`, camelCase fields), so the harness's
/// `{ kind: 'ok' | 'err', ... }` branch is unchanged.
#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
enum NapiResult {
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

/// Compile `source` with `opts_json` and return the result as a JSON string.
///
/// A malformed `opts_json` throws (a wiring bug, not user-source rejection) —
/// mirrors `wasm.rs`, which returns `Err(JsValue)` for the same case.
///
/// Thin shim: the napi host symbols (`napi_*`) only resolve when the cdylib is
/// dlopen'd by Node. The actual mapping lives in [`compile_to_result`], which
/// returns a plain `Result<_, String>` so it links and unit-tests under bare
/// `cargo test` (no Node host in that loop).
#[napi]
pub fn compile(source: String, opts_json: String) -> napi::Result<String> {
    let result = compile_to_result(&source, &opts_json).map_err(napi::Error::from_reason)?;
    serde_json::to_string(&result)
        .map_err(|e| napi::Error::from_reason(format!("serialize result: {e}")))
}

/// Pure core, no napi types — keeps the mapping unit-testable under plain
/// `cargo test` (the `.node` host is not in that loop). `Err(String)` is the
/// thrown-error reason; the `#[napi]` shim wraps it in `napi::Error`.
fn compile_to_result(source: &str, opts_json: &str) -> Result<NapiResult, String> {
    let parsed: NapiOpts = if opts_json.trim().is_empty() {
        NapiOpts::default()
    } else {
        serde_json::from_str(opts_json).map_err(|e| format!("opts JSON parse: {e}"))?
    };

    let mode = match parsed.mode.as_deref() {
        Some("component") => CompileMode::Component,
        Some("auto") | None => CompileMode::Auto,
        Some(other) => {
            return Err(format!(
                "unknown mode {other:?} — expected component / auto"
            ))
        }
    };

    let component_meta = parsed.component_name.as_ref().map(|name| ComponentMeta {
        name: name.clone(),
        shadow: parsed.shadow,
    });

    let opts = CompileOpts {
        setup_url: parsed.setup_url,
        mode,
        source_path: parsed.source_path.map(std::path::PathBuf::from),
        styles: parsed.styles,
        component_meta,
        emit_dts: false,
        emit_component_imports: false,
    };

    Ok(match native_compile(source, opts) {
        Ok(out) => NapiResult::Ok {
            js: out.js,
            css: out.css,
            dts: out.dts,
        },
        Err(err) => NapiResult::Err {
            message: err.message.clone(),
            line: err.span.map(|s| line_for_offset(source, s.start)),
            col: err.span.map(|s| col_for_offset(source, s.start)),
            category: err.category.clone(),
        },
    })
}

/// 1-based line number for `offset`. Mirrors `wasm.rs::line_for_offset`.
fn line_for_offset(source: &str, offset: usize) -> usize {
    let clamped = offset.min(source.len());
    source[..clamped].matches('\n').count() + 1
}

/// 1-based column for `offset`. Mirrors `wasm.rs::col_for_offset`.
fn col_for_offset(source: &str, offset: usize) -> usize {
    let clamped = offset.min(source.len());
    let last_nl = source[..clamped].rfind('\n').map(|i| i + 1).unwrap_or(0);
    source[last_nl..clamped].chars().count() + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiles_a_view_to_ok() {
        let json = compile_to_result("<div>hi</div>", "{}").unwrap();
        let v = serde_json::to_value(&json).unwrap();
        assert_eq!(v["kind"], "ok");
        assert!(v["js"].as_str().unwrap().contains("export default"));
    }

    #[test]
    fn empty_opts_is_valid() {
        // matches wasm.rs: empty opts → all defaults, no throw.
        compile_to_result("<div>x</div>", "   ").unwrap();
    }

    #[test]
    fn unknown_mode_is_a_wiring_error() {
        let e = compile_to_result("<div/>", r#"{"mode":"bogus"}"#).unwrap_err();
        assert!(e.contains("unknown mode"), "{e}");
    }

    #[test]
    fn component_name_forces_component_mode() {
        let json = compile_to_result(
            "<prop name=\"x\"/>\n<div>hi</div>",
            r#"{"componentName":"sui-x"}"#,
        )
        .unwrap();
        let v = serde_json::to_value(&json).unwrap();
        assert_eq!(v["kind"], "ok");
        // component mode emits a customElements registration.
        assert!(v["js"].as_str().unwrap().contains("sui-x"));
    }

    #[test]
    fn compile_error_carries_line_col() {
        // An unterminated tag in the body is a parse error — line/col
        // are carried through the napi boundary into the Err result.
        // (Previously this test used a CSS-token typo as the Error
        // source, but unknown `--name` is now a Warn — the build
        // accepts the var() and the lint still surfaces a close-match
        // suggestion. Parse errors remain the canonical Error
        // source for the napi error-path contract.)
        let json = compile_to_result(
            "<prop name=\"x\"/>\n<style>.a{ color: var(--nope-not-a-token); }</style>\n<div class=\"a\">x</div",
            r#"{"componentName":"sui-x"}"#,
        )
        .unwrap();
        let v = serde_json::to_value(&json).unwrap();
        assert_eq!(v["kind"], "err");
        assert_eq!(v["line"], 3);
    }

    #[test]
    fn lint_error_rule_gates_compile_through_library_path() {
        // #255: a `sui-prop-value` Error rule lives ONLY in the full lint
        // stream, not in `check::run`. Before the gate it passed silently
        // through library `compile()` — which the daemon + this napi harness
        // both call — while the CLI rejected it (dishonest exit codes / silent
        // daemon accept). It must now reject here too: one contract per
        // boundary.
        let json =
            compile_to_result(r#"<sui-button variant="primary">x</sui-button>"#, "{}").unwrap();
        let v = serde_json::to_value(&json).unwrap();
        assert_eq!(v["kind"], "err", "{v}");
        assert_eq!(v["category"], "sui-prop-value");
    }
}
