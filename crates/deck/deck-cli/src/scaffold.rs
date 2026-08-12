//! `deck scaffold route|component` — generate boilerplate for a new SAIL route
//! or primitive. Ported from the retired
//! `scripts/scaffold-route.mjs` + `scripts/scaffold-sail-component.mjs`.
//!
//! Scaffolding is a deck capability because deck owns the app/component
//! shape (the single-file `.sail` convention — state lives in an inline
//! `<controller>` block). A route is just a directory with a `page.sail`;
//! deck derives the URL from the tree (no manifest, no host metadata). The
//! retired .mjs called into `node` and
//! pointed at since-renamed paths (`packages/ui-components/src/`); the
//! port writes to the current layout (`packages/components/sail/primitives/`)
//! and the next-steps text reflects today's toolchain.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};

/// Validate `name` against `/^[a-z][a-z0-9-]*$/` — the route-segment shape
/// the daemon resolver expects and the file-system shape we write.
fn validate_route_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("route name is required");
    }
    let first = name.chars().next().unwrap();
    if !first.is_ascii_lowercase() {
        bail!("route name must start with a lowercase letter — got {name:?}");
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        bail!("route name must match /^[a-z][a-z0-9-]*$/ — got {name:?}");
    }
    Ok(())
}

/// Same shape as `validate_route_name` plus the `sail-` prefix the runtime
/// custom-element registry expects for primitives.
fn validate_component_tag(tag: &str) -> Result<()> {
    let Some(rest) = deck_core::strip_sail_tag_prefix(tag) else {
        bail!(
            "component tag must start with `{}` — got {tag:?}",
            deck_core::SAIL_TAG_PREFIX
        );
    };
    if !rest.starts_with(|c: char| c.is_ascii_lowercase())
        || !rest
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        bail!("component tag must match /^sail-[a-z][a-z0-9-]*$/ — got {tag:?}");
    }
    Ok(())
}

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

/// Resolve the Cargo workspace root from `start`. Lets `deck scaffold` work
/// from any cwd inside the repo; the views/ + deck-core monorepo markers
/// surface naturally when the scaffold tries to write into those paths.
fn find_repo_root(start: &Path) -> Result<PathBuf> {
    deck_bundler::paths::topmost_cargo_root(start)
        .ok_or_else(|| anyhow!("no Cargo workspace root above {}", start.display()))
}

pub fn run_route(
    name: String,
    route_path: Option<String>,
    component_tag: Option<String>,
) -> Result<()> {
    validate_route_name(&name)?;
    let route = route_path.unwrap_or_else(|| format!("/{name}"));
    if !route.starts_with('/') {
        bail!("--path must start with `/` — got {route:?}");
    }
    if let Some(tag) = component_tag.as_deref() {
        let prefix = format!("{name}-");
        if !tag.starts_with(&prefix) {
            bail!(
                "--component {tag:?} must be prefixed with route name `{name}-` to avoid registry collisions"
            );
        }
    }

    // `route` is validated (`/`-prefixed) above; deck derives the URL from the
    // directory tree, so the scaffold no longer records it in a manifest.
    let _ = &route;
    let repo_root = find_repo_root(&std::env::current_dir().context("cwd")?)?;
    let base = repo_root.join("routes").join(&name);
    if base.exists() {
        bail!("directory already exists: {}", base.display());
    }
    fs::create_dir_all(&base).with_context(|| format!("mkdir {}", base.display()))?;

    let title = title_case(&name);
    // Tier-1 stub: layout element + typography, no sx, no CSS — legal under
    // every styling policy. State arrives later as an inline `<controller>`.
    // A route is simply a directory with a `page.sail` — deck discovers the URL
    // from the tree (a `[param]` dir is a dynamic). No manifest, no host
    // metadata: `auth`/`rail`/`layout`/`title` are a Ship-shell concern, not
    // deck's (see the ship-ui handoff).
    let page = format!(
        "<column gap=\"4\">\n  \
         <h1>{title}</h1>\n  <p>New route — replace this body.</p>\n</column>\n"
    );
    fs::write(base.join("page.sail"), page)?;

    if let Some(tag) = component_tag {
        let tag_dir = base.join("_components").join(&tag);
        fs::create_dir_all(&tag_dir)?;
        let comp_sail = format!(
            "<!--\n  \
             {tag} — route-private widget for routes/{name}/.\n\n  \
             Tag prefix matches the parent route name so the runtime can register it\n  \
             without colliding with any other route's _components/ tree.\n\
             -->\n\n\
             <scope>\n  props: {{ tone = 'default' }}\n</scope>\n\n\
             <card tone={{tone()}}>\n  <slot></slot>\n</card>\n"
        );
        fs::write(tag_dir.join(format!("{tag}.sail")), comp_sail)?;
    }

    println!("[scaffold] created routes/{name}/");
    println!("[scaffold]   page.sail");
    println!("[scaffold] next steps:");
    println!("  1. Edit page.sail — replace placeholder body");
    println!("  2. Add an inline <controller> block when the route needs state");
    println!("  3. Reload the app — deck discovers the route from the tree");
    Ok(())
}

pub fn run_component(tag: String, want_controller: bool) -> Result<()> {
    validate_component_tag(&tag)?;
    let repo_root = find_repo_root(&std::env::current_dir().context("cwd")?)?;
    // Primitives live under packages/components/sail/primitives/src/ today — the
    // retired .mjs wrote to packages/ui-components/src/ which no longer exists.
    let base = repo_root
        .join("packages/components/sail/primitives/src")
        .join(&tag);
    if base.exists() {
        bail!("directory already exists: {}", base.display());
    }
    fs::create_dir_all(&base)?;

    // Current-generation stub: `<scope>` declares the static surface (the
    // bare `tone` default opts into the shared tone vocabulary), the body is
    // layout elements + sx, and state — when asked for — is an inline
    // `<controller>` block in the same file. No siblings, no `<style>`,
    // no `:host`: the compiler rejects all three.
    let mut sail = format!(
        "<!--\n  \
         {tag} — SAIL primitive.\n\n  \
         Props / slots / events / ARIA: document as they land.\n\
         -->\n\n\
         <scope>\n  props: {{ tone = 'default' }}\n</scope>\n\n"
    );
    if want_controller {
        // Ambient idents (controller, signal, …) need no import; types do.
        // Never scaffold `() => ({})` — that is a compile error
        // (`sail-controller-empty`). Seed a trivial signal so the stub is a
        // real controller the author can grow, or delete the whole block.
        sail.push_str("<controller>\nimport type { ControllerScope } from '@ship/sail-runtime'\n");
        sail.push_str(&format!("import type {{ Scope }} from './{tag}.sail'\n\n"));
        sail.push_str(
            "export default controller((_ctx: ControllerScope<Scope>) => {\n  \
             const ready = signal(true)\n  \
             return { ready }\n\
             })\n</controller>\n\n",
        );
    }
    sail.push_str(
        "<row align=\"center\" gap=\"1\" style={style.fg(foreground)}>\n  <slot></slot>\n</row>\n",
    );
    fs::write(base.join(format!("{tag}.sail")), sail)?;

    println!("[scaffold] created packages/components/sail/primitives/src/{tag}/");
    println!("[scaffold] next steps:");
    println!("  1. Edit {tag}.sail — scope props, template body");
    println!("  2. Run: cargo run -p deck-cli -- build-assets");
    println!("     (deck compiles the .sail in-process via the shared SailPlugin)");
    Ok(())
}

/// Validate a package name — lowercase ASCII + digits + hyphens, starts with a
/// letter. Becomes both the directory name and the npm package suffix
/// (`@ship/<name>`).
fn validate_components_package_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("package name is required");
    }
    if !name.starts_with(|c: char| c.is_ascii_lowercase()) {
        bail!("package name must start with a lowercase letter — got {name:?}");
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        bail!("package name must match /^[a-z][a-z0-9-]*$/ — got {name:?}");
    }
    Ok(())
}

pub fn run_components_package(name: String) -> Result<()> {
    validate_components_package_name(&name)?;
    let repo_root = find_repo_root(&std::env::current_dir().context("cwd")?)?;
    let base = repo_root.join("packages/components").join(&name);
    if base.exists() {
        bail!("directory already exists: {}", base.display());
    }
    fs::create_dir_all(base.join("src").join(format!("{name}-example")))?;

    let pkg_json = format!(
        "{{\n  \
         \"name\": \"@ship/{name}\",\n  \
         \"version\": \"0.1.0\",\n  \
         \"description\": \"Ship {name} components package — SAIL components registry. \
         Author tags under src/<tag>/<tag>.sail; the umbrella in src/index.ts side-effect-imports each compiled.js so a single `import \\\"@ship/{name}\\\"` registers them all.\",\n  \
         \"type\": \"module\",\n  \
         \"main\": \"src/index.ts\",\n  \
         \"types\": \"src/index.ts\",\n  \
         \"exports\": {{\n    \
         \".\": \"./src/index.ts\"\n  \
         }},\n  \
         \"scripts\": {{\n    \
         \"test\": \"vitest run\",\n    \
         \"test:watch\": \"vitest\"\n  \
         }},\n  \
         \"dependencies\": {{\n    \
         \"@ship/sail-dom\": \"workspace:*\",\n    \
         \"@ship/sail-runtime\": \"workspace:*\"\n  \
         }},\n  \
         \"devDependencies\": {{\n    \
         \"vitest\": \"^4.1.5\"\n  \
         }}\n\
         }}\n"
    );
    fs::write(base.join("package.json"), pkg_json)?;

    // Align with packages/sail/tsconfig/tsconfig.base.json (TOOLING-1 WI2)
    // plus package emit flags (declaration/outDir). Inline so a freshly
    // scaffolded components package typechecks without an extra workspace dep.
    let tsconfig = "{\n  \
         \"compilerOptions\": {\n    \
         \"target\": \"ESNext\",\n    \
         \"module\": \"ESNext\",\n    \
         \"moduleResolution\": \"Bundler\",\n    \
         \"lib\": [\"ESNext\", \"DOM\", \"DOM.Iterable\"],\n    \
         \"strict\": true,\n    \
         \"noUncheckedIndexedAccess\": true,\n    \
         \"exactOptionalPropertyTypes\": true,\n    \
         \"noImplicitOverride\": true,\n    \
         \"verbatimModuleSyntax\": true,\n    \
         \"erasableSyntaxOnly\": true,\n    \
         \"isolatedModules\": true,\n    \
         \"declaration\": true,\n    \
         \"outDir\": \"dist\",\n    \
         \"rootDir\": \"src\",\n    \
         \"skipLibCheck\": true,\n    \
         \"esModuleInterop\": true,\n    \
         \"noEmit\": false\n  \
         },\n  \
         \"include\": [\"src/**/*\"],\n  \
         \"exclude\": [\"test\", \"dist\"]\n\
         }\n";
    fs::write(base.join("tsconfig.json"), tsconfig)?;

    let index_ts = format!(
        "// @ship/{name} — SAIL component registry.\n\
         //\n\
         // Side-effect imports for every tag in the package. Each\n\
         // `<tag>.compiled.js` calls `customElements.define()` on import,\n\
         // so a single `import '@ship/{name}'` at the consumer registers\n\
         // every tag in this package globally. Add new tags below as you\n\
         // scaffold them.\n\n\
         import './{name}-example/{name}-example.compiled.js'\n"
    );
    fs::write(base.join("src/index.ts"), index_ts)?;

    let example_sail = format!(
        "<!--\n  \
         {name}-example — placeholder primitive for the @ship/{name} components package.\n\n  \
         Replace or delete this when you author the package's first real tag.\n\
         -->\n\n\
         <scope>\n  props: {{ label = 'hello' }}\n</scope>\n\n\
         <row align=\"center\" sx=\"px-3 py-2 border border-border rounded-md\">{{label()}}</row>\n"
    );
    fs::write(
        base.join(format!("src/{name}-example/{name}-example.sail")),
        example_sail,
    )?;

    println!("[scaffold] created packages/components/{name}/");
    println!("[scaffold]   package.json  tsconfig.json");
    println!("[scaffold]   src/index.ts  src/{name}-example/{name}-example.sail");
    println!("[scaffold] next steps:");
    println!("  1. bun install — wires the new workspace package.");
    println!("  2. Replace the {name}-example placeholder with the package's first tag.");
    println!("  3. (When ready to consume) Add `@ship/{name}: workspace:*` to the");
    println!("     consuming app + `import '@ship/{name}'` where the tags are used —");
    println!("     deck traces the tags into that entry's chunk (no separate build).");
    Ok(())
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
///   themes/ship.json     # modern project theme SSOT (baked catalog)
///   .gitignore           # includes .deck/ (build output — never commit)
/// ```
///
/// Also seeds `node_modules/.sail/types.d.ts`, strict `tsconfig.json`
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
    // typescript + oxlint as devDeps so a fresh app type-checks / lints with
    // zero hand edits after install (TOOLING-1 WI2).
    let pkg_json = format!(
        "{{\n  \
         \"name\": \"{basen}\",\n  \
         \"private\": true,\n  \
         \"version\": \"0.1.0\",\n  \
         \"type\": \"module\",\n  \
         \"description\": \"Sail app scaffolded by `sail create`.\",\n  \
         \"scripts\": {{\n    \
         \"dev\": \"sail dev\",\n    \
         \"build\": \"sail build\",\n    \
         \"check\": \"sail check .\",\n    \
         \"typecheck\": \"tsc --noEmit -p tsconfig.json\",\n    \
         \"lint\": \"oxlint .\"\n  \
         }},\n  \
         \"dependencies\": {{\n    \
         \"@ship/sail-runtime\": \"^0.3.0\",\n    \
         \"@ship/sail-dom\": \"^0.3.0\",\n    \
         \"@ship/tokens\": \"^0.3.0\"\n  \
         }},\n  \
         \"devDependencies\": {{\n    \
         \"typescript\": \"~5.8.3\",\n    \
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

    // .gitignore with .deck/ + .sail/README.md (S0 / project_store).
    sail_ops::ensure_project_store(&app_dir)
        .map_err(|e| anyhow!("ensure project store (.gitignore / .sail): {e}"))?;

    // Ambient pack + strict tsconfig + oxlint so `tsc` / `oxlint` work with
    // zero hand edits (TOOLING-1 WI2). Pack lives under `node_modules/.sail/`
    // (SPEC-OUTPUT-1 §4); regenerate with `sail types` after install.
    let _ = sail_compiler::ambients::write_project_types(&app_dir)?;
    let tsconfig = sail_compiler::ambients::emit_project_tsconfig_json(&[
        "src/**/*.ts",
        "routes/**/*.ts",
    ]);
    fs::write(app_dir.join("tsconfig.json"), tsconfig)
        .with_context(|| format!("write {}/tsconfig.json", app_dir.display()))?;
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
    println!("[create]   .gitignore  (includes .deck/)");
    println!("[create]   tsconfig.json  node_modules/.sail/types.d.ts  .oxlintrc.json");
    println!("[create] next steps:");
    println!("  1. cd {rel}");
    println!("  2. bun install   # after @ship packages are published");
    println!("  3. bun run typecheck && bun run lint");
    println!("  4. sail dev");
    println!("  5. sail check .");
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
    // Prefer --id; else existing sail.config theme.default; else ship (modes-in-file).
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
        "  2. sail dev — app-local .deck/themes/themes.css from baked catalog + \
         project themes (monorepo also regenerates package delivery when present)"
    );
    Ok(())
}
