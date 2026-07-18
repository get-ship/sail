//! Scratch scanner for the confinement-by-default rollout (#98). Not a CI
//! gate — a one-shot tool to enumerate every inline-`<controller>` `.sail`
//! file in the repo and report which ones the confined-by-default
//! `sail-controller-no-ambient-authority` rule now refuses. Run with:
//!   cargo test -p sail-compiler --test confinement_blast_radius -- --ignored --nocapture

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use sail_compiler::{compile, CompileMode, CompileOpts, ComponentMeta};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
        .canonicalize()
        .expect("canonicalize repo root")
}

fn walk_collect(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        if is_dir {
            let name = entry.file_name();
            let n = name.to_string_lossy();
            if n == "dist" || n == "node_modules" || n == "target" || n == ".git" {
                continue;
            }
            walk_collect(&path, out);
        } else if path.extension() == Some(OsStr::new("sail")) {
            out.push(path);
        }
    }
}

/// Same tag-name rule the real bundler uses
/// (`crates/deck/deck-bundler/src/sail_plugin.rs::detect_mode`): a valid
/// custom-element tag must contain a hyphen and every component `.sail` uses
/// its stem as the tag; `page`/`index` (no hyphen) are views. Both map to
/// `sail_compiler::CompileMode::Component` (`compile.rs::compile_sui`) — the
/// View/Component distinction only decides whether `component_meta` is set.
fn classify(path: &Path) -> (CompileMode, Option<ComponentMeta>) {
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
    let is_component = stem.contains('-') && stem != "page" && stem != "index";
    (CompileMode::Component, is_component.then(|| ComponentMeta { name: stem }))
}

#[test]
#[ignore]
fn list_newly_refused_controllers() {
    let repo = repo_root();
    let mut all = Vec::new();
    walk_collect(&repo, &mut all);
    all.sort();

    let mut refused = Vec::new();
    let mut checked = 0usize;
    for path in &all {
        let source = fs::read_to_string(path).unwrap();
        if !source.contains("<controller>") {
            continue;
        }
        checked += 1;
        let (mode, component_meta) = classify(path);
        let opts = CompileOpts {
            route_params: None,
            setup_url: None,
            mode,
            source_path: Some(path.clone()),
            component_meta,
            emit_dts: false,
            scope_capabilities: Vec::new(),
            emit_component_imports: false,
            lint_overrides: sail_compiler::check::policy::Policy::overrides_for(path),
        };
        if let Err(e) = compile(&source, opts) {
            let rel = path.strip_prefix(&repo).unwrap_or(path);
            if e.category.as_deref() == Some("sail-controller-no-ambient-authority") {
                refused.push(format!("{}: {}", rel.display(), e.render(&source)));
            } else {
                refused.push(format!("{}: [OTHER ERROR, not ambient-authority] {}", rel.display(), e.render(&source)));
            }
        }
    }
    println!("checked {checked} controller-bearing files");
    println!("refused: {}", refused.len());
    for r in &refused {
        println!("---\n{r}");
    }
}
