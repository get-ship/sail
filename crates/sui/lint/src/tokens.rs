//! The shared design-token registry — the **single source** both the
//! `sui-css-token` lint and the `sx` vocabulary read from. Loads
//! `crates/sui/compiler/tokens.json` once.
//!
//! Token names are stored WITHOUT the `--` prefix (e.g. `"space-4"`,
//! `"text-3xl"`). `sx` must never hand-maintain its own copy of the scales:
//! its allowed spacing/text/radius values are *derived* from this registry, so
//! adding `--text-3xl` to the tokens makes `sx="text:3xl"` legal automatically
//! and the two can never drift (the drift that made `text:3xl` wrongly fail).

use std::collections::HashSet;
use std::sync::OnceLock;

const TOKENS_JSON: &str = include_str!("../../compiler/tokens.json");

/// Every theme-token name (no `--` prefix), flattened from `tokens.json`.
pub fn registry() -> &'static HashSet<&'static str> {
    static CACHE: OnceLock<HashSet<&'static str>> = OnceLock::new();
    CACHE.get_or_init(|| {
        let v: serde_json::Value =
            serde_json::from_str(TOKENS_JSON).expect("tokens.json must be valid JSON");
        let mut out = HashSet::new();
        collect_string_leaves(&v, &mut out);
        out
    })
}

/// The suffix values of a scale family, e.g. `scale_suffixes("space-")` →
/// `["1",…,"8"]`, `scale_suffixes("text-")` → `["2xl","3xl","4xl","base",…]`.
/// Sorted for deterministic diagnostics. Derived from [`registry`], so the
/// scale can never drift from the tokens.
fn scale_suffixes(prefix: &'static str) -> Vec<&'static str> {
    // Registry entries are `&'static str`, so `strip_prefix` yields a
    // `&'static str` suffix directly. Exclude the empty tail (the bare
    // `radius` token has no `-suffix`; `radius-full` → `full`).
    let mut out: Vec<&'static str> = registry()
        .iter()
        .filter_map(|&t| t.strip_prefix(prefix).filter(|s| !s.is_empty()))
        .collect();
    out.sort_unstable();
    out
}

/// Allowed `sx` spacing values: the `--space-N` scale plus the literal `0`
/// (no `--space-0` token exists; `p:0` must mean zero padding, not a var).
pub fn space_values() -> &'static [&'static str] {
    static C: OnceLock<Vec<&'static str>> = OnceLock::new();
    C.get_or_init(|| {
        let mut v = vec!["0"];
        v.extend(scale_suffixes("space-"));
        v
    })
}

/// Allowed `sx` text values: the full `--text-*` scale (xs … 4xl, base).
pub fn text_values() -> &'static [&'static str] {
    static C: OnceLock<Vec<&'static str>> = OnceLock::new();
    C.get_or_init(|| scale_suffixes("text-"))
}

/// Allowed `sx` radius values: the `--radius-*` scale plus the literal `none`
/// (→ `0`). No phantom `xs` — the scale is whatever the tokens declare.
pub fn radius_values() -> &'static [&'static str] {
    static C: OnceLock<Vec<&'static str>> = OnceLock::new();
    C.get_or_init(|| {
        let mut v = vec!["none"];
        v.extend(scale_suffixes("radius-"));
        v
    })
}

/// Recursively pull every JSON string leaf into `out`, skipping `$`-prefixed
/// metadata keys (`$comment`, `$schema`). Mirrors the walk the CSS-token lint
/// used before this module centralized it.
fn collect_string_leaves(v: &serde_json::Value, out: &mut HashSet<&'static str>) {
    match v {
        serde_json::Value::String(s) => {
            out.insert(Box::leak(s.clone().into_boxed_str()));
        }
        serde_json::Value::Array(items) => {
            for item in items {
                collect_string_leaves(item, out);
            }
        }
        serde_json::Value::Object(map) => {
            for (k, val) in map {
                if k.starts_with('$') {
                    continue;
                }
                collect_string_leaves(val, out);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_scale_includes_the_full_registry_range() {
        let t = text_values();
        // the exact values that sx used to be missing — proof of no-drift.
        for want in ["xs", "sm", "md", "base", "lg", "xl", "2xl", "3xl", "4xl"] {
            assert!(t.contains(&want), "text scale missing `{want}`: {t:?}");
        }
    }

    #[test]
    fn space_has_zero_plus_scale_no_phantom() {
        let s = space_values();
        assert!(s.contains(&"0"), "p:0 must be allowed (literal zero)");
        for n in ["1", "4", "8"] {
            assert!(s.contains(&n), "space missing `{n}`");
        }
    }

    #[test]
    fn radius_has_none_plus_scale_without_xs() {
        let r = radius_values();
        assert!(r.contains(&"none"));
        assert!(r.contains(&"full") && r.contains(&"lg"));
        assert!(!r.contains(&"xs"), "radius `xs` was a phantom — not in tokens.css");
    }
}
