//! Icon roles, pack pins, and compile-time resolve (SPEC-ICONS-1).
//!
//! Sail owns the role set and the role→(pack, glyph) map. Deck only
//! materializes already-resolved used glyph bytes. Author markup uses
//! roles (`<icon name="settings">`), never pack prefixes.

use std::collections::BTreeMap;
use std::path::Path;

use crate::diagnostics::Span;
use crate::ir::{AttrInstr, Instr};

/// Framework default pack (Iconify collection id).
pub const DEFAULT_PACK: &str = "lucide";

/// Stable rule id for an unknown / mistyped icon role.
pub const RULE_UNKNOWN_ICON: &str = "sail-unknown-icon";

/// Stable rule id for a role bound to a missing offline glyph.
pub const RULE_ICON_GLYPH: &str = "sail-icon-glyph";

/// One row of the Iconify discovery catalog (install targets, not roles).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackCatalogEntry {
    pub id: &'static str,
    pub title: &'static str,
    pub summary: &'static str,
}

/// Iconify sets agents/humans may pin in `sail.config`. Not embedded.
pub const ICONIFY_CATALOG: &[PackCatalogEntry] = &[
    PackCatalogEntry {
        id: "lucide",
        title: "Lucide",
        summary: "outline UI set — framework default",
    },
    PackCatalogEntry {
        id: "mdi",
        title: "Material Design Icons",
        summary: "Google-style filled / outline",
    },
    PackCatalogEntry {
        id: "heroicons-outline",
        title: "Heroicons outline",
        summary: "Tailwind-adjacent outline",
    },
    PackCatalogEntry {
        id: "heroicons-solid",
        title: "Heroicons solid",
        summary: "Tailwind-adjacent filled",
    },
    PackCatalogEntry {
        id: "tabler",
        title: "Tabler",
        summary: "outline UI",
    },
    PackCatalogEntry {
        id: "ph",
        title: "Phosphor",
        summary: "flexible weights",
    },
    PackCatalogEntry {
        id: "feather",
        title: "Feather",
        summary: "minimal outline",
    },
];

/// Core role → default-pack glyph. Derived from stdlib + framework surfaces
/// plus identity rows for names already used as roles in the corpus.
/// Closed and versioned with the release.
const CORE_ROLES: &[CoreRole] = &[
    // Navigation
    CoreRole::new("home", "home", "go home"),
    CoreRole::new("menu", "menu", "open navigation"),
    CoreRole::new("close", "x", "dismiss / close"),
    CoreRole::new("search", "search", "search"),
    // Actions
    CoreRole::new("settings", "settings", "settings / preferences"),
    CoreRole::new("edit", "pencil", "edit"),
    CoreRole::new("trash", "trash-2", "delete"),
    CoreRole::new("add", "plus", "add / create"),
    CoreRole::new("check", "check", "confirm / done"),
    CoreRole::new("download", "download", "download"),
    CoreRole::new("plus", "plus", "add"),
    CoreRole::new("send", "send", "send"),
    CoreRole::new("attach", "paperclip", "attach"),
    // Direction
    CoreRole::new("arrow-left", "arrow-left", "back / previous"),
    CoreRole::new("arrow-right", "arrow-right", "forward / next"),
    CoreRole::new("chevron-down", "chevron-down", "expand / sort desc"),
    CoreRole::new("chevron-up", "chevron-up", "collapse / sort asc"),
    CoreRole::new("chevron-right", "chevron-right", "drill in"),
    CoreRole::new("chevron-left", "chevron-left", "drill out"),
    // Status
    CoreRole::new("info", "info", "informational"),
    CoreRole::new("warning", "triangle-alert", "warning"),
    CoreRole::new("success", "check-circle", "success"),
    CoreRole::new("error", "x-circle", "error"),
    // Media
    CoreRole::new("image", "image", "image / media"),
    CoreRole::new("file", "file", "file"),
    // Framework / studio surfaces (identity + aliases)
    CoreRole::new("activity", "activity", "activity"),
    CoreRole::new("alarm-clock", "alarm-clock", "alarm"),
    CoreRole::new("alert-triangle", "alert-triangle", "warning (lucide id)"),
    CoreRole::new("check-circle", "check-circle", "success (lucide id)"),
    CoreRole::new("circle", "circle", "circle"),
    CoreRole::new("compass", "compass", "explore"),
    CoreRole::new("ellipsis", "ellipsis", "more actions"),
    CoreRole::new("file-code", "file-code-2", "source file"),
    CoreRole::new("file-code-2", "file-code-2", "source file (lucide id)"),
    CoreRole::new("folder", "folder", "folder"),
    CoreRole::new("git", "git-branch", "git"),
    CoreRole::new("git-branch", "git-branch", "git branch"),
    CoreRole::new("heart", "heart", "favorite"),
    CoreRole::new("inbox", "inbox", "empty / inbox"),
    CoreRole::new("log-out", "log-out", "sign out"),
    CoreRole::new("mail", "mail", "mail"),
    CoreRole::new("moon", "moon", "dark mode"),
    CoreRole::new("more-vertical", "more-vertical", "overflow menu"),
    CoreRole::new("radio", "radio", "live / broadcast"),
    CoreRole::new("redo", "redo-2", "redo"),
    CoreRole::new("redo-2", "redo-2", "redo (lucide id)"),
    CoreRole::new("refresh", "refresh-cw", "refresh"),
    CoreRole::new("refresh-cw", "refresh-cw", "refresh (lucide id)"),
    CoreRole::new("rocket", "rocket", "launch"),
    CoreRole::new("ship", "ship", "ship / brand"),
    CoreRole::new("sparkles", "sparkles", "highlight"),
    CoreRole::new("star", "star", "star"),
    CoreRole::new("stop", "stop-circle", "stop"),
    CoreRole::new("sun", "sun", "light mode"),
    CoreRole::new("trash-2", "trash-2", "delete (lucide id)"),
    CoreRole::new("undo", "undo-2", "undo"),
    CoreRole::new("undo-2", "undo-2", "undo (lucide id)"),
    CoreRole::new("user", "user", "user / account"),
    CoreRole::new("bell", "bell", "notifications"),
    CoreRole::new("x", "x", "close (lucide id)"),
    CoreRole::new("x-circle", "x-circle", "error (lucide id)"),
    CoreRole::new("zap", "zap", "power / instant"),
];

struct CoreRole {
    role: &'static str,
    glyph: &'static str,
    summary: &'static str,
}

impl CoreRole {
    const fn new(role: &'static str, glyph: &'static str, summary: &'static str) -> Self {
        Self {
            role,
            glyph,
            summary,
        }
    }
}

/// Pack id + optional version pin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackRef {
    pub id: String,
    pub version: Option<String>,
}

impl PackRef {
    pub fn new(id: impl Into<String>, version: Option<String>) -> Self {
        Self {
            id: id.into(),
            version,
        }
    }
}

/// `role → { pack, glyph }`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleBinding {
    pub pack: String,
    pub glyph: String,
}

/// Project icons section after merge (core ∪ app overrides).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IconConfig {
    pub pack: PackRef,
    pub packs: BTreeMap<String, PackRef>,
    pub roles: BTreeMap<String, RoleBinding>,
}

impl Default for IconConfig {
    fn default() -> Self {
        Self {
            pack: PackRef::new(DEFAULT_PACK, None),
            packs: BTreeMap::new(),
            roles: BTreeMap::new(),
        }
    }
}

/// Effective role table + default pack for one compile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleRegistry {
    pack: PackRef,
    packs: BTreeMap<String, PackRef>,
    /// role → (binding, summary). Summary empty for app-only roles.
    roles: BTreeMap<String, (RoleBinding, String)>,
}

/// Machine-readable role row for MCP / LSP / skill tables.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct RoleSurface {
    pub role: String,
    pub summary: String,
    pub pack: String,
    pub glyph: String,
}

/// Machine-readable configured / catalog pack row.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct PackSurface {
    pub id: String,
    pub title: String,
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub configured: bool,
}

/// Used (pack, glyph) after role resolve.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct UsedGlyph {
    pub role: String,
    pub pack: String,
    pub glyph: String,
}

impl RoleRegistry {
    /// Framework core only (no project config).
    pub fn core() -> Self {
        Self::from_config(IconConfig::default())
    }

    pub fn from_config(cfg: IconConfig) -> Self {
        let default_pack = cfg.pack.id.clone();
        let mut roles = BTreeMap::new();
        for row in CORE_ROLES {
            roles.insert(
                row.role.to_string(),
                (
                    RoleBinding {
                        pack: default_pack.clone(),
                        glyph: row.glyph.to_string(),
                    },
                    row.summary.to_string(),
                ),
            );
        }
        for (role, binding) in cfg.roles {
            let summary = roles.get(&role).map(|(_, s)| s.clone()).unwrap_or_default();
            roles.insert(role, (binding, summary));
        }
        let mut packs = cfg.packs;
        packs
            .entry(cfg.pack.id.clone())
            .or_insert_with(|| cfg.pack.clone());
        Self {
            pack: cfg.pack,
            packs,
            roles,
        }
    }

    /// Load core ∪ nearest `sail.config` `icons` for `source_path`.
    pub fn load(source_path: Option<&Path>) -> Self {
        let Some(path) = source_path else {
            return Self::core();
        };
        #[cfg(target_arch = "wasm32")]
        {
            let _ = path;
            return Self::core();
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            match sail_lint::config::resolve_product(path) {
                Some((_, cfg)) => Self::from_config(parse_icon_config(cfg.icons_value().as_ref())),
                None => Self::core(),
            }
        }
    }

    /// Load from a project root (`sail.config.jsonc` first).
    pub fn load_from_project(project: &Path) -> Self {
        Self::load(Some(project))
    }

    pub fn default_pack(&self) -> &PackRef {
        &self.pack
    }

    pub fn configured_packs(&self) -> &BTreeMap<String, PackRef> {
        &self.packs
    }

    pub fn resolve(&self, role: &str) -> Option<&RoleBinding> {
        self.roles.get(role).map(|(b, _)| b)
    }

    pub fn contains(&self, role: &str) -> bool {
        self.roles.contains_key(role)
    }

    pub fn role_ids(&self) -> Vec<String> {
        self.roles.keys().cloned().collect()
    }

    pub fn role_surfaces(&self) -> Vec<RoleSurface> {
        self.roles
            .iter()
            .map(|(role, (binding, summary))| RoleSurface {
                role: role.clone(),
                summary: summary.clone(),
                pack: binding.pack.clone(),
                glyph: binding.glyph.clone(),
            })
            .collect()
    }

    pub fn pack_surfaces(&self) -> Vec<PackSurface> {
        let mut out = Vec::new();
        for entry in ICONIFY_CATALOG {
            let pin = self.packs.get(entry.id);
            out.push(PackSurface {
                id: entry.id.to_string(),
                title: entry.title.to_string(),
                summary: entry.summary.to_string(),
                version: pin.and_then(|p| p.version.clone()),
                configured: pin.is_some() || entry.id == self.pack.id,
            });
        }
        for (id, pin) in &self.packs {
            if ICONIFY_CATALOG.iter().any(|e| e.id == id) {
                continue;
            }
            out.push(PackSurface {
                id: id.clone(),
                title: id.clone(),
                summary: "project pack".into(),
                version: pin.version.clone(),
                configured: true,
            });
        }
        out
    }

    /// Closest role ids (Levenshtein), closest first.
    pub fn close_matches(&self, unknown: &str) -> Vec<String> {
        let mut scored: Vec<(usize, &str)> = Vec::new();
        for role in self.roles.keys() {
            let d = levenshtein(unknown, role);
            let max_d = (role.len().max(unknown.len()) / 3).max(2);
            if d <= max_d {
                scored.push((d, role.as_str()));
            }
        }
        scored.sort_by_key(|(d, name)| (*d, *name));
        scored
            .into_iter()
            .take(5)
            .map(|(_, n)| n.to_string())
            .collect()
    }
}

/// Parse the `icons` object. Absent / wrong-typed → defaults.
pub fn parse_icon_config(value: Option<&serde_json::Value>) -> IconConfig {
    let Some(v) = value else {
        return IconConfig::default();
    };
    if !v.is_object() {
        return IconConfig::default();
    }
    let mut cfg = IconConfig::default();
    parse_pack_field(v.get("pack"), &mut cfg.pack);
    if let Some(ver) = v.get("version").and_then(|x| x.as_str()) {
        if !ver.is_empty() && cfg.pack.version.is_none() {
            cfg.pack.version = Some(ver.to_string());
        }
    }
    if let Some(obj) = v.get("packs").and_then(|x| x.as_object()) {
        for (id, spec) in obj {
            let mut pin = PackRef::new(id.clone(), None);
            if let Some(ver) = spec.get("version").and_then(|x| x.as_str()) {
                pin.version = Some(ver.to_string());
            } else if let Some(ver) = spec.as_str() {
                pin.version = Some(ver.to_string());
            }
            cfg.packs.insert(id.clone(), pin);
        }
    }
    if let Some(obj) = v.get("roles").and_then(|x| x.as_object()) {
        for (role, spec) in obj {
            if let Some(binding) = parse_role_binding(spec, &cfg.pack.id) {
                if binding.pack != cfg.pack.id {
                    cfg.packs
                        .entry(binding.pack.clone())
                        .or_insert_with(|| PackRef::new(binding.pack.clone(), None));
                }
                cfg.roles.insert(role.clone(), binding);
            }
        }
    }
    cfg.packs
        .entry(cfg.pack.id.clone())
        .or_insert_with(|| cfg.pack.clone());
    cfg
}

fn parse_pack_field(value: Option<&serde_json::Value>, pack: &mut PackRef) {
    let Some(v) = value else {
        return;
    };
    if let Some(id) = v.as_str() {
        if !id.is_empty() {
            pack.id = id.to_string();
        }
        return;
    }
    if let Some(obj) = v.as_object() {
        if let Some(id) = obj.get("id").and_then(|x| x.as_str()) {
            if !id.is_empty() {
                pack.id = id.to_string();
            }
        }
        if let Some(ver) = obj.get("version").and_then(|x| x.as_str()) {
            if !ver.is_empty() {
                pack.version = Some(ver.to_string());
            }
        }
    }
}

fn parse_role_binding(spec: &serde_json::Value, default_pack: &str) -> Option<RoleBinding> {
    if let Some(s) = spec.as_str() {
        let s = s.trim();
        if s.is_empty() {
            return None;
        }
        if let Some((pack, glyph)) = s.split_once(':') {
            if pack.is_empty() || glyph.is_empty() {
                return None;
            }
            return Some(RoleBinding {
                pack: pack.to_string(),
                glyph: glyph.to_string(),
            });
        }
        return Some(RoleBinding {
            pack: default_pack.to_string(),
            glyph: s.to_string(),
        });
    }
    let obj = spec.as_object()?;
    let glyph = obj.get("glyph").and_then(|x| x.as_str())?.trim();
    if glyph.is_empty() {
        return None;
    }
    let pack = obj
        .get("pack")
        .and_then(|x| x.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(default_pack);
    Some(RoleBinding {
        pack: pack.to_string(),
        glyph: glyph.to_string(),
    })
}

/// Teaching diagnostic for an unknown role.
pub fn unknown_role_message(role: &str, registry: &RoleRegistry) -> String {
    if let Some((pack, glyph)) = role.split_once(':') {
        if !pack.is_empty() && !glyph.is_empty() {
            if registry.contains(glyph) {
                return format!(
                    "unknown icon role `{role}` — author markup uses roles, not pack prefixes. \
                     Did you mean `{glyph}`?"
                );
            }
            let hints = registry.close_matches(glyph);
            return format!(
                "unknown icon role `{role}` — author markup uses roles, not pack prefixes.{}",
                format_hints(&hints)
            );
        }
    }
    let hints = registry.close_matches(role);
    if hints.is_empty() {
        format!(
            "unknown icon role `{role}` — not in the project role set. \
             Add it under sail.config `icons.roles` or pick a core role \
             (search, settings, home, …)."
        )
    } else {
        format!(
            "unknown icon role `{role}` — did you mean {}?",
            hints
                .iter()
                .map(|h| format!("`{h}`"))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

fn format_hints(hints: &[String]) -> String {
    if hints.is_empty() {
        String::new()
    } else {
        format!(
            " Did you mean {}?",
            hints
                .iter()
                .map(|h| format!("`{h}`"))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

/// Static `name=` on an icon host, with span.
pub fn static_icon_role(el: &crate::ir::ElementInstr) -> Option<(String, Span)> {
    if !is_icon_host(el) {
        return None;
    }
    for a in &el.attrs {
        if let AttrInstr::Static { name, value, span } = a {
            if name == "name" {
                let v = value.trim();
                if !v.is_empty() {
                    return Some((v.to_string(), *span));
                }
            }
        }
    }
    None
}

pub fn is_icon_host(el: &crate::ir::ElementInstr) -> bool {
    let tag = el.tag.as_str();
    if tag == "icon" || tag == "sail-icon" {
        return true;
    }
    el.attrs.iter().any(|a| {
        matches!(
            a,
            AttrInstr::Use {
                action,
                ..
            } if action == "icon"
        )
    })
}

/// Walk template and collect static icon role names (no resolve).
pub fn collect_static_roles(instrs: &[Instr]) -> Vec<(String, Span)> {
    let mut out = Vec::new();
    visit_roles(instrs, &mut out);
    out
}

fn visit_roles(instrs: &[Instr], out: &mut Vec<(String, Span)>) {
    for instr in instrs {
        match instr {
            Instr::Element(el) => {
                if let Some(hit) = static_icon_role(el) {
                    out.push(hit);
                }
                visit_roles(&el.children, out);
            }
            Instr::If(i) => {
                for branch in &i.branches {
                    visit_roles(&branch.body, out);
                }
                if let Some(else_body) = &i.else_body {
                    visit_roles(else_body, out);
                }
            }
            Instr::For(f) => {
                visit_roles(&f.body, out);
                if let Some(else_body) = &f.else_body {
                    visit_roles(else_body, out);
                }
            }
            Instr::Match(m) => {
                for case in &m.cases {
                    visit_roles(&case.body, out);
                }
                if let Some(default) = &m.default {
                    visit_roles(default, out);
                }
            }
            Instr::When(w) => {
                visit_roles(&w.body, out);
                if let Some(e) = &w.else_body {
                    visit_roles(e, out);
                }
            }
            Instr::StaticText { .. } | Instr::DynamicText { .. } | Instr::Comment { .. } => {}
        }
    }
}

/// Resolve collected role names against `registry`. Unknown roles omitted.
pub fn resolve_used_glyphs(
    roles: impl IntoIterator<Item = String>,
    registry: &RoleRegistry,
) -> Vec<UsedGlyph> {
    let mut out = Vec::new();
    for role in roles {
        if let Some(b) = registry.resolve(&role) {
            out.push(UsedGlyph {
                role,
                pack: b.pack.clone(),
                glyph: b.glyph.clone(),
            });
        }
    }
    out.sort();
    out.dedup();
    out
}

/// Roots to search for offline pack data (`node_modules`, crate dir).
pub fn default_glyph_search_roots() -> Vec<std::path::PathBuf> {
    let mut roots = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        roots.push(cwd.clone());
        roots.push(cwd.join("node_modules"));
    }
    let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    roots.push(manifest.clone());
    roots.push(manifest.join("../../../node_modules"));
    roots
}

/// Load a pack glyph as a standalone SVG. Offline only — never hits a CDN.
///
/// Search order per root: `@iconify-json/{pack}/icons.json`, then
/// `lucide-static/icons/{glyph}.svg` when `pack == lucide`.
#[cfg(not(target_arch = "wasm32"))]
pub fn load_glyph_svg(pack: &str, glyph: &str, search_roots: &[&Path]) -> Option<String> {
    if !is_safe_id(pack) || !is_safe_id(glyph) {
        return None;
    }
    for root in search_roots {
        if let Some(svg) = load_from_iconify_json(pack, glyph, root) {
            return Some(svg);
        }
        if pack == "lucide" {
            if let Some(svg) = load_from_lucide_static(glyph, root) {
                return Some(svg);
            }
        }
        // Walk up so a crate test dir still finds the workspace node_modules.
        let mut cur = root.parent();
        while let Some(dir) = cur {
            if let Some(svg) = load_from_iconify_json(pack, glyph, dir) {
                return Some(svg);
            }
            if pack == "lucide" {
                if let Some(svg) = load_from_lucide_static(glyph, dir) {
                    return Some(svg);
                }
            }
            cur = dir.parent();
        }
    }
    None
}

#[cfg(target_arch = "wasm32")]
pub fn load_glyph_svg(_pack: &str, _glyph: &str, _search_roots: &[&Path]) -> Option<String> {
    None
}

/// Glyph ids in one installed pack (config binding completion, not markup).
#[cfg(not(target_arch = "wasm32"))]
pub fn pack_glyph_ids(pack: &str, search_roots: &[&Path]) -> Vec<String> {
    if !is_safe_id(pack) {
        return Vec::new();
    }
    for root in search_roots {
        if let Some(ids) = iconify_glyph_ids(pack, root) {
            return ids;
        }
        if pack == "lucide" {
            if let Some(ids) = lucide_static_ids(root) {
                return ids;
            }
        }
        let mut cur = root.parent();
        while let Some(dir) = cur {
            if let Some(ids) = iconify_glyph_ids(pack, dir) {
                return ids;
            }
            if pack == "lucide" {
                if let Some(ids) = lucide_static_ids(dir) {
                    return ids;
                }
            }
            cur = dir.parent();
        }
    }
    Vec::new()
}

#[cfg(target_arch = "wasm32")]
pub fn pack_glyph_ids(_pack: &str, _search_roots: &[&Path]) -> Vec<String> {
    Vec::new()
}

fn is_safe_id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

#[cfg(not(target_arch = "wasm32"))]
fn node_modules(root: &Path) -> std::path::PathBuf {
    root.join("node_modules")
}

#[cfg(not(target_arch = "wasm32"))]
fn load_from_iconify_json(pack: &str, glyph: &str, root: &Path) -> Option<String> {
    let path = node_modules(root)
        .join("@iconify-json")
        .join(pack)
        .join("icons.json");
    let raw = std::fs::read_to_string(path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&raw).ok()?;
    let icon = v.get("icons")?.get(glyph)?;
    let body = icon.get("body")?.as_str()?;
    let width = icon
        .get("width")
        .or_else(|| v.get("width"))
        .and_then(|x| x.as_u64())
        .unwrap_or(24);
    let height = icon
        .get("height")
        .or_else(|| v.get("height"))
        .and_then(|x| x.as_u64())
        .unwrap_or(24);
    Some(wrap_svg(body, width, height))
}

#[cfg(not(target_arch = "wasm32"))]
fn iconify_glyph_ids(pack: &str, root: &Path) -> Option<Vec<String>> {
    let path = node_modules(root)
        .join("@iconify-json")
        .join(pack)
        .join("icons.json");
    let raw = std::fs::read_to_string(path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&raw).ok()?;
    let mut ids: Vec<String> = v.get("icons")?.as_object()?.keys().cloned().collect();
    ids.sort();
    Some(ids)
}

#[cfg(not(target_arch = "wasm32"))]
fn load_from_lucide_static(glyph: &str, root: &Path) -> Option<String> {
    let path = node_modules(root)
        .join("lucide-static")
        .join("icons")
        .join(format!("{glyph}.svg"));
    let raw = std::fs::read_to_string(path).ok()?;
    let inner = extract_inner(&raw)?;
    Some(wrap_svg(&inner, 24, 24))
}

#[cfg(not(target_arch = "wasm32"))]
fn lucide_static_ids(root: &Path) -> Option<Vec<String>> {
    let dir = node_modules(root).join("lucide-static").join("icons");
    let entries = std::fs::read_dir(dir).ok()?;
    let mut ids = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if let Some(stem) = name.strip_suffix(".svg") {
            ids.push(stem.to_string());
        }
    }
    if ids.is_empty() {
        return None;
    }
    ids.sort();
    Some(ids)
}

/// Canonical currentColor envelope (same as the retired daemon wrap).
pub fn wrap_svg(inner: &str, width: u64, height: u64) -> String {
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {width} {height}\" fill=\"none\" \
         stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" \
         stroke-linejoin=\"round\">{inner}</svg>"
    )
}

fn extract_inner(svg: &str) -> Option<String> {
    let mut s = String::with_capacity(svg.len());
    let bytes = svg.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i..].starts_with(b"<!--") {
            let rest = &svg[i..];
            if let Some(end) = rest.find("-->") {
                i += end + 3;
                continue;
            }
            return None;
        }
        s.push(bytes[i] as char);
        i += 1;
    }
    let stripped = s.trim();
    let open = stripped.find("<svg")?;
    let open_end = stripped[open..].find('>').map(|n| open + n)?;
    let close_start = stripped.rfind("</svg>")?;
    if close_start <= open_end {
        return None;
    }
    let inner = stripped[open_end + 1..close_start].trim();
    let mut out = String::with_capacity(inner.len());
    let mut in_ws = false;
    for c in inner.chars() {
        if c.is_whitespace() {
            if !in_ws {
                out.push(' ');
                in_ws = true;
            }
        } else {
            out.push(c);
            in_ws = false;
        }
    }
    Some(out.trim().to_string())
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.is_empty() {
        return b.len();
    }
    if b.is_empty() {
        return a.len();
    }
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0; b.len() + 1];
    for i in 1..=a.len() {
        cur[0] = i;
        for j in 1..=b.len() {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            cur[j] = (prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_includes_usage_derived_roles() {
        let r = RoleRegistry::core();
        for name in ["search", "settings", "home", "trash", "inbox", "git"] {
            assert!(r.contains(name), "missing core role {name}");
        }
        let git = r.resolve("git").expect("git");
        assert_eq!(git.glyph, "git-branch");
        let trash = r.resolve("trash").expect("trash");
        assert_eq!(trash.glyph, "trash-2");
    }

    #[test]
    fn catalog_and_studio_roles_resolve() {
        let r = RoleRegistry::core();
        for name in [
            "sparkles",
            "heart",
            "inbox",
            "plus",
            "download",
            "user",
            "bell",
            "mail",
            "close",
            "refresh",
            "trash",
            "file-code",
            "undo",
            "redo",
            "ellipsis",
            "file",
            "compass",
            "image",
            "circle",
            "star",
            "home",
            "check",
            "edit",
        ] {
            assert!(r.contains(name), "missing catalog/studio role {name}");
        }
    }

    #[test]
    fn lucide_id_aliases_resolve_to_same_glyph() {
        let r = RoleRegistry::core();
        for (role, alias, glyph) in [
            ("refresh", "refresh-cw", "refresh-cw"),
            ("trash", "trash-2", "trash-2"),
            ("close", "x", "x"),
            ("file-code", "file-code-2", "file-code-2"),
            ("git", "git-branch", "git-branch"),
            ("undo", "undo-2", "undo-2"),
            ("redo", "redo-2", "redo-2"),
        ] {
            let a = r.resolve(role).unwrap_or_else(|| panic!("role {role}"));
            let b = r.resolve(alias).unwrap_or_else(|| panic!("alias {alias}"));
            assert_eq!(a.glyph, glyph, "role {role}");
            assert_eq!(b.glyph, glyph, "alias {alias}");
            assert_eq!(a.pack, b.pack);
        }
    }

    #[test]
    fn resolve_is_offline_pack_and_glyph_not_a_url() {
        let r = RoleRegistry::core();
        for role in r.role_ids() {
            let b = r.resolve(&role).expect(role.as_str());
            assert!(
                !b.pack.contains("://") && !b.glyph.contains("://"),
                "role {role} must not resolve to a URL (no CDN): {b:?}"
            );
        }
    }

    #[test]
    fn app_override_and_custom_role() {
        let cfg = parse_icon_config(Some(&serde_json::json!({
            "pack": "lucide",
            "roles": {
                "home": "house",
                "settings": "mdi:cog",
                "brand-mark": { "pack": "app", "glyph": "logo" }
            }
        })));
        let r = RoleRegistry::from_config(cfg);
        assert_eq!(r.resolve("home").unwrap().glyph, "house");
        let settings = r.resolve("settings").unwrap();
        assert_eq!(settings.pack, "mdi");
        assert_eq!(settings.glyph, "cog");
        let brand = r.resolve("brand-mark").unwrap();
        assert_eq!(brand.pack, "app");
        assert_eq!(brand.glyph, "logo");
        assert!(r.configured_packs().contains_key("mdi"));
        assert!(r.configured_packs().contains_key("app"));
    }

    #[test]
    fn pack_object_pin() {
        let cfg = parse_icon_config(Some(&serde_json::json!({
            "pack": { "id": "lucide", "version": "1.28.0" }
        })));
        assert_eq!(cfg.pack.id, "lucide");
        assert_eq!(cfg.pack.version.as_deref(), Some("1.28.0"));
    }

    #[test]
    fn close_match_settings() {
        let r = RoleRegistry::core();
        let hints = r.close_matches("setttings");
        assert!(
            hints.iter().any(|h| h == "settings"),
            "expected settings in {hints:?}"
        );
        let msg = unknown_role_message("setttings", &r);
        assert!(msg.contains("settings"), "{msg}");
        assert!(msg.contains("did you mean"), "{msg}");
    }

    #[test]
    fn prefix_is_refused_with_role_hint() {
        let r = RoleRegistry::core();
        let msg = unknown_role_message("lucide:settings", &r);
        assert!(msg.contains("roles, not pack prefixes"), "{msg}");
        assert!(msg.contains("`settings`"), "{msg}");
    }

    #[test]
    fn catalog_lists_lucide() {
        assert!(ICONIFY_CATALOG.iter().any(|e| e.id == "lucide"));
        let packs = RoleRegistry::core().pack_surfaces();
        assert!(packs.iter().any(|p| p.id == "lucide" && p.configured));
    }

    #[test]
    fn lucide_static_search_loads_when_installed() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let svg = load_glyph_svg("lucide", "search", &[root]);
        if svg.is_none() {
            // Minimal CI slices without node_modules — skip, don't fail.
            return;
        }
        let svg = svg.unwrap();
        assert!(svg.contains("<svg"), "{svg}");
        assert!(svg.contains("viewBox=\"0 0 24 24\""), "{svg}");
    }

    #[test]
    fn load_glyph_svg_is_offline_only() {
        let missing = Path::new("/tmp/sail-icons-no-pack");
        assert!(
            load_glyph_svg("lucide", "search", &[missing]).is_none(),
            "missing local pack must not invent a glyph (no CDN fallback)"
        );
        assert!(
            load_glyph_svg("https://cdn.example/lucide", "search", &[missing]).is_none(),
            "URL-shaped pack id must not load"
        );
    }
}
