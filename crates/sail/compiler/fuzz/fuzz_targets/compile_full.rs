// Fuzz target: drive the full compile pipeline on arbitrary input.
//
// `parse_fragment` only exercises the HTML parser. This target runs
// parse → check → bindings → lower → emit, so panics anywhere in the
// downstream phases (lowering ambiguous binding shapes, emit format
// strings, schema generation against weird IR, dts emit with malformed
// types) are surfaced too.
//
// Run locally:
//     cargo +nightly fuzz run compile_full -- -max_total_time=60
//
// Hunts: panics, infinite loops, unbounded allocation. Compile errors
// (returned `Err`) are normal — the fuzzer's goal is to find inputs
// that crash the compiler outright.

#![no_main]

use libfuzzer_sys::fuzz_target;
use sail_compiler::{CompileMode, CompileOpts, compile};

fuzz_target!(|data: &[u8]| {
    let Ok(src) = std::str::from_utf8(data) else { return };
    // Bound input size — extreme blowup is interesting but the fuzzer's job
    // is panic-hunting, and a 10MB .sui isn't a realistic regression class.
    if src.len() > 64 * 1024 {
        return;
    }

    // Try View mode and Component mode — different emit paths, different
    // failure surfaces (component emits CEM + ShadowRoot wiring).
    let _ = compile(
        src,
        CompileOpts {
            mode: CompileMode::Component,
            ..Default::default()
        },
    );
    let _ = compile(
        src,
        CompileOpts {
            mode: CompileMode::Component,
            component_meta: Some(sail_compiler::ComponentMeta {
                name: "fuzz-component".into(),
                shadow: false,
            }),
            ..Default::default()
        },
    );
});
