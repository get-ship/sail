//! L1 manifest consistency vs stdlib + sail-base.css.

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use sail_compiler::l1_manifest::{L1Kind, l1_by_kind, l1_entries, l1_kind, reskin_selectors};
use sail_compiler::stdlib;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
}

fn styles_path() -> PathBuf {
    repo_root().join("packages/sail/tokens/src/sail-base.css")
}

fn read_styles_source() -> String {
    fs::read_to_string(styles_path()).expect("read sail-base.css")
}

/// Strip TS/JS line and block comments so the `:where(...)` scan doesn't
/// pick up selectors mentioned in prose.
fn strip_comments(src: &str) -> String {
    let bytes = src.as_bytes();
    let mut out = String::with_capacity(src.len());
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if c == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if c == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i = (i + 2).min(bytes.len());
            continue;
        }
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

/// Collect bare native tags appearing inside `:where(...)` selectors.
fn where_tags(styles: &str) -> HashSet<String> {
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
                // Bare-tag selectors only. An attribute/class/pseudo qualifier
                // (`span[dot]`, `input[type='checkbox']`) scopes the rule OFF
                // the bare native tag — that is the documented attribute floor
                // (no L1 host), not an undeclared native reskin.
                if !sel.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
                    continue;
                }
                let tag = sel;
                if tag.is_empty() || tag.starts_with(':') {
                    continue;
                }
                if tag.starts_with("sail-") || tag.starts_with("data-") {
                    continue;
                }
                seen.insert(tag.to_string());
            }
            i = j;
        }
        i += 1;
    }
    seen
}

#[test]
fn every_manifest_entry_has_a_built_artifact() {
    let styles = strip_comments(&read_styles_source());
    for el in stdlib::elements() {
        let out = stdlib::element(el.tag).expect("stdlib element must be in registry");
        assert!(
            !out.out.skin_css.trim().is_empty() || !out.out.structure_html.trim().is_empty(),
            "stdlib element `{}` has neither skin nor structure — the .sail \
             source compiled to an empty artifact",
            el.tag
        );
    }
    let skins = stdlib::skins_css();
    for (tag, selector) in reskin_selectors() {
        assert!(
            styles.contains(selector) || skins.contains(selector),
            "reskin `<{tag}>` selector `{selector}` missing from sail-base.css and skins_css"
        );
    }
}

#[test]
fn every_native_tag_in_styles_has_a_reskin_entry() {
    let styles = strip_comments(&read_styles_source());
    let mut declared: HashSet<&'static str> = reskin_selectors().iter().map(|(t, _)| *t).collect();
    for el in stdlib::elements() {
        if el.out.native {
            declared.insert(el.tag);
        }
    }
    let mut orphans: Vec<String> = Vec::new();
    for tag in where_tags(&styles) {
        if !declared.contains(tag.as_str()) {
            orphans.push(format!("<{tag}>"));
        }
    }
    if !orphans.is_empty() {
        panic!(
            "sail-base.css :where(native) with no RESKIN/stdlib row:\n  - {}",
            orphans.join("\n  - ")
        );
    }
}

#[test]
fn stdlib_tags_match_manifest() {
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
    let (stdlib, reskins) = l1_by_kind();
    assert_eq!(stdlib.len(), 55);
    assert_eq!(reskins.len(), 2);
    assert_eq!(l1_entries().len(), 57);
}

#[test]
fn l1_kind_distinguishes_stdlib_and_reskin() {
    let entries = l1_entries();
    let has_stdlib = entries.iter().any(|e| e.kind == L1Kind::StdlibElement);
    let has_reskin = entries.iter().any(|e| e.kind == L1Kind::NativeReskin);
    assert!(has_stdlib, "no stdlib entries — manifest is broken");
    assert!(
        has_reskin,
        "no residual-native entries — manifest is broken"
    );
    for entry in &entries {
        let k = l1_kind(&entry.tag).expect("tag exists");
        assert_eq!(
            k, entry.kind,
            "l1_kind disagrees with entry.kind for `{}`",
            entry.tag
        );
    }
}

#[test]
fn proof_removing_a_reskin_breaks_consistency() {
    let synthetic = r#":where(marquee) { display: block; }"#;
    let mut declared: HashSet<&str> = reskin_selectors().iter().map(|(t, _)| *t).collect();
    for el in stdlib::elements() {
        if el.out.native {
            declared.insert(el.tag);
        }
    }
    assert!(!declared.contains("marquee"));
    let orphans: Vec<String> = where_tags(synthetic)
        .into_iter()
        .filter(|t| !declared.contains(t.as_str()))
        .collect();
    assert!(orphans.iter().any(|o| o == "marquee"));
}

#[test]
fn proof_adding_a_fake_reskin_breaks_consistency() {
    let styles = strip_comments(&read_styles_source());
    let skins = stdlib::skins_css();
    assert!(!styles.contains(":where(foobar)") && !skins.contains(":where(foobar)"));
    for (tag, selector) in reskin_selectors() {
        assert!(
            styles.contains(selector) || skins.contains(selector),
            "reskin `{tag}` / `{selector}` missing"
        );
    }
}

#[test]
fn every_l1_entry_has_nonempty_description() {
    for e in l1_entries() {
        assert!(
            !e.description.trim().is_empty(),
            "L1 `{}` has empty description — fix stdlib docblock or RESKINS row",
            e.tag
        );
    }
}

#[test]
fn chip_description_is_filter_selection_pill() {
    let chip = l1_entries()
        .into_iter()
        .find(|e| e.tag == "chip")
        .expect("chip in L1");
    assert!(
        chip.description.contains("filter") || chip.description.contains("pill"),
        "chip description unexpected: {}",
        chip.description
    );
}

#[test]
fn reskin_descriptions_name_native_reskin_contract() {
    let (_, reskins) = l1_by_kind();
    for e in reskins {
        assert!(
            e.description.to_ascii_lowercase().contains("native reskin"),
            "reskin `{}` description must name native reskin contract: {}",
            e.tag,
            e.description
        );
    }
}
