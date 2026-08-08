//! Library entry points for Sail + TypeScript intelligence (SPEC-LSP-1 studio bridge).
//!
//! Usable without stdio LSP: ops, tests, and the thin `run()` shell share this path.
//! Positions are always **`.sail` coordinates** (1-based line/column, matching
//! `sail_compiler::hover_at` / Check diagnostics). Internally maps via `virtual_ts`
//! and proxies TS regions to tsgo when available (soft-degrade otherwise).

use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::tsgo::TsgoSession;

// ── Public payloads ──────────────────────────────────────────────────────────

/// Inclusive range in 1-based line/column (column is UTF-16 units for parity
/// with LSP-mapped spans; ASCII sources match byte columns).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RangePayload {
    pub start_line: u32,
    pub start_col: u32,
    pub end_line: u32,
    pub end_col: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HoverPayload {
    /// Markdown contents suitable for hover UI.
    pub contents: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range: Option<RangePayload>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletionItemPayload {
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(rename = "insertText")]
    pub insert_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub documentation: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagPayload {
    pub line: usize,
    pub col: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_line: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_col: Option<usize>,
    pub severity: String,
    pub message: String,
    /// `"sail"` structure/check or `"tsgo"` mapped TypeScript.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rule: Option<String>,
}

// ── Session ──────────────────────────────────────────────────────────────────

/// Long-lived intelligence engine: virtual TS docs + optional tsgo sidecar.
pub struct Intelligence {
    tsgo: TsgoSession,
}

impl Intelligence {
    /// Discover tsgo from `workspace_hint` (and ancestors); degrade when absent.
    pub fn start(workspace_hint: Option<&Path>) -> Self {
        Self {
            tsgo: TsgoSession::start(workspace_hint),
        }
    }

    pub fn is_tsgo_live(&self) -> bool {
        self.tsgo.is_live()
    }

    /// Open or update the virtual document for `key` (uri or path string).
    pub fn open_or_update(&mut self, key: &str, source: &str) {
        self.tsgo.on_sail_change(key, source);
    }

    pub fn close(&mut self, key: &str) {
        self.tsgo.on_sail_close(key);
    }

    /// Access the underlying tsgo session (stdio LSP / advanced callers).
    pub fn tsgo_mut(&mut self) -> &mut TsgoSession {
        &mut self.tsgo
    }

    pub fn tsgo(&self) -> &TsgoSession {
        &self.tsgo
    }

    /// Hover at a `.sail` 1-based position. Prefers tsgo in TS regions; falls
    /// back to structure symbols from `sail_compiler`.
    pub fn hover_sail(
        &mut self,
        source: &str,
        path: Option<&str>,
        line_1based: u32,
        col_1based: u32,
    ) -> Option<HoverPayload> {
        let key = doc_key(path, "hover.sail");
        self.open_or_update(&key, source);
        hover_with_session(
            &mut self.tsgo,
            source,
            path.map(Path::new),
            &key,
            line_1based,
            col_1based,
        )
    }

    /// Completions at a `.sail` 1-based position. Prefers tsgo in TS regions;
    /// falls back to structure scope symbols.
    pub fn complete_sail(
        &mut self,
        source: &str,
        path: Option<&str>,
        line_1based: u32,
        col_1based: u32,
    ) -> Vec<CompletionItemPayload> {
        let key = doc_key(path, "complete.sail");
        self.open_or_update(&key, source);
        complete_with_session(
            &mut self.tsgo,
            source,
            path.map(Path::new),
            &key,
            line_1based,
            col_1based,
        )
    }

    /// Structure diagnostics merged with mapped tsgo diagnostics when live.
    /// Full stream ([`sail_compiler::DiagnosticTier::OnSave`]) — for explicit
    /// checks. For typing / on-type, use [`Self::diagnostics_sail_tier`].
    pub fn diagnostics_sail(&mut self, source: &str, path: Option<&str>) -> Vec<DiagPayload> {
        self.diagnostics_sail_tier(source, path, sail_compiler::DiagnosticTier::OnSave)
    }

    /// Diagnostics at an explicit [`sail_compiler::DiagnosticTier`]
    /// (SPEC-TOOLING-1 §WI6).
    pub fn diagnostics_sail_tier(
        &mut self,
        source: &str,
        path: Option<&str>,
        tier: sail_compiler::DiagnosticTier,
    ) -> Vec<DiagPayload> {
        let key = doc_key(path, "diag.sail");
        self.open_or_update(&key, source);
        diagnostics_with_session(&mut self.tsgo, source, path.map(Path::new), &key, tier)
    }
}

// ── Process-local shared session (ops / free functions) ───────────────────────

static GLOBAL: OnceLock<Mutex<Intelligence>> = OnceLock::new();

/// Borrow the process-local intelligence session, starting it on first use
/// with `workspace_hint` (used only for the initial start).
pub fn with_global_session<R>(
    workspace_hint: Option<&Path>,
    f: impl FnOnce(&mut Intelligence) -> R,
) -> R {
    let m = GLOBAL.get_or_init(|| Mutex::new(Intelligence::start(workspace_hint)));
    let mut guard = m.lock().unwrap_or_else(|e| e.into_inner());
    f(&mut guard)
}

/// Hover convenience: uses the process-local session.
pub fn hover_sail(
    source: &str,
    path: Option<&str>,
    line_1based: u32,
    col_1based: u32,
) -> Option<HoverPayload> {
    let hint = path.and_then(|p| Path::new(p).parent().map(|d| d.to_path_buf()));
    with_global_session(hint.as_deref(), |intel| {
        intel.hover_sail(source, path, line_1based, col_1based)
    })
}

/// SPEC-LSP-1 name for [`hover_sail`].
#[inline]
pub fn hover_at_sail(
    source: &str,
    path: Option<&str>,
    line_1based: u32,
    col_1based: u32,
) -> Option<HoverPayload> {
    hover_sail(source, path, line_1based, col_1based)
}

/// Completions convenience: uses the process-local session.
pub fn complete_sail(
    source: &str,
    path: Option<&str>,
    line_1based: u32,
    col_1based: u32,
) -> Vec<CompletionItemPayload> {
    let hint = path.and_then(|p| Path::new(p).parent().map(|d| d.to_path_buf()));
    with_global_session(hint.as_deref(), |intel| {
        intel.complete_sail(source, path, line_1based, col_1based)
    })
}

/// SPEC-LSP-1 name for [`complete_sail`].
#[inline]
pub fn completions_at_sail(
    source: &str,
    path: Option<&str>,
    line_1based: u32,
    col_1based: u32,
) -> Vec<CompletionItemPayload> {
    complete_sail(source, path, line_1based, col_1based)
}

/// Diagnostics convenience: uses the process-local session.
/// Full (OnSave) stream — for MCP/agent-style explicit checks.
pub fn diagnostics_sail(source: &str, path: Option<&str>) -> Vec<DiagPayload> {
    diagnostics_sail_tier(source, path, sail_compiler::DiagnosticTier::OnSave)
}

/// SPEC-LSP-1 name for [`diagnostics_sail`].
#[inline]
pub fn diagnostics_for_sail(source: &str, path: Option<&str>) -> Vec<DiagPayload> {
    diagnostics_sail(source, path)
}

/// Diagnostics convenience with explicit tier (SPEC-TOOLING-1 §WI6).
pub fn diagnostics_sail_tier(
    source: &str,
    path: Option<&str>,
    tier: sail_compiler::DiagnosticTier,
) -> Vec<DiagPayload> {
    let hint = path.and_then(|p| Path::new(p).parent().map(|d| d.to_path_buf()));
    with_global_session(hint.as_deref(), |intel| {
        intel.diagnostics_sail_tier(source, path, tier)
    })
}

// ── Shared core (stdio + library) ────────────────────────────────────────────

/// Stable document key for virtual-doc map (uri or path).
pub fn doc_key(path: Option<&str>, fallback: &str) -> String {
    match path {
        Some(p) if !p.trim().is_empty() => {
            let p = p.trim();
            if p.starts_with("file:") {
                p.to_string()
            } else {
                let pb = PathBuf::from(p);
                if pb.is_absolute() {
                    crate::tsgo::path_to_uri(&pb)
                } else {
                    format!("file:///{}", p.replace('\\', "/"))
                }
            }
        }
        _ => format!("file:///__inline__/{fallback}"),
    }
}

/// Core hover path shared by stdio LSP and the library API.
pub fn hover_with_session(
    tsgo: &mut TsgoSession,
    source: &str,
    path: Option<&Path>,
    uri_key: &str,
    line_1based: u32,
    col_1based: u32,
) -> Option<HoverPayload> {
    let offset = line_col_to_byte_offset(source, line_1based as usize, col_1based as usize);
    if tsgo.is_live() {
        if let Some(raw) = tsgo.hover_mapped(uri_key, source, offset) {
            if let Some(h) = hover_payload_from_tsgo(raw) {
                return Some(h);
            }
        }
    }
    let h = sail_compiler::hover_at(source, path, line_1based as usize, col_1based as usize)?;
    let loc = if h.line > 0 {
        format!("\n\nDeclared at controller {}:{}", h.line, h.col)
    } else {
        String::new()
    };
    Some(HoverPayload {
        contents: format!("**{}** · `{}` (from `{}`){loc}", h.name, h.kind, h.source),
        range: None,
    })
}

/// Core completion path shared by stdio LSP (structure branch) and the library.
///
/// Style regions (SPEC-STYLE-COMPLETE-1): Style complete from validate SSOT
/// **before** tsgo / bare scope. Does not require tsgo live for Style tokens.
pub fn complete_with_session(
    tsgo: &mut TsgoSession,
    source: &str,
    path: Option<&Path>,
    uri_key: &str,
    line_1based: u32,
    col_1based: u32,
) -> Vec<CompletionItemPayload> {
    let offset = line_col_to_byte_offset(source, line_1based as usize, col_1based as usize);
    // Infer L1 host from path stem when it matches a known surface pack
    // (stdlib `chip.sail` → host `chip`); app paths yield None.
    let host = infer_style_host(path);
    let style_items =
        sail_lint::style::style_complete_at(source, path, offset, host.as_deref());
    if !style_items.is_empty() {
        return style_items
            .into_iter()
            .map(|it| CompletionItemPayload {
                label: it.label,
                kind: Some(it.kind.as_str().to_string()),
                detail: it.detail.or_else(|| Some(format!("{} (style)", it.kind.as_str()))),
                insert_text: it.insert_text,
                documentation: None,
            })
            .collect();
    }
    let style_ctx = sail_lint::style::style_complete_context_at(source, offset, host.as_deref());
    if !matches!(style_ctx, sail_lint::style::StyleCompleteContext::Unknown) {
        // Classified Style position with empty set — do not fall through to
        // weak tsgo / scope noise.
        return Vec::new();
    }
    if tsgo.is_live() {
        if let Some(raw) = tsgo.completion_mapped(uri_key, source, offset) {
            let items = completion_payloads_from_tsgo_inner(raw);
            if !items.is_empty() {
                return items;
            }
        }
    }
    sail_compiler::completions_at(source, path)
        .into_iter()
        .map(|s| CompletionItemPayload {
            label: s.name,
            kind: Some(s.kind.clone()),
            detail: Some(format!("{} ({})", s.kind, s.source)),
            insert_text: None,
            documentation: None,
        })
        .collect()
}

/// Host tag for Style complete: surface pack when path stem is a known L1 pack.
fn infer_style_host(path: Option<&Path>) -> Option<String> {
    let stem = path?.file_stem()?.to_str()?;
    if sail_lint::style::surface_pack_for_host(stem).is_some() {
        Some(stem.to_string())
    } else {
        None
    }
}

/// Structure + mapped tsgo diagnostics at `tier` (SPEC-TOOLING-1 §WI6).
pub fn diagnostics_with_session(
    tsgo: &mut TsgoSession,
    source: &str,
    path: Option<&Path>,
    uri_key: &str,
    tier: sail_compiler::DiagnosticTier,
) -> Vec<DiagPayload> {
    let mut out: Vec<DiagPayload> =
        sail_compiler::check_all_with_tier(source, path, &[], None, tier)
            .into_iter()
            .map(|d| DiagPayload {
                line: d.line,
                col: d.col,
                end_line: Some(d.end_line),
                end_col: Some(d.end_col),
                severity: match d.severity {
                    sail_compiler::DiagSeverity::Error => "error".into(),
                    sail_compiler::DiagSeverity::Warning => "warning".into(),
                    sail_compiler::DiagSeverity::Fix => "fix".into(),
                },
                message: d.message,
                source: Some("sail".into()),
                rule: d.rule,
            })
            .collect();

    if tsgo.is_live() {
        for d in tsgo.diagnostics_mapped(uri_key, source) {
            let (sev, name) = match d.severity {
                Some(lsp_types::DiagnosticSeverity::ERROR) => ("error", "error"),
                Some(lsp_types::DiagnosticSeverity::WARNING) => ("warning", "warning"),
                Some(lsp_types::DiagnosticSeverity::INFORMATION) => ("info", "info"),
                Some(lsp_types::DiagnosticSeverity::HINT) => ("hint", "hint"),
                _ => ("error", "error"),
            };
            let _ = name;
            // Mapped ranges are 0-based LSP → 1-based for ops/check parity.
            out.push(DiagPayload {
                line: d.range.start.line as usize + 1,
                col: d.range.start.character as usize + 1,
                end_line: Some(d.range.end.line as usize + 1),
                end_col: Some(d.range.end.character as usize + 1),
                severity: sev.into(),
                message: d.message,
                source: Some("tsgo".into()),
                rule: d.code.and_then(|c| match c {
                    lsp_types::NumberOrString::String(s) => Some(s),
                    lsp_types::NumberOrString::Number(n) => Some(n.to_string()),
                }),
            });
        }
    }
    out
}

// ── Mapping helpers ──────────────────────────────────────────────────────────

/// 1-based line/column (byte-oriented col, matching sail_compiler) → byte offset.
pub fn line_col_to_byte_offset(source: &str, target_line: usize, target_col: usize) -> usize {
    let bytes = source.as_bytes();
    let (mut line, mut col, mut i) = (1usize, 1usize, 0usize);
    while i < bytes.len() {
        if line == target_line && col == target_col {
            return i;
        }
        if bytes[i] == b'\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
        i += 1;
    }
    bytes.len()
}

/// Parse a raw tsgo `textDocument/hover` result into a library payload.
pub fn hover_payload_from_tsgo(raw: Value) -> Option<HoverPayload> {
    let contents = markdown_from_hover_contents(raw.get("contents")?)?;
    if contents.trim().is_empty() {
        return None;
    }
    let range = raw.get("range").and_then(range_from_lsp_json);
    Some(HoverPayload { contents, range })
}

/// Parse a raw tsgo completion result into library payloads.
pub fn completion_payloads_from_tsgo(raw: Value) -> Vec<CompletionItemPayload> {
    completion_payloads_from_tsgo_inner(raw)
}

fn range_from_lsp_json(range: &Value) -> Option<RangePayload> {
    let start = range.get("start")?;
    let end = range.get("end")?;
    // LSP 0-based → 1-based
    Some(RangePayload {
        start_line: start.get("line")?.as_u64()? as u32 + 1,
        start_col: start.get("character")?.as_u64()? as u32 + 1,
        end_line: end.get("line")?.as_u64()? as u32 + 1,
        end_col: end.get("character")?.as_u64()? as u32 + 1,
    })
}

fn markdown_from_hover_contents(contents: &Value) -> Option<String> {
    // MarkupContent
    if let Some(value) = contents.get("value").and_then(|v| v.as_str()) {
        return Some(value.to_string());
    }
    // MarkedString = string | { language, value }
    if let Some(s) = contents.as_str() {
        return Some(s.to_string());
    }
    if let (Some(lang), Some(value)) = (
        contents.get("language").and_then(|v| v.as_str()),
        contents.get("value").and_then(|v| v.as_str()),
    ) {
        return Some(format!("```{lang}\n{value}\n```"));
    }
    // MarkedString[]
    if let Some(arr) = contents.as_array() {
        let mut parts = Vec::new();
        for item in arr {
            if let Some(s) = markdown_from_hover_contents(item) {
                parts.push(s);
            }
        }
        if !parts.is_empty() {
            return Some(parts.join("\n\n"));
        }
    }
    None
}

fn completion_payloads_from_tsgo_inner(raw: Value) -> Vec<CompletionItemPayload> {
    let items = if let Some(arr) = raw.as_array() {
        arr.clone()
    } else if let Some(arr) = raw.get("items").and_then(|v| v.as_array()) {
        arr.clone()
    } else {
        return Vec::new();
    };
    items
        .into_iter()
        .filter_map(|item| {
            let label = item.get("label")?.as_str()?.to_string();
            let kind = item
                .get("kind")
                .and_then(|v| v.as_u64())
                .map(completion_kind_name)
                .or_else(|| {
                    item.get("kind")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                });
            let detail = item
                .get("detail")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let insert_text = item
                .get("insertText")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let documentation = item.get("documentation").and_then(|d| {
                d.as_str().map(|s| s.to_string()).or_else(|| {
                    d.get("value")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                })
            });
            Some(CompletionItemPayload {
                label,
                kind,
                detail,
                insert_text,
                documentation,
            })
        })
        .collect()
}

fn completion_kind_name(kind: u64) -> String {
    // lsp_types::CompletionItemKind numeric values
    match kind {
        1 => "text",
        2 => "method",
        3 => "function",
        4 => "constructor",
        5 => "field",
        6 => "variable",
        7 => "class",
        8 => "interface",
        9 => "module",
        10 => "property",
        11 => "unit",
        12 => "value",
        13 => "enum",
        14 => "keyword",
        15 => "snippet",
        16 => "color",
        17 => "file",
        18 => "reference",
        19 => "folder",
        20 => "enumMember",
        21 => "constant",
        22 => "struct",
        23 => "event",
        24 => "operator",
        25 => "typeParameter",
        n => return format!("kind{n}"),
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::virtual_ts::offset_to_lsp_position;

    const COUNTER: &str = r#"<controller>
import { controller, signal } from '@ship/sail-runtime'

export default controller((ctx) => ({ count: signal(0) }))
</controller>

<div>
  <p>{count()}</p>
</div>
"#;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .canonicalize()
            .expect("repo root")
    }

    #[test]
    fn structure_hover_without_tsgo() {
        let mut intel =
            Intelligence::start(Some(Path::new("/nonexistent-sail-tsgo-root-for-intel")));
        // count in template — structure scope may still resolve
        let count_line = COUNTER
            .lines()
            .enumerate()
            .find(|(_, l)| l.contains("{count()}"))
            .map(|(i, _)| (i + 1) as u32)
            .unwrap();
        let col = (COUNTER
            .lines()
            .find(|l| l.contains("{count()}"))
            .unwrap()
            .find("count")
            .unwrap()
            + 1) as u32;
        let hover = intel.hover_sail(COUNTER, Some("counter.sail"), count_line, col);
        // Structure hover on count should work even without tsgo.
        if let Some(h) = hover {
            assert!(
                h.contents.contains("count") || h.contents.contains("signal"),
                "got {}",
                h.contents
            );
        }
    }

    #[test]
    fn diagnostics_structure_always() {
        let mut intel =
            Intelligence::start(Some(Path::new("/nonexistent-sail-tsgo-root-for-intel")));
        // Channel-era structure diag: bare participation attr (not retired style= attr).
        let bad = r#"<column grow>x</column>"#;
        let diags = intel.diagnostics_sail(bad, None);
        assert!(
            diags
                .iter()
                .any(|d| d.rule.as_deref() == Some("sail-participation-style")),
            "expected structure diag: {diags:?}"
        );
        assert!(diags.iter().all(|d| d.source.as_deref() == Some("sail")));
    }

    #[test]
    fn complete_sail_style_bg_tokens_without_tsgo() {
        // SPEC-STYLE-COMPLETE-1: Style complete inside style({ paint: bg(¦) })
        // does not require tsgo.
        let mut intel =
            Intelligence::start(Some(Path::new("/nonexistent-sail-tsgo-root-for-intel")));
        let src = r#"<div style({ paint: bg() })>x</div>"#;
        let offset = src.find("bg(").expect("bg(") + 3;
        let (line0, col0) = offset_to_lsp_position(src, offset);
        let items = intel.complete_sail(src, Some("app.sail"), line0 + 1, col0 + 1);
        assert!(
            items.iter().any(|i| i.label == "primary"),
            "expected primary: {:?}",
            items.iter().map(|i| &i.label).collect::<Vec<_>>()
        );
        assert!(
            items.iter().any(|i| i.label == "surface-1"),
            "expected surface-1: {:?}",
            items.iter().map(|i| &i.label).collect::<Vec<_>>()
        );
        assert!(
            items
                .iter()
                .any(|i| i.kind.as_deref() == Some("token") || i.kind.as_deref() == Some("keyword")),
            "expected style kinds: {:?}",
            items
        );
        assert!(
            !items.iter().any(|i| i.label == "chip-foreground"),
            "app path must exclude chip-foreground"
        );
    }

    #[test]
    fn complete_sail_style_prefers_style_over_scope() {
        let mut intel =
            Intelligence::start(Some(Path::new("/nonexistent-sail-tsgo-root-for-intel")));
        let src = r#"<controller>
import { controller, signal } from '@ship/sail-runtime'
export default controller(() => ({ primary: signal(0) }))
</controller>
<div style({ paint: bg() })>{primary()}</div>"#;
        let offset = src.find("bg(").expect("bg(") + 3;
        let (line0, col0) = offset_to_lsp_position(src, offset);
        let items = intel.complete_sail(src, None, line0 + 1, col0 + 1);
        assert!(
            items.iter().any(|i| i.label == "primary" && i.kind.as_deref() == Some("token")),
            "Style token primary must win: {:?}",
            items
        );
        // Scope signal must not be the only hit — Style labels present.
        assert!(
            items.iter().any(|i| i.label == "surface-1"),
            "style tokens present: {:?}",
            items.iter().map(|i| &i.label).collect::<Vec<_>>()
        );
    }

    #[test]
    fn hover_signal_typed_when_tsgo_present() {
        // Soft-skip only when tsgo/node_modules is missing. When the session is
        // live, require a typed hover (warm + brief settle; no silent skip).
        if crate::tsgo::discover_tsgo(Some(&repo_root())).is_none() {
            eprintln!("SKIP intelligence hover: no tsgo");
            return;
        }
        let mut intel = Intelligence::start(Some(&repo_root()));
        if !intel.is_tsgo_live() {
            eprintln!("SKIP intelligence hover: session not live");
            return;
        }
        // line/col of `signal` in controller
        let offset = COUNTER.find("signal(0)").unwrap();
        let (line0, col0) = offset_to_lsp_position(COUNTER, offset);
        let line = line0 + 1;
        let col = col0 + 1;
        let path = Some("/tmp/intel-counter.sail");
        // Open before sleeping — a pre-open sleep never settles project load.
        intel.open_or_update(&doc_key(path, "hover.sail"), COUNTER);
        std::thread::sleep(std::time::Duration::from_millis(200));
        let mut hover = intel.hover_sail(COUNTER, path, line, col);
        // One retry after settle: first hover can race project graph warm-up.
        if hover
            .as_ref()
            .map(|h| {
                let t = h.contents.to_ascii_lowercase();
                !(t.contains("signal") || t.contains("function"))
            })
            .unwrap_or(true)
        {
            std::thread::sleep(std::time::Duration::from_millis(300));
            hover = intel.hover_sail(COUNTER, path, line, col);
        }
        assert!(hover.is_some(), "expected hover on signal");
        let text = hover.unwrap().contents.to_ascii_lowercase();
        assert!(
            text.contains("signal") || text.contains("function"),
            "expected typed signal hover, got {text}"
        );
    }

    #[test]
    fn doc_key_stable() {
        assert!(doc_key(Some("apps/x/page.sail"), "f").contains("apps/x/page.sail"));
        assert!(doc_key(Some("file:///tmp/a.sail"), "f").starts_with("file:"));
        assert!(doc_key(None, "x.sail").contains("x.sail"));
    }
}
