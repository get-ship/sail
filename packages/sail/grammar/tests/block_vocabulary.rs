//! Anti-drift check: fail the build if `grammar.js`'s template-block
//! vocabulary diverges from the compiler's own canonical table
//! (`sail_compiler::block_manifest::BLOCKS`,
//! `crates/sail/compiler/src/block_manifest.rs`).
//!
//! Element-form control flow (`<if>`, `<for>`, …) is locked by open-tag
//! prefix literals (`'<if'`) and close-tag tokens (`'</if>'`) in grammar.js.

const GRAMMAR_SRC: &str = include_str!("../grammar.js");

/// Tag name from element-form open/clause, e.g. `"<if cond={…}>"` → `"if"`.
fn tag_from(token: &str) -> &str {
    let t = token.trim_start_matches('<').trim_start_matches('/');
    t.split(|c: char| c.is_whitespace() || c == '>' || c == '/')
        .next()
        .unwrap_or(token)
}

/// True if grammar.js recognizes this block/clause open as an element
/// control-flow token (`'<if'`, `'</if>'`, etc.).
fn token_present(token: &str) -> bool {
    let tag = tag_from(token);
    let open = format!("'<{tag}'");
    let close = format!("'</{tag}>'");
    GRAMMAR_SRC.contains(&open) || GRAMMAR_SRC.contains(&close) || GRAMMAR_SRC.contains(&format!("'{tag}'"))
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
    assert_eq!(
        sail_compiler::block_manifest::BLOCKS.len(),
        5,
        "block_manifest::BLOCKS changed size — update grammar.js's block \
         rules (if/for/match/while-loading/while-error today) and this \
         test's expected count together.",
    );
}

#[test]
fn grammar_does_not_recognize_brace_control_flow() {
    // Brace form is retired. The grammar must not still declare `{#if` etc.
    for retired in ["'{#if'", "'{#for'", "'{#match'", "'{:else}'", "'{:else if'"] {
        assert!(
            !GRAMMAR_SRC.contains(retired),
            "grammar.js still declares retired brace marker {retired}",
        );
    }
}

#[test]
fn grammar_does_not_recognize_retired_style_directive() {
    assert!(
        !GRAMMAR_SRC.contains("style_attr") && !GRAMMAR_SRC.contains("'style:'"),
        "grammar.js still declares a `style:` attribute rule, but the \
         compiler hard-refuses `style:`.",
    );
}

#[test]
fn grammar_does_not_recognize_retired_class_directive() {
    assert!(
        !GRAMMAR_SRC.contains("class_attr") && !GRAMMAR_SRC.contains("'class:'"),
        "grammar.js still declares a `class:` attribute rule, but the \
         compiler hard-refuses `class:`.",
    );
}
