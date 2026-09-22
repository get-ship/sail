//! Compile-time token **group member** tables for Style paint args.
//!
//! A color group is a semantic paint stem (`primary`, `info`, …) with members
//! (`default`, `foreground`, optional `hover`) that map to public token ids
//! (`primary`, `primary-foreground`, `primary-hover`). The product type name
//! for that set is **`Tone`** — one example of a group-typed prop, not a
//! special language feature for the prop name `tone`.
//!
//! Style author forms:
//! - Static: `bg(primary)`, `fg(primary.foreground)` — resolve to public names.
//! - Bound prop: `bg(tone)`, `fg(tone.foreground)`, `bg(shade)` with `shade: Tone`
//!   — emit bridge vars (`var(--tone)`, `var(--tone-foreground)`); the host
//!   reflects var *references* (`--tone: var(--primary)`) from the prop value.
//!
//! **SSOT:** group tables are **derived** from the token registry (same DTCG
//! paths that generate TS `Tone` / `TONE`), never dual hand tables.
//!
//! Binding is **type-keyed** (any prop whose type is the color-group union),
//! not name-keyed on `"tone"`.
//!
//! **Size** is a separate discrete ladder type (`sm`/`md`/`lg`, optional
//! `xs`/`xl`) for Style `match` conditionality — not continuous paint bind.

use std::sync::OnceLock;

use crate::tokens;

/// One member of a token group: author segment → public CSS token id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GroupMember {
    /// Author member name (`default`, `foreground`, `hover`).
    pub name: &'static str,
    /// Public token id without `--` (`primary`, `primary-foreground`).
    pub public_name: &'static str,
}

/// One semantic paint group (Tone member or static group stem).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenGroup {
    /// Group stem / Tone member (`primary`, `accent`, …).
    pub name: &'static str,
    /// Members present on this group. Always includes `default` + `foreground`.
    pub members: &'static [GroupMember],
}

/// Preferred color-group (Tone) allowlist order (membership from registry; this only sorts).
const COLOR_GROUP_ORDER: &[&str] = &[
    "primary", "info", "success", "warning", "danger", "accent", "muted",
];

/// Canonical Size ladder steps (base). Hosts may extend with `xs` / `xl`.
const SIZE_STEPS: &[&str] = &["xs", "sm", "md", "lg", "xl"];

/// DTCG path stem for a public Tone name. v2 stores roles under `color.{name}`.
fn dtcg_stem(public_group: &str) -> &str {
    public_group
}

fn leak_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

fn leak_members(members: Vec<GroupMember>) -> &'static [GroupMember] {
    Box::leak(members.into_boxed_slice())
}

fn leak_groups(groups: Vec<TokenGroup>) -> &'static [TokenGroup] {
    Box::leak(groups.into_boxed_slice())
}

fn leak_names(names: Vec<&'static str>) -> &'static [&'static str] {
    Box::leak(names.into_boxed_slice())
}

/// Build color-group (Tone) tables from the token registry path map.
fn derive_color_group_tables() -> (&'static [&'static str], &'static [TokenGroup]) {
    let mut groups: Vec<TokenGroup> = Vec::with_capacity(COLOR_GROUP_ORDER.len());
    let mut names: Vec<&'static str> = Vec::with_capacity(COLOR_GROUP_ORDER.len());

    for &name in COLOR_GROUP_ORDER {
        let stem = dtcg_stem(name);
        let mut members: Vec<GroupMember> = Vec::new();

        // default — v2 solid step, then legacy default leaf
        let default_public = tokens::path_to_css_name(&format!("color.{stem}.solid"))
            .or_else(|| tokens::path_to_css_name(&format!("color.{stem}.default")))
            .or_else(|| tokens::path_to_css_name(&format!("color.{stem}")))
            .unwrap_or(name);
        members.push(GroupMember {
            name: "default",
            public_name: if default_public == name {
                name
            } else {
                leak_str(default_public.to_string())
            },
        });

        // foreground — v2 text step, then legacy foreground leaf
        let fg_public = tokens::path_to_css_name(&format!("color.{stem}.text"))
            .or_else(|| tokens::path_to_css_name(&format!("color.{stem}.foreground")))
            .map(|s| {
                if s == name {
                    leak_str(format!("{name}-foreground"))
                } else {
                    leak_str(s.to_string())
                }
            })
            .unwrap_or_else(|| leak_str(format!("{name}-foreground")));
        members.push(GroupMember {
            name: "foreground",
            public_name: fg_public,
        });

        // hover only when present in registry
        if let Some(h) = tokens::path_to_css_name(&format!("color.{stem}.hover")) {
            members.push(GroupMember {
                name: "hover",
                public_name: leak_str(h.to_string()),
            });
        } else if name == "primary" && tokens::registry().contains("primary-hover") {
            members.push(GroupMember {
                name: "hover",
                public_name: "primary-hover",
            });
        }

        names.push(name);
        groups.push(TokenGroup {
            name,
            members: leak_members(members),
        });
    }

    (leak_names(names), leak_groups(groups))
}

fn color_group_tables() -> (&'static [&'static str], &'static [TokenGroup]) {
    static CACHE: OnceLock<(&'static [&'static str], &'static [TokenGroup])> = OnceLock::new();
    *CACHE.get_or_init(derive_color_group_tables)
}

/// Color-group allowlist (product type name: **Tone**) — curated semantic paint
/// groups (not entire `color.*`).
///
/// `'default'` is **not** a member. Rest paint = component tokens when the
/// group-typed prop attribute is unset.
pub fn color_group_names() -> &'static [&'static str] {
    color_group_tables().0
}

/// Alias — historic name; prefer [`color_group_names`].
#[inline]
pub fn tone_group_names() -> &'static [&'static str] {
    color_group_names()
}

/// Group → members → public name. Derived from the token registry.
pub fn color_groups() -> &'static [TokenGroup] {
    color_group_tables().1
}

/// Alias — historic name; prefer [`color_groups`].
#[inline]
pub fn tone_groups() -> &'static [TokenGroup] {
    color_groups()
}

/// Bridge CSS custom-property name for a bound group member path.
/// - `tone` / default → `tone`
/// - `tone.foreground` → `tone-foreground`
/// - `shade.hover` → `shade-hover`
pub fn bridge_var_name(prop: &str, path: &[String]) -> String {
    let prop_kebab = camel_to_kebab(prop);
    match path {
        [] => prop_kebab,
        [one] if one == "default" => prop_kebab,
        [one] => format!("{prop_kebab}-{one}"),
        segs => format!("{prop_kebab}-{}", segs.join("-")),
    }
}

/// CSS `var(--…)` read for a bound group member path.
pub fn bridge_var_ref(prop: &str, path: &[String]) -> String {
    format!("var(--{})", bridge_var_name(prop, path))
}

/// Public token id for one (group value, member path).
pub fn public_name_for_group_value(group_value: &str, path: &[String]) -> Option<&'static str> {
    let group = color_groups().iter().find(|g| g.name == group_value)?;
    resolve_group_member(group, path).ok()
}

/// Host reflection value: `var(--primary)` / `var(--primary-foreground)`.
pub fn reflect_var_reference(group_value: &str, path: &[String]) -> Option<String> {
    public_name_for_group_value(group_value, path).map(|p| format!("var(--{p})"))
}

/// All (member name or "" for default, public_name) pairs for a group value.
pub fn group_value_member_publics(group_value: &str) -> Option<Vec<(&'static str, &'static str)>> {
    let group = color_groups().iter().find(|g| g.name == group_value)?;
    Some(
        group
            .members
            .iter()
            .map(|m| {
                let path = if m.name == "default" { "" } else { m.name };
                (path, m.public_name)
            })
            .collect(),
    )
}

/// Whether `members` is a color-group (Tone) union — every member is a
/// color-group name, non-empty, no `'default'`. Type-keyed paint binding.
pub fn is_color_group_union(members: &[String]) -> bool {
    if members.is_empty() {
        return false;
    }
    let names = color_group_names();
    members
        .iter()
        .all(|m| m != "default" && names.contains(&m.as_str()))
}

/// Alias — historic name; prefer [`is_color_group_union`].
#[inline]
pub fn is_tone_union(members: &[String]) -> bool {
    is_color_group_union(members)
}

/// Whether `members` is a Size ladder union (only known size steps, ≥1 step,
/// must include at least one of `sm`/`md`/`lg`). Discrete Style match — not paint bind.
pub fn is_size_union(members: &[String]) -> bool {
    if members.is_empty() {
        return false;
    }
    let all_known = members.iter().all(|m| SIZE_STEPS.contains(&m.as_str()));
    if !all_known {
        return false;
    }
    // Reject empty or non-ladder sets.
    members
        .iter()
        .any(|m| matches!(m.as_str(), "xs" | "sm" | "md" | "lg" | "xl"))
}

/// Canonical Size step names for ambient emission (base ladder; hosts may narrow/extend).
pub fn size_step_names() -> &'static [&'static str] {
    // Product ambient `Size` is the common sm|md|lg core; xs/xl are extensions.
    &["sm", "md", "lg"]
}

/// Static group stem lookup (e.g. `primary` in `primary.foreground`).
pub fn static_token_group(name: &str) -> Option<&'static TokenGroup> {
    color_groups().iter().find(|g| g.name == name)
}

/// Resolve a member path on a group. Empty `path` → `default` member.
pub fn resolve_group_member(group: &TokenGroup, path: &[String]) -> Result<&'static str, String> {
    let member = match path {
        [] => "default",
        [one] => one.as_str(),
        _ => {
            return Err(format!(
                "token group `{}` only supports one member segment (got `{}`)",
                group.name,
                path.join(".")
            ));
        }
    };
    group
        .members
        .iter()
        .find(|m| m.name == member)
        .map(|m| m.public_name)
        .ok_or_else(|| {
            let known: Vec<&str> = group.members.iter().map(|m| m.name).collect();
            format!(
                "token group `{}` has no member `{}` (known: {})",
                group.name,
                member,
                known.join(", ")
            )
        })
}

/// Resolve static `root.path…` to a public token id, if `root` is a known group.
pub fn resolve_static_member_path(root: &str, path: &[String]) -> Result<String, String> {
    let group = static_token_group(root).ok_or_else(|| format!("unknown token group `{root}`"))?;
    resolve_group_member(group, path).map(|s| s.to_string())
}

/// Whether every color group has the given member path (bound member validity).
pub fn bound_member_valid(path: &[String]) -> Result<(), String> {
    for g in color_groups() {
        resolve_group_member(g, path)?;
    }
    Ok(())
}

/// True if `root` (+ optional member path) looks like a bound group-typed paint
/// root — not a static group stem, not a surface.
///
/// Type-keyed: only roots installed via
/// [`crate::host_ctx::with_bound_group_props`] (plus product ambient `tone`
/// as the common color-group prop name) are recognized. Scale tokens
/// (`sm`, `2`, `solid`) stay ordinary Tokens.
pub fn looks_like_bound_group_root(root: &str, path: &[String]) -> bool {
    let _ = path;
    if static_token_group(root).is_some() {
        return false;
    }
    if crate::component_surfaces::current_surface_pack().as_deref() == Some(root) {
        return false;
    }
    // Type-keyed (or product ambient `tone`). Member validity is checked at
    // parse (diagnostic) and binding (refusal for non-universal members).
    crate::host_ctx::is_bound_group_prop(root)
}

fn camel_to_kebab(name: &str) -> String {
    let mut out = String::with_capacity(name.len() + 4);
    for (i, c) in name.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i != 0 {
                out.push('-');
            }
            out.extend(c.to_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_group_allowlist_has_no_default_group() {
        let names = color_group_names();
        let groups = color_groups();
        assert!(!names.contains(&"default"));
        assert_eq!(groups.len(), names.len());
        for (g, n) in groups.iter().zip(names.iter()) {
            assert_eq!(&g.name, n);
            assert!(g.members.iter().any(|m| m.name == "default"));
            assert!(g.members.iter().any(|m| m.name == "foreground"));
        }
        // Historic aliases stay wired.
        assert_eq!(tone_group_names(), color_group_names());
        assert_eq!(tone_groups().len(), color_groups().len());
    }

    #[test]
    fn static_primary_foreground() {
        assert_eq!(
            resolve_static_member_path("primary", &["foreground".into()]).unwrap(),
            "primary-foreground"
        );
        assert_eq!(
            resolve_static_member_path("primary", &[]).unwrap(),
            "primary"
        );
        assert_eq!(
            resolve_static_member_path("accent", &["foreground".into()]).unwrap(),
            "accent-foreground"
        );
    }

    #[test]
    fn hover_only_where_present() {
        assert!(resolve_static_member_path("primary", &["hover".into()]).is_ok());
        assert!(resolve_static_member_path("muted", &["hover".into()]).is_err());
    }

    #[test]
    fn bound_member_default_and_foreground() {
        assert!(bound_member_valid(&[]).is_ok());
        assert!(bound_member_valid(&["foreground".into()]).is_ok());
        assert!(bound_member_valid(&["hover".into()]).is_err());
        assert_eq!(
            public_name_for_group_value("danger", &["foreground".into()]),
            Some("danger-foreground")
        );
    }

    #[test]
    fn bridge_var_names() {
        assert_eq!(bridge_var_name("tone", &[]), "tone");
        assert_eq!(
            bridge_var_name("tone", &["foreground".into()]),
            "tone-foreground"
        );
        assert_eq!(bridge_var_name("shade", &["hover".into()]), "shade-hover");
    }

    #[test]
    fn color_group_union_detection() {
        assert!(is_color_group_union(&[
            "primary".into(),
            "info".into(),
            "success".into(),
            "warning".into(),
            "danger".into(),
            "accent".into(),
            "muted".into(),
        ]));
        assert!(!is_color_group_union(&["default".into(), "primary".into()]));
        assert!(!is_color_group_union(&["sm".into(), "md".into()]));
        assert!(is_tone_union(&["primary".into(), "muted".into()]));
    }

    #[test]
    fn size_union_detection() {
        assert!(is_size_union(&["sm".into(), "md".into(), "lg".into()]));
        assert!(is_size_union(&[
            "xs".into(),
            "sm".into(),
            "md".into(),
            "lg".into(),
            "xl".into(),
        ]));
        assert!(!is_size_union(&[
            "sm".into(),
            "md".into(),
            "lg".into(),
            "full".into()
        ]));
        assert!(!is_size_union(&["primary".into(), "info".into()]));
        assert!(!is_size_union(&["full".into()]));
        assert_eq!(size_step_names(), &["sm", "md", "lg"]);
    }
}
