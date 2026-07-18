//! SAIL resolve/load plugin for Rolldown. Handles:
//!   - `resolve_id`: redirect `./X.compiled.js` siblings to a virtual id,
//!     externalize the framework singletons, re-anchor `@ship/<pkg>` to
//!     workspace `src/`, resolve `@sail-tag/<tag>` to the component source.
//!   - `load`: compile `.sail` (and `.compiled.js`→sibling `.sail`) via
//!     sail-compiler, returning JS.

use std::borrow::Cow;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use anyhow::{anyhow, Result};
use rolldown::plugin::{
    HookLoadArgs, HookLoadOutput, HookLoadReturn, HookResolveIdArgs, HookResolveIdOutput,
    HookResolveIdReturn, HookUsage, Plugin, PluginContext, SharedLoadPluginContext,
};
use rolldown_common::{ModuleType, ResolvedExternal, side_effects::HookSideEffects};


use crate::compile::{compile_sui, CompileMode};

// NOTE: `@ship/sdk-client` was here historically (deck-self-typed as a
// framework singleton). The deck-jsonc-simplify cutover moves it to
// provider-declared discovery: the package self-declares its serve
// contract in `package.json#deckFramework`, the bundler discovers it
// via the app's dep graph, and `AppPlugin` externalizes whatever the
// importmap declares. Hardcoding the alias here would silently anchor
// `@ship/sdk-client` to workspace source — the OPPOSITE of what the
// importmap says — and break the host-declared singleton pattern.
const SINGLETONS: [&str; 2] = ["@ship/sail-runtime", "@ship/sail-dom"];

/// Virtual id for an unresolved `@sail-tag/<tag>` import — a shared empty module
/// so the import is a harmless no-op. `\0` marks it virtual (Rollup/Rolldown
/// convention: never read/written on disk).
const SAIL_TAG_NOOP: &str = "\0sail-tag-noop";

/// Detect compile mode for a `.sail` file using the HTML custom-element tag-name
/// rule: a valid custom element tag MUST contain a hyphen (HTML spec §4.13.3).
/// Every component `.sail` file uses its stem as the tag name (`sail-button`,
/// `sail-rail-item`) and therefore contains a hyphen. Every view uses `page.sail`
/// or `index.sail` — no hyphen. So the rule is exact and path-structure-free:
///
///   stem contains `'-'` AND stem ∉ {"page", "index"} → Component
///   otherwise → View
///
/// This replaces the old `stem == parent_dir` heuristic, which missed all
/// GROUPED-DIRECTORY components (e.g. `sail-rail/sail-rail-item.sail`,
/// `sail-command-bar/sail-command-chat.sail`) and required a special-case for
/// `packages/components/ui-components/src`. That shortcut is now subsumed by
/// this rule (every `.sail` in that tree has a hyphenated stem) and is removed.
pub(crate) fn detect_mode(abs: &Path, _repo_root: &Path) -> (CompileMode, String) {
    let stem = abs.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
    let mode = if stem.contains('-') && stem != "page" && stem != "index" {
        CompileMode::Component
    } else {
        CompileMode::View
    };
    (mode, stem)
}

pub(crate) fn normalize(p: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in p.components() {
        match c {
            std::path::Component::ParentDir => {
                out.pop();
            }
            std::path::Component::CurDir => {}
            o => out.push(o.as_os_str()),
        }
    }
    out
}

/// SAIL compile plugin — `resolve_id` redirects `*.compiled.js` siblings to a
/// virtual id and re-anchors `@ship/<pkg>` to workspace `src/`; `load` compiles
/// the sibling `.sail` via sail-compiler. Reused by the app build (`app.rs`) so
/// the SPA graph compiles its own `.sail` in-process — one compile path, no
/// on-disk `.compiled.js` pre-step.
#[derive(Debug)]
pub struct SailPlugin {
    pub repo_root: PathBuf,
    /// View builds opt in: each `.sail` compiled in this build emits
    /// `@sail-tag/<tag>` registration imports (the per-view resolver). Off for
    /// framework/app builds.
    pub emit_component_imports: bool,
    /// `<tag>` → component source `.sail`, for resolving `@sail-tag/<tag>`.
    /// Empty when `emit_component_imports` is false.
    pub component_index: Arc<HashMap<String, PathBuf>>,
    /// `<tag>` → providing-pack name, for every tag a declared-`packs` scope
    /// EXCLUDES. Empty under global resolution. When a built view references one
    /// of these tags, the resolver no-ops it AND warns the author (naming the
    /// pack to declare).
    pub shadowed_tags: Arc<HashMap<String, String>>,
    /// npm package name → absolute package root, for resolving a `@ship/<name>`
    /// import to its workspace source. DERIVED from the bun/npm workspace package
    /// map (`component_roots`) — deck bakes no monorepo layout and keeps
    /// the resolver's idea of "workspace" identical to bun's. Empty when the
    /// host's importmap is the source of truth.
    pub component_roots: Arc<HashMap<String, PathBuf>>,
}

/// The per-view resolver's lookup tables, built once per `(repo_root, packs)`.

/// `index` resolves `@sail-tag/<tag>` to component source; `shadowed` explains a
/// tag a declared scope excluded (empty under global resolution).
#[derive(Clone)]
pub struct ResolverIndex {
    pub index: Arc<HashMap<String, PathBuf>>,
    pub shadowed: Arc<HashMap<String, String>>,
}

impl ResolverIndex {
    /// Empty resolver — for builds that emit no `@sail-tag` imports (framework/
    /// app non-view targets).
    pub(crate) fn empty() -> Self {
        Self { index: Arc::new(HashMap::new()), shadowed: Arc::new(HashMap::new()) }
    }
}

impl SailPlugin {
    /// Compile a `.sail` to JS via `sail_compiler::compile` IN-PROCESS (#255):
    /// the compiler is the linked crate, not a spawned binary — no temp dir, no
    /// file roundtrip, no version drift, no flag that a stale binary can reject.
    fn compile(&self, sail: &Path) -> Result<String> {
        let (mode, stem) = detect_mode(sail, &self.repo_root);
        compile_sui(sail, mode, &stem, self.emit_component_imports).map_err(|e| {
            // Surface the real compiler error: rolldown collapses a plugin error
            // to a generic "plugin `ship-sail` threw an error", swallowing the
            // message/line/col. Print it so a consumer can see WHY a view failed
            // to compile (deck invariant: compile errors are surfaced, not swallowed).
            eprintln!("deck ship-sail: compile {} failed:\n  {e}", sail.display());
            anyhow!("compile {}: {e}", sail.display())
        })
    }
}

impl Plugin for SailPlugin {
    fn name(&self) -> Cow<'static, str> {
        "ship-sail".into()
    }

    async fn resolve_id(
        &self,
        _ctx: &PluginContext,
        args: &HookResolveIdArgs<'_>,
    ) -> HookResolveIdReturn {
        let spec = args.specifier;
        // Framework singletons stay external (one runtime instance).
        if SINGLETONS.contains(&spec) {
            return Ok(Some(HookResolveIdOutput {
                id: spec.into(),
                external: Some(ResolvedExternal::Bool(true)),
                ..Default::default()
            }));
        }
        // `@sail-tag/<tag>` — the per-view resolver. Map to the component's
        // source `.sail` (so it compiles into the view bundle); an unknown tag
        // (view-private, or no CEM yet) → virtual no-op. Emitted only under
        // `--emit-component-imports`.
        if let Some(tag) = spec.strip_prefix("@sail-tag/") {
            return Ok(Some(match self.component_index.get(tag) {
                Some(src) => HookResolveIdOutput::from_id(src.to_string_lossy().to_string()),
                None => {
                    // A scoped-out tag (exists in the workspace, but its pack is
                    // not in this project's declared `packs`) gets an author
                    // diagnostic — it would otherwise vanish silently. A tag the
                    // workspace doesn't provide at all (view-private, HTML, typo)
                    // is not shadowed and stays a quiet no-op.
                    if let Some(pack) = self.shadowed_tags.get(tag) {
                        let view = args
                            .importer
                            .map(|i| {
                                Path::new(i)
                                    .strip_prefix(&self.repo_root)
                                    .unwrap_or(Path::new(i))
                                    .display()
                                    .to_string()
                            })
                            .unwrap_or_else(|| "<entry>".to_string());
                        tracing::warn!(
                            tag,
                            pack,
                            view,
                            "deck: `{tag}` is provided by pack `{pack}`, not declared in \
                             deck.jsonc `packs` — it will not render. Add `{pack}` to `packs`."
                        );
                    }
                    // Resolve to a shared empty module — a harmless no-op. NOT
                    // external, which would leave an unresolvable
                    // `@sail-tag/<tag>` specifier in the served view.
                    HookResolveIdOutput::from_id(SAIL_TAG_NOOP.to_string())
                }
            }));
        }
        // `<file>.sail.controller.ts` → virtual id: the inline `<controller>`
        // block of the named `.sail`, served by `load` (SFC — nothing on disk).
        // Must be redirected here: the native resolver would fail on the
        // nonexistent file. Relative (`./page.sail.controller.ts`, what the
        // compiled `.sail` module imports) and absolute (the synthetic client
        // emit) shapes both land on the normalized absolute id.
        if spec.ends_with(".sail.controller.ts") {
            if Path::new(spec).is_absolute() {
                return Ok(Some(HookResolveIdOutput::from_id(
                    normalize(Path::new(spec)).to_string_lossy().to_string(),
                )));
            }
            if let Some(importer) = args.importer {
                let dir = Path::new(importer).parent().unwrap_or(&self.repo_root);
                let abs = normalize(&dir.join(spec));
                return Ok(Some(HookResolveIdOutput::from_id(abs.to_string_lossy().to_string())));
            }
        }
        // `*.compiled.js` → virtual id; `load` compiles the sibling `.sail`. The
        // `.compiled.js` is synthetic (never on disk), so we must redirect here
        // rather than fall through to Rolldown's native resolver, which would
        // fail to resolve a nonexistent file. Two shapes reach here: relative
        // `./X.compiled.js` siblings (umbrella `index.ts` imports), and the
        // ABSOLUTE paths the virtual-aggregate entry emits — both must become
        // a virtual id.
        if spec.ends_with(".compiled.js") {
            if Path::new(spec).is_absolute() {
                return Ok(Some(HookResolveIdOutput::from_id(
                    normalize(Path::new(spec)).to_string_lossy().to_string(),
                )));
            }
            if (spec.starts_with("./") || spec.starts_with("../"))
                && let Some(importer) = args.importer
            {
                let dir = Path::new(importer).parent().unwrap_or(&self.repo_root);
                let abs = normalize(&dir.join(spec));
                return Ok(Some(HookResolveIdOutput::from_id(abs.to_string_lossy().to_string())));
            }
        }
        // `@ship/<non-singleton>` → workspace package source (not prebuilt
        // dist) so its components compile from source too. A bare specifier
        // (`@ship/tokens`) resolves to `src/index.ts`; a subpath
        // (`@ship/tokens/themes`) resolves to the matching file
        // under `src/` — honoring the package's `exports` subpath map. Without
        // this the subpath collapsed to `src/index.ts`, surfacing as a
        // `MissingExport` for the named imports that live in the subpath file.
        if let Some(rest) = spec.strip_prefix("@ship/") {
            let name = rest.split('/').next().unwrap_or(rest);
            let subpath = rest.strip_prefix(name).map(|s| s.trim_start_matches('/'));
            // Resolve `@ship/<name>` via the DERIVED workspace package map
            // (`component_roots`, keyed by package.json `name`) — the same map
            // bun/npm resolve through. No hand-maintained search order, no
            // `sail-` dir-name heuristic: the map IS the authoritative
            // name→source binding. A name absent from the map (a prod build with
            // no workspace, a true node_modules dep) falls through to Rolldown's
            // native resolver below. (There is no umbrella bundle — chrome is
            // per-tag tree-shaking only.)
            if let Some(pkg) = self.component_roots.get(&format!("@ship/{name}")) {
                if pkg.join("src/index.ts").exists() {
                    let src = pkg.join("src");
                    let target = match subpath {
                        None | Some("") => src.join("index.ts"),
                        Some(sub) => {
                            // `tokens/themes` → tokens/themes.ts, or its index.ts.
                            let direct = src.join(sub);
                            if direct.extension().is_some() && direct.exists() {
                                direct
                            } else {
                                let with_ext = src.join(format!("{sub}.ts"));
                                if with_ext.exists() {
                                    with_ext
                                } else {
                                    src.join(sub).join("index.ts")
                                }
                            }
                        }
                    };
                    if target.exists() {
                        // Don't surface the owning `package.json` here.
                        // If we do, Rolldown's `lazy_check_side_effects`
                        // honors `package.json#sideEffects: false` and
                        // drops the body — but a bare `import
                        // '@ship/<pkg>'` is a deliberate side-effect
                        // request from the importer, and the package's
                        // `sideEffects: false` is a tree-shaking hint
                        // for tooling that can't tell a binding import
                        // from a side-effect import. The deck bundler
                        // CAN, so the importer's intent wins. Setting
                        // `package_json_path: None` makes Rolldown
                        // fall through to its analyzer (any source
                        // statement with a side effect keeps the body)
                        // and per-statement tree-shaking still shakes
                        // dead exports inside the kept body.
                        return Ok(Some(HookResolveIdOutput::from_id(
                            target.to_string_lossy().to_string(),
                        )));
                    }
                }
            }
        }
        // Everything else: let Rolldown's native resolver handle it (relative
        // .ts/.js, node_modules, extension resolution).
        Ok(None)
    }

    async fn load(&self, _ctx: SharedLoadPluginContext, args: &HookLoadArgs<'_>) -> HookLoadReturn {
        let id = args.id;
        if id == SAIL_TAG_NOOP {
            // Empty no-op for an unresolved component tag. Tree-shaken away
            // if nothing keeps it.
            return Ok(Some(HookLoadOutput {
                code: String::new().into(),
                module_type: Some(ModuleType::Js),
                ..Default::default()
            }));
        }
        // Virtual inline-controller module (`<file>.sail.controller.ts`): the
        // `<controller>` block extracted from the named `.sail`, returned as TS
        // so Rolldown's transform strips the types. Checked BEFORE the generic
        // `.ts` read below — the file never exists on disk.
        if let Some(base) = id.strip_suffix(".controller.ts")
            && base.ends_with(".sail")
        {
            let sail = Path::new(base);
            let source = std::fs::read_to_string(sail).map_err(|e| {
                anyhow!("read {} for its inline <controller>: {e}", sail.display())
            })?;
            let body = sail_compiler::parse::preprocess::extract_controller_block(&source)
                .1
                .ok_or_else(|| {
                    anyhow!(
                        "`{id}` names the inline controller of `{}`, but that file has no \
                         <controller> block",
                        sail.display()
                    )
                })?;
            // Lower `query()`/`mutation()`/`invalidate()` before serving —
            // see `sail_compiler::check::query_keys`. A refusal here is a
            // real dev-server compile error, not a runtime one. The
            // rewrite itself injects the `__invalidate` import a lowered
            // `invalidate()` call needs, so the served module is
            // self-contained without running the full ambient-import wrap
            // (which this path has never run, and which auto-imports a
            // wider set of names this hook has no reason to newly affect).
            let body = sail_compiler::check::query_keys::rewrite(&body).map_err(|e| {
                anyhow!("{}", e.render(&body))
            })?;
            return Ok(Some(HookLoadOutput {
                code: body.into(),
                module_type: Some(ModuleType::Ts),
                ..Default::default()
            }));
        }
        let sail: Option<PathBuf> = if let Some(base) = id.strip_suffix(".compiled.js") {
            Some(PathBuf::from(format!("{base}.sail")))
        } else if id.ends_with(".sail") {
            Some(PathBuf::from(id))
        } else {
            None
        };
        if let Some(sail) = sail {
            let code = self
                .compile(&sail)
                .map_err(|e| anyhow!("compile {}: {e}", sail.display()))?;
            return Ok(Some(HookLoadOutput {
                code: code.into(),
                module_type: Some(ModuleType::Js),
                ..Default::default()
            }));
        }
        // Real .ts/.js — we read the source and override
        // `side_effects` to `True` so the importer's bare
        // `import 'x'` keeps the body regardless of
        // `package.json#sideEffects: false` (sail-sideeffects
        // footgun). Per-statement tree-shaking still applies
        // inside the kept body, so a barrel's dead export is
        // still shaken.
        // Skip CSS / JSON entirely — the built-in AssetModulePlugin
        // handles them via the `module_types: { ".css" -> Asset }` map
        // set on `BundlerOptions`. Returning anything here (even
        // `side_effects: Some(True)`) would short-circuit the asset
        // pipeline and trip the "module with type `asset` during AST
        // parsing" error. Side-effect preservation for CSS is moot
        // anyway: the AssetModulePlugin emits the CSS file as a hashed
        // `assets/<name>-<hash>.css` regardless of `package.json#sideEffects`
        // (the asset IS the body, there's no JS to drop).
        if id.ends_with(".css") || id.ends_with(".json") {
            return Ok(None);
        }
        let module_type = if id.ends_with(".tsx") {
            ModuleType::Tsx
        } else if id.ends_with(".ts") {
            ModuleType::Ts
        } else if id.ends_with(".jsx") {
            ModuleType::Jsx
        } else {
            ModuleType::Js
        };
        // Read the source. A read error is propagated as a
        // build diagnostic (fail-loud per the "no silent
        // errors" rule) — silently emitting an empty `code`
        // would surface downstream as a confusing
        // `MissingExport` on a named-import consumer, masking
        // the real cause (a missing or unreadable file).
        let code = std::fs::read_to_string(id)
            .map_err(|e| {
                anyhow::anyhow!(
                    "deck-bundler: failed to read module `{id}`: {e}"
                )
            })?;
        Ok(Some(HookLoadOutput {
            code: code.into(),
            side_effects: Some(HookSideEffects::True),
            module_type: Some(module_type),
            ..Default::default()
        }))
    }

    fn register_hook_usage(&self) -> HookUsage {
        HookUsage::ResolveId | HookUsage::Load
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile::CompileMode;

    fn fake_root() -> PathBuf {
        PathBuf::from("/repo")
    }

    fn detect(path: &str) -> CompileMode {
        detect_mode(Path::new(path), &fake_root()).0
    }

    /// GROUPED-DIRECTORY component: stem differs from parent dir but has a hyphen.
    #[test]
    fn grouped_dir_component_detected_as_component() {
        assert_eq!(
            detect("/repo/apps/shell/src/components/sail-rail/sail-rail-item.sail"),
            CompileMode::Component,
            "sail-rail-item.sail must be a Component (hyphenated stem)"
        );
    }

    /// Views use page.sail (no hyphen in stem).
    #[test]
    fn page_sui_detected_as_view() {
        assert_eq!(
            detect("/repo/routes/agents/page.sail"),
            CompileMode::View,
            "page.sail must be a View"
        );
    }

    /// SAME-NAME component: the classic convention still works.
    #[test]
    fn same_name_component_detected_as_component() {
        assert_eq!(
            detect("/repo/packages/components/sail/primitives/src/sail-button/sail-button.sail"),
            CompileMode::Component,
            "sail-button/sail-button.sail must be a Component"
        );
    }

    /// index.sail is a view even though it could theoretically have a hyphen
    /// (no known case, but the exclusion is correct by spec).
    #[test]
    fn index_sui_detected_as_view() {
        assert_eq!(
            detect("/repo/.ship/views/component-viewer/index.sail"),
            CompileMode::View,
            "index.sail must be a View"
        );
    }

    /// The stem is returned unchanged as the tag/name.
    #[test]
    fn detect_mode_returns_stem_as_tag() {
        let (_, tag) = detect_mode(
            Path::new("/repo/apps/shell/src/components/sail-command-bar/sail-command-chat.sail"),
            &fake_root(),
        );
        assert_eq!(tag, "sail-command-chat");
    }

    fn write(path: &Path, body: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, body).unwrap();
    }

    /// Two-pack fixture: a declared primitives pack + an undeclared agents pack.
    fn two_pack_fixture() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(&root.join("package.json"), r#"{"workspaces":["packages/*"]}"#);
        write(
            &root.join("packages/primitives/package.json"),
            r#"{"name":"@ship/sail-primitives"}"#,
        );
        write(
            &root.join("packages/primitives/src/sail-button/sail-button.sail"),
            "<button><slot></slot></button>\n",
        );
        write(&root.join("packages/agents/package.json"), r#"{"name":"@ship/agents"}"#);
        write(
            &root.join("packages/agents/src/ship-agent-card/ship-agent-card.sail"),
            "<article><slot></slot></article>\n",
        );
        tmp
    }

    /// `component_index` assembles BOTH halves: a declared scope yields a scoped
    /// index (declared tag in, undeclared tag out) and a shadowed map naming the
    /// excluded tag's pack — the data the resolver's author diagnostic reads.
    #[test]
    fn component_index_scoped_carries_index_and_shadowed() {
        let tmp = two_pack_fixture();
        let root = tmp.path();
        let declared: Vec<String> = vec!["@ship/sail-primitives".into()];

        let scoped = component_index(root, &declared);
        assert!(scoped.index.contains_key("sail-button"), "declared tag resolves");
        assert!(
            !scoped.index.contains_key("ship-agent-card"),
            "undeclared tag is excluded from the index"
        );
        assert_eq!(
            scoped.shadowed.get("ship-agent-card").map(String::as_str),
            Some("@ship/agents"),
            "excluded tag is shadowed with its providing pack (for the diagnostic)"
        );
    }

    /// Global (empty `packs`): every tag resolves and nothing is shadowed, so
    /// the diagnostic never fires — zero behavior change for existing projects.
    #[test]
    fn component_index_global_resolves_all_and_shadows_nothing() {
        let tmp = two_pack_fixture();
        let root = tmp.path();
        let global = component_index(root, &[]);
        assert!(global.index.contains_key("sail-button"));
        assert!(global.index.contains_key("ship-agent-card"));
        assert!(global.shadowed.is_empty(), "global scope shadows nothing");
    }

    /// Watch-mode invalidation: the index is memoized process-globally, so a
    /// component ADDED mid-session is invisible until `invalidate_component_index`
    /// clears the cache (the documented hazard — the new tag would otherwise
    /// resolve to a virtual no-op until restart). Proves stale-then-fresh across
    /// the clear. The temp root is unique, so this test owns its cache key; only
    /// this test invalidates, so the `stale` read is deterministic.
    #[test]
    fn invalidate_component_index_picks_up_a_mid_session_component() {
        let tmp = two_pack_fixture();
        let root = tmp.path();

        // First walk caches the index — sail-button present, sail-late absent.
        let first = component_index(root, &[]);
        assert!(first.index.contains_key("sail-button"));
        assert!(!first.index.contains_key("sail-late"), "the new tag does not exist yet");

        // A component appears mid-session (what an author does under `deck dev`).
        write(
            &root.join("packages/primitives/src/sail-late/sail-late.sail"),
            "<span><slot></slot></span>\n",
        );

        // Without invalidation the cache still serves the STALE index — the bug.
        let stale = component_index(root, &[]);
        assert!(
            !stale.index.contains_key("sail-late"),
            "cached index is stale: the mid-session component is invisible (the hazard)"
        );

        // The watcher's structural-change hook clears the cache; the re-walk sees it.
        invalidate_component_index();
        let fresh = component_index(root, &[]);
        assert!(
            fresh.index.contains_key("sail-late"),
            "after invalidation the re-walk picks up the new component"
        );
        assert!(fresh.index.contains_key("sail-button"), "and keeps the original");
    }
}

/// Cached `<tag>` → source `.sail` index for a repo (the per-view resolver's
/// lookup). Memoized per `(repo_root, packs)` so a one-shot build walks the CEM
/// once. Under `deck dev`'s watcher a component can be ADDED/REMOVED mid-session,
/// so the watcher calls [`invalidate_component_index`] on a structural change to
/// clear this cache — without that, a newly-authored tag resolves to a virtual
/// no-op until process restart (the hazard this memo used to assert away).
///
/// `` comes from `deck.jsonc` and drives the shell-local CEM
/// projection (which app root gets a special origin label). `None` in
/// zero-config hosts — the projection simply doesn't apply.
///
/// `packs` is the project's declared component scope (`deck.jsonc packs`). When
/// non-empty the index is restricted to those packs (per-project access
/// control) and `shadowed` carries every excluded tag → its pack for author
/// diagnostics; when empty the index is global and nothing is shadowed. The
/// cache is keyed by `(repo_root, packs)` so two projects sharing a repo root
/// but declaring different packs don't collide.
type ComponentIndexCacheKey = (PathBuf, Vec<String>);
static COMPONENT_INDEX_CACHE: OnceLock<Mutex<HashMap<ComponentIndexCacheKey, ResolverIndex>>> =
    OnceLock::new();

fn component_index_cache() -> &'static Mutex<HashMap<ComponentIndexCacheKey, ResolverIndex>> {
    COMPONENT_INDEX_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn component_index(
    repo_root: &Path,
    packs: &[String],
) -> ResolverIndex {
    let cache = component_index_cache();
    let key: ComponentIndexCacheKey = (repo_root.to_path_buf(), packs.to_vec());
    let mut guard = cache.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(idx) = guard.get(&key) {
        return idx.clone();
    }
    let resolver = ResolverIndex {
        index: Arc::new(crate::catalog::tag_source_index_scoped(
            repo_root,
            Some(packs),
        )),
        shadowed: Arc::new(crate::catalog::shadowed_tags(repo_root, packs)),
    };
    guard.insert(key, resolver.clone());
    resolver
}

/// Clear the memoized component index. The CEM walk is cached process-globally on
/// the assumption it's stable for the life of the process — TRUE for a one-shot
/// build, FALSE under `deck dev`'s watcher, where a component added or removed
/// mid-session would otherwise stay invisible (its tag resolving to a virtual
/// no-op) until restart. [`Bundler::invalidate_targets_for_watch`] calls this on
/// a structural change so the next build re-walks the CEM and sees the new or
/// removed component. A no-op before the cache is first populated.
///
/// Public (dev-loop-atomic): `deck dev`'s OWN HMR watcher — the one that
/// actually drives `bundle_app` rebuilds — must also call this on a
/// structural change (a new `src/<tag>/` component dir, a created/removed
/// `.sail`). The bundler-internal watcher only covers TARGET builds, so
/// without this hook a component dir added mid-session never registers its
/// tag until full restart.
pub fn invalidate_component_index() {
    if let Some(cache) = COMPONENT_INDEX_CACHE.get() {
        cache.lock().unwrap_or_else(|e| e.into_inner()).clear();
    }
}
