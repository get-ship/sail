//! `sail-font-literal` — anti-smell #5 from `ship-design-system`.
//!
//! "Never hardcode 'Inter' or 'Syne' in a component. Read the role token
//!  (`--heading-font`, `--body-font`, `--label-font`, `--mono-font`)."
//!
//! A theme like `paper` demotes display typography by remapping
//! `--heading-font: var(--font-body)`. A component that hand-writes
//! `font-family: "Syne", sans-serif;` defeats that remap: the literal
//! name bypasses the role token, so the demoted theme keeps rendering
//! Syne for that one component. The fix is the matching role token —
//! the compiler has no way to enforce this from the CSS itself.
//!
//! What this rule flags:
//! - A `font-family: …;` declaration whose value contains a quoted (or
//!   bare) font name from the canonical palette (Inter / Syne / DM Sans /
//!   JetBrains Mono / Space Grotesk / Space Mono / IBM Plex).
//!
//! What it deliberately does NOT flag:
//! - A `var(--…)` reference. The whole point of the role token is that
//!   a `var(--font-display-app)` or `var(--heading-font)` use defers to
//!   the theme.
//! - A `--font-…: …;` (or any `--…: …;`) **declaration**. These are
//!   token definitions — they're where the literal IS supposed to live,
//!   defining the slot other components read from. The exemption uses
//!   the same `collect_local_definitions` shape as the sibling
//!   `sail-css-token` rule.
//! - CSS comments (handled by the declaration walker).
//!
//! Severity: `Warn` (advisory). This is a "prefer" rule, not a
//! correctness gate.
use std::collections::HashSet;
use std::path::Path;

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sail-font-literal";

const SEVERITY: Severity = Severity::Warn;

/// Canonical palette. A whole-token (case-insensitive) match of any of
/// these inside a `font-family` value fires. Substring matches inside
/// other identifiers (e.g. `Inter` inside `Internal`) are not flagged —
/// the boundary check requires non-identifier bytes on both sides.
///
/// Decision tree (which role token to suggest):
/// - Serif/display family → `var(--heading-font)`
/// - Mono family          → `var(--mono-font)`
/// - Otherwise (sans)     → `var(--body-font)`
const PALETTE: &[(&str, &str)] = &[
    // Display / serif.
    ("Syne", "--heading-font"),
    ("Syne Variable", "--heading-font"),
    ("Space Grotesk", "--heading-font"),
    ("IBM Plex Serif", "--heading-font"),
    // Mono.
    ("JetBrains Mono", "--mono-font"),
    ("JetBrains Mono Variable", "--mono-font"),
    ("Space Mono", "--mono-font"),
    ("IBM Plex Mono", "--mono-font"),
    // Sans (body).
    ("Inter", "--body-font"),
    ("Inter Variable", "--body-font"),
    ("DM Sans", "--body-font"),
    ("DM Sans Variable", "--body-font"),
    ("IBM Plex Sans", "--body-font"),
];

/// Shared fix hint template. `{family}` and `{role}` are substituted at
/// the firing site.
fn suggest_text(family: &str, role: &str) -> String {
    format!(
        "Hardcoded font family `{}` defeats themes that remap \
         `--heading-font` / `--body-font` / `--mono-font` (e.g. `paper` \
         demotes display to body). Replace with `var({})`.",
        family, role
    )
}

/// Role tokens `fontFamily` accepts. Quoted strings / family names are red.
const FONT_FAMILY_ROLES: &[&str] = &["body", "heading", "mono", "display"];

/// Scan Style `fontFamily(...)` calls. A quoted string or non-role ident
/// is a product-UI family leak — authors bind roles, not faces.
fn check_style_font_family(path: &Path, source: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    let bytes = source.as_bytes();
    let needle = b"fontFamily";
    let mut i = 0;
    while i + needle.len() < bytes.len() {
        if &bytes[i..i + needle.len()] != needle {
            i += 1;
            continue;
        }
        let after = i + needle.len();
        // word boundary: not part of a longer ident
        if i > 0 && is_ident_byte(bytes[i - 1]) {
            i = after;
            continue;
        }
        let mut k = after;
        while k < bytes.len() && bytes[k].is_ascii_whitespace() {
            k += 1;
        }
        if k >= bytes.len() || bytes[k] != b'(' {
            i = after;
            continue;
        }
        k += 1;
        while k < bytes.len() && bytes[k].is_ascii_whitespace() {
            k += 1;
        }
        if k >= bytes.len() {
            break;
        }
        let arg_start = k;
        let arg = if bytes[k] == b'"' || bytes[k] == b'\'' {
            let q = bytes[k];
            k += 1;
            while k < bytes.len() && bytes[k] != q {
                k += 1;
            }
            if k < bytes.len() {
                k += 1;
            }
            &source[arg_start..k]
        } else {
            let start = k;
            while k < bytes.len()
                && (is_ident_byte(bytes[k]) || bytes[k] == b':' || bytes[k] == b'/')
            {
                k += 1;
            }
            &source[start..k]
        };
        let trimmed = arg.trim();
        if trimmed.is_empty() || trimmed.starts_with("bind") {
            i = k;
            continue;
        }
        let inner = trimmed.trim_matches(|c| c == '"' || c == '\'');
        let quoted = (trimmed.starts_with('"') && trimmed.ends_with('"'))
            || (trimmed.starts_with('\'') && trimmed.ends_with('\''));
        let is_role = FONT_FAMILY_ROLES.iter().any(|r| *r == inner);
        if quoted || !is_role {
            let (line, col) = byte_offset_to_line_col(source, arg_start);
            findings.push(Finding {
                file: path.to_path_buf(),
                line,
                col,
                rule: RULE_ID,
                severity: Severity::Error,
                message: format!(
                    "fontFamily accepts role tokens only (body, heading, mono, display), not `{trimmed}`"
                ),
                suggest: Some(
                    "Use fontFamily(body), fontFamily(heading), fontFamily(mono), or fontFamily(display). Bind faces in theme / sail.config fonts.roles."
                        .into(),
                ),
            });
        }
        i = k;
    }
    findings
}

/// Lint a `.sail` source. Walks Style `fontFamily(...)` plus every
/// `<style>...</style>` block; for every `font-family: …;` declaration,
/// looks for a whole-token occurrence of any palette name in the value.
pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(check_style_font_family(path, source));
    for block in scan_style_blocks(source) {
        let local = collect_local_definitions(block.content);
        for decl in scan_declarations(block.content) {
            if decl.property != "font-family" {
                continue;
            }
            // Declarations of the form `--font-…: …;` are token
            // definitions — the literal IS supposed to live there.
            // Same exemption shape as `sail-css-token`.
            if is_local_definition(decl.property, &local) {
                continue;
            }
            for (name, family, role) in scan_palette_hits(decl.value) {
                let abs = block.content_offset + decl.value_offset + name;
                let (line, col) = byte_offset_to_line_col(source, abs);
                findings.push(Finding {
                    file: path.to_path_buf(),
                    line,
                    col,
                    rule: RULE_ID,
                    severity: SEVERITY,
                    message: format!(
                        "hardcoded font family `{}` in `font-family` — use the role token instead",
                        family
                    ),
                    suggest: Some(suggest_text(family, role)),
                });
            }
        }
    }
    findings.sort_by(|a, b| a.line.cmp(&b.line).then_with(|| a.col.cmp(&b.col)));
    findings
}

/// True iff `property` names a custom property defined locally in the
/// same block. `--font-display-app: …;` is a definition; the
/// `font-family` value is exactly where the literal should live, and
/// linting it would just create noise for the person maintaining the
/// slot.
fn is_local_definition(property: &str, local: &HashSet<&str>) -> bool {
    if !property.starts_with("--") {
        return false;
    }
    local.contains(&property[2..])
}

/// Walk a `font-family` value, returning a hit for every whole-token
/// (case-insensitive) occurrence of a palette name. The hit is
/// `(name_offset_within_value, family_text, role_token)`.
fn scan_palette_hits(value: &str) -> Vec<(usize, &'static str, &'static str)> {
    let bytes = value.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        // Left anchor: start of buffer, or preceded by a non-identifier
        // byte. This keeps `Inter` from matching inside `Internal` or
        // any other identifier.
        if !(i == 0 || !is_ident_byte(bytes[i - 1])) {
            i += 1;
            continue;
        }
        let mut matched = false;
        for &(family, role) in palette_sorted() {
            let needle = family.as_bytes();
            if bytes.len() < i + needle.len() {
                continue;
            }
            if !bytes[i..i + needle.len()].eq_ignore_ascii_case(needle) {
                continue;
            }
            let right = i + needle.len();
            if right < bytes.len() && is_ident_byte(bytes[right]) {
                // The match bleeds into an identifier on the right
                // (e.g. `Inter` inside `Internal`).
                continue;
            }
            out.push((i, family, role));
            i = right;
            matched = true;
            break;
        }
        if !matched {
            i += 1;
        }
    }
    out
}

/// Palette sorted by length, longest first. Computed once per
/// process. Longest-first matching is what stops `Inter` from
/// eating the bytes a longer palette name would have matched
/// (e.g. `Inter Variable` inside a value that also lists `Inter`).
fn palette_sorted() -> &'static [(&'static str, &'static str)] {
    use std::sync::OnceLock;
    static CACHE: OnceLock<Vec<(&'static str, &'static str)>> = OnceLock::new();
    CACHE.get_or_init(|| {
        let mut v: Vec<_> = PALETTE.iter().copied().collect();
        v.sort_by(|a, b| b.0.len().cmp(&a.0.len()).then_with(|| a.0.cmp(b.0)));
        v
    })
}

/// True for CSS identifier bytes: ASCII alphanumeric plus `-` and `_`.
/// Used for boundary checks (left/right of a palette match).
fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'_'
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    let last = haystack.len() - needle.len();
    (0..=last).find(|&i| &haystack[i..i + needle.len()] == needle)
}

// ---------------------------------------------------------------------
// Style-block + declaration scanning. Mirrors the local convention of
// the sibling `css_token` / `css_literals` / `layout_primitive` rules
// (each rule owns its position-mapping and the shapes may diverge).
// ---------------------------------------------------------------------

struct StyleBlock<'a> {
    /// Byte offset of the block's *content* (after `<style …>`) into
    /// the original source.
    content_offset: usize,
    content: &'a str,
}

struct Decl<'a> {
    /// Property name, with its original case.
    property: &'a str,
    /// Byte offset of the value into the style-block content.
    value_offset: usize,
    value: &'a str,
}

fn scan_style_blocks(source: &str) -> Vec<StyleBlock<'_>> {
    let mut out = Vec::new();
    let bytes = source.as_bytes();
    let mut i = 0;
    while let Some(rel) = find_subslice(&bytes[i..], b"<style") {
        let tag_start = i + rel;
        let after_tag = tag_start + b"<style".len();
        if !matches!(
            bytes.get(after_tag).copied(),
            Some(b' ' | b'\t' | b'\n' | b'\r' | b'>' | b'/')
        ) {
            i = after_tag;
            continue;
        }
        let Some(open_end_rel) = find_subslice(&bytes[after_tag..], b">") else {
            break;
        };
        if open_end_rel > 0 && bytes.get(after_tag + open_end_rel - 1) == Some(&b'/') {
            i = after_tag + open_end_rel + 1;
            continue;
        }
        let content_start = after_tag + open_end_rel + 1;
        let Some(close_rel) = find_subslice(&bytes[content_start..], b"</style>") else {
            break;
        };
        let content_end = content_start + close_rel;
        out.push(StyleBlock {
            content_offset: content_start,
            content: &source[content_start..content_end],
        });
        i = content_end + b"</style>".len();
    }
    out
}

/// Walk every `property: value` declaration in a style block. Skips
/// CSS comments and balances parens so a `;` inside `var(...)` /
/// `color-mix(...)` doesn't terminate the value early.
fn scan_declarations(content: &str) -> Vec<Decl<'_>> {
    let mut out = Vec::new();
    let bytes = content.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // Skip leading whitespace and at-rule / selector junk.
        while i < bytes.len()
            && (bytes[i].is_ascii_whitespace() || matches!(bytes[i], b';' | b'{' | b'}'))
        {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        // Skip CSS comments.
        if bytes[i] == b'/' && bytes.get(i + 1) == Some(&b'*') {
            if let Some(end_rel) = find_subslice(&bytes[i + 2..], b"*/") {
                i += 2 + end_rel + 2;
                continue;
            }
            break;
        }
        let name_start = i;
        while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'-') {
            i += 1;
        }
        if i == name_start {
            i += 1;
            continue;
        }
        let name = &content[name_start..i];
        // Skip whitespace, expect `:`.
        let mut k = i;
        while k < bytes.len() && bytes[k].is_ascii_whitespace() {
            k += 1;
        }
        if k >= bytes.len() || bytes[k] != b':' {
            // Not a declaration — skip to next `;` or `}`.
            while i < bytes.len() && !matches!(bytes[i], b';' | b'{' | b'}') {
                i += 1;
            }
            continue;
        }
        // Skip past the `:` and any whitespace.
        let mut v = k + 1;
        while v < bytes.len() && bytes[v].is_ascii_whitespace() {
            v += 1;
        }
        let value_start = v;
        // Read the value, balancing parens so a `;` inside
        // `var(...)` / `color-mix(...)` doesn't terminate early.
        let mut depth = 0i32;
        while v < bytes.len() {
            match bytes[v] {
                b'(' => depth += 1,
                b')' => depth -= 1,
                b';' | b'}' if depth <= 0 => break,
                _ => {}
            }
            v += 1;
        }
        out.push(Decl {
            property: name,
            value_offset: value_start,
            value: content[value_start..v].trim_end(),
        });
        i = v;
    }
    out
}

/// Names defined inside the block via `--name: …;` declarations. The
/// shape is borrowed from `css_token::collect_local_definitions`: walk
/// looking for `--ident` followed by a colon. The list is used to
/// exempt token-declaration `font-family` values from the rule.
fn collect_local_definitions(content: &str) -> HashSet<&str> {
    let mut out = HashSet::new();
    let bytes = content.as_bytes();
    let mut i = 0;
    while i + 2 < bytes.len() {
        if bytes[i] == b'-' && bytes[i + 1] == b'-' && is_ident_start(bytes[i + 2]) {
            let name_start = i + 2;
            let mut j = name_start;
            while j < bytes.len() && is_ident_cont(bytes[j]) {
                j += 1;
            }
            // After the name, skip whitespace, then check for `:`.
            let mut k = j;
            while k < bytes.len() && bytes[k].is_ascii_whitespace() {
                k += 1;
            }
            if bytes.get(k) == Some(&b':') {
                out.insert(&content[name_start..j]);
            }
            i = j;
        } else {
            i += 1;
        }
    }
    out
}

fn is_ident_start(b: u8) -> bool {
    b.is_ascii_lowercase()
}

fn is_ident_cont(b: u8) -> bool {
    b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-'
}

/// Convert a byte offset into the source string into a 1-based
/// `(line, col)` pair. Mirrors the helper in the sibling rules.
fn byte_offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let bytes = source.as_bytes();
    let mut line = 1usize;
    let mut col = 1usize;
    let cap = offset.min(bytes.len());
    for &b in &bytes[..cap] {
        if b == b'\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fake_path() -> PathBuf {
        PathBuf::from("/fake/foo.sail")
    }

    fn run(src: &str) -> Vec<Finding> {
        check(&fake_path(), src)
    }

    // --- The bare cases the spec calls out ---

    #[test]
    fn inter_fires_body_font() {
        let src = r#"<style>:host { font-family: "Inter"; }</style>"#;
        let f = run(src);
        assert_eq!(f.len(), 1, "{f:?}");
        assert_eq!(f[0].rule, RULE_ID);
        assert_eq!(f[0].severity, Severity::Warn);
        assert!(f[0].message.contains("Inter"), "{}", f[0].message);
        let suggest = f[0].suggest.as_deref().unwrap();
        assert!(suggest.contains("var(--body-font)"), "suggest: {suggest}");
    }

    #[test]
    fn syne_fires_heading_font() {
        let src = r#"<style>:host { font-family: "Syne"; }</style>"#;
        let f = run(src);
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(f[0].message.contains("Syne"), "{}", f[0].message);
        let suggest = f[0].suggest.as_deref().unwrap();
        assert!(
            suggest.contains("var(--heading-font)"),
            "suggest: {suggest}"
        );
    }

    #[test]
    fn jetbrains_mono_fires_mono_font() {
        let src = r#"<style>:host { font-family: "JetBrains Mono"; }</style>"#;
        let f = run(src);
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(f[0].message.contains("JetBrains Mono"), "{}", f[0].message);
        let suggest = f[0].suggest.as_deref().unwrap();
        assert!(suggest.contains("var(--mono-font)"), "suggest: {suggest}");
    }

    // --- The non-firing cases the spec calls out ---

    #[test]
    fn var_body_font_does_not_fire() {
        let src = r#"<style>:host { font-family: var(--body-font); }</style>"#;
        assert!(run(src).is_empty(), "{:?}", run(src));
    }

    #[test]
    fn var_font_display_app_does_not_fire() {
        // The literal lives in the token definition, not in component
        // CSS. A `var(--font-display-app)` reference is the correct
        // way to consume the display role.
        let src = r#"<style>:host { font-family: var(--font-display-app); }</style>"#;
        assert!(run(src).is_empty(), "{:?}", run(src));
    }

    #[test]
    fn font_display_app_declaration_with_inter_does_not_fire() {
        // The literal is the definition itself — `--font-display-app:
        // "Inter", sans-serif;` is the canonical font slot in
        // tokens.css. Lint must skip.
        let src = r#"<style>:root { --font-display-app: "Inter", sans-serif; }</style>"#;
        assert!(run(src).is_empty(), "{:?}", run(src));
    }

    #[test]
    fn font_body_app_declaration_with_inter_does_not_fire() {
        let src = r#"<style>:root { --font-body-app: "Inter", sans-serif; }</style>"#;
        assert!(run(src).is_empty(), "{:?}", run(src));
    }

    // --- Multi-family value + boundaries ---

    #[test]
    fn multi_family_value_fires_for_each_palette_name() {
        // "Inter Variable" + "Inter" + sans-serif — both palette names
        // fire (one each), sans-serif is fine.
        let src = r#"<style>:host { font-family: "Inter Variable", "Inter", sans-serif; }</style>"#;
        let f = run(src);
        assert_eq!(f.len(), 2, "{f:?}");
        assert!(f.iter().any(|x| x.message.contains("Inter Variable")));
        assert!(f.iter().any(|x| x.message.contains("`Inter`")));
    }

    #[test]
    fn internal_word_does_not_match_inter() {
        // `Interrogate` is a CSS class name, not a font family. The
        // whole-token boundary check ensures we don't flag the `Inter`
        // substring.
        let src = r#"<style>:host { font-family: Interrogate, sans-serif; }</style>"#;
        assert!(run(src).is_empty(), "{:?}", run(src));
    }

    #[test]
    fn single_quoted_inter_fires() {
        // CSS allows single-quoted family names too.
        let src = r#"<style>:host { font-family: 'Inter'; }</style>"#;
        assert_eq!(run(src).len(), 1);
    }

    #[test]
    fn case_insensitive_match() {
        // Browsers match family names case-insensitively. The lint
        // should too, so a poorly-cased `inter` doesn't slip through.
        let src = r#"<style>:host { font-family: "inter"; }</style>"#;
        assert_eq!(run(src).len(), 1);
    }

    #[test]
    fn dm_sans_fires_body_font() {
        let src = r#"<style>:host { font-family: "DM Sans"; }</style>"#;
        let f = run(src);
        assert_eq!(f.len(), 1, "{f:?}");
        let suggest = f[0].suggest.as_deref().unwrap();
        assert!(suggest.contains("var(--body-font)"), "suggest: {suggest}");
    }

    #[test]
    fn ibm_plex_serif_fires_heading_font() {
        let src = r#"<style>:host { font-family: "IBM Plex Serif"; }</style>"#;
        let f = run(src);
        assert_eq!(f.len(), 1, "{f:?}");
        let suggest = f[0].suggest.as_deref().unwrap();
        assert!(
            suggest.contains("var(--heading-font)"),
            "suggest: {suggest}"
        );
    }

    #[test]
    fn ibm_plex_mono_fires_mono_font() {
        let src = r#"<style>:host { font-family: "IBM Plex Mono"; }</style>"#;
        let f = run(src);
        assert_eq!(f.len(), 1, "{f:?}");
        let suggest = f[0].suggest.as_deref().unwrap();
        assert!(suggest.contains("var(--mono-font)"), "suggest: {suggest}");
    }

    #[test]
    fn space_grotesk_fires_heading_font() {
        let src = r#"<style>:host { font-family: "Space Grotesk"; }</style>"#;
        let f = run(src);
        assert_eq!(f.len(), 1, "{f:?}");
        let suggest = f[0].suggest.as_deref().unwrap();
        assert!(
            suggest.contains("var(--heading-font)"),
            "suggest: {suggest}"
        );
    }

    // --- Comment handling ---

    #[test]
    fn commented_out_inter_does_not_fire() {
        let src = "<style>/* :host { font-family: \"Inter\"; } */</style>";
        assert!(run(src).is_empty(), "{:?}", run(src));
    }

    #[test]
    fn comment_around_real_decl_does_not_swallow_it() {
        // The comment between the block and the declaration should not
        // confuse the walker — the real `font-family` still fires.
        let src = r#"<style>:host { /* hint */ font-family: "Inter"; }</style>"#;
        assert_eq!(run(src).len(), 1);
    }

    // --- Scoping / robustness ---

    #[test]
    fn no_style_block_no_findings() {
        let src = r#"<div>font-family: "Inter"</div>"#;
        assert!(run(src).is_empty(), "{:?}", run(src));
    }

    #[test]
    fn non_font_family_declaration_with_inter_does_not_fire() {
        // `Inter` in a `content` or other non-font-family property is
        // out of scope. The rule is font-family-specific.
        let src = r#"<style>:host::before { content: "Inter"; }</style>"#;
        assert!(run(src).is_empty(), "{:?}", run(src));
    }

    #[test]
    fn position_points_at_the_match() {
        // The finding should land on the `"Inter"` (or the `I` of
        // `Inter`) so the editor highlights the right token.
        let src = "<style>\n  :host { font-family: \"Inter\"; }\n</style>";
        let f = run(src);
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].line, 2);
        // "  :host { font-family: " is 24 chars; the `I` is at col 25.
        assert_eq!(f[0].col, 25);
    }

    #[test]
    fn multiple_style_blocks_each_scanned() {
        // Two style blocks, each with one hardcoded font, two findings.
        let src = r#"<style>:host { font-family: "Inter"; }</style>
<style>:host { font-family: "Syne"; }</style>"#;
        let f = run(src);
        assert_eq!(f.len(), 2, "{f:?}");
    }

    #[test]
    fn self_audit_canonical_clean_block() {
        // A clean block — only role tokens — produces no findings.
        let src = r#"<style>
            :host {
                font-family: var(--body-font);
                color: var(--foreground);
            }
            h1 { font-family: var(--heading-font); }
            code { font-family: var(--mono-font); }
            .label { font-family: var(--label-font); }
        </style>"#;
        assert!(run(src).is_empty(), "{:?}", run(src));
    }

    #[test]
    fn style_font_family_quoted_string_is_red() {
        let src = r#"<p style({ text: fontFamily("Inter") })>hi</p>"#;
        let f = run(src);
        assert_eq!(f.len(), 1, "{f:?}");
        assert_eq!(f[0].severity, Severity::Error);
        assert!(
            f[0].message.contains("role tokens only"),
            "{}",
            f[0].message
        );
    }

    #[test]
    fn style_font_family_role_token_is_clean() {
        let src = r#"<p style({ text: fontFamily(body).fontSize(sm) })>hi</p>"#;
        assert!(run(src).is_empty(), "{:?}", run(src));
    }

    #[test]
    fn style_font_family_bind_is_clean() {
        let src = r#"<p style({ text: fontFamily(bind(role)) })>hi</p>"#;
        assert!(run(src).is_empty(), "{:?}", run(src));
    }
}
