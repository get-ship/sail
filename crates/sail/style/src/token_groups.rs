//! Compile-time token **group member** tables for Style paint args.
//!
//! A group is a semantic paint stem (`primary`, `info`, …) with members
//! (`default`, `foreground`, optional `hover`) that map to public token ids
//! (`primary`, `primary-foreground`, `primary-hover`).
//!
//! Style author forms:
//! - Static: `bg(primary)`, `fg(primary.foreground)` — resolve to public names.
//! - Bound prop: `bg(tone)`, `fg(tone.foreground)` — expand over the prop's
//!   group family with attribute-gated classes (`[tone='primary']…`).
//!
//! IR is a general member path; **Tone** is the first consumer (bound prop
//! `tone` → [`TONE_GROUPS`]). Do not call group members "channels".

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

/// Closed family of groups (e.g. Tone allowlist).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenGroupFamily {
    /// Prop name that expands over this family when used as a paint root.
    pub prop: &'static str,
    pub groups: &'static [TokenGroup],
}

const fn m(name: &'static str, public_name: &'static str) -> GroupMember {
    GroupMember { name, public_name }
}

/// Tone allowlist v1 — curated semantic paint groups (not entire `color.*`).
///
/// `'default'` is **not** a Tone member. Rest paint = component tokens when
/// the `tone` attribute is unset.
pub const TONE_GROUP_NAMES: &[&str] = &[
    "primary",
    "info",
    "success",
    "warning",
    "destructive",
    "accent",
    "muted",
];

/// Group → members → public name. Mirrors DTCG `color.<stem>.*` with
/// `brandAccent` published as `accent` (generator `public_name` policy).
pub const TONE_GROUPS: &[TokenGroup] = &[
    TokenGroup {
        name: "primary",
        members: &[
            m("default", "primary"),
            m("foreground", "primary-foreground"),
            m("hover", "primary-hover"),
        ],
    },
    TokenGroup {
        name: "info",
        members: &[m("default", "info"), m("foreground", "info-foreground")],
    },
    TokenGroup {
        name: "success",
        members: &[
            m("default", "success"),
            m("foreground", "success-foreground"),
        ],
    },
    TokenGroup {
        name: "warning",
        members: &[
            m("default", "warning"),
            m("foreground", "warning-foreground"),
        ],
    },
    TokenGroup {
        name: "destructive",
        members: &[
            m("default", "destructive"),
            m("foreground", "destructive-foreground"),
        ],
    },
    TokenGroup {
        name: "accent",
        members: &[m("default", "accent"), m("foreground", "accent-foreground")],
    },
    TokenGroup {
        name: "muted",
        members: &[m("default", "muted"), m("foreground", "muted-foreground")],
    },
];

/// Bound prop `tone` expands over [`TONE_GROUPS`].
pub const TONE_FAMILY: TokenGroupFamily = TokenGroupFamily {
    prop: "tone",
    groups: TONE_GROUPS,
};

/// Bound paint roots that expand over a group family (attr-gated emit).
pub fn bound_group_family(root: &str) -> Option<&'static TokenGroupFamily> {
    match root {
        "tone" => Some(&TONE_FAMILY),
        _ => None,
    }
}

/// Static group stem lookup (e.g. `primary` in `primary.foreground`).
pub fn static_token_group(name: &str) -> Option<&'static TokenGroup> {
    TONE_GROUPS.iter().find(|g| g.name == name)
}

/// Resolve a member path on a group. Empty `path` → `default` member.
pub fn resolve_group_member(
    group: &TokenGroup,
    path: &[String],
) -> Result<&'static str, String> {
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
    let group = static_token_group(root)
        .ok_or_else(|| format!("unknown token group `{root}`"))?;
    resolve_group_member(group, path).map(|s| s.to_string())
}

/// Whether every group in a bound family has the given member path.
pub fn bound_member_valid(family: &TokenGroupFamily, path: &[String]) -> Result<(), String> {
    for g in family.groups {
        resolve_group_member(g, path)?;
    }
    Ok(())
}

/// Public token id for one (group, member path) on a bound family.
pub fn bound_public_name(
    family: &TokenGroupFamily,
    group_name: &str,
    path: &[String],
) -> Option<&'static str> {
    let group = family.groups.iter().find(|g| g.name == group_name)?;
    resolve_group_member(group, path).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tone_allowlist_has_no_default_group() {
        assert!(!TONE_GROUP_NAMES.contains(&"default"));
        assert_eq!(TONE_GROUPS.len(), TONE_GROUP_NAMES.len());
        for (g, n) in TONE_GROUPS.iter().zip(TONE_GROUP_NAMES.iter()) {
            assert_eq!(&g.name, n);
            assert!(g.members.iter().any(|m| m.name == "default"));
            assert!(g.members.iter().any(|m| m.name == "foreground"));
        }
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
        assert!(resolve_static_member_path("info", &["hover".into()]).is_err());
    }

    #[test]
    fn bound_tone_default_and_foreground() {
        let f = bound_group_family("tone").unwrap();
        assert!(bound_member_valid(f, &[]).is_ok());
        assert!(bound_member_valid(f, &["foreground".into()]).is_ok());
        // hover is not on every tone group
        assert!(bound_member_valid(f, &["hover".into()]).is_err());
        assert_eq!(
            bound_public_name(f, "destructive", &["foreground".into()]),
            Some("destructive-foreground")
        );
    }
}
