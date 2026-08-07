//! Scratch driver: render a .sail file to PNG.
//! Usage: cargo run -p sail-render --example render -- <file.sail> <out.png> [WxH] [theme] [mode]

use sail_render::{RenderOptions, render_file, write_png};

fn workspace_root() -> Option<std::path::PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    for _ in 0..10 {
        if dir.join("Cargo.toml").is_file() && dir.join("packages/sail/tokens").is_dir() {
            return Some(dir);
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let file = args.get(1).expect("usage: render <file.sail> <out.png> [WxH] [theme] [mode]");
    let out = args.get(2).expect("out.png required");
    let viewport = args.get(3).map(|s| {
        let (w, h) = s.split_once('x').expect("WxH");
        (w.parse().unwrap(), h.parse().unwrap())
    });
    let opts = RenderOptions {
        viewport: viewport.unwrap_or((1440, 900)),
        theme: args.get(4).cloned().filter(|s| !s.is_empty()),
        mode: args.get(5).cloned(),
        workspace: workspace_root(),
        ..RenderOptions::default()
    };
    let result = render_file(std::path::Path::new(file), opts).expect("render failed");
    write_png(&result.image, std::path::Path::new(out)).expect("png write failed");
    if std::env::var("RENDER_DUMP_PRE").is_ok() {
        let src = std::fs::read_to_string(file).unwrap();
        let pre = sail_compiler::parse::preprocess::preprocess(&src).expect("preprocess");
        eprintln!("--- PREPROCESSED ---\n{}", pre);
        let (html_raw, _css) = sail_render::pipeline::split_html_css(&pre);
        eprintln!("--- HTML_RAW (pre-strip) ---\n{}", html_raw);
    }
    if std::env::var("RENDER_DUMP_HTML").is_ok() {
        eprintln!("--- HTML ---\n{}", result.html);
    }
    eprintln!(
        "rendered {} ({}x{}) in {}ms; regions={} dynamics={} projected={:?}",
        out,
        result.viewport.0,
        result.viewport.1,
        result.ms,
        result.regions.len(),
        result.dynamics.len(),
        result.projected
    );
}
