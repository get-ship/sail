//! `SailAuthoring` RPCs — Surface, Check, Edit, LayoutCheck, ListFiles.

use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::{Value, json};

use crate::ctx::Ctx;
use crate::error::OpError;

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
                "- No `<style>` / `style=` / raw px-hex — tokens only.\n\
                 - Style product: `sx={{style.pad(4).bg(surface-2)}}` — not utility strings.\n\
                 - Layout keywords: `<row>` `<column>` `<grid>` `<center>` `<stack>`.\n\
                 - Participation attrs only: `grow` `shrink` `w` `min-w` `max-w` \
                   (`sail-participation-style` bans `style.grow`/`shrink`).\n\
                 - Never `style.display(…)` on layout keywords (`sail-display-on-layout`):\n  \
                   `.sail-col` / `.sail-row` beat `.sx-displaynone`. Plain host + nest layout.\n\
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
                "\n**Participation (any element):** `grow` / `shrink` (bool or `\"0\"|\"1\"`), \
                 `w` / `min-w` / `max-w` (size tokens).\n\n\
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
                 **Not listed:** `grow` / `shrink` — layout attrs only.\n"
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
                "{} stdlib + {} reskins (from `l1_manifest`).\n\n### Stdlib\n",
                stdlib.len(),
                reskin.len()
            );
            for e in &stdlib {
                let _ = writeln!(md, "- `<{}>` — {}", e.tag, e.description);
            }
            let _ = writeln!(md, "\n### Native reskins\n");
            for e in &reskin {
                let _ = writeln!(md, "- `<{}>` — {}", e.tag, e.description);
            }
        }
        SurfaceTopic::ControlFlow => {
            let _ = writeln!(md, "# Sail surface — control flow\n");
            let _ = writeln!(
                md,
                "Element blocks (brace `{{#if}}` / `{{#for}}` are retired).\n\
                 **Product form is brace-primary** — no `cond=` / `on=` / `of=` tax:\n\n\
                 ```\n\
                 <if {{open()}}>…\
                 <else-if {{x()}}>…</else-if>\
                 <else>…</else></if>\n\
                 <for {{items()}} as=\"item\" key={{item.id}}>…</for>\n\
                 <match {{status()}}>\n  \
                   <case {{'ok'}}>…</case>\n  \
                   <default>…</default>\n\
                 </match>\n\
                 <while-loading {{sig}}>…</while-loading>\n\
                 <while-error as=\"err\" {{sig}}>…</while-error>\n\
                 ```\n\n\
                 Residual named attrs (`cond=`, `on=`, `of=`) still parse.\n\
                 `key=` is required on `<for>` — without it the reconciler falls back to\n\
                 positional matching and loses DOM identity on reorder.\n"
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

    let mut diags = run_check_on(&src, src_path.as_deref())?;
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

/// List workspace-relative `.sail` files under a project (explorer seed).
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
    for sub in ["routes", "src", "packs"] {
        let dir = root.join(sub);
        if dir.is_dir() {
            walk_sail_under(&root, &dir, &project, &mut files);
        }
    }
    files.sort();
    files.dedup();
    Ok(json!({
        "ok": true,
        "project": project,
        "files": files,
    }))
}

fn walk_sail_under(project_root: &Path, dir: &Path, project_rel: &str, out: &mut Vec<String>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for ent in rd.flatten() {
        let p = ent.path();
        let name = ent.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') || name == "node_modules" || name == "target" {
            continue;
        }
        if p.is_dir() {
            walk_sail_under(project_root, &p, project_rel, out);
        } else if p.extension().and_then(|e| e.to_str()) == Some("sail") {
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
    }

    #[test]
    fn check_flags_display_on_layout() {
        let src = r#"<column sx={style.display(none)}>x</column>"#;
        let diags = run_check(None, Some(src)).unwrap();
        assert!(
            diags
                .iter()
                .any(|d| d.rule.as_deref() == Some("sail-display-on-layout")),
            "expected display-on-layout: {diags:?}"
        );
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
}
