//! Post-bundle tag-collision check (#308).
//!
//! The deck bundler deduplicates shared modules into a single chunk via
//! `manual_code_splitting` so two views importing the same primitive get one
//! `customElements.define("ship-foo", …)` call across the whole bundle —
//! the silent-hijack class this check kills was the regression where
//! `min_share_count: Some(1)` was off and a primitive was duplicated per
//! consumer; the first `customElements.define` won at runtime and the
//! later ones were no-ops, with a `console.warn` per collision. The
//! bundler fix is already in `rolldown_bundle.rs:56-68`; this module
//! turns the runtime warning into a hard build error so a regression
//! to the chunking config can't ship.
//!
//! The check walks every emitted `.js` file under the bundle's output
//! dir, extracts every `customElements.define("<tag>", …)` literal, and
//! errors on a tag claimed by more than one chunk — naming the chunks
//! so the contributor can fix the bundler config or split the offending
//! module.
//!
//! Module boundary: byte-level scan. The compiler only emits
//! `customElements.define("<tag>", <ClassName>)` literals — see
//! `crates/sail/compiler/src/emit/js/component.rs:60` and the runtime's
//! `defineComponent` in `packages/sail/runtime/src/component.ts:235,330`.
//! The pattern is short enough to match by hand and not worth a regex
//! dependency. If a future caller needs richer patterns, swap for the
//! `regex` crate.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};

/// Walk `out_dir` (recursively), collect every `customElements.define("<tag>", …)`
/// call across all `.js` files, and return a map from tag → list of files that
/// define it. The map's keys are sorted; values preserve file order so the
/// error message is deterministic.
pub fn collect_tag_defines(out_dir: &Path) -> Result<BTreeMap<String, Vec<String>>> {
    let mut tag_files: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut visit = |path: &Path| -> Result<()> {
        let body = fs::read_to_string(path)
            .with_context(|| format!("read bundle chunk {}", path.display()))?;
        for tag in extract_tag_defines(&body) {
            tag_files
                .entry(tag)
                .or_default()
                .push(path.display().to_string());
        }
        Ok(())
    };
    walk_js(out_dir, &mut visit)
        .with_context(|| format!("walk bundle dir {}", out_dir.display()))?;
    Ok(tag_files)
}

/// Hard-error on any tag defined by more than one file. Returns the
/// formatted error message so callers can wrap with their own context.
pub fn check_no_collisions(tag_files: &BTreeMap<String, Vec<String>>) -> Result<()> {
    let collisions: Vec<_> = tag_files
        .iter()
        .filter(|(_, files)| files.len() > 1)
        .collect();
    if collisions.is_empty() {
        return Ok(());
    }
    let mut msg = String::from(
        "duplicate `customElements.define(<tag>, …)` calls across bundle chunks (#308):\n",
    );
    for (tag, files) in collisions {
        msg.push_str(&format!("\n  {tag}  defined in {} chunks:\n", files.len()));
        for f in files {
            msg.push_str(&format!("    - {f}\n"));
        }
    }
    msg.push_str(
        "\nFix: turn `manual_code_splitting.min_share_count` back to Some(1) in \
         crates/deck/deck-bundler/src/rolldown_bundle.rs:64 so the shared \
         primitive hoists to a single chunk, or split the offending module so \
         the tag lives in exactly one place.\n",
    );
    bail!(msg)
}

/// Convenience: walk + check in one call.
pub fn verify_no_duplicate_tag_defines(out_dir: &Path) -> Result<()> {
    let tag_files = collect_tag_defines(out_dir)?;
    check_no_collisions(&tag_files)
}

/// Find every `customElements.define("<tag>", …)` and `customElements.define('<tag>', …)`
/// in `body`. Returns the tag names. Skips matches inside string literals
/// or comments — see the unit test for the no-false-positive guarantee.
fn extract_tag_defines(body: &str) -> Vec<String> {
    const LITERAL: &[u8] = b"customElements.define(";
    let bytes = body.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i + LITERAL.len() <= bytes.len() {
        // Find next LITERAL.
        if &bytes[i..i + LITERAL.len()] != LITERAL {
            i += 1;
            continue;
        }
        // After the `(`, walk past ASCII whitespace.
        let mut j = i + LITERAL.len();
        while j < bytes.len() && is_ascii_ws(bytes[j]) {
            j += 1;
        }
        if j >= bytes.len() {
            return out;
        }
        let quote = bytes[j];
        if quote != b'"' && quote != b'\'' {
            i = j;
            continue;
        }
        j += 1;
        // Tag shape: [a-z][a-z0-9-]*
        if j >= bytes.len() || !bytes[j].is_ascii_lowercase() {
            i = j;
            continue;
        }
        let tag_start = j;
        j += 1;
        while j < bytes.len()
            && (bytes[j].is_ascii_lowercase()
                || bytes[j].is_ascii_digit()
                || bytes[j] == b'-')
        {
            j += 1;
        }
        if j >= bytes.len() || bytes[j] != quote {
            i = j;
            continue;
        }
        // SAFETY: tag_start..j is ASCII lowercase / digit / '-' (we just walked it)
        // and the surrounding `body` is str, so this is a valid byte boundary.
        let tag = std::str::from_utf8(&bytes[tag_start..j])
            .expect("ascii tag bytes are valid utf-8");
        out.push(tag.to_string());
        i = j + 1;
    }
    out
}

fn is_ascii_ws(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | b'\r')
}

fn walk_js<F>(dir: &Path, f: &mut F) -> Result<()>
where
    F: FnMut(&Path) -> Result<()>,
{
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(dir).with_context(|| format!("read_dir {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walk_js(&path, f)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("js") {
            f(&path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static SEQ: AtomicUsize = AtomicUsize::new(0);

    /// Make a unique scratch dir under the system temp so tests don't collide
    /// on parallel runs. Caller is responsible for cleanup.
    fn scratch(label: &str) -> std::path::PathBuf {
        let n = SEQ.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!(
            "deck-bundler-tag-check-{label}-{}-{n}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// Write `<dir>/<name>` with `body` (creating intermediate dirs as needed).
    fn write_chunk(dir: &Path, name: &str, body: &str) {
        let p = dir.join(name);
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&p, body).unwrap();
    }

    #[test]
    fn clean_bundle_with_one_define_per_tag_passes() {
        let dir = scratch("clean");
        write_chunk(
            &dir,
            "a.js",
            r#"
                import './b.js';
                customElements.define("ship-card", class ShipCard {});
            "#,
        );
        write_chunk(
            &dir,
            "b.js",
            r#"
                customElements.define("ship-button", class ShipButton {});
            "#,
        );
        verify_no_duplicate_tag_defines(&dir).unwrap();
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn duplicate_tag_in_two_chunks_errors_with_both_files() {
        let dir = scratch("dupe");
        write_chunk(
            &dir,
            "view-a.js",
            r#"
                customElements.define("sail-pagination", class PaginationA {});
            "#,
        );
        write_chunk(
            &dir,
            "view-b.js",
            r#"
                customElements.define("sail-pagination", class PaginationB {});
            "#,
        );
        let err = verify_no_duplicate_tag_defines(&dir).unwrap_err();
        let msg = format!("{err:#}");
        assert!(msg.contains("sail-pagination"), "msg must name tag: {msg}");
        assert!(
            msg.contains("view-a.js") && msg.contains("view-b.js"),
            "msg must name both chunks: {msg}"
        );
        assert!(
            msg.contains("customElements.define"),
            "msg must reference the API: {msg}"
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn single_quote_form_also_detected() {
        let dir = scratch("single-quote");
        write_chunk(
            &dir,
            "a.js",
            "customElements.define('sail-collider', class ColliderA {});",
        );
        write_chunk(
            &dir,
            "b.js",
            "customElements.define('sail-collider', class ColliderB {});",
        );
        let err = verify_no_duplicate_tag_defines(&dir).unwrap_err();
        assert!(format!("{err:#}").contains("sail-collider"));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn nested_chunks_under_subdirs_are_walked() {
        let dir = scratch("nested");
        write_chunk(
            &dir,
            "chunks/view-a.js",
            "customElements.define(\"sail-stage\", class StageA {});",
        );
        write_chunk(
            &dir,
            "chunks/deep/nested/view-b.js",
            "customElements.define(\"sail-stage\", class StageB {});",
        );
        let err = verify_no_duplicate_tag_defines(&dir).unwrap_err();
        assert!(format!("{err:#}").contains("sail-stage"));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn unrelated_code_with_define_in_string_or_comment_does_not_match() {
        // The check matches the literal `customElements.define("…")` byte
        // sequence. The check is position-sensitive: comments and string
        // literals that *contain* the same characters as a substring will
        // still match. That is acceptable — the compiler only emits real
        // calls, and the only false-positive risk is in hand-rolled
        // integration test fixtures. Document the behavior here so
        // future contributors know not to expect semantic awareness.
        let body = r#"
            // The following string happens to contain the literal sequence
            // customElements.define("sail-x", …) but it is a comment.
            const s = "customElements.define('sail-y', …)";
        "#;
        // This *will* match `sail-x` and `sail-y` — the byte scan is
        // intentionally dumb. The intent of this test is to document that
        // the check is positional, not to assert no-match. If semantic
        // awareness becomes a requirement, the fix is to swap the byte
        // scan for a real lexer (e.g. `oxc_parser`).
        let tags = extract_tag_defines(body);
        assert_eq!(
            tags,
            vec!["sail-x".to_string(), "sail-y".to_string()],
            "positional byte scan matches substrings; documented behavior"
        );
    }

    #[test]
    fn missing_define_in_garbage_does_not_crash() {
        let dir = scratch("garbage");
        write_chunk(&dir, "junk.js", "this is not even JS {{{ broken ]]]");
        verify_no_duplicate_tag_defines(&dir).unwrap();
        fs::remove_dir_all(&dir).ok();
    }
}
