//! Anti-drift gate for `crates/deck/FEATURES.md`.
//!
//! The feature inventory is only useful if its implementation pointers stay
//! true. This test reads `FEATURES.md`, extracts every implementation
//! reference, and asserts for each that the referenced file exists AND — when
//! the ref carries a `::symbol` suffix — that the symbol is a real EXPORTED
//! Rust item (`pub fn`, `pub struct`, `pub enum`, `pub type`, `pub trait`,
//! `pub const`, `pub static`, or `pub use …`). A substring match is NOT
//! enough: P2-11 caught a regression where a symbol surviving only in a
//! comment passed the gate. The token check is a regex match on a Rust item
//! declaration (with `pub` visibility), so a renamed symbol or a moved-but-
//! still-mentioned-in-comments item fails the build.
//!
//! Reference grammar (matched anywhere in the file):
//!   - `crates/<...>/<file>.rs::<symbol>` — file must exist, symbol must be
//!     a real exported Rust item (`pub fn/struct/enum/type/trait/const/static`
//!     or `pub use …`).
//!   - `crates/<...>/<file>.rs`           — file must exist (no symbol check).
//!   - `crates/<...>:<line>`              — file must exist (a `path:line` ref).
//!
//! The repo root is resolved from `CARGO_MANIFEST_DIR`
//! (`<repo>/crates/deck/deck-core`) up three parents, then `FEATURES.md` is read
//! from `<repo>/crates/deck/FEATURES.md`.
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// `<repo>/crates/deck/deck-core` → `<repo>`.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent() // crates/deck
        .and_then(|p| p.parent()) // crates
        .and_then(|p| p.parent()) // repo root
        .expect("CARGO_MANIFEST_DIR has three ancestors")
        .to_path_buf()
}

/// One extracted reference: the repo-relative file path, and an optional symbol.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Ref {
    path: String,
    symbol: Option<String>,
}

/// Characters that can appear in a repo-relative path token we extract. We
/// deliberately stop at backtick, whitespace, `|`, `:`, and `*` (the
/// table-cell and markdown delimiters) so a ref inside a `crates/...` code span
/// is captured cleanly.
fn is_path_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '/' | '_' | '-' | '.')
}

fn is_symbol_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Scan `text` for every `crates/...` reference and return the deduped set.
///
/// A reference starts at a `crates/` occurrence, runs while [`is_path_char`],
/// and — if immediately followed by `::` — captures the trailing symbol token.
/// A path that does not end in `.rs` is kept only when it is a `path:line`
/// `true` if `contents` contains `sym` as a real exported Rust item —
/// the declaration's keyword is preceded by `pub` and followed by the
/// symbol's identifier. The shape we recognise:
///
///   pub fn <sym>( … ) { … }              (function)
///   pub async fn <sym>( … ) { … }        (async function)
///   pub unsafe fn <sym>( … ) { … }       (unsafe function)
///   pub struct <sym>( … ) | { … } | ;     (struct)
///   pub enum <sym> { … }                  (enum)
///   pub trait <sym>( … ) | { … }          (trait)
///   pub type <sym> = …;                   (type alias)
///   pub const <sym>: … = …;               (const)
///   pub static <sym>: … = …;              (static)
///   pub use (…|)::<sym>( … )? ;           (re-export — symbol appears in the
///                                          import list, optionally aliased)
///
/// Substring matches (a symbol surviving only in a comment or doc-string
/// without a `pub` declaration nearby) return `false`. The check is
/// intentionally conservative — it might miss a `pub fn` declaration split
/// across an attribute (`#[inline] pub fn …`) in unusual shapes, but the
/// FEATURES.md inventory is hand-authored and the handful of items it cites
/// are all plain `pub fn`/`pub struct` etc. declarations.
fn is_exported_rust_item(contents: &str, sym: &str) -> bool {
    use std::sync::OnceLock;
    // The regex matches `pub` followed by an optional modifier (`async`,
    // `unsafe`, or `default`) and one of the eight item keywords
    // (`fn`, `struct`, `enum`, `trait`, `type`, `const`, `static`, `use`),
    // then captures the symbol identifier that follows. RE2 has no
    // look-around, so `const fn` is matched by the modifier-aware shape
    // form is matched by the keyword branch. Both shapes survive.
    //
    // `pub use` is matched as a `use` keyword followed by a path; we capture
    // the LAST identifier in the path (the re-exported symbol). RE2 has
    // no lazy quantifier we can use cleanly, so we use a greedy tail-match
    // and let the word-boundary at the end keep it tight.
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    let re = RE.get_or_init(|| {
        regex::Regex::new(
            r"(?xm)
              \bpub\b
              (?:[\s]+(?:async|unsafe|default))?[\s]*
              (?:fn|struct|enum|trait|type|const|static)\b
              (?:[\s]+[^;{]*?\b)??(?P<sym1>\w+)\b
              |
              \bpub\b[\s]+use\b[^;\n]*\b(?P<sym2>\w+)\b",
        )
        .expect("is_exported_rust_item regex is valid")
    });
    re.captures_iter(contents).any(|cap| cap.name("sym1").or_else(|| cap.name("sym2")).map_or(false, |m| m.as_str() == sym))
}
/// itself is captured up to the `.rs` or directory token). We only assert on
/// refs whose captured path ends in `.rs`, which is every Implementation cell.
fn extract_refs(text: &str) -> BTreeSet<Ref> {
    let mut out = BTreeSet::new();
    let bytes = text.as_bytes();
    let mut i = 0usize;
    while let Some(rel) = text[i..].find("crates/") {
        let start = i + rel;
        // Walk the path token.
        let mut j = start;
        while j < bytes.len() {
            let c = text[j..].chars().next().unwrap();
            if is_path_char(c) {
                j += c.len_utf8();
            } else {
                break;
            }
        }
        let path = &text[start..j];
        // Only consider `.rs` file refs — that is what every Implementation
        // cell uses; a stray `crates/...` directory mention in prose is ignored.
        if path.ends_with(".rs") {
            // Optional `::symbol` suffix.
            let symbol = if text[j..].starts_with("::") {
                let sym_start = j + 2;
                let mut k = sym_start;
                while k < bytes.len() {
                    let c = text[k..].chars().next().unwrap();
                    if is_symbol_char(c) {
                        k += c.len_utf8();
                    } else {
                        break;
                    }
                }
                if k > sym_start {
                    Some(text[sym_start..k].to_string())
                } else {
                    None
                }
            } else {
                None
            };
            out.insert(Ref { path: path.to_string(), symbol });
        }
        i = j.max(start + 1);
    }
    out
}

#[test]
fn every_implementation_ref_resolves() {
    let root = repo_root();
    let doc_path = root.join("crates/deck/FEATURES.md");
    let doc = std::fs::read_to_string(&doc_path)
        .unwrap_or_else(|e| panic!("read FEATURES.md at {}: {e}", doc_path.display()));

    let refs = extract_refs(&doc);
    assert!(
        !refs.is_empty(),
        "extracted zero implementation refs from {} — the doc or the extractor is broken",
        doc_path.display(),
    );

    // Cache file contents so a file referenced N times is read once.
    let mut cache: std::collections::HashMap<String, Option<String>> =
        std::collections::HashMap::new();

    let mut first_failure: Option<String> = None;
    for r in &refs {
        let contents = cache
            .entry(r.path.clone())
            .or_insert_with(|| std::fs::read_to_string(root.join(&r.path)).ok());

        let Some(contents) = contents else {
            first_failure = Some(format!(
                "unresolved ref `{}`: file does not exist at {}",
                r.path,
                root.join(&r.path).display(),
            ));
            break;
        };
        if let Some(sym) = &r.symbol
            && !is_exported_rust_item(&contents, sym)
        {
            first_failure = Some(format!(
                "unresolved ref `{}::{}`: symbol `{}` is not a `pub` Rust item in {} \
                 (the gate rejects substring-only matches — comments and doc-strings \
                 are not enough; the symbol must be declared `pub fn`/`pub struct`/`pub enum`/\
                 `pub type`/`pub trait`/`pub const`/`pub static`/`pub use` in the file)",
                r.path,
                sym,
                sym,
                root.join(&r.path).display(),
            ));
            break;
        }
    }

    assert!(
        first_failure.is_none(),
        "FEATURES.md has a stale implementation reference (the code is truth — \
         fix the doc, not this test):\n  {}\n\nChecked {} refs total.",
        first_failure.unwrap(),
        refs.len(),
    );
}

#[cfg(test)]
mod extractor_tests {
    use super::*;

    #[test]
    fn extracts_path_and_symbol() {
        let refs = extract_refs("see `crates/deck/deck-core/src/view.rs::serve_sui_view_setup` here");
        assert_eq!(refs.len(), 1);
        let r = refs.iter().next().unwrap();
        assert_eq!(r.path, "crates/deck/deck-core/src/view.rs");
        assert_eq!(r.symbol.as_deref(), Some("serve_sui_view_setup"));
    }

    #[test]
    fn extracts_bare_file_ref() {
        let refs = extract_refs("at `crates/deck/deck-cli/src/main.rs` (no symbol)");
        assert_eq!(refs.len(), 1);
        let r = refs.iter().next().unwrap();
        assert_eq!(r.path, "crates/deck/deck-cli/src/main.rs");
        assert!(r.symbol.is_none());
    }

    #[test]
    fn ignores_non_rs_directory_mentions() {
        // A `crates/...` mention that is not a `.rs` file ref is not asserted.
        let refs = extract_refs("output lands in `crates/deck/deck-core/assets/`");
        assert!(refs.is_empty(), "directory mention must not be captured: {refs:?}");
    }

    #[test]
    fn stops_at_table_delimiters() {
        // A ref in a markdown table cell ends at the surrounding backtick/pipe.
        let refs = extract_refs(
            "| Feature | `crates/deck/deck-core/src/response.rs::DeckResponse` | desc |",
        );
        assert_eq!(refs.len(), 1);
        let r = refs.iter().next().unwrap();
        assert_eq!(r.path, "crates/deck/deck-core/src/response.rs");
        assert_eq!(r.symbol.as_deref(), Some("DeckResponse"));
    }

    /// P2-11 — the symbol check must NOT be a substring match. A symbol
    /// surviving only in a comment / doc-string / use-line / pub(crate)
    /// alias must fail, because the inventory cites a `pub` API surface.
    /// These tests pin that behaviour: a regression to substring-match
    /// (the pre-P2-11 shape) breaks ALL of them.
    #[test]
    fn exported_item_check_accepts_pub_declarations() {
        let contents = "\
            pub fn serve_thing() {}\n\
            pub struct MyStruct {}\n\
            pub enum MyEnum { A }\n\
            pub trait MyTrait {}\n\
            pub type MyAlias = u32;\n\
            pub const MY_CONST: u32 = 0;\n\
            pub static MY_STATIC: u32 = 0;\n\
            pub use crate::other::serve_other;\n\
            pub async fn serve_async() {}\n\
            pub unsafe fn serve_unsafe() {}\n";
        assert!(is_exported_rust_item(contents, "serve_thing"));
        assert!(is_exported_rust_item(contents, "MyStruct"));
        assert!(is_exported_rust_item(contents, "MyEnum"));
        assert!(is_exported_rust_item(contents, "MyTrait"));
        assert!(is_exported_rust_item(contents, "MyAlias"));
        assert!(is_exported_rust_item(contents, "MY_CONST"));
        assert!(is_exported_rust_item(contents, "MY_STATIC"));
        assert!(is_exported_rust_item(contents, "serve_other"));
        assert!(is_exported_rust_item(contents, "serve_async"));
        assert!(is_exported_rust_item(contents, "serve_unsafe"));
    }

    #[test]
    fn exported_item_check_rejects_comment_only_match() {
        // Substring match would pass; the `pub`-item check must NOT.
        let contents = "\
            // `serve_thing` was here before, the impl is gone now.\n\
            fn serve_thing() { /* no `pub` */ }\n";
        assert!(
            !is_exported_rust_item(contents, "serve_thing"),
            "comment-only mention must not satisfy the gate (pre-P2-11 substring-match bug)"
        );
    }

    #[test]
    fn exported_item_check_rejects_pub_crate_only() {
        // `pub(crate)` is crate-private — the inventory cites the public
        // surface, so crate-only symbols must fail the gate.
        let contents = "\
            pub(crate) fn serve_thing() {}\n\
            pub(crate) struct SuiPlugin {}\n";
        assert!(!is_exported_rust_item(contents, "serve_thing"));
        assert!(!is_exported_rust_item(contents, "SuiPlugin"));
    }

    #[test]
    fn exported_item_check_rejects_use_only_re_export() {
        // A `use` import (not `pub use`) brings a symbol into scope but
        // does not re-export it; the gate's `pub use` branch must NOT
        // accept a plain `use` mention.
        let contents = "use crate::other::serve_other;\n";
        assert!(
            !is_exported_rust_item(contents, "serve_other"),
            "plain `use` must not satisfy the gate (only `pub use` re-exports)"
        );
    }
}
