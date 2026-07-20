//! Anti-drift check: fail the build if `grammar.js`'s template-block
//! vocabulary diverges from the compiler's own canonical table
//! (`sail_compiler::block_manifest::BLOCKS`,
//! `crates/sail/compiler/src/block_manifest.rs`).
//!
//! This is what let the grammar rot the first time: `grammar.js` listed
//! `style:`, a `<state>` element, element-form `<while-loading>`/
//! `<while-error>`, and a `{:elif}` alias that the compiler never
//! recognized — all invisible to any test because nothing compared the
//! grammar's declared vocabulary against the compiler's. This test is that
//! comparison, so a block/clause added to (or removed from) the compiler
//! without a matching grammar change fails CI instead of silently
//! mis-parsing `.sail` in editor tooling.

const GRAMMAR_SRC: &str = include_str!("../grammar.js");

/// True if `token` (e.g. `"{#if}"`, `"{:else if}"`) is recognized by
/// grammar.js as a keyword literal. Two shapes exist in the grammar:
///   - Blocks/clauses with dynamic content between the marker and `}`
///     (`{#if cond}`, `{:case value}`, …) declare only the PREFIX as a
///     literal token, e.g. `'{#if'`.
///   - Fixed no-argument clauses (`{:else}`, `{:default}`) declare the
///     WHOLE marker, including the closing `}`, as one literal token.
/// Check both shapes rather than guessing which applies.
fn token_present(token: &str) -> bool {
    let full = format!("'{token}'");
    if GRAMMAR_SRC.contains(&full) {
        return true;
    }
    let prefix = token.trim_end_matches('}');
    if prefix != token {
        let partial = format!("'{prefix}'");
        if GRAMMAR_SRC.contains(&partial) {
            return true;
        }
    }
    false
}

#[test]
fn grammar_block_vocabulary_matches_compiler_manifest() {
    for block in sail_compiler::block_manifest::BLOCKS {
        assert!(
            token_present(block.open),
            "grammar.js has no rule recognizing compiler block `{}` \
             (declared in block_manifest::BLOCKS) — add a grammar rule for it, \
             or this block silently mis-parses in editor tooling.",
            block.open,
        );
        for clause in block.clauses {
            assert!(
                token_present(clause),
                "grammar.js has no rule recognizing clause `{}` of block `{}` \
                 (declared in block_manifest::BLOCKS).",
                clause,
                block.open,
            );
        }
    }
    // Pins the row count so a block ADDED to block_manifest::BLOCKS without
    // a matching grammar.js rule still fails loudly even in the edge case
    // where its open/clause tokens happen to be substrings of something
    // else already in grammar.js.
    assert_eq!(
        sail_compiler::block_manifest::BLOCKS.len(),
        5,
        "block_manifest::BLOCKS changed size — update grammar.js's block \
         rules (if/for/match/while-loading/while-error today) and this \
         test's expected count together.",
    );
}

#[test]
fn grammar_does_not_recognize_elif_alias() {
    // The compiler's tokenizer (parse/template/tokenize.rs::classify_brace)
    // recognizes only `{:else if`, never `{:elif`. The grammar must not
    // accept `{:elif}` as a live else-if spelling.
    assert!(
        !GRAMMAR_SRC.contains("'{:elif'"),
        "grammar.js declares `{{:elif}}` as recognized syntax, but the \
         compiler's tokenizer has no `{{:elif}}` arm — only `{{:else if}}` \
         parses. `{{:elif}}` in a real .sail file should be a parse error, \
         not a silently-accepted else-if.",
    );
}

#[test]
fn grammar_does_not_recognize_retired_style_directive() {
    // `style:` is a hard compile error
    // (parse/template/attrs.rs::compile_attr) — runtime geometry moved to
    // named `use:<action>` directives. The grammar must not have a
    // dedicated rule for `style:` (that would make it parse as a valid
    // directive instead of producing an ERROR node).
    assert!(
        !GRAMMAR_SRC.contains("style_attr") && !GRAMMAR_SRC.contains("'style:'"),
        "grammar.js still declares a `style:` attribute rule, but the \
         compiler hard-refuses `style:` (parse/template/attrs.rs). A .sail \
         file using `style:` must fail to parse, not round-trip as valid.",
    );
}

#[test]
fn grammar_does_not_recognize_retired_class_directive() {
    // `class:` is a hard compile error
    // (parse/template/attrs.rs::compile_attr) — styling is sx / state is
    // aria-* and closed sx prefixes. The grammar must not have a
    // dedicated rule for `class:` (that would make it parse as a valid
    // directive instead of producing an ERROR node).
    assert!(
        !GRAMMAR_SRC.contains("class_attr") && !GRAMMAR_SRC.contains("'class:'"),
        "grammar.js still declares a `class:` attribute rule, but the \
         compiler hard-refuses `class:` (parse/template/attrs.rs). A .sail \
         file using `class:` must fail to parse, not round-trip as valid.",
    );
}
