//! Type gate — `tsgo` red = sail red (JOB-E1).
//!
//! Doctrine: if TypeScript does not compile under the per-kind type
//! environment, sail does not compile. No report mode, no silent skip.
//! Product harness diagnostics become sail Errors: DOM-surface unnameables
//! rewrite to `sail-pure-region` / `sail-action-surface`; intrinsic type
//! errors become `sail-type-gate`.
//!
//! ## Design (synthesize-vs-harness)
//!
//! **Harness** (extends the existing props/variants bridge). Whole-region
//! checking writes the **full** authored `<controller>` body (module-level
//! `type` / `const` / helpers + `export default`, not a wrapped fragment)
//! into a temp module under a Pure (or Action) tsconfig and runs
//! `tsgo --noEmit` once. `<scope>` type aliases / type imports are
//! **appended** to that module (product IR: aliases hoist, so appending
//! preserves absolute controller spans). Span mapping is line-padding: the
//! harness preserves absolute 1-based line numbers from the `.sail` author
//! source so tsgo `(L,C)` maps straight back for controller body lines.
//!
//! The isolated `region.ts` is **not** a full project checkout, but has
//! **project-resolution parity** so missing modules hard-fail (no mask):
//! 1. **Faithful-enough ambient** — generic runtime HOF stubs, typed
//!    `@ship/*` module shapes (`Action`, `ControllerScope`, …), inject ABI.
//!    Non-precise packages get per-import open-any `declare module "exact"`
//!    (requested names); package roots prefer real `node_modules/@ship` types.
//!    TS2305/TS2614 are **not** suppressed (pattern ambient always reports
//!    `Module '"@ship/*"'` for both roots and subpaths).
//! 2. **Sibling tree** — relative imports resolve against the author file's
//!    real directory (open-any stubs when present; missing → TS2307).
//! 3. **`@/` paths** — `at_alias_paths()` + per-import stubs; missing → TS2307.
//!
//! We deliberately do **not** reuse `emit/synthetic` output here — that path
//! owns the template↔controller *scope contract* (`check_synthetic_controller`)
//! and filters a different diagnostic set. The type gate owns ambient law +
//! in-file type parity for the harnessed region.
//!
//! ## Resolution (STEP ZERO)
//!
//! Prefer the native `@typescript/native-preview-*` binary (same order as
//! `sail-lsp`), then `node_modules/.bin/tsgo`, then `tsgo` on PATH. Absent →
//! loud `sail-type-gate` error naming `bun install` / `@typescript/native-preview`.
//! Never a silent skip (including wasm32 — unsupported target fails loud).
//!
//! ## Tiers
//!
//! OnSave only (compile / `check_all` / MCP / LSP didSave). Never OnType.
//!
//! ## Process cache
//!
//! `collect_sail_type_gate` caches by `(source_path, source hash)` so deck
//! success (`compile` + `emit_build_warnings` → `check_all`) does not double
//! invoke tsgo for the same file contents in one process.

use std::collections::{BTreeSet, HashMap};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, OnceLock};

use crate::ambients::{TypeEnv, emit_action_observation_dts, lib_for_type_env, type_env_for_path};
use crate::diagnostics::{DiagSeverity, Diagnostic};
use crate::runtime_ambient::RUNTIME_AMBIENT;

/// One diagnostic parsed from tsgo's `path(L,C): error TSxxxx: message` output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TsDiag {
    pub file: String,
    pub line: usize,
    pub col: usize,
    pub code: String,
    pub message: String,
}

/// Resolved tsgo invocation.
#[derive(Debug, Clone)]
pub struct TsgoTool {
    /// Native binary path, OR `node` when using the JS entry.
    pub program: PathBuf,
    /// When `Some`, invoke as `node <js_entry> …` (legacy script entry).
    pub js_entry: Option<PathBuf>,
}

/// Rule ids owned by the type gate.
pub const RULE_PURE_REGION: &str = "sail-pure-region";
pub const RULE_ACTION_SURFACE: &str = "sail-action-surface";
pub const RULE_NO_TS_IGNORE: &str = "sail-no-ts-ignore";
pub const RULE_NO_TYPE_ERASE: &str = "sail-no-type-erase";
pub const RULE_TYPE_GATE: &str = "sail-type-gate";

/// Loud install fix when tsgo cannot be resolved.
pub const TSGO_MISSING_MSG: &str = "\
Type gate requires `tsgo` but none was found \
(searched node_modules/.bin/tsgo, @typescript/native-preview-*, and PATH). \
Install with `bun install` (provides `@typescript/native-preview`). \
If tsc does not compile, sail does not compile — the gate never skips.";

// ── tsgo resolution ────────────────────────────────────────────────────────

/// Discover tsgo starting from `start` (file or directory) walking up to root,
/// then CWD (when `start` is outside the workspace, e.g. `/tmp` fixtures), then
/// PATH. Returns `None` only when truly absent — callers must fail loud.
///
/// Paths are **canonicalized** when possible so later `Command` runs with
/// `current_dir(temp)` still find the binary (relative workspace paths would
/// otherwise ENOENT under the harness temp dir).
pub fn resolve_tsgo(start: Option<&Path>) -> Option<TsgoTool> {
    let mut starts: Vec<PathBuf> = Vec::new();
    if let Some(p) = start {
        let p = p.canonicalize().unwrap_or_else(|_| p.to_path_buf());
        starts.push(if p.is_file() {
            p.parent().unwrap_or(p.as_path()).to_path_buf()
        } else {
            p
        });
    }
    if let Ok(cwd) = std::env::current_dir() {
        if starts.first().map(|s| s.as_path()) != Some(cwd.as_path()) {
            starts.push(cwd);
        }
    }
    // CARGO_MANIFEST_DIR is unavailable at runtime; walk each start to root.

    for start_dir in starts {
        let mut dir = Some(start_dir);
        while let Some(d) = dir {
            if let Some(tool) = look_in_dir(&d) {
                return Some(tool);
            }
            dir = d.parent().map(|p| p.to_path_buf());
        }
    }

    if let Some(path) = which("tsgo") {
        return Some(TsgoTool {
            program: abs(path),
            js_entry: None,
        });
    }
    None
}

fn look_in_dir(d: &Path) -> Option<TsgoTool> {
    let bin = d.join("node_modules/.bin/tsgo");
    if bin.exists() {
        if let Some(nm) = bin.parent().and_then(|p| p.parent()) {
            if let Some(native) = resolve_native_under(nm) {
                return Some(TsgoTool {
                    program: abs(native),
                    js_entry: None,
                });
            }
        }
        // Shell script → run via node + package entry when possible.
        let js = d.join("node_modules/@typescript/native-preview/lib/tsgo.js");
        if js.exists() {
            if let Some(node) = which("node") {
                return Some(TsgoTool {
                    program: abs(node),
                    js_entry: Some(abs(js)),
                });
            }
        }
        return Some(TsgoTool {
            program: abs(bin),
            js_entry: None,
        });
    }
    if let Some(native) = platform_native_under(&d.join("node_modules")) {
        return Some(TsgoTool {
            program: abs(native),
            js_entry: None,
        });
    }
    None
}

fn abs(p: PathBuf) -> PathBuf {
    p.canonicalize().unwrap_or(p)
}

fn resolve_native_under(node_modules: &Path) -> Option<PathBuf> {
    platform_native_under(node_modules)
}

fn platform_native_under(node_modules: &Path) -> Option<PathBuf> {
    let os = std::env::consts::OS;
    let arch = match std::env::consts::ARCH {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        other => other,
    };
    let os = if os == "windows" { "win32" } else { os };
    let pkg = format!("@typescript/native-preview-{os}-{arch}");
    let mut exe = node_modules.join(&pkg).join("lib").join("tsgo");
    if cfg!(windows) {
        exe.set_extension("exe");
    }
    exe.exists().then_some(exe)
}

fn which(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn run_tsgo(tool: &TsgoTool, args: &[&str], cwd: &Path) -> std::io::Result<std::process::Output> {
    let mut cmd = Command::new(&tool.program);
    if let Some(js) = &tool.js_entry {
        cmd.arg(js);
    }
    cmd.args(args).current_dir(cwd).output()
}

// ── public type-gate entry (OnSave) ────────────────────────────────────────

/// Loud message when the type gate cannot run on this target.
pub const TSGO_UNSUPPORTED_TARGET_MSG: &str = "\
Type gate requires a native `tsgo` host and is not supported on this target \
(wasm32). Build/check on a native host — if tsc does not compile, sail does \
not compile; the gate never skips silently.";

/// Run the type gate for one `.sail` author source. Returns diagnostics in the
/// shared stream shape. OnSave only — caller enforces tier.
///
/// `author_source` is the **original** `.sail` text (controller still present).
/// `source_path` feeds kind → env resolution and tsgo discovery.
///
/// Results are cached per `(path, source hash)` for the process lifetime so a
/// second OnSave collect over the same bytes (e.g. deck `emit_build_warnings`)
/// does not spawn another tsgo.
pub fn collect_sail_type_gate(author_source: &str, source_path: Option<&Path>) -> Vec<Diagnostic> {
    #[cfg(target_arch = "wasm32")]
    {
        let line = extract_controller_region(author_source)
            .map(|r| r.open_line)
            .unwrap_or(1);
        return vec![Diagnostic {
            line,
            col: 1,
            end_line: line,
            end_col: 1,
            severity: DiagSeverity::Error,
            message: TSGO_UNSUPPORTED_TARGET_MSG.to_string(),
            rule: Some(RULE_TYPE_GATE.to_string()),
        }];
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let cache_key = cache_key_for(author_source, source_path);
        if let Some(hit) = gate_cache_get(&cache_key) {
            return hit;
        }
        let diags = collect_sail_type_gate_native(author_source, source_path);
        gate_cache_put(cache_key, diags.clone());
        diags
    }
}

fn cache_key_for(author_source: &str, source_path: Option<&Path>) -> (String, u64) {
    let path = source_path
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| "<no-path>".to_string());
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    author_source.hash(&mut hasher);
    (path, hasher.finish())
}

fn gate_cache() -> &'static Mutex<HashMap<(String, u64), Vec<Diagnostic>>> {
    static CACHE: OnceLock<Mutex<HashMap<(String, u64), Vec<Diagnostic>>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn gate_cache_get(key: &(String, u64)) -> Option<Vec<Diagnostic>> {
    gate_cache().lock().ok()?.get(key).cloned()
}

fn gate_cache_put(key: (String, u64), diags: Vec<Diagnostic>) {
    if let Ok(mut g) = gate_cache().lock() {
        // Bound cache size for long-lived LSP processes.
        if g.len() > 256 {
            g.clear();
        }
        g.insert(key, diags);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn collect_sail_type_gate_native(
    author_source: &str,
    source_path: Option<&Path>,
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    // E1c: @ts-ignore / @ts-nocheck / @ts-expect-error inside .sail TS regions.
    diags.extend(scan_ts_ignore(author_source));
    // E4a R1: type erasure ban in Pure.
    diags.extend(scan_type_erase(author_source, source_path));

    let Some(region) = extract_controller_region(author_source) else {
        // No controller region — nothing for the type env gate to check.
        return diags;
    };

    let env = source_path
        .map(|p| type_env_for_path(p, Some(author_source)))
        .unwrap_or(TypeEnv::Pure);
    // Framework packages keep DOM — type gate does not ban window there.
    if env == TypeEnv::Framework {
        return diags;
    }

    let Some(tool) = resolve_tsgo(source_path) else {
        diags.push(Diagnostic {
            line: region.open_line,
            col: 1,
            end_line: region.open_line,
            end_col: 1,
            severity: DiagSeverity::Error,
            message: TSGO_MISSING_MSG.to_string(),
            rule: Some(RULE_TYPE_GATE.to_string()),
        });
        return diags;
    };

    match typecheck_region_harness(&region, env, source_path, &tool, author_source) {
        Ok(ts_diags) => {
            for d in ts_diags {
                // Only product harness file — ignore ambient .d.ts noise.
                if !d.file.ends_with("region.ts") && !d.file.ends_with("region.tsx") {
                    continue;
                }
                if let Some(mapped) = map_region_diag(&d, env) {
                    diags.push(mapped);
                }
            }
        }
        Err(e) => {
            diags.push(Diagnostic {
                line: region.open_line,
                col: 1,
                end_line: region.open_line,
                end_col: 1,
                severity: DiagSeverity::Error,
                message: format!(
                    "Type gate failed to invoke tsgo ({e}). \
                     Install with `bun install` (`@typescript/native-preview`)."
                ),
                rule: Some(RULE_TYPE_GATE.to_string()),
            });
        }
    }

    diags
}

/// Project-scoped type gate over `.ts` sources (E1b).
///
/// Groups files by [`TypeEnv`] (Pure / Action; Framework skipped), writes
/// per-kind tsconfigs, runs **one** `tsgo -p` per env that has files.
/// Returns diagnostics with paths encoded in the message when multi-file
/// (`… [real/path.ts]`) so multi-file CLIs can print `path:line:col`.
pub fn collect_project_type_gate(
    project_root: &Path,
    ts_files: &[(PathBuf, String)],
) -> Vec<Diagnostic> {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (project_root, ts_files);
        return vec![Diagnostic {
            line: 1,
            col: 1,
            end_line: 1,
            end_col: 1,
            severity: DiagSeverity::Error,
            message: TSGO_UNSUPPORTED_TARGET_MSG.to_string(),
            rule: Some(RULE_TYPE_GATE.to_string()),
        }];
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        collect_project_type_gate_native(project_root, ts_files)
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn collect_project_type_gate_native(
    project_root: &Path,
    ts_files: &[(PathBuf, String)],
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    if ts_files.is_empty() {
        return diags;
    }

    let Some(tool) = resolve_tsgo(Some(project_root)) else {
        diags.push(Diagnostic {
            line: 1,
            col: 1,
            end_line: 1,
            end_col: 1,
            severity: DiagSeverity::Error,
            message: TSGO_MISSING_MSG.to_string(),
            rule: Some(RULE_TYPE_GATE.to_string()),
        });
        return diags;
    };

    // E1c + E4a R1: @ts-ignore and type-erase in product envs.
    for (path, src) in ts_files {
        let env = type_env_for_path(path, Some(src));
        if env == TypeEnv::Framework {
            continue;
        }
        for d in scan_ts_ignore(src) {
            let mut d = d;
            d.message = format!("{} ({})", d.message, path.display());
            diags.push(d);
        }
        for d in scan_type_erase(src, Some(path.as_path())) {
            let mut d = d;
            d.message = format!("{} ({})", d.message, path.display());
            diags.push(d);
        }
    }

    let mut pure: Vec<(&Path, &str)> = Vec::new();
    let mut action: Vec<(&Path, &str)> = Vec::new();
    for (path, src) in ts_files {
        match type_env_for_path(path, Some(src)) {
            TypeEnv::Pure => pure.push((path.as_path(), src.as_str())),
            TypeEnv::Action => action.push((path.as_path(), src.as_str())),
            TypeEnv::Framework => {}
        }
    }

    if let Err(e) = typecheck_project_env(project_root, TypeEnv::Pure, &pure, &tool, &mut diags) {
        diags.push(io_fail_diag(&e));
    }
    if let Err(e) = typecheck_project_env(project_root, TypeEnv::Action, &action, &tool, &mut diags)
    {
        diags.push(io_fail_diag(&e));
    }
    diags
}

fn io_fail_diag(e: &std::io::Error) -> Diagnostic {
    Diagnostic {
        line: 1,
        col: 1,
        end_line: 1,
        end_col: 1,
        severity: DiagSeverity::Error,
        message: format!(
            "Type gate failed to invoke tsgo ({e}). \
             Install with `bun install` (`@typescript/native-preview`)."
        ),
        rule: Some(RULE_TYPE_GATE.to_string()),
    }
}

// ── region harness ─────────────────────────────────────────────────────────

struct ControllerRegion {
    /// Body text between `<controller…>` and `</controller>`.
    /// Leading **blank** lines are dropped for line-padding; indentation on the
    /// first content line is preserved so tsgo columns match author columns.
    body: String,
    /// 1-based line of the first body character in the author source.
    body_start_line: usize,
    /// Byte offset of the first body character (after leading blank lines).
    #[allow(dead_code)]
    body_start_byte: usize,
    /// 1-based line of the `<controller` open tag (for missing-tsgo pointing).
    open_line: usize,
}

fn extract_controller_region(source: &str) -> Option<ControllerRegion> {
    let start = find_outside_comments(source, "<controller")?;
    let open_rel = source[start..].find('>')?;
    let body_start = start + open_rel + 1;
    let rel = source[body_start..].find("</controller>")?;
    let body_end = body_start + rel;
    let raw = &source[body_start..body_end];
    // Skip leading blank lines only — do **not** strip indentation on the first
    // content line (that would shift diagnostic columns left).
    let mut lead = 0usize;
    {
        let bytes = raw.as_bytes();
        let mut i = 0usize;
        while i < bytes.len() {
            let line_start = i;
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            let line_end = i; // exclusive, before newline
            let line = &raw[line_start..line_end];
            if line.chars().all(|c| c.is_whitespace()) {
                // Consume this blank line including its trailing newline if any.
                if i < bytes.len() && bytes[i] == b'\n' {
                    i += 1;
                }
                lead = i;
                continue;
            }
            break;
        }
    }
    let body = raw[lead..].trim_end().to_string();
    if body.is_empty() {
        return None;
    }
    let body_start_byte = body_start + lead;
    let body_start_line = line_of(source, body_start_byte);
    let open_line = line_of(source, start);
    Some(ControllerRegion {
        body,
        body_start_line,
        body_start_byte,
        open_line,
    })
}

/// Harvest `<scope>` type aliases / type imports for the controller harness.
/// Product threads these into the controller module (`Program.scope_types`);
/// the type gate must too so `type Row` in scope is visible to the controller.
fn extract_scope_types_text(source: &str) -> String {
    let mut out = String::new();
    let mut cursor = 0usize;
    while cursor < source.len() {
        let Some(rel) = find_outside_comments(&source[cursor..], "<scope") else {
            break;
        };
        let start = cursor + rel;
        let Some(open_rel) = source[start..].find('>') else {
            break;
        };
        let body_start = start + open_rel + 1;
        let Some(close_rel) = source[body_start..].find("</scope>") else {
            break;
        };
        let body_end = body_start + close_rel;
        let body = &source[body_start..body_end];
        match crate::parse::scope_block::parse_scope_block(
            body,
            crate::diagnostics::Span::point(start),
        ) {
            Ok(block) => {
                if !block.types.is_empty() {
                    out.push_str(block.types.trim_end());
                    out.push('\n');
                }
            }
            Err(_) => {
                // Incomplete / mid-edit scope: still surface bare type/import
                // lines so local aliases aren't dropped as harness noise.
                let harvested = harvest_typeish_lines(body);
                if !harvested.is_empty() {
                    out.push_str(&harvested);
                    if !out.ends_with('\n') {
                        out.push('\n');
                    }
                }
            }
        }
        cursor = body_end + "</scope>".len();
    }
    out
}

/// Best-effort type/import lines when full scope parse fails mid-edit.
fn harvest_typeish_lines(body: &str) -> String {
    let mut out = String::new();
    let mut i = 0usize;
    let bytes = body.as_bytes();
    while i < bytes.len() {
        // Skip trivia.
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        let rest = &body[i..];
        let is_type = rest.starts_with("type ")
            || rest.starts_with("export type ")
            || rest.starts_with("import type ")
            || rest.starts_with("import { type ")
            || (rest.starts_with("import ") && rest.contains(" type "));
        if !is_type {
            // Advance to next statement-ish boundary.
            if let Some(nl) = rest.find('\n') {
                i += nl + 1;
            } else {
                break;
            }
            continue;
        }
        // Consume until semicolon or double newline after balanced braces.
        let start = i;
        let mut depth = 0i32;
        let mut j = i;
        while j < bytes.len() {
            let c = bytes[j];
            match c {
                b'{' | b'<' => depth += 1,
                b'}' | b'>' => depth = depth.saturating_sub(1),
                b';' if depth == 0 => {
                    j += 1;
                    break;
                }
                b'\n' if depth == 0 => {
                    // Type alias without trailing `;` — take the line.
                    j += 1;
                    break;
                }
                _ => {}
            }
            j += 1;
        }
        out.push_str(body[start..j].trim());
        out.push('\n');
        i = j;
    }
    out
}

/// Comment-aware search — mirrors `parse::preprocess` for `<controller`.
/// Walks by **bytes** but only compares the needle at ASCII-safe starts so
/// multi-byte UTF-8 in comments never panics on a mid-char slice.
fn find_outside_comments(source: &str, needle: &str) -> Option<usize> {
    let bytes = source.as_bytes();
    let needle_b = needle.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'<' && i + 3 < bytes.len() && &bytes[i..i + 4] == b"<!--" {
            // Search for `-->` in remaining bytes without slicing as str mid-char.
            if let Some(rel) = find_bytes(&bytes[i + 4..], b"-->") {
                i = i + 4 + rel + 3;
                continue;
            }
            return None;
        }
        if bytes[i..].starts_with(needle_b) {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn find_bytes(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len()).position(|w| w == needle)
}

fn line_of(source: &str, byte: usize) -> usize {
    // Count newlines via bytes so multi-byte UTF-8 never trips a char-boundary panic.
    let end = byte.min(source.len());
    source.as_bytes()[..end]
        .iter()
        .filter(|&&b| b == b'\n')
        .count()
        + 1
}

fn typecheck_region_harness(
    region: &ControllerRegion,
    env: TypeEnv,
    source_path: Option<&Path>,
    tool: &TsgoTool,
    author_source: &str,
) -> std::io::Result<Vec<TsDiag>> {
    let out_dir = gate_temp_dir(source_path);
    std::fs::create_dir_all(&out_dir)?;

    // Scope type aliases / type imports — append so aliases hoist and
    // controller body line numbers stay absolute.
    let scope_types = extract_scope_types_text(author_source);

    // Line-padding harness: full controller module begins at its absolute
    // .sail line (types/consts/helpers + export default — never a fragment).
    let mut harness = String::new();
    for _ in 1..region.body_start_line {
        harness.push('\n');
    }
    harness.push_str(&region.body);
    if !harness.ends_with('\n') {
        harness.push('\n');
    }
    if !scope_types.is_empty() {
        harness.push_str("\n// <scope> types\n");
        harness.push_str(&scope_types);
        if !harness.ends_with('\n') {
            harness.push('\n');
        }
    }

    // Import resolution scans controller + scope types (scope may import type).
    let import_scan_body = if scope_types.is_empty() {
        region.body.clone()
    } else {
        format!("{}\n{}", region.body, scope_types)
    };

    // Sibling-tree parity (E4b R2): mirror author dir so `./` / `../` resolve.
    // Present siblings → open-any stubs (existence proven). Missing → TS2307.
    let project_root = source_path.and_then(find_project_root);
    let mirror_rel = mirror_region_rel(source_path, project_root.as_deref());
    let region_path = out_dir.join("mirror").join(&mirror_rel).join("region.ts");
    if let Some(parent) = region_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&region_path, &harness)?;
    let region_rel = if mirror_rel.is_empty() {
        "./mirror/region.ts".to_string()
    } else {
        format!("./mirror/{}/region.ts", mirror_rel.replace('\\', "/"))
    };

    std::fs::write(
        out_dir.join("sail-runtime-ambient.d.ts"),
        runtime_ambient_dts(),
    )?;

    let mut files = vec![
        format!("\"{region_rel}\""),
        "\"./sail-runtime-ambient.d.ts\"".to_string(),
    ];
    for stub_rel in
        materialize_relative_import_siblings(&import_scan_body, &out_dir, source_path, &mirror_rel)?
    {
        files.push(format!("\"{stub_rel}\""));
    }
    let (at_files, at_paths_block) =
        materialize_at_alias_stubs(&import_scan_body, &out_dir, project_root.as_deref())?;
    for f in at_files {
        files.push(format!("\"{f}\""));
    }
    // Non-precise `@ship/*` open-any: specific `declare module "exact-spec"` so
    // requested names work. Prefer real `node_modules/@ship` types for package
    // roots when present (named-export fidelity); open-any covers subpaths and
    // roots without install.
    let ship_nm = find_ship_node_modules(project_root.as_deref().or(source_path));
    let open_ship = emit_open_ship_package_ambient(&import_scan_body, ship_nm.is_some());
    if !open_ship.is_empty() {
        std::fs::write(out_dir.join("ship-open-stubs.d.ts"), open_ship)?;
        files.push("\"./ship-open-stubs.d.ts\"".to_string());
    }
    if env == TypeEnv::Action {
        std::fs::write(
            out_dir.join("action-observation.d.ts"),
            emit_action_observation_dts(),
        )?;
        files.push("\"./action-observation.d.ts\"".to_string());
    }

    let lib = lib_for_type_env(env);
    let lib_arr = lib
        .iter()
        .map(|e| format!("\"{e}\""))
        .collect::<Vec<_>>()
        .join(", ");

    let base_paths = if at_paths_block.is_empty() {
        region_harness_paths_block(project_root.as_deref(), &out_dir)
    } else {
        at_paths_block
    };
    let ship_paths = ship_nm
        .as_ref()
        .and_then(|nm| ship_packages_paths_entry(nm, &out_dir))
        .unwrap_or_default();
    let paths_block = merge_paths_blocks(&[base_paths, ship_paths]);

    // noImplicitAny true (product pure law). Open-any sibling stubs still make
    // callback params `any` (TS7006) — those are filtered in `map_region_diag`,
    // not by weakening the harness. Real type errors still hard-fail.
    let tsconfig = format!(
        r#"{{
  "compilerOptions": {{
    "target": "ESNext",
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "lib": [{lib_arr}],
    "strict": true,
    "noImplicitAny": true,
    "noEmit": true,
    "skipLibCheck": true,
    "noUncheckedIndexedAccess": true,
    "isolatedModules": true,
    "paths": {{
{paths_block}
    }}
  }},
  "files": [{files}]
}}
"#,
        files = files.join(", "),
    );
    let tsconfig_path = out_dir.join("tsconfig.json");
    std::fs::write(&tsconfig_path, tsconfig)?;

    let output = run_tsgo(tool, &["-p", "tsconfig.json", "--noEmit"], &out_dir);
    let result = match output {
        Ok(output) => {
            let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
            text.push_str(&String::from_utf8_lossy(&output.stderr));
            Ok(text.lines().filter_map(parse_line).collect())
        }
        Err(e) => Err(e),
    };
    let _ = std::fs::remove_dir_all(&out_dir);
    result
}

fn find_project_root(start: &Path) -> Option<PathBuf> {
    // Prefer the app package root (`package.json` + `routes`/`src`) over a
    // nested `sail.config.json` (e.g. `src/sail.config.json` is lint preset
    // only). `@/` alias targets are app-root-relative (`./src/stores/*`).
    let start = if start.is_file() {
        start.parent()?.to_path_buf()
    } else {
        start.to_path_buf()
    };
    let mut dir = Some(start.as_path());
    let mut nested_config: Option<PathBuf> = None;
    while let Some(d) = dir {
        if d.join("package.json").is_file() && (d.join("routes").is_dir() || d.join("src").is_dir())
        {
            return Some(d.to_path_buf());
        }
        if nested_config.is_none()
            && (d.join("sail.config.jsonc").is_file() || d.join("sail.config.json").is_file())
        {
            nested_config = Some(d.to_path_buf());
        }
        dir = d.parent();
    }
    nested_config
}

fn mirror_region_rel(source_path: Option<&Path>, project_root: Option<&Path>) -> String {
    let Some(src) = source_path else {
        return "__region".to_string();
    };
    let parent = if src.is_file() {
        src.parent().unwrap_or(src)
    } else {
        src
    };
    if let Some(root) = project_root {
        let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
        let parent = parent
            .canonicalize()
            .unwrap_or_else(|_| parent.to_path_buf());
        if let Ok(rel) = parent.strip_prefix(&root) {
            return rel.to_string_lossy().replace('\\', "/");
        }
    }
    src.file_stem()
        .and_then(|s| s.to_str())
        .map(|s| format!("__region/{s}"))
        .unwrap_or_else(|| "__region".to_string())
}

fn region_harness_paths_block(project_root: Option<&Path>, out_dir: &Path) -> String {
    let Some(root) = project_root else {
        return crate::project_map::at_alias_paths()
            .iter()
            .map(|m| format!("      \"{}\": [\"./__missing__/*\"]", m.pattern))
            .collect::<Vec<_>>()
            .join(",\n");
    };
    let root_canon = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let out_canon = out_dir
        .canonicalize()
        .unwrap_or_else(|_| out_dir.to_path_buf());
    crate::project_map::at_alias_paths()
        .iter()
        .map(|m| {
            let target = m.target.trim_start_matches("./");
            let target_path = root_canon.join(target.trim_end_matches("/*"));
            let rel = pathdiff_relative(&out_canon, &target_path)
                .unwrap_or_else(|| target_path.to_string_lossy().into_owned());
            let rel = rel.replace('\\', "/");
            if m.pattern.ends_with("/*") {
                format!("      \"{}\": [\"{rel}/*\"]", m.pattern)
            } else {
                format!("      \"{}\": [\"{rel}\"]", m.pattern)
            }
        })
        .collect::<Vec<_>>()
        .join(",\n")
}

fn pathdiff_relative(from: &Path, to: &Path) -> Option<String> {
    let from_c = from.canonicalize().ok()?;
    let to_c = to.canonicalize().ok().unwrap_or_else(|| to.to_path_buf());
    let from_comps: Vec<_> = from_c.components().collect();
    let to_comps: Vec<_> = to_c.components().collect();
    let mut i = 0;
    while i < from_comps.len() && i < to_comps.len() && from_comps[i] == to_comps[i] {
        i += 1;
    }
    let mut out = String::new();
    for _ in i..from_comps.len() {
        if !out.is_empty() {
            out.push('/');
        }
        out.push_str("..");
    }
    for c in &to_comps[i..] {
        if let std::path::Component::Normal(s) = c {
            if !out.is_empty() {
                out.push('/');
            }
            out.push_str(&s.to_string_lossy());
        }
    }
    if out.is_empty() {
        Some(".".to_string())
    } else {
        Some(out)
    }
}

fn runtime_ambient_dts() -> String {
    let mut out = String::from(
        "// GENERATED for type-gate harness — runtime ambient stubs (not product types).\n\
         // Faithful-enough vocabulary so in-file type errors + DOM unnameables fire,\n\
         // without module-resolution / stub-arity noise drowning legal controllers.\n",
    );
    for (authored, _, _) in RUNTIME_AMBIENT {
        // Generic + contextual typing for higher-order helpers:
        // - `(ctx) =>` callbacks avoid TS7006 under strict
        // - `signal<T>(…)` / `controller<T>(…)` avoid TS2558 arity gaps
        match *authored {
            "controller" | "defineStore" => {
                out.push_str(&format!(
                    "declare function {authored}<Ctx = any, SC = any>(fn: (ctx: Ctx) => SC, ...rest: any[]): any;\n"
                ));
            }
            "form" => {
                // form({ initial, validate, submit }) — options bag, not a HOF.
                // Infer T from `initial` so validate/submit destructured params are
                // typed (TS7031 under noImplicitAny). Mirrors product FormOptions.
                out.push_str(
                    "declare function form<T extends Record<string, unknown>>(opts: {\n\
                     \x20 initial: T | (() => T);\n\
                     \x20 validate?: (values: T) => Partial<Record<keyof T, string | null>>;\n\
                     \x20 submit?: (values: T) => unknown | Promise<unknown>;\n\
                     }, ...rest: any[]): {\n\
                     \x20 fields: { [K in keyof T]: any };\n\
                     \x20 values: () => T;\n\
                     \x20 errors: () => Partial<Record<keyof T, string | null>>;\n\
                     \x20 valid: () => boolean;\n\
                     \x20 dirty: () => boolean;\n\
                     \x20 pending: () => boolean;\n\
                     \x20 lastError: () => Error | null;\n\
                     \x20 reset: () => void;\n\
                     \x20 submit: (e?: { preventDefault(): void }) => Promise<void>;\n\
                     };\n",
                );
            }
            "signal" | "derived" | "query" | "mutation" | "filteredList" | "createVirtualizer"
            | "timeline" | "debounce" | "interval" => {
                out.push_str(&format!(
                    "declare function {authored}<T = any>(...args: any[]): any;\n"
                ));
            }
            "effect" | "mount" | "batch" | "untrack" | "onCleanup" | "onMount" => {
                out.push_str(&format!(
                    "declare function {authored}(fn: (...args: any[]) => any, ...rest: any[]): any;\n"
                ));
            }
            _ => {
                out.push_str(&format!(
                    "declare function {authored}<T = any>(...args: any[]): any;\n"
                ));
            }
        }
    }
    // Opaque host brand + host bag — free types so controllers need no DOM import.
    // SailHost is the capability firewall; HostBag is a structural bag for host
    // identity props / callbacks that components install (not DOM lib names).
    out.push_str(
        "type SailHost = { readonly __sailHostBrand?: unique symbol };
\
         type SailFile = { name: string; size: number; type: string };
\
         type HostBag = SailHost & {
\
           addEventListener(t: string, h: (...a: any[]) => void, o?: any): void;
\
           removeEventListener(t: string, h: (...a: any[]) => void, o?: any): void;
\
           closest(s: string): HostBag | null;
\
           querySelector(s: string): HostBag | null;
\
           querySelectorAll(s: string): ArrayLike<HostBag>;
\
           click(): void;
\
           focus(): void;
\
           blur(): void;
\
           appendChild?(c: unknown): unknown;
\
           removeChild?(c: unknown): unknown;
\
           replaceChildren?(...c: unknown[]): void;
\
           insertBefore?(n: unknown, ref: unknown): unknown;
\
           contains?(n: unknown): boolean;
\
           getBoundingClientRect?(): { x: number; y: number; width: number; height: number; top: number; left: number; right: number; bottom: number };
\
           getRootNode?(): unknown;
\
           matches?(s: string): boolean;
\
           dataset: Record<string, string | undefined>;
\
           style: Record<string, string | number | undefined>;
\
           classList: { add(...c: string[]): void; remove(...c: string[]): void; toggle(c: string, f?: boolean): boolean; contains(c: string): boolean };
\
           value: string;
\
           checked?: boolean;
\
           files: ArrayLike<SailFile>;
\
           textContent?: string | null;
\
           isConnected?: boolean;
\
           clientWidth: number;
\
           clientHeight: number;
\
           offsetWidth?: number;
\
           offsetHeight?: number;
\
           scrollTop: number;
\
           scrollLeft?: number;
\
           scrollWidth?: number;
\
           scrollHeight: number;
\
           id?: string;
\
           tagName?: string;
\
           isContentEditable?: boolean;
\
           parentElement?: HostBag | null;
\
           firstElementChild?: HostBag | null;
\
           scrollTo?(opts: { top?: number; left?: number; behavior?: string } | number, y?: number): void;
\
           emit?(name: string, detail?: unknown): void;
\
           onOpen?: (...a: any[]) => any;
\
           onChange?: (...a: any[]) => any;
\
           onSelect?: (...a: any[]) => any;
\
           onClick?: (...a: any[]) => any;
\
           onVariantChange?: (...a: any[]) => any;
\
           showModal(...a: any[]): any;
\
           close(...a: any[]): any;
\
           open: boolean;
\
           onSubmit?: (...a: any[]) => any;
\
           setPointerCapture(...a: any[]): any;
\
           disabled?: any;
\
           onErrorCaught?: (...a: any[]) => any;
\
           scrollIntoView(...a: any[]): any;
\
         };
\
         type DomEv = {
\
           key: string; button: number; clientX: number; clientY: number;
\
           preventDefault(): void; stopPropagation(): void;
\
           target?: HostBag | null; currentTarget?: HostBag | null; detail?: any;
\
           dataTransfer?: { files: ArrayLike<SailFile> };
\
           deltaY?: number; metaKey?: boolean; ctrlKey?: boolean; shiftKey?: boolean; altKey?: boolean;
\
           [k: string]: any;
\
         };
\
         declare function asSailHost(node: unknown): SailHost;
\
         declare function objectUrlFromBytes(data: ArrayBuffer | ArrayBufferView, type?: string): string;
\
         declare function revokeObjectUrl(url: string): void;
\
         declare function defineHostProperty(
\
           host: SailHost | HostBag,
\
           name: string,
\
           desc: { get: () => unknown; set?: (v: unknown) => void },
\
         ): () => void;
\
         declare function hostGetAttr(host: SailHost | HostBag | null | undefined, name: string): string | null;
\
         declare function hostHasAttr(host: SailHost | HostBag | null | undefined, name: string): boolean;
\
         declare function hostSetAttr(
\
           host: SailHost | HostBag | null | undefined,
\
           name: string,
\
           value: string | boolean | null | undefined,
\
         ): void;
\
         declare function observeContentBox(
\
           node: unknown,
\
           onBox: (width: number, height: number) => void,
\
         ): () => void;\
         declare function hostIsDarkMode(): boolean;\
         declare function hostCssVar(name: string): string;\
         declare function hostFindById(id: string): HostBag | null;\
         declare function hostCreateElement(tag: string): HostBag;\
         declare function hostObserveRootAttrs(attrs: string[], cb: () => void): () => void;\
         declare function hostListenDocument(event: string, handler: () => void): () => void;\
         declare function observeHostAttrs(node: unknown, attrs: string[], cb: () => void): () => void;
",
    );
    // Names commonly free in controllers that live in DOM lib in stock tsc
    // (not ESNext) but are NOT the Pure-region ban surface. Stub so they do
    // not flood the gate with non-product ambient gaps.
    for name in [
        "console",
        "queueMicrotask",
        "structuredClone",
        "URL",
        "URLSearchParams",
        "TextEncoder",
        "TextDecoder",
        "AbortController",
        "AbortSignal",
        "setTimeout",
        "clearTimeout",
        "setInterval",
        "clearInterval",
    ] {
        out.push_str(&format!("declare const {name}: any;\n"));
    }
    // Keep this file a **script** (no top-level import/export) so the
    // `declare function` runtime names stay global.
    //
    // Typed module shapes (not empty `declare module "x"`) so
    // `import type { Action }` is a type, not TS2709 namespace-as-type.
    // `@ship/*` covers inject ABI (`…/internal`), virtual, and other subpaths.
    out.push_str(
        r#"
declare module "@ship/*" {
  export type Action<P = unknown> = (
    node: any,
    param?: P,
  ) => { update?(param: P): void; destroy?(): void } | void;
  export type Signal<T = any> = ((...args: any[]) => any) & {
    set(...args: any[]): any;
  };
  export type Derived<T = any> = Signal<T>;
  export type ControllerScope<S = object> = S;
  export type StoreDef<T = any> = any;
  export type Theme = any;
  /** Opaque empty shape — not HTMLElement, not any (capability firewall). */
  export type SailHost = { readonly __sailHostBrand?: unique symbol };
  export type Host = any;
  export type BoundarySource = any;
  export const scheduleFrame: (fn: () => void) => void;
  export const asSailHost: (node: unknown) => SailHost;
  export function isSignal(v: unknown): boolean;
  export function onCleanup(...args: any[]): any;
  export const objectUrlFromBytes: (data: ArrayBuffer | ArrayBufferView, type?: string) => string;
  export const revokeObjectUrl: (url: string) => void;
  export const defineHostProperty: (
    host: SailHost,
    name: string,
    desc: { get: () => unknown; set?: (v: unknown) => void },
  ) => () => void;
  export const hostGetAttr: (host: SailHost | null | undefined, name: string) => string | null;
  export const hostHasAttr: (host: SailHost | null | undefined, name: string) => boolean;
  export const hostSetAttr: (
    host: SailHost | null | undefined,
    name: string,
    value: string | boolean | null | undefined,
  ) => void;
  export const observeContentBox: (
    node: unknown,
    onBox: (width: number, height: number) => void,
  ) => () => void;
  export type Virtual = any;
  export type VirtualItem = any;
  export type ViewMount = any;
  export function controller<T = any>(...args: any[]): any;
  export function signal<T = any>(...args: any[]): any;
  export function derived<T = any>(...args: any[]): any;
  export function effect<T = any>(...args: any[]): any;
  export const bindHtml: any;
  export const emit: any;
  export function mount(...args: any[]): any;
  export function onCleanup(...args: any[]): any;
  export function untrack<T = any>(...args: any[]): any;
  export function batch<T = any>(...args: any[]): any;
  export function form<T extends Record<string, unknown>>(opts: {
    initial: T | (() => T);
    validate?: (values: T) => Partial<Record<keyof T, string | null>>;
    submit?: (values: T) => unknown | Promise<unknown>;
  }, ...rest: any[]): any;
  export function defineStore<T = any>(...args: any[]): any;
  export function query<T = any>(...args: any[]): any;
  export function mutation<T = any>(...args: any[]): any;
  export const navigate: any;
  export const useStore: any;
  export const theme: any;
  export function createVirtualizer<T = any>(...args: any[]): any;
  const __default: any;
  export default __default;
}
declare module "@ship/sail-runtime" {
  export type Action<P = unknown> = (
    node: any,
    param?: P,
  ) => { update?(param: P): void; destroy?(): void } | void;
  export type Signal<T = any> = ((...args: any[]) => any) & {
    set(...args: any[]): any;
  };
  export type Derived<T = any> = Signal<T>;
  export type ControllerScope<S = object> = S;
  export type StoreDef<T = any> = any;
  export type Theme = any;
  /** Opaque empty shape — not HTMLElement, not any (capability firewall). */
  export type SailHost = { readonly __sailHostBrand?: unique symbol };
  export type Host = any;
  export type BoundarySource = any;
  export const scheduleFrame: (fn: () => void) => void;
  export const asSailHost: (node: unknown) => SailHost;
  export function isSignal(v: unknown): boolean;
  export function onCleanup(...args: any[]): any;
  export const objectUrlFromBytes: (data: ArrayBuffer | ArrayBufferView, type?: string) => string;
  export const revokeObjectUrl: (url: string) => void;
  export const defineHostProperty: (
    host: SailHost,
    name: string,
    desc: { get: () => unknown; set?: (v: unknown) => void },
  ) => () => void;
  export const hostGetAttr: (host: SailHost | null | undefined, name: string) => string | null;
  export const hostHasAttr: (host: SailHost | null | undefined, name: string) => boolean;
  export const hostSetAttr: (
    host: SailHost | null | undefined,
    name: string,
    value: string | boolean | null | undefined,
  ) => void;
  export const observeContentBox: (
    node: unknown,
    onBox: (width: number, height: number) => void,
  ) => () => void;
  export function controller<T = any>(...args: any[]): any;
  export function signal<T = any>(...args: any[]): any;
  export function derived<T = any>(...args: any[]): any;
  export function effect<T = any>(...args: any[]): any;
  export function batch<T = any>(...args: any[]): any;
  export function untrack<T = any>(...args: any[]): any;
  export function isSignal(v: unknown): boolean;
  // Real package surface used by app .ts modules (not free ambient only).
  export function mount(...args: any[]): any;
  export function mountAmbient(...args: any[]): any;
  export function navigate(...args: any[]): any;
  export function debounce(...args: any[]): any;
  export function interval(...args: any[]): any;
  export function persisted(...args: any[]): any;
  export const hotkeys: any;
  export const bindHtml: any;
  export const emit: any;
  const __default: any;
  export default __default;
}
declare module "@ship/sail-runtime/internal" {
  export function bindHtml(...args: any[]): any;
  export function emit(...args: any[]): any;
  export function controller(...args: any[]): any;
  export function signal(...args: any[]): any;
  export function derived(...args: any[]): any;
  export function effect(...args: any[]): any;
  export function mount(...args: any[]): any;
  export function onCleanup(...args: any[]): any;
  export function untrack(...args: any[]): any;
  export function batch(...args: any[]): any;
  export function form<T extends Record<string, unknown>>(opts: {
    initial: T | (() => T);
    validate?: (values: T) => Partial<Record<keyof T, string | null>>;
    submit?: (values: T) => unknown | Promise<unknown>;
  }, ...rest: any[]): any;
  export function defineStore(...args: any[]): any;
  export function query(...args: any[]): any;
  export function mutation(...args: any[]): any;
  export function navigate(...args: any[]): any;
  export function useStore(...args: any[]): any;
  export function theme(...args: any[]): any;
  export type Action<P = unknown> = (
    node: any,
    param?: P,
  ) => { update?(param: P): void; destroy?(): void } | void;
  export type Signal<T = any> = any;
  export type ControllerScope<S = object> = S;
  const __default: any;
  export default __default;
}
declare module "@ship/sail-dom" {
  const __default: any;
  export default __default;
}
declare module "@ship/sail-test" {
  const __default: any;
  export default __default;
}
// NOTE: no blanket `declare module "*.sail"`. Relative `.sail` imports are
// proven by sibling-tree materialization (present → open-any stub; missing →
// TS2307). A wildcard ambient would mask missing modules of the primary
// product relative form (`./foo.sail`).
"#,
    );
    out
}

/// Open `any` exports collected for one relative import specifier.
#[derive(Default)]
struct RelativeStub {
    named: BTreeSet<String>,
    has_default: bool,
    /// `import * as ns` — emit a namespace-friendly default object shape.
    has_namespace: bool,
}

/// Materialize relative imports from the **real sibling tree** (E4b R2).
/// Present on disk → open-any stub at mirrored path. Missing → no stub (TS2307).
fn materialize_relative_import_siblings(
    body: &str,
    out_dir: &Path,
    source_path: Option<&Path>,
    mirror_rel: &str,
) -> std::io::Result<Vec<String>> {
    let stubs = collect_relative_import_stubs(body);
    let author_dir = source_path.and_then(|p| {
        if p.is_file() {
            p.parent().map(|d| d.to_path_buf())
        } else {
            Some(p.to_path_buf())
        }
    });
    let mirror_region_dir = out_dir.join("mirror").join(mirror_rel);
    let mut files = Vec::new();
    for (spec, stub) in stubs {
        let real = author_dir
            .as_ref()
            .and_then(|dir| resolve_relative_module(dir, &spec));
        let Some(real_path) = real else {
            continue;
        };
        let dest = mirror_join(&mirror_region_dir, &spec);
        let Some(dest) = dest else {
            continue;
        };
        let dest = ensure_ts_dest(dest, &real_path);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        write_open_any_stub(&dest, &stub)?;
        if let Ok(rel) = dest.strip_prefix(out_dir) {
            let rel_s = rel.to_string_lossy().replace('\\', "/");
            files.push(format!("./{rel_s}"));
        }
    }
    Ok(files)
}

fn write_open_any_stub(dest: &Path, stub: &RelativeStub) -> std::io::Result<()> {
    let mut src =
        String::from("// GENERATED type-gate sibling stub — open any (existence proven).\n");
    for name in &stub.named {
        if is_ts_ident(name) {
            // Generic type params so `Table<T>` / `Foo<A, B>` don't TS2315.
            // `const x: any = function…` is both callable (createTable<T>()) and
            // indexable (EDGE_STROKE[kind]) — pure `function` decls reject index.
            src.push_str(&format!(
                "export type {name}<T = any, U = any, V = any> = any;\n\
                 export const {name}: any = function {name}(..._args: any[]): any {{ return undefined as any }};\n"
            ));
        }
    }
    if stub.has_default || stub.has_namespace || stub.named.is_empty() {
        src.push_str("const __default: any = undefined as any;\nexport default __default;\n");
    }
    if src.lines().count() <= 1 {
        src.push_str("export {};\n");
    }
    std::fs::write(dest, src)
}

/// Precise framework modules keep faithful ambients in `runtime_ambient_dts`.
fn is_precise_ship_module(spec: &str) -> bool {
    matches!(
        spec.trim(),
        "@ship/sail-runtime"
            | "@ship/sail-runtime/internal"
            | "@ship/sail-dom"
            | "@ship/sail-test"
    )
}

/// `@ship/<pkg>` (one segment) — not a component deep import.
fn is_ship_package_root(spec: &str) -> bool {
    let s = spec.trim();
    if !s.starts_with("@ship/") {
        return false;
    }
    !s["@ship/".len()..].contains('/')
}

/// Walk upward for `node_modules/@ship` (app or monorepo root).
fn find_ship_node_modules(start: Option<&Path>) -> Option<PathBuf> {
    let start = start?;
    let mut dir = if start.is_file() {
        start.parent().map(|p| p.to_path_buf())?
    } else {
        start.to_path_buf()
    };
    loop {
        let candidate = dir.join("node_modules").join("@ship");
        if candidate.is_dir() {
            return Some(candidate);
        }
        if !dir.pop() {
            return None;
        }
    }
}

/// `paths` entry mapping `@ship/*` → installed packages (real `.d.ts` fidelity).
fn ship_packages_paths_entry(ship_nm: &Path, out_dir: &Path) -> Option<String> {
    let out_canon = out_dir
        .canonicalize()
        .unwrap_or_else(|_| out_dir.to_path_buf());
    let nm_canon = ship_nm
        .canonicalize()
        .unwrap_or_else(|_| ship_nm.to_path_buf());
    let rel = pathdiff_relative(&out_canon, &nm_canon).unwrap_or_else(|| {
        nm_canon.to_string_lossy().replace('\\', "/")
    });
    let rel = rel.replace('\\', "/");
    Some(format!("      \"@ship/*\": [\"{rel}/*\"]"))
}

fn merge_paths_blocks(blocks: &[String]) -> String {
    blocks
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(",\n")
}

/// Whether this non-precise `@ship/…` import should get an open-any ambient.
/// Subpaths always (component packs lack types on deep exports). Package roots
/// skip when real `node_modules/@ship` types are available.
fn should_emit_open_ship_stub(spec: &str, has_ship_nm: bool) -> bool {
    let s = spec.trim();
    if !s.starts_with("@ship/") || is_precise_ship_module(s) {
        return false;
    }
    if is_ship_package_root(s) && has_ship_nm {
        return false;
    }
    true
}

/// Collect non-precise `@ship/*` imports and their requested binding shapes.
fn collect_ship_package_import_stubs(body: &str) -> HashMap<String, RelativeStub> {
    let mut map: HashMap<String, RelativeStub> = HashMap::new();
    let bytes = body.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'/' && i + 1 < bytes.len() {
            if bytes[i + 1] == b'/' {
                i += 2;
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
                continue;
            }
            if bytes[i + 1] == b'*' {
                i += 2;
                while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                i = (i + 2).min(bytes.len());
                continue;
            }
        }
        if rest_starts_with_kw(bytes, i, b"import") {
            let after_kw = i + 6;
            let j = skip_ws_bytes(bytes, after_kw);
            if j < bytes.len() && (bytes[j] == b'\'' || bytes[j] == b'"') {
                if let Some((spec, next)) = parse_string_lit(bytes, j) {
                    if spec.starts_with("@ship/") && !is_precise_ship_module(&spec) {
                        map.entry(spec).or_default();
                    }
                    i = next;
                    continue;
                }
            }
        }
        if rest_starts_with_kw(bytes, i, b"from") {
            let after_kw = i + 4;
            let j = skip_ws_bytes(bytes, after_kw);
            if j < bytes.len() && (bytes[j] == b'\'' || bytes[j] == b'"') {
                if let Some((spec, next)) = parse_string_lit(bytes, j) {
                    if spec.starts_with("@ship/") && !is_precise_ship_module(&spec) {
                        let clause = clause_before_from(body, i);
                        let entry = map.entry(spec).or_default();
                        fill_stub_from_clause(clause, entry);
                    }
                    i = next;
                    continue;
                }
            }
        }
        i += 1;
    }
    map
}

/// Emit specific `declare module "exact-spec"` open-any ambients for non-precise
/// `@ship/*` imports (requested names only). More specific than `declare module
/// "@ship/*"`, so TS resolves these instead of the pattern ambient.
fn emit_open_ship_package_ambient(body: &str, has_ship_nm: bool) -> String {
    let stubs = collect_ship_package_import_stubs(body);
    let mut specs: Vec<_> = stubs.into_iter().collect();
    specs.sort_by(|a, b| a.0.cmp(&b.0));
    let mut out = String::new();
    for (spec, stub) in specs {
        if !should_emit_open_ship_stub(&spec, has_ship_nm) {
            continue;
        }
        if out.is_empty() {
            out.push_str(
                "// GENERATED type-gate open-any for non-precise @ship packages \
                 (requested import names).\n",
            );
        }
        out.push_str(&format!("declare module \"{spec}\" {{\n"));
        for name in &stub.named {
            if is_ts_ident(name) {
                out.push_str(&format!(
                    "  export type {name}<T = any, U = any, V = any> = any;\n\
                      export const {name}: any;\n"
                ));
            }
        }
        if stub.has_default || stub.has_namespace || stub.named.is_empty() {
            out.push_str("  const __default: any;\n  export default __default;\n");
        }
        out.push_str("}\n");
    }
    out
}

fn resolve_relative_module(author_dir: &Path, spec: &str) -> Option<PathBuf> {
    let base = if spec.starts_with("./") || spec.starts_with("../") {
        author_dir.join(spec)
    } else {
        return None;
    };
    // Append extensions — do NOT use `with_extension` which clobbers multi-dot
    // stems (`sail-annotation.internals` → `.internals` gone).
    let base_s = base.to_string_lossy();
    let candidates = [
        base.clone(),
        PathBuf::from(format!("{base_s}.ts")),
        PathBuf::from(format!("{base_s}.tsx")),
        PathBuf::from(format!("{base_s}.sail")),
        base.join("index.ts"),
        base.join("index.tsx"),
    ];
    for c in candidates {
        if c.is_file() {
            return Some(c);
        }
    }
    None
}

fn mirror_join(base: &Path, spec: &str) -> Option<PathBuf> {
    if !(spec.starts_with("./") || spec.starts_with("../")) {
        return None;
    }
    let mut out = base.to_path_buf();
    for part in spec.split(['/', '\\']) {
        if part.is_empty() || part == "." {
            continue;
        }
        if part == ".." {
            if !out.pop() {
                return None;
            }
        } else {
            out.push(part);
        }
    }
    Some(out)
}

fn ensure_ts_dest(dest: PathBuf, real: &Path) -> PathBuf {
    if dest.extension().is_some() {
        if real.extension().and_then(|e| e.to_str()) == Some("sail") {
            return PathBuf::from(format!("{}.ts", dest.display()));
        }
        return dest;
    }
    dest.with_extension("ts")
}

/// `@/` imports with open-any named/default/namespace shape (same as relative).
fn collect_at_alias_import_stubs(body: &str) -> HashMap<String, RelativeStub> {
    let mut map: HashMap<String, RelativeStub> = HashMap::new();
    let bytes = body.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'/' && i + 1 < bytes.len() {
            if bytes[i + 1] == b'/' {
                i += 2;
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
                continue;
            }
            if bytes[i + 1] == b'*' {
                i += 2;
                while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                i = (i + 2).min(bytes.len());
                continue;
            }
        }
        if rest_starts_with_kw(bytes, i, b"from") {
            let after_kw = i + 4;
            let j = skip_ws_bytes(bytes, after_kw);
            if j < bytes.len() && (bytes[j] == b'\'' || bytes[j] == b'"') {
                if let Some((spec, next)) = parse_string_lit(bytes, j) {
                    if spec.starts_with("@/") {
                        let clause = clause_before_from(body, i);
                        let entry = map.entry(spec).or_default();
                        fill_stub_from_clause(clause, entry);
                    }
                    i = next;
                    continue;
                }
            }
        }
        i += 1;
    }
    map
}

fn materialize_at_alias_stubs(
    body: &str,
    out_dir: &Path,
    project_root: Option<&Path>,
) -> std::io::Result<(Vec<String>, String)> {
    let stubs = collect_at_alias_import_stubs(body);
    if stubs.is_empty() {
        return Ok((Vec::new(), String::new()));
    }
    let Some(root) = project_root else {
        let paths: Vec<String> = stubs
            .keys()
            .map(|s| format!("      \"{s}\": [\"./__missing_at_alias__\"]"))
            .collect();
        return Ok((Vec::new(), paths.join(",\n")));
    };
    let mut files = Vec::new();
    let mut path_entries = Vec::new();
    let mut i = 0usize;
    for (spec, stub) in stubs {
        let resolved = crate::project_map::resolve_at_alias(root, &spec);
        let exists = resolved.as_ref().is_some_and(|p| {
            p.is_file()
                || p.with_extension("ts").is_file()
                || p.with_extension("tsx").is_file()
                || p.join("index.ts").is_file()
        });
        if !exists {
            path_entries.push(format!("      \"{spec}\": [\"./__missing_at__/{i}\"]"));
            i += 1;
            continue;
        }
        let stub_name = format!("at_stub_{i}.ts");
        i += 1;
        let dest = out_dir.join("at").join(&stub_name);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        write_open_any_stub(&dest, &stub)?;
        files.push(format!("./at/{stub_name}"));
        path_entries.push(format!("      \"{spec}\": [\"./at/{stub_name}\"]"));
    }
    Ok((files, path_entries.join(",\n")))
}

fn is_ts_ident(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' || c == '$' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

/// Map `./markdown-render` → `markdown-render.ts`, `./x.sail` → `x.sail.ts`.
/// Rejects path escape (`..` climbing above harness root).
#[allow(dead_code)]
fn relative_stub_file_name(spec: &str) -> Option<String> {
    let stripped = spec
        .strip_prefix("./")
        .or_else(|| spec.strip_prefix(".\\"))
        .unwrap_or(spec);
    if stripped.is_empty() || stripped.starts_with('/') || stripped.starts_with('\\') {
        return None;
    }
    // Disallow escaping the harness dir via `..` segments.
    let mut depth = 0i32;
    for part in stripped.split(['/', '\\']) {
        if part.is_empty() || part == "." {
            continue;
        }
        if part == ".." {
            depth -= 1;
            if depth < 0 {
                return None;
            }
        } else {
            depth += 1;
        }
    }
    Some(format!("{stripped}.ts"))
}

/// Parse relative `import` / `export … from` specs in a controller body.
fn collect_relative_import_stubs(body: &str) -> HashMap<String, RelativeStub> {
    let mut map: HashMap<String, RelativeStub> = HashMap::new();
    let bytes = body.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // Skip // line comments and /* */ blocks so strings inside comments
        // never look like import specs.
        if bytes[i] == b'/' && i + 1 < bytes.len() {
            if bytes[i + 1] == b'/' {
                i += 2;
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
                continue;
            }
            if bytes[i + 1] == b'*' {
                i += 2;
                while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                i = (i + 2).min(bytes.len());
                continue;
            }
        }
        // Side-effect: import './x' / import "./x"
        if rest_starts_with_kw(bytes, i, b"import") {
            let after_kw = i + 6;
            let j = skip_ws_bytes(bytes, after_kw);
            if j < bytes.len() && (bytes[j] == b'\'' || bytes[j] == b'"') {
                if let Some((spec, next)) = parse_string_lit(bytes, j) {
                    if is_relative_spec(&spec) {
                        map.entry(spec).or_default();
                    }
                    i = next;
                    continue;
                }
            }
        }
        // from './x' after import/export clauses
        if rest_starts_with_kw(bytes, i, b"from") {
            let after_kw = i + 4;
            let j = skip_ws_bytes(bytes, after_kw);
            if j < bytes.len() && (bytes[j] == b'\'' || bytes[j] == b'"') {
                if let Some((spec, next)) = parse_string_lit(bytes, j) {
                    if is_relative_spec(&spec) {
                        // Walk back over whitespace to the import/export clause.
                        let clause = clause_before_from(body, i);
                        let entry = map.entry(spec).or_default();
                        fill_stub_from_clause(clause, entry);
                    }
                    i = next;
                    continue;
                }
            }
        }
        i += 1;
    }
    map
}

fn is_relative_spec(spec: &str) -> bool {
    spec.starts_with("./") || spec.starts_with("../") || spec.starts_with(".\\")
}

fn rest_starts_with_kw(bytes: &[u8], i: usize, kw: &[u8]) -> bool {
    if i + kw.len() > bytes.len() {
        return false;
    }
    if &bytes[i..i + kw.len()] != kw {
        return false;
    }
    // Left boundary: start or non-ident.
    if i > 0 {
        let c = bytes[i - 1];
        if c.is_ascii_alphanumeric() || c == b'_' || c == b'$' {
            return false;
        }
    }
    // Right boundary.
    if i + kw.len() < bytes.len() {
        let c = bytes[i + kw.len()];
        if c.is_ascii_alphanumeric() || c == b'_' || c == b'$' {
            return false;
        }
    }
    true
}

fn skip_ws_bytes(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    i
}

fn parse_string_lit(bytes: &[u8], i: usize) -> Option<(String, usize)> {
    let quote = *bytes.get(i)?;
    if quote != b'\'' && quote != b'"' {
        return None;
    }
    let mut j = i + 1;
    let mut out = String::new();
    while j < bytes.len() {
        let c = bytes[j];
        if c == quote {
            return Some((out, j + 1));
        }
        if c == b'\\' && j + 1 < bytes.len() {
            out.push(bytes[j + 1] as char);
            j += 2;
            continue;
        }
        if c == b'\n' {
            return None;
        }
        out.push(c as char);
        j += 1;
    }
    None
}

/// Text of the import/export clause immediately before a `from` keyword.
fn clause_before_from(body: &str, from_idx: usize) -> &str {
    let before = &body[..from_idx];
    // Find the nearest preceding `import` or `export` keyword.
    let mut start = 0;
    let b = before.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if rest_starts_with_kw(b, i, b"import") || rest_starts_with_kw(b, i, b"export") {
            start = i;
            i += 6;
            continue;
        }
        i += 1;
    }
    before[start..].trim()
}

fn fill_stub_from_clause(clause: &str, stub: &mut RelativeStub) {
    // clause examples:
    //   import { a, b as c, type D } from
    //   import def, { a } from
    //   import * as ns from
    //   import type { Scope } from
    //   export { x } from
    //   export type { Scope } from
    let s = clause
        .trim_start()
        .trim_start_matches("export")
        .trim_start()
        .trim_start_matches("import")
        .trim_start()
        .trim_start_matches("type")
        .trim_start();

    // namespace: * as name
    if let Some(rest) = s.strip_prefix('*') {
        let rest = rest.trim_start();
        if rest.starts_with("as") {
            stub.has_namespace = true;
            stub.has_default = true;
        }
        // still may have nothing else
        return;
    }

    // default + optional named: def, { … }  OR  def
    // named only: { … }
    if let Some(brace) = s.find('{') {
        let head = s[..brace].trim().trim_end_matches(',').trim();
        if !head.is_empty() && head != "type" {
            // default binding (possibly `type Default` — rare)
            let def = head.trim_start_matches("type").trim();
            if is_ts_ident(def) {
                stub.has_default = true;
            }
        }
        if let Some(end) = s[brace + 1..].find('}') {
            let inner = &s[brace + 1..brace + 1 + end];
            for part in inner.split(',') {
                let part = part.trim();
                if part.is_empty() {
                    continue;
                }
                // type Foo / Foo as Bar / default as X
                let mut tok = part.trim_start_matches("type").trim();
                if tok.is_empty() {
                    continue;
                }
                if tok.starts_with("default") {
                    stub.has_default = true;
                    continue;
                }
                // take left of `as`
                if let Some((left, _)) = tok.split_once(" as ") {
                    tok = left.trim();
                } else if let Some((left, _)) = tok.split_once("\tas\t") {
                    tok = left.trim();
                } else if let Some(idx) = tok.find(" as") {
                    // "Foo as Bar"
                    tok = tok[..idx].trim();
                }
                if is_ts_ident(tok) {
                    stub.named.insert(tok.to_string());
                }
            }
        }
    } else {
        // bare default: import Foo from './x'
        let def = s.trim().trim_end_matches(';').trim();
        if is_ts_ident(def) {
            stub.has_default = true;
        }
    }
}

fn gate_temp_dir(source_path: Option<&Path>) -> PathBuf {
    static COUNTER: OnceLock<std::sync::atomic::AtomicU64> = OnceLock::new();
    let n = COUNTER
        .get_or_init(|| std::sync::atomic::AtomicU64::new(0))
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let stem = source_path
        .and_then(|p| p.file_stem())
        .and_then(|s| s.to_str())
        .unwrap_or("anon");
    // Sanitize stem for path safety.
    let safe: String = stem
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    std::env::temp_dir().join(format!("sail-type-gate-{safe}-{n}"))
}

// ── project env batch ──────────────────────────────────────────────────────

fn typecheck_project_env(
    project_root: &Path,
    env: TypeEnv,
    files: &[(&Path, &str)],
    tool: &TsgoTool,
    diags: &mut Vec<Diagnostic>,
) -> std::io::Result<()> {
    if files.is_empty() {
        return Ok(());
    }
    let out_dir = project_root
        .join(".deck")
        .join(format!("type-gate-{}", env_label(env)));
    // Fresh tree each run so stale mirrors cannot leak.
    let _ = std::fs::remove_dir_all(&out_dir);
    std::fs::create_dir_all(&out_dir)?;

    // Materialize sources under out_dir preserving relative paths so relative
    // imports between product files still resolve (no f{i}.ts rename).
    let mut file_list: Vec<String> = Vec::new();
    let mut path_map: Vec<(String, PathBuf)> = Vec::new();
    for (i, (path, src)) in files.iter().enumerate() {
        let rel = match path.strip_prefix(project_root) {
            Ok(r) if !r.as_os_str().is_empty() => r.to_path_buf(),
            _ => PathBuf::from(format!(
                "_external/f{i}_{}",
                path.file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("anon.ts")
            )),
        };
        let dest = out_dir.join(&rel);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&dest, src)?;
        // tsconfig paths are relative to out_dir.
        let rel_s = rel.to_string_lossy().replace('\\', "/");
        file_list.push(format!("\"./{rel_s}\""));
        path_map.push((rel_s, path.to_path_buf()));
    }

    std::fs::write(
        out_dir.join("sail-runtime-ambient.d.ts"),
        runtime_ambient_dts(),
    )?;
    file_list.push("\"./sail-runtime-ambient.d.ts\"".into());

    // Open-any for non-precise `@ship/*` (subpaths; roots when no node_modules).
    let mut scan_body = String::new();
    for (_, src) in files {
        scan_body.push_str(src);
        scan_body.push('\n');
    }
    let ship_nm = find_ship_node_modules(Some(project_root));
    let open_ship = emit_open_ship_package_ambient(&scan_body, ship_nm.is_some());
    if !open_ship.is_empty() {
        std::fs::write(out_dir.join("ship-open-stubs.d.ts"), open_ship)?;
        file_list.push("\"./ship-open-stubs.d.ts\"".into());
    }

    if env == TypeEnv::Action {
        std::fs::write(
            out_dir.join("action-observation.d.ts"),
            emit_action_observation_dts(),
        )?;
        file_list.push("\"./action-observation.d.ts\"".into());
    }

    let lib = lib_for_type_env(env);
    let lib_arr = lib
        .iter()
        .map(|e| format!("\"{e}\""))
        .collect::<Vec<_>>()
        .join(", ");
    // Mirror lives at `{project}/.deck/type-gate-{env}/`. Map @/ aliases back to
    // the real project tree with paths relative to this config dir (tsgo has
    // removed `baseUrl` — TS5102 — so we must not emit it here).
    let path_entries: Vec<String> = crate::project_map::at_alias_paths()
        .iter()
        .map(|m| {
            let target = m.target.trim_start_matches("./");
            // out_dir is project/.deck/type-gate-*/ → ../../ reaches project root.
            format!("      \"{}\": [\"../../{target}\"]", m.pattern)
        })
        .collect();
    let at_paths = path_entries.join(",\n");
    let ship_paths = ship_nm
        .as_ref()
        .and_then(|nm| ship_packages_paths_entry(nm, &out_dir))
        .unwrap_or_default();
    let paths_block = merge_paths_blocks(&[at_paths, ship_paths]);
    let tsconfig = format!(
        r#"{{
  "compilerOptions": {{
    "target": "ESNext",
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "lib": [{lib_arr}],
    "strict": true,
    "noEmit": true,
    "skipLibCheck": true,
    "noUncheckedIndexedAccess": true,
    "isolatedModules": true,
    "paths": {{
{paths_block}
    }}
  }},
  "files": [{files}]
}}
"#,
        files = file_list.join(", "),
    );
    std::fs::write(out_dir.join("tsconfig.json"), tsconfig)?;

    let output = run_tsgo(tool, &["-p", "tsconfig.json", "--noEmit"], &out_dir)?;
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&output.stderr));

    for d in text.lines().filter_map(parse_line) {
        // Skip ambient pack diagnostics — only product mirror files.
        if d.file.contains("sail-runtime-ambient")
            || d.file.contains("ship-open-stubs")
            || d.file.contains("action-observation")
            || d.file.ends_with("tsconfig.json")
        {
            continue;
        }
        // Resolve mirrored relative path → real path for teaching message.
        // Only product mirror files are in path_map; dependency noise from
        // node_modules / Framework packages (resolved under Pure lib) is dropped.
        let Some(real) = path_map.iter().find(|(n, _)| {
            let n_slash = n.replace('\\', "/");
            let f = d.file.replace('\\', "/");
            f.ends_with(&n_slash) || f.contains(&format!("/{n_slash}"))
        }) else {
            continue;
        };
        let real = real.1.display().to_string();
        if let Some(mut mapped) = map_region_diag(&d, env) {
            // Bracketed real path — CLI `project_ts_type_gate` prints it as the
            // diagnostic file path (`path:line:col`).
            mapped.message = format!("{} [{}]", mapped.message, real);
            diags.push(mapped);
        }
    }
    Ok(())
}

fn env_label(env: TypeEnv) -> &'static str {
    match env {
        TypeEnv::Pure => "pure",
        TypeEnv::Action => "action",
        TypeEnv::Framework => "framework",
    }
}

// ── diagnostic mapping ─────────────────────────────────────────────────────

/// Codes that mean "name/type not found" under a stripped lib — candidates for
/// Pure/Action DOM teaching rewrites when the missing name is a DOM surface.
const NAME_CODES: &[&str] = &["TS2304", "TS2552", "TS2584", "TS2503", "TS2551", "TS2339"];

/// Map one tsgo diagnostic into the sail stream.
///
/// Doctrine: **if tsc does not compile under a faithful env, sail does not
/// compile.** DOM-surface unnameables rewrite to `sail-pure-region` /
/// `sail-action-surface`; intrinsic type errors become `sail-type-gate`.
/// Residual harness module-resolution noise (`TS2307` for relative / `@ship/`
/// / `@/` project-alias specs) is **not** promoted until the region runs
/// under full project resolution — see module docs.
fn map_region_diag(d: &TsDiag, env: TypeEnv) -> Option<Diagnostic> {
    // Prefer teaching rewrite for DOM-surface name errors.
    if NAME_CODES.contains(&d.code.as_str()) {
        if let Some(name) = extract_not_found_name(&d.message) {
            if is_dom_surface_name(&name) {
                // Action observation set is allowed under Action env.
                if env == TypeEnv::Action && is_action_observation_name(&name) {
                    return None;
                }
                let (rule, message) = match env {
                    TypeEnv::Pure | TypeEnv::Framework => (
                        RULE_PURE_REGION,
                        format!(
                            "`{name}` is not nameable here — controllers/compositions/stores are Pure \
                             (ESNext only; no DOM lib). Move DOM work to a `use:` action."
                        ),
                    ),
                    TypeEnv::Action => (
                        RULE_ACTION_SURFACE,
                        format!(
                            "`{name}` is not nameable on the action surface — actions get the observation \
                             set only (requestAnimationFrame, IntersectionObserver, ResizeObserver, \
                             MutationObserver, getComputedStyle). Use a handed `node` argument; do not \
                             name document/window."
                        ),
                    ),
                };
                return Some(Diagnostic {
                    line: d.line,
                    col: d.col,
                    end_line: d.line,
                    end_col: d.col + name.len().max(1),
                    severity: DiagSeverity::Error,
                    message,
                    rule: Some(rule.to_string()),
                });
            }
        }
    }

    // E4b R2: only ambient `@ship/*` stubs may suppress TS2307.
    // Relative / `@/` / `*.sail` hard-fail when unresolved (no silent mask).
    // Sibling materialization proves present `.sail` modules; missing → TS2307.
    if d.code == "TS2307" {
        if let Some(spec) = extract_cannot_find_module(&d.message) {
            if is_ambient_module_stub_spec(&spec) {
                return None;
            }
        }
    }
    // Do **not** suppress TS2305/TS2614 for `@ship/*`. Pattern ambient diagnostics
    // always name `Module '"@ship/*"'` for both roots and subpaths, so specifier
    // heuristics cannot distinguish them. Non-precise packages instead get
    // per-import open-any `declare module "exact-spec"` (requested names only)
    // and package roots prefer real `node_modules/@ship` types when installed.
    // Open-any sibling stubs force callback params to `any`. Keep noImplicitAny
    // on for author body, but do not promote harness TS7006 noise.
    if d.code == "TS7006" {
        return None;
    }

    // Intrinsic type errors and remaining name gaps: hard sail-type-gate fail.
    let end_col = d.col + 1;
    Some(Diagnostic {
        line: d.line,
        col: d.col,
        end_line: d.line,
        end_col,
        severity: DiagSeverity::Error,
        message: format!("{}: {}", d.code, d.message),
        rule: Some(RULE_TYPE_GATE.to_string()),
    })
}

/// `"Cannot find module 'X' or its corresponding type declarations."`
fn extract_cannot_find_module(message: &str) -> Option<String> {
    const PFX: &str = "Cannot find module '";
    let rest = message.find(PFX).map(|i| &message[i + PFX.len()..])?;
    let end = rest.find('\'').unwrap_or(rest.len());
    let spec = &rest[..end];
    if spec.is_empty() {
        None
    } else {
        Some(spec.to_string())
    }
}

fn is_ambient_module_stub_spec(spec: &str) -> bool {
    // Package ambients (`@ship/*`) are harness-stubbed.
    // Bare npm / scoped package names (no leading `.`/`/`) are suppressed:
    // the region harness has no node_modules — install/resolution is a bundler
    // concern, not Pure ambient law. Relative + `@/` stay hard-fail.
    // Relative `.sail` is resolved via sibling mirror — never suppress TS2307
    // for a missing `./foo.sail`.
    let s = spec.trim();
    if s.starts_with("@ship/") {
        return true;
    }
    if s.is_empty() || s.starts_with('.') || s.starts_with('/') || s.starts_with("@/") {
        return false;
    }
    // bare `mermaid` or scoped `@codemirror/state`
    !s.starts_with('.') && !s.starts_with('/')
}

fn extract_not_found_name(message: &str) -> Option<String> {
    // "Cannot find name 'window'." / "Cannot find namespace 'NodeJS'."
    // "Property 'x' does not exist on type ..."
    let patterns = [
        "Cannot find name '",
        "Cannot find namespace '",
        "Cannot find name “",
        "Property '",
    ];
    for p in patterns {
        if let Some(rest) = message
            .strip_prefix(p)
            .or_else(|| message.find(p).map(|i| &message[i + p.len()..]))
        {
            let end = rest.find(['\'', '’', '”', '"']).unwrap_or(rest.len());
            let name = &rest[..end];
            if !name.is_empty() {
                // Property messages are not free-global DOM unnameables for
                // teaching rewrite (they still promote via sail-type-gate).
                if p.starts_with("Property") {
                    return None;
                }
                return Some(name.to_string());
            }
        }
    }
    // TS2584: Cannot find name 'document'. Do you need to change your target library?
    if let Some(i) = message.find("Cannot find name '") {
        let rest = &message[i + "Cannot find name '".len()..];
        let end = rest.find('\'').unwrap_or(rest.len());
        let name = &rest[..end];
        if !name.is_empty() {
            return Some(name.to_string());
        }
    }
    None
}

fn is_dom_surface_name(name: &str) -> bool {
    // Allowlist of known DOM globals + constructors/interfaces. Avoid bare
    // suffix matching (`UserEvent`, `RowElement`) and ambiguous free names
    // (`self` / `parent` / `top`) that are common user identifiers.
    matches!(
        name,
        "window"
            | "document"
            | "Document"
            | "Window"
            | "HTMLElement"
            | "Element"
            | "Node"
            | "Event"
            | "EventTarget"
            | "localStorage"
            | "sessionStorage"
            | "navigator"
            | "location"
            | "history"
            | "History"
            | "Location"
            | "Navigator"
            | "matchMedia"
            | "MediaQueryList"
            | "getComputedStyle"
            | "requestAnimationFrame"
            | "cancelAnimationFrame"
            | "IntersectionObserver"
            | "ResizeObserver"
            | "MutationObserver"
            | "customElements"
            | "DOMParser"
            | "XMLHttpRequest"
            | "WebSocket"
            | "EventSource"
            | "fetch"
            | "Headers"
            | "Request"
            | "Response"
            | "FormData"
            | "FileReader"
            | "Blob"
            | "File"
            | "Image"
            | "Audio"
            | "Worker"
            | "SharedWorker"
            | "ServiceWorker"
            | "MessageChannel"
            | "BroadcastChannel"
            | "CSSStyleDeclaration"
            | "DOMRect"
            | "DOMRectReadOnly"
            | "HTMLDivElement"
            | "HTMLInputElement"
            | "HTMLButtonElement"
            | "HTMLCanvasElement"
            | "HTMLSpanElement"
            | "HTMLAnchorElement"
            | "HTMLImageElement"
            | "HTMLTextAreaElement"
            | "HTMLSelectElement"
            | "HTMLFormElement"
            | "HTMLUListElement"
            | "HTMLLIElement"
            | "HTMLTableElement"
            | "SVGElement"
            | "SVGSVGElement"
            | "ShadowRoot"
            | "MutationRecord"
            | "ResizeObserverEntry"
            | "IntersectionObserverEntry"
            | "PointerEvent"
            | "MouseEvent"
            | "KeyboardEvent"
            | "FocusEvent"
            | "InputEvent"
            | "TouchEvent"
            | "WheelEvent"
            | "DragEvent"
            | "ClipboardEvent"
            | "CustomEvent"
            | "frames"
            | "screen"
            | "performance"
            | "crypto"
            | "indexedDB"
            | "caches"
            | "cookieStore"
            | "visualViewport"
    ) || (name.starts_with("HTML") && name.ends_with("Element"))
        || (name.starts_with("SVG") && name.len() > 3)
}

fn is_action_observation_name(name: &str) -> bool {
    matches!(
        name,
        "requestAnimationFrame"
            | "cancelAnimationFrame"
            | "IntersectionObserver"
            | "ResizeObserver"
            | "MutationObserver"
            | "getComputedStyle"
            | "Element"
            | "DOMRectReadOnly"
            | "CSSStyleDeclaration"
            | "ResizeObserverEntry"
            | "ResizeObserverOptions"
            | "ResizeObserverCallback"
            | "IntersectionObserverEntry"
            | "IntersectionObserverInit"
            | "IntersectionObserverCallback"
            | "MutationRecord"
            | "MutationObserverInit"
            | "MutationCallback"
            | "MutationRecordType"
    )
}

// ── @ts-ignore scan (E1c) ──────────────────────────────────────────────────

fn scan_ts_ignore(source: &str) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    // Only regions that are TS: inside <controller>…</controller> and bare
    // .ts files (whole source). For .sail, limit to controller body.
    let region = if source.contains("<controller") {
        let Some(r) = extract_controller_region(source) else {
            return diags;
        };
        Some(r)
    } else {
        None
    };
    let (line_lo, line_hi) = match &region {
        Some(r) => {
            let body_lines = r.body.lines().count().max(1);
            (r.body_start_line, r.body_start_line + body_lines)
        }
        None => (1, usize::MAX),
    };
    let needles = ["@ts-ignore", "@ts-nocheck", "@ts-expect-error"];
    for (line_idx, line) in source.lines().enumerate() {
        let line_no = line_idx + 1;
        if line_no < line_lo || line_no >= line_hi {
            continue;
        }
        for needle in needles {
            if let Some(col0) = line.find(needle) {
                diags.push(Diagnostic {
                    line: line_no,
                    col: col0 + 1,
                    end_line: line_no,
                    end_col: col0 + 1 + needle.len(),
                    severity: DiagSeverity::Error,
                    message: format!(
                        "`{needle}` is not allowed — the type gate is the point. \
                         Fix the type or move the code to its sanctioned region \
                         (Pure controller/composition/store vs Action observation)."
                    ),
                    rule: Some(RULE_NO_TS_IGNORE.to_string()),
                });
            }
        }
    }
    diags
}

// ── type-erase scan (E4a R1) ───────────────────────────────────────────────

fn scan_type_erase(source: &str, source_path: Option<&Path>) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    if let Some(p) = source_path {
        if type_env_for_path(p, Some(source)) == TypeEnv::Framework {
            return diags;
        }
    }
    let region = if source.contains("<controller") {
        let Some(r) = extract_controller_region(source) else {
            return diags;
        };
        Some(r)
    } else {
        None
    };
    let (line_lo, line_hi) = match &region {
        Some(r) => {
            let body_lines = r.body.lines().count().max(1);
            (r.body_start_line, r.body_start_line + body_lines)
        }
        None => (1, usize::MAX),
    };

    for (line_idx, line) in source.lines().enumerate() {
        let line_no = line_idx + 1;
        if line_no < line_lo || line_no >= line_hi {
            continue;
        }
        let code = strip_line_comment(line);
        if code.trim().is_empty() {
            continue;
        }
        // Mask string / template literal bodies so doc messages containing
        // `as any` / `Function(` do not false-positive (columns stay aligned).
        let code = mask_string_literals(code);
        if let Some(col0) = find_as_any(&code) {
            diags.push(type_erase_diag(
                line_no,
                col0,
                "as any",
                "`as any` erases the type the gate enforces. Use a sanctioned ambient \
                 (`SailHost` / action `node`) or move DOM work to a `use:` action.",
            ));
        }
        if let Some(col0) = code.find("as unknown as") {
            diags.push(type_erase_diag(
                line_no,
                col0,
                "as unknown as",
                "`as unknown as` is double type erasure. Name a real type or move the \
                 code to Action / Framework where the surface is legal.",
            ));
        }
        if let Some(col0) = find_satisfies_any(&code) {
            diags.push(type_erase_diag(
                line_no,
                col0,
                "satisfies any",
                "`satisfies any` erases type information. Satisfy a real type or use \
                 the sanctioned Pure ambient surface.",
            ));
        }
        if let Some(col0) = find_function_ctor(&code) {
            diags.push(type_erase_diag(
                line_no,
                col0,
                "Function()",
                "`Function(...)` constructs code without types. Use a typed function \
                 or a sanctioned action.",
            ));
        }
        if let Some(col0) = find_globalthis_string_index(&code) {
            diags.push(type_erase_diag(
                line_no,
                col0,
                "globalThis[…]",
                "String-reflective global access bypasses Pure ambient law. Use a \
                 `use:` action, or name `SailHost` / sanctioned helpers — never \
                 `globalThis['window']` / `globalThis['document']`.",
            ));
        }
    }
    diags
}

fn type_erase_diag(line: usize, col0: usize, needle: &str, message: &str) -> Diagnostic {
    Diagnostic {
        line,
        col: col0 + 1,
        end_line: line,
        end_col: col0 + 1 + needle.len().max(1),
        severity: DiagSeverity::Error,
        message: message.to_string(),
        rule: Some(RULE_NO_TYPE_ERASE.to_string()),
    }
}

fn strip_line_comment(line: &str) -> &str {
    let mut in_single = false;
    let mut in_double = false;
    let mut in_template = false;
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if !in_double && !in_template && c == b'\'' {
            in_single = !in_single;
        } else if !in_single && !in_template && c == b'"' {
            in_double = !in_double;
        } else if !in_single && !in_double && c == b'`' {
            in_template = !in_template;
        } else if !in_single
            && !in_double
            && !in_template
            && c == b'/'
            && i + 1 < bytes.len()
            && bytes[i + 1] == b'/'
        {
            return &line[..i];
        }
        i += 1;
    }
    line
}

/// Replace interiors of string/template literals with spaces so needle search
/// does not match inside them. Delimiter quotes stay so outside columns align.
fn mask_string_literals(line: &str) -> String {
    let bytes = line.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    let mut quote: Option<u8> = None;
    while i < bytes.len() {
        let c = bytes[i];
        if let Some(q) = quote {
            if c == b'\\' {
                out.push(b' ');
                if i + 1 < bytes.len() {
                    out.push(b' ');
                    i += 2;
                    continue;
                }
                i += 1;
                continue;
            }
            if c == q {
                out.push(c);
                quote = None;
            } else {
                out.push(b' ');
            }
            i += 1;
            continue;
        }
        if c == b'\'' || c == b'"' || c == b'`' {
            quote = Some(c);
            out.push(c);
            i += 1;
            continue;
        }
        out.push(c);
        i += 1;
    }
    String::from_utf8(out).unwrap_or_else(|_| line.to_string())
}

fn find_as_any(code: &str) -> Option<usize> {
    let mut start = 0;
    while let Some(rel) = code[start..].find("as any") {
        let col0 = start + rel;
        let after = col0 + "as any".len();
        let ok_after = code
            .as_bytes()
            .get(after)
            .map(|b| !b.is_ascii_alphanumeric() && *b != b'_')
            .unwrap_or(true);
        let ok_before = if col0 == 0 {
            true
        } else {
            let b = code.as_bytes()[col0 - 1];
            !b.is_ascii_alphanumeric() && b != b'_'
        };
        if ok_before && ok_after {
            return Some(col0);
        }
        start = col0 + 1;
    }
    None
}

fn find_satisfies_any(code: &str) -> Option<usize> {
    let mut start = 0;
    while let Some(rel) = code[start..].find("satisfies any") {
        let col0 = start + rel;
        let after = col0 + "satisfies any".len();
        let ok_after = code
            .as_bytes()
            .get(after)
            .map(|b| !b.is_ascii_alphanumeric() && *b != b'_')
            .unwrap_or(true);
        if ok_after {
            return Some(col0);
        }
        start = col0 + 1;
    }
    None
}

fn find_function_ctor(code: &str) -> Option<usize> {
    if let Some(col0) = code.find("new Function") {
        let rest = &code[col0 + "new Function".len()..];
        if rest.trim_start().starts_with('(') {
            return Some(col0);
        }
    }
    let mut start = 0;
    while let Some(rel) = code[start..].find("Function") {
        let col0 = start + rel;
        let ok_before = if col0 == 0 {
            true
        } else {
            let b = code.as_bytes()[col0 - 1];
            !b.is_ascii_alphanumeric() && b != b'_' && b != b'.'
        };
        if ok_before {
            let after = col0 + "Function".len();
            let rest = &code[after..];
            if rest.trim_start().starts_with('(') {
                return Some(col0);
            }
        }
        start = col0 + 1;
    }
    None
}

fn find_globalthis_string_index(code: &str) -> Option<usize> {
    for n in ["globalThis['", "globalThis[\"", "globalThis[`"] {
        if let Some(col0) = code.find(n) {
            return Some(col0);
        }
    }
    None
}

// ── legacy bridge (props/variants + synthetic controller) ──────────────────
//
// These helpers are **not** wired into `collect` / `collect_sail_type_gate` /
// `collect_project_type_gate`. Production OnSave type law is the region/project
// harness above. Keep for props/variants bridge tests and the synthetic
// template↔controller scope-contract gate (separate filter set).

/// Type-check `block` (a TS module body) against `harness` (the `.ts` providing
/// `props()`/`variants()` + the built-in symbols). Returns tsgo's diagnostics —
/// empty means clean. `node`/`tsgo_js` locate the shipped compiler; production
/// resolves them from the toolchain, the spike passes them in.
///
/// **Not** part of the E1 type gate (`collect_*`); test/legacy props bridge only.
pub fn typecheck_block(
    harness: &str,
    block: &str,
    out_dir: &Path,
    node: &str,
    tsgo_js: &str,
) -> std::io::Result<Vec<TsDiag>> {
    std::fs::create_dir_all(out_dir)?;
    let harness_path = out_dir.join("sail-embed-harness.ts");
    let block_path = out_dir.join("sail-embed-block.ts");
    std::fs::write(&harness_path, harness)?;
    std::fs::write(&block_path, block)?;
    let output = Command::new(node)
        .arg(tsgo_js)
        .arg("--noEmit")
        .arg("--strict")
        .arg("--skipLibCheck")
        .arg(&harness_path)
        .arg(&block_path)
        .output()?;
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    Ok(text.lines().filter_map(parse_line).collect())
}

/// Parse one tsgo diagnostic line: `…/file.ts(5,3): error TS2552: message`.
fn parse_line(line: &str) -> Option<TsDiag> {
    let open = line.find('(')?;
    let close = line[open..].find(')')? + open;
    let (l, c) = line[open + 1..close].split_once(',')?;
    let line_no = l.trim().parse().ok()?;
    let col_no = c.trim().parse().ok()?;
    let file = line[..open].trim().to_string();
    let rest = line[close + 1..].trim_start_matches(':').trim();
    let rest = rest.strip_prefix("error").unwrap_or(rest).trim();
    let (code, msg) = rest.split_once(':')?;
    if !code.trim().starts_with("TS") {
        return None;
    }
    Some(TsDiag {
        file,
        line: line_no,
        col: col_no,
        code: code.trim().to_string(),
        message: msg.trim().to_string(),
    })
}

/// Type-check the synthetic controller + render pair for an SFC `source`.
///
/// Returns tsgo diagnostics (empty = clean). `Ok(vec![])` when there's no inline
/// controller to check against (bare fragment) or the pair can't be emitted.
///
/// **Not** wired into `collect` / compile gate — owns template↔controller
/// *scope contract* codes only ([`SCOPE_CONTRACT_CODES`]). E1 type gate is
/// [`collect_sail_type_gate`] / [`collect_project_type_gate`].
pub fn check_synthetic_controller(
    source: &str,
    source_dir: &Path,
    out_dir: &Path,
    node: &str,
    tsgo_js: &str,
    workspace_root: &Path,
) -> std::io::Result<Vec<TsDiag>> {
    use crate::CompileMode;
    use crate::emit::synthetic::{AppManifest, AuthoredController, emit_synthetic};
    use crate::emit::writer::js_string;
    use crate::parse::{self, preprocess};

    let (stripped, inline_ctrl) = preprocess::extract_controller_block(source);
    let Some(body) = inline_ctrl else {
        return Ok(Vec::new());
    };
    let Ok(program) = parse::parse(&stripped) else {
        return Ok(Vec::new());
    };

    std::fs::create_dir_all(out_dir)?;
    let Ok(rewritten) = crate::check::query_keys::rewrite(&body) else {
        return Ok(Vec::new());
    };
    std::fs::write(
        out_dir.join("view.controller.ts"),
        preprocess::wrap_controller_body(&rewritten),
    )?;
    let authored = AuthoredController {
        specifier: "./view.controller.ts".to_string(),
    };
    let Ok(out) = emit_synthetic(
        &program,
        &AppManifest::default(),
        Some(&authored),
        "./view.synthetic.controller.ts",
        CompileMode::Component,
        None,
    ) else {
        return Ok(Vec::new());
    };
    std::fs::write(
        out_dir.join("view.synthetic.controller.ts"),
        &out.controller_ts,
    )?;
    std::fs::write(out_dir.join("view.render.ts"), &out.render_ts)?;

    if let Ok(entries) = std::fs::read_dir(source_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            let Some(name) = p.file_name().and_then(|s| s.to_str()) else {
                continue;
            };
            if name.ends_with(".ts") {
                let _ = std::fs::copy(&p, out_dir.join(name));
            } else if name.ends_with(".sail") {
                if let Ok(sib) = std::fs::read_to_string(&p) {
                    let stem = name.strip_suffix(".sail").unwrap_or(name);
                    let opts = crate::CompileOpts {
                        setup_url: None,
                        mode: crate::CompileMode::Component,
                        source_path: Some(p.clone()),
                        component_meta: Some(crate::ComponentMeta {
                            name: stem.to_string(),
                        }),
                        route_params: None,
                        emit_dts: true,
                        scope_capabilities: Vec::new(),
                        emit_component_imports: false,
                        lint_overrides: Vec::new(),
                    };
                    if let Ok(o) = crate::compile(&sib, opts)
                        && let Some(dts) = o.dts
                    {
                        let _ = std::fs::write(out_dir.join(format!("{name}.d.ts")), dts);
                    }
                }
            }
        }
    }

    let runtime = workspace_root.join("packages/sail/runtime/src/index.ts");
    let dom = workspace_root.join("packages/sail/dom/src/index.ts");
    let tsconfig = format!(
        "{{\n  \"compilerOptions\": {{\n    \"noEmit\": true, \"strict\": true, \"skipLibCheck\": true,\n\
         \"moduleResolution\": \"bundler\", \"module\": \"esnext\", \"target\": \"esnext\",\n\
         \"allowImportingTsExtensions\": true,\n    \"paths\": {{\n      \"@ship/sail-runtime\": [{}],\n\
         \"@ship/sail-dom\": [{}]\n    }}\n  }},\n  \
         \"include\": [\"view.render.ts\", \"view.synthetic.controller.ts\", \"view.controller.ts\"]\n}}\n",
        js_string(&runtime.to_string_lossy()),
        js_string(&dom.to_string_lossy()),
    );
    let tsconfig_path = out_dir.join("tsconfig.json");
    std::fs::write(&tsconfig_path, tsconfig)?;

    let output = Command::new(node)
        .arg(tsgo_js)
        .arg("-p")
        .arg(&tsconfig_path)
        .output()?;
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    Ok(text
        .lines()
        .filter_map(parse_line)
        .filter(|d| SCOPE_CONTRACT_CODES.contains(&d.code.as_str()))
        .collect())
}

/// tsgo diagnostic codes that mean "the template referenced a name/member the
/// controller's scope does not provide" — the only thing the synthetic gate
/// enforces. TS2339: property does not exist; TS2551: property does not exist
/// (+ did-you-mean); TS2304: cannot find name; TS2552: cannot find name (+
/// did-you-mean).
const SCOPE_CONTRACT_CODES: &[&str] = &["TS2339", "TS2551", "TS2304", "TS2552"];

// ── tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn tsgo_env() -> Option<(String, String)> {
        let node = std::env::var("SUI_TSGO_NODE").ok()?;
        let js = std::env::var("SUI_TSGO_JS").ok()?;
        if Path::new(&node).exists() && Path::new(&js).exists() {
            Some((node, js))
        } else {
            None
        }
    }

    #[test]
    fn resolve_tsgo_finds_workspace() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("workspace root");
        let tool = resolve_tsgo(Some(root));
        assert!(
            tool.is_some(),
            "tsgo must resolve from workspace (bun install @typescript/native-preview)"
        );
    }

    #[test]
    fn pure_region_flags_window() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("workspace root");
        if resolve_tsgo(Some(root)).is_none() {
            panic!("tsgo required for type gate tests — run bun install");
        }
        let src = r#"
<controller>
export default controller((ctx) => {
  const hist = window.history
  return { hist }
})
</controller>
<div>{hist}</div>
"#;
        let path = root.join("apps/fake/src/widget/widget.sail");
        let diags = collect_sail_type_gate(src, Some(&path));
        assert!(
            diags.iter().any(|d| {
                d.rule.as_deref() == Some(RULE_PURE_REGION) && d.message.contains("window")
            }),
            "expected sail-pure-region on window, got {diags:?}"
        );
        // Span should land near the window reference (line in fixture).
        let hit = diags
            .iter()
            .find(|d| d.rule.as_deref() == Some(RULE_PURE_REGION))
            .unwrap();
        assert!(
            hit.line >= 3,
            "window line must map into controller body, got line {}",
            hit.line
        );
    }

    #[test]
    fn clean_controller_no_false_positive() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("workspace root");
        if resolve_tsgo(Some(root)).is_none() {
            panic!("tsgo required");
        }
        let src = r#"
<controller>
export default controller((ctx) => {
  const count = signal(0)
  const inc = () => count.set(count() + 1)
  return { count, inc }
})
</controller>
<button on:click={inc}>{count()}</button>
"#;
        let path = root.join("apps/fake/src/widget/widget.sail");
        let diags = collect_sail_type_gate(src, Some(&path));
        let type_errs: Vec<_> = diags
            .iter()
            .filter(|d| {
                matches!(
                    d.rule.as_deref(),
                    Some(RULE_PURE_REGION | RULE_ACTION_SURFACE | RULE_TYPE_GATE)
                )
            })
            .collect();
        assert!(
            type_errs.is_empty(),
            "clean Pure controller must not type-gate fail: {type_errs:?}"
        );
    }

    fn type_gate_errs(diags: &[Diagnostic]) -> Vec<&Diagnostic> {
        diags
            .iter()
            .filter(|d| {
                d.severity == DiagSeverity::Error
                    && matches!(
                        d.rule.as_deref(),
                        Some(
                            RULE_PURE_REGION
                                | RULE_ACTION_SURFACE
                                | RULE_TYPE_GATE
                                | RULE_NO_TS_IGNORE
                                | RULE_NO_TYPE_ERASE
                        )
                    )
            })
            .collect()
    }

    /// G2a: controller-local `type` used later in the same block must not
    /// TS2304 — whole module is harnessed, not a wrapped fragment.
    #[test]
    fn harness_controller_local_type_is_visible() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("workspace root");
        if resolve_tsgo(Some(root)).is_none() {
            panic!("tsgo required");
        }
        let src = r#"
<controller>
type Row = { id: string; city: string }
const CITIES = ['Osaka', 'Nairobi'] as const
function cityAt(i: number): string {
  return CITIES[i % CITIES.length]!
}
export default controller(() => {
  const mk = (): Row => ({ id: '1', city: cityAt(0) })
  const rows = signal<Row[]>([mk()])
  return { rows }
})
</controller>
<div />
"#;
        let path = root.join("apps/fake/src/widget/widget.sail");
        let diags = collect_sail_type_gate(src, Some(&path));
        let errs = type_gate_errs(&diags);
        assert!(
            errs.is_empty(),
            "controller-local type/const/helper must type-gate green: {errs:?}"
        );
    }

    /// G2a: module-level const/helper referenced from the default export
    /// (and thus available to the template via return) must stay green.
    #[test]
    fn harness_controller_local_const_helper_green() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("workspace root");
        if resolve_tsgo(Some(root)).is_none() {
            panic!("tsgo required");
        }
        let src = r#"
<controller>
const LABEL = 'board'
const double = (n: number) => n * 2
export default controller(() => {
  const n = signal(double(1))
  return { n, label: LABEL }
})
</controller>
<span>{label}{n()}</span>
"#;
        let path = root.join("apps/fake/src/widget/widget.sail");
        let diags = collect_sail_type_gate(src, Some(&path));
        let errs = type_gate_errs(&diags);
        assert!(
            errs.is_empty(),
            "module-level const/helper must type-gate green: {errs:?}"
        );
    }

    /// G2a / flow-trace: `type Row` in `<scope>` is visible to the controller.
    #[test]
    fn harness_scope_type_referenced_from_controller() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("workspace root");
        if resolve_tsgo(Some(root)).is_none() {
            panic!("tsgo required");
        }
        let src = r#"
<scope>
  type Row = { id: string; city: string }
  style board = { box: pad(4) }
</scope>
<controller>
export default controller(() => {
  const mk = (): Row => ({ id: '1', city: 'Osaka' })
  const rows = signal<Row[]>([mk()])
  return { rows }
})
</controller>
<div />
"#;
        let path = root.join("apps/fake/src/widget/widget.sail");
        let diags = collect_sail_type_gate(src, Some(&path));
        let errs = type_gate_errs(&diags);
        assert!(
            errs.is_empty(),
            "scope type Row must be visible in controller (no TS2304): {errs:?}"
        );
    }

    /// Live flow-trace fixture: was the verified G2a false positive (TS2304 Row).
    #[test]
    fn flow_trace_type_gate_green() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("workspace root");
        if resolve_tsgo(Some(root)).is_none() {
            panic!("tsgo required");
        }
        let path = root.join("apps/sail-site/src/flow-trace/flow-trace.sail");
        let src = std::fs::read_to_string(&path).expect("flow-trace.sail");
        let diags = collect_sail_type_gate(&src, Some(&path));
        let errs = type_gate_errs(&diags);
        assert!(
            errs.is_empty(),
            "flow-trace.sail must type-gate green after G2a: {errs:?}"
        );
    }

    /// G2a: existing `@/` alias import must not false-positive when the target exists.
    #[test]
    fn harness_at_alias_import_green() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("workspace root");
        if resolve_tsgo(Some(root)).is_none() {
            panic!("tsgo required");
        }
        let src = r#"
<controller>
import { tally } from '@/stores/counter-store'

export default controller(() => {
  const n = signal(0)
  return { n, tally }
})
</controller>
<div />
"#;
        // Path under sail-site so project root + @/stores/* resolve.
        let path = root.join("apps/sail-site/src/widget/widget.sail");
        let diags = collect_sail_type_gate(src, Some(&path));
        let errs = type_gate_errs(&diags);
        assert!(
            errs.is_empty(),
            "@/ alias import must type-gate green when target exists: {errs:?}"
        );
    }

    /// G2a: npm package import (built / node_modules) is not a type-gate false positive.
    #[test]
    fn harness_npm_component_package_import_green() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("workspace root");
        if resolve_tsgo(Some(root)).is_none() {
            panic!("tsgo required");
        }
        // Bare / scoped package names are not Pure ambient law (bundler resolve);
        // must not fail the gate as TS2307 harness noise.
        // Component *subpaths* get open-any of requested names (no TS2305 suppress).
        let src = r#"
<controller>
import { sailCalendar } from '@ship/sail-primitives/sail-calendar'

export default controller(() => {
  const n = signal(0)
  return { n, sailCalendar }
})
</controller>
<div />
"#;
        let path = root.join("apps/fake/src/widget/widget.sail");
        let diags = collect_sail_type_gate(src, Some(&path));
        let errs = type_gate_errs(&diags);
        assert!(
            errs.is_empty(),
            "npm component package import must not type-gate fail: {errs:?}"
        );
    }

    /// Package-root named-import fidelity: wrong export must fail the type-gate
    /// when real `node_modules/@ship` types are available (not silent green via
    /// pattern-ambient TS2305 suppression).
    #[test]
    fn harness_ship_package_root_missing_export_fails() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("workspace root");
        if resolve_tsgo(Some(root)).is_none() {
            panic!("tsgo required");
        }
        assert!(
            root.join("node_modules/@ship/sail-studio-shell").is_dir(),
            "fixture requires monorepo node_modules/@ship/sail-studio-shell"
        );
        let src = r#"
<controller>
import { noSuchThing } from '@ship/sail-studio-shell'

export default controller(() => {
  const n = signal(0)
  return { n, noSuchThing }
})
</controller>
<div />
"#;
        // Path under workspace so find_ship_node_modules walks to monorepo root.
        let path = root.join("apps/fake/src/widget/widget.sail");
        let diags = collect_sail_type_gate(src, Some(&path));
        let errs = type_gate_errs(&diags);
        assert!(
            errs.iter().any(|d| {
                d.message.contains("TS2305")
                    || d.message.contains("TS2614")
                    || d.message.contains("no exported member")
                    || d.message.contains("noSuchThing")
            }),
            "package-root missing named export must type-gate fail, got: {errs:?}"
        );
    }

    /// Package-root *real* export stays green via node_modules types (not open-any).
    #[test]
    fn harness_ship_package_root_real_export_green() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("workspace root");
        if resolve_tsgo(Some(root)).is_none() {
            panic!("tsgo required");
        }
        if !root.join("node_modules/@ship/sail-studio-shell").is_dir() {
            return;
        }
        let src = r#"
<controller>
import { postStudioMessage } from '@ship/sail-studio-shell'

export default controller(() => {
  const n = signal(0)
  return { n, postStudioMessage }
})
</controller>
<div />
"#;
        let path = root.join("apps/fake/src/widget/widget.sail");
        let diags = collect_sail_type_gate(src, Some(&path));
        let errs = type_gate_errs(&diags);
        assert!(
            errs.is_empty(),
            "package-root real export must type-gate green: {errs:?}"
        );
    }

    #[test]
    fn ts_ignore_is_banned() {
        let src = r#"
<controller>
export default controller((ctx) => {
  // @ts-ignore
  const x = 1
  return { x }
})
</controller>
<div />
"#;
        let diags = collect_sail_type_gate(src, None);
        assert!(
            diags
                .iter()
                .any(|d| d.rule.as_deref() == Some(RULE_NO_TS_IGNORE)),
            "expected sail-no-ts-ignore, got {diags:?}"
        );
    }

    #[test]
    fn action_document_fails_raf_passes() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("workspace root");
        if resolve_tsgo(Some(root)).is_none() {
            panic!("tsgo required");
        }
        let bad = (
            root.join("apps/x/src/actions/bad.ts"),
            "export function bad(node: Element) { document.body.append(node) }\n".to_string(),
        );
        let ok = (
            root.join("apps/x/src/actions/ok.ts"),
            "export function ok(node: Element) {\n  requestAnimationFrame(() => {})\n  new IntersectionObserver(() => {}).observe(node)\n}\n"
                .to_string(),
        );
        let diags = collect_project_type_gate(root, &[bad, ok]);
        assert!(
            diags.iter().any(|d| {
                d.rule.as_deref() == Some(RULE_ACTION_SURFACE) && d.message.contains("`document`")
            }),
            "document must fail under Action: {diags:?}"
        );
        // No false positive that names rAF/IO as the banned symbol (teaching
        // messages may list them as allowed — check the backtick-wrapped name).
        assert!(
            !diags
                .iter()
                .any(|d| d.message.contains("`requestAnimationFrame`")
                    || d.message.contains("`IntersectionObserver`")),
            "observation set must pass under Action: {diags:?}"
        );
    }

    #[test]
    fn composition_window_fails() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("workspace root");
        if resolve_tsgo(Some(root)).is_none() {
            panic!("tsgo required");
        }
        let file = (
            root.join("apps/x/src/compositions/use-win.ts"),
            "export function useWin() { return window.innerWidth }\n".to_string(),
        );
        let diags = collect_project_type_gate(root, &[file]);
        assert!(
            diags.iter().any(|d| {
                d.rule.as_deref() == Some(RULE_PURE_REGION) && d.message.contains("window")
            }),
            "composition window must fail Pure: {diags:?}"
        );
    }

    #[test]
    fn non_dom_type_error_is_type_gate() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("workspace root");
        if resolve_tsgo(Some(root)).is_none() {
            panic!("tsgo required");
        }
        // Doctrine: real type errors (not just DOM names) fail the gate.
        let src = r#"
<controller>
export default controller((ctx) => {
  const n: number = "not-a-number"
  return { n }
})
</controller>
<div />
"#;
        let path = root.join("apps/fake/src/widget/widget.sail");
        let diags = collect_sail_type_gate(src, Some(&path));
        assert!(
            diags
                .iter()
                .any(|d| d.rule.as_deref() == Some(RULE_TYPE_GATE)),
            "expected sail-type-gate on type mismatch, got {diags:?}"
        );
    }

    #[test]
    fn ambient_ship_imports_are_not_type_gate_noise() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("workspace root");
        if resolve_tsgo(Some(root)).is_none() {
            panic!("tsgo required");
        }
        let src = r#"
<controller>
import { bindHtml } from '@ship/sail-runtime/internal'
import type { Action } from '@ship/sail-runtime'

export default controller((ctx) => {
  const paint: Action<void> = (node) => {
    const stop = bindHtml(node, () => 0)
    return { destroy: stop }
  }
  return { paint }
})
</controller>
<div />
"#;
        let path = root.join("apps/fake/src/widget/widget.sail");
        let diags = collect_sail_type_gate(src, Some(&path));
        let type_errs: Vec<_> = diags
            .iter()
            .filter(|d| {
                matches!(
                    d.rule.as_deref(),
                    Some(
                        RULE_PURE_REGION
                            | RULE_ACTION_SURFACE
                            | RULE_TYPE_GATE
                            | RULE_NO_TYPE_ERASE
                    )
                )
            })
            .collect();
        assert!(
            type_errs.is_empty(),
            "@ship ambient imports without DOM/type errors must not sail-type-gate: {type_errs:?}"
        );
    }

    #[test]
    fn missing_relative_module_hard_fails_ts2307() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("workspace root");
        if resolve_tsgo(Some(root)).is_none() {
            panic!("tsgo required");
        }
        let src = r#"
<controller>
import { helper } from './definitely-missing-mod-xyz'

export default controller((ctx) => {
  return { helper }
})
</controller>
<div />
"#;
        let path = root.join("apps/fake/src/widget/widget.sail");
        let diags = collect_sail_type_gate(src, Some(&path));
        assert!(
            diags.iter().any(|d| {
                d.rule.as_deref() == Some(RULE_TYPE_GATE) && d.message.contains("TS2307")
            }),
            "missing relative module must hard-fail TS2307, got {diags:?}"
        );
    }

    #[test]
    fn missing_at_alias_module_hard_fails_ts2307() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("workspace root");
        if resolve_tsgo(Some(root)).is_none() {
            panic!("tsgo required");
        }
        let src = r#"
<controller>
import { directory } from '@/compositions/definitely-missing-foo-xyz'

export default controller((ctx) => {
  const n = signal(0)
  return { n, directory }
})
</controller>
<div />
"#;
        let path = root.join("apps/sail-site/src/widget/widget.sail");
        let diags = collect_sail_type_gate(src, Some(&path));
        assert!(
            diags.iter().any(|d| {
                d.rule.as_deref() == Some(RULE_TYPE_GATE) && d.message.contains("TS2307")
            }),
            "missing @/ module must hard-fail TS2307, got {diags:?}"
        );
    }

    #[test]
    fn type_erase_as_any_globalthis_document_fails() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("workspace root");
        if resolve_tsgo(Some(root)).is_none() {
            panic!("tsgo required");
        }
        let src = r#"
<controller>
export default controller((ctx) => {
  const doc = (globalThis as any).document
  return { doc }
})
</controller>
<div />
"#;
        let path = root.join("apps/fake/src/widget/widget.sail");
        let diags = collect_sail_type_gate(src, Some(&path));
        assert!(
            diags
                .iter()
                .any(|d| d.rule.as_deref() == Some(RULE_NO_TYPE_ERASE)),
            "expected sail-no-type-erase on (globalThis as any).document, got {diags:?}"
        );
    }

    #[test]
    fn type_erase_ban_arms_each_fail() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("workspace root");
        if resolve_tsgo(Some(root)).is_none() {
            panic!("tsgo required");
        }
        let cases = [
            ("const x = v as unknown as string", "as unknown as"),
            ("const x = v satisfies any", "satisfies any"),
            ("const f = Function('return 1')", "Function"),
            ("const f = new Function('return 1')", "new Function"),
            ("const d = globalThis['document']", "globalThis"),
        ];
        for (stmt, label) in cases {
            let src = format!(
                r#"
<controller>
export default controller((ctx) => {{
  {stmt}
  return {{}}
}})
</controller>
<div />
"#
            );
            let path = root.join("apps/fake/src/widget/widget.sail");
            let diags = collect_sail_type_gate(&src, Some(&path));
            assert!(
                diags
                    .iter()
                    .any(|d| d.rule.as_deref() == Some(RULE_NO_TYPE_ERASE)),
                "expected sail-no-type-erase for {label}, got {diags:?}"
            );
        }
    }

    #[test]
    fn type_erase_does_not_match_inside_string_literals() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("workspace root");
        if resolve_tsgo(Some(root)).is_none() {
            panic!("tsgo required");
        }
        let src = r#"
<controller>
export default controller((ctx) => {
  const msg = "never write as any or Function('x') here"
  const tip = `avoid as unknown as and satisfies any`
  return { msg, tip }
})
</controller>
<div />
"#;
        let path = root.join("apps/fake/src/widget/widget.sail");
        let diags = collect_sail_type_gate(src, Some(&path));
        assert!(
            !diags
                .iter()
                .any(|d| d.rule.as_deref() == Some(RULE_NO_TYPE_ERASE)),
            "string/template contents must not emit sail-no-type-erase: {diags:?}"
        );
    }

    #[test]
    fn missing_relative_sail_module_hard_fails_ts2307() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("workspace root");
        if resolve_tsgo(Some(root)).is_none() {
            panic!("tsgo required");
        }
        let src = r#"
<controller>
import type { Scope } from './missing-sibling.sail'

export default controller((ctx) => {
  return {}
})
</controller>
<div />
"#;
        let path = root.join("apps/fake/src/widget/widget.sail");
        let diags = collect_sail_type_gate(src, Some(&path));
        assert!(
            diags.iter().any(|d| {
                d.rule.as_deref() == Some(RULE_TYPE_GATE) && d.message.contains("TS2307")
            }),
            "missing ./missing-sibling.sail must hard-fail TS2307 (no *.sail ambient mask), got {diags:?}"
        );
    }

    #[test]
    fn signal_generic_arity_is_not_type_gate_noise() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("workspace root");
        if resolve_tsgo(Some(root)).is_none() {
            panic!("tsgo required");
        }
        let src = r#"
<controller>
export default controller((ctx) => {
  const n = signal<number>(0)
  const s = signal<string | null>(null)
  return { n, s }
})
</controller>
<div />
"#;
        let path = root.join("apps/fake/src/widget/widget.sail");
        let diags = collect_sail_type_gate(src, Some(&path));
        let type_errs: Vec<_> = diags
            .iter()
            .filter(|d| d.rule.as_deref() == Some(RULE_TYPE_GATE))
            .collect();
        assert!(
            type_errs.is_empty(),
            "signal<T> must not emit TS2558 harness noise: {type_errs:?}"
        );
    }

    #[test]
    fn form_validate_destructure_infers_from_initial() {
        // Ambient form must infer T from `initial` so destructured validate
        // params are not TS7031 under noImplicitAny (docs/guide/forms shape).
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("workspace root");
        if resolve_tsgo(Some(root)).is_none() {
            panic!("tsgo required");
        }
        let src = r#"
<controller>
export default controller((ctx) => {
  const profile = form({
    initial: { name: '', email: '' },
    validate: ({ name, email }) => ({
      name: name.trim() === '' ? 'required' : null,
      email: email.includes('@') ? null : 'must include @',
    }),
    submit: async (values) => {
      void values.name
      void values.email
    },
  })
  return { profile }
})
</controller>
<div />
"#;
        let path = root.join("apps/fake/src/widget/widget.sail");
        let diags = collect_sail_type_gate(src, Some(&path));
        let type_errs: Vec<_> = diags
            .iter()
            .filter(|d| d.rule.as_deref() == Some(RULE_TYPE_GATE))
            .collect();
        assert!(
            type_errs.is_empty(),
            "form validate destructure must infer from initial (no TS7031): {type_errs:?}"
        );
    }

    #[test]
    fn first_body_line_indent_preserves_column() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("workspace root");
        if resolve_tsgo(Some(root)).is_none() {
            panic!("tsgo required");
        }
        // First content line is indented; `window` starts at author col 14
        // (2-space indent + "const x = " is 10 → window at 13? Let's compute:
        // "  const x = window" → w is at index 12 → col 13).
        let src = "<controller>\n  const x = window.history\n  export default controller(() => ({ x }))\n</controller>\n<div />\n";
        let path = root.join("apps/fake/src/widget/widget.sail");
        let diags = collect_sail_type_gate(src, Some(&path));
        let hit = diags
            .iter()
            .find(|d| d.rule.as_deref() == Some(RULE_PURE_REGION) && d.message.contains("`window`"))
            .unwrap_or_else(|| panic!("expected window pure-region, got {diags:?}"));
        // Author: line 2, "  const x = window" → col of 'w' is 13 (1-based).
        assert_eq!(hit.line, 2, "line padding must keep absolute line");
        assert_eq!(
            hit.col, 13,
            "column must preserve indent (not strip first-line spaces), got col {}",
            hit.col
        );
    }

    #[test]
    fn user_shaped_parent_is_not_pure_region_teaching() {
        // Undefined `parent` must not be taught as "move DOM work to a use: action".
        // It may still fail as sail-type-gate (Cannot find name).
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("workspace root");
        if resolve_tsgo(Some(root)).is_none() {
            panic!("tsgo required");
        }
        let src = r#"
<controller>
export default controller((ctx) => {
  const x = parent
  return { x }
})
</controller>
<div />
"#;
        let path = root.join("apps/fake/src/widget/widget.sail");
        let diags = collect_sail_type_gate(src, Some(&path));
        assert!(
            !diags
                .iter()
                .any(|d| d.rule.as_deref() == Some(RULE_PURE_REGION)
                    && d.message.contains("`parent`")),
            "parent must not be sail-pure-region: {diags:?}"
        );
        assert!(
            diags
                .iter()
                .any(|d| d.rule.as_deref() == Some(RULE_TYPE_GATE)),
            "undefined parent should still fail type gate: {diags:?}"
        );
    }

    /// The template↔controller gate: an inline-controller SFC whose template
    /// reads a name the controller's typed scope provides typechecks clean;
    /// a typo (`countt`) is a hard tsgo error.
    #[test]
    fn synthetic_controller_gate_catches_template_typo() {
        let Some((node, js)) = tsgo_env() else {
            eprintln!("skip: SUI_TSGO_NODE/SUI_TSGO_JS not set");
            return;
        };
        let ws = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("workspace root above crates/sail/compiler")
            .to_path_buf();
        let dir = std::env::temp_dir().join("sail-synthetic-gate-test");

        const CTRL: &str = "<controller>\n\
            import { controller, signal } from '@ship/sail-runtime'\n\
            export default controller((ctx) => ({ count: signal(0) }))\n\
            </controller>\n";

        let no_sibs = Path::new("/nonexistent-sail-siblings");
        let ok =
            format!("{CTRL}<button on:click={{() => count.set(count() + 1)}}>{{count()}}</button>");
        let diags = check_synthetic_controller(&ok, no_sibs, &dir, &node, &js, &ws).unwrap();
        assert!(
            diags.is_empty(),
            "valid SFC must typecheck clean, got {diags:?}"
        );

        let bad = format!("{CTRL}<button>{{countt()}}</button>");
        let diags = check_synthetic_controller(&bad, no_sibs, &dir, &node, &js, &ws).unwrap();
        assert!(
            diags.iter().any(|d| d.message.contains("countt")),
            "template typo scope.countt must be a compile error, got {diags:?}"
        );
    }

    const HARNESS: &str = r#"
export type Prop<T> = { readonly __prop: T }
export const tone: Prop<'default'|'primary'|'info'|'success'|'warning'|'destructive'|'accent'|'muted'> = { __prop: 'default' }
export const size: Prop<'sm'|'md'|'lg'> = { __prop: 'md' }
export const bool: Prop<boolean> = { __prop: false }
export const string: Prop<string> = { __prop: '' }
export function props<T extends Record<string, Prop<unknown> | readonly string[]>>(defs: T): T { return defs }
"#;

    #[test]
    fn tsgo_bridge_catches_typo_with_suggestion() {
        let Some((node, js)) = tsgo_env() else {
            eprintln!("skip: SUI_TSGO_NODE/SUI_TSGO_JS not set");
            return;
        };
        let dir = std::env::temp_dir().join("sail-tscheck-test");

        let ok = "import { props, tone, size, bool, string } from './sail-embed-harness'\n\
                  export const p = props({ tone, size, variant: ['solid','outline'], disabled: bool, label: string })\n";
        let diags = typecheck_block(HARNESS, ok, &dir, &node, &js).unwrap();
        assert!(
            diags.is_empty(),
            "valid block should be clean, got {diags:?}"
        );

        let bad = "import { props, tone, size, bool } from './sail-embed-harness'\n\
                   export const p = props({ tone, sizee, disabled: bool })\n";
        let diags = typecheck_block(HARNESS, bad, &dir, &node, &js).unwrap();
        let hit = diags
            .iter()
            .find(|d| d.code == "TS2552" || d.message.contains("sizee"));
        assert!(
            hit.is_some(),
            "expected a 'Cannot find name sizee' diagnostic, got {diags:?}"
        );
        let hit = hit.unwrap();
        assert!(
            hit.message.contains("Did you mean 'size'"),
            "expected the 'did you mean size' suggestion, got: {}",
            hit.message
        );
    }
}
