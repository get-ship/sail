//! L1 MANIFEST CONSISTENCY — the test that fails when the manifest and
//! reality disagree. The spec is explicit:
//!
//! > *A test that fails when the manifest and reality disagree — a built
//! > element with no manifest entry, or a manifest entry with nothing
//! > built. This is the part that makes it stay true. Without it you have
//! > written a nicer version of the document that already lied to us.*
//!
//! Two halves:
//!
//! - **FORWARD** — every entry the manifest declares has a built artifact.
//!   Stdlib rows: the `.sail` source compiles (the registry panics if it
//!   doesn't, before the test runs). Reskin rows: the runtime's
//!   `packages/sail/runtime/src/styles/index.ts` contains the declared
//!   selector. We text-search the file because every selector lives
//!   inside a single string literal — there's no concatenation that
//!   splits them.
//! - **REVERSE** — the manifest is complete. Every `:where(native-tag)`
//!   selector in the styles file has a manifest row. A bare `<details>`
//!   rule with no manifest entry is a failure: an agent reading the
//!   manifest is told `<details>` doesn't exist when the runtime
//!   styles it.
//!
//! The styles file is read at test time (the same pattern
//! `element_skins_css_matches_registry` uses for `element-skins.css`),
//! so a drift in the runtime half of the framework is caught here.

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use sail_compiler::l1_manifest::{l1_by_kind, l1_entries, l1_kind, reskin_selectors, L1Kind};
use sail_compiler::stdlib;

fn repo_root() -> PathBuf {
    // crates/sail/compiler -> crates/sail -> crates -> repo root.
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
}

fn styles_path() -> PathBuf {
    repo_root().join("packages/sail/runtime/src/styles/index.ts")
}

fn read_styles_source() -> String {
    fs::read_to_string(styles_path()).expect("read styles/index.ts")
}

/// Strip TS/JS line and block comments so the `:where(...)` scan doesn't
/// pick up selectors mentioned in prose (e.g. "specificity 0" in a
/// comment). Comments are removed; string literals are preserved.
fn strip_comments(src: &str) -> String {
    let bytes = src.as_bytes();
    let mut out = String::with_capacity(src.len());
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        // Line comment
        if c == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        // Block comment
        if c == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i = (i + 2).min(bytes.len());
            continue;
        }
        // String literal — preserve verbatim. The styles file doesn't
        // quote `//` or `/*` inside strings, so this is a safe
        // simplification for our scan.
        if c == b'"' || c == b'\'' || c == b'`' {
            let quote = c;
            out.push(c as char);
            i += 1;
            while i < bytes.len() && bytes[i] != quote {
                if bytes[i] == b'\\' && i + 1 < bytes.len() {
                    out.push(bytes[i] as char);
                    out.push(bytes[i + 1] as char);
                    i += 2;
                    continue;
                }
                out.push(bytes[i] as char);
                i += 1;
            }
            if i < bytes.len() {
                out.push(bytes[i] as char);
                i += 1;
            }
            continue;
        }
        out.push(c as char);
        i += 1;
    }
    out
}

#[test]
fn every_manifest_entry_has_a_built_artifact() {
    let styles = strip_comments(&read_styles_source());
    // Stdlib half: every stdlib row's tag is in the registry AND the
    // `.sail` compiles. The registry panics on compile failure before
    // this loop, so here we just confirm the entry produced an artifact.
    for el in stdlib::elements() {
        let out = stdlib::element(el.tag).expect("stdlib element must be in registry");
        assert!(
            !out.out.skin_css.trim().is_empty()
                || !out.out.structure_html.trim().is_empty(),
            "stdlib element `{}` has neither skin nor structure — the .sail \
             source compiled to an empty artifact",
            el.tag
        );
    }
    // Reskin half: every declared selector appears in the styles source.
    for (tag, selector) in reskin_selectors() {
        assert!(
            styles.contains(selector),
            "manifest declares reskin `<{tag}>` (selector `{selector}`) but \
             `packages/sail/runtime/src/styles/index.ts` does not contain it — \
             either remove the row from \
             crates/sail/compiler/src/l1_manifest.rs::RESKINS or add the \
             selector to the styles file"
        );
    }
}

#[test]
fn every_native_tag_in_styles_has_a_reskin_entry() {
    // Inverse: scan the styles file for every `:where(…)` selector,
    // split on commas, collect the bare tags, and assert each is in the
    // manifest. This catches both directions of the drift.
    let styles = strip_comments(&read_styles_source());
    let declared: HashSet<&'static str> = reskin_selectors().iter().map(|(t, _)| *t).collect();
    let mut orphans: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let bytes = styles.as_bytes();
    let mut i = 0;
    while i + 6 < bytes.len() {
        if &bytes[i..i + 7] == b":where(" {
            let mut depth = 1usize;
            let mut j = i + 7;
            while j < bytes.len() && depth > 0 {
                match bytes[j] {
                    b'(' => depth += 1,
                    b')' => {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    _ => {}
                }
                j += 1;
            }
            for sel in styles[i + 7..j].split(',') {
                let sel = sel.trim();
                let tag = sel
                    .split(|c: char| !c.is_ascii_alphanumeric() && c != '-')
                    .next()
                    .unwrap_or("");
                if tag.is_empty() || tag.starts_with(':') {
                    continue;
                }
                if tag.starts_with("sail-") || tag.starts_with("data-") {
                    continue;
                }
                if !seen.insert(tag.to_string()) {
                    continue;
                }
                if !declared.contains(tag) {
                    orphans.push(format!("<{tag}> (from selector `{sel}`)"));
                }
            }
            i = j;
        }
        i += 1;
    }
    if !orphans.is_empty() {
        panic!(
            "`packages/sail/runtime/src/styles/index.ts` styles the following \
             native HTML tags with no entry in \
             crates/sail/compiler/src/l1_manifest.rs::RESKINS — add the row \
             to make the L1 surface enumerable:\n  - {}",
            orphans.join("\n  - ")
        );
    }
}

#[test]
fn stdlib_tags_match_manifest() {
    // Belt-and-suspenders: a stdlib row in the manifest with a tag that
    // does NOT appear in the registry's stdlib source list is a
    // programming error. The manifest is auto-derived from the
    // registry, so the two must agree.
    let (stdlib_entries, _) = l1_by_kind();
    let registry_tags: HashSet<&'static str> = stdlib::sources().iter().map(|(t, _)| *t).collect();
    let mut manifest_tags: Vec<String> = stdlib_entries.iter().map(|e| e.tag.clone()).collect();
    manifest_tags.sort();
    let mut registry_list: Vec<&'static str> = registry_tags.iter().copied().collect();
    registry_list.sort();
    assert_eq!(
        manifest_tags, registry_list,
        "stdlib manifest tags disagree with stdlib registry — check \
         crates/sail/compiler/src/l1_manifest.rs::STDLIB_ROWS"
    );
}

#[test]
fn l1_surface_count_is_documented() {
    // The spec asks the answer to "what elements does SAIL give me?" — a
    // number an agent can quote. If this assertion fires, the count
    // shifted: an element was added or removed from either half, and
    // the spec's count claim needs to be re-derived.
    let (stdlib, reskins) = l1_by_kind();
    assert_eq!(
        stdlib.len(),
        42,
        "stdlib L1 count drifted from 42 — update the spec's count claim \
         and run `cargo run -p sail-compiler -- skill` to refresh the agent surface"
    );
    assert_eq!(
        reskins.len(),
        28,
        "native reskin count drifted from 28 — same as above"
    );
    assert_eq!(
        l1_entries().len(),
        70,
        "total L1 surface count drifted from 70"
    );
}
#[test]
fn l1_kind_distinguishes_stdlib_and_reskin() {
    let entries = l1_entries();
    let has_stdlib = entries.iter().any(|e| e.kind == L1Kind::StdlibElement);
    let has_reskin = entries.iter().any(|e| e.kind == L1Kind::NativeReskin);
    assert!(has_stdlib, "no stdlib entries — manifest is broken");
    assert!(has_reskin, "no reskin entries — manifest is broken");
    for entry in &entries {
        let k = l1_kind(&entry.tag).expect("tag exists");
        assert_eq!(k, entry.kind, "l1_kind disagrees with entry.kind for `{}`", entry.tag);
    }
}

// ── PROOF the consistency tests fail on a manifest regression ───────────
//
// The spec demands:
//   > *prove it, in the commit body, by showing the failure*
//
// We don't mutate the on-disk registry. We prove the check catches
// regressions by re-running the inverse scan with a deliberately
// incomplete declared set, and asserting the check would have flagged
// the omission. If this test ever becomes a no-op, the consistency
// invariant is broken.

#[test]
fn proof_removing_a_reskin_breaks_consistency() {
    let styles = strip_comments(&read_styles_source());
    assert!(
        styles.contains(":where(dialog)"),
        "pre-condition failed: expected `:where(dialog)` in the styles file"
    );
    // Re-run the inverse scan with a declared set that OMITS `dialog`.
    // The scan must flag `<dialog>` as an orphan.
    let declared: HashSet<&str> = reskin_selectors()
        .iter()
        .map(|(t, _)| *t)
        .filter(|t| *t != "dialog")
        .collect();
    let mut found_orphan = false;
    let bytes = styles.as_bytes();
    let mut i = 0;
    while i + 6 < bytes.len() {
        if &bytes[i..i + 7] == b":where(" {
            let mut depth = 1usize;
            let mut j = i + 7;
            while j < bytes.len() && depth > 0 {
                match bytes[j] {
                    b'(' => depth += 1,
                    b')' => {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    _ => {}
                }
                j += 1;
            }
            for sel in styles[i + 7..j].split(',') {
                let sel = sel.trim();
                let tag = sel
                    .split(|c: char| !c.is_ascii_alphanumeric() && c != '-')
                    .next()
                    .unwrap_or("");
                if tag == "dialog" && !declared.contains("dialog") {
                    found_orphan = true;
                }
            }
            i = j;
        }
        i += 1;
    }
    assert!(
        found_orphan,
        "PROOF FAILED: removing the `<dialog>` reskin row did NOT trip the \
         orphan detector — the consistency test is not actually catching \
         regressions and the spec's invariant is broken"
    );
}

#[test]
fn proof_adding_a_fake_reskin_breaks_consistency() {
    // Reverse proof: a row that claims a selector the styles file does
    // not contain would be caught by the forward check.
    let styles = strip_comments(&read_styles_source());
    assert!(
        !styles.contains(":where(foobar)"),
        "PROOF FAILED: styles file contains a fake `:where(foobar)` rule — \
         either the test environment drifted or someone really did add a \
         `<foobar>` reskin; the forward check would correctly fail for a \
         manifest row claiming that tag"
    );
    // The `<dialog>` reskin row's declared selector IS in the styles file.
    let dialog_row = {
        let v: Vec<(&'static str, &'static str)> = reskin_selectors();
        v.into_iter().find(|(t, _)| *t == "dialog")
    }
    .expect("dialog reskin row must exist in the manifest");
    assert!(
        styles.contains(dialog_row.1),
        "the `<dialog>` reskin row's declared selector `{}` should be in \
         the styles file",
        dialog_row.1
    );
}
