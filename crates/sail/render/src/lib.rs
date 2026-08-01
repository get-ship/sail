//! Render a `.sail` page to a painted, themed PNG screenshot without a browser.
//!
//! Pipeline (SSR-RENDER): controller S-tier initial projection → preprocess
//! (HTML+CSS) → taffy layout → token/theme paint → takumi raster. Regions join
//! to span-keyed node ids; residual dynamic/unknown subtrees are listed honestly
//! in sidecar metadata. See `docs/SSR-RENDER.md`.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

pub mod convert;
pub mod fonts;
pub mod identity;
pub mod paint;
pub mod pipeline;
pub mod placeholder;
pub mod ssr_project;
pub mod theme;

pub use identity::{
    DynamicRegion, Region, RegionStyle, absolute_location, is_unjoined_id,
};
pub use pipeline::PipelineOutput;
pub use takumi::prelude::Bitmap;
pub use theme::{ThemePalette, ThemeSelect};

/// Options controlling the render.
#[derive(Debug, Clone)]
pub struct RenderOptions {
    /// Viewport size in CSS pixels (width, height).
    pub viewport: (u32, u32),
    /// Directory containing the token woff2 fonts. If `None`, the renderer
    /// looks for fonts next to the workspace under `packages/sail/tokens/src/fonts/`.
    pub fonts_dir: Option<PathBuf>,
    /// App-vendored fonts dir (`<project>/fonts` from FontAdd). Same path as Studio/bundler.
    pub app_fonts_dir: Option<PathBuf>,
    /// Theme id (e.g. `ship`) and mode (`light`/`dark`).
    pub theme: Option<String>,
    /// Explicit mode override (`light` / `dark`).
    pub mode: Option<String>,
    /// Workspace root for theme file lookup.
    pub workspace: Option<PathBuf>,
    /// Workspace-relative file label for region node ids.
    pub file: Option<String>,
    /// Test hook: remove a token mapping before paint (golden fail probe).
    pub drop_bg_token: Option<String>,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            viewport: (1440, 900),
            fonts_dir: None,
            app_fonts_dir: None,
            theme: None,
            mode: None,
            workspace: None,
            file: None,
            drop_bg_token: None,
        }
    }
}

/// Output of a successful render.
#[derive(Debug)]
pub struct RenderOutput {
    pub image: Bitmap,
    pub viewport: (u32, u32),
    pub regions: Vec<Region>,
    pub dynamics: Vec<DynamicRegion>,
    pub theme: ThemeSelect,
    pub ms: u64,
    /// HTML after expression stripping (for leak assertions).
    pub html: String,
    /// Author tree node ids (SailInspect-compatible) used for region join.
    pub tree_ids: Vec<String>,
    /// SSR projection notes (which S-tier bindings/branches resolved).
    pub projected: Vec<String>,
}

/// Render `.sail` source text to an in-memory image + metadata.
pub fn render_source(source: &str, opts: RenderOptions) -> Result<RenderOutput> {
    let fonts_dir = opts
        .fonts_dir
        .clone()
        .or_else(default_fonts_dir)
        .context("could not locate token fonts; pass fonts_dir explicitly")?;

    let theme = ThemeSelect::parse(opts.theme.as_deref(), opts.mode.as_deref());
    let pipe = pipeline::PipelineOptions {
        viewport: opts.viewport,
        fonts_dir,
        app_fonts_dir: opts.app_fonts_dir,
        theme,
        workspace: opts.workspace,
        file: opts.file,
        drop_bg_token: opts.drop_bg_token,
    };

    let out = pipeline::render(source, &pipe).context("rendering .sail source")?;

    Ok(RenderOutput {
        image: out.image,
        viewport: out.viewport,
        regions: out.regions,
        dynamics: out.dynamics,
        theme: out.theme,
        ms: out.ms,
        html: out.html,
        tree_ids: out.tree.nodes.iter().map(|n| n.id.clone()).collect(),
        projected: out.projected,
    })
}

/// Render a `.sail` file to an in-memory image + metadata.
pub fn render_file(path: &Path, opts: RenderOptions) -> Result<RenderOutput> {
    let source =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let mut opts = opts;
    if opts.file.is_none() {
        opts.file = Some(path.display().to_string());
    }
    render_source(&source, opts)
}

/// Write a rendered image to a PNG file.
pub fn write_png(image: &Bitmap, path: &Path) -> Result<()> {
    let mut file =
        std::fs::File::create(path).with_context(|| format!("creating {}", path.display()))?;
    takumi::write_image(image, &mut file, takumi::prelude::OutputFormat::Png)
        .with_context(|| format!("writing PNG to {}", path.display()))?;
    Ok(())
}

/// Encode PNG to bytes (ops/MCP base64 path).
pub fn png_bytes(image: &Bitmap) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    takumi::write_image(image, &mut buf, takumi::prelude::OutputFormat::Png)
        .context("encoding PNG")?;
    Ok(buf)
}

/// Compare two RGBA bitmaps with per-channel tolerance. Returns `Ok(())` or
/// writes a diff PNG when `diff_out` is Some.
pub fn assert_pixel_match(
    actual: &Bitmap,
    expected_png: &[u8],
    tolerance: u8,
    diff_out: Option<&Path>,
) -> Result<()> {
    let expected = decode_png(expected_png).context("decode golden PNG")?;
    if actual.width() != expected.0 || actual.height() != expected.1 {
        anyhow::bail!(
            "size mismatch: actual {}x{} vs golden {}x{}",
            actual.width(),
            actual.height(),
            expected.0,
            expected.1
        );
    }
    let aw = actual.width() as usize;
    let ah = actual.height() as usize;
    let a = actual.as_raw();
    let e = &expected.2;
    let mut max_delta = 0u8;
    let mut mismatches = 0usize;
    let mut diff = vec![0u8; aw * ah * 4];
    for y in 0..ah {
        for x in 0..aw {
            let i = (y * aw + x) * 4;
            let mut dmax = 0u8;
            for c in 0..4 {
                let d = a[i + c].abs_diff(e[i + c]);
                dmax = dmax.max(d);
            }
            max_delta = max_delta.max(dmax);
            if dmax > tolerance {
                mismatches += 1;
                // Magenta mismatch pixel.
                diff[i] = 255;
                diff[i + 1] = 0;
                diff[i + 2] = 255;
                diff[i + 3] = 255;
            } else {
                // Dim actual for context.
                diff[i] = a[i] / 2;
                diff[i + 1] = a[i + 1] / 2;
                diff[i + 2] = a[i + 2] / 2;
                diff[i + 3] = 255;
            }
        }
    }
    if mismatches > 0 {
        if let Some(path) = diff_out {
            write_rgba_png(path, aw as u32, ah as u32, &diff)?;
        }
        anyhow::bail!(
            "pixel mismatch: {mismatches} px over tolerance {tolerance} (max delta {max_delta})"
        );
    }
    Ok(())
}

fn decode_png(bytes: &[u8]) -> Result<(u32, u32, Vec<u8>)> {
    // Minimal PNG decode via takumi: write temp is heavy; use png crate if available.
    // Fallback: re-render path uses file. For tests we use a tiny decoder.
    use std::io::Cursor;
    // takumi doesn't expose decode; use image-less manual via `png` if linked,
    // otherwise fail with clear message — we depend on `png` in Cargo.toml.
    let decoder = png::Decoder::new(Cursor::new(bytes));
    let mut reader = decoder.read_info().context("png read_info")?;
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).context("png frame")?;
    let w = info.width;
    let h = info.height;
    // Expand to RGBA8 if needed.
    let rgba = match info.color_type {
        png::ColorType::Rgba => buf[..info.buffer_size()].to_vec(),
        png::ColorType::Rgb => {
            let mut out = Vec::with_capacity((info.buffer_size() / 3) * 4);
            for chunk in buf[..info.buffer_size()].chunks_exact(3) {
                out.extend_from_slice(chunk);
                out.push(255);
            }
            out
        }
        other => anyhow::bail!("unsupported golden color type: {other:?}"),
    };
    Ok((w, h, rgba))
}

fn write_rgba_png(path: &Path, w: u32, h: u32, rgba: &[u8]) -> Result<()> {
    let file =
        std::fs::File::create(path).with_context(|| format!("create diff {}", path.display()))?;
    let mut encoder = png::Encoder::new(file, w, h);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().context("png header")?;
    writer.write_image_data(rgba).context("png data")?;
    Ok(())
}

fn default_fonts_dir() -> Option<PathBuf> {
    let workspace_relative = PathBuf::from("packages/sail/tokens/src/fonts");
    if workspace_relative.is_dir() {
        return Some(workspace_relative);
    }
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
    // Crate-relative walk.
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for _ in 0..8 {
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
