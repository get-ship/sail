//! First-party concern-form pin. Hard-fail (no warn-only).
//!
//! Stdlib must author SPEC-SPINNAKER-2 concern form. Absorbed property
//! setter names are a compile-process fail, not a warning.

use sail_compiler::compile_element;
use sail_lint::style::{parse, retired_dual_border_sites, rewrite_concern_source, StyleDiagnostic};
use std::fs;
use std::path::{Path, PathBuf};

fn compiler_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn collect_sail(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|e| panic!("read_dir {}: {e}", dir.display()));
    for entry in entries {
        let entry = entry.expect("dirent");
        let path = entry.path();
        if path.is_dir() {
            collect_sail(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("sail") {
            out.push(path);
        }
    }
}

fn stdlib_sail_files() -> Vec<PathBuf> {
    let dir = compiler_root().join("stdlib");
    let mut files = Vec::new();
    collect_sail(&dir, &mut files);
    files.sort();
    files
}

fn element_tag(path: &Path) -> String {
    let stem = path.file_stem().unwrap().to_string_lossy();
    if stem.as_ref() == "anchor" {
        "a".into()
    } else {
        stem.into()
    }
}

/// Absorbed property names (atlas "Absorbs" columns + gap). Concern names
/// themselves (`border`, `fill`, `size`, …) are the live language.
const ABSORBED: &[&str] = &[
    "bg",
    "bgImage",
    "bgClip",
    "bgGlow",
    "colorMix",
    "colorDeepen",
    "fg",
    "caret",
    "accent",
    "stroke",
    "textGradient",
    "borderWidth",
    "borderStyle",
    "borderX",
    "borderY",
    "borderT",
    "borderB",
    "borderL",
    "borderR",
    "borderS",
    "borderE",
    "borderTWidth",
    "borderBWidth",
    "borderLWidth",
    "borderRWidth",
    "borderSWidth",
    "borderEWidth",
    "outline",
    "outlineOffset",
    "ring",
    "borderImage",
    "radiusT",
    "radiusB",
    "radiusL",
    "radiusR",
    "shadow",
    "blur",
    "brightness",
    "contrast",
    "grayscale",
    "hueRotate",
    "invert",
    "sepia",
    "saturate",
    "backdrop",
    "mixBlend",
    "opacity",
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "aspect",
    "basis",
    "pad",
    "padX",
    "padY",
    "padT",
    "padB",
    "padL",
    "padR",
    "padS",
    "padE",
    "m",
    "mx",
    "my",
    "mt",
    "mb",
    "ml",
    "mr",
    "ms",
    "me",
    "position",
    "inset",
    "top",
    "bottom",
    "left",
    "right",
    "z",
    "order",
    "display",
    "flexDir",
    "grow",
    "shrink",
    "self",
    "justifySelf",
    "overflowX",
    "overflowY",
    "overscroll",
    "scroll",
    "resize",
    "contain",
    "willChange",
    "visibility",
    "fontFamily",
    "fontSize",
    "fontWeight",
    "leading",
    "tracking",
    "textAlign",
    "textCase",
    "whitespace",
    "break",
    "truncate",
    "italic",
    "nums",
    "decoration",
    "underline",
    "underlineOffset",
    "indent",
    "antialiased",
    "list",
    "rotate",
    "rotateX",
    "rotateY",
    "rotateZ",
    "scale",
    "translateX",
    "translateY",
    "transform",
    "origin",
    "perspective",
    "transition",
    "duration",
    "ease",
    "animate",
    "gap",
    "gapX",
    "gapY",
];

fn skip_string(src: &str, i: usize) -> Option<usize> {
    let b = src.as_bytes();
    if i >= b.len() || (b[i] != b'\'' && b[i] != b'"') {
        return None;
    }
    let q = b[i];
    let mut j = i + 1;
    while j < b.len() {
        if b[j] == b'\\' && j + 1 < b.len() {
            j += 2;
            continue;
        }
        if b[j] == q {
            return Some(j + 1);
        }
        j += 1;
    }
    Some(j)
}

fn skip_line_comment(src: &str, i: usize) -> Option<usize> {
    if src[i..].starts_with("//") {
        return Some(src[i..].find('\n').map(|rel| i + rel).unwrap_or(src.len()));
    }
    if src[i..].starts_with("/*") {
        return Some(
            src[i + 2..]
                .find("*/")
                .map(|rel| i + 2 + rel + 2)
                .unwrap_or(src.len()),
        );
    }
    if src[i..].starts_with("<!--") {
        return Some(
            src[i + 4..]
                .find("-->")
                .map(|rel| i + 4 + rel + 3)
                .unwrap_or(src.len()),
        );
    }
    None
}

/// Call sites of absorbed setter names. Skips comments/strings. Not a substring scan.
fn absorbed_setter_sites(src: &str) -> Vec<String> {
    let bytes = src.as_bytes();
    let mut sites = Vec::new();
    let mut i = 0usize;
    while i < bytes.len() {
        if !src.is_char_boundary(i) {
            i += 1;
            continue;
        }
        let ch_len = src[i..].chars().next().map(|c| c.len_utf8()).unwrap_or(1);
        if let Some(end) = skip_line_comment(src, i).or_else(|| skip_string(src, i)) {
            i = end;
            continue;
        }
        if bytes[i].is_ascii_alphabetic() || bytes[i] == b'_' {
            if i > 0 && (bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'_') {
                i += ch_len;
                continue;
            }
            let start = i;
            i += 1;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            let name = &src[start..i];
            let mut j = i;
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b'(' && ABSORBED.contains(&name) {
                let snippet: String = src[start..].chars().take(48).collect();
                sites.push(snippet);
            }
            continue;
        }
        i += ch_len;
    }
    sites
}

#[test]
fn rewrite_dialog_chain_to_concern_name() {
    let got = rewrite_concern_source("style({ paint: borderWidth(1).border(dialog.border) })");
    assert_eq!(got, "style({ paint: border(dialog) })", "{got}");
}

#[test]
fn rewrite_does_not_touch_radius() {
    let got = rewrite_concern_source("style({ paint: borderWidth(1).border(primary).radius(md) })");
    assert_eq!(
        got, "style({ paint: border({ color: primary, width: 1 }).radius(md) })",
        "{got}"
    );
}

#[test]
fn dual_detector_hard_fails_on_retired_chain() {
    let sites = retired_dual_border_sites("style({ paint: borderWidth(1).border(primary) })");
    assert!(
        !sites.is_empty(),
        "detector must see borderWidth next to border("
    );
}

#[test]
fn stdlib_has_no_absorbed_setters() {
    let files = stdlib_sail_files();
    assert!(!files.is_empty(), "stdlib dir missing");
    let mut failures = Vec::new();
    for path in &files {
        let src = fs::read_to_string(path).unwrap();
        let sites = absorbed_setter_sites(&src);
        if !sites.is_empty() {
            failures.push(format!("{}:\n  {}", path.display(), sites.join("\n  ")));
        }
    }
    assert!(
        failures.is_empty(),
        "stdlib still authors absorbed setters (hard fail):\n{}",
        failures.join("\n")
    );
}

#[test]
fn stdlib_all_files_compile() {
    let files = stdlib_sail_files();
    assert_eq!(
        files.len(),
        60,
        "expected 60 stdlib .sail files, got {}",
        files.len()
    );
    let mut failed = Vec::new();
    for path in &files {
        let src = fs::read_to_string(path).unwrap();
        let tag = element_tag(path);
        if let Err(e) = compile_element(&src, &tag) {
            failed.push(format!("{tag}: {}", e.message));
        }
    }
    assert!(
        failed.is_empty(),
        "{} stdlib files failed to compile:\n  {}",
        failed.len(),
        failed.join("\n  ")
    );
}

#[test]
fn dialog_sail_authors_concern_form_and_compiles() {
    let path = compiler_root().join("stdlib/dialog.sail");
    let src = fs::read_to_string(&path).expect("dialog.sail");
    assert!(
        authors_border_dialog_concern(&src),
        "dialog.sail must author border(dialog), not border(dialog.border)"
    );
    assert!(
        !src.contains("borderWidth"),
        "dialog.sail must not author borderWidth"
    );
    compile_element(&src, "dialog").expect("dialog.sail must still compile");
}

#[test]
fn dialog_concern_pin_rejects_dotted_color_slot() {
    assert!(authors_border_dialog_concern(
        "style({ paint: border(dialog).radius(xl) })"
    ));
    assert!(!authors_border_dialog_concern(
        "style({ paint: border(dialog.border).radius(xl) })"
    ));
}

fn authors_border_dialog_concern(src: &str) -> bool {
    let prefix = "border(dialog";
    let bytes = src.as_bytes();
    let mut from = 0usize;
    while let Some(rel) = src[from..].find(prefix) {
        let after = from + rel + prefix.len();
        if bytes.get(after) == Some(&b')') {
            return true;
        }
        from = after;
    }
    false
}

#[test]
fn dialog_border_dotted_form_is_retired() {
    // Host is not dialog, so `dialog.border` is the retired color poke (not a
    // host-surface MemberPath).
    let src = r#"<div style({ paint: border(dialog.border) })>x</div>"#;
    let err = compile_element(src, "card")
        .expect_err("border(dialog.border) is the retired color poke — use border(dialog)");
    assert!(
        err.message.contains("border") || err.message.contains("retired"),
        "{}",
        err.message
    );
}

#[test]
fn card_sail_compiles_after_two_arg_fold() {
    let path = compiler_root().join("stdlib/card.sail");
    let src = fs::read_to_string(&path).expect("card.sail");
    assert!(
        src.contains("border({ color: border, width: 1 })"),
        "card.sail must author the slot-object fold, not border(1, border)"
    );
    compile_element(&src, "card").expect("card.sail must still compile");
}

#[test]
fn property_setters_do_not_parse() {
    let src = "style({ paint: borderWidth(1).border(primary) })";
    let (_style, diags) = parse(src);
    assert!(
        diags
            .iter()
            .any(|d| matches!(d, StyleDiagnostic::RetiredSetter { .. })),
        "old property setters must RetiredSetter, not parse: {diags:?}"
    );
}

#[test]
fn select_password_keep_pad_s_not_pad_x() {
    let select = fs::read_to_string(compiler_root().join("stdlib/select.sail")).unwrap();
    assert!(
        select.contains("pad-s:") && select.contains("pad-e:"),
        "select must keep pad-s/pad-e (chevron room)"
    );
    let sizes = select
        .lines()
        .filter(|l| l.contains("style size") || l.contains("pad-"))
        .collect::<Vec<_>>();
    for line in &sizes {
        if line.contains("sizeX")
            || line.contains("sizeS")
            || line.contains("sizeM")
            || line.contains("sizeL")
        {
            assert!(
                !line.contains("pad-x:"),
                "select size ladder must not use pad-x: {line}"
            );
        }
    }
    let input = fs::read_to_string(compiler_root().join("stdlib/input.sail")).unwrap();
    assert!(
        input.contains("pad-s:") && input.contains("pad-e:"),
        "password field must keep pad-s/pad-e"
    );
    assert!(
        input.contains("sizeXsPw") && input.contains("pad-s:"),
        "password size ladder must use pad-s"
    );
}

#[test]
fn tooltip_carets_are_per_edge_slot_objects() {
    let src = fs::read_to_string(compiler_root().join("stdlib/tooltip.sail")).unwrap();
    assert!(
        src.contains("edge: t")
            && src.contains("edge: b")
            && src.contains("edge: l")
            && src.contains("edge: r"),
        "tooltip carets must be per-edge slot objects"
    );
    assert!(
        !src.contains("border(tooltip)"),
        "tooltip carets must not smash into border(tooltip)"
    );
}
