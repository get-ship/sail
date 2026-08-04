//! Virtual TypeScript documents for a `.sail` source.
//!
//! Builds synthesized TS text plus a bidirectional source-map so tsgo can
//! type-check controller / scope / template-expression regions while results
//! (hover, completion, definition, diagnostics) round-trip to `.sail` offsets.
//!
//! Two virtual files (mirrors the synthetic emitter's controller+render split):
//! 1. **controller** — authored `<controller>` body (verbatim after ambient
//!    wrap), 1:1 mapped onto the `.sail` controller body.
//! 2. **view** — imports the controller default export, recovers a typed
//!    `scope` via `__def`, embeds `<scope>` type aliases and template
//!    expressions rewritten with a `scope.` prefix.

use sail_compiler::diagnostics::{Span, line_col};
use sail_compiler::emit::expr_rewrite::rewrite_with_prefix;
use sail_compiler::ir::Expr;
use sail_compiler::parse::parse;
use sail_compiler::parse::preprocess::{extract_controller_block, wrap_controller_body};

/// Kind of TS region in the original `.sail`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionKind {
    Controller,
    Scope,
    Expression,
}

/// Which virtual file a segment lives in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtualFile {
    Controller,
    View,
}

/// One contiguous map between virtual TS bytes and `.sail` bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapSegment {
    pub file: VirtualFile,
    pub virtual_start: usize,
    pub virtual_end: usize,
    pub sail_start: usize,
    pub sail_end: usize,
    /// When true, virtual and sail lengths match and offsets translate 1:1.
    pub one_to_one: bool,
}

/// A TS-bearing region of the `.sail` source (for cursor routing).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TsRegion {
    pub start: usize,
    pub end: usize,
    pub kind: RegionKind,
}

/// Virtual TS documents + maps for one `.sail` buffer.
#[derive(Debug, Clone)]
pub struct VirtualTs {
    /// Controller virtual file text (may be empty when no `<controller>`).
    pub controller_text: String,
    /// View virtual file text (scope recovery + expressions + scope types).
    pub view_text: String,
    /// Bidirectional segments.
    pub segments: Vec<MapSegment>,
    /// Regions of the `.sail` that contain TypeScript.
    pub regions: Vec<TsRegion>,
    #[allow(dead_code)]
    pub controller_sail_span: Option<Span>,
    #[allow(dead_code)]
    pub scope_sail_span: Option<Span>,
}

impl VirtualTs {
    pub fn is_ts_region(&self, sail_offset: usize) -> bool {
        self.region_at(sail_offset).is_some()
    }

    pub fn region_at(&self, sail_offset: usize) -> Option<RegionKind> {
        self.regions
            .iter()
            .find(|r| sail_offset >= r.start && sail_offset < r.end)
            .map(|r| r.kind)
    }

    /// Text of a virtual file.
    pub fn text(&self, file: VirtualFile) -> &str {
        match file {
            VirtualFile::Controller => &self.controller_text,
            VirtualFile::View => &self.view_text,
        }
    }

    /// Map a `.sail` byte offset → (virtual file, virtual byte offset).
    pub fn sail_to_virtual(&self, sail_offset: usize) -> Option<(VirtualFile, usize)> {
        for seg in &self.segments {
            if sail_offset >= seg.sail_start && sail_offset <= seg.sail_end {
                if seg.one_to_one {
                    let rel = sail_offset - seg.sail_start;
                    return Some((seg.file, seg.virtual_start + rel));
                }
                return Some((seg.file, seg.virtual_start));
            }
        }
        None
    }

    /// Map a virtual offset in `file` → `.sail` byte offset.
    pub fn virtual_to_sail(&self, file: VirtualFile, virtual_offset: usize) -> Option<usize> {
        for seg in &self.segments {
            if seg.file != file {
                continue;
            }
            if virtual_offset >= seg.virtual_start && virtual_offset <= seg.virtual_end {
                if seg.one_to_one {
                    let rel = virtual_offset - seg.virtual_start;
                    return Some(seg.sail_start + rel);
                }
                let virt_len = seg.virtual_end - seg.virtual_start;
                let sail_len = seg.sail_end - seg.sail_start;
                if virt_len == 0 {
                    return Some(seg.sail_start);
                }
                let rel = virtual_offset - seg.virtual_start;
                let mapped = seg.sail_start + (rel * sail_len) / virt_len;
                return Some(mapped.min(seg.sail_end));
            }
        }
        None
    }

    /// Map a virtual `[start, end)` range in `file` → sail `[start, end)`.
    pub fn map_virtual_range(
        &self,
        file: VirtualFile,
        v_start: usize,
        v_end: usize,
    ) -> Option<(usize, usize)> {
        for seg in &self.segments {
            if seg.file != file {
                continue;
            }
            if v_start >= seg.virtual_start && v_start < seg.virtual_end {
                if seg.one_to_one {
                    let s = self.virtual_to_sail(file, v_start)?;
                    let e = if v_end > v_start {
                        self.virtual_to_sail(file, v_end.saturating_sub(1))
                            .map(|x| (x + 1).min(seg.sail_end))
                            .unwrap_or(s)
                    } else {
                        s
                    };
                    return Some((s, e.max(s)));
                }
                return Some((seg.sail_start, seg.sail_end));
            }
        }
        let s = self.virtual_to_sail(file, v_start)?;
        Some((s, s))
    }

    #[allow(dead_code)]
    pub fn map_sail_range(
        &self,
        s_start: usize,
        s_end: usize,
    ) -> Option<(VirtualFile, usize, usize)> {
        let (file, v) = self.sail_to_virtual(s_start)?;
        if s_end <= s_start {
            return Some((file, v, v));
        }
        for seg in &self.segments {
            if s_start >= seg.sail_start && s_start < seg.sail_end {
                if seg.one_to_one {
                    let end = s_end.min(seg.sail_end);
                    let rel = end - seg.sail_start;
                    return Some((file, v, seg.virtual_start + rel));
                }
                return Some((file, seg.virtual_start, seg.virtual_end));
            }
        }
        Some((file, v, v))
    }
}

/// Build virtual TypeScript documents from a `.sail` source.
///
/// Returns `None` when the source has no TS regions.
pub fn build_virtual_ts(sail_source: &str) -> Option<VirtualTs> {
    let mut controller_text = String::new();
    let mut view_text = String::new();
    let mut segments: Vec<MapSegment> = Vec::new();
    let mut regions: Vec<TsRegion> = Vec::new();

    controller_text.push_str("// AUTO-GENERATED by sail-lsp virtual_ts (controller)\n");
    view_text.push_str("// AUTO-GENERATED by sail-lsp virtual_ts (view)\n");

    // ── Controller file ────────────────────────────────────────────────────
    let ctrl = find_controller_raw(sail_source);
    let mut controller_sail_span = None;
    let mut has_controller = false;

    if let Some((sail_start, sail_end, raw_body)) = ctrl {
        has_controller = true;
        controller_sail_span = Some(Span::new(sail_start, sail_end));
        regions.push(TsRegion {
            start: sail_start,
            end: sail_end,
            kind: RegionKind::Controller,
        });

        let wrapped = wrap_controller_body(raw_body);
        let prefix_len = wrapped.len().saturating_sub(raw_body.len());
        if prefix_len > 0 {
            controller_text.push_str(&wrapped[..prefix_len]);
        }
        let body_v_start = controller_text.len();
        controller_text.push_str(raw_body);
        if !raw_body.ends_with('\n') {
            controller_text.push('\n');
        }
        segments.push(MapSegment {
            file: VirtualFile::Controller,
            virtual_start: body_v_start,
            virtual_end: body_v_start + raw_body.len(),
            sail_start,
            sail_end,
            one_to_one: true,
        });
    }

    // ── View: scope types ──────────────────────────────────────────────────
    let scope = find_scope_raw(sail_source);
    let mut scope_sail_span = None;
    if let Some((sail_start, sail_end, raw_body)) = scope {
        scope_sail_span = Some(Span::new(sail_start, sail_end));
        regions.push(TsRegion {
            start: sail_start,
            end: sail_end,
            kind: RegionKind::Scope,
        });
        view_text.push_str("\n// ── <scope> type aliases ───────────────────────────────────\n");
        for (line_start_in_body, line) in iter_lines_with_offsets(raw_body) {
            let trimmed = line.trim_start();
            if trimmed.starts_with("type ")
                || trimmed.starts_with("export type ")
                || (trimmed.starts_with("export ") && trimmed.contains(" type "))
            {
                let v_start = view_text.len();
                view_text.push_str(line);
                if !line.ends_with('\n') {
                    view_text.push('\n');
                }
                let content_len = line.trim_end_matches(['\r', '\n']).len();
                let sail_line_start = sail_start + line_start_in_body;
                segments.push(MapSegment {
                    file: VirtualFile::View,
                    virtual_start: v_start,
                    virtual_end: v_start + content_len,
                    sail_start: sail_line_start,
                    sail_end: sail_line_start + content_len,
                    one_to_one: true,
                });
            }
        }
    }

    // ── View: scope recovery ───────────────────────────────────────────────
    // Import the controller module under a relative name the session rewrites
    // to the real controller virtual file when materializing on disk.
    view_text.push_str("\n// ── typed scope recovery ────────────────────────────────────\n");
    if has_controller {
        view_text.push_str("import __sail_authored from \"./__ctrl__.ts\";\n");
        view_text.push_str(
            "type __AuthScope = typeof __sail_authored extends { __def: infer D } \
             ? D extends (...a: never[]) => infer R ? R : object \
             : object;\n\
             const scope: __AuthScope = null! as __AuthScope;\n\
             export type Scope = __AuthScope;\n\
             void scope;\n\
             void __sail_authored;\n",
        );
    } else {
        view_text.push_str(
            "const scope: Record<string, never> = {};\n\
             export type Scope = typeof scope;\n",
        );
    }

    // ── View: expression embeddings ────────────────────────────────────────
    let (stripped, _) = extract_controller_block(sail_source);
    let exprs = collect_template_exprs(sail_source, &stripped);
    if !exprs.is_empty() {
        view_text.push_str("\n// ── template expressions ────────────────────────────────────\n");
        for emb in &exprs {
            regions.push(TsRegion {
                start: emb.sail_start,
                end: emb.sail_end,
                kind: RegionKind::Expression,
            });
            let rewritten = rewrite_with_prefix(&emb.source, &[], "scope.");
            view_text.push_str("void (() => (");
            let v_start = view_text.len();
            view_text.push_str(&rewritten);
            let v_end = view_text.len();
            view_text.push_str("));\n");
            segments.push(MapSegment {
                file: VirtualFile::View,
                virtual_start: v_start,
                virtual_end: v_end,
                sail_start: emb.sail_start,
                sail_end: emb.sail_end,
                one_to_one: rewritten.len() == emb.source.len() && rewritten == emb.source,
            });
        }
    }

    regions.sort_by_key(|r| r.start);
    segments.sort_by_key(|s| (s.file as u8, s.virtual_start));

    if regions.is_empty() {
        return None;
    }

    Some(VirtualTs {
        controller_text,
        view_text,
        segments,
        regions,
        controller_sail_span,
        scope_sail_span,
    })
}

// ── collection helpers ─────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct ExprEmbed {
    source: String,
    sail_start: usize,
    sail_end: usize,
}

fn collect_template_exprs(original: &str, stripped: &str) -> Vec<ExprEmbed> {
    let mut out = scan_brace_exprs(original);
    if out.is_empty() {
        if let Ok(program) = parse(stripped) {
            let mut used: Vec<(usize, usize)> = Vec::new();
            let mut exprs: Vec<Expr> = Vec::new();
            let _ = sail_compiler::check::visit_exprs(&program, &mut |e| {
                exprs.push(e.clone());
                Ok(())
            });
            for e in exprs {
                if e.source.trim().is_empty() {
                    continue;
                }
                if let Some((s, end)) = locate_expr_in_original(original, &e, &used) {
                    used.push((s, end));
                    out.push(ExprEmbed {
                        source: e.source.clone(),
                        sail_start: s,
                        sail_end: end,
                    });
                }
            }
        }
    }
    out.sort_by_key(|e| (e.sail_start, e.sail_end));
    out.dedup_by_key(|e| (e.sail_start, e.sail_end));
    out
}

fn locate_expr_in_original(
    original: &str,
    expr: &Expr,
    used: &[(usize, usize)],
) -> Option<(usize, usize)> {
    let src = expr.source.as_str();
    if expr.span.end <= original.len()
        && expr.span.start < expr.span.end
        && original[expr.span.start..expr.span.end].trim() == src.trim()
    {
        let slice = &original[expr.span.start..expr.span.end];
        let trim_start = slice.len() - slice.trim_start().len();
        let trim_end = slice.len() - slice.trim_end().len();
        let s = expr.span.start + trim_start;
        let e = expr.span.end - trim_end;
        if !used.iter().any(|&(us, ue)| us == s && ue == e) {
            return Some((s, e));
        }
    }
    let ctrl = find_controller_raw(original).map(|(s, e, _)| (s, e));
    let scope = find_scope_raw(original).map(|(s, e, _)| (s, e));
    let mut search_from = 0usize;
    while let Some(rel) = original[search_from..].find(src) {
        let abs = search_from + rel;
        search_from = abs + src.len().max(1);
        if ctrl.is_some_and(|(s, e)| abs >= s && abs < e) {
            continue;
        }
        if scope.is_some_and(|(s, e)| abs >= s && abs < e) {
            continue;
        }
        let candidate = (abs, abs + src.len());
        if used
            .iter()
            .any(|&(us, ue)| us == candidate.0 && ue == candidate.1)
        {
            continue;
        }
        if let Some(before) = original[..abs].rfind('{') {
            let between = &original[before + 1..abs];
            if between.chars().all(|c| c.is_whitespace()) {
                return Some(candidate);
            }
        }
        return Some(candidate);
    }
    None
}

fn scan_brace_exprs(original: &str) -> Vec<ExprEmbed> {
    let mut out = Vec::new();
    let ctrl = find_controller_raw(original).map(|(s, e, _)| (s, e));
    let scope = find_scope_raw(original).map(|(s, e, _)| (s, e));
    let style = find_tag_body(original, "style");
    let bytes = original.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] != b'{' {
            i += 1;
            continue;
        }
        if let Some(end) = find_matching_brace(original, i) {
            let raw_inner = &original[i + 1..end];
            let inner = raw_inner.trim();
            let lead = raw_inner.len() - raw_inner.trim_start().len();
            let inner_start = i + 1 + lead;
            let inner_end = inner_start + inner.len();
            let in_ctrl = ctrl.is_some_and(|(s, e)| i >= s && i < e);
            let in_scope = scope.is_some_and(|(s, e)| i >= s && i < e);
            let in_style = style.is_some_and(|(s, e, _)| i >= s && i < e);
            if !inner.is_empty() && !in_ctrl && !in_scope && !in_style {
                if !(inner.contains(';') && inner.contains(':') && !inner.contains("=>")) {
                    out.push(ExprEmbed {
                        source: inner.to_string(),
                        sail_start: inner_start,
                        sail_end: inner_end,
                    });
                }
            }
            i = end + 1;
        } else {
            i += 1;
        }
    }
    out
}

fn find_matching_brace(src: &str, open: usize) -> Option<usize> {
    let bytes = src.as_bytes();
    if open >= bytes.len() || bytes[open] != b'{' {
        return None;
    }
    let mut depth = 0i32;
    let mut in_str: Option<u8> = None;
    let mut i = open;
    while i < bytes.len() {
        let c = bytes[i];
        if let Some(q) = in_str {
            if c == b'\\' && i + 1 < bytes.len() {
                i += 2;
                continue;
            }
            if c == q {
                in_str = None;
            }
            i += 1;
            continue;
        }
        match c {
            b'"' | b'\'' | b'`' => {
                in_str = Some(c);
                i += 1;
            }
            b'{' => {
                depth += 1;
                i += 1;
            }
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
                i += 1;
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'/' => {
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            _ => i += 1,
        }
    }
    None
}

pub fn find_controller_raw(source: &str) -> Option<(usize, usize, &str)> {
    find_tag_body(source, "controller")
}

pub fn find_scope_raw(source: &str) -> Option<(usize, usize, &str)> {
    find_tag_body(source, "scope")
}

fn find_tag_body<'a>(source: &'a str, tag: &str) -> Option<(usize, usize, &'a str)> {
    let open_pat = format!("<{tag}");
    let close_pat = format!("</{tag}>");
    let start = find_outside_comments(source, &open_pat)?;
    let open_rel = source[start..].find('>')?;
    let body_start_raw = start + open_rel + 1;
    let rel = source[body_start_raw..].find(&close_pat)?;
    let body_end_raw = body_start_raw + rel;
    let body = source[body_start_raw..body_end_raw].trim();
    if body.is_empty() {
        return Some((body_start_raw, body_end_raw, ""));
    }
    let raw_interior = &source[body_start_raw..body_end_raw];
    let trim_start = raw_interior.find(body).unwrap_or(0);
    let sail_start = body_start_raw + trim_start;
    let sail_end = sail_start + body.len();
    Some((sail_start, sail_end, body))
}

fn find_outside_comments(source: &str, needle: &str) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if source[i..].starts_with("<!--") {
            match source[i + 4..].find("-->") {
                Some(rel) => {
                    i = i + 4 + rel + 3;
                    continue;
                }
                None => return None,
            }
        }
        if source[i..].starts_with(needle) {
            let after = i + needle.len();
            let next_ok = after >= bytes.len()
                || matches!(bytes[after], b'>' | b'/' | b' ' | b'\t' | b'\n' | b'\r');
            if next_ok {
                return Some(i);
            }
        }
        let mut j = i + 1;
        while j < bytes.len() && (bytes[j] & 0xC0) == 0x80 {
            j += 1;
        }
        i = j;
    }
    None
}

fn iter_lines_with_offsets(body: &str) -> impl Iterator<Item = (usize, &str)> {
    let mut offset = 0usize;
    body.split_inclusive('\n').map(move |line| {
        let start = offset;
        offset += line.len();
        (start, line)
    })
}

pub fn offset_to_lsp_position(text: &str, offset: usize) -> (u32, u32) {
    let offset = offset.min(text.len());
    let mut line = 0u32;
    let mut col_utf16 = 0u32;
    for (i, ch) in text.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col_utf16 = 0;
        } else {
            col_utf16 += ch.len_utf16() as u32;
        }
    }
    (line, col_utf16)
}

pub fn lsp_position_to_offset(text: &str, line: u32, character: u32) -> usize {
    let mut cur_line = 0u32;
    let mut idx = 0usize;
    for (i, row) in text.split_inclusive('\n').enumerate() {
        if cur_line == line {
            let content = row.strip_suffix('\n').unwrap_or(row);
            let mut units = 0u32;
            for (b, ch) in content.char_indices() {
                if units >= character {
                    return idx + b;
                }
                units += ch.len_utf16() as u32;
            }
            return idx + content.len();
        }
        idx += row.len();
        cur_line = (i + 1) as u32;
        if cur_line > line {
            break;
        }
    }
    text.len()
}

#[allow(dead_code)]
pub fn sail_line_col(source: &str, offset: usize) -> (usize, usize) {
    line_col(source, offset)
}

#[cfg(test)]
mod tests {
    use super::*;

    const COUNTER: &str = r#"<controller>
import { controller, signal } from '@ship/sail-runtime'

export default controller((ctx) => ({ count: signal(0) }))
</controller>

<div>
  <p>{count()}</p>
  <button on:click={() => count.set(count() + 1)}>inc</button>
  <span title={count()}>x</span>
</div>
"#;

    const SCOPE_FILE: &str = r#"<scope>
type Count = number
props: { n: Count = 0 }
</scope>
<div>{n}</div>
"#;

    const CTRL_TYPO: &str = r#"<controller>
import { controller, signal } from '@ship/sail-runtime'
export default controller((ctx) => {
  const c = signal(0)
  return { count: c, bad: c.nope }
})
</controller>
<div>{count()}</div>
"#;

    const EXPR_TYPO: &str = r#"<controller>
import { controller, signal } from '@ship/sail-runtime'
export default controller((ctx) => ({ count: signal(0) }))
</controller>
<div>{countt()}</div>
"#;

    #[test]
    fn builds_controller_and_expression_regions() {
        let v = build_virtual_ts(COUNTER).expect("virtual");
        assert!(
            v.regions.iter().any(|r| r.kind == RegionKind::Controller),
            "controller region"
        );
        assert!(
            v.regions
                .iter()
                .filter(|r| r.kind == RegionKind::Expression)
                .count()
                >= 3,
            "text + on:click + title expressions, got {:?}",
            v.regions
        );
        assert!(v.controller_text.contains("export default controller"));
        assert!(
            v.view_text.contains("scope.count"),
            "expressions rewritten: {}",
            v.view_text
        );
        assert!(v.view_text.contains("import __sail_authored"));
    }

    #[test]
    fn controller_offset_round_trips_one_to_one() {
        let v = build_virtual_ts(COUNTER).expect("virtual");
        let (sail_start, sail_end, body) = find_controller_raw(COUNTER).unwrap();
        for (i, _) in body.char_indices() {
            let sail = sail_start + i;
            let (file, virt) = v.sail_to_virtual(sail).expect("to virtual");
            assert_eq!(file, VirtualFile::Controller);
            let back = v.virtual_to_sail(file, virt).expect("to sail");
            assert_eq!(back, sail, "round-trip at sail {sail}");
        }
        let (file, vs) = v.sail_to_virtual(sail_start).unwrap();
        let ve = vs + (sail_end - sail_start);
        assert_eq!(&v.text(file)[vs..ve], body);
    }

    #[test]
    fn expression_regions_cover_interpolation_and_handlers() {
        let v = build_virtual_ts(COUNTER).expect("virtual");
        let text_at = COUNTER.find("{count()}").unwrap() + 1;
        assert_eq!(v.region_at(text_at), Some(RegionKind::Expression));
        let (file, virt) = v.sail_to_virtual(text_at).expect("map text expr");
        assert_eq!(file, VirtualFile::View);
        let back = v.virtual_to_sail(file, virt).expect("back");
        assert!(v.is_ts_region(back));

        let click = COUNTER.find("() => count.set").unwrap();
        assert_eq!(v.region_at(click), Some(RegionKind::Expression));

        let title = COUNTER.find("title={count()}").unwrap() + "title={".len();
        assert_eq!(v.region_at(title), Some(RegionKind::Expression));
    }

    #[test]
    fn scope_type_alias_is_mapped() {
        let v = build_virtual_ts(SCOPE_FILE).expect("virtual");
        assert!(
            v.regions.iter().any(|r| r.kind == RegionKind::Scope),
            "scope region"
        );
        let type_at = SCOPE_FILE.find("type Count").unwrap();
        assert_eq!(v.region_at(type_at), Some(RegionKind::Scope));
        let (file, virt) = v.sail_to_virtual(type_at).expect("map type");
        assert_eq!(file, VirtualFile::View);
        assert!(
            v.view_text[virt..].starts_with("type Count"),
            "virtual at {virt}: {}",
            &v.view_text[virt..virt + 20.min(v.view_text.len() - virt)]
        );
        assert_eq!(v.virtual_to_sail(file, virt), Some(type_at));
    }

    #[test]
    fn map_virtual_range_for_expression_typo_span() {
        let v = build_virtual_ts(EXPR_TYPO).expect("virtual");
        let virt_idx = v.view_text.find("countt").expect("countt in virtual");
        let (s, e) = v
            .map_virtual_range(VirtualFile::View, virt_idx, virt_idx + "countt".len())
            .expect("range");
        let sail_slice = &EXPR_TYPO[s..e.min(EXPR_TYPO.len())];
        assert!(
            sail_slice.contains("countt"),
            "mapped sail [{s},{e}) = {sail_slice:?}"
        );
    }

    #[test]
    fn controller_typo_region_is_controller() {
        let v = build_virtual_ts(CTRL_TYPO).expect("virtual");
        let nope = CTRL_TYPO.find("nope").unwrap();
        assert_eq!(v.region_at(nope), Some(RegionKind::Controller));
        let (file, virt) = v.sail_to_virtual(nope).unwrap();
        assert_eq!(file, VirtualFile::Controller);
        assert_eq!(&v.controller_text[virt..virt + 4], "nope");
        assert_eq!(v.virtual_to_sail(file, virt), Some(nope));
    }

    #[test]
    fn empty_markup_only_returns_none() {
        assert!(build_virtual_ts("<div>hello</div>").is_none());
    }

    #[test]
    fn lsp_position_round_trip_on_virtual() {
        let v = build_virtual_ts(COUNTER).expect("virtual");
        let offset = v.controller_text.find("signal").unwrap();
        let (line, col) = offset_to_lsp_position(&v.controller_text, offset);
        let back = lsp_position_to_offset(&v.controller_text, line, col);
        assert_eq!(back, offset);
    }

    #[test]
    fn is_ts_region_false_outside() {
        let v = build_virtual_ts(COUNTER).expect("virtual");
        let div = COUNTER.find("<div>").unwrap();
        assert!(!v.is_ts_region(div));
        assert!(v.region_at(div).is_none());
    }
}
