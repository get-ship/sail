//! L1 manifest consistency vs stdlib (no native-reskin surface).

use std::collections::HashSet;

use sail_compiler::l1_manifest::{L1JsKind, L1Kind, l1_by_kind, l1_entries, l1_kind};
use sail_compiler::stdlib;

#[test]
fn every_manifest_entry_has_a_built_artifact() {
    for el in stdlib::elements() {
        let out = stdlib::element(el.tag).expect("stdlib element must be in registry");
        assert!(
            !out.out.skin_css.trim().is_empty() || !out.out.structure_html.trim().is_empty(),
            "stdlib element `{}` has neither skin nor structure — the .sail \
             source compiled to an empty artifact",
            el.tag
        );
    }
}

#[test]
fn stdlib_tags_match_manifest() {
    let (stdlib_entries, reskins) = l1_by_kind();
    assert!(
        reskins.is_empty(),
        "l1_by_kind().1 (reskins) must be empty, got {}",
        reskins.len()
    );
    let registry_tags: HashSet<&'static str> = stdlib::sources().iter().map(|(t, _)| *t).collect();
    let mut manifest_tags: Vec<String> = stdlib_entries.iter().map(|e| e.tag.clone()).collect();
    manifest_tags.sort();
    let mut registry_list: Vec<&'static str> = registry_tags.iter().copied().collect();
    registry_list.sort();
    assert_eq!(
        manifest_tags, registry_list,
        "stdlib manifest tags disagree with stdlib registry — check \
         crates/sail/compiler/src/stdlib.rs::SOURCES"
    );
}

#[test]
fn l1_surface_count_is_documented() {
    let (stdlib, reskins) = l1_by_kind();
    assert_eq!(stdlib.len(), 60, "stdlib L1 count");
    assert_eq!(reskins.len(), 0, "reskins empty");
    assert_eq!(l1_entries().len(), 60);
}

#[test]
fn l1_kind_is_stdlib_only() {
    let entries = l1_entries();
    assert!(
        !entries.is_empty(),
        "no stdlib entries — manifest is broken"
    );
    for entry in &entries {
        assert_eq!(
            entry.kind,
            L1Kind::StdlibElement,
            "L1 `{}` must be StdlibElement",
            entry.tag
        );
        let k = l1_kind(&entry.tag).expect("tag exists");
        assert_eq!(
            k, entry.kind,
            "l1_kind disagrees with entry.kind for `{}`",
            entry.tag
        );
    }
    assert!(
        entries.iter().any(|e| e.tag == "accordion"),
        "accordion must be L1"
    );
    assert!(entries.iter().any(|e| e.tag == "kbd"), "kbd must be L1");
    assert!(
        entries.iter().any(|e| e.tag == "skeleton"),
        "skeleton must be L1"
    );
    assert!(
        entries.iter().any(|e| e.tag == "separator"),
        "separator must be L1"
    );
}

#[test]
fn every_l1_entry_has_nonempty_description() {
    for e in l1_entries() {
        assert!(
            !e.description.trim().is_empty(),
            "L1 `{}` has empty description — fix stdlib docblock",
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
fn accordion_description_names_disclosure() {
    let acc = l1_entries()
        .into_iter()
        .find(|e| e.tag == "accordion")
        .expect("accordion in L1");
    let d = acc.description.to_ascii_lowercase();
    assert!(
        d.contains("disclosure") || d.contains("accordion") || d.contains("accordion"),
        "accordion description unexpected: {}",
        acc.description
    );
}

#[test]
fn kbd_description_names_keycap() {
    let kbd = l1_entries()
        .into_iter()
        .find(|e| e.tag == "kbd")
        .expect("kbd in L1");
    let d = kbd.description.to_ascii_lowercase();
    assert!(
        d.contains("key") || d.contains("mono") || d.contains("chip"),
        "kbd description unexpected: {}",
        kbd.description
    );
}

#[test]
fn skeleton_description_names_loading_placeholder() {
    let sk = l1_entries()
        .into_iter()
        .find(|e| e.tag == "skeleton")
        .expect("skeleton in L1");
    let d = sk.description.to_ascii_lowercase();
    assert!(
        d.contains("load")
            || d.contains("placeholder")
            || d.contains("shimmer")
            || d.contains("sheen"),
        "skeleton description unexpected: {}",
        sk.description
    );
    let names: Vec<&str> = sk.attributes.iter().map(|a| a.name.as_str()).collect();
    assert!(names.contains(&"shape"), "skeleton must declare shape prop");
    assert!(names.contains(&"size"), "skeleton must declare size prop");
}

#[test]
fn every_l1_entry_declares_component_tokens_and_js() {
    let entries = l1_entries();
    assert_eq!(entries.len(), 59, "stdlib L1 count");
    for e in &entries {
        // Fields must be present (empty vec / None-kind is valid).
        let _ = &e.component_tokens;
        let _ = &e.js;
        for tok in &e.component_tokens {
            assert!(
                !tok.name.trim().is_empty(),
                "L1 `{}` component token has empty name",
                e.tag
            );
            assert!(
                matches!(
                    tok.ty,
                    sail_compiler::L1SlotType::Scalar | sail_compiler::L1SlotType::Composite
                ),
                "L1 `{}` token `{}` slot type",
                e.tag,
                tok.name
            );
        }
        match e.js.kind {
            L1JsKind::None => assert!(
                e.js.behaviors.is_empty(),
                "L1 `{}` js=none must have empty behaviors: {:?}",
                e.tag,
                e.js.behaviors
            ),
            L1JsKind::Enhancement | L1JsKind::Required => assert!(
                !e.js.behaviors.is_empty(),
                "L1 `{}` js={:?} must list behaviors",
                e.tag,
                e.js.kind
            ),
        }
    }
}

#[test]
fn component_tokens_and_js_drift_pin_against_compiled_metadata() {
    let entries = l1_entries();
    for el in stdlib::elements() {
        let row = entries
            .iter()
            .find(|e| e.tag == el.tag)
            .unwrap_or_else(|| panic!("manifest missing {}", el.tag));
        let html = &el.out.structure_html;
        for behavior in &row.js.behaviors {
            let stamp = format!("use:{behavior}");
            assert!(
                html.contains(&stamp),
                "L1 `{}` lists js behavior `{behavior}` not present as `{stamp}` in structure",
                el.tag
            );
        }
        // Every use: in structure is listed.
        let mut i = 0;
        let bytes = html.as_bytes();
        while i + 4 < bytes.len() {
            if &bytes[i..i + 4] == b"use:" {
                let start = i + 4;
                let mut end = start;
                while end < bytes.len()
                    && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'-')
                {
                    end += 1;
                }
                if end > start {
                    let name = &html[start..end];
                    assert!(
                        row.js.behaviors.iter().any(|b| b == name),
                        "L1 `{}` structure has `{name}` but manifest js.behaviors={:?}",
                        el.tag,
                        row.js.behaviors
                    );
                }
                i = end;
                continue;
            }
            i += 1;
        }
        if sail_lint::style::surface_pack_for_host(el.tag).is_some()
            && !row.component_tokens.is_empty()
        {
            assert!(
                row.component_tokens.iter().any(|t| !t.name.is_empty()),
                "L1 `{}` pack tokens missing a named slot: {:?}",
                el.tag,
                row.component_tokens
            );
        }
    }
}
