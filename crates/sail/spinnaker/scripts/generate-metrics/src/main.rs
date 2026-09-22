//! Regenerate `crates/sail/spinnaker/src/layout/metrics/metrics_generated.rs` from
//! the framework's self-hosted woff2 fonts in `packages/sail/tokens/src/fonts/`.
//!
//! Run from this directory:
//!     cargo run

use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Family {
    Body,
    Mono,
    Display,
    PressStart,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Size {
    Xs,
    Sm,
    Base,
    Md,
    Lg,
    Xl,
    Xxl,
    Xxxl,
    Xxxxl,
    Display,
}

fn main() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let font_dir = manifest_dir.join("../../../../../packages/sail/tokens/src/fonts");
    let out_path = manifest_dir.join("../../src/layout/metrics/metrics_generated.rs");

    let families = [
        (Family::Body, "dm-sans-latin-wght-normal.woff2"),
        (Family::Mono, "jetbrains-mono-latin-wght-normal.woff2"),
        (Family::Display, "syne-latin-wght-normal.woff2"),
        (Family::PressStart, "press-start-2p-latin-normal.woff2"),
    ];
    let sizes = [
        (Size::Xs, 11.0),
        (Size::Sm, 12.0),
        (Size::Base, 13.0),
        (Size::Md, 14.0),
        (Size::Lg, 16.0),
        (Size::Xl, 20.0),
        (Size::Xxl, 24.0),
        (Size::Xxxl, 28.0),
        (Size::Xxxxl, 36.0),
        (Size::Display, 129.6),
    ];

    let mut table: HashMap<(Family, Size), Vec<f32>> = HashMap::new();
    for (family, file) in &families {
        let woff2 = fs::read(font_dir.join(file)).unwrap();
        let ttf = woofwoof::decompress(&woff2)
            .unwrap_or_else(|| panic!("decompress failed for {file:?}"));
        let font = fontdue::Font::from_bytes(ttf, fontdue::FontSettings::default())
            .unwrap_or_else(|e| panic!("fontdue load failed for {file:?}: {e:?}"));
        for (size, px) in &sizes {
            let mut widths = Vec::with_capacity(128);
            for c in 0u8..=127 {
                let m = font.metrics(c as char, *px);
                widths.push(m.advance_width);
            }
            table.insert((*family, *size), widths);
        }
    }

    let mut out = fs::File::create(&out_path).unwrap();
    writeln!(
        out,
        "//! Generated font metrics from packages/sail/tokens/src/fonts/"
    )
    .unwrap();
    writeln!(
        out,
        "//! Do not edit by hand. Regenerate with scripts/generate-metrics/."
    )
    .unwrap();
    writeln!(out, "use std::collections::HashMap;").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]").unwrap();
    writeln!(out, "pub enum Family {{").unwrap();
    writeln!(out, "    Body,").unwrap();
    writeln!(out, "    Mono,").unwrap();
    writeln!(out, "    Display,").unwrap();
    writeln!(out, "    PressStart,").unwrap();
    writeln!(out, "}}").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]").unwrap();
    writeln!(out, "pub enum Size {{").unwrap();
    writeln!(
        out,
        "    Xs, Sm, Base, Md, Lg, Xl, Xxl, Xxxl, Xxxxl, Display"
    )
    .unwrap();
    writeln!(out, "}}").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "impl Size {{").unwrap();
    writeln!(out, "    pub fn px(self) -> f32 {{").unwrap();
    writeln!(out, "        match self {{").unwrap();
    writeln!(out, "            Size::Xs => 11.0,").unwrap();
    writeln!(out, "            Size::Sm => 12.0,").unwrap();
    writeln!(out, "            Size::Base => 13.0,").unwrap();
    writeln!(out, "            Size::Md => 14.0,").unwrap();
    writeln!(out, "            Size::Lg => 16.0,").unwrap();
    writeln!(out, "            Size::Xl => 20.0,").unwrap();
    writeln!(out, "            Size::Xxl => 24.0,").unwrap();
    writeln!(out, "            Size::Xxxl => 28.0,").unwrap();
    writeln!(out, "            Size::Xxxxl => 36.0,").unwrap();
    writeln!(out, "            Size::Display => 129.6,").unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(out, "}}").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "#[derive(Debug, Clone, Copy, PartialEq)]").unwrap();
    writeln!(out, "pub struct Metrics {{").unwrap();
    writeln!(out, "    pub font_size_px: f32,").unwrap();
    writeln!(out, "    pub line_height_px: f32,").unwrap();
    writeln!(
        out,
        "    /// Widths for ASCII 32..=126 (95 slots), index = (c as u8) - 32."
    )
    .unwrap();
    writeln!(out, "    pub ascii_widths: [f32; 95],").unwrap();
    writeln!(out, "    pub fallback_width_px: f32,").unwrap();
    writeln!(out, "}}").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "pub fn table() -> HashMap<(Family, Size), Metrics> {{").unwrap();
    writeln!(out, "    let mut t = HashMap::new();").unwrap();
    for ((family, size), widths) in &table {
        let family_name = format!("{:?}", family);
        let size_name = format!("{:?}", size);
        let px = sizes.iter().find(|(s, _)| s == size).unwrap().1;
        let line_height = px * 1.5;
        let ascii_widths: Vec<String> = widths[32..=126]
            .iter()
            .map(|w| format!("{:.3}", w))
            .collect();
        // Average ASCII width as fallback for non-ASCII characters.
        let fallback = widths[32..=126].iter().sum::<f32>() / 95.0;
        writeln!(
            out,
            "    t.insert((Family::{}, Size::{}), Metrics {{",
            family_name, size_name
        )
        .unwrap();
        writeln!(out, "        font_size_px: {:.1},", px).unwrap();
        writeln!(out, "        line_height_px: {:.1},", line_height).unwrap();
        writeln!(out, "        ascii_widths: [").unwrap();
        writeln!(out, "            {}", ascii_widths.join(", ")).unwrap();
        writeln!(out, "        ],").unwrap();
        writeln!(out, "        fallback_width_px: {:.3},", fallback).unwrap();
        writeln!(out, "    }});").unwrap();
    }
    writeln!(out, "    t").unwrap();
    writeln!(out, "}}").unwrap();
}
