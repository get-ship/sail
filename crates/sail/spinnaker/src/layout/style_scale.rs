// FILE_CAP_OVERRIDE: attr-gated CVA compound matcher + denorm tables + L1 size regression tests (RENDER-1).
//! Scale-driven sx class → taffy size/padding (product Style atoms + CVA compounds).
//!
//! Hosts stamp the full CVA class list; CSS gates with attribute/pseudo selectors.
//! Layout must do the same — never apply `::before` / hover / wrong size compounds
//! to the host box.

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::layout::mapping::{PaddingSide, SX_SCALE_RULES, SideSet, SxScaleRule};
use crate::scales;
use taffy::prelude::*;
use taffy::style::Style;

/// Apply every scale atom on `classes` that is valid for host `attrs`.
/// Tokens Style already resolved are not re-inferred from class spelling.
pub(crate) fn apply_scale_classes(
    style: &mut Style,
    classes: &[&str],
    attrs: &HashMap<String, String>,
    resolved: &crate::layout::box_ir::ResolvedBox,
) {
    for c in classes {
        if let Some((tail, rule)) = match_sx_scale_rule(c, attrs) {
            if scale_rule_already_resolved(rule, resolved) {
                continue;
            }
            apply_sx_scale(style, rule, &tail);
        }
    }
}

fn scale_rule_already_resolved(
    rule: SxScaleRule,
    resolved: &crate::layout::box_ir::ResolvedBox,
) -> bool {
    match rule {
        SxScaleRule::Padding(side) => match side {
            SideSet::All => resolved.pad.all.is_some(),
            SideSet::X => resolved.pad.all.is_some() || resolved.pad.x.is_some(),
            SideSet::Y => resolved.pad.all.is_some() || resolved.pad.y.is_some(),
            SideSet::T => {
                resolved.pad.all.is_some() || resolved.pad.y.is_some() || resolved.pad.t.is_some()
            }
            SideSet::B => {
                resolved.pad.all.is_some() || resolved.pad.y.is_some() || resolved.pad.b.is_some()
            }
            SideSet::L => {
                resolved.pad.all.is_some() || resolved.pad.x.is_some() || resolved.pad.l.is_some()
            }
            SideSet::R => {
                resolved.pad.all.is_some() || resolved.pad.x.is_some() || resolved.pad.r.is_some()
            }
        },
        SxScaleRule::Size => resolved.width.is_some() || resolved.height.is_some(),
        SxScaleRule::Width => resolved.width.is_some(),
        SxScaleRule::Height => resolved.height.is_some(),
        SxScaleRule::MinWidth => resolved.min_width.is_some(),
        SxScaleRule::MinHeight => resolved.min_height.is_some(),
        SxScaleRule::MaxWidth => resolved.max_width.is_some(),
        SxScaleRule::MaxHeight => resolved.max_height.is_some(),
    }
}

/// Product Style mints `s-{setter}-{arg}` and CVA compounds
/// `s-{path}-{setter}-{arg}` (BUCKETS §2.6). Path is kebabed attr+pseudo
/// tags; only bare atoms and compounds whose path matches host attrs
/// (and is free of pseudo markers) apply.
fn match_sx_scale_rule(c: &str, attrs: &HashMap<String, String>) -> Option<(String, SxScaleRule)> {
    let rest = c.strip_prefix("s-")?;
    if let Some(hit) = match_setter_and_tail(rest) {
        return Some(hit);
    }
    if path_has_pseudo(rest) {
        return None;
    }
    let segs = attr_path_segments();
    let mut rem = rest;
    let mut peeled = false;
    while !rem.is_empty() {
        let mut hit_seg = false;
        for (seg, attr, val) in segs.iter() {
            if let Some(after) = rem.strip_prefix(seg.as_str()) {
                if after.is_empty() || after.starts_with('-') {
                    if !host_attr_matches(attrs, attr, val) {
                        return None;
                    }
                    rem = after.strip_prefix('-').unwrap_or(after);
                    peeled = true;
                    hit_seg = true;
                    break;
                }
            }
        }
        if !hit_seg {
            break;
        }
    }
    if peeled {
        match_setter_and_tail(rem)
    } else {
        None
    }
}

fn match_setter_and_tail(body: &str) -> Option<(String, SxScaleRule)> {
    for (prefix, rule) in SX_SCALE_RULES {
        let setter = prefix.strip_prefix("s-")?;
        if let Some(after) = body.strip_prefix(setter) {
            let tail = after.strip_prefix('-').unwrap_or(after);
            if is_plausible_scale_tail(tail, *rule) {
                return Some((tail.to_string(), *rule));
            }
        }
    }
    None
}

/// Pseudo / interaction / media path tags — never size the host.
fn path_has_pseudo(path: &str) -> bool {
    const MARKERS: &[&str] = &[
        "before",
        "after",
        "hover",
        "focus",
        "active",
        "disabled",
        "checked",
        "focus-visible",
        "motion-reduce",
        "forced-colors",
        "open",
        "selected",
        "invalid",
        "empty",
        "indeterminate",
        "popover-open",
    ];
    MARKERS.iter().any(|m| path.contains(m))
}

/// Attr+value segments as they appear in product class paths (kebabed).
/// Longer values first so `2xs` wins over `xs`.
fn attr_path_segments() -> &'static [(String, &'static str, &'static str)] {
    static SEGS: OnceLock<Vec<(String, &'static str, &'static str)>> = OnceLock::new();
    SEGS.get_or_init(|| {
        let mut out = Vec::new();
        let gates: &[(&str, &[&str])] = &[
            (
                "type",
                &[
                    "checkbox", "password", "radio", "email", "number", "search", "tel", "url",
                    "text", "range", "file", "hidden", "submit", "button", "date", "time",
                ],
            ),
            (
                "size",
                &[
                    "3xs", "2xs", "2xl", "3xl", "iconLg", "iconSm", "icon", "xs", "sm", "md", "lg",
                    "xl",
                ],
            ),
            (
                "variant",
                &[
                    "default",
                    "outline",
                    "solid",
                    "ghost",
                    "soft",
                    "line",
                    "pill",
                    "enclosed",
                    "segmented",
                ],
            ),
            ("padding", &["none", "sm", "md", "lg"]),
            ("placement", &["bottom", "top", "rail"]),
            (
                "tone",
                &[
                    "destructive",
                    "primary",
                    "success",
                    "warning",
                    "default",
                    "accent",
                    "muted",
                    "info",
                ],
            ),
            ("accent-bar", &["static", "hover", "off"]),
        ];
        fn kebab_seg(s: &str) -> String {
            let mut out = String::with_capacity(s.len() + 4);
            let mut prev_dash = true;
            for (i, c) in s.chars().enumerate() {
                if c.is_ascii_uppercase() {
                    if !prev_dash && i > 0 {
                        out.push('-');
                    }
                    out.push(c.to_ascii_lowercase());
                    prev_dash = false;
                } else if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                    if c == '-' && prev_dash {
                        continue;
                    }
                    out.push(c.to_ascii_lowercase());
                    prev_dash = c == '-';
                }
            }
            out
        }
        for (attr, values) in gates {
            for val in *values {
                let seg = format!("{}-{}", kebab_seg(attr), kebab_seg(val));
                out.push((seg, *attr, *val));
            }
        }
        // Boolean attrs (presence = true): path is the kebab name.
        for b in [
            "interactive",
            "sticky",
            "bordered",
            "fill",
            "pill",
            "centered",
            "statusrail",
            "status-rail",
        ] {
            out.push((kebab_seg(b), b, "true"));
        }
        out.sort_by_key(|b| std::cmp::Reverse(b.0.len()));
        out
    })
}

fn host_attr_matches(attrs: &HashMap<String, String>, attr: &str, want: &str) -> bool {
    let key = attr.to_ascii_lowercase();
    // status-rail / statusrail
    let got = attrs
        .get(&key)
        .or_else(|| {
            if key == "statusrail" {
                attrs.get("status-rail")
            } else if key == "status-rail" {
                attrs.get("statusrail")
            } else if key == "accent-bar" {
                attrs.get("accentbar")
            } else {
                None
            }
        })
        .map(|s| s.as_str());

    match got {
        Some(v) if want == "true" => {
            // Bare boolean: present (empty or "true")
            v.is_empty() || v.eq_ignore_ascii_case("true")
        }
        Some(v) => v.eq_ignore_ascii_case(want),
        None => {
            // Product defaults when the attribute is omitted on the host.
            matches!(
                (attr, want),
                ("size", "md")
                    | ("variant", "default")
                    | ("padding", "md")
                    | ("tone", "default")
                    | ("placement", "top")
                    | ("type", "text")
                    | ("accent-bar", "off")
            )
        }
    }
}

fn is_plausible_scale_tail(tail: &str, rule: SxScaleRule) -> bool {
    if tail.is_empty() {
        return false;
    }
    match rule {
        SxScaleRule::Padding(_) => {
            tail == "0"
                || tail == "density"
                || tail == "px"
                || tail.parse::<u32>().is_ok()
                || space_fraction_token(tail).is_some()
        }
        SxScaleRule::Size
        | SxScaleRule::Width
        | SxScaleRule::Height
        | SxScaleRule::MinWidth
        | SxScaleRule::MinHeight
        | SxScaleRule::MaxWidth
        | SxScaleRule::MaxHeight => {
            matches!(
                tail,
                "0" | "auto" | "screen" | "fill" | "fit" | "min" | "max" | "railfill" | "half"
            ) || tail.parse::<u32>().is_ok()
                || denormalize_size_token(tail).is_some()
        }
    }
}

/// Half-steps: author `1_5` or the alnum-stripped fossil `15`.
fn space_fraction_token(tail: &str) -> Option<&'static str> {
    match tail {
        "1_5" | "15" => Some("1_5"),
        "2_5" | "25" => Some("2_5"),
        "3_5" | "35" => Some("3_5"),
        "4_5" | "45" => Some("4_5"),
        _ => None,
    }
}

fn apply_sx_scale(style: &mut Style, rule: SxScaleRule, value: &str) {
    match rule {
        SxScaleRule::Padding(side) => {
            let px = padding_token_to_px(value);
            apply_padding(style, map_padding_side(side), px);
        }
        SxScaleRule::Size => {
            let d = size_token_to_dimension(value);
            style.size.width = d;
            style.size.height = d;
        }
        SxScaleRule::Width => style.size.width = size_token_to_dimension(value),
        SxScaleRule::Height => style.size.height = size_token_to_dimension(value),
        SxScaleRule::MinWidth => style.min_size.width = size_token_to_dimension(value),
        SxScaleRule::MinHeight => style.min_size.height = size_token_to_dimension(value),
        SxScaleRule::MaxWidth => style.max_size.width = size_token_to_dimension(value),
        SxScaleRule::MaxHeight => style.max_size.height = size_token_to_dimension(value),
    }
}

fn map_padding_side(side: SideSet) -> PaddingSide {
    match side {
        SideSet::All => PaddingSide::All,
        SideSet::X => PaddingSide::X,
        SideSet::Y => PaddingSide::Y,
        SideSet::T => PaddingSide::T,
        SideSet::R => PaddingSide::R,
        SideSet::B => PaddingSide::B,
        SideSet::L => PaddingSide::L,
    }
}

fn apply_padding(style: &mut Style, side: PaddingSide, px: f32) {
    let v = LengthPercentage::length(px);
    match side {
        PaddingSide::All => {
            style.padding = taffy::Rect {
                top: v,
                right: v,
                bottom: v,
                left: v,
            };
        }
        PaddingSide::X => {
            style.padding.right = v;
            style.padding.left = v;
        }
        PaddingSide::Y => {
            style.padding.top = v;
            style.padding.bottom = v;
        }
        PaddingSide::T => style.padding.top = v,
        PaddingSide::R => style.padding.right = v,
        PaddingSide::B => style.padding.bottom = v,
        PaddingSide::L => style.padding.left = v,
    }
}

fn token_px_for_space(step: u32) -> f32 {
    match step {
        1 => 4.0,
        2 => 8.0,
        3 => 12.0,
        4 => 16.0,
        5 => 20.0,
        6 => 24.0,
        7 => 28.0,
        8 => 32.0,
        9 => 36.0,
        10 => 40.0,
        11 => 44.0,
        12 => 48.0,
        14 => 56.0,
        16 => 64.0,
        20 => 80.0,
        24 => 96.0,
        28 => 112.0,
        32 => 128.0,
        _ => step as f32 * 4.0,
    }
}

pub(crate) fn padding_token_to_px(value: &str) -> f32 {
    if value == "0" {
        return 0.0;
    }
    if value == "density" {
        return 8.0;
    }
    if value == "px" {
        return 1.0;
    }
    if let Some(token) = space_fraction_token(value) {
        if let Some(css) = scales::lower("space", token) {
            if let Some(d) = parse_css_length(&css) {
                if let Some(px) = length_px(d) {
                    return px;
                }
            }
        }
        // Fallback if registry missing the half-step.
        return match token {
            "1_5" => 6.0,
            "2_5" => 10.0,
            "3_5" => 14.0,
            "4_5" => 18.0,
            _ => 0.0,
        };
    }
    if let Ok(step) = value.parse::<u32>() {
        return token_px_for_space(step);
    }
    0.0
}

fn length_px(d: Dimension) -> Option<f32> {
    // Dimension is CompactLength internally — only length tags we set ourselves.
    use taffy::style::CompactLength;
    let raw = d.into_raw();
    if raw.tag() == CompactLength::LENGTH_TAG {
        Some(raw.value())
    } else {
        None
    }
}

pub(crate) fn size_token_to_dimension(value: &str) -> Dimension {
    let value = value.trim();
    let value = value
        .strip_prefix('\'')
        .and_then(|s| s.strip_suffix('\''))
        .or_else(|| value.strip_prefix('"').and_then(|s| s.strip_suffix('"')))
        .unwrap_or(value);
    match value {
        "0" => Dimension::length(0.0),
        "auto" => Dimension::auto(),
        "screen" => Dimension::percent(1.0),
        "fill" => Dimension::percent(1.0),
        "railfill" => Dimension::percent(1.0),
        "half" => Dimension::percent(0.5),
        "fit" => Dimension::auto(),
        "min" => Dimension::auto(),
        "max" => Dimension::auto(),
        other => {
            if let Some(d) = parse_css_length(other) {
                return d;
            }
            if let Ok(step) = other.parse::<u32>() {
                return Dimension::length(token_px_for_space(step));
            }
            let token = denormalize_size_token(other).unwrap_or_else(|| other.to_string());
            match scales::lower("size", &token) {
                Some(s) => parse_css_length(&s).unwrap_or(Dimension::auto()),
                None => Dimension::auto(),
            }
        }
    }
}

fn denormalize_size_token(raw: &str) -> Option<String> {
    if scales::contains("size", raw) {
        return Some(raw.to_string());
    }
    static STRIP_MAP: OnceLock<HashMap<String, String>> = OnceLock::new();
    let map = STRIP_MAP.get_or_init(|| {
        let mut m = HashMap::new();
        for v in scales::values("size") {
            let stripped: String = v.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
            if stripped != v {
                m.insert(stripped, v);
            }
        }
        m
    });
    map.get(raw).cloned()
}

fn parse_css_length(v: &str) -> Option<Dimension> {
    let v = v.trim();
    if v.is_empty() {
        return None;
    }
    if v == "0" {
        return Some(Dimension::length(0.0));
    }
    if let Some(inner) = v.strip_prefix("var(--").and_then(|s| s.strip_suffix(')')) {
        let name = inner.split(',').next()?.trim();
        let name = name.strip_prefix("--").unwrap_or(name);
        if let Some(resolved) = crate::tokens::token_value(name) {
            return parse_css_length(resolved);
        }
        return None;
    }
    if let Some(rest) = v.strip_suffix("px") {
        let n: f32 = rest.parse().ok()?;
        return Some(Dimension::length(n));
    }
    if let Some(rest) = v.strip_suffix("rem") {
        let n: f32 = rest.parse().ok()?;
        return Some(Dimension::length(n * 16.0));
    }
    if let Some(rest) = v.strip_suffix("ch") {
        let n: f32 = rest.parse().ok()?;
        return Some(Dimension::length(n * 8.0));
    }
    if v.ends_with('%') {
        if let Some(rest) = v.strip_suffix('%') {
            let n: f32 = rest.parse().ok()?;
            return Some(Dimension::percent(n / 100.0));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::html::Element;
    use crate::layout::style::resolve;

    #[test]
    fn full_is_not_a_percent_stamp() {
        assert_ne!(
            size_token_to_dimension("full"),
            Dimension::percent(1.0),
            "full is not a size-slot keyword"
        );
        assert_eq!(size_token_to_dimension("100%"), Dimension::percent(1.0));
    }

    #[test]
    fn sx_size_sets_width_and_height() {
        let e = Element {
            tag: "div".into(),
            class: "s-size-2".into(),
            ..Default::default()
        };
        let all = vec![Element::default(), e.clone()];
        let resolved = resolve(&e, &all, 800.0);
        assert_eq!(
            length_px(resolved.style.size.width),
            Some(8.0),
            "size(2) → space-2 = 8px width"
        );
        assert_eq!(
            length_px(resolved.style.size.height),
            Some(8.0),
            "size(2) → space-2 = 8px height"
        );
    }

    #[test]
    #[ignore]
    fn switch_skin_size_classes_resolve_nonzero() {
        let elem = Element {
            tag: "input".into(),
            class: "sail-switch s-w-control-h-sm s-h-icon-size-md s-shrink-0".into(),
            ..Default::default()
        };
        let resolved = resolve(&elem, &[Element::default()], 480.0);
        assert_eq!(resolved.style.size.width, Dimension::length(28.0));
        assert_eq!(resolved.style.size.height, Dimension::length(16.0));
    }

    #[test]
    #[ignore]
    fn switch_multi_size_stamps_honor_size_md_attr() {
        // Full CVA stamp order like the skin: sm then md then lg — last must
        // not win; only size=md compounds apply.
        let elem = Element {
            tag: "input".into(),
            class: "sail-switch s-w-control-h-sm s-h-icon-size-md \
                    s-size-sm-w-control-h-xs s-size-sm-h-icon-size-sm \
                    s-size-md-w-control-h-sm s-size-md-h-icon-size-md \
                    s-size-lg-w7 s-size-lg-h-icon-size-lg"
                .into(),
            attrs: {
                let mut m = HashMap::new();
                m.insert("size".into(), "md".into());
                m
            },
            ..Default::default()
        };
        let resolved = resolve(&elem, &[Element::default()], 480.0);
        assert_eq!(
            resolved.style.size.width,
            Dimension::length(28.0),
            "md width not lg"
        );
        assert_eq!(
            resolved.style.size.height,
            Dimension::length(16.0),
            "md height not lg"
        );
    }

    #[test]
    #[ignore]
    fn input_compound_minh_and_pad_resolve() {
        let elem = Element {
            tag: "input".into(),
            class:
                "sail-input s-type-text-min-h-control-h-md s-type-text-pad-y1_5 s-type-text-pad-x3 \
                 s-type-text-size-lg-min-h-control-h-lg"
                    .into(),
            attrs: {
                let mut m = HashMap::new();
                m.insert("type".into(), "text".into());
                m.insert("size".into(), "md".into());
                m
            },
            ..Default::default()
        };
        let resolved = resolve(&elem, &[Element::default()], 480.0);
        assert_eq!(
            resolved.style.min_size.height,
            Dimension::length(34.0),
            "md minH not lg"
        );
        assert_eq!(resolved.style.padding.top, LengthPercentage::length(6.0));
        assert_eq!(resolved.style.padding.left, LengthPercentage::length(12.0));
    }

    #[test]
    #[ignore]
    fn button_sizeicon_square_compounds_resolve() {
        let elem = Element {
            tag: "button".into(),
            class: "sail-button s-size-icon-w-control-h-md s-size-icon-h-control-h-md \
                    s-size-icon-sm-w-control-h-sm s-size-icon-sm-h-control-h-sm \
                    s-size-md-min-h-control-h-md"
                .into(),
            attrs: {
                let mut m = HashMap::new();
                m.insert("size".into(), "icon".into());
                m
            },
            ..Default::default()
        };
        let resolved = resolve(&elem, &[Element::default()], 480.0);
        assert_eq!(
            resolved.style.size.width,
            Dimension::length(34.0),
            "size=icon must apply control-h-md square, not fall through to content"
        );
        assert_eq!(resolved.style.size.height, Dimension::length(34.0));
    }

    fn button_sizemd_minh_compound_resolves() {
        let elem = Element {
            tag: "button".into(),
            class: "sail-button s-size-md-min-h-control-h-md s-size-md-pad-x3 \
                    s-size-lg-min-h-control-h-lg s-size-lg-pad-x4"
                .into(),
            attrs: {
                let mut m = HashMap::new();
                m.insert("size".into(), "md".into());
                m
            },
            ..Default::default()
        };
        let resolved = resolve(&elem, &[Element::default()], 480.0);
        assert_eq!(resolved.style.min_size.height, Dimension::length(34.0));
        assert_eq!(resolved.style.padding.left, LengthPercentage::length(12.0));
    }

    #[test]
    fn accent_bar_before_height_does_not_size_card_host() {
        let elem = Element {
            tag: "div".into(),
            class: "sail-card s-pad-y4 s-pad-x5 s-accent-bar-static-before-h-half \
                    s-accent-bar-hover-before-h-half s-bg-card"
                .into(),
            attrs: {
                let mut m = HashMap::new();
                m.insert("padding".into(), "md".into());
                m.insert("accent-bar".into(), "off".into());
                m
            },
            ..Default::default()
        };
        let resolved = resolve(&elem, &[Element::default()], 480.0);
        // Height must stay auto (not 50% from ::before h(half)).
        assert_eq!(
            resolved.style.size.height,
            Dimension::auto(),
            "pseudo before-hhalf must not set host height"
        );
        assert_eq!(resolved.style.padding.top, LengthPercentage::length(16.0));
    }

    #[test]
    fn paddingmd_compound_does_not_clobber_via_false_pad_prefix() {
        let elem = Element {
            tag: "div".into(),
            class: "s-pad-y4 s-padding-md-pad-y4".into(),
            attrs: {
                let mut m = HashMap::new();
                m.insert("padding".into(), "md".into());
                m
            },
            ..Default::default()
        };
        let resolved = resolve(&elem, &[Element::default()], 480.0);
        assert_eq!(resolved.style.padding.top, LengthPercentage::length(16.0));
    }

    #[test]
    fn placement_rail_compound_skipped_when_placement_top() {
        let elem = Element {
            tag: "div".into(),
            class: "s-min-h-14 s-pad-y2 s-placement-rail-min-h-0 s-placement-rail-pad-y5 \
                    s-placement-top-min-h-14 s-placement-top-pad-y2"
                .into(),
            attrs: {
                let mut m = HashMap::new();
                m.insert("placement".into(), "top".into());
                m
            },
            ..Default::default()
        };
        let resolved = resolve(&elem, &[Element::default()], 480.0);
        // Bare + top compounds: minH14 (space-14 = 56), not rail minH0.
        assert_eq!(resolved.style.min_size.height, Dimension::length(56.0));
        assert_eq!(resolved.style.padding.top, LengthPercentage::length(8.0));
    }
}
