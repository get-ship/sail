use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use sail_render::{RenderOptions, render_file, write_png};

#[derive(Parser, Debug)]
#[command(about = "Render a .sail page to a PNG screenshot (no browser)")]
struct Args {
    /// Path to the .sail source file.
    #[arg(long)]
    source: PathBuf,

    /// Viewport in CSS pixels (WxH). Defaults to 1440x900.
    #[arg(long, value_parser = parse_viewport, default_value = "1440x900")]
    viewport: (u32, u32),

    /// Output PNG path.
    #[arg(long)]
    out: PathBuf,

    /// Optional directory containing token woff2 fonts.
    #[arg(long)]
    fonts_dir: Option<PathBuf>,
}

fn parse_viewport(s: &str) -> Result<(u32, u32), String> {
    let (w, h) = s.split_once('x').ok_or_else(|| "viewport must be `WxH`".to_string())?;
    let w: u32 = w.parse().map_err(|_| format!("bad width {w:?}"))?;
    let h: u32 = h.parse().map_err(|_| format!("bad height {h:?}"))?;
    Ok((w, h))
}

fn main() -> Result<()> {
    let args = Args::parse();

    let opts = RenderOptions {
        viewport: args.viewport,
        fonts_dir: args.fonts_dir,
    };

    let output = render_file(&args.source, opts)
        .with_context(|| format!("rendering {}", args.source.display()))?;

    write_png(&output.image, &args.out)
        .with_context(|| format!("writing {}", args.out.display()))?;

    eprintln!(
        "rendered {} → {} ({}x{})",
        args.source.display(),
        args.out.display(),
        output.viewport.0,
        output.viewport.1
    );

    Ok(())
}
