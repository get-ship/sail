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
//! ([`crate::host_ctx`]). App/L2 Style cannot use surface identities.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::OnceLock;

use super::host_ctx;
use super::tokens;

/// Packs where storage leaf `track` is the default fill (like `.bg`).
const TRACK_AS_PRIMARY: &[&str] = &["meter", "progress", "switch"];

/// Host unit name → surface pack name when they differ (stdlib renames).
fn host_pack_alias(host: &str) -> &str {
    match host {
        "a" | "anchor" => "button",
        "input" | "textarea" | "select" | "date" | "file" | "color" => "field",
        "pagination" => "page",
        other => other,
    }
}

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
    /// Every component public name (rest + state) — excluded from app color paint.
    all_public_names: HashSet<&'static str>,
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
    let mut all_public_names: HashSet<&'static str> = HashSet::new();
    let mut rest_public_names: HashSet<&'static str> = HashSet::new();

    for (path, css) in tokens::component_color_paths() {
        let path_s = leak_str(path);
        let css_s = leak_str(css);
        path_to_public.insert(path_s, css_s);
        all_public_names.insert(css_s);

        let Some(rest) = path_s.strip_prefix("component.") else {
            continue;
        };
        let Some((pack, leaf)) = rest.split_once('.') else {
            continue;
        };
        if leaf.contains('.') {
            continue;
        }
        // v1 rest-only Style surface slots — no hover/focus as author surface.
        if is_state_leaf(leaf) {
            continue;
        }
        let author = author_member_for_leaf(pack, leaf);
        let author_s = leak_str(author);
        let leaf_s = leak_str(leaf);
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
        all_public_names,
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

/// Whether `css_name` is any component surface public name (app color paint must exclude).
pub fn is_component_public_name(css_name: &str) -> bool {
    index().all_public_names.contains(css_name)
}

/// Whether `css_name` is a rest surface emit label (kebab form forbidden in Style).
pub fn is_rest_surface_public_name(css_name: &str) -> bool {
    index().rest_public_names.contains(css_name)
}

/// Surface pack in scope for the current host unit, if any.
pub fn current_surface_pack() -> Option<&'static str> {
    let host = host_ctx::current_host_surface()?;
    let pack = host_pack_alias(&host);
    index().by_pack.keys().find(|k| **k == pack).copied()
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
    let slots = surface_slots(pack).ok_or_else(|| format!("unknown surface pack `{pack}`"))?;
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
    let Some(slots) = surface_slots(pack) else {
        return Vec::new();
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

/// Host unit name → surface pack (for compile_element).
pub fn surface_pack_for_host(host: &str) -> Option<&'static str> {
    let pack = host_pack_alias(host);
    index().by_pack.keys().find(|k| **k == pack).copied()
}

/// Color-paint vocabulary for a context: globals (+ keywords) + optional host surface forms.
/// Completions and validate sets share this SSOT.
pub fn color_paint_vocabulary(host: Option<&str>) -> HashSet<String> {
    let mut set: HashSet<String> = tokens::color_values()
        .iter()
        .filter(|s| !is_component_public_name(s))
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
    use crate::host_ctx::with_host_surface;

    #[test]
    fn chip_surface_tone_parallel() {
        with_host_surface(Some("chip"), || {
            assert_eq!(resolve_surface_author("chip", &[]).unwrap(), "chip");
            assert_eq!(
                resolve_surface_author("chip", &["foreground".into()]).unwrap(),
                "chip-foreground"
            );
            assert_eq!(
                resolve_surface_author("chip", &["border".into()]).unwrap(),
                "chip-border"
            );
            assert!(resolve_surface_author("chip", &["hover-bg".into()]).is_err());
        });
    }

    #[test]
    fn app_context_rejects_surfaces() {
        with_host_surface(None, || {
            assert!(resolve_surface_author("chip", &[]).is_err());
        });
    }

    #[test]
    fn foreign_host_surface_rejected() {
        with_host_surface(Some("chip"), || {
            assert!(resolve_surface_author("button", &[]).is_err());
        });
    }

    #[test]
    fn field_alias_for_input() {
        with_host_surface(Some("input"), || {
            assert_eq!(current_surface_pack(), Some("field"));
            assert_eq!(resolve_surface_author("field", &[]).unwrap(), "field");
            assert_eq!(
                resolve_surface_author("field", &["foreground".into()]).unwrap(),
                "field-foreground"
            );
        });
    }

    #[test]
    fn field_alias_for_date_file_color() {
        for host in ["date", "file", "color"] {
            with_host_surface(Some(host), || {
                assert_eq!(current_surface_pack(), Some("field"), "host={host}");
                assert_eq!(
                    resolve_surface_author("field", &["foreground".into()]).unwrap(),
                    "field-foreground",
                    "host={host}"
                );
                assert_eq!(
                    resolve_surface_author("field", &["border".into()]).unwrap(),
                    "field-border",
                    "host={host}"
                );
            });
        }
    }

    #[test]
    fn unaliased_host_rejects_field_surface() {
        with_host_surface(Some("chip"), || {
            assert_eq!(current_surface_pack(), Some("chip"));
            let err = resolve_surface_author("field", &["foreground".into()]).unwrap_err();
            assert!(
                err.contains("not in scope") || err.contains("field"),
                "{err}"
            );
        });
    }

    #[test]
    fn component_public_names_indexed() {
        assert!(is_component_public_name("chip"));
        assert!(is_component_public_name("chip-foreground"));
        assert!(is_component_public_name("button-hover-bg"));
        assert!(!is_component_public_name("primary"));
        assert!(is_rest_surface_public_name("chip-foreground"));
        assert!(!is_rest_surface_public_name("button-hover-bg"));
    }

    #[test]
    fn app_vocab_excludes_chip_surface() {
        let app = color_paint_vocabulary(None);
        assert!(app.contains("primary"));
        assert!(!app.contains("chip"));
        assert!(!app.contains("chip.foreground"));
        assert!(!app.contains("chip-foreground"));
        let host = color_paint_vocabulary(Some("chip"));
        assert!(host.contains("chip"));
        assert!(host.contains("chip.foreground"));
        assert!(host.contains("chip.border"));
        assert!(!host.contains("chip-foreground"));
    }
}

#[cfg(test)]
mod emit_integration {
    use super::*;
    use crate::host_ctx::with_host_surface;
    use crate::{lower_style, parse};
    use std::collections::HashMap;

    #[test]
    fn host_surface_emits_css_var() {
        with_host_surface(Some("chip"), || {
            assert_eq!(current_surface_pack(), Some("chip"));
            assert_eq!(resolve_surface_author("chip", &[]).unwrap(), "chip");
            let (style, diags) =
                parse("style({ paint: bg(chip).fg(chip.foreground).border(chip.border) })");
            assert!(
                diags.is_empty(),
                "{:?}",
                diags.iter().map(|d| d.message()).collect::<Vec<_>>()
            );
            let (_, css) = lower_style(&style, &HashMap::new());
            assert!(css.contains("var(--chip)"), "css={css}");
            assert!(css.contains("var(--chip-foreground)"), "css={css}");
            assert!(css.contains("var(--chip-border)"), "css={css}");
        });
    }
}
