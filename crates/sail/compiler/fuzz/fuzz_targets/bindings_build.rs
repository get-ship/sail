// Fuzz target: parse → bindings::build pipeline.
//
// The bindings module walks the parsed IR and computes the per-attribute
// binding metadata (key, kind, deps) the runtime hydrates against. Bugs
// here often manifest as emit-time inconsistencies that the LSP and
// `sail_view.meta.json` consumers also depend on. Fuzzing this layer
// catches IR shapes that the binding builder hasn't accounted for
// without paying the cost of the full emit pipeline.
//
// Run locally:
//     cargo +nightly fuzz run bindings_build -- -max_total_time=60

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(src) = std::str::from_utf8(data) else { return };
    if src.len() > 64 * 1024 {
        return;
    }
    let Ok(program) = sail_compiler::parse::parse(src) else { return };
    let _ = sail_compiler::bindings::build(&program);
});
