//! Real-corpus layout canary: preprocess `.sail`, then scan experimental
//! `unintended-overflow` at the compiler viewport set.

use std::collections::BTreeMap;
use std::fs;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::Instant;

use serde::Serialize;

use crate::check::layout;
use crate::diagnostics::line_col;
use crate::parse::preprocess::{preprocess, preprocess_impl};
use sail_layout::canary::{self, KindFinding};
use sail_layout::{CheckOptions, OverflowGeometry};

const KIND: &str = "unintended-overflow";

const SKIP_DIRS: &[&str] = &["target", "node_modules", ".git", "dist", "build", ".deck"];

const UNTRIAGED_NOTE: &str = "not triaged";

/// Tree names the corpus walk must actually visit.
pub const TREE_NAMES: &[&str] = &["sail-site", "studio", "components", "docs"];

#[derive(Debug, Clone, Copy)]
struct TreeSpec {
    name: &'static str,
    rel: &'static str,
}

const TREES: &[TreeSpec] = &[
    TreeSpec {
        name: "sail-site",
        rel: "apps/sail-site/routes",
    },
    TreeSpec {
        name: "sail-site",
        rel: "apps/sail-site/src",
    },
    TreeSpec {
        name: "studio",
        rel: "apps/studio/src",
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

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CorpusSpan {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub col: usize,
    pub end_line: usize,
    pub end_col: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CorpusFinding {
    pub path: String,
    pub tree: String,
    pub kind: String,
    pub span_mapped: bool,
    pub span: CorpusSpan,
    pub preprocessed_span: [usize; 2],
    pub viewport: [u32; 2],
    pub element_tag: String,
    pub element_class: String,
    pub element_index: usize,
    pub geometry: Option<OverflowGeometry>,
    pub judgment: &'static str,
    pub note: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Counts {
    pub files_discovered: usize,
    pub files_scanned: usize,
    pub files_skipped: usize,
    pub files_preprocess_error: usize,
    pub files_layout_error: usize,
    pub unintended_overflow: usize,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct TreeStats {
    pub files_discovered: usize,
    pub files_scanned: usize,
    pub files_preprocess_error: usize,
    pub files_layout_error: usize,
    pub unintended_overflow: usize,
    pub layout_ms: u64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Perf {
    pub total_wall_ms: u64,
    pub preprocess_ms: u64,
    pub layout_ms: u64,
    pub sail_site_layout_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Promotion {
    pub decided: bool,
    pub experimental_only: bool,
    pub note: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Determinism {
    #[serde(rename = "match")]
    pub matched: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileError {
    pub path: String,
    pub stage: &'static str,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CorpusReport {
    pub kind_filter: &'static str,
    pub experimental: bool,
    pub stdlib_expand: bool,
    pub stdlib_error: Option<String>,
    pub promotion: Promotion,
    pub counts: Counts,
    pub trees: BTreeMap<String, TreeStats>,
    pub perf: Perf,
    pub determinism: Option<Determinism>,
    pub errors: Vec<FileError>,
    pub skipped: Vec<FileError>,
    pub findings: Vec<CorpusFinding>,
}

/// Walk the product corpus trees and scan `unintended-overflow`.
///
/// Findings are information. This function does not fail because overflow
/// exists, and it does not stop on the first preprocess/layout error.
pub fn scan(workspace: &Path) -> anyhow::Result<CorpusReport> {
    let wall = Instant::now();
    let files = discover_files(workspace)?;
    let opts = CheckOptions::experimental();
    let (stdlib_expand, stdlib_error) = stdlib_probe();

    let mut counts = Counts {
        files_discovered: files.len(),
        ..Counts::default()
    };
    let mut trees: BTreeMap<String, TreeStats> = BTreeMap::new();
    let mut errors = Vec::new();
    let mut skipped = Vec::new();
    let mut findings = Vec::new();
    let mut preprocess_ms = 0u64;
    let mut layout_ms = 0u64;
    let mut sail_site_layout_ms = 0u64;

    for file in &files {
        let tree = trees.entry(file.tree.to_string()).or_default();
        tree.files_discovered += 1;

        let source = match fs::read_to_string(&file.path) {
            Ok(s) => s,
            Err(err) => {
                counts.files_skipped += 1;
                skipped.push(FileError {
                    path: file.rel.clone(),
                    stage: "read",
                    message: err.to_string(),
                });
                continue;
            }
        };

        let pre_t = Instant::now();
        let preprocessed = match preprocess_corpus(&source) {
            Ok(p) => p,
            Err(err) => {
                counts.files_preprocess_error += 1;
                tree.files_preprocess_error += 1;
                errors.push(FileError {
                    path: file.rel.clone(),
                    stage: "preprocess",
                    message: err.to_string(),
                });
                continue;
            }
        };
        preprocess_ms += pre_t.elapsed().as_millis() as u64;

        let viewports = layout::layout_viewports(&preprocessed);
        let scan = canary::scan_preprocessed(&preprocessed, &viewports, KIND, opts);
        layout_ms += scan.layout_ms;
        tree.layout_ms += scan.layout_ms;
        if file.tree == "sail-site" {
            sail_site_layout_ms += scan.layout_ms;
        }

        if scan.viewports_ok == 0 {
            counts.files_layout_error += 1;
            tree.files_layout_error += 1;
            errors.push(FileError {
                path: file.rel.clone(),
                stage: "layout",
                message: scan.viewport_errors.join("; "),
            });
            continue;
        }
        if !scan.viewport_errors.is_empty() {
            errors.push(FileError {
                path: file.rel.clone(),
                stage: "layout-viewport",
                message: scan.viewport_errors.join("; "),
            });
        }

        counts.files_scanned += 1;
        tree.files_scanned += 1;

        for hit in scan.findings {
            let mapped = map_finding(&source, &preprocessed, &file.rel, file.tree, hit);
            tree.unintended_overflow += 1;
            counts.unintended_overflow += 1;
            findings.push(mapped);
        }
    }

    findings.sort_by(|a, b| {
        (
            a.path.as_str(),
            a.span.start,
            a.span.end,
            a.viewport,
            a.element_index,
        )
            .cmp(&(
                b.path.as_str(),
                b.span.start,
                b.span.end,
                b.viewport,
                b.element_index,
            ))
    });

    Ok(CorpusReport {
        kind_filter: KIND,
        experimental: true,
        stdlib_expand,
        stdlib_error,
        promotion: Promotion {
            decided: false,
            experimental_only: true,
            note: "experimental only; promotion not decided".into(),
        },
        counts,
        trees,
        perf: Perf {
            total_wall_ms: wall.elapsed().as_millis() as u64,
            preprocess_ms,
            layout_ms,
            sail_site_layout_ms,
        },
        determinism: None,
        errors,
        skipped,
        findings,
    })
}

pub fn canonical_findings_json(findings: &[CorpusFinding]) -> String {
    serde_json::to_string(findings).expect("corpus findings must serialize")
}

pub fn write_report(path: &Path, report: &CorpusReport) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(report)?;
    fs::write(path, json)?;
    Ok(())
}

pub fn default_out_path(workspace: &Path) -> PathBuf {
    if let Ok(p) = std::env::var("LAYOUT_CANARY_OUT") {
        return PathBuf::from(p);
    }
    workspace.join("target/layout-1-canary-findings.json")
}

struct Discovered {
    path: PathBuf,
    rel: String,
    tree: &'static str,
}

fn stdlib_probe() -> (bool, Option<String>) {
    static STATE: OnceLock<(bool, Option<String>)> = OnceLock::new();
    STATE
        .get_or_init(|| {
            let prev = std::panic::take_hook();
            std::panic::set_hook(Box::new(|_| {}));
            let probed = match catch_unwind(AssertUnwindSafe(|| preprocess("<div></div>"))) {
                Ok(Ok(_)) => (true, None),
                Ok(Err(err)) => (false, Some(err.to_string())),
                Err(payload) => (false, Some(panic_message(payload))),
            };
            std::panic::set_hook(prev);
            probed
        })
        .clone()
}

fn preprocess_corpus(source: &str) -> Result<String, String> {
    if stdlib_probe().0 {
        preprocess(source).map_err(|e| e.to_string())
    } else {
        preprocess_impl(source, false).map_err(|e| e.to_string())
    }
}

fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "stdlib preprocess panicked".into()
    }
}

fn discover_files(workspace: &Path) -> anyhow::Result<Vec<Discovered>> {
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
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                if SKIP_DIRS.contains(&name) {
                    continue;
                }
            }
            walk_sail(&path, out);
        } else if path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("sail"))
        {
            out.push(path);
        }
    }
}

fn map_finding(
    original: &str,
    preprocessed: &str,
    rel: &str,
    tree: &str,
    hit: KindFinding,
) -> CorpusFinding {
    let stub = sail_layout::Diagnostic {
        kind: KIND,
        severity: sail_layout::Severity::Warning,
        element_index: hit.element_index,
        element_tag: hit.element_tag.clone(),
        element_class: hit.element_class.clone(),
        message: String::new(),
        span: hit.span,
        geometry: hit.geometry,
    };
    let mapped = layout::original_byte_span(original, preprocessed, &stub);
    let (span_mapped, span) = match mapped {
        Some((start, end)) => {
            let (line, col) = line_col(original, start);
            let (end_line, end_col) = line_col(original, end);
            (
                true,
                CorpusSpan {
                    start,
                    end,
                    line,
                    col,
                    end_line,
                    end_col,
                },
            )
        }
        None => (
            false,
            CorpusSpan {
                start: 0,
                end: 0,
                line: 0,
                col: 0,
                end_line: 0,
                end_col: 0,
            },
        ),
    };
    CorpusFinding {
        path: rel.to_string(),
        tree: tree.to_string(),
        kind: hit.kind,
        span_mapped,
        span,
        preprocessed_span: [hit.span.0, hit.span.1],
        viewport: hit.viewport,
        element_tag: hit.element_tag,
        element_class: hit.element_class,
        element_index: hit.element_index,
        geometry: hit.geometry,
        judgment: "unclear",
        note: UNTRIAGED_NOTE.into(),
    }
}
