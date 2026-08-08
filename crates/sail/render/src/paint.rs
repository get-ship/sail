//! Resolve paint + text channel from lowered CSS + theme.
//!
//! Paint: bg/fg/border/radius/shadow/opacity.
//! Text: font-size/weight/family, letter-spacing, line-height, text-transform.
//!
//! Author page CSS (preprocess `<style>`) and L1 element skins share one
//! stylesheet. Attribute-qualified rules (`[tone='primary'].s-…`) match the
//! lowered host attrs so compound host class lists do not all paint at once.

use std::collections::HashMap;

use sail_layout::html::Element;
use takumi::prelude::*;

use crate::identity::RegionStyle;
use crate::theme::ThemePalette;

/// One paint rule: simple compound selector (no combinators) + decls.
#[derive(Debug, Clone)]
struct PaintRule {
    tag: Option<String>,
    /// `(name, value)` — value `None` means bare attribute presence.
    attrs: Vec<(String, Option<String>)>,
    class: Option<String>,
    /// `:empty` — only when the element has no children and no text.
    empty: bool,
    decls: Vec<(String, String)>,
}

/// Parsed CSS used for static paint (author atoms + L1 skins).
#[derive(Debug, Default, Clone)]
pub struct StyleSheet {
    /// Rules that key on a class (fast path for host-stamped sx atoms).
    by_class: HashMap<String, Vec<PaintRule>>,
    /// Tag / attribute-only rules (no class).
    no_class: Vec<PaintRule>,
    /// Flat class → decls for the legacy class-only lookup (tests / sx without attrs).
    class_flat: HashMap<String, Vec<(String, String)>>,
}

impl StyleSheet {
    pub fn parse(css: &str) -> Self {
        let mut sheet = StyleSheet::default();
        let mut i = 0;
        let bytes = css.as_bytes();
        while i < bytes.len() {
            // Skip @layer wrappers' open; still parse bodies.
            if bytes[i..].starts_with(b"@media") || bytes[i..].starts_with(b"@supports") {
                if let Some(rel) = find_byte(bytes, i, b'{') {
                    i = skip_balanced(bytes, rel);
                    continue;
                }
            }
            if bytes[i..].starts_with(b"@layer") {
                // @layer name { … } or @layer a, b;
                if let Some(rel) = find_byte(bytes, i, b'{') {
                    // Enter layer body without consuming — advance past `{`.
                    i = rel + 1;
                    continue;
                }
                // @layer foo; — skip to `;`
                if let Some(rel) = find_byte(bytes, i, b';') {
                    i = rel + 1;
                    continue;
                }
            }
            // Skip comments.
            if bytes[i..].starts_with(b"/*") {
                if let Some(rel) = find_from(bytes, i + 2, b"*/") {
                    i = rel + 2;
                    continue;
                }
                break;
            }
            // Skip whitespace / closing braces from layers.
            if bytes[i].is_ascii_whitespace() || bytes[i] == b'}' {
                i += 1;
                continue;
            }

            // Selector run until `{`.
            let sel_start = i;
            let Some(brace) = find_byte(bytes, i, b'{') else {
                break;
            };
            let selector = css[sel_start..brace].trim();
            let body_end = skip_balanced(bytes, brace) - 1;
            let body = if body_end > brace + 1 {
                &css[brace + 1..body_end]
            } else {
                ""
            };
            let decls = parse_decls(body);
            i = body_end + 1;
            if decls.is_empty() || selector.is_empty() {
                continue;
            }
            for part in selector.split(',') {
                let part = part.trim();
                if part.is_empty() {
                    continue;
                }
                if let Some(rule) = parse_simple_selector(part, decls.clone()) {
                    sheet.push_rule(rule);
                }
            }
        }
        sheet
    }

    fn push_rule(&mut self, rule: PaintRule) {
        if let Some(class) = rule.class.clone() {
            // Flat map: only unconditional class rules (no attrs/tag/empty).
            if rule.attrs.is_empty() && rule.tag.is_none() && !rule.empty {
                self.class_flat
                    .entry(class.clone())
                    .or_default()
                    .extend(rule.decls.iter().cloned());
            }
            self.by_class.entry(class).or_default().push(rule);
        } else {
            self.no_class.push(rule);
        }
    }

    /// Merge declarations for an element's class list (document order of classes).
    /// Ignores attribute conditions — used by unit tests of pure class atoms.
    pub fn decls_for_classes(&self, class_attr: &str) -> Vec<(String, String)> {
        let mut out = Vec::new();
        for c in class_attr.split_whitespace() {
            if let Some(decls) = self.class_flat.get(c) {
                out.extend(decls.iter().cloned());
            }
        }
        out
    }

    /// Resolve paint decls for a lowered element (classes + tag + attrs + empty).
    ///
    /// **Cascade invariant:** matching rules append in **host class-list order**
    /// (then declaration order within a rule). `Paint::from_decls` last-write-wins
    /// per property — not CSS specificity. Host emit must stamp CVA compounds
    /// (e.g. `[tone][variant]` fills) **after** base classes like
    /// `s-bgtransparent`, or primary fills are crushed by earlier matches.
    pub fn decls_for_element(&self, elem: &Element) -> Vec<(String, String)> {
        let classes: Vec<&str> = elem.class.split_whitespace().collect();
        let is_empty = elem.children.is_empty() && elem.text.trim().is_empty();
        let mut out = Vec::new();

        for rule in &self.no_class {
            if rule_matches(rule, elem, &classes, is_empty) {
                out.extend(rule.decls.iter().cloned());
            }
        }
        for c in &classes {
            if let Some(rules) = self.by_class.get(*c) {
                for rule in rules {
                    if rule_matches(rule, elem, &classes, is_empty) {
                        out.extend(rule.decls.iter().cloned());
                    }
                }
            }
        }
        out
    }
}

fn rule_matches(rule: &PaintRule, elem: &Element, classes: &[&str], is_empty: bool) -> bool {
    if rule.empty && !is_empty {
        return false;
    }
    if let Some(tag) = &rule.tag {
        if !tag.eq_ignore_ascii_case(&elem.tag) {
            return false;
        }
    }
    if let Some(class) = &rule.class {
        if !classes.iter().any(|c| *c == class) {
            return false;
        }
    }
    for (name, val) in &rule.attrs {
        match val {
            None => {
                // Bare [attr] — present (including empty string for boolean).
                if !elem.attrs.contains_key(name) {
                    return false;
                }
            }
            Some(want) => {
                let Some(got) = elem.attrs.get(name) else {
                    return false;
                };
                if got != want {
                    return false;
                }
            }
        }
    }
    true
}

/// Parse a single simple selector (no combinators). Returns None if unsupported
/// (hover/focus/before, descendant, etc.) — static rest-state paint only.
fn parse_simple_selector(sel: &str, decls: Vec<(String, String)>) -> Option<PaintRule> {
    let sel = sel.trim();
    if sel.is_empty() {
        return None;
    }
    // Reject combinators / universal.
    if sel.contains([' ', '>', '+', '~']) {
        return None;
    }
    let bytes = sel.as_bytes();
    let mut i = 0;
    let mut tag: Option<String> = None;
    let mut attrs: Vec<(String, Option<String>)> = Vec::new();
    let mut class: Option<String> = None;
    let mut empty = false;

    // Optional tag
    if i < bytes.len() && (bytes[i].is_ascii_alphabetic() || bytes[i] == b'*') {
        if bytes[i] == b'*' {
            i += 1;
        } else {
            let start = i;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'-') {
                i += 1;
            }
            tag = Some(sel[start..i].to_ascii_lowercase());
        }
    }

    while i < bytes.len() {
        match bytes[i] {
            b'.' => {
                i += 1;
                let start = i;
                while i < bytes.len()
                    && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'-' || bytes[i] == b'_')
                {
                    i += 1;
                }
                if start == i {
                    return None;
                }
                // Only one class in simple host atoms; if multiple, last wins key.
                class = Some(sel[start..i].to_string());
            }
            b'[' => {
                i += 1;
                let name_start = i;
                while i < bytes.len()
                    && bytes[i] != b'='
                    && bytes[i] != b']'
                    && !bytes[i].is_ascii_whitespace()
                {
                    i += 1;
                }
                if name_start == i {
                    return None;
                }
                let name = sel[name_start..i].to_ascii_lowercase();
                while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                    i += 1;
                }
                if i < bytes.len() && bytes[i] == b'=' {
                    i += 1;
                    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                        i += 1;
                    }
                    let value = if i < bytes.len() && (bytes[i] == b'\'' || bytes[i] == b'"') {
                        let q = bytes[i];
                        i += 1;
                        let v_start = i;
                        while i < bytes.len() && bytes[i] != q {
                            i += 1;
                        }
                        let v = sel[v_start..i].to_string();
                        if i < bytes.len() {
                            i += 1;
                        }
                        v
                    } else {
                        let v_start = i;
                        while i < bytes.len() && bytes[i] != b']' && !bytes[i].is_ascii_whitespace()
                        {
                            i += 1;
                        }
                        sel[v_start..i].to_string()
                    };
                    while i < bytes.len() && bytes[i] != b']' {
                        i += 1;
                    }
                    if i < bytes.len() {
                        i += 1;
                    }
                    attrs.push((name, Some(value)));
                } else {
                    while i < bytes.len() && bytes[i] != b']' {
                        i += 1;
                    }
                    if i < bytes.len() {
                        i += 1;
                    }
                    attrs.push((name, None));
                }
            }
            b':' => {
                // ::pseudo-element or :pseudo-class
                let double = i + 1 < bytes.len() && bytes[i + 1] == b':';
                i += if double { 2 } else { 1 };
                let start = i;
                while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'-') {
                    i += 1;
                }
                let pseudo = sel[start..i].to_ascii_lowercase();
                // Functional pseudo: :is(...), :not(...) — skip whole rule.
                if i < bytes.len() && bytes[i] == b'(' {
                    return None;
                }
                match pseudo.as_str() {
                    "empty" => empty = true,
                    // Interactive / structural states not modeled in static paint.
                    "hover" | "focus" | "focus-visible" | "active" | "disabled" | "checked"
                    | "indeterminate" | "invalid" | "open" | "popover-open" | "before"
                    | "after" | "where" | "is" | "not" | "has" => return None,
                    _ => return None,
                }
            }
            _ => return None,
        }
    }

    // Need something to match on.
    if tag.is_none() && class.is_none() && attrs.is_empty() {
        return None;
    }

    Some(PaintRule {
        tag,
        attrs,
        class,
        empty,
        decls,
    })
}

fn find_from(bytes: &[u8], from: usize, needle: &[u8]) -> Option<usize> {
    bytes[from..]
        .windows(needle.len())
        .position(|w| w == needle)
        .map(|p| from + p)
}

fn parse_decls(body: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for part in body.split(';') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if let Some((k, v)) = part.split_once(':') {
            out.push((k.trim().to_ascii_lowercase(), v.trim().to_string()));
        }
    }
    out
}

fn find_byte(bytes: &[u8], from: usize, b: u8) -> Option<usize> {
    bytes[from..].iter().position(|&x| x == b).map(|p| from + p)
}

fn skip_balanced(bytes: &[u8], open_brace_at: usize) -> usize {
    let mut depth = 0i32;
    let mut i = open_brace_at;
    while i < bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return i + 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    bytes.len()
}

/// Paint + text-channel properties resolved for one node.
#[derive(Debug, Default, Clone)]
pub struct Paint {
    pub background_color: Option<Color>,
    pub color: Option<Color>,
    pub border_color: Option<Color>,
    pub border_width: Option<f32>,
    pub border_radius: Option<f32>,
    pub opacity: Option<f32>,
    pub box_shadow: Option<BoxShadow>,
    /// When true, bg mapping was requested but could not be resolved (test hooks).
    pub missing_bg: bool,
    // --- text channel (annex E) — resolved from sx atoms via the sheet ---
    pub font_size_px: Option<f32>,
    pub font_weight: Option<f32>,
    pub letter_spacing: Option<Length>,
    pub line_height: Option<LineHeight>,
    pub text_transform: Option<TextTransform>,
    pub font_family: Option<FontFamily>,
}

impl Paint {
    pub fn from_decls(decls: &[(String, String)], palette: &ThemePalette) -> Self {
        let mut paint = Paint::default();
        for (prop, raw) in decls {
            let resolved = palette.resolve_css_value(raw);
            match prop.as_str() {
                "background-color" | "background" => {
                    if raw.contains("var(--") && palette_misses_var(raw, palette) {
                        paint.missing_bg = true;
                    }
                    if let Some(c) = parse_color(&resolved) {
                        // Ignore fully transparent keyword backgrounds for layering
                        // unless explicitly transparent is the only paint.
                        paint.background_color = Some(c);
                    }
                }
                "color" => {
                    if let Some(c) = parse_color(&resolved) {
                        paint.color = Some(c);
                    }
                }
                "border-color"
                | "border-top-color"
                | "border-right-color"
                | "border-bottom-color"
                | "border-left-color" => {
                    if let Some(c) = parse_color(&resolved) {
                        paint.border_color = Some(c);
                    }
                }
                "border-width"
                | "border-top-width"
                | "border-right-width"
                | "border-bottom-width"
                | "border-left-width" => {
                    if let Some(px) = parse_px(&resolved) {
                        paint.border_width = Some(px);
                    }
                }
                "border-radius"
                | "border-top-left-radius"
                | "border-top-right-radius"
                | "border-bottom-right-radius"
                | "border-bottom-left-radius" => {
                    if let Some(px) = parse_px(&resolved) {
                        paint.border_radius = Some(px);
                    }
                }
                "opacity" => {
                    if let Ok(v) = resolved.trim().parse::<f32>() {
                        paint.opacity = Some(v);
                    } else if let Some(pct) = resolved.trim().strip_suffix('%') {
                        if let Ok(v) = pct.parse::<f32>() {
                            paint.opacity = Some(v / 100.0);
                        }
                    }
                }
                "box-shadow" => {
                    if let Some(shadow) = parse_box_shadow(&resolved) {
                        paint.box_shadow = Some(shadow);
                    }
                }
                "font-size" => {
                    if let Some(px) = parse_font_size_px(&resolved) {
                        paint.font_size_px = Some(px);
                    }
                }
                "font-weight" => {
                    if let Some(w) = parse_font_weight(&resolved) {
                        paint.font_weight = Some(w);
                    }
                }
                "letter-spacing" => {
                    if let Ok(len) = Length::from_css_str(resolved.trim()) {
                        paint.letter_spacing = Some(len);
                    } else if resolved.trim() == "0" {
                        paint.letter_spacing = Some(Length::Px(0.0));
                    }
                }
                "line-height" => {
                    if let Ok(lh) = LineHeight::from_css_str(resolved.trim()) {
                        paint.line_height = Some(lh);
                    }
                }
                "text-transform" => {
                    if let Ok(tt) = TextTransform::from_css_str(resolved.trim()) {
                        paint.text_transform = Some(tt);
                    }
                }
                "font-family" => {
                    if let Ok(ff) = FontFamily::from_css_str(resolved.trim()) {
                        paint.font_family = Some(ff);
                    }
                }
                _ => {}
            }
        }
        paint
    }

    /// Apply text-channel decls onto a takumi Style (used by `text_node`).
    pub fn apply_text(&self, mut style: Style) -> Style {
        if let Some(px) = self.font_size_px {
            style = style.with(StyleDeclaration::font_size(FontSize::Length(Length::Px(px))));
        }
        if let Some(w) = self.font_weight {
            style = style.with(StyleDeclaration::font_weight(FontWeight::from(w)));
        }
        if let Some(ls) = self.letter_spacing {
            style = style.with(StyleDeclaration::letter_spacing(ls));
        }
        if let Some(lh) = self.line_height {
            style = style.with(StyleDeclaration::line_height(lh));
        }
        if let Some(tt) = self.text_transform {
            style = style.with(StyleDeclaration::text_transform(tt));
        }
        if let Some(ref ff) = self.font_family {
            style = style.with(StyleDeclaration::font_family(ff.clone()));
        }
        style
    }

    /// Serialize resolved paint for region sidecar / PARITY-1 paint compare.
    pub fn to_region_style(&self) -> Option<RegionStyle> {
        let style = RegionStyle {
            background_color: self.background_color.map(color_to_css_rgba),
            color: self.color.map(color_to_css_rgba),
            border_color: self.border_color.map(color_to_css_rgba),
            border_width: self.border_width,
            border_radius: self.border_radius,
            opacity: self.opacity,
        };
        if style.background_color.is_none()
            && style.color.is_none()
            && style.border_color.is_none()
            && style.border_width.is_none()
            && style.border_radius.is_none()
            && style.opacity.is_none()
        {
            None
        } else {
            Some(style)
        }
    }

    /// Apply paint onto a takumi layout Style builder.
    pub fn apply(&self, mut style: Style) -> Style {
        if let Some(c) = self.background_color {
            style = style.with(StyleDeclaration::background_color(ColorInput::Value(c)));
        }
        if let Some(c) = self.color {
            style = style.with(StyleDeclaration::color(ColorInput::Value(c)));
        }
        if let Some(c) = self.border_color {
            style = style
                .with(StyleDeclaration::border_top_color(ColorInput::Value(c)))
                .with(StyleDeclaration::border_right_color(ColorInput::Value(c)))
                .with(StyleDeclaration::border_bottom_color(ColorInput::Value(c)))
                .with(StyleDeclaration::border_left_color(ColorInput::Value(c)));
        }
        if let Some(w) = self.border_width {
            let lw = LineWidth::Length(Length::Px(w));
            style = style
                .with(StyleDeclaration::border_top_width(lw))
                .with(StyleDeclaration::border_right_width(lw))
                .with(StyleDeclaration::border_bottom_width(lw))
                .with(StyleDeclaration::border_left_width(lw))
                .with(StyleDeclaration::border_top_style(BorderStyle::Solid))
                .with(StyleDeclaration::border_right_style(BorderStyle::Solid))
                .with(StyleDeclaration::border_bottom_style(BorderStyle::Solid))
                .with(StyleDeclaration::border_left_style(BorderStyle::Solid));
        }
        if let Some(r) = self.border_radius {
            let len = Length::Px(r);
            // SpacePair for each corner.
            style = style
                .with(StyleDeclaration::border_top_left_radius(
                    SpacePair::from_single(len),
                ))
                .with(StyleDeclaration::border_top_right_radius(
                    SpacePair::from_single(len),
                ))
                .with(StyleDeclaration::border_bottom_right_radius(
                    SpacePair::from_single(len),
                ))
                .with(StyleDeclaration::border_bottom_left_radius(
                    SpacePair::from_single(len),
                ));
        }
        if let Some(o) = self.opacity {
            style = style.with(StyleDeclaration::opacity(PercentageNumber(o)));
        }
        if let Some(shadow) = self.box_shadow {
            let shadows: Box<[BoxShadow]> = Box::from([shadow]);
            style = style.with(StyleDeclaration::box_shadow(Some(shadows)));
        }
        style
    }
}

/// Parse a single `box-shadow` layer (CSS). Multi-layer: first only for v1.
fn parse_box_shadow(css: &str) -> Option<BoxShadow> {
    let css = css.trim();
    if css.is_empty() || css == "none" {
        return None;
    }
    // Prefer takumi CSS parser when available.
    if let Ok(shadow) = BoxShadow::from_css_str(css.split(',').next().unwrap_or(css).trim()) {
        return Some(shadow);
    }
    // Fallback: `0 1px 2px oklch(...)` / `inset 0 0 0 1px …`
    let first = css.split(',').next()?.trim();
    let mut inset = false;
    let mut lengths: Vec<f32> = Vec::new();
    let mut color = ColorInput::Value(Color::from_rgb(0x000000));
    let mut rest = first;
    if let Some(r) = rest.strip_prefix("inset") {
        inset = true;
        rest = r.trim();
    }
    // Walk tokens: lengths then optional color.
    let mut i = 0;
    let bytes = rest.as_bytes();
    while i < bytes.len() {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        // Color function or hex starts non-digit (except sign).
        if bytes[i] == b'#' || bytes[i].is_ascii_alphabetic() {
            let color_str = rest[i..].trim();
            if let Some(c) = parse_color(color_str) {
                color = ColorInput::Value(c);
            }
            break;
        }
        // Length token
        let start = i;
        if bytes[i] == b'-' || bytes[i] == b'+' {
            i += 1;
        }
        while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b'.') {
            i += 1;
        }
        let num: f32 = rest[start..i].parse().ok()?;
        // unit
        let unit_start = i;
        while i < bytes.len() && bytes[i].is_ascii_alphabetic() {
            i += 1;
        }
        let unit = &rest[unit_start..i];
        let px = match unit {
            "" | "px" => num,
            _ => num, // treat as px for static render
        };
        lengths.push(px);
        if lengths.len() >= 4 {
            // remaining is color
            let color_str = rest[i..].trim();
            if !color_str.is_empty() {
                if let Some(c) = parse_color(color_str) {
                    color = ColorInput::Value(c);
                }
            }
            break;
        }
    }
    if lengths.len() < 2 {
        return None;
    }
    Some(
        BoxShadow::builder()
            .inset(inset)
            .offset_x(Length::Px(lengths[0]))
            .offset_y(Length::Px(lengths[1]))
            .blur_radius(Length::Px(*lengths.get(2).unwrap_or(&0.0)))
            .spread_radius(Length::Px(*lengths.get(3).unwrap_or(&0.0)))
            .color(color)
            .build(),
    )
}

fn palette_misses_var(raw: &str, palette: &ThemePalette) -> bool {
    // Extract first var(--name)
    if let Some(start) = raw.find("var(--") {
        let rest = &raw[start + 6..];
        let name = rest
            .split(|c: char| c == ')' || c == ',' || c.is_whitespace())
            .next()
            .unwrap_or("");
        return !name.is_empty() && palette.get(name).is_none();
    }
    false
}

/// Format `Color([r,g,b,a])` as `rgba(r, g, b, a)` (a 0–255).
pub fn color_to_css_rgba(c: Color) -> String {
    let [r, g, b, a] = c.0;
    format!("rgba({r}, {g}, {b}, {a})")
}

pub fn parse_color(css: &str) -> Option<Color> {
    let css = css.trim();
    if css.is_empty() || css == "none" || css == "inherit" || css == "currentcolor" {
        return None;
    }
    if css.eq_ignore_ascii_case("transparent") {
        return Some(Color::transparent());
    }
    // Takumi CSS color parser (oklch, hex, rgb, color-mix, …).
    match ColorInput::from_css_str(css) {
        Ok(ColorInput::Value(c)) => Some(c),
        Ok(ColorInput::CurrentColor) | Ok(_) => None,
        Err(_) => {
            // Hex fallback #rgb / #rrggbb
            if let Some(hex) = css.strip_prefix('#') {
                return parse_hex(hex);
            }
            None
        }
    }
}

fn parse_hex(hex: &str) -> Option<Color> {
    let hex = hex.trim();
    match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
            Some(Color([r, g, b, 255]))
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(Color([r, g, b, 255]))
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
            Some(Color([r, g, b, a]))
        }
        _ => None,
    }
}

pub fn parse_px(css: &str) -> Option<f32> {
    let css = css.trim();
    if css == "0" {
        return Some(0.0);
    }
    if let Some(n) = css.strip_suffix("px") {
        return n.trim().parse().ok();
    }
    if let Some(n) = css.strip_suffix('%') {
        // Treat % radius as 0 for static screenshot (needs box size).
        let _ = n;
        return None;
    }
    css.parse().ok()
}

/// Parse `font-size` CSS into device px for static render.
/// Handles `px`, `rem`, and `clamp(min, preferred, max)` (uses max for hero fidelity).
pub fn parse_font_size_px(css: &str) -> Option<f32> {
    let css = css.trim();
    if let Some(px) = parse_css_length_px(css) {
        return Some(px);
    }
    // clamp(3rem, 9vw, 8.5rem) — prefer the max length for static display type.
    if let Some(inner) = css
        .strip_prefix("clamp(")
        .and_then(|s| s.strip_suffix(')'))
    {
        let mut best: Option<f32> = None;
        for part in inner.split(',') {
            if let Some(px) = parse_css_length_px(part.trim()) {
                best = Some(best.map_or(px, |b| b.max(px)));
            }
        }
        return best;
    }
    None
}

fn parse_css_length_px(css: &str) -> Option<f32> {
    let css = css.trim();
    if let Some(n) = css.strip_suffix("px") {
        return n.trim().parse().ok();
    }
    if let Some(n) = css.strip_suffix("rem") {
        return n.trim().parse::<f32>().ok().map(|v| v * 16.0);
    }
    if let Some(n) = css.strip_suffix("em") {
        // Relative to an unknown parent — treat as root 16px for static paint.
        return n.trim().parse::<f32>().ok().map(|v| v * 16.0);
    }
    None
}

fn parse_font_weight(css: &str) -> Option<f32> {
    let css = css.trim();
    match css {
        "normal" => Some(400.0),
        "bold" => Some(700.0),
        _ => css.parse().ok(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{ThemePalette, ThemeSelect};
    use sail_layout::html::Element;
    use std::collections::HashMap;

    #[test]
    fn parses_atomic_rules() {
        let sheet = StyleSheet::parse(
            ".s-bgprimary{background-color:var(--primary)}.s-radiussm{border-radius:var(--radius-sm)}",
        );
        let d = sheet.decls_for_classes("s-bgprimary s-radiussm");
        assert_eq!(d.len(), 2);
        assert_eq!(d[0].0, "background-color");
    }

    #[test]
    fn resolves_primary_to_color() {
        let palette = ThemePalette::resolve(ThemeSelect::default(), None).unwrap();
        let sheet = StyleSheet::parse(".s-bgprimary{background-color:var(--primary)}");
        let paint = Paint::from_decls(&sheet.decls_for_classes("s-bgprimary"), &palette);
        assert!(paint.background_color.is_some(), "primary must paint");
    }

    /// Hosts stamp every CVA class; only attr-qualified compounds may paint.
    /// Cascade is **class-list order** (last matching decl wins in
    /// `Paint::from_decls`) — compounds must stamp after base transparent.
    #[test]
    fn cva_attr_selectors_filter_and_class_order_wins() {
        let sheet = StyleSheet::parse(
            r#"
            .s-bgtransparent{background-color:transparent}
            [variant='default'].s-variantdefault-bgbackground{background-color:var(--background)}
            [variant='solid'].s-variantsolid-bgforeground{background-color:var(--foreground)}
            [tone='primary'][variant='solid'].s-toneprimaryvariantsolid-bgprimary{background-color:var(--primary)}
            [tone='info'][variant='solid'].s-toneinfovariantsolid-bginfo{background-color:var(--info)}
            "#,
        );

        // Real-ish stamp: base + all variants/tones; attrs select solid+primary.
        let mut attrs = HashMap::new();
        attrs.insert("variant".into(), "solid".into());
        attrs.insert("tone".into(), "primary".into());
        let host = Element {
            tag: "button".into(),
            class: [
                "s-bgtransparent",
                "s-variantdefault-bgbackground",
                "s-variantsolid-bgforeground",
                "s-toneinfovariantsolid-bginfo",
                "s-toneprimaryvariantsolid-bgprimary",
            ]
            .join(" "),
            attrs,
            ..Default::default()
        };
        let decls = sheet.decls_for_element(&host);
        let bgs: Vec<&str> = decls
            .iter()
            .filter(|(k, _)| k == "background-color")
            .map(|(_, v)| v.as_str())
            .collect();
        // transparent base + solid foreground + primary (info tone must NOT match).
        assert!(
            bgs.contains(&"transparent"),
            "base transparent must match: {bgs:?}"
        );
        assert!(
            bgs.contains(&"var(--foreground)"),
            "solid variant fill: {bgs:?}"
        );
        assert!(
            bgs.contains(&"var(--primary)"),
            "primary+solid compound: {bgs:?}"
        );
        assert!(
            !bgs.contains(&"var(--info)"),
            "mismatched tone must not contribute: {bgs:?}"
        );
        assert_eq!(
            bgs.last().copied(),
            Some("var(--primary)"),
            "class-list order: compound after base → last write is primary ({bgs:?})"
        );

        // Default variant alone: no primary compound.
        let mut attrs = HashMap::new();
        attrs.insert("variant".into(), "default".into());
        let def = Element {
            tag: "button".into(),
            class: "s-bgtransparent s-variantdefault-bgbackground s-toneprimaryvariantsolid-bgprimary"
                .into(),
            attrs,
            ..Default::default()
        };
        let def_decls = sheet.decls_for_element(&def);
        let def_bgs: Vec<&str> = def_decls
            .iter()
            .filter(|(k, _)| k == "background-color")
            .map(|(_, v)| v.as_str())
            .collect();
        assert!(
            !def_bgs.contains(&"var(--primary)"),
            "primary+solid class without matching attrs must not paint: {def_bgs:?}"
        );
        assert!(
            def_bgs.contains(&"var(--background)"),
            "default variant must paint: {def_bgs:?}"
        );
    }
}
