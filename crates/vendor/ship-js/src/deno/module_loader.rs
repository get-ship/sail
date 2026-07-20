//! Deny-by-default `ModuleLoader` — the closed world for imports.
//!
//! Three constructors:
//! - `new(sdk, main)` — deny-by-default. Only `ship` resolves.
//! - `with_extra_modules(sdk, main, list)` — simple-list constructor; the
//!   runtime adapts the `Vec<String>` to the [`ModuleAllowList`] trait.
//! - `with_module_allow_list(sdk, main, Arc<dyn ModuleAllowList>)` —
//!   trait-shaped constructor for arbitrary policy implementations.
//!
//! `ship` always resolves regardless of the allow-list. Specifiers passed
//! to the gate must be URL-formatted per `deno_core::ModuleSpecifier`
//! (e.g. `"ship:foo"`); bare names do not parse and the loader does not
//! URL-magic them.
//!
//! This is the import-side twin of the host's capability gate: the isolate
//! can import only what the host hands it. `new(..)` preserves the original
//! behavior, so existing callers do not need to change.

use std::sync::Arc;

use deno_core::error::ModuleLoaderError;
use deno_core::{
    ModuleLoadOptions, ModuleLoadReferrer, ModuleLoadResponse, ModuleLoader, ModuleSource,
    ModuleSourceCode, ModuleSpecifier, ModuleType, ResolutionKind,
};
use deno_error::JsErrorBox;

/// The host's main module specifier — the entry point.
pub const MAIN: &str = "ship:main";
/// The scoped-SDK module specifier — the one thing `import "ship"` resolves to.
pub const SDK: &str = "ship:sdk";

/// A closed-world module loader. Default deny-by-default permits only `ship`;
/// a caller-supplied `ModuleAllowList` extends the permitted set.
///
/// Three constructors:
/// - `new(sdk, main)` — deny-by-default; only `ship` resolves. Original
///   behavior, preserved for back-compat.
/// - `with_extra_modules(sdk, main, Arc<Vec<String>>)` — convenience that
///   adapts a list of specifiers to the [`ModuleAllowList`] trait. `ship`
///   still resolves.
/// - `with_module_allow_list(sdk, main, Arc<dyn ModuleAllowList>)` — the
///   trait-shaped constructor; any implementation of [`ModuleAllowList`]
///   plugs in. Caller controls the policy; the runtime adds nothing.
///
/// `ship` always resolves regardless of the allow-list — the closed world
/// is the floor, the allow-list extends the ceiling.
pub struct ShipModuleLoader {
    sdk_src: String,
    main_src: String,
    /// Per-call module gate. When `None`, only `ship` resolves; when `Some`,
    /// the gate is consulted for everything outside the closed-world floor.
    /// Per-call: each `run_module` constructs a fresh loader, so the gate is
    /// per-call, not per-isolate.
    allow_list: Option<Arc<dyn crate::dispatch::ModuleAllowList>>,
}

impl ShipModuleLoader {
    /// `sdk_src` is served for `import "ship"`; `main_src` is the entry
    /// module. Only `ship` resolves. Original behavior.
    pub fn new(sdk_src: String, main_src: String) -> Self {
        Self {
            sdk_src,
            main_src,
            allow_list: None,
        }
    }

    /// A loader that additionally resolves every specifier in `extra_modules`.
    /// `ship` still resolves. Convenience over
    /// [`Self::with_module_allow_list`] for callers that have a flat
    /// `Vec<String>`; the runtime adapts it through [`ListModuleAllowList`].
    /// Caller-controlled; the runtime adds nothing.
    pub fn with_extra_modules(
        sdk_src: String,
        main_src: String,
        extra_modules: Arc<Vec<String>>,
    ) -> Self {
        Self {
            sdk_src,
            main_src,
            allow_list: Some(Arc::new(ListModuleAllowList(extra_modules))),
        }
    }

    /// A loader that consults `allow_list` for specifiers outside the closed
    /// world. The trait-level seam: any policy implementation works (a
    /// generated allow-list per skill, a regex match, a server-derived set).
    /// `ship` still resolves unconditionally.
    pub fn with_module_allow_list(
        sdk_src: String,
        main_src: String,
        allow_list: Arc<dyn crate::dispatch::ModuleAllowList>,
    ) -> Self {
        Self {
            sdk_src,
            main_src,
            allow_list: Some(allow_list),
        }
    }
}

/// Adapter: `Arc<Vec<String>>` implements `ModuleAllowList` so the simple-list
/// constructor doesn't make every caller construct a trait object. Lives in
/// `module_loader` rather than `dispatch` because it is a concrete data
/// shape, not a policy — the policy stays at the trait seam.
pub struct ListModuleAllowList(pub Arc<Vec<String>>);
impl crate::dispatch::ModuleAllowList for ListModuleAllowList {
    fn permits(&self, specifier: &str) -> bool {
        self.0.iter().any(|s| s.as_str() == specifier)
    }
}

impl ModuleLoader for ShipModuleLoader {
    fn resolve(
        &self,
        specifier: &str,
        _referrer: &str,
        _kind: ResolutionKind,
    ) -> Result<ModuleSpecifier, ModuleLoaderError> {
        if specifier == "ship" {
            return ModuleSpecifier::parse(SDK).map_err(JsErrorBox::from_err);
        }
        if specifier == MAIN || specifier == SDK {
            return ModuleSpecifier::parse(specifier).map_err(JsErrorBox::from_err);
        }
        if let Some(gate) = &self.allow_list
            && gate.permits(specifier)
        {
            return ModuleSpecifier::parse(specifier).map_err(JsErrorBox::from_err);
        }
        Err(JsErrorBox::generic(format!(
            "import denied: `{specifier}` — only `ship` is importable in this runtime"
        )))
    }

    fn load(
        &self,
        module_specifier: &ModuleSpecifier,
        _maybe_referrer: Option<&ModuleLoadReferrer>,
        _options: ModuleLoadOptions,
    ) -> ModuleLoadResponse {
        let src = match module_specifier.as_str() {
            SDK => self.sdk_src.clone(),
            MAIN => self.main_src.clone(),
            other => {
                return ModuleLoadResponse::Sync(Err(JsErrorBox::generic(format!(
                    "load denied: `{other}`"
                ))));
            }
        };
        ModuleLoadResponse::Sync(Ok(ModuleSource::new(
            ModuleType::JavaScript,
            ModuleSourceCode::String(src.into()),
            module_specifier,
            None,
        )))
    }
}
