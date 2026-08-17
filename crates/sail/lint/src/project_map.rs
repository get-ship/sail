//! Project-map file-kind classification + `@/` alias table
//! (SPEC-PROJECT-STRUCTURE-1 §2–§3).
//!
//! Single SSOT: path + shape signals → [`FileKind`] + fixed home.
//! Misplacement diagnostics name the right home. The alias table is the
//! import matrix — lint, compiler, and deck resolve must not invent a
//! second map.
//!
//! Lives in `sail-lint` so the lint crate can consume the matrix without
//! a compiler ↔ lint cycle. Compiler re-exports this module.

use std::path::{Component, Path, PathBuf};

/// Kind of authoring file under the fixed project map.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileKind {
    /// Document `.sail` (`page.sail` / `index.sail` / `_layout.sail` under `routes/`).
    Route,
    /// `.sail` fragment. Global home is `src/components/<domain>/`.
    /// Hyphenated siblings of a route document are route-local (still L3).
    Component,
    /// Action module — path `src/actions/**` (or action-shaped export).
    Action,
    /// Store — `defineStore(…)` export and/or path `src/stores/**`.
    Store,
    /// Composition — pure signal helpers under `src/compositions/**`.
    Composition,
    /// Catalog / copy tables — `src/constants/**`.
    Constant,
    /// Theme assets under `themes/`.
    Theme,
    /// Unclassified path (outside the fixed map).
    Other,
}

/// Result of classifying a path (+ optional source text).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Classified {
    pub kind: FileKind,
    /// Fixed home for this kind (teaching string; may be a path fragment).
    pub home: String,
    /// Whether the file currently lives in its fixed home.
    pub ok: bool,
}

/// Classify `path` using path segments and optional `source` shape signals.
///
/// Shape rules (S1.1 minimal):
/// - `.sail` document (`page`/`index`/`_layout`) under `routes/` → Route
///   (wins over a later `themes` segment: `routes/themes/page.sail`)
/// - hyphenated `.sail` sibling of a route document → Component (route-local)
/// - `.sail` elsewhere → Component
/// - `defineStore(` in source → Store (even under compositions — misplacement)
/// - path under `src/actions/` → Action
/// - path under `src/stores/` → Store
/// - path under `src/compositions/` without defineStore → Composition
/// - path under `themes/` → Theme
pub fn classify(path: &Path, source: Option<&str>) -> Classified {
    let path_str = path_as_str(path);
    let segs = path_segments(path);

    // .sail documents under routes/ are Route. Hyphenated siblings are
    // route-local components (the page is the document; they are tags).
    // Must beat the themes/ check: `routes/themes/page.sail` is a route.
    if is_sail(path) {
        if has_segment(&segs, "routes") {
            if is_document_sail(path) {
                return Classified {
                    kind: FileKind::Route,
                    home: home_for(FileKind::Route).to_string(),
                    ok: true,
                };
            }
            return Classified {
                kind: FileKind::Component,
                home: home_for(FileKind::Component).to_string(),
                ok: true,
            };
        }
        // Component home is src/components/**; corpus may still use src/<name>/.
        // S1.1: path-based kind only; ok when not under routes (home enforcement later).
        return Classified {
            kind: FileKind::Component,
            home: home_for(FileKind::Component).to_string(),
            ok: true,
        };
    }

    // Theme assets (JSON etc.) under themes/. .sail already returned above.
    if has_segment(&segs, "themes") {
        return Classified {
            kind: FileKind::Theme,
            home: home_for(FileKind::Theme).to_string(),
            ok: true,
        };
    }

    // defineStore shape wins over path for Store kind
    let has_define_store = source.is_some_and(source_has_define_store);
    if has_define_store {
        let under_stores = is_under_src_dir(&segs, "stores");
        return Classified {
            kind: FileKind::Store,
            home: home_for(FileKind::Store).to_string(),
            ok: under_stores,
        };
    }

    // Path-based TS homes
    if is_under_src_dir(&segs, "actions") {
        return Classified {
            kind: FileKind::Action,
            home: home_for(FileKind::Action).to_string(),
            ok: true,
        };
    }
    if is_under_src_dir(&segs, "stores") {
        return Classified {
            kind: FileKind::Store,
            home: home_for(FileKind::Store).to_string(),
            ok: true,
        };
    }
    if is_under_src_dir(&segs, "compositions") {
        return Classified {
            kind: FileKind::Composition,
            home: home_for(FileKind::Composition).to_string(),
            ok: true,
        };
    }
    if is_under_src_dir(&segs, "constants") {
        return Classified {
            kind: FileKind::Constant,
            home: home_for(FileKind::Constant).to_string(),
            ok: true,
        };
    }

    let _ = path_str; // kept for future path-string heuristics
    Classified {
        kind: FileKind::Other,
        home: String::new(),
        ok: true,
    }
}

/// Human-readable misplacement diagnostic naming the fixed home for `kind`.
///
/// Example: `stores live in `src/stores/` — move counter-store.ts.`
pub fn misplacement_message(kind: FileKind, path: &Path) -> String {
    let home = home_for(kind);
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("this file");
    let label = kind_label(kind);
    format!("{label} live in `{home}` — move {name}.")
}

/// One TypeScript `compilerOptions.paths` entry (deck resolve twin).
///
/// SPEC-PROJECT-STRUCTURE-1 §3 — shared so tsconfig emit and the deck
/// bundler cannot drift.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AtAliasMapping {
    /// `paths` key, e.g. `"@/*"` or `"@/stores/*"`.
    pub pattern: &'static str,
    /// Project-relative target, e.g. `"./src/*"` or `"./src/stores/*"`.
    pub target: &'static str,
}

/// SSOT `@/` alias table for project tsconfig `paths` + deck `resolve_id`.
///
/// SPEC-PROJECT-STRUCTURE-1 §3 fixed import roots:
/// - `@/components/*` → `./src/components/*`
/// - `@/compositions/*` → `./src/compositions/*`
/// - `@/constants/*` → `./src/constants/*`
/// - `@/stores/*` → `./src/stores/*`
/// - `@/actions/*` → `./src/actions/*`
/// - `@/themes/*` → `./themes/*`
///
/// Keep this table as the only source of truth — `emit_project_tsconfig_json`
/// and deck resolve must not invent parallel maps.
pub fn at_alias_paths() -> &'static [AtAliasMapping] {
    &[
        AtAliasMapping {
            pattern: "@/components/*",
            target: "./src/components/*",
        },
        AtAliasMapping {
            pattern: "@/compositions/*",
            target: "./src/compositions/*",
        },
        AtAliasMapping {
            pattern: "@/constants/*",
            target: "./src/constants/*",
        },
        AtAliasMapping {
            pattern: "@/stores/*",
            target: "./src/stores/*",
        },
        AtAliasMapping {
            pattern: "@/actions/*",
            target: "./src/actions/*",
        },
        AtAliasMapping {
            pattern: "@/themes/*",
            target: "./themes/*",
        },
    ]
}

/// Resolve a project `@/…` import specifier against `project_root` using
/// [`at_alias_paths`] as the only mapping table (SPEC-PROJECT-STRUCTURE-1 §3).
///
/// Returns a path under `project_root` when `specifier` matches an SSOT
/// pattern (e.g. `@/stores/tally` → `{root}/src/stores/tally`). Does **not**
/// require the target file to exist — pure alias math for tsconfig/deck parity.
///
/// Deck bundler `resolve_id` must call this (or a thin wrapper over the same
/// table) so emit paths and runtime resolve cannot drift.
pub fn resolve_at_alias(project_root: &Path, specifier: &str) -> Option<PathBuf> {
    // Project aliases only — framework packages (`@ship/…`) and relative
    // paths are not in this table.
    if !specifier.starts_with("@/") {
        return None;
    }

    // Longest pattern prefix wins when multiple entries could match (future
    // catch-all `@/*` coexists with fixed roots without drifting homes).
    let mut best: Option<(usize, PathBuf)> = None;
    for m in at_alias_paths() {
        let Some(resolved) = apply_alias_mapping(project_root, specifier, m) else {
            continue;
        };
        let score = m.pattern.len();
        match &best {
            Some((s, _)) if score <= *s => {}
            _ => best = Some((score, resolved)),
        }
    }
    best.map(|(_, p)| p)
}

/// Apply one SSOT mapping row: pattern `@/stores/*` + target `./src/stores/*`
/// maps `@/stores/tally` → `{root}/src/stores/tally`.
fn apply_alias_mapping(
    project_root: &Path,
    specifier: &str,
    m: &AtAliasMapping,
) -> Option<PathBuf> {
    let pattern = m.pattern;
    let target = m.target;

    if let Some(pat_prefix) = pattern.strip_suffix("/*") {
        // `@/stores` matches `@/stores` or `@/stores/…` only (not `@/storesX`).
        let rest = if specifier == pat_prefix {
            ""
        } else {
            let with_slash = format!("{pat_prefix}/");
            specifier.strip_prefix(&with_slash)?
        };
        if !alias_rest_stays_under_home(rest) {
            return None;
        }
        let tgt_base = target
            .strip_suffix("/*")
            .unwrap_or(target)
            .trim_start_matches("./");
        let path = if rest.is_empty() {
            project_root.join(tgt_base)
        } else {
            project_root.join(tgt_base).join(rest)
        };
        return Some(path);
    }

    // Exact pattern forms (no wildcard) — rare; keep for table completeness.
    if pattern == specifier {
        let tgt = target.trim_start_matches("./");
        return Some(project_root.join(tgt));
    }
    if pattern == "@/" || pattern == "@/*" {
        // Catch-all: `@/` + remainder → target base + remainder.
        let rest = specifier.strip_prefix("@/")?;
        if !alias_rest_stays_under_home(rest) {
            return None;
        }
        let tgt_base = target
            .strip_suffix("/*")
            .unwrap_or(target)
            .trim_start_matches("./");
        let path = if rest.is_empty() {
            project_root.join(tgt_base)
        } else {
            project_root.join(tgt_base).join(rest)
        };
        return Some(path);
    }

    None
}

/// Rest after a table prefix must stay inside that home.
///
/// `..` would walk out (`@/stores/../lib/helpers`). Extra slashes are not a
/// second spelling of a table row.
fn alias_rest_stays_under_home(rest: &str) -> bool {
    if rest.is_empty() {
        return true;
    }
    let rest = rest.replace('\\', "/");
    let rest = rest.trim_end_matches('/');
    if rest.is_empty() {
        return true;
    }
    rest.split('/').all(|part| !part.is_empty() && part != "..")
}

/// True when `specifier` is a `@/` import that matches [`at_alias_paths`].
///
/// `@/lib/…` is not a home. The table is the allowlist — there is no
/// catch-all `@/*`.
pub fn is_legal_at_alias(specifier: &str) -> bool {
    resolve_at_alias(Path::new(""), specifier).is_some()
}

/// Project-relative home fragments from the alias table (`src/stores`, `themes`, …).
pub fn alias_home_fragments() -> impl Iterator<Item = &'static str> {
    at_alias_paths()
        .iter()
        .map(|m| m.target.trim_start_matches("./").trim_end_matches("/*"))
}

/// Homes a controller may value-import via a relative path.
///
/// Stores are excluded: the legal form is `@/stores/…`, not `../stores/…`.
pub fn relative_value_import_homes() -> impl Iterator<Item = &'static str> {
    alias_home_fragments().filter(|h| *h != store_home_fragment())
}

/// Table target for `@/stores/*` (`src/stores`).
fn store_home_fragment() -> &'static str {
    at_alias_paths()
        .iter()
        .find(|m| m.pattern == "@/stores/*")
        .map(|m| m.target.trim_start_matches("./").trim_end_matches("/*"))
        .unwrap_or("src/stores")
}

/// True when `path` sits under a table home fragment (`src/stores`, `themes`, …).
pub fn path_is_under_home(path: &str, home: &str) -> bool {
    let p = path.replace('\\', "/");
    let home = home.trim_matches('/');
    if home.is_empty() {
        return false;
    }
    p.contains(&format!("/{home}/"))
        || p.starts_with(&format!("{home}/"))
        || p.ends_with(&format!("/{home}"))
        || p == home
}

/// True when `path` sits under the store home (`src/stores`).
pub fn path_is_under_store_home(path: &str) -> bool {
    path_is_under_home(path, store_home_fragment())
}

fn home_for(kind: FileKind) -> &'static str {
    match kind {
        FileKind::Route => "routes/",
        FileKind::Component => "src/components/",
        FileKind::Action => "src/actions/",
        FileKind::Store => "src/stores/",
        FileKind::Composition => "src/compositions/",
        FileKind::Constant => "src/constants/",
        FileKind::Theme => "themes/",
        FileKind::Other => "",
    }
}

fn kind_label(kind: FileKind) -> &'static str {
    match kind {
        FileKind::Route => "routes",
        FileKind::Component => "components",
        FileKind::Action => "actions",
        FileKind::Store => "stores",
        FileKind::Composition => "compositions",
        FileKind::Constant => "constants",
        FileKind::Theme => "themes",
        FileKind::Other => "files",
    }
}

fn path_as_str(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn path_segments(path: &Path) -> Vec<String> {
    path.components()
        .filter_map(|c| match c {
            Component::Normal(s) => Some(s.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect()
}

fn has_segment(segs: &[String], name: &str) -> bool {
    segs.iter().any(|s| s == name)
}

/// True when path has consecutive segments `src` then `dir` (e.g. `src/stores`).
fn is_under_src_dir(segs: &[String], dir: &str) -> bool {
    segs.windows(2).any(|w| w[0] == "src" && w[1] == dir)
}

fn is_sail(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()) == Some("sail")
}

/// The document file in a route folder. Everything else hyphenated next to it
/// is a tag for that route.
fn is_document_sail(path: &Path) -> bool {
    matches!(
        path.file_stem().and_then(|s| s.to_str()),
        Some("page" | "index" | "_layout")
    )
}

/// Conservative source scan: `defineStore(` as a call site (not a full AST).
///
/// Comments, string-literal bodies, and template-literal bodies are ignored.
/// Only real code tokens count — `"defineStore("` in a string must not trip Store.
fn source_has_define_store(source: &str) -> bool {
    let stripped = strip_ts_noise(source);
    stripped.contains("defineStore(")
}

/// Drop // and /* */ comments plus string/template interiors so shape scans
/// only see code tokens. Empty delimiters remain so adjacent tokens don't fuse.
///
/// Template `${…}` expressions are code and are retained (static template text
/// is not). Nested noise rules apply inside those expressions.
fn strip_ts_noise(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut out = String::with_capacity(source.len());
    let mut i = 0;
    while i < bytes.len() {
        i = emit_code_byte(bytes, i, &mut out);
    }
    out
}

/// Process one span starting at `i`: either skip noise, or emit a code byte.
/// Returns the next index to process.
fn emit_code_byte(bytes: &[u8], i: usize, out: &mut String) -> usize {
    let n = bytes.len();
    if i >= n {
        return n;
    }

    // line comment
    if bytes[i] == b'/' && i + 1 < n && bytes[i + 1] == b'/' {
        let mut j = i + 2;
        while j < n && bytes[j] != b'\n' {
            j += 1;
        }
        return j;
    }
    // block comment
    if bytes[i] == b'/' && i + 1 < n && bytes[i + 1] == b'*' {
        let mut j = i + 2;
        while j + 1 < n && !(bytes[j] == b'*' && bytes[j + 1] == b'/') {
            j += 1;
        }
        return (j + 2).min(n);
    }
    // string — keep delimiters, drop body
    if bytes[i] == b'"' || bytes[i] == b'\'' {
        let quote = bytes[i];
        out.push(quote as char);
        let mut j = i + 1;
        while j < n {
            let b = bytes[j];
            j += 1;
            if b == b'\\' && j < n {
                j += 1;
                continue;
            }
            if b == quote {
                out.push(quote as char);
                break;
            }
        }
        return j;
    }
    // template — keep delimiters + `${ expr }` code; drop static text
    if bytes[i] == b'`' {
        out.push('`');
        let mut j = i + 1;
        while j < n {
            let b = bytes[j];
            if b == b'\\' && j + 1 < n {
                j += 2;
                continue;
            }
            if b == b'`' {
                out.push('`');
                return j + 1;
            }
            if b == b'$' && j + 1 < n && bytes[j + 1] == b'{' {
                out.push_str("${");
                j += 2;
                let mut depth = 1usize;
                while j < n && depth > 0 {
                    if bytes[j] == b'{' {
                        // only count braces outside noise
                        out.push('{');
                        depth += 1;
                        j += 1;
                        continue;
                    }
                    if bytes[j] == b'}' {
                        depth -= 1;
                        out.push('}');
                        j += 1;
                        continue;
                    }
                    // Reuse noise rules for comments/strings/templates inside expr.
                    // `{` / `}` above short-circuit first so depth stays accurate for
                    // bare braces; emit_code_byte handles the rest (may emit `{`/`}`
                    // only when they appear as code — already handled).
                    let before = j;
                    j = emit_code_byte(bytes, j, out);
                    if j == before {
                        // safety: always advance
                        out.push(bytes[j] as char);
                        j += 1;
                    }
                }
                continue;
            }
            j += 1;
        }
        return j;
    }

    out.push(bytes[i] as char);
    i + 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn route_and_component_smoke() {
        let r = classify(Path::new("apps/x/routes/foo/page.sail"), None);
        assert_eq!(r.kind, FileKind::Route);
        assert!(r.ok);
        let local = classify(Path::new("apps/x/routes/foo/pricing-hero.sail"), None);
        assert_eq!(local.kind, FileKind::Component);
        assert!(local.ok);
        let c = classify(Path::new("apps/x/src/widget/widget.sail"), None);
        assert_eq!(c.kind, FileKind::Component);
        let k = classify(Path::new("apps/x/src/constants/pricing.ts"), None);
        assert_eq!(k.kind, FileKind::Constant);
        assert!(k.ok);
    }

    #[test]
    fn define_store_shape_wins() {
        let src = "export const x = defineStore('x', () => ({}))\n";
        let c = classify(Path::new("apps/x/src/compositions/x.ts"), Some(src));
        assert_eq!(c.kind, FileKind::Store);
        assert!(!c.ok);
        assert!(
            misplacement_message(c.kind, Path::new("apps/x/src/compositions/x.ts"))
                .contains("src/stores/")
        );
    }

    #[test]
    fn define_store_inside_string_does_not_trigger_store() {
        // Review bug: string body used to keep interiors → false Store.
        let src =
            "const note = \"call defineStore( later)\";\nexport function useFoo() { return 1 }\n";
        let c = classify(Path::new("apps/x/src/compositions/use-foo.ts"), Some(src));
        assert_eq!(
            c.kind,
            FileKind::Composition,
            "defineStore( only inside a string must not classify as Store"
        );
        assert!(c.ok);
        assert!(!source_has_define_store(src));
    }

    #[test]
    fn define_store_inside_template_static_does_not_trigger_store() {
        let src = "const tip = `use defineStore( in stores/`;\nexport const x = 1\n";
        assert!(
            !source_has_define_store(src),
            "template static text must not count as a call site"
        );
        let c = classify(Path::new("apps/x/src/compositions/tip.ts"), Some(src));
        assert_eq!(c.kind, FileKind::Composition);
    }

    #[test]
    fn define_store_in_comment_does_not_trigger() {
        let src = "// defineStore('x', () => ({}))\nexport const y = 1\n";
        assert!(!source_has_define_store(src));
        let src2 = "/* defineStore( */\nexport const y = 1\n";
        assert!(!source_has_define_store(src2));
    }

    #[test]
    fn real_define_store_call_still_detected() {
        let src = "export const x = defineStore('x', () => ({}))\n";
        assert!(source_has_define_store(src));
    }

    #[test]
    fn alias_table_is_the_legal_at_prefix_set() {
        for m in at_alias_paths() {
            let spec = format!("{}/x", m.pattern.trim_end_matches("/*"));
            assert!(is_legal_at_alias(&spec), "table row {spec} must resolve");
        }
        assert!(!is_legal_at_alias("@/lib/foo"), "@/lib is not a home");
        assert!(!is_legal_at_alias("@/serverfns/x"));
        assert!(!is_legal_at_alias("@/storesX"), "@/storesX is not @/stores");
        assert!(
            !is_legal_at_alias("@/storesX/tally"),
            "@/storesX/… is not @/stores/…"
        );
        assert!(
            !is_legal_at_alias("@/stores/../lib/helpers"),
            ".. must not walk out of a table home"
        );
    }

    #[test]
    fn relative_value_homes_exclude_stores_and_cover_table() {
        let homes: Vec<_> = relative_value_import_homes().collect();
        assert!(
            !homes.contains(&"src/stores"),
            "stores are alias-only: {homes:?}"
        );
        for expected in [
            "src/components",
            "src/compositions",
            "src/constants",
            "src/actions",
            "themes",
        ] {
            assert!(
                homes.contains(&expected),
                "relative homes must include {expected}: {homes:?}"
            );
        }
    }

    #[test]
    fn path_under_home_matches_fragments_not_serverfns() {
        assert!(path_is_under_home(
            "apps/x/src/actions/save.ts",
            "src/actions"
        ));
        assert!(path_is_under_home("apps/x/themes/acme.json", "themes"));
        assert!(path_is_under_store_home("apps/x/src/stores/tally.ts"));
        assert!(!path_is_under_store_home(
            "apps/x/src/compositions/tally.ts"
        ));
        assert!(!path_is_under_home(
            "apps/x/src/serverfns/run.ts",
            "src/actions"
        ));
        assert!(!path_is_under_home(
            "apps/x/src/serverfns/run.ts",
            "src/stores"
        ));
        assert!(
            !path_is_under_home("apps/x/src/storesX/tally.ts", "src/stores"),
            "src/storesX is not src/stores"
        );
    }
}
