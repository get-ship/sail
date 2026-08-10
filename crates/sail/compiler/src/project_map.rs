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

use std::path::{Component, Path};

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
fn source_has_define_store(source: &str) -> bool {
    // Strip // and /* */ comments roughly so commented-out defineStore does not trip.
    let stripped = strip_ts_comments(source);
    stripped.contains("defineStore(")
}

fn strip_ts_comments(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let bytes = source.as_bytes();
    let mut i = 0;
    let n = bytes.len();
    while i < n {
        // line comment
        if bytes[i] == b'/' && i + 1 < n && bytes[i + 1] == b'/' {
            i += 2;
            while i < n && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        // block comment
        if bytes[i] == b'/' && i + 1 < n && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < n && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i = (i + 2).min(n);
            continue;
        }
        // string literals — keep structure, skip content that could false-positive
        if bytes[i] == b'"' || bytes[i] == b'\'' {
            let quote = bytes[i];
            out.push(quote as char);
            i += 1;
            while i < n {
                let b = bytes[i];
                out.push(b as char);
                i += 1;
                if b == b'\\' && i < n {
                    out.push(bytes[i] as char);
                    i += 1;
                    continue;
                }
                if b == quote {
                    break;
                }
            }
            continue;
        }
        // template literals
        if bytes[i] == b'`' {
            out.push('`');
            i += 1;
            while i < n {
                let b = bytes[i];
                out.push(b as char);
                i += 1;
                if b == b'\\' && i < n {
                    out.push(bytes[i] as char);
                    i += 1;
                    continue;
                }
                if b == b'`' {
                    break;
                }
            }
            continue;
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
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
}
