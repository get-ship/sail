//! L1 component **rest surfaces** — author graph, storage identity, emit label.
//!
//! Three concepts (SPEC-D1):
//! - **Identity** — storage path (`component.chip.fg`)
//! - **Value resolution** — theme / default alias (generate)
//! - **Emit label** — CSS public name (`chip-foreground`) — generate only
//!
//! Style author spelling is graph-shaped and Tone-parallel:
//! `chip` / `chip.foreground` / `chip.border` (not `chip-foreground`).
//!
//! Surface vocabulary is legal only while compiling that L1 host unit
//! ([`crate::host_ctx`]). The compiler installs the pack name it owns
//! (L1 manifest). App/L2 Style cannot use surface identities.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::OnceLock;

use super::host_ctx;
use super::tokens;

/// Packs where storage leaf `track` is the default fill (like `.bg`).
const TRACK_AS_PRIMARY: &[&str] = &["meter", "progress", "switch"];

/// One rest surface slot on a host pack.
#[derive(Debug, Clone)]
pub struct SurfaceSlot {
    /// Author member segment; empty string = bare pack stem (default fill).
    pub author_member: &'static str,
    /// Storage leaf under `component.{pack}.` (`bg`, `fg`, `border`, …).
    pub storage_leaf: &'static str,
    /// Full storage identity (`component.chip.fg`).
    pub storage_path: &'static str,
    /// Emit label / CSS public name without `--` (`chip-foreground`).
    pub public_name: &'static str,
}

struct SurfaceIndex {
    by_pack: BTreeMap<&'static str, &'static [SurfaceSlot]>,
    /// storage path → public_name
    path_to_public: HashMap<&'static str, &'static str>,
    /// Rest-only public names that have dotted author form (dual-spelling ban).
    rest_public_names: HashSet<&'static str>,
}

fn index() -> &'static SurfaceIndex {
    static CACHE: OnceLock<SurfaceIndex> = OnceLock::new();
    CACHE.get_or_init(build_index)
}

fn leak_str(s: impl Into<String>) -> &'static str {
    Box::leak(s.into().into_boxed_str())
}

fn build_index() -> SurfaceIndex {
    let mut pack_slots: BTreeMap<String, Vec<SurfaceSlot>> = BTreeMap::new();
    let mut path_to_public: HashMap<&'static str, &'static str> = HashMap::new();
    let mut rest_public_names: HashSet<&'static str> = HashSet::new();

    for (path, css) in tokens::component_color_paths() {
        let path_s = leak_str(path);
        let css_s = leak_str(css);
        path_to_public.insert(path_s, css_s);

        let Some(rest) = path_s.strip_prefix("component.") else {
            continue;
        };
        let Some((pack, leaf)) = rest.split_once('.') else {
            continue;
        };
        // Folded composite color member (`border.color`) is the historical
        // surface slot (`border`). Other dotted leaves stay off the author graph.
        let storage_leaf = if let Some(slot) = leaf.strip_suffix(".color") {
            if slot.is_empty() || slot.contains('.') {
                continue;
            }
            slot
        } else if leaf.contains('.') {
            continue;
        } else {
            leaf
        };
        // v1 rest-only Style surface slots — no hover/focus as author surface.
        if is_state_leaf(storage_leaf) {
            continue;
        }
        let author = author_member_for_leaf(pack, storage_leaf);
        let author_s = leak_str(author);
        let leaf_s = leak_str(storage_leaf);
        rest_public_names.insert(css_s);
        pack_slots
            .entry(pack.to_string())
            .or_default()
            .push(SurfaceSlot {
                author_member: author_s,
                storage_leaf: leaf_s,
                storage_path: path_s,
                public_name: css_s,
            });
    }

    let mut by_pack: BTreeMap<&'static str, &'static [SurfaceSlot]> = BTreeMap::new();
    for (pack, mut slots) in pack_slots {
        slots.sort_by(|a, b| a.author_member.cmp(b.author_member));
        let pack_s = leak_str(pack);
        let slice: &'static [SurfaceSlot] = Box::leak(slots.into_boxed_slice());
        by_pack.insert(pack_s, slice);
    }

    SurfaceIndex {
        by_pack,
        path_to_public,
        rest_public_names,
    }
}

fn is_state_leaf(leaf: &str) -> bool {
    leaf.contains("hover") || leaf.contains("focus")
}

/// Author member for a storage leaf (Tone-parallel: bare = fill, `.foreground` = on-fill).
fn author_member_for_leaf(pack: &str, leaf: &str) -> String {
    match leaf {
        "bg" => String::new(),
        "fg" => "foreground".into(),
        "track" if TRACK_AS_PRIMARY.contains(&pack) => String::new(),
        other => other.to_string(),
    }
}

/// Gone with `component.*`. Kept as an empty predicate so call sites compile.
pub fn is_component_public_name(_css_name: &str) -> bool {
    false
}

/// Gone with `component.*`. No theme-owned author roots remain.
pub fn is_theme_owned_author_root(_root: &str) -> bool {
    false
}

/// Every L1 surface pack name (sorted).
pub fn all_surface_packs() -> Vec<&'static str> {
    index().by_pack.keys().copied().collect()
}

/// Gone with `component.*`. No theme-owned token names remain.
pub fn is_theme_owned_token_name(_name: &str) -> bool {
    false
}

/// Whether `css_name` is a rest surface emit label (kebab form forbidden in Style).
pub fn is_rest_surface_public_name(css_name: &str) -> bool {
    index().rest_public_names.contains(css_name)
}

/// Surface pack in scope for the current host unit, if any.
///
/// This is whatever pack name the compiler installed — not a table in
/// this crate. `chip.foreground` is legal because the compiler said so.
pub fn current_surface_pack() -> Option<String> {
    host_ctx::current_host_surface()
}

/// Slots for a surface pack.
pub fn surface_slots(pack: &str) -> Option<&'static [SurfaceSlot]> {
    index().by_pack.get(pack).copied()
}

/// Resolve author path on the current host surface → emit public name.
///
/// `root` is the bare pack stem (`chip`); `path` is member segments
/// (`["foreground"]`, or empty for default fill).
pub fn resolve_surface_author(root: &str, path: &[String]) -> Result<&'static str, String> {
    let Some(pack) = current_surface_pack() else {
        return Err(format!(
            "component surface `{root}` is only legal in that L1 host unit"
        ));
    };
    if root != pack {
        return Err(format!(
            "component surface `{root}` is not in scope (this host surface is `{pack}`)"
        ));
    }
    let slots = surface_slots(&pack).ok_or_else(|| format!("unknown surface pack `{pack}`"))?;
    let member = match path {
        [] => "",
        [one] => one.as_str(),
        _ => {
            return Err(format!(
                "surface `{pack}` only supports one member segment (got `{pack}.{}`)",
                path.join(".")
            ));
        }
    };
    slots
        .iter()
        .find(|s| s.author_member == member)
        .map(|s| s.public_name)
        .ok_or_else(|| {
            let known: Vec<String> = slots
                .iter()
                .map(|s| {
                    if s.author_member.is_empty() {
                        pack.to_string()
                    } else {
                        format!("{pack}.{}", s.author_member)
                    }
                })
                .collect();
            let got = if member.is_empty() {
                pack.to_string()
            } else {
                format!("{pack}.{member}")
            };
            format!(
                "surface `{pack}` has no member `{got}` (known: {})",
                known.join(", ")
            )
        })
}

/// Author-facing forms for the in-scope surface (for complete/validate sets).
pub fn in_scope_surface_author_forms() -> Vec<String> {
    let Some(pack) = current_surface_pack() else {
        return Vec::new();
    };
    let Some(slots) = surface_slots(&pack) else {
        // Compiler installed a pack; token slot table may be empty
        // (skins spend globals). Tone-parallel author graph still holds.
        return vec![pack.clone(), format!("{pack}.foreground")];
    };
    let mut out = Vec::new();
    for s in slots {
        if s.author_member.is_empty() {
            out.push(pack.to_string());
        } else {
            out.push(format!("{pack}.{}", s.author_member));
        }
    }
    out.sort();
    out
}

/// Whether a theme/studio write path is a known component storage path.
pub fn is_known_component_storage_path(path: &str) -> bool {
    let p = path.trim();
    let p = p
        .strip_prefix("modes.light.")
        .or_else(|| p.strip_prefix("modes.dark."))
        .unwrap_or(p);
    let p = p.strip_suffix(".$value").unwrap_or(p);
    if index().path_to_public.contains_key(p) {
        return true;
    }
    // Non-color component leaves (radius, backdrop-filter, …).
    tokens::path_to_css_name(p).is_some() && p.starts_with("component.")
}

/// Color-paint vocabulary for a context: globals (+ keywords) + optional host surface forms.
/// Completions and validate sets share this SSOT.
pub fn color_paint_vocabulary(host: Option<&str>) -> HashSet<String> {
    let mut set: HashSet<String> = tokens::color_values()
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    // Static tone group paths (non-default members).
    for g in crate::token_groups::tone_groups() {
        for m in g.members {
            if m.name != "default" {
                set.insert(format!("{}.{}", g.name, m.name));
            }
        }
    }
    set.insert("tone".into());
    set.insert("tone.foreground".into());
    if let Some(h) = host {
        host_ctx::with_host_surface(Some(h), || {
            for form in in_scope_surface_author_forms() {
                set.insert(form);
            }
        });
    }
    set
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn component_namespace_is_gone() {
        assert!(!is_component_public_name("chip"));
        assert!(!is_component_public_name("chip-foreground"));
        assert!(!is_component_public_name("button-hover-bg"));
        assert!(!is_component_public_name("primary"));
        assert!(!is_theme_owned_author_root("button"));
        assert!(!is_theme_owned_author_root("card"));
        assert!(!is_theme_owned_author_root("chip"));
        assert!(!is_theme_owned_token_name("button"));
        assert!(!is_theme_owned_token_name("button.bg"));
        assert!(!is_theme_owned_token_name("card"));
        assert!(all_surface_packs().is_empty());
        assert!(current_surface_pack().is_none());
        host_ctx::with_host_surface(Some("chip"), || {
            assert_eq!(current_surface_pack().as_deref(), Some("chip"));
        });
    }

    #[test]
    fn app_vocab_includes_promoted_globals() {
        let app = color_paint_vocabulary(None);
        assert!(app.contains("primary"));
        assert!(app.contains("card"));
        assert!(app.contains("card-foreground"));
        assert!(app.contains("popover"));
        assert!(app.contains("popover-foreground"));
        assert!(!app.contains("chip"));
        assert!(!app.contains("chip.foreground"));
        let chip = color_paint_vocabulary(Some("chip"));
        assert!(chip.contains("chip"));
        assert!(chip.contains("chip.foreground"));
    }
}

#[cfg(test)]
mod emit_integration {
    use crate::host_ctx::with_host_surface;
    use crate::parse;

    #[test]
    fn app_style_card_is_a_global() {
        with_host_surface(None, || {
            let (_, diags) = parse("style({ bg(card) })");
            assert!(
                !diags
                    .iter()
                    .any(|d| matches!(d, crate::ir::StyleDiagnostic::ThemeOwnedToken { .. })),
                "card is a global color: {:?}",
                diags.iter().map(|d| d.message()).collect::<Vec<_>>()
            );
            let (_, diags) = parse("style({ bg(primary) })");
            assert!(
                !diags
                    .iter()
                    .any(|d| matches!(d, crate::ir::StyleDiagnostic::ThemeOwnedToken { .. })),
                "bg(primary) must stay legal: {:?}",
                diags.iter().map(|d| d.message()).collect::<Vec<_>>()
            );
            let (_, diags) = parse("style({ border(page) })");
            assert!(
                diags.iter().any(|d| {
                    matches!(d, crate::ir::StyleDiagnostic::BadToken { value, .. } if value == "page")
                        || (matches!(d, crate::ir::StyleDiagnostic::ThemeOwnedToken { name, .. } if name == "page")
                            && d.message().contains("theme-owned")
                            && d.message().contains("theme="))
                }),
                "{:?}",
                diags.iter().map(|d| d.message()).collect::<Vec<_>>()
            );
        });
    }

    #[test]
    fn skins_spend_globals() {
        with_host_surface(Some("chip"), || {
            let (style, diags) = parse(
                "style({ bg({ color: muted }).fg({ color: muted-foreground }).border({ color: border }) })",
            );
            assert!(
                diags.is_empty(),
                "{:?}",
                diags.iter().map(|d| d.message()).collect::<Vec<_>>()
            );
            let _ = style;
        });
    }

    #[test]
    fn chip_foreground_legal_because_compiler_installed_pack() {
        with_host_surface(Some("chip"), || {
            let (_, diags) = parse("style({ fg(chip.foreground) })");
            assert!(
                diags.is_empty(),
                "chip.foreground must be legal when the compiler installs the pack: {:?}",
                diags.iter().map(|d| d.message()).collect::<Vec<_>>()
            );
        });
        with_host_surface(None, || {
            let (_, diags) = parse("style({ fg(chip.foreground) })");
            assert!(
                diags.iter().any(|d| {
                    matches!(d, crate::ir::StyleDiagnostic::BadToken { value, .. } if value == "chip.foreground")
                }),
                "app Style must reject chip.foreground: {:?}",
                diags.iter().map(|d| d.message()).collect::<Vec<_>>()
            );
        });
    }
}
