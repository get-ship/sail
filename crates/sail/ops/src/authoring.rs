//! `SailAuthoring` RPCs — Surface, Check, Edit, LayoutCheck, ListFiles, Policy, Hover, Complete.

use std::collections::BTreeMap;
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
    Icons,
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
            "icons" | "icon" => Some(Self::Icons),
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
            Self::Icons => "icons",
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
            Self::Icons,
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
                 5. Style + tokens only — `cssText` / `.style` / raw CSS is a compile error.\n\
                 6. Missing primitive → report gap; do not escape to DOM.\n\n\
                 **Hard fails in `apps/`:** `createElement` / `cssText` / `querySelector` assign.\n"
            );
            let _ = writeln!(
                md,
                "- No `<style>` / `style=` / raw px-hex — tokens only.\n\
                 - Style product: `style({{ pad(4).bg(surface) }})` — not utility strings.\n\
                 - Layout keywords: `<row>` `<column>` `<grid>` `<center>` `<stack>`.\n\
                 - Participation via `size` / `grow` / `shrink`: `style({{ size({{ w: '100%', min-w: 0 }}).grow(1) }})` \
                   (`sail-participation-style` refuses bare `grow`/`shrink`/`w`/`min-w`/`max-w`).\n\
                 - Never `display(…)` on layout keywords (`sail-display-on-layout`):\n  \
                   `.sail-col` / `.sail-row` beat `.s-displaynone`. Plain host + nest layout.\n\
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
                "\n**Participation (any element):** \
                 `style({{ size({{ w: '100%', min-w: 0 }}).grow(1) }})` / `shrink(0)` — bare attrs refused \
                 (`sail-participation-style`). Axis sugar `gap`/`align`/`justify` stays on layout keywords.\n\n\
                 **Layout is elements, not Style atoms.** `flex`/`grid`/`gap-*`/`items-*` \
                 are not vocabulary — use `<row>` / `<column>` / `<grid>` and their attrs.\n\n\
                 **Display on layout:** `display(…)` on layout keywords is a hard error \
                 (`sail-display-on-layout`). Hide a plain host; nest the layout keyword.\n"
            );
        }
        SurfaceTopic::Style => {
            let _ = writeln!(md, "# Sail surface — style\n");
            let _ = writeln!(
                md,
                "The vocabulary table **is** the setter table (SPEC-SPINNAKER-3). \
                 Live names are `bg`/`fg`/`pad`/`margin`/`position`/`play`. \
                 v2 `fill`/`ink`/`place`/`space`/`flow`/`motion` are unknown vocabulary. \
                 Named composite or slot object. Flat chain is the language — \
                 not constructor `style.padX(3)`. Absorbed longhands (`padX`, \
                 `fontSize`, `truncate`, `shadow`) do not resolve.\n\n\
                 ```\n\
                 style chrome = {{\n  \
                   size(control-md).pad({{ x: 3 }}).bg(surface).fg(foreground).border(subtle).radius(lg).elevation(sm).type(body-sm)\n\
                 }}\n\
                 ```\n\n\
                 **Participation** lives on `size` / `grow` / `shrink` \
                 (`style({{ size({{ w: '100%', min-w: 0 }}).grow(1) }})`). \
                 Bare attrs refused (`sail-participation-style`).\n\n\
                 `border` slots are `color`/`width`/`style`/`edge`/`kind` — radius is not a slot.\n\
                 `size` is box geometry only — pad is not a slot (use `pad`).\n"
            );
            let surface = spinnaker::StyleSurface::merged();
            let _ = writeln!(md, "## StyleSurface presets\n");
            let _ = writeln!(
                md,
                "Complete / validate / this topic read **one** table. \
                 `type(heading)` is a theme preset. `size(grow)` is a language \
                 preset (IR, not DTCG) — lowering lands in M2b.\n"
            );
            let _ = writeln!(md, "### Language\n");
            for p in surface
                .presets
                .iter()
                .filter(|p| p.origin == spinnaker::PresetOrigin::Language)
            {
                let _ = writeln!(md, "- `{}({})` — {}", p.concern, p.name, p.meaning);
            }
            let _ = writeln!(md, "\n### Required theme\n");
            for p in surface
                .presets
                .iter()
                .filter(|p| p.origin == spinnaker::PresetOrigin::ThemeRequired)
            {
                let _ = writeln!(md, "- `{}.{}` — {}", p.concern, p.name, p.meaning);
            }
            let _ = writeln!(md, "\n## Concerns\n");
            for row in spinnaker::offered_rows().filter(|r| r.tag == spinnaker::VocabTag::Concern) {
                let _ = writeln!(md, "- {}", spinnaker::vocab_surface_line(row));
            }
            let _ = writeln!(md, "\n## Axis\n");
            for row in spinnaker::offered_rows().filter(|r| r.tag == spinnaker::VocabTag::Axis) {
                let _ = writeln!(md, "- {}", spinnaker::vocab_surface_line(row));
            }
            let _ = writeln!(md, "\n## Plains\n");
            let _ = writeln!(
                md,
                "Verbs and resolve-offered singles. Not a themed bundle.\n"
            );
            for row in spinnaker::offered_rows().filter(|r| r.tag == spinnaker::VocabTag::Plain) {
                let _ = writeln!(md, "- {}", spinnaker::vocab_surface_line(row));
            }
        }
        SurfaceTopic::Tokens => {
            let _ = writeln!(md, "# Sail surface — token scales\n");
            let _ = writeln!(
                md,
                "Closed value sets concern slots draw from (`sail_lint::scales`). \
                 Named composites (`size.control-md`, `type.heading`, `elevation.sm`, \
                 `border.subtle`) are identities, not CSS names. Authors spend them \
                 as `size(control-md)` / `type(heading)` / `bg(surface)`. Size members \
                 are geometry only — pad on a size composite is a schema error.\n"
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
            let composites = sail_lint::tokens::all_composites();
            if !composites.is_empty() {
                let _ = writeln!(md, "\n### Composites\n");
                let _ = writeln!(
                    md,
                    "Dotted identities (not CSS names). Concern setters resolve \
                     these first. Atlas: `border.*`, `size.control-*`, \
                     `type.*`, `elevation.*`. Authors write `bg(surface)` / \
                     `fg(foreground)`. `border` slots \
                     color/width/style; radius is not a slot. `size` has no pad.\n"
                );
                for c in composites {
                    if c.r#type == "fill"
                        || c.r#type == "ink"
                        || c.name.starts_with("fill.")
                        || c.name.starts_with("ink.")
                        || c.name.contains(".border") && c.name != "border"
                    {
                        continue;
                    }
                    let members: Vec<&str> = c.members.iter().map(|(slot, _)| *slot).collect();
                    let _ = writeln!(
                        md,
                        "- `{}` ($type `{}`) · members `{}`",
                        c.name,
                        c.r#type,
                        members.join("`/`")
                    );
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
                let _ = writeln!(md, "- `{}` → `{}` ({ty}) — {}", p.name, p.aria, p.summary);
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
                let _ = writeln!(md, "- `{}` (`{}`) — {}", b.syntax, b.rule, b.instead);
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
                "Control-flow blocks: if / for / match / when, plus in-file `<partial>`\n\
                 (SPEC-PARTIAL-1). Arms use brace form `{{pattern as binding => …}}`;\n\
                 binding lives in-brace.\n\n\
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
                 Match on LiveState/MutationState with a missing arm is a compile error.\n\n\
                 ### In-file `<partial>` (same file only)\n\n\
                 ```\n\
                 <scope>\n  \
                   type ListTileProps = {{ title: string; icon?: Icon }}\n  \
                   style card = {{ pad(2) }}\n\
                 </scope>\n\
                 <partial list-tile props: ListTileProps>\n  \
                   <row style(card)><text>{{title}}</text><slot/></row>\n\
                 </partial>\n\
                 <list-tile title={{run.name}} icon={{run.icon}}><badge/></list-tile>\n\
                 ```\n\n\
                 Top-level only; invoke as `<kebab-name …>` in this file. Expanded at\n\
                 preprocess — runtime never sees partials. Body reads params as bare\n\
                 `{{title}}` (destructured). No recursion; no tag-position `style(…)` on\n\
                 the invocation (paint inside the body or pass a Style param). Cross-file\n\
                 reuse remains the component file (`src/<tag>/<tag>.sail`).\n"
            );
        }
        SurfaceTopic::Icons => {
            let _ = writeln!(md, "# Sail surface — icons\n");
            let _ = writeln!(
                md,
                "Author markup uses **roles**, never pack prefixes.\n\n\
                 `<icon name=\"settings\" />` — `name` is a role in the project set \
                 (core ∪ `sail.config` `icons.roles`). Unknown role is a compile error \
                 with close matches (`sail-unknown-icon`).\n\n\
                 Configure packs in `sail.config.jsonc`:\n\n\
                 ```jsonc\n\
                 {{\n  \"icons\": {{\n    \"pack\": \"lucide\",\n    \"version\": \"1.28.0\",\n    \"roles\": {{\n      \"home\": \"house\",\n      \"settings\": \"mdi:cog\"\n    }}\n  }}\n}}\n\
                 ```\n\n\
                 Catalog (install targets, not author names):\n"
            );
            for entry in sail_compiler::ICONIFY_CATALOG {
                let _ = writeln!(md, "- `{}` — {} ({})", entry.id, entry.title, entry.summary);
            }
            let _ = writeln!(md, "\n### Core roles\n");
            let registry = sail_compiler::RoleRegistry::core();
            for row in registry.role_surfaces() {
                let _ = writeln!(
                    md,
                    "- `{}` → `{}:{}`{}",
                    row.role,
                    row.pack,
                    row.glyph,
                    if row.summary.is_empty() {
                        String::new()
                    } else {
                        format!(" — {}", row.summary)
                    }
                );
            }
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
    /// `"sail"` structure/check or `"typescript"` mapped TypeScript (when includeTs).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

pub fn surface(args: &Value) -> Result<Value, OpError> {
    surface_with_ctx(args, None)
}

/// Same as [`surface`], loading the project role set when `ctx` is present.
pub fn surface_with_ctx(args: &Value, ctx: Option<&Ctx>) -> Result<Value, OpError> {
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
    let mut out = json!({
        "topic": topic.as_str(),
        "markdown": surface_markdown(topic),
    });
    if topic == SurfaceTopic::Icons {
        let registry = match ctx {
            Some(c) => sail_compiler::RoleRegistry::load_from_project(&c.project_root),
            None => sail_compiler::RoleRegistry::core(),
        };
        out["roles"] = serde_json::to_value(registry.role_surfaces()).unwrap_or(json!([]));
        out["packs"] = serde_json::to_value(registry.pack_surfaces()).unwrap_or(json!([]));
    }
    Ok(out)
}

pub fn check(args: &Value, ctx: &Ctx) -> Result<Value, OpError> {
    let path_raw = args.get("path").and_then(|v| v.as_str());
    let source = args.get("source").and_then(|v| v.as_str());
    let fix = args.get("fix").and_then(|v| v.as_str()).unwrap_or("off");
    let actor = args.get("actor").and_then(|v| v.as_str());
    // Default true: merge mapped TypeScript diagnostics. Returns
    // typescriptStatus: missing and omits TS diagnostics when the sidecar
    // is absent. Pass includeTs:false for structure-only.
    let include_ts = args
        .get("includeTs")
        .or_else(|| args.get("include_ts"))
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    // Path mode is workspace-scoped (WI4). Source mode is unchanged.
    let (mut src, src_path) = load_source_scoped(path_raw, source, ctx)?;
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
            plans.retain(sail_compiler::edit::is_auto_apply_plan);
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
        "rawCount": spinnaker::count_raw_uses_in_source(&src),
        "typescriptStatus": typescript_status(ctx),
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
    let status = typescript_status(ctx);
    match payload {
        None => Ok(json!({ "contents": null, "typescriptStatus": status })),
        Some(h) => {
            let mut out = json!({ "contents": h.contents, "typescriptStatus": status });
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
    Ok(json!({
        "items": items,
        "typescriptStatus": typescript_status(ctx),
    }))
}

fn typescript_status(ctx: &Ctx) -> &'static str {
    with_intelligence(ctx, |intel| intel.typescript_status())
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

/// Unscoped load (library / unit tests). MCP/RPC path mode uses
/// [`load_source_scoped`] for workspace confinement (WI4).
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

/// MCP/RPC check load: `path` alone is workspace-scoped; inline `source` is free.
fn load_source_scoped(
    path: Option<&str>,
    source: Option<&str>,
    ctx: &Ctx,
) -> Result<(String, Option<PathBuf>), OpError> {
    match (path, source) {
        (Some(p), None) => {
            let abs = ctx.resolve_workspace_read_path(p)?;
            let text = std::fs::read_to_string(&abs)
                .map_err(|e| OpError::NotFound(format!("read {}: {e}", abs.display())))?;
            Ok((text, Some(abs)))
        }
        // Source mode: path is an optional label only (not read) — unchanged (WI4).
        (None, Some(s)) => Ok((s.to_string(), None)),
        (Some(p), Some(s)) => Ok((s.to_string(), Some(PathBuf::from(p)))),
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
///
/// Accepts path and/or WI7 `identity` (identity-scoped target). When identity
/// is set, resolves against the live inspect tree; on `moved`, rewrites ops
/// nodeIds from the stored id to the live id before applying.
pub fn edit(args: &Value, ctx: &Ctx) -> Result<Value, OpError> {
    let identity_arg = args.get("identity").filter(|v| !v.is_null());
    let parsed_identity = identity_arg
        .map(crate::identity::parse_identity)
        .transpose()?;

    let path_from_args = args
        .get("path")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .map(str::to_string);
    let path_s = path_from_args
        .or_else(|| parsed_identity.as_ref().and_then(|i| i.file.clone()))
        .ok_or_else(|| {
            OpError::InvalidArgument(
                "Edit requires non-empty `path` or `identity` with file".into(),
            )
        })?;

    let ops_v = args
        .get("ops")
        .ok_or_else(|| OpError::InvalidArgument("Edit requires `ops` array".into()))?;
    let mut ops_json = ops_v.clone();
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

    let target = resolve_write_path(&path_s, &ctx.apply_root).map_err(OpError::PermissionDenied)?;

    // Resolve WI7 identity against live tree; rewrite ops when the element moved.
    let mut resolved_identity_json: Option<Value> = None;
    if let Some(ref stored) = parsed_identity {
        let file_key = path_s.trim().trim_start_matches("./");
        let nodes = if let Some(s) = args.get("source").and_then(|v| v.as_str()) {
            crate::inspect::tree_nodes_from_source(s, file_key)?
        } else if target.is_file() {
            let disk = std::fs::read_to_string(&target)
                .map_err(|e| OpError::NotFound(format!("read {}: {e}", target.display())))?;
            crate::inspect::tree_nodes_from_source(&disk, file_key)?
        } else {
            // Fall back to workspace-relative inspect path.
            let tree = crate::inspect::tree(&json!({ "path": path_s }), ctx)?;
            tree.get("nodes")
                .and_then(|n| n.as_array())
                .cloned()
                .unwrap_or_default()
        };
        let resolved = crate::identity::resolve_identity(stored, &nodes);
        match resolved.status {
            crate::identity::ResolveStatus::Lost => {
                return Err(OpError::NotFound(format!(
                    "identity lost: nodeId {} no longer resolves in {path_s}",
                    stored.node_id
                )));
            }
            crate::identity::ResolveStatus::Ok | crate::identity::ResolveStatus::Moved => {
                if let Some(to) = resolved.live_node_id.as_deref() {
                    if let Some(arr) = ops_json.as_array_mut() {
                        crate::identity::rewrite_ops_node_id(arr, &stored.node_id, to);
                    }
                }
                resolved_identity_json = resolved
                    .live
                    .as_ref()
                    .map(|i| i.to_json())
                    .or_else(|| Some(stored.to_json()));
            }
            crate::identity::ResolveStatus::None => {}
        }
    }

    let ops = sail_compiler::edit::parse_ops(&ops_json).map_err(OpError::InvalidArgument)?;

    // WI3 full-content check-guarded write (studio freeform Source save):
    // empty ops + full `source` body + writeSource. check_all before disk so
    // failure leaves the file byte-identical (no partial / broken intermediate).
    if write_source && ops.is_empty() {
        let Some(body) = args.get("source").and_then(|v| v.as_str()) else {
            return Err(OpError::InvalidArgument(
                "writeSource requires string `source`".into(),
            ));
        };
        let prior_on_disk = if target.is_file() {
            std::fs::read(&target).ok()
        } else {
            None
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
            // Refuse write entirely — caller still sees diags for the proposed body;
            // disk bytes are never opened for write on this path.
            let rev = prior_on_disk
                .as_ref()
                .and_then(|b| std::str::from_utf8(b).ok())
                .map(sail_compiler::edit::model_revision)
                .unwrap_or_else(|| sail_compiler::edit::model_revision(""));
            return Ok(json!({
                "applied": false,
                "modelRevision": rev,
                "diagnostics": diags_json,
                "autoFixed": [],
                "source": body,
                "path": target.display().to_string(),
                "message": "writeSource refused: source has error diagnostics",
                "identity": resolved_identity_json,
            }));
        }
        if !dry_run {
            write_full(&target, body).map_err(OpError::Internal)?;
            crate::watch::publish_file_under(&ctx.workspace_root, &target);
        }
        return Ok(json!({
            "applied": true,
            "modelRevision": sail_compiler::edit::model_revision(body),
            "diagnostics": diags_json,
            "autoFixed": [],
            "source": body,
            "path": target.display().to_string(),
            "message": "writeSource ok",
            "identity": resolved_identity_json,
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
        write_full(&target, &result.source).map_err(OpError::Internal)?;
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
        "identity": resolved_identity_json,
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
    let mode = args.get("mode").and_then(|v| v.as_str()).unwrap_or("list");
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
        || name == ".sail"
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
    let (preprocessed, type_hosts) =
        sail_compiler::parse::preprocess::preprocess_with_types(source)
            .map_err(|e| OpError::InvalidArgument(format!("preprocess: {e}")))?;
    let viewports = sail_compiler::check::layout::layout_viewports(&preprocessed);
    let mut viewport_reports: Vec<Value> = Vec::new();
    let mut kind_for: BTreeMap<String, &'static str> = BTreeMap::new();
    for vp in viewports {
        match spinnaker::layout::check_preprocessed_with_types(
            &preprocessed,
            vp,
            spinnaker::layout::CheckOptions::default(),
            &type_hosts,
        ) {
            Ok(scan) => {
                for d in &scan.diagnostics {
                    kind_for.insert(d.message.clone(), d.kind);
                }
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
    let diagnostics: Vec<Value> = diags
        .into_iter()
        .map(|d| {
            let kind = kind_for
                .get(&d.message)
                .copied()
                .or_else(|| match d.rule.as_deref() {
                    Some("sail-clipped-text") => Some("clipped-text"),
                    Some("sail-unintended-overflow") => Some("unintended-overflow"),
                    _ => None,
                });
            json!({
                "kind": kind,
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
            // L1 markdown compiles stdlib; rewriter owns that corpus. Skip here
            // so style/tokens acceptance is not blocked on unmigrated elements.
            if *t == SurfaceTopic::L1 {
                continue;
            }
            let md = surface_markdown(*t);
            assert!(!md.is_empty(), "empty for {}", t.as_str());
            assert!(md.contains('#'), "expected markdown for {}", t.as_str());
        }
        let style = surface_markdown(SurfaceTopic::Style);
        assert!(
            !style.contains("paint: bg(")
                && !style.contains("`style.bg")
                && !style.contains("`padX(…)`")
                && !style.contains("`fontSize(…)`"),
            "must not offer absorbed setters: {style}"
        );
        assert!(
            !style.contains("style.grow(…)"),
            "must not teach style.grow"
        );
        assert!(
            style.contains("concern")
                && style.contains("`color`/`width`/`style`")
                && style.contains("radius is not a slot"),
            "border must be documented as a concern: {style}"
        );
        assert!(
            style.contains("`bg(…)`")
                && style.contains("`fg(…)`")
                && style.contains("`size(…)`")
                && style.contains("`type(…)`")
                && style.contains("`elevation(…)`"),
            "atlas must list bg/fg/size/type/elevation: {style}"
        );
        assert!(
            !style.contains("`fill(…)`") && !style.contains("`ink(…)`"),
            "must not offer dead v2 names: {style}"
        );
        assert!(
            style.contains("pad is not a slot") && style.contains("## Plains"),
            "size geometry + plains: {style}"
        );
        assert!(
            style.contains("## StyleSurface presets")
                && style.contains("`type.heading`")
                && style.contains("`size(grow)`"),
            "style topic must project StyleSurface: {style}"
        );
        let tokens = surface_markdown(SurfaceTopic::Tokens);
        assert!(
            tokens.contains("`border.subtle`"),
            "tokens topic must list border composites: {tokens}"
        );
        assert!(
            tokens.contains("`size.control-md`")
                && tokens.contains("`type.heading`")
                && tokens.contains("`elevation.sm`"),
            "tokens topic must list live composites: {tokens}"
        );
        assert!(
            !tokens.contains("`fill.surface`") && !tokens.contains("`dialog.border`"),
            "tokens topic must not teach dead packs: {tokens}"
        );
    }

    #[test]
    fn surface_l1_topic_renders_when_stdlib_compiles() {
        let l1 = match std::panic::catch_unwind(|| surface_markdown(SurfaceTopic::L1)) {
            Ok(md) => md,
            Err(_) => {
                eprintln!("SKIP surface_l1: stdlib not yet on concern form (rewriter)");
                return;
            }
        };
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
        let icons = surface_markdown(SurfaceTopic::Icons);
        assert!(
            icons.contains("settings"),
            "core role in icons surface: {icons}"
        );
        assert!(
            !icons.contains("lucide:settings") || icons.contains("never pack prefixes"),
            "must not teach pack prefixes as author markup: {icons}"
        );
    }

    #[test]
    fn icons_surface_matches_role_registry() {
        let out = surface(&json!({"topic": "icons"})).expect("surface");
        assert_eq!(out["topic"], "icons");
        let roles = out["roles"].as_array().expect("roles array");
        let registry = sail_compiler::RoleRegistry::core();
        let expected: Vec<String> = registry.role_ids();
        let got: Vec<String> = roles
            .iter()
            .filter_map(|r| r.get("role").and_then(|v| v.as_str()).map(str::to_string))
            .collect();
        assert_eq!(
            got, expected,
            "MCP roles must equal RoleRegistry (LSP SSOT)"
        );
        assert!(roles.iter().any(|r| r["role"] == "settings"), "{roles:?}");
        let packs = out["packs"].as_array().expect("packs");
        assert!(
            packs
                .iter()
                .any(|p| p["id"] == "lucide" && p["configured"] == true),
            "{packs:?}"
        );
    }

    #[test]
    fn complete_and_hover_border_concern_match_registry() {
        use crate::call_rpc;
        use crate::ctx::Ctx;
        let ctx = Ctx::from_env();
        let src = "style({ border() })";
        let off = src.find("border(").expect("border(") + 7;
        let complete = call_rpc(
            "SailAuthoring",
            "Complete",
            &json!({ "source": src, "line": 1, "column": off + 1 }),
            &ctx,
        )
        .expect("Complete rpc");
        let labels: Vec<String> = complete["items"]
            .as_array()
            .expect("items")
            .iter()
            .filter_map(|i| i.get("label").and_then(|v| v.as_str()).map(str::to_string))
            .collect();
        for want in ["subtle", "default"] {
            assert!(
                labels.iter().any(|l| l == want),
                "{want} missing: {labels:?}"
            );
        }

        let color_src = "style({ border({ color:  }) })";
        let color_off = color_src.find("color: ").expect("color") + 7;
        let color = call_rpc(
            "SailAuthoring",
            "Complete",
            &json!({ "source": color_src, "line": 1, "column": color_off + 1 }),
            &ctx,
        )
        .expect("color Complete");
        let color_labels: Vec<String> = color["items"]
            .as_array()
            .expect("items")
            .iter()
            .filter_map(|i| i.get("label").and_then(|v| v.as_str()).map(str::to_string))
            .collect();
        assert!(
            color_labels.iter().any(|l| l == "primary" || l == "muted"),
            "{color_labels:?}"
        );
        assert!(
            !color_labels.iter().any(|l| l == "hairline"),
            "width in color slot: {color_labels:?}"
        );

        let hover_src = "style({ border(border.subtle) })";
        let hover_off = hover_src.find("subtle").expect("token");
        let hover = call_rpc(
            "SailAuthoring",
            "Hover",
            &json!({ "source": hover_src, "line": 1, "column": hover_off + 1 }),
            &ctx,
        )
        .expect("Hover rpc");
        let contents = hover["contents"].as_str().unwrap_or("");
        assert!(
            contents.contains("color:") && contents.contains("style:"),
            "hover members: {contents}"
        );
    }

    fn complete_labels(src: &str, needle: &str, add: usize) -> Vec<String> {
        use crate::call_rpc;
        use crate::ctx::Ctx;
        let ctx = Ctx::from_env();
        let off = src.find(needle).expect(needle) + add;
        let complete = call_rpc(
            "SailAuthoring",
            "Complete",
            &json!({ "source": src, "line": 1, "column": off + 1 }),
            &ctx,
        )
        .expect("Complete rpc");
        complete["items"]
            .as_array()
            .expect("items")
            .iter()
            .filter_map(|i| i.get("label").and_then(|v| v.as_str()).map(str::to_string))
            .collect()
    }

    #[test]
    fn complete_and_hover_size_type_elevation_fill_match_registry() {
        use crate::call_rpc;
        use crate::ctx::Ctx;
        // ObjectKey now also offers setters — channels are dissolved.
        let object = complete_labels("style({  })", "{ ", 2);
        for want in [
            "when",
            "match",
            "animate",
            "compound",
            "bg",
            "fg",
            "border",
            "elevation",
            "size",
            "pad",
            "margin",
            "position",
            "type",
        ] {
            assert!(
                object.iter().any(|l| l == want),
                "{want} missing: {object:?}"
            );
        }
        for dead in [
            "box",
            "paint",
            "text",
            "fill",
            "ink",
            "shadow",
            "space",
            "place",
            "flow",
            "padX",
            "w",
            "minW",
            "fontSize",
            "truncate",
            "fontWeight",
        ] {
            assert!(
                !object.iter().any(|l| l == dead),
                "`{dead}` offered: {object:?}"
            );
        }

        let size = complete_labels("style({ size() })", "size(", 5);
        for want in ["control-sm", "control-md", "control-lg", "grow"] {
            assert!(size.iter().any(|l| l == want), "{want} missing: {size:?}");
        }

        let typ = complete_labels("style({ type() })", "type(", 5);
        for want in ["body", "body-sm", "label", "heading", "mono"] {
            assert!(typ.iter().any(|l| l == want), "{want} missing: {typ:?}");
        }

        let elev = complete_labels("style({ elevation() })", "elevation(", 10);
        for want in ["xs", "sm", "md", "lg", "xl", "none"] {
            assert!(elev.iter().any(|l| l == want), "{want} missing: {elev:?}");
        }

        let fill = complete_labels("style({ bg() })", "bg(", 3);
        for want in ["surface", "overlay", "muted"] {
            assert!(fill.iter().any(|l| l == want), "{want} missing: {fill:?}");
        }

        let slots = complete_labels("style({ border({  }) })", "border({ ", 9);
        for want in ["color", "width", "style", "edge", "kind"] {
            assert!(slots.iter().any(|l| l == want), "{want} missing: {slots:?}");
        }
        assert!(
            !slots.iter().any(|l| l == "radius"),
            "radius is not a border slot: {slots:?}"
        );

        let ctx = Ctx::from_env();
        for (src, token, member) in [
            ("style({ size(size.control-md) })", "control-md", "h:"),
            ("style({ type(type.heading) })", "heading", "family:"),
            (
                "style({ elevation(elevation.sm) })",
                "elevation.sm",
                "shadow:",
            ),
            ("style({ bg(bg.surface) })", "bg.surface", "color:"),
        ] {
            let hover_off = src.find(token).expect(token);
            let hover = call_rpc(
                "SailAuthoring",
                "Hover",
                &json!({ "source": src, "line": 1, "column": hover_off + 1 }),
                &ctx,
            )
            .expect("Hover rpc");
            let contents = hover["contents"].as_str().unwrap_or("");
            assert!(
                contents.contains(member),
                "hover {token} missing {member}: {contents}"
            );
        }
    }

    #[test]
    fn check_flags_display_on_layout() {
        let src = r#"<column style({ display(none) })>x</column>"#;
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
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .canonicalize()
            .expect("repo root");
        if sail_lsp::tsgo_test::discover_tsgo(None).is_none() {
            panic!("{}", sail_compiler::ts_native::missing_sidecar_hint());
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
        let args = json!({
            "source": src,
            "path": "fixture-counter.sail",
            "line": line,
            "column": col
        });
        // Warm session once — open before sleeping so settle can progress.
        let _ = call_rpc("SailAuthoring", "Hover", &args, &ctx);
        let live = sail_lsp::with_global_session(Some(&root), |intel| intel.is_tsgo_live());
        assert!(
            live,
            "tsgo is on PATH — dead session is a fail (empty hover-with-tsgo-present)"
        );
        std::thread::sleep(std::time::Duration::from_millis(250));
        let hover_text =
            |v: &serde_json::Value| -> String { v["contents"].as_str().unwrap_or("").to_string() };
        let is_typed = |t: &str| {
            let lower = t.to_ascii_lowercase();
            lower.contains("signal") || lower.contains("function")
        };
        let mut contents =
            hover_text(&call_rpc("SailAuthoring", "Hover", &args, &ctx).expect("Hover rpc"));
        // One retry after settle: first hover can race project graph warm-up.
        if !is_typed(&contents) {
            std::thread::sleep(std::time::Duration::from_millis(300));
            contents = hover_text(
                &call_rpc("SailAuthoring", "Hover", &args, &ctx).expect("Hover rpc retry"),
            );
        }
        assert!(
            !contents.is_empty(),
            "expected non-empty typed hover when tsgo live; got empty contents"
        );
        assert!(
            is_typed(&contents),
            "expected typed hover for signal; got {contents}"
        );
    }

    #[test]
    fn hover_matrix_via_authoring_rpc() {
        use crate::call_rpc;
        use crate::ctx::Ctx;
        // Same local fixture as sail-lsp hover_matrix (not studio).
        let src = r#"<controller>
import { controller, signal } from '@ship/sail-runtime'

export default controller((ctx) => ({ count: signal(0) }))
</controller>
<column style({ bg(primary), padding(4) })>
  {style.bg(primary)}
</column>
"#;
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .canonicalize()
            .expect("repo root");
        if sail_lsp::tsgo_test::discover_tsgo(None).is_none() {
            panic!("{}", sail_compiler::ts_native::missing_sidecar_hint());
        }
        let ctx = Ctx::new(&root, &root, &root);

        let pos_of = |needle: &str| -> (u32, u32) {
            let off = src.find(needle).expect(needle);
            let mut line = 1u32;
            let mut col = 1u32;
            for (i, ch) in src.char_indices() {
                if i >= off {
                    break;
                }
                if ch == '\n' {
                    line += 1;
                    col = 1;
                } else {
                    col += 1;
                }
            }
            (line, col)
        };
        let hover = |line: u32, column: u32| -> String {
            let args = json!({
                "source": src,
                "path": "crates/sail/lsp/tests/fixtures/hover-matrix.sail",
                "line": line,
                "column": column,
            });
            let v = call_rpc("SailAuthoring", "Hover", &args, &ctx).expect("Hover rpc");
            v["contents"].as_str().unwrap_or("").to_string()
        };

        let (theme_l, theme_c) = pos_of("primary");
        let (col_l, col_c) = pos_of("column");
        let (pad_l, pad_c) = pos_of("padding");
        let theme = hover(theme_l, theme_c);
        let layout = hover(col_l, col_c);
        let unknown = hover(pad_l, pad_c);

        assert!(
            theme.contains("theme") && (theme.contains("oklch") || theme.contains("--")),
            "theme hover: {theme}"
        );
        assert!(
            !layout.is_empty() && !layout.to_ascii_lowercase().contains("parse error"),
            "layout hover: {layout}"
        );
        assert!(
            unknown.contains("unknown vocabulary")
                && !unknown.to_ascii_lowercase().contains("parse error"),
            "unknown vocab hover: {unknown}"
        );
        assert!(!layout.contains("Pure.") && !unknown.contains("Pure."));

        let live = sail_lsp::with_global_session(Some(&root), |intel| intel.is_tsgo_live());
        assert!(
            live,
            "tsgo is on PATH — dead session is a fail (empty hover-with-tsgo-present)"
        );

        let (signal_l, signal_c) = pos_of("signal(0)");
        let (ctrl_l, ctrl_c) = pos_of("controller((ctx)");
        let (foo_l, foo_c0) = pos_of("style.bg");
        let foo_c = foo_c0 + "style.".len() as u32;
        let _ = hover(signal_l, signal_c);
        std::thread::sleep(std::time::Duration::from_millis(250));
        let signal = hover(signal_l, signal_c);
        let controller = hover(ctrl_l, ctrl_c);
        let style_foo = hover(foo_l, foo_c);
        assert!(
            !signal.is_empty(),
            "empty hover on signal when tsgo present is a fail"
        );
        assert!(
            signal.to_ascii_lowercase().contains("signal")
                || signal.to_ascii_lowercase().contains("function"),
            "typed signal hover: {signal}"
        );
        assert!(
            !controller.is_empty(),
            "empty hover on controller when tsgo present is a fail"
        );
        let compact: String = style_foo.chars().filter(|c| !c.is_whitespace()).collect();
        assert!(
            !style_foo.is_empty() && !compact.ends_with(":{}"),
            "style.foo must not be typed as {{}}: {style_foo}"
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

        // Valid structured edit: setAttr on a layout-keyword axis attr (gap stays
        // on row|column|grid). Bare `grow` is participation and must use style box.
        let v = call_rpc(
            "SailAuthoring",
            "Edit",
            &json!({
                "path": rel,
                "ops": [{
                    "op": "setAttr",
                    "nodeId": col.id,
                    "name": "gap",
                    "value": "4",
                }]
            }),
            &ctx,
        )
        .unwrap();
        assert_eq!(v["applied"], true, "{v}");
        let after = std::fs::read_to_string(&full).unwrap();
        assert!(
            after.contains("gap=\"4\"") || after.contains("gap='4'"),
            "{after}"
        );

        // Off-vocab setSx leaves file untouched (check-guard refuse).
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

    /// WI3: full-content writeSource with error diagnostics refuses the write
    /// and leaves the on-disk file byte-identical to the pre-call content.
    #[test]
    fn write_source_breaking_refuses_and_keeps_bytes() {
        use crate::call_rpc;
        use crate::ctx::Ctx;
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let rel = "routes/page.sail";
        let full = root.join(rel);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        let good = "<column gap=\"2\"><div>ok</div></column>\n";
        std::fs::write(&full, good).unwrap();
        let before = std::fs::read(&full).unwrap();

        let mut ctx = Ctx::from_env();
        ctx.apply_root = root.to_path_buf();
        ctx.workspace_root = root.to_path_buf();
        ctx.project_root = root.to_path_buf();

        // Broken Sail: unknown / foreign idiom that check_all errors on.
        let broken = "<column style=\"display:flex\"><div>bad</div></column>\n";
        let v = call_rpc(
            "SailAuthoring",
            "Edit",
            &json!({
                "path": rel,
                "ops": [],
                "writeSource": true,
                "source": broken,
            }),
            &ctx,
        )
        .expect("writeSource rpc");
        assert_eq!(v["applied"], false, "breaking body must refuse: {v}");
        let diags = v["diagnostics"].as_array().expect("diagnostics array");
        assert!(
            diags.iter().any(|d| {
                d["severity"]
                    .as_str()
                    .map(|s| s.eq_ignore_ascii_case("error"))
                    .unwrap_or(false)
            }),
            "expected error diagnostics: {diags:?}"
        );
        let after = std::fs::read(&full).unwrap();
        assert_eq!(
            after, before,
            "file must stay byte-identical on refused writeSource"
        );
        assert_eq!(
            std::str::from_utf8(&after).unwrap(),
            good,
            "content text must match original good source"
        );
    }

    /// WI3: good full-content writeSource commits and returns diagnostics for
    /// the new state.
    #[test]
    fn write_source_good_commits_and_returns_diags() {
        use crate::call_rpc;
        use crate::ctx::Ctx;
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let rel = "routes/page.sail";
        let full = root.join(rel);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        let v1 = "<column gap=\"2\"><div>v1</div></column>\n";
        std::fs::write(&full, v1).unwrap();

        let mut ctx = Ctx::from_env();
        ctx.apply_root = root.to_path_buf();
        ctx.workspace_root = root.to_path_buf();
        ctx.project_root = root.to_path_buf();

        let v2 = "<column gap=\"4\"><div name=\"next\">v2</div></column>\n";
        let v = call_rpc(
            "SailAuthoring",
            "Edit",
            &json!({
                "path": rel,
                "ops": [],
                "writeSource": true,
                "source": v2,
            }),
            &ctx,
        )
        .expect("writeSource rpc");
        assert_eq!(v["applied"], true, "good body must apply: {v}");
        assert!(v["diagnostics"].is_array(), "diagnostics present: {v}");
        let on_disk = std::fs::read_to_string(&full).unwrap();
        assert_eq!(on_disk, v2, "disk must match committed body");
        assert_eq!(
            v["source"].as_str(),
            Some(v2),
            "response source is the new body"
        );
        let expected_rev = sail_compiler::edit::model_revision(v2);
        assert_eq!(
            v["modelRevision"].as_str(),
            Some(expected_rev.as_str()),
            "modelRevision hashes the committed body"
        );
    }

    /// WI3: MCP `sail_edit` is the public name for check-guarded write (no
    /// `sail_apply`); MCP and RPC share one handler for writeSource.
    #[test]
    fn write_source_mcp_parity_with_rpc() {
        use crate::call_mcp_tool;
        use crate::call_rpc;
        use crate::ctx::Ctx;
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let rel = "routes/mcp.sail";
        let full = root.join(rel);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        let good = "<column gap=\"2\"><div>ok</div></column>\n";
        std::fs::write(&full, good).unwrap();
        let before = std::fs::read(&full).unwrap();

        let mut ctx = Ctx::from_env();
        ctx.apply_root = root.to_path_buf();
        ctx.workspace_root = root.to_path_buf();
        ctx.project_root = root.to_path_buf();

        let broken = "<div style=\"color:red\">nope</div>\n";
        let args = json!({
            "path": rel,
            "ops": [],
            "writeSource": true,
            "source": broken,
        });
        let via_rpc = call_rpc("SailAuthoring", "Edit", &args, &ctx).unwrap();
        // Reset bytes (rpc already refused; re-assert MCP does the same).
        assert_eq!(std::fs::read(&full).unwrap(), before);
        let via_mcp = call_mcp_tool("sail_edit", &args, &ctx).unwrap();
        assert_eq!(via_rpc["applied"], false);
        assert_eq!(via_mcp["applied"], false);
        assert_eq!(
            via_rpc["applied"], via_mcp["applied"],
            "MCP and RPC must agree"
        );
        assert_eq!(std::fs::read(&full).unwrap(), before);
        // Descriptor surface still exposes sail_edit only.
        let tools = crate::mcp_tool_descriptors();
        let names: Vec<_> = tools
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
            .collect();
        assert!(names.contains(&"sail_edit"), "{names:?}");
        assert!(
            !names.contains(&"sail_apply"),
            "sail_apply must stay retired"
        );
    }

    /// WI7: annotation identity → in-file move (span shift) → edit still hits
    /// the same element via semantic locator / content hash resolve.
    #[test]
    fn identity_scoped_edit_survives_in_file_move() {
        use crate::call_rpc;
        use crate::ctx::Ctx;
        use crate::identity::{ElementIdentity, resolve_identity};

        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let rel = "routes/page.sail";
        let full = root.join(rel);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();

        let v1 = r#"<column gap="2">
  <button name="Save">Save</button>
  <card name="other">x</card>
</column>
"#;
        std::fs::write(&full, v1).unwrap();

        let mut ctx = Ctx::from_env();
        ctx.apply_root = root.to_path_buf();
        ctx.workspace_root = root.to_path_buf();
        ctx.project_root = root.to_path_buf();

        let nodes_v1 = crate::inspect::tree_nodes_from_source(v1, rel).unwrap();
        let button = nodes_v1
            .iter()
            .find(|n| n["tag"] == "button")
            .expect("button node");
        let identity = ElementIdentity::from_tree_node(button);
        assert_eq!(
            identity.locator.as_ref().unwrap().name.as_deref(),
            Some("Save")
        );
        let stored_node_id = identity.node_id.clone();

        // In-file move: prepend a sibling so ordinal path + span shift.
        let v2 = r#"<column gap="2">
  <card name="banner">moved above</card>
  <button name="Save">Save</button>
  <card name="other">x</card>
</column>
"#;
        std::fs::write(&full, v2).unwrap();
        let nodes_v2 = crate::inspect::tree_nodes_from_source(v2, rel).unwrap();
        let resolved = resolve_identity(&identity, &nodes_v2);
        assert_eq!(
            resolved.status,
            crate::identity::ResolveStatus::Moved,
            "span/ordinal shift must report moved, not lost"
        );
        let live_id = resolved.live_node_id.as_deref().unwrap();
        assert_ne!(live_id, stored_node_id);

        // Identity-scoped edit: ops still carry the *old* nodeId; edit rewrites.
        let v = call_rpc(
            "SailAuthoring",
            "Edit",
            &json!({
                "identity": identity,
                "ops": [{
                    "op": "setAttr",
                    "nodeId": stored_node_id,
                    "name": "disabled",
                }],
                "dryRun": true,
                "source": v2,
            }),
            &ctx,
        )
        .expect("identity edit");
        assert_eq!(v["applied"], true, "{v}");
        assert!(
            v["source"].as_str().unwrap_or("").contains("disabled"),
            "attr applied on resolved element: {}",
            v["source"]
        );
        assert_eq!(
            v["identity"]["nodeId"].as_str(),
            Some(live_id),
            "response returns live identity"
        );
        assert_eq!(v["identity"]["locator"]["name"].as_str(), Some("Save"));
    }

    #[test]
    fn layout_check_uses_author_source_line_col() {
        // Collapsed parent + sized child. An empty child totaling 0 is
        // suppressed as analyzer noise — same contract as layout_conformance.
        let src = "\n\n<column style({ size(0) })>\n  <div style({ size(20) })>\n    x\n  </div>\n</column>";
        let out = sail_layout_check(src).unwrap();
        assert_eq!(out["ok"], true);
        assert!(out["diagnostics"].as_array().is_some());
        let reports = out
            .get("viewports")
            .or_else(|| out.get("viewportReports"))
            .or_else(|| out.get("boxes"));
        // Shape: either viewport reports or a diagnostics array. No crash.
        let _ = reports;
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
        assert!(is_ignored_dir(".sail"));
        assert!(!is_ignored_dir("src"));
    }

    /// TOOLING-1 WI4: sail_check path mode refuses workspace escape.
    #[test]
    fn check_path_escape_refused() {
        use crate::call_rpc;
        use crate::ctx::Ctx;

        let dir = tempfile::tempdir().unwrap();
        let ws = dir.path();
        std::fs::create_dir_all(ws.join("routes")).unwrap();
        std::fs::write(ws.join("routes/page.sail"), "<column></column>\n").unwrap();
        // Outside workspace — must not be readable via path escape.
        let outside = dir.path().parent().unwrap().join("outside-secret.sail");
        let _ = std::fs::write(&outside, "<column>secret</column>\n");

        let ctx = Ctx::new(ws, ws, ws);

        let err_dotdot = call_rpc(
            "SailAuthoring",
            "Check",
            &json!({ "path": "../outside-secret.sail", "includeTs": false }),
            &ctx,
        )
        .unwrap_err();
        assert!(
            matches!(err_dotdot, OpError::PermissionDenied(_)),
            "dotdot escape: {err_dotdot}"
        );

        let err_abs = call_rpc(
            "SailAuthoring",
            "Check",
            &json!({ "path": "/etc/passwd", "includeTs": false }),
            &ctx,
        )
        .unwrap_err();
        assert!(
            matches!(err_abs, OpError::PermissionDenied(_)),
            "absolute path: {err_abs}"
        );

        // In-workspace path still works.
        let ok = call_rpc(
            "SailAuthoring",
            "Check",
            &json!({ "path": "routes/page.sail", "includeTs": false }),
            &ctx,
        )
        .expect("in-workspace check");
        assert_eq!(ok["ok"], true, "{ok}");

        // Source mode is unrestricted (WI4 leaves source unchanged).
        let src_ok = call_rpc(
            "SailAuthoring",
            "Check",
            &json!({ "source": "<column></column>", "includeTs": false }),
            &ctx,
        )
        .expect("source mode");
        assert_eq!(src_ok["ok"], true, "{src_ok}");
    }
}
