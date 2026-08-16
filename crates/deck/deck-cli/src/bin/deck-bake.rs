//! `deck-bake` — workspace bake CLI (not product `sail`).
//!
//! Owns `build-assets` and `emit`. A stranger never runs these; this
//! checkout bakes with this binary.

#[path = "../bake.rs"]
mod bake;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "deck-bake",
    about = "Workspace bake CLI (build-assets, emit). Not the product sail CLI.",
    version,
    propagate_version = true
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Emit already-baked target bundles into a directory.
    Emit {
        /// Target name(s). Repeatable.
        #[arg(long = "target", required = true)]
        targets: Vec<String>,
        /// Destination directory (created if absent).
        #[arg(long)]
        out: PathBuf,
    },

    /// Bake all framework + consumer bundles via the in-process Rolldown bundler.
    BuildAssets {
        /// Skip touching `crates/deck/deck-core/framework-stamp` (tests).
        #[arg(long = "no-stamp")]
        no_stamp: bool,
    },
}

fn main() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .without_time()
        .try_init();
    if let Err(e) = run() {
        eprintln!("deck-bake: {e:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    match Cli::parse().command {
        Command::Emit { targets, out } => bake::emit(targets, out),
        Command::BuildAssets { no_stamp } => bake::build_assets(no_stamp),
    }
}
