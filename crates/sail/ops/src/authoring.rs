//! `SailAuthoring` RPCs — Surface, Check, Edit, LayoutCheck, ListFiles, Policy, Hover, Complete.

use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::{Value, json};

use crate::ctx::Ctx;
use crate::error::OpError;

/// Process-local sail-lsp intelligence session (virtual_ts + tsgo).
/// Shared across Hover / Complete / Check(includeTs) so Studio does not spawn
/// a new tsgo per request.
fn with_intelligence<R>(ctx: &Ctx, f: impl FnOnce(&mut sail_lsp::Intelligence) -> R) -> R {
    sail_lsp::with_global_session(Some(ctx.workspace()), f)
}

/// Topics for progressive surface disclosure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceTopic {
    Core,
    Layout,
    Style,
    Tokens,
    L1,
    ControlFlow,
}

impl SurfaceTopic {
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "core" => Some(Self::Core),
            "layout" => Some(Self::Layout),
            "style" => Some(Self::Style),
            "tokens" => Some(Self::Tokens),
            "l1" | "elements" => Some(Self::L1),
            "control-flow" | "control_flow" | "controlflow" => Some(Self::ControlFlow),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Core => "core",
            Self::Layout => "layout",
            Self::Style => "style",
            Self::Tokens => "tokens",
            Self::L1 => "l1",
            Self::ControlFlow => "control-flow",
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Core,
            Self::Layout,
            Self::Style,
            Self::Tokens,
            Self::L1,
            Self::ControlFlow,
        ]
    }
}

/// Render focused markdown for one surface topic (compiler SSOT).
pub fn surface_markdown(topic: SurfaceTopic) -> String {
    use std::fmt::Write as _;
    let mut md = String::new();
    match topic {
        SurfaceTopic::Core => {
            let _ = writeln!(md, "# Sail surface — core\n");
            let _ = writeln!(
                md,
                "Hard rules for agent authoring. Full tables: other `sail_surface` topics.\n"
            );
            let _ = writeln!(
                md,
                "## Framework stack (do not invent alternatives)\n\n\
                 1. Signals + stores (`defineStore`/`useStore`/`signal` from `@ship/sail-runtime`) — not module `let` + sync.\n\
                 2. Actions (`use:`) — public catalog is attach-anywhere capabilities; \
widget-private (tabs/menu/pagination/radioGroup/combobox) is closed inside L1 widgets. \
Author actions: `src/actions/` under floor ERROR (no style, no state primitives, no network, host-only). \
Controllers never `.style` / `createElement`.\n\
                 3. Compositions / L1–L2 first (tabs, tooltip, sail-tree, …).\n\
                 4. `prop:tree={{table}}` identity bind for sail-tree — never querySelector+assign host.\n\
                 5. sx + tokens only — `cssText` / `.style` / raw CSS is a compile error.\n\
                 6. Missing primitive → report gap; do not escape to DOM.\n\n\
                 **Hard fails in `apps/`:** `createElement` / `cssText` / `querySelector` assign.\n"
            );
            let _ = writeln!(
                md,
                "- No `<style>` / `style=` / raw px-hex — tokens only.\n\
                 - Style product: `style({{ box: pad(4), paint: bg(surface-2) }})` — not utility strings.\n\
                 - Layout keywords: `<row>` `<column>` `<grid>` `<center>` `<stack>`.\n\
                 - Participation on the box channel: `style({{ box: grow(1).minW(0) }})` \
                   (`sail-participation-style` refuses bare `grow`/`shrink`/`w`/`min-w`/`max-w`).\n\
                 - Never `style.display(…)` on layout keywords (`sail-display-on-layout`):\n  \
                   `.sail-col` / `.sail-row` beat `.sx-displaynone`. Plain host + nest layout.\n\
                 - **a11y IR (SPEC-ARIA-1):** never free `aria-*` / free ARIA `role=` \
                   (Warning now, Error next). Semantic props: `label` `description` \
                   `expanded` `selected` `pressed` `current` `level` `live` `invalid` \
                   `hidden` — compiler stamps the correct aria on the correct part. \
                   Fixed roles come from native tags or L1 anatomy (navbar/dialog/tabs/…). \
                   Icon-only buttons **require** `label`. Design identity that used `role=` \
                   → `token=`.\n\
                 - Comments are not load-bearing — tools/lints own the recipes.\n\
                 - Check with MCP `sail_check` (optional `fix`: off|suggest|apply).\n\
                 - Structured edits via MCP `sail_edit` (validate-before-write; never leaves broken intermediates).\n\
                 - After layout edits always run `sail_layout_check` — multi-viewport taffy boxes + collapse diagnostics.\n"
            );
            let _ = writeln!(md, "## Topics\n");
            for t in SurfaceTopic::all() {
                let _ = writeln!(md, "- `{}`", t.as_str());
            }
        }
        SurfaceTopic::Layout => {
            let _ = writeln!(md, "# Sail surface — layout\n");
            let _ = writeln!(
                md,
                "Layout is elements, not CSS. Bases own `display:flex|grid`.\n"
            );
            for tag in sail_lint::rules::layout::TAGS {
                let props = sail_lint::rules::layout::tag_props(tag);
                let _ = writeln!(
                    md,
                    "- `<{tag}>` props: {}",
                    if props.is_empty() {
                        "(none)".into()
                    } else {
                        props
                            .iter()
                            .map(|p| format!("`{p}`"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    }
                );
            }
            let _ = writeln!(
                md,
                "\n**Participation (any element, box channel):** \
                 `style({{ box: grow(1).minW(0).w(full) }})` / `shrink(0)` — bare attrs refused \
                 (`sail-participation-style`). Axis sugar `gap`/`align`/`justify` stays on layout keywords.\n\n\
                 **Refused layout-as-sx** (`sx_vocab::refusals()`):\n"
            );
            for (atom, instead) in sail_lint::sx_vocab::refusals() {
                let _ = writeln!(md, "- `{atom}` — {instead}");
            }
            let _ = writeln!(
                md,
                "\n**Display on layout:** `style.display(…)` on layout keywords is a hard error \
                 (`sail-display-on-layout`). Hide a plain host; nest the layout keyword.\n"
            );
        }
        SurfaceTopic::Style => {
            let _ = writeln!(md, "# Sail surface — style setters\n");
            let _ = writeln!(
                md,
                "Closed product table from `sail_lint::style::setters`. \
                 **Participation:** `grow` / `shrink` live on the box channel \
                 (`style({{ box: grow(1) }})`); bare attrs refused.\n"
            );
            for s in sail_lint::style::setters::setters() {
                if matches!(s.name, "grow" | "shrink") {
                    continue;
                }
                let scale = s.scale.unwrap_or("—");
                let kws = if s.keywords.is_empty() {
                    String::new()
                } else {
                    format!(" keywords: {}", s.keywords.join(", "))
                };
                let arity = match s.name {
                    "colorMix" => {
                        " **`(colorToken, pct)`** soft fill toward transparent; pct 5..95 step 5"
                    }
                    "colorDeepen" => {
                        " **`(colorToken, pct)`** solid deepen toward foreground; pct 5..95 step 5"
                    }
                    _ => "",
                };
                let _ = writeln!(
                    md,
                    "- `style.{}(…)` → vocab `{}` scale `{scale}`{kws}{arity}",
                    s.name, s.vocab
                );
            }
        }
        SurfaceTopic::Tokens => {
            let _ = writeln!(md, "# Sail surface — token scales\n");
            let _ = writeln!(
                md,
                "Closed value sets Style setters draw from (`sail_lint::scales`).\n"
            );
            for scale in &[
                "space",
                "size",
                "color",
                "radius",
                "text",
                "weight",
                "leading",
                "tracking",
                "shadow",
                "z",
                "duration",
                "ease",
                "border-width",
                "opacity",
                "angle",
                "font",
            ] {
                let values = sail_lint::scales::values(scale);
                if !values.is_empty() {
                    let _ = writeln!(md, "- `{scale}`: {}", values.join(", "));
                }
            }
        }
        SurfaceTopic::L1 => {
            let _ = writeln!(md, "# Sail surface — L1 elements\n");
            let (stdlib, reskin) = sail_compiler::l1_by_kind();
            let _ = writeln!(
                md,
                "{} stdlib elements (from `l1_manifest`; native reskins retired).\n\n\
                 ### Semantic a11y props (SPEC-ARIA-1)\n\n\
                 Authors never write free `aria-*` / free ARIA `role=`. Use these \
                 props; the compiler stamps the correct attribute. Fixed host roles \
                 come from L1 anatomy (see each element's contract).\n",
                stdlib.len(),
            );
            let _ = reskin; // always empty post SPEC-L1-1 reskin scrub
            for p in sail_compiler::SEMANTIC_PROPS {
                let ty = match p.kind {
                    sail_compiler::SemanticKind::String => "string".to_string(),
                    sail_compiler::SemanticKind::Bool => "bool".to_string(),
                    sail_compiler::SemanticKind::Number => "number".to_string(),
                    sail_compiler::SemanticKind::Union(vs) => vs.join(" | "),
                };
                let _ = writeln!(
                    md,
                    "- `{}` → `{}` ({ty}) — {}",
                    p.name, p.aria, p.summary
                );
            }
            let _ = writeln!(
                md,
                "\nIcon-only `<button>` requires `label`. `selected` dual-stamps \
                 `aria-selected` while keeping the design attr for chip/pressed.\n\n\
                 ### Host anatomy (compiler-stamped — not free author API)\n\n\
                 The compiler stamps these roles / live / atomic / labels onto L1 \
                 hosts. Authors never write free `role=` / `aria-live` / `aria-atomic` \
                 for them.\n\n\
                 The compiler also stamps `data-sail-host` as **part identity for closed \
                 L1 widgets** (runtime action part discovery). It is not an author API \
                 and is refused under `sail-no-data-attr` if written in app sources. \
                 Framework emit keeps the stamp; app tests locate UI via role, accessible \
                 name, and visible text — not `[data-sail-host]`.\n"
            );
            for (tag, a) in sail_compiler::HOST_ANATOMY {
                let mut parts = Vec::new();
                if let Some(r) = a.role {
                    parts.push(format!("role=`{r}`"));
                }
                if let Some(l) = a.live {
                    parts.push(format!("aria-live=`{l}`"));
                }
                if let Some(at) = a.atomic {
                    parts.push(format!("aria-atomic=`{at}`"));
                }
                if let Some(lb) = a.label {
                    parts.push(format!("aria-label=`{lb}`"));
                }
                let detail = if parts.is_empty() {
                    "(structure-only / native tag carries semantics)".into()
                } else {
                    parts.join(", ")
                };
                let _ = writeln!(md, "- `<{tag}>` — {detail}");
            }
            let _ = writeln!(md, "\n### Authoring bans\n");
            for b in sail_compiler::AUTHORING_BANS {
                let _ = writeln!(
                    md,
                    "- `{}` (`{}`) — {}",
                    b.syntax, b.rule, b.instead
                );
            }
            let _ = writeln!(md, "\n### Stdlib\n");
            for e in &stdlib {
                let _ = writeln!(md, "- `<{}>` — {}", e.tag, e.description);
            }
        }
        SurfaceTopic::ControlFlow => {
            let _ = writeln!(md, "# Sail surface — control flow\n");
            let _ = writeln!(
                md,
                "Four blocks only: if / for / match / when. Arms use brace form\n\
                 `{{pattern as binding => …}}`; binding lives in-brace.\n\n\
                 ```\n\
                 <if {{open()}}>…{{else-if {{x()}} => …}}{{else => …}}</if>\n\
                 <for {{items() as item}} key={{item.id}}>…{{else => …}}</for>\n\
                 <match {{status()}}>\n  \
                   {{ok as value => …}}\n  \
                   {{error as err => …}}\n  \
                   {{loading => …}}\n\
                 </match>\n\
                 <when {{tablet}}>…wide…{{else => …mobile-first default…}}</when>\n\
                 ```\n\n\
                 `key=` is required on `<for>` — without it the reconciler falls back to\n\
                 positional matching and loses DOM identity on reorder.\n\
                 `<when {{token}}>` is structural responsiveness (different trees per\n\
                 theme breakpoint: mobile/tablet/desktop). SSR emits the `{{else => …}}`\n\
                 / default branch only (mobile-first); the client matchMedia-swaps when\n\
                 the query matches. Paint-level Style `.when({{ tablet: style.… }})` is\n\
                 separate and unchanged.\n\
                 Match on LiveState/MutationState with a missing arm is a compile error.\n"
            );
        }
    }
    md
}

/// Structured diagnostic for Check JSON output.
#[derive(Debug, Clone, Serialize)]
pub struct CheckDiag {
    pub severity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule: Option<String>,
    pub message: String,
    pub line: usize,
    pub col: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_col: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Machine-readable rewrite (FixPlan JSON) when confluent and fix:suggest|apply.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rewrite: Option<Value>,
    /// `"sail"` structure/check or `"tsgo"` mapped TypeScript (when includeTs).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

pub fn surface(args: &Value) -> Result<Value, OpError> {
    let topic_s = args
        .get("topic")
        .and_then(|v| v.as_str())
        .ok_or_else(|| OpError::InvalidArgument("Surface requires string `topic`".into()))?;
    let topic = SurfaceTopic::parse(topic_s).ok_or_else(|| {
        OpError::InvalidArgument(format!(
            "unknown topic `{topic_s}`; expected one of: {}",
            SurfaceTopic::all()
                .iter()
                .map(|t| t.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ))
    })?;
    Ok(json!({
        "topic": topic.as_str(),
        "markdown": surface_markdown(topic),
    }))
}

pub fn check(args: &Value, ctx: &Ctx) -> Result<Value, OpError> {
    let path = args.get("path").and_then(|v| v.as_str()).map(Path::new);
    let source = args.get("source").and_then(|v| v.as_str());
    let fix = args.get("fix").and_then(|v| v.as_str()).unwrap_or("off");
    let actor = args.get("actor").and_then(|v| v.as_str());
    // Default true: Studio gutters get tsgo without a second poll (SPEC-LSP-1).
    // Soft-degrades when tsgo is absent. Pass includeTs:false for structure-only.
    let include_ts = args
        .get("includeTs")
        .or_else(|| args.get("include_ts"))
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let (mut src, src_path) = load_source(path, source)?;
    let file_key = src_path
        .as_ref()
        .map(|p| relativize_path(p, ctx))
        .unwrap_or_else(|| "_inline".into());

    let mut auto_fixed: Vec<Value> = Vec::new();
    let mut fixes_out: Vec<Value> = Vec::new();

    let mode = sail_compiler::edit::AutofixMode::parse(fix)
        .unwrap_or(sail_compiler::edit::AutofixMode::Off);

    let mut plans: Vec<sail_compiler::edit::FixPlan> = Vec::new();
    if matches!(
        mode,
        sail_compiler::edit::AutofixMode::Suggest | sail_compiler::edit::AutofixMode::Apply
    ) {
        plans = sail_compiler::edit::suggest_fixes_for_source(&src, &file_key, src_path.as_deref());
        for p in &plans {
            let v = serde_json::to_value(p).unwrap_or(json!({}));
            fixes_out.push(v);
        }
        if mode == sail_compiler::edit::AutofixMode::Apply {
            let patches: Vec<_> = plans
                .iter()
                .filter_map(|p| {
                    p.text_patch
                        .as_ref()
                        .map(|t| (t.start, t.end, t.replacement.clone()))
                })
                .collect();
            if !patches.is_empty() {
                let (next, skipped) = sail_compiler::edit::apply_text_patches(&src, &patches);
                if skipped > 0 {
                    return Err(OpError::Internal(format!(
                        "autofix aborted: {skipped} text patch(es) had invalid spans"
                    )));
                }
                src = next;
                // Write back when path is set.
                if let Some(p) = &src_path {
                    write_under_root(p, &src, &ctx.apply_root)
                        .map_err(|e| OpError::Internal(format!("write {}: {e}", p.display())))?;
                }
            }
            for mut p in plans.iter().cloned() {
                p.actor = actor.map(|a| a.to_string());
                auto_fixed.push(serde_json::to_value(p).unwrap_or(json!({})));
            }
        }
    }

    let mut diags = if include_ts {
        run_check_with_ts(&src, src_path.as_deref(), ctx)?
    } else {
        run_check_on(&src, src_path.as_deref())?
    };
    // Attach rewrite payloads onto matching diagnostic rows (spec: LLMs read diags).
    if !plans.is_empty() {
        attach_rewrites_to_check_diags(&mut diags, &plans);
    }
    let mut out = json!({
        "ok": true,
        "diagnostics": diags,
        "count": diags.len(),
    });
    if !fixes_out.is_empty() {
        out["fixes"] = Value::Array(fixes_out);
    }
    if !auto_fixed.is_empty() {
        out["autoFixed"] = Value::Array(auto_fixed);
        out["source"] = json!(src);
    }
    Ok(out)
}

/// Structure check + optional mapped tsgo diagnostics via sail-lsp.
fn run_check_with_ts(
    src: &str,
    src_path: Option<&Path>,
    ctx: &Ctx,
) -> Result<Vec<CheckDiag>, OpError> {
    let path_s = src_path.map(|p| p.display().to_string());
    let payloads = with_intelligence(ctx, |intel| intel.diagnostics_sail(src, path_s.as_deref()));
    Ok(payloads
        .into_iter()
        .map(|d| CheckDiag {
            severity: d.severity,
            rule: d.rule,
            message: d.message,
            line: d.line,
            col: d.col,
            end_line: d.end_line,
            end_col: d.end_col,
            path: path_s.clone(),
            rewrite: None,
            source: d.source,
        })
        .collect())
}

/// Hover at 1-based position — sail-lsp library (virtual_ts + tsgo).
pub fn hover(args: &Value, ctx: &Ctx) -> Result<Value, OpError> {
    let path = args.get("path").and_then(|v| v.as_str()).map(Path::new);
    let source = args.get("source").and_then(|v| v.as_str());
    let line = args
        .get("line")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| OpError::InvalidArgument("Hover requires numeric `line`".into()))?
        as u32;
    let column = args
        .get("column")
        .or_else(|| args.get("col"))
        .and_then(|v| v.as_u64())
        .ok_or_else(|| OpError::InvalidArgument("Hover requires numeric `column`".into()))?
        as u32;
    let (src, src_path) = load_source(path, source)?;
    let path_s = src_path
        .as_ref()
        .map(|p| p.display().to_string())
        .or_else(|| path.map(|p| p.display().to_string()));
    let payload = with_intelligence(ctx, |intel| {
        intel.hover_sail(&src, path_s.as_deref(), line, column)
    });
    match payload {
        None => Ok(json!({ "contents": null })),
        Some(h) => {
            let mut out = json!({ "contents": h.contents });
            if let Some(r) = h.range {
                out["range"] = json!({
                    "startLine": r.start_line,
                    "startCol": r.start_col,
                    "endLine": r.end_line,
                    "endCol": r.end_col,
                });
            }
            Ok(out)
        }
    }
}

/// Completions at 1-based position — sail-lsp library.
pub fn complete(args: &Value, ctx: &Ctx) -> Result<Value, OpError> {
    let path = args.get("path").and_then(|v| v.as_str()).map(Path::new);
    let source = args.get("source").and_then(|v| v.as_str());
    let line = args
        .get("line")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| OpError::InvalidArgument("Complete requires numeric `line`".into()))?
        as u32;
    let column = args
        .get("column")
        .or_else(|| args.get("col"))
        .and_then(|v| v.as_u64())
        .ok_or_else(|| OpError::InvalidArgument("Complete requires numeric `column`".into()))?
        as u32;
    let (src, src_path) = load_source(path, source)?;
    let path_s = src_path
        .as_ref()
        .map(|p| p.display().to_string())
        .or_else(|| path.map(|p| p.display().to_string()));
    let items = with_intelligence(ctx, |intel| {
        intel.complete_sail(&src, path_s.as_deref(), line, column)
    });
    let items: Vec<Value> = items
        .into_iter()
        .map(|i| serde_json::to_value(i).unwrap_or(json!({})))
        .collect();
    Ok(json!({ "items": items }))
}

fn attach_rewrites_to_check_diags(diags: &mut [CheckDiag], plans: &[sail_compiler::edit::FixPlan]) {
    for d in diags.iter_mut() {
        if d.rewrite.is_some() {
            continue;
        }
        if let Some(plan) = plans.iter().find(|p| {
            p.line == d.line && p.col == d.col && d.rule.as_deref() == Some(p.rule.as_str())
        }) {
            d.rewrite = serde_json::to_value(plan).ok();
        } else if let Some(plan) = plans.iter().find(|p| {
            // Fallback: same rule + line (col can drift after apply).
            p.line == d.line && d.rule.as_deref() == Some(p.rule.as_str())
        }) {
            d.rewrite = serde_json::to_value(plan).ok();
        }
    }
}

pub fn run_check(path: Option<&Path>, source: Option<&str>) -> Result<Vec<CheckDiag>, OpError> {
    let (src, src_path) = load_source(path, source)?;
    run_check_on(&src, src_path.as_deref())
}

fn load_source(
    path: Option<&Path>,
    source: Option<&str>,
) -> Result<(String, Option<PathBuf>), OpError> {
    match (path, source) {
        (Some(p), None) => {
            let text = std::fs::read_to_string(p)
                .map_err(|e| OpError::NotFound(format!("read {}: {e}", p.display())))?;
            Ok((text, Some(p.to_path_buf())))
        }
        (None, Some(s)) => Ok((s.to_string(), None)),
        (Some(p), Some(s)) => Ok((s.to_string(), Some(p.to_path_buf()))),
        (None, None) => Err(OpError::InvalidArgument(
            "Check requires `path` and/or `source`".into(),
        )),
    }
}

fn run_check_on(src: &str, src_path: Option<&Path>) -> Result<Vec<CheckDiag>, OpError> {
    let diags = sail_compiler::check_all(src, src_path, &[], None);
    Ok(diags
        .into_iter()
        .map(|d| CheckDiag {
            severity: match d.severity {
                sail_compiler::DiagSeverity::Error => "error".into(),
                sail_compiler::DiagSeverity::Warning => "warning".into(),
                sail_compiler::DiagSeverity::Fix => "fix".into(),
            },
            rule: d.rule,
            message: d.message,
            line: d.line,
            col: d.col,
            end_line: Some(d.end_line),
            end_col: Some(d.end_col),
            path: src_path.map(|p| p.display().to_string()),
            rewrite: None,
            source: Some("sail".into()),
        })
        .collect())
}

fn relativize_path(p: &Path, ctx: &Ctx) -> String {
    let s = p.display().to_string();
    let root = ctx.workspace_root.display().to_string();
    s.strip_prefix(&root)
        .map(|r| r.trim_start_matches('/').to_string())
        .unwrap_or(s)
}

/// Structured SailAuthoring/Edit.
pub fn edit(args: &Value, ctx: &Ctx) -> Result<Value, OpError> {
    let path_s = args
        .get("path")
        .and_then(|v| v.as_str())
        .filter(|p| !p.trim().is_empty())
        .ok_or_else(|| OpError::InvalidArgument("Edit requires non-empty `path`".into()))?;
    let ops_v = args
        .get("ops")
        .ok_or_else(|| OpError::InvalidArgument("Edit requires `ops` array".into()))?;
    let ops = sail_compiler::edit::parse_ops(ops_v).map_err(|e| OpError::InvalidArgument(e))?;
    let dry_run = args
        .get("dryRun")
        .or_else(|| args.get("dry_run"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let write_source = args
        .get("writeSource")
        .or_else(|| args.get("write_source"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let autofix_s = args
        .get("autofix")
        .and_then(|v| v.as_str())
        .unwrap_or("off");
    let autofix = sail_compiler::edit::AutofixMode::parse(autofix_s)
        .unwrap_or(sail_compiler::edit::AutofixMode::Off);
    let actor = args.get("actor").and_then(|v| v.as_str());

    let target =
        resolve_write_path(path_s, &ctx.apply_root).map_err(|e| OpError::PermissionDenied(e))?;

    // Studio freeform Source save: empty ops + full `source` body + writeSource.
    // Still validate-before-write via check_all so disk never holds broken Sail.
    if write_source && ops.is_empty() {
        let Some(body) = args.get("source").and_then(|v| v.as_str()) else {
            return Err(OpError::InvalidArgument(
                "writeSource requires string `source`".into(),
            ));
        };
        let diags = run_check_on(body, Some(target.as_path()))?;
        let has_error = diags
            .iter()
            .any(|d| d.severity.eq_ignore_ascii_case("error"));
        let diags_json: Vec<Value> = diags
            .iter()
            .map(|d| serde_json::to_value(d).unwrap_or(json!({})))
            .collect();
        if has_error {
            return Ok(json!({
                "applied": false,
                "modelRevision": 0,
                "diagnostics": diags_json,
                "autoFixed": [],
                "source": body,
                "path": target.display().to_string(),
                "message": "writeSource refused: source has error diagnostics",
            }));
        }
        if !dry_run {
            write_full(&target, body).map_err(|e| OpError::Internal(e))?;
            crate::watch::publish_file_under(&ctx.workspace_root, &target);
        }
        return Ok(json!({
            "applied": true,
            "modelRevision": 1,
            "diagnostics": diags_json,
            "autoFixed": [],
            "source": body,
            "path": target.display().to_string(),
            "message": "writeSource ok",
        }));
    }

    let source = if let Some(s) = args.get("source").and_then(|v| v.as_str()) {
        s.to_string()
    } else {
        std::fs::read_to_string(&target)
            .map_err(|e| OpError::NotFound(format!("read {}: {e}", target.display())))?
    };

    let file_key = path_s.trim().trim_start_matches("./");
    let result = sail_compiler::edit::apply_ops_to_source(
        &source,
        file_key,
        &ops,
        Some(target.as_path()),
        autofix,
        actor,
    );

    if result.applied && !dry_run {
        write_full(&target, &result.source).map_err(|e| OpError::Internal(e))?;
        crate::watch::publish_file_under(&ctx.workspace_root, &target);
    }

    let diags: Vec<Value> = result
        .diagnostics
        .iter()
        .map(|d| serde_json::to_value(d).unwrap_or(json!({})))
        .collect();
    let auto_fixed: Vec<Value> = result
        .auto_fixed
        .iter()
        .map(|f| serde_json::to_value(f).unwrap_or(json!({})))
        .collect();

    Ok(json!({
        "applied": result.applied,
        "modelRevision": result.model_revision,
        "diagnostics": diags,
        "autoFixed": auto_fixed,
        "source": result.source,
        "path": target.display().to_string(),
        "message": result.message,
    }))
}

fn resolve_write_path(path_s: &str, apply_root: &Path) -> Result<PathBuf, String> {
    let root = apply_root
        .canonicalize()
        .unwrap_or_else(|_| apply_root.to_path_buf());
    let candidate = {
        let p = PathBuf::from(path_s);
        if p.is_absolute() { p } else { root.join(p) }
    };
    let resolved = canonicalize_for_write(&candidate)?;
    if !resolved.starts_with(&root) {
        return Err(format!(
            "path `{}` resolves outside apply root `{}` (set SAIL_MCP_ROOT to widen)",
            resolved.display(),
            root.display()
        ));
    }
    Ok(resolved)
}

fn canonicalize_for_write(path: &Path) -> Result<PathBuf, String> {
    if path.exists() {
        return path
            .canonicalize()
            .map_err(|e| format!("canonicalize {}: {e}", path.display()));
    }
    let mut components = Vec::new();
    let mut cur = path.to_path_buf();
    while !cur.exists() {
        let name = cur
            .file_name()
            .ok_or_else(|| format!("cannot resolve {}", path.display()))?
            .to_os_string();
        components.push(name);
        cur = cur
            .parent()
            .ok_or_else(|| format!("cannot resolve {}", path.display()))?
            .to_path_buf();
        if components.len() > 64 {
            return Err(format!("path too deep: {}", path.display()));
        }
    }
    let mut base = cur
        .canonicalize()
        .map_err(|e| format!("canonicalize {}: {e}", cur.display()))?;
    for name in components.into_iter().rev() {
        if name == "." || name == ".." {
            return Err("path components `.` / `..` not allowed after root join".into());
        }
        base.push(name);
    }
    Ok(base)
}

fn write_full(path: &Path, body: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("mkdir {}: {e}", parent.display()))?;
    }
    std::fs::write(path, body).map_err(|e| format!("write {}: {e}", path.display()))
}

fn write_under_root(path: &Path, body: &str, apply_root: &Path) -> Result<(), String> {
    let root = apply_root
        .canonicalize()
        .unwrap_or_else(|_| apply_root.to_path_buf());
    let resolved = if path.exists() {
        path.canonicalize()
            .map_err(|e| format!("canonicalize {}: {e}", path.display()))?
    } else {
        canonicalize_for_write(path)?
    };
    if !resolved.starts_with(&root) {
        return Err(format!(
            "path `{}` outside apply root `{}`",
            resolved.display(),
            root.display()
        ));
    }
    write_full(&resolved, body)
}

pub fn layout_check(args: &Value) -> Result<Value, OpError> {
    let source = if let Some(s) = args.get("source").and_then(|v| v.as_str()) {
        s.to_string()
    } else if let Some(p) = args.get("path").and_then(|v| v.as_str()) {
        std::fs::read_to_string(p).map_err(|e| OpError::NotFound(format!("read {p}: {e}")))?
    } else {
        return Err(OpError::InvalidArgument(
            "LayoutCheck requires `path` or `source`".into(),
        ));
    };
    sail_layout_check(&source)
}

/// Compiler policy registry — `mode=list|get`, same payload as CLI / axum.
pub fn policy(args: &Value, ctx: &Ctx) -> Result<Value, OpError> {
    let mode = args
        .get("mode")
        .and_then(|v| v.as_str())
        .unwrap_or("list");
    let project = args
        .get("project")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let project_path = project.map(|p| {
        let path = PathBuf::from(&p);
        if path.is_absolute() {
            path
        } else {
            ctx.workspace().join(p)
        }
    });
    // When get has no project, resolve against the ops project root.
    let dir = project_path.or_else(|| {
        if mode == "get" {
            Some(ctx.project().to_path_buf())
        } else {
            None
        }
    });
    sail_compiler::policy::serialize_policy(mode, dir.as_deref())
        .map_err(|e| OpError::InvalidArgument(e.message))
}

/// List workspace-relative project source files (explorer seed).
/// Walks the full project tree (ts/json/css/sail/…) with sane directory ignores.
pub fn list_files(args: &Value, ctx: &Ctx) -> Result<Value, OpError> {
    let project = args
        .get("project")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            // Prefer workspace-relative apps/... when project_root lives under workspace.
            let p = ctx.project();
            let ws = ctx.workspace();
            p.strip_prefix(ws)
                .map(|r| r.to_string_lossy().replace('\\', "/"))
                .unwrap_or_else(|_| p.display().to_string().replace('\\', "/"))
        });
    if project.contains("..") {
        return Err(OpError::InvalidArgument(
            "ListFiles project must not contain ..".into(),
        ));
    }
    let root = ctx.workspace().join(&project);
    if !root.is_dir() {
        return Err(OpError::NotFound(format!("project not found: {project}")));
    }
    let mut files: Vec<String> = Vec::new();
    walk_project_files(&root, &root, &project, &mut files);
    files.sort();
    files.dedup();
    Ok(json!({
        "ok": true,
        "project": project,
        "files": files,
    }))
}

/// Directory names skipped while listing project sources for the explorer.
fn is_ignored_dir(name: &str) -> bool {
    name.starts_with('.')
        || name == "node_modules"
        || name == "target"
        || name == "dist"
        || name == ".deck"
        || name == "coverage"
        || name == "__pycache__"
}

/// True for ordinary text-ish source files (not binary blobs / lock noise).
fn is_listed_source_file(name: &str) -> bool {
    if name.starts_with('.') {
        // Keep common config dotfiles; skip the rest of hidden files.
        return matches!(
            name,
            ".gitignore"
                | ".npmrc"
                | ".editorconfig"
                | ".env"
                | ".env.example"
                | ".prettierrc"
                | ".eslintrc"
                | ".eslintrc.cjs"
                | ".eslintrc.js"
        );
    }
    // Skip heavy / generated lockfiles from the tree (still list package.json).
    if name.ends_with(".lock") || name == "package-lock.json" || name == "pnpm-lock.yaml" {
        return false;
    }
    let Some(ext) = Path::new(name).extension().and_then(|e| e.to_str()) else {
        // Extensionless: allow common script names only.
        return matches!(
            name,
            "Makefile" | "Dockerfile" | "Procfile" | "LICENSE" | "README"
        );
    };
    matches!(
        ext,
        "sail"
            | "ts"
            | "tsx"
            | "js"
            | "jsx"
            | "mjs"
            | "cjs"
            | "json"
            | "jsonc"
            | "css"
            | "scss"
            | "md"
            | "mdx"
            | "html"
            | "svg"
            | "toml"
            | "yaml"
            | "yml"
            | "txt"
            | "rs"
            | "sh"
    )
}

fn walk_project_files(project_root: &Path, dir: &Path, project_rel: &str, out: &mut Vec<String>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for ent in rd.flatten() {
        let p = ent.path();
        let name = ent.file_name().to_string_lossy().into_owned();
        if p.is_dir() {
            if is_ignored_dir(&name) {
                continue;
            }
            walk_project_files(project_root, &p, project_rel, out);
        } else if is_listed_source_file(&name) {
            if let Ok(rel) = p.strip_prefix(project_root) {
                let rel = rel.to_string_lossy().replace('\\', "/");
                out.push(format!("{project_rel}/{rel}"));
            }
        }
    }
}

pub fn sail_layout_check(source: &str) -> Result<Value, OpError> {
    let diags = sail_compiler::check::layout::collect(source);
    let diagnostics: Vec<Value> = diags
        .into_iter()
        .map(|d| {
            json!({
                "severity": match d.severity {
                    sail_compiler::DiagSeverity::Error => "error",
                    sail_compiler::DiagSeverity::Warning => "warning",
                    sail_compiler::DiagSeverity::Fix => "fix",
                },
                "rule": d.rule.unwrap_or_else(|| "sail-layout".into()),
                "message": d.message,
                "line": d.line,
                "col": d.col,
                "end_line": d.end_line,
                "end_col": d.end_col,
            })
        })
        .collect();

    let preprocessed = sail_compiler::parse::preprocess::preprocess(source)
        .map_err(|e| OpError::InvalidArgument(format!("preprocess: {e}")))?;
    let viewports = sail_compiler::check::layout::layout_viewports(&preprocessed);
    let mut viewport_reports: Vec<Value> = Vec::new();
    for vp in viewports {
        match sail_layout::check_preprocessed(&preprocessed, vp) {
            Ok(scan) => {
                let boxes: Vec<Value> = scan
                    .boxes
                    .iter()
                    .map(|b| {
                        json!({
                            "index": b.index,
                            "tag": b.tag,
                            "class": b.class,
                            "x": b.x,
                            "y": b.y,
                            "width": b.width,
                            "height": b.height,
                            "unknown_subtree": b.unknown_subtree,
                            "collapsed": b.width < 0.5 || b.height < 0.5,
                        })
                    })
                    .collect();
                let collapsed = boxes
                    .iter()
                    .filter(|b| {
                        b["collapsed"].as_bool() == Some(true)
                            && b["unknown_subtree"].as_bool() != Some(true)
                    })
                    .count();
                viewport_reports.push(json!({
                    "viewport": { "width": vp.0, "height": vp.1 },
                    "box_count": boxes.len(),
                    "collapsed_count": collapsed,
                    "boxes": boxes,
                }));
            }
            Err(e) => {
                viewport_reports.push(json!({
                    "viewport": { "width": vp.0, "height": vp.1 },
                    "error": e.to_string(),
                }));
            }
        }
    }

    let count = diagnostics.len();
    Ok(json!({
        "ok": true,
        "engine": "taffy",
        "note": "Taffy box-tree + multi-viewport diagnostics. High-signal for agents; not browser pixel-parity. Prefer this after any layout edit. Locations on diagnostics are original-source line/col.",
        "diagnostics": diagnostics,
        "count": count,
        "viewports": viewport_reports,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surface_topics_render() {
        for t in SurfaceTopic::all() {
            let md = surface_markdown(*t);
            assert!(!md.is_empty(), "empty for {}", t.as_str());
            assert!(md.contains('#'), "expected markdown for {}", t.as_str());
        }
        let style = surface_markdown(SurfaceTopic::Style);
        assert!(style.contains("style.bg"), "{style}");
        assert!(
            !style.contains("style.grow(…)"),
            "must not teach style.grow"
        );
        let l1 = surface_markdown(SurfaceTopic::L1);
        assert!(l1.contains("label"), "semantic props: {l1}");
        assert!(l1.contains("Host anatomy"), "host anatomy section: {l1}");
        assert!(l1.contains("sail-no-data-attr"), "authoring bans: {l1}");
        assert!(
            l1.contains("data-sail-host") && l1.contains("part identity"),
            "host anatomy must document compiler-only data-sail-host: {l1}"
        );
        assert!(
            l1.contains("<accordion>") || l1.contains("<separator>") || l1.contains("<kbd>"),
            "stdlib L1 rows present: {l1}"
        );
        assert!(
            !l1.contains("### Native reskins"),
            "native reskins section must be gone: {l1}"
        );
    }

    #[test]
    fn check_flags_display_on_layout() {
        let src = r#"<column style={style.display(none)}>x</column>"#;
        let diags = run_check(None, Some(src)).unwrap();
        assert!(
            diags
                .iter()
                .any(|d| d.rule.as_deref() == Some("sail-display-on-layout")),
            "expected display-on-layout: {diags:?}"
        );
    }

    #[test]
    fn hover_signal_typed_when_tsgo_present() {
        use crate::call_rpc;
        use crate::ctx::Ctx;
        // Soft-skip when tsgo is missing (CI without bun install).
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .canonicalize()
            .expect("repo root");
        if sail_lsp::tsgo_test::discover_tsgo(Some(&root)).is_none() {
            eprintln!("SKIP hover_signal: tsgo not found");
            return;
        }
        let ctx = Ctx::new(&root, &root, &root);
        let src = r#"<controller>
import { controller, signal } from '@ship/sail-runtime'
export default controller((ctx) => ({ count: signal(0) }))
</controller>
<div>{count()}</div>
"#;
        // 1-based line/col of `signal` in controller body.
        let offset = src.find("signal(0)").expect("signal");
        let mut line = 1u32;
        let mut col = 1u32;
        for (i, ch) in src.char_indices() {
            if i >= offset {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        // Warm session once (tsgo settle).
        let _ = call_rpc(
            "SailAuthoring",
            "Hover",
            &json!({ "source": src, "path": "fixture-counter.sail", "line": line, "column": col }),
            &ctx,
        );
        std::thread::sleep(std::time::Duration::from_millis(250));
        let v = call_rpc(
            "SailAuthoring",
            "Hover",
            &json!({ "source": src, "path": "fixture-counter.sail", "line": line, "column": col }),
            &ctx,
        )
        .expect("Hover rpc");
        let contents = v["contents"].as_str().unwrap_or("");
        if contents.is_empty() {
            eprintln!("SKIP hover_signal: empty contents (tsgo not ready / degrade)");
            return;
        }
        let lower = contents.to_ascii_lowercase();
        assert!(
            lower.contains("signal") || lower.contains("function"),
            "expected typed hover for signal; got {contents}"
        );
    }

    #[test]
    fn complete_returns_items_array() {
        use crate::call_rpc;
        use crate::ctx::Ctx;
        let ctx = Ctx::from_env();
        let src = r#"<controller>
import { controller, signal } from '@ship/sail-runtime'
export default controller((ctx) => ({ count: signal(0) }))
</controller>
<div></div>
"#;
        let v = call_rpc(
            "SailAuthoring",
            "Complete",
            &json!({ "source": src, "line": 3, "column": 40 }),
            &ctx,
        )
        .expect("Complete rpc");
        assert!(v["items"].is_array(), "items array: {v}");
    }

    #[test]
    fn edit_set_attr_writes_under_root() {
        use crate::call_rpc;
        use crate::ctx::Ctx;
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let rel = "nested/demo.sail";
        let full = root.join(rel);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(&full, "<column gap=\"2\"><div>x</div></column>\n").unwrap();

        let mut ctx = Ctx::from_env();
        ctx.apply_root = root.to_path_buf();
        ctx.workspace_root = root.to_path_buf();
        ctx.project_root = root.to_path_buf();

        // Discover node id via inspect-compatible index
        let src = std::fs::read_to_string(&full).unwrap();
        let idx = sail_compiler::edit::build_node_index(&src, rel).unwrap();
        let col = idx.values().find(|n| n.tag == "column").unwrap();

        let v = call_rpc(
            "SailAuthoring",
            "Edit",
            &json!({
                "path": rel,
                "ops": [{
                    "op": "setAttr",
                    "nodeId": col.id,
                    "name": "grow",
                }]
            }),
            &ctx,
        )
        .unwrap();
        assert_eq!(v["applied"], true, "{v}");
        let after = std::fs::read_to_string(&full).unwrap();
        assert!(after.contains("grow"), "{after}");

        // Off-vocab setSx leaves file untouched
        let before = after.clone();
        let col2 = sail_compiler::edit::build_node_index(&before, rel)
            .unwrap()
            .into_values()
            .find(|n| n.tag == "column")
            .unwrap();
        let v2 = call_rpc(
            "SailAuthoring",
            "Edit",
            &json!({
                "path": rel,
                "ops": [{
                    "op": "setSx",
                    "nodeId": col2.id,
                    "setter": "notARealSetter",
                    "args": ["x"]
                }]
            }),
            &ctx,
        )
        .unwrap();
        assert_eq!(v2["applied"], false, "{v2}");
        assert_eq!(std::fs::read_to_string(&full).unwrap(), before);
    }

    #[test]
    fn layout_check_uses_author_source_line_col() {
        let src = "\n\n<column gap=\"0\"><div></div></column>";
        let out = sail_layout_check(src).unwrap();
        assert_eq!(out["ok"], true);
        let diags = out["diagnostics"].as_array().expect("diagnostics array");
        let zero = diags
            .iter()
            .find(|d| {
                d["message"]
                    .as_str()
                    .map(|m| m.contains("0px on its main axis"))
                    .unwrap_or(false)
            })
            .expect("zero-main-axis");
        assert_eq!(zero["line"], 3, "must map to original <column> line");
    }

    #[test]
    fn list_files_includes_non_sail_and_ignores_deps() {
        use crate::ctx::Ctx;
        let dir = tempfile::tempdir().unwrap();
        let ws = dir.path();
        let proj = ws.join("apps/demo");
        std::fs::create_dir_all(proj.join("routes/edit")).unwrap();
        std::fs::create_dir_all(proj.join("src")).unwrap();
        std::fs::create_dir_all(proj.join("node_modules/x")).unwrap();
        std::fs::create_dir_all(proj.join("target/debug")).unwrap();
        std::fs::create_dir_all(proj.join("dist")).unwrap();
        std::fs::create_dir_all(proj.join(".git/objects")).unwrap();
        std::fs::write(proj.join("routes/page.sail"), "<column></column>\n").unwrap();
        std::fs::write(proj.join("routes/edit/page.sail"), "<column></column>\n").unwrap();
        std::fs::write(proj.join("src/main.ts"), "export {}\n").unwrap();
        std::fs::write(proj.join("package.json"), "{}\n").unwrap();
        std::fs::write(proj.join("node_modules/x/index.js"), "1\n").unwrap();
        std::fs::write(proj.join("target/debug/foo"), "bin\n").unwrap();
        std::fs::write(proj.join("dist/out.js"), "1\n").unwrap();
        std::fs::write(proj.join(".git/objects/x"), "x\n").unwrap();

        let ctx = Ctx::new(ws, &proj, ws);
        let out = list_files(&json!({ "project": "apps/demo" }), &ctx).unwrap();
        assert_eq!(out["ok"], true, "{out}");
        let files = out["files"].as_array().expect("files array");
        let paths: Vec<&str> = files.iter().filter_map(|v| v.as_str()).collect();
        assert!(
            paths.iter().any(|p| p.ends_with("routes/page.sail")),
            "sail listed: {paths:?}"
        );
        assert!(
            paths.iter().any(|p| p.ends_with("src/main.ts")),
            "ts listed: {paths:?}"
        );
        assert!(
            paths.iter().any(|p| p.ends_with("package.json")),
            "json listed: {paths:?}"
        );
        assert!(
            paths.iter().all(|p| !p.contains("node_modules")),
            "node_modules ignored: {paths:?}"
        );
        assert!(
            paths
                .iter()
                .all(|p| !p.contains("/target/") && !p.ends_with("/target")),
            "target ignored: {paths:?}"
        );
        assert!(
            paths
                .iter()
                .all(|p| !p.contains("/dist/") && !p.contains(".git")),
            "dist/.git ignored: {paths:?}"
        );
    }

    #[test]
    fn is_listed_source_file_accepts_common_exts() {
        assert!(is_listed_source_file("page.sail"));
        assert!(is_listed_source_file("app.ts"));
        assert!(is_listed_source_file("package.json"));
        assert!(is_listed_source_file("theme.css"));
        assert!(!is_listed_source_file("package-lock.json"));
        assert!(!is_listed_source_file("foo.lock"));
        assert!(!is_listed_source_file("secret.bin"));
        assert!(is_ignored_dir("node_modules"));
        assert!(is_ignored_dir(".git"));
        assert!(is_ignored_dir("target"));
        assert!(is_ignored_dir("dist"));
        assert!(is_ignored_dir(".deck"));
        assert!(!is_ignored_dir("src"));
    }
}
