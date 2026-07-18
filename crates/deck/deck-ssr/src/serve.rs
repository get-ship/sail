//! The serve-path SSR renderer — the [`deck_core::synthetic_ssr::SyntheticSsr`]
//! implementation deck-ssr registers into deck-core's gated seam.
//!
//! deck-core declares the result ("first-paint HTML for this view + request");
//! this owns the procedure: locate the app manifest + sibling controller
//! relative to the source `.sail`, synthesize + Rolldown-bundle (mtime-cached on
//! the source), then run the bundle in a POOLED isolate ([`ScriptPool`], not
//! single-shot [`crate::run`]) with route + data resolved over the
//! host-dispatch wire.
//!
//! `first_paint` is called from deck-core's synchronous serve path (itself
//! invoked on an axum worker thread). The async work — Rolldown and the pool —
//! runs on a dedicated worker thread with its own runtime, so it never nests
//! inside the caller's runtime.

use std::collections::HashMap;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use anyhow::{anyhow, Result};
use deck_core::synthetic_ssr::{SsrRequest, SyntheticSsr};
use deck_core::DataPort;
use ship_js::Dispatch;
use sail_compiler::emit::synthetic::AppManifest;
use sail_compiler::CompileMode;

use crate::build::{repo_root, synthesize_and_bundle_with, ClientAlias};
use crate::host::SsrDispatch;
use crate::pool::ScriptPool;
use crate::SHIP_SDK;

/// Construct the ship-js SSR renderer as a MOUNT-SCOPED handle — an
/// `Arc<dyn SyntheticSsr>` the host layers as an axum `Extension` and the serve
/// path threads per-request. NOT a process-global `register` install (that
/// global was removed for `inv-deck-mount-scoped-resolution`), so one process
/// can carry different (or no) engines per mount. `data` is the host's live
/// data surface — the `<data>`/`<client>` op resolver; pass `None` for a
/// route-only host.
pub fn engine(data: Option<Arc<dyn DataPort>>) -> Result<Arc<dyn SyntheticSsr>> {
    Ok(Arc::new(ShipJsSsr::new(data)?))
}

/// One cached bundle, keyed by the source `.sail`'s mtime.
struct BundleEntry {
    mtime: SystemTime,
    bundle: Arc<String>,
}

/// The registered renderer: a pool of isolates + a per-source bundle cache.
pub struct ShipJsSsr {
    pool: Arc<ScriptPool>,
    data: Option<Arc<dyn DataPort>>,
    cache: Mutex<HashMap<PathBuf, BundleEntry>>,
    repo: PathBuf,
}

impl ShipJsSsr {
    /// Build the renderer. Spawns the isolate pool and anchors the repo root
    /// (for the `@ship/*` server-entry aliases).
    pub fn new(data: Option<Arc<dyn DataPort>>) -> Result<Self> {
        Ok(Self {
            pool: Arc::new(ScriptPool::for_host()),
            data,
            cache: Mutex::new(HashMap::new()),
            repo: repo_root()?,
        })
    }

    /// The bundled ES module for `source`, mtime-cached. A source whose mtime
    /// advanced is a MISS and is re-synthesized, so an edited view is never
    /// stale-pinned (mirrors deck-core's SPLIT_CACHE/SSR_CACHE gate).
    fn bundle_for(&self, source: &Path) -> Result<Arc<String>> {
        let mtime = std::fs::metadata(source).and_then(|m| m.modified()).ok();
        if let Some(mt) = mtime {
            let cache = self.cache.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(e) = cache.get(source)
                && e.mtime == mt
            {
                return Ok(e.bundle.clone());
            }
        }
        let bundle = Arc::new(self.synth_bundle(source)?);
        if let Some(mt) = mtime {
            let mut cache = self.cache.lock().unwrap_or_else(|e| e.into_inner());
            cache.insert(source.to_path_buf(), BundleEntry { mtime: mt, bundle: bundle.clone() });
        }
        Ok(bundle)
    }

    /// Synthesize + Rolldown-bundle one view. Discovers the app manifest +
    /// sibling controller relative to `source`; aliases the manifest's
    /// `@app/*` clients to their `.server.ts` entries.
    fn synth_bundle(&self, source: &Path) -> Result<String> {
        let manifest_path = find_manifest(source)
            .ok_or_else(|| anyhow!("no app.manifest.jsonc above {}", source.display()))?;
        let app_dir = manifest_path
            .parent()
            .ok_or_else(|| anyhow!("manifest has no parent dir"))?
            .to_path_buf();
        let route_rel = rel(&self.repo, source)?;
        let manifest_rel = rel(&self.repo, &manifest_path)?;
        let authored = sibling_controller(source);
        let aliases = client_aliases(&app_dir)?;
        let build_dir = self.repo.join("target/deck-ssr-serve").join(sanitize(&route_rel));
        let repo = self.repo.clone();
        on_worker(true, move || async move {
            synthesize_and_bundle_with(
                &repo,
                &route_rel,
                &manifest_rel,
                CompileMode::Component,
                authored.as_deref(),
                &aliases,
                &build_dir,
            )
            .await
        })
    }
}

impl SyntheticSsr for ShipJsSsr {
    fn first_paint(&self, req: &SsrRequest) -> Result<String> {
        let bundle = self.bundle_for(req.source_path)?;
        let mut dispatch = SsrDispatch::for_request(req.pathname, req.params.clone(), req.query.clone());
        if let Some(d) = &self.data {
            dispatch = dispatch.with_data(d.clone());
        }
        let dispatch: Arc<dyn Dispatch> = Arc::new(dispatch);
        let pool = self.pool.clone();
        let main = (*bundle).clone();
        let emits = on_worker(false, move || async move {
            pool.run(dispatch, SHIP_SDK.to_string(), main).await
        })?;
        let first = emits.first().ok_or_else(|| anyhow!("ssr entry emitted nothing"))?;
        Ok(first["html"].as_str().unwrap_or_default().to_string())
    }
}

/// Run an async job to completion on a dedicated worker thread with its own
/// tokio runtime, returning the result. A fresh thread (no ambient runtime)
/// lets `block_on` run even when the caller is inside the axum runtime; a
/// panicking job becomes an error rather than aborting the daemon.
fn on_worker<Fut, T>(multi: bool, fut_fn: impl FnOnce() -> Fut + Send + 'static) -> Result<T>
where
    Fut: Future<Output = Result<T>>,
    T: Send + 'static,
{
    std::thread::Builder::new()
        .name("deck-ssr-serve".into())
        .spawn(move || {
            let rt = if multi {
                tokio::runtime::Builder::new_multi_thread().enable_all().build()
            } else {
                tokio::runtime::Builder::new_current_thread().enable_all().build()
            }
            .expect("deck-ssr worker runtime");
            rt.block_on(fut_fn())
        })
        .map_err(|e| anyhow!("spawn deck-ssr worker: {e}"))?
        .join()
        .map_err(|_| anyhow!("deck-ssr worker thread panicked"))?
}

/// Walk up from `source`'s directory to the nearest `app.manifest.jsonc`.
fn find_manifest(source: &Path) -> Option<PathBuf> {
    let mut dir = source.parent();
    while let Some(d) = dir {
        let candidate = d.join("app.manifest.jsonc");
        if candidate.is_file() {
            return Some(candidate);
        }
        dir = d.parent();
    }
    None
}

/// The sibling authored controller (`controller.ts`) next to a `page.sail`, if
/// present. The synthetic controller composes it via `setup(ctx)`.
fn sibling_controller(source: &Path) -> Option<PathBuf> {
    let c = source.parent()?.join("controller.ts");
    c.is_file().then_some(c)
}

/// Aliases mapping the manifest's `@app/*` client specifiers to their
/// `.server.ts` host-dispatch entries. `@ship/*` specifiers are aliased by
/// `build::bundle` itself; only the app's own clients are resolved here.
fn client_aliases(app_dir: &Path) -> Result<Vec<ClientAlias>> {
    let manifest_src = std::fs::read_to_string(app_dir.join("app.manifest.jsonc"))?;
    let manifest = AppManifest::parse(&manifest_src).map_err(|e| anyhow!("parse manifest: {e:?}"))?;
    let mut seen: HashMap<String, ClientAlias> = HashMap::new();
    for client in manifest.clients.values() {
        let Some(rest) = client.from.strip_prefix("@app/") else { continue };
        if seen.contains_key(&client.from) {
            continue;
        }
        let server_entry = app_dir.join(format!("{rest}.server.ts"));
        seen.insert(
            client.from.clone(),
            ClientAlias { specifier: client.from.clone(), server_entry },
        );
    }
    Ok(seen.into_values().collect())
}

/// Path relative to the repo root, as a forward-slash string.
fn rel(repo: &Path, p: &Path) -> Result<String> {
    p.strip_prefix(repo)
        .map(|r| r.to_string_lossy().replace('\\', "/"))
        .map_err(|_| anyhow!("{} is not under repo root {}", p.display(), repo.display()))
}

/// A filesystem-safe build-dir name from a relative view path.
fn sanitize(rel: &str) -> String {
    rel.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}
