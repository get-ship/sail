//! Render a `.sail` page to a PNG screenshot without a browser.
//!
//! The renderer reuses the same preprocess + style resolution path as the
//! compiler's layout diagnostics, then converts the resolved styles into a
//! `takumi` node tree and rasterizes it.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

pub mod convert;
pub mod fonts;
pub mod pipeline;
pub mod placeholder;

pub use takumi::prelude::Bitmap;

/// Options controlling the render.
#[derive(Debug, Clone)]
pub struct RenderOptions {
    /// Viewport size in CSS pixels (width, height).
    pub viewport: (u32, u32),
    /// Directory containing the token woff2 fonts. If `None`, the renderer
    /// looks for fonts next to the workspace under `packages/sail/tokens/src/fonts/`.
    pub fonts_dir: Option<PathBuf>,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self { viewport: (1440, 900), fonts_dir: None }
    }
}

/// Output of a successful render.
#[derive(Debug)]
pub struct RenderOutput {
    pub image: Bitmap,
    pub viewport: (u32, u32),
}

/// Render `.sail` source text to an in-memory image.
pub fn render_source(source: &str, opts: RenderOptions) -> Result<RenderOutput> {
    let fonts_dir = opts
        .fonts_dir
        .or_else(default_fonts_dir)
        .context("could not locate token fonts; pass fonts_dir explicitly")?;

    let image = pipeline::render(source, opts.viewport, &fonts_dir)
        .context("rendering .sail source")?;

    Ok(RenderOutput { image, viewport: opts.viewport })
}

/// Render a `.sail` file to an in-memory image.
pub fn render_file(path: &Path, opts: RenderOptions) -> Result<RenderOutput> {
    let source = std::fs::read_to_string(path)
        .with_context(|| format!("reading {}", path.display()))?;
    render_source(&source, opts)
}

/// Write a rendered image to a PNG file.
pub fn write_png(image: &Bitmap, path: &Path) -> Result<()> {
    let mut file = std::fs::File::create(path)
        .with_context(|| format!("creating {}", path.display()))?;
    takumi::write_image(image, &mut file, takumi::prelude::OutputFormat::Png)
        .with_context(|| format!("writing PNG to {}", path.display()))?;
    Ok(())
}

fn default_fonts_dir() -> Option<PathBuf> {
    // When running from the workspace root, the fonts are at:
    // packages/sail/tokens/src/fonts/
    let workspace_relative = PathBuf::from("packages/sail/tokens/src/fonts");
    if workspace_relative.is_dir() {
        return Some(workspace_relative);
    }
    // If the crate is inside the workspace, walk up to find it.
    let mut dir = std::env::current_dir().ok()?;
    for _ in 0..6 {
        let candidate = dir.join("packages/sail/tokens/src/fonts");
        if candidate.is_dir() {
            return Some(candidate);
        }
        if !dir.pop() {
            break;
        }
    }
    None
}
