//! `sail editor setup` — write editor config so `*.sail` uses `sail lsp`
//! and the tree-sitter grammar. No monorepo paths. Default command is
//! `sail` on PATH (`--bin` is the developer override, same as MCP).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

const MARKER: &str = "# sail-editor (sail editor setup)";

const HIGHLIGHTS_SCM: &str =
    include_str!("../../../../packages/sail/grammar/queries/highlights.scm");
const INJECTIONS_SCM: &str =
    include_str!("../../../../packages/sail/grammar/queries/injections.scm");
const PARSER_C: &str = include_str!("../../../../packages/sail/grammar/src/parser.c");
const TS_PARSER_H: &str =
    include_str!("../../../../packages/sail/grammar/src/tree_sitter/parser.h");
const TS_ALLOC_H: &str = include_str!("../../../../packages/sail/grammar/src/tree_sitter/alloc.h");
const TS_ARRAY_H: &str = include_str!("../../../../packages/sail/grammar/src/tree_sitter/array.h");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorKind {
    Helix,
    Nvim,
    Vscode,
}

impl EditorKind {
    pub fn parse(s: &str) -> Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "helix" | "hx" => Ok(Self::Helix),
            "nvim" | "neovim" => Ok(Self::Nvim),
            "vscode" | "code" | "cursor" => Ok(Self::Vscode),
            other => bail!("unknown editor {other:?} — helix | nvim | vscode"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SetupReport {
    pub editor: &'static str,
    pub writes: Vec<PathBuf>,
    pub notes: Vec<String>,
    pub command: String,
}

/// Install Sail into `editor`. Paths come from `home` / XDG (tests inject).
pub fn setup(editor: EditorKind, bin: Option<&Path>, home: &Path) -> Result<SetupReport> {
    let command = launch_command(bin);
    let mut report = match editor {
        EditorKind::Helix => setup_helix(&command, home)?,
        EditorKind::Nvim => setup_nvim(&command, home)?,
        EditorKind::Vscode => setup_vscode(&command)?,
    };
    match crate::ts_native::ensure_best_effort() {
        Ok(Some(path)) => {
            report
                .notes
                .push(format!("typescript sidecar: {}", path.display()));
        }
        Ok(None) => {}
        Err(e) => {
            eprintln!(
                "sail editor setup: TypeScript 7 native sidecar missing ({e}). \
                 Grammar was written. Run `sail ts-native setup`."
            );
        }
    }
    Ok(report)
}

fn launch_command(bin: Option<&Path>) -> String {
    match bin {
        Some(p) => p.display().to_string(),
        None => "sail".into(),
    }
}

fn data_dir(home: &Path) -> PathBuf {
    std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".local/share"))
        .join("sail")
}

fn config_dir(home: &Path) -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".config"))
}

fn extract_grammar(home: &Path) -> Result<PathBuf> {
    let root = data_dir(home).join("grammar");
    fs::create_dir_all(root.join("queries")).context("mkdir grammar/queries")?;
    fs::create_dir_all(root.join("src/tree_sitter")).context("mkdir grammar/src")?;
    write_if_changed(&root.join("queries/highlights.scm"), HIGHLIGHTS_SCM)?;
    write_if_changed(&root.join("queries/injections.scm"), INJECTIONS_SCM)?;
    write_if_changed(&root.join("src/parser.c"), PARSER_C)?;
    write_if_changed(&root.join("src/tree_sitter/parser.h"), TS_PARSER_H)?;
    write_if_changed(&root.join("src/tree_sitter/alloc.h"), TS_ALLOC_H)?;
    write_if_changed(&root.join("src/tree_sitter/array.h"), TS_ARRAY_H)?;
    Ok(root)
}

fn write_if_changed(path: &Path, body: &str) -> Result<()> {
    if path.is_file() && fs::read_to_string(path).map(|e| e == body).unwrap_or(false) {
        return Ok(());
    }
    fs::write(path, body).with_context(|| format!("write {}", path.display()))
}

fn setup_helix(command: &str, home: &Path) -> Result<SetupReport> {
    let grammar = extract_grammar(home)?;
    let helix_cfg = std::env::var_os("HELIX_CONFIG")
        .map(PathBuf::from)
        .unwrap_or_else(|| config_dir(home).join("helix"));
    fs::create_dir_all(&helix_cfg).context("mkdir helix config")?;
    let langs = helix_cfg.join("languages.toml");
    let block = helix_languages_block(command, &grammar);
    let mut notes = Vec::new();
    if langs.is_file() && fs::read_to_string(&langs)?.contains(MARKER) {
        notes.push(format!("already configured: {}", langs.display()));
    } else if langs.is_file() {
        let mut existing = fs::read_to_string(&langs)?;
        if !existing.ends_with('\n') {
            existing.push('\n');
        }
        existing.push_str(&block);
        fs::write(&langs, existing).context("append helix languages.toml")?;
    } else {
        fs::write(&langs, block).context("write helix languages.toml")?;
    }

    let mut writes = vec![grammar.clone(), langs];
    match compile_sail_so(&grammar, &helix_cfg) {
        Ok(so) => {
            writes.push(so);
        }
        Err(e) => notes.push(format!(
            "sail.so not compiled ({e}) — `cc` + parser.c in {}",
            grammar.display()
        )),
    }
    notes.push("reload Helix (:config-reload). Do not run bare `hx --grammar build`.".into());

    Ok(SetupReport {
        editor: "helix",
        writes,
        notes,
        command: command.to_string(),
    })
}

fn helix_languages_block(command: &str, grammar: &Path) -> String {
    format!(
        r#"
{MARKER}
[language-server.sail]
command = {command:?}
args = ["lsp"]

[[language]]
name = "sail"
scope = "source.sail"
file-types = ["sail"]
comment-token = "<!--"
indent = {{ tab-width = 2, unit = "  " }}
language-servers = ["sail"]
grammar = "sail"

[[grammar]]
name = "sail"
source = {{ path = {path:?} }}
"#,
        command = command,
        path = grammar.display().to_string(),
    )
}

fn helix_runtime(helix_cfg: &Path) -> Result<PathBuf> {
    let runtime = helix_cfg.join("runtime");
    if runtime.is_symlink() {
        return fs::read_link(&runtime)
            .map(|p| {
                if p.is_absolute() {
                    p
                } else {
                    helix_cfg.join(p)
                }
            })
            .or_else(|_| runtime.canonicalize())
            .context("resolve helix runtime symlink");
    }
    if let Ok(r) = std::env::var("HELIX_RUNTIME") {
        return Ok(PathBuf::from(r));
    }
    if runtime.is_dir() {
        return Ok(runtime);
    }
    // Do not mkdir a blank runtime — that hides stock themes.
    bail!(
        "no Helix runtime at {} (and no HELIX_RUNTIME) — refusing to create an empty overlay",
        runtime.display()
    )
}

fn compile_sail_so(grammar: &Path, helix_cfg: &Path) -> Result<PathBuf> {
    let runtime = helix_runtime(helix_cfg)?;
    let grammars = runtime.join("grammars");
    fs::create_dir_all(&grammars).context("mkdir helix grammars")?;
    let queries = runtime.join("queries/sail");
    fs::create_dir_all(&queries).context("mkdir helix queries/sail")?;
    fs::copy(
        grammar.join("queries/highlights.scm"),
        queries.join("highlights.scm"),
    )?;
    fs::copy(
        grammar.join("queries/injections.scm"),
        queries.join("injections.scm"),
    )?;
    let so = grammars.join("sail.so");
    let status = Command::new("cc")
        .args(["-shared", "-fPIC", "-O2", "-I"])
        .arg(grammar.join("src"))
        .arg("-o")
        .arg(&so)
        .arg(grammar.join("src/parser.c"))
        .status()
        .context("spawn cc")?;
    if !status.success() {
        bail!("cc exited {status}");
    }
    Ok(so)
}

fn setup_nvim(command: &str, home: &Path) -> Result<SetupReport> {
    let grammar = extract_grammar(home)?;
    let plugin_dir = config_dir(home).join("nvim/plugin");
    fs::create_dir_all(&plugin_dir).context("mkdir nvim/plugin")?;
    let lua = plugin_dir.join("sail.lua");
    let body = format!(
        r#"-- {MARKER}
-- Intelligence: sail lsp. Highlight: nvim-treesitter parser at
-- {grammar} (install via :TSInstallFromGrammar / a path parser).
vim.filetype.add({{ extension = {{ sail = "sail" }} }})
vim.api.nvim_create_autocmd("FileType", {{
  pattern = "sail",
  callback = function()
    vim.lsp.start({{
      name = "sail",
      cmd = {{ {command:?}, "lsp" }},
      root_dir = vim.fs.root(0, {{ "sail.config.jsonc", "sail.config.json" }}),
    }})
  end,
}})
"#,
        grammar = grammar.display(),
        command = command,
    );
    write_if_changed(&lua, &body)?;
    Ok(SetupReport {
        editor: "nvim",
        writes: vec![grammar, lua],
        notes: vec![
            "nvim-treesitter still needs a sail parser registered from the extracted grammar path."
                .into(),
        ],
        command: command.to_string(),
    })
}

fn setup_vscode(command: &str) -> Result<SetupReport> {
    Ok(SetupReport {
        editor: "vscode",
        writes: vec![],
        notes: vec![
            format!("VS Code / Cursor: install the Sail VSIX, then put `{command}` on PATH."),
            "Command Palette → Extensions: Install from VSIX… → sail-*.vsix".into(),
            format!(
                "Or set sail.lsp.path to the `{command}` binary if the editor cannot see PATH."
            ),
            "Marketplace publish is a later identity cut — the VSIX is the product artifact."
                .into(),
        ],
        command: command.to_string(),
    })
}

pub fn print_report(r: &SetupReport) {
    println!("sail editor setup {}", r.editor);
    println!("  command: {} lsp", r.command);
    for w in &r.writes {
        println!("  wrote: {}", w.display());
    }
    for n in &r.notes {
        println!("  note: {n}");
    }
}
