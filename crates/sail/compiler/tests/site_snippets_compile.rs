//! Every SUI snippet on the website must compile.
//!
//! Docs code that does not compile is worse than no docs: the reader assumes they
//! made the mistake. Snippets live in `apps/sail-site/routes/**/snippets.ts` as
//! backtick strings; the ones that are SUI get compiled.

use std::fs;
use std::path::{Path, PathBuf};

use sail_compiler::{compile, CompileMode, CompileOpts};

fn site_routes() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../apps/sail-site/routes")
}

fn snippet_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            snippet_files(&p, out);
        } else if p.file_name().is_some_and(|n| n == "snippets.ts") {
            out.push(p);
        }
    }
}

/// The backtick-delimited template literals in a snippets module. JS comments are
/// skipped: a comment may quote SUI source without being any.
fn template_literals(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut chars = src.chars().peekable();
    let mut cur: Option<String> = None;
    while let Some(c) = chars.next() {
        if cur.is_none() && c == '/' {
            match chars.peek() {
                Some('/') => {
                    for n in chars.by_ref() {
                        if n == '\n' {
                            break;
                        }
                    }
                    continue;
                }
                Some('*') => {
                    chars.next();
                    let mut prev = '\0';
                    for n in chars.by_ref() {
                        if prev == '*' && n == '/' {
                            break;
                        }
                        prev = n;
                    }
                    continue;
                }
                _ => continue,
            }
        }
        match (&mut cur, c) {
            (None, '`') => cur = Some(String::new()),
            (Some(_), '\\') => {
                if let Some(esc) = chars.next() {
                    if let Some(buf) = cur.as_mut() {
                        buf.push('\\');
                        buf.push(esc);
                    }
                }
            }
            (Some(_), '`') => out.push(cur.take().unwrap_or_default()),
            (Some(buf), ch) => buf.push(ch),
            (None, _) => {}
        }
    }
    out
}

/// Is this snippet SUI source — as opposed to shell, TypeScript, a reference
/// table, or server-rendered HTML output?
///
/// A tag must be present: the reference page lists directives in a table
/// (`on:click={handler}     attach a DOM event`), which is a cheatsheet, not a
/// template. It must not read as TS or shell. And a directive only counts when it
/// binds `={`: the SSR page shows server OUTPUT, where `on:click="b1"` is a
/// hydration marker rather than an expression.
fn is_sui(s: &str) -> bool {
    let has_tag = s
        .as_bytes()
        .windows(2)
        .any(|w| w[0] == b'<' && (w[1].is_ascii_alphabetic() || w[1] == b'/'));
    if !has_tag {
        return false;
    }

    let first = s.lines().map(str::trim).find(|l| !l.is_empty()).unwrap_or("");
    let reads_as_code_not_markup = first.starts_with("import ")
        || first.starts_with("//")
        || first.starts_with("export ")
        || first.starts_with('#')
        || first.starts_with("npm")
        || first.starts_with("pnpm");
    if reads_as_code_not_markup {
        return false;
    }

    s.contains("<scope>")
        || s.contains("<controller>")
        || s.contains("{#if")
        || s.contains("{#each")
        || s.contains("{#await")
        || s.contains("motion:")
        || s.contains("use:")
        || s.contains("bind:")
        || (s.contains("on:") && s.contains("={"))
}

#[test]
fn every_sui_snippet_on_the_site_compiles() {
    let routes = site_routes();
    if !routes.exists() {
        return; // the site is not checked out beside the compiler
    }

    let mut files = Vec::new();
    snippet_files(&routes, &mut files);
    assert!(
        !files.is_empty(),
        "found no snippets.ts under {} — a guard that silently checks nothing is \
         worse than no guard",
        routes.display()
    );

    let mut checked = 0usize;
    let mut broken = Vec::new();

    for f in &files {
        let Ok(src) = fs::read_to_string(f) else { continue };
        let route = f
            .parent()
            .and_then(|p| p.strip_prefix(&routes).ok())
            .map(|p| p.display().to_string())
            .unwrap_or_default();

        for snip in template_literals(&src) {
            if !is_sui(&snip) {
                continue;
            }
            checked += 1;
            // A snippet that declares `<scope>` props/variants teaches a
            // COMPONENT — compile it as one (a tag), or the position gate
            // rightly refuses props in a tagless route/page compile.
            let component_meta = snip
                .contains("<scope>")
                .then(|| sail_compiler::ComponentMeta { name: "snippet-demo".into() });
            let opts =
                CompileOpts { mode: CompileMode::Component, component_meta, ..Default::default() };
            if let Err(e) = compile(&snip, opts) {
                let first = snip.lines().next().unwrap_or("").trim().to_string();
                broken.push(format!("  {route}: {first}\n    -> {}", e.message));
            }
        }
    }

    assert!(checked > 0, "no SUI snippets recognized — the classifier is broken");

    assert!(
        broken.is_empty(),
        "{} of {checked} SUI snippets on the website do not compile:\n\n{}",
        broken.len(),
        broken.join("\n")
    );
}

/// EVERY snippets.ts must PARSE.
///
/// Twice now a backtick inside a template literal — ``…at least `min` as fit…`` —
/// silently closed the string, turning the rest of the file into garbage. The
/// snippet compile guard never sees it (the extractor just finds fewer
/// literals), so it surfaces at BUNDLE time as a wall of oxc noise, minutes
/// later, with the dev server dead.
///
/// A balanced-backtick check is not enough (an escaped \` is legal), so this
/// asks the question that actually matters: does the file's template-literal
/// structure make sense? An odd number of unescaped backticks cannot.
#[test]
fn every_snippets_file_has_balanced_template_literals() {
    let routes = site_routes();
    if !routes.exists() {
        return;
    }
    let mut files = Vec::new();
    snippet_files(&routes, &mut files);

    let mut broken = Vec::new();
    for f in &files {
        let Ok(src) = fs::read_to_string(f) else { continue };
        // Count backticks that are neither escaped nor inside a line comment.
        let mut ticks = 0usize;
        let mut chars = src.chars().peekable();
        while let Some(c) = chars.next() {
            match c {
                '\\' => {
                    chars.next();
                }
                '`' => ticks += 1,
                _ => {}
            }
        }
        if ticks % 2 != 0 {
            broken.push(format!(
                "  {}: {ticks} unescaped backticks — a template literal is left open",
                f.display()
            ));
        }
    }
    assert!(
        broken.is_empty(),
        "a backtick inside a template literal closes it. The build dies at BUNDLE time \
         with an unreadable parser error, minutes after you typed it:\n\n{}",
        broken.join("\n")
    );
}
