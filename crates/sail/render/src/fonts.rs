//! Load the token woff2 fonts into a `takumi::Fonts` registry.
//!
//! Framework defaults live under `packages/sail/tokens/src/fonts/`. App-vendored
//! fonts (SailTheme/FontAdd) live under `<project>/fonts/` — same path the
//! bundler stages and Studio specimen should use (no second font pipeline).

use std::path::Path;

use anyhow::{Context, Result};
use takumi::prelude::{FontResource, Fonts};

const FONT_FILES: &[(&str, &str)] = &[
    ("DM Sans", "dm-sans-latin-wght-normal.woff2"),
    ("JetBrains Mono", "jetbrains-mono-latin-wght-normal.woff2"),
    ("Syne", "syne-latin-wght-normal.woff2"),
    ("Press Start 2P", "press-start-2p-latin-normal.woff2"),
];

/// Load the framework fonts from `fonts_dir` into a `takumi::Fonts` registry.
pub fn load_fonts(fonts_dir: &Path) -> Result<Fonts> {
    let mut fonts = Fonts::default();
    for (family_name, file_name) in FONT_FILES {
        let path = fonts_dir.join(file_name);
        let bytes =
            std::fs::read(&path).with_context(|| format!("reading font {}", path.display()))?;
        fonts
            .register(FontResource::new(bytes))
            .with_context(|| format!("registering font {} ({})", file_name, family_name))?;
    }
    Ok(fonts)
}

/// Merge app-vendored woff2 files from `<project>/fonts/` (FontAdd) into an
/// existing registry. Skips `vendor-manifest` / non-woff2. Specimen/Studio use
/// the same directory the bundler stages — one font path.
pub fn load_app_fonts(fonts: &mut Fonts, app_fonts_dir: &Path) -> Result<usize> {
    if !app_fonts_dir.is_dir() {
        return Ok(0);
    }
    let mut n = 0;
    for entry in std::fs::read_dir(app_fonts_dir)
        .with_context(|| format!("read_dir {}", app_fonts_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("woff2") {
            continue;
        }
        let bytes =
            std::fs::read(&path).with_context(|| format!("reading app font {}", path.display()))?;
        fonts
            .register(FontResource::new(bytes))
            .with_context(|| format!("registering app font {}", path.display()))?;
        n += 1;
    }
    Ok(n)
}

/// Load fonts from embedded bytes for tests / deterministic CI.
pub fn load_fonts_embedded() -> Result<Fonts> {
    let mut fonts = Fonts::default();
    let embedded: &[(&str, &[u8])] = &[
        (
            "DM Sans",
            include_bytes!("../tests/fixtures/fonts/dm-sans-latin-wght-normal.woff2"),
        ),
        (
            "JetBrains Mono",
            include_bytes!("../tests/fixtures/fonts/jetbrains-mono-latin-wght-normal.woff2"),
        ),
        (
            "Syne",
            include_bytes!("../tests/fixtures/fonts/syne-latin-wght-normal.woff2"),
        ),
        (
            "Press Start 2P",
            include_bytes!("../tests/fixtures/fonts/press-start-2p-latin-normal.woff2"),
        ),
    ];
    for (family_name, bytes) in embedded {
        fonts
            .register(FontResource::new(bytes.to_vec()))
            .with_context(|| format!("registering embedded font {family_name}"))?;
    }
    Ok(fonts)
}
