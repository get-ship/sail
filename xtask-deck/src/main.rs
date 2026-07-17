//! `cargo run -p xtask-deck -- gen-icons` — bake the lucide SVG set into the
//! sui repo's deck-core assets directory (see SMELL comment in `pub fn run`
//! for the exact hardcoded path). The output is the embedded source the
//! daemon reads to answer `/_deck/icons/{name}.svg` per-icon. The
//! `crates/deck/deck-core/assets/` directory lives in the external sui repo
//! post-carve; the assets are committed source there.
//! This replaces the retired `scripts/codegen/generate-lucide-registry.mjs`.
//! `gen-icons` is the only task in this crate — codegen that crosses a
//! language boundary belongs here, not in `scripts/`.
//!
//! Run after a `lucide-static` version bump:
//!   bun install
//!   cargo run -p xtask-deck -- gen-icons
//!   git add <sail-repo>/crates/deck/deck-core/assets/lucide-icons.json && git commit
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

/// Walk the repo's `node_modules` and locate the `lucide-static/icons` dir.
/// Two layouts are accepted: the bun-store-pinned path
/// (`node_modules/.bun/lucide-static@<version>/node_modules/lucide-static/icons`)
/// and the npm-flat path (`node_modules/lucide-static/icons`). We pick the
/// first match — both ship the same SVG files.
fn locate_lucide_icons(repo_root: &Path) -> Result<PathBuf> {
    let flat = repo_root.join("node_modules/lucide-static/icons");
    if flat.is_dir() {
        return Ok(flat);
    }
    let bun_store = repo_root.join("node_modules/.bun");
    if let Ok(entries) = fs::read_dir(&bun_store) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            if let Some(s) = name.to_str() && s.starts_with("lucide-static@") {
                let candidate = entry.path().join("node_modules/lucide-static/icons");
                if candidate.is_dir() {
                    return Ok(candidate);
                }
            }
        }
    }
    bail!(
        "lucide-static/icons not found under node_modules — run `bun install` first \
         (looked under node_modules/lucide-static and node_modules/.bun/lucide-static@*)"
    );
}

/// Strip the outer `<svg …>` wrapper and collapse internal whitespace, matching
/// the format the embedded JSON has shipped since the registry landed. The
/// daemon wraps the fragment in its own canonical `<svg>` when serving.
fn extract_inner(svg: &str) -> Result<String> {
    // Drop `<!-- … -->` comments first (lucide source files have header comments).
    let mut s = String::with_capacity(svg.len());
    let bytes = svg.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i..].starts_with(b"<!--") {
            if let Some(end) = svg[i..].find("-->") {
                i += end + 3;
                continue;
            }
            bail!("unterminated comment");
        }
        s.push(bytes[i] as char);
        i += 1;
    }
    let stripped = s.trim();

    let open = stripped.find("<svg").context("no <svg in source")?;
    let open_end = stripped[open..]
        .find('>')
        .map(|n| open + n)
        .context("unterminated <svg open tag")?;
    let close_start = stripped.rfind("</svg>").context("no </svg> in source")?;
    if close_start <= open_end {
        bail!("malformed SVG (close before open end)");
    }
    let inner = stripped[open_end + 1..close_start].trim();
    // Collapse runs of whitespace to single spaces — the wire form has always
    // been one line, and shrinks the embed by ~40%.
    let mut out = String::with_capacity(inner.len());
    let mut in_ws = false;
    for c in inner.chars() {
        if c.is_whitespace() {
            if !in_ws {
                out.push(' ');
                in_ws = true;
            }
        } else {
            out.push(c);
            in_ws = false;
        }
    }
    Ok(out.trim().to_string())
}

pub fn run(repo_root: &Path) -> Result<()> {
    let icons_dir = locate_lucide_icons(repo_root)?;
    // Output lives inside this sui repo at the canonical deck-core assets
    // path. We join against `repo_root` instead of hardcoding an absolute
    // path so the tool is portable across checkouts and worktrees; pass
    // `repo_root` as the cwd (or override via SUI_GEN_ICONS_OUT) for a
    // custom layout.
    let out_path = std::env::var_os("SUI_GEN_ICONS_OUT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            repo_root.join("crates/deck/deck-core/assets/lucide-icons.json")
        });
    fs::create_dir_all(out_path.parent().unwrap())
        .with_context(|| format!("mkdir parent of {}", out_path.display()))?;

    // BTreeMap so the emitted JSON is stably ordered — diffs on lucide upgrades
    // show what icons were added/removed, not noise from HashMap rehashing.
    let mut out: BTreeMap<String, String> = BTreeMap::new();
    for entry in fs::read_dir(&icons_dir)
        .with_context(|| format!("read_dir {}", icons_dir.display()))?
    {
        let entry = entry?;
        let name = entry.file_name();
        let name_s = name.to_string_lossy();
        let Some(stem) = name_s.strip_suffix(".svg") else {
            continue;
        };
        let raw = fs::read_to_string(entry.path())
            .with_context(|| format!("read {}", entry.path().display()))?;
        let inner = extract_inner(&raw)
            .with_context(|| format!("extract inner from {}", entry.path().display()))?;
        out.insert(stem.to_string(), inner);
    }

    if out.is_empty() {
        bail!("no icons read from {} (empty dir?)", icons_dir.display());
    }

    // Compact JSON — same shape `<sail-icon>` has always seen. The size
    // (~410KB) is the daemon's embed cost; clients fetch one icon at a time
    // via `/_deck/icons/{name}.svg`, never the whole map.
    let json = serde_json::to_string(&out).context("serialize icons JSON")?;
    fs::write(&out_path, json).with_context(|| format!("write {}", out_path.display()))?;

    println!(
        "wrote {} icons to {} ({})",
        out.len(),
        out_path.display(),
        format_size(fs::metadata(&out_path)?.len()),
    );
    Ok(())
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

fn main() {
    let repo_root = std::path::Path::new(".");
    if let Err(e) = run(repo_root) {
        eprintln!("gen-icons failed: {e:#}");
        std::process::exit(1);
    }
}
