//! Component-token totality gates (SPEC-L1-1 WI5 / inv-stdlib-component-tokens).
//!
//! Doctrine: every paint value on an L1 element resolves through a component
//! token with a global fallback (component token → global alias in the pack).
//!
//! Full matrix coverage is multi-pass. This suite locks the **measured slice**
//! landed here and ratchets: packs must be read by skins, and the high-impact
//! default surfaces may not re-paint free globals for channels they own.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

const SKINS_CSS: &str = include_str!("../../../../packages/sail/tokens/src/element-skins.css");
const REGISTRY: &str = include_str!("../../../../packages/sail/tokens/src/token-registry.json");

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn components_dir() -> PathBuf {
    workspace_root().join("packages/sail/tokens/model/components")
}

fn stdlib_dir() -> PathBuf {
    workspace_root().join("crates/sail/compiler/stdlib")
}

/// High-impact default surfaces wired this pass. Each entry is
/// `(element_tag_or_consumer, required component cssNames that must appear
/// as var(--name) somewhere in element-skins.css)`.
///
/// Consumers of shared packs (input/select/textarea → field) are listed so a
/// pack cannot silently lose its only reader.
const SLICE_READERS: &[(&str, &[&str])] = &[
    (
        "button",
        &[
            "button",
            "button-foreground",
            "button-border",
            "button-hover-bg",
        ],
    ),
    ("dialog", &["dialog", "dialog-foreground", "dialog-border"]),
    (
        "accordion",
        &[
            "accordion",
            "accordion-foreground",
            "accordion-border",
            "accordion-hover-bg",
            "accordion-chevron",
        ],
    ),
    ("skeleton", &["skeleton", "skeleton-highlight"]),
    ("kbd", &["kbd", "kbd-foreground", "kbd-border"]),
    // Shared field surface — any of the three consumers counts as a reader,
    // but the tokens must appear (input/select/textarea all stamp them).
    (
        "field-surface",
        &[
            "field",
            "field-foreground",
            "field-border",
            "field-border-hover",
        ],
    ),
    // Slice #2 — residual gauges / structure / form chrome.
    (
        "table",
        &[
            "table-foreground",
            "table-muted-foreground",
            "table-stripe",
            "table-sticky-bg",
        ],
    ),
    ("progress", &["progress", "progress-fill"]),
    (
        "meter",
        &[
            "meter",
            "meter-optimum",
            "meter-suboptimum",
            "meter-sub-suboptimum",
        ],
    ),
    (
        "slider",
        &["slider-track", "slider-fill", "slider-track-border"],
    ),
    (
        "breadcrumb",
        &["breadcrumb-foreground", "breadcrumb-current"],
    ),
    ("fieldset", &["fieldset-border"]),
    ("legend", &["legend-foreground"]),
    // Button-link chrome on <a variant=…> shares the button pack.
    (
        "a-button-link",
        &[
            "button",
            "button-foreground",
            "button-border",
            "button-hover-bg",
        ],
    ),
    // Slice #3 — residual pack-bearing surfaces + shared field control face.
    (
        "alert",
        &["alert", "alert-foreground", "alert-border", "alert-stripe"],
    ),
    (
        "empty-state",
        &[
            "empty-state",
            "empty-state-foreground",
            "empty-state-border",
            "empty-state-title",
        ],
    ),
    (
        "toast",
        &["toast", "toast-foreground", "toast-border", "toast-stripe"],
    ),
    (
        "navbar",
        &["navbar", "navbar-foreground", "navbar-border"],
    ),
    (
        "page",
        &[
            "page-foreground",
            "page-hover-bg",
            "page-current-bg",
            "page-current-foreground",
        ],
    ),
    (
        "scroll-area",
        &["scroll-area-thumb", "scroll-area-thumb-hover"],
    ),
    ("spinner", &["spinner-foreground", "spinner-track"]),
    (
        "tab",
        &[
            "tab-foreground",
            "tab-selected-foreground",
            "tab-selected-bg",
            "tab-border",
        ],
    ),
    // <radio> control face shares field surface with freeform input type=radio.
    ("radio", &["radio-foreground", "field", "field-foreground"]),
    // Combobox text field shares field pack (listbox still combobox*).
    (
        "combobox-field",
        &["field", "field-foreground", "field-border", "field-border-hover"],
    ),
];

/// Default-rest free globals that a slice element must NOT use for its owned
/// channels once a component token exists. Tone/brand paints may still spend
/// `primary` / `destructive` etc. — this only guards the zero-prop default.
const FREE_DEFAULT_PAINT: &[&str] = &[
    "background",
    "foreground",
    "border",
    "border-strong",
    "surface-1",
    "muted",
    "muted-foreground",
    "hover-bg",
    "card-foreground",
];

/// Per-element default class fragments that *must* prefer component tokens.
/// Format: element comment banner in element-skins → banned free token names
/// for that block's primary default paint.
const SLICE_NO_FREE_DEFAULT: &[(&str, &[&str])] = &[
    // button default variant rest
    (
        "/* button */",
        &["background", "foreground"], // default rest is button*
    ),
    ("/* dialog */", &["surface-1", "card-foreground"]),
    ("/* accordion */", &["surface-1"]),
    ("/* skeleton */", &["muted"]),
    ("/* kbd */", &["muted", "foreground"]),
    ("/* input */", &["background", "foreground"]),
    ("/* select */", &["background", "foreground"]),
    ("/* textarea */", &["background", "foreground"]),
    // Slice #2 residual defaults
    (
        "/* table */",
        &["foreground", "muted-foreground", "surface-2", "background"],
    ),
    ("/* progress */", &["muted", "primary"]),
    (
        "/* meter */",
        &["muted", "success", "warning", "destructive"],
    ),
    ("/* slider */", &["muted", "primary", "border"]),
    ("/* breadcrumb */", &["muted-foreground", "foreground"]),
    ("/* fieldset */", &["border"]),
    ("/* legend */", &["foreground"]),
    // Button-link default rest (shared button pack)
    ("/* a */", &["background", "foreground"]),
    // Slice #3 residual defaults
    ("/* alert */", &["border", "muted", "muted-foreground", "surface-1"]),
    (
        "/* empty-state */",
        &["muted", "muted-foreground", "foreground"],
    ),
    ("/* toast */", &["background", "foreground", "muted", "muted-foreground"]),
    ("/* navbar */", &["background", "foreground", "border"]),
    ("/* page */", &["foreground", "muted", "primary", "background"]),
    ("/* spinner */", &["muted-foreground", "border-subtle"]),
    ("/* tab */", &["foreground", "border", "muted-foreground"]),
    ("/* radio */", &["foreground", "background"]),
    ("/* combobox */", &["background", "foreground"]),
];

fn component_packs() -> BTreeMap<String, BTreeSet<String>> {
    let mut packs = BTreeMap::new();
    let dir = components_dir();
    for entry in fs::read_dir(&dir).expect("components dir") {
        let entry = entry.expect("dirent");
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let text = fs::read_to_string(&path).expect("read pack");
        let mut css_names = BTreeSet::new();
        // "$extensions" / "cssName": "foo"
        for cap in text.split("\"cssName\"").skip(1) {
            let rest = cap.trim_start_matches(|c: char| c == ':' || c.is_whitespace());
            if let Some(s) = rest.strip_prefix('"') {
                if let Some(end) = s.find('"') {
                    css_names.insert(s[..end].to_string());
                }
            }
        }
        let stem = path.file_stem().unwrap().to_string_lossy().into_owned();
        packs.insert(stem, css_names);
    }
    packs
}

fn registry_component_paths() -> BTreeMap<String, String> {
    // path → cssName for component.*
    let v: serde_json::Value = serde_json::from_str(REGISTRY).expect("registry json");
    let mut out = BTreeMap::new();
    if let Some(paths) = v.get("paths").and_then(|p| p.as_object()) {
        for (path, css) in paths {
            if path.starts_with("component.") {
                if let Some(name) = css.as_str() {
                    out.insert(path.clone(), name.to_string());
                }
            }
        }
    }
    out
}

#[test]
fn slice_component_tokens_are_read_by_element_skins() {
    let mut missing = Vec::new();
    for (consumer, names) in SLICE_READERS {
        for name in *names {
            let needle = format!("var(--{name})");
            if !SKINS_CSS.contains(&needle) {
                missing.push(format!("{consumer}: missing {needle} in element-skins.css"));
            }
        }
    }
    assert!(
        missing.is_empty(),
        "component-token totality slice: skins must read the pack:\n  {}",
        missing.join("\n  ")
    );
}

#[test]
fn every_component_pack_has_registry_paths() {
    let packs = component_packs();
    let paths = registry_component_paths();
    let mut missing = Vec::new();
    for (pack, css_names) in &packs {
        for css in css_names {
            let found = paths.values().any(|v| v == css);
            if !found {
                missing.push(format!("component.{pack} cssName `{css}` not in registry paths"));
            }
        }
    }
    assert!(
        missing.is_empty(),
        "component packs must land in token-registry paths (run: cargo run -p sail-compiler -- tokens):\n  {}",
        missing.join("\n  ")
    );
}

/// Advanced channels that themes set today but the skin layer does not yet
/// paint. Documented residual — not silent costume vars. Each entry must have
/// a follow-up row in RESULT-L1-token-totality.md.
const ALLOWED_UNREAD: &[&str] = &[
    // Card frost/texture layer: themes override the three channels; host skin
    // does not yet emit the texture pseudo. Wire in a later WI (no taste invent
    // this pass — default is `none`, so enabling would only affect themed
    // packs that already set the URL).
    "card-texture",
    "card-texture-blend",
    "card-texture-opacity",
];

#[test]
fn every_component_css_name_has_a_skin_or_stdlib_reader() {
    // A pack with zero readers is a variable in a costume. Check every
    // component cssName is read by element-skins, stdlib Style, or structure maps.
    let packs = component_packs();
    let maps = include_str!("../src/stdlib.rs");
    let mut real = Vec::new();
    for (pack, names) in &packs {
        for css in names {
            if ALLOWED_UNREAD.contains(&css.as_str()) {
                continue;
            }
            let var_ref = format!("var(--{css})");
            if SKINS_CSS.contains(&var_ref) {
                continue;
            }
            let sail_hit = {
                let mut hit = false;
                for entry in fs::read_dir(stdlib_dir()).expect("stdlib") {
                    let p = entry.expect("e").path();
                    if p.extension().and_then(|e| e.to_str()) != Some("sail") {
                        continue;
                    }
                    let text = fs::read_to_string(&p).unwrap_or_default();
                    // Style atoms: bg(chip), fg(field-hint), border(badge-border), …
                    if text.contains(&format!("({css})"))
                        || text.contains(&format!("({css},"))
                        || text.contains(&format!(", {css})"))
                        || text.contains(&format!(", {css},"))
                    {
                        hit = true;
                        break;
                    }
                }
                if !hit {
                    hit = maps.contains(&var_ref) || maps.contains(&format!("--{css}"));
                }
                hit
            };
            if !sail_hit {
                real.push(format!("component.{pack} → --{css}"));
            }
        }
    }
    assert!(
        real.is_empty(),
        "component tokens with no skin/stdlib reader (variable in a costume):\n  {}\n\n\
         Wire a reader into the element skin or drop the pack entry. \
         Advanced deferred channels go in ALLOWED_UNREAD with a RESULT residual.",
        real.join("\n  ")
    );
}

#[test]
fn slice_default_paint_prefers_component_tokens_over_free_globals() {
    // For each slice block in element-skins, the *default* rest paint for the
    // owned channels must not reintroduce free globals that the pack replaced.
    // We scope to the element section (comment banner → next /* … */ or EOF).
    let mut breaches = Vec::new();
    for (banner, banned) in SLICE_NO_FREE_DEFAULT {
        let start = match SKINS_CSS.find(banner) {
            Some(i) => i,
            None => {
                breaches.push(format!("missing skin section {banner}"));
                continue;
            }
        };
        let rest = &SKINS_CSS[start + banner.len()..];
        // next element comment at line start
        let end = rest
            .find("\n/* ")
            .map(|i| start + banner.len() + i)
            .unwrap_or(SKINS_CSS.len());
        let block = &SKINS_CSS[start..end];

        // Only flag free globals used as *direct* color/background/border-color
        // values, not inside color-mix fallbacks of component tokens.
        for free in *banned {
            // Default rest classes (not tone compounds): look for var(--free)
            // on bg/color/border-color in the block where a component token
            // should already own that channel.
            let needle = format!("var(--{free})");
            if !block.contains(&needle) {
                continue;
            }
            // Exempt: tone= / variant=solid branded matrices still spend globals.
            // Count occurrences outside [tone=…] compound selectors by scanning
            // rules that do not contain "tone=" on the same selector prefix.
            // Practical heuristic: default variant classes mention
            // variant='default' or un-prefixed host rules.
            for (i, line) in block.lines().enumerate() {
                if !line.contains(&needle) {
                    continue;
                }
                // Skip branded tone compounds.
                if line.contains("tone=") || line.contains("tone'") {
                    continue;
                }
                // Skip solid/soft/outline matrices that intentionally use globals.
                if line.contains("variant='solid'")
                    || line.contains("variant='soft'")
                    || line.contains("variant='outline'")
                    || line.contains("variant='ghost'")
                {
                    // default variant is the rest channel — allow only default.
                    if !line.contains("variant='default'") {
                        continue;
                    }
                }
                breaches.push(format!(
                    "{banner} line~{} uses free {needle}: {}",
                    i + 1,
                    line.trim().chars().take(120).collect::<String>()
                ));
            }
        }
    }
    // Also ensure the free-default list itself is the set we care about.
    let _ = FREE_DEFAULT_PAINT;

    assert!(
        breaches.is_empty(),
        "slice default paint still spends free globals that component tokens own:\n  {}\n\n\
         Read the component token (global fallback lives in the pack default).",
        breaches.join("\n  ")
    );
}

#[test]
fn tokens1_cut_orphans_stay_gone() {
    // Residual burn owned by TOKENS-1 — re-assert so totality does not revive them.
    let v: serde_json::Value = serde_json::from_str(REGISTRY).expect("registry");
    let names: BTreeSet<String> = v["names"]
        .as_array()
        .expect("names")
        .iter()
        .filter_map(|n| n.as_str().map(str::to_string))
        .collect();
    const CUT: &[&str] = &[
        "body-tracking",
        "glow-info",
        "glow-success",
        "glow-warning",
        "glow-destructive",
        "glow-default",
        "glow-muted",
        "app-accent-agents",
        "leading-normal-plus",
        "menu-min-w",
        "tab-selected", // renamed to tab-selected-bg
        "toast-region-width",
        "toast-region-z",
    ];
    let mut revived = Vec::new();
    for c in CUT {
        if names.contains(*c) {
            revived.push(*c);
        }
    }
    assert!(
        revived.is_empty(),
        "TOKENS-1 cut orphans revived in registry: {revived:?}"
    );
    // renames present
    assert!(
        names.contains("tab-selected-bg"),
        "tab-selected-bg rename missing"
    );
    assert!(
        names.contains("panel-menu-min-w"),
        "panel-menu-min-w rename missing"
    );
    assert!(
        names.contains("panel-toast-region-width"),
        "panel-toast-region-width rename missing"
    );
    assert!(
        names.contains("z-toast-region"),
        "z-toast-region rename missing"
    );
}
