//! Generated ambient pack for sail TypeScript regions.
//!
//! Emit path: `<project>/.sail/types.d.ts` via `sail-compiler types` (or the
//! deck `--typecheck` hook). The pack is **not** hand-authored in app source —
//! every name comes from the same tables the compiler / lint already own:
//!
//! - [`crate::runtime_ambient::RUNTIME_AMBIENT`] — controller vocabulary
//! - `sail_lint::style::setters` + scales — Style builder surface
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
use sail_lint::style::setters::{SetterSpec, setters};

/// Relative path of the ambient pack inside a sail project.
pub const PROJECT_TYPES_REL: &str = ".sail/types.d.ts";

/// Regenerated header — agents/humans must not hand-edit the pack.
const HEADER: &str = "\
// GENERATED FILE — do not edit by hand.
//
// Sail ambient pack. Source of truth:
//   crates/sail/compiler/src/runtime_ambient.rs
//   crates/sail/style/src/setters.rs (+ scales / token registry)
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

/// Motion chain methods on Style (`style.enter({…})`).
const MOTION_CHAIN_METHODS: &[&str] = &[
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
    out.push_str(
        "import type {\n\
         \x20 Signal,\n\
         \x20 Derived,\n\
         \x20 ControllerDef,\n\
         \x20 ControllerScope,\n\
         \x20 QueryOpts,\n\
         \x20 Mutation,\n\
         \x20 MutationOpts,\n\
         \x20 MutationState,\n\
         \x20 LiveState,\n\
         \x20 LiveSignal,\n\
         \x20 LiveSource,\n\
         \x20 OptimisticHandle,\n\
         \x20 QueryKey,\n\
         } from '@ship/sail-runtime'\n\
         \n",
    );

    let bare_tokens = emit_token_unions(&mut out);
    emit_style_types(&mut out);
    emit_globals(&mut out, &bare_tokens);

    // Ensure this file is a module (global augmentation needs export {}).
    out.push_str("export {}\n");
    out
}

/// Write `<project>/.sail/types.d.ts`. Creates `.sail/` as needed.
pub fn write_project_types(project: &Path) -> Result<PathBuf> {
    let path = project.join(PROJECT_TYPES_REL);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("mkdir {}", parent.display()))?;
    }
    let text = emit_types_dts();
    fs::write(&path, text).with_context(|| format!("write {}", path.display()))?;
    Ok(path)
}

/// Default sail project `compilerOptions` shared by typecheck + scaffold docs.
pub fn sail_tsconfig_compiler_options_json() -> &'static str {
    r#"{
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "strict": true,
    "noImplicitAny": true,
    "skipLibCheck": true,
    "noEmit": true,
    "allowImportingTsExtensions": true
  }"#
}

/// Build a project-root `tsconfig.json` body that includes the ambient pack.
///
/// `include_globs` are relative to the project (e.g. `src/**/*.ts`).
pub fn emit_project_tsconfig_json(include_globs: &[&str]) -> String {
    let mut includes = Vec::with_capacity(include_globs.len() + 1);
    includes.push(format!("\"{PROJECT_TYPES_REL}\""));
    for g in include_globs {
        includes.push(format!("\"{g}\""));
    }
    format!(
        "{{\n  \
         \"compilerOptions\": {opts},\n  \
         \"include\": [{inc}],\n  \
         \"exclude\": [\"node_modules\", \"dist\", \".deck\", \".sail-out\"]\n\
         }}\n",
        opts = sail_tsconfig_compiler_options_json(),
        inc = includes.join(", "),
    )
}

// ── token unions ───────────────────────────────────────────────────────────

/// Emit scale unions; return bare token strings eligible for global consts.
fn emit_token_unions(out: &mut String) -> BTreeSet<String> {
    out.push_str("// ── Closed token unions (from lint scales / token registry) ──\n");

    let scales = [
        "color", "space", "size", "radius", "text", "weight", "font", "leading",
        "tracking", "shadow", "z", "duration", "ease", "percent", "angle",
        "border-width", "opacity",
    ];
    let mut all_bare: BTreeSet<String> = BTreeSet::new();

    for scale in scales {
        let name = scale_type_name(scale);
        let vals = scales::values(scale);
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
        out.push_str("\n");
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

    all_bare
        .into_iter()
        .filter(|t| is_ambient_const_name(t))
        .collect()
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
    if JS_RESERVED.iter().any(|r| *r == tok) {
        return false;
    }
    true
}

// ── Style builder ──────────────────────────────────────────────────────────

fn emit_style_types(out: &mut String) {
    out.push_str("// ── Style builder (from sail_lint::style::setters) ──\n");

    out.push_str(
        "/** Author motion config object (`enter({ opacity: 'fade' })`).\n\
         \x20*  Hyphenated property keys need quotes under pure tsc. */\n\
         type SailStyleMotionConfig = {\n\
         \x20 opacity?: string | number\n\
         \x20 scale?: string | number\n\
         \x20 'scale-x'?: string | number\n\
         \x20 'scale-y'?: string | number\n\
         \x20 'translate-x'?: string | number\n\
         \x20 'translate-y'?: string | number\n\
         \x20 rotate?: string | number\n\
         \x20 stagger?: string | number\n\
         \x20 duration?: string | number\n\
         \x20 delay?: string | number\n\
         \x20 [key: string]: string | number | undefined\n\
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
        "/** Channel partial — product spine keys. */\n\
         type SailStyleChannels = {\n\
         \x20 box?: SailStyleValue\n\
         \x20 paint?: SailStyleValue\n\
         \x20 text?: SailStyleValue\n\
         \x20 animate?: SailStyleValue\n\
         }\n\
         \n\
         type SailWhenMap = { [K in SailWhenKey]?: SailStyleValue | SailStyleChannels }\n\
         \n\
         /** Opaque compile-time Style value (chain / match / when / list / ref). */\n\
         type SailStyleValue = SailStyleChain\n\
         \n",
    );

    out.push_str("interface SailStyleChain {\n");
    for spec in setters() {
        emit_setter_method(out, spec);
    }
    // Motion methods on the chain
    for m in MOTION_CHAIN_METHODS {
        // scroll/press dual paint — already on setters; overload with motion object.
        if *m == "scroll" || *m == "press" {
            out.push_str(&format!(
                "  {m}(config: SailStyleMotionConfig): SailStyleChain\n"
            ));
            continue;
        }
        if *m == "focus-visible" {
            // invalid bare method name in TS — quote
            out.push_str(
                "  'focus-visible'(config: SailStyleMotionConfig): SailStyleChain\n",
            );
            continue;
        }
        out.push_str(&format!(
            "  {m}(config: SailStyleMotionConfig): SailStyleChain\n"
        ));
    }
    // Map form + cond form (product: `.when(hover, { paint: … })`).
    out.push_str("  when(map: SailWhenMap): SailStyleChain\n");
    out.push_str(
        "  when(key: SailWhenKey, partial: SailStyleChannels | SailStyleValue): SailStyleChain\n",
    );
    out.push_str("}\n\n");

    // Callable channel head + residual flat chain methods (unit/codemod dual).
    out.push_str(
        "/** Compile-time Style head — product: `style({ box, paint, text, animate })`.\n\
         \x20*  Residual flat `style.method` remains typed for codemod/unit dual path. */\n\
         interface SailStyleBuilder extends SailStyleChain {\n\
         \x20 (channels?: SailStyleChannels): SailStyleChain\n\
         \x20 (items: SailStyleValue[]): SailStyleChain\n\
         \x20 (ref: SailStyleValue): SailStyleChain\n\
         }\n\
         \n",
    );

    // Free forms
    out.push_str(
        "type SailMatchArms = { [arm: string]: SailStyleValue }\n\
         \n",
    );
}

fn emit_setter_method(out: &mut String, spec: &SetterSpec) {
    let name = spec.name;
    if ZERO_ARG_SETTERS.contains(&name) {
        out.push_str(&format!("  {name}(): SailStyleChain\n"));
        // press also has motion form — handled in motion loop
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
        out.push_str(&format!(
            "  {name}(a: {arg}, b?: {arg}): SailStyleChain\n"
        ));
        return;
    }
    out.push_str(&format!("  {name}(token: {arg}): SailStyleChain\n"));
}

fn arg_type_for_setter(spec: &SetterSpec) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(scale) = spec.scale {
        parts.push(scale_type_name(scale));
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

    // Setter / motion / when-key names that also appear as token idents need a
    // callable∩literal hybrid — cannot emit both `const border` and `function border`.
    let setter_names: BTreeSet<&str> = setters().iter().map(|s| s.name).collect();
    let motion_free: BTreeSet<&str> = MOTION_CHAIN_METHODS
        .iter()
        .copied()
        .filter(|m| *m != "focus-visible")
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
            "  const {tok}: {lit}\n",
            lit = ts_literal(tok)
        ));
    }
    out.push('\n');

    // Runtime ambient vocabulary — typed via @ship/sail-runtime imports.
    for (authored, _exported, doc) in crate::runtime_ambient::RUNTIME_AMBIENT {
        if *authored == "__invalidate" {
            // Compiler-only; still declare so rewritten output typechecks.
            out.push_str("  /** compiler-only lowering target for invalidate(ref) */\n");
            out.push_str(
                "  function __invalidate(prefix: QueryKey): void\n",
            );
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
                out.push_str(
                    "  function effect(fn: () => void | (() => void)): () => void\n",
                );
            }
            "batch" => {
                out.push_str("  function batch(fn: () => void): void\n");
            }
            "untrack" => {
                out.push_str("  function untrack<T>(fn: () => T): T\n");
            }
            "mount" => {
                out.push_str(
                    "  function mount(fn: (el: Element) => void | (() => void)): void\n",
                );
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
                out.push_str(&format!(
                    "  const {other}: (...args: never[]) => unknown\n"
                ));
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
         \x20 /** Freeze-locked optimistic write handle (type-only until impl lands). */\n\
         \x20 type OptimisticWriteHandle<T> = OptimisticHandle<T>\n\
         \n",
    );

    // Setter keyword idents used as bare args: `cursor(pointer)`, `overflow(hidden)`.
    // Not always present on a token scale — still need ambient consts for tsc.
    out.push_str("  // Style setter keyword idents (bare args).\n");
    let mut kw_seen: BTreeSet<&str> = BTreeSet::new();
    for spec in setters() {
        for kw in spec.keywords {
            if !is_ambient_const_name(kw) {
                continue;
            }
            if bare_tokens.contains(*kw) || hybrid_names.contains(kw) || !kw_seen.insert(kw) {
                continue;
            }
            out.push_str(&format!("  const {kw}: '{kw}'\n"));
        }
    }
    out.push('\n');

    // Style head + free forms (channel product).
    out.push_str(
        "  /** Compile-time Style builder — erased to classes by sail-compiler. */\n\
         \x20 const style: SailStyleBuilder\n\
         \x20 function match(scrutinee: unknown, arms: SailMatchArms): SailStyleValue\n\
         \x20 function when(map: SailWhenMap): SailStyleValue\n\
         \x20 function when(key: SailWhenKey, partial: SailStyleChannels | SailStyleValue): SailStyleValue\n\
         \x20 function compound(\n\
         \x20\x20  keys: Record<string, unknown>,\n\
         \x20\x20  body: SailStyleValue,\n\
         \x20\x20): SailStyleValue\n\
         \x20 function bind(name: string): SailBind\n\
         \n",
    );

    // Free setters for channel bodies: `box: pad(4).gap(2)`, `paint: bg(primary)`.
    out.push_str("  // Free Style setters (channel body chains).\n");
    for spec in setters() {
        emit_free_setter(out, spec, bare_tokens);
    }

    // Free motion triggers for animate channel: `animate: enter({…}).exit({…})`.
    out.push_str("  // Free motion triggers (animate channel).\n");
    for m in MOTION_CHAIN_METHODS {
        if *m == "focus-visible" {
            // Invalid bare ident — virtual_ts rewrites `focus-visible(` → this name.
            out.push_str(
                "  function __sailFocusVisible(config: SailStyleMotionConfig): SailStyleChain\n",
            );
            continue;
        }
        if setter_names.contains(m) {
            // press/scroll/hover already emitted as free setter (paint) — add motion overload via hybrid below.
            continue;
        }
        if when_key_set.contains(m) {
            continue; // hybrid with when-key
        }
        out.push_str(&format!(
            "  function {m}(config: SailStyleMotionConfig): SailStyleChain\n"
        ));
    }

    // Hybrids: token∩setter, when-key∩motion, press dual paint/motion.
    out.push_str("  // Hybrid token / when-key / dual paint-motion free forms.\n");
    for tok in bare_tokens {
        if !setter_names.contains(tok.as_str()) {
            continue;
        }
        let Some(spec) = setters().iter().find(|s| s.name == tok.as_str()) else {
            continue;
        };
        emit_hybrid_token_setter(out, spec, tok);
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
                if let Some(spec) = setters().iter().find(|s| s.name == *k) {
                    emit_hybrid_when_setter(out, spec, k, is_motion);
                }
            }
            continue;
        }
        if is_motion {
            out.push_str(&format!(
                "  const {k}: ((config: SailStyleMotionConfig) => SailStyleChain) & '{k}'\n"
            ));
            continue;
        }
        if is_token {
            out.push_str(&format!("  const {k}: '{k}'\n"));
        }
    }
    // press: paint zero-arg + motion object (setter already hybrid if token; else dual free).
    if setter_names.contains("press") && !bare_tokens.contains("press") {
        out.push_str(
            "  const press: {\n\
             \x20\x20 (): SailStyleChain\n\
             \x20\x20 (config: SailStyleMotionConfig): SailStyleChain\n\
             \x20 }\n",
        );
    }
    if setter_names.contains("scroll") && !bare_tokens.contains("scroll") {
        // scroll paint form takes a token; free setter emit covers paint — motion overload:
        // free setter already emitted; add ambient overload via interface merge is hard.
        // Chain method + free motion: re-emit motion-capable free scroll.
        out.push_str(
            "  function scroll(config: SailStyleMotionConfig): SailStyleChain\n",
        );
    }

    out.push_str("}\n\n");
}

fn emit_free_setter(out: &mut String, spec: &SetterSpec, bare_tokens: &BTreeSet<String>) {
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
    if JS_RESERVED.iter().any(|r| *r == name) {
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

fn emit_hybrid_token_setter(out: &mut String, spec: &SetterSpec, tok: &str) {
    let name = spec.name;
    if name.contains('-') {
        out.push_str(&format!("  const {tok}: {lit}\n", lit = ts_literal(tok)));
        return;
    }
    if ZERO_ARG_SETTERS.contains(&name) {
        let motion = if name == "press" {
            "  (config: SailStyleMotionConfig): SailStyleChain\n"
        } else {
            ""
        };
        out.push_str(&format!(
            "  const {name}: {{\n\
             \x20\x20 (): SailStyleChain\n\
             {motion}\
             \x20 }} & {lit}\n",
            lit = ts_literal(tok)
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
             \x20 }} & {lit}\n",
            lit = ts_literal(tok)
        ));
        return;
    }
    let motion = if name == "hover" || name == "scroll" {
        "  (config: SailStyleMotionConfig): SailStyleChain\n"
    } else {
        ""
    };
    out.push_str(&format!(
        "  const {name}: {{\n\
         \x20\x20 (token: {arg}): SailStyleChain\n\
         {motion}\
         \x20 }} & {lit}\n",
        lit = ts_literal(tok)
    ));
}

fn emit_hybrid_when_setter(out: &mut String, spec: &SetterSpec, key: &str, is_motion: bool) {
    let arg = arg_type_for_setter(spec);
    let motion = if is_motion {
        "  (config: SailStyleMotionConfig): SailStyleChain\n"
    } else {
        ""
    };
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
        for spec in setters() {
            // focus-visible is quoted in the interface
            let needle = if spec.name.contains('-') {
                format!("'{}'", spec.name)
            } else {
                format!("{}(", spec.name)
            };
            // zero-arg: `group()`
            let alt = format!("{}()", spec.name);
            assert!(
                dts.contains(&needle) || dts.contains(&alt) || dts.contains(spec.name),
                "ambient pack missing Style setter `{}`",
                spec.name
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
    fn pack_types_style_head_and_free_forms() {
        let dts = emit_types_dts();
        assert!(dts.contains("const style: SailStyleBuilder"));
        assert!(
            dts.contains("interface SailStyleBuilder extends SailStyleChain"),
            "style head must be callable channel form"
        );
        assert!(
            dts.contains("box?: SailStyleValue") && dts.contains("paint?: SailStyleValue"),
            "SailStyleChannels missing"
        );
        // Free channel-body setters
        assert!(dts.contains("function pad(") || dts.contains("function padX("), "{dts}");
        assert!(dts.contains("function bg(") || dts.contains("const bg:"), "free bg");
        assert!(dts.contains("function enter("), "free enter motion");
        assert!(dts.contains("const hover:"), "when-key / motion hybrid hover");
        assert!(dts.contains("function match("));
        assert!(dts.contains("function when("));
        assert!(dts.contains("function compound("));
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
    fn ambient_const_name_rules() {
        assert!(is_ambient_const_name("primary"));
        assert!(is_ambient_const_name("full"));
        assert!(!is_ambient_const_name("muted-foreground"));
        assert!(!is_ambient_const_name("2"));
        assert!(!is_ambient_const_name("1_5"));
        assert!(!is_ambient_const_name("default"));
    }
}
