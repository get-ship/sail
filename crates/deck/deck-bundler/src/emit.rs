//! Deck-owned asset emission (`deck emit`).
//!
//! A consumer app (e.g. apps/site's static Astro build) needs deck's baked
//! bundles next to its own static output so the browser can load them without
//! the daemon. Historically the consumer hand-copied deck's build output with
//! a hardcoded `.deck/` path AND re-implemented deck's content-hashed chunk
//! chasing. That is deck's job: deck knows where each target lands
//! (`BundleLayout::assets_dir_for`) and how its lazy chunks are named.
//!
//! `emit_targets` resolves each requested target's output file via the same
//! layout the bundler writes through, copies it into `out_dir`, then follows
//! its relative `import("./chunk.js")` graph and copies the whole closure —
//! so a consumer never encodes deck's internal output layout.

use std::collections::HashSet;
use std::path::Path;

use anyhow::{Context, Result, bail};

use crate::targets::BundleLayout;

/// Match relative sibling-chunk specifiers: `"./Geometry-AbC1.js"`,
/// `'./worker.js'`. Captures the bare filename. The bundler emits code-split
/// lazy chunks as same-dir relative imports, so the closure lives beside the
/// entry file.
fn chunk_names(src: &str) -> impl Iterator<Item = String> + '_ {
    src.match_indices("./").filter_map(|(i, _)| {
        // Require the `./` to be inside a quote (import/specifier), and
        // capture up to the closing quote.
        let before = src[..i].chars().next_back();
        if !matches!(before, Some('"') | Some('\'') | Some('`')) {
            return None;
        }
        let rest = &src[i + 2..];
        let end = rest.find(['"', '\'', '`'])?;
        let name = &rest[..end];
        if name.ends_with(".js")
            && name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
        {
            Some(name.to_string())
        } else {
            None
        }
    })
}

/// Copy `file` from `src_dir` into `out_dir`, then recursively copy every
/// relative chunk it imports (from the same `src_dir`). Returns the count of
/// files written. `seen` dedupes the closure across calls.
fn copy_with_closure(
    src_dir: &Path,
    file: &str,
    out_dir: &Path,
    seen: &mut HashSet<String>,
) -> Result<usize> {
    if !seen.insert(file.to_string()) {
        return Ok(0);
    }
    let src = src_dir.join(file);
    let body =
        std::fs::read_to_string(&src).with_context(|| format!("read bundle {}", src.display()))?;
    let dst = out_dir.join(file);
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    crate::atomic::write(&dst, &body).with_context(|| format!("write {}", dst.display()))?;
    let mut n = 1;
    let chunks: Vec<String> = chunk_names(&body).collect();
    for chunk in chunks {
        if src_dir.join(&chunk).exists() {
            n += copy_with_closure(src_dir, &chunk, out_dir, seen)?;
        }
    }
    Ok(n)
}

/// Emit each named target (and its lazy-chunk closure) into `out_dir`.
///
/// Target names are the logical names deck knows: framework singletons
/// (`sail-runtime`, `sail-dom`, `sail-sdk-client`) and consumer targets declared
/// in `deck.jsonc` (`sail-marketing`, `sail-arcade`, `ship-code-editor`, …).
/// The output path of each is resolved through `BundleLayout` — the consumer
/// passes names, never paths.
pub fn emit_targets(layout: &BundleLayout, names: &[String], out_dir: &Path) -> Result<usize> {
    std::fs::create_dir_all(out_dir)
        .with_context(|| format!("create out dir {}", out_dir.display()))?;
    let all = layout.all_targets();
    let mut total = 0;
    let mut seen = HashSet::new();
    for name in names {
        let target = all.iter().find(|t| &t.name == name).ok_or_else(|| {
            anyhow::anyhow!(
                "unknown deck target `{name}` — known targets: {}",
                all.iter()
                    .map(|t| t.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })?;
        let src_dir = layout.assets_dir_for(name);
        let src = src_dir.join(&target.out_file);
        if !src.exists() {
            bail!(
                "target `{name}` not built ({} missing) — run `deck-bake build-assets` first",
                src.display()
            );
        }
        // out_file may itself be nested (e.g. `routes/git/route.js`); the closure
        // copy preserves that subpath under out_dir.
        total += copy_with_closure(&src_dir, &target.out_file, out_dir, &mut seen)?;
    }
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DeckConfig;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn chunk_names_captures_relative_siblings() {
        let src = r#"import("./Geometry-Ab12.js");const x='./plain.js';let y="./no-ext";"#;
        let names: Vec<String> = chunk_names(src).collect();
        assert!(names.contains(&"Geometry-Ab12.js".to_string()));
        assert!(names.contains(&"plain.js".to_string()));
        assert!(!names.iter().any(|n| n == "no-ext"));
    }

    #[test]
    fn emits_target_with_chunk_closure() {
        let tmp = tempdir().unwrap();
        let repo = tmp.path().to_path_buf();
        // sail-marketing is a consumer target → lands in `.sail/` (generate).
        std::fs::create_dir_all(repo.join(".sail")).unwrap();
        std::fs::write(
            repo.join(".sail/sail-marketing.js"),
            r#"import("./Chunk-X.js");"#,
        )
        .unwrap();
        std::fs::write(repo.join(".sail/Chunk-X.js"), "export const x = 1;").unwrap();
        // Declare the consumer bundle via per-package self-declaration; the
        // bundler discovers it from the workspace (no central deck.jsonc).
        std::fs::write(
            repo.join("package.json"),
            r#"{ "workspaces": ["packages/*"] }"#,
        )
        .unwrap();
        std::fs::create_dir_all(repo.join("packages/site")).unwrap();
        std::fs::write(
            repo.join("packages/site/package.json"),
            r#"{ "name": "@ship/site", "deck": { "bundles": [ { "name": "sail-marketing", "entry": "x.ts" } ] } }"#,
        )
        .unwrap();

        let layout = BundleLayout::with_config(repo.clone(), None, DeckConfig::defaults());
        let out: PathBuf = tmp.path().join("public");
        let n = emit_targets(&layout, &["sail-marketing".into()], &out).unwrap();
        assert_eq!(n, 2, "bundle + its chunk");
        assert!(out.join("sail-marketing.js").exists());
        assert!(out.join("Chunk-X.js").exists());
    }
}
