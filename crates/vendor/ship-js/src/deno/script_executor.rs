//! The `!Send` bridge — drive `!Send` V8 isolates from a `Send` world.
//!
//! deno_core's `JsRuntime` is `!Send`, but most async hosts (e.g. a rig agent
//! loop, an axum handler) need a `Send` future. So the JsRuntime never crosses a
//! thread boundary: it lives on a DEDICATED current-thread executor thread. A
//! caller sends a job over a channel and awaits a `Send` result. The returned
//! future holds only channel ends (Send-clean); all V8 work stays on the one VM
//! thread.
//! Concurrency model: jobs run **one at a time** on the VM thread — each
//! `run_module` is awaited to completion before the next job starts. This is
//! deliberate: a V8 isolate is thread-bound AND not safely re-entrant across
//! `.await` points, so two isolates must not interleave on one thread. Many
//! agents per process is therefore a *pool* of these threads (one isolate live
//! per thread at a time), not many isolates multiplexed on one thread. The pool
//! is a Phase-D concern; this is one sequential VM thread.
// DEFERRED SEAM — keep single-threaded (Phase-D pool: this executor is one
// current-thread VM; many concurrent agents is a *pool* of these, not
// multiple isolates multiplexed on one thread. Do not change the
// single-threaded shape without landing the pool.)
//!
//! Auth-agnostic: each job carries its own [`Dispatch`], which is where the
//! host has captured whatever credential/gate it wants. The executor itself sees
//! only opaque dispatch + source strings.

use std::sync::Arc;

use serde_json::Value;
use tokio::sync::{mpsc, oneshot};

use crate::dispatch::Dispatch;

struct ScriptJob {
    dispatch: Arc<dyn Dispatch>,
    sdk_src: String,
    main_src: String,
    reply: oneshot::Sender<anyhow::Result<Vec<Value>>>,
}

/// Handle to the VM thread. Cloneable-cheap (just a channel sender). Dropping the
/// last handle ends the recv loop and the thread exits cleanly.
#[derive(Clone)]
pub struct ScriptExecutor {
    tx: mpsc::UnboundedSender<ScriptJob>,
}

impl ScriptExecutor {
    /// Start the dedicated VM thread. The `!Send` JsRuntime created per job stays
    /// on this thread — never sent anywhere. Jobs are processed sequentially:
    /// each `run_module` is awaited fully before the next, so only one isolate is
    /// ever live on the thread.
    pub fn spawn() -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel::<ScriptJob>();
        std::thread::Builder::new()
            .name("ship-script-vm".into())
            .spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("current-thread runtime for the script VM");
                // A current-thread runtime drives the `!Send` run_module future
                // inline. No LocalSet/spawn_local: jobs run one at a time, so two
                // isolates never interleave on the thread (V8 aborts otherwise).
                rt.block_on(async move {
                    while let Some(job) = rx.recv().await {
                        let res =
                            super::runtime::run_module(job.dispatch, job.sdk_src, job.main_src)
                                .await;
                        let _ = job.reply.send(res);
                    }
                });
            })
            .expect("spawn script VM thread");
        Self { tx }
    }

    /// Run a module on the VM thread. The returned future is `Send` (it only
    /// awaits a oneshot), so it composes inside any async host. `dispatch` is the
    /// host's effect wire for this run; `sdk_src`/`main_src` are the module
    /// sources the deny-by-default loader serves.
    pub async fn run(
        &self,
        dispatch: Arc<dyn Dispatch>,
        sdk_src: String,
        main_src: String,
    ) -> anyhow::Result<Vec<Value>> {
        let (reply, rx) = oneshot::channel();
        self.tx
            .send(ScriptJob {
                dispatch,
                sdk_src,
                main_src,
                reply,
            })
            .map_err(|_| anyhow::anyhow!("script executor stopped"))?;
        rx.await
            .map_err(|_| anyhow::anyhow!("script executor dropped the reply"))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dispatch::DispatchError;
    use serde_json::json;

    struct Echo;
    #[async_trait::async_trait]
    impl Dispatch for Echo {
        async fn call(&self, op: &str, args: Value) -> Result<Value, DispatchError> {
            Ok(json!({ "op": op, "args": args }))
        }
    }

    const SDK: &str =
        r#"export const call = (op, args) => Deno.core.ops.op_ship_call(op, args ?? {});"#;

    // The bridge runs a `!Send` isolate on the VM thread and returns a Send
    // result to a multi-thread caller — and two jobs multiplex on the one thread.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn runs_modules_on_the_vm_thread_from_a_send_caller() {
        let exec = ScriptExecutor::spawn();
        let main = r#"
            import { call } from "ship";
            Deno.core.ops.op_emit(await call("a.op", { n: 1 }));
        "#;
        let a = exec.run(Arc::new(Echo), SDK.into(), main.into());
        let b = exec.run(Arc::new(Echo), SDK.into(), main.into());
        let (ra, rb) = tokio::join!(a, b);
        for r in [ra, rb] {
            let out = r.expect("job runs");
            assert_eq!(out.len(), 1);
            assert_eq!(out[0]["op"], "a.op");
            assert_eq!(out[0]["args"]["n"], 1);
        }
    }
}
