//! Shared test helpers for `sui-compiler` integration tests.
//!
//! Rust treats `tests/common/mod.rs` as a non-test submodule (not its
//! own test binary), so consuming test files declare `mod common;` and
//! call into it.

/// Strip lines or JSON fields named `_compilerVersion` so snapshots
/// and golden fixtures don't churn on every rebuild (git sha +
/// timestamp + dirty flag vary per build, but the rest of the output
/// is deterministic).
pub fn strip_version_stamp(input: &str) -> String {
    // Unwrap the view-default Object.assign(...) wrapper so the body
    // matches pre-stamp snapshots. The wrapper exists solely to
    // attach `_compilerVersion` to the default export.
    let s = input.replace(
        "export default Object.assign(view((ctx) => {",
        "export default view((ctx) => {",
    );
    // Drop the matching `}), { _compilerVersion: ... });` closer
    // (single-line) by collapsing to `});`. We match conservatively
    // to avoid touching unrelated `})` sequences.
    let s = {
        let mut out = String::with_capacity(s.len());
        for line in s.lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("}), { _compilerVersion:") && line.ends_with(");") {
                let indent_len = line.len() - trimmed.len();
                out.push_str(&line[..indent_len]);
                out.push_str("});");
                out.push('\n');
            } else {
                out.push_str(line);
                out.push('\n');
            }
        }
        out
    };
    strip_version_stamp_inner(&s)
}

fn strip_version_stamp_inner(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    // Brace-counted skip for multi-line JSON `"_compilerVersion": { ... }`
    // emitted by serde_json::to_string_pretty.
    let mut skip_depth: Option<i32> = None;
    let mut trailing_comma_pending = false;
    for line in s.lines() {
        if let Some(depth) = skip_depth.as_mut() {
            // Count braces on this line; once balanced, exit skip mode.
            for c in line.chars() {
                match c {
                    '{' => *depth += 1,
                    '}' => *depth -= 1,
                    _ => {}
                }
            }
            if *depth <= 0 {
                skip_depth = None;
                // A `},` closer means there's a sibling field after us
                // (impossible — `_compilerVersion` is always last in our
                // emitters). A bare `}` closer means we dropped the
                // last field; the previous in-snapshot field's trailing
                // comma must be removed to keep the JSON valid.
                if !line.trim_end().ends_with(',') && trailing_comma_pending {
                    // Remove the trailing comma from the last pushed line.
                    if out.ends_with(",\n") {
                        out.truncate(out.len() - 2);
                        out.push('\n');
                    }
                    trailing_comma_pending = false;
                }
            }
            continue;
        }
        let t = line.trim_start();
        if t.starts_with("// _compilerVersion") || t.starts_with("/*! _compilerVersion") {
            continue;
        }
        // JS lines that attach `_compilerVersion` to the default
        // export — match both shapes the compiler emits.
        if t.starts_with("export const _compilerVersion")
            || t.starts_with("_compilerVersion: ")
            || t.starts_with("_compilerVersion:{")
            || t.starts_with("}), { _compilerVersion:")
        {
            continue;
        }
        if t.starts_with("\"_compilerVersion\":") {
            // Count braces on the field's first line. If balanced on a
            // single line, skip just this one; otherwise enter skip mode.
            let mut depth = 0i32;
            for c in line.chars() {
                match c {
                    '{' => depth += 1,
                    '}' => depth -= 1,
                    _ => {}
                }
            }
            if depth > 0 {
                skip_depth = Some(depth);
                trailing_comma_pending = true;
            } else {
                // Single-line: if it ends with `,` the JSON is still
                // valid after removal; if `}` (last field), strip the
                // previous line's comma.
                if !line.trim_end().ends_with(',') {
                    if out.ends_with(",\n") {
                        out.truncate(out.len() - 2);
                        out.push('\n');
                    }
                }
            }
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}
