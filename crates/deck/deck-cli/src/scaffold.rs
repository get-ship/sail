//! Product scaffold: `sail create` and `sail scaffold theme`.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};

/// Titlecase `hello-world` → `Hello World` for default view titles.
fn title_case(name: &str) -> String {
    name.split('-')
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                Some(c) => c.to_ascii_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Resolve the Cargo workspace root from `start` (catalog theme source).
fn find_repo_root(start: &Path) -> Result<PathBuf> {
    deck_bundler::paths::topmost_cargo_root(start)
        .ok_or_else(|| anyhow!("no Cargo workspace root above {}", start.display()))
}

/// Validate an external app directory basename (`my-app`).
///
/// Same shape as route/package names: `/^[a-z][a-z0-9-]*$/`. Becomes the npm
/// `package.json` `name` field (unscoped) and the on-disk directory name.
fn validate_app_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("app name is required");
    }
    if name == "." || name == ".." || name.contains('/') || name.contains('\\') {
        // Multi-segment paths are allowed as the create *path*; only the
        // basename is validated here. Callers strip parents first.
        bail!("app name must be a single path segment — got {name:?}");
    }
    if !name.starts_with(|c: char| c.is_ascii_lowercase()) {
        bail!("app name must start with a lowercase letter — got {name:?}");
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        bail!("app name must match /^[a-z][a-z0-9-]*$/ — got {name:?}");
    }
    Ok(())
}

/// `sail create my-app` — write the external-app 4-entry tree + gitignore.
///
/// Shape (EXT-1 E8):
/// ```text
/// my-app/
///   package.json         # registry-style @ship/* deps (no file:/workspace:*)
///   sail.config.jsonc    # theme + deck.render (nested serve)
///   routes/page.sail     # hello-world source
///   themes/sail.json     # catalog default (baked)
///   .gitignore           # includes dist/ + .sail/ (deploy + generate — never commit)
/// ```
///
/// Also seeds `.sail/types.d.ts`, solution `tsconfig.json`
/// (TOOLING-1 WI2), and `.oxlintrc.json` so typecheck + oxlint work out of the
/// box with zero hand edits. Does **not** publish to a registry (operator).
pub fn run_create(name: PathBuf) -> Result<()> {
    let cwd = std::env::current_dir().context("cwd")?;
    let app_dir = if name.as_os_str().is_empty() {
        bail!("app name is required");
    } else if name.is_absolute() {
        name
    } else {
        cwd.join(&name)
    };

    let basen = app_dir
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow!("app path has no basename: {}", app_dir.display()))?;
    validate_app_name(basen)?;

    if app_dir.exists() {
        bail!("directory already exists: {}", app_dir.display());
    }
    fs::create_dir_all(app_dir.join("routes"))
        .with_context(|| format!("mkdir {}", app_dir.display()))?;

    // Registry-style deps only (no file:, no workspace:*). Versions track the
    // monorepo package versions; operator npm publish is out of band (E8).
    // Typecheck is `sail types --check` (native sidecar). oxlint is optional
    // after install — not what turns types on.
    let pkg_json = format!(
        "{{\n  \
         \"name\": \"{basen}\",\n  \
         \"private\": true,\n  \
         \"version\": \"0.1.0\",\n  \
         \"type\": \"module\",\n  \
         \"description\": \"Sail app scaffolded by `sail create`.\",\n  \
         \"scripts\": {{\n    \
         \"dev\": \"sail dev\",\n    \
         \"build\": \"sail build --bundle-framework\",\n    \
         \"check\": \"sail check .\",\n    \
         \"typecheck\": \"sail types --check\",\n    \
         \"lint\": \"oxlint .\"\n  \
         }},\n  \
         \"dependencies\": {{\n    \
         \"@ship/sail-runtime\": \"^0.3.0\",\n    \
         \"@ship/sail-dom\": \"^0.3.0\",\n    \
         \"@ship/tokens\": \"^0.3.0\"\n  \
         }},\n  \
         \"devDependencies\": {{\n    \
         \"oxlint\": \"^1.77.0\"\n  \
         }}\n\
         }}\n"
    );
    fs::write(app_dir.join("package.json"), pkg_json)
        .with_context(|| format!("write {}/package.json", app_dir.display()))?;

    // Hello-world page — legal under every styling policy (layout + typography).
    let title = title_case(basen);
    let page = format!(
        "<column gap=\"4\">\n  \
         <h1>{title}</h1>\n  \
         <p>Sail app ready. Edit routes/page.sail.</p>\n\
         </column>\n"
    );
    fs::write(app_dir.join("routes").join("page.sail"), page)
        .with_context(|| format!("write {}/routes/page.sail", app_dir.display()))?;

    // Theme SSOT from the baked catalog (no monorepo tree required). Prefer
    // disk catalog when this CLI is run inside the monorepo so forks stay live.
    let catalog_dir = find_repo_root(&app_dir)
        .or_else(|_| find_repo_root(&cwd))
        .ok()
        .map(|root| root.join("packages/sail/tokens/model/themes"))
        .filter(|p| p.is_dir());
    let theme_id = sail_compiler::tokens_generate::DEFAULT_PROJECT_THEME_ID;
    let theme_dest = sail_compiler::tokens_generate::scaffold_default_project_theme(
        &app_dir,
        catalog_dir.as_deref(),
        Some(theme_id),
    )?;
    let _ = sail_compiler::tokens_generate::scaffold_theme_config(&app_dir, theme_id)?;

    // .gitignore with dist/ + .sail/ (S0 / project_store).
    sail_ops::ensure_project_store(&app_dir)
        .map_err(|e| anyhow!("ensure project store (.gitignore / .sail): {e}"))?;

    // Ambient pack + pure/action kind tsconfigs under .sail/ +
    // solution root tsconfig.json + oxlint (G2c / TOOLING-1 WI2). Zero hand
    // edits; regenerate with `sail types` / build / dev after install.
    let _ = sail_compiler::ambients::write_project_type_configs(&app_dir)?;
    let _ = sail_compiler::ambients::write_project_oxlintrc(&app_dir)?;

    let rel = app_dir
        .strip_prefix(&cwd)
        .unwrap_or(&app_dir)
        .display()
        .to_string();
    println!("[create] {rel}/");
    println!("[create]   package.json");
    println!("[create]   sail.config.jsonc");
    println!("[create]   routes/page.sail");
    println!(
        "[create]   {}",
        theme_dest
            .strip_prefix(&app_dir)
            .unwrap_or(&theme_dest)
            .display()
    );
    println!("[create]   .gitignore  (includes dist/ + .sail/)");
    println!(
        "[create]   tsconfig.json (solution)  .sail/{{types.d.ts,tsconfig.*.json}}  .oxlintrc.json"
    );
    println!("[create] next steps:");
    println!("  1. cd {rel}");
    println!("  2. sail check .");
    println!("  3. sail types --check");
    println!("  4. sail dev");
    Ok(())
}

/// Copy one Sail catalog theme into the project's `themes/` and seed a thin
/// `sail.config.jsonc` theme section when the project has none.
///
/// Theme source is the **baked** catalog (no monorepo `packages/` walk). When
/// a monorepo disk model is present under cwd, it is preferred so catalog
/// development forks the live tree (`sail tokens --model` story).
pub fn run_theme(project: PathBuf, theme_id: Option<String>) -> Result<()> {
    let cwd = std::env::current_dir().context("cwd")?;
    let project_dir = if project.is_absolute() {
        project
    } else {
        cwd.join(project)
    };
    if !project_dir.is_dir() {
        bail!("project dir does not exist: {}", project_dir.display());
    }
    // Prefer --id; else existing sail.config theme.default; else sail (modes-in-file).
    let id = match theme_id {
        Some(id) => id,
        None => sail_compiler::tokens_generate::load_theme_config(&project_dir)?
            .default_theme_id()
            .to_string(),
    };
    sail_compiler::tokens_generate::validate_theme_id(&id)?;

    // Disk catalog when monorepo model is on disk (cwd or project ancestor via
    // cargo root); otherwise `None` → baked catalog body.
    let catalog_dir = find_repo_root(&project_dir)
        .or_else(|_| find_repo_root(&cwd))
        .ok()
        .map(|root| root.join("packages/sail/tokens/model/themes"))
        .filter(|p| p.is_dir());

    let dest = sail_compiler::tokens_generate::scaffold_default_project_theme(
        &project_dir,
        catalog_dir.as_deref(),
        Some(&id),
    )?;
    println!(
        "[scaffold] theme → {}",
        dest.strip_prefix(&project_dir).unwrap_or(&dest).display()
    );
    match sail_compiler::tokens_generate::scaffold_theme_config(&project_dir, &id)? {
        Some(cfg) => {
            println!(
                "[scaffold] config → {}",
                cfg.strip_prefix(&project_dir).unwrap_or(&cfg).display()
            );
        }
        None => {
            println!(
                "[scaffold] sail.config already present — left theme section untouched \
                 (set theme.default / theme.enabled by hand if needed)"
            );
        }
    }
    // S0: ensure app .gitignore + short `.sail/README.md` so build/tasks never commit.
    sail_ops::ensure_project_store(&project_dir)
        .map_err(|e| anyhow!("ensure project store (.gitignore / .sail): {e}"))?;
    println!("[scaffold] ensured .gitignore + .sail/README.md");
    println!("[scaffold] next steps:");
    println!("  1. Edit themes/{id}.json — project DTCG SSOT");
    println!(
        "  2. sail dev — app-local .sail/themes/themes.css from baked catalog + \
         project themes (monorepo also regenerates package delivery when present)"
    );
    Ok(())
}
