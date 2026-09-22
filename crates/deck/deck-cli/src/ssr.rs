//! First-paint SSR engine for `sail dev`.
//!
//! Default `sail` does not link `deck-ssr` / ship-js / V8. An app that sets
//! `deck.render: "ssr"` is refused until this crate is built with `--features ssr`.

use std::sync::Arc;

use anyhow::{Result, bail};
use deck_config::RenderMode;
use deck_core::synthetic_ssr::SyntheticSsr;

/// True when this `sail` was compiled with the `ssr` Cargo feature (V8 linked).
pub const LINKED: bool = cfg!(feature = "ssr");

/// Mount-scoped renderer, or `None` for CSR.
///
/// CSR is always available. SSR without the `ssr` feature is an error — we
/// do not silently degrade. SSR with the feature still fails if the engine
/// cannot start.
pub fn engine_for(mode: RenderMode) -> Result<Option<Arc<dyn SyntheticSsr>>> {
    match mode {
        RenderMode::Csr => Ok(None),
        RenderMode::Ssr => start_engine(),
    }
}

fn start_engine() -> Result<Option<Arc<dyn SyntheticSsr>>> {
    #[cfg(feature = "ssr")]
    {
        Ok(Some(deck_ssr::serve::engine(None)?))
    }
    #[cfg(not(feature = "ssr"))]
    {
        bail!("SSR is not in this `sail` (no V8). Set `deck.render` to `\"csr\"` (the default).")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csr_does_not_start_an_engine() {
        assert!(engine_for(RenderMode::Csr).unwrap().is_none());
    }

    #[cfg(not(feature = "ssr"))]
    #[test]
    fn ssr_is_refused_when_v8_is_unlinked() {
        let err = match engine_for(RenderMode::Ssr) {
            Ok(_) => panic!("ssr must be refused when V8 is unlinked"),
            Err(e) => e.to_string(),
        };
        assert!(
            err.contains("no V8") && err.contains("csr"),
            "expected refuse message, got: {err}"
        );
        assert!(!LINKED);
    }
}
