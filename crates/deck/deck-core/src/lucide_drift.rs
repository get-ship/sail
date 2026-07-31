//! Freshness drift test: committed `lucide-icons.json` must match a regen
//! from `node_modules/lucide-static` (when installed). CI / local: after a
//! lucide bump, `cargo xtask gen-icons` and commit.

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .canonicalize()
            .expect("workspace root")
    }

    fn locate_lucide_icons(repo_root: &Path) -> Option<PathBuf> {
        let flat = repo_root.join("node_modules/lucide-static/icons");
        if flat.is_dir() {
            return Some(flat);
        }
        let bun_store = repo_root.join("node_modules/.bun");
        if let Ok(entries) = std::fs::read_dir(&bun_store) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                if let Some(s) = name.to_str()
                    && s.starts_with("lucide-static@")
                {
                    let candidate = entry.path().join("node_modules/lucide-static/icons");
                    if candidate.is_dir() {
                        return Some(candidate);
                    }
                }
            }
        }
        None
    }

    fn extract_inner(svg: &str) -> Option<String> {
        let mut s = String::with_capacity(svg.len());
        let bytes = svg.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i..].starts_with(b"<!--") {
                if let Some(end) = svg[i..].find("-->") {
                    i += end + 3;
                    continue;
                }
                return None;
            }
            s.push(bytes[i] as char);
            i += 1;
        }
        let stripped = s.trim();
        let open = stripped.find("<svg")?;
        let open_end = stripped[open..].find('>').map(|n| open + n)?;
        let close_start = stripped.rfind("</svg>")?;
        if close_start <= open_end {
            return None;
        }
        let inner = stripped[open_end + 1..close_start].trim();
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
        Some(out.trim().to_string())
    }

    #[test]
    fn lucide_icons_json_matches_lucide_static_when_installed() {
        let root = repo_root();
        let Some(icons_dir) = locate_lucide_icons(&root) else {
            // No node_modules in minimal CI slices — skip rather than fail.
            eprintln!("lucide-static not installed; skipping freshness drift check");
            return;
        };

        let mut expected: BTreeMap<String, String> = BTreeMap::new();
        for entry in std::fs::read_dir(&icons_dir).expect("read lucide-static icons") {
            let entry = entry.expect("entry");
            let name = entry.file_name();
            let name_s = name.to_string_lossy();
            let Some(stem) = name_s.strip_suffix(".svg") else {
                continue;
            };
            let raw = std::fs::read_to_string(entry.path()).expect("read svg");
            let inner = extract_inner(&raw).expect("extract inner");
            expected.insert(stem.to_string(), inner);
        }
        assert!(
            !expected.is_empty(),
            "lucide-static icons dir empty at {}",
            icons_dir.display()
        );

        let committed_path = root.join("crates/deck/deck-core/assets/lucide-icons.json");
        let committed_raw =
            std::fs::read_to_string(&committed_path).expect("read committed lucide-icons.json");
        let committed: BTreeMap<String, String> =
            serde_json::from_str(&committed_raw).expect("parse committed lucide-icons.json");

        assert_eq!(
            committed.len(),
            expected.len(),
            "icon count drift: committed={} lucide-static={}. Run `cargo xtask gen-icons`",
            committed.len(),
            expected.len()
        );
        for (name, inner) in &expected {
            match committed.get(name) {
                Some(got) if got == inner => {}
                Some(_) => panic!(
                    "lucide icon `{name}` body drifted from lucide-static. \
                     Run `cargo xtask gen-icons` and commit."
                ),
                None => panic!(
                    "lucide icon `{name}` missing from committed registry. \
                     Run `cargo xtask gen-icons` and commit."
                ),
            }
        }
    }

    #[test]
    fn lucide_icon_svg_helper_covers_known_name() {
        let svg = crate::lucide_icon_svg("search").expect("search in pack");
        assert!(svg.contains("<svg"), "{svg}");
        assert!(svg.contains("viewBox=\"0 0 24 24\""), "{svg}");
        assert!(crate::lucide_icon_names().len() > 100);
    }
}
