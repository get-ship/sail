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
         \x20 LiveSource,\n\
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
        "/** Author motion config object (`style.enter({ opacity: 'fade' })`).\n\
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
        "type SailWhenMap = { [K in SailWhenKey]?: SailStyleValue }\n\
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
    out.push_str("  when(map: SailWhenMap): SailStyleChain\n");
    out.push_str("}\n\n");

    // `style` is a chain head (same methods, starts empty).
    out.push_str(
        "/** Compile-time Style builder head — `style.bg(primary).padX(2)`. */\n\
         type SailStyleBuilder = SailStyleChain\n\
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

    // Bare Style-token identifiers so `style.bg(primary)` typechecks across
    // files. Hyphenated tokens use the string form under tsc.
    out.push_str(
        "  // Style token idents (valid JS names only).\n\
         \x20 // Hyphenated: style.bg('muted-foreground') under tsc;\n\
         \x20 // sail-compiler still accepts the Style-DSL bare form.\n",
    );
    for tok in bare_tokens {
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
    out.push_str(
        "  /** Public read primitive — compiler derives the cache key. */\n\
         \x20 function query<T>(\n\
         \x20\x20  fn: (signal: AbortSignal) => Promise<T> | LiveSource<T>,\n\
         \x20\x20  opts?: QueryOpts,\n\
         \x20\x20): Signal<LiveState<T>>\n\
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
         \x20\x20): Signal<LiveState<T>>\n\
         \x20 function invalidate(ref: unknown): void\n\
         \n",
    );

    // Style head + free forms
    out.push_str(
        "  /** Compile-time Style builder — erased to classes by sail-compiler. */\n\
         \x20 const style: SailStyleBuilder\n\
         \x20 function match(scrutinee: unknown, arms: SailMatchArms): SailStyleValue\n\
         \x20 function when(map: SailWhenMap): SailStyleValue\n\
         \x20 function compound(\n\
         \x20\x20  keys: Record<string, unknown>,\n\
         \x20\x20  body: SailStyleValue,\n\
         \x20\x20): SailStyleValue\n\
         \x20 function bind(name: string): SailBind\n\
         \n",
    );

    out.push_str("}\n\n");
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
    fn pack_types_style_head_and_free_forms() {
        let dts = emit_types_dts();
        assert!(dts.contains("const style: SailStyleBuilder"));
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
