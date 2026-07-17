//! Typed dynamic-route codegen (#168) — deck owns route discovery, since deck
//! builds the app. Walks an app's `routes/` view tree for `[param]` route dirs
//! and emits `routes.gen.ts` (the `RouteShape[]` the shell registers via
//! `defineRoutes`, plus typed path builders). Ported from the retired
//! `scripts/gen-routes.ts`; run by `deck build` (and `deck routes`).
//!
//! Multi-root: a single invocation can walk several roots (the shell's first-
//! party `routes/`, the spaces scaffold's `routes/`) and emit a `routes.gen.ts`
//! at each requested output path. Every root is a `routes/` dir — a route IS a
//! directory with a `page.sail` and (optionally) `controller.ts`. A `[param]`
//! segment in the dir name is a dynamic. No manifests, no per-route metadata.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

/// One codegen spec: walk `dir` recursively for mountable views (`page.sail`),
/// emit the generated `routes.gen.ts` content to `out`. `label` is the
/// human-readable surface the file is for (e.g. `shell`, `spaces`) — it shows
/// up in the file header so the file is self-describing.
#[derive(Debug, Clone)]
pub struct RouteGenSpec {
    pub dir: PathBuf,
    pub out: PathBuf,
    pub label: String,
    /// Customizes the always-emitted SPA bootstrap (`entry.gen.ts`). The
    /// codegen ALWAYS writes `entry.gen.ts` (next to the data-only
    /// `routes.gen.ts`) — the bootstrap that imports the data tables, the
    /// token CSS, and calls `mountRouter`. When `Some` (a host
    /// `deck.jsonc#entry`), the host's named `clients` are imported off its
    /// `clientsModule` and spread into the `clients` map; when `None`
    /// (standalone), no clients are wired and `mountRouter` runs with the
    /// routes/layouts/modules alone — deck's default bootstrap stands on its
    /// own.
    pub entry_bootstrap: Option<deck_config::EntryBootstrap>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Route {
    pub(crate) pattern: String,
    pub(crate) params: Vec<String>,
    pub(crate) builder: String,
}

/// A `_layout.sail` discovered at the top level of a `routes/` dir. The
/// runtime wraps the subtree's content in the layout component; the layout
/// template has a `<slot>` that the route content fills.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Layout {
    pub(crate) path_prefix: String,
    pub(crate) tag: String,
}

/// Recursively collect dirs holding a mountable view (`page.sail`).
/// The root `dir` itself is also a candidate — `routes/page.sail`
/// is a mount at `/` (the index route). Subdirs are walked for
/// nested mounts.
fn view_dirs(dir: &Path, acc: &mut Vec<std::path::PathBuf>) -> Result<()> {
    if dir.join("page.sail").is_file() {
        acc.push(dir.to_path_buf());
    }
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            view_dirs(&entry.path(), acc)?;
        }
    }
    Ok(())
}
/// `studio/agents/[id]/settings` → pattern `/studio/agents/:id/settings`,
/// params `[id]`, builder `studioAgentsIdSettings`. Static routes (no
/// `[param]`) return `None` — they need no `defineRoutes` entry.
///
/// The first segment of the relative path is preserved as the URL's
/// first segment (e.g. `studio`, `home`, anything under the routes root).
/// The codegen is fs-routing at the app level; it has no opinion about
/// which segments are valid route roots — every dir under the routes root
/// that contains a mountable view is a route.
fn to_route(routes_dir: &Path, dir: &Path) -> Option<Route> {
    let rel = dir.strip_prefix(routes_dir).ok()?;
    let segs_raw: Vec<String> =
        rel.components().map(|c| c.as_os_str().to_string_lossy().into_owned()).collect();
    let mut params = Vec::new();
    let pattern_segs: Vec<String> = segs_raw
        .iter()
        .map(|s| {
            if let Some(p) = s.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                params.push(p.to_string());
                format!(":{p}")
            } else {
                s.clone()
            }
        })
        .collect();
    let pattern = if pattern_segs.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", pattern_segs.join("/"))
    };
    // Builder name: camelCase of the segments, including the first
    // segment (the top-level route name) so builders namespace per
    // route. e.g. `studio/agents/[id]` → `studioAgentsId`. The root
    // `routes/page.sail` (a mount at `/`) gets the reserved `index`
    // name — a single segment is too short to camelCase meaningfully.
    let builder = if segs_raw.is_empty() {
        "index".to_string()
    } else {
        let joined = segs_raw
            .iter()
            .map(|s| s.trim_start_matches('[').trim_end_matches(']'))
            .collect::<Vec<_>>()
            .join("-");
        camel_case(&joined)
    };
    Some(Route { pattern, params, builder })
}

/// Reconstruct a route's view-module import path from its URL pattern — the
/// inverse of `to_route`'s segment mapping (`:param` ↔ `[param]` dir):
/// `/studio/agents/:id` → `./routes/studio/agents/[id]/page.sail`; `/` →
/// `./routes/page.sail`. The emitter uses it to statically import each view.
fn module_import_path(pattern: &str) -> String {
    let segs: Vec<String> = pattern
        .split('/')
        .filter(|s| !s.is_empty())
        .map(|s| {
            s.strip_prefix(':')
                .map(|p| format!("[{p}]"))
                .unwrap_or_else(|| s.to_string())
        })
        .collect();
    if segs.is_empty() {
        "./routes/page.sail".to_string()
    } else {
        format!("./routes/{}/page.sail", segs.join("/"))
    }
}

/// Inverse of the layout discovery: `/` → `./routes/_layout.sail`
/// (the root layout, sibling of `routes/page.sail`); `/studio` →
/// `./routes/studio/_layout.sail`. The codegen imports the layout's
/// `.sail` so the chrome wrapper it composes registers at first
/// paint (the per-view resolver chain — layout → its first
/// `<sail-*` / `<ship-*` tag → that wrapper's source `.sail` —
/// lands the `customElements.define` call when the module loads).
fn layout_module_path(path_prefix: &str) -> String {
    if path_prefix == "/" {
        return "./routes/_layout.sail".to_string();
    }
    let rest = path_prefix.strip_prefix('/').unwrap_or(path_prefix);
    format!("./routes/{rest}/_layout.sail")
}

/// `agents-id-settings` → `agentsIdSettings` (uppercase the char after each `-`).
fn camel_case(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut upper_next = false;
    for c in s.chars() {
        if c == '-' {
            upper_next = true;
        } else if upper_next {
            out.extend(c.to_uppercase());
            upper_next = false;
        } else {
            out.push(c);
        }
    }
    out
}

/// Walk `routes_dir` for `_layout.sail` files. Two flavors:
///   - A root `routes/_layout.sail` (sibling of `routes/page.sail`) becomes
///     a layout entry with prefix `/` — the runtime's longest-prefix
///     match treats `/` as "every pathname" (see
///     `packages/deck/router/src/layouts.ts::matchPrefix`). This is how
///     the hub root layout covers the index route `/` and every other
///     route under it; without it the nav had to be composed per-page.
///   - A `routes/<name>/_layout.sail` becomes a layout entry with prefix
///     `/<name>`. Unchanged from the pre-root-layout codegen.
///
/// The tag is read from the first `<sail-…>` or `<ship-…>` element in
/// the file. Layouts are emitted sorted by `path_prefix` so the
/// generated `routes.gen.ts` is byte-stable across runs.
fn discover_layouts(routes_dir: &Path) -> Result<Vec<Layout>> {
    let mut layouts = Vec::new();
    // Root layout: `routes/_layout.sail` → prefix `/`. Discovered before
    // the subdir loop so the prefix `/` sorts first (shortest path_prefix,
    // and the loop below sorts by `path_prefix` for byte stability).
    let root_layout = routes_dir.join("_layout.sail");
    if root_layout.is_file() {
        let tag = extract_layout_tag(&root_layout)
            .with_context(|| format!("parsing `{}`", root_layout.display()))?;
        layouts.push(Layout { path_prefix: "/".to_string(), tag });
    }
    for entry in std::fs::read_dir(routes_dir).with_context(|| {
        format!("reading routes dir `{}`", routes_dir.display())
    })? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let child = entry.path();
        let layout_file = child.join("_layout.sail");
        if !layout_file.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        let prefix = format!("/{name}");
        let tag = extract_layout_tag(&layout_file)
            .with_context(|| format!("parsing `{}`", layout_file.display()))?;
        layouts.push(Layout { path_prefix: prefix, tag });
    }
    layouts.sort_by(|a, b| a.path_prefix.cmp(&b.path_prefix));
    Ok(layouts)
}

/// Read up to the first 4 KiB of a `_layout.sail` and return the tag name
/// of the first `<sail-…>` or `<ship-…>` element. The SUI template grammar
/// is HTML-flavoured; the codegen does not need a full parse — it just needs
/// the wrapper component the runtime should import. If no qualifying tag is
/// found, error: a layout with no `<sail-*` / `<ship-*` element has no
/// component to mount.
fn extract_layout_tag(path: &Path) -> Result<String> {
    // Cap the read so a stray giant file can't pin the codegen. The tag is
    // always on the first line in practice; 4 KiB is generous.
    const HEAD_BYTES: usize = 4096;
    let bytes = std::fs::read(path)
        .with_context(|| format!("reading `{}`", path.display()))?;
    let head_bytes = if bytes.len() > HEAD_BYTES { &bytes[..HEAD_BYTES] } else { &bytes[..] };
    let head = std::str::from_utf8(head_bytes)
        .with_context(|| format!("non-utf8 in `{}`", path.display()))?;
    // Match the opening of the first element whose tag starts with `sail-`
    // or `ship-`. Whitespace, a tag name (letters/digits/`-`/`_`), and
    // either a `>` or whitespace ends the tag name. This is intentionally
    // minimal — the layout is just a chrome wrapper, not a full template
    // parse.
    let mut idx = 0;
    let bytes_str = head.as_bytes();
    while idx < bytes_str.len() {
        if bytes_str[idx] != b'<' {
            idx += 1;
            continue;
        }
        let after_lt = &bytes_str[idx + 1..];
        // Skip closing tags (`</foo>`) and doctype/processing (`<!…`, `<?…`).
        if let Some(&b) = after_lt.first() {
            if b == b'/' || b == b'!' || b == b'?' {
                idx += 1;
                continue;
            }
        }
        // Read tag name bytes.
        let name_start = idx + 1;
        let mut name_end = name_start;
        while name_end < bytes_str.len() {
            let b = bytes_str[name_end];
            if b.is_ascii_alphanumeric() || b == b'-' || b == b'_' {
                name_end += 1;
            } else {
                break;
            }
        }
        if name_end == name_start {
            idx += 1;
            continue;
        }
        let tag = std::str::from_utf8(&bytes_str[name_start..name_end])
            .context("non-utf8 tag name")?;
        if tag.starts_with("sail-") || tag.starts_with("ship-") {
            return Ok(tag.to_string());
        }
        idx = name_end;
    }
    bail!(
        "`{}` has no `<sail-…>` or `<ship-…>` element — a layout must name a wrapper component",
        path.display()
    );
}
/// Generate `routes.gen.ts` for every spec. Each spec walks its `dir`
/// (a `routes/` tree) and writes a `routes.gen.ts` to its `out` path.
/// Returns the total number of dynamic routes emitted across all specs.
///
/// No-op for a spec whose `dir` is absent. Per-spec write-if-changed
/// (`routes.gen.ts` lives under the watched `apps/...` root and the app
/// build regenerates it on every rebuild — an unconditional write would
pub fn generate_routes(specs: &[RouteGenSpec]) -> Result<usize> {
    let mut total = 0;
    for spec in specs {
        total += generate_routes_one(spec)?;
        // View-route discovery is a sibling walker: emit
        // `views/routes.gen.ts` (router fields) and
        // `views/meta.gen.ts` (host fields) for the same spec.
        // No-op if no `<root>/views/` dir exists.
        write_view_artifacts(spec)?;
    }
    Ok(total)
}

/// One raw view manifest entry. Parsed from `views/<name>/manifest.jsonc`.
/// The JSON allows comments (`// …`) which serde_json doesn't natively
/// accept, so we strip line-`//` comments before parsing. The router
/// fields (pattern, view name) go to `views/routes.gen.ts`; the host
/// fields (title, layout, auth, rail) go to `views/meta.gen.ts`. The
/// split is the contract — the router is host-agnostic, the host reads
/// what it needs.
struct ViewManifest {
    name: String,
    path: String,
    title: String,
    layout: String,
    auth: String,
    rail: Option<serde_json::Value>,
}

/// Strip `//` line comments from a JSONC source so serde_json can parse
/// it. Trailing comments on a line are stripped before any JSON value
/// closes the line; strings containing `//` are preserved.
fn strip_jsonc(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let mut in_string = false;
    let mut escape = false;
    let mut chars = src.chars().peekable();
    while let Some(c) = chars.next() {
        if in_string {
            out.push(c);
            if escape {
                escape = false;
                continue;
            }
            if c == '\\' {
                escape = true;
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }
        if c == '"' {
            in_string = true;
            out.push(c);
            continue;
        }
        if c == '/' && chars.peek() == Some(&'/') {
            chars.next();
            // Skip until newline.
            for nc in chars.by_ref() {
                if nc == '\n' {
                    out.push('\n');
                    break;
                }
            }
            continue;
        }
        out.push(c);
    }
    out
}

/// Walk `views/<name>/` for top-level dirs with a `manifest.jsonc`.
/// Reads each manifest and returns the parsed entries. Skips entries
/// missing `path` or whose `path` doesn't start with `/` (the router
/// requires absolute patterns).
fn discover_views(views_dir: &Path) -> Result<Vec<ViewManifest>> {
    let mut out = Vec::new();
    if !views_dir.is_dir() {
        return Ok(out);
    }
    for entry in std::fs::read_dir(views_dir)
        .with_context(|| format!("reading `{}`", views_dir.display()))?
    {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let manifest_path = entry.path().join("manifest.jsonc");
        if !manifest_path.exists() {
            continue;
        }
        let dir_name = entry.file_name().to_string_lossy().into_owned();
        let raw = std::fs::read_to_string(&manifest_path)
            .with_context(|| format!("reading `{}`", manifest_path.display()))?;
        let json = strip_jsonc(&raw);
        let parsed: serde_json::Value = serde_json::from_str(&json)
            .with_context(|| format!("parsing `{}`", manifest_path.display()))?;
        let path = parsed.get("path").and_then(|v| v.as_str()).unwrap_or("").to_string();
        if path.is_empty() {
            eprintln!(
                "view-discovery: skipping `{}` — manifest has no `path` field",
                manifest_path.display()
            );
            continue;
        }
        if !path.starts_with('/') {
            eprintln!(
                "view-discovery: skipping `{}` — `path` is `{}`, must start with `/`",
                manifest_path.display(),
                path
            );
            continue;
        }
        let manifest_name = parsed.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
        // The dir name is canonical; warn if the manifest's `name`
        // disagrees (defensive — keeps the dir as the source of truth).
        if !manifest_name.is_empty() && manifest_name != dir_name {
            eprintln!(
                "view-discovery: `{}` manifest `name` is `{}` but dir is `{}` — using dir",
                manifest_path.display(),
                manifest_name,
                dir_name
            );
        }
        let title = parsed.get("title").and_then(|v| v.as_str()).unwrap_or(&dir_name).to_string();
        let layout = parsed.get("layout").and_then(|v| v.as_str()).unwrap_or("default").to_string();
        let auth = parsed.get("auth").and_then(|v| v.as_str()).unwrap_or("user").to_string();
        let rail = parsed.get("rail").cloned();
        out.push(ViewManifest {
            name: dir_name,
            path,
            title,
            layout,
            auth,
            rail,
        });
    }
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

/// Write `views/routes.gen.ts` — the router's contract. `ViewRoute[]`
/// (pattern + view) and a derived `DYNAMIC_ROUTE_PATTERNS` for the
/// host to pass to `defineRoutes`. The router is host-agnostic.
fn write_views_routes(views_dir: &Path, label: &str, manifests: &[ViewManifest]) -> Result<()> {
    let entries = manifests
        .iter()
        .map(|m| format!("  {{ pattern: '{}', view: '{}' }},", m.path, m.name))
        .collect::<Vec<_>>()
        .join("\n");
    let patterns = manifests
        .iter()
        .map(|m| format!("  {{ pattern: '{}' }},", m.path))
        .collect::<Vec<_>>()
        .join("\n");
    let body = format!(
        "\
// @generated by `deck routes` (deck-bundler) — DO NOT EDIT.
// Router-only view table for the `{label}` view tree (every
// `views/<name>/manifest.jsonc`). The router cares about the URL
// pattern and the view resource name; host fields (title, layout,
// auth, rail) live in `views/meta.gen.ts`. The two files are
// separate contracts: the router is host-agnostic.
// See ADR unified-deck-router.

import type {{ RouteShape }} from '@ship/deck/router'

/** Per-view route entries. `pattern` is the URL; `view` is the daemon
 *  resource name (the view is served at `/_deck/views/<view>/…`).
 *  The host registers the patterns with the runtime's `defineRoutes`
 *  IF the view is downloaded/active at boot. */
export const VIEW_ROUTES: ViewRoute[] = [
{entries}
]

/** Convenience table for `defineRoutes` — the patterns the host
 *  should pre-register. Static patterns need no entry (the runtime
 *  matches them by segment equality) but listing them here is
 *  defensive and matches the codegen's behavior for `routes/`. */
export const DYNAMIC_ROUTE_PATTERNS: RouteShape[] = [
{patterns}
]

export type ViewRoute = {{ pattern: string; view: string }}
",
        label = label,
    );
    let target = views_dir.join("routes.gen.ts");
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).context("create views/routes.gen.ts parent")?;
    }
    let changed = std::fs::read_to_string(&target)
        .map(|cur| cur != body)
        .unwrap_or(true);
    if changed {
        std::fs::write(&target, body).context("write views/routes.gen.ts")?;
    }
    Ok(())
}

/// Write `views/meta.gen.ts` — the host's contract. `ViewMeta[]` with
/// chrome/auth/rail. The host reads this at boot to choose chrome,
/// gate access, and add launcher rail entries. The router does NOT
/// see this file.
fn write_views_meta(views_dir: &Path, label: &str, manifests: &[ViewManifest]) -> Result<()> {
    let entries = manifests
        .iter()
        .map(|m| {
            let rail_str = m
                .rail
                .as_ref()
                .map(|r| serde_json::to_string(r).unwrap_or_else(|_| "undefined".to_string()))
                .unwrap_or_else(|| "undefined".to_string());
            format!(
                "  {{ name: '{}', title: '{}', layout: '{}', auth: '{}', rail: {rail_str} }},",
                m.name, m.title, m.layout, m.auth
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let body = format!(
        "\
// @generated by `deck routes` (deck-bundler) — DO NOT EDIT.
// Host-only view metadata for the `{label}` view tree (every
// `views/<name>/manifest.jsonc`). The host reads this to choose
// chrome (layout), gate access (auth), and add launcher rail
// entries. The router is host-agnostic and does NOT see this file
// (the router's contract lives in `views/routes.gen.ts`).
// See ADR unified-deck-router.

/** Per-view host metadata. The host builds its launcher rail from
 *  `name` (the navigation key) and the `rail` shape; chooses the
 *  chrome wrapper from `layout`; gates access on `auth`. `path` is
 *  NOT here — the router is the only consumer of `path` and it
 *  reads it from `views/routes.gen.ts`. */
export const VIEW_META: ViewMeta[] = [
{entries}
]

export interface ViewRail {{
  label: string
  icon: string
}}

export interface ViewMeta {{
  name: string
  title: string
  layout: string
  auth: string
  rail?: ViewRail
}}
",
        label = label,
    );
    let target = views_dir.join("meta.gen.ts");
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).context("create views/meta.gen.ts parent")?;
    }
    let changed = std::fs::read_to_string(&target)
        .map(|cur| cur != body)
        .unwrap_or(true);
    if changed {
        std::fs::write(&target, body).context("write views/meta.gen.ts")?;
    }
    Ok(())
}

/// The full set of data `routes.gen.ts` registers. Walking the `routes/`
/// tree produces one of these; the formatter turns it into a TypeScript
/// module. Tests assert on this data layer (stable across format
/// changes), not on the emitted text — the text is presentation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RouteTree {
    pub label: String,
    pub routes: Vec<Route>,
    pub layouts: Vec<Layout>,
}


// Retained as a thin wrapper over `render_routes_gen_with` for the 3 in-file
// tests below; the `#[cfg(test)]` usages make the function unreachable in
// non-test builds, hence the allow. Removing the wrapper would force the
// tests to call `render_routes_gen_with(tree, true)` with the magic `true`
// — the wrapper's only contract is "legacy `include_token_import=true` shape".
#[allow(dead_code)]
fn render_routes_gen(tree: &RouteTree) -> String {
    render_routes_gen_with(tree, true)
}

/// Render `routes.gen.ts`. `include_token_import` controls whether the
/// `@ship/tokens/tokens.css` side-effect import lives in this file
/// (the legacy shape — `routes.gen.ts` is the SPA entry) or in
/// `entry.gen.ts` (the split shape — data + bootstrap are separate).
/// The split shape is the spec's preferred form; the legacy shape is
/// retained for builds that don't declare an `entry` bootstrap in
/// `deck.jsonc` so the token import keeps shipping with every per-app
/// build (the design tokens MUST ride the bundle for `var(--token)`
/// to resolve at first paint — see `ssot_tokens_import_pinned`).
fn render_routes_gen_with(tree: &RouteTree, include_token_import: bool) -> String {
    let pattern_lines = tree
        .routes
        .iter()
        .map(|r| format!("  {{ pattern: '{}' }},", r.pattern))
        .collect::<Vec<_>>()
        .join("\n");
    let builder_lines = tree
        .routes
        .iter()
        .map(|r| {
            let arg = format!(
                "p: {{ {} }}",
                r.params.iter().map(|k| format!("{k}: string")).collect::<Vec<_>>().join("; ")
            );
            // `/studio/agents/:id` → template `/studio/agents/${p.id}`.
            let mut path = r.pattern.clone();
            for k in &r.params {
                path = path.replace(&format!(":{k}"), &format!("${{p.{k}}}"));
            }
            format!("  /** {} */\n  {}: ({arg}): string => `{path}`,", r.pattern, r.builder)
        })
        .collect::<Vec<_>>()
        .join("\n");
    let layout_lines = tree
        .layouts
        .iter()
        .map(|l| format!("  {{ pathPrefix: '{}', tag: '{}' }},", l.path_prefix, l.tag))
        .collect::<Vec<_>>()
        .join("\n");
    // Route modules are CODE-SPLIT: the module map holds `() => import(...)`
    // thunks (below), so rolldown emits one chunk per route and the client loads
    // a route's chunk on demand (mountRouter awaits the thunk — the lazy path).
    // NO static `import * as route{i}`: a static import would co-bundle the
    // module into the entry and defeat the split. dev==prod parity — both builds
    // split the same way, so HMR can later swap a single route chunk.
    //
    // PREREQUISITE: the served `@ship/deck-router` framework bundle must carry
    // the lazy mountRouter (awaits function-valued module entries). It ships from
    // packages/deck/router/dist — rebuild it (`bun run build`) after touching the
    // router src, else the bake serves an eager mountRouter and the lazy thunks
    // arrive un-awaited (mount() sees a function, not a module).
    let route_imports = String::new();
    // Each layout's `.sail` is imported as a named module (import * as layoutN)
    // so the bundler tracks it as a real use (no tree-shake removal) AND
    // `mountRouter` receives the module object to render the layout's authored
    // markup via mount(). The named binding also guarantees the custom-element
    // side-effect (registration) fires on first paint — same as the former
    // side-effect-only import, but now the module is also passed to the router.
    let layout_imports = tree
        .layouts
        .iter()
        .enumerate()
        .map(|(i, l)| format!("import * as layout{i} from '{}';", layout_module_path(&l.path_prefix)))
        .collect::<Vec<_>>()
        .join("\n");
    // Pattern → `() => import('<route>/page.sail')` thunk. The dynamic import is
    // rolldown's code-split signal (one chunk per route); mountRouter's lazy path
    // awaits it on navigation.
    let module_map = tree
        .routes
        .iter()
        .map(|r| {
            format!(
                "    '{}': () => import('{}'),",
                r.pattern,
                module_import_path(&r.pattern)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    // pathPrefix → layout module map, parallel to `modules` for routes.
    // mountRouter uses this to render each layout as a view module (authored
    // markup) rather than a bare createElement(tag). Layouts without a
    // _layout.sail would not be in this table — the tag-only fallback path
    // in the router handles them. Currently every discovered layout has a
    // _layout.sail (discover_layouts only records files that exist), so the
    // map is always complete.
    let layout_modules_map = tree
        .layouts
        .iter()
        .enumerate()
        .map(|(i, l)| format!("    '{}': layout{i},", l.path_prefix))
        .collect::<Vec<_>>()
        .join("\n");
    // The token import (design tokens — OKLCH palette, semantic surfaces,
    // tone ramps, role tokens, motion/density channels) and the
    // `mountRouter` call used to live here. They've moved to `entry.gen.ts`
    // (the host-owned bootstrap): `routes.gen.ts` is now DATA only — route
    // table + layout table + typed path builders + the modules / layoutModules
    // maps. Decoupled per the spec's "routes.gen.ts ↔ entry.gen.ts split":
    // the data + the bootstrap are separately consumable, separately testable.
    // The token import lands in `entry.gen.ts` so it ships with the bootstrap
    // (the single module the entry chunk pulls in); see `render_entry_gen`.
    // The token import (design tokens — OKLCH palette, semantic surfaces,
    // tone ramps, role tokens, motion/density channels) is conditionally
    // emitted by THIS file: when `include_token_import` is true (the
    // legacy shape — `routes.gen.ts` is the SPA entry, no host bootstrap),
    // the import rides here. When false (the split shape — the host
    // declared an `entry` bootstrap), the import moves to `entry.gen.ts`
    // (the host-owned bootstrap). Either way the tokens ship with the
    // build — the split is which file pulls them in.
    let token_import_block = if include_token_import {
        // Design tokens + the framework stylesheet (light-DOM contracts +
        // stdlib element skins) ride the entry so a static build ships them
        // as hashed assets, styled without runtime JS injection. Each CSS
        // file is a direct JS import (Rolldown bundles JS-imported CSS; it
        // does NOT inline a wrapper's `@import`).
        format!(
            "import '@ship/tokens/tokens.css';\n\
             import '@ship/tokens/contracts.css';\n\
             import '@ship/tokens/element-skins.css';\n"
        )
    } else {
        String::new()
    };
    format!(
"// @generated by `deck routes` (deck-bundler) — DO NOT EDIT.
// Typed route table + view-module map for the `{label}` view tree (every
// `[param]` dir under `routes/`). This file is DATA ONLY when the host
import {{ type RouteShape, type LayoutShape }} from '@ship/deck/router'
{token_import_block}{route_imports}
{layout_imports}
export const DYNAMIC_ROUTE_PATTERNS: RouteShape[] = [
{pattern_lines}
]
export const LAYOUTS: LayoutShape[] = [
{layout_lines}
]
export const routes = {{
{builder_lines}
}}
export const modules: Record<string, unknown> = {{
{module_map}
}}
export const layoutModules: Record<string, unknown> = {{
{layout_modules_map}
}}
",
        label = tree.label,
        layout_imports = layout_imports,
        route_imports = route_imports,
        pattern_lines = pattern_lines,
        builder_lines = builder_lines,
        layout_lines = layout_lines,
        module_map = module_map,
        layout_modules_map = layout_modules_map,
     )
}

/// Render the host-owned `entry.gen.ts` — the bootstrap that imports
/// the data tables from `./routes.gen`, constructs the named `clients`
/// map (off the host's `clientsModule` exports), and calls `mountRouter`.
///
/// Why this file exists (separately from `routes.gen.ts`): the spec
/// requires deck stop bundling client wiring into its own route-table
/// emission. Routes are data + a stable contract every per-app host
/// consumes; client construction is the host's call (which clients to
/// expose, with what names, against which daemon). The split mirrors
/// the `DataPort` / `AppRegistry` discipline: the host binds its own
/// concrete inside the impl; the framework provides the slot.
///
/// `entry_bootstrap` declares the host's `clientsModule` path (relative
/// to the generated entry, which lives at `<project>/entry.gen.ts` next
/// to `routes.gen.ts`) and the names the bundler reads off that module's
/// named exports. The map keys are those names; the map values are the
/// actual client instances the host constructed.
///
/// Design-token side-effect import (`@ship/tokens/tokens.css`) rides
/// THIS file (not `routes.gen.ts`) so the bundle still ships the tokens
/// before first paint — the data file is data-only.
pub(crate) fn render_entry_gen(
    tree: &RouteTree,
    entry_bootstrap: Option<&deck_config::EntryBootstrap>,
) -> String {
    // Host clients (when declared). A standalone deck app has none: no
    // client import, and the `clients` field is omitted from the call
    // (MountRouterOptions.clients is optional). When a host declares an
    // `entry` bootstrap, its named clients are imported off `clientsModule`
    // and spread into the map.
    let (clients_import, clients_field) = match entry_bootstrap {
        Some(b) if !b.expose_names.is_empty() => {
            let names_csv = b.expose_names.join(", ");
            let clients_obj = b
                .expose_names
                .iter()
                .map(|n| format!("    {n}: {n},"))
                .collect::<Vec<_>>()
                .join("\n");
            (
                format!("import {{ {names_csv} }} from '{}';\n", b.clients_module),
                format!("\n  clients: {{\n{clients_obj}\n  }},"),
            )
        }
        _ => (String::new(), String::new()),
    };
    // Even with zero routes the bootstrap still runs — it sets up the
    // mount target + the token CSS. The router displays its not-found
    // fallback.
    format!(
"// @generated by `deck routes` (deck-bundler) — DO NOT EDIT.
// SPA bootstrap for the `{label}` view tree. Imports the data tables from
// `./routes.gen`, (optionally) constructs the named `clients` map, and hands
// everything to the framework router `mountRouter` (@ship/deck/router), which
// owns history, route match, the layout chain, and SPA navigation. See ADR
// `unified-deck-router`.
//
// deck ALWAYS emits this bootstrap — it is the framework's DEFAULT client
// wiring, so a standalone app (no host, no `deck.jsonc`) mounts on its own.
// A host `deck.jsonc#entry` only CUSTOMIZES the `clients` map (which names,
// off which module); the router call itself is deck's. Routes are data
// (`routes.gen.ts`); clients are the host's concrete. The framework names no
// ship path. The runtime exposes each client to every view's
// `ctx.clients.<name>` (and the legacy `ctx.<name>` alias).
import {{ mountRouter }} from '@ship/deck/router';
{clients_import}import {{
  DYNAMIC_ROUTE_PATTERNS,
  LAYOUTS,
  modules,
  layoutModules,
}} from './routes.gen';
import '@ship/tokens/tokens.css';
import '@ship/tokens/contracts.css';
import '@ship/tokens/element-skins.css';
// On an HMR re-import the framework router is already mounted (it exposes
// `globalThis.__deckHmr`). Swap the freshly-bundled `modules` (their thunks
// point at the new chunk URLs) into the running router + re-mount preserving
// state, instead of mounting a second time. First load: `__deckHmr` is unset,
// so mount normally.
const __deckHmr = (globalThis as unknown as {{ __deckHmr?: {{ swapModules: (m: unknown) => void }} }}).__deckHmr;
if (__deckHmr) {{
  __deckHmr.swapModules(modules);
}} else {{
  mountRouter({{
    root: '#{root_id}',
    routes: DYNAMIC_ROUTE_PATTERNS,
    layouts: LAYOUTS,
    modules,
    layoutModules,{clients_field}
  }});
}}
",
        label = tree.label,
        root_id = crate::templates::ROOT_ID,
    )
}

/// Write a generated file only when its contents actually moved. The app
/// build regenerates these files on EVERY rebuild, and they live under the
/// watched `apps/...` root — an unconditional write retriggers the bundler
/// watcher and self-perpetuates a rebuild/reload loop (shipd CPU spin,
/// page reloads ~6×/sec). Only touch the file when the generated content
/// actually moved.
pub(crate) fn write_if_changed(target: &Path, body: &str) -> Result<()> {
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create parent for {}", target.display()))?;
    }
    let changed = std::fs::read_to_string(target)
        .map(|cur| cur != body)
        .unwrap_or(true);
    if changed {
        std::fs::write(target, body)
            .with_context(|| format!("write {}", target.display()))?;
    }
    Ok(())
}

pub(crate) fn build_route_tree(routes_dir: &Path, label: &str) -> Result<RouteTree> {
    if !routes_dir.is_dir() {
        return Ok(RouteTree { label: label.to_string(), routes: vec![], layouts: vec![] });
    }
    let mut dirs = Vec::new();
    view_dirs(routes_dir, &mut dirs)
        .with_context(|| format!("walking routes tree for `{label}`"))?;
    let mut routes: Vec<Route> = dirs.iter().filter_map(|d| to_route(routes_dir, d)).collect();
    routes.sort_by(|a, b| cmp_route_patterns(&a.pattern, &b.pattern));
    let layouts = discover_layouts(routes_dir)
        .with_context(|| format!("walking layouts for `{label}`"))?;
    Ok(RouteTree { label: label.to_string(), routes, layouts })
}

/// Specificity rank of one path segment: static literal (0) before dynamic
/// `:param` (1) before splat `*` (2). The runtime router is first-match-wins
/// over the emitted order, so this ordering is what makes a more-specific
/// route win.
fn segment_rank(seg: &str) -> u8 {
    if seg.starts_with('*') {
        2
    } else if seg.starts_with(':') {
        1
    } else {
        0
    }
}

/// Order route patterns so a STATIC segment always sorts before a DYNAMIC
/// (`:param`) one at the same position (and dynamic before a splat). Within the
/// same class, a byte comparison keeps the order deterministic. This makes the
/// first-match-wins router prefer e.g. `/domains/new` over `/domains/:id`,
/// instead of the old plain lexicographic sort where `:` (0x3A) sorted before
/// letters and a dynamic route shadowed its static sibling.
fn cmp_route_patterns(a: &str, b: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    let mut sa = a.split('/');
    let mut sb = b.split('/');
    loop {
        match (sa.next(), sb.next()) {
            (Some(x), Some(y)) => {
                let r = segment_rank(x).cmp(&segment_rank(y)).then_with(|| x.cmp(y));
                if r != Ordering::Equal {
                    return r;
                }
            }
            // A shorter path sorts before a longer one sharing its prefix.
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (None, None) => return Ordering::Equal,
        }
    }
}
fn generate_routes_one(spec: &RouteGenSpec) -> Result<usize> {
    let tree = build_route_tree(&spec.dir, &spec.label)?;
    // No-op when the source dir is absent: nothing to register, no
    // file to write. Specs with a missing `routes/` must be safe to
    // call repeatedly without creating empty artifacts.
    if !spec.dir.is_dir() {
        return Ok(0);
    }
    // `routes.gen.ts` is ALWAYS data-only now: the token import + the
    // `mountRouter` call live in the bootstrap (`entry.gen.ts`), which deck
    // always emits (below).
    let body = render_routes_gen_with(&tree, false);
    write_if_changed(&spec.out, &body)?;
    // deck ALWAYS emits the SPA bootstrap (`entry.gen.ts`) next to the data
    // file — it is the framework's default client wiring, so a standalone app
    // mounts with no host. A host `deck.jsonc#entry` (when present) only
    // customizes the `clients` map; the router call is deck's. Without this, a
    // standalone deck app served a data-only entry that never called
    // mountRouter — `#root` stayed empty (inv-deck-standalone).
    let entry_path = spec
        .out
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .join("entry.gen.ts");
    let entry_body = render_entry_gen(&tree, spec.entry_bootstrap.as_ref());
    write_if_changed(&entry_path, &entry_body)?;
    Ok(tree.routes.len())
}
// files and emit TWO files: `views/routes.gen.ts` (router fields only)
// and `views/meta.gen.ts` (host fields only). The split is the
// contract — the router is host-agnostic; the host reads the meta.

/// Generate the view route + meta tables for a single views bucket dir
/// (`views/` by default; a host may override `view_dirs` to add more).
/// Walks `<views_dir>/<name>/manifest.jsonc`
/// and writes `<views_dir>/routes.gen.ts` (router fields) +
/// `<views_dir>/meta.gen.ts` (host fields). No-op when the dir is absent
/// or holds no manifests. Returns the number of views discovered.
///
/// This is the framework-owned home for the view tables: the bundler's
/// `build_all` (the asset bake / `deck build-assets`) calls it for every
/// discovered view bucket, so a fresh clone + `build-assets` produces them
/// with no manual `deck routes --views` step. The bundler's watch path
/// ignores `.gen.ts`, so these writes never self-trigger a rebuild loop.
pub fn generate_view_artifacts(views_dir: &Path, label: &str) -> Result<usize> {
    if !views_dir.is_dir() {
        return Ok(0);
    }
    let manifests = discover_views(views_dir)
        .with_context(|| format!("walking views for `{label}`"))?;
    if manifests.is_empty() {
        return Ok(0);
    }
    write_views_routes(views_dir, label, &manifests)
        .with_context(|| format!("writing views/routes.gen.ts for `{label}`"))?;
    write_views_meta(views_dir, label, &manifests)
        .with_context(|| format!("writing views/meta.gen.ts for `{label}`"))?;
    Ok(manifests.len())
}

/// Per-spec view-artifact emit used by `generate_routes` (the `deck routes`
/// CLI path). Resolves the views dir relative to a `routes/` spec, then
/// delegates to `generate_view_artifacts`.
fn write_view_artifacts(spec: &RouteGenSpec) -> Result<()> {
    // Sibling of `routes/`: `<root>/../views/` (or `<root>/views/` if
    // `routes/` is a peer dir, e.g. `apps/shell/routes/…` →
    // `apps/shell/views/…`). Try the parent first; fall back to the
    // dir itself if it has a `views/` child.
    let candidates = [
        spec.dir.parent().map(|p| p.join("views")),
        Some(spec.dir.join("views")),
    ];
    let views_dir = candidates
        .into_iter()
        .flatten()
        .find(|p| p.is_dir());
    let Some(views_dir) = views_dir else {
        return Ok(());
    };
    generate_view_artifacts(&views_dir, &spec.label)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_route_sorts_before_dynamic_sibling() {
        // The first-match-wins router must see `/domains/new` before
        // `/domains/:id`, else the static create page is shadowed by the
        // dynamic detail route (the old lexicographic sort put `:` first).
        let mut pats = vec![
            "/domains/:id".to_string(),
            "/domains/new".to_string(),
            "/domains".to_string(),
            "/domains/:id/doc".to_string(),
        ];
        pats.sort_by(|a, b| cmp_route_patterns(a, b));
        assert_eq!(
            pats,
            vec![
                "/domains".to_string(),
                "/domains/new".to_string(),
                "/domains/:id".to_string(),
                "/domains/:id/doc".to_string(),
            ],
            "static segments must sort before dynamic :param segments at each position"
        );
    }

    fn spec(root: &Path, label: &str) -> RouteGenSpec {
        RouteGenSpec {
            dir: root.join("routes"),
            out: root.join("routes.gen.ts"),
            label: label.to_string(),
            // Tests run the legacy shape: routes.gen.ts IS the entry,
            // the token import rides there. The split-shape path has its
            // own dedicated tests below.
            entry_bootstrap: None,
        }
    }

    #[test]
    fn shell_route_discovery() {
        let root = tempfile::tempdir().unwrap();
        let r = root.path().join("routes");
        for d in ["a/b/[id]/c", "a/d/[name]", "a/e"] {
            std::fs::create_dir_all(r.join(d)).unwrap();
            std::fs::write(r.join(d).join("page.sail"), "<div/>").unwrap();
        }
        let tree = build_route_tree(&r, "app").unwrap();
        // Static routes are NOT filtered out: a static mount (`/a/e`, no
        // `[param]` segment) must still land in the table, not be dropped.
        let patterns: Vec<&str> = tree.routes.iter().map(|r| r.pattern.as_str()).collect();
        assert_eq!(patterns, vec!["/a/b/:id/c", "/a/d/:name", "/a/e"]);
        // The static route has no params.
        let stat = tree.routes.iter().find(|r| r.pattern == "/a/e").unwrap();
        assert!(stat.params.is_empty());
        // Builders are namespaced by the top-level segment.
        let builders: Vec<&str> = tree.routes.iter().map(|r| r.builder.as_str()).collect();
        assert_eq!(builders, vec!["aBIdC", "aDName", "aE"]);
    }

    #[test]
    fn multi_root_emits_one_file_per_spec() {
        // Two roots in the same tempdir, each with a different mountable
        // view. The codegen must walk both and write one file per spec
        // with the right data. Asserts the DISPATCH behavior; the
        // per-spec data correctness is covered by `shell_route_discovery`.
        let root = tempfile::tempdir().unwrap();
        let one_routes = root.path().join("apps/one/routes");
        let two_routes = root.path().join("apps/two/routes");
        std::fs::create_dir_all(one_routes.join("x/[id]")).unwrap();
        std::fs::write(one_routes.join("x/[id]/page.sail"), "<div/>").unwrap();
        std::fs::create_dir_all(two_routes.join("[id]")).unwrap();
        std::fs::write(two_routes.join("[id]/page.sail"), "<div/>").unwrap();
        let specs = vec![
            RouteGenSpec {
                dir: one_routes,
                out: root.path().join("apps/one/routes.gen.ts"),
                label: "one".to_string(),
                entry_bootstrap: None,
            },
            RouteGenSpec {
                dir: two_routes,
                out: root.path().join("apps/two/routes.gen.ts"),
                label: "two".to_string(),
                entry_bootstrap: None,
            },
        ];
        let n = generate_routes(&specs).unwrap();
        assert_eq!(n, 2, "one dynamic route per root, total two");
        // Both files were written. The contents are the data table —
        // not the format, which is render_routes_gen's contract.
        let one = build_route_tree(&specs[0].dir, "one").unwrap();
        let two = build_route_tree(&specs[1].dir, "two").unwrap();
        assert_eq!(one.routes[0].pattern, "/x/:id");
        assert_eq!(two.routes[0].pattern, "/:id");
        assert_eq!(one.label, "one");
        assert_eq!(two.label, "two");
    }

    #[test]
    fn missing_routes_dir_is_noop() {
        let root = tempfile::tempdir().unwrap();
        // The `routes/` dir doesn't exist; the spec is a no-op (returns 0)
        // and no file is written.
        let n = generate_routes(&[spec(root.path(), "shell")]).unwrap();
        assert_eq!(n, 0);
        assert!(!root.path().join("routes.gen.ts").exists());
    }

    #[test]
    fn layout_discovery_emits_layouts_block() {
        // A `routes/foo/_layout.sail` becomes a LAYOUTS entry with
        // `pathPrefix: '/foo'` and the tag read from the file.
        let root = tempfile::tempdir().unwrap();
        let r = root.path().join("routes");
        std::fs::create_dir_all(r.join("foo/bar/[id]")).unwrap();
        std::fs::write(r.join("foo/bar/[id]/page.sail"), "<div/>").unwrap();
        // Realistic layout: comment + a `<sail-*` wrapper + a `<slot>`.
        std::fs::write(
            r.join("foo/_layout.sail"),
            "// chrome\n<sail-foo-chrome>\n  <slot />\n</sail-foo-chrome>\n",
        )
        .unwrap();
        let tree = build_route_tree(&r, "app").unwrap();
        assert_eq!(tree.layouts.len(), 1);
        assert_eq!(tree.layouts[0].path_prefix, "/foo");
        assert_eq!(tree.layouts[0].tag, "sail-foo-chrome");
    }

    #[test]
    fn layout_discovery_ship_prefixed_tag() {
        // `<ship-*` is also accepted (chrome wrappers under @ship/* packs).
        let root = tempfile::tempdir().unwrap();
        let r = root.path().join("routes");
        std::fs::create_dir_all(r.join("baz")).unwrap();
        std::fs::write(r.join("baz/page.sail"), "<div/>").unwrap();
        std::fs::write(r.join("baz/_layout.sail"), "<ship-baz-chrome/>\n").unwrap();
        let tree = build_route_tree(&r, "app").unwrap();
        assert_eq!(tree.layouts[0].tag, "ship-baz-chrome");
        assert_eq!(tree.layouts[0].path_prefix, "/baz");
    }

    #[test]
    fn layout_discovery_root_layout_has_slash_prefix() {
        // A `routes/_layout.sail` (sibling of `routes/page.sail`) becomes a
        // LAYOUTS entry with `pathPrefix: '/'` — the runtime's longest-
        // prefix match treats `/` as "every pathname" (see
        // `packages/deck/router/src/layouts.ts::matchPrefix`). The root
        // layout covers the index route AND every nested route (the
        // longest-prefix match keeps nested layouts outermost-in-chain).
        let root = tempfile::tempdir().unwrap();
        let r = root.path().join("routes");
        // Index route + a nested subdir with its OWN layout (the
        // subdir layout must coexist with the root layout).
        std::fs::create_dir_all(r.join("studio/components/[name]")).unwrap();
        std::fs::write(r.join("page.sail"), "<div/>").unwrap();
        std::fs::write(
            r.join("studio/components/[name]/page.sail"),
            "<div/>",
        )
        .unwrap();
        std::fs::write(r.join("studio/_layout.sail"), "<sail-studio-chrome/>\n").unwrap();
        std::fs::write(
            r.join("_layout.sail"),
            "<ship-hub-chrome>\n  <slot name=\"route\"/>\n</ship-hub-chrome>\n",
        )
        .unwrap();
        let tree = build_route_tree(&r, "hub").unwrap();
        // Both layouts discovered, sorted by path_prefix (`/` before
        // `/studio`).
        assert_eq!(tree.layouts.len(), 2);
        assert_eq!(tree.layouts[0].path_prefix, "/");
        assert_eq!(tree.layouts[0].tag, "ship-hub-chrome");
        assert_eq!(tree.layouts[1].path_prefix, "/studio");
        assert_eq!(tree.layouts[1].tag, "sail-studio-chrome");
    }

    #[test]
    fn layout_discovery_root_layout_only_when_no_subdir_layouts() {
        // A sole `routes/_layout.sail` → a single LAYOUTS entry with
        // prefix `/`. The /studio case above is a richer version of
        // this; this one isolates the root-only path.
        let root = tempfile::tempdir().unwrap();
        let r = root.path().join("routes");
        std::fs::create_dir_all(&r).unwrap();
        std::fs::write(r.join("page.sail"), "<div/>").unwrap();
        std::fs::write(r.join("_layout.sail"), "<ship-app-chrome/>\n").unwrap();
        let tree = build_route_tree(&r, "hub").unwrap();
        assert_eq!(tree.layouts.len(), 1);
        assert_eq!(tree.layouts[0].path_prefix, "/");
        assert_eq!(tree.layouts[0].tag, "ship-app-chrome");
    }

    #[test]
    fn layout_discovery_skips_div_only_layout() {
        // A `_layout.sail` with no `<sail-*` / `<ship-*` element errors out
        // — a layout must name a wrapper component.
        let root = tempfile::tempdir().unwrap();
        let r = root.path().join("routes");
        std::fs::create_dir_all(r.join("home")).unwrap();
        std::fs::write(r.join("home/page.sail"), "<div/>").unwrap();
        std::fs::write(r.join("home/_layout.sail"), "<div>no wrapper here</div>\n").unwrap();
        let err = build_route_tree(&r, "shell").unwrap_err();
        let msg = format!("{err:#}");
        assert!(
            msg.contains("no `<sail-…>` or `<ship-…>` element"),
            "expected layout-tag error, got: {msg}"
        );
    }

    #[test]
    fn module_import_path_inverts_to_route_segment_mapping() {
        // Pure logic, generic inputs — no app routes. `:param` ↔ `[param]` dir.
        assert_eq!(module_import_path("/"), "./routes/page.sail");
        assert_eq!(module_import_path("/x"), "./routes/x/page.sail");
        assert_eq!(module_import_path("/x/:id"), "./routes/x/[id]/page.sail");
        assert_eq!(module_import_path("/a/:b/c/:d"), "./routes/a/[b]/c/[d]/page.sail");
    }

    #[test]
    fn layout_module_path_inverts_to_layout_file() {
        // Pure logic, mirrors `module_import_path` for layouts. The
        // codegen statically imports each layout's `.sail` so the
        // chrome wrapper it composes registers at first paint; the
        // generated path is the on-disk layout relative to the
        // entry (the entry lives next to `routes/`).
        //   `/`        → `./routes/_layout.sail` (the root layout, sibling
        //                  of `routes/page.sail`)
        //   `/studio`  → `./routes/studio/_layout.sail` (the subdir layout)
        assert_eq!(layout_module_path("/"), "./routes/_layout.sail");
        assert_eq!(layout_module_path("/studio"), "./routes/studio/_layout.sail");
        assert_eq!(layout_module_path("/a/b"), "./routes/a/b/_layout.sail");
    }

    
    /// `routes.gen.ts` is DATA ONLY after the entry-split — route table +
    /// layout table + typed path builders + the `modules` / `layoutModules`
    /// maps. The framework router call moved to `entry.gen.ts` (the
    /// host-owned bootstrap). This test pins the data shape, not the
    /// router call.
    #[test]
    fn render_emits_data_only_route_table() {
        // Contract, not a text snapshot, and built from a GENERIC tree so a
        // framework test never names an app's routes. The data file must
        // ship the route table, the typed path builders, and the modules
        // map keyed by pattern — the same data the host's entry.gen.ts
        // hands to `mountRouter`. The retired v0 hand-rolled mount must
        // not reappear.
        let tree = RouteTree {
            label: "x".to_string(),
            routes: vec![
                Route {
                    pattern: "/".to_string(),
                    params: vec![],
                    builder: "index".to_string(),
                },
                Route {
                    pattern: "/a/:id".to_string(),
                    params: vec!["id".to_string()],
                    builder: "aId".to_string(),
                },
            ],
            layouts: vec![],
        };
        let ts = render_routes_gen_with(&tree, false);
        // DATA: route table + typed path builders + modules map.
        assert!(ts.contains("DYNAMIC_ROUTE_PATTERNS"), "route table emitted");
        assert!(ts.contains("export const routes = {"), "path-builder map emitted");
        assert!(ts.contains("export const modules:"), "modules map exported for entry.gen");
        for r in &tree.routes {
            assert!(
                ts.contains(&format!("'{}':", r.pattern)),
                "every route is in the modules map; `{}` is missing",
                r.pattern,
            );
        }
        // DATA-ONLY: no router call, no token CSS, no client wiring —
        // those moved to `entry.gen.ts` (the host-owned bootstrap).
        assert!(!ts.contains("mountRouter("), "routes.gen.ts is data-only — no mountRouter");
        assert!(
            !ts.contains("@ship/tokens/tokens.css"),
            "routes.gen.ts is data-only — the token import lives in entry.gen.ts",
        );
        assert!(!ts.contains("clients:"), "routes.gen.ts is data-only — no clients wiring");
        // The v0 hand-rolled index mount stays gone.
        assert!(!ts.contains("getElementById"), "the v0 hand-rolled index mount is gone");
    }

    /// The data file exports `layoutModules` as a typed map for `entry.gen.ts`
    /// to forward into `mountRouter({ layoutModules })`. Layouts stay imported
    /// as named modules (no tree-shake removal) and the registration side
    /// effect (custom-element `define`) fires on first paint. Layout sort
    /// order: '/' < '/studio' → layout0 = root, layout1 = studio.
    #[test]
    fn render_emits_layout_imports_and_layout_modules_map() {
        let tree = RouteTree {
            label: "x".to_string(),
            routes: vec![
                Route {
                    pattern: "/".to_string(),
                    params: vec![],
                    builder: "index".to_string(),
                },
            ],
            layouts: vec![
                Layout { path_prefix: "/".to_string(), tag: "ship-app-chrome".to_string() },
                Layout { path_prefix: "/studio".to_string(), tag: "sail-studio-chrome".to_string() },
            ],
        };
        let ts = render_routes_gen(&tree);
        // Named imports (not bare side-effect imports).
        assert!(
            ts.contains("import * as layout0 from './routes/_layout.sail';"),
            "root layout must be a named module import: {ts}",
        );
        assert!(
            ts.contains("import * as layout1 from './routes/studio/_layout.sail';"),
            "subdir layout must be a named module import: {ts}",
        );
        // Data-only shape: `layoutModules` is a typed export for entry.gen.ts.
        assert!(
            ts.contains("export const layoutModules:"),
            "routes.gen.ts must export the layoutModules map (entry.gen.ts consumes it): {ts}",
        );
        assert!(
            ts.contains("'/': layout0,"),
            "root prefix maps to layout0: {ts}",
        );
        assert!(
            ts.contains("'/studio': layout1,"),
            "studio prefix maps to layout1: {ts}",
        );
    }

    /// `entry.gen.ts` is the host-owned bootstrap: imports the data
    /// tables from `./routes.gen`, constructs the host's named `clients`
    /// map, and calls `mountRouter` with both. Pin the shape so a future
    /// emission tweak doesn't quietly drop the host-wiring or the token
    /// CSS side-effect.
    #[test]
    fn render_entry_gen_wires_clients_and_router() {
        let tree = RouteTree {
            label: "x".to_string(),
            routes: vec![Route {
                pattern: "/".to_string(),
                params: vec![],
                builder: "index".to_string(),
            }],
            layouts: vec![Layout {
                path_prefix: "/".to_string(),
                tag: "ship-app-chrome".to_string(),
            }],
        };
        let bootstrap = deck_config::EntryBootstrap {
            clients_module: "./src/lib/client.ts".to_string(),
            expose_names: vec!["ship".to_string(), "ops".to_string()],
        };
        let ts = render_entry_gen(&tree, Some(&bootstrap));
        // Host wiring: imports the named clients off the host's module
        // and spreads them into the `clients` map mountRouter receives.
        assert!(
            ts.contains("import { ship, ops } from './src/lib/client.ts'"),
            "host's clientsModule is imported and named imports are emitted: {ts}",
        );
        assert!(
            ts.contains("    ship: ship,"),
            "client map keys ship + ops present: {ts}",
        );
        assert!(
            ts.contains("    ops: ops,"),
            "client map keys ship + ops present: {ts}",
        );
        // Data wiring: imports the data tables from routes.gen.
        assert!(
            ts.contains("DYNAMIC_ROUTE_PATTERNS"),
            "data table import present: {ts}",
        );
        assert!(
            ts.contains("LAYOUTS"),
            "data layout-table import present: {ts}",
        );
        assert!(ts.contains("modules,"), "modules map imported: {ts}");
        assert!(ts.contains("layoutModules,"), "layoutModules map imported: {ts}");
        // The token import rides the bootstrap so the bundle ships
        // design tokens before first paint.
        assert!(
            ts.contains("import '@ship/tokens/tokens.css';"),
            "design-token CSS side-effect import lives in entry.gen.ts: {ts}",
        );
        // Router call shape: root, routes, layouts, modules, layoutModules, clients.
        assert!(ts.contains("mountRouter("), "mountRouter called: {ts}");
        assert!(ts.contains("clients: {"), "mountRouter receives clients: {ts}");
    }

    /// The DEFAULT bootstrap (no host `entry`): deck still emits an
    /// `entry.gen.ts` that calls `mountRouter` so a standalone app mounts on
    /// its own. No client-module import, no `clients` field — just the router
    /// call with routes/layouts/modules + the token CSS. This is the case
    /// that used to render nothing (routes.gen.ts-as-entry never mounted).
    #[test]
    fn render_entry_gen_default_bootstrap_mounts_without_clients() {
        let tree = RouteTree {
            label: "standalone".to_string(),
            routes: vec![Route {
                pattern: "/".to_string(),
                params: vec![],
                builder: "index".to_string(),
            }],
            layouts: vec![],
        };
        let ts = render_entry_gen(&tree, None);
        assert!(ts.contains("import { mountRouter } from '@ship/deck/router';"), "imports mountRouter: {ts}");
        assert!(ts.contains("mountRouter("), "calls mountRouter so the app mounts: {ts}");
        assert!(ts.contains("root: '#root'"), "mounts into #root: {ts}");
        assert!(ts.contains("import '@ship/tokens/tokens.css';"), "ships tokens before first paint: {ts}");
        // No host clients: neither a client-module import nor a clients field.
        assert!(!ts.contains("clients:"), "default bootstrap wires no clients: {ts}");
        assert!(!ts.contains("from './src/"), "default bootstrap imports no host clientsModule: {ts}");
    }


    // Regression: routes.gen.ts lives under the watched `apps/...` root and
    // the app build regenerates it every rebuild. An unconditional write
    // self-triggered the bundler watcher → endless rebuild/reload loop (shipd CPU
    // spin, ~6 reloads/sec). A second generate with an UNCHANGED tree must NOT
    // rewrite the file. Proof: make it read-only — a skipped write succeeds; an
    // attempted write fails with permission denied.
    #[cfg(unix)]
    #[test]
    fn unchanged_tree_does_not_rewrite_routes_gen() {
        use std::os::unix::fs::PermissionsExt;
        let root = tempfile::tempdir().unwrap();
        let r = root.path().join("routes/studio");
        std::fs::create_dir_all(r.join("agents/[id]")).unwrap();
        std::fs::write(r.join("agents/[id]/page.sail"), "<div/>").unwrap();
        let s = spec(root.path(), "shell");
        generate_routes(&[s.clone()]).unwrap();
        let target = &s.out;
        let mut perms = std::fs::metadata(target).unwrap().permissions();
        perms.set_mode(0o444); // read-only: any write attempt now errors
        std::fs::set_permissions(target, perms).unwrap();
        // Unchanged tree → write-if-changed must SKIP the write, so this is Ok.
        // Before the fix this errored (EACCES) — the loop's signature.
        generate_routes(&[s])
            .expect("unchanged routes must not rewrite the file (would self-trigger the watcher)");
    }

    /// After the entry-split, the design-token CSS side-effect import
    /// rides `entry.gen.ts` (the host-owned bootstrap) — NOT
    /// `routes.gen.ts` (data only). The router call + token import are
    /// both bootstrap-layer concerns: the framework emits the data;
    /// the host's bootstrap pulls tokens in + calls the router.
    ///
    /// The token CSS package stays the canonical home
    /// (`packages/sail/tokens/package.json` declares `./tokens.css` as
    /// a subpath export); `entry.gen.ts` is the conventional entry
    /// that pulls it in.
    #[test]
    fn ssot_tokens_import_pinned() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let tokens_pkg = manifest_dir
            .parent() // crates/deck
            .and_then(|p| p.parent()) // crates
            .and_then(|p| p.parent()) // repo root
            .expect("CARGO_MANIFEST_DIR has three ancestors")
            .join("packages/sail/tokens/package.json");
        let pkg = std::fs::read_to_string(&tokens_pkg)
            .unwrap_or_else(|e| panic!("read {}: {e}", tokens_pkg.display()));
        assert!(
            pkg.contains("\"./tokens.css\": \"./src/tokens.css\""),
            "tokens package.json must expose the token CSS subpath `\
             ./tokens.css → ./src/tokens.css` (got: {pkg})",
        );

        let tree = RouteTree {
            label: "x".to_string(),
            routes: vec![Route {
                pattern: "/".to_string(),
                params: vec![],
                builder: "index".to_string(),
            }],
            layouts: vec![],
        };
        // `entry.gen.ts` must import the token CSS — that's the bootstrap
        // layer's responsibility. The data file must NOT (data-only
        // contract).
        let bootstrap = deck_config::EntryBootstrap {
            clients_module: "./src/lib/client.ts".to_string(),
            expose_names: vec!["ship".to_string()],
        };
        let entry = render_entry_gen(&tree, Some(&bootstrap));
        assert!(
            entry.contains("import '@ship/tokens/tokens.css';"),
            "rendered entry.gen.ts must import @ship/tokens/tokens.css (got: {entry})",
        );
        // And the old daemon-served link must be gone — it's a static-only
        // deploy hazard and the SSOT literal this used to pin (`<DECK_API_ASSETS>
        // /view-tokens.css`) is now exclusively deck-core's concern.
        assert!(
            !entry.contains("view-tokens.css"),
            "rendered entry.gen.ts must NOT reference the daemon-served view-tokens.css path (got: {entry})",
        );
        let routes = render_routes_gen_with(&tree, false);
        assert!(
            !routes.contains("@ship/tokens/tokens.css"),
            "rendered routes.gen.ts is data-only — must NOT import tokens (got: {routes})",
        );
    }

    /// `generate_view_artifacts` is the framework-owned view-table codegen
    /// `build_all` calls per bucket. Given a `views/` bucket with manifests, it
    /// must emit BOTH `routes.gen.ts` (router fields: pattern + view) and
    /// `meta.gen.ts` (host fields: title/layout/auth/rail) — the host-agnostic
    /// split is the contract. JSONC comments in the manifest are tolerated.
    #[test]
    fn generate_view_artifacts_emits_routes_and_meta() {
        let root = tempfile::tempdir().unwrap();
        let views = root.path().join("views");
        std::fs::create_dir_all(views.join("git")).unwrap();
        std::fs::write(
            views.join("git/manifest.jsonc"),
            "// a comment\n{ \"name\": \"git\", \"path\": \"/git\", \"title\": \"Git\", \
             \"layout\": \"default\", \"auth\": \"user\", \
             \"rail\": { \"label\": \"Git\", \"icon\": \"git-branch\" } }\n",
        )
        .unwrap();
        std::fs::create_dir_all(views.join("eval")).unwrap();
        std::fs::write(
            views.join("eval/manifest.jsonc"),
            "{ \"name\": \"eval\", \"path\": \"/eval\" }\n",
        )
        .unwrap();

        let n = generate_view_artifacts(&views, "views").unwrap();
        assert_eq!(n, 2, "two manifests discovered");

        // routes.gen.ts: router contract (pattern + view name). The host table
        // is a SEPARATE file — the router export must not appear here.
        let routes = std::fs::read_to_string(views.join("routes.gen.ts")).unwrap();
        assert!(routes.contains("VIEW_ROUTES"), "router table present");
        assert!(routes.contains("pattern: '/git', view: 'git'"), "git route entry");
        assert!(routes.contains("pattern: '/eval', view: 'eval'"), "eval route entry");
        assert!(!routes.contains("VIEW_META"), "host table stays out of the router file");

        // meta.gen.ts: host contract (title/layout/auth/rail). The router table
        // export must not appear here — the split is the contract.
        let meta = std::fs::read_to_string(views.join("meta.gen.ts")).unwrap();
        assert!(meta.contains("VIEW_META"), "host table present");
        assert!(meta.contains("name: 'git'") && meta.contains("title: 'Git'"), "git meta");
        assert!(meta.contains("\"icon\":\"git-branch\""), "rail rides through to meta");
        assert!(!meta.contains("VIEW_ROUTES"), "router table stays out of the meta file");
    }

    /// No-op safety: an absent bucket dir and a present-but-empty bucket both
    /// return 0 and write nothing. `build_all` calls this for EVERY configured
    /// bucket (the default `views`, plus any a host overrides in); a bucket a
    /// project does not have must not get an empty artifact written there.
    #[test]
    fn generate_view_artifacts_noop_without_dir_or_manifests() {
        let root = tempfile::tempdir().unwrap();
        // Absent dir → 0, nothing written.
        let absent = root.path().join("ship-views");
        assert_eq!(generate_view_artifacts(&absent, "x").unwrap(), 0);
        assert!(!absent.exists());
        // Present but no manifests → 0, no artifacts.
        let empty = root.path().join("views");
        std::fs::create_dir_all(&empty).unwrap();
        assert_eq!(generate_view_artifacts(&empty, "x").unwrap(), 0);
        assert!(!empty.join("routes.gen.ts").exists());
        assert!(!empty.join("meta.gen.ts").exists());
    }
}
