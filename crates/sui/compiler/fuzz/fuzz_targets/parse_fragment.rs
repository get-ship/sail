// Fuzz target: drive the .sui parser on arbitrary input bytes.
//
// The bug class this exists to catch was demonstrated by an infinite loop
// in `parse/html/parser.rs::parse_children`: a stray `<` followed by a
// non-tag character (e.g. the JS less-than operator inside `{#if x < y}`)
// failed to advance the cursor and spun forever, exhausting heap until
// the OS killed the process. Any AFL-style fuzzer would have surfaced
// that within seconds. This harness makes such regressions automatic.
//
// Run locally:
//     cargo +nightly fuzz run parse_fragment -- -max_total_time=60
//
// Seed corpus: `corpus/parse_fragment/` (start with copies of every .sui
// in packages/ui-components/src so the fuzzer has real-world inputs to
// mutate from).

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Reject non-UTF8 inputs cheaply — the public `parse` entry point
    // takes &str. The fuzzer still has plenty of structure to mutate.
    if let Ok(src) = std::str::from_utf8(data) {
        // Errors are fine — we're hunting panics, infinite loops, and
        // unbounded allocation. A returned `Err` is a clean death.
        let _ = sui_compiler::parse::parse(src);
    }
});
