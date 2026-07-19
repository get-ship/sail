//! Load the token woff2 fonts into a `takumi::Fonts` registry.

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
        let bytes = std::fs::read(&path)
            .with_context(|| format!("reading font {}", path.display()))?;
        fonts
            .register(FontResource::new(bytes))
            .with_context(|| format!("registering font {} ({})", file_name, family_name))?;
    }
    Ok(fonts)
}

/// Load fonts from embedded bytes for tests / deterministic CI.
pub fn load_fonts_embedded() -> Result<Fonts> {
    let mut fonts = Fonts::default();
    let embedded: &[(&str, &[u8])] = &[
        ("DM Sans", include_bytes!("../tests/fixtures/fonts/dm-sans-latin-wght-normal.woff2")),
        ("JetBrains Mono", include_bytes!("../tests/fixtures/fonts/jetbrains-mono-latin-wght-normal.woff2")),
        ("Syne", include_bytes!("../tests/fixtures/fonts/syne-latin-wght-normal.woff2")),
        ("Press Start 2P", include_bytes!("../tests/fixtures/fonts/press-start-2p-latin-normal.woff2")),
    ];
    for (family_name, bytes) in embedded {
        fonts
            .register(FontResource::new(bytes.to_vec()))
            .with_context(|| format!("registering embedded font {family_name}"))?;
    }
    Ok(fonts)
}
