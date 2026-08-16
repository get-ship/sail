//! Anti-drift check: fail the build if `grammar.js`'s template-block
//! vocabulary diverges from the compiler's own canonical table
//! (`sail_compiler::block_manifest::BLOCKS`,
//! `crates/sail/compiler/src/block_manifest.rs`).
//!
//! Element-form control flow (`<if>`, `<for>`, …) is locked by open-tag
//! prefix literals (`'<if'`) and close-tag tokens (`'</if>'`) in grammar.js.
//! Arm clauses (`{else => …}`) ride the interpolation rule.

const GRAMMAR_SRC: &str = include_str!("../grammar.js");

/// Tag name from element-form open/clause, e.g. `"<if {…}>"` → `"if"`.
fn tag_from(token: &str) -> &str {
    let t = token.trim_start_matches('<').trim_start_matches('/');
    t.split(|c: char| c.is_whitespace() || c == '>' || c == '/')
        .next()
        .unwrap_or(token)
}

/// True if grammar.js recognizes this block/clause.
fn token_present(token: &str) -> bool {
    // Arm clauses are brace-form — covered by `interpolation`.
    if token.trim_start().starts_with('{') {
        return GRAMMAR_SRC.contains("interpolation:") || GRAMMAR_SRC.contains("$.interpolation");
    }
    let tag = tag_from(token);
    let open = format!("'<{tag}'");
    let close = format!("'</{tag}>'");
    GRAMMAR_SRC.contains(&open)
        || GRAMMAR_SRC.contains(&close)
        || GRAMMAR_SRC.contains(&format!("'{tag}'"))
        || GRAMMAR_SRC.contains(&format!("/{tag}/"))
        || GRAMMAR_SRC.contains(&format!("<{tag}>"))
        || GRAMMAR_SRC.contains(&format!("<{tag}["))
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
         rules (if/for/match/when/partial) and this test's expected count together.",
    );
}

#[test]
fn grammar_block_rules_are_if_for_match_when_partial_only() {
    assert!(
        !GRAMMAR_SRC.contains("while_loading_block")
            && !GRAMMAR_SRC.contains("while_error_block")
            && !GRAMMAR_SRC.contains("case_clause")
            && !GRAMMAR_SRC.contains("default_clause")
            && !GRAMMAR_SRC.contains("else_if_clause")
            && !GRAMMAR_SRC.contains("else_clause"),
        "grammar.js block element rules must be if/for/match/when/partial only",
    );
}

#[test]
fn grammar_has_no_style_colon_directive() {
    assert!(
        !GRAMMAR_SRC.contains("'style:'"),
        "grammar.js still declares a `style:` attribute rule, but the \
         compiler hard-refuses `style:`. Live attach is style_attr `style(…)`.",
    );
}

#[test]
fn grammar_has_no_class_attr_directive() {
    assert!(
        !GRAMMAR_SRC.contains("class_attr") && !GRAMMAR_SRC.contains("'class:'"),
        "grammar.js still declares a `class:` attribute rule, but the \
         compiler hard-refuses `class:`.",
    );
}
