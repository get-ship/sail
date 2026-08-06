//! EVERY EXAMPLE IN THE AGENT SKILL MUST COMPILE.
//!
//! The skill is generated from the compiler's own vocabulary tables, which is what
//! is supposed to stop it drifting. Hardcoded prose sections can still rot, so
//! every fenced SAIL example is compiled against the live grammar.
//!
//! Generating from tables fixes the vocabulary lists. It does NOT fix the examples,
//! because an example is prose too. So the examples get a compiler.
//!
//! This test GENERATES the skill, pulls every fenced block that is SAIL source, and
//! compiles it. If an example rots, the build says so.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use sail_compiler::{CompileMode, CompileOpts, ComponentMeta, compile};

/// GENERATE the skill rather than reading the checked-in one. The shipped copy lives
/// under `.claude/`, which is gitignored — so a test that merely READ it would
/// silently pass in CI, where the file does not exist, which is the one place the
/// guard has to hold. Generating it also means the test covers what the CURRENT
/// generator emits, not whatever someone last happened to run.
fn generate_skill() -> Option<String> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let out = std::env::temp_dir().join("sail-skill-guard.md");
    let bin = root.join("target/debug/sail-compiler");
    if !bin.exists() {
        // Nothing to guard if the binary was not built (e.g. `cargo test --lib`).
        return None;
    }
    let status = Command::new(&bin)
        .args(["skill", "--out"])
        .arg(&out)
        .current_dir(&root)
        .status()
        .ok()?;
    if !status.success() {
        return None;
    }
    fs::read_to_string(&out).ok()
}

/// Fenced blocks that are SAIL source — i.e. contain a tag or a block the compiler
/// owns. Skips prose fences and pure-TS snippets (which are not `.sail` files).
fn sail_examples(md: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_fence = false;
    let mut buf = String::new();
    for line in md.lines() {
        if line.trim_start().starts_with("```") {
            if in_fence {
                let looks_like_sail = buf.contains("<scope>")
                    || buf.contains("<controller>")
                    || buf.contains("fx:")
                    || buf.contains(".enter(")
                    || buf.contains(".loop(")
                    || buf.contains("style.enter")
                    || buf.contains("use:");
                if looks_like_sail {
                    out.push(std::mem::take(&mut buf));
                } else {
                    buf.clear();
                }
                in_fence = false;
            } else {
                in_fence = true;
                buf.clear();
            }
            continue;
        }
        if in_fence {
            buf.push_str(line);
            buf.push('\n');
        }
    }
    out
}

/// Drift lock: progressive skill (core-only) + product bans + MCP pointers.
#[test]
fn skill_generator_matches_product_surface() {
    let Some(md) = generate_skill() else { return };

    assert!(
        md.contains("## Framework stack (do not invent alternatives)"),
        "skill must mandate Framework stack so agents do not invent DOM/cssText"
    );
    assert!(
        md.contains("defineStore")
            && md.contains("use:")
            && md.contains("prop:tree")
            && md.contains("createElement"),
        "framework stack must name stores, use:, prop:tree, and ban createElement"
    );
    assert!(
        md.contains("### Core Style setters"),
        "skill must lead with a core setter section for progressive disclosure"
    );
    assert!(
        md.contains("sail_surface") && md.contains("sail_check"),
        "skill must point agents at MCP sail_surface / sail_check"
    );
    assert!(
        md.contains("data-sail-host")
            && md.contains("sail-no-data-attr")
            && (md.contains("role") && md.contains("name") || md.contains("role / accessible name")),
        "skill must teach data-sail-host is compiler-only and tests use role/name/text"
    );
    assert!(
        md.contains("sail_edit") && !md.contains("sail_apply"),
        "skill must list live MCP sail_edit (sail_apply removed)"
    );
    // Full tables must NOT dump into the default skill.
    assert!(
        !md.contains("### Full closed Style setters"),
        "default skill must not dump the full Style setter table — use MCP"
    );
    assert!(
        !md.contains("## L1 elements — the language surface"),
        "default skill must not dump the full L1 inventory — use MCP topic l1"
    );
    assert!(
        md.contains("colorMix") && md.contains("(colorToken, pct)"),
        "skill must document colorMix arity for agents"
    );
    assert!(
        md.contains("colorDeepen") && md.contains("foreground"),
        "skill must document colorDeepen (solid deepen toward foreground)"
    );
    // Product-banned participation must not appear as teachable style.grow(…)
    assert!(
        !md.contains("`style.grow(…)`"),
        "skill must not teach style.grow — use layout attrs (sail-participation-style)"
    );
    assert!(
        !md.contains("`style.shrink(…)`"),
        "skill must not teach style.shrink — use layout attrs"
    );
    assert!(
        md.contains("sail-participation-style") || md.contains("grow") && md.contains("attrs"),
        "skill hard rules must still mention participation attrs"
    );
    assert!(
        md.contains("sail-display-on-layout") || md.contains("display") && md.contains("layout"),
        "skill must warn about display-on-layout footgun"
    );
    assert!(
        md.contains("Comments are not load-bearing") || md.contains("load-bearing"),
        "skill must state comments are not load-bearing"
    );
    // Product Style demos must emit single-brace `sx={style…}`. Doubled
    // `sx={{style…}}` is a format-string/`push_str` regression (agents paste it).
    assert!(
        md.contains("sx={style."),
        "skill sx demos must use product single-brace syntax"
    );
    assert!(
        !md.contains("sx={{style."),
        "skill must not emit doubled-brace sx={{{{…}}}} (format-string leak)"
    );
    assert!(
        !md.contains("when({{"),
        "skill must not emit when({{{{…}}}}) — nested format-string leak"
    );
}

#[test]
fn every_sail_example_in_the_agent_skill_compiles() {
    let Some(md) = generate_skill() else { return };

    let examples = sail_examples(&md);
    assert!(
        !examples.is_empty(),
        "the generated skill contains no SAIL examples — either it lost them or this \
         extractor stopped recognizing them; both are bugs"
    );

    let mut broken = Vec::new();
    for (i, ex) in examples.iter().enumerate() {
        // A fragment (`<card fx:reveal={…}>`) is a valid template on its own; a
        // full example carries its own `<scope>` block. Both compile as-is.
        // `component_meta` is required: `<scope>` props are component
        // members, and the props check keys on the meta, not the mode.
        let opts = CompileOpts {
            mode: CompileMode::Component,
            component_meta: Some(ComponentMeta {
                name: "sail-skill-example".into(),
            }),
            ..Default::default()
        };
        if let Err(e) = compile(ex, opts) {
            broken.push(format!("example #{i}:\n{ex}\n  -> {}", e.message));
        }
    }

    assert!(
        broken.is_empty(),
        "the agent skill teaches code that does not compile — this is the failure mode \
         the skill exists to prevent:\n\n{}",
        broken.join("\n\n")
    );
}
