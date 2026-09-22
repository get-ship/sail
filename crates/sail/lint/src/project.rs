//! A Sail **application** project. One detector.
//!
//! Marker: `sail.config.jsonc` or `sail.config.json` (jsonc wins in a
//! directory). Nearest config wins. `routes/` is a FileKind, not a
//! discovery tie-break. Never Cargo. Never `package.json` alone.
//!
//! Workspace / `@ship` bake roots live in `deck_bundler::paths`, not here.

use std::path::{Path, PathBuf};

use crate::config::{CONFIG_FILES, dir_has_config};

/// Ancestor hops for [`Project::discover`] and deck detect. Same cap so
/// those walks cannot disagree.
pub const DISCOVER_MAX_HOPS: usize = 32;

/// Directory that owns the app's `sail.config` file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    root: PathBuf,
    config_path: PathBuf,
}

impl Project {
    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    /// Walk up from `from` (file → parent). Nearest `sail.config.jsonc` /
    /// `.json` wins. `routes/` does not outrank a closer config.
    pub fn discover(from: &Path) -> Option<Self> {
        let abs = if from.is_absolute() {
            from.to_path_buf()
        } else {
            std::env::current_dir().ok()?.join(from)
        };
        let mut cur = if abs.is_file() {
            abs.parent()?.to_path_buf()
        } else {
            abs
        };
        for _ in 0..DISCOVER_MAX_HOPS {
            if let Some(config_path) = config_file_in(&cur) {
                return Some(Self {
                    root: cur,
                    config_path,
                });
            }
            match cur.parent() {
                Some(p) => cur = p.to_path_buf(),
                None => break,
            }
        }
        None
    }
}

/// Config file in `dir` if any. jsonc before json.
pub fn config_file_in(dir: &Path) -> Option<PathBuf> {
    for name in CONFIG_FILES {
        let p = dir.join(name);
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

/// True when `dir` itself is a Sail project root.
pub fn is_project_dir(dir: &Path) -> bool {
    dir_has_config(dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn jsonc_beside_routes_is_the_project() {
        let tmp = tempfile::tempdir().unwrap();
        let app = tmp.path().join("hello");
        fs::create_dir_all(app.join("routes")).unwrap();
        fs::write(app.join("sail.config.jsonc"), "{}\n").unwrap();
        fs::write(app.join("routes/page.sail"), "<p>x</p>\n").unwrap();
        let p = Project::discover(&app.join("routes/page.sail")).unwrap();
        assert_eq!(p.root(), app.as_path());
        assert_eq!(p.config_path(), app.join("sail.config.jsonc"));
    }

    #[test]
    fn cargo_toml_is_not_a_marker() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("Cargo.toml"), "[workspace]\n").unwrap();
        assert!(Project::discover(tmp.path()).is_none());
    }

    #[test]
    fn isolated_tempdir_with_only_cargo_toml_is_not_a_project() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(
            tmp.path().join("Cargo.toml"),
            "[package]\nname = \"solo\"\n",
        )
        .unwrap();
        assert!(
            Project::discover(tmp.path()).is_none(),
            "Cargo.toml alone is not a Sail Project"
        );
    }

    #[test]
    fn nested_cwd_still_finds_app() {
        let tmp = tempfile::tempdir().unwrap();
        let app = tmp.path().join("hello");
        fs::create_dir_all(app.join("routes/about")).unwrap();
        fs::write(app.join("sail.config.jsonc"), "{}\n").unwrap();
        let p = Project::discover(&app.join("routes/about")).unwrap();
        assert_eq!(p.root(), app.as_path());
    }

    #[test]
    fn jsonc_wins_over_json_in_same_dir() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("sail.config.json"), "{}\n").unwrap();
        fs::write(tmp.path().join("sail.config.jsonc"), "{}\n").unwrap();
        let p = Project::discover(tmp.path()).unwrap();
        assert_eq!(p.config_path(), tmp.path().join("sail.config.jsonc"));
    }

    #[test]
    fn nearest_config_wins_even_without_routes() {
        let tmp = tempfile::tempdir().unwrap();
        let app = tmp.path().join("hello");
        fs::create_dir_all(app.join("routes")).unwrap();
        fs::write(app.join("sail.config.jsonc"), "{}\n").unwrap();
        let foo = app.join("packages/foo");
        fs::create_dir_all(&foo).unwrap();
        fs::write(foo.join("sail.config.json"), "{}\n").unwrap();
        let p = Project::discover(&foo).unwrap();
        assert_eq!(
            p.root(),
            foo.as_path(),
            "routes/ is not a tie-break over a closer config"
        );
        assert_eq!(p.config_path(), foo.join("sail.config.json"));
    }
}
