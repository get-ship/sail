//! `cargo run -p sui-lint --example check -- <path>` — lint one file
//! from CLI. Dispatches by extension: `*.controller.ts` runs the
//! controller rules, `*.sui` runs the template rules.

fn main() {
    let path_str = std::env::args().nth(1).expect("usage: check <path>");
    let path = std::path::Path::new(&path_str);
    let findings = sui_lint::lint_path(path);
    println!("findings: {}", findings.len());
    for f in &findings {
        println!("  {}:{} [{}] {}", f.line, f.col, f.rule, f.message);
        if let Some(s) = &f.suggest {
            println!("    suggest: {}", s);
        }
    }
    std::process::exit(if findings.is_empty() { 0 } else { 1 });
}
