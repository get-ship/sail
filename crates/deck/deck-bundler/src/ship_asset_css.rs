//! `ShipAssetCssPlugin` — rewrites `url(...)` references inside framework
//! CSS assets to point at bundled, hashed assets (woff2 today; png/svg
//! tomorrow — the rewrite is general, NOT a font-only special case).
//! In production builds it also minifies CSS asset bodies via
//! [`deck_core::minify_css`] (after URL rewrite so placeholders resolve first).
//!
//! # Why
//!
//! The framework emits **sail-base** (`sail-base.css`; historical name
//! `contracts.css`) with relative `url('./fonts/X.woff2')` references.
//! After Rolldown's `AssetModulePlugin` runs, the CSS lives at
//! `assets/sail-base-<hash>.css` and the woff2 should ship as siblings in
//! `assets/<name>-<hash>.woff2` — but Rolldown's `AssetModulePlugin` does
//! NOT touch the CSS BODY: it only rewrites the JS chunk's
//! `__ROLLDOWN_ASSET__#<ref>` placeholders. The browser fetches the CSS
//! at `assets/sail-base-<hash>.css` and resolves `url('./fonts/X.woff2')`
//! to `/sail-site/fonts/X.woff2` — 404, because the woff2 live under
//! `/sail-site/assets/`, not `/sail-site/fonts/`.
//!
//! # The fix (chosen path: mutable `generate_bundle`)
//!
//! Two hooks cooperate:
//!
//! - `load` (runs at default order, BEFORE the `AssetModulePlugin` which
//!   is `Post`): when a `.css` is loaded, scan the body for `url(...)`
//!   references. For each unique local file, read its bytes, register it
//!   as a hashed asset via `ctx.emit_file_async`, and rewrite the CSS body
//!   so `url('./fonts/X.woff2')` becomes
//!   `url("__SHIP_CSS_ASSET__#<woff2_ref_id>")`. The CSS file itself is
//!   then emitted as an asset too, with the rewritten body — and the
//!   `load` returns JS that exports `__ROLLDOWN_ASSET__#<css_ref_id>`,
//!   which the existing `AssetModulePlugin::render_chunk` hook rewrites
//!   in the JS chunk to the CSS's hashed URL.
//!
//! - `generate_bundle` (runs at `Post` order, AFTER everything else
//!   that mutates the asset list): walk every `Output::Asset` whose body
//!   is CSS, find each `__SHIP_CSS_ASSET__#<ref>` placeholder, resolve
//!   it to the woff2's final hashed filename via `ctx.get_file_name(ref)`,
//!   compute the relative path from the CSS asset's filename to the
//!   woff2 asset's filename (both live under `assets/`), and rewrite
//!   the body in place. The result: the CSS file on disk reads
//!   `url("./dm-sans-<hash>.woff2")` against its own `assets/` directory,
//!   and the browser fetches the hashed woff2 — real, cacheable, no
//!   base64, no CDN.
//!
//! # The mutable-hook path (vs. a re-read/re-write pass)
//!
//! We picked `generate_bundle` because the asset list is `&mut Vec<Output>`
//! and the body bytes are `&mut` via the cloned `Arc<OutputAsset>`. The
//! `WriteBundle` hook runs AFTER files have been written to disk — at that
//! point only metadata is mutable, not body bytes, and we'd have to
//! re-read + re-write the file ourselves. `generate_bundle` is the latest
//! stage where the body is still mutable AND the asset hashes are known
//! (via `ctx.get_file_name(ref_id)`).
//!
//! # What the plugin does NOT do
//!
//! - It does not touch `Output::Chunk` (JS) bodies. The
//!   `AssetModulePlugin` owns that rewrite; the new placeholder prefix is
//!   distinct (`__SHIP_CSS_ASSET__#` vs `__ROLLDOWN_ASSET__#`) so the
//!   two never collide.
//! - It does not touch non-CSS asset bodies (woff2, png, …). The woff2
//!   themselves are emitted as bytes and written as-is; only the CSS
//!   that references them is rewritten.
//! - It does not resolve `data:` URLs. Those are inlined content; no
//!   rewrite needed.

use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, anyhow};

use crate::fonts;
use rolldown::plugin::{
    HookGenerateBundleArgs, HookLoadArgs, HookLoadOutput, HookLoadReturn, HookNoopReturn,
    HookUsage, Plugin, PluginContext, PluginHookMeta, PluginOrder, SharedLoadPluginContext,
};
use rolldown_common::{EmittedAsset, ModuleType, Output, OutputAsset, StrOrBytes};
use rustc_hash::FxHashMap;

/// Marker prefix for in-CSS-body asset placeholders. The
/// `AssetModulePlugin`'s `render_chunk` hook scans the JS chunk code for
/// `__ROLLDOWN_ASSET__#<ref>` and rewrites them. THIS plugin scans CSS
/// bodies for `__SHIP_CSS_ASSET__#<ref>` in `generate_bundle` and
/// rewrites them. Distinct prefixes so the two pipelines never trip on
/// each other.
const CSS_ASSET_PREFIX: &str = "__SHIP_CSS_ASSET__#";

/// Subset of file extensions the plugin will rewrite. Conservative on
/// purpose: only paths the plugin can resolve to a real file on disk
/// with a known asset extension (woff2 today; png/jpg/svg for future
/// generalisation). A `url(./other.css)` reference would be a different
/// concern (a CSS importing a CSS) and is out of scope.
const REWRITE_EXTENSIONS: &[&str] = &[
    ".woff2", ".woff", ".png", ".jpg", ".jpeg", ".svg", ".gif", ".webp",
];

/// Rolldown plugin: rewrite local `url(...)` in CSS assets to hashed paths,
/// and optionally minify the CSS body (production).
#[derive(Debug, Clone)]
pub struct ShipAssetCssPlugin {
    /// When true, minify every emitted CSS asset after URL rewrites resolve.
    /// Dev builds leave CSS readable.
    pub minify: bool,
    /// Framework woff2 basenames reachable from the token/style graph.
    /// `None` = do not filter (synthetic CSS tests). `Some` = strip unused
    /// `@font-face` for known FONTS files before emit.
    pub used_font_files: Option<Arc<HashSet<String>>>,
}

impl Default for ShipAssetCssPlugin {
    fn default() -> Self {
        Self {
            minify: false,
            used_font_files: None,
        }
    }
}

impl ShipAssetCssPlugin {
    /// Production helper — minify on. Used-set filled by [`Self::with_used_fonts`].
    pub fn production() -> Self {
        Self {
            minify: true,
            used_font_files: None,
        }
    }

    /// Development helper — readable CSS.
    pub fn development() -> Self {
        Self {
            minify: false,
            used_font_files: None,
        }
    }

    /// Restrict framework font emit to `files` (SPEC-FONTS-1 unused-family drop).
    pub fn with_used_fonts(mut self, files: HashSet<String>) -> Self {
        self.used_font_files = Some(Arc::new(files));
        self
    }

    /// Default used-set from the document-floor token graph (no catalog themes).
    pub fn with_default_used_fonts(self) -> Self {
        self.with_used_fonts(fonts::used_font_files_for_project(None))
    }
}

impl Plugin for ShipAssetCssPlugin {
    fn name(&self) -> std::borrow::Cow<'static, str> {
        "ship-asset-css".into()
    }

    fn register_hook_usage(&self) -> HookUsage {
        HookUsage::Load | HookUsage::GenerateBundle
    }

    /// `load` runs at the DEFAULT order — BEFORE the `AssetModulePlugin`,
    /// which is `Post`. The plugin intercepts a `.css` file, scans its
    /// body for `url(...)` references to local asset files, emits each
    /// referenced file as a hashed asset via `ctx.emit_file_async`, and
    /// rewrites the CSS body to use `__SHIP_CSS_ASSET__#<ref>`
    /// placeholders. Then it emits the rewritten CSS itself as an asset
    /// and returns a JS module exporting the standard
    /// `__ROLLDOWN_ASSET__#<css_ref>` placeholder so the existing
    /// `AssetModulePlugin::render_chunk` rewrites the JS chunk's link
    /// to the CSS's hashed URL.
    fn load_meta(&self) -> Option<PluginHookMeta> {
        // Default order — runs before the `AssetModulePlugin`'s `Post`
        // `load`, so we get first crack at the .css body. We handle the
        // asset emission ourselves and return `Some` from `load`,
        // short-circuiting the chain — the `AssetModulePlugin` never
        // runs on this module.
        None
    }

    fn generate_bundle_meta(&self) -> Option<PluginHookMeta> {
        // `Post` — runs after the `AssetModulePlugin`'s default-order
        // `generate_bundle` (which is a no-op for it; it doesn't
        // override the hook). The asset list is in its final shape:
        // chunk list + asset list, with the woff2 already in the file
        // emitter's map. We mutate the asset bytes in place to resolve
        // the placeholders.
        Some(PluginHookMeta {
            order: Some(PluginOrder::Post),
        })
    }

    async fn load(&self, ctx: SharedLoadPluginContext, args: &HookLoadArgs<'_>) -> HookLoadReturn {
        // Only act on `.css` files. Anything else: defer to the chain.
        if !args.id.ends_with(".css") {
            return Ok(None);
        }
        let css_path = Path::new(args.id);
        if !css_path.is_file() {
            return Ok(None);
        }

        // Read the CSS body. A read failure here is a real error — a
        // CSS file the bundler resolved but cannot read is a build
        // hazard (the consumer would silently get the asset-plugin's
        // default behaviour and any `url(...)` references would 404 at
        // runtime).
        let mut body = match std::fs::read_to_string(css_path) {
            Ok(b) => b,
            Err(e) => {
                return Err(anyhow!(
                    "ship-asset-css: failed to read CSS `{}`: {e}",
                    css_path.display()
                ));
            }
        };
        if let Some(used) = self.used_font_files.as_deref() {
            body = fonts::strip_unused_framework_faces(&body, used);
        }

        // Scan for `url(...)` references to local asset files. We only
        // rewrite references that (a) point at a path the bundler can
        // resolve to a real file on disk, AND (b) have an extension in
        // `REWRITE_EXTENSIONS`. `data:`, `http(s):`, and absolute
        // `file:` URLs are passed through verbatim — the browser
        // handles them and the bundler has nothing to emit.
        let css_dir = css_path.parent().unwrap_or_else(|| Path::new("."));
        let mut woff2_refs: FxHashMap<String, String> = FxHashMap::default();
        let mut rewritten = String::with_capacity(body.len());
        let mut last_end = 0usize;
        let mut did_rewrite = false;
        for (start, _quote, url_text) in url_references(&body) {
            if !is_local_path(url_text) {
                continue;
            }
            let resolved = resolve_relative(css_dir, url_text);
            if !has_rewrite_extension(&resolved) {
                continue;
            }
            // Read the referenced file. A missing file is a build
            // error — a CSS that references a woff2 the bundler
            // cannot find is a real issue, and silent 404s are exactly
            // what this mission fixes.
            let bytes = match std::fs::read(&resolved) {
                Ok(b) => b,
                Err(e) => {
                    return Err(anyhow!(
                        "ship-asset-css: CSS `{}` references `{}` (resolved to `{}`) \
                         but the file is unreadable: {e}",
                        css_path.display(),
                        url_text,
                        resolved.display(),
                    ));
                }
            };
            // Dedup by URL text — the same woff2 may be referenced
            // from multiple `@font-face` rules. Emit once, reuse the
            // ref id.
            let ref_id: String = if let Some(existing) = woff2_refs.get(url_text) {
                existing.clone()
            } else {
                let file_name = resolved
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("asset")
                    .to_string();
                let original_file_name = resolved
                    .strip_prefix(ctx.cwd())
                    .unwrap_or(&resolved)
                    .to_string_lossy()
                    .into_owned();
                let ref_id = ctx
                    .emit_file_async(EmittedAsset {
                        name: Some(file_name),
                        original_file_name: Some(original_file_name),
                        source: StrOrBytes::Bytes(bytes),
                        ..Default::default()
                    })
                    .await
                    .with_context(|| {
                        format!(
                            "ship-asset-css: emit_file_async for `{}`",
                            resolved.display()
                        )
                    })?;
                woff2_refs.insert(url_text.to_string(), ref_id.to_string());
                ref_id.to_string()
            };
            // The load hook standardizes the placeholder's surrounding
            // quotes to `"` (the `css_placeholder_occurrences` scanner
            // in `generate_bundle` only looks for `"`). The body
            // becomes `url("__SHIP_CSS_ASSET__#<ref>") …` regardless
            // of whether the source used `'` or `"`. The browser
            // treats `url("...")` and `url('...')` identically.
            //
            // `body[..start - 1]` is the body up to (but not including)
            // the source's opening quote; we then emit `"` to
            // standardize the opener. The body positions `start..`
            // are the inner text and the closing quote of the source;
            // the closing quote is consumed by the `+1` in `last_end`.
            if !did_rewrite {
                rewritten.push_str(&body[..start - 1]);
                rewritten.push('"');
                did_rewrite = true;
            } else {
                rewritten.push_str(&body[last_end..start - 1]);
                rewritten.push('"');
            }
            rewritten.push_str(&format!("{}{}\"", CSS_ASSET_PREFIX, ref_id.as_str(),));
            // Advance past the inner text AND the source's closing
            // quote — the tail flush re-emits the rest of the URL
            // group (the `)`, surrounding whitespace, etc.) verbatim.
            last_end = start + url_text.len() + 1;
        }
        if !did_rewrite {
            // No local-asset references — no rewrite needed. Emit the
            // CSS as-is (the asset plugin doesn't get a chance to run
            // because we short-circuited its `load`, but we still want
            // the CSS to ship as an asset).
            return emit_css_as_asset(ctx, css_path, body.into_bytes()).await;
        }
        rewritten.push_str(&body[last_end..]);
        // The body is UTF-8 throughout; the conversion to bytes is
        // infallible.
        let new_bytes = rewritten.into_bytes();
        emit_css_as_asset(ctx, css_path, new_bytes).await
    }

    async fn generate_bundle(
        &self,
        ctx: &PluginContext,
        args: &mut HookGenerateBundleArgs<'_>,
    ) -> HookNoopReturn {
        // Walk every emitted CSS asset. Resolve `__SHIP_CSS_ASSET__#`
        // placeholders (if any), then optionally minify for production.
        // Assets without placeholders are still minified when `self.minify`
        // is set — pure CSS imports (tokens, skins) need the same pass.
        for output in args.bundle.iter_mut() {
            let asset = match output {
                Output::Asset(a) => a.clone(),
                Output::Chunk(_) => continue,
            };
            if !asset.filename.ends_with(".css") {
                continue;
            }
            let src = match &asset.source {
                StrOrBytes::Bytes(b) => b.clone(),
                StrOrBytes::Str(s) => s.as_bytes().to_vec(),
            };
            let has_placeholder = src
                .windows(CSS_ASSET_PREFIX.len())
                .any(|w| w == CSS_ASSET_PREFIX.as_bytes());
            if !has_placeholder && !self.minify {
                continue;
            }
            // Convert to a lossy string for the rewrite (CSS is UTF-8
            // by convention; any invalid bytes pass through unchanged
            // — we only operate on placeholder matches).
            let body = String::from_utf8_lossy(&src).into_owned();
            let mut rewritten = String::with_capacity(body.len());
            let mut last_end = 0usize;
            let css_filename = asset.filename.as_str();
            for (start, ref_id) in css_placeholder_occurrences(&body) {
                // `start` is the position of the `<ref_id>` text
                // (right after `__SHIP_CSS_ASSET__#`). The body bytes
                // around the placeholder are
                // `…__SHIP_CSS_ASSET__#<ref_id>"<rest>`. We replace
                // just the prefix + ref id with the URL, keeping the
                // closing `"` and the `<rest>` bytes verbatim.
                let prefix_start = start - CSS_ASSET_PREFIX.len();
                let asset_filename = ctx.get_file_name(&ref_id).map_err(|e| {
                    anyhow!(
                        "ship-asset-css: generate_bundle: get_file_name(`{ref_id}`) failed: {e}"
                    )
                })?;
                let url = compute_relative_url(css_filename, asset_filename.as_str());
                rewritten.push_str(&body[last_end..prefix_start]);
                rewritten.push_str(&url);
                // The body bytes after the URL are the ref id (now
                // consumed) and the closing `"` of the placeholder
                // (which is part of the URL and stays). Skip past the
                // ref id; the tail flush re-emits the closing `"`,
                // the `)`, the following rules, etc. verbatim.
                last_end = start + ref_id.len();
            }
            rewritten.push_str(&body[last_end..]);
            let final_body = if self.minify {
                deck_core::minify_css(&rewritten)
            } else {
                rewritten
            };
            let new_bytes = final_body.into_bytes();
            if new_bytes.as_slice() == src.as_slice() {
                continue;
            }
            // Replace the asset in the bundle with a new Arc whose
            // body is rewritten. We owned the only reference at this
            // point (the bundle vec holds each output by value), so
            // this is a cheap write-through.
            let new_asset = Arc::new(OutputAsset {
                names: asset.names.clone(),
                original_file_names: asset.original_file_names.clone(),
                filename: asset.filename.clone(),
                source: StrOrBytes::Bytes(new_bytes),
            });
            *output = Output::Asset(new_asset);
        }
        Ok(())
    }
}

/// Emit the given bytes as a CSS asset, register a
/// `__ROLLDOWN_ASSET__#<ref>` placeholder, and return the JS module the
/// bundler will treat as the module's code. The
/// `AssetModulePlugin::render_chunk` (Pre order) rewrites that
/// placeholder in the JS chunk to the CSS's hashed URL — so the
/// entry's CSS `<link>` (or `import`) resolves to the real CSS path.
async fn emit_css_as_asset(
    ctx: SharedLoadPluginContext,
    css_path: &Path,
    bytes: Vec<u8>,
) -> HookLoadReturn {
    let file_name = css_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("style.css")
        .to_string();
    let original_file_name = css_path
        .strip_prefix(ctx.cwd())
        .unwrap_or(css_path)
        .to_string_lossy()
        .into_owned();
    let css_ref = ctx
        .emit_file_async(EmittedAsset {
            name: Some(file_name),
            original_file_name: Some(original_file_name),
            source: StrOrBytes::Bytes(bytes),
            ..Default::default()
        })
        .await
        .with_context(|| {
            format!(
                "ship-asset-css: emit_file_async for CSS `{}`",
                css_path.display()
            )
        })?;
    // The `AssetModulePlugin::render_chunk` scans the chunk code for
    // `__ROLLDOWN_ASSET__#<ref>` and rewrites each match to the
    // asset's hashed relative URL. We piggyback on that infrastructure
    // so the existing JS-side link to the CSS keeps working — the new
    // `__SHIP_CSS_ASSET__#<ref>` placeholders are CSS-BODY-ONLY and
    // never appear in the JS chunk code.
    let placeholder = format!("__ROLLDOWN_ASSET__#{css_ref}");
    let code = format!("module.exports = \"{placeholder}\"");
    Ok(Some(HookLoadOutput {
        code: code.into(),
        module_type: Some(ModuleType::Js),
        ..Default::default()
    }))
}

/// Iterator over `url(...)` references in the CSS body. Yields
/// `(inner_text_offset, quote_char, inner_text)` triples. The
/// `inner_text_offset` is the offset of the inner text (right after
/// the opening quote); the `quote_char` is the original quote
/// character (`'` or `"`); the `inner_text` does NOT include the
/// surrounding quotes. We only return references where the inner text
/// is quoted — bare unquoted `url(x)` (CSS3-legal) is rare in
/// practice and we conservatively skip it.
fn url_references(body: &str) -> Vec<(usize, char, &str)> {
    // `url(` is a fixed prefix; we look for it byte-by-byte to avoid
    // a regex dependency. The first `(` after `url` opens the URL; the
    // matching `)` closes it. CSS permits nested parens in `url()`
    // (rare, RFC 5987-style) — we don't support them here because the
    // generated `sail-base.css` doesn't use them. If a future asset
    // path does, the conservative skip is safer than a broken rewrite.
    let bytes = body.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i + 3 < bytes.len() {
        // Skip CSS comments `/* ... */` (CSS spec 4.3). Comments can
        // contain `url(...)` text — the framework's banner comments
        // include illustrative examples — so we MUST skip them or
        // the parser would try to resolve placeholder filenames like
        // `<file>-<hash>.woff2` as real assets.
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            let mut k = i + 2;
            while k + 1 < bytes.len() && !(bytes[k] == b'*' && bytes[k + 1] == b'/') {
                k += 1;
            }
            i = (k + 2).min(bytes.len());
            continue;
        }
        if &bytes[i..i + 4] == b"url(" {
            let open = i + 3;
            let mut j = open + 1;
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            let quote_byte = bytes[j];
            if quote_byte != b'"' && quote_byte != b'\'' {
                i = j + 1;
                continue;
            }
            let quote_char = quote_byte as char;
            let mut k = j + 1;
            while k < bytes.len() && bytes[k] != quote_byte {
                k += 1;
            }
            if k >= bytes.len() {
                break;
            }
            let mut m = k + 1;
            while m < bytes.len() && bytes[m].is_ascii_whitespace() {
                m += 1;
            }
            if m >= bytes.len() || bytes[m] != b')' {
                i = k + 1;
                continue;
            }
            // `inner_start` is the offset of the first inner character
            // (right after the opening quote). The caller copies
            // `body[..start - 1]` (which excludes the opening quote)
            // and re-emits the standard `"` opener to keep the
            // placeholder's quotes consistent (`css_placeholder_occurrences`
            // in `generate_bundle` only looks for `"`).
            let inner_start = j + 1;
            let inner = &body[inner_start..k];
            out.push((inner_start, quote_char, inner));
            i = m + 1;
        } else {
            i += 1;
        }
    }
    out
}

/// Iterator over `__SHIP_CSS_ASSET__#<ref>` placeholders in a (CSS)
/// body. Yields `(ref_start_offset, ref_id)` pairs — the offset is
/// where the `<ref_id>` text starts (right after the `#`), so the
/// caller can replace `__SHIP_CSS_ASSET__#<ref_id>"` with the resolved
/// URL. The scanner stops at the next `"` because the load hook
/// standardizes the placeholder's quotes to `"`.
fn css_placeholder_occurrences(body: &str) -> Vec<(usize, String)> {
    let bytes = body.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i + CSS_ASSET_PREFIX.len() < bytes.len() {
        if &bytes[i..i + CSS_ASSET_PREFIX.len()] == CSS_ASSET_PREFIX.as_bytes() {
            let ref_start = i + CSS_ASSET_PREFIX.len();
            let mut k = ref_start;
            while k < bytes.len() && bytes[k] != b'"' {
                k += 1;
            }
            if k >= bytes.len() {
                break;
            }
            let ref_id = body[ref_start..k].to_string();
            out.push((ref_start, ref_id));
            i = k + 1;
        } else {
            i += 1;
        }
    }
    out
}

/// `true` for paths the plugin should consider rewriting: NOT a
/// `data:` URL, NOT `http(s):`, NOT `file:`, NOT an absolute
/// filesystem path. Relative paths (`./x.woff2`, `../fonts/x.woff2`,
/// `x.woff2`) qualify.
fn is_local_path(s: &str) -> bool {
    !s.starts_with("data:")
        && !s.starts_with("http:")
        && !s.starts_with("https:")
        && !s.starts_with("file:")
        && !s.starts_with('/')
}

/// `true` if the resolved file path has an extension the plugin knows
/// how to emit as a hashed asset. Conservative — only known static
/// asset types. Other extensions are skipped (the browser will 404 at
/// runtime, which is the pre-existing behaviour, not a regression).
fn has_rewrite_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ext| {
            let ext_with_dot = format!(".{ext}");
            REWRITE_EXTENSIONS
                .iter()
                .any(|known| known.eq_ignore_ascii_case(&ext_with_dot))
        })
}

/// Resolve a URL string (from a CSS `url(...)`) against the CSS file's
/// directory. Handles `./`, `../`, and bare relative paths; absolute
/// paths pass through unchanged. We deliberately don't reach for a URL
/// parser — the generator only emits simple relative paths.
fn resolve_relative(css_dir: &Path, url: &str) -> PathBuf {
    let url_path = Path::new(url);
    if url_path.is_absolute() {
        url_path.to_path_buf()
    } else {
        css_dir.join(url_path)
    }
}

/// Compute the URL string the browser will see for an asset reference,
/// given the CSS asset's emitted filename and the referenced asset's
/// emitted filename. Both live under the same `assets/` directory
/// (Rolldown's default asset path: `assets/[name]-[hash][extname]`),
/// so the relative URL is `./<basename>` in the common case — but we
/// walk the directory tree in case the bundler uses a non-default
/// template (nested subdirs) so the URL stays valid.
fn compute_relative_url(css_filename: &str, asset_filename: &str) -> String {
    let css_dir = Path::new(css_filename)
        .parent()
        .unwrap_or_else(|| Path::new(""));
    let asset_path = Path::new(asset_filename);
    // CSS at the bundle root — relative URL is the asset's basename.
    if css_dir.as_os_str().is_empty() {
        let basename = asset_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(asset_filename);
        return format!("./{basename}");
    }
    // Walk up `css_dir` until we find a prefix of `asset_path`; the
    // `../`s we had to walk plus the remaining suffix is the relative
    // URL. Bounded at 32 to defend against accidental infinite loops on
    // pathological input.
    let mut ups: usize = 0;
    let mut probe = css_dir.to_path_buf();
    loop {
        if asset_path.starts_with(&probe) || ups > 32 {
            let mut rel: PathBuf = std::iter::repeat_n(Component::ParentDir, ups).collect();
            let suffix = asset_path.strip_prefix(&probe).unwrap_or(asset_path);
            if !suffix.as_os_str().is_empty() {
                rel.push(suffix);
            }
            let s = rel.to_string_lossy().into_owned().replace('\\', "/");
            return if s.is_empty() {
                ".".to_string()
            } else if s.starts_with("..") || s.starts_with('/') {
                s
            } else {
                format!("./{s}")
            };
        }
        if !probe.pop() {
            return asset_filename.to_string();
        }
        ups += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_references_skips_comments() {
        // parser must skip `/* ... */` blocks or it would try to
        // resolve `<file>-<hash>.woff2` as a real asset.
        let body = r#"/* illustrative: url('./<file>-<hash>.woff2') */
        @font-face { src: url('./fonts/x.woff2') format('woff2-variations'); }"#;
        let refs: Vec<&str> = url_references(body)
            .into_iter()
            .map(|(_, _q, s)| s)
            .collect();
        assert_eq!(refs, vec!["./fonts/x.woff2"]);
    }

    #[test]
    fn url_references_finds_quoted_paths_and_skips_unquoted_data() {
        // body) are conservatively skipped; in practice the framework
        // only emits base64 inlined woff2 inside the JS source, never
        // in a CSS body (and the previous p2 work used data: URLs only
        // for the inlined-base64 case, which this mission REMOVES).
        let body = r#"@font-face { src: url('./fonts/x.woff2') format('woff2-variations'); }
        .x { background: url("img.png"); }
        .y { background: url(data:image/png;base64,XXX); }"#;
        let refs: Vec<&str> = url_references(body)
            .into_iter()
            .map(|(_, _q, s)| s)
            .collect();
        assert_eq!(refs, vec!["./fonts/x.woff2", "img.png"]);
    }

    #[test]
    fn url_references_skips_unquoted() {
        let body = "url(x)";
        assert!(url_references(body).is_empty());
    }

    #[test]
    fn has_rewrite_extension_matches_known() {
        assert!(has_rewrite_extension(Path::new("a.woff2")));
        assert!(has_rewrite_extension(Path::new("a.WOFF2")));
        assert!(has_rewrite_extension(Path::new("a.png")));
        assert!(!has_rewrite_extension(Path::new("a.css")));
        assert!(!has_rewrite_extension(Path::new("a.ts")));
        assert!(!has_rewrite_extension(Path::new("noext")));
    }

    #[test]
    fn is_local_path_filters_schemes() {
        assert!(is_local_path("./fonts/x.woff2"));
        assert!(is_local_path("../fonts/x.woff2"));
        assert!(is_local_path("x.woff2"));
        assert!(!is_local_path("data:font/woff2;base64,XXX"));
        assert!(!is_local_path("https://cdn/x.woff2"));
        assert!(!is_local_path("file:///etc/x.woff2"));
        assert!(!is_local_path("/abs/path.woff2"));
    }

    #[test]
    fn compute_relative_url_under_assets() {
        // Both under assets/ → relative is just the basename.
        let url = compute_relative_url("assets/sail-base-HASH.css", "assets/dm-sans-OTHER.woff2");
        assert_eq!(url, "./dm-sans-OTHER.woff2");
    }

    #[test]
    fn compute_relative_url_nested_dirs() {
        // If assets are nested, the relative math should still produce a
        // valid URL.
        let url = compute_relative_url(
            "assets/sub/sail-base-HASH.css",
            "assets/dm-sans-OTHER.woff2",
        );
        assert_eq!(url, "../dm-sans-OTHER.woff2");
    }

    #[test]
    fn css_placeholder_occurrences_extracts_refs() {
        let body = r#"url("__SHIP_CSS_ASSET__#r1") url("__SHIP_CSS_ASSET__#r2")"#;
        let occ: Vec<String> = css_placeholder_occurrences(body)
            .into_iter()
            .map(|(_, s)| s)
            .collect();
        assert_eq!(occ, vec!["r1", "r2"]);
    }

    #[test]
    fn resolve_relative_basic() {
        let css_dir = Path::new("/repo/packages/sail/tokens/src");
        assert_eq!(
            resolve_relative(css_dir, "./fonts/x.woff2"),
            Path::new("/repo/packages/sail/tokens/src/fonts/x.woff2")
        );
        assert_eq!(
            resolve_relative(css_dir, "../shared/y.woff2"),
            Path::new("/repo/packages/sail/tokens/src/../shared/y.woff2")
        );
    }

    // ── End-to-end through Rolldown ──────────────────────────────────────
    //
    // The unit tests above cover the helper functions. This test
    // drives the plugin through the actual Rolldown `Bundler` to
    // verify the `load` → `generate_bundle` hook chain works
    // end-to-end: a `.css` file with a `url('./fonts/x.woff2')`
    // reference is loaded, the woff2 is emitted as a hashed asset,
    // and the CSS body's URL is rewritten to the hashed path.
    //
    // Why a separate test: the helper functions are pure; the
    // plugin's behavior depends on `ctx.emit_file_async` (which
    // requires a real `PluginContext`), `ctx.get_file_name` (same),
    // and the `Output::Asset` mutation in `generate_bundle`. Only a
    // real Rolldown pass exercises all three.
    #[tokio::test(flavor = "multi_thread")]
    async fn plugin_rewrites_css_url_end_to_end() {
        use rolldown::{BundlerBuilder, BundlerOptions, InputItem, OutputFormat};
        use rolldown_common::ChunkFilenamesOutputOption;
        use std::sync::Arc;

        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let entry = root.join("entry.js");
        let css = root.join("style.css");
        let woff2 = root.join("fonts/x.woff2");
        // 4 bytes of fake woff2 content — Rolldown hashes the source
        // for the asset filename, so any deterministic content works.
        let woff2_bytes: &[u8] = b"wOF2";
        let css_body = r#"@font-face { font-family: "X"; src: url('./fonts/x.woff2') format('woff2-variations'); }"#;
        let entry_body = "import './style.css';\nexport const _ = 1;\n";

        std::fs::create_dir_all(root.join("fonts")).unwrap();
        std::fs::write(&entry, entry_body).unwrap();
        std::fs::write(&css, css_body).unwrap();
        std::fs::write(&woff2, woff2_bytes).unwrap();

        let out_dir = root.join("dist");
        std::fs::create_dir_all(&out_dir).unwrap();

        let mut module_types: rustc_hash::FxHashMap<String, rolldown_common::ModuleType> =
            rustc_hash::FxHashMap::default();
        module_types.insert(".css".to_string(), rolldown_common::ModuleType::Asset);
        let options = BundlerOptions {
            input: Some(vec![InputItem {
                name: Some("entry".to_string()),
                import: entry.to_string_lossy().to_string(),
            }]),
            cwd: Some(root.to_path_buf()),
            dir: Some(out_dir.to_string_lossy().to_string()),
            format: Some(OutputFormat::Esm),
            entry_filenames: Some(ChunkFilenamesOutputOption::String(
                "assets/[name]-[hash].js".to_string(),
            )),
            chunk_filenames: Some(ChunkFilenamesOutputOption::String(
                "assets/[name]-[hash].js".to_string(),
            )),
            module_types: Some(module_types),
            ..Default::default()
        };
        let mut bundler = BundlerBuilder::default()
            .with_options(options)
            .with_plugins(vec![Arc::new(ShipAssetCssPlugin::development())])
            .build()
            .unwrap();
        let output = bundler.generate().await.unwrap();

        // Find the CSS asset. It must (a) exist, (b) be under
        // `assets/`, and (c) its body must contain the rewritten
        // URL pointing at the woff2's hashed asset.
        let css_asset = output
            .assets
            .iter()
            .find_map(|a| match a {
                rolldown_common::Output::Asset(asset) => {
                    let s = String::from_utf8_lossy(asset.source.as_bytes());
                    if s.contains("@font-face") {
                        Some((asset, s.into_owned()))
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .expect("CSS asset must be emitted");
        let (asset, body) = css_asset;
        assert!(
            asset.filename.starts_with("assets/"),
            "CSS asset must land under `assets/`, got: {}",
            asset.filename
        );
        assert!(
            !body.contains("__SHIP_CSS_ASSET__#"),
            "the placeholder must be resolved by `generate_bundle`, got: {body}"
        );
        assert!(
            !body.contains("./fonts/x.woff2"),
            "the source `url('./fonts/x.woff2')` must be rewritten, got: {body}"
        );
        assert!(
            !body.contains("data:font/woff2"),
            "no base64 should appear in the CSS, got: {body}"
        );
        // The body must reference the woff2 by a `url(...)` whose
        // basename is the woff2 filename. The hashed prefix is
        // content-derived; we just assert the suffix and a
        // `url(` wrapper. The standardized quote style is `"` (the
        // load hook rewrites the original `'` to `"` so the
        // `generate_bundle` scanner finds a consistent delimiter).
        assert!(
            body.contains(".woff2"),
            "the rewritten URL must reference `*.woff2`, got: {body}"
        );
        assert!(
            body.contains(".woff2\")"),
            "the rewritten URL must keep the closing `\"` of the placeholder, got: {body}"
        );

        // The woff2 must also be emitted as a separate asset under
        // `assets/`. `bundler.generate()` returns the in-memory
        // bundle; check the assets list for a second `Output::Asset`
        // whose filename ends with `.woff2`.
        let woff2_emitted: Vec<_> = output
            .assets
            .iter()
            .filter_map(|a| match a {
                rolldown_common::Output::Asset(asset) => {
                    let s = &*asset.filename;
                    if s.ends_with(".woff2") {
                        Some(s.to_string())
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .collect();
        assert_eq!(
            woff2_emitted.len(),
            1,
            "the woff2 must be emitted as exactly one hashed asset, got: {woff2_emitted:?}"
        );
    }

    /// Production path: URL rewrite still works AND body is minified
    /// (comments stripped / smaller than the unminified sibling).
    #[tokio::test(flavor = "multi_thread")]
    async fn plugin_minifies_css_in_production_without_breaking_url_rewrite() {
        use rolldown::{BundlerBuilder, BundlerOptions, InputItem, OutputFormat};
        use rolldown_common::ChunkFilenamesOutputOption;
        use std::sync::Arc;

        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let entry = root.join("entry.js");
        let css = root.join("style.css");
        let woff2 = root.join("fonts/x.woff2");
        let woff2_bytes: &[u8] = b"wOF2";
        // Deliberately verbose CSS so minify has room to shrink.
        let css_body = r#"/* framework banner — should vanish in production */
@font-face {
  font-family: "X";
  src: url('./fonts/x.woff2') format('woff2-variations');
}
.foo  {
  color:  red;
}
"#;
        let entry_body = "import './style.css';\nexport const _ = 1;\n";

        std::fs::create_dir_all(root.join("fonts")).unwrap();
        std::fs::write(&entry, entry_body).unwrap();
        std::fs::write(&css, css_body).unwrap();
        std::fs::write(&woff2, woff2_bytes).unwrap();

        let out_dir = root.join("dist");
        std::fs::create_dir_all(&out_dir).unwrap();

        let mut module_types: rustc_hash::FxHashMap<String, rolldown_common::ModuleType> =
            rustc_hash::FxHashMap::default();
        module_types.insert(".css".to_string(), rolldown_common::ModuleType::Asset);

        let make_options = |dir: &Path| BundlerOptions {
            input: Some(vec![InputItem {
                name: Some("entry".to_string()),
                import: entry.to_string_lossy().to_string(),
            }]),
            cwd: Some(root.to_path_buf()),
            dir: Some(dir.to_string_lossy().to_string()),
            format: Some(OutputFormat::Esm),
            entry_filenames: Some(ChunkFilenamesOutputOption::String(
                "assets/[name]-[hash].js".to_string(),
            )),
            chunk_filenames: Some(ChunkFilenamesOutputOption::String(
                "assets/[name]-[hash].js".to_string(),
            )),
            module_types: Some(module_types.clone()),
            ..Default::default()
        };

        let mut dev_bundler = BundlerBuilder::default()
            .with_options(make_options(&out_dir.join("dev")))
            .with_plugins(vec![Arc::new(ShipAssetCssPlugin::development())])
            .build()
            .unwrap();
        let dev_out = dev_bundler.generate().await.unwrap();
        let dev_body = dev_out
            .assets
            .iter()
            .find_map(|a| match a {
                rolldown_common::Output::Asset(asset) => {
                    let s = String::from_utf8_lossy(asset.source.as_bytes());
                    if s.contains("@font-face") || s.contains("font-face") {
                        Some(s.into_owned())
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .expect("dev CSS asset");

        let mut prod_bundler = BundlerBuilder::default()
            .with_options(make_options(&out_dir.join("prod")))
            .with_plugins(vec![Arc::new(ShipAssetCssPlugin::production())])
            .build()
            .unwrap();
        let prod_out = prod_bundler.generate().await.unwrap();
        let prod_body = prod_out
            .assets
            .iter()
            .find_map(|a| match a {
                rolldown_common::Output::Asset(asset) => {
                    let s = String::from_utf8_lossy(asset.source.as_bytes());
                    if s.contains("@font-face") || s.contains("font-face") || s.contains(".woff2") {
                        Some(s.into_owned())
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .expect("prod CSS asset");

        assert!(
            prod_body.len() < dev_body.len(),
            "production CSS must shrink: prod={} dev={}",
            prod_body.len(),
            dev_body.len()
        );
        assert!(
            !prod_body.contains("/*"),
            "production CSS strips comments: {prod_body}"
        );
        assert!(
            !prod_body.contains("__SHIP_CSS_ASSET__#"),
            "placeholders resolved before minify: {prod_body}"
        );
        assert!(
            !prod_body.contains("./fonts/x.woff2"),
            "source font path rewritten: {prod_body}"
        );
        assert!(
            prod_body.contains(".woff2"),
            "hashed woff2 url() survives minify: {prod_body}"
        );
    }
}
