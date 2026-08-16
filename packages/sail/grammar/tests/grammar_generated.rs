//! Anti-drift gate: fail the build if the generated grammar files
//! (`grammar.js`, `queries/highlights.scm`) diverge from the generator,
//! or if `src/grammar.json` (compiled `parser.c`) does not list the same
//! rules as `grammar.js`.
//!
//! A manifest change without `bun run generate`, or a generate without
//! `tree-sitter generate`, is a red gate.

use std::collections::BTreeSet;
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
    let output = cmd
        .output()
        .map_err(|e| format!("failed to spawn {cmd:?}: {e}"))?;
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
    assert!(
        vocab.exists(),
        "vocab dump should write {}",
        vocab.display()
    );

    // 2. Run the generator into the temp directory (same outputs as `bun run generate`).
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
    let checks = [
        (
            temp.join("grammar.js"),
            grammar_dir.join("grammar.js"),
            "grammar.js",
        ),
        (
            temp.join("queries/highlights.scm"),
            grammar_dir.join("queries/highlights.scm"),
            "queries/highlights.scm",
        ),
    ];
    for (generated, committed, label) in checks {
        let lhs = std::fs::read_to_string(&generated)
            .unwrap_or_else(|_| panic!("generated {label} missing: {}", generated.display()));
        let rhs = std::fs::read_to_string(&committed)
            .unwrap_or_else(|_| panic!("committed {label} missing: {}", committed.display()));
        if lhs != rhs {
            panic!(
                "{label} is stale; regenerate with `bun run generate` in packages/sail/grammar.\n\
                 Generated: {}\nCommitted: {}",
                generated.display(),
                committed.display(),
            );
        }
    }

    let _ = std::fs::remove_dir_all(&temp);
}

/// `grammar.js` is the authored tree-sitter DSL. `src/grammar.json` is what
/// `tree-sitter generate` compiled into `parser.c`. If the JSON rules lag
/// the JS, Helix/nvim load a stale highlighter.
#[test]
fn compiled_parser_rules_match_grammar_js() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let js = std::fs::read_to_string(dir.join("grammar.js")).expect("grammar.js");
    let json = std::fs::read_to_string(dir.join("src/grammar.json")).expect("src/grammar.json");
    let from_js = rule_names_from_grammar_js(&js);
    let from_json = rule_names_from_grammar_json(&json);
    assert!(
        !from_js.is_empty() && !from_json.is_empty(),
        "expected to find rules in grammar.js and src/grammar.json"
    );
    if from_js != from_json {
        let only_js: Vec<_> = from_js.difference(&from_json).collect();
        let only_json: Vec<_> = from_json.difference(&from_js).collect();
        panic!(
            "src/grammar.json (parser.c) drifted from grammar.js — run `tree-sitter generate` \
             in packages/sail/grammar with a real node (not bun).\n\
             only in grammar.js: {only_js:?}\n\
             only in grammar.json: {only_json:?}"
        );
    }
}

fn rule_names_from_grammar_js(src: &str) -> BTreeSet<String> {
    let start = src
        .find("rules: {")
        .expect("grammar.js must contain `rules: {`");
    let mut names = BTreeSet::new();
    for line in src[start..].lines() {
        let t = line.trim();
        let Some((name, _)) = t.split_once(": $ =>") else {
            continue;
        };
        if name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            names.insert(name.to_string());
        }
    }
    names
}

fn rule_names_from_grammar_json(src: &str) -> BTreeSet<String> {
    let start = src
        .find("\"rules\": {")
        .expect("grammar.json must contain `\"rules\": {`");
    let mut names = BTreeSet::new();
    for line in src[start..].lines() {
        if !line.starts_with("    \"") {
            continue;
        }
        let rest = &line[5..];
        let Some(end) = rest.find('"') else {
            continue;
        };
        let name = &rest[..end];
        let after = &rest[end + 1..];
        if after.trim_start().starts_with(':') && after.contains('{') {
            names.insert(name.to_string());
        }
    }
    names
}
