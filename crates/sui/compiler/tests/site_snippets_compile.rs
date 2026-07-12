//! EVERY SUI SNIPPET ON THE WEBSITE MUST COMPILE.
//!
//! This is the same guard as `skill_examples_compile.rs`, aimed at the other
//! audience. The agent skill teaches machines; the site teaches people; both
//! teach with EXAMPLES, and an example is prose — it rots.
//!
//! It had already rotted. The getting-started page — the first SUI a human ever
//! reads — opened with
//!
//!     <prop name="who" type="string" default="world" />
//!
//! long after props moved into the `<component>` block. The front door was
//! teaching a form the compiler no longer accepts.
//!
//! Docs prose can be wrong and a reader shrugs. Docs CODE that does not compile
//! is worse than no docs: the reader assumes they made the mistake. So the build
//! compiles it.
//!
//! Snippets live in `apps/sui-site/routes/**/snippets.ts` as backtick strings.
//! Many are shell, TS, or HTML — those are skipped. What is left is SUI, and SUI
//! has to compile.

use std::fs;
use std::path::{Path, PathBuf};

use sui_compiler::{compile, CompileMode, CompileOpts};

fn site_routes() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../apps/sui-site/routes")
}

/// Every `snippets.ts` under the site's routes.
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

/// Pull the backtick-delimited template literals out of a snippets module.
/// Deliberately simple — these files are a flat `export const code = { k: \`…\` }`
/// and contain no interpolation.
///
/// A COMMENT IS OPAQUE. Third time today, and this one bit the guard itself: the
/// header comment on the motion page explains what the page replaced, and writes
/// the dead directive in backticks —
///
///     // …which taught `transition:` / `in:` / `out:` …
///
/// — so a scanner that hunts backticks blind reads a JS comment as a snippet, and
/// the retired-vocabulary check fires on the page DOCUMENTING the retirement. You
/// cannot explain what was removed without tripping the rule that removed it.
fn template_literals(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut chars = src.chars().peekable();
    let mut cur: Option<String> = None;
    while let Some(c) = chars.next() {
        // Outside a literal, skip over JS comments entirely.
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
                // Skip the escaped char so `\`` does not close the literal.
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

/// Drop `<!-- … -->` regions, leaving only what the compiler would act on.
fn strip_html_comments(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(open) = rest.find("<!--") {
        out.push_str(&rest[..open]);
        match rest[open + 4..].find("-->") {
            Some(close) => rest = &rest[open + 4 + close + 3..],
            None => return out, // unterminated: the compile guard reports it
        }
    }
    out.push_str(rest);
    out
}

/// Is this snippet SUI SOURCE — as opposed to shell, TypeScript, a reference
/// table, or server-rendered HTML *output*?
///
/// Three filters, and each earns its place:
///
///   * a TAG must be present. The reference page lists directives in a table —
///     `on:click={handler}     attach a DOM event` — which is a cheatsheet, not a
///     template. No tag, not source.
///   * it must not READ as TypeScript or shell. A snippet opening with `import`,
///     `//`, or `npm` is the runtime API or a terminal command; the `.d.ts`
///     conformance test covers the former.
///   * and it needs a marker only SUI source has. The SSR page shows server
///     OUTPUT — `<button on:click="b1">`, where the value is a hydration marker,
///     not an expression — so a directive only counts when it binds `={`.
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

    s.contains("<component>")
        || s.contains("<controller>")
        || s.contains("<prop ") // the RETIRED form — matched on purpose, so it fails loudly
        || s.contains("{#if")
        || s.contains("{#each")
        || s.contains("{#await")
        || s.contains("motion:")
        || s.contains("use:")
        || s.contains("bind:")
        || (s.contains("on:") && s.contains("={"))
}

/// Constructs the language no longer has. A snippet may be perfectly valid
/// TypeScript and still teach something that was deleted — `ref()` is the case in
/// point: it compiles as prose, reads as current, and does not exist. The compile
/// guard cannot catch that, because a TS snippet is never compiled as SUI. So the
/// retired vocabulary gets its own list, checked against EVERY snippet whatever
/// its language.
/// A construct earns a place here ONLY when the compiler actually rejects it.
///
/// The temptation is to list what we have DECIDED to remove. That is how docs get
/// wrong in the other direction: `ref` / `bind:this` were slated for removal, and
/// this list nearly enforced it — but `ref` is still exported from the runtime,
/// `bind:this` still compiles, and 61 `.sui` files use it. Deleting true docs for
/// a live feature is the same sin as keeping stale docs for a dead one. The
/// language IS what the compiler accepts; a plan is not a language.
const RETIRED: &[(&str, &str)] = &[
    ("transition:", "one animation system: use `motion:enter` / `motion:exit`"),
    ("in:fade", "one animation system: use `motion:enter`"),
    ("in:slide", "one animation system: use `motion:enter`"),
    ("in:scale", "one animation system: use `motion:enter`"),
    ("out:fade", "one animation system: use `motion:exit`"),
    ("out:slide", "one animation system: use `motion:exit`"),
    ("out:scale", "one animation system: use `motion:exit`"),
    ("<prop ", "props live in the `<component>{ props: {…} }` block"),
    ("<host ", "the top template element IS the root — style it directly"),
    ("<variants>", "a variant is a PROP with a union type; key its CSS off the attribute"),
    ("ControllerHook", "hooks are gone — state helpers are plain COMPOSITIONS"),
];

#[test]
fn every_sui_snippet_on_the_site_compiles() {
    let routes = site_routes();
    if !routes.exists() {
        return; // the site is not checked out beside the compiler — nothing to guard
    }

    let mut files = Vec::new();
    snippet_files(&routes, &mut files);
    assert!(
        !files.is_empty(),
        "found no snippets.ts under {} — either the site moved or this walker \
         broke; both are bugs, because a guard that silently checks nothing is \
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
            let opts = CompileOpts { mode: CompileMode::Component, ..Default::default() };
            if let Err(e) = compile(&snip, opts) {
                let first = snip.lines().next().unwrap_or("").trim().to_string();
                broken.push(format!("  {route}: {first}\n    -> {}", e.message));
            }
        }
    }

    assert!(checked > 0, "no SUI snippets recognized — the classifier is broken");

    assert!(
        broken.is_empty(),
        "{} of {checked} SUI snippets on the website do not compile. A reader who \
         pastes one of these will assume THEY made the mistake:\n\n{}",
        broken.len(),
        broken.join("\n")
    );
}

/// THE SITE MAY NOT TEACH A CONSTRUCT THE LANGUAGE NO LONGER HAS.
///
/// Compiling the snippets is not enough. A TypeScript snippet is never compiled
/// as SUI, so it can teach `ref()` — removed from the language — indefinitely and
/// stay green. Retiring a construct means retiring the pages that teach it, and
/// that step is the one everybody forgets, because nothing fails when you skip it.
/// Now something does.
#[test]
fn the_site_teaches_no_retired_construct() {
    let routes = site_routes();
    if !routes.exists() {
        return;
    }

    let mut files = Vec::new();
    snippet_files(&routes, &mut files);

    let mut sins = Vec::new();
    for f in &files {
        let Ok(src) = fs::read_to_string(f) else { continue };
        let route = f
            .parent()
            .and_then(|p| p.strip_prefix(&routes).ok())
            .map(|p| p.display().to_string())
            .unwrap_or_default();

        for snip in template_literals(&src) {
            // A COMMENT IS OPAQUE — the fourth time today, and the lesson keeps
            // arriving from a new direction: you cannot document a removal without
            // naming the thing removed. The reference snippet says, in an HTML
            // comment, "there is no separate <variants> block" — and the rule that
            // retired `<variants>` fired on the sentence explaining the retirement.
            // Police the CODE. Prose about dead syntax is how people learn it died.
            let code_only = strip_html_comments(&snip);
            for (dead, fix) in RETIRED {
                if code_only.contains(dead) {
                    sins.push(format!("  {route}: teaches `{dead}` — {fix}"));
                }
            }
        }
    }
    sins.sort();
    sins.dedup();

    assert!(
        sins.is_empty(),
        "the website teaches {} construct(s) the language does not have:\n\n{}",
        sins.len(),
        sins.join("\n")
    );
}
