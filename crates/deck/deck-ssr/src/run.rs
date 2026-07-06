//! Run the bundle in ship-js and collect the first-paint HTML.
//!
//! A `JsRuntime` is `!Send`, so the run happens on a dedicated current-thread
//! VM thread; the caller blocks on it. The production view-host would instead
//! pool `ship_js::deno::ScriptExecutor` threads (one isolate live per thread),
//! awaiting their `Send` reply futures — a fresh isolate per request, pooled
//! across VM threads. This single-shot path is the unit the pool repeats.
//!
//! **Bounded:** every run carries a [`RunLimits`] (wall-clock + heap cap). A
//! runaway controller — infinite loop, stuck host op, unbounded allocation —
//! is **terminated** and `render` returns an `Err`, so the SSR gate
//! (`deck_core::synthetic_ssr`) serves an empty `#sui-root` for the client to
//! mount (CSR; the error is logged loudly) rather than 500ing the request or
//! wedging the daemon. The bound is configurable
//! per-call (`render_with_limits`) or host-wide via `SHIP_JS_DEADLINE_MS` /
//! `SHIP_JS_MAX_HEAP_MB` env vars; defaults are 5 s wall-clock + 256 MiB heap.
//!
//! The `deadline` failure surface:
//! `RunError::Terminated { reason: WallClockDeadline | NearHeapLimit, .. }` →
//! `anyhow!("module terminated by wall-clock deadline after ...")`. Callers
//! (`ShipJsSsr::first_paint`) catch the `Err` and fall back; this is the
//! exact contract [`tests::reliability::slice1_reliability_sweep`] assumes
//! ("a failure records the precise gap, never a silent exclusion") — the
//! bound's `Err` is treated as one such failure.

use std::sync::Arc;

use anyhow::{anyhow, Result};
use serde_json::Value;
use ship_js::deno::{run_module_with_limits, RunLimits};
use ship_js::dispatch::{CancellationToken, SesPolicy};
use ship_js::Dispatch;

use crate::host::SsrDispatch;
use crate::{RenderOut, SHIP_SDK};

/// Render `bundle` once through `run_module` with `dispatch`, returning the
/// first-paint HTML + the host-dispatch call path. The run is bounded by
/// `RunLimits::from_env_or(default)` — env vars `SHIP_JS_DEADLINE_MS` /
/// `SHIP_JS_MAX_HEAP_MB` override the defaults (5 s / 256 MiB).
pub fn render(bundle: &str, dispatch: Arc<SsrDispatch>) -> Result<RenderOut> {
    render_with_limits(bundle, dispatch, RunLimits::default())
}

/// Render `bundle` once with an explicit `limits` bound. Use this when a host
/// wants to tune the wall-clock / heap cap per request (e.g. a richer view
/// getting more time) rather than relying on the env defaults.
pub fn render_with_limits(
    bundle: &str,
    dispatch: Arc<SsrDispatch>,
    limits: RunLimits,
) -> Result<RenderOut> {
    let limits = limits.from_env_or();
    let bundle_bytes = bundle.len();
    let emits = on_vm_thread(
        bundle.to_string(),
        dispatch.clone(),
        move |b, d| async move {
            let host: Arc<dyn Dispatch> = d;
            // ship-exec grew per-run gates (allow-lists, expression policy,
            // compartments, module cache, cancellation, SES). deck-ssr wants
            // the "no gate" semantics ship-js's own `run_module` wrapper uses:
            // no lists/policy/compartment/cache, a fresh (un-cancelled) token,
            // SES off — behavior-identical to before the signature grew.
            run_module_with_limits(
                host,
                None, // op_allow_list
                None, // module_allow_list
                None, // expression_policy
                None, // compartment_id
                None, // module_cache_key
                CancellationToken::new(),
                SesPolicy::Off,
                SHIP_SDK.to_string(),
                b,
                limits,
            )
            .await
            .map_err(|e| anyhow!("run_module: {e:?}"))
        },
    )?;
    let first = emits.first().ok_or_else(|| anyhow!("ssr entry emitted nothing"))?;
    Ok(RenderOut {
        html: first["html"].as_str().unwrap_or_default().to_string(),
        route_params: first["routeParams"].clone(),
        call_path: dispatch.call_path(),
        bundle_bytes,
    })
}

/// Run an async job on a dedicated current-thread VM thread, blocking until done.
fn on_vm_thread<F, Fut>(bundle: String, dispatch: Arc<SsrDispatch>, job: F) -> Result<Vec<Value>>
where
    F: FnOnce(String, Arc<SsrDispatch>) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = Result<Vec<Value>>>,
{
    std::thread::Builder::new()
        .name("deck-ssr-vm".into())
        .spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("current-thread runtime for the SSR VM");
            rt.block_on(job(bundle, dispatch))
        })
        .expect("spawn SSR VM thread")
        .join()
        .map_err(|_| anyhow!("SSR VM thread panicked"))?
}


