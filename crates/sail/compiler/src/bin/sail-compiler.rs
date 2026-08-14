//! `sail-compiler` CLI. Compiles a single `.sail` file or a directory of
//! them, writing the emitted JS (and CEM/schema/meta/dts/ssr.html sidecars)
//! into a project-level mirror tree at `<repo-root>/.deck/<source-
//! relative-path>/` (SPEC-OUTPUT-1 product out). Pass `--out <dir>` to
//! override and emit every file to a single directory (legacy behaviour).
//! MIGRATION-DEBT: dies with CLI-legacy cleanup (`--out` single-dir mode).
//! Errors render with file path, line, and column.
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

use anyhow::{Context, Result, anyhow};
use clap::{Parser, Subcommand};

use sail_compiler::out_layout::resolve_out_dir;
use sail_compiler::{CompileMode, CompileOpts, ComponentMeta, Policy, compile};

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
        let lim = libc::rlimit {
            rlim_cur: MEM_CAP_BYTES as libc::rlim_t,
            rlim_max: MEM_CAP_BYTES as libc::rlim_t,
        };
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
        /// project mirror tree at `<repo-root>/.deck/<source-rel>/`
        /// and the JS module is named `<stem>.compiled.js`. When supplied,
        /// every file is emitted into this single directory and the JS
        /// module is named `<stem>.js` (legacy behaviour).
        /// MIGRATION-DEBT: dies with CLI-legacy cleanup (`--out` single-dir mode).
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
    /// resolved styling policy (`sail.config.json` walk-up), the Style vocabulary
    /// tables the lint validates against, the layout keywords, and the motion
    /// preset catalog — so the skill can never drift from the compiler. A
    /// strict-policy directory gets a skill without the Style product surface; a
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
    /// Print the compiler policy registry (SPEC-POLICY-1). This IS the docs
    /// projection — no markdown file. Default: human table. `--json` emits the
    /// shared serialization used by MCP `sail_policy` and GET `/_deck/policy`.
    Policy {
        /// `list` (registry rows) or `get` (registry + resolved values).
        #[arg(long, default_value = "list")]
        mode: String,
        /// Directory for `get` resolution (walks up for sail.config). Default: cwd.
        #[arg(long)]
        dir: Option<PathBuf>,
        /// Emit the shared JSON payload instead of the human table.
        #[arg(long)]
        json: bool,
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
    /// Emit design tokens from the DTCG model (`packages/sail/tokens/model/**`)
    /// into `tokens.css`, `themes.css`, `token-registry.json`, and
    /// `generated/unions.ts`. Authoritative token generate — edit the model,
    /// then run this (no Node).
    Tokens {
        /// Model directory (DTCG JSON + themes/). Default:
        /// `packages/sail/tokens/model` relative to the workspace root.
        #[arg(long)]
        model: Option<PathBuf>,
        /// Output directory for generated artifacts. Default:
        /// `packages/sail/tokens/src` relative to the workspace root.
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Emit the sail ambient TypeScript pack (`node_modules/.sail/types.d.ts`)
    /// from the same manifests that drive the compiler (runtime ambient
    /// vocabulary + Style setters + token scales). Regenerable editor helper —
    /// not hand-authored app source (SPEC-OUTPUT-1 §4).
    Types {
        /// Project directory that receives `node_modules/.sail/types.d.ts`.
        /// Default: cwd.
        #[arg(long)]
        project: Option<PathBuf>,
        /// Optional path for a committed base `tsconfig` snippet (stdout if
        /// `-`). When set, also writes (or prints) the recommended sail
        /// project tsconfig that includes the ambient pack.
        #[arg(long)]
        tsconfig_out: Option<PathBuf>,
        /// Write the pack to this path instead of
        /// `<project>/node_modules/.sail/types.d.ts`.
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
            input,
            out,
            clean,
            mode,
            no_dts,
            component_name,
            emit_component_imports,
        } => build(
            input,
            out,
            clean,
            mode,
            no_dts,
            component_name,
            emit_component_imports,
        ),
        Command::Skill { dir, out } => gen_skill(dir, out),
        Command::Policy { mode, dir, json } => print_policy(mode, dir, json),
        Command::FrameworkCss { out } => gen_framework_css(out),
        Command::PropRegistry { out, ts_out } => gen_prop_registry(out, ts_out),
        Command::Synthesize {
            input,
            manifest,
            out_dir,
        } => synthesize(input, manifest, out_dir),
        Command::Duplicates { dir, out } => gen_duplicates_report(dir, out),
        Command::Vocabulary { out } => gen_vocabulary(out),
        Command::Tokens { model, out } => gen_tokens(model, out),
        Command::Types {
            project,
            tsconfig_out,
            out,
        } => gen_types(project, tsconfig_out, out),
    }
}

/// Ambient pack → `<project>/node_modules/.sail/types.d.ts` (+ per-kind
/// tsconfigs). E1c: generated configs always regenerate — hand edits wiped.
fn gen_types(
    project: Option<PathBuf>,
    tsconfig_out: Option<PathBuf>,
    out: Option<PathBuf>,
) -> Result<()> {
    let project_dir = match project {
        Some(p) => p,
        None => std::env::current_dir().context("cwd")?,
    };
    let path = match out {
        Some(p) => {
            if let Some(parent) = p.parent() {
                if !parent.as_os_str().is_empty() {
                    fs::create_dir_all(parent)
                        .with_context(|| format!("mkdir {}", parent.display()))?;
                }
            }
            let text = sail_compiler::ambients::emit_types_dts();
            fs::write(&p, text).with_context(|| format!("write {}", p.display()))?;
            // Still regenerate per-kind configs next to the project.
            let _ = sail_compiler::ambients::write_project_type_configs(&project_dir)?;
            p
        }
        None => {
            let (types, pure, action) =
                sail_compiler::ambients::write_project_type_configs(&project_dir)?;
            println!("tsconfig.pure: {}", pure.display());
            println!("tsconfig.action: {}", action.display());
            types
        }
    };
    println!("types: {}", path.display());

    if let Some(ts_out) = tsconfig_out {
        // Product default is Pure (E1c) — never Framework.
        // Explicit dump: pure kind body (child under .sail/). Solution root is
        // always written as project/tsconfig.json by write_project_type_configs.
        let body = sail_compiler::ambients::emit_project_tsconfig_json(
            sail_compiler::ambients::PROJECT_PURE_INCLUDE,
        );
        if ts_out.as_os_str() == "-" {
            print!("{body}");
        } else {
            if let Some(parent) = ts_out.parent() {
                if !parent.as_os_str().is_empty() {
                    fs::create_dir_all(parent)
                        .with_context(|| format!("mkdir {}", parent.display()))?;
                }
            }
            fs::write(&ts_out, body).with_context(|| format!("write {}", ts_out.display()))?;
            println!("tsconfig: {}", ts_out.display());
        }
    }
    Ok(())
}

/// DTCG model → tokens.css + themes.css + token-registry.json + generated/unions.ts.
///
/// `--model` is the monorepo catalog-development override; default prefers
/// on-disk `packages/sail/tokens/model`, else the baked catalog.
fn gen_tokens(model: Option<PathBuf>, out: Option<PathBuf>) -> Result<()> {
    let root = workspace_root().ok();
    let model_dir =
        sail_compiler::tokens_generate::resolve_model_dir(root.as_deref(), model.as_deref())?;
    let out_dir = match out {
        Some(p) => p,
        None => root
            .as_ref()
            .map(|r| r.join(sail_compiler::tokens_generate::MONOREPO_TOKENS_SRC_REL))
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "tokens --out required when workspace root is unknown (no monorepo checkout)"
                )
            })?,
    };
    let report = sail_compiler::tokens_generate::generate(&model_dir, &out_dir)?;
    println!(
        "generate: {} tokens → {}",
        report.token_count,
        report.tokens_css.display()
    );
    println!(
        "  themes: {} → {}",
        report.theme_count,
        report.themes_css.display()
    );
    println!(
        "  registry: {} (colors: {}; paths: {})",
        report.registry_json.display(),
        report.color_count,
        report.path_count
    );
    println!("  unions: {}", report.unions_ts.display());
    println!("  layers: {}", report.layers.join(", "));
    Ok(())
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
                  * L1 element skins: lowered host Style + CVA-in-props attr rules.\n \
                  * Platform adaptor CSS (stdlib/platform/*.css) is spliced in under\n \
                  * @provenance headers. sail-base is the document floor. */\n";
    // Same `sail` cascade layer as route/page style install. Page atomics
    // must not ship unlayered — they share class names with skins and would
    // crush compound fills (e.g. solid bg loses to .s-bgtransparent).
    let body = format!(
        "{banner}@layer sail {{\n{}}}\n",
        sail_compiler::stdlib::skins_css()
    );
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
        // SPEC-D1: dotted surface members (`fg(spinner.foreground)`) are legal
        // only under that L1 host unit. Prop-registry re-parses stdlib sources
        // outside `compile_element`, so install the same host surface context.
        // Element sources use parse_element_source (no stdlib expand) — structure
        // may contain private native types (type=checkbox inside <checkbox> host)
        // that freeform author markup may not use.
        let program = sail_lint::style::with_host_surface(Some(tag), || {
            sail_compiler::parse::parse_element_source(source)
        })
        .map_err(|e| anyhow!("stdlib {tag} failed to parse: {e:?}"))?;
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
    println!(
        "wrote {} ({} elements)",
        path.display(),
        file["primitives"].as_object().map(|o| o.len()).unwrap_or(0)
    );

    let mut ts = String::new();
    ts.push_str(
        "// AUTO-GENERATED by `cargo run -p sail-compiler -- prop-registry`. Do not edit.\n",
    );
    ts.push_str("// Source: <scope> props declarations in crates/sail/compiler/stdlib/*.sail.\n");
    ts.push_str(
        "// The docs detail pages read THIS — props tables are generated, never hand-authored.\n\n",
    );
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
    sf.push_str(
        "// AUTO-GENERATED by `cargo run -p sail-compiler -- prop-registry`. Do not edit.\n",
    );
    sf.push_str("// The complete language surface, from the compiler's own tables: what the\n");
    sf.push_str("// lint enforces is what this file documents, by construction.\n\n");
    // SPEC-D1: docs language-surface must not teach component public names as
    // free Style color Tokens (validate rejects them). Filter at emit so
    // prop-registry regen stays the single command for this surface.
    let atoms: Vec<serde_json::Value> = sail_lint::rules::style_rule::vocabulary()
        .into_iter()
        .map(|(prop, values)| {
            let values: Vec<String> = values
                .into_iter()
                .filter(|v| !sail_lint::component_surfaces::is_component_public_name(v))
                .collect();
            serde_json::json!({ "prop": prop, "values": values })
        })
        .collect();
    sf.push_str("export interface SxAtom {\n  prop: string\n  values: string[]\n}\n\n");
    sf.push_str("export const SX_ATOMS: SxAtom[] = ");
    sf.push_str(&serde_json::to_string_pretty(&atoms)?);
    sf.push_str("\n\n");
    // SPEC-SPINNAKER-2: concern atlas + markers are the live Style table.
    let concern_rows: Vec<serde_json::Value> = sail_lint::style::concerns()
        .iter()
        .map(|c| {
            serde_json::json!({
                "name": c.name,
                "channel": c.channel.key(),
                "slots": c.slots.iter().map(|s| s.name).collect::<Vec<_>>(),
            })
        })
        .collect();
    sf.push_str(
        "export interface ConcernDoc {\n  name: string\n  channel: string\n  slots: string[]\n}\n\n",
    );
    sf.push_str("export const CONCERNS: ConcernDoc[] = ");
    sf.push_str(&serde_json::to_string_pretty(&concern_rows)?);
    sf.push_str("\n\n");
    sf.push_str("export const MARKERS: string[] = ");
    sf.push_str(&serde_json::to_string_pretty(&sail_lint::style::MARKERS)?);
    sf.push_str("\n\n");
    let states: Vec<serde_json::Value> = sail_lint::rules::style_rule::STATE_PREFIXES
        .iter()
        .map(|(n, sel)| serde_json::json!({ "prefix": n, "selector": sel }))
        .collect();
    sf.push_str("export const SX_STATE_PREFIXES: Array<{ prefix: string; selector: string }> = ");
    sf.push_str(&serde_json::to_string_pretty(&states)?);
    sf.push_str("\n\n");
    let conds: Vec<serde_json::Value> = sail_lint::rules::style_rule::CONDITION_PREFIXES
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
    // instead. Sourced from `style_vocab::refusals()` (the same table the lint
    // reads), so the docs teach the closure, by construction. The /reference
    // page renders this as the "Not sx — use this" table; the agent SKILL.md
    // leads with it.
    let refusals: Vec<serde_json::Value> = sail_lint::style_vocab::refusals()
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
    sf.push_str(
        "export interface MotionPreset {\n  name: string\n  kind: string\n  doc: string\n}\n\n",
    );
    sf.push_str("export const MOTION: { triggers: string[]; properties: string[]; staggerKey: string; presets: MotionPreset[] } = ");
    sf.push_str(&serde_json::to_string_pretty(&motion)?);
    sf.push_str("\n\n");

    // ELEMENTS — the L1 manifest (stdlib only; reskins array always empty
    // post SPEC-L1-1). Prop tables live in prop-registry.gen.ts.
    let (stdlib_l1, reskin_l1) = sail_compiler::l1_by_kind();
    debug_assert!(reskin_l1.is_empty(), "native reskins retired");
    let to_element_row = |e: &sail_compiler::l1_manifest::L1Entry| serde_json::json!({ "tag": e.tag, "description": e.description });
    let elements = serde_json::json!({
        "stdlib": stdlib_l1.iter().map(to_element_row).collect::<Vec<_>>(),
        "reskins": reskin_l1.iter().map(to_element_row).collect::<Vec<_>>(),
    });
    sf.push_str("export interface ElementDoc {\n  tag: string\n  description: string\n}\n\n");
    sf.push_str("export const ELEMENTS: { stdlib: ElementDoc[]; reskins: ElementDoc[] } = ");
    sf.push_str(&serde_json::to_string_pretty(&elements)?);
    sf.push_str("\n\n");

    // SEMANTIC PROPS — author a11y IR (SPEC-ARIA-1). Compiler stamps aria.
    let semantic_props: Vec<serde_json::Value> = sail_compiler::SEMANTIC_PROPS
        .iter()
        .map(|p| {
            let kind = match p.kind {
                sail_compiler::SemanticKind::String => "string".to_string(),
                sail_compiler::SemanticKind::Bool => "bool".to_string(),
                sail_compiler::SemanticKind::Number => "number".to_string(),
                sail_compiler::SemanticKind::Union(vs) => vs.join(" | "),
            };
            serde_json::json!({
                "name": p.name,
                "aria": p.aria,
                "kind": kind,
                "summary": p.summary,
            })
        })
        .collect();
    sf.push_str(
        "export interface SemanticPropDoc {\n  name: string\n  aria: string\n  kind: string\n  summary: string\n}\n\n",
    );
    sf.push_str("export const SEMANTIC_PROPS: SemanticPropDoc[] = ");
    sf.push_str(&serde_json::to_string_pretty(&semantic_props)?);
    sf.push_str("\n\n");

    // HOST_ANATOMY — compiler-stamped roles/live/atomic/labels (not free author API).
    let host_anatomy: Vec<serde_json::Value> = sail_compiler::HOST_ANATOMY
        .iter()
        .map(|(tag, a)| {
            serde_json::json!({
                "tag": tag,
                "role": a.role,
                "live": a.live,
                "atomic": a.atomic,
                "label": a.label,
            })
        })
        .collect();
    sf.push_str(
        "export interface HostAnatomyDoc {\n  tag: string\n  role: string | null\n  live: string | null\n  atomic: string | null\n  label: string | null\n}\n\n",
    );
    sf.push_str("/** Compiler-stamped L1 host anatomy — not free author API. */\n");
    sf.push_str("export const HOST_ANATOMY: HostAnatomyDoc[] = ");
    sf.push_str(&serde_json::to_string_pretty(&host_anatomy)?);
    sf.push_str("\n\n");

    // AUTHORING_BANS — data-*/free aria/style/script with rule ids.
    sf.push_str(
        "export interface AuthoringBanDoc {\n  syntax: string\n  rule: string\n  instead: string\n}\n\n",
    );
    sf.push_str("export const AUTHORING_BANS: AuthoringBanDoc[] = ");
    sf.push_str(&serde_json::to_string_pretty(
        &sail_compiler::AUTHORING_BANS,
    )?);
    sf.push_str("\n\n");

    // ACTIONS — the `use:` registry, straight from action_manifest::ACTIONS
    // (already sorted, already carries a prose `summary` per entry).
    // `public` marks the curated app-author surface (PUBLIC_ACTION_NAMES).
    let actions: Vec<serde_json::Value> = sail_compiler::action_manifest::ACTIONS
        .iter()
        .map(|a| {
            serde_json::json!({
                "name": a.name,
                "param": a.param,
                "summary": a.summary,
                "public": sail_compiler::action_manifest::is_public_action(a.name),
                "writes": a.writes,
            })
        })
        .collect();
    let public_actions: Vec<serde_json::Value> = sail_compiler::action_manifest::public_actions()
        .map(|a| {
            serde_json::json!({
                "name": a.name,
                "param": a.param,
                "summary": a.summary,
                "public": true,
                "writes": a.writes,
            })
        })
        .collect();
    sf.push_str(
        "export interface ActionDoc {\n  name: string\n  param: string\n  summary: string\n  public: boolean\n  /** camelCase bind vocabulary this action writes (`anchorLeft`, …). */\n  writes: string[]\n}\n\n",
    );
    sf.push_str("export const ACTIONS: ActionDoc[] = ");
    sf.push_str(&serde_json::to_string_pretty(&actions)?);
    sf.push_str("\n\n");
    sf.push_str("/** Curated app-author `use:` surface (subset of ACTIONS). */\n");
    sf.push_str("export const PUBLIC_ACTIONS: ActionDoc[] = ");
    sf.push_str(&serde_json::to_string_pretty(&public_actions)?);
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
    sf.push_str(&serde_json::to_string_pretty(
        &sail_compiler::directive_manifest::DIRECTIVES,
    )?);
    sf.push_str("\n\n");

    sf.push_str("export interface EventDoc {\n  name: string\n  summary: string\n}\n\n");
    sf.push_str("export const EVENT_NAMES: EventDoc[] = ");
    sf.push_str(&serde_json::to_string_pretty(
        &sail_compiler::directive_manifest::EVENT_NAMES,
    )?);
    sf.push_str("\n\n");

    sf.push_str("export interface ModifierDoc {\n  canonical: string\n  alias: string | null\n  arg: string | null\n  summary: string\n}\n\n");
    sf.push_str("export const EVENT_MODIFIERS: ModifierDoc[] = ");
    sf.push_str(&serde_json::to_string_pretty(
        &sail_compiler::directive_manifest::EVENT_MODIFIERS,
    )?);
    sf.push_str("\n\n");

    sf.push_str("export interface BindTargetDoc {\n  target: string\n  summary: string\n}\n\n");
    sf.push_str("export const BIND_TARGETS: BindTargetDoc[] = ");
    sf.push_str(&serde_json::to_string_pretty(
        &sail_compiler::directive_manifest::BIND_TARGETS,
    )?);
    sf.push_str("\n\n");

    sf.push_str("export interface ForeignIdiomDoc {\n  syntax: string\n  instead: string\n}\n\n");
    sf.push_str("export const FOREIGN_IDIOMS: ForeignIdiomDoc[] = ");
    sf.push_str(&serde_json::to_string_pretty(
        &sail_compiler::directive_manifest::FOREIGN_IDIOMS,
    )?);
    sf.push_str("\n\n");

    sf.push_str("export interface MotionValueFormDoc {\n  form: string\n  summary: string\n}\n\n");
    sf.push_str("export const MOTION_VALUE_FORMS: MotionValueFormDoc[] = ");
    sf.push_str(&serde_json::to_string_pretty(
        &sail_compiler::directive_manifest::MOTION_VALUE_FORMS,
    )?);
    sf.push_str("\n\n");

    // SCOPE GRAMMAR — every <scope> construct + refusal, straight from
    // scope_manifest (each row's `example` is compiled by that module's tests).
    sf.push_str("export interface ScopeGrammarDoc {\n  form: string\n  syntax: string\n  summary: string\n  example: string\n}\n\n");
    sf.push_str("export const SCOPE_GRAMMAR: ScopeGrammarDoc[] = ");
    sf.push_str(&serde_json::to_string_pretty(
        &sail_compiler::scope_manifest::SCOPE_GRAMMAR,
    )?);
    sf.push_str("\n\n");

    sf.push_str("export interface ScopeRefusalDoc {\n  syntax: string\n  instead: string\n  example: string\n}\n\n");
    sf.push_str("export const SCOPE_REFUSALS: ScopeRefusalDoc[] = ");
    sf.push_str(&serde_json::to_string_pretty(
        &sail_compiler::scope_manifest::SCOPE_REFUSALS,
    )?);
    sf.push_str("\n\n");

    // CONFIG — every sail.config.json key, straight from
    // sail_lint::config::CONFIG_KEY_DOCS (kept in lockstep with
    // RECOGNIZED_KEYS by that module's own test).
    let config_keys: Vec<serde_json::Value> = sail_lint::config::CONFIG_KEY_DOCS
        .iter()
        .map(|(key, ty, default, doc, layer)| {
            serde_json::json!({
                "key": key,
                "type": ty,
                "default": default,
                "doc": doc,
                "layer": layer,
            })
        })
        .collect();
    sf.push_str("export interface ConfigKeyDoc {\n  key: string\n  type: string\n  default: string\n  doc: string\n  layer: \"POLICY\" | \"TASTE\"\n}\n\n");
    sf.push_str("export const CONFIG_KEYS: ConfigKeyDoc[] = ");
    sf.push_str(&serde_json::to_string_pretty(&config_keys)?);
    sf.push_str("\n\n");

    // SX EXTRAS — the group/state-<value>/(--var) forms that are grammar
    // rather than a closed name table, straight from
    // sail_lint::rules::style_rule::SX_EXTRA_FORMS.
    let sx_extras: Vec<serde_json::Value> = sail_lint::rules::style_rule::SX_EXTRA_FORMS
        .iter()
        .map(|(form, doc)| serde_json::json!({ "form": form, "doc": doc }))
        .collect();
    sf.push_str("export interface SxExtraFormDoc {\n  form: string\n  doc: string\n}\n\n");
    sf.push_str("export const SX_EXTRA_FORMS: SxExtraFormDoc[] = ");
    sf.push_str(&serde_json::to_string_pretty(&sx_extras)?);
    sf.push_str("\n\n");

    // REGION CAPABILITY MATRIX — CHANNELS-1 WI10 / freeze §1. SSOT is
    // region_capability::REGION_CAPABILITY_MATRIX (lint + compiler citations).
    sf.push_str(
        "export interface RegionCapabilityDoc {\n  region: string\n  capability: string\n  legal: string\n  status: string\n  enforcement: string\n  notes: string\n}\n\n",
    );
    sf.push_str("export const REGION_CAPABILITY_MATRIX: RegionCapabilityDoc[] = ");
    sf.push_str(&serde_json::to_string_pretty(
        &sail_compiler::region_capability::REGION_CAPABILITY_MATRIX,
    )?);
    sf.push_str("\n\n");
    sf.push_str(&format!(
        "/** Thin-controller LOC threshold (sail-controller-size). */\nexport const CONTROLLER_SIZE_THRESHOLD = {}\n",
        sail_compiler::region_capability::CONTROLLER_SIZE_THRESHOLD
    ));

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

/// Fenced Style demos for `gen_skill`. Product attach is a **tag-position call**
/// (`style({…})` on the open tag), not an HTML `style=` attribute. Plain string for `push_str`.
///
/// `raw_containers`: when false, avoid bare `<div>`/`<span>` (no-raw policy).
fn style_skill_demo(raw_containers: bool) -> &'static str {
    if raw_containers {
        "```
\
         <div style({ pad(4).bg(surface).radius(lg), when: { hover: { bg(muted) } } })>
\
         </div>
\
         <span style({ fg(muted).type(body-sm) })>
\
         </span>
\
         ```

"
    } else {
        "```
\
         <column gap=\"3\" style({ pad(4).bg(surface).radius(lg), when: { hover: { bg(muted) } } })>
\
           <p style({ fg(muted).type(body-sm) })>
\
             …
\
           </p>
\
         </column>
\
         ```

\
         No bare `<div>`/`<span>` here — use layout keywords + content tags.

"
    }
}

/// Print the compiler policy registry (table or shared JSON serialization).
fn print_policy(mode: String, dir: Option<PathBuf>, as_json: bool) -> Result<()> {
    let dir = match dir {
        Some(d) => d,
        None => std::env::current_dir()?,
    };
    // Human table is the docs projection for `list` without `--json`.
    if !as_json && mode == "list" {
        print!("{}", sail_compiler::policy::format_registry_table());
        return Ok(());
    }
    let payload = sail_compiler::policy::serialize_policy(&mode, Some(&dir))
        .map_err(|e| anyhow::anyhow!(e.message))?;
    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}

/// Generate the agent-authoring skill for `dir` — **progressive disclosure**.
///
/// Default SKILL.md is ~one screen: hard rules, layout/participation, core
/// Style concerns only, and pointers to MCP tools for full tables (atlas,
/// tokens, L1, control-flow). Full inventories live in `sail_surface` (MCP) /
/// compiler SSOT — never dump absorbed property setters + L1 tags into the skill.
fn gen_skill(dir: Option<PathBuf>, out: Option<PathBuf>) -> Result<()> {
    use sail_compiler::check::policy::Policy;
    use std::fmt::Write as _;

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
    // Registry-resolved knobs (layout.raw_containers + lint.severity.*) — the
    // skill section is a projection of the same table as CLI/MCP/axum.
    // When the nested policy object did not set raw_containers, the capability
    // tier (preset / top-level key) still owns the effective value.
    let mut registry_resolved = sail_compiler::policy::resolve_for_source(&dir.join("_probe.sail"))
        .unwrap_or_else(|_| sail_compiler::policy::ResolvedPolicy::defaults());
    if !registry_resolved.raw_containers_explicit {
        registry_resolved.raw_containers = effective.raw_containers;
    }

    let mut md = String::new();
    let _ = writeln!(
        md,
        "---\nname: sail-authoring\ndescription: Author .sail in THIS directory (policy: {policy_line}). Core surface only — use MCP sail_surface / sail_check for full tables. Generated by `sail-compiler skill`; DO NOT hand-edit.\n---\n"
    );
    let _ = writeln!(md, "# Authoring SAIL here\n");
    let _ = writeln!(
        md,
        "GENERATED from the compiler's vocabulary + resolved styling policy. Off-list\n\
         values are compile errors that name the allowed list. Trust errors over memory.\n\n\
         **Progressive disclosure:** this skill is the core screen only. The concern\n\
         atlas, token scales, L1 inventory, motion detail, and control-flow\n\
         grammar come from tooling — not from this file.\n\n\
         | need | tool |\n\
         | --- | --- |\n\
         | focused surface tables | MCP `sail_surface` (`topic`: `core` \\| `layout` \\| `style` \\| `tokens` \\| `l1` \\| `control-flow`) |\n\
         | lint/compile diagnostics (JSON) | MCP `sail_check` (prefer `source`; optional `fix`: off|suggest|apply) |\n\
         | structured grammar-aware edit | MCP `sail_edit` (ops batch; validate-before-write) |\n\
         | regenerate this skill | `cargo run -p sail-compiler -- skill` |\n\n\
         MCP server: `cargo run -p sail-mcp` or `target/release/sail-mcp`. Workspace\n\
         `.mcp.json` may be **Ship-managed** and wiped on `ship compile` — re-add the\n\
         sail server (prefer the release binary) if tools disappear.\n"
    );
    let _ = writeln!(md, "## Framework stack (do not invent alternatives)\n");
    let _ = writeln!(
        md,
        "Skip this and agents invent `createElement` / `cssText` / querySelector\n\
         host assign. That is a **hard fail**, not a style preference.\n\n\
         1. **Signals + stores in the signal graph** — not module `let state` +\n   \
            manual sync when the store API exists. Shared/stateful logic:\n   \
            `defineStore` / `useStore` from `@ship/sail-runtime` (studio + apps).\n   \
            Local UI: `signal` / `derived` in `<controller>`, still in the graph.\n\
         2. **Actions (`use:`)** — sanctioned DOM/measurement home. Public\n   \
            catalog is attach-anywhere capabilities (anchor, autosize, …).\n   \
            Widget-private behavior (tabs/menu/pagination/radioGroup/combobox)\n   \
            is closed inside L1 widgets — use the widget tag, not a public\n   \
            `use:tabs`. Author actions live in `src/actions/` under a strict\n   \
            floor (ERROR: no style/class/layout, no signal/store/query/mutation,\n   \
            no network, host-only). App controllers do **not** touch `.style`\n   \
            / `createElement`.\n\
         3. **Compositions / L1–L2 primitives** — tabs, tooltip, sail-tree, form,\n   \
            selection, …. Use the primitive before hand-rolling.\n\
         4. **`prop:tree={{table}}`** (identity bind) for sail-tree — NEVER\n   \
            `document.querySelector` + assign the host (same as `prop:table`).\n\
         5. **Style + tokens** — only styling path. `style.cssText` / `.style` / raw\n   \
            CSS / retired utility-string forms (`sx=\"…\"`) are compile errors.\n\
         6. **If the primitive is missing** — stop and report the language gap.\n   \
            Do not escape to DOM.\n\n\
         **Hard fails:** any PR with `createElement` / `cssText` / `querySelector`\n\
         assign in `apps/` is rejected.\n"
    );

    // Resolved policy section — FROM the registry (SPEC-POLICY-1).
    md.push_str(&sail_compiler::policy::format_skill_section(
        &registry_resolved,
    ));

    let _ = writeln!(md, "## The hard rules (every policy)\n");
    let _ = writeln!(
        md,
        "- NO `<style>` blocks, NO product Style as an HTML attribute. NO raw values (px/hex) — spend a token.\n\
         - NO retired utility-string / `sx=` forms (`sx=\"p-4 bg-surface-2\"`). Product is a **tag** Style call:\n  \
           `style({{ pad(4).bg(surface) }})` / `when: {{ hover: {{ bg(muted) }} }}` / `match`.\n\
         - NO `data-*` in app markup (`sail-no-data-attr`). The compiler stamps\n  \
           `data-sail-host` on L1 as closed-widget part identity — never author it.\n  \
           App tests use role / accessible name / visible text, not that stamp.\n\
         - **Comments are not load-bearing.** Layout recipes that used to live only as\n  \
           HTML comments are **lints** (`sail-display-on-layout`, participation, …).\n  \
           Do not invent workarounds in comments — fix the structure so tools pass.\n\
         - Layout is elements, not CSS: `<row>`, `<column>`, `<grid>`, `<center>`,\n  \
           `<stack>` with `gap` / `align` / `justify` / `cols` attrs. Rows **wrap by\n  \
           default**; `<row nowrap>` is the single-line row. Unrecognized attrs on a\n  \
           layout keyword are a compile error.\n\
         - **Participation on the box channel** — `style({{ size({{ w: full, min-w: 0 }}).grow(1) }})`\n  \
           (or `shrink(0)`). Bare attrs `grow` / `shrink` / `w` / `min-w` / `max-w` are a\n  \
           hard lint (`sail-participation-style`). Layout-keyword **axis** attrs\n  \
           (`gap` / `align` / `justify` on row|column|grid) stay. Paint (`bg`, `fg`,\n  \
           chrome) stays on paint/text channels.\n\
         - **Never `display(…)` on layout keywords** (`row`/`column`/`grid`/\n  \
           `center`/`stack`). Bases own `display:flex|grid` and beat `.s-displaynone`\n  \
           (`.sail-col` footgun). Hide a plain host and nest the layout inside\n  \
           (`sail-display-on-layout`).\n\
         - Behavior is unrestricted TypeScript in `<controller>`. Static surface in\n  \
           `<scope>` (`props: {{ … }}`). If it computes, it is controller code.\n"
    );
    if effective.sx {
        let _ = writeln!(md, "## Style product (core)\n");
        // Demo tags must respect raw_containers: under no-raw, bare div/span fail check.
        // Block channel form is a plain string (style_skill_demo). In writeln! format
        // strings, `{{` / `}}` escape to single braces in the skill markdown.
        md.push_str(style_skill_demo(effective.raw_containers));
        let _ = writeln!(
            md,
            "Attach Style as a **tag call** `style({{…}})` on the open tag — not HTML `style=` or retired `sx=`.\n\
             Prefer named Style `style name = {{…}}` (or scope Style consts where dual path still applies),\n\
             `style({{ match: {{…}}, when: {{…}}, compound: […] }})`,\n\
             and first-class **`bind(name)`** for values that track props/signals/\n\
             derived/tokens/action writes. Style is compile-time static — no dynamic\n\
             JS expressions inside Style (computation lives in the controller; the\n\
             template names it). Concern atlas: MCP `sail_surface` topic\n\
             `style`. Layout atoms refused by name: MCP topic `layout`\n\
             (same `style_vocab::refusals()`).\n\
             \n\
             ### `bind(name)` units contract\n\
             \n\
             | Source → slot | Realization |\n\
             |---|---|\n\
             | number → length (`place.top`, `size.w`, `space.pad`, …) | legal; means **px** (`@property` length + reflection) |\n\
             | number → number (`filter.opacity`, `place.z`) | direct |\n\
             | string → any | pass-through (`syntax: '*'`) |\n\
             | number → color (`fill`, `ink`, …) | **compile error** |\n\
             | `derived` by name | allowed (read-only); annotate `(): number` for length |\n\
             | action write (`use:anchor` → `anchorLeft`) | via action manifest `writes` |\n\
             \n\
             Do not enumerate a prop as ≥3 compound rows that only vary a setter arg —\n\
             that is `sail-style-value-enumeration`; use `bind(name)` instead.\n"
        );

        // Concern atlas only — absorbed property names are not offered.
        let _ = writeln!(md, "### Core Style concerns (start here)\n");
        let _ = writeln!(
            md,
            "Flat chain is the language (`bg(surface).pad(4)`), not constructor\n\
             `style.padX`. Named composite or slot object. **Not listed:** absorbed\n\
             longhands (`padX`/`fontSize`/`truncate`/`shadow`) or dead v2 names\n\
             (`fill`/`ink`/`place`/`space`/`flow`/`motion`). Participation\n\
             (`grow`/`shrink`/`w`) lives on `size` / `grow` / `shrink`.\n"
        );
        for row in sail_lint::style::offered_rows()
            .filter(|r| r.tag == sail_lint::style::VocabTag::Concern)
        {
            let _ = writeln!(md, "- {}", sail_lint::style::vocab_surface_line(row));
        }
        let _ = writeln!(md);
        let _ = writeln!(md, "### Axis\n");
        for row in
            sail_lint::style::offered_rows().filter(|r| r.tag == sail_lint::style::VocabTag::Axis)
        {
            let _ = writeln!(md, "- {}", sail_lint::style::vocab_surface_line(row));
        }
        let _ = writeln!(md);
        let _ = writeln!(md, "### Plains\n");
        let _ = writeln!(
            md,
            "Verbs and singles. Full list: MCP `sail_surface` topic `style`.\n"
        );
        for name in ["cursor", "group", "isolate", "srOnly", "select"] {
            let _ = writeln!(md, "- `{name}(…)`");
        }
        let _ = writeln!(md);

        // Compact high-traffic scales only; full scales via MCP tokens topic.
        let _ = writeln!(md, "### Token scales (core)\n");
        let _ = writeln!(
            md,
            "Spend a token — never a raw unit. Full scales: MCP `sail_surface` topic `tokens`.\n"
        );
        for scale in &["space", "size", "color", "radius", "text", "weight"] {
            let values = sail_lint::scales::values(scale);
            if !values.is_empty() {
                let _ = writeln!(md, "- `{scale}`: {}", values.join(", "));
            }
        }
        let _ = writeln!(md);
    } else {
        let _ = writeln!(
            md,
            "## Style is NOT available here\n\nApplication-tier directory: compose components with props/tokens only.\n"
        );
    }
    if effective.variants {
        let _ = writeln!(
            md,
            "## variants\n\n\
             A variant is a prop. Prefer `match` / scope Style consts over residual\n\
             utility maps. Token-group props (e.g. `tone: Tone`) and Size ladders are\n\
             ordinary typed props — not special language features. Color groups bind\n\
             in Style paint (`bg({{ color: tone }})`); Size is discrete `match` conditionality.\n"
        );
    }
    if !effective.raw_containers {
        let _ = writeln!(
            md,
            "## Raw containers banned\n\nNo bare `<div>`/`<span>` — layout elements or a component.\n"
        );
    }

    let _ = writeln!(md, "## Motion (pointer)\n");
    let _ = writeln!(
        md,
        "Style animate channel: `.enter` / `.exit` / `.loop` / `.reveal` / `.scroll` /\n\
         `.hover` / `.press` / `.focus` / `.focus-visible` (public `fx:` is a compile error).\n\
         Fire-and-forget presets vs `timeline()` scrub lane — full tables + presets:\n\
         MCP `sail_surface` (or compile errors naming allowed presets).\n\n\
         ```\n\
         <card>\n\
           style({{ animate: reveal({{ opacity: 'fade', translate-y: 'spring:gentle' }}) }})\n\
           reveal\n\
         </card>\n\
         <button>\n\
           style({{ animate: press({{ scale: 'press' }}) }})\n\
           tactile\n\
         </button>\n\
         ```\n"
    );

    let _ = writeln!(md, "## Behavior + state (pointer)\n");
    let _ = writeln!(
        md,
        "See **Framework stack** above — this section is the syntax pointer only.\n\n\
         - `use:action` on elements; compositions (`form`, `selection`, …) return signals.\n\
         - State in `<controller>` via `controller(() => {{ … return {{ … }} }})`.\n\
         - Shared state: `defineStore` / `useStore` from `@ship/sail-runtime` (never `/internal`).\n\
         - Identity binds for engines: `prop:tree={{table}}`, `prop:table={{…}}` — not querySelector.\n\
         - Control flow: `<if>` / `<for key={{…}}>` / `<match>` / `<when {{tablet}}>` — full grammar:\n  \
           MCP `sail_surface` topic `control-flow`.\n\
         - In-file `<partial kebab props: Type>`: same-file functional component; invoke as\n  \
           `<kebab …>`; body reads bare `{{title}}`; expanded at compile time (SPEC-PARTIAL-1).\n  \
           No recursion; no `style(…)` on the invocation. Cross-file = component file.\n\
         - **Structural responsiveness** is `<when {{token}}>` (theme breakpoints:\n  \
           mobile/tablet/desktop). Different trees per viewport; SSR paints the\n  \
           `{{else => …}}` / default branch only (mobile-first), then hydrates with matchMedia.\n\
         - Style `.when({{ tablet: style.… }})` stays paint-level (density, color) — not trees.\n\
         - **No layout `matchMedia` in controllers** (`sail-controller-matchmedia-layout`):\n  \
           do not drive template-visible layout from `window.matchMedia('(min-width:…)')`\n  \
           / `max-width` / `orientation`. Use `<when {{tablet}}>` for structure; Style\n  \
           `when` for paint/density. Non-layout media (`prefers-reduced-motion`, `prefers-color-scheme`,\n  \
           `print`) may stay imperative.\n\n\
         ```\n\
         <controller>\n  \
           import {{ controller, signal }} from '@ship/sail-runtime'\n  \
           export default controller(() => {{\n    \
             const open = signal(false)\n    \
             return {{ open }}\n  \
           }})\n\
         </controller>\n\
         <if {{open()}}><column gap=\"4\">…</column></if>\n\
         <when {{tablet}}><row>…wide…</row>{{else => <column>…narrow…</column>}}</when>\n\
         ```\n"
    );

    // Count source rows — do not compile stdlib here. Rewriter owns corpus
    // compile; `l1_by_kind()` / `elements()` panic on unmigrated elements.
    let stdlib_n = sail_compiler::stdlib::sources().len();
    let _ = writeln!(md, "## L1 elements (pointer)\n");
    let _ = writeln!(
        md,
        "{stdlib_n} stdlib elements (native reskins retired). Bare tags are the API\n\
         (`<button>`, `<card>`, …). Full inventory: MCP `sail_surface` topic `l1`\n\
         (same `l1_manifest` the consistency tests enforce).\n"
    );

    let _ = writeln!(md, "## When the compiler rejects you\n");
    let _ = writeln!(
        md,
        "The rejection **is** the documentation. Do not escape with `<style>` or\n\
         inline style. Run MCP `sail_check` for structured diagnostics. If the\n\
         vocabulary cannot express the design, stop and report a language gap.\n"
    );

    let out_path = out.unwrap_or_else(|| dir.join(".claude/skills/sail-authoring/SKILL.md"));
    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&out_path, md)?;
    println!(
        "skill written: {} (policy: {policy_line})",
        out_path.display()
    );
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
        let Ok(rd) = std::fs::read_dir(dir) else {
            return;
        };
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
        let sail_sources: Vec<PathBuf> = std::fs::read_dir(dir_path)
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
                it.filter_map(Result::ok).any(|e| {
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
        let has_sail = !sail_sources.is_empty();
        let status: String = if !has_sail && has_controller {
            "orphan".to_string()
        } else if has_sail {
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

fn synthesize(input: PathBuf, manifest_path: PathBuf, out_dir: PathBuf) -> Result<()> {
    use sail_compiler::emit::synthetic::{AppManifest, AuthoredController, emit_synthetic};
    use sail_compiler::parse;

    let source = fs::read_to_string(&input).with_context(|| format!("read {}", input.display()))?;
    let manifest_src = fs::read_to_string(&manifest_path)
        .with_context(|| format!("read manifest {}", manifest_path.display()))?;
    // SFC: strip the inline `<controller>` block before parse (mirrors
    // `compile()`), so its body isn't misread as a template expression.
    let (stripped, inline_controller_src) = parse::preprocess::extract_controller_block(&source);
    let program = parse::parse(&stripped).map_err(|e| anyhow!("{}", e.render(&stripped)))?;
    let manifest =
        AppManifest::parse(&manifest_src).map_err(|e| anyhow!("{}", e.render(&manifest_src)))?;

    let file = input
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("view.sail");
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
        fs::write(&ctrl_path, wrapped).with_context(|| format!("write {}", ctrl_path.display()))?;
        Some(AuthoredController {
            specifier: format!("./{ctrl_name}"),
        })
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
        if out.used_clients.is_empty() {
            "none".to_string()
        } else {
            out.used_clients.join(", ")
        },
        if authored.is_some() {
            "; composed authored controller.ts"
        } else {
            ""
        },
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
    Some(AuthoredController {
        specifier: rel_specifier(&candidate, out_dir),
    })
}

/// POSIX relative-path specifier from `out_dir` to `target` (both made
/// absolute against CWD), suitable as an ESM import string.
fn rel_specifier(target: &Path, out_dir: &Path) -> String {
    let cwd = std::env::current_dir().unwrap_or_default();
    let abs = |p: &Path| {
        if p.is_absolute() {
            p.to_path_buf()
        } else {
            cwd.join(p)
        }
    };
    let (target, out_dir) = (abs(target), abs(out_dir));
    let from: Vec<_> = out_dir.components().collect();
    let to: Vec<_> = target.components().collect();
    let common = from
        .iter()
        .zip(to.iter())
        .take_while(|(a, b)| a == b)
        .count();
    let mut out = String::new();
    for _ in 0..(from.len() - common) {
        out.push_str("../");
    }
    let rest: Vec<String> = to[common..]
        .iter()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
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
        return Err(anyhow!(
            "not a .sail file or directory: {}",
            input.display()
        ));
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

    // E1c: regenerate ambient pack + per-kind tsconfigs. Hand-edited lib
    // entries are overwritten.
    //
    // Project type gate (E1b) runs for app-shaped projects (`routes/`).
    // Component packages (L2 npm corpus under packages/components, or any
    // package without routes/) are gated by per-file `.sail` controller
    // regions (E1a Pure+strict) — their `src/**/*.ts` engines are not the
    // app product surface.
    if input.is_dir() {
        let mut projects = collect_project_roots(&input)?;
        if projects.is_empty() {
            if let Some(p) = nearest_project_root(&targets[0]) {
                projects.push(p);
            }
        }
        for project in &projects {
            let _ = sail_compiler::ambients::write_project_type_configs(project);
            let is_app = project.join("routes").is_dir();
            if is_app {
                if let Err(e) = project_ts_type_gate(project) {
                    return Err(e);
                }
            }
        }
    } else if let Some(project) = nearest_project_root(&targets[0]) {
        let _ = sail_compiler::ambients::write_project_type_configs(&project);
    }

    // Component mode: tag name comes from --component-name when supplied,
    // otherwise defaults to the filename stem at compile-one time. Shadow
    // is always on for components — no opt-out flag.
    //
    // Guardrail (#176): when an input matches the `<tag>/<tag>.sail` component
    // convention AND no explicit mode/component-name was given, refuse rather
    // than emit component JS with no tag — except directory builds of a
    // component tree, which auto-infer the tag from the path (L2 package gate).
    let count = targets.len();
    let dir_auto_component = input.is_dir();
    for path in targets {
        let path_tag = looks_like_component_path(&path);
        let explicit = mode.is_some() || component_name.is_some();
        if !explicit && path_tag.is_some() && !dir_auto_component {
            let tag = path_tag.as_deref().unwrap_or("?");
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
            let out_dir = out
                .clone()
                .unwrap_or_else(|| path.parent().map(|p| p.to_path_buf()).unwrap_or_default());
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
                name: name.unwrap_or_else(|| path_tag.clone().unwrap_or_else(|| stem_of(&path))),
            }),
            (_, Some(name)) => Some(ComponentMeta { name }),
            // Directory auto: prefer `<tag>/<tag>.sail`, else filename stem
            // (covers co-located siblings like sail-timeline/sail-timeline-item.sail).
            _ if dir_auto_component => Some(ComponentMeta {
                name: path_tag.unwrap_or_else(|| stem_of(&path)),
            }),
            _ => None,
        };
        compile_one(
            &path,
            out.as_deref(),
            mode.clone(),
            no_dts,
            component_meta,
            emit_component_imports,
        )?;
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
    if stem == parent {
        Some(stem.to_string())
    } else {
        None
    }
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

/// Walk up from a source path looking for a sail project root.
///
/// Prefer `package.json` (+ `src`/`routes`) over a nested `sail.config.json`
/// (e.g. `src/sail.config.json` is a lint preset only — not the app root).
/// Used to place generated per-kind tsconfigs and run the project-scoped
/// type gate (E1b).
fn nearest_project_root(from: &Path) -> Option<PathBuf> {
    let mut dir = if from.is_file() {
        from.parent().map(|p| p.to_path_buf())
    } else {
        Some(from.to_path_buf())
    };
    let mut nested_config: Option<PathBuf> = None;
    while let Some(d) = dir {
        // App package root wins (same preference as tscheck::find_project_root).
        if d.join("package.json").is_file() && (d.join("src").is_dir() || d.join("routes").is_dir())
        {
            return Some(d);
        }
        if nested_config.is_none()
            && (d.join("sail.config.json").is_file() || d.join("sail.config.jsonc").is_file())
        {
            nested_config = Some(d.to_path_buf());
        }
        // Stop at workspace root markers so monorepo apps keep their own root.
        if d.join("Cargo.toml").is_file() && d.join("crates").is_dir() {
            return nested_config;
        }
        dir = d.parent().map(|p| p.to_path_buf());
    }
    nested_config
}

/// Collect authored product `.ts` files under `project` and run the E1b
/// project type gate (one tsgo invocation per TypeEnv that has files).
fn project_ts_type_gate(project: &Path) -> Result<()> {
    let mut files: Vec<(PathBuf, String)> = Vec::new();
    collect_ts_sources(project, project, &mut files)?;
    if files.is_empty() {
        return Ok(());
    }
    let diags = sail_compiler::check::tscheck::collect_project_type_gate(project, &files);
    let fatals: Vec<_> = diags
        .into_iter()
        .filter(|d| d.severity == sail_compiler::DiagSeverity::Error)
        .collect();
    if fatals.is_empty() {
        return Ok(());
    }
    for d in &fatals {
        let rule = d.rule.as_deref().unwrap_or("sail-type-gate");
        // Project batch encodes the real file as a trailing ` [path]` suffix
        // (see `typecheck_project_env`). Prefer that for `path:line:col` tooling.
        let (path_disp, message) = match split_bracketed_path(&d.message) {
            Some((p, msg)) => (p, msg),
            None => (project.display().to_string(), d.message.as_str()),
        };
        eprintln!(
            "{}:{}:{}: error[{}]: {}",
            path_disp, d.line, d.col, rule, message
        );
    }
    Err(anyhow!(
        "type gate failed ({} error{})",
        fatals.len(),
        if fatals.len() == 1 { "" } else { "s" }
    ))
}

/// Split a trailing ` [absolute/or/rel/path]` suffix from a type-gate message.
/// Returns `(path, message_without_suffix)`.
fn split_bracketed_path(message: &str) -> Option<(String, &str)> {
    let start = message.rfind(" [")?;
    let inner = message[start + 2..].strip_suffix(']')?;
    if inner.is_empty() {
        return None;
    }
    // Heuristic: real paths look like files (contain `/` or end with .ts/.sail).
    let looks_like_path = inner.contains('/')
        || inner.contains('\\')
        || inner.ends_with(".ts")
        || inner.ends_with(".tsx")
        || inner.ends_with(".sail");
    if !looks_like_path {
        return None;
    }
    Some((inner.to_string(), message[..start].trim_end()))
}

fn collect_ts_sources(project: &Path, dir: &Path, out: &mut Vec<(PathBuf, String)>) -> Result<()> {
    let skip = [
        "node_modules",
        "dist",
        ".deck",
        "target",
        ".git",
        "test",
        "tests",
        "__tests__",
    ];
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Ok(()),
    };
    for entry in entries {
        let entry = entry?;
        let p = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if p.is_dir() {
            if skip.iter().any(|s| *s == name) {
                continue;
            }
            collect_ts_sources(project, &p, out)?;
        } else if name.ends_with(".ts") && !name.ends_with(".d.ts") {
            // Tests / harness helpers are not product law under the type gate.
            if name.ends_with(".test.ts")
                || name.ends_with(".spec.ts")
                || name == "test-utils.ts"
                || name == "setup.ts"
                || name == "index.ts"
            {
                // index.ts is side-effect re-exports of compiled components —
                // not product Pure law under the type gate.
                continue;
            }
            // Only product homes — compositions / stores / actions / routes / src.
            let rel = p.strip_prefix(project).unwrap_or(&p);
            let rel_s = rel.to_string_lossy();
            let product = rel_s.contains("src/actions/")
                || rel_s.contains("src/compositions/")
                || rel_s.contains("src/stores/")
                || rel_s.contains("routes/")
                || (rel_s.starts_with("src/") && rel_s.ends_with(".ts"));
            if !product {
                continue;
            }
            if let Ok(src) = fs::read_to_string(&p) {
                out.push((p, src));
            }
        }
    }
    Ok(())
}

/// Package roots under `dir` that look like sail projects (`package.json` +
/// `src/` or `routes/`). Multi-domain component trees gate each package.
fn collect_project_roots(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    collect_project_roots_rec(dir, &mut out, 0)?;
    out.sort();
    out.dedup();
    Ok(out)
}

fn collect_project_roots_rec(dir: &Path, out: &mut Vec<PathBuf>, depth: usize) -> Result<()> {
    if depth > 6 {
        return Ok(());
    }
    let skip = ["node_modules", "dist", ".deck", "target", ".git"];
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Ok(()),
    };
    for entry in entries {
        let entry = entry?;
        let p = entry.path();
        if !p.is_dir() {
            continue;
        }
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if skip.iter().any(|s| *s == name) {
            continue;
        }
        if p.join("package.json").is_file() && (p.join("src").is_dir() || p.join("routes").is_dir())
        {
            out.push(p.clone());
            continue;
        }
        collect_project_roots_rec(&p, out, depth + 1)?;
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
    let orig_source =
        fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
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
    // the project mirror tree at `<repo-root>/.deck/<src-rel>/`;
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
    // compile() already gates the full Error-severity lint stream internally
    // (`check::gate`, #255) — a second check_all here re-ran the identical
    // pass and doubled compile latency on large files.
    let result = compile(&orig_source, opts).map_err(|e| {
        let rendered = e.render(&orig_source);
        anyhow!("{rendered}")
    })?;

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
    fs::write(&js_path, &result.js).with_context(|| format!("write {}", js_path.display()))?;
    if let Some(cem) = result.cem {
        let cem_path = out_dir.join(format!("{stem_no_ext}.custom-elements.json"));
        let pretty = serde_json::to_string_pretty(&cem)?;
        fs::write(&cem_path, pretty).with_context(|| format!("write {}", cem_path.display()))?;
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
        fs::write(&meta_path, pretty).with_context(|| format!("write {}", meta_path.display()))?;
    }
    // `.sail.d.ts` is a TYPE companion to its source, so it is written
    // ADJACENT to the `.sail` (not into the build mirror like the JS/CEM/SSR
    // artifacts). That lets a controller's `import type { Scope } from
    // './<name>.sail'` resolve natively to the sibling `.d.ts` — no `rootDirs`
    // round-trip into a build mirror in every consumer's tsconfig (which was
    // fragile and, when mispathed, silently masked type errors as `any`).
    // Gitignored via `*.sail.d.ts`.
    if let Some(dts) = result.dts {
        let dts_dir = path.parent().unwrap_or(out_dir.as_path());
        let dts_path = dts_dir.join(format!("{stem}.d.ts"));
        fs::write(&dts_path, dts).with_context(|| format!("write {}", dts_path.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod skill_demo_tests {
    use super::style_skill_demo;

    #[test]
    fn raw_style_demo_uses_block_channel_and_div_span() {
        let md = style_skill_demo(true);
        assert!(
            md.contains("style({"),
            "product Style is block channel form: {md}"
        );
        assert!(
            !md.contains("style={style."),
            "must not teach attr host style={{style.…}}: {md}"
        );
        assert!(!md.contains("sx="), "must not teach retired sx=: {md}");
        assert!(
            !md.contains("style={{style."),
            "doubled braces are invalid Style / format-string leak: {md}"
        );
        assert!(
            !md.contains("when({{"),
            "nested when({{ is format-string leak: {md}"
        );
        assert!(
            md.contains("<div style("),
            "full policy demos use raw div with Style attach: {md}"
        );
        assert!(
            md.contains("<span style("),
            "full policy demos use raw span with Style attach: {md}"
        );
    }

    #[test]
    fn no_raw_style_demo_uses_layout_keywords_and_block_channel() {
        let md = style_skill_demo(false);
        assert!(
            md.contains("style({"),
            "product Style is block channel form: {md}"
        );
        assert!(
            !md.contains("style={style."),
            "must not teach attr host style={{style.…}}: {md}"
        );
        assert!(!md.contains("sx="), "must not teach retired sx=: {md}");
        assert!(
            !md.contains("style={{style."),
            "doubled braces are invalid Style / format-string leak: {md}"
        );
        assert!(
            !md.contains("when({{"),
            "nested when({{ is format-string leak: {md}"
        );
        assert!(
            md.contains("<column ") && md.contains("<p style("),
            "no-raw demos use layout + content tags: {md}"
        );
        assert!(
            !md.contains("<div ")
                && !md.contains("<span ")
                && !md
                    .lines()
                    .any(|l| l.trim_start().starts_with("<div>")
                        || l.trim_start().starts_with("<span>")),
            "no-raw demos must not teach bare div/span tags: {md}"
        );
        assert!(
            md.contains("No bare"),
            "no-raw branch should note the ban: {md}"
        );
    }
}
