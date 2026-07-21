//! `sail-compiler` CLI. Compiles a single `.sail` file or a directory of
//! them, writing the emitted JS (and CEM/schema/meta/dts/ssr.html sidecars)
//! into a project-level mirror tree at `<repo-root>/.sail-out/<source-
//! relative-path>/`. Pass `--out <dir>` to override and emit every file to
//! a single directory (legacy behaviour). Errors render with file path,
//! line, and column.
//
// FILE_CAP_OVERRIDE: single cohesive CLI driver — arg parsing (clap), the
// component-path guardrail, mirror-tree resolution, per-file emit, and the
// fatal-diagnostics exit-code gate are one binary's responsibility. Splitting
// would scatter the one entry point across modules for no clarity gain.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::thread;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};

use sail_compiler::out_layout::resolve_out_dir;
use sail_compiler::{check_all, compile, CompileMode, ComponentMeta, CompileOpts, DiagSeverity, Policy};

/// Hard caps on virtual memory and wall-clock time. A bug in the
/// compiler (or a pathological input) must never be able to OOM the
/// host: a single uncaught `Vec::with_capacity` once allocated 19+GB
/// before the kernel killed the process and took down the surrounding
/// VM. These bounds guarantee a clean death long before that.
///
/// Generous for any real input: a single `.sail` file compiles in
/// milliseconds with <100MB RSS. A directory of hundreds compiles in
/// seconds. Hitting either cap is a bug to file, not a knob to tune.
/// To override (e.g. unusually large project, deliberate stress run),
/// set `SAIL_COMPILER_NO_LIMITS=1`.
const MEM_CAP_BYTES: u64 = 2 * 1024 * 1024 * 1024; // 2 GB virtual memory
const WALL_CAP_SECS: u64 = 60;

fn install_resource_bounds() {
    if std::env::var_os("SAIL_COMPILER_NO_LIMITS").is_some() {
        return;
    }
    // SAFETY: setrlimit with valid resource id and rlimit struct is sound.
    unsafe {
        let lim = libc::rlimit { rlim_cur: MEM_CAP_BYTES as libc::rlim_t, rlim_max: MEM_CAP_BYTES as libc::rlim_t };
        libc::setrlimit(libc::RLIMIT_AS, &lim);
    }
    thread::Builder::new()
        .name("sail-compiler-watchdog".into())
        .spawn(|| {
            thread::sleep(Duration::from_secs(WALL_CAP_SECS));
            eprintln!("sail-compiler: aborted after {WALL_CAP_SECS}s wall-clock cap");
            process::abort();
        })
        .expect("spawn watchdog thread");
}

#[derive(Parser)]
#[command(name = "sail-compiler", about = "AOT compiler for .sail files")]
struct Cli {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Compile one file or every `.sail` under a directory.
    Build {
        /// Path to a `.sail` file or a directory.
        input: PathBuf,
        /// Output directory. When omitted, each file's outputs land in the
        /// project mirror tree at `<repo-root>/.sail-out/<source-rel>/`
        /// and the JS module is named `<stem>.compiled.js`. When supplied,
        /// every file is emitted into this single directory and the JS
        /// module is named `<stem>.js` (legacy behaviour).
        #[arg(long)]
        out: Option<PathBuf>,
        /// Remove each source file's mirror directory (or `--out` directory)
        /// before emitting. Idempotent rebuilds.
        #[arg(long)]
        clean: bool,
        /// Override mode (otherwise auto-detected from --component-name).
        #[arg(long)]
        mode: Option<ModeArg>,
        /// Suppress emitting the `.sail.d.ts` declaration file.
        #[arg(long)]
        no_dts: bool,
        /// Emit `import "@sail-tag/<tag>";` per used custom-element tag so the
        /// bundler can tree-shake exactly the components a route uses (the
        /// per-route resolver). The bundler resolves `@sail-tag/<tag>` to the
        /// component source; default off leaves component loading to the host.
        #[arg(long)]
        emit_component_imports: bool,
        /// Custom element tag name for this component file. Setting this flag
        /// implies component mode — no `--mode component` needed. When omitted
        /// for a file in component mode, defaults to the filename stem.
        #[arg(long)]
        component_name: Option<String>,
    },
    /// Generate the agent-authoring SKILL.md for a working directory. The
    /// content derives from the SAME sources the compiler enforces — the
    /// resolved styling policy (`sail.config.json` walk-up), the sx vocabulary
    /// tables the lint validates against, the layout keywords, and the motion
    /// preset catalog — so the skill can never drift from the compiler. A
    /// strict-policy directory gets a skill that never mentions sx; a
    /// full-policy (stdlib) directory gets the complete grammar.
    Skill {
        /// Directory the skill describes (policy resolves from here).
        /// Default: the current directory.
        #[arg(long)]
        dir: Option<PathBuf>,
        /// Output file. Default: `<dir>/.claude/skills/sail-authoring/SKILL.md`.
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Emit the framework's stdlib element skins as a single CSS file — the
    /// compiler's half of the framework stylesheet (`sail.css`); the runtime's
    /// contracts sheet is the other half. The registry is the SSOT; a drift
    /// test keeps the materialized file honest. Regenerates
    /// `packages/sail/tokens/src/element-skins.css` by default.
    FrameworkCss {
        /// Output file. Default: `packages/sail/tokens/src/element-skins.css`
        /// relative to the workspace root.
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Emit the typed-props registry from the stdlib `<scope>` prop
    /// declarations — the data `sail-prop-value` validates template attributes
    /// against, and the SSOT for any props UI (docs detail pages). Replaces
    /// the dead `<prop>`-tag scanner whose registry regenerated empty.
    PropRegistry {
        /// Output file. Default: `crates/sail/compiler/primitives.json`
        /// relative to the workspace root.
        #[arg(long)]
        out: Option<PathBuf>,
        /// Also emit a TypeScript module for the docs site (the generated
        /// props tables on /components/[tag]). Default:
        /// `apps/sail-site/src/prop-registry.gen.ts` relative to the
        /// workspace root. Pass an explicit path to override.
        #[arg(long)]
        ts_out: Option<PathBuf>,
    },
    /// Emit the closed vocabulary used by the tree-sitter grammar generator
    /// as JSON. The grammar package consumes this to regenerate `grammar.js`
    /// and `queries/highlights.scm` so they cannot drift from the compiler's
    /// own manifests. Default writes to stdout; pass `--out` for a file.
    Vocabulary {
        /// Output file. Default: stdout.
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Synthesize a single `.sail` into a manifest-driven, tsc-checkable
    /// pair — a **synthetic controller** (`<stem>.synthetic.controller.ts`)
    /// and a **render** (`<stem>.render.ts`). Each name in `--manifest`
    /// becomes a real ESM import; the compiler stays ignorant of it. A
    /// sibling `controller.ts` (if present) is composed, not rewritten.
    /// Additive: the ambient/`BUILTINS` path and render-eval are untouched.
    Synthesize {
        /// Path to a single `.sail` file.
        input: PathBuf,
        /// Path to the app manifest (JSON / JSONC-lite) — a name → import map.
        #[arg(long)]
        manifest: PathBuf,
        /// Directory to write `<stem>.synthetic.controller.ts` +
        /// `<stem>.render.ts` into.
        #[arg(long)]
        out_dir: PathBuf,
    },
    /// Classify every `sail-*.sail` component in `packages/components/`
    /// against the L1 manifest: a directory whose tag is a stdlib
    /// element or a native-reskin domain is a duplicate; a directory
    /// whose tag has no L1 equivalent is novel. The output is the
    /// evidence the spec asks for — the list of components that
    /// re-implement an L1 element, derived from the same manifest the
    /// consistency test enforces.
    Duplicates {
        /// Directory to walk. Default: `packages/components/` relative
        /// to the workspace root.
        #[arg(long)]
        dir: Option<PathBuf>,
        /// Output file. Default: stdout.
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

#[derive(Clone, Debug, clap::ValueEnum)]
enum ModeArg {
    Component,
    /// L1 language element — static structure + slots + skin; no runtime
    /// identity. Emits `<name>.element.json` (structure/skin/manifest) and
    /// `<name>.skin.css`.
    Element,
    Auto,
}

impl From<ModeArg> for CompileMode {
    fn from(m: ModeArg) -> Self {
        match m {
            ModeArg::Component => CompileMode::Component,
            // Element mode never reaches the component compile path — it is
            // dispatched before compile_one (see the Build handler).
            ModeArg::Element => CompileMode::Component,
            ModeArg::Auto => CompileMode::Auto,
        }
    }
}

fn main() -> Result<()> {
    install_resource_bounds();
    let cli = Cli::parse();
    match cli.cmd {
        Command::Build {
            input, out, clean, mode, no_dts, component_name,
            emit_component_imports,
        } => build(
            input, out, clean, mode, no_dts, component_name,
            emit_component_imports,
        ),
        Command::Skill { dir, out } => gen_skill(dir, out),
        Command::FrameworkCss { out } => gen_framework_css(out),
        Command::PropRegistry { out, ts_out } => gen_prop_registry(out, ts_out),
        Command::Synthesize { input, manifest, out_dir } => {
            synthesize(input, manifest, out_dir)
        }
        Command::Duplicates { dir, out } => gen_duplicates_report(dir, out),
        Command::Vocabulary { out } => gen_vocabulary(out),
    }
}

/// Emit the closed grammar vocabulary as JSON. Used by the tree-sitter
/// grammar generator so `grammar.js` and `queries/highlights.scm` stay in
/// lockstep with the compiler's own manifests.
fn gen_vocabulary(out: Option<PathBuf>) -> Result<()> {
    let value = sail_compiler::grammar_vocabulary_to_json();
    let text = serde_json::to_string_pretty(&value)? + "\n";
    match out {
        Some(path) => {
            std::fs::write(&path, text)?;
            println!("wrote {}", path.display());
        }
        None => {
            print!("{}", text);
        }
    }
    Ok(())
}

/// Materialize the stdlib element skins to a static CSS file — the
/// compiler-sourced half of the framework stylesheet. deck-core serves it
/// (concatenated with the runtime's contracts sheet) as `sail.css`; the
/// drift test `element_skins_css_matches_registry` keeps it in sync.
fn gen_framework_css(out: Option<PathBuf>) -> Result<()> {
    let path = match out {
        Some(p) => p,
        None => workspace_root()?.join("packages/sail/tokens/src/element-skins.css"),
    };
    let banner = "/* GENERATED from the sail-compiler stdlib registry — do not hand-edit.\n \
                  * Regenerate: cargo run -p sail-compiler -- framework-css\n \
                  * L1 element skins: per-element skin_css (usually empty) + residual\n \
                  * rules from stdlib/_residues.css (relocated from author <style>).\n \
                  * Framework contracts (tone/size channels) ride in contracts.css. */\n";
    // Wrapped in the `sail` cascade layer (matches the runtime contracts) so
    // author CSS (unlayered) overrides element skins by layer order.
    let body = format!("{banner}@layer sail {{\n{}}}\n", sail_compiler::stdlib::skins_css());
    std::fs::write(&path, body)?;
    println!("wrote {}", path.display());
    Ok(())
}

fn gen_prop_registry(out: Option<PathBuf>, ts_out: Option<PathBuf>) -> Result<()> {
    use sail_compiler::ir::directive::PropKind;
    let path = match out {
        Some(p) => p,
        None => workspace_root()?.join("crates/sail/compiler/primitives.json"),
    };
    let ts_path = match ts_out {
        Some(p) => p,
        None => workspace_root()?.join("apps/sail-site/src/prop-registry.gen.ts"),
    };
    // Two consumers, two shapes from one walk:
    //  * primitives.json — the lint's validation registry: attribute-checkable
    //    props only (Identity props are plain-field JS values, nothing for
    //    static-attribute validation to check).
    //  * prop-registry.gen.ts — the docs site's props tables: the FULL surface
    //    including Identity props (shown as their verbatim TS type), so the
    //    detail pages are generated, never hand-authored.
    let mut tags = std::collections::BTreeMap::new();
    let mut ts_tags: std::collections::BTreeMap<String, Vec<serde_json::Value>> =
        std::collections::BTreeMap::new();
    for (tag, source) in sail_compiler::stdlib::sources() {
        let program = sail_compiler::parse::parse(source)
            .unwrap_or_else(|e| panic!("stdlib {tag} failed to parse: {e:?}"));
        let mut props = std::collections::BTreeMap::new();
        let mut ts_props: Vec<serde_json::Value> = Vec::new();
        for p in &program.props {
            let ts_type = match &p.kind {
                PropKind::Union(vs) => vs.join(" · "),
                PropKind::Bool => "bool".to_string(),
                PropKind::Number => "number".to_string(),
                PropKind::String => "string".to_string(),
                PropKind::Identity(t) => t.clone(),
            };
            let mut ts_row = serde_json::Map::new();
            ts_row.insert("name".into(), serde_json::json!(p.name));
            ts_row.insert("type".into(), serde_json::json!(ts_type));
            if let Some(d) = &p.default {
                ts_row.insert("default".into(), serde_json::json!(d));
            }
            if let Some(desc) = &p.description {
                ts_row.insert("desc".into(), serde_json::json!(desc));
            }
            ts_props.push(serde_json::Value::Object(ts_row));

            let (values, ty) = match &p.kind {
                PropKind::Union(vs) => (Some(vs.clone()), None),
                PropKind::Bool => (None, Some("bool")),
                PropKind::Number => (None, Some("number")),
                PropKind::String => (None, Some("string")),
                PropKind::Identity(_) => continue,
            };
            let mut rule = serde_json::Map::new();
            if let Some(vs) = values {
                rule.insert("values".into(), serde_json::json!(vs));
            }
            if let Some(t) = ty {
                rule.insert("type".into(), serde_json::json!(t));
            }
            if let Some(d) = &p.default {
                rule.insert("default".into(), serde_json::json!(d));
            }
            if let Some(desc) = &p.description {
                rule.insert("description".into(), serde_json::json!(desc));
            }
            props.insert(p.name.clone(), serde_json::Value::Object(rule));
        }
        if !props.is_empty() {
            tags.insert(tag.to_string(), props);
        }
        if !ts_props.is_empty() {
            ts_tags.insert(tag.to_string(), ts_props);
        }
    }
    let file = serde_json::json!({
        "$comment": "Generated by `cargo run -p sail-compiler -- prop-registry`. Do not hand-edit. Source: <scope> props declarations in crates/sail/compiler/stdlib/*.sail.",
        "primitives": tags,
    });
    std::fs::write(&path, serde_json::to_string_pretty(&file)? + "\n")?;
    println!("wrote {} ({} elements)", path.display(), file["primitives"].as_object().map(|o| o.len()).unwrap_or(0));

    let mut ts = String::new();
    ts.push_str("// AUTO-GENERATED by `cargo run -p sail-compiler -- prop-registry`. Do not edit.\n");
    ts.push_str("// Source: <scope> props declarations in crates/sail/compiler/stdlib/*.sail.\n");
    ts.push_str("// The docs detail pages read THIS — props tables are generated, never hand-authored.\n\n");
    ts.push_str("export interface GenProp {\n  name: string\n  /** Rendered type: unions as `a · b · c`, identity props verbatim TS. */\n  type: string\n  default?: string\n  /** JSDoc from the stdlib declaration, when the prop is documented. */\n  desc?: string\n}\n\n");
    ts.push_str("export const PROP_REGISTRY: Record<string, GenProp[]> = ");
    ts.push_str(&serde_json::to_string_pretty(&ts_tags)?);
    ts.push_str("\n");
    std::fs::write(&ts_path, ts)?;
    println!("wrote {} ({} elements)", ts_path.display(), ts_tags.len());

    // The LANGUAGE surface — sx vocabulary, state/condition prefixes, layout
    // keywords, motion tables — emitted from the same tables the lint and
    // lowering consult, so the /reference page documents the whole surface by
    // construction. Sibling of the prop registry: one regen refreshes both.
    let surface_path = ts_path.with_file_name("language-surface.gen.ts");
    let mut sf = String::new();
    sf.push_str("// AUTO-GENERATED by `cargo run -p sail-compiler -- prop-registry`. Do not edit.\n");
    sf.push_str("// The complete language surface, from the compiler's own tables: what the\n");
    sf.push_str("// lint enforces is what this file documents, by construction.\n\n");
    let atoms: Vec<serde_json::Value> = sail_lint::rules::sx::vocabulary()
        .into_iter()
        .map(|(prop, values)| serde_json::json!({ "prop": prop, "values": values }))
        .collect();
    sf.push_str("export interface SxAtom {\n  prop: string\n  values: string[]\n}\n\n");
    sf.push_str("export const SX_ATOMS: SxAtom[] = ");
    sf.push_str(&serde_json::to_string_pretty(&atoms)?);
    sf.push_str("\n\n");
    let states: Vec<serde_json::Value> = sail_lint::rules::sx::STATE_PREFIXES
        .iter()
        .map(|(n, sel)| serde_json::json!({ "prefix": n, "selector": sel }))
        .collect();
    sf.push_str("export const SX_STATE_PREFIXES: Array<{ prefix: string; selector: string }> = ");
    sf.push_str(&serde_json::to_string_pretty(&states)?);
    sf.push_str("\n\n");
    let conds: Vec<serde_json::Value> = sail_lint::rules::sx::CONDITION_PREFIXES
        .iter()
        .map(|(n, m)| serde_json::json!({ "prefix": n, "media": m }))
        .collect();
    sf.push_str("export const SX_CONDITION_PREFIXES: Array<{ prefix: string; media: string }> = ");
    sf.push_str(&serde_json::to_string_pretty(&conds)?);
    sf.push_str("\n\n");
    let layout: Vec<serde_json::Value> = sail_lint::rules::layout::TAGS
        .iter()
        .map(|t| serde_json::json!({ "tag": t, "props": sail_lint::rules::layout::tag_props(t) }))
        .collect();
    sf.push_str("export const LAYOUT_KEYWORDS: Array<{ tag: string; props: string[] }> = ");
    sf.push_str(&serde_json::to_string_pretty(&layout)?);
    sf.push_str("\n\n");
    // The REFUSED table — every atom the lint refuses, with the thing to use
    // instead. Sourced from `sx_vocab::refusals()` (the same table the lint
    // reads), so the docs teach the closure, by construction. The /reference
    // page renders this as the "Not sx — use this" table; the agent SKILL.md
    // leads with it.
    let refusals: Vec<serde_json::Value> = sail_lint::sx_vocab::refusals()
        .into_iter()
        .map(|(atom, instead)| serde_json::json!({ "atom": atom, "instead": instead }))
        .collect();
    sf.push_str("export const SX_REFUSALS: Array<{ atom: string; instead: string }> = ");
    sf.push_str(&serde_json::to_string_pretty(&refusals)?);
    sf.push_str("\n\n");
    let preset_db = sail_compiler::lower::presets::load().unwrap_or_default();
    let mut presets: Vec<serde_json::Value> = preset_db
        .presets
        .iter()
        .map(|(name, spec)| {
            let kind = match spec.kind {
                sail_compiler::lower::presets::PresetKind::Spring => "spring",
                sail_compiler::lower::presets::PresetKind::Tween => "tween",
            };
            serde_json::json!({ "name": name, "kind": kind, "doc": spec.doc.clone().unwrap_or_default() })
        })
        .collect();
    presets.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
    let motion = serde_json::json!({
        "triggers": sail_lint::rules::animate_token_only::MOTION_TRIGGERS,
        "properties": sail_lint::rules::animate_token_only::MOTION_PROPERTIES,
        "staggerKey": sail_lint::rules::animate_token_only::MOTION_STAGGER_KEY,
        "presets": presets,
    });
    sf.push_str("export interface MotionPreset {\n  name: string\n  kind: string\n  doc: string\n}\n\n");
    sf.push_str("export const MOTION: { triggers: string[]; properties: string[]; staggerKey: string; presets: MotionPreset[] } = ");
    sf.push_str(&serde_json::to_string_pretty(&motion)?);
    sf.push_str("\n\n");

    // ELEMENTS — the L1 manifest (stdlib + native reskins), WITH the
    // one-line description every entry already carries. The prop tables
    // themselves live in prop-registry.gen.ts; this is the "what is this
    // element, in prose" companion.
    let (stdlib_l1, reskin_l1) = sail_compiler::l1_by_kind();
    let to_element_row = |e: &sail_compiler::l1_manifest::L1Entry| {
        serde_json::json!({ "tag": e.tag, "description": e.description })
    };
    let elements = serde_json::json!({
        "stdlib": stdlib_l1.iter().map(to_element_row).collect::<Vec<_>>(),
        "reskins": reskin_l1.iter().map(to_element_row).collect::<Vec<_>>(),
    });
    sf.push_str("export interface ElementDoc {\n  tag: string\n  description: string\n}\n\n");
    sf.push_str("export const ELEMENTS: { stdlib: ElementDoc[]; reskins: ElementDoc[] } = ");
    sf.push_str(&serde_json::to_string_pretty(&elements)?);
    sf.push_str("\n\n");

    // ACTIONS — the `use:` registry, straight from action_manifest::ACTIONS
    // (already sorted, already carries a prose `summary` per entry).
    let actions: Vec<serde_json::Value> = sail_compiler::action_manifest::ACTIONS
        .iter()
        .map(|a| serde_json::json!({ "name": a.name, "param": a.param, "summary": a.summary }))
        .collect();
    sf.push_str("export interface ActionDoc {\n  name: string\n  param: string\n  summary: string\n}\n\n");
    sf.push_str("export const ACTIONS: ActionDoc[] = ");
    sf.push_str(&serde_json::to_string_pretty(&actions)?);
    sf.push_str("\n\n");

    // CONTROLLER — the ambient runtime vocabulary a `<controller>` block may
    // use without importing (signal/derived/effect/mount/onCleanup/the
    // controller factory), straight from runtime_ambient::RUNTIME_AMBIENT.
    let controller_vocab: Vec<serde_json::Value> = sail_compiler::runtime_ambient::RUNTIME_AMBIENT
        .iter()
        .map(|(authored, _exported, doc)| serde_json::json!({ "name": authored, "doc": doc }))
        .collect();
    sf.push_str("export interface AmbientDoc {\n  name: string\n  doc: string\n}\n\n");
    sf.push_str("export const CONTROLLER_AMBIENT: AmbientDoc[] = ");
    sf.push_str(&serde_json::to_string_pretty(&controller_vocab)?);
    sf.push_str("\n\n");

    // BLOCKS — the template block vocabulary (#if/#for/#match/#while-*),
    // straight from block_manifest::BLOCKS.
    let blocks: Vec<serde_json::Value> = sail_compiler::block_manifest::BLOCKS
        .iter()
        .map(|b| serde_json::json!({ "open": b.open, "clauses": b.clauses, "summary": b.summary }))
        .collect();
    sf.push_str("export interface BlockDoc {\n  open: string\n  clauses: string[]\n  summary: string\n}\n\n");
    sf.push_str("export const BLOCKS: BlockDoc[] = ");
    sf.push_str(&serde_json::to_string_pretty(&blocks)?);
    sf.push_str("\n");

    // DIRECTIVES — every attribute-position directive, straight from
    // directive_manifest::DIRECTIVES.
    sf.push_str("export interface DirectiveDoc {\n  syntax: string\n  summary: string\n}\n\n");
    sf.push_str("export const DIRECTIVES: DirectiveDoc[] = ");
    sf.push_str(&serde_json::to_string_pretty(&sail_compiler::directive_manifest::DIRECTIVES)?);
    sf.push_str("\n\n");

    sf.push_str("export interface EventDoc {\n  name: string\n  summary: string\n}\n\n");
    sf.push_str("export const EVENT_NAMES: EventDoc[] = ");
    sf.push_str(&serde_json::to_string_pretty(&sail_compiler::directive_manifest::EVENT_NAMES)?);
    sf.push_str("\n\n");

    sf.push_str("export interface ModifierDoc {\n  canonical: string\n  alias: string | null\n  arg: string | null\n  summary: string\n}\n\n");
    sf.push_str("export const EVENT_MODIFIERS: ModifierDoc[] = ");
    sf.push_str(&serde_json::to_string_pretty(&sail_compiler::directive_manifest::EVENT_MODIFIERS)?);
    sf.push_str("\n\n");

    sf.push_str("export interface BindTargetDoc {\n  target: string\n  summary: string\n}\n\n");
    sf.push_str("export const BIND_TARGETS: BindTargetDoc[] = ");
    sf.push_str(&serde_json::to_string_pretty(&sail_compiler::directive_manifest::BIND_TARGETS)?);
    sf.push_str("\n\n");

    sf.push_str("export interface RetiredDoc {\n  syntax: string\n  instead: string\n}\n\n");
    sf.push_str("export const RETIRED_DIRECTIVES: RetiredDoc[] = ");
    sf.push_str(&serde_json::to_string_pretty(&sail_compiler::directive_manifest::RETIRED_DIRECTIVES)?);
    sf.push_str("\n\n");

    sf.push_str("export interface MotionValueFormDoc {\n  form: string\n  summary: string\n}\n\n");
    sf.push_str("export const MOTION_VALUE_FORMS: MotionValueFormDoc[] = ");
    sf.push_str(&serde_json::to_string_pretty(&sail_compiler::directive_manifest::MOTION_VALUE_FORMS)?);
    sf.push_str("\n\n");

    // SCOPE GRAMMAR — every <scope> construct + refusal, straight from
    // scope_manifest (each row's `example` is compiled by that module's tests).
    sf.push_str("export interface ScopeGrammarDoc {\n  form: string\n  syntax: string\n  summary: string\n  example: string\n}\n\n");
    sf.push_str("export const SCOPE_GRAMMAR: ScopeGrammarDoc[] = ");
    sf.push_str(&serde_json::to_string_pretty(&sail_compiler::scope_manifest::SCOPE_GRAMMAR)?);
    sf.push_str("\n\n");

    sf.push_str("export interface ScopeRefusalDoc {\n  syntax: string\n  instead: string\n  example: string\n}\n\n");
    sf.push_str("export const SCOPE_REFUSALS: ScopeRefusalDoc[] = ");
    sf.push_str(&serde_json::to_string_pretty(&sail_compiler::scope_manifest::SCOPE_REFUSALS)?);
    sf.push_str("\n\n");

    // CONFIG — every sail.config.json key, straight from
    // sail_lint::config::CONFIG_KEY_DOCS (kept in lockstep with
    // RECOGNIZED_KEYS by that module's own test).
    let config_keys: Vec<serde_json::Value> = sail_lint::config::CONFIG_KEY_DOCS
        .iter()
        .map(|(key, ty, default, doc)| {
            serde_json::json!({ "key": key, "type": ty, "default": default, "doc": doc })
        })
        .collect();
    sf.push_str("export interface ConfigKeyDoc {\n  key: string\n  type: string\n  default: string\n  doc: string\n}\n\n");
    sf.push_str("export const CONFIG_KEYS: ConfigKeyDoc[] = ");
    sf.push_str(&serde_json::to_string_pretty(&config_keys)?);
    sf.push_str("\n\n");

    // SX EXTRAS — the group/state-<value>/(--var) forms that are grammar
    // rather than a closed name table, straight from
    // sail_lint::rules::sx::SX_EXTRA_FORMS.
    let sx_extras: Vec<serde_json::Value> = sail_lint::rules::sx::SX_EXTRA_FORMS
        .iter()
        .map(|(form, doc)| serde_json::json!({ "form": form, "doc": doc }))
        .collect();
    sf.push_str("export interface SxExtraFormDoc {\n  form: string\n  doc: string\n}\n\n");
    sf.push_str("export const SX_EXTRA_FORMS: SxExtraFormDoc[] = ");
    sf.push_str(&serde_json::to_string_pretty(&sx_extras)?);
    sf.push_str("\n");

    std::fs::write(&surface_path, sf)?;
    println!(
        "wrote {} (sx {} atoms, {} presets, {} elements, {} actions, {} ambient, {} blocks, {} directives, {} config keys)",
        surface_path.display(),
        atoms.len(),
        presets.len(),
        stdlib_l1.len() + reskin_l1.len(),
        actions.len(),
        controller_vocab.len(),
        blocks.len(),
        sail_compiler::directive_manifest::DIRECTIVES.len(),
        config_keys.len(),
    );
    Ok(())
}

/// Walk up from the current dir to the workspace root (the dir whose
/// `Cargo.toml` declares `[workspace]`).
fn workspace_root() -> Result<PathBuf> {
    let mut dir = std::env::current_dir()?;
    loop {
        let manifest = dir.join("Cargo.toml");
        if manifest.exists()
            && std::fs::read_to_string(&manifest)
                .map(|s| s.contains("[workspace]"))
                .unwrap_or(false)
        {
            return Ok(dir);
        }
        if !dir.pop() {
            anyhow::bail!("could not find workspace root (no [workspace] Cargo.toml above cwd)");
        }
    }
}

/// Generate the agent-authoring skill for `dir` — the compiler's surface,
/// rendered for agents. Sections are conditional on the resolved policy so
/// the skill describes exactly what THIS directory may author.
fn gen_skill(dir: Option<PathBuf>, out: Option<PathBuf>) -> Result<()> {
    use std::fmt::Write as _;
    use sail_compiler::check::policy::Policy;

    let dir = match dir {
        Some(d) => d,
        None => std::env::current_dir()?,
    };
    // Policy resolution walks up from a file; probe with a synthetic path.
    let policy = Policy::resolve_opt(&dir.join("_probe.sail"));
    let effective = policy.unwrap_or(Policy::FULL);
    let policy_line = match policy {
        Some(p) if p == Policy::STRICT => "strict (application tier)",
        Some(_) => "full (stdlib/primitive authoring)",
        None => "none configured — full surface (stdlib default)",
    };

    let mut md = String::new();
    let _ = writeln!(
        md,
        "---\nname: sail-authoring\ndescription: Author .sail components in THIS directory — the compiler-enforced surface (policy: {policy_line}). Generated by `sail-compiler skill`; regenerate after config or token changes, DO NOT hand-edit.\n---\n"
    );
    let _ = writeln!(md, "# Authoring SAIL here\n");
    let _ = writeln!(
        md,
        "This file is GENERATED from the compiler's own vocabulary tables and the\nresolved styling policy for this directory. Everything below is enforced —\nan off-list value is a compile error, and the error message carries the\nallowed list. Trust errors over memory.\n"
    );
    let _ = writeln!(md, "## The hard rules (every policy)\n");
    let _ = writeln!(
        md,
        "- NO `<style>` blocks, NO `style=` attributes — ever. Visuals flow through\n  tokens, props, and the surfaces listed below.\n- NO raw values anywhere (no px, no hex): spend a token.\n- Layout is elements, not CSS: `<row>`, `<column>`, `<grid>`, `<center>`,\n  `<stack>` with `gap`/`align`/`justify`/`cols` attributes. `cols` (and `rows`)\n  is a fixed track count (1..=12) on `<grid>`; `cols` takes precedence over\n  `min` when both are present. `<row nowrap>` is the single-line row (rows wrap\n  by default). Unrecognized attributes on a layout keyword are a compile error —\n  the language won't silently drop them.\n- Behavior is unrestricted TypeScript in `<controller>` — the restraint is\n  visual, never logical.\n- Components declare their static surface in `<scope>` — TypeScript field\n  syntax, types and literal values only: `props: {{ label = 'Default';\n  tone = 'default'; kind: 'a' | 'b' = 'a' }}`. `tone`/`size` are the framework\n  channels. Styling tables live in `props` as CVA key→sx maps\n  (`variant: {{ solid: 'bg-tone …' }}` — first key = default). There is no\n  separate `variants:` block. If it computes, it goes in `<controller>`.\n"
    );
    if effective.sx {
        // LEAD WITH WHAT WE ARE NOT. The agent's Tailwind prior is right
        // about the syntax and wrong about the scope — the most valuable
        // section of this skill is the honest refusal list, so a refused
        // atom names the thing to use instead. The mission is never
        // "find another way." Sources the same table the lint consults
        // (`sail_lint::sx_vocab::refusals()`) — the docs can never drift.
        let _ = writeln!(md, "## What sx is NOT\n");
        let _ = writeln!(
            md,
            "These atoms are refused by name. Each entry below names the thing to\nuse instead — `find another way` is not the answer. The list is the closed\nrefusal table, from the same source the lint enforces (`sx_vocab::refusals()`):\n"
        );
        for (atom, instead) in sail_lint::sx_vocab::refusals() {
            let _ = writeln!(md, "- `sx=\"{atom}\"` — {instead}");
        }
        let _ = writeln!(md);

        let _ = writeln!(md, "## sx — the surface styling attribute\n");
        let _ = writeln!(
            md,
            "`sx=\"name-value name-value …\"` — Tailwind's names, our values. An\natom is `[prefix:]name-value` (e.g. `bg-surface-2`, `hover:bg-muted`,\n`motion-reduce:shadow-none`); several utilities share a name and disambiguate\nby value (`text-sm` is a font size, `text-foreground` is a colour,\n`text-center` is an alignment). Static only (`sx={{expr}}` is a compile\nerror). State prefixes: `hover:` `focus:` `focus-within:` `active:`\n`disabled:`, ancestor propagation `group-hover:` (mark the ancestor with bare\n`group`), conditions `motion-reduce:` `motion-safe:` `forced-colors:` `dark:`\n`light:`. ALLOWED values per prop:\n"
        );
        for (prop, values) in sail_lint::rules::sx::vocabulary() {
            let _ = writeln!(md, "- `{prop}-` {}", values.join(", "));
        }
        let _ = writeln!(md);

        // THE VALUE SPACE — the scales themselves, inline. The agent's prior
        // gets the property right and the VALUE wrong, every time, on every
        // colour and size. Putting the value space in context converts the
        // biggest failure mode from a round-trip into a non-event. Sourced
        // from the same token tables the lint reads, so they cannot drift.
        let _ = writeln!(md, "## Token scales — spend one, never a raw value\n");
        let _ = writeln!(
            md,
            "These are the closed value sets the scales above draw from. A\nscale-valued utility spends a token from one of these — never a raw number,\ncolour, or unit.\n"
        );
        for scale in &["space", "size", "color", "radius", "text", "weight", "leading", "tracking", "shadow", "z", "duration", "ease", "border-width", "opacity", "angle", "font"] {
            let values = sail_lint::scales::values(scale);
            if !values.is_empty() {
                let _ = writeln!(md, "- `{scale}`: {}", values.join(", "));
            }
        }
        let _ = writeln!(md);
    } else {
        let _ = writeln!(
            md,
            "## sx is NOT available here\n\nThis is an application-tier directory: compose components and drive them\nwith props and tokens. There is no styling attribute to reach for.\n"
        );
    }
    if effective.variants {
        // This section used to teach a top-level `variants: { … }` field in the
        // component block. That field is RETIRED — the compiler rejects it outright
        // ("unknown `<component>` field `variants`") — so the skill was instructing
        // agents to write code that cannot compile. It was hardcoded prose rather
        // than generated from a table, which is exactly how it rotted while the rest
        // of this file stayed honest.
        let _ = writeln!(md, "## variants — a prop whose values are sx tables\n");
        let _ = writeln!(
            md,
            "A variant IS A PROP. Declare it inside `props`, with the value being the\nvariant->sx table; the prop's type is the union of the keys (first = default),\nand it reflects to an attribute so CSS can key on it:\n\n```\n<component>{{\n  props: {{\n    variant: {{ solid: 'bg-tone text-tone-fg', soft: 'bg-tone-bg', outline: 'border-tone' }},\n    tone: 'default' | 'primary' | 'destructive',\n  }},\n}}</component>\n```\n\nValues are sx lists and take the same state prefixes\n(`solid: 'bg-tone hover:bg-tone-bg-hover'`). `compound` (a sibling field of `props`)\nkeys rules on COMBINATIONS of variant values.\n\nThere is no separate `variants:` block. One declaration surface: `props`.\n"
        );
    }
    if !effective.raw_containers {
        let _ = writeln!(
            md,
            "## Raw containers are banned\n\nNo bare `<div>`/`<span>` — use the layout elements or a component.\n"
        );
    }
    // Motion is ONE system. The triggers, properties and presets below are read from
    // the same tables the compiler enforces and the runtime dispatches on — not
    // retyped here, because the last time a motion vocabulary was retyped by hand the
    // copies drifted and a valid key failed the lint.
    use sail_lint::rules::animate_token_only::{
        MOTION_PROPERTIES, MOTION_STAGGER_KEY, MOTION_TRIGGERS,
    };
    let _ = writeln!(md, "## Motion — one system\n");
    let _ = writeln!(
        md,
        "`motion:<trigger>={{{{ <property>: '<preset>' }}}}`. There is exactly one animation\ndirective; `transition:` / `in:` / `out:` are retired.\n"
    );
    let _ = writeln!(
        md,
        "TRIGGERS ({}) come in three kinds:\n\n\
         - INTERACTION — `hover` `press` `focus` `focus-visible`: rest <-> active, driven by events.\n\
         - LIFECYCLE — `enter` `exit`: the element joining or leaving the DOM. `exit` is the one\n  \
           thing CSS cannot do alone (the framework holds the element in the tree until it finishes).\n\
         - TIMELINE — `reveal` `scroll`: PROGRESS-driven, not time-driven. Reveal-on-scroll and\n  \
           parallax, at zero JS runtime cost. `reveal` = this element's passage through the viewport;\n  \
           `scroll` = the scroll container's total progress.\n",
        MOTION_TRIGGERS.join(", ")
    );
    let _ = writeln!(
        md,
        "PROPERTIES: {}. Plus the reserved key `{}` — a DURATION (`'40ms'`), not a preset:\nit is the delay STEP between siblings arriving together, which is what turns a list\nthat appears into a list that arrives.\n",
        MOTION_PROPERTIES.join(", "),
        MOTION_STAGGER_KEY
    );
    let _ = writeln!(
        md,
        "```\n<card motion:reveal={{{{ opacity: 'fade', translate-y: 'spring:gentle' }}}}>reveal on scroll</card>\n<row motion:enter={{{{ opacity: 'fade', stagger: '40ms' }}}}>cascades in</row>\n<button motion:press={{{{ scale: 'press' }}}}>tactile</button>\n```\n"
    );
    let _ = writeln!(
        md,
        "PRESETS are NAMED DEFAULTS — the teachable starting point, not a cage.\nAn unknown preset NAME still fails the build, but you are never stuck waiting\nfor one to be minted:\n"
    );
    if let Ok(db) = sail_compiler::lower::presets::load() {
        let mut names = db.known_preset_names();
        names.sort_unstable();
        let _ = writeln!(md, "{}\n", names.join(", "));
    }
    // The calibration, stated where an author will actually read it. sx is closed
    // because colour and spacing are theming integrity. `motion:` is the animation
    // library's surface and numbers are native there — the alternative (a global
    // `--duration-cinematic` token minted so ONE hero loop could say 3600ms) is
    // the mistake this paragraph exists to prevent recurring.
    let _ = writeln!(
        md,
        "INLINE VALUES — full control on any config. Unlike sx (closed: colour and\nspacing are theming integrity), `motion:` takes real values, because a duration\nis a number of milliseconds and an easing is a curve.\n\n\
         ONE RULE: **quoted is a literal, unquoted is code.**\n\n\
         - `duration: 3600` / `delay: 200` — a bare number is MILLISECONDS.\n\
         - `easing: [0.6, 0, 0.2, 1]` — an inline cubic-bezier. Or a keyword:\n  \
           `easing: 'in-out'` / `'out'` / `'in'` / `'linear'`.\n\
         - `translate-x: {{ from: 0, to: '120px' }}` — inline magnitudes per channel. A\n  \
           bare number is the channel's native unit (px translate, deg rotate,\n  \
           unitless scale/opacity); a quoted string carries its own (`'-12%'`).\n\
         - `scale: {{ preset: 'pulse', to: 1.9 }}` — start from a preset, override a field.\n\
         - Timing sits at the config level or on one channel; the channel wins.\n\
         - SIGNALS: any field may be an expression — `duration: speed()`,\n  \
           `translate-x: {{ to: dist() }}`. A change re-creates the animation with the\n  \
           new values, and it is cleaned up on dispose.\n"
    );
    let _ = writeln!(
        md,
        "```\n<card motion:loop={{{{ translate-x: {{ from: 0, to: '120px' }}, rotate: {{ to: '15deg' }},\n                     duration: 3600, easing: [0.6, 0, 0.2, 1] }}}}>the whole hero, inline</card>\n```\n"
    );
    // The second motion lane. `motion:` above is the whole story for ambient
    // animation; it is NOT the story for animation whose clock something else
    // holds, and an agent that only knows `motion:` hand-rolls Math.min/max
    // offset arithmetic in an effect the moment it meets a scrub or a
    // multi-stop sequence. Teach both lanes and which one a task belongs in.
    let _ = writeln!(md, "## Motion — the second lane: `timeline()`\n");
    let _ = writeln!(
        md,
        "`motion:` above is FIRE-AND-FORGET: you name a preset, the browser owns the\n\
         clock, the compositor runs it. Reach for `timeline()` when something ELSE owns\n\
         the clock — a scrub, a scroll position, a sequence that must land on exact\n\
         offsets and HOLD there.\n\n\
         | you want | lane |\n\
         | --- | --- |\n\
         | hover/press feedback, enter/exit, reveal-on-scroll, an ambient loop | `motion:` |\n\
         | a slider scrubbing an animation | `timeline()` |\n\
         | many elements choreographed against ONE progress value | `timeline()` |\n\
         | keyframes with holds, or per-segment easing | `timeline()` |\n\n\
         `motion:` is cheaper (compositor, off the main thread) — prefer it whenever it\n\
         can express the motion. `timeline()` writes custom properties from JS every\n\
         frame; that is fine at real page scale (60 elements x 3 channels holds 60fps)\n\
         but it is main-thread work `motion:` does not do.\n"
    );
    let _ = writeln!(
        md,
        "KEYFRAMES ARE DATA. `timeline(progress, tracks)` takes a progress reader (0..1)\n\
         and returns ONE SIGNAL PER TRACK. Destructure them into the returned scope and\n\
         each becomes a `(--name)` an sx read can use — the same reflection a plain\n\
         `signal(...)` gets. Never hand-roll offset arithmetic in an effect.\n\n\
         - a stop is `[offset, value]`; offsets are 0..1 and must ASCEND.\n\
         - a HOLD is the same VALUE repeated at a later offset. (Repeating an OFFSET is\n  \
           an error.)\n\
         - `[offset, value, {{ easing: [0.6, 0, 0.2, 1] }}]` eases the segment that\n  \
           STARTS at that stop — WAAPI's rule, so captured `cubic-bezier(...)` values\n  \
           transfer verbatim. `{{ easing }}` on the whole timeline sets the default.\n\
         - a track is all numbers OR all strings sharing ONE unit. Mixed units throw.\n\
         - progress outside the first/last offset CLAMPS.\n"
    );
    let _ = writeln!(
        md,
        "```\n\
         <controller>\n\
         import {{ controller, timeline, clock }} from '@ship/sail-runtime'\n\
         export default controller(() => {{\n  \
           const p = clock(3600, {{ loop: true }})\n  \
           const {{ tileX, tileScale }} = timeline(p, {{\n    \
             tileX: [[0, '0px'], [0.12, '0px'], [0.34, '120px'], [0.44, '120px'], [1, '0px']],\n    \
             tileScale: [[0.44, 1], [0.54, 1.75], [0.64, 1]],\n  \
           }}, {{ easing: [0.6, 0, 0.2, 1] }})\n  \
           return {{ p, tileX, tileScale }}\n\
         }})\n\
         </controller>\n\
         <card sx=\"translate-x-(--tile-x) scale-(--tile-scale)\">choreographed</card>\n\
         ```\n"
    );
    let _ = writeln!(
        md,
        "DRIVERS — each returns a plain signal, so they are interchangeable with a\n\
         hand-written `signal(0)` a range input writes:\n\n\
         - `clock(ms, {{ loop?, direction?, autoplay?, reducedMotion? }})` — wall-clock\n  \
           progress, plus `.play()` / `.pause()` / `.stop()` / `.running()`.\n\
         - `scrollProgress(target?)` — document scroll, or an element's travel through\n  \
           the viewport (0 as it enters, 1 as it leaves).\n\
         - `springValue(source, opts)` — real physics chasing another signal;\n  \
           retargets mid-flight without losing velocity. Its output is a VALUE, not\n  \
           necessarily 0..1 — reflect it directly (`translate-y-(--ball-y)`).\n\n\
         Every driver stops itself when the scope disposes. Reduced motion is handled\n\
         once, in the driver: `clock` jumps to the END and never schedules a frame\n\
         (override with `reducedMotion: 'start' | 'play'`), `springValue` mirrors its\n\
         source instantly, and `scrollProgress` deliberately does NOT suppress — it\n\
         reports what the user is already doing.\n"
    );

    // The three things an author reaches for that the skill never mentioned: how
    // behavior attaches, where state comes from, and which door to import through.
    let _ = writeln!(md, "## Behavior — `use:` attaches, functions compose\n");
    let _ = writeln!(
        md,
        "ONE RULE: **if it touches the DOM it is an action; if it returns state it is a\nfunction.** \"use\" means exactly one thing in this language — the `use:` directive.\n\n\
         - `use:action` (template) — behavior bound to an element. `Action<P>`: takes the node\n  \
           (+ an optional reactive param), returns `{{ update?, destroy? }}`. It replaces the\n  \
           ref + effect + cleanup triad entirely. Yours work exactly like the built-ins:\n  \
           `import {{ myAction }} from './my-action'` then `use:myAction={{param}}`.\n\
         - COMPOSITIONS (`@ship/sail-std`) — plain functions that combine the reactive\n  \
           primitives and return signals. `form()`, `selection()`, `filteredList()`. There is\n  \
           no type, no registration, no lifecycle contract — nothing to author against, which\n  \
           is why anyone can write one. Call it in the `<controller>`, and the signals it\n  \
           returns are ordinary scope members the template binds to.\n"
    );
    let _ = writeln!(md, "## State + imports\n");
    let _ = writeln!(
        md,
        "State lives in the `<controller>` — a raw `<script>` is banned, and the `<state>`\ndirective is retired. `controller(fn)` is ONE function, no sections: whatever it\nRETURNS is the template's scope — state signals, derived values, plain helpers, and\nevent handlers alike, all as ordinary object properties.\n\n\
         ```\n\
         <controller>\n  \
           import {{ controller, signal, derived }} from '@ship/sail-runtime'\n  \
           import {{ selection }} from '@ship/sail-std'\n\n  \
           export default controller((ctx) => {{\n    \
             const rows = signal([])\n    \
             const sel = selection(rows, (r) => r.id)\n    \
             return {{ rows, sel }}\n  \
           }})\n\
         </controller>\n\
         ```\n\n\
         Mutation is ALWAYS `signal.set(newValue)` (or `signal.set(prev => next)`). Lifecycle\nis ambient, not a section — call `mount(fn)` (fires once after this instance's DOM\nconnects; a returned function is cleanup run on dispose) or `onCleanup(fn)` (runs on\ndispose) from anywhere in the setup call graph, including a composition function\ncalled from setup. Both are auto-injected — no import needed.\n\n\
         Import primitives from `@ship/sail-runtime` — the front door. Never from\n`@ship/sail-runtime/internal`: that is the COMPILER's ABI (`bindFor`,\n`runAction`, …). If you are typing a symbol from there, you are hand-writing something\nthe compiler emits.\n\n\
         Reactivity: `signal` `derived` `effect` `batch` `untrack` `shared`. Reach for\n`effect` rarely — it is the escape hatch, not the pattern. Cleanup is automatic: an\neffect created during setup dies with the component.\n\n\
         Control flow: `{{#if cond}}…{{/if}}`, `{{#for item of items key={{item.id}}}}…{{/for}}`\n(the `key=` is required — without it the reconciler falls back to positional matching\nand loses DOM identity on reorder).\n"
    );
    let _ = writeln!(md, "## Runtime CSS — the `(--x)` binding\n");
    let _ = writeln!(
        md,
        "ONE RULE: **scope owns props, the controller owns state** — there is no state block\nin scope; state is code, not a declaration table. `style:` is gone; a `style:` use\nrefuses with a diagnostic that teaches this.\n\n\
         `sx` reads runtime values through `(--name)` (camelCase → kebab, e.g. a prop or\nsignal named `rowY` reads as `(--row-y)`): `translate-y-(--row-y)`, `bg-(--hue)`. When\n`(--name)` matches a declared prop or a value `setup()` returns via `signal(...)`, the\ncompiler registers a typed CSS `@property` (a `number` gets `syntax: '<number>'`, so a\nCSS transition on it tweens natively, no JS) and reflects the value onto the host as an\ninline custom property, kept live automatically — no action, no manual `effect`. A name\nnever read via `(--x)` gets no wiring; an unmatched name is a compile error with a\ndid-you-mean.\n\n\
         Runtime GEOMETRY (a virtualizer offset, a drag position — not a prop, not scope\nstate) stays on `use:` actions: a named, typed action\n(`use:virtualRow`, `use:eventSpan`, `use:rect`, `use:zoomPan`, `use:swatch`, …) writes\nits own fixed custom property; sx reads it the same way, `(--name)`. `el.style` stays\nclosed — actions gain no styling powers beyond the one property they own.\n"
    );
    // L1 ELEMENTS — the language's first-class surface. Listed
    // verbatim from the same manifest the consistency test enforces
    // (crates/sail/compiler/src/l1_manifest.rs).
    let (stdlib_entries, reskin_entries) = sail_compiler::l1_by_kind();
    let _ = writeln!(
        md,
        "## L1 elements — the language surface ({stdlib} stdlib + {reskin} reskins = {total} total)\n",
        stdlib = stdlib_entries.len(),
        reskin = reskin_entries.len(),
        total = stdlib_entries.len() + reskin_entries.len()
    );
    let _ = writeln!(
        md,
        "These are the elements the language gives you. **The bare tag is\n\
         the API** — a `<button>` and a `<card>` work the same way:\n\
         tag + props (tone/size/etc.) + slotted children. No CSS, no\n\
         <style>, no class soup. The list below is generated from the\n\
         compiler's L1 manifest; if the count drifts, the consistency\n\
         test in `crates/sail/compiler/tests/l1_manifest.rs` fails and\n\
         this file is out of date.\n"
    );
    let _ = writeln!(
        md,
        "### Stdlib elements (compiled `.sail`, lower at compile time)\n"
    );
    for entry in &stdlib_entries {
        let props = if entry.attributes.is_empty() {
            "no props".to_string()
        } else {
            entry
                .attributes
                .iter()
                .map(|a| format!("`{}`", a.name))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let _ = writeln!(
            md,
            "- `<{tag}>` — {desc} (props: {props})\n",
            tag = entry.tag,
            desc = entry.description,
            props = props
        );
    }
    let _ = writeln!(md);
    let _ = writeln!(
        md,
        "### Native reskins (bare HTML tags the framework styles)\n"
    );
    for entry in &reskin_entries {
        let attrs = if entry.attributes.is_empty() {
            "presence / value via attribute".to_string()
        } else {
            entry
                .attributes
                .iter()
                .map(|a| {
                    if a.values.is_empty() {
                        format!("`{}`", a.name)
                    } else {
                        format!("`{}={}`", a.name, a.values.join("|"))
                    }
                })
                .collect::<Vec<_>>()
                .join(", ")
        };
        let _ = writeln!(
            md,
            "- `<{tag}>` — {desc} ({attrs})\n",
            tag = entry.tag,
            desc = entry.description,
            attrs = attrs
        );
    }
    let _ = writeln!(md);
    let _ = writeln!(md, "## When the compiler rejects you\n");
    let _ = writeln!(
        md,
        "The rejection IS the documentation: every sx/lint error names the allowed\nvalues. Do not work around an error with a `<style>` escape or an inline\nstyle — if the vocabulary genuinely cannot express the design, STOP and\nreport it as a language gap.\n"
    );

    let out_path = out.unwrap_or_else(|| dir.join(".claude/skills/sail-authoring/SKILL.md"));
    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&out_path, md)?;
    println!("skill written: {} (policy: {policy_line})", out_path.display());
    Ok(())
}


/// Walk `packages/components/` (or the supplied directory), find every
/// `sail-*.sail` source, and classify it against the L1 manifest. The
/// output is the evidence the spec asks for: every component in the
/// codebase that re-implements an L1 element. Derived from the same
/// manifest the consistency test enforces, so the report cannot drift
/// from the surface.
///
/// Heuristics:
/// - The component's tag is the directory name (`sail-xxx/`
///   -> `sail-xxx`), minus the `sail-` prefix.
/// - "Duplicate" = the resulting tag is a stdlib element OR matches
///   a native reskin's domain (e.g. `dialog`, `select`).
/// - "Novel" = the tag has no L1 equivalent in the manifest.
/// - "Orphan" = the directory has a `.controller.ts` but no
///   `.sail` source, a residue from the L1 cutover.
fn gen_duplicates_report(dir: Option<PathBuf>, out: Option<PathBuf>) -> Result<()> {
    use std::fmt::Write as _;
    let dir = match dir {
        Some(d) => d,
        None => workspace_root()?.join("packages/components"),
    };
    if !dir.is_dir() {
        return Err(anyhow!(
            "duplicates: directory not found: {}",
            dir.display()
        ));
    }
    // Build a fast lookup: tag-stem (e.g. "button") -> manifest entry.
    let entries = sail_compiler::l1_entries();
    let mut by_tag: std::collections::HashMap<String, &sail_compiler::L1Entry> =
        std::collections::HashMap::new();
    for e in &entries {
        // For attribute-keyed reskins like `input[type=checkbox]`, the
        // "domain" stem is the part before `[`. Use it so the report
        // groups by parent tag.
        let stem = e.tag.split('[').next().unwrap_or(&e.tag);
        by_tag.entry(stem.to_string()).or_insert(e);
    }
    // Walk every `sail-*/` directory.
    let mut rows: Vec<(String, String, String, String)> = Vec::new();
    fn walk_components(
        dir: &std::path::Path,
        max_depth: usize,
        out: &mut Vec<(std::path::PathBuf, usize)>,
        depth: usize,
    ) {
        if depth > max_depth {
            return;
        }
        let Ok(rd) = std::fs::read_dir(dir) else { return };
        for entry in rd.filter_map(Result::ok) {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let name = entry.file_name().to_str().unwrap_or("").to_string();
            if name.starts_with("sail-") {
                out.push((path.clone(), depth));
            }
            walk_components(&path, max_depth, out, depth + 1);
        }
    }
    let mut component_dirs: Vec<(std::path::PathBuf, usize)> = Vec::new();
    walk_components(&dir, 5, &mut component_dirs, 0);
    component_dirs.sort_by(|a, b| a.0.cmp(&b.0));
    for (dir_path, depth) in &component_dirs {
        let dir_name = dir_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        // Find the .sail source(s) in the directory.
        let sui_sources: Vec<PathBuf> = std::fs::read_dir(dir_path)
            .ok()
            .map(|it| {
                it.filter_map(Result::ok)
                    .filter(|e| {
                        e.path()
                            .extension()
                            .and_then(|s| s.to_str())
                            .map(|s| s == "sail")
                            .unwrap_or(false)
                    })
                    .map(|e| e.path())
                    .collect()
            })
            .unwrap_or_default();
        let has_controller = std::fs::read_dir(dir_path)
            .ok()
            .map(|it| {
                it.filter_map(Result::ok)
                    .any(|e| {
                        e.path()
                            .extension()
                            .and_then(|s| s.to_str())
                            .map(|s| s == "ts")
                            .unwrap_or(false)
                    })
            })
            .unwrap_or(false);
        // Strip `sail-` prefix; the report's tag column is the bare stem.
        let stem = dir_name.strip_prefix("sail-").unwrap_or(&dir_name);
        let has_sui = !sui_sources.is_empty();
        let status: String = if !has_sui && has_controller {
            "orphan".to_string()
        } else if has_sui {
            match by_tag.get(stem) {
                Some(e) => {
                    let kind = match e.kind {
                        sail_compiler::L1Kind::StdlibElement => "stdlib",
                        sail_compiler::L1Kind::NativeReskin => "reskin",
                    };
                    format!("duplicate (L1 {kind} <{tag}>)", tag = e.tag)
                }
                None => "novel".to_string(),
            }
        } else {
            "novel".to_string()
        };
        let rel = dir_path
            .strip_prefix(&dir)
            .unwrap_or(dir_path)
            .display()
            .to_string();
        rows.push((dir_name, status, rel, depth.to_string()));
    }
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    let mut out_str = String::new();
    let _ = writeln!(
        out_str,
        "# L1 duplicate report, every `sail-*.sail` component\n"
    );
    let _ = writeln!(
        out_str,
        "Generated by `cargo run -p sail-compiler -- duplicates` from the\n\
         L1 manifest (`crates/sail/compiler/src/l1_manifest.rs`). The\n\
         **duplicate** count is the answer to the spec's question,\n\
         *which components re-implement an L1 element*, and is the\n\
         evidence the next cutover wave acts on.\n"
    );
    let duplicates: Vec<_> = rows
        .iter()
        .filter(|r| r.1.starts_with("duplicate"))
        .collect();
    let orphans: Vec<_> = rows.iter().filter(|r| r.1 == "orphan").collect();
    let novels: Vec<_> = rows.iter().filter(|r| r.1 == "novel").collect();
    let _ = writeln!(
        out_str,
        "**Tally:** {duplicates} duplicates of L1 / {orphans} orphans / {novels} novel / {total} total\n",
        duplicates = duplicates.len(),
        orphans = orphans.len(),
        novels = novels.len(),
        total = rows.len(),
    );
    let _ = writeln!(
        out_str,
        "| Component | Classification | Path | Depth |\n\
         | --- | --- | --- | --- |"
    );
    for (name, status, rel, depth) in &rows {
        let _ = writeln!(out_str, "| `<{name}>` | {status} | `{rel}` | {depth} |");
    }
    let _ = writeln!(
        out_str,
        "\n## What this is and isn't\n\n\
         * **Duplicate** components re-implement an L1 element. The next\n\
         cutover wave deletes them (this job does NOT touch them).\n\
         * **Orphan** components have a controller but no source, an L1\n\
         cutover residue. The existing `PRIMITIVES-DISSOLUTION.md` covers\n\
         them by hand; this report is the mechanical cross-check.\n\
         * **Novel** components have no L1 equivalent; they earn their\n\
         keep by domain (charts, canvas, code editor, ...).\n"
    );
    match out {
        Some(path) => {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&path, &out_str)?;
            println!("duplicates report written: {}", path.display());
        }
        None => {
            print!("{out_str}");
        }
    }
    Ok(())
}

fn synthesize(
    input: PathBuf,
    manifest_path: PathBuf,
    out_dir: PathBuf,
) -> Result<()> {
    use sail_compiler::emit::synthetic::{emit_synthetic, AppManifest, AuthoredController};
    use sail_compiler::parse;

    let source = fs::read_to_string(&input)
        .with_context(|| format!("read {}", input.display()))?;
    let manifest_src = fs::read_to_string(&manifest_path)
        .with_context(|| format!("read manifest {}", manifest_path.display()))?;
    // SFC: strip the inline `<controller>` block before parse (mirrors
    // `compile()`), so its body isn't misread as a template expression.
    let (stripped, inline_controller_src) = parse::preprocess::extract_controller_block(&source);
    let program =
        parse::parse(&stripped).map_err(|e| anyhow!("{}", e.render(&stripped)))?;
    let manifest =
        AppManifest::parse(&manifest_src).map_err(|e| anyhow!("{}", e.render(&manifest_src)))?;

    let file = input.file_name().and_then(|f| f.to_str()).unwrap_or("view.sail");
    let raw_stem = file.strip_suffix(".sail").unwrap_or(file);
    // Every `.sail` synthesizes to a component; the retired `page.sail` route
    // path is gone, so the mode is always `Component`.
    let mode = CompileMode::Component;

    let stem = {
        let s = raw_stem.replace(['/', '\\'], "_");
        if s.is_empty() { "view".to_string() } else { s }
    };
    // Compose the controller. SFC: an inline `<controller>` block is
    // materialized to `<stem>.controller.ts` (wrapped into the canonical
    // `controller({…})` form) and composed by `emit_synthetic`, exactly as a
    // paired file would be — so the synthetic controller + render pair is
    // tsc-checkable for the SFC model too. Otherwise fall back to a sibling
    // `controller.ts` on disk (route modules / not-yet-migrated).
    fs::create_dir_all(&out_dir)
        .with_context(|| format!("create out dir {}", out_dir.display()))?;
    let authored = if let Some(body) = &inline_controller_src {
        // Lower `query()`/`mutation()`/`invalidate()` before the ambient-import
        // wrap — see `check::query_keys` module header. A refusal here is a
        // real compile error, same as any other structural check.
        let rewritten = sail_compiler::check::query_keys::rewrite(body)
            .map_err(|e| anyhow!("{}", e.render(body)))?;
        let wrapped = parse::preprocess::wrap_controller_body(&rewritten);
        let ctrl_name = format!("{stem}.controller.ts");
        let ctrl_path = out_dir.join(&ctrl_name);
        fs::write(&ctrl_path, wrapped)
            .with_context(|| format!("write {}", ctrl_path.display()))?;
        Some(AuthoredController { specifier: format!("./{ctrl_name}") })
    } else {
        sibling_controller(&input, &out_dir, raw_stem)
    };
    let controller_name = format!("{stem}.synthetic.controller.ts");
    let render_name = format!("{stem}.render.ts");
    let client_name = format!("{stem}.client.ts");

    let out = emit_synthetic(
        &program,
        &manifest,
        authored.as_ref(),
        &format!("./{controller_name}"),
        mode,
        None,
    )
    .map_err(|e| anyhow!("{}", e.render(&source)))?;

    fs::create_dir_all(&out_dir)
        .with_context(|| format!("create out dir {}", out_dir.display()))?;
    let controller_path = out_dir.join(&controller_name);
    let render_path = out_dir.join(&render_name);
    let client_path = out_dir.join(&client_name);
    fs::write(&controller_path, &out.controller_ts)
        .with_context(|| format!("write {}", controller_path.display()))?;
    fs::write(&render_path, &out.render_ts)
        .with_context(|| format!("write {}", render_path.display()))?;
    fs::write(&client_path, &out.render_client_ts)
        .with_context(|| format!("write {}", client_path.display()))?;
    println!(
        "synthesized {} ({:?}) → {}, {}, {} (clients: {}{})",
        input.display(),
        mode,
        controller_path.display(),
        render_path.display(),
        client_path.display(),
        if out.used_clients.is_empty() { "none".to_string() } else { out.used_clients.join(", ") },
        if authored.is_some() { "; composed authored controller.ts" } else { "" },
    );
    Ok(())
}

/// A sibling `controller.ts` (for `page.sail`) or `<stem>.controller.ts`,
/// composed by the synthetic controller via a relative import from `out_dir`.
fn sibling_controller(
    input: &Path,
    out_dir: &Path,
    stem: &str,
) -> Option<sail_compiler::emit::synthetic::AuthoredController> {
    use sail_compiler::emit::synthetic::AuthoredController;
    let parent = input.parent()?;
    let candidate = if stem == "page" {
        parent.join("controller.ts")
    } else {
        parent.join(format!("{stem}.controller.ts"))
    };
    if !candidate.exists() {
        return None;
    }
    Some(AuthoredController { specifier: rel_specifier(&candidate, out_dir) })
}

/// POSIX relative-path specifier from `out_dir` to `target` (both made
/// absolute against CWD), suitable as an ESM import string.
fn rel_specifier(target: &Path, out_dir: &Path) -> String {
    let cwd = std::env::current_dir().unwrap_or_default();
    let abs = |p: &Path| if p.is_absolute() { p.to_path_buf() } else { cwd.join(p) };
    let (target, out_dir) = (abs(target), abs(out_dir));
    let from: Vec<_> = out_dir.components().collect();
    let to: Vec<_> = target.components().collect();
    let common = from.iter().zip(to.iter()).take_while(|(a, b)| a == b).count();
    let mut out = String::new();
    for _ in 0..(from.len() - common) {
        out.push_str("../");
    }
    let rest: Vec<String> =
        to[common..].iter().map(|c| c.as_os_str().to_string_lossy().into_owned()).collect();
    out.push_str(&rest.join("/"));
    if !out.starts_with('.') {
        out = format!("./{out}");
    }
    out
}

fn build(
    input: PathBuf,
    out: Option<PathBuf>,
    clean: bool,
    mode: Option<ModeArg>,
    no_dts: bool,
    component_name: Option<String>,
    emit_component_imports: bool,
) -> Result<()> {
    let mut targets = Vec::new();
    if input.is_dir() {
        collect_targets(&input, &mut targets)?;
    } else if input
        .file_name()
        .and_then(|f| f.to_str())
        .is_some_and(is_sail_source)
    {
        targets.push(input.clone());
    } else {
        return Err(anyhow!("not a .sail file or directory: {}", input.display()));
    }
    if targets.is_empty() {
        return Err(anyhow!("no .sail files found under {}", input.display()));
    }

    if clean {
        if let Some(out_dir) = out.as_deref() {
            // Single --out dir mode: clean it once before emit.
            if out_dir.exists() {
                fs::remove_dir_all(out_dir)
                    .with_context(|| format!("clean {}", out_dir.display()))?;
            }
        } else {
            // Mirror-tree mode: clean each unique target mirror directory.
            let mut cleaned: HashSet<PathBuf> = HashSet::new();
            for path in &targets {
                let mirror = resolve_out_dir(path);
                if cleaned.insert(mirror.clone()) && mirror.exists() {
                    fs::remove_dir_all(&mirror)
                        .with_context(|| format!("clean {}", mirror.display()))?;
                }
            }
        }
    }

    if let Some(out_dir) = out.as_deref() {
        fs::create_dir_all(out_dir)
            .with_context(|| format!("create out dir {}", out_dir.display()))?;
    }

    // Component mode: tag name comes from --component-name when supplied,
    // otherwise defaults to the filename stem at compile-one time. Shadow
    // is always on for components — no opt-out flag.
    //
    // Guardrail (#176): when an input matches the `<tag>/<tag>.sail` component
    // convention AND no explicit mode/component-name was given, refuse rather
    // than emit component JS with no tag. Without a tag the output never calls
    // `customElements.define` — the bundle succeeds, the browser loads it, and
    // nothing registers. Surface the ambiguity at compile time instead of
    // debugging it at runtime.
    let count = targets.len();
    for path in targets {
        let explicit = mode.is_some() || component_name.is_some();
        if !explicit && let Some(tag) = looks_like_component_path(&path) {
            return Err(anyhow!(
                "refusing to compile {} without a component tag\n\
                 \n\
                 The path matches the `<tag>/<tag>.sail` component convention\n\
                 (tag = `{tag}`). Compiling it without a tag emits component\n\
                 JS that never calls `customElements.define` — the browser\n\
                 loads the bundle but nothing registers.\n\
                 \n\
                 Pass one of these to confirm what you mean:\n  \
                 --component-name {tag}    (register under this exact tag)\n  \
                 --mode component          (register under the filename stem)",
                path.display(),
            ));
        }
        // ELEMENT MODE dispatches to its own pipeline: structure + skin +
        // manifest artifacts, no component module.
        if matches!(mode, Some(ModeArg::Element)) {
            let name = component_name.clone().unwrap_or_else(|| stem_of(&path));
            let source = std::fs::read_to_string(&path)?;
            let el = sail_compiler::compile_element(&source, &name)
                .map_err(|e| anyhow!("{}: {e}", path.display()))?;
            let out_dir = out.clone().unwrap_or_else(|| {
                path.parent().map(|p| p.to_path_buf()).unwrap_or_default()
            });
            std::fs::create_dir_all(&out_dir)?;
            let artifact = serde_json::json!({
                "tag": name,
                "rootClass": el.root_class,
                "structure": el.structure_html,
                "skin": el.skin_css,
                "manifest": el.manifest,
            });
            std::fs::write(
                out_dir.join(format!("{name}.element.json")),
                serde_json::to_string_pretty(&artifact)?,
            )?;
            std::fs::write(out_dir.join(format!("{name}.skin.css")), &el.skin_css)?;
            continue;
        }
        let component_meta = match (mode.clone(), component_name.clone()) {
            (Some(ModeArg::Component), name) => Some(ComponentMeta {
                name: name.unwrap_or_else(|| stem_of(&path)),
            }),
            (_, Some(name)) => Some(ComponentMeta { name }),
            _ => None,
        };
        compile_one(&path, out.as_deref(), mode.clone(), no_dts, component_meta, emit_component_imports)?;
    }
    println!("compiled {count} file(s)");
    Ok(())
}

fn stem_of(path: &Path) -> String {
    path.file_name()
        .and_then(|f| f.to_str())
        .map(|s| s.strip_suffix(".sail").unwrap_or(s).to_string())
        .unwrap_or_default()
}

fn is_sail_source(name: &str) -> bool {
    name.ends_with(".sail")
}

/// `<tag>/<tag>.sail` — the convention every SAIL component follows. The
/// directory name IS the component contract; if it matches the file stem,
/// this is a component source. Returns the tag on match.
///
/// Pre-existing footgun (#176): running `sail-compiler build` directly on a
/// component path without `--component-name` (or `--mode component`) emits
/// component JS with no tag. The bundle succeeds, the browser loads it,
/// nothing happens — no custom element registers, because no
/// `customElements.define()` call was emitted.
fn looks_like_component_path(path: &Path) -> Option<String> {
    let stem = path.file_stem().and_then(|s| s.to_str())?;
    let parent = path.parent()?.file_name().and_then(|s| s.to_str())?;
    if stem == parent { Some(stem.to_string()) } else { None }
}

fn collect_targets(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let p = entry.path();
        if p.is_dir() {
            collect_targets(&p, out)?;
        } else if p
            .file_name()
            .and_then(|f| f.to_str())
            .is_some_and(is_sail_source)
        {
            out.push(p);
        }
    }
    Ok(())
}

/// The scope capabilities for `path`, from the nearest `app.manifest.jsonc` at
/// or above its directory — the same opt-in the synthetic client path uses, and
/// the same walk-up as the `sail.config.json` policy lookup. Each declared
/// client becomes a typed member of the generated `Scope`, so a controller
/// reads it precisely and an undeclared name is a type error. No manifest → no
/// capabilities → `Scope` is exactly the template's own declarations, as before.
///
/// A host that injects context beyond the app's own manifest (deck's
/// `route`/`navigate`) merges its grant in on its own build path; this CLI
/// names no host.
fn app_manifest_capabilities(path: &Path) -> Vec<sail_compiler::emit::dts::ScopeCapability> {
    let mut dir = path.parent();
    while let Some(d) = dir {
        let candidate = d.join("app.manifest.jsonc");
        if candidate.is_file() {
            return fs::read_to_string(&candidate)
                .ok()
                .and_then(|src| sail_compiler::emit::synthetic::AppManifest::parse(&src).ok())
                .map(|m| sail_compiler::emit::synthetic::scope_capabilities(&m))
                .unwrap_or_default();
        }
        dir = d.parent();
    }
    Vec::new()
}

fn compile_one(
    path: &Path,
    out_override: Option<&Path>,
    mode: Option<ModeArg>,
    no_dts: bool,
    component_meta: Option<ComponentMeta>,
    emit_component_imports: bool,
) -> Result<()> {
    // The ORIGINAL, unstripped source — kept so the synthetic-controller tsgo
    // gate below can re-extract the inline `<controller>` block (the stripped
    // `source` no longer has it).
    let orig_source = fs::read_to_string(path)
        .with_context(|| format!("read {}", path.display()))?;
    // SFC: the inline `<controller>` block IS the controller. Nothing is
    // materialized next to the source — a sibling `<stem>.controller.ts` is a
    // compile error (`sail-paired-controller`), so writing one would author the
    // very thing the compiler rejects.
    // `compile()` and `check_all()` are SFC-aware: they blank the `<controller>`
    // block themselves and parse it into the scope the template is checked
    // against. They must see the SOURCE — handing them a pre-stripped copy leaves
    // them with no block, and every name the controller provides reads as
    // undeclared. The extraction here decides one thing: whether there is a
    // controller to import.
    let inline_controller =
        sail_compiler::parse::preprocess::extract_controller_block(&orig_source).1;
    let mode = mode.map(|m| m.into()).unwrap_or(CompileMode::Auto);
    // Resolve the emit dir for this file. Without --out, outputs land in
    // the project mirror tree at `<repo-root>/.sail-out/<src-rel>/`;
    // with --out, all files share that directory (legacy single-dir mode).
    let mirror_mode = out_override.is_none();
    let out_dir: PathBuf = match out_override {
        Some(p) => p.to_path_buf(),
        None => resolve_out_dir(path),
    };
    fs::create_dir_all(&out_dir)
        .with_context(|| format!("create out dir {}", out_dir.display()))?;
    // The controller is the source's `<controller>` block. The emit imports it
    // as the VIRTUAL `<file>.sail.controller.ts` sibling — the same specifier
    // deck's bundler serves by extracting the block; nothing is on disk. A
    // template with no block gets no setup import.
    let setup_url = inline_controller.as_ref().map(|_| {
        let file = path
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("component.sail");
        if mirror_mode {
            // The mirror tree is far from the source: import by absolute path so
            // the bundler resolves the virtual module against the real source.
            format!("{}.controller.ts", path.display())
        } else {
            format!("./{file}.controller.ts")
        }
    });
    // Resolve the CAPABILITY policy from the nearest `sail.config.json` walking up
    // from this file. The FLOOR (no authored `<style>`/`style=`/`class`/undeclared
    // stylesheet) is NOT resolved here — the compiler enforces it unconditionally
    // for every `.sail` (`check::lint_overrides::enforce_floor`), so `overrides_for`
    // returns only the capability overrides declared above the floor. Capabilities
    // are opt-in per package: a package with no config declares none.
    //
    // A package configures capabilities with `sail.config.json`. The key the parser
    // reads is `"preset"`, values `full` | `strict` (`{"preset":"strict"}`), plus
    // optional per-key booleans `"sx"` / `"variants"` / `"raw_containers"` that
    // override the preset. `full` unlocks every capability (primitive authoring);
    // `strict` locks all three (app authoring — compose primitives, no raw CSS
    // surface). A present-but-unrecognized config fails CLOSED to `strict`, never
    // `full`. (The floor holds under every tier — it is not a key.)
    let opts = CompileOpts {
        route_params: None,
        setup_url,
        mode,
        source_path: Some(path.to_path_buf()),
        component_meta,
        emit_dts: !no_dts,
        scope_capabilities: app_manifest_capabilities(path),
        emit_component_imports,
        lint_overrides: Policy::overrides_for(path),
    };
    let result = compile(&orig_source, opts).map_err(|e| {
        let rendered = e.render(&orig_source);
        anyhow!("{rendered}")
    })?;

    fatal_diagnostics_gate(&orig_source, path)?;

    let stem = path
        .file_name()
        .and_then(|f| f.to_str())
        .ok_or_else(|| anyhow!("bad path"))?;
    let stem_no_ext = stem.strip_suffix(".sail").unwrap_or(stem);
    // In mirror mode the emitted JS is named `<stem>.compiled.js` so
    // imports can name it directly without a separate rename step in
    // the build pipeline. With --out we keep the legacy `<stem>.js`
    // for callers that expect that name.
    let js_name = if mirror_mode {
        format!("{stem_no_ext}.compiled.js")
    } else {
        format!("{stem_no_ext}.js")
    };
    let js_path = out_dir.join(&js_name);
    fs::write(&js_path, &result.js)
        .with_context(|| format!("write {}", js_path.display()))?;
    if let Some(cem) = result.cem {
        let cem_path = out_dir.join(format!("{stem_no_ext}.custom-elements.json"));
        let pretty = serde_json::to_string_pretty(&cem)?;
        fs::write(&cem_path, pretty)
            .with_context(|| format!("write {}", cem_path.display()))?;
    }
    if let Some(schema) = result.schema {
        let schema_path = out_dir.join(format!("{stem_no_ext}.schema.json"));
        let pretty = serde_json::to_string_pretty(&schema)?;
        fs::write(&schema_path, pretty)
            .with_context(|| format!("write {}", schema_path.display()))?;
    }
    // `.sail.meta.json` sidecar — binding info for LSP/IDE tooling. The
    // runtime never loads this; it's purely developer-facing fuel.
    if let Some(meta) = result.meta {
        let meta_path = out_dir.join(format!("{stem_no_ext}.sail.meta.json"));
        let pretty = serde_json::to_string_pretty(&meta)?;
        fs::write(&meta_path, pretty)
            .with_context(|| format!("write {}", meta_path.display()))?;
    }
    // `.sail.d.ts` is a TYPE companion to its source, so it is written
    // ADJACENT to the `.sail` (not into the build mirror like the JS/CEM/SSR
    // artifacts). That lets a controller's `import type { Scope } from
    // './<name>.sail'` resolve natively to the sibling `.d.ts` — no `rootDirs`
    // round-trip into `.ship/sail-out` in every consumer's tsconfig (which was
    // fragile and, when mispathed, silently masked type errors as `any`).
    // Gitignored via `*.sail.d.ts`.
    if let Some(dts) = result.dts {
        let dts_dir = path.parent().unwrap_or(out_dir.as_path());
        let dts_path = dts_dir.join(format!("{stem}.d.ts"));
        fs::write(&dts_path, dts)
            .with_context(|| format!("write {}", dts_path.display()))?;
    }
    Ok(())
}

/// Honest exit codes. `compile()` short-circuits on the first diagnostic its
/// `check::run` pass gates (expr-subset, scope-bound, css-token,
/// controller-purity, …) and returns `Err` — that path already exits non-zero.
/// But the full `sail-lint` stream carries Error-severity rules `check::run`
/// does NOT invoke (e.g. `sail-attr-bool-prop`, `sail-prop-value`,
/// `sail-css-literal`, `sail-css-content-text`, `controller-*`). Without this
/// gate those compile to `Ok`, the CLI prints "compiled N file(s)" and returns
/// 0 while the LSP/agent show a hard error — the dishonest-exit-code bug
/// (#255). Re-run the full diagnostic pass and treat any Error-severity finding
/// as fatal so batch OK-checks (`sail-compiler build <dir> && …`) mean what they


/// say.
/// `source` is the SFC as authored: `check_all` resolves the `<controller>`
/// block's scope from it, and reports spans in its coordinates.
fn fatal_diagnostics_gate(source: &str, path: &Path) -> Result<()> {
    let fatal: Vec<_> = check_all(source, Some(path), &[], None)
        .into_iter()
        .filter(|d| d.severity == DiagSeverity::Error)
        .collect();
    if fatal.is_empty() {
        return Ok(());
    }
    // `check_all` diagnostics are already self-describing — lint findings that
    // flow through this path carry their `[rule]` tag inside the message, and
    // structural check-pass errors carry a clear category. Render the message
    // verbatim with just the file/line/col prefix; adding a separate `[rule]`
    // tag here would double the one the message already embeds.
    let rendered = fatal
        .iter()
        .map(|d| format!("{}: line {} col {}: {}", path.display(), d.line, d.col, d.message))
        .collect::<Vec<_>>()
        .join("\n");
    Err(anyhow!("{rendered}"))
}
