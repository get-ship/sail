//! First-party concern-form pin. Hard-fail (no warn-only).
//!
//! Stdlib must author SPEC-SPINNAKER-2 concern form. Absorbed property
//! setter names are a compile-process fail, not a warning.

use sail_compiler::compile_element;
use spinnaker::{StyleDiagnostic, parse};
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

fn workspace_root() -> PathBuf {
    compiler_root()
        .join("../../..")
        .canonicalize()
        .expect("workspace root")
}

fn skip_dir(name: &str) -> bool {
    matches!(
        name,
        "node_modules"
            | "dist"
            | "target"
            | ".deck"
            | ".git"
            | "test-results"
            | "content"
            | "tutorial"
            | "docs"
            | "e2e"
            | "fixtures"
            | "test"
    )
}

fn collect_sail_skipping(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|e| panic!("read_dir {}: {e}", dir.display()));
    for entry in entries {
        let entry = entry.expect("dirent");
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if skip_dir(name) {
                continue;
            }
            collect_sail_skipping(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("sail") {
            out.push(path);
        }
    }
}

fn first_party_trees() -> Vec<(&'static str, PathBuf)> {
    let root = workspace_root();
    [
        ("stdlib", compiler_root().join("stdlib")),
        // Studio is still v2 Style (component lane). Do not walk it as product.
        ("sail-site", root.join("apps/sail-site")),
        ("packages/components", root.join("packages/components")),
        ("packages/sail", root.join("packages/sail")),
    ]
    .into_iter()
    .collect()
}

fn first_party_sail_files() -> Vec<(String, PathBuf)> {
    let mut out = Vec::new();
    for (label, dir) in first_party_trees() {
        let mut files = Vec::new();
        collect_sail_skipping(&dir, &mut files);
        for path in files {
            out.push((label.to_string(), path));
        }
    }
    out.sort_by(|a, b| a.1.cmp(&b.1));
    out
}

fn element_tag(path: &Path) -> String {
    let stem = path.file_stem().unwrap().to_string_lossy();
    if stem.as_ref() == "anchor" {
        "a".into()
    } else {
        stem.into()
    }
}

/// Retired longhands + dead v2 author names. Live v3 (`bg`/`fg`/`pad`/
/// `position`/`display`/`grow`/…) is the language.
const ABSORBED: &[&str] = &[
    "fill",
    "ink",
    "place",
    "space",
    "flow",
    "motion",
    "bgImage",
    "bgClip",
    "bgGlow",
    "colorMix",
    "colorDeepen",
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
    "w",
    "h",
    "minW",
    "minH",
    "maxW",
    "maxH",
    "aspect",
    "basis",
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
    "inset",
    "top",
    "bottom",
    "left",
    "right",
    "z",
    "order",
    "flexDir",
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
    "perspective",
    "gap",
    "gapX",
    "gapY",
];

fn skip_string(src: &str, i: usize) -> Option<usize> {
    let b = src.as_bytes();
    if i >= b.len() {
        return None;
    }
    let q = b[i];
    if q != b'"' && q != b'\'' {
        return None;
    }
    // Prose apostrophes (`Don't`) are not string opens. `'` only starts a
    // string in attr/arg position (`=`, `(`, …).
    if q == b'\'' && !apostrophe_opens_string(b, i) {
        return None;
    }
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

fn apostrophe_opens_string(b: &[u8], i: usize) -> bool {
    let mut j = i;
    while j > 0 {
        j -= 1;
        if b[j].is_ascii_whitespace() {
            continue;
        }
        return matches!(b[j], b'=' | b'(' | b'[' | b',' | b':' | b'{');
    }
    false
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

fn skip_raw_block(src: &str, i: usize) -> Option<usize> {
    for tag in ["controller", "script"] {
        let open = format!("<{tag}");
        if !src[i..].starts_with(&open) {
            continue;
        }
        let after = i + open.len();
        let bytes = src.as_bytes();
        if after < bytes.len() {
            let c = bytes[after];
            if !(c.is_ascii_whitespace() || c == b'>' || c == b'/') {
                continue;
            }
        }
        let close = format!("</{tag}>");
        return src[i..].find(&close).map(|rel| i + rel + close.len());
    }
    None
}

/// Call sites of absorbed setter names. Skips comments/strings/controller.
/// Not a substring scan. JS methods in `<controller>` (`.list()`, `.transform()`)
/// are not Style.
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
        if let Some(end) = skip_line_comment(src, i)
            .or_else(|| skip_string(src, i))
            .or_else(|| skip_raw_block(src, i))
        {
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
            // Style calls are `name(` with no space. Teaching prose
            // (`inset (number)`, `fg (theme override)`) is not a setter.
            if i < bytes.len() && bytes[i] == b'(' && ABSORBED.contains(&name) {
                // `{position()}` / `duration={duration()}` are bindings, not Style.
                if start > 0 && matches!(bytes[start - 1], b'{' | b'=') {
                    i += ch_len;
                    continue;
                }
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
fn stdlib_has_no_absorbed_setters() {
    first_party_has_no_absorbed_setters_in("stdlib");
}

#[test]
fn first_party_has_no_absorbed_setters() {
    let files = first_party_sail_files();
    assert!(
        files.len() > 60,
        "first-party trees missing, got {}",
        files.len()
    );
    let mut failures = Vec::new();
    for (_label, path) in &files {
        let src = fs::read_to_string(path).unwrap();
        let sites = absorbed_setter_sites(&src);
        if !sites.is_empty() {
            failures.push(format!("{}:\n  {}", path.display(), sites.join("\n  ")));
        }
    }
    assert!(
        failures.is_empty(),
        "first-party still authors absorbed setters (hard fail):\n{}",
        failures.join("\n")
    );
}

fn first_party_has_no_absorbed_setters_in(tree: &str) {
    let files: Vec<_> = first_party_sail_files()
        .into_iter()
        .filter(|(label, _)| label == tree)
        .collect();
    assert!(!files.is_empty(), "{tree} dir missing");
    let mut failures = Vec::new();
    for (_label, path) in &files {
        let src = fs::read_to_string(path).unwrap();
        let sites = absorbed_setter_sites(&src);
        if !sites.is_empty() {
            failures.push(format!("{}:\n  {}", path.display(), sites.join("\n  ")));
        }
    }
    assert!(
        failures.is_empty(),
        "{tree} still authors absorbed setters (hard fail):\n{}",
        failures.join("\n")
    );
}

#[test]
fn first_party_has_no_sx_attribute() {
    let files = first_party_sail_files();
    let mut failures = Vec::new();
    for (_label, path) in &files {
        let src = fs::read_to_string(path).unwrap();
        if let Some(hit) = sx_attr_site(&src) {
            failures.push(format!("{}: {hit}", path.display()));
        }
    }
    assert!(
        failures.is_empty(),
        "first-party still authors sx= (hard fail):\n{}",
        failures.join("\n")
    );
}

/// `sx=` product attr. Skips comments/strings. Teaching prose mentioning `sx=` is fine.
fn sx_attr_site(src: &str) -> Option<String> {
    let bytes = src.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if !src.is_char_boundary(i) {
            i += 1;
            continue;
        }
        if let Some(end) = skip_line_comment(src, i).or_else(|| skip_string(src, i)) {
            i = end;
            continue;
        }
        if src[i..].starts_with("sx=") {
            let prev_ok = i == 0
                || matches!(
                    bytes[i - 1],
                    b' ' | b'\t' | b'\n' | b'\r' | b'/' | b'"' | b'\''
                );
            if prev_ok {
                let snippet: String = src[i..].chars().take(32).collect();
                return Some(snippet);
            }
        }
        i += src[i..].chars().next().map(|c| c.len_utf8()).unwrap_or(1);
    }
    None
}

#[test]
fn absorbed_scanner_exemption_edges() {
    let cases: &[(&str, bool)] = &[
        ("inset (number)", false),
        ("inset(0)", true),
        ("{position()}", false),
        ("duration={duration()}", false),
        ("Don't call padX(3)", true),
        ("<p>Sticky top/left inset (number).</p>", false),
    ];
    for (src, want_hit) in cases {
        let sites = absorbed_setter_sites(src);
        assert_eq!(!sites.is_empty(), *want_hit, "{src}: sites={sites:?}");
    }
}

#[test]
fn sx_scanner_exemption_edges() {
    assert!(
        sx_attr_site(r#"<code>sx="…"</code>"#).is_none(),
        "teaching sx= in <code> is not a product attr"
    );
    assert!(
        sx_attr_site(r#"<div sx={style({ bg({ color: primary }) })}>x</div>"#).is_some(),
        "product sx= must hit"
    );
    assert!(
        sx_attr_site(r#"<div sx="p-4">x</div>"#).is_some(),
        "quoted product sx= must hit"
    );
}

#[test]
fn stdlib_all_files_compile() {
    let files = stdlib_sail_files();
    assert_eq!(
        files.len(),
        61,
        "expected 61 stdlib .sail files, got {}",
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
    let path = compiler_root().join("stdlib/dialog/dialog.sail");
    let src = fs::read_to_string(&path).expect("dialog.sail");
    assert!(
        !src.contains("border(dialog.border)") && !src.contains("border(dialog)"),
        "dialog.sail spends global border, not a deleted pack"
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
        "style({ border(dialog).radius(xl) })"
    ));
    assert!(!authors_border_dialog_concern(
        "style({ border(dialog.border).radius(xl) })"
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
    let src = r#"<div style({ border(dialog.border) })>x</div>"#;
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
    let path = compiler_root().join("stdlib/card/card.sail");
    let src = fs::read_to_string(&path).expect("card.sail");
    assert!(
        src.contains("border({ color: border, width: 1 })"),
        "card.sail must author the slot-object fold, not border(1, border)"
    );
    compile_element(&src, "card").expect("card.sail must still compile");
}

#[test]
fn property_setters_do_not_parse() {
    let src = "style({ borderWidth(1).border(primary) })";
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
    let select = fs::read_to_string(compiler_root().join("stdlib/select/select.sail")).unwrap();
    assert!(
        select.contains("s:") && select.contains("e:"),
        "select must keep pad s/e (chevron room)"
    );
    let sizes = select
        .lines()
        .filter(|l| l.contains("style size") || l.contains("pad("))
        .collect::<Vec<_>>();
    for line in &sizes {
        if line.contains("sizeX")
            || line.contains("sizeS")
            || line.contains("sizeM")
            || line.contains("sizeL")
        {
            assert!(
                !line.contains("x:") || line.contains("s:"),
                "select size ladder must not use pad x in place of s: {line}"
            );
        }
    }
    let input = fs::read_to_string(compiler_root().join("stdlib/input/input.sail")).unwrap();
    assert!(
        input.contains("s:") && input.contains("e:"),
        "password field must keep pad s/e"
    );
    assert!(
        input.contains("sizeXsPw") && input.contains("s:"),
        "password size ladder must use pad s"
    );
}

#[test]
fn tooltip_carets_are_per_edge_slot_objects() {
    let src = fs::read_to_string(compiler_root().join("stdlib/tooltip/tooltip.sail")).unwrap();
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
