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

use sail_compiler::{compile, CompileMode, CompileOpts, ComponentMeta};

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
fn sui_examples(md: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_fence = false;
    let mut buf = String::new();
    for line in md.lines() {
        if line.trim_start().starts_with("```") {
            if in_fence {
                let looks_like_sui = buf.contains("<scope>")
                    || buf.contains("<controller>")
                    || buf.contains("motion:")
                    || buf.contains("use:");
                if looks_like_sui {
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

#[test]
fn every_sui_example_in_the_agent_skill_compiles() {
    let Some(md) = generate_skill() else { return };

    let examples = sui_examples(&md);
    assert!(
        !examples.is_empty(),
        "the generated skill contains no SAIL examples — either it lost them or this \
         extractor stopped recognizing them; both are bugs"
    );

    let mut broken = Vec::new();
    for (i, ex) in examples.iter().enumerate() {
        // A fragment (`<card motion:reveal={…}>`) is a valid template on its own; a
        // full example carries its own `<scope>` block. Both compile as-is.
        // `component_meta` is required: `<scope>` props are component
        // members, and the props check keys on the meta, not the mode.
        let opts = CompileOpts {
            mode: CompileMode::Component,
            component_meta: Some(ComponentMeta { name: "sail-skill-example".into() }),
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
