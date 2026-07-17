//! `sail-layout` — compile-time box-tree diagnostics for a `.sui` template.
//!
//! SPIKE. Usage:
//!
//!     sail-layout <FILE> [--viewport WxH]
//!
//! Example:
//!
//!     sail-layout apps/sui-site/routes/tutorial/\[slug\]/page.sui \
//!         --viewport 1440x900
//!
//! Emits each diagnostic on its own line in the rough shape the compiler
//! error formatter produces, so a future LSP integration doesn't need to
//! invent a new text protocol.

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use sail_layout as layout;

#[derive(Parser, Debug)]
#[command(about = "Compile-time box-tree diagnostics for .sui templates (spike)")]
struct Args {
    /// Path to a `.sui` file. Pass `-` to read from stdin.
    input: PathBuf,

    /// Viewport in CSS pixels (W x H). Defaults to 1440 x 900 — the canonical
    /// "desktop" used by the docs site.
    #[arg(long, value_parser = parse_viewport, default_value = "1440x900")]
    viewport: (f32, f32),

    /// Don't emit diagnostics — only a JSON dump of the computed tree.
    #[arg(long)]
    json: bool,
}

fn parse_viewport(s: &str) -> Result<(f32, f32), String> {
    let (w, h) = s
        .split_once('x')
        .ok_or_else(|| "viewport must be `WxH`".to_string())?;
    let w: f32 = w.parse().map_err(|_| format!("bad width {w:?}"))?;
    let h: f32 = h.parse().map_err(|_| format!("bad height {h:?}"))?;
    Ok((w, h))
}

fn main() -> Result<()> {
    let args = Args::parse();
    let source = read_source(&args.input)
        .with_context(|| format!("reading {}", args.input.display()))?;

    // Allow `<column viewport="1440x900">` in source to override.
    let vp = layout::html::source_viewport(&source).unwrap_or(args.viewport);

    let result = layout::check(&source, vp)
        .with_context(|| "compiling layout")?;

    if args.json {
        let json = serde_json::to_string_pretty(&result.diagnostics)
            .context("serializing diagnostics to JSON")?;
        println!("{json}");
        return Ok(());
    }

    if result.diagnostics.is_empty() {
        eprintln!("ok — no layout diagnostics at viewport {}x{}", vp.0 as u32, vp.1 as u32);
        return Ok(());
    }

    for d in &result.diagnostics {
        let sev = match d.severity {
            layout::Severity::Error => "error",
            layout::Severity::Warning => "warning",
        };
        eprintln!(
            "{sev}: [{kind}] at byte {}..{} \
            (tag={}, class=\"{cls}\") — {msg}",
            d.span.0,
            d.span.1,
            d.element_tag,
            cls = d.element_class,
            kind = d.kind,
            msg = d.message,
        );
    }

    eprintln!(
        "\n{}/{} layout diagnostic(s) at viewport {}x{}",
        result.diagnostics.len(),
        result.diagnostics.len(),
        vp.0 as u32,
        vp.1 as u32
    );
    Ok(())
}

fn read_source(path: &PathBuf) -> Result<String> {
    if path == &PathBuf::from("-") {
        let mut s = String::new();
        std::io::stdin().read_line(&mut s)?;
        Ok(s)
    } else {
        std::fs::read_to_string(path).map_err(Into::into)
    }
}
