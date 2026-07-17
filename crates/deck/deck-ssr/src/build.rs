//! Synthesize + bundle: turn a `.sui` + app manifest into ONE self-contained
//! ES module ready to run in the isolate.
//!
//! `synthesize_and_bundle` calls the synthetic emitter as a pure function (no
//! CLI shell-out) and drives Rolldown — the project's own bundler — so the
//! module matches the real pipeline. `@ship/sui-runtime` and `@ship/deck/router`
//! alias to their SERVER entries (no browser side-effects, host-backed route),
//! app `<client>` modules alias to their server entries, and `ship` stays
//! external (the host-dispatch wire).

use std::path::{Component, Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use rolldown::BundlerBuilder;
use rolldown_common::{BundlerOptions, InputItem, Output, OutputFormat, ResolveOptions};
use sail_compiler::emit::synthetic::{emit_synthetic, AppManifest};
use sail_compiler::parse::parse;
use sail_compiler::parse::preprocess::extract_controller_block;
use sail_compiler::CompileMode;

/// Walk up from this crate to the repo root (the dir holding `package.json`
/// and `packages/`). Anchors the view / manifest / server-entry paths.
pub fn repo_root() -> Result<PathBuf> {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    loop {
        if d.join("package.json").exists() && d.join("packages").is_dir() {
            return Ok(d);
        }
        if !d.pop() {
            return Err(anyhow!("repo root (package.json + packages/) not found"));
        }
    }
}

/// One alias from an app-manifest client module specifier to its server
/// entry. The browser build resolves the same specifier to the client
/// module; the SSR build resolves it here, to a host-dispatch-backed entry.
pub struct ClientAlias {
    /// Manifest `from` specifier, e.g. `@app/clients/demo`.
    pub specifier: String,
    /// Absolute path to the server entry that backs it.
    pub server_entry: PathBuf,
}

/// Synthesize `view_rel` with `manifest_rel`, lay the artifacts + the SSR
/// entry out in `build_dir`, then Rolldown-bundle into one ES module string.
pub async fn synthesize_and_bundle(
    repo: &Path,
    view_rel: &str,
    manifest_rel: &str,
    mode: CompileMode,
    client_aliases: &[ClientAlias],
    build_dir: &Path,
) -> Result<String> {
    synthesize_and_bundle_with(repo, view_rel, manifest_rel, mode, None, client_aliases, build_dir)
        .await
}

/// As [`synthesize_and_bundle`], but composing an authored controller. The
/// `authored` path is the on-disk `controller.ts`; the synthetic controller
/// imports it by an absolute specifier Rolldown resolves directly.
#[allow(clippy::too_many_arguments)]
pub async fn synthesize_and_bundle_with(
    repo: &Path,
    view_rel: &str,
    manifest_rel: &str,
    mode: CompileMode,
    authored: Option<&Path>,
    client_aliases: &[ClientAlias],
    build_dir: &Path,
) -> Result<String> {
    use sail_compiler::emit::synthetic::AuthoredController;

    let source = std::fs::read_to_string(repo.join(view_rel))
        .with_context(|| format!("read view {view_rel}"))?;
    let manifest_src = std::fs::read_to_string(repo.join(manifest_rel))
        .with_context(|| format!("read manifest {manifest_rel}"))?;
    // SUI sources may declare a `<controller>…</controller>` block alongside
    // the template — the synthetic emit reads it as a paired controller.
    // The template-only parser expects the block to be stripped first; the
    // CLI does this in `parse::preprocess`, the SSR bundler must too.
    let (stripped, inline) = extract_controller_block(&source);
    let program = parse(&stripped).map_err(|e| anyhow!("parse view: {}", e.render(&stripped)))?;
    let mut manifest = AppManifest::parse(&manifest_src)
        .map_err(|e| anyhow!("parse manifest: {}", e.render(&manifest_src)))?;
    // Deck declares its host context (route/navigate -> @ship/deck/router) into
    // the manifest seam; the app's own entries win. SUI names no host.
    for (name, client) in deck_core::host_context::host_context_clients() {
        manifest.clients.entry(name).or_insert(client);
    }
    std::fs::create_dir_all(build_dir)
        .with_context(|| format!("mkdir {}", build_dir.display()))?;
    // The controller IS the .sui's own `<controller>` block; an explicit
    // `authored` path (test-only) overrides it. For an inline block, write it to a
    // module beside the synthetic controller and import it by the same absolute
    // specifier an authored sibling would use.
    let authored = if let Some(p) = authored {
        Some(AuthoredController { specifier: path_str(p) })
    } else if let Some(inline_src) = inline.as_deref() {
        let inline_path = build_dir.join("page.inline.controller.ts");
        std::fs::write(&inline_path, inline_src)
            .with_context(|| format!("write inline controller {}", inline_path.display()))?;
        Some(AuthoredController { specifier: path_str(&inline_path) })
    } else {
        None
    };
    let out =
        emit_synthetic(&program, &manifest, authored.as_ref(), "./page.synthetic.controller.ts", mode, None)
            .map_err(|e| anyhow!("synthesize: {}", e.render(&source)))?;
    std::fs::write(build_dir.join("page.synthetic.controller.ts"), &out.controller_ts)?;
    std::fs::write(build_dir.join("page.render.ts"), &out.render_ts)?;
    // The SSR entry (server-DOM + host-dispatch + serialize), from the crate.
    let ssr_entry = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ssr/ssr-entry.ts");
    std::fs::copy(&ssr_entry, build_dir.join("ssr-entry.ts"))
        .with_context(|| format!("copy {}", ssr_entry.display()))?;

    bundle(repo, build_dir, client_aliases).await
}

async fn bundle(repo: &Path, build_dir: &Path, client_aliases: &[ClientAlias]) -> Result<String> {
    let runtime_server = repo.join("packages/sui/runtime/src/server.ts");
    let runtime_internal = repo.join("packages/sui/runtime/src/internal.ts");
    let route_server = repo.join("packages/deck/router/src/server.ts");
    let mut alias = vec![
        ("@ship/sui-runtime".to_string(), vec![Some(path_str(&runtime_server))]),
        ("@ship/sui-runtime/internal".to_string(), vec![Some(path_str(&runtime_internal))]),
        ("@ship/deck/router".to_string(), vec![Some(path_str(&route_server))]),
    ];
    for c in client_aliases {
        alias.push((c.specifier.clone(), vec![Some(path_str(&c.server_entry))]));
    }

    let entry = build_dir.join("ssr-entry.ts");
    let options = BundlerOptions {
        input: Some(vec![InputItem {
            name: Some("ssr".to_string()),
            import: path_str(&entry),
        }]),
        cwd: Some(repo.to_path_buf()),
        format: Some(OutputFormat::Esm),
        external: Some(vec!["ship".to_string()].into()),
        resolve: Some(ResolveOptions {
            alias: Some(alias),
            extensions: Some(vec![".ts".to_string(), ".js".to_string(), ".mjs".to_string()]),
            ..Default::default()
        }),
        ..Default::default()
    };
    let mut bundler = BundlerBuilder::default()
        .with_options(options)
        .build()
        .map_err(|e| anyhow!("rolldown build: {e:?}"))?;
    let generated = bundler.generate().await.map_err(|e| anyhow!("rolldown generate: {e:?}"))?;
    for asset in generated.assets {
        if let Output::Chunk(c) = asset
            && c.is_entry
        {
            return Ok(c.code.clone());
        }
    }
    Err(anyhow!("rolldown produced no entry chunk"))
}


fn path_str(p: &Path) -> String {
     normalize(p).to_string_lossy().into_owned()
 }

/// Collapse `.`/`..` components without touching the filesystem (the paths
/// are built from a canonical repo root + known-good relatives).
fn normalize(p: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in p.components() {
        match c {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}
