//! Compose pass — rhythm, contrast, and alignment advisory gates.
//!
//! Runs after the layout pass and reuses the same taffy tree (built from the
//! preprocessed source) so we never pay for a second layout computation.
//!
//! **Spacing is resolved structurally** via `spinnaker::parse` on each
//! child's block `style({…})` statement (and parent `gap` attrs) — not by
//! substring fingerprinting lowered `s-mr*` class tokens.
//!
//! Rhythm: ≥3 siblings, modal spacing ≥60% → repair outliers to the modal
//! space-scale token. No mode → WARN only (deliberate asymmetry).
//! Contrast: fg/bg token pairs under WCAG AA → confluent in-scale repair.
//! Alignment-drift: sibling edges 2–6px off a dominant edge → advisory WARN.

use std::collections::HashMap;
use std::path::Path;

use crate::check::layout::{SharedBoxes, SharedViewport};
use crate::check::lint_overrides::{LintOverride, OverrideSeverity};
use crate::diagnostics::{DiagSeverity, Diagnostic};

const RHYTHM_RULE: &str = "sail-compose-rhythm";
const CONTRAST_RULE: &str = "sail-compose-contrast";
const ALIGN_RULE: &str = "sail-compose-align";
const SPACING_RULE: &str = "sail-compose-spacing";
/// Two gaps are considered the same value within this pixel tolerance.
const GAP_EPSILON: f32 = 1.0;
/// A mode is "clear" when it covers at least this fraction of measured gaps.
const MODE_THRESHOLD: f64 = 0.60;
/// WCAG AA normal-text contrast ratio.
const WCAG_AA: f64 = 4.5;
/// Alignment-drift band (px) from the dominant edge — advisory only.
const ALIGN_DRIFT_MIN: f32 = 2.0;
const ALIGN_DRIFT_MAX: f32 = 6.0;

/// Collect rhythm diagnostics for `source`.
///
/// `overrides` lets a host force the pass severity (Error/Warning/Fix/Off).
/// When no override is present, the nearest `sail.config.json` decides: the
/// `compose_rhythm` key defaults to `Fix`; set it to `false` for warnings only.
pub fn collect(
    source: &str,
    source_path: Option<&Path>,
    overrides: &[LintOverride],
) -> Vec<Diagnostic> {
    collect_with(source, source_path, overrides, None)
}

/// Like [`collect`], using layout's forest + trees when the check walk
/// already built them. Does not rebuild taffy.
pub fn collect_with(
    source: &str,
    source_path: Option<&Path>,
    overrides: &[LintOverride],
    boxes: Option<&SharedBoxes>,
) -> Vec<Diagnostic> {
    let severity = match resolve_severity(source_path, overrides) {
        Some(s) => s,
        None => return Vec::new(),
    };

    let owned;
    let (preprocessed, elements, trees): (&str, &[sail_layout::html::Element], &[SharedViewport]) =
        if let Some(b) = boxes {
            (
                b.preprocessed.as_str(),
                b.elements.as_slice(),
                b.trees.as_slice(),
            )
        } else {
            match crate::parse::preprocess::preprocess_with_style(source) {
                Ok((pre, hosts)) => {
                    owned = crate::check::layout::prepare_boxes(
                        pre,
                        &hosts,
                        crate::check::layout::layout_check_options_for(source_path).1,
                    );
                    (
                        owned.preprocessed.as_str(),
                        owned.elements.as_slice(),
                        owned.trees.as_slice(),
                    )
                }
                Err(_) => {
                    // Style errors can prevent preprocess; still surface off-scale
                    // spacing from raw author tags so starter rewrites stay available.
                    return collect_off_scale_for_check(source, source_path, overrides);
                }
            }
        };

    let ids: Vec<Fingerprint> = elements
        .iter()
        .map(|e| element_identity(e, elements))
        .collect();

    // Per-parent-instance rhythm data (elem_idx), merged across viewports.
    // Key by elem_idx — not fingerprint alone — so two sibling <row>s with the
    // same tag fingerprint are checked independently (AUTOFIX-1 multi-parent).
    let mut buckets: HashMap<usize, RhythmBucket> = HashMap::new();

    for tree in trees {
        let Ok(layout) = &tree.layout else {
            continue;
        };

        for elem_idx in 1..elements.len() {
            let Some(node_id) = layout.nodes[elem_idx] else {
                continue;
            };
            let style = &layout.styles[elem_idx].style;
            let container_kind = container_kind(style);
            if container_kind == ContainerKind::None {
                continue;
            }
            let Ok(taffy_children) = layout.taffy.children(node_id) else {
                continue;
            };
            if taffy_children.len() < 3 {
                continue;
            }

            // Build (elem_index, layout) pairs for children that taffy laid out.
            let mut child_layouts: Vec<(usize, sail_layout::taffy::tree::Layout)> = Vec::new();
            for child_node in taffy_children {
                let Ok(child_layout) = layout.taffy.layout(child_node).cloned() else {
                    continue;
                };
                let Some(ctx) = layout.taffy.get_node_context(child_node) else {
                    continue;
                };
                child_layouts.push((ctx.elem_index, child_layout));
            }

            let fp = ids[elem_idx].clone();
            let pre_index = count_identity_before(&ids, elem_idx);

            // Read source-level spacing values. Parent gap can never vary, so it
            // only produces a fix when it is absent and children express spacing
            // via margin styles. Use this instance's pre_index (not 0).
            let Some(pattern) = spacing_pattern(
                source,
                preprocessed,
                &fp,
                pre_index,
                elements,
                &ids,
                elem_idx,
                container_kind,
            ) else {
                continue;
            };
            let gaps = pattern.values();
            if gaps.len() < 3 {
                continue;
            }

            let bucket = buckets.entry(elem_idx).or_default();
            if bucket.pattern.is_some() {
                // Source-level spacing patterns are viewport-independent; keep
                // the first collected viewport to avoid duplicate diagnostics.
                continue;
            }
            bucket.gaps.extend(gaps);
            bucket.container_kind = container_kind;
            bucket.pre_index = pre_index;
            bucket.fp = Some(fp);
            bucket.child_layouts = child_layouts;
            bucket.pattern = Some(pattern);
        }
    }

    let mut out: Vec<Diagnostic> = Vec::new();
    for bucket in buckets.into_values() {
        let Some(fp) = bucket.fp else {
            continue;
        };
        let Some(pattern) = bucket.pattern else {
            continue;
        };
        let Some((mode_px, _count)) = find_mode(&bucket.gaps) else {
            // No clear mode — deliberate asymmetry; warn only (never a fix).
            if severity != DiagSeverity::Fix && severity != DiagSeverity::Warning {
                continue;
            }
            let sev = if severity == DiagSeverity::Fix {
                DiagSeverity::Warning
            } else {
                severity
            };
            let message = format!(
                "rhythm violation: sibling spacing has no clear mode ({}) — manual review required [sail:fix:warn]",
                gap_list(&bucket.gaps)
            );
            if let Some(span) = find_original_span(source, preprocessed, &fp, bucket.pre_index) {
                let (line, col, end_line, end_col) = span_to_coords(source, span);
                out.push(Diagnostic {
                    line,
                    col,
                    end_line,
                    end_col,
                    severity: sev,
                    message,
                    rule: Some(RHYTHM_RULE.into()),
                });
            }
            continue;
        };

        let target_token = snap_to_space_token(mode_px);

        if severity == DiagSeverity::Error || severity == DiagSeverity::Warning {
            let message = format!(
                "rhythm violation: sibling spacing is inconsistent ({}) — consider matching margins or gap [sail:fix:warn]",
                gap_list(&bucket.gaps)
            );
            if let Some(span) = find_original_span(source, preprocessed, &fp, bucket.pre_index) {
                let (line, col, end_line, end_col) = span_to_coords(source, span);
                out.push(Diagnostic {
                    line,
                    col,
                    end_line,
                    end_col,
                    severity,
                    message,
                    rule: Some(RHYTHM_RULE.into()),
                });
            }
            continue;
        }

        match pattern {
            SpacingPattern::ChildMargin { setter, pairs } => {
                for (child_pre_idx, current) in pairs {
                    let Some(current_token) = current.as_scale_token() else {
                        // Expression / non-literal — cannot confluently repair.
                        continue;
                    };
                    if current_token == target_token {
                        continue;
                    }
                    let child_fp = ids[child_pre_idx].clone();
                    let child_pre_index = count_identity_before(&ids, child_pre_idx);
                    if let Some(span) =
                        find_original_span(source, preprocessed, &child_fp, child_pre_index)
                    {
                        let (line, col, end_line, end_col) = span_to_coords(source, span);
                        let message = format!(
                            "rhythm violation: sibling margins are inconsistent ({}) — fix child {setter} to \"{}\" [sail:fix:margin={setter}:{target_token}]",
                            gap_list(&bucket.gaps),
                            target_token
                        );
                        out.push(Diagnostic {
                            line,
                            col,
                            end_line,
                            end_col,
                            severity: DiagSeverity::Fix,
                            message,
                            rule: Some(RHYTHM_RULE.into()),
                        });
                    }
                }
            }
        }
    }

    // Contrast + alignment + off-scale spacing gates.
    out.extend(collect_contrast(
        source,
        preprocessed,
        elements,
        &ids,
        severity,
    ));
    out.extend(collect_alignment(
        source,
        preprocessed,
        elements,
        &ids,
        trees,
        severity,
    ));
    out.extend(collect_off_scale_spacing(
        source,
        preprocessed,
        elements,
        severity,
    ));

    out.sort_by(|a, b| a.line.cmp(&b.line).then(a.col.cmp(&b.col)));
    out
}

fn resolve_severity(
    source_path: Option<&Path>,
    overrides: &[LintOverride],
) -> Option<DiagSeverity> {
    for o in overrides {
        if o.rule == RHYTHM_RULE {
            return match o.severity {
                OverrideSeverity::Off => return None,
                OverrideSeverity::Error => Some(DiagSeverity::Error),
                OverrideSeverity::Warn => Some(DiagSeverity::Warning),
                OverrideSeverity::Fix => Some(DiagSeverity::Fix),
            };
        }
    }
    let default = crate::check::policy::Policy::resolve_opt(source_path.unwrap_or(Path::new("")))
        .map(|p| p.compose_rhythm_severity())
        .unwrap_or(OverrideSeverity::Fix);
    match default {
        OverrideSeverity::Off => None,
        OverrideSeverity::Error => Some(DiagSeverity::Error),
        OverrideSeverity::Warn => Some(DiagSeverity::Warning),
        OverrideSeverity::Fix => Some(DiagSeverity::Fix),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum ContainerKind {
    #[default]
    None,
    Row,
    Column,
}

fn container_kind(style: &sail_layout::taffy::style::Style) -> ContainerKind {
    use sail_layout::taffy::style::{Display, FlexDirection};
    if style.display != Display::Flex {
        return ContainerKind::None;
    }
    match style.flex_direction {
        FlexDirection::Row | FlexDirection::RowReverse => ContainerKind::Row,
        FlexDirection::Column | FlexDirection::ColumnReverse => ContainerKind::Column,
    }
}

/// Typed spacing value from the Style AST (not lowered class text).
#[derive(Debug, Clone, PartialEq)]
enum SpacingValue {
    /// Space-scale token reference (`4`, `2`, `half`, …).
    ScaleToken(String),
    /// Numeric / px literal — may be snapped to a token for repair.
    Literal(f32),
    /// Non-literal expression (bind, match arm variance) — never auto-fixed.
    Expression,
}

impl SpacingValue {
    fn as_px(&self) -> Option<f32> {
        match self {
            SpacingValue::ScaleToken(t) => token_to_px(t),
            SpacingValue::Literal(px) => Some(*px),
            SpacingValue::Expression => None,
        }
    }

    fn as_scale_token(&self) -> Option<String> {
        match self {
            SpacingValue::ScaleToken(t) => Some(t.clone()),
            SpacingValue::Literal(px) => Some(snap_to_space_token(*px)),
            SpacingValue::Expression => None,
        }
    }
}

#[derive(Debug, Clone)]
enum SpacingPattern {
    /// Per-child margin style (mr for rows, mb for columns).
    ChildMargin {
        setter: &'static str,
        pairs: Vec<(usize, SpacingValue)>,
    },
}

impl SpacingPattern {
    fn values(&self) -> Vec<f32> {
        match self {
            SpacingPattern::ChildMargin { pairs, .. } => {
                pairs.iter().filter_map(|(_, v)| v.as_px()).collect()
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
struct RhythmBucket {
    gaps: Vec<f32>,
    container_kind: ContainerKind,
    pre_index: usize,
    /// Parent fingerprint for span mapping (set when pattern is collected).
    fp: Option<Fingerprint>,
    child_layouts: Vec<(usize, sail_layout::taffy::tree::Layout)>,
    pattern: Option<SpacingPattern>,
}

fn find_mode(gaps: &[f32]) -> Option<(f32, usize)> {
    if gaps.is_empty() {
        return None;
    }
    // Find the largest epsilon-cluster.
    let mut best: Option<(f32, usize)> = None;
    for (i, g) in gaps.iter().enumerate() {
        // Only start a cluster from a gap that isn't already covered by a
        // better cluster's epsilon neighbourhood? To avoid O(n^2) still okay
        // for small gap counts; just count each centre.
        let count = gaps
            .iter()
            .filter(|h| (g - *h).abs() <= GAP_EPSILON)
            .count();
        if best.is_none_or(|(_, c)| count > c) {
            best = Some((*g, count));
        }
        // Don't let very close duplicate centres both win; advance past this
        // epsilon neighbourhood so we don't re-count the same cluster.
        if i + 1 < gaps.len() && (gaps[i + 1] - g).abs() <= GAP_EPSILON {
            continue;
        }
    }
    let (mode_px, count) = best?;
    if (count as f64) < (gaps.len() as f64) * MODE_THRESHOLD {
        return None;
    }
    Some((mode_px, count))
}

fn gap_list(gaps: &[f32]) -> String {
    gaps.iter()
        .map(|g| format!("{:.1}", g))
        .collect::<Vec<_>>()
        .join(", ")
}

fn spacing_pattern(
    source: &str,
    preprocessed: &str,
    parent_fp: &Fingerprint,
    parent_pre_index: usize,
    elements: &[sail_layout::html::Element],
    ids: &[Fingerprint],
    parent_idx: usize,
    kind: ContainerKind,
) -> Option<SpacingPattern> {
    // Parent gap attr → uniform container spacing; no per-child variance fix.
    // Resolve *this* parent instance (parent_pre_index), not fingerprint#0.
    let parent_span = find_original_span(source, preprocessed, parent_fp, parent_pre_index)?;
    let (parent_tag_end, _, parent_attrs) = parse_open_tag(source, parent_span.0)?;
    if parent_attrs.iter().any(|(n, _)| n == "gap") {
        return None;
    }
    // Structural: parent Style gap setter also means uniform gap.
    if let Some(sx) = element_style_expr(source, parent_tag_end, &parent_attrs) {
        if spacing_from_sx_expr(&sx, "gap").is_some() {
            return None;
        }
    }

    let setter = match kind {
        ContainerKind::Row => "mr",
        ContainerKind::Column => "mb",
        ContainerKind::None => return None,
    };

    let mut pairs = Vec::new();
    for &child_idx in &elements[parent_idx].children {
        let child_fp = ids[child_idx].clone();
        let child_pre = count_identity_before(ids, child_idx);
        let Some(span) = find_original_span(source, preprocessed, &child_fp, child_pre) else {
            continue;
        };
        let Some((tag_end, _, attrs)) = parse_open_tag(source, span.0) else {
            continue;
        };
        let Some(sx) = element_style_expr(source, tag_end, &attrs) else {
            continue;
        };
        let Some(val) = spacing_from_sx_expr(&sx, setter) else {
            continue;
        };
        pairs.push((child_idx, val));
    }
    if pairs.len() < 3 {
        return None;
    }
    Some(SpacingPattern::ChildMargin { setter, pairs })
}

fn attr_value(attrs: &[(String, String)], name: &str) -> Option<String> {
    attrs
        .iter()
        .find(|(n, _)| n == name)
        .map(|(_, v)| v.clone())
}

/// Resolve a spacing setter from a Style expression using the Style AST.
fn spacing_from_sx_expr(sx_raw: &str, setter: &str) -> Option<SpacingValue> {
    let expr = strip_sx_braces(sx_raw);
    if expr.is_empty() {
        return None;
    }
    let wrapped = if expr.starts_with("style(") {
        expr.to_string()
    } else {
        format!("style({{ {expr} }})")
    };
    let (style, _diags) = spinnaker::parse(&wrapped);
    let mut ops = Vec::new();
    collect_paint_ops(&style, &mut ops);
    for op in ops.into_iter().rev() {
        if op.setter == setter {
            return paint_arg_to_spacing(op.first_arg());
        }
        // v3 concern form: margin({ b: 4 }) is the retired `mb(4)` slot.
        if let Some((concern, slot)) = retired_spacing_slot(setter) {
            if op.setter == concern {
                if let spinnaker::PaintArg::Slots(slots) = op.first_arg() {
                    if let Some(arg) = slots.get(slot) {
                        return paint_arg_to_spacing(arg);
                    }
                }
            }
        }
    }
    None
}

fn retired_spacing_slot(setter: &str) -> Option<(&'static str, &'static str)> {
    match setter {
        "mb" => Some(("margin", "b")),
        "mt" => Some(("margin", "t")),
        "ml" | "ms" => Some(("margin", "s")),
        "mr" | "me" => Some(("margin", "e")),
        "mx" => Some(("margin", "x")),
        "my" => Some(("margin", "y")),
        "gap" => Some(("pad", "gap")),
        _ => None,
    }
}

fn strip_sx_braces(raw: &str) -> &str {
    let t = raw.trim();
    if t.starts_with('{') && t.ends_with('}') && t.len() >= 2 {
        t[1..t.len() - 1].trim()
    } else {
        t
    }
}

/// Author Style expression for an element: block `style({…})` after the open
/// tag (product), with residual `style={…}` attr fallback for fixtures mid-migrate.
fn element_style_expr(source: &str, tag_end: usize, attrs: &[(String, String)]) -> Option<String> {
    if let Some(sx) = attr_value(attrs, "style") {
        let expr = strip_sx_braces(&sx);
        if !expr.is_empty() {
            return Some(expr.to_string());
        }
    }
    tag_style_expr_before(source, tag_end).or_else(|| block_style_expr_after(source, tag_end))
}

/// First block Style statement immediately after an open tag (`>`).

/// Tag-position `style(…)` call ending at open tag `>` (`gt_pos`).
fn tag_style_expr_before(source: &str, gt_pos: usize) -> Option<String> {
    if gt_pos == 0 || gt_pos > source.len() {
        return None;
    }
    let bytes = source.as_bytes();
    let mut open = gt_pos;
    while open > 0 {
        if bytes[open - 1] == b'<' {
            open -= 1;
            break;
        }
        open -= 1;
    }
    let mut tag_end = open + 1;
    while tag_end < gt_pos
        && (bytes[tag_end].is_ascii_alphanumeric()
            || bytes[tag_end] == b'-'
            || bytes[tag_end] == b'_'
            || bytes[tag_end] == b':')
    {
        tag_end += 1;
    }
    let mut j = tag_end;
    let mut quote: Option<u8> = None;
    while j < gt_pos {
        match (quote, bytes[j]) {
            (Some(q), b) if b == q => {
                quote = None;
                j += 1;
            }
            (None, b'"') | (None, b'\'') => {
                quote = Some(bytes[j]);
                j += 1;
            }
            (Some(_), _) => j += 1,
            (None, _) => {
                if j + 5 <= gt_pos
                    && &bytes[j..j + 5] == b"style"
                    && (j == 0
                        || (!bytes[j - 1].is_ascii_alphanumeric()
                            && bytes[j - 1] != b'_'
                            && bytes[j - 1] != b'$'))
                    && (j + 5 >= gt_pos
                        || (!bytes[j + 5].is_ascii_alphanumeric()
                            && bytes[j + 5] != b'_'
                            && bytes[j + 5] != b':'))
                {
                    let mut k = j + 5;
                    while k < gt_pos && bytes[k].is_ascii_whitespace() {
                        k += 1;
                    }
                    if k < gt_pos && bytes[k] == b'(' {
                        let rest = &source[j..gt_pos];
                        if let Some(end) = consume_balanced_style_expr(rest) {
                            let expr = rest[..end].trim_end();
                            if !expr.is_empty() {
                                return Some(expr.to_string());
                            }
                        } else {
                            // layout.rs may not have consume_balanced_style_expr
                            // fallback: simple paren balance
                            let end = rest.find('>').unwrap_or(rest.len());
                            let expr = rest[..end].trim_end();
                            if expr.starts_with("style(") {
                                return Some(expr.to_string());
                            }
                        }
                    }
                }
                j += 1;
            }
        }
    }
    None
}

fn block_style_expr_after(source: &str, tag_end: usize) -> Option<String> {
    if tag_end >= source.len() {
        return None;
    }
    let body = &source[tag_end..];
    let bytes = body.as_bytes();
    let mut start = 0;
    while start < bytes.len() && bytes[start].is_ascii_whitespace() {
        start += 1;
    }
    if start >= bytes.len() {
        return None;
    }
    if bytes[start] == b'<' || bytes[start] == b'{' {
        return None;
    }
    let rest = &body[start..];
    let t = rest.trim_start();
    if !(t.starts_with("style(")
        || t.starts_with("style.")
        || t.starts_with("match(")
        || t.starts_with("when(")
        || t.starts_with("compound(")
        || t.starts_with('['))
    {
        return None;
    }
    let end = consume_balanced_style_expr(rest)?;
    let expr = rest[..end].trim_end();
    if expr.is_empty() {
        return None;
    }
    Some(expr.to_string())
}

/// Consume a Style product expression (paren/brace/bracket aware).
fn consume_balanced_style_expr(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut i = 0;
    let mut paren = 0i32;
    let mut brace = 0i32;
    let mut bracket = 0i32;
    let mut quote: Option<u8> = None;
    let mut saw_group = false;
    while i < bytes.len() {
        let b = bytes[i];
        if let Some(q) = quote {
            if b == q && bytes.get(i.wrapping_sub(1)) != Some(&b'\\') {
                quote = None;
            }
            i += 1;
            continue;
        }
        match b {
            b'"' | b'\'' => quote = Some(b),
            b'(' => {
                paren += 1;
                saw_group = true;
            }
            b')' => {
                paren -= 1;
                if paren < 0 {
                    return None;
                }
            }
            b'{' => {
                brace += 1;
                saw_group = true;
            }
            b'}' => {
                brace -= 1;
                if brace < 0 {
                    return None;
                }
            }
            b'[' => {
                bracket += 1;
                saw_group = true;
            }
            b']' => {
                bracket -= 1;
                if bracket < 0 {
                    return None;
                }
            }
            _ => {}
        }
        i += 1;
        if saw_group && paren == 0 && brace == 0 && bracket == 0 {
            // Finished a top-level call: `style({…})` / `style({…}).when(…)`
            // Continue while `.` chains follow.
            let mut j = i;
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b'.' {
                i = j;
                continue;
            }
            return Some(i);
        }
    }
    if saw_group && paren == 0 && brace == 0 && bracket == 0 {
        Some(i)
    } else {
        None
    }
}

fn collect_paint_ops<'a>(style: &'a spinnaker::Style, out: &mut Vec<&'a spinnaker::PaintOp>) {
    use spinnaker::Style;
    match style {
        Style::Empty | Style::Ref(_) | Style::MotionAttach { .. } => {}
        Style::Set(op) => out.push(op),
        Style::Chain(items) | Style::List(items) => {
            for i in items {
                collect_paint_ops(i, out);
            }
        }
        Style::Match { arms, .. } => {
            for (_, s) in arms {
                collect_paint_ops(s, out);
            }
        }
        Style::Compound { body, .. } => collect_paint_ops(body, out),
        Style::When { entries } => {
            for (_, s) in entries {
                collect_paint_ops(s, out);
            }
        }
    }
}

fn paint_arg_to_spacing(arg: &spinnaker::PaintArg) -> Option<SpacingValue> {
    use spinnaker::PaintArg;
    match arg {
        PaintArg::Bind(_) | PaintArg::MemberPath { .. } => Some(SpacingValue::Expression),
        PaintArg::Token(t) => {
            let t = t.trim();
            if t.is_empty() {
                return None;
            }
            if let Some(px) = parse_px_literal(t) {
                return Some(SpacingValue::Literal(px));
            }
            // Space-scale token (including negatable `-4`).
            let bare = t.strip_prefix('-').unwrap_or(t);
            if token_to_px(bare).is_some() || sail_lint::scales::contains("space", bare) {
                return Some(SpacingValue::ScaleToken(t.to_string()));
            }
            Some(SpacingValue::Expression)
        }
        PaintArg::Slots(_) => None,
    }
}

fn parse_px_literal(t: &str) -> Option<f32> {
    let t = t.trim();
    if let Some(n) = t.strip_suffix("px") {
        return n.trim().parse().ok();
    }
    // Bare number outside scale is a literal px-ish authoring mistake.
    if t.chars().all(|c| c.is_ascii_digit() || c == '.') {
        // Scale tokens are mostly short integers that ARE on the scale —
        // those go through token_to_px first. Only treat as literal when
        // not a known scale name.
        if token_to_px(t).is_none() && !sail_lint::scales::contains("space", t) {
            return t.parse().ok();
        }
    }
    None
}

/// Space-scale token → CSS px (closed table matching token registry defaults).
fn space_token_px_table() -> &'static [(&'static str, f32)] {
    static TOKENS: &[(&str, f32)] = &[
        ("0", 0.0),
        ("px", 1.0),
        ("half", 2.0),
        ("0_75", 3.0),
        ("1", 4.0),
        ("1_25", 5.0),
        ("1_5", 6.0),
        ("2", 8.0),
        ("2_5", 10.0),
        ("3", 12.0),
        ("3_5", 14.0),
        ("4", 16.0),
        ("4_5", 18.0),
        ("5", 20.0),
        ("6", 24.0),
        ("7", 28.0),
        ("8", 32.0),
        ("9", 36.0),
        ("10", 40.0),
        ("11", 44.0),
        ("12", 48.0),
        ("14", 56.0),
        ("16", 64.0),
        ("20", 80.0),
        ("24", 96.0),
        ("28", 112.0),
        ("32", 128.0),
        ("36", 144.0),
        ("40", 160.0),
        ("44", 176.0),
        ("48", 192.0),
        ("52", 208.0),
        ("56", 224.0),
        ("60", 240.0),
        ("64", 256.0),
    ];
    TOKENS
}

fn token_to_px(token: &str) -> Option<f32> {
    space_token_px_table()
        .iter()
        .find(|(t, _)| *t == token)
        .map(|(_, px)| *px)
}

fn snap_to_space_token(px: f32) -> String {
    let mut best = "0";
    let mut best_dist = (px - 0.0).abs();
    for (token, value) in space_token_px_table() {
        let dist = (px - value).abs();
        if dist < best_dist {
            best = token;
            best_dist = dist;
        }
    }
    best.to_string()
}

// ---------------------------------------------------------------------------
// Source mapping — host identity / Style role / author tag (not s-* atoms)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Fingerprint {
    /// Author-facing tag: `data-sail-host`, layout keyword, or Style/taffy role.
    tag: String,
}

fn element_identity(
    elem: &sail_layout::html::Element,
    all: &[sail_layout::html::Element],
) -> Fingerprint {
    if let Some(host) = elem.attrs.get("data-sail-host") {
        let h = host.trim();
        if !h.is_empty() {
            return Fingerprint {
                tag: h.to_ascii_lowercase(),
            };
        }
    }
    let tag = elem.tag.to_ascii_lowercase();
    if layout_keyword_tag(&tag) {
        return Fingerprint { tag };
    }
    let resolved = sail_layout::style::resolve(elem, all, 1440.0);
    match container_kind(&resolved.style) {
        ContainerKind::Column => Fingerprint {
            tag: "column".into(),
        },
        ContainerKind::Row => Fingerprint { tag: "row".into() },
        ContainerKind::None => {
            use sail_layout::taffy::style::Display;
            if resolved.style.display == Display::Grid {
                Fingerprint { tag: "grid".into() }
            } else {
                Fingerprint { tag }
            }
        }
    }
}

fn layout_keyword_tag(tag: &str) -> bool {
    matches!(tag, "column" | "row" | "grid" | "center" | "stack")
}

fn count_identity_before(ids: &[Fingerprint], elem_idx: usize) -> usize {
    if elem_idx <= 1 {
        return 0;
    }
    let tag = &ids[elem_idx].tag;
    ids.iter()
        .enumerate()
        .skip(1)
        .take(elem_idx.saturating_sub(1))
        .filter(|(_, id)| id.tag == *tag)
        .count()
}

fn find_original_span(
    original: &str,
    _preprocessed: &str,
    fingerprint: &Fingerprint,
    pre_index: usize,
) -> Option<(usize, usize)> {
    let mut seen = 0;
    let mut skip_to = 0usize;

    for (pos, ch) in original.char_indices() {
        if pos < skip_to {
            continue;
        }
        if original[pos..].starts_with("<!--") {
            if let Some(end) = original[pos..].find("-->") {
                skip_to = pos + end + 3;
            } else {
                break;
            }
            continue;
        }
        if ch == '<' {
            if let Some((tag_end, tag_name, attrs)) = parse_open_tag(original, pos) {
                if tag_matches_fingerprint(original, tag_end, tag_name, &attrs, fingerprint) {
                    if seen == pre_index {
                        return Some((pos, tag_end));
                    }
                    seen += 1;
                }
                skip_to = tag_end;
            }
        }
    }
    None
}

fn tag_matches_fingerprint(
    _source: &str,
    _tag_end: usize,
    tag_name: &str,
    _attrs: &[(String, String)],
    fingerprint: &Fingerprint,
) -> bool {
    tag_name.eq_ignore_ascii_case(&fingerprint.tag)
}

fn parse_open_tag(source: &str, pos: usize) -> Option<(usize, &str, Vec<(String, String)>)> {
    let bytes = source.as_bytes();
    if bytes.get(pos) != Some(&b'<') {
        return None;
    }
    if bytes.get(pos + 1) == Some(&b'/') || bytes.get(pos + 1) == Some(&b'!') {
        return None;
    }
    let tag_start = pos + 1;
    let mut i = tag_start;
    while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'>' {
        i += 1;
    }
    let tag_name = &source[tag_start..i];
    let mut tag_end = i;
    let mut in_quote: Option<u8> = None;
    while tag_end < bytes.len() {
        match bytes[tag_end] {
            b'"' | b'\'' if in_quote == Some(bytes[tag_end]) => in_quote = None,
            b'"' | b'\'' if in_quote.is_none() => in_quote = Some(bytes[tag_end]),
            b'>' if in_quote.is_none() => {
                tag_end += 1;
                break;
            }
            _ => {}
        }
        tag_end += 1;
    }
    if in_quote.is_some() {
        return None;
    }
    let attrs = parse_attrs(&source[i..tag_end.saturating_sub(1)]);
    Some((tag_end, tag_name, attrs))
}

fn parse_attrs(text: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        let name_start = i;
        while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'=' {
            i += 1;
        }
        let name = text[name_start..i].to_ascii_lowercase();
        let mut value = String::new();
        if i < bytes.len() && bytes[i] == b'=' {
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if i < bytes.len() && (bytes[i] == b'"' || bytes[i] == b'\'') {
                let q = bytes[i];
                i += 1;
                let v_start = i;
                while i < bytes.len() && bytes[i] != q {
                    i += 1;
                }
                value = text[v_start..i].to_string();
                if i < bytes.len() {
                    i += 1;
                }
            } else {
                let v_start = i;
                while i < bytes.len() && !bytes[i].is_ascii_whitespace() {
                    i += 1;
                }
                value = text[v_start..i].to_string();
            }
        }
        if !name.is_empty() {
            out.push((name, value));
        }
    }
    out
}

fn span_to_coords(source: &str, span: (usize, usize)) -> (usize, usize, usize, usize) {
    let (l, c) = crate::diagnostics::line_col(source, span.0);
    let (el, ec) = crate::diagnostics::line_col(source, span.1);
    (l, c, el, ec)
}

// ---------------------------------------------------------------------------
// Contrast gate (auto-fix when confluent within the same scale)
// ---------------------------------------------------------------------------

/// Approximate relative luminance for a small closed set of theme tokens used
/// in compose fixtures and common authoring pairs. Off-scale / unknown pairs
/// are WARN-only (no rewrite).
fn token_luminance(name: &str) -> Option<f64> {
    // Order: approximate WCAG relative luminance L in [0,1] for light mode.
    match name {
        "background" | "surface-0" | "surface-1" | "card" | "muted" | "popover" => Some(0.96),
        "surface-2" | "surface-3" => Some(0.94),
        "primary" => Some(0.35),
        "primary-foreground" => Some(0.98),
        "secondary" => Some(0.40),
        "secondary-foreground" => Some(0.25),
        "accent" => Some(0.45),
        "accent-foreground" => Some(0.98),
        "destructive" => Some(0.30),
        "destructive-foreground" => Some(0.98),
        "muted-foreground" => Some(0.35),
        "foreground" => Some(0.12),
        "border" => Some(0.80),
        _ => None,
    }
}

fn contrast_ratio(l1: f64, l2: f64) -> f64 {
    let (hi, lo) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
    (hi + 0.05) / (lo + 0.05)
}

/// Same-scale families for confluent contrast repair (preserve hue family).
fn color_family(token: &str) -> &'static str {
    if token.starts_with("primary") {
        "primary"
    } else if token.starts_with("secondary") {
        "secondary"
    } else if token.starts_with("accent") {
        "accent"
    } else if token.starts_with("destructive") {
        "destructive"
    } else if token.starts_with("muted") || token == "foreground" || token == "background" {
        "neutral"
    } else if token.starts_with("surface") || token == "card" || token == "popover" {
        "surface"
    } else {
        "other"
    }
}

fn nearest_passing_fg(bg: &str, current_fg: &str) -> Option<&'static str> {
    let bg_l = token_luminance(bg)?;
    let family = color_family(current_fg);
    // Confluent repair stays inside the same hue family (AUTOFIX-1).
    // Surface is a *background* family; readable text on surface is neutral,
    // so surface-fg repairs may land on neutral tokens (not a hue jump).
    let candidates: &[&str] = match family {
        "primary" => &["primary-foreground", "primary"],
        "secondary" => &["secondary-foreground", "secondary"],
        "accent" => &["accent-foreground", "accent"],
        "destructive" => &["destructive-foreground", "destructive"],
        "neutral" => &["foreground", "muted-foreground", "muted", "background"],
        "surface" => &["foreground", "muted-foreground"],
        _ => return None,
    };
    candidates.iter().copied().find(|c| {
        let cand_family = color_family(c);
        let family_ok = cand_family == family || (family == "surface" && cand_family == "neutral");
        family_ok
            && token_luminance(c)
                .map(|l| contrast_ratio(bg_l, l) >= WCAG_AA)
                .unwrap_or(false)
    })
}

fn collect_contrast(
    source: &str,
    preprocessed: &str,
    elements: &[sail_layout::html::Element],
    ids: &[Fingerprint],
    severity: DiagSeverity,
) -> Vec<Diagnostic> {
    if severity == DiagSeverity::Error {
        // strict autofix: still report, but as Error without rewrite payload path
    }
    let mut out = Vec::new();
    for idx in 1..elements.len() {
        let fp = ids[idx].clone();
        let pre_i = count_identity_before(ids, idx);
        let Some(span) = find_original_span(source, preprocessed, &fp, pre_i) else {
            continue;
        };
        let Some((_, _, attrs)) = parse_open_tag(source, span.0) else {
            continue;
        };
        let Some(sx) = attr_value(&attrs, "style") else {
            continue;
        };
        let expr = strip_sx_braces(&sx);
        let (style, _) = spinnaker::parse(expr);
        let mut ops = Vec::new();
        collect_paint_ops(&style, &mut ops);
        let mut bg: Option<String> = None;
        let mut fg: Option<String> = None;
        for op in &ops {
            match op.setter.as_str() {
                "bg" => {
                    if let spinnaker::PaintArg::Token(t) = op.first_arg() {
                        bg = Some(t.clone());
                    }
                }
                "fg" | "color" => {
                    if let spinnaker::PaintArg::Token(t) = op.first_arg() {
                        fg = Some(t.clone());
                    }
                }
                _ => {}
            }
        }
        let (Some(bg_t), Some(fg_t)) = (bg, fg) else {
            continue;
        };
        let (Some(bl), Some(fl)) = (token_luminance(&bg_t), token_luminance(&fg_t)) else {
            // Off-scale / unknown → WARN only.
            if matches!(severity, DiagSeverity::Fix | DiagSeverity::Warning) {
                let (line, col, end_line, end_col) = span_to_coords(source, span);
                out.push(Diagnostic {
                    line,
                    col,
                    end_line,
                    end_col,
                    severity: DiagSeverity::Warning,
                    message: format!(
                        "contrast: unknown token pair fg={fg_t} bg={bg_t} — cannot auto-fix [sail:fix:warn]"
                    ),
                    rule: Some(CONTRAST_RULE.into()),
                });
            }
            continue;
        };
        let ratio = contrast_ratio(bl, fl);
        if ratio >= WCAG_AA {
            continue;
        }
        let (line, col, end_line, end_col) = span_to_coords(source, span);
        if let Some(next_fg) = nearest_passing_fg(&bg_t, &fg_t) {
            let sev = severity;
            out.push(Diagnostic {
                line,
                col,
                end_line,
                end_col,
                severity: sev,
                message: format!(
                    "contrast: {fg_t} on {bg_t} is {ratio:.2}:1 (need {WCAG_AA}) — set fg to \"{next_fg}\" [sail:fix:fg={next_fg}]"
                ),
                rule: Some(CONTRAST_RULE.into()),
            });
        } else {
            // No same-family confluent repair — WARN only.
            out.push(Diagnostic {
                line,
                col,
                end_line,
                end_col,
                severity: DiagSeverity::Warning,
                message: format!(
                    "contrast: {fg_t} on {bg_t} is {ratio:.2}:1 (need {WCAG_AA}) — no in-family repair [sail:fix:warn]"
                ),
                rule: Some(CONTRAST_RULE.into()),
            });
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Off-scale spacing literal → nearest space token (confluent starter rewrite)
// ---------------------------------------------------------------------------

const SPACING_SETTERS: &[&str] = &[
    "mr", "mb", "mt", "ml", "ms", "me", "mx", "my", "m", "pad", "padX", "padY", "padT", "padB",
    "padL", "padR", "gap", "gapX", "gapY",
];

fn collect_off_scale_spacing(
    source: &str,
    _preprocessed: &str,
    _elements: &[sail_layout::html::Element],
    severity: DiagSeverity,
) -> Vec<Diagnostic> {
    collect_off_scale_from_source(source, severity)
}

/// Production entry for off-scale spacing when full parse/preprocess fails
/// (e.g. `mr(25)` is a space-scale BadToken). Used by `check_all` so rewrites
/// still surface on the real check/CLI/MCP path.
pub fn collect_off_scale_for_check(
    source: &str,
    source_path: Option<&Path>,
    overrides: &[LintOverride],
) -> Vec<Diagnostic> {
    let severity = match resolve_severity(source_path, overrides) {
        Some(s) => s,
        None => return Vec::new(),
    };
    collect_off_scale_from_source(source, severity)
}

/// Source-only off-scale scan (does not require successful preprocess).
fn collect_off_scale_from_source(source: &str, severity: DiagSeverity) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    let mut skip_to = 0usize;
    for (pos, ch) in source.char_indices() {
        if pos < skip_to {
            continue;
        }
        if source[pos..].starts_with("<!--") {
            if let Some(end) = source[pos..].find("-->") {
                skip_to = pos + end + 3;
            } else {
                break;
            }
            continue;
        }
        if ch != '<' {
            continue;
        }
        let Some((tag_end, _tag, attrs)) = parse_open_tag(source, pos) else {
            continue;
        };
        skip_to = tag_end;
        let Some(sx) = element_style_expr(source, tag_end, &attrs) else {
            continue;
        };
        let (style, _) = spinnaker::parse(&sx);
        let mut ops = Vec::new();
        collect_paint_ops(&style, &mut ops);
        for op in ops {
            if !SPACING_SETTERS.contains(&op.setter.as_str()) {
                continue;
            }
            let spinnaker::PaintArg::Token(t) = op.first_arg() else {
                continue;
            };
            let t = t.trim();
            if token_to_px(t).is_some() || sail_lint::scales::contains("space", t) {
                continue;
            }
            let Some(px) = (if t.chars().all(|c| c.is_ascii_digit()) {
                t.parse().ok()
            } else {
                parse_px_literal(t)
            }) else {
                continue;
            };
            let target = snap_to_space_token(px);
            let span = (pos, tag_end);
            if !unique_nearest_space_token(px, &target) {
                let (line, col, end_line, end_col) = span_to_coords(source, span);
                out.push(Diagnostic {
                    line,
                    col,
                    end_line,
                    end_col,
                    severity: DiagSeverity::Warning,
                    message: format!(
                        "off-scale spacing: {}({t}) has no unique nearest space token [sail:fix:warn]",
                        op.setter
                    ),
                    rule: Some(SPACING_RULE.into()),
                });
                continue;
            }
            let (line, col, end_line, end_col) = span_to_coords(source, span);
            let sev = severity;
            let setter = op.setter.as_str();
            out.push(Diagnostic {
                line,
                col,
                end_line,
                end_col,
                severity: sev,
                message: format!(
                    "off-scale spacing: {setter}({t}) is not a space token — set {setter} to \"{target}\" [sail:fix:margin={setter}:{target}]"
                ),
                rule: Some(SPACING_RULE.into()),
            });
        }
    }
    out
}

fn unique_nearest_space_token(px: f32, nearest: &str) -> bool {
    let nearest_px = match token_to_px(nearest) {
        Some(v) => v,
        None => return false,
    };
    let nearest_dist = (px - nearest_px).abs();
    let mut second = f32::MAX;
    for (tok, v) in space_token_px_table() {
        if *tok == nearest {
            continue;
        }
        let d = (px - v).abs();
        if d < second {
            second = d;
        }
    }
    // Confluent when unique winner (strictly closer than any other).
    nearest_dist + 0.01 < second
}

// ---------------------------------------------------------------------------
// Alignment-drift (advisory only — never auto-fix)
// ---------------------------------------------------------------------------

/// Pure detector: siblings whose left edge is 2–6px from the dominant left.
/// Returns `(child_index, drift_px, dominant_left)`.
fn left_edge_drifts(
    lefts: &[(usize, f32)],
    min_drift: f32,
    max_drift: f32,
) -> Vec<(usize, f32, f32)> {
    if lefts.len() < 2 {
        return Vec::new();
    }
    let mut counts: HashMap<i32, usize> = HashMap::new();
    for (_, x) in lefts {
        *counts.entry(x.round() as i32).or_default() += 1;
    }
    let Some((&dom, _)) = counts.iter().max_by_key(|(_, c)| *c) else {
        return Vec::new();
    };
    let dom_f = dom as f32;
    let mut out = Vec::new();
    for (child_idx, x) in lefts {
        let drift = (x - dom_f).abs();
        if drift >= min_drift && drift <= max_drift {
            out.push((*child_idx, drift, dom_f));
        }
    }
    out
}

fn collect_alignment(
    source: &str,
    preprocessed: &str,
    elements: &[sail_layout::html::Element],
    ids: &[Fingerprint],
    trees: &[SharedViewport],
    severity: DiagSeverity,
) -> Vec<Diagnostic> {
    if !matches!(
        severity,
        DiagSeverity::Fix | DiagSeverity::Warning | DiagSeverity::Error
    ) {
        return Vec::new();
    }
    let Some(layout) = trees.iter().find_map(|t| t.layout.as_ref().ok()) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for elem_idx in 1..elements.len() {
        let Some(node_id) = layout.nodes[elem_idx] else {
            continue;
        };
        let style = &layout.styles[elem_idx].style;
        if container_kind(style) == ContainerKind::None {
            continue;
        }
        let Ok(taffy_children) = layout.taffy.children(node_id) else {
            continue;
        };
        if taffy_children.len() < 2 {
            continue;
        }
        let mut lefts: Vec<(usize, f32)> = Vec::new();
        for child_node in taffy_children {
            let Ok(child_layout) = layout.taffy.layout(child_node).cloned() else {
                continue;
            };
            let Some(ctx) = layout.taffy.get_node_context(child_node) else {
                continue;
            };
            lefts.push((ctx.elem_index, child_layout.location.x));
        }
        if lefts.len() < 2 {
            continue;
        }
        for (child_idx, drift, dom_f) in left_edge_drifts(&lefts, ALIGN_DRIFT_MIN, ALIGN_DRIFT_MAX)
        {
            let fp = ids[child_idx].clone();
            let pre_i = count_identity_before(ids, child_idx);
            if let Some(span) = find_original_span(source, preprocessed, &fp, pre_i) {
                let (line, col, end_line, end_col) = span_to_coords(source, span);
                out.push(Diagnostic {
                        line,
                        col,
                        end_line,
                        end_col,
                        severity: DiagSeverity::Warning,
                        message: format!(
                            "alignment drift: left edge is {drift:.1}px from dominant left ({dom_f:.0}px) — advisory only [sail:fix:warn]"
                        ),
                        rule: Some(ALIGN_RULE.into()),
                    });
            }
        }
    }
    out
}

#[cfg(test)]
mod spacing_slot_tests {
    use super::*;

    #[test]
    fn margin_b_slot_is_mb() {
        let v = spacing_from_sx_expr("margin({ b: 4 })", "mb");
        assert!(
            matches!(v, Some(SpacingValue::ScaleToken(ref t)) if t == "4"),
            "expected mb slot from margin({{ b: 4 }}), got {v:?}"
        );
    }

    #[test]
    fn collect_with_shared_boxes_does_not_rebuild_taffy() {
        let src = "<column><p>a</p><p>b</p><p>c</p></column>";
        let (pre, hosts) =
            crate::parse::preprocess::preprocess_with_style(src).expect("preprocess");
        let boxes =
            crate::check::layout::prepare_boxes(pre, &hosts, sail_layout::CheckOptions::default());
        let after_build = sail_layout::layout_pass_count();
        let _ = collect_with(src, None, &[], Some(&boxes));
        assert_eq!(
            sail_layout::layout_pass_count(),
            after_build,
            "shared compose must not rebuild taffy"
        );
    }

    #[test]
    fn identity_uses_host_and_style_not_s_atoms() {
        let mut host = sail_layout::html::Element {
            tag: "div".into(),
            class: "s-mr4 s-mb8".into(),
            ..Default::default()
        };
        host.attrs.insert("data-sail-host".into(), "card".into());
        let all = vec![sail_layout::html::Element::default(), host];
        assert_eq!(element_identity(&all[1], &all).tag, "card");

        let col = sail_layout::html::Element {
            tag: "div".into(),
            class: "sail-col".into(),
            ..Default::default()
        };
        let all = vec![sail_layout::html::Element::default(), col];
        assert_eq!(element_identity(&all[1], &all).tag, "column");
    }
}
