//! Generated ambient pack for sail TypeScript regions.
//!
//! Emit path: `<project>/node_modules/.sail/types.d.ts` via `sail-compiler
//! types` (or the deck `--typecheck` hook) — SPEC-OUTPUT-1 Option B. The pack
//! is **not** hand-authored in app source — every name comes from the same
//! tables the compiler / lint already own:
//!
//! - [`crate::runtime_ambient::RUNTIME_AMBIENT`] — controller vocabulary
//! - `spinnaker` vocab + plains emit — Style builder surface
//! - token registry values — closed token unions + bare-ident ambient consts
//!
//! ## Two-checker contract
//!
//! | Checker | Owns |
//! |---------|------|
//! | **tsc / tsgo** | Scope/controller/logic types; Style setter + token types when the source is valid TypeScript; ambient globals (`signal`, `controller`, `style`, …) |
//! | **sail-compiler** | Template/layout/vocabulary; Style DSL forms that are not valid TS (hyphenated bare tokens, `scale-y:` object keys); structural grammar |
//!
//! Style product text inside `style={…}` is a compile-time DSL. Full hyphenated
//! bare-token fidelity under tsc requires a virtual rewrite (string form);
//! the compiler still validates Style. This pack types the builder so wrong
//! setters and wrong token *types* fail under tsc on valid-TS spelling.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use sail_lint::scales;
use sail_lint::token_groups::{self, color_group_names, color_groups, size_step_names};
use spinnaker::component_surfaces;
use spinnaker::plains::{find_plain, PlainSpec};
use spinnaker::{find_concern, vocab_rows, VocabRow, VocabTag};

/// Relative path of the ambient pack inside a sail project (SPEC-OUTPUT-1 §4).
/// Lives under `node_modules/.sail/` so `rm -rf node_modules` clears it;
/// `.deck/` stays purely servable output; ops session stays under `.sail/`.
pub const PROJECT_TYPES_REL: &str = "node_modules/.sail/types.d.ts";

/// Relative path of the action observation ambient (SPEC-PLATFORM-NEUTRALITY-1 §1).
/// Pair with [`TypeEnv::Action`] + [`lib_for_type_env`] (ESNext only — no DOM lib).
pub const PROJECT_ACTION_OBSERVATION_REL: &str = "node_modules/.sail/action-observation.d.ts";

/// Script-form CSS side-effect ambient. tsgo TS2882 ignores `declare module '*.css'`
/// inside a module (the types pack has `import`/`export`); a sibling script `.d.ts`
/// is what makes `import '@ship/tokens/*.css'` legal under Pure.
pub const PROJECT_CSS_MODULES_REL: &str = "node_modules/.sail/css-modules.d.ts";

/// Regenerated header — agents/humans must not hand-edit the pack.
const HEADER: &str = "\
// GENERATED FILE — do not edit by hand.
//
// Sail ambient pack. Source of truth:
//   crates/sail/compiler/src/runtime_ambient.rs
//   crates/sail/spinnaker/src/vocab.rs (+ leftover plains / scales / token registry)
//   crates/sail/spinnaker/src/token_groups.rs (color groups / Tone + Size + member paths)
// Regenerate: cargo run -p sail-compiler -- types [--project <dir>]
//             (or: sail types --project <dir>)
//
// Two-checker contract:
//   tsc/tsgo  — controller/scope/logic types + Style builder types (valid TS)
//   sail-compiler — template/layout/vocabulary + Style DSL not expressible as TS
//
";

/// Marker setters that take no paint arg (`style.group()`).
const ZERO_ARG_SETTERS: &[&str] = &[
    "group",
    "scrollbarThin",
    "isolate",
    "truncate",
    "srOnly",
    "controlFrame",
    "controlLift",
    "press",
];

/// Keys of the `animate:` map (`animate: { enter: play(…), transition: play(…) }`).
const ANIMATE_MAP_KEYS: &[&str] = &[
    "transition",
    "enter",
    "exit",
    "loop",
    "reveal",
    "scroll",
    "hover",
    "press",
    "focus",
    "focus-visible",
];

/// Closed `when({ key })` product keys that are plain JS identifiers.
const WHEN_IDENT_KEYS: &[&str] = &[
    "hover",
    "focus",
    "focusWithin",
    "active",
    "disabled",
    "empty",
    "before",
    "after",
    "first",
    "last",
    "selected",
    "expanded",
    "checked",
    "pressed",
    "open",
    "current",
    "busy",
    "sortable",
    "stateLive",
    "stateEdit",
    "stateLocked",
    "groupCurrent",
    "groupRequired",
    "groupInvalid",
    "groupDisabled",
    "groupHover",
    "groupFocusWithin",
    "groupDismissible",
    "groupRemovable",
    "groupOpen",
    "groupStateHasDetails",
    "groupStateOpen",
    "groupCaught",
    "groupInline",
    "mobile",
    "tablet",
    "desktop",
    "dark",
    "light",
];

/// JS reserved words — never emit as ambient `const` names.
const JS_RESERVED: &[&str] = &[
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "function",
    "if",
    "import",
    "in",
    "instanceof",
    "new",
    "null",
    "return",
    "super",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "typeof",
    "var",
    "void",
    "while",
    "with",
    "yield",
    "let",
    "static",
    "implements",
    "interface",
    "package",
    "private",
    "protected",
    "public",
    "await",
    "arguments",
    "eval",
];

/// Emit the full ambient pack text.
pub fn emit_types_dts() -> String {
    let mut out = String::with_capacity(64 * 1024);
    out.push_str(HEADER);

    // ── Runtime type imports (module so we can import; then augment global) ──
    // Every tier-B public export from author-surface.json is either imported
    // here (types) or declared under `declare global` below (values). Drift
    // test: `pack_covers_author_surface_exports`.
    out.push_str(
        "import type {\n\
         \x20 Signal,\n\
         \x20 Derived,\n\
         \x20 ControllerDef,\n\
         \x20 ControllerScope,\n\
         \x20 ControllerSetup,\n\
         \x20 ControllerExport,\n\
         \x20 ReadonlyState,\n\
         \x20 ReadonlyView,\n\
         \x20 MutableState,\n\
         \x20 QueryOpts,\n\
         \x20 Mutation,\n\
         \x20 MutationOpts,\n\
         \x20 MutationState,\n\
         \x20 LiveState,\n\
         \x20 LiveSignal,\n\
         \x20 LiveSource,\n\
         \x20 LiveValueOf,\n\
         \x20 OptimisticHandle,\n\
         \x20 QueryKey,\n\
         \x20 StoreDef,\n\
         \x20 StoreInstance,\n\
         \x20 FormOptions,\n\
         \x20 ListSource,\n\
         \x20 Action,\n\
         \x20 ActionResult,\n\
         \x20 Disposer,\n\
         \x20 TimelineValue,\n\
         \x20 TimelineEasing,\n\
         \x20 TimelineEasingName,\n\
         \x20 TimelineStop,\n\
         \x20 TimelineStopOptions,\n\
         \x20 TimelineTrack,\n\
         \x20 TimelineTracks,\n\
         \x20 TimelineSignals,\n\
         \x20 TimelineOptions,\n\
         \x20 ClockOptions,\n\
         \x20 ClockSignal,\n\
         \x20 ScrollProgressOptions,\n\
         \x20 SpringOptions,\n\
         \x20 Theme,\n\
         \x20 ThemePatch,\n\
         \x20 ThemeTokenName,\n\
         \x20 ThemeState,\n\
         \x20 ThemePersist,\n\
         \x20 ThemeDoc,\n\
         \x20 ThemeApplyScopeOptions,\n\
         \x20 ThemeScopePatch,\n\
         \x20 LoweredThemeToken,\n\
         \x20 LowerThemeResult,\n\
         \x20 BoundaryHandle,\n\
         \x20 BoundarySource,\n\
         \x20 PropDef,\n\
         \x20 ComponentDef,\n\
         \x20 ComponentSetupContext,\n\
         \x20 ComponentFactory,\n\
         \x20 ComponentSchema,\n\
         \x20 EffectScope,\n\
         \x20 ContextKey,\n\
         \x20 PersistedOpts,\n\
         \x20 AnchorPlacement,\n\
         \x20 ClipboardParam,\n\
         \x20 HotkeyEntry,\n\
         \x20 HotkeyHandler,\n\
         \x20 HotkeysMap,\n\
         \x20 Host,\n\
         \x20 SailHost,\n\
         \x20 RouteSignals,\n\
         } from '@ship/sail-runtime'\n\
         \n",
    );

    let bare_tokens = emit_token_unions(&mut out);
    emit_style_types(&mut out);
    emit_globals(&mut out, &bare_tokens);
    out.push_str(&emit_css_side_effect_module());

    // Ensure this file is a module (global augmentation needs export {}).
    out.push_str("export {}\n");
    out
}

/// Write `<project>/node_modules/.sail/types.d.ts` and the action observation
/// ambient beside it. Creates parents as needed. Returns the types pack path
/// (primary ambient).
pub fn write_project_types(project: &Path) -> Result<PathBuf> {
    let path = project.join(PROJECT_TYPES_REL);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("mkdir {}", parent.display()))?;
    }
    let text = emit_types_dts();
    fs::write(&path, text).with_context(|| format!("write {}", path.display()))?;
    // Always materialize the observation pack so Action-kind typechecks can
    // reference it without a separate generate step (SPEC-PLATFORM-NEUTRALITY-1).
    let _ = write_project_action_observation(project)?;
    let _ = write_project_css_modules(project)?;
    Ok(path)
}

/// Write [`PROJECT_CSS_MODULES_REL`] — script-form `declare module '*.css'`.
pub fn write_project_css_modules(project: &Path) -> Result<PathBuf> {
    let path = project.join(PROJECT_CSS_MODULES_REL);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("mkdir {}", parent.display()))?;
    }
    fs::write(&path, emit_css_side_effect_module())
        .with_context(|| format!("write {}", path.display()))?;
    Ok(path)
}

/// Write [`PROJECT_ACTION_OBSERVATION_REL`] for the project. Creates parents.
pub fn write_project_action_observation(project: &Path) -> Result<PathBuf> {
    let path = project.join(PROJECT_ACTION_OBSERVATION_REL);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("mkdir {}", parent.display()))?;
    }
    let text = emit_action_observation_dts();
    fs::write(&path, text).with_context(|| format!("write {}", path.display()))?;
    Ok(path)
}

// ── Per-kind type environments (SPEC-PLATFORM-NEUTRALITY-1 §1 / S2.1) ───────
//
// Product branches the TypeScript lib/ambient by file kind so app
// controllers/compositions/stores cannot name document/window/localStorage.
// Action files get the observation port only. Framework packages keep DOM.
//
// Gate: crates/sail/compiler/tests/type_env_per_kind.rs

/// Per-kind TypeScript type environment (SPEC-PLATFORM-NEUTRALITY-1 §1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TypeEnv {
    /// Product files (routes, components, compositions, stores) — the sail
    /// typecheck: ESNext only, sail ambients, DOM globals unnameable.
    Sail,
    /// author action — observation set only (no `document` / `window`).
    Action,
    /// framework runtime packages — full DOM lib allowed.
    Framework,
}

/// Map a classified [`crate::project_map::FileKind`] to its type environment.
///
/// | Kind | Env |
/// |------|-----|
/// | Route / Component (controllers + scopes) | Sail |
/// | Composition / Store | Sail |
/// | Action | Action |
/// | Theme / Other | Sail (apps never gain free DOM via kind alone) |
///
/// Framework packages are selected via [`type_env_for_path`], not this table.
pub fn type_env_for_kind(kind: crate::project_map::FileKind) -> TypeEnv {
    use crate::project_map::FileKind;
    match kind {
        FileKind::Action => TypeEnv::Action,
        FileKind::Route
        | FileKind::Component
        | FileKind::Composition
        | FileKind::Store
        | FileKind::Theme
        | FileKind::Other => TypeEnv::Sail,
    }
}

/// Path-aware type-env selection.
///
/// 1. Framework packages (`packages/sail/**`) → [`TypeEnv::Framework`]
/// 2. Else [`crate::project_map::classify`] → [`type_env_for_kind`]
///
/// Framework wins over path-shaped Action (runtime catalog may live under
/// `…/actions/**` and is still allowed full DOM).
pub fn type_env_for_path(path: &Path, source: Option<&str>) -> TypeEnv {
    if is_framework_package_path(path) {
        return TypeEnv::Framework;
    }
    type_env_for_kind(crate::project_map::classify(path, source).kind)
}

/// True when `path` sits under a framework package that may use the full DOM lib.
///
/// Matches `packages/sail/**` (runtime, dom, tokens, …). App / component
/// packages do **not** get free DOM — they go through [`type_env_for_kind`].
fn is_framework_package_path(path: &Path) -> bool {
    use std::path::Component;
    let segs: Vec<_> = path
        .components()
        .filter_map(|c| match c {
            Component::Normal(s) => Some(s.to_string_lossy()),
            _ => None,
        })
        .collect();
    segs.windows(2)
        .any(|w| w[0] == "packages" && w[1] == "sail")
}

/// `compilerOptions.lib` entries for `env`.
///
/// - [`TypeEnv::Sail`] / [`TypeEnv::Action`]: ESNext only — no `DOM` / `DOM.Iterable`
/// - [`TypeEnv::Framework`]: `ESNext` + `DOM` + `DOM.Iterable`
///
/// Action observation names (rAF, observers, `getComputedStyle`) come from
/// [`emit_action_observation_dts`], not from the DOM lib.
pub fn lib_for_type_env(env: TypeEnv) -> &'static [&'static str] {
    match env {
        TypeEnv::Sail | TypeEnv::Action => &["ESNext"],
        TypeEnv::Framework => &["ESNext", "DOM", "DOM.Iterable"],
    }
}

/// Framework-owned platform values that stock tsc pulls from the DOM lib.
/// Pure/Action stay ESNext-only — these names are declared here, never via
/// `lib: ["DOM"]`. Isolated harness and the generated pack share
/// `emit_framework_platform`; this list is the pack-test name lock.
pub const FRAMEWORK_PLATFORM_VALUE_NAMES: &[&str] = &[
    "AbortController",
    "AbortSignal",
    "TextDecoder",
    "TextEncoder",
    "URL",
    "URLSearchParams",
    "atob",
    "btoa",
    "clearInterval",
    "clearTimeout",
    "console",
    "queueMicrotask",
    "setInterval",
    "setTimeout",
    "structuredClone",
];

/// Side-effect-only CSS module so `import '@ship/tokens/*.css'` typechecks.
pub fn emit_css_side_effect_module() -> String {
    "declare module '*.css' {}\n".to_string()
}

/// Typed framework platform names for the generated pack (`declare global`).
pub fn emit_framework_platform_global_body() -> String {
    emit_framework_platform(true)
}

/// Script-form platform + CSS ambient (isolated harness / Pure tsc fixtures).
pub fn emit_framework_platform_script_dts() -> String {
    let mut out = emit_css_side_effect_module();
    out.push_str(&emit_framework_platform(false));
    out
}

fn emit_framework_platform(in_global: bool) -> String {
    let indent = if in_global { "  " } else { "" };
    let decl = if in_global { "" } else { "declare " };
    let mut out = String::new();
    out.push_str(indent);
    out.push_str("// Framework-owned platform names (ESNext lib omits these).\n");
    // Interfaces first so value-side constructors can name them.
    out.push_str(&format!(
        "{indent}interface URLSearchParams {{\n\
         {indent}  append(name: string, value: string): void\n\
         {indent}  delete(name: string): void\n\
         {indent}  get(name: string): string | null\n\
         {indent}  getAll(name: string): string[]\n\
         {indent}  has(name: string): boolean\n\
         {indent}  set(name: string, value: string): void\n\
         {indent}  toString(): string\n\
         {indent}}}\n\
         {indent}interface URL {{\n\
         {indent}  hash: string\n\
         {indent}  host: string\n\
         {indent}  hostname: string\n\
         {indent}  href: string\n\
         {indent}  origin: string\n\
         {indent}  password: string\n\
         {indent}  pathname: string\n\
         {indent}  port: string\n\
         {indent}  protocol: string\n\
         {indent}  search: string\n\
         {indent}  searchParams: URLSearchParams\n\
         {indent}  username: string\n\
         {indent}  toJSON(): string\n\
         {indent}  toString(): string\n\
         {indent}}}\n\
         {indent}interface TextEncoder {{\n\
         {indent}  encode(input?: string): Uint8Array\n\
         {indent}}}\n\
         {indent}interface TextDecoder {{\n\
         {indent}  decode(input?: ArrayBuffer | ArrayBufferView | null): string\n\
         {indent}}}\n\
         {indent}interface AbortSignal {{\n\
         {indent}  readonly aborted: boolean\n\
         {indent}  readonly reason: unknown\n\
         {indent}}}\n\
         {indent}interface AbortController {{\n\
         {indent}  readonly signal: AbortSignal\n\
         {indent}  abort(reason?: unknown): void\n\
         {indent}}}\n\
         {indent}{decl}const URL: {{ new (url: string | URL, base?: string | URL): URL }}\n\
         {indent}{decl}const URLSearchParams: {{\n\
         {indent}  new (init?: string | string[][] | Record<string, string>): URLSearchParams\n\
         {indent}}}\n\
         {indent}{decl}const TextEncoder: {{ new (): TextEncoder }}\n\
         {indent}{decl}const TextDecoder: {{ new (label?: string): TextDecoder }}\n\
         {indent}{decl}const AbortController: {{ new (): AbortController }}\n\
         {indent}{decl}const AbortSignal: {{ abort(reason?: unknown): AbortSignal }}\n\
         {indent}{decl}const console: {{\n\
         {indent}  log(...data: unknown[]): void\n\
         {indent}  info(...data: unknown[]): void\n\
         {indent}  warn(...data: unknown[]): void\n\
         {indent}  error(...data: unknown[]): void\n\
         {indent}  debug(...data: unknown[]): void\n\
         {indent}}}\n\
         {indent}{decl}function setTimeout(handler: (...args: never[]) => unknown, ms?: number): number\n\
         {indent}{decl}function clearTimeout(id?: number): void\n\
         {indent}{decl}function setInterval(handler: (...args: never[]) => unknown, ms?: number): number\n\
         {indent}{decl}function clearInterval(id?: number): void\n\
         {indent}{decl}function queueMicrotask(callback: () => void): void\n\
         {indent}{decl}function structuredClone<T>(value: T): T\n\
         {indent}{decl}function atob(data: string): string\n\
         {indent}{decl}function btoa(data: string): string\n"
    ));
    out
}

/// Ambient `.d.ts` body for the action observation port.
///
/// Declares:
/// - `requestAnimationFrame` / `cancelAnimationFrame`
/// - `IntersectionObserver`, `ResizeObserver`, `MutationObserver`
/// - `getComputedStyle`
/// - a minimal `Element` handle for the action first argument
///
/// Does **not** introduce usable ambient `document`, `window`, or `localStorage`
/// (those stay unnameable under the Action lib).
pub fn emit_action_observation_dts() -> String {
    // Teaching comments may name banned globals; product code must not declare them.
    r#"// GENERATED FILE — do not edit by hand.
//
// Sail action observation ambient (SPEC-PLATFORM-NEUTRALITY-1 §1).
// Pair with compilerOptions.lib = ["ESNext"] only — never DOM.
//
// Actions receive a handed Element (`node`) and may observe it via rAF,
// ResizeObserver / IntersectionObserver / MutationObserver, and getComputedStyle.
// They must not name document / window / localStorage — those stay unnameable
// under the Action type environment (no ambient declaration here).
//
// Regenerate: cargo run -p sail-compiler -- types [--project <dir>]
//

/** Minimal host element handle for the action first argument `(node, param)`. */
interface Element {
  readonly nodeType: number
  readonly nodeName: string
  readonly tagName: string
}

/** Read-only geometry used by observer entries. */
interface DOMRectReadOnly {
  readonly x: number
  readonly y: number
  readonly width: number
  readonly height: number
  readonly top: number
  readonly right: number
  readonly bottom: number
  readonly left: number
}

interface CSSStyleDeclaration {
  getPropertyValue(property: string): string
}

declare function getComputedStyle(
  elt: Element,
  pseudoElt?: string | null,
): CSSStyleDeclaration

declare function requestAnimationFrame(
  callback: (time: number) => void,
): number
declare function cancelAnimationFrame(handle: number): void

// ── ResizeObserver ──────────────────────────────────────────────────────────

interface ResizeObserverOptions {
  box?: 'content-box' | 'border-box' | 'device-pixel-content-box'
}

interface ResizeObserverEntry {
  readonly target: Element
  readonly contentRect: DOMRectReadOnly
}

interface ResizeObserverCallback {
  (entries: ResizeObserverEntry[], observer: ResizeObserver): void
}

declare class ResizeObserver {
  constructor(callback: ResizeObserverCallback)
  observe(target: Element, options?: ResizeObserverOptions): void
  unobserve(target: Element): void
  disconnect(): void
}

// ── IntersectionObserver ────────────────────────────────────────────────────

interface IntersectionObserverInit {
  root?: Element | null
  rootMargin?: string
  threshold?: number | number[]
}

interface IntersectionObserverEntry {
  readonly target: Element
  readonly isIntersecting: boolean
  readonly intersectionRatio: number
  readonly boundingClientRect: DOMRectReadOnly
  readonly intersectionRect: DOMRectReadOnly
  readonly rootBounds: DOMRectReadOnly | null
  readonly time: number
}

interface IntersectionObserverCallback {
  (entries: IntersectionObserverEntry[], observer: IntersectionObserver): void
}

declare class IntersectionObserver {
  constructor(
    callback: IntersectionObserverCallback,
    options?: IntersectionObserverInit,
  )
  readonly root: Element | null
  readonly rootMargin: string
  readonly thresholds: ReadonlyArray<number>
  observe(target: Element): void
  unobserve(target: Element): void
  disconnect(): void
  takeRecords(): IntersectionObserverEntry[]
}

// ── MutationObserver ────────────────────────────────────────────────────────

type MutationRecordType = 'attributes' | 'characterData' | 'childList'

interface MutationRecord {
  readonly type: MutationRecordType
  readonly target: Element
  readonly addedNodes: ReadonlyArray<Element>
  readonly removedNodes: ReadonlyArray<Element>
  readonly attributeName: string | null
  readonly attributeNamespace: string | null
  readonly oldValue: string | null
}

interface MutationObserverInit {
  childList?: boolean
  attributes?: boolean
  characterData?: boolean
  subtree?: boolean
  attributeFilter?: string[]
  attributeOldValue?: boolean
  characterDataOldValue?: boolean
}

interface MutationCallback {
  (mutations: MutationRecord[], observer: MutationObserver): void
}

declare class MutationObserver {
  constructor(callback: MutationCallback)
  observe(target: Element, options?: MutationObserverInit): void
  disconnect(): void
  takeRecords(): MutationRecord[]
}
"#
    .to_string()
}

/// TOOLING-1 WI2 flags that every sail project tsconfig must carry.
///
/// Kept as a public list so scaffold smoke tests and package drift checks
/// share one SSOT with [`sail_tsconfig_compiler_options_json`].
pub const SAIL_TSCONFIG_REQUIRED_FLAGS: &[&str] = &[
    "\"strict\": true",
    "\"noUncheckedIndexedAccess\": true",
    "\"exactOptionalPropertyTypes\": true",
    "\"noImplicitOverride\": true",
    "\"verbatimModuleSyntax\": true",
    "\"erasableSyntaxOnly\": true",
    "\"isolatedModules\": true",
    "\"moduleResolution\": \"Bundler\"",
    "\"target\": \"ESNext\"",
    "\"skipLibCheck\": true",
];

/// Default sail project `compilerOptions` (Pure / product typecheck).
///
/// Product FileKinds (Route/Component/Composition/Store) are Sail — ESNext
/// only, no DOM. Framework packages that need DOM use
/// [`sail_tsconfig_compiler_options_json_for`]([`TypeEnv::Framework`]) or
/// `packages/sail/tsconfig/tsconfig.base.json` (TOOLING-1 WI2).
///
/// Per-kind (Pure / Action / Framework) options:
/// [`sail_tsconfig_compiler_options_json_for`]. Ambient pack registration is
/// **not** a `compilerOptions.types` package name: the generated file lives
/// at [`PROJECT_TYPES_REL`] and is wired via `files` in
/// [`emit_project_tsconfig_json`] so globals resolve under tsc.
pub fn sail_tsconfig_compiler_options_json() -> String {
    sail_tsconfig_compiler_options_json_for(TypeEnv::Sail)
}

/// `compilerOptions` JSON object body for a specific [`TypeEnv`].
///
/// Pure/Action get ESNext-only `lib` (document unnameable). Framework keeps
/// DOM. Shared TOOLING-1 WI2 flags are identical across envs.
pub fn sail_tsconfig_compiler_options_json_for(env: TypeEnv) -> String {
    let lib = lib_for_type_env(env);
    let lib_arr = lib
        .iter()
        .map(|e| format!("\"{e}\""))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        r#"{{
    "target": "ESNext",
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "lib": [{lib_arr}],
    "strict": true,
    "noUncheckedIndexedAccess": true,
    "exactOptionalPropertyTypes": true,
    "noImplicitOverride": true,
    "verbatimModuleSyntax": true,
    "erasableSyntaxOnly": true,
    "isolatedModules": true,
    "skipLibCheck": true,
    "noEmit": true,
    "allowImportingTsExtensions": true,
    "types": []
  }}"#
    )
}

/// Relative path of the scaffolded oxlint config inside a sail project.
pub const PROJECT_OXLINT_REL: &str = ".oxlintrc.json";

/// Default oxlint config for TS regions (TOOLING-1 WI2).
///
/// Recommended correctness + typescript plugin rule set. Sail-compiler owns
/// sail-specific template/layout/vocabulary rules — not reimplemented here.
/// Mirrors `packages/sail/tsconfig/oxlint.base.json`.
pub fn emit_project_oxlintrc_json() -> String {
    // No `$comment` — oxlint rejects unknown fields. Docs live in FEATURES.md
    // and package.json description (TOOLING-1 WI2).
    r#"{
  "$schema": "./node_modules/oxlint/configuration_schema.json",
  "plugins": ["eslint", "typescript", "oxc"],
  "categories": {
    "correctness": "error",
    "suspicious": "warn"
  },
  "env": {
    "browser": true
  },
  "ignorePatterns": [
    "**/node_modules/**",
    "**/dist/**",
    "**/.deck/**",
    "**/.sail/**"
  ]
}
"#
    .to_string()
}

/// Write `<project>/.oxlintrc.json` (scaffold / create path).
pub fn write_project_oxlintrc(project: &Path) -> Result<PathBuf> {
    let path = project.join(PROJECT_OXLINT_REL);
    fs::write(&path, emit_project_oxlintrc_json())
        .with_context(|| format!("write {}", path.display()))?;
    Ok(path)
}

/// Directory for generated type machinery (ambient pack + pure/action
/// tsconfigs). Lives under `node_modules/.sail/` so `rm -rf node_modules`
/// clears it; users never author these files (G2c).
pub const PROJECT_SAIL_DIR_REL: &str = "node_modules/.sail";

/// Relative path from [`PROJECT_SAIL_DIR_REL`] up to the project root.
/// Pure/action configs live under `.sail/`; include/exclude/`paths` use this
/// prefix so globs and aliases still cover the project tree.
const SAIL_DIR_TO_PROJECT_ROOT: &str = "../..";

/// Build a Pure-env child tsconfig body (written under [`PROJECT_SAIL_DIR_REL`]).
///
/// Default uses [`TypeEnv::Sail`] (ESNext only — no DOM) so product projects
/// enforce unnameable `document`/`window` (SPEC-PLATFORM-NEUTRALITY-1).
/// Framework packages that need the DOM lib use
/// [`emit_project_tsconfig_json_for`]([`TypeEnv::Framework`]) or path-aware
/// selection via [`type_env_for_path`].
///
/// `include_globs` are **project-root relative** (e.g. `src/**/*.ts`); emit
/// rewrites them for the config's nest under [`PROJECT_SAIL_DIR_REL`].
/// The ambient pack is listed under `files` as a same-dir path (`./types.d.ts`)
/// so `exclude: ../../node_modules` does not drop it (SPEC-OUTPUT-1 §4 Option B).
///
/// `compilerOptions.paths` come from [`crate::project_map::at_alias_paths`]
/// (SPEC-PROJECT-STRUCTURE-1 §3). Targets are rewritten relative to `.sail/`
/// (`../../src/…`) — **no `baseUrl`**: tsgo removes it (TS5102) and the gate
/// must use the same config as `tsc -p` / `tsgo -p`.
pub fn emit_project_tsconfig_json(include_globs: &[&str]) -> String {
    emit_project_tsconfig_json_for(TypeEnv::Sail, include_globs)
}

/// Pure/action child tsconfig body for a specific [`TypeEnv`].
///
/// Emitted under [`PROJECT_SAIL_DIR_REL`] (see path rewriting notes on
/// [`emit_project_tsconfig_json`]).
///
/// - `lib` from [`lib_for_type_env`]
/// - [`TypeEnv::Action`] also lists `./action-observation.d.ts` under `files`
///   so the observation set is nameable without the DOM lib
pub fn emit_project_tsconfig_json_for(env: TypeEnv, include_globs: &[&str]) -> String {
    // Globs are project-root relative; config lives under node_modules/.sail/.
    let root = SAIL_DIR_TO_PROJECT_ROOT;
    let includes: Vec<String> = include_globs
        .iter()
        .map(|g| format!("\"{root}/{g}\""))
        .collect();
    let opts = sail_tsconfig_compiler_options_json_for(env);
    // Drop outer braces so we can inject paths from the SSOT.
    let opts_inner = opts
        .trim()
        .trim_start_matches('{')
        .trim_end_matches('}')
        .trim();
    // Paths relative to this config dir (no baseUrl — tsgo TS5102).
    // SSOT targets are project-root relative (`./src/stores/*`); prefix `../../`.
    let path_entries: Vec<String> = crate::project_map::at_alias_paths()
        .iter()
        .map(|m| {
            let target = m.target.trim_start_matches("./");
            format!("      \"{}\": [\"{root}/{target}\"]", m.pattern)
        })
        .collect();
    let paths_block = path_entries.join(",\n");
    // Packs sit beside this config — same-dir paths, not project-root paths.
    // css-modules.d.ts is a script (no import/export) so tsgo honors TS2882.
    let mut files = vec![
        "\"./types.d.ts\"".to_string(),
        "\"./css-modules.d.ts\"".to_string(),
    ];
    if env == TypeEnv::Action {
        files.push("\"./action-observation.d.ts\"".to_string());
    }
    let env_note = match env {
        TypeEnv::Sail => {
            "TypeEnv::Sail — sail typecheck: ESNext only, sail ambients; document unnameable."
        }
        TypeEnv::Action => {
            "TypeEnv::Action — ESNext + action-observation.d.ts; no document/window."
        }
        TypeEnv::Framework => "TypeEnv::Framework — ESNext + DOM (framework packages).",
    };
    // Sail product config must not typecheck actions under the sail typecheck
    // (no observation pack). Action files live solely under tsconfig.action.json.
    let exclude = match env {
        TypeEnv::Sail => format!(
            "\"{root}/node_modules\", \"{root}/dist\", \"{root}/.deck\", \
             \"{root}/src/actions\", \"{root}/src/actions/**\""
        ),
        _ => format!("\"{root}/node_modules\", \"{root}/dist\", \"{root}/.deck\""),
    };
    format!(
        "{{\n  \
         \"$comment\": \"Sail kind tsconfig under {sail_dir}/ (G2c / TOOLING-1 WI2 / SPEC-PLATFORM-NEUTRALITY-1). GENERATED — hand edits are overwritten on build. {env_note} Lives beside types.d.ts; root tsconfig.json is a solution that references this file. No baseUrl (tsgo TS5102); paths are relative to this file.\",\n  \
         \"compilerOptions\": {{\n    {opts_inner},\n    \"incremental\": true,\n    \"tsBuildInfoFile\": \"./{tsbuild}\",\n    \"paths\": {{\n{paths}\n    }}\n  }},\n  \
         \"files\": [{files}],\n  \
         \"include\": [{inc}],\n  \
         \"exclude\": [{exclude}]\n\
         }}\n",
        sail_dir = PROJECT_SAIL_DIR_REL,
        paths = paths_block,
        files = files.join(", "),
        inc = includes.join(", "),
        tsbuild = match env {
            TypeEnv::Action => "tsconfig.action.tsbuildinfo",
            _ => "tsconfig.sail.tsbuildinfo",
        },
    )
}

/// Root project `tsconfig.json` — solution entry only (G2c).
///
/// IDEs open this file and resolve sail + action children under
/// [`PROJECT_SAIL_DIR_REL`] via `references`. No compilerOptions here; the
/// type gate never loads this file.
pub fn emit_project_solution_tsconfig_json() -> String {
    format!(
        "{{\n  \
         \"$comment\": \"Sail project solution tsconfig (G2c / TOOLING-1). GENERATED — hand edits are overwritten on build/dev. IDE entry only: references sail + action configs under {sail_dir}/. Users never edit this file. The type gate never reads it.\",\n  \
         \"files\": [],\n  \
         \"references\": [\n    \
         {{ \"path\": \"./{pure}\" }},\n    \
         {{ \"path\": \"./{action}\" }}\n  \
         ]\n\
         }}\n",
        sail_dir = PROJECT_SAIL_DIR_REL,
        pure = PROJECT_TSCONFIG_SAIL_REL,
        action = PROJECT_TSCONFIG_ACTION_REL,
    )
}

/// Relative path of the generated sail-typecheck project tsconfig (G2c — under `.sail/`).
pub const PROJECT_TSCONFIG_SAIL_REL: &str = "node_modules/.sail/tsconfig.sail.json";
/// Relative path of the generated Action-env project tsconfig (G2c — under `.sail/`).
pub const PROJECT_TSCONFIG_ACTION_REL: &str = "node_modules/.sail/tsconfig.action.json";

/// Pre-G2c root-level kind configs. Removed on regenerate when marked GENERATED.
const LEGACY_ROOT_TSCONFIG_PURE: &str = "tsconfig.pure.json";
const LEGACY_ROOT_TSCONFIG_ACTION: &str = "tsconfig.action.json";

/// Default include globs for the sail typecheck (controllers synthesized
/// at the gate, compositions, stores, and general `src/**/*.ts`).
/// Actions are **excluded** via `exclude` in [`emit_project_tsconfig_json_for`]
/// so IDE/`tsgo -p node_modules/.sail/tsconfig.sail.json` never typechecks
/// `src/actions/**` under the sail typecheck (no observation pack). Actions ride
/// [`PROJECT_TSCONFIG_ACTION_REL`].
pub const PROJECT_SAIL_INCLUDE: &[&str] = &[
    "src/**/*.ts",
    "routes/**/*.ts",
    "src/compositions/**/*.ts",
    "src/stores/**/*.ts",
];

/// Default include globs for Action typecheck.
pub const PROJECT_ACTION_INCLUDE: &[&str] = &["src/actions/**/*.ts"];

/// Whether a root `tsconfig.json` body is sail-owned (safe to overwrite).
fn is_sail_owned_tsconfig(body: &str) -> bool {
    body.contains("GENERATED")
        || body.contains("Sail project tsconfig")
        || body.contains("Sail project solution")
        || body.contains("Sail kind tsconfig")
        || body.contains("TOOLING-1")
}

/// Drop pre-G2c root-level pure/action mirrors when they were compiler-owned.
fn remove_legacy_root_kind_tsconfigs(project: &Path) {
    for name in [LEGACY_ROOT_TSCONFIG_PURE, LEGACY_ROOT_TSCONFIG_ACTION] {
        let path = project.join(name);
        match fs::read_to_string(&path) {
            Ok(body) if is_sail_owned_tsconfig(&body) => {
                let _ = fs::remove_file(&path);
            }
            _ => {}
        }
    }
}

/// Keep `node_modules/.sail/` free of scratch kind configs
/// (`tsconfig.pure.g2d.json`, agent debris, etc.). Canonical names only.
fn sweep_stale_sail_kind_tsconfigs(project: &Path) {
    let sail_dir = project.join(PROJECT_SAIL_DIR_REL);
    let Ok(entries) = fs::read_dir(&sail_dir) else {
        return;
    };
    const KEEP: &[&str] = &[
        "tsconfig.sail.json",
        "tsconfig.action.json",
        "types.d.ts",
        "action-observation.d.ts",
        "css-modules.d.ts",
    ];
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name_s) = name.to_str() else {
            continue;
        };
        // tsconfig*.json except the two canonical kind files.
        if name_s.starts_with("tsconfig") && name_s.ends_with(".json") && !KEEP.contains(&name_s) {
            let _ = fs::remove_file(entry.path());
        }
        // tsbuildinfo for removed scratch configs
        if name_s.starts_with("tsconfig")
            && name_s.ends_with(".tsbuildinfo")
            && name_s != "tsconfig.sail.tsbuildinfo"
            && name_s != "tsconfig.action.tsbuildinfo"
        {
            let _ = fs::remove_file(entry.path());
        }
    }
}

/// Write `contents` only when the file is missing or the bytes differ.
/// Same-bytes writes still bump mtime and will self-trigger `sail dev` HMR
/// when the project root is watched.
fn write_if_changed(path: &Path, contents: &str) -> Result<bool> {
    if let Ok(existing) = fs::read_to_string(path) {
        if existing == contents {
            return Ok(false);
        }
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("mkdir {}", parent.display()))?;
    }
    fs::write(path, contents).with_context(|| format!("write {}", path.display()))?;
    Ok(true)
}

/// Write ambient packs + pure/action kind tsconfigs under
/// [`PROJECT_SAIL_DIR_REL`], plus a solution-style root `tsconfig.json`.
///
/// **Regeneration is the law** (E1c/G2c): every build / `sail types` call
/// overwrites these files. Hand-edited `lib` entries (e.g. sneaking DOM back
/// in) are wiped. Root pure/action filenames die — only the solution entry
/// remains user-visible at the project root. Returns paths written: types pack,
/// pure tsconfig, action tsconfig.
pub fn write_project_type_configs(project: &Path) -> Result<(PathBuf, PathBuf, PathBuf)> {
    let types = write_project_types(project)?;
    let pure_path = project.join(PROJECT_TSCONFIG_SAIL_REL);
    let action_path = project.join(PROJECT_TSCONFIG_ACTION_REL);
    write_if_changed(
        &pure_path,
        &emit_project_tsconfig_json_for(TypeEnv::Sail, PROJECT_SAIL_INCLUDE),
    )?;
    write_if_changed(
        &action_path,
        &emit_project_tsconfig_json_for(TypeEnv::Action, PROJECT_ACTION_INCLUDE),
    )?;
    // Root tsconfig.json is a solution that references pure + action children.
    let default_path = project.join("tsconfig.json");
    // Only overwrite when the file carries our GENERATED marker or is absent —
    // never clobber a hand-authored root that is not sail-owned. Presence of
    // the marker (or the older TOOLING-1 header) means compiler owns it.
    let should_write_default = match fs::read_to_string(&default_path) {
        Ok(body) => is_sail_owned_tsconfig(&body),
        Err(_) => true,
    };
    if should_write_default {
        write_if_changed(&default_path, &emit_project_solution_tsconfig_json())?;
    }
    // Pre-G2c root kind files are dead; drop when sail-owned.
    remove_legacy_root_kind_tsconfigs(project);
    // Drop agent/scratch kind configs under `.sail/` (tsconfig.pure.g2d*.json).
    sweep_stale_sail_kind_tsconfigs(project);
    Ok((types, pure_path, action_path))
}

// ── token unions ───────────────────────────────────────────────────────────

/// Emit scale unions; return bare token strings eligible for global consts.
fn emit_token_unions(out: &mut String) -> BTreeSet<String> {
    out.push_str("// ── Closed token unions (from lint scales / token registry) ──\n");

    let scales = [
        "color",
        "space",
        "size",
        "radius",
        "text",
        "weight",
        "font",
        "leading",
        "tracking",
        "shadow",
        "z",
        "duration",
        "ease",
        "percent",
        "angle",
        "border-width",
        "opacity",
    ];
    let mut all_bare: BTreeSet<String> = BTreeSet::new();

    for scale in scales {
        let name = scale_type_name(scale);
        let mut vals = scales::values(scale);
        // SPEC-D1: component surface public names are not app Style paint vocabulary.
        if scale == "color" {
            vals.retain(|v| !component_surfaces::is_component_public_name(v));
        }
        out.push_str(&format!("type {name} =\n"));
        if vals.is_empty() {
            out.push_str("  | never\n");
        } else {
            for v in &vals {
                out.push_str("  | ");
                out.push_str(&ts_literal(v));
                out.push('\n');
                all_bare.insert(v.clone());
            }
            for kw in extra_keywords_for_scale(scale) {
                out.push_str("  | ");
                out.push_str(&ts_literal(kw));
                out.push('\n');
                all_bare.insert(kw.to_string());
            }
        }
        out.push('\n');
    }

    // Bind placeholder — Style `bind(name)` is compile-time; type-level brand.
    out.push_str(
        "/** Compile-time Style bind probe (`bind(rowY)`). */\n\
         declare const __sailBindBrand: unique symbol\n\
         type SailBind = { readonly [__sailBindBrand]: true }\n\
         \n\
         /** Accept a token, a bind probe, or a numeric space/size step. */\n\
         type TokenOrBind<T> = T | SailBind\n\
         \n",
    );

    emit_token_group_types(out);

    all_bare
        .into_iter()
        .filter(|t| is_ambient_const_name(t))
        .collect()
}

/// Emit color-group (Tone) + Size + static/bound group member path unions.
///
/// Same graph as validate/emit ([`color_groups`]) so authors complete
/// `fg(tone.foreground)` / `bg(primary)` / `'primary.foreground'`.
/// `Tone` is the product type name for the color-group set — not a special
/// language feature for the prop name `tone`. Size is a discrete ladder type
/// for Style match (not continuous paint bind).
fn emit_token_group_types(out: &mut String) {
    out.push_str("// ── Token groups: color (Tone) + Size (token_groups SSOT) ──\n");
    out.push_str(
        "/** Color-group allowlist (product type name: Tone). Type-keyed paint bind:\n\
         \x20*  any prop typed as Tone binds (`bg(tone)`, `bg(shade)` with `shade: Tone`).\n\
         \x20*  No `'default'` — rest paint = omit the attribute (component pack tokens). */\n         type Tone =\n",
    );
    for name in color_group_names() {
        out.push_str(&format!("  | '{name}'\n"));
    }
    out.push('\n');

    out.push_str(
        "/** Discrete control size ladder (Style `match size` conditionality).\n\
         \x20*  Hosts may narrow or extend (`xs`/`xl`/`full`); not continuous paint bind. */\n         type Size =\n",
    );
    for name in size_step_names() {
        out.push_str(&format!("  | '{name}'\n"));
    }
    out.push('\n');

    // Static group.member paths (non-default members only — default is the bare stem).
    out.push_str(
        "/** Static token-group member paths — product: `fg(primary.foreground)`.\n\
         \x20*  Resolves to public leaf tokens at compile (same as validate). */\n         type SailStaticGroupMemberPath =\n",
    );
    let mut any_static = false;
    for g in color_groups() {
        for m in g.members {
            if m.name == "default" {
                continue;
            }
            any_static = true;
            out.push_str(&format!("  | '{}.{}'\n", g.name, m.name));
        }
    }
    if !any_static {
        out.push_str("  | never\n");
    }
    out.push('\n');

    // Bound prop paths — bridge-var realization (host reflects var references).
    // Product ambient lists the common prop name `tone`; any Tone-typed prop binds.
    out.push_str(
        "/** Bound color-group prop paths — product ambient name `tone` is the common example:\n\
         \x20*  `bg(tone)`, `fg(tone.foreground)`. Type-keyed: `shade: Tone` also binds.\n\
         \x20*  Emits bridge vars; host sets `--tone: var(--primary)` from the prop. */\n         type SailBoundGroupPath =\n         \x20 | 'tone'\n",
    );
    // Members that exist on *every* group (bound_member_valid) — default is bare `tone`.
    let common_members = common_bound_member_names();
    for m in &common_members {
        if *m == "default" {
            continue;
        }
        out.push_str(&format!("  | 'tone.{m}'\n"));
    }
    out.push('\n');
    // Historic name used by generated docs that still name SailBoundTonePath.
    out.push_str("type SailBoundTonePath = SailBoundGroupPath\n\n");

    out.push_str(
        "/** Closed Style paint color args: leaf tokens + group paths + bound group props.\n\
         \x20*  Used by bg/fg/border/… so completion offers Tone members, not free text. */\n         type SailColorPaintArg =\n         \x20 | SailColorToken\n         \x20 | SailStaticGroupMemberPath\n         \x20 | SailBoundGroupPath\n         \n",
    );

    // Document color-group table shape for hover/docs (mirrors packages/sail/tokens unions).
    out.push_str("/** Color group → members → public token name (informational; paint uses paths above). */\n");
    out.push_str("type SailToneTable = {\n");
    for g in color_groups() {
        out.push_str(&format!("  {}: {{\n", g.name));
        for m in g.members {
            out.push_str(&format!("    {}: '{}'\n", m.name, m.public_name));
        }
        out.push_str("  }\n");
    }
    out.push_str("}\n\n");
}

/// Member names present on every color group (excl. gaps like hover).
fn common_bound_member_names() -> Vec<&'static str> {
    let groups = color_groups();
    let Some(first) = groups.first() else {
        return Vec::new();
    };
    first
        .members
        .iter()
        .map(|m| m.name)
        .filter(|name| {
            groups
                .iter()
                .all(|g| g.members.iter().any(|m| m.name == *name))
        })
        .collect()
}

/// Nested ambient type for a static group stem (`primary` → `.foreground`, `.hover`).
fn static_group_member_object_type(group_name: &str) -> Option<String> {
    let group = token_groups::static_token_group(group_name)?;
    let mut fields = Vec::new();
    for m in group.members {
        if m.name == "default" {
            continue;
        }
        fields.push(format!("readonly {}: '{}.{}'", m.name, group.name, m.name));
    }
    if fields.is_empty() {
        return None;
    }
    Some(format!("{{ {} }}", fields.join("; ")))
}

/// Ambient type expression for a bare color/group token (leaf ∩ optional members).
fn bare_token_type_expr(tok: &str) -> String {
    let lit = ts_literal(tok);
    if let Some(obj) = static_group_member_object_type(tok) {
        format!("{lit} & {obj}")
    } else {
        lit
    }
}

fn scale_type_name(scale: &str) -> String {
    let mut parts = scale.split('-');
    let mut name = String::from("Sail");
    for p in parts.by_ref() {
        let mut c = p.chars();
        if let Some(first) = c.next() {
            name.push(first.to_ascii_uppercase());
            name.push_str(c.as_str());
        }
    }
    name.push_str("Token");
    name
}

fn extra_keywords_for_scale(scale: &str) -> &'static [&'static str] {
    // Keywords that appear on setters for this scale but may not be in the scale table.
    match scale {
        "color" => &["transparent", "current", "currentColor", "inherit", "none"],
        _ => &[],
    }
}

fn ts_literal(v: &str) -> String {
    if v.is_empty() {
        return "''".into();
    }
    // Pure integer → number literal (Style `padX(2)`).
    if v.chars().all(|c| c.is_ascii_digit()) {
        return v.to_string();
    }
    // Quote everything else (incl. 1_5, muted-foreground, full).
    let escaped = v.replace('\\', "\\\\").replace('\'', "\\'");
    format!("'{escaped}'")
}

fn is_ambient_const_name(tok: &str) -> bool {
    if tok.is_empty() || tok.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    let mut chars = tok.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_' || first == '$') {
        return false;
    }
    if !chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$') {
        return false;
    }
    if JS_RESERVED.contains(&tok) {
        return false;
    }
    true
}

// ── Style builder ──────────────────────────────────────────────────────────

fn emit_style_types(out: &mut String) {
    out.push_str("// ── Style builder (from spinnaker vocab) ──\n");

    out.push_str(
        "/** `animate:` keyed map. The concern inside is `play`. */\n\
         type SailAnimateMap = {\n\
         \x20 transition?: SailStyleValue\n\
         \x20 enter?: SailStyleValue\n\
         \x20 exit?: SailStyleValue\n\
         \x20 reveal?: SailStyleValue\n\
         \x20 loop?: SailStyleValue\n\
         \x20 scroll?: SailStyleValue\n\
         \x20 hover?: SailStyleValue\n\
         \x20 press?: SailStyleValue\n\
         \x20 focus?: SailStyleValue\n\
         \x20 'focus-visible'?: SailStyleValue\n\
         }\n\
         \n",
    );

    // when key type — known idents + string escape hatch for dynamic families
    out.push_str("type SailWhenKey =\n");
    for k in WHEN_IDENT_KEYS {
        out.push_str(&format!("  | '{k}'\n"));
    }
    out.push_str("  | (string & {})\n\n");

    out.push_str(
        "/** Style object partial — keyed members only (SPEC-SPINNAKER-3). */\n\
         type SailStyleObj = {\n\
         \x20 animate?: SailAnimateMap\n\
         \x20 when?: SailWhenMap\n\
         \x20 match?: SailMatchBlock\n\
         \x20 compound?: SailCompoundRow[]\n\
         }\n\
         \n\
         type SailWhenMap = { [K in SailWhenKey]?: SailStyleObj | SailStyleValue }\n\
         type SailMatchArms = { [arm: string]: SailStyleObj | SailStyleValue }\n\
         type SailMatchBlock = { [prop: string]: SailMatchArms }\n\
         type SailCompoundRow = SailStyleObj & Record<string, unknown>\n\
         \n\
         /** Opaque compile-time Style value (object / ref). */\n\
         type SailStyleValue = SailStyleChain\n\
         \n",
    );

    out.push_str("interface SailStyleChain {\n");
    for row in vocab_rows() {
        emit_vocab_method(out, row);
    }
    out.push_str("}\n\n");

    // Callable style head — variadic partials (SPEC-STYLE-OBJECT-1).
    out.push_str(
        "/** Compile-time Style head — product: `style(partial, partial, …)`. */\n\
         interface SailStyleBuilder extends SailStyleChain {\n\
         \x20 (...partials: Array<SailStyleObj | SailStyleValue>): SailStyleChain\n\
         }\n\
         \n",
    );
}

fn emit_vocab_method(out: &mut String, row: &VocabRow) {
    if let Some(spec) = find_plain(row.name) {
        emit_setter_method(out, spec);
        return;
    }
    if row.name == "play" {
        out.push_str(
            "  play(preset: TokenOrBind<string>, duration?: TokenOrBind<string>): SailStyleChain\n",
        );
        return;
    }
    if matches!(row.tag, VocabTag::Concern | VocabTag::Axis) {
        out.push_str(&format!(
            "  {}(arg?: TokenOrBind<string | number> | Record<string, unknown>): SailStyleChain\n",
            row.name
        ));
        return;
    }
    out.push_str(&format!(
        "  {}(token?: TokenOrBind<string | number>): SailStyleChain\n",
        row.name
    ));
}

fn emit_setter_method(out: &mut String, spec: &PlainSpec) {
    let name = spec.name;
    if ZERO_ARG_SETTERS.contains(&name) {
        out.push_str(&format!("  {name}(): SailStyleChain\n"));
        // press also has motion form — handled in motion loop
        return;
    }

    // colorMix / colorDeepen: color paint arg + closed percent ramp
    if matches!(name, "colorMix" | "colorDeepen") {
        out.push_str(&format!(
            "  {name}(token: TokenOrBind<SailColorPaintArg>, pct: TokenOrBind<SailPercentToken | number>): SailStyleChain\n"
        ));
        return;
    }

    let arg = arg_type_for_setter(spec);
    // border composites accept 1 or 2 args
    if matches!(
        name,
        "border"
            | "borderX"
            | "borderY"
            | "borderT"
            | "borderB"
            | "borderL"
            | "borderR"
            | "borderS"
            | "borderE"
            | "outline"
            | "ring"
    ) {
        out.push_str(&format!("  {name}(a: {arg}, b?: {arg}): SailStyleChain\n"));
        return;
    }
    out.push_str(&format!("  {name}(token: {arg}): SailStyleChain\n"));
}

fn arg_type_for_setter(spec: &PlainSpec) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(scale) = spec.scale {
        // Color paint setters accept leaf tokens + group.member + bound tone paths.
        if scale == "color" {
            parts.push("SailColorPaintArg".into());
        } else {
            parts.push(scale_type_name(scale));
        }
    }
    for kw in spec.keywords {
        // Avoid duplicating scale members that are also keywords.
        parts.push(ts_literal(kw));
    }
    if parts.is_empty() {
        // Fall back to string | number | bind for unscaled setters.
        return "TokenOrBind<string | number>".into();
    }
    // Dedupe while preserving order
    let mut seen = BTreeSet::new();
    let mut uniq = Vec::new();
    for p in parts {
        if seen.insert(p.clone()) {
            uniq.push(p);
        }
    }
    format!("TokenOrBind<{}>", uniq.join(" | "))
}

// ── global ambient names ───────────────────────────────────────────────────

fn emit_globals(out: &mut String, bare_tokens: &BTreeSet<String>) {
    out.push_str("// ── Ambient globals (controller / scope / template regions) ──\n");
    out.push_str("declare global {\n");

    // SPEC-AMBIENT-SURFACE-1 §1: public types are free globals (no `import type`
    // from @ship/* in app code). Module-level `import type` above feeds
    // signature positions; these aliases re-export them into the global scope.
    out.push_str(
        "  // Public types — free global names (declare global re-exports).\n\
         \x20 type Signal<T> = import('@ship/sail-runtime').Signal<T>\n\
         \x20 type Derived<T> = import('@ship/sail-runtime').Derived<T>\n\
         \x20 type StoreDef<Shape> = import('@ship/sail-runtime').StoreDef<Shape>\n\
         \x20 type Theme = import('@ship/sail-runtime').Theme\n\
         \x20 type LiveSignal<T> = import('@ship/sail-runtime').LiveSignal<T>\n\
         \x20 type LiveState<T> = import('@ship/sail-runtime').LiveState<T>\n\
         \x20 type Mutation<T> = import('@ship/sail-runtime').Mutation<T>\n\
         \x20 type MutationState<T> = import('@ship/sail-runtime').MutationState<T>\n\
         \x20 type QueryKey = import('@ship/sail-runtime').QueryKey\n\
         \x20 type ControllerScope<S> = import('@ship/sail-runtime').ControllerScope<S>\n\
         \x20 /** Opaque mount host — Pure-safe; pass to actions/runtime (JOB-E2 SailHost). */\n\
         \x20 type SailHost = import('@ship/sail-runtime').SailHost\n\
         \x20 type HostPropertyDesc = import('@ship/sail-runtime').HostPropertyDesc\n\
         \x20 type Host = import('@ship/sail-runtime').Host\n",
    );

    // Setter / motion / when-key names that also appear as token idents need a
    // callable∩literal hybrid — cannot emit both `const border` and `function border`.
    let setter_names: BTreeSet<&str> = vocab_rows().iter().map(|r| r.name).collect();
    let motion_free: BTreeSet<&str> = ANIMATE_MAP_KEYS
        .iter()
        .copied()
        .filter(|m| *m != "focus-visible" && *m != "transition")
        .collect();
    let when_key_set: BTreeSet<&str> = WHEN_IDENT_KEYS.iter().copied().collect();
    let hybrid_names: BTreeSet<&str> = setter_names
        .iter()
        .copied()
        .chain(motion_free.iter().copied())
        .chain(when_key_set.iter().copied())
        .collect();

    // Bare Style-token identifiers so `bg(primary)` typechecks across files.
    // Hyphenated tokens use the string form under pure tsc (or virtual_ts
    // quotes them for sail check). Skip names that get hybrid emit below.
    out.push_str(
        "  // Style token idents (valid JS names only).\n\
         \x20 // Hyphenated: bg('muted-foreground') under pure tsc;\n\
         \x20 // sail-compiler + virtual_ts accept the Style-DSL bare form.\n",
    );
    for tok in bare_tokens {
        if hybrid_names.contains(tok.as_str()) {
            continue;
        }
        out.push_str(&format!(
            "  const {tok}: {ty}\n",
            ty = bare_token_type_expr(tok)
        ));
    }

    // Bound tone prop root — not a leaf color token; nested `.foreground` for paint.
    // Product: `bg(tone)`, `fg(tone.foreground)`.
    if !bare_tokens.contains("tone") && !hybrid_names.contains("tone") {
        out.push_str(
            "  /** Bound Tone prop — omit attribute for rest; paint via group paths. */\n             \x20 const tone: 'tone' & { readonly foreground: 'tone.foreground' }\n",
        );
    }
    out.push('\n');

    // Runtime ambient vocabulary — typed via @ship/sail-runtime imports.
    // Names that already have full signatures in the author-surface block
    // below (query/mutation/stores/theme/interval/debounce, …) skip the
    // generic fallback here so the pack never double-declares.
    const TYPED_IN_AUTHOR_SURFACE: &[&str] = &[
        "debounce",
        "defineStore",
        "interval",
        "mutation",
        "provideStore",
        "query",
        "theme",
        "useStore",
    ];
    for (authored, _exported, doc) in crate::runtime_ambient::RUNTIME_AMBIENT {
        if *authored == "__invalidate" {
            // Compiler-only; still declare so rewritten output typechecks.
            out.push_str("  /** compiler-only lowering target for invalidate(ref) */\n");
            out.push_str("  function __invalidate(prefix: QueryKey): void\n");
            continue;
        }
        if TYPED_IN_AUTHOR_SURFACE.contains(authored) {
            // Declared with product signatures in the blocks below.
            continue;
        }
        out.push_str(&format!("  /** {doc} */\n"));
        match *authored {
            "signal" => {
                out.push_str("  function signal<T>(initial: T): Signal<T>\n");
            }
            "derived" => {
                out.push_str("  function derived<T>(fn: () => T): Derived<T>\n");
            }
            "effect" => {
                out.push_str("  function effect(fn: () => void | (() => void)): () => void\n");
            }
            "batch" => {
                out.push_str("  function batch(fn: () => void): void\n");
            }
            "untrack" => {
                out.push_str("  function untrack<T>(fn: () => T): T\n");
            }
            "mount" => {
                out.push_str("  function mount(fn: (el: Element) => void | (() => void)): void\n");
            }
            "onCleanup" => {
                out.push_str("  function onCleanup(fn: () => void): void\n");
            }
            "controller" => {
                // Match runtime `controller<Ctx, SC>`: Ctx from the setup
                // parameter annotation, SC from the return — so authors can
                // write `controller((ctx: ControllerScope<Scope>) => …)`.
                out.push_str(
                    "  function controller<\n\
                     \x20\x20  Ctx = object,\n\
                     \x20\x20  SC extends Record<string, unknown> = Record<string, unknown>,\n\
                     \x20\x20>(\n\
                     \x20\x20  setup: (ctx: ControllerScope<Ctx>, host?: Element) => SC,\n\
                     \x20\x20): ControllerDef<SC>\n",
                );
            }
            other => {
                // Future ambient rows — declare as unknown callable until typed.
                out.push_str(&format!("  const {other}: (...args: never[]) => unknown\n"));
            }
        }
    }

    // query / mutation — author surface (import or ambient for typecheck).
    // LiveSignal / OptimisticHandle are imported as types above so TS-1
    // ambient pack pins the freeze-locked optimistic set/rollback shape (WI3).
    out.push_str(
        "  /** Public read primitive — compiler derives the cache key. */\n\
         \x20 function query<T>(\n\
         \x20\x20  fn: (signal: AbortSignal) => Promise<T> | LiveSource<T>,\n\
         \x20\x20  opts?: QueryOpts,\n\
         \x20\x20): LiveSignal<T>\n\
         \x20 /** Public write primitive. */\n\
         \x20 function mutation<T>(\n\
         \x20\x20  fn: (input: never) => Promise<T>,\n\
         \x20\x20  opts?: MutationOpts,\n\
         \x20\x20): Mutation<T>\n\
         \x20 /** Explicit-key query (tests / uncompiled). */\n\
         \x20 function queryKeyed<T>(\n\
         \x20\x20  keyFn: () => QueryKey,\n\
         \x20\x20  fn: (signal: AbortSignal) => Promise<T> | LiveSource<T>,\n\
         \x20\x20  opts?: QueryOpts,\n\
         \x20\x20): LiveSignal<T>\n\
         \x20 function invalidate(ref: unknown): void\n\
         \x20 function unwrap<T>(live: LiveSignal<T>): T | undefined\n\
         \x20 function unwrapOr<T>(live: LiveSignal<T>, fallback: T): T\n\
         \x20 /** Freeze-locked optimistic write handle (type-only until impl lands). */\n\
         \x20 type OptimisticWriteHandle<T> = OptimisticHandle<T>\n\
         \n",
    );

    // Stores, timeline, compositions, theme, components — public author
    // surface (TS-1 WI2.5). Import-or-ambient for typecheck; runtime still
    // requires explicit import except RUNTIME_AMBIENT inject names.
    out.push_str(
        "  // ── Public author runtime (tier B; author-surface.json) ──\n\
         \x20 function defineStore<Shape>(id: string, setup: () => Shape): StoreDef<Shape>\n\
         \x20 function useStore<Shape>(def: StoreDef<Shape>): Shape\n\
         \x20 function provideStore<Shape>(def: StoreDef<Shape>): Shape\n\
         \x20 function peekStore<Shape>(def: StoreDef<Shape>): Shape | null\n\
         \x20 function snapshotStore(inst: object): Record<string, unknown>\n\
         \x20 function form<T extends Record<string, unknown>>(opts: FormOptions<T>): unknown\n\
         \x20 function selection<T>(source: () => T[], keyOf?: (item: T) => string): unknown\n\
         \x20 function filteredList<T>(source: () => T[], matches?: (item: T, q: string) => boolean): unknown\n\
         \x20 function timeline<const T extends TimelineTracks>(\n\
         \x20\x20  progress: () => number,\n\
         \x20\x20  tracks: T,\n\
         \x20\x20  options?: TimelineOptions,\n\
         \x20\x20): TimelineSignals<T>\n\
         \x20 function clock(durationMs: number, options?: ClockOptions): ClockSignal\n\
         \x20 function scrollProgress(el: Element, options?: ScrollProgressOptions): Signal<number>\n\
         \x20 function springValue(target: () => number, options?: SpringOptions): Signal<number>\n\
         \x20 function debounce<Args extends unknown[]>(fn: (...args: Args) => void, ms: number): (...args: Args) => void\n\
         \x20 function interval(fn: () => void, ms: number): void\n\
         \x20 function persisted<T>(key: string, initial: T, opts?: PersistedOpts): Signal<T>\n\
         \x20 function isSignal(v: unknown): v is Signal<unknown>\n\
         \x20 function provideContext<T>(key: ContextKey<T>, value: T): void\n\
         \x20 function injectContext<T>(key: ContextKey<T>): T | null\n\
         \x20 function contextKey<T>(description: string): ContextKey<T>\n\
         \x20 /** Opaque mount host coerce — Pure-safe (JOB-E2 SailHost). */\n\
         \x20 function asSailHost(node: unknown): SailHost\n\
         \x20 /** Next animation frame (or microtask) — Pure must not name rAF. */\n\
         \x20 function scheduleFrame(fn: () => void): void\n\
         \x20 const theme: Theme\n\
         \x20 function setThemePersist(persist: ThemePersist): void\n\
         \x20 function hydrateTheme(): void\n\
         \x20 function clearThemeOverrides(): void\n\
         \x20 function clearThemeScope(scope?: string): void\n\
         \x20 function lowerThemeDoc(doc: ThemeDoc): LowerThemeResult\n\
         \x20 function attachBoundary(el: Element, handle: BoundaryHandle): void\n\
         \x20 function detachBoundary(el: Element): void\n\
         \x20 function findNearestBoundary(el: Element): BoundaryHandle | null\n\
         \x20 function runProtected<T>(node: Element, source: BoundarySource, fn: () => T): T\n\
         \x20 // Virtualizer (RUNTIME_AMBIENT inject; types may stay type-only from /virtual)\n\
         \x20 function createVirtualizer(options: {\n\
         \x20\x20  count: () => number\n\
         \x20\x20  estimateSize: (index: number) => number\n\
         \x20\x20  getScrollElement: () => Element | null\n\
         \x20\x20  horizontal?: boolean\n\
         \x20\x20  overscan?: number\n\
         \x20\x20  paddingStart?: number\n\
         \x20\x20  paddingEnd?: number\n\
         \x20\x20  gap?: number\n\
         \x20\x20  scrollMargin?: number\n\
         \x20\x20  getItemKey?: (index: number) => string | number\n\
         \x20\x20  lanes?: number\n\
         \x20\x20  onChange?: (v: unknown) => void\n\
         \x20 }): unknown\n\
         \x20 function component(def: ComponentDef): ComponentFactory\n\
         \x20 function prop<T>(def: PropDef<T>): PropDef<T>\n\
         \x20 function defineComponent(\n\
         \x20\x20  tag: string,\n\
         \x20\x20  schema: ComponentSchema,\n\
         \x20\x20  factory: ComponentFactory,\n\
         \x20\x20): void\n\
         \x20 type ActionFn<P = unknown> = Action<P>\n\
         \x20 type ActionResultType<P = unknown> = ActionResult<P>\n\
         \x20 type DisposerType = Disposer\n\
         \x20 const SailTimelineError: { new (message: string): Error }\n\
         \x20 const SailMountOutsideSetupError: { new (message?: string): Error }\n\
         \x20 // Public use: actions (import-or-ambient typecheck; use: still via compiler).\n\
         \x20 const anchor: Action<unknown>\n\
         \x20 const autosize: Action<void>\n\
         \x20 const clipboard: Action<ClipboardParam | string | undefined>\n\
         \x20 const clickOutside: Action<(() => void) | undefined>\n\
         \x20 const contextMenu: Action<unknown>\n\
         \x20 const dropzone: Action<unknown>\n\
         \x20 const focus: Action<boolean | null | undefined | void>\n\
         \x20 const hotkeys: Action<HotkeysMap | HotkeyEntry[] | undefined>\n\
         \x20 const icon: Action<string | undefined>\n\
         \x20 const inView: Action<InViewHandler | undefined>\n\
         \x20 type InViewHandler = (entry: IntersectionObserverEntry) => void\n\
         \x20 function primeIconCache(map: Record<string, string>): void\n\
         \x20 const rect: Action<string | undefined>\n\
         \x20 const tipDescribe: Action<string | undefined>\n\
         \x20 const virtualRow: Action<unknown>\n\
         \x20 const zoomPan: Action<unknown>\n\
         \x20 // mountAmbient is the package export; free name `mount` is ambient above.\n\
         \x20 function mountAmbient(fn: (el: Element) => void | (() => void)): void\n\
         \n",
    );

    // Setter keyword idents used as bare args: `cursor(pointer)`, `overflow(hidden)`.
    // Not always present on a token scale — still need ambient consts for tsc.
    // `style` / `bind` are the builder + bind heads — CSS `contain(style)` uses
    // the string form (`contain('style')`), never a colliding const.
    const RESERVED_HEADS: &[&str] = &["style", "bind"];
    out.push_str("  // Style setter keyword idents (bare args).\n");
    let mut kw_seen: BTreeSet<&str> = BTreeSet::new();
    for row in vocab_rows() {
        if let Some(spec) = find_plain(row.name) {
            for kw in spec.keywords {
                if !is_ambient_const_name(kw) {
                    continue;
                }
                if RESERVED_HEADS.contains(kw) {
                    continue;
                }
                if bare_tokens.contains(*kw) || hybrid_names.contains(kw) || !kw_seen.insert(kw) {
                    continue;
                }
                out.push_str(&format!("  const {kw}: '{kw}'\n"));
            }
        }
        if let Some(concern) = find_concern(row.name) {
            for slot in concern.slots {
                for kw in slot.keywords {
                    if !is_ambient_const_name(kw) {
                        continue;
                    }
                    if RESERVED_HEADS.contains(kw) {
                        continue;
                    }
                    if bare_tokens.contains(*kw) || hybrid_names.contains(kw) || !kw_seen.insert(kw)
                    {
                        continue;
                    }
                    out.push_str(&format!("  const {kw}: '{kw}'\n"));
                }
            }
        }
    }
    out.push('\n');

    // Style head (SPEC-STYLE-OBJECT-1 — free match/when/compound deleted).
    out.push_str(
        "  /** Compile-time Style builder — erased to classes by sail-compiler. */\n\
         \x20 const style: SailStyleBuilder\n\
         \x20 function bind(name: string): SailBind\n\
         \n",
    );

    // Free setters for flat concern/plain chains: `pad(4).bg(primary)`.
    out.push_str("  // Free Style setters (concern / axis / plain).\n");
    for row in vocab_rows() {
        if let Some(spec) = find_plain(row.name) {
            emit_free_setter(out, spec, bare_tokens);
        } else {
            emit_free_vocab(out, row, bare_tokens);
        }
    }

    // `play` is the only motion concern; trigger names are map keys, not calls.

    // Hybrids: token∩setter, when-key.
    out.push_str("  // Hybrid token / when-key / dual paint-motion free forms.\n");
    for tok in bare_tokens {
        if !setter_names.contains(tok.as_str()) {
            continue;
        }
        if let Some(spec) = find_plain(tok.as_str()) {
            emit_hybrid_token_setter(out, spec, tok);
            continue;
        }
        if let Some(row) = vocab_rows().iter().find(|r| r.name == tok.as_str()) {
            emit_hybrid_vocab(out, row, tok);
        }
    }
    for k in WHEN_IDENT_KEYS {
        let is_motion = motion_free.contains(k);
        let is_token = bare_tokens.contains(*k);
        let is_setter = setter_names.contains(k);
        if !is_motion && !is_token && !is_setter {
            // Pure when-key ambient const.
            out.push_str(&format!("  const {k}: '{k}'\n"));
            continue;
        }
        if is_setter {
            // Already hybridized with token/setter above when it's a token; pure setter when-keys rare.
            if !is_token {
                if let Some(spec) = find_plain(k) {
                    emit_hybrid_when_setter(out, spec, k, is_motion);
                } else if vocab_rows().iter().any(|r| r.name == *k) {
                    emit_hybrid_when_vocab(out, k, is_motion);
                }
            }
            continue;
        }
        if is_motion {
            // Trigger names are `animate:` map keys, not callables.
            out.push_str(&format!("  const {k}: '{k}'\n"));
            continue;
        }
        if is_token {
            out.push_str(&format!("  const {k}: '{k}'\n"));
        }
    }
    // press is paint-only as a free setter (`press()`). Hover/press motion is
    // `animate: { hover: play(…) }` / `when: { hover: … }` + `transition`.
    if setter_names.contains("press") && !bare_tokens.contains("press") {
        out.push_str("  function press(): SailStyleChain\n");
    }

    out.push_str(&emit_framework_platform_global_body());
    out.push_str("}\n\n");
}

fn emit_free_vocab(out: &mut String, row: &VocabRow, bare_tokens: &BTreeSet<String>) {
    let name = row.name;
    if bare_tokens.contains(name) || WHEN_IDENT_KEYS.contains(&name) || name == "press" {
        return;
    }
    if name.contains('-') || JS_RESERVED.contains(&name) {
        return;
    }
    if row.name == "play" {
        out.push_str(
            "  function play(preset: TokenOrBind<string>, duration?: TokenOrBind<string>): SailStyleChain\n",
        );
        return;
    }
    if matches!(row.tag, VocabTag::Concern | VocabTag::Axis) {
        out.push_str(&format!(
            "  function {name}(arg?: TokenOrBind<string | number> | Record<string, unknown>): SailStyleChain\n"
        ));
        return;
    }
    out.push_str(&format!(
        "  function {name}(token?: TokenOrBind<string | number>): SailStyleChain\n"
    ));
}

fn emit_hybrid_vocab(out: &mut String, row: &VocabRow, tok: &str) {
    let tok_ty = bare_token_type_expr(tok);
    if row.name.contains('-') {
        out.push_str(&format!("  const {tok}: {tok_ty}\n"));
        return;
    }
    if matches!(row.tag, VocabTag::Concern | VocabTag::Axis) {
        out.push_str(&format!(
            "  const {}: {{\n\
             \x20\x20 (arg?: TokenOrBind<string | number> | Record<string, unknown>): SailStyleChain\n\
             \x20 }} & {tok_ty}\n",
            row.name
        ));
        return;
    }
    out.push_str(&format!(
        "  const {}: {{\n\
         \x20\x20 (token?: TokenOrBind<string | number>): SailStyleChain\n\
         \x20 }} & {tok_ty}\n",
        row.name
    ));
}

fn emit_hybrid_when_vocab(out: &mut String, key: &str, _is_motion: bool) {
    out.push_str(&format!(
        "  const {key}: {{\n\
         \x20\x20 (arg?: TokenOrBind<string | number> | Record<string, unknown>): SailStyleChain\n\
         \x20 }} & '{key}'\n"
    ));
}

fn emit_free_setter(out: &mut String, spec: &PlainSpec, bare_tokens: &BTreeSet<String>) {
    let name = spec.name;
    // Hybrids / duals handled separately.
    if bare_tokens.contains(name) {
        return;
    }
    if WHEN_IDENT_KEYS.contains(&name) {
        return;
    }
    if name == "press" {
        return; // dual paint/motion hybrid
    }
    if name.contains('-') {
        return; // not a free TS ident
    }
    // JS reserved words cannot be free function names. Emit a renamable
    // `__sailX` form; virtual_ts rewrites product `break(` → `__sailBreak(`.
    if JS_RESERVED.contains(&name) {
        let arg = arg_type_for_setter(spec);
        let alias = free_reserved_alias(name);
        if ZERO_ARG_SETTERS.contains(&name) {
            out.push_str(&format!("  function {alias}(): SailStyleChain\n"));
        } else {
            out.push_str(&format!(
                "  function {alias}(token: {arg}): SailStyleChain\n"
            ));
        }
        return;
    }
    if ZERO_ARG_SETTERS.contains(&name) {
        out.push_str(&format!("  function {name}(): SailStyleChain\n"));
        return;
    }
    if matches!(name, "colorMix" | "colorDeepen") {
        out.push_str(&format!(
            "  function {name}(token: TokenOrBind<SailColorPaintArg>, pct: TokenOrBind<SailPercentToken | number>): SailStyleChain\n"
        ));
        return;
    }
    let arg = arg_type_for_setter(spec);
    if matches!(
        name,
        "border"
            | "borderX"
            | "borderY"
            | "borderT"
            | "borderB"
            | "borderL"
            | "borderR"
            | "borderS"
            | "borderE"
            | "outline"
            | "ring"
    ) {
        out.push_str(&format!(
            "  function {name}(a: {arg}, b?: {arg}): SailStyleChain\n"
        ));
        return;
    }
    out.push_str(&format!(
        "  function {name}(token: {arg}): SailStyleChain\n"
    ));
}

/// Ambient free-name for a Style setter that is a JS reserved word.
fn free_reserved_alias(name: &str) -> String {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return format!("__sail_{name}");
    };
    format!("__sail{}{}", first.to_ascii_uppercase(), chars.as_str())
}

fn emit_hybrid_token_setter(out: &mut String, spec: &PlainSpec, tok: &str) {
    let name = spec.name;
    let tok_ty = bare_token_type_expr(tok);
    if name.contains('-') {
        out.push_str(&format!("  const {tok}: {tok_ty}\n"));
        return;
    }
    if ZERO_ARG_SETTERS.contains(&name) {
        out.push_str(&format!(
            "  const {name}: {{\n\
             \x20\x20 (): SailStyleChain\n\
             \x20 }} & {tok_ty}\n"
        ));
        return;
    }
    if matches!(name, "colorMix" | "colorDeepen") {
        out.push_str(&format!(
            "  const {name}: {{\n\
             \x20\x20 (token: TokenOrBind<SailColorPaintArg>, pct: TokenOrBind<SailPercentToken | number>): SailStyleChain\n\
             \x20 }} & {tok_ty}\n"
        ));
        return;
    }
    let arg = arg_type_for_setter(spec);
    if matches!(
        name,
        "border"
            | "borderX"
            | "borderY"
            | "borderT"
            | "borderB"
            | "borderL"
            | "borderR"
            | "borderS"
            | "borderE"
            | "outline"
            | "ring"
    ) {
        out.push_str(&format!(
            "  const {name}: {{\n\
             \x20\x20 (a: {arg}, b?: {arg}): SailStyleChain\n\
             \x20 }} & {tok_ty}\n"
        ));
        return;
    }
    out.push_str(&format!(
        "  const {name}: {{\n\
         \x20\x20 (token: {arg}): SailStyleChain\n\
         \x20 }} & {tok_ty}\n"
    ));
}

fn emit_hybrid_when_setter(out: &mut String, spec: &PlainSpec, key: &str, _is_motion: bool) {
    let arg = arg_type_for_setter(spec);
    let motion = "";
    if ZERO_ARG_SETTERS.contains(&spec.name) {
        out.push_str(&format!(
            "  const {key}: {{\n\
             \x20\x20 (): SailStyleChain\n\
             {motion}\
             \x20 }} & '{key}'\n"
        ));
        return;
    }
    out.push_str(&format!(
        "  const {key}: {{\n\
         \x20\x20 (token: {arg}): SailStyleChain\n\
         {motion}\
         \x20 }} & '{key}'\n"
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_contains_every_runtime_ambient() {
        let dts = emit_types_dts();
        for (authored, _, _) in crate::runtime_ambient::RUNTIME_AMBIENT {
            assert!(
                dts.contains(authored),
                "ambient pack missing runtime name `{authored}`"
            );
        }
    }

    #[test]
    fn pack_contains_every_style_setter() {
        let dts = emit_types_dts();
        for row in vocab_rows() {
            let needle = if row.name.contains('-') {
                format!("'{}'", row.name)
            } else {
                format!("{}(", row.name)
            };
            let alt = format!("{}()", row.name);
            assert!(
                dts.contains(&needle) || dts.contains(&alt) || dts.contains(row.name),
                "ambient pack missing Style name `{}`",
                row.name
            );
        }
    }

    #[test]
    fn pack_exports_live_signal_and_optimistic_handle_types() {
        // SPEC-HARDENING-1 WI3 — optimistic LiveSignal set/update shape is
        // visible in the generated ambient pack (TS-1).
        let dts = emit_types_dts();
        assert!(
            dts.contains("LiveSignal"),
            "ambient pack must import/use LiveSignal"
        );
        assert!(
            dts.contains("OptimisticHandle"),
            "ambient pack must import OptimisticHandle (freeze lock)"
        );
        assert!(
            dts.contains("type OptimisticWriteHandle"),
            "ambient pack must surface OptimisticWriteHandle alias"
        );
    }

    #[test]
    fn pack_covers_author_surface_exports() {
        // TS-1 WI2.5: every remaining public export (author-surface.json)
        // appears in the ambient pack — type import or global declaration.
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let path = root.join("packages/sail/runtime/author-surface.json");
        let raw = fs::read_to_string(&path).expect("author-surface.json readable");
        let v: serde_json::Value = serde_json::from_str(&raw).expect("author-surface.json is JSON");
        let exports = v
            .get("exports")
            .and_then(|e| e.as_object())
            .expect("exports object");
        assert!(
            exports.len() >= 50,
            "expected >=50 author-surface exports, got {}",
            exports.len()
        );
        let dts = emit_types_dts();
        let mut missing = Vec::new();
        for name in exports.keys() {
            if !dts.contains(name.as_str()) {
                missing.push(name.clone());
            }
        }
        assert!(
            missing.is_empty(),
            "ambient pack missing author-surface names:\n  {}",
            missing.join("\n  ")
        );
        // Freeze list called out in SPEC-TS-1 WI2.5.
        for must in [
            "defineStore",
            "useStore",
            "provideStore",
            "timeline",
            "Action",
            "query",
            "mutation",
            "signal",
            "controller",
        ] {
            assert!(dts.contains(must), "freeze public name missing: {must}");
        }
    }

    #[test]
    fn pack_types_style_head_and_free_forms() {
        let dts = emit_types_dts();
        assert!(dts.contains("const style: SailStyleBuilder"));
        assert!(
            !dts.contains("const style: 'style'"),
            "CSS contain(style) must not shadow the Style builder"
        );
        assert!(
            dts.contains("interface SailStyleBuilder extends SailStyleChain"),
            "style.foo must be the chain methods, not {{}}"
        );
        assert!(
            dts.contains("type SailStyleObj") && dts.contains("animate?: SailStyleValue"),
            "SailStyleObj missing"
        );
        assert!(
            !dts.contains("box?: SailStyleValue")
                && !dts.contains("paint?: SailStyleValue")
                && !dts.contains("text?: SailStyleValue"),
            "SailStyleObj must not teach dissolved channel keys"
        );
        // Free concern/plain setters
        assert!(
            dts.contains("function pad(") || dts.contains("function padX("),
            "{dts}"
        );
        assert!(
            dts.contains("function bg(") || dts.contains("const bg:"),
            "free bg"
        );
        assert!(dts.contains("function enter("), "free enter motion");
        assert!(
            dts.contains("const hover:"),
            "when-key / motion hybrid hover"
        );
        assert!(!dts.contains("function match("), "free match() deleted");
        assert!(
            !dts.contains("function compound("),
            "free compound() deleted"
        );
        assert!(
            dts.contains("match?: SailMatchBlock") || dts.contains("type SailMatchBlock"),
            "match: key type"
        );
        assert!(dts.contains("function bind("));
        assert!(dts.contains("function query<"));
        assert!(dts.contains("function mutation<"));
    }

    #[test]
    fn pack_is_module_with_generated_header() {
        let dts = emit_types_dts();
        assert!(dts.starts_with("// GENERATED FILE"));
        assert!(dts.contains("export {}"));
        assert!(dts.contains("declare global"));
    }

    #[test]
    fn scale_type_names_are_stable() {
        assert_eq!(scale_type_name("color"), "SailColorToken");
        assert_eq!(scale_type_name("border-width"), "SailBorderWidthToken");
    }

    #[test]
    fn pack_documents_tone_and_group_member_paint_paths() {
        // Ambient paint args closed over color-group (Tone) graph + Size ladder.
        let dts = emit_types_dts();
        assert!(
            dts.contains("type Tone ="),
            "ambient must declare Tone (color-group) allowlist:\n{dts}"
        );
        assert!(
            dts.contains("type Size ="),
            "ambient must declare Size ladder:\n{dts}"
        );
        assert!(
            dts.contains("| 'sm'") && dts.contains("| 'md'") && dts.contains("| 'lg'"),
            "Size must list sm/md/lg:\n{dts}"
        );
        assert!(
            !dts.contains("| 'default'\n") || dts.contains("type Tone ="),
            "sanity"
        );
        // No fake default Tone member in the Tone union block.
        let tone_idx = dts.find("type Tone =").expect("Tone");
        let tone_end = dts[tone_idx..]
            .find("\ntype ")
            .map(|i| tone_idx + i)
            .unwrap_or(dts.len());
        let tone_block = &dts[tone_idx..tone_end];
        assert!(
            !tone_block.contains("'default'"),
            "Tone must not include 'default':\n{tone_block}"
        );
        for name in color_group_names() {
            assert!(
                tone_block.contains(&format!("'{name}'")),
                "Tone missing {name}:\n{tone_block}"
            );
        }
        assert!(
            dts.contains("tone.foreground") && dts.contains("primary.foreground"),
            "ambient must include bound + static foreground member forms"
        );
        assert!(
            dts.contains("type SailColorPaintArg =")
                && dts.contains("SailBoundGroupPath")
                && dts.contains("type SailBoundTonePath = SailBoundGroupPath"),
            "paint arg / bound-group path unions missing"
        );
        assert!(
            dts.contains("SailColorPaintArg"),
            "bg/fg must reference SailColorPaintArg"
        );
        assert!(
            dts.contains("const tone:") && dts.contains("foreground: 'tone.foreground'"),
            "nested tone.foreground ambient const missing"
        );
        assert!(
            dts.contains("foreground: 'primary.foreground'"),
            "nested primary.foreground ambient missing"
        );
    }

    #[test]
    fn ambient_const_name_rules() {
        assert!(is_ambient_const_name("primary"));
        assert!(is_ambient_const_name("full"));
        assert!(!is_ambient_const_name("muted-foreground"));
        assert!(!is_ambient_const_name("2"));
        assert!(!is_ambient_const_name("1_5"));
        assert!(!is_ambient_const_name("default"));
    }

    #[test]
    fn pack_declares_css_side_effect_module() {
        let dts = emit_types_dts();
        assert!(
            dts.contains("declare module '*.css'"),
            "generated pack must declare side-effect CSS modules:\n{}",
            dts.lines().take(20).collect::<Vec<_>>().join("\n")
        );
    }

    #[test]
    fn pack_declares_framework_platform_names() {
        let dts = emit_types_dts();
        for name in [
            "setTimeout",
            "clearTimeout",
            "setInterval",
            "clearInterval",
            "queueMicrotask",
            "structuredClone",
            "atob",
            "btoa",
        ] {
            assert!(
                dts.contains(&format!("function {name}")),
                "generated pack must declare typed function `{name}`"
            );
        }
        for name in [
            "URL",
            "URLSearchParams",
            "TextEncoder",
            "TextDecoder",
            "AbortController",
            "AbortSignal",
            "console",
        ] {
            assert!(
                dts.contains(&format!("const {name}:")),
                "generated pack must declare value `{name}` as `const {name}:`"
            );
        }
        for name in FRAMEWORK_PLATFORM_VALUE_NAMES {
            assert!(
                dts.contains(name),
                "FRAMEWORK_PLATFORM_VALUE_NAMES entry `{name}` missing from pack"
            );
        }
    }

    #[test]
    fn css_side_effect_module_is_script_form() {
        let dts = emit_css_side_effect_module();
        assert!(
            dts.contains("declare module '*.css'"),
            "CSS ambient must declare the wildcard:\n{dts}"
        );
        assert!(
            !dts.contains("import") && !dts.contains("export"),
            "css-modules sibling must stay a script (no import/export) or tsgo TS2882 returns:\n{dts}"
        );
        assert!(
            !dts.contains("CSSStyleSheet") && !dts.contains("className"),
            "side-effect CSS must not grow a value export:\n{dts}"
        );
    }

    #[test]
    fn project_tsconfig_carries_tooling_wi2_flags_and_ambient_include() {
        let opts = sail_tsconfig_compiler_options_json();
        for flag in SAIL_TSCONFIG_REQUIRED_FLAGS {
            assert!(
                opts.contains(flag),
                "sail_tsconfig_compiler_options_json missing {flag}:\n{opts}"
            );
        }
        // Default product emit is Pure — lib comes from lib_for_type_env SSOT.
        for entry in lib_for_type_env(TypeEnv::Sail) {
            assert!(
                opts.contains(&format!("\"{entry}\"")),
                "default opts must include Pure lib entry {entry}:\n{opts}"
            );
        }
        assert!(
            !opts.contains("\"DOM\""),
            "default product opts must not include DOM:\n{opts}"
        );
        let body = emit_project_tsconfig_json(&["src/**/*.ts", "routes/**/*.ts"]);
        for flag in SAIL_TSCONFIG_REQUIRED_FLAGS {
            assert!(
                body.contains(flag),
                "emit_project_tsconfig_json missing {flag}:\n{body}"
            );
        }
        assert!(
            !body.contains("DOM"),
            "default emit_project_tsconfig_json must be Pure (no DOM):\n{body}"
        );
        assert!(
            body.contains("\"files\"")
                && (body.contains("\"./types.d.ts\"") || body.contains("types.d.ts")),
            "kind tsconfig must list ambient pack under files (same-dir under .sail/):\n{body}"
        );
        assert!(
            body.contains("css-modules.d.ts"),
            "kind tsconfig must list script-form CSS ambient (tsgo TS2882):\n{body}"
        );
        assert!(
            body.contains("src/**/*.ts") && body.contains("routes/**/*.ts"),
            "project tsconfig must include authored globs:\n{body}"
        );
        // Pure default must exclude actions so IDE pure check matches compiler env.
        assert!(
            body.contains("src/actions"),
            "Pure project tsconfig must exclude src/actions:\n{body}"
        );
        // Nested under node_modules/.sail/ — no baseUrl (tsgo TS5102); paths
        // reach project root via ../../ prefix.
        assert!(
            !body.contains("\"baseUrl\""),
            "kind tsconfig must not emit baseUrl (tsgo removed it):\n{body}"
        );
        assert!(
            body.contains(&format!("\"{SAIL_DIR_TO_PROJECT_ROOT}/src/stores/*\""))
                || body.contains(&format!("{SAIL_DIR_TO_PROJECT_ROOT}/src/stores/*")),
            "kind paths must be relative to .sail/ (../../src/…):\n{body}"
        );
    }

    #[test]
    fn solution_tsconfig_references_pure_and_action_under_sail_dir() {
        let body = emit_project_solution_tsconfig_json();
        assert!(
            body.contains("GENERATED") && body.contains("references"),
            "solution must be GENERATED with references:\n{body}"
        );
        assert!(
            body.contains(PROJECT_TSCONFIG_SAIL_REL) && body.contains(PROJECT_TSCONFIG_ACTION_REL),
            "solution must reference pure + action under .sail/:\n{body}"
        );
        assert!(
            !body.contains("\"compilerOptions\""),
            "solution must not carry compilerOptions (children own env):\n{body}"
        );
        assert!(
            PROJECT_TSCONFIG_SAIL_REL.starts_with(PROJECT_SAIL_DIR_REL)
                && PROJECT_TSCONFIG_ACTION_REL.starts_with(PROJECT_SAIL_DIR_REL),
            "kind configs must live under {PROJECT_SAIL_DIR_REL}"
        );
    }

    #[test]
    fn write_project_type_configs_emits_sail_dir_kinds_and_solution_root() {
        let dir = tempfile::tempdir().unwrap();
        let project = dir.path();
        // Scratch debris under .sail/ must be swept on regenerate.
        let sail_dir = project.join(PROJECT_SAIL_DIR_REL);
        fs::create_dir_all(&sail_dir).unwrap();
        fs::write(
            sail_dir.join("tsconfig.pure.g2d.json"),
            r#"{ "$comment": "scratch" }"#,
        )
        .unwrap();
        fs::write(
            sail_dir.join("tsconfig.pure.g2d2.json"),
            r#"{ "$comment": "scratch" }"#,
        )
        .unwrap();
        // Pre-G2c root kind files — must be removed when sail-owned.
        fs::write(
            project.join(LEGACY_ROOT_TSCONFIG_PURE),
            r#"{ "$comment": "GENERATED pre-G2c pure" }"#,
        )
        .unwrap();
        fs::write(
            project.join(LEGACY_ROOT_TSCONFIG_ACTION),
            r#"{ "$comment": "GENERATED pre-G2c action" }"#,
        )
        .unwrap();

        let (types, pure, action) = write_project_type_configs(project).unwrap();
        assert!(types.ends_with(std::path::Path::new(PROJECT_TYPES_REL)) || types.exists());
        assert!(pure.exists() && pure.ends_with("tsconfig.sail.json"));
        assert!(action.exists() && action.ends_with("tsconfig.action.json"));
        let pure_s = pure.to_string_lossy();
        assert!(
            pure_s.contains("node_modules") && pure_s.contains(".sail"),
            "pure must live under .sail/: {}",
            pure.display()
        );

        let solution = fs::read_to_string(project.join("tsconfig.json")).unwrap();
        assert!(
            solution.contains("references") && solution.contains(PROJECT_TSCONFIG_SAIL_REL),
            "root must be solution:\n{solution}"
        );
        assert!(
            !project.join(LEGACY_ROOT_TSCONFIG_PURE).exists()
                && !project.join(LEGACY_ROOT_TSCONFIG_ACTION).exists(),
            "pre-G2c root kind tsconfigs must be deleted"
        );
        assert!(
            !sail_dir.join("tsconfig.pure.g2d.json").exists()
                && !sail_dir.join("tsconfig.pure.g2d2.json").exists(),
            "scratch kind configs under .sail/ must be swept"
        );
        assert!(
            pure.exists() && action.exists(),
            "canonical pure/action must remain after sweep"
        );

        let css = project.join(PROJECT_CSS_MODULES_REL);
        assert!(
            css.is_file(),
            "write_project_type_configs must materialize {}",
            PROJECT_CSS_MODULES_REL
        );
        let css_body = fs::read_to_string(&css).unwrap();
        assert!(
            css_body.contains("declare module '*.css'")
                && !css_body.contains("import")
                && !css_body.contains("export"),
            "written css-modules.d.ts must be script-form:\n{css_body}"
        );

        // Hand-authored root is preserved.
        let hand = dir.path().join("hand");
        fs::create_dir_all(&hand).unwrap();
        fs::write(
            hand.join("tsconfig.json"),
            r#"{ "compilerOptions": { "strict": true } }"#,
        )
        .unwrap();
        let _ = write_project_type_configs(&hand).unwrap();
        let kept = fs::read_to_string(hand.join("tsconfig.json")).unwrap();
        assert!(
            kept.contains("\"strict\": true") && !kept.contains("references"),
            "hand-authored root must not be clobbered:\n{kept}"
        );
        assert!(hand.join(PROJECT_TSCONFIG_SAIL_REL).exists());
    }

    #[test]
    fn generated_tsconfig_rewrite_does_not_bump_mtime() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write_project_type_configs(root).unwrap();
        let ts = root.join("tsconfig.json");
        let mtime = fs::metadata(&ts).unwrap().modified().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(20));
        write_project_type_configs(root).unwrap();
        let again = fs::metadata(&ts).unwrap().modified().unwrap();
        assert_eq!(
            mtime, again,
            "same-bytes tsconfig.json rewrite must not bump mtime (HMR loop)"
        );
    }

    #[test]
    fn pure_and_action_tsconfig_emit_esnext_only_lib() {
        for env in [TypeEnv::Sail, TypeEnv::Action] {
            let opts = sail_tsconfig_compiler_options_json_for(env);
            assert!(
                opts.contains("\"ESNext\""),
                "{env:?} opts need ESNext:\n{opts}"
            );
            assert!(
                !opts.contains("\"DOM\""),
                "{env:?} opts must not include DOM:\n{opts}"
            );
        }
        let action_body = emit_project_tsconfig_json_for(TypeEnv::Action, &["src/actions/**/*.ts"]);
        assert!(
            action_body.contains("action-observation.d.ts"),
            "Action tsconfig must list observation ambient:\n{action_body}"
        );
        assert!(
            action_body.contains("css-modules.d.ts"),
            "Action tsconfig must list CSS script ambient:\n{action_body}"
        );
        let pure_body =
            emit_project_tsconfig_json_for(TypeEnv::Sail, &["src/compositions/**/*.ts"]);
        assert!(
            !pure_body.contains("action-observation.d.ts"),
            "Pure tsconfig must not list action observation pack:\n{pure_body}"
        );
        assert!(
            pure_body.contains("css-modules.d.ts"),
            "Pure tsconfig must list CSS script ambient:\n{pure_body}"
        );
    }

    #[test]
    fn type_env_selection_and_observation_pack_smoke() {
        use crate::project_map::FileKind;
        assert_eq!(type_env_for_kind(FileKind::Composition), TypeEnv::Sail);
        assert_eq!(type_env_for_kind(FileKind::Action), TypeEnv::Action);
        assert_eq!(
            type_env_for_path(Path::new("packages/sail/runtime/src/index.ts"), None),
            TypeEnv::Framework
        );
        let dts = emit_action_observation_dts();
        assert!(dts.contains("requestAnimationFrame"));
        assert!(dts.contains("ResizeObserver"));
        assert!(!dts
            .lines()
            .filter(|l| !l.trim_start().starts_with("//"))
            .any(|l| {
                l.contains("declare var document") || l.contains("declare const document")
            }));
    }

    #[test]
    fn project_oxlintrc_is_recommended_plus_typescript() {
        let body = emit_project_oxlintrc_json();
        assert!(
            body.contains("\"typescript\""),
            "oxlint must enable typescript plugin:\n{body}"
        );
        assert!(
            body.contains("\"correctness\""),
            "oxlint must enable recommended correctness category:\n{body}"
        );
        assert!(
            body.contains("\"eslint\"") && body.contains("\"oxc\""),
            "oxlint recommended plugin set incomplete:\n{body}"
        );
        // Sail-specific rules stay in sail-compiler — config must not pretend
        // to own template vocabulary (no `sail-*` / `sail/` rule entries).
        assert!(
            !body.contains("\"sail-") && !body.contains("\"sail/"),
            "oxlint must not embed sail-compiler rules:\n{body}"
        );
    }

    #[test]
    fn package_tsconfig_base_matches_emit_flags() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let path = root.join("packages/sail/tsconfig/tsconfig.base.json");
        let raw = fs::read_to_string(&path).expect("tsconfig.base.json readable");
        for flag in SAIL_TSCONFIG_REQUIRED_FLAGS {
            assert!(
                raw.contains(flag),
                "packages/sail/tsconfig/tsconfig.base.json missing {flag}:\n{raw}"
            );
        }
    }

    #[test]
    fn package_oxlint_base_matches_emit() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let path = root.join("packages/sail/tsconfig/oxlint.base.json");
        let raw = fs::read_to_string(&path).expect("oxlint.base.json readable");
        let emit = emit_project_oxlintrc_json();
        // Structural parity: plugins + categories must agree (schema path may
        // differ slightly between package and emit — compare rule surface).
        for needle in [
            "\"typescript\"",
            "\"eslint\"",
            "\"oxc\"",
            "\"correctness\": \"error\"",
            "\"suspicious\": \"warn\"",
        ] {
            assert!(
                raw.contains(needle),
                "package oxlint missing {needle}:\n{raw}"
            );
            assert!(
                emit.contains(needle),
                "emit oxlint missing {needle}:\n{emit}"
            );
        }
    }
}
