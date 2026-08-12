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

const PHASE_A_NOTE: &str = "phase A: auto-unclear; DEFECT/INTENDED triage is phase B";

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
    pub phase: &'static str,
    pub experimental_only: bool,
    pub note: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Determinism {
    pub run1_sha256: String,
    pub run2_sha256: String,
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
    let files = discover_files(workspace);
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
            phase: "A",
            experimental_only: true,
            note: "Promotion is phase B; this scan only gathers evidence.".into(),
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
    serde_json::to_string(findings).unwrap_or_else(|_| "[]".into())
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
    let helm = workspace
        .join("../../helm/data/missions/m-sail-tooling/scratch/layout-1-canary-findings.json");
    if helm.parent().is_some_and(|d| d.is_dir()) {
        helm
    } else {
        workspace.join("target/layout-1-canary-findings.json")
    }
}

pub fn sha256_hex(data: &[u8]) -> String {
    sha256(data).iter().map(|b| format!("{b:02x}")).collect()
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

fn discover_files(workspace: &Path) -> Vec<Discovered> {
    let mut out = Vec::new();
    for spec in TREES {
        let root = workspace.join(spec.rel);
        if !root.exists() {
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
    out.sort_by(|a, b| a.rel.cmp(&b.rel));
    out.dedup_by(|a, b| a.rel == b.rel);
    out
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
    let (start, end) = layout::original_byte_span(original, preprocessed, &stub);
    let (line, col) = line_col(original, start);
    let (end_line, end_col) = line_col(original, end);
    CorpusFinding {
        path: rel.to_string(),
        tree: tree.to_string(),
        kind: hit.kind,
        span: CorpusSpan {
            start,
            end,
            line,
            col,
            end_line,
            end_col,
        },
        preprocessed_span: [hit.span.0, hit.span.1],
        viewport: hit.viewport,
        element_tag: hit.element_tag,
        element_class: hit.element_class,
        element_index: hit.element_index,
        geometry: hit.geometry,
        judgment: "unclear",
        note: PHASE_A_NOTE.into(),
    }
}

fn sha256(data: &[u8]) -> [u8; 32] {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let bit_len = (data.len() as u64).saturating_mul(8);
    let mut padded = data.to_vec();
    padded.push(0x80);
    while (padded.len() % 64) != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in padded.chunks_exact(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes(chunk[i * 4..i * 4 + 4].try_into().unwrap());
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let mut a = h[0];
        let mut b = h[1];
        let mut c = h[2];
        let mut d = h[3];
        let mut e = h[4];
        let mut f = h[5];
        let mut g = h[6];
        let mut hh = h[7];
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }

    let mut out = [0u8; 32];
    for (i, word) in h.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    out
}

#[cfg(test)]
mod sha256_tests {
    use super::sha256_hex;

    #[test]
    fn sha256_empty_and_abc() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
