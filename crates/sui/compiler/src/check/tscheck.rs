//! Bridge to the shipped TypeScript compiler (tsgo / @typescript/native-preview).
//!
//! The SUI compiler shells out to `tsgo --noEmit` to type-check embedded TS
//! blocks (`<props>`, `<variants>`) — REAL TypeScript inference (autocomplete,
//! "did you mean", type errors) instead of a hand-rolled subset parser. This is
//! the load-bearing bridge: it writes a harness (the props()/variants() surface)
//! + the author's block to a temp dir, runs tsgo, and parses the diagnostics
//! back into structured form the compiler can remap onto `.sui` spans.
use std::path::Path;
use std::process::Command;

/// One diagnostic parsed from tsgo's `path(L,C): error TSxxxx: message` output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TsDiag {
    pub line: usize,
    pub col: usize,
    pub code: String,
    pub message: String,
}

/// Type-check `block` (a TS module body) against `harness` (the `.ts` providing
/// `props()`/`variants()` + the built-in symbols). Returns tsgo's diagnostics —
/// empty means clean. `node`/`tsgo_js` locate the shipped compiler; production
/// resolves them from the toolchain, the spike passes them in.
pub fn typecheck_block(
    harness: &str,
    block: &str,
    out_dir: &Path,
    node: &str,
    tsgo_js: &str,
) -> std::io::Result<Vec<TsDiag>> {
    std::fs::create_dir_all(out_dir)?;
    let harness_path = out_dir.join("sui-embed-harness.ts");
    let block_path = out_dir.join("sui-embed-block.ts");
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
    let rest = line[close + 1..].trim_start_matches(':').trim();
    let rest = rest.strip_prefix("error").unwrap_or(rest).trim();
    let (code, msg) = rest.split_once(':')?;
    if !code.trim().starts_with("TS") {
        return None;
    }
    Some(TsDiag {
        line: line_no,
        col: col_no,
        code: code.trim().to_string(),
        message: msg.trim().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // The props() surface the compiler would generate/ship. Real TS.
    const HARNESS: &str = r#"
export type Prop<T> = { readonly __prop: T }
export const tone: Prop<'default'|'primary'|'info'|'success'|'warning'|'destructive'|'accent'|'muted'> = { __prop: 'default' }
export const size: Prop<'sm'|'md'|'lg'> = { __prop: 'md' }
export const bool: Prop<boolean> = { __prop: false }
export const string: Prop<string> = { __prop: '' }
export function props<T extends Record<string, Prop<unknown> | readonly string[]>>(defs: T): T { return defs }
"#;

    fn tsgo() -> Option<(String, String)> {
        // Spike: locate node + the shipped tsgo. Skips (returns None) when the
        // toolchain isn't present so this stays CI-safe.
        let node = std::env::var("SUI_TSGO_NODE").ok()?;
        let js = std::env::var("SUI_TSGO_JS").ok()?;
        if Path::new(&node).exists() && Path::new(&js).exists() {
            Some((node, js))
        } else {
            None
        }
    }

    #[test]
    fn tsgo_bridge_catches_typo_with_suggestion() {
        let Some((node, js)) = tsgo() else {
            eprintln!("skip: SUI_TSGO_NODE/SUI_TSGO_JS not set");
            return;
        };
        let dir = std::env::temp_dir().join("sui-tscheck-test");

        // VALID: the whole prop surface, tone/size as built-in shorthands.
        let ok = "import { props, tone, size, bool, string } from './sui-embed-harness'\n\
                  export const p = props({ tone, size, variant: ['solid','outline'], disabled: bool, label: string })\n";
        let diags = typecheck_block(HARNESS, ok, &dir, &node, &js).unwrap();
        assert!(diags.is_empty(), "valid block should be clean, got {diags:?}");

        // BROKEN: a typo'd built-in shorthand → autocomplete-grade suggestion.
        // (`size` is imported so tsgo has a near name to suggest.)
        let bad = "import { props, tone, size, bool } from './sui-embed-harness'\n\
                   export const p = props({ tone, sizee, disabled: bool })\n";
        let diags = typecheck_block(HARNESS, bad, &dir, &node, &js).unwrap();
        let hit = diags.iter().find(|d| d.code == "TS2552" || d.message.contains("sizee"));
        assert!(hit.is_some(), "expected a 'Cannot find name sizee' diagnostic, got {diags:?}");
        let hit = hit.unwrap();
        assert!(
            hit.message.contains("Did you mean 'size'"),
            "expected the 'did you mean size' suggestion, got: {}",
            hit.message
        );
    }
}
