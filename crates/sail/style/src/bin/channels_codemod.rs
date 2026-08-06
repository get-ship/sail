//! Rewrite flat `style.…` chains to channel spines in `.sail` files.
//!
//! Usage:
//!   cargo run -p sail-style --bin sail-style-channels-codemod -- [--dry-run] <path>…
//!
//! Paths may be files or directories (recurses for `*.sail`).

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use sail_style::rewrite_source;

fn main() -> ExitCode {
    let mut args: Vec<String> = env::args().skip(1).collect();
    let dry_run = if let Some(i) = args.iter().position(|a| a == "--dry-run") {
        args.remove(i);
        true
    } else {
        false
    };
    if args.is_empty() {
        eprintln!("usage: sail-style-channels-codemod [--dry-run] <path>…");
        return ExitCode::from(2);
    }

    let mut files = Vec::new();
    for a in &args {
        collect_sail(Path::new(a), &mut files);
    }
    files.sort();
    files.dedup();

    let mut changed = 0usize;
    let mut scanned = 0usize;
    for path in &files {
        scanned += 1;
        let Ok(src) = fs::read_to_string(path) else {
            eprintln!("skip (read fail): {}", path.display());
            continue;
        };
        // Skip files with no flat style. chains.
        if !src.contains("style.") {
            continue;
        }
        let out = rewrite_source(&src);
        if out == src {
            continue;
        }
        changed += 1;
        if dry_run {
            println!("would rewrite: {}", path.display());
        } else {
            if let Err(e) = fs::write(path, out) {
                eprintln!("write fail {}: {e}", path.display());
                return ExitCode::FAILURE;
            }
            println!("rewrote: {}", path.display());
        }
    }
    eprintln!("scanned={scanned} changed={changed} dry_run={dry_run}");
    ExitCode::SUCCESS
}

fn collect_sail(path: &Path, out: &mut Vec<PathBuf>) {
    if path.is_file() {
        if path.extension().and_then(|e| e.to_str()) == Some("sail") {
            out.push(path.to_path_buf());
        }
        return;
    }
    let Ok(rd) = fs::read_dir(path) else {
        return;
    };
    for ent in rd.flatten() {
        let p = ent.path();
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if name.starts_with('.') || name == "node_modules" || name == "target" || name == "dist" {
            continue;
        }
        if p.is_dir() {
            collect_sail(&p, out);
        } else if p.extension().and_then(|e| e.to_str()) == Some("sail") {
            out.push(p);
        }
    }
}
