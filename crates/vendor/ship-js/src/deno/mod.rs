//! The deno_core / V8 backend — the agent JS engine (S16 Phase C1).
//!
//! The entire subtree is reachable only with the `deno` feature, so the default
//! ship-js build (and any non-engine consumer) never pays the V8 cost. The public
//! surface: [`run_script`]/[`run_module`] execute a program in a locked-down
//! isolate whose ONLY effect is the [`crate::Dispatch`] wire; [`ScriptExecutor`]
//! drives `!Send` isolates from a `Send` caller; [`ShipModuleLoader`] is the
//! deny-by-default import world.
//!
//! Every run is **bounded**: a [`RunLimits`] (default 5 s wall-clock + 256 MiB
//! heap) terminates the isolate on overrun instead of wedging the process.
//! The bounded variants are [`run_script_with_limits`] and
//! [`run_module_with_limits`]; the un-suffixed entry points are bounded
//! wrappers for back-compat.

mod module_loader;
mod runtime;
mod script_executor;

pub use module_loader::{MAIN, SDK, ShipModuleLoader};
pub use runtime::{
    RunError, RunLimits, TerminateReason, run_module, run_module_with_limits, run_script,
    run_script_with_limits,
};
pub use script_executor::ScriptExecutor;
