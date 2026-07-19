//! Anti-drift gate: fail the build if the generated grammar files
//! (`grammar.js`, `queries/highlights.scm`) diverge from the output of
//! the generator script. A manifest change without regenerating the
//! grammar files is a red gate.
//!
//! The test runs the native vocabulary dump (`sail-compiler vocabulary`) and
//! then the TypeScript generator (`scripts/generate-grammar.ts`) into a
//! temporary directory, then asserts the result is byte-identical to the
//! committed files. This is the cannot-differ mechanism for the grammar.

use std::path::{Path, PathBuf};
use std::process::Command;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn run(cmd: &mut Command) -> Result<String, String> {
    let output = cmd.output().map_err(|e| format!("failed to spawn {cmd:?}: {e}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    if !output.status.success() {
        return Err(format!(
            "command failed: {cmd:?}\nstdout:\n{stdout}\nstderr:\n{stderr}",
        ));
    }
    Ok(format!("{stdout}\n{stderr}"))
}

#[test]
fn generated_grammar_matches_committed_files() {
    let temp = std::env::temp_dir().join(format!(
        "grammar-generated-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    ));
    std::fs::create_dir_all(&temp).expect("create temp dir");
    let vocab = temp.join("vocab.json");

    let ws = workspace_root();
    let grammar_dir = ws.join("packages/sail/grammar");
    let generator = grammar_dir.join("scripts/generate-grammar.ts");

    // 1. Dump the compiler's closed vocabulary to JSON.
    let mut vocab_cmd = Command::new("cargo");
    vocab_cmd
        .args([
            "run",
            "-q",
            "-p",
            "sail-compiler",
            "--",
            "vocabulary",
            "--out",
            vocab.to_str().unwrap(),
        ])
        .current_dir(&ws);
    run(&mut vocab_cmd).expect("vocab dump should succeed");
    assert!(vocab.exists(), "vocab dump should write {}", vocab.display());

    // 2. Run the generator into the temp directory.
    let mut gen_cmd = Command::new("bun");
    gen_cmd
        .args([
            "run",
            generator.to_str().unwrap(),
            "--vocab",
            vocab.to_str().unwrap(),
            "--out-dir",
            temp.to_str().unwrap(),
        ])
        .current_dir(&ws);
    run(&mut gen_cmd).expect("generator should succeed");

    // 3. Compare generated files with committed files.
    let generated_grammar = temp.join("grammar.js");
    let generated_highlights = temp.join("queries/highlights.scm");
    let committed_grammar = grammar_dir.join("grammar.js");
    let committed_highlights = grammar_dir.join("queries/highlights.scm");

    let lhs_grammar = std::fs::read_to_string(&generated_grammar)
        .unwrap_or_else(|_| panic!("generated grammar missing: {}", generated_grammar.display()));
    let rhs_grammar = std::fs::read_to_string(&committed_grammar)
        .unwrap_or_else(|_| panic!("committed grammar missing: {}", committed_grammar.display()));
    if lhs_grammar != rhs_grammar {
        panic!(
            "grammar.js is stale; regenerate with `bun run generate` in packages/sail/grammar.\n\
             Generated: {}\nCommitted: {}",
            generated_grammar.display(),
            committed_grammar.display(),
        );
    }

    let lhs_highlights = std::fs::read_to_string(&generated_highlights)
        .unwrap_or_else(|_| panic!("generated highlights missing: {}", generated_highlights.display()));
    let rhs_highlights = std::fs::read_to_string(&committed_highlights)
        .unwrap_or_else(|_| panic!("committed highlights missing: {}", committed_highlights.display()));
    if lhs_highlights != rhs_highlights {
        panic!(
            "queries/highlights.scm is stale; regenerate with `bun run generate` in packages/sail/grammar.\n\
             Generated: {}\nCommitted: {}",
            generated_highlights.display(),
            committed_highlights.display(),
        );
    }

    // 4. Clean up.
    let _ = std::fs::remove_dir_all(&temp);
}
