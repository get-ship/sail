//! `deck scaffold view|component` — generate boilerplate for a new SUI view
//! or primitive. Ported verbatim from the retired
//! `scripts/scaffold-view.mjs` + `scripts/scaffold-sui-component.mjs`.
//!
//! Scaffolding is a deck capability because deck owns the app/component
//! shape (manifest fields, paired-controller convention, the `.sui` +
//! `.controller.ts` stem pair). The retired .mjs called into `node` and
//! pointed at since-renamed paths (`packages/ui-components/src/`); the
//! port writes to the current layout (`packages/components/sui/primitives/`)
//! and the next-steps text reflects today's toolchain.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};

/// Validate `name` against `/^[a-z][a-z0-9-]*$/` — the route-segment shape
/// the daemon resolver expects and the file-system shape we write.
fn validate_view_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("view name is required");
    }
    let first = name.chars().next().unwrap();
    if !first.is_ascii_lowercase() {
        bail!("view name must start with a lowercase letter — got {name:?}");
    }
    if !name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        bail!("view name must match /^[a-z][a-z0-9-]*$/ — got {name:?}");
    }
    Ok(())
}

/// Same shape as `validate_view_name` plus the `sui-` prefix the runtime
/// custom-element registry expects for primitives.
fn validate_component_tag(tag: &str) -> Result<()> {
    if !tag.starts_with("sui-") {
        bail!("component tag must start with `sui-` — got {tag:?}");
    }
    let rest = &tag[4..];
    if rest.is_empty()
        || !rest.starts_with(|c: char| c.is_ascii_lowercase())
        || !rest.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        bail!("component tag must match /^sui-[a-z][a-z0-9-]*$/ — got {tag:?}");
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
    deck_bundler::paths::topmost_cargo_root(start).ok_or_else(|| {
        anyhow!("no Cargo workspace root above {}", start.display())
    })
}

pub fn run_view(name: String, route_path: Option<String>, component_tag: Option<String>) -> Result<()> {
    validate_view_name(&name)?;
    let route = route_path.unwrap_or_else(|| format!("/{name}"));
    if !route.starts_with('/') {
        bail!("--path must start with `/` — got {route:?}");
    }
    if let Some(tag) = component_tag.as_deref() {
        let prefix = format!("{name}-");
        if !tag.starts_with(&prefix) {
            bail!(
                "--component {tag:?} must be prefixed with view name `{name}-` to avoid registry collisions"
            );
        }
    }

    let repo_root = find_repo_root(&std::env::current_dir().context("cwd")?)?;
    let base = repo_root.join("views").join(&name);
    if base.exists() {
        bail!("directory already exists: {}", base.display());
    }
    fs::create_dir_all(&base).with_context(|| format!("mkdir {}", base.display()))?;

    let title = title_case(&name);
    let manifest = format!(
        "// Route metadata for views/{name}/. See\n\
         // docs/ship-adrs/references/docs/view-directory-convention.md.\n\
         {{\n  \
         \"name\": \"{name}\",\n  \
         \"path\": \"{route}\",\n  \
         \"title\": \"{title}\",\n  \
         \"layout\": \"default\",\n  \
         \"auth\": \"public\"\n\
         }}\n"
    );
    let page = format!(
        "<div class=\"{name}\">\n  \
         <h1>{title}</h1>\n  <p>New view — replace this body.</p>\n</div>\n"
    );
    let controller = format!(
        "// {name} view — tier-1 stub. Replace with state, derived, hooks, lifecycle as\n\
         // the view grows. See .ship/skills/ship-views/SKILL.md.\n\nexport default null\n"
    );
    let style = format!(
        ".{name} {{\n  max-width: 36rem;\n  margin: 4rem auto;\n  \
         font-family: system-ui, sans-serif;\n}}\n\
         .{name} h1 {{ margin: 0 0 0.5rem; font-size: 1.5rem; }}\n"
    );
    fs::write(base.join("manifest.jsonc"), manifest)?;
    fs::write(base.join("page.sui"), page)?;
    fs::write(base.join("controller.ts"), controller)?;
    fs::write(base.join("style.css"), style)?;

    if let Some(tag) = component_tag {
        let tag_dir = base.join("_components").join(&tag);
        fs::create_dir_all(&tag_dir)?;
        let comp_sui = format!(
            "<!--\n  \
             {tag} — view-private widget for views/{name}/.\n\n  \
             Tag prefix matches the parent view name so the runtime can register it\n  \
             without colliding with any other view's _components/ tree.\n\
             -->\n\n\
             <prop name=\"tone\" values=\"default|accent\" default=\"default\" reflect />\n\n\
             <style>\n  :host {{ display: block; }}\n  \
             .root {{ padding: 1rem; border: 1px solid var(--border, #d0d7de); border-radius: 0.5rem; }}\n\
             </style>\n\n\
             <div class=\"root\" part=\"root\"><slot></slot></div>\n"
        );
        let comp_ctrl = format!(
            "// {tag} — view-private widget controller. Tier-1 stub.\n\nexport default null\n"
        );
        fs::write(tag_dir.join(format!("{tag}.sui")), comp_sui)?;
        fs::write(tag_dir.join(format!("{tag}.controller.ts")), comp_ctrl)?;
    }

    println!("[scaffold] created views/{name}/");
    println!("[scaffold]   page.sui  controller.ts  style.css  manifest.jsonc");
    println!("[scaffold] next steps:");
    println!("  1. Edit page.sui — replace placeholder body");
    println!("  2. Edit manifest.jsonc — set rail/preload/events as needed");
    println!("  3. Reload the shell — views auto-mount via the daemon");
    Ok(())
}

pub fn run_component(tag: String, want_controller: bool) -> Result<()> {
    validate_component_tag(&tag)?;
    let repo_root = find_repo_root(&std::env::current_dir().context("cwd")?)?;
    // Primitives live under packages/components/sui/primitives/src/ today — the
    // retired .mjs wrote to packages/ui-components/src/ which no longer exists.
    let base = repo_root.join("packages/components/sui/primitives/src").join(&tag);
    if base.exists() {
        bail!("directory already exists: {}", base.display());
    }
    fs::create_dir_all(&base)?;

    let sui = format!(
        "<!--\n  \
         {tag} — SUI primitive.\n\n  \
         Parity vs Lit <ship-{stem}>:\n    \
         Props:    (list)\n    Slots:    (list)\n    Events:   (list)\n    \
         Methods:  (list)\n    ARIA:     (list)\n\
         -->\n\n\
         <prop name=\"tone\" values=\"default|primary|info|success|warning|destructive|accent|muted\" default=\"default\" reflect />\n\n\
         <style>\n  \
         :host {{ display: inline-flex; vertical-align: middle; --tone-fg: var(--foreground); }}\n  \
         :host([tone='primary'])     {{ --tone-fg: var(--primary); }}\n  \
         :host([tone='info'])        {{ --tone-fg: var(--info); }}\n  \
         :host([tone='success'])     {{ --tone-fg: var(--success); }}\n  \
         :host([tone='warning'])     {{ --tone-fg: var(--warning); }}\n  \
         :host([tone='destructive']) {{ --tone-fg: var(--destructive); }}\n  \
         :host([tone='accent'])      {{ --tone-fg: var(--accent); }}\n  \
         :host([tone='muted'])       {{ --tone-fg: var(--muted-foreground); }}\n\n  \
         .root {{ color: var(--tone-fg); }}\n\
         </style>\n\n\
         <span class=\"root\" part=\"root\"><slot></slot></span>\n",
        stem = tag.strip_prefix("sui-").unwrap_or(&tag),
    );
    fs::write(base.join(format!("{tag}.sui")), sui)?;

    if want_controller {
        let ts = format!(
            "import {{ controller }} from '@ship/sui-runtime'\n\
             import type {{ Scope }} from './{tag}.sui'\n\n\
             export default controller<Scope>({{\n  \
             mount: (_state, _host) => {{\n    return () => {{}}\n  }},\n\
             }})\n"
        );
        fs::write(base.join(format!("{tag}.controller.ts")), ts)?;
    }

    println!("[scaffold] created packages/components/sui/primitives/src/{tag}/");
    println!("[scaffold] next steps:");
    println!("  1. Edit {tag}.sui — props, styles, template body, parity comment");
    println!("  2. Run: cargo run -p deck-cli -- build-assets");
    println!("     (deck compiles the .sui in-process via the shared SuiPlugin)");
    Ok(())
}

/// Validate a pack name — lowercase ASCII + digits + hyphens, starts with a
/// letter. Becomes both the directory name and the npm package suffix
/// (`@ship/<name>`).
fn validate_pack_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("pack name is required");
    }
    if !name.starts_with(|c: char| c.is_ascii_lowercase()) {
        bail!("pack name must start with a lowercase letter — got {name:?}");
    }
    if !name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        bail!("pack name must match /^[a-z][a-z0-9-]*$/ — got {name:?}");
    }
    Ok(())
}

pub fn run_pack(name: String) -> Result<()> {
    validate_pack_name(&name)?;
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
         \"description\": \"Ship {name} pack — SUI components registry. \
         Author tags under src/<tag>/<tag>.sui; the umbrella in src/index.ts side-effect-imports each compiled.js so a single `import \\\"@ship/{name}\\\"` registers them all.\",\n  \
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
         \"@ship/sui-dom\": \"workspace:*\",\n    \
         \"@ship/sui-runtime\": \"workspace:*\"\n  \
         }},\n  \
         \"devDependencies\": {{\n    \
         \"vitest\": \"^4.1.5\"\n  \
         }}\n\
         }}\n"
    );
    fs::write(base.join("package.json"), pkg_json)?;

    let tsconfig = format!(
        "{{\n  \
         \"compilerOptions\": {{\n    \
         \"target\": \"ES2022\",\n    \
         \"module\": \"ESNext\",\n    \
         \"moduleResolution\": \"Bundler\",\n    \
         \"lib\": [\"ES2022\", \"DOM\", \"DOM.Iterable\"],\n    \
         \"strict\": true,\n    \
         \"noUncheckedIndexedAccess\": true,\n    \
         \"exactOptionalPropertyTypes\": true,\n    \
         \"declaration\": true,\n    \
         \"outDir\": \"dist\",\n    \
         \"rootDir\": \"src\",\n    \
         \"skipLibCheck\": true,\n    \
         \"esModuleInterop\": true,\n    \
         \"isolatedModules\": true,\n    \
         \"verbatimModuleSyntax\": true,\n    \
         \"noEmit\": false\n  \
         }},\n  \
         \"include\": [\"src/**/*\"],\n  \
         \"exclude\": [\"test\", \"dist\"]\n\
         }}\n"
    );
    fs::write(base.join("tsconfig.json"), tsconfig)?;

    let index_ts = format!(
        "// @ship/{name} — SUI component registry.\n\
         //\n\
         // Side-effect imports for every tag in the pack. Each\n\
         // `<tag>.compiled.js` calls `customElements.define()` on import,\n\
         // so a single `import '@ship/{name}'` at the consumer registers\n\
         // every tag in this pack globally. Add new tags below as you\n\
         // scaffold them.\n\n\
         import './{name}-example/{name}-example.compiled.js'\n"
    );
    fs::write(base.join("src/index.ts"), index_ts)?;

    let example_sui = format!(
        "<!--\n  \
         {name}-example — placeholder primitive for the @ship/{name} pack.\n\n  \
         Replace or delete this when you author the pack's first real tag.\n\
         -->\n\n\
         <prop name=\"label\" type=\"string\" default=\"hello\" reflect />\n\n\
         <style>\n  \
         :host {{ display: inline-block; padding: 0.5em 1em; \n         \
         border: 1px solid var(--border, #d0d7de); border-radius: 0.5em; \n         \
         font-family: system-ui, sans-serif; color: var(--foreground); }}\n\
         </style>\n\n\
         <span part=\"root\">{{label()}}</span>\n"
    );
    fs::write(
        base.join(format!("src/{name}-example/{name}-example.sui")),
        example_sui,
    )?;

    let example_controller = format!(
        "// {name}-example — placeholder controller; replace alongside the .sui.\n\n\
         export default null\n"
    );
    fs::write(
        base.join(format!("src/{name}-example/{name}-example.controller.ts")),
        example_controller,
    )?;

    println!("[scaffold] created packages/components/{name}/");
    println!("[scaffold]   package.json  tsconfig.json");
    println!("[scaffold]   src/index.ts  src/{name}-example/<.sui + .controller.ts>");
    println!("[scaffold] next steps:");
    println!("  1. bun install — wires the new workspace package.");
    println!("  2. Replace the {name}-example placeholder with the pack's first tag.");
    println!("  3. (When ready to consume) Add `@ship/{name}: workspace:*` to the");
    println!("     consuming app + `import '@ship/{name}'` where the tags are used —");
    println!("     deck traces the tags into that entry's chunk (no separate build).");
    Ok(())
}
