//! Generated ambient pack for sail TypeScript regions.
//!
//! Emit path: `<project>/node_modules/.sail/types.d.ts` via `sail-compiler
//! types` (or the deck `--typecheck` hook) — SPEC-OUTPUT-1 Option B. The pack
//! is **not** hand-authored in app source — every name comes from the same
//! tables the compiler / lint already own:
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

use sail_lint::component_surfaces;
use sail_lint::scales;
use sail_lint::style::setters::{SetterSpec, setters};
use sail_lint::token_groups::{self, color_group_names, color_groups, size_step_names};

/// Relative path of the ambient pack inside a sail project (SPEC-OUTPUT-1 §4).
/// Lives under `node_modules/.sail/` so `rm -rf node_modules` clears it;
/// `.deck/` stays purely servable output; ops session stays under `.sail/`.
pub const PROJECT_TYPES_REL: &str = "node_modules/.sail/types.d.ts";

/// Regenerated header — agents/humans must not hand-edit the pack.
const HEADER: &str = "\
// GENERATED FILE — do not edit by hand.
//
// Sail ambient pack. Source of truth:
//   crates/sail/compiler/src/runtime_ambient.rs
//   crates/sail/style/src/setters.rs (+ scales / token registry)
//   crates/sail/style/src/token_groups.rs (color groups / Tone + Size + member paths)
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
         \x20 RouteSignals,\n\
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

/// Write `<project>/node_modules/.sail/types.d.ts`. Creates parents as needed.
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

/// Default sail project `compilerOptions` shared by typecheck + scaffold.
///
/// Mirrors `packages/sail/tsconfig/tsconfig.base.json` (TOOLING-1 WI2).
/// Ambient pack registration is **not** a `compilerOptions.types` package
/// name: the generated file lives at [`PROJECT_TYPES_REL`] and is wired via
/// `include` in [`emit_project_tsconfig_json`] so globals resolve under tsc.
pub fn sail_tsconfig_compiler_options_json() -> &'static str {
    r#"{
    "target": "ESNext",
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "lib": ["ESNext", "DOM", "DOM.Iterable"],
    "strict": true,
    "noUncheckedIndexedAccess": true,
    "exactOptionalPropertyTypes": true,
    "noImplicitOverride": true,
    "verbatimModuleSyntax": true,
    "erasableSyntaxOnly": true,
    "isolatedModules": true,
    "skipLibCheck": true,
    "noEmit": true,
    "allowImportingTsExtensions": true
  }"#
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

/// Build a project-root `tsconfig.json` body that registers the ambient pack.
///
/// `include_globs` are relative to the project (e.g. `src/**/*.ts`).
/// The generated ambient pack ([`PROJECT_TYPES_REL`]) is listed under `files`
/// (not `include`) so tsc still loads it when `exclude` contains `node_modules`
/// — pack lives under `node_modules/.sail/` (SPEC-OUTPUT-1 §4 Option B).
///
/// `compilerOptions.baseUrl` + `paths` come from
/// [`crate::project_map::at_alias_paths`] (SPEC-PROJECT-STRUCTURE-1 §3) so
/// editors/tsc and deck resolve share one alias map.
pub fn emit_project_tsconfig_json(include_globs: &[&str]) -> String {
    let includes: Vec<String> = include_globs.iter().map(|g| format!("\"{g}\"")).collect();
    let opts = sail_tsconfig_compiler_options_json();
    // Drop outer braces so we can inject baseUrl + paths from the SSOT.
    let opts_inner = opts
        .trim()
        .trim_start_matches('{')
        .trim_end_matches('}')
        .trim();
    let path_entries: Vec<String> = crate::project_map::at_alias_paths()
        .iter()
        .map(|m| format!("      \"{}\": [\"{}\"]", m.pattern, m.target))
        .collect();
    let paths_block = path_entries.join(",\n");
    format!(
        "{{\n  \
         \"$comment\": \"Sail project tsconfig (TOOLING-1 WI2). Ambient pack in files (not include) so node_modules exclude does not drop {pack}.\",\n  \
         \"compilerOptions\": {{\n    {opts_inner},\n    \"baseUrl\": \".\",\n    \"paths\": {{\n{paths}\n    }}\n  }},\n  \
         \"files\": [\"{pack}\"],\n  \
         \"include\": [{inc}],\n  \
         \"exclude\": [\"node_modules\", \"dist\", \".deck\"]\n\
         }}\n",
        pack = PROJECT_TYPES_REL,
        paths = paths_block,
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
    // Historic alias for generated docs that still name SailBoundTonePath.
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
            out.push_str(&format!(
                "    {}: '{}'\n",
                m.name, m.public_name
            ));
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
        fields.push(format!(
            "readonly {}: '{}.{}'",
            m.name, group.name, m.name
        ));
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
        "/** Style object partial — channels + condition keys (SPEC-STYLE-OBJECT-1). */\n\
         type SailStyleObj = {\n\
         \x20 box?: SailStyleValue\n\
         \x20 paint?: SailStyleValue\n\
         \x20 text?: SailStyleValue\n\
         \x20 animate?: SailStyleValue\n\
         \x20 when?: SailWhenMap\n\
         \x20 match?: SailMatchBlock\n\
         \x20 compound?: SailCompoundRow[]\n\
         }\n\
         \n\
         type SailStyleChannels = SailStyleObj\n\
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
    out.push_str("}\n\n");

    // Callable style head — variadic partials (SPEC-STYLE-OBJECT-1).
    out.push_str(
        "/** Compile-time Style head — product: `style(partial, partial, …)`. */\n\
         interface SailStyleBuilder {\n\
         \x20 (...partials: Array<SailStyleObj | SailStyleValue>): SailStyleChain\n\
         }\n\
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

    // Style head (SPEC-STYLE-OBJECT-1 — free match/when/compound deleted).
    out.push_str(
        "  /** Compile-time Style builder — erased to classes by sail-compiler. */\n\
         \x20 const style: SailStyleBuilder\n\
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

fn emit_hybrid_token_setter(out: &mut String, spec: &SetterSpec, tok: &str) {
    let name = spec.name;
    let tok_ty = bare_token_type_expr(tok);
    if name.contains('-') {
        out.push_str(&format!("  const {tok}: {tok_ty}\n"));
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
    let motion = if name == "hover" || name == "scroll" {
        "  (config: SailStyleMotionConfig): SailStyleChain\n"
    } else {
        ""
    };
    out.push_str(&format!(
        "  const {name}: {{\n\
         \x20\x20 (token: {arg}): SailStyleChain\n\
         {motion}\
         \x20 }} & {tok_ty}\n"
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
    fn pack_covers_author_surface_exports() {
        // TS-1 WI2.5: every remaining public export (author-surface.json)
        // appears in the ambient pack — type import or global declaration.
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let path = root.join("packages/sail/runtime/author-surface.json");
        let raw = fs::read_to_string(&path).expect("author-surface.json readable");
        let v: serde_json::Value =
            serde_json::from_str(&raw).expect("author-surface.json is JSON");
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
            dts.contains("interface SailStyleBuilder"),
            "style head must be callable channel form"
        );
        assert!(
            dts.contains("type SailStyleObj") && dts.contains("paint?: SailStyleValue"),
            "SailStyleObj missing"
        );
        // Free channel-body setters
        assert!(dts.contains("function pad(") || dts.contains("function padX("), "{dts}");
        assert!(dts.contains("function bg(") || dts.contains("const bg:"), "free bg");
        assert!(dts.contains("function enter("), "free enter motion");
        assert!(dts.contains("const hover:"), "when-key / motion hybrid hover");
        assert!(!dts.contains("function match("), "free match() deleted");
        assert!(!dts.contains("function compound("), "free compound() deleted");
        assert!(dts.contains("match?: SailMatchBlock") || dts.contains("type SailMatchBlock"), "match: key type");
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
    fn project_tsconfig_carries_tooling_wi2_flags_and_ambient_include() {
        let opts = sail_tsconfig_compiler_options_json();
        for flag in SAIL_TSCONFIG_REQUIRED_FLAGS {
            assert!(
                opts.contains(flag),
                "sail_tsconfig_compiler_options_json missing {flag}:\n{opts}"
            );
        }
        let body = emit_project_tsconfig_json(&["src/**/*.ts", "routes/**/*.ts"]);
        for flag in SAIL_TSCONFIG_REQUIRED_FLAGS {
            assert!(
                body.contains(flag),
                "emit_project_tsconfig_json missing {flag}:\n{body}"
            );
        }
        assert!(
            body.contains(PROJECT_TYPES_REL)
                && body.contains("\"files\"")
                && body.contains(format!("\"{PROJECT_TYPES_REL}\"").as_str()),
            "project tsconfig must list ambient pack under files:\n{body}"
        );
        assert!(
            body.contains("\"src/**/*.ts\"") && body.contains("\"routes/**/*.ts\""),
            "project tsconfig must include authored globs:\n{body}"
        );
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
            assert!(raw.contains(needle), "package oxlint missing {needle}:\n{raw}");
            assert!(emit.contains(needle), "emit oxlint missing {needle}:\n{emit}");
        }
    }
}
