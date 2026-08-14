//! Style autocomplete from validate SSOT (SPEC-STYLE-COMPLETE-1).
//!
//! Completions at a caret inside a Style context are **exactly** the values
//! Style validate would accept in that position. One Style service;
//! complete ⊆ validate.

use std::path::Path;

use crate::channel::Channel;
use crate::component_surfaces::color_paint_vocabulary;
use crate::plains::{self, PlainSpec};
use crate::scales;

/// One completion item offered inside a Style region.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyleCompleteItem {
    pub label: String,
    pub kind: StyleCompleteKind,
    pub detail: Option<String>,
    pub insert_text: Option<String>,
}

/// Kind of Style completion (maps to LSP detail / editor icons).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StyleCompleteKind {
    Setter,
    Token,
    Keyword,
    ObjectKey,
    WhenKey,
    StyleRef,
}

impl StyleCompleteKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Setter => "setter",
            Self::Token => "token",
            Self::Keyword => "keyword",
            Self::ObjectKey => "object-key",
            Self::WhenKey => "when-key",
            Self::StyleRef => "style-ref",
        }
    }
}

/// Classified caret position inside a Style expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StyleCompleteContext {
    /// After a channel colon / chain head / `.` — product setter names.
    SetterName {
        partial: String,
        /// Channel key when known (`box` / `paint` / `text` / `animate`).
        channel: Option<String>,
    },
    /// Inside `setter(¦partial)` — tokens/keywords legal for that setter.
    SetterArg {
        setter: String,
        partial: String,
        host: Option<String>,
    },
    /// Inside `border({ ¦` — concern slot names.
    ConcernSlotKey { setter: String, partial: String },
    /// Inside `border({ color: ¦` — values legal for that slot's scale.
    ConcernSlotArg {
        setter: String,
        slot: String,
        partial: String,
        host: Option<String>,
    },
    /// Object key position `{ ¦` / after comma at style-object level.
    ObjectKey { partial: String },
    /// Inside `when: { ¦` key position.
    WhenKey { partial: String },
    /// First-arg style ref: `style(base¦` or composition head word.
    StyleRef { partial: String },
    /// Caret not in a Style region, or position not classified.
    Unknown,
}

/// Keyed members legal at the top of `style({ … })`. Channels are dissolved.
pub const OBJECT_KEYS: &[&str] = &["animate", "when", "match", "compound"];

/// Closed product `when` keys. Dynamic `state-*` / `group-*` families
/// remain open-ended and are not enumerated here.
pub fn when_keys() -> Vec<&'static str> {
    crate::intent::closed_when_keys()
}

/// Completions at `byte_offset` inside `source`. Prefer Style when the caret
/// is in a Style region; empty when outside / unclassified.
///
/// `host` is the L1 host tag when completing a component unit (`Some("chip")`);
/// `None` for app Style (D1: no component public surface names).
pub fn style_complete_at(
    source: &str,
    _path: Option<&Path>,
    byte_offset: usize,
    host: Option<&str>,
) -> Vec<StyleCompleteItem> {
    let ctx = style_complete_context_at(source, byte_offset, host);
    style_complete_for_context(&ctx, source)
}

/// Detect Style completion context at `byte_offset`.
pub fn style_complete_context_at(
    source: &str,
    byte_offset: usize,
    host: Option<&str>,
) -> StyleCompleteContext {
    let offset = byte_offset.min(source.len());
    let Some(region) = find_style_region(source, offset) else {
        return StyleCompleteContext::Unknown;
    };
    classify_in_style(source, region, offset, host)
}

/// Produce completion items for an already-classified context.
///
/// `source` is used only for StyleRef env names (`const name = style`).
pub fn style_complete_for_context(
    ctx: &StyleCompleteContext,
    source: &str,
) -> Vec<StyleCompleteItem> {
    match ctx {
        StyleCompleteContext::Unknown => Vec::new(),
        StyleCompleteContext::ObjectKey { partial } => {
            let mut items: Vec<StyleCompleteItem> = OBJECT_KEYS
                .iter()
                .map(|k| {
                    item(
                        (*k).to_string(),
                        StyleCompleteKind::ObjectKey,
                        Some("style object key".into()),
                        Some(format!("{k}: ")),
                    )
                })
                .collect();
            // Dissolved channels: concern/axis/plain calls start here too.
            for row in crate::vocab::offered_rows() {
                if row.name == "play" {
                    continue;
                }
                let detail = match row.tag {
                    crate::VocabTag::Concern => "concern",
                    crate::VocabTag::Axis => "axis",
                    crate::VocabTag::Plain => "plain",
                };
                items.push(item(
                    row.name.to_string(),
                    StyleCompleteKind::Setter,
                    Some(detail.into()),
                    Some(format!("{}(", row.name)),
                ));
            }
            filter_sort(items.into_iter(), partial)
        }
        StyleCompleteContext::WhenKey { partial } => filter_sort(
            when_keys().into_iter().map(|k| {
                item(
                    k.to_string(),
                    StyleCompleteKind::WhenKey,
                    Some("when key".into()),
                    Some(format!("{k}: ")),
                )
            }),
            partial,
        ),
        StyleCompleteContext::SetterName { partial, channel } => {
            let ch = channel.as_deref().and_then(parse_channel);
            let mut items: Vec<StyleCompleteItem> = Vec::new();
            for row in crate::vocab::offered_rows() {
                if row.name == "play" && ch != Some(Channel::Animate) {
                    continue;
                }
                if ch.is_some_and(|c| parse_channel(row.channel) != Some(c)) {
                    continue;
                }
                let detail = match row.tag {
                    crate::VocabTag::Concern => format!("concern · {}", row.channel),
                    crate::VocabTag::Axis => format!("axis · {}", row.channel),
                    crate::VocabTag::Plain => format!("plain · {}", row.channel),
                };
                items.push(item(
                    row.name.to_string(),
                    StyleCompleteKind::Setter,
                    Some(detail),
                    Some(format!("{}(", row.name)),
                ));
            }
            filter_sort(items.into_iter(), partial)
        }
        StyleCompleteContext::SetterArg {
            setter,
            partial,
            host,
        } => {
            if setter == "play" {
                return filter_sort(
                    crate::ThemeStyleEnv::from_registry().play_names().map(|n| {
                        item(
                            n.to_string(),
                            StyleCompleteKind::Token,
                            Some("play preset".into()),
                            None,
                        )
                    }),
                    partial,
                );
            }
            if let Some(concern) = crate::concerns::find_concern(setter) {
                return filter_sort(concern_arg_items(concern, host.as_deref()), partial);
            }
            let Some(row) = crate::vocab::find_vocab(setter) else {
                return Vec::new();
            };
            if row.tag != crate::vocab::VocabTag::Plain {
                return Vec::new();
            }
            let Some(spec) = plains::find_plain(row.name) else {
                return Vec::new();
            };
            filter_sort(plain_arg_items(spec, host.as_deref()), partial)
        }
        StyleCompleteContext::ConcernSlotKey { setter, partial } => {
            let Some(concern) = crate::concerns::find_concern(setter) else {
                return Vec::new();
            };
            filter_sort(
                concern.slots.iter().map(|s| {
                    item(
                        s.name.to_string(),
                        StyleCompleteKind::ObjectKey,
                        Some(format!("slot · {}", setter)),
                        Some(format!("{}: ", s.name)),
                    )
                }),
                partial,
            )
        }
        StyleCompleteContext::ConcernSlotArg {
            setter,
            slot,
            partial,
            host,
        } => {
            let Some(concern) = crate::concerns::find_concern(setter) else {
                return Vec::new();
            };
            let Some(slot_spec) = crate::concerns::find_slot(concern, slot) else {
                return Vec::new();
            };
            filter_sort(concern_slot_arg_items(slot_spec, host.as_deref()), partial)
        }
        StyleCompleteContext::StyleRef { partial } => {
            // Theme-first in both modes (SPEC-SPINNAKER-3 §3). Locals only in Open.
            let theme = crate::ThemeStyleEnv::from_registry();
            let mut items: Vec<StyleCompleteItem> = theme
                .named_style_names()
                .map(|n| {
                    item(
                        n.to_string(),
                        StyleCompleteKind::StyleRef,
                        Some("theme style".into()),
                        None,
                    )
                })
                .collect();
            if crate::current_styling_policy() == crate::StylingPolicy::Open {
                for n in collect_style_const_names(source) {
                    if items.iter().any(|it| it.label == n) {
                        continue;
                    }
                    items.push(item(
                        n,
                        StyleCompleteKind::StyleRef,
                        Some("style const".into()),
                        None,
                    ));
                }
            }
            filter_sort(items.into_iter(), partial)
        }
    }
}

// ── arg vocabulary ──────────────────────────────────────────────────────────

fn concern_arg_items(
    concern: &crate::concerns::ConcernSpec,
    host: Option<&str>,
) -> impl Iterator<Item = StyleCompleteItem> {
    let mut labels: Vec<(String, StyleCompleteKind, Option<String>)> = Vec::new();
    for composite in crate::tokens::composites_of_type(concern.name) {
        let short = composite
            .name
            .strip_prefix(&format!("{}.", concern.name))
            .or_else(|| composite.name.strip_suffix(&format!(".{}", concern.name)))
            .unwrap_or(composite.name);
        if !labels.iter().any(|(l, _, _)| l == short) {
            labels.push((
                short.to_string(),
                StyleCompleteKind::Token,
                Some(format!("composite `{}`", composite.name)),
            ));
        }
    }
    let primary = crate::concerns::primary_slot(concern);
    if let Some(slot) = crate::concerns::find_slot(concern, primary) {
        labels.extend(concern_slot_arg_tuples(slot, host));
    }
    labels
        .into_iter()
        .map(|(label, kind, detail)| item(label, kind, detail, None))
}

fn plain_arg_items(
    spec: &PlainSpec,
    host: Option<&str>,
) -> impl Iterator<Item = StyleCompleteItem> {
    let mut labels: Vec<(String, StyleCompleteKind, Option<String>)> = Vec::new();
    if spec.scale == Some("color") {
        let mut v: Vec<String> = color_paint_vocabulary(host).into_iter().collect();
        v.sort();
        for t in v {
            labels.push((t, StyleCompleteKind::Token, Some("color paint".into())));
        }
    } else if let Some(scale) = spec.scale {
        let mut v = scales::values(scale);
        v.sort();
        v.dedup();
        for t in v {
            labels.push((
                t,
                StyleCompleteKind::Token,
                Some(format!("scale `{scale}`")),
            ));
        }
    }
    for k in spec.keywords {
        if labels.iter().any(|(l, _, _)| l == *k) {
            continue;
        }
        labels.push((
            (*k).to_string(),
            StyleCompleteKind::Keyword,
            Some(format!("keyword · {}", spec.name)),
        ));
    }
    labels
        .into_iter()
        .map(|(label, kind, detail)| item(label, kind, detail, None))
}

fn concern_slot_arg_tuples(
    slot: &crate::concerns::ConcernSlot,
    host: Option<&str>,
) -> Vec<(String, StyleCompleteKind, Option<String>)> {
    let mut labels: Vec<(String, StyleCompleteKind, Option<String>)> = Vec::new();
    if slot.scale == Some("color") {
        let mut v: Vec<String> = color_paint_vocabulary(host).into_iter().collect();
        v.sort();
        for t in v {
            labels.push((t, StyleCompleteKind::Token, Some("color paint".into())));
        }
    } else if let Some(scale) = slot.scale {
        let mut v = scales::values(scale);
        v.sort();
        v.dedup();
        for t in v {
            labels.push((
                t,
                StyleCompleteKind::Token,
                Some(format!("scale `{scale}`")),
            ));
        }
    }
    for k in slot.keywords {
        if labels.iter().any(|(l, _, _)| l == *k) {
            continue;
        }
        labels.push((
            (*k).to_string(),
            StyleCompleteKind::Keyword,
            Some(format!("keyword · {}", slot.name)),
        ));
    }
    labels
}

fn concern_slot_arg_items(
    slot: &crate::concerns::ConcernSlot,
    host: Option<&str>,
) -> impl Iterator<Item = StyleCompleteItem> {
    concern_slot_arg_tuples(slot, host)
        .into_iter()
        .map(|(label, kind, detail)| item(label, kind, detail, None))
}

// ── context detection ───────────────────────────────────────────────────────

/// Inclusive start of `style(` that owns `offset`.
#[derive(Debug, Clone, Copy)]
struct StyleRegion {
    /// Index of `(` after `style`.
    paren_open: usize,
}

fn find_style_region(source: &str, offset: usize) -> Option<StyleRegion> {
    let bytes = source.as_bytes();
    let mut best: Option<StyleRegion> = None;
    let mut i = 0usize;
    while i + 6 <= offset {
        if &bytes[i..i + 6] == b"style(" && is_word_start(bytes, i) {
            let paren_open = i + 5;
            // Depth from this `(` to `offset`; if still open, caret is inside.
            let mut depth = 1i32;
            let mut j = paren_open + 1;
            let mut closed_before = false;
            while j < offset {
                match bytes[j] {
                    b'(' => depth += 1,
                    b')' => {
                        depth -= 1;
                        if depth == 0 {
                            closed_before = true;
                            break;
                        }
                    }
                    b'"' | b'\'' => {
                        j = skip_string(bytes, j);
                        continue;
                    }
                    b'/' if j + 1 < bytes.len() && bytes[j + 1] == b'/' => {
                        while j < offset && bytes[j] != b'\n' {
                            j += 1;
                        }
                        continue;
                    }
                    b'/' if j + 1 < bytes.len() && bytes[j + 1] == b'*' => {
                        j += 2;
                        while j + 1 < offset {
                            if bytes[j] == b'*' && bytes[j + 1] == b'/' {
                                j += 2;
                                break;
                            }
                            j += 1;
                        }
                        continue;
                    }
                    _ => {}
                }
                j += 1;
            }
            if !closed_before && depth > 0 {
                best = Some(StyleRegion { paren_open });
            }
            i += 6;
            continue;
        }
        i += 1;
    }
    best
}

fn is_word_start(bytes: &[u8], i: usize) -> bool {
    if i == 0 {
        return true;
    }
    let p = bytes[i - 1];
    !(p.is_ascii_alphanumeric() || p == b'_' || p == b'$')
}

fn skip_string(bytes: &[u8], start: usize) -> usize {
    let quote = bytes[start];
    let mut j = start + 1;
    while j < bytes.len() {
        if bytes[j] == b'\\' {
            j += 2;
            continue;
        }
        if bytes[j] == quote {
            return j + 1;
        }
        j += 1;
    }
    bytes.len()
}

fn classify_in_style(
    source: &str,
    region: StyleRegion,
    offset: usize,
    host: Option<&str>,
) -> StyleCompleteContext {
    let (partial, partial_start) = partial_before(source, offset);
    let before = &source[region.paren_open + 1..partial_start];

    // Style ref: first arg is a bare word, not `{` — `style(base¦` / `style(base, …)`.
    if let Some(ctx) = classify_style_ref_head(before, &partial) {
        return ctx;
    }

    // Walk `before` tracking brace/paren nesting and structural position.
    let mut stack: Vec<Frame> = Vec::new();
    // After `style(` the next non-ws is usually `{` for object form.
    let bytes = before.as_bytes();
    let mut i = 0usize;
    let mut last_word: Option<(usize, usize)> = None; // start..end in `before`
    let mut channel: Option<String> = None;
    let mut after_colon = false;
    let mut after_dot = false;
    let mut after_comma_or_open = true; // object key-ish at `{`

    while i < bytes.len() {
        let c = bytes[i];
        match c {
            b' ' | b'\t' | b'\n' | b'\r' => {
                i += 1;
                continue;
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'/' => {
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
                continue;
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'*' => {
                i += 2;
                while i + 1 < bytes.len() {
                    if bytes[i] == b'*' && bytes[i + 1] == b'/' {
                        i += 2;
                        break;
                    }
                    i += 1;
                }
                continue;
            }
            b'"' | b'\'' => {
                i = skip_string(bytes, i);
                after_colon = false;
                after_dot = false;
                after_comma_or_open = false;
                last_word = None;
                continue;
            }
            b'{' => {
                let kind = match stack.last() {
                    Some(Frame::WhenValue) => Frame::WhenMap,
                    Some(Frame::SetterCall { name })
                        if crate::concerns::find_concern(name).is_some() =>
                    {
                        Frame::SlotObject {
                            setter: name.clone(),
                        }
                    }
                    Some(Frame::Object) | Some(Frame::Partial) | None => {
                        if stack.is_empty() {
                            Frame::Object
                        } else {
                            Frame::Partial
                        }
                    }
                    _ => Frame::Partial,
                };
                stack.push(kind);
                after_comma_or_open = true;
                after_colon = false;
                after_dot = false;
                last_word = None;
                i += 1;
            }
            b'}' => {
                stack.pop();
                after_comma_or_open = false;
                after_colon = false;
                after_dot = false;
                last_word = None;
                i += 1;
            }
            b'(' => {
                // Word immediately before `(` is a setter call.
                if let Some((s, e)) = last_word {
                    let name = &before[s..e];
                    stack.push(Frame::SetterCall {
                        name: name.to_string(),
                    });
                } else {
                    stack.push(Frame::Paren);
                }
                after_colon = false;
                after_dot = false;
                after_comma_or_open = false;
                last_word = None;
                i += 1;
            }
            b')' => {
                stack.pop();
                after_colon = false;
                after_dot = false;
                after_comma_or_open = false;
                last_word = None;
                i += 1;
            }
            b'[' => {
                stack.push(Frame::Bracket);
                after_comma_or_open = true;
                after_colon = false;
                after_dot = false;
                last_word = None;
                i += 1;
            }
            b']' => {
                stack.pop();
                after_comma_or_open = false;
                after_colon = false;
                after_dot = false;
                last_word = None;
                i += 1;
            }
            b':' => {
                if let Some((s, e)) = last_word {
                    let key = &before[s..e];
                    match key {
                        "box" | "paint" | "text" | "animate" => {
                            channel = Some(key.to_string());
                            after_colon = true;
                            after_dot = false;
                            after_comma_or_open = false;
                            // Channel body starts — not a when map.
                        }
                        "when" => {
                            stack.push(Frame::WhenValue);
                            after_colon = true;
                            after_dot = false;
                            after_comma_or_open = false;
                        }
                        "match" | "compound" => {
                            after_colon = true;
                            after_dot = false;
                            after_comma_or_open = false;
                        }
                        _ => {
                            // when key or match arm key: value follows
                            if matches!(stack.last(), Some(Frame::WhenMap)) {
                                // after when key colon → partial body
                                after_colon = true;
                            } else {
                                after_colon = true;
                            }
                            after_dot = false;
                            after_comma_or_open = false;
                        }
                    }
                } else {
                    after_colon = true;
                    after_comma_or_open = false;
                }
                last_word = None;
                i += 1;
            }
            b',' => {
                after_comma_or_open = true;
                after_colon = false;
                after_dot = false;
                last_word = None;
                // Leaving a channel value: reset channel when at object level.
                if matches!(stack.last(), Some(Frame::Object) | Some(Frame::Partial)) {
                    channel = None;
                }
                i += 1;
            }
            b'.' => {
                after_dot = true;
                after_colon = false;
                after_comma_or_open = false;
                last_word = None;
                i += 1;
            }
            c if is_ident_start_byte(c) => {
                let start = i;
                i += 1;
                while i < bytes.len() && is_ident_byte(bytes[i]) {
                    i += 1;
                }
                // Dotted token path: `chip.foreground` — keep consuming.
                // But `.` after a completed call is chain, not path inside word.
                last_word = Some((start, i));
                after_comma_or_open = false;
                // after_colon / after_dot stay until we see punctuation
            }
            _ => {
                after_colon = false;
                after_dot = false;
                after_comma_or_open = false;
                last_word = None;
                i += 1;
            }
        }
    }

    // Classify caret using stack + flags.
    if let Some(Frame::SlotObject { setter }) = stack.last() {
        if after_colon {
            let slot = last_word
                .map(|(s, e)| before[s..e].to_string())
                .unwrap_or_default();
            let slot_name = slot_key_before_colon(before).unwrap_or(slot);
            return StyleCompleteContext::ConcernSlotArg {
                setter: setter.clone(),
                slot: slot_name,
                partial,
                host: host.map(|h| h.to_string()),
            };
        }
        if after_comma_or_open || looks_like_object_key_position(before) {
            return StyleCompleteContext::ConcernSlotKey {
                setter: setter.clone(),
                partial,
            };
        }
        return StyleCompleteContext::ConcernSlotKey {
            setter: setter.clone(),
            partial,
        };
    }
    if let Some(Frame::SetterCall { name }) = stack.last() {
        return StyleCompleteContext::SetterArg {
            setter: name.clone(),
            partial,
            host: host.map(|h| h.to_string()),
        };
    }

    if matches!(stack.last(), Some(Frame::WhenMap)) && after_comma_or_open {
        return StyleCompleteContext::WhenKey { partial };
    }

    // Object key: top of style object / nested partial at key position.
    if after_comma_or_open
        && matches!(
            stack.last(),
            Some(Frame::Object) | Some(Frame::Partial) | None
        )
        && !after_colon
        && !after_dot
    {
        // Nested partials use same object keys (minus axes sometimes) —
        // offer the full object key set; validate refuses axes if illegal.
        if matches!(stack.last(), Some(Frame::Object) | Some(Frame::Partial))
            || (stack.is_empty() && before.chars().all(|c| c.is_whitespace()))
        {
            // Empty `style(¦` already handled as StyleRef; `style({¦` is Object.
            if matches!(stack.last(), Some(Frame::Object) | Some(Frame::Partial)) {
                return StyleCompleteContext::ObjectKey { partial };
            }
        }
    }

    // Setter name: after channel colon, after `.` chain, or mid-channel body.
    if after_dot || after_colon {
        return StyleCompleteContext::SetterName {
            partial,
            channel: channel.clone(),
        };
    }

    // Mid-channel: after a finished `setter(...)` the next token is a `.` or
    // another setter only if object form uses `.` chains — after `bg(primary)`
    // before `.` we don't complete. After whitespace following a call in
    // channel body, authors type `.radius` — require after_dot.
    //
    // If we're inside a channel body (channel known, stack has Object) and the
    // partial is non-empty with no open setter, treat as SetterName.
    if channel.is_some()
        && matches!(stack.last(), Some(Frame::Object) | Some(Frame::Partial))
        && !partial.is_empty()
    {
        return StyleCompleteContext::SetterName {
            partial,
            channel: channel.clone(),
        };
    }

    // Nested when map key mid-word without trailing comma flag.
    if matches!(stack.last(), Some(Frame::WhenMap)) {
        return StyleCompleteContext::WhenKey { partial };
    }

    // Inside object, empty partial at key position already handled; mid-key:
    if matches!(stack.last(), Some(Frame::Object) | Some(Frame::Partial))
        && !after_colon
        && !after_dot
    {
        // Could be mid object key (`pa¦int`) or mid setter after channel —
        // if last structural was colon (channel value), last_word path may
        // have left after_colon false once we consumed partial into partial.
        // When partial is being typed at object level without a prior colon
        // on this entry, it's an object key.
        if looks_like_object_key_position(before) {
            return StyleCompleteContext::ObjectKey { partial };
        }
        if channel.is_some() {
            return StyleCompleteContext::SetterName {
                partial,
                channel: channel.clone(),
            };
        }
    }

    StyleCompleteContext::Unknown
}

#[derive(Debug, Clone)]
enum Frame {
    Object,
    Partial,
    WhenValue,
    WhenMap,
    SetterCall { name: String },
    SlotObject { setter: String },
    Paren,
    Bracket,
}

fn slot_key_before_colon(before: &str) -> Option<String> {
    let trimmed = before.trim_end();
    let colon = trimmed.rfind(':')?;
    let head = trimmed[..colon].trim_end();
    let start = head
        .rfind(|c: char| !c.is_ascii_alphanumeric() && c != '-' && c != '_')
        .map(|i| i + 1)
        .unwrap_or(0);
    let key = &head[start..];
    if key.is_empty() {
        None
    } else {
        Some(key.to_string())
    }
}

fn classify_style_ref_head(before: &str, partial: &str) -> Option<StyleCompleteContext> {
    let t = before.trim_start();
    if t.is_empty() {
        // `style(¦` or `style(par¦` — StyleRef (not yet `{`).
        // If partial is empty and user might type `{`, we still offer refs;
        // object keys kick in after `{`.
        return Some(StyleCompleteContext::StyleRef {
            partial: partial.to_string(),
        });
    }
    // `style(base, ¦` — composition: second arg is usually `{` or another ref.
    // First non-ws is `{` → not StyleRef head.
    if t.starts_with('{') {
        return None;
    }
    // Only whitespace + optional first word already in `partial` handled via
    // before being empty of structure.
    // If before has a word then comma, second-arg position:
    let mut chars = t.chars().peekable();
    if let Some(c) = chars.peek() {
        if is_ident_start_byte(*c as u8) || (*c as u8).is_ascii_alphanumeric() {
            // Consume first ident
            let rest = t.trim_start();
            let mut end = 0;
            let rb = rest.as_bytes();
            if !rb.is_empty() && is_ident_start_byte(rb[0]) {
                end = 1;
                while end < rb.len() && is_ident_byte(rb[end]) {
                    end += 1;
                }
            }
            let after = rest[end..].trim_start();
            if after.is_empty() {
                // `style(base¦` — partial is that word (already stripped into partial)
                // Actually before excludes partial, so `style(bas` → before empty, partial "bas".
                // `style(base` with caret after e → before empty if partial is "base".
                return None;
            }
            if let Some(stripped) = after.strip_prefix(',') {
                let after_comma = stripped.trim_start();
                if after_comma.is_empty() || !after_comma.starts_with('{') {
                    return Some(StyleCompleteContext::StyleRef {
                        partial: partial.to_string(),
                    });
                }
            }
        }
    }
    None
}

fn looks_like_object_key_position(before: &str) -> bool {
    let t = before.trim_end();
    t.ends_with('{') || t.ends_with(',')
}

fn parse_channel(s: &str) -> Option<Channel> {
    match s {
        "box" => Some(Channel::Box),
        "paint" => Some(Channel::Paint),
        "text" => Some(Channel::Text),
        "animate" => Some(Channel::Animate),
        _ => None,
    }
}

/// Word (ident + optional dotted path) immediately left of `offset`.
fn partial_before(source: &str, offset: usize) -> (String, usize) {
    let bytes = source.as_bytes();
    let end = offset.min(bytes.len());
    // Don't consume into following ident if caret mid-token — end is caret.
    let mut start = end;
    while start > 0 {
        let b = bytes[start - 1];
        if is_ident_byte(b) || b == b'-' || b == b'.' {
            // Dotted paths / kebab tokens (`surface-1`, `chip.foreground`).
            start -= 1;
        } else {
            break;
        }
    }
    // Don't let partial cross whitespace or punctuation incorrectly.
    // Strip leading `.` if we started mid-chain after a call's `.`.
    while start < end && bytes[start] == b'.' {
        start += 1;
    }
    let partial = std::str::from_utf8(&bytes[start..end])
        .unwrap_or("")
        .to_string();
    (partial, start)
}

fn is_ident_start_byte(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'_' || b == b'$'
}

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'$'
}

// ── style const env ─────────────────────────────────────────────────────────

/// Collect file-local style names: `const name = style` and `style name =`.
pub fn collect_style_const_names(source: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = source.as_bytes();
    let mut i = 0usize;
    while i + 5 < bytes.len() {
        if &bytes[i..i + 5] == b"style" && is_word_start(bytes, i) {
            let after = i + 5;
            if after < bytes.len() && is_ident_byte(bytes[after]) {
                i += 1;
                continue;
            }
            let mut j = after;
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            if j < bytes.len() && is_ident_start_byte(bytes[j]) {
                let name_start = j;
                j += 1;
                while j < bytes.len() && is_ident_byte(bytes[j]) {
                    j += 1;
                }
                let name = std::str::from_utf8(&bytes[name_start..j])
                    .unwrap_or("")
                    .to_string();
                let mut k = j;
                while k < bytes.len() && bytes[k].is_ascii_whitespace() {
                    k += 1;
                }
                if k < bytes.len() && bytes[k] == b'=' {
                    out.push(name);
                    i = k + 1;
                    continue;
                }
            }
            i += 1;
            continue;
        }
        if &bytes[i..i + 5] == b"const" && is_word_start(bytes, i) {
            let after = i + 5;
            if after < bytes.len() && is_ident_byte(bytes[after]) {
                i += 1;
                continue;
            }
            let mut j = after;
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            if j >= bytes.len() || !is_ident_start_byte(bytes[j]) {
                i += 1;
                continue;
            }
            let name_start = j;
            j += 1;
            while j < bytes.len() && is_ident_byte(bytes[j]) {
                j += 1;
            }
            let name = std::str::from_utf8(&bytes[name_start..j])
                .unwrap_or("")
                .to_string();
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b'=' {
                j += 1;
                while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                    j += 1;
                }
                if j + 5 <= bytes.len() && &bytes[j..j + 5] == b"style" {
                    let after_style = j + 5;
                    if after_style >= bytes.len() || !is_ident_byte(bytes[after_style]) {
                        out.push(name);
                    }
                }
            }
            i = j;
            continue;
        }
        i += 1;
    }
    out.sort();
    out.dedup();
    out
}

// ── helpers ─────────────────────────────────────────────────────────────────

fn item(
    label: String,
    kind: StyleCompleteKind,
    detail: Option<String>,
    insert_text: Option<String>,
) -> StyleCompleteItem {
    StyleCompleteItem {
        label,
        kind,
        detail,
        insert_text,
    }
}

fn filter_sort<I>(items: I, partial: &str) -> Vec<StyleCompleteItem>
where
    I: Iterator<Item = StyleCompleteItem>,
{
    let p = partial.to_ascii_lowercase();
    let mut out: Vec<StyleCompleteItem> = items
        .filter(|it| {
            if p.is_empty() {
                true
            } else {
                it.label.to_ascii_lowercase().starts_with(&p)
            }
        })
        .collect();
    out.sort_by(|a, b| {
        a.label
            .cmp(&b.label)
            .then_with(|| a.kind.as_str().cmp(b.kind.as_str()))
    });
    out.dedup_by(|a, b| a.label == b.label);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host_ctx::with_host_surface;
    use crate::ir::StyleDiagnostic;
    use crate::parse;

    fn offset_of(src: &str, needle: &str) -> usize {
        src.find(needle)
            .unwrap_or_else(|| panic!("needle `{needle}` not in source"))
    }

    /// Caret just after `¦` marker (marker removed from source).
    fn at_caret(src: &str) -> (String, usize) {
        let i = src.find('¦').expect("caret marker ¦");
        let mut s = String::with_capacity(src.len() - '¦'.len_utf8());
        s.push_str(&src[..i]);
        s.push_str(&src[i + '¦'.len_utf8()..]);
        (s, i)
    }

    fn labels(items: &[StyleCompleteItem]) -> Vec<&str> {
        items.iter().map(|i| i.label.as_str()).collect()
    }

    #[test]
    fn fill_color_slot_includes_primary_and_surface_excludes_radius_and_chip_fg() {
        let (src, off) = at_caret("style({ bg({ color: ¦ }) })");
        let items = style_complete_at(&src, None, off, None);
        let labs = labels(&items);
        assert!(labs.contains(&"primary"), "primary: {labs:?}");
        assert!(labs.contains(&"surface-1"), "surface-1: {labs:?}");
        // radius scale-only names must not dominate color complete
        assert!(
            !labs.contains(&"md") || labs.contains(&"primary"),
            "color complete should be color tokens, not radius-first"
        );
        // Bare radius token name isn't a color paint token
        assert!(!labs.iter().any(|l| {
            *l == "full"
                && items
                    .iter()
                    .any(|i| i.label == "full" && i.detail.as_deref() == Some("scale `radius`"))
        }));
        // Pack names are gone — complete spends globals.
        assert!(
            !labs.contains(&"chip-foreground"),
            "chip-foreground: {labs:?}"
        );
        assert!(!labs.contains(&"chip"), "bare chip: {labs:?}");
        assert!(labs.contains(&"card"), "card is a global: {labs:?}");
    }

    #[test]
    fn radius_arg_includes_md_sm_full_not_color_primaries_as_primary_set() {
        let (src, off) = at_caret("style({ radius(¦) })");
        let items = style_complete_at(&src, None, off, None);
        let labs = labels(&items);
        assert!(
            labs.contains(&"md") || labs.contains(&"sm"),
            "radius steps: {labs:?}"
        );
        assert!(
            labs.contains(&"full") || labs.contains(&"none"),
            "radius keywords: {labs:?}"
        );
        // Color primaries may coincidentally not appear; ensure radius set is scale-driven.
        let radius_vals = scales::values("radius");
        assert!(
            labs.iter().any(|l| radius_vals.iter().any(|r| r == l)),
            "expected radius scale members in {labs:?}"
        );
        // primary is a color token — should not be required here; if present, still
        // complete ⊆ validate must hold (radius may reject it). Prefer absence.
        assert!(
            !labs.contains(&"primary"),
            "color primary must not appear in radius complete: {labs:?}"
        );
    }

    #[test]
    fn object_key_after_style_brace() {
        let (src, off) = at_caret("style({ ¦ })");
        let items = style_complete_at(&src, None, off, None);
        let labs = labels(&items);
        for k in ["when", "animate", "match", "compound"] {
            assert!(labs.contains(&k), "missing object key `{k}` in {labs:?}");
        }
        for k in ["box", "paint", "text"] {
            assert!(
                !labs.contains(&k),
                "dissolved channel key `{k}` must not be offered: {labs:?}"
            );
        }
    }

    #[test]
    fn host_chip_spends_globals() {
        let (src, off) = at_caret("style({ bg({ color: ¦ }) })");
        let items = style_complete_at(&src, None, off, Some("chip"));
        let labs = labels(&items);
        assert!(labs.contains(&"muted"), "muted: {labs:?}");
        assert!(labs.contains(&"card"), "card: {labs:?}");
        assert!(!labs.contains(&"chip"), "pack name gone: {labs:?}");
        assert!(
            !labs.contains(&"chip-foreground"),
            "kebab pack label gone: {labs:?}"
        );
    }

    #[test]
    fn complete_tokens_subset_of_validate_for_common_setters() {
        for setter in ["radius"] {
            let (src, off) = at_caret(&format!("style({{ {setter}(¦) }})"));
            let items = style_complete_at(&src, None, off, None);
            assert!(!items.is_empty(), "{setter} must complete something");
            // Sample up to 40 labels (stable sort) and prove validate accepts.
            for it in items.iter().take(40) {
                let expr = format!("style({{ {setter}({}) }})", it.label);
                let check = || {
                    let (_, diags) = parse::parse(&expr);
                    let bad = diags.iter().any(|d| {
                        matches!(
                            d,
                            StyleDiagnostic::BadToken { .. }
                                | StyleDiagnostic::ThemeOwnedToken { .. }
                                | StyleDiagnostic::UnknownSetter { .. }
                        )
                    });
                    assert!(
                        !bad,
                        "complete ⊈ validate for {setter}({}): {:?}",
                        it.label,
                        diags.iter().map(|d| d.message()).collect::<Vec<_>>()
                    );
                };
                // Host-free app context for this table.
                with_host_surface(None, check);
            }
            // Scale membership for non-keyword tokens — concern primary slot.
            if let Some(concern) = crate::concerns::find_concern(setter) {
                let primary = crate::concerns::primary_slot(concern);
                if let Some(slot) = crate::concerns::find_slot(concern, primary) {
                    if let Some(scale) = slot.scale {
                        if scale != "color" {
                            for it in items
                                .iter()
                                .filter(|i| i.kind == StyleCompleteKind::Token)
                                .take(20)
                            {
                                assert!(
                                    scales::contains(scale, &it.label)
                                        || slot.keywords.contains(&it.label.as_str()),
                                    "{setter} token `{}` not on scale `{scale}`",
                                    it.label
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn when_key_position() {
        let (src, off) = at_caret("style({ bg(primary), when: { ¦ } })");
        let items = style_complete_at(&src, None, off, None);
        let labs = labels(&items);
        assert!(labs.contains(&"hover"), "when keys: {labs:?}");
        assert!(
            labs.contains(&"tablet") || labs.contains(&"desktop"),
            "{labs:?}"
        );
    }

    #[test]
    fn setter_names_at_style_object_start() {
        let (src, off) = at_caret("style({ ¦ })");
        let items = style_complete_at(&src, None, off, None);
        let labs = labels(&items);
        assert!(labs.contains(&"bg"), "live author name: {labs:?}");
        assert!(labs.contains(&"fg"), "live author name: {labs:?}");
        assert!(labs.contains(&"pad"), "axis at object start: {labs:?}");
        assert!(labs.contains(&"radius"), "setters: {labs:?}");
        assert!(labs.contains(&"elevation"), "concern setter: {labs:?}");
        assert!(
            !labs.contains(&"fill") && !labs.contains(&"ink"),
            "fill/ink are alias targets, not offered: {labs:?}"
        );
    }

    #[test]
    fn complete_offers_bg_not_fill_dual() {
        let (src, off) = at_caret("style({ ¦ })");
        let items = style_complete_at(&src, None, off, None);
        let labs = labels(&items);
        assert!(labs.contains(&"bg"), "complete must offer bg: {labs:?}");
        assert!(
            !labs.contains(&"fill"),
            "complete must not dual-offer fill beside bg: {labs:?}"
        );
        assert!(!labs.contains(&"z"), "z is a position slot, not a setter");
    }

    #[test]
    fn border_arg_completes_concern_names() {
        let (src, off) = at_caret("style({ border(¦) })");
        let items = style_complete_at(&src, None, off, None);
        let labs = labels(&items);
        assert!(labs.contains(&"subtle"), "border.subtle: {labs:?}");
        assert!(labs.contains(&"default"), "border.default: {labs:?}");
        assert!(labs.contains(&"strong"), "border.strong: {labs:?}");
        assert!(!labs.contains(&"dialog"), "dialog.border is gone: {labs:?}");
    }

    #[test]
    fn border_slot_object_keys() {
        let (src, off) = at_caret("style({ border({ ¦ }) })");
        let items = style_complete_at(&src, None, off, None);
        let labs = labels(&items);
        assert!(labs.contains(&"color"), "{labs:?}");
        assert!(labs.contains(&"width"), "{labs:?}");
        assert!(labs.contains(&"style"), "{labs:?}");
        assert!(!labs.contains(&"radius"), "radius is not a border slot");
    }

    #[test]
    fn border_color_slot_completes_color_roles() {
        let (src, off) = at_caret("style({ border({ color: ¦ }) })");
        let items = style_complete_at(&src, None, off, None);
        let labs = labels(&items);
        assert!(
            labs.contains(&"primary") || labs.contains(&"muted"),
            "{labs:?}"
        );
        for width in ["hairline", "control-stroke", "0", "1", "2", "8"] {
            assert!(
                !labs.contains(&width),
                "width token `{width}` must not appear in color slot: {labs:?}"
            );
        }
        assert!(
            !items
                .iter()
                .any(|i| i.detail.as_deref() == Some("scale `border-width`")),
            "color slot must not offer width scale: {labs:?}"
        );
    }

    #[test]
    fn style_ref_from_const() {
        let src = "const filled = style({ bg(primary) })\nstyle(fil¦)";
        let (src, off) = at_caret(src);
        let items = style_complete_at(&src, None, off, None);
        let labs = labels(&items);
        assert!(labs.contains(&"filled"), "style refs: {labs:?}");
    }

    #[test]
    fn style_ref_from_style_name_decl() {
        let src = "style filled = { bg(primary) }\nstyle(fil¦)";
        let (src, off) = at_caret(src);
        let items = style_complete_at(&src, None, off, None);
        let labs = labels(&items);
        assert!(labs.contains(&"filled"), "style name = refs: {labs:?}");
    }

    #[test]
    fn themed_complete_omits_file_local_refs() {
        let src = "const filled = style({ bg(primary) })\nstyle(fil¦)";
        let (src, off) = at_caret(src);
        crate::with_styling_policy(crate::StylingPolicy::Themed, || {
            let items = style_complete_at(&src, None, off, None);
            let labs = labels(&items);
            assert!(
                !labs.contains(&"filled"),
                "themed must not offer locals: {labs:?}"
            );
        });
    }

    #[test]
    fn play_arg_completes_fade_from_theme() {
        let (src, off) = at_caret("style({ animate: { enter: play(¦) } })");
        let items = style_complete_at(&src, None, off, None);
        let labs = labels(&items);
        assert!(labs.contains(&"fade"), "play presets: {labs:?}");
        assert!(labs.contains(&"colors"), "play prop-set: {labs:?}");
    }

    #[test]
    fn outside_style_is_unknown() {
        let (src, off) = at_caret("<div>¦</div>");
        let items = style_complete_at(&src, None, off, None);
        assert!(items.is_empty());
        assert!(matches!(
            style_complete_context_at(&src, off, None),
            StyleCompleteContext::Unknown
        ));
    }

    #[test]
    fn partial_prefix_filters() {
        let (src, off) = at_caret("style({ bg({ color: pri¦ }) })");
        let items = style_complete_at(&src, None, off, None);
        assert!(!items.is_empty());
        for it in &items {
            assert!(
                it.label.to_ascii_lowercase().starts_with("pri"),
                "label {}",
                it.label
            );
        }
        assert!(labels(&items).contains(&"primary"));
    }

    #[test]
    fn context_detection_helpers() {
        let src = "style({ bg() })";
        let off = offset_of(src, "bg(") + 3; // inside parens
        let ctx = style_complete_context_at(src, off, None);
        assert!(
            matches!(ctx, StyleCompleteContext::SetterArg { ref setter, .. } if setter == "bg"),
            "{ctx:?}"
        );
    }
}
