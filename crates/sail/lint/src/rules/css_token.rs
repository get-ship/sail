//! `sail-css-token` — `var(--xyz)` inside a `.sui` `<style>` block must
//! resolve to either the canonical token CSS
//! (`packages/sail/tokens/src/tokens.css`, parsed by `crate::tokens`) or a
//! custom property set locally inside the same `<style>` block.
//!
//! Catches typos that today fall through silently (the variable is
//! undefined, the fallback or empty string is used, the visual is
//! wrong but nothing screams). When the name is unknown we Levenshtein-
//! suggest the closest match across both the theme registry and the
//! component-local definitions.
//!
//! Some token names are not declared in the canonical
//! `packages/sail/tokens/src/{tokens,themes}.css` but are still
//! legitimate, well-known conventions — the canonical source uses
//! them in `var(--name, fallback)` chains across every theme-preview
//! block. The canonical example is `--font-sans`: `@ship/tokens`
//! sets `--font-body: var(--font-sans, system-ui, sans-serif);` in
//! every theme preview even though no `--font-sans: ...;`
//! declaration exists. These *convention* tokens are listed in
//! `crate::tokens::CONVENTION_TOKENS` so the lint recognizes them. When
//! `@ship/tokens` introduces a new convention, add the name there.
//!
//! Severity: `Error` — the build hard-fails on an unknown token. The
//! intent is to catch typos, NOT to reject real tokens; the
//! convention-tokens block above is the way real-but-undeclared
//! names are recognized. External SUI apps that want a softer
//! behavior can opt-out via `CompileOpts::lint_overrides`
//! (`OverrideSeverity::Warn` or `Off`); the internal monorepo
//! compiles with the default (Error) and stays strict.
//!
//! Out of scope:
//! - Light/dark theme variant validation — we just check token-name
//!   existence; whether dark-mode also defines the value is the
//!   theme's job.
//! - Removing fallback values in `var(--x, fallback)`. The fallback's
//!   own `var()` references are still scanned, but the fallback
//!   itself is not lint-restricted.
//!
//! Position reporting is on the `var(...)` call, not the `<style>`
//! block as a whole — exactly where the typo lives.
//!
//! The registry is parsed once from `tokens.css`'s `--name:` declarations
//! by `crate::tokens` (plus the convention allowlist) — no mirror file.
//!
//! Related rule: `sail-controller-purity` (controllers don't write raw
//! HTML) — same finding shape, same plumbing into the compiler check
//! phase.
//!
//! Suffix conventions match `controller_purity.rs`: rule ID
//! `sail-<descriptor>`, public `RULE_ID` const, public
//! `check(path, source) -> Vec<Finding>`.

use std::collections::HashSet;
use std::path::Path;

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sail-css-token";

/// Lint a `.sui` source. Walks every `<style>...</style>` block: each
/// `var(--xyz)` reference must be in the theme registry or defined
/// locally as `--xyz: ...` somewhere in the same block.
pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let theme = theme_tokens();
    let mut findings = Vec::new();

    for block in scan_style_blocks(source) {
        let local = collect_local_definitions(block.content);
        for reference in collect_var_references(block.content) {
            if theme.contains(reference.name) || local.contains(reference.name) {
                continue;
            }
            let abs_offset = block.content_offset + reference.offset;
            let (line, col) = byte_offset_to_line_col(source, abs_offset);
            let suggestion = nearest_token(reference.name, theme, &local);
            let mut message = format!("unknown CSS token --{}", reference.name);
            if let Some(s) = &suggestion {
                message.push_str(&format!("; did you mean --{s}?"));
            }
            findings.push(Finding {
                file: path.to_path_buf(),
                line,
                col,
                rule: RULE_ID,
                severity: Severity::Error,
                message,
                suggest: Some(suggest_text(suggestion.as_deref())),
            });
        }
    }
    findings
}

/// Hint shown beneath the message. Keeps the registry path in front
/// of the human author so they know where to add a real new token vs
/// fix the typo.
fn suggest_text(closest: Option<&str>) -> String {
    let mut s = String::from(
        "Use a token from the canonical token CSS (packages/sail/tokens/src/tokens.css) \
         or define `--<name>: ...;` in this <style> block.",
    );
    if let Some(name) = closest {
        s.push_str(&format!(" Closest known token: --{name}."));
    }
    s
}

/// The flat set of theme token names — delegated to the shared
/// `crate::tokens` registry, parsed from `tokens.css` (plus the convention
/// allowlist). The single source both this lint and the `sx` vocabulary read.
fn theme_tokens() -> &'static HashSet<&'static str> {
    crate::tokens::registry()
}

/// One scanned `<style>` block.
struct StyleBlock<'a> {
    /// Byte offset of the block's *content* (after `<style …>`) into
    /// the original source.
    content_offset: usize,
    content: &'a str,
}

/// Find every `<style …>…</style>` block. Tolerates attributes on the
/// opening tag and any whitespace; the content stops at the next
/// `</style>`. We do not parse arbitrary HTML — `<style>` is a
/// raw-text element so its contents are CSS, never nested tags.
fn scan_style_blocks(source: &str) -> Vec<StyleBlock<'_>> {
    let mut out = Vec::new();
    let bytes = source.as_bytes();
    let mut i = 0;
    while let Some(rel) = find_subslice(&bytes[i..], b"<style") {
        let tag_start = i + rel;
        let after_tag = tag_start + b"<style".len();
        // Must be a real tag, not e.g. <styled> or <styles> — next
        // char is whitespace or '>'.
        if !matches!(
            bytes.get(after_tag).copied(),
            Some(b' ' | b'\t' | b'\n' | b'\r' | b'>' | b'/')
        ) {
            i = after_tag;
            continue;
        }
        // Find the end of the opening tag.
        let Some(open_end_rel) = find_subslice(&bytes[after_tag..], b">") else {
            break;
        };
        // Self-closing `<style …/>` carries no content.
        if open_end_rel > 0 && bytes.get(after_tag + open_end_rel - 1) == Some(&b'/') {
            i = after_tag + open_end_rel + 1;
            continue;
        }
        let content_start = after_tag + open_end_rel + 1;
        let Some(close_rel) = find_subslice(&bytes[content_start..], b"</style>") else {
            // Unterminated block — bail rather than scan to EOF (which
            // would let comments after the block leak in).
            break;
        };
        let content_end = content_start + close_rel;
        let content = &source[content_start..content_end];
        out.push(StyleBlock {
            content_offset: content_start,
            content,
        });
        i = content_end + b"</style>".len();
    }
    out
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    let last = haystack.len() - needle.len();
    for i in 0..=last {
        if &haystack[i..i + needle.len()] == needle {
            return Some(i);
        }
    }
    None
}

/// Names defined inside the block via `--name: …;` declarations. We
/// look for a `--ident` token followed by optional whitespace and a
/// colon — the only CSS shape that introduces a custom property.
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
            // After the name, skip whitespace, then check for ':'. A
            // matching colon means this is a property declaration; a
            // `)` or `,` indicates a `var(--name, …)` reference and is
            // ignored.
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

/// Position + name of a `var(--name)` use inside the style block.
struct VarRef<'a> {
    /// Byte offset of the `--` (start of the name minus 2) inside the
    /// block content.
    offset: usize,
    name: &'a str,
}

/// Find every `var(--name)` reference. Whitespace between `var(`,
/// `--`, and the name is allowed (CSS spec). Fallbacks of the form
/// `var(--x, fallback)` are discovered alongside any `var(--y)` that
/// might appear inside the fallback — we scan the entire block, not
/// just balanced groups.
fn collect_var_references(content: &str) -> Vec<VarRef<'_>> {
    let mut out = Vec::new();
    let bytes = content.as_bytes();
    let needle = b"var(";
    let mut i = 0;
    while i + needle.len() < bytes.len() {
        if &bytes[i..i + needle.len()] == needle {
            // Skip whitespace.
            let mut k = i + needle.len();
            while k < bytes.len() && bytes[k].is_ascii_whitespace() {
                k += 1;
            }
            if bytes.get(k) == Some(&b'-') && bytes.get(k + 1) == Some(&b'-') {
                let dashes = k;
                let name_start = k + 2;
                if name_start < bytes.len() && is_ident_start(bytes[name_start]) {
                    let mut j = name_start;
                    while j < bytes.len() && is_ident_cont(bytes[j]) {
                        j += 1;
                    }
                    out.push(VarRef {
                        offset: dashes,
                        name: &content[name_start..j],
                    });
                    i = j;
                    continue;
                }
            }
            i = k;
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

/// Return the closest known token to `name` across both the theme
/// registry and the local definitions, if the Levenshtein distance
/// is small enough to be a plausible typo (≤ 3, capped at half the
/// length so we don't pair `--x` with `--abc`).
fn nearest_token(name: &str, theme: &HashSet<&str>, local: &HashSet<&str>) -> Option<String> {
    let max_dist = std::cmp::min(3, name.len() / 2 + 1);
    let mut best: Option<(usize, String)> = None;
    for candidate in theme.iter().copied().chain(local.iter().copied()) {
        if (candidate.len() as isize - name.len() as isize).unsigned_abs() > max_dist {
            continue;
        }
        let d = levenshtein(name, candidate, max_dist + 1);
        if d <= max_dist && best.as_ref().is_none_or(|(b, _)| d < *b) {
            best = Some((d, candidate.to_string()));
        }
    }
    best.map(|(_, s)| s)
}

/// Bounded Levenshtein. Returns `cap` once the row minimum exceeds it
/// — caller treats `cap` as "no useful match." Standard 2-row DP, no
/// allocations beyond the row buffer.
fn levenshtein(a: &str, b: &str, cap: usize) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.is_empty() {
        return b.len().min(cap);
    }
    if b.is_empty() {
        return a.len().min(cap);
    }
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut curr: Vec<usize> = vec![0; b.len() + 1];
    for i in 1..=a.len() {
        curr[0] = i;
        let mut row_min = curr[0];
        for j in 1..=b.len() {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            curr[j] = (prev[j] + 1)
                .min(curr[j - 1] + 1)
                .min(prev[j - 1] + cost);
            if curr[j] < row_min {
                row_min = curr[j];
            }
        }
        if row_min > cap {
            return cap;
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[b.len()].min(cap)
}

/// Convert a byte offset into the source string into a 1-based
/// `(line, col)` pair. Mirrors the helper in `controller_purity.rs` —
/// kept duplicated rather than imported because each rule owns its
/// position-mapping and the shape may diverge (e.g. UTF-16 columns
/// for an LSP that needs them).
fn byte_offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let bytes = source.as_bytes();
    let mut line = 1usize;
    let mut col = 1usize;
    let cap = offset.min(bytes.len());
    let mut i = 0;
    while i < cap {
        if bytes[i] == b'\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
        i += 1;
    }
    (line, col)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fake_path() -> PathBuf {
        PathBuf::from("/fake/foo.sui")
    }

    fn run(src: &str) -> Vec<Finding> {
        check(&fake_path(), src)
    }

    #[test]
    fn known_theme_token_passes() {
        let src = "<style>:host { color: var(--foreground); }</style>";
        assert!(run(src).is_empty());
    }

    #[test]
    fn locally_defined_token_passes() {
        let src = r#"<style>
            :host { --btn-h: 34px; height: var(--btn-h); }
        </style>"#;
        assert!(run(src).is_empty(), "{:?}", run(src));
    }

    #[test]
    fn unknown_token_reports() {
        let src = "<style>:host { color: var(--definitely-not-real); }</style>";
        let f = run(src);
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].rule, RULE_ID);
        assert!(f[0].message.contains("--definitely-not-real"), "{}", f[0].message);
    }

    #[test]
    fn one_letter_typo_suggests() {
        let src = "<style>:host { color: var(--forground); }</style>";
        let f = run(src);
        assert_eq!(f.len(), 1);
        assert!(
            f[0].message.contains("--foreground"),
            "expected suggestion in message, got: {}",
            f[0].message
        );
    }

    #[test]
    fn fallback_inner_var_is_scanned() {
        // `var(--x, var(--bogus))` — inner reference is also checked.
        let src = "<style>:host { color: var(--foreground, var(--definitely-not-real)); }</style>";
        let f = run(src);
        assert_eq!(f.len(), 1);
        assert!(f[0].message.contains("--definitely-not-real"));
    }

    #[test]
    fn no_style_blocks_no_findings() {
        let src = "<div>var(--definitely-not-real)</div>";
        assert!(run(src).is_empty());
    }

    #[test]
    fn similarly_named_html_tag_is_ignored() {
        // `<styled>` should not be parsed as `<style>` — the suffix
        // check guards against false positives.
        let src = "<styled>var(--definitely-not-real)</styled>";
        assert!(run(src).is_empty());
    }

    #[test]
    fn whitespace_in_var_reference_is_handled() {
        let src = "<style>:host { color: var( --definitely-not-real ); }</style>";
        let f = run(src);
        assert_eq!(f.len(), 1);
    }

    #[test]
    fn local_def_with_whitespace_before_colon() {
        let src = r#"<style>:host { --foo : 1px ; height: var(--foo); }</style>"#;
        assert!(run(src).is_empty());
    }

    #[test]
    fn position_points_at_var_reference() {
        let src = "<style>\n  .x { color: var(--bogus-token); }\n</style>";
        let f = run(src);
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].line, 2);
        // 1-based col of the `--` in `var(--bogus-token)`.
        // "  .x { color: var(" is 18 chars; `--` starts at col 19.
        assert_eq!(f[0].col, 19);
    }

    #[test]
    fn multiple_style_blocks_each_scoped() {
        // Local def in block A doesn't satisfy a reference in block B.
        let src = r#"<style>:host { --my-local: 1px; }</style>
<style>:host { height: var(--my-local); }</style>"#;
        let f = run(src);
        assert_eq!(f.len(), 1, "second block lacks the local def: {f:?}");
    }
}
