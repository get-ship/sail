//! Regression tests for inputs found by `cargo +nightly fuzz` runs.
//! These ensure previously-discovered crash inputs do not regress.

use sui_compiler::{CompileMode, CompileOpts, compile};

/// Issue: `){&&deg;\x129\n}` crashed the compiler.
/// Found by `compile_full` fuzz on 2026-05-11.
#[test]
fn fuzz_2026_05_11_amp_entity_in_interpolation() {
    let src_bytes = [41u8, 123, 38, 38, 100, 101, 103, 59, 18, 57, 10, 125];
    let src = std::str::from_utf8(&src_bytes).expect("utf8");
    // Compile must return Err(CompileError) gracefully, not panic.
    let _ = compile(
        src,
        CompileOpts {
            mode: CompileMode::Component,
            ..Default::default()
        },
    );
}

/// Issue: deeply-nested template literals starting with `\x01\x00\x00X{`${.{` ...
/// crash compile_full. New panic site after the previous fixes.
#[test]
fn fuzz_2026_05_12_deep_template_nesting() {
    let src_bytes = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("fuzz/artifacts/compile_full/crash-73f190831da830fed29a521bd5f764c8bbf6987b"),
    );
    let Ok(bytes) = src_bytes else { return };
    let Ok(src) = std::str::from_utf8(&bytes) else { return };
    let _ = compile(
        src,
        CompileOpts {
            mode: CompileMode::Component,
            ..Default::default()
        },
    );
}

/// Issue: `>M{#foro of r.&harr;{-}{{}}}}(***r` crashed bindings::build.
/// Found by `bindings_build` fuzz on 2026-05-12.
#[test]
fn fuzz_2026_05_12_for_with_harr_entity() {
    let src_bytes = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("fuzz/artifacts/bindings_build/crash-72cc65fa184179f2c7ddfc24f3e0cc33ade86975"),
    );
    let Ok(bytes) = src_bytes else { return };
    let Ok(src) = std::str::from_utf8(&bytes) else { return };
    // Test the bindings path specifically (not the full compile).
    let Ok(program) = sui_compiler::parse::parse(src) else { return };
    let _ = sui_compiler::bindings::build(&program);
}

/// Issue: nested template literals with `&sect;` entity crashed
/// compile_full. Different panic site after the lex + expr_subset
/// fixes above.
#[test]
fn fuzz_2026_05_12_nested_template_sect_entity() {
    let src_bytes = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("fuzz/artifacts/compile_full/crash-f98d000f2fe0cb4a4d345b8bec36594d32173eaf"),
    );
    let Ok(bytes) = src_bytes else { return };
    let Ok(src) = std::str::from_utf8(&bytes) else { return };
    let _ = compile(
        src,
        CompileOpts {
            mode: CompileMode::Component,
            ..Default::default()
        },
    );
}

/// Issue: a template-literal payload with ~100 stacked `$` chars
/// crashed compile_full. Reproducer below; previously panicked on the
/// stack (recursive template-expr parsing) or other.
#[test]
fn fuzz_2026_05_12_dollar_stack_in_template() {
    let src_bytes = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("fuzz/artifacts/compile_full/crash-d2d6111bd0e376b61c0e1ba6598fb60ec422f55c"),
    );
    let Ok(bytes) = src_bytes else { return };
    let Ok(src) = std::str::from_utf8(&bytes) else { return };
    let _ = compile(
        src,
        CompileOpts {
            mode: CompileMode::Component,
            ..Default::default()
        },
    );
}

/// Issue: `){&.&}{`K…&Prime;…function…\{`…&radr&}` crashed compile_full.
/// Found by `compile_full` fuzz on 2026-05-12, after the byte-slice
/// fix in 862f9b94. Independent code path — guards against a different
/// panic class on weird HTML-entity sequences mixed with template
/// literals.
#[test]
fn fuzz_2026_05_12_amp_prime_entity_in_template_literal() {
    let src_bytes = [
        0x29, 0x7b, 0x26, 0x2e, 0x02, 0x26, 0x7d, 0x7b, 0x60, 0x4b, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x5c, 0x5c, 0x5c, 0x5c, 0x5c, 0x5c,
        0x5c, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x5c, 0x00, 0x00,
        0x00, 0x5c, 0x5c, 0x5c, 0x00, 0x5c, 0x5c, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x48, 0x5c, 0x5e, 0x5c, 0x5c, 0x5c, 0x26, 0x50, 0x72, 0x69,
        0x6d, 0x65, 0x3b, 0x5d, 0x5d, 0x5d, 0x5d, 0x5d, 0x5d, 0x5d, 0x5d, 0x5d,
        0x5d, 0x5d, 0x5d, 0x5d, 0x5d, 0x66, 0x75, 0x6e, 0x63, 0x74, 0x69, 0x6f,
        0x6e, 0x5d, 0x5d, 0x5d, 0x5d, 0x5d, 0x5d, 0x5d, 0x5d, 0x5d, 0x5d, 0x5d,
        0x5d, 0x5d, 0x5d, 0x5d, 0x5d, 0x5d, 0x5d, 0x5d, 0x5d, 0x5d, 0x5d, 0x5d,
        0x5d, 0x5d, 0x5d, 0x5d, 0x5d, 0x5d, 0x5d, 0x5d, 0x5d, 0x5d, 0x5d, 0x5d,
        0x5d, 0x5d, 0x5d, 0x5d, 0x5d, 0x5c, 0x7b, 0x60, 0x5c, 0x5c, 0x25, 0x72,
        0x61, 0x23, 0x26, 0x64, 0x72, 0x0a, 0x72, 0x26, 0x7d,
    ];
    let Ok(src) = std::str::from_utf8(&src_bytes) else { return };
    let _ = compile(
        src,
        CompileOpts {
            mode: CompileMode::Component,
            ..Default::default()
        },
    );
}
