//! Compile-phase measurement sink used by `sail-compiler profile`.
//!
//! Inactive unless [`begin`] is called. Product compile stays byte-identical;
//! spans are timers around existing work. Not an env-var switch.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde::Serialize;

use crate::check::tscheck;
use crate::{CompileMode, CompileOpts, ComponentMeta, Policy, check_all, compile};

thread_local! {
    static SESSION: RefCell<Option<Session>> = const { RefCell::new(None) };
}

#[derive(Debug, Clone, Serialize)]
pub struct Sample {
    pub phase: String,
    pub file: Option<String>,
    pub wall_ms: f64,
    pub rss_kb: u64,
}

struct Session {
    file: Option<String>,
    samples: Vec<Sample>,
    peak_rss_kb: u64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct FileFeatures {
    pub bytes: usize,
    pub lines: usize,
    pub controller_bytes: usize,
    pub theme_eq: usize,
    pub open_tags: usize,
    pub preprocess_out_bytes: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileReport {
    pub path: String,
    pub tree: String,
    pub ok: bool,
    pub error: Option<String>,
    pub wall_ms: f64,
    pub phases_ms: BTreeMap<String, f64>,
    pub features: FileFeatures,
    pub pathology: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PhaseTotals {
    pub wall_ms: f64,
    pub files: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct TreeReport {
    pub files: usize,
    pub ok: usize,
    pub err: usize,
    pub wall_ms: f64,
    pub phases_ms: BTreeMap<String, f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TypeGateAnatomy {
    pub invocation_shape: String,
    pub incremental_program_reuse: bool,
    pub tsgo_invocations: u64,
    pub cache_hits: u64,
    pub probes: Vec<TypeGateProbe>,
    pub project_gates: Vec<ProjectGateProbe>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TypeGateProbe {
    pub path: String,
    pub cold_ms: f64,
    pub warm_cache_ms: f64,
    pub warm_process_miss_ms: f64,
    pub tsgo_ms_on_miss: f64,
    pub harness_overhead_ms: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectGateProbe {
    pub project: String,
    pub files: usize,
    pub wall_ms: f64,
    pub tsgo_invocations: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct LayoutAnatomy {
    pub viewports_per_file: usize,
    pub default_viewports: [[u32; 2]; 2],
    pub taffy_passes: usize,
    pub layout_ms: f64,
    pub per_viewport_ms: BTreeMap<String, f64>,
    pub canary_compare: Option<CanaryCompare>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CanaryCompare {
    pub canary_preprocess_ms: u64,
    pub canary_layout_ms: u64,
    pub canary_wall_ms: u64,
    pub note: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RamAnatomy {
    pub peak_rss_kb_compile: u64,
    pub rss_after_compile_kb: u64,
    pub per_phase_peak_rss_kb: BTreeMap<String, u64>,
    pub sail_dev: SailDevRss,
}

#[derive(Debug, Clone, Serialize)]
pub struct SailDevRss {
    pub attempted: bool,
    pub rss_kb: Option<u64>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct HmrMap {
    pub does_rerun: Vec<String>,
    pub should_rerun: Vec<String>,
    pub should_not_rerun: Vec<String>,
    pub probes: Vec<HmrProbe>,
    pub budget: HmrBudget,
}

#[derive(Debug, Clone, Serialize)]
pub struct HmrProbe {
    pub path: String,
    pub cold_compile_ms: f64,
    pub warm_same_bytes_ms: f64,
    pub mutated_ms: f64,
    pub plus_check_all_ms: f64,
    pub phases_on_mutate: BTreeMap<String, f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HmrBudget {
    pub target_p95_ms: u64,
    pub measured_single_file_p50_ms: f64,
    pub measured_single_file_p95_ms: f64,
    pub measured_non_megapage_p95_ms: f64,
    pub studio_green_p50_ms: f64,
    pub studio_green_p95_ms: f64,
    pub leftover_for_dev_gate_ms: f64,
    pub cite: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrorTaxonomy {
    pub total_err: usize,
    pub counts: BTreeMap<String, usize>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Hotspot {
    pub rank: usize,
    pub path: String,
    pub wall_ms: f64,
    pub dominant_phase: String,
    pub dominant_ms: f64,
    pub pathology: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProfileReport {
    pub captured: String,
    pub label: String,
    pub git_sha: String,
    pub wall_cap_secs: u64,
    pub mem_cap_bytes: u64,
    pub files: Vec<FileReport>,
    pub trees: BTreeMap<String, TreeReport>,
    pub phase_totals: BTreeMap<String, PhaseTotals>,
    pub type_gate: TypeGateAnatomy,
    pub layout: LayoutAnatomy,
    pub ram: RamAnatomy,
    pub tokens: TokensProbe,
    pub hmr: HmrMap,
    pub hotspots: Vec<Hotspot>,
    pub error_taxonomy: ErrorTaxonomy,
}

#[derive(Debug, Clone, Serialize)]
pub struct TokensProbe {
    pub wall_ms: f64,
    pub ok: bool,
    pub error: Option<String>,
    pub note: String,
}

struct TreeSpec {
    name: &'static str,
    rel: &'static str,
}

const TREES: &[TreeSpec] = &[
    TreeSpec {
        name: "site",
        rel: "apps/sail-site/routes",
    },
    TreeSpec {
        name: "site",
        rel: "apps/sail-site/src",
    },
    TreeSpec {
        name: "site",
        rel: "apps/sail-site/content",
    },
    TreeSpec {
        name: "studio",
        rel: "apps/studio/src",
    },
    TreeSpec {
        name: "studio",
        rel: "apps/studio/routes",
    },
    TreeSpec {
        name: "components",
        rel: "packages/components",
    },
    TreeSpec {
        name: "docs",
        rel: "packages/deck/docs",
    },
];

const SKIP_DIRS: &[&str] = &[
    "target",
    "node_modules",
    ".git",
    "dist",
    "build",
    ".sail",
    ".deck",
    "test",
    "tests",
];

/// Start recording samples on this thread.
pub fn begin() {
    SESSION.with(|s| {
        *s.borrow_mut() = Some(Session {
            file: None,
            samples: Vec::new(),
            peak_rss_kb: rss_kb(),
        });
    });
}

pub fn set_file(path: Option<&Path>) {
    SESSION.with(|s| {
        if let Some(sess) = s.borrow_mut().as_mut() {
            sess.file = path.map(|p| p.to_string_lossy().into_owned());
        }
    });
}

pub fn take() -> Vec<Sample> {
    SESSION.with(|s| {
        s.borrow_mut()
            .as_mut()
            .map(|sess| std::mem::take(&mut sess.samples))
            .unwrap_or_default()
    })
}

pub fn peak_rss_kb() -> u64 {
    SESSION.with(|s| {
        s.borrow()
            .as_ref()
            .map(|sess| sess.peak_rss_kb)
            .unwrap_or(0)
    })
}

pub fn end() {
    SESSION.with(|s| *s.borrow_mut() = None);
}

fn active() -> bool {
    SESSION.with(|s| s.borrow().is_some())
}

/// Time `f` as `phase` when a session is active; otherwise call through.
pub fn span<R>(phase: impl Into<String>, f: impl FnOnce() -> R) -> R {
    if !active() {
        return f();
    }
    let phase = phase.into();
    let t = Instant::now();
    let out = f();
    let wall_ms = t.elapsed().as_secs_f64() * 1000.0;
    let rss = rss_kb();
    SESSION.with(|s| {
        if let Some(sess) = s.borrow_mut().as_mut() {
            if rss > sess.peak_rss_kb {
                sess.peak_rss_kb = rss;
            }
            sess.samples.push(Sample {
                phase,
                file: sess.file.clone(),
                wall_ms,
                rss_kb: rss,
            });
        }
    });
    out
}

pub fn rss_kb() -> u64 {
    let Ok(text) = fs::read_to_string("/proc/self/status") else {
        return 0;
    };
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("VmRSS:") {
            return rest
                .split_whitespace()
                .next()
                .and_then(|n| n.parse().ok())
                .unwrap_or(0);
        }
    }
    0
}

struct Discovered {
    path: PathBuf,
    rel: String,
    tree: &'static str,
}

/// Run the compiler phase profile and write JSON to `out`.
pub fn run(workspace: &Path, out: &Path, skip_dev_rss: bool) -> anyhow::Result<ProfileReport> {
    begin();
    let files = discover(workspace)?;
    let mut reports = Vec::with_capacity(files.len());
    let mut phase_peak: BTreeMap<String, u64> = BTreeMap::new();

    for file in &files {
        set_file(Some(&file.path));
        let source = match fs::read_to_string(&file.path) {
            Ok(s) => s,
            Err(err) => {
                reports.push(FileReport {
                    path: file.rel.clone(),
                    tree: file.tree.to_string(),
                    ok: false,
                    error: Some(err.to_string()),
                    wall_ms: 0.0,
                    phases_ms: BTreeMap::new(),
                    features: FileFeatures::default(),
                    pathology: Vec::new(),
                });
                continue;
            }
        };
        let features = features_of(&source);
        let t = Instant::now();
        let compiled = compile(&source, compile_opts(&file.path));
        let wall_ms = t.elapsed().as_secs_f64() * 1000.0;
        let samples = take();
        let phases_ms = sum_phases(&samples);
        for s in &samples {
            let e = phase_peak.entry(s.phase.clone()).or_insert(0);
            if s.rss_kb > *e {
                *e = s.rss_kb;
            }
        }
        let (ok, error) = match compiled {
            Ok(_) => (true, None),
            Err(e) => (false, Some(e.to_string())),
        };
        let pathology = classify(&features);
        reports.push(FileReport {
            path: file.rel.clone(),
            tree: file.tree.to_string(),
            ok,
            error,
            wall_ms,
            phases_ms,
            features,
            pathology,
        });
    }
    set_file(None);

    let type_gate = measure_type_gate(workspace, &files, &reports);
    let layout = measure_layout(workspace, &reports);
    let tokens = measure_tokens(workspace);
    let hmr = measure_hmr(workspace, &files, &reports);
    let ram = RamAnatomy {
        peak_rss_kb_compile: peak_rss_kb(),
        rss_after_compile_kb: rss_kb(),
        per_phase_peak_rss_kb: phase_peak,
        sail_dev: measure_sail_dev(workspace, skip_dev_rss),
    };

    let mut trees: BTreeMap<String, TreeReport> = BTreeMap::new();
    let mut phase_totals: BTreeMap<String, PhaseTotals> = BTreeMap::new();
    for r in &reports {
        let tree = trees.entry(r.tree.clone()).or_insert(TreeReport {
            files: 0,
            ok: 0,
            err: 0,
            wall_ms: 0.0,
            phases_ms: BTreeMap::new(),
        });
        tree.files += 1;
        if r.ok {
            tree.ok += 1;
        } else {
            tree.err += 1;
        }
        tree.wall_ms += r.wall_ms;
        for (k, v) in &r.phases_ms {
            *tree.phases_ms.entry(k.clone()).or_insert(0.0) += *v;
            let tot = phase_totals.entry(k.clone()).or_insert(PhaseTotals {
                wall_ms: 0.0,
                files: 0,
            });
            tot.wall_ms += *v;
            tot.files += 1;
        }
    }

    let mut ranked = reports.clone();
    ranked.sort_by(|a, b| b.wall_ms.total_cmp(&a.wall_ms));
    let hotspots = ranked
        .iter()
        .take(10)
        .enumerate()
        .map(|(i, r)| {
            let (dominant_phase, dominant_ms) = dominant_inner_phase(&r.phases_ms);
            Hotspot {
                rank: i + 1,
                path: r.path.clone(),
                wall_ms: r.wall_ms,
                dominant_phase,
                dominant_ms,
                pathology: r.pathology.clone(),
            }
        })
        .collect();

    end();

    let git_sha = git_sha(workspace);
    let error_taxonomy = error_taxonomy(&reports);
    let report = ProfileReport {
        captured: chrono_today(),
        label: "compiler-perf-0".into(),
        git_sha,
        wall_cap_secs: 60,
        mem_cap_bytes: 2 * 1024 * 1024 * 1024,
        files: reports,
        trees,
        phase_totals,
        type_gate,
        layout,
        ram,
        tokens,
        hmr,
        hotspots,
        error_taxonomy,
    };

    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(out, serde_json::to_string_pretty(&report)? + "\n")?;
    Ok(report)
}

fn discover(workspace: &Path) -> anyhow::Result<Vec<Discovered>> {
    let mut out = Vec::new();
    let mut missing = Vec::new();
    for spec in TREES {
        let root = workspace.join(spec.rel);
        if !root.exists() {
            missing.push(spec.rel);
            continue;
        }
        let mut files = Vec::new();
        walk_sail(&root, &mut files);
        for path in files {
            let rel = path
                .strip_prefix(workspace)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            out.push(Discovered {
                path,
                rel,
                tree: spec.name,
            });
        }
    }
    if !missing.is_empty() {
        anyhow::bail!("corpus roots missing: {}", missing.join(", "));
    }
    out.sort_by(|a, b| a.rel.cmp(&b.rel));
    out.dedup_by(|a, b| a.rel == b.rel);
    Ok(out)
}

fn walk_sail(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|s| s.to_str())
                && SKIP_DIRS.contains(&name)
            {
                continue;
            }
            walk_sail(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("sail") {
            out.push(path);
        }
    }
}

fn compile_opts(path: &Path) -> CompileOpts {
    let component_meta = component_meta_for(path);
    CompileOpts {
        setup_url: None,
        mode: CompileMode::Auto,
        source_path: Some(path.to_path_buf()),
        component_meta,
        route_params: None,
        emit_dts: true,
        scope_capabilities: Vec::new(),
        emit_component_imports: false,
        lint_overrides: Policy::overrides_for(path),
        ..Default::default()
    }
}

fn component_meta_for(path: &Path) -> Option<ComponentMeta> {
    // Same SSOT as the CLI operator: routes are not custom elements.
    if crate::project_map::classify(path, None).kind == crate::project_map::FileKind::Route {
        return None;
    }
    let stem = path.file_stem()?.to_str()?.to_string();
    let parent = path.parent()?.file_name()?.to_str()?;
    if stem == parent {
        return Some(ComponentMeta { name: stem });
    }
    let s = path.to_string_lossy();
    if s.contains("/src/") || s.contains("/packages/components/") {
        return Some(ComponentMeta { name: stem });
    }
    None
}

fn features_of(source: &str) -> FileFeatures {
    let controller_bytes = extract_controller_len(source);
    let preprocess_out_bytes = crate::parse::preprocess::preprocess(source)
        .map(|s| s.len())
        .unwrap_or(0);
    FileFeatures {
        bytes: source.len(),
        lines: source.lines().count(),
        controller_bytes,
        theme_eq: source.matches("theme=").count() + source.matches("theme =").count(),
        open_tags: source.matches('<').count(),
        preprocess_out_bytes,
    }
}

fn extract_controller_len(source: &str) -> usize {
    crate::parse::preprocess::extract_controller_block(source)
        .1
        .map(|s| s.len())
        .unwrap_or(0)
}

fn classify(f: &FileFeatures) -> Vec<String> {
    let mut out = Vec::new();
    if f.lines >= 400 || f.bytes >= 40_000 {
        out.push("megapage".into());
    }
    if f.controller_bytes >= 3_000 && f.controller_bytes * 2 > f.bytes {
        out.push("dense-controller".into());
    }
    if f.theme_eq >= 8 {
        out.push("theme-tree".into());
    }
    if f.open_tags >= 200 {
        out.push("dense-template".into());
    }
    if f.preprocess_out_bytes > 0 && f.bytes > 0 && f.preprocess_out_bytes >= f.bytes * 3 {
        out.push("stdlib-expansion".into());
    }
    if out.is_empty() {
        out.push("ordinary".into());
    }
    out
}

fn sum_phases(samples: &[Sample]) -> BTreeMap<String, f64> {
    let mut m = BTreeMap::new();
    for s in samples {
        *m.entry(s.phase.clone()).or_insert(0.0) += s.wall_ms;
    }
    m
}

fn measure_type_gate(
    workspace: &Path,
    files: &[Discovered],
    reports: &[FileReport],
) -> TypeGateAnatomy {
    let mut ranked: Vec<&FileReport> = reports.iter().filter(|r| r.ok).collect();
    ranked.sort_by(|a, b| b.wall_ms.total_cmp(&a.wall_ms));
    let mut picks: Vec<&FileReport> = Vec::new();
    if let Some(first) = ranked.first() {
        picks.push(*first);
    }
    if ranked.len() > 2 {
        picks.push(ranked[ranked.len() / 2]);
    }
    if let Some(last) = ranked.last()
        && picks.iter().all(|p| p.path != last.path)
    {
        picks.push(*last);
    }

    let mut probes = Vec::new();
    for pick in picks {
        let Some(disc) = files.iter().find(|f| f.rel == pick.path) else {
            continue;
        };
        let Ok(source) = fs::read_to_string(&disc.path) else {
            continue;
        };
        tscheck::clear_type_gate_cache();
        tscheck::reset_type_gate_servers();
        let before = tscheck::tsgo_invocation_count();
        let t = Instant::now();
        let _ = tscheck::collect_sail_type_gate(&source, Some(&disc.path));
        let cold = t.elapsed().as_secs_f64() * 1000.0;
        let after_cold = tscheck::tsgo_invocation_count();
        let tsgo_ms = take()
            .into_iter()
            .filter(|s| s.phase == "tsgo")
            .map(|s| s.wall_ms)
            .sum::<f64>();

        let t = Instant::now();
        let _ = tscheck::collect_sail_type_gate(&source, Some(&disc.path));
        let warm_cache = t.elapsed().as_secs_f64() * 1000.0;
        let _ = take();

        // Same process, cache miss (one-byte append inside a comment-safe tail).
        let mutated = format!("{source}\n");
        let t = Instant::now();
        let _ = tscheck::collect_sail_type_gate(&mutated, Some(&disc.path));
        let miss = t.elapsed().as_secs_f64() * 1000.0;
        let miss_tsgo = take()
            .into_iter()
            .filter(|s| s.phase == "tsgo")
            .map(|s| s.wall_ms)
            .sum::<f64>();
        let _ = after_cold.saturating_sub(before);
        probes.push(TypeGateProbe {
            path: pick.path.clone(),
            cold_ms: cold,
            warm_cache_ms: warm_cache,
            warm_process_miss_ms: miss,
            tsgo_ms_on_miss: miss_tsgo,
            harness_overhead_ms: (miss - miss_tsgo).max(0.0),
        });
        let _ = tsgo_ms;
    }

    let mut project_gates = Vec::new();
    for rel in ["apps/studio", "apps/sail-site"] {
        let project = workspace.join(rel);
        if !project.is_dir() {
            continue;
        }
        let ts_files = collect_ts_sources(&project);
        let before = tscheck::tsgo_invocation_count();
        let t = Instant::now();
        let _ = tscheck::collect_project_type_gate(&project, &ts_files);
        let wall_ms = t.elapsed().as_secs_f64() * 1000.0;
        let inv = tscheck::tsgo_invocation_count().saturating_sub(before);
        let _ = take();
        project_gates.push(ProjectGateProbe {
            project: rel.into(),
            files: ts_files.len(),
            wall_ms,
            tsgo_invocations: inv,
        });
    }

    let incremental_program_reuse = probes.iter().any(|p| {
        p.cold_ms >= 10.0
            && p.warm_process_miss_ms > 0.0
            && p.warm_process_miss_ms < p.cold_ms * 0.5
    });

    TypeGateAnatomy {
        invocation_shape: "long-lived tsgo --lsp program per TypeEnv+project; .sail region miss is didChange, not a spawn; project .ts is one program invalidated by file identity + bytes".into(),
        incremental_program_reuse,
        tsgo_invocations: tscheck::tsgo_invocation_count(),
        cache_hits: tscheck::type_gate_cache_hits(),
        probes,
        project_gates,
    }
}

fn collect_ts_sources(project: &Path) -> Vec<(PathBuf, String)> {
    let mut out = Vec::new();
    walk_ts(project, project, &mut out);
    out
}

fn walk_ts(project: &Path, dir: &Path, out: &mut Vec<(PathBuf, String)>) {
    let skip = [
        "node_modules",
        "dist",
        ".sail",
        ".deck",
        "target",
        ".git",
        "test",
        "tests",
        "__tests__",
    ];
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if p.is_dir() {
            if skip.iter().any(|s| *s == name) {
                continue;
            }
            walk_ts(project, &p, out);
        } else if name.ends_with(".ts") && !name.ends_with(".d.ts") {
            if name.ends_with(".test.ts") || name.ends_with(".spec.ts") {
                continue;
            }
            let rel = p.strip_prefix(project).unwrap_or(&p);
            let rel_s = rel.to_string_lossy();
            let product = rel_s.contains("src/actions/")
                || rel_s.contains("src/compositions/")
                || rel_s.contains("src/stores/")
                || rel_s.contains("routes/")
                || (rel_s.starts_with("src/") && rel_s.ends_with(".ts"));
            if !product {
                continue;
            }
            if let Ok(src) = fs::read_to_string(&p) {
                out.push((p, src));
            }
        }
    }
}

fn measure_layout(workspace: &Path, reports: &[FileReport]) -> LayoutAnatomy {
    let mut layout_ms = 0.0;
    let mut per_viewport: BTreeMap<String, f64> = BTreeMap::new();
    let mut taffy_passes = 0usize;
    for r in reports {
        if let Some(v) = r.phases_ms.get("layout") {
            layout_ms += *v;
        }
        for (k, v) in &r.phases_ms {
            if let Some(rest) = k.strip_prefix("layout-viewport.") {
                *per_viewport.entry(rest.to_string()).or_insert(0.0) += *v;
                taffy_passes += 1;
            }
        }
    }
    let canary_compare = crate::check::layout_corpus::scan(workspace)
        .ok()
        .map(|r| CanaryCompare {
            canary_preprocess_ms: r.perf.preprocess_ms,
            canary_layout_ms: r.perf.layout_ms,
            canary_wall_ms: r.perf.total_wall_ms,
            note: "same driver as LAYOUT-1 canary; compare against compile-path layout_ms".into(),
        });
    LayoutAnatomy {
        viewports_per_file: 2,
        default_viewports: [[390, 844], [1440, 900]],
        taffy_passes,
        layout_ms,
        per_viewport_ms: per_viewport,
        canary_compare,
    }
}

fn measure_tokens(workspace: &Path) -> TokensProbe {
    let model = workspace.join("packages/sail/tokens/model");
    if !model.is_dir() {
        return TokensProbe {
            wall_ms: 0.0,
            ok: false,
            error: Some("tokens model missing".into()),
            note: "token CSS is not regenerated on sail-compiler build / HMR; separate `tokens` command".into(),
        };
    }
    let tmp = std::env::temp_dir().join(format!("sail-perf-tokens-{}", std::process::id()));
    let _ = fs::create_dir_all(&tmp);
    let t = Instant::now();
    let result = crate::tokens_generate::generate(&model, &tmp);
    let wall_ms = t.elapsed().as_secs_f64() * 1000.0;
    let _ = fs::remove_dir_all(&tmp);
    match result {
        Ok(_) => TokensProbe {
            wall_ms,
            ok: true,
            error: None,
            note: "measured `tokens_generate::generate` into a temp dir; product HMR does not call this".into(),
        },
        Err(e) => TokensProbe {
            wall_ms,
            ok: false,
            error: Some(e.to_string()),
            note: "token generate failed; HMR still does not invoke this path".into(),
        },
    }
}

fn measure_hmr(_workspace: &Path, files: &[Discovered], reports: &[FileReport]) -> HmrMap {
    let mut ranked: Vec<&FileReport> = reports.iter().filter(|r| r.ok).collect();
    ranked.sort_by(|a, b| a.wall_ms.total_cmp(&b.wall_ms));
    let mut picks = Vec::new();
    if let Some(median) = ranked.get(ranked.len() / 2) {
        picks.push(*median);
    }
    if let Some(slow) = reports
        .iter()
        .filter(|r| r.ok)
        .max_by(|a, b| a.wall_ms.total_cmp(&b.wall_ms))
    {
        if picks.iter().all(|p| p.path != slow.path) {
            picks.push(slow);
        }
    }

    let mut probes = Vec::new();
    let mut mutate_walls = Vec::new();
    for pick in picks {
        let Some(disc) = files.iter().find(|f| f.rel == pick.path) else {
            continue;
        };
        let Ok(source) = fs::read_to_string(&disc.path) else {
            continue;
        };
        tscheck::clear_type_gate_cache();
        set_file(Some(&disc.path));
        let opts = compile_opts(&disc.path);

        let t = Instant::now();
        let _ = compile(&source, opts.clone());
        let cold = t.elapsed().as_secs_f64() * 1000.0;
        let _ = take();

        let t = Instant::now();
        let _ = compile(&source, opts.clone());
        let warm = t.elapsed().as_secs_f64() * 1000.0;
        let _ = take();

        let mutated = format!("{source}\n");
        let t = Instant::now();
        let _ = compile(&mutated, opts.clone());
        let mutated_ms = t.elapsed().as_secs_f64() * 1000.0;
        let phases = sum_phases(&take());

        let t = Instant::now();
        let _ = check_all(&mutated, Some(disc.path.as_path()), &[], None);
        let plus = t.elapsed().as_secs_f64() * 1000.0;
        let _ = take();

        mutate_walls.push(mutated_ms);
        probes.push(HmrProbe {
            path: pick.path.clone(),
            cold_compile_ms: cold,
            warm_same_bytes_ms: warm,
            mutated_ms,
            plus_check_all_ms: plus,
            phases_on_mutate: phases,
        });
    }
    set_file(None);

    let mut file_walls: Vec<f64> = reports.iter().filter(|r| r.ok).map(|r| r.wall_ms).collect();
    file_walls.sort_by(|a, b| a.total_cmp(b));
    let mut non_mega: Vec<f64> = reports
        .iter()
        .filter(|r| r.ok && !r.pathology.iter().any(|p| p == "megapage"))
        .map(|r| r.wall_ms)
        .collect();
    non_mega.sort_by(|a, b| a.total_cmp(b));
    let mut studio_ok: Vec<f64> = reports
        .iter()
        .filter(|r| r.ok && r.tree == "studio")
        .map(|r| r.wall_ms)
        .collect();
    studio_ok.sort_by(|a, b| a.total_cmp(b));
    let p50 = percentile(&file_walls, 0.50);
    let p95 = percentile(&file_walls, 0.95);
    let non_mega_p95 = percentile(&non_mega, 0.95);
    let studio_p50 = percentile(&studio_ok, 0.50);
    let studio_p95 = percentile(&studio_ok, 0.95);
    let leftover = (100.0 - p95).max(0.0);

    HmrMap {
        does_rerun: vec![
            "preprocess + parse of the changed .sail".into(),
            "check::run structural passes".into(),
            "lint + layout (2 viewports) + compose".into(),
            "per-file type-gate (reused tsgo program; cache miss on edit)".into(),
            "lower + emit".into(),
            "deck emit_build_warnings → check_all (layout/lint again; type-gate cache hit)".into(),
            "rolldown rebuild of graph-affected targets".into(),
        ],
        should_rerun: vec![
            "parse/preprocess of the changed file".into(),
            "structural + lint of the changed file".into(),
            "layout of the changed file".into(),
            "type-gate of the changed region with program reuse".into(),
            "emit of the changed file + importer graph".into(),
        ],
        should_not_rerun: vec![
            "token CSS regen".into(),
            "project type-gate from scratch".into(),
            "layout of unchanged files".into(),
            "tsgo cold-start per keystroke".into(),
            "second full collect() after a successful compile".into(),
        ],
        probes,
        budget: HmrBudget {
            target_p95_ms: 100,
            measured_single_file_p50_ms: p50,
            measured_single_file_p95_ms: p95,
            measured_non_megapage_p95_ms: non_mega_p95,
            studio_green_p50_ms: studio_p50,
            studio_green_p95_ms: studio_p95,
            leftover_for_dev_gate_ms: leftover,
            cite: format!(
                "PERF-0 HMR budget (DEV-GATE must cite): ARC p95<100ms incremental single-file. \
                 Measured product compile() p50={p50:.1}ms p95={p95:.1}ms (n={} green). \
                 Non-megapage p95={non_mega_p95:.1}ms. Studio green p50={studio_p50:.1}ms p95={studio_p95:.1}ms. \
                 Leftover vs 100ms p95: {leftover:.1}ms. \
                 Project type-gate and per-file tsgo numbers are in type_gate. \
                 Token CSS regen is not on the HMR path.",
                file_walls.len()
            ),
        },
    }
}

fn error_taxonomy(reports: &[FileReport]) -> ErrorTaxonomy {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut total_err = 0usize;
    for r in reports {
        if r.ok {
            continue;
        }
        total_err += 1;
        let msg = r.error.as_deref().unwrap_or("");
        let cat = if r.path.contains("tutorial/") {
            "tutorial-pedagogy"
        } else if msg.contains("block order") || msg.contains("must come before") {
            "block-order"
        } else if msg.contains("theme-owned") {
            "component-token-theme-owned"
        } else if msg.contains("not declared") {
            "unbound"
        } else {
            "other"
        };
        *counts.entry(cat.into()).or_insert(0) += 1;
    }
    ErrorTaxonomy {
        total_err,
        counts,
        note: "Phase timings on theme-owned red files are still valid: gate collect() runs compose+layout+type-gate before selecting the first Error.".into(),
    }
}

/// Parent spans that wrap other recorded phases. Ranking on these hides the
/// inner cost (compose vs layout vs type-gate).
fn is_wrapper_phase(name: &str) -> bool {
    matches!(name, "gate" | "parse-total" | "type-gate-cache")
        || name.starts_with("layout-viewport")
}

fn dominant_inner_phase(phases_ms: &BTreeMap<String, f64>) -> (String, f64) {
    phases_ms
        .iter()
        .filter(|(k, _)| !is_wrapper_phase(k))
        .max_by(|a, b| a.1.total_cmp(b.1))
        .map(|(k, v)| (k.clone(), *v))
        .unwrap_or_else(|| ("(none)".into(), 0.0))
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((sorted.len() as f64 - 1.0) * p).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

fn measure_sail_dev(workspace: &Path, skip: bool) -> SailDevRss {
    if skip {
        return SailDevRss {
            attempted: false,
            rss_kb: None,
            note: "skipped by --skip-dev-rss".into(),
        };
    }
    let sail = workspace.join("target/debug/sail");
    if !sail.is_file() {
        return SailDevRss {
            attempted: false,
            rss_kb: None,
            note: "target/debug/sail missing — RSS of sail-dev not sampled".into(),
        };
    }
    let studio = workspace.join("apps/studio");
    let mut child = match Command::new(&sail)
        .args([
            "dev",
            "--project",
            studio.to_str().unwrap_or("apps/studio"),
            "--addr",
            "127.0.0.1:18765",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            return SailDevRss {
                attempted: true,
                rss_kb: None,
                note: format!("spawn sail dev failed: {e}"),
            };
        }
    };
    let pid = child.id();
    let deadline = Instant::now() + Duration::from_secs(25);
    let mut peak = 0u64;
    while Instant::now() < deadline {
        if let Some(rss) = rss_of_pid(pid) {
            if rss > peak {
                peak = rss;
            }
        }
        if let Ok(Some(_)) = child.try_wait() {
            break;
        }
        std::thread::sleep(Duration::from_millis(250));
    }
    let _ = child.kill();
    let _ = child.wait();
    SailDevRss {
        attempted: true,
        rss_kb: (peak > 0).then_some(peak),
        note: "peak VmRSS of `sail dev apps/studio` over ~25s boot window (compiler/daemon only; browser tab not included)".into(),
    }
}

fn rss_of_pid(pid: u32) -> Option<u64> {
    let text = fs::read_to_string(format!("/proc/{pid}/status")).ok()?;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("VmRSS:") {
            return rest.split_whitespace().next()?.parse().ok();
        }
    }
    None
}

fn git_sha(workspace: &Path) -> String {
    Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(workspace)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".into())
}

fn chrono_today() -> String {
    // Avoid a chrono dep: local date from `date`.
    Command::new("date")
        .args(["+%Y-%m-%d"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inactive_span_is_passthrough() {
        end();
        assert_eq!(span("x", || 7), 7);
        assert!(take().is_empty());
    }

    #[test]
    fn active_span_records() {
        begin();
        let _ = span("parse", || 1);
        let samples = take();
        end();
        assert_eq!(samples.len(), 1);
        assert_eq!(samples[0].phase, "parse");
    }

    #[test]
    fn hotspot_dominant_phase_skips_wrapper_spans() {
        let mut phases = BTreeMap::new();
        phases.insert("gate".into(), 67_042.0);
        phases.insert("compose".into(), 61_798.0);
        phases.insert("layout".into(), 4_875.0);
        phases.insert("layout-viewport.390x844".into(), 1_600.0);
        phases.insert("parse-total".into(), 376.0);
        phases.insert("preprocess".into(), 344.0);
        phases.insert("type-gate".into(), 35.0);
        let (name, ms) = dominant_inner_phase(&phases);
        assert_eq!(name, "compose");
        assert_eq!(ms, 61_798.0);
    }
}
