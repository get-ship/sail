//! Project-map file-kind classification (SPEC-PROJECT-STRUCTURE-1 §2).
//!
//! Single SSOT: path + shape signals → [`FileKind`] + fixed home.
//! Misplacement diagnostics name the right home.
//!
//! # Retirement plan (dual-path detectors)
//!
//! Deck `detect_mode` (hyphen heuristics) and host `route_params_for`
//! grants should eventually call [`classify`] for Route/Component.
//! This module does **not** change those call sites in S1.1 — pure
//! library API + unit tests only.

use std::path::{Component, Path, PathBuf};

/// Kind of authoring file under the fixed project map.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileKind {
    /// `.sail` under `routes/` (page.sail grant shape).
    Route,
    /// `.sail` outside `routes/`.
    Component,
    /// Action module — path `src/actions/**` (or action-shaped export).
    Action,
    /// Store — `defineStore(…)` export and/or path `src/stores/**`.
    Store,
    /// Composition — pure signal helpers under `src/compositions/**`.
    Composition,
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
/// - `.sail` + path contains a `routes` segment → Route
/// - `.sail` elsewhere → Component
/// - `defineStore(` in source → Store (even under compositions — misplacement)
/// - path under `src/actions/` → Action
/// - path under `src/stores/` → Store
/// - path under `src/compositions/` without defineStore → Composition
/// - path under `themes/` → Theme
pub fn classify(path: &Path, source: Option<&str>) -> Classified {
    let path_str = path_as_str(path);
    let segs = path_segments(path);

    // Theme: under themes/
    if has_segment(&segs, "themes") {
        return Classified {
            kind: FileKind::Theme,
            home: home_for(FileKind::Theme).to_string(),
            ok: true,
        };
    }

    // .sail: routes/ → Route, else Component
    if is_sail(path) {
        if has_segment(&segs, "routes") {
            return Classified {
                kind: FileKind::Route,
                home: home_for(FileKind::Route).to_string(),
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
///
/// S1.3 product fills the body; empty stub keeps gate tests compile-red →
/// runtime-red.
pub fn resolve_at_alias(project_root: &Path, specifier: &str) -> Option<PathBuf> {
    let _ = (project_root, specifier);
    None
}

fn home_for(kind: FileKind) -> &'static str {
    match kind {
        FileKind::Route => "routes/",
        FileKind::Component => "src/components/",
        FileKind::Action => "src/actions/",
        FileKind::Store => "src/stores/",
        FileKind::Composition => "src/compositions/",
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
    segs.windows(2)
        .any(|w| w[0] == "src" && w[1] == dir)
}

fn is_sail(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()) == Some("sail")
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
        let c = classify(Path::new("apps/x/src/widget/widget.sail"), None);
        assert_eq!(c.kind, FileKind::Component);
    }

    #[test]
    fn define_store_shape_wins() {
        let src = "export const x = defineStore('x', () => ({}))\n";
        let c = classify(Path::new("apps/x/src/compositions/x.ts"), Some(src));
        assert_eq!(c.kind, FileKind::Store);
        assert!(!c.ok);
        assert!(misplacement_message(c.kind, Path::new("apps/x/src/compositions/x.ts"))
            .contains("src/stores/"));
    }

    #[test]
    fn define_store_inside_string_does_not_trigger_store() {
        // Review bug: string body used to keep interiors → false Store.
        let src = "const note = \"call defineStore( later)\";\nexport function useFoo() { return 1 }\n";
        let c = classify(
            Path::new("apps/x/src/compositions/use-foo.ts"),
            Some(src),
        );
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
}
