//! Bridge to the shipped TypeScript compiler (tsgo / @typescript/native-preview).
//!
//! The SUI compiler shells out to `tsgo --noEmit` to type-check embedded TS
//! blocks (`<props>`, `<variants>`) — REAL TypeScript inference (autocomplete,
//! "did you mean", type errors) instead of a hand-rolled subset parser. This is
//! the load-bearing bridge: it writes a harness (the props()/variants() surface)
//! + the author's block to a temp dir, runs tsgo, and parses the diagnostics
//! back into structured form the compiler can remap onto `.sui` spans.
use crate::parse::preprocess::{extract_component_body, parse_object_fields};
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

/// Type-check the `<component>` block's `variants:` object against the declared
/// `props:` — REAL TS validates the table's group keys are declared props and
/// its value keys are members of that prop's union (catches `variant: { sold }`,
/// a typo'd group like `varient`, a value outside the union). The sx *strings*
/// are the sx lint's job; this owns the *shape* — killing the hand-parse's
/// silent shadowing. Returns tsgo diagnostics (empty = clean); `Ok(vec![])` when
/// there is no `<component>` block or no `variants:`.
pub fn check_component_variants(
    source: &str,
    out_dir: &Path,
    node: &str,
    tsgo_js: &str,
) -> std::io::Result<Vec<TsDiag>> {
    let Some(body) = extract_component_body(source) else {
        return Ok(Vec::new());
    };
    let fields = parse_object_fields(&body);
    let find = |k: &str| {
        fields
            .iter()
            .find(|(fk, _)| fk == k)
            .map(|(_, v)| v.clone())
    };
    let (Some(props), Some(variants)) = (find("props"), find("variants")) else {
        return Ok(Vec::new());
    };
    let harness = gen_variants_harness(&props);
    let block = format!("import {{ variants }} from './sui-embed-harness'\nvariants({variants})\n");
    typecheck_block(&harness, &block, out_dir, node, tsgo_js)
}

/// Build the `variants()` harness from the declared props: a `PropVals` map
/// (prop → value union) and a `variants()` signature whose table keys must be
/// declared props and whose value keys must be members of that prop's union.
fn gen_variants_harness(props: &str) -> String {
    const TONE: &str =
        "'default'|'primary'|'info'|'success'|'warning'|'destructive'|'accent'|'muted'";
    const SIZE: &str = "'xs'|'sm'|'md'|'lg'";
    let mut members = Vec::new();
    for (name, val) in parse_object_fields(props) {
        let n = name.as_str();
        let v = val.trim();
        let ty = if n == "tone" {
            TONE.to_string()
        } else if n == "size" {
            SIZE.to_string()
        } else if v.contains('|') {
            // Real-TS union — already valid TS, pass through verbatim.
            v.to_string()
        } else if v.starts_with('[') {
            v.trim_start_matches('[')
                .trim_end_matches(']')
                .split(',')
                .filter(|s| !s.trim().is_empty())
                .map(|s| format!("'{}'", s.trim().trim_matches(['\'', '"'])))
                .collect::<Vec<_>>()
                .join("|")
        } else if v == "boolean" || v == "bool" {
            "boolean".to_string()
        } else if v == "number" {
            "number".to_string()
        } else {
            "string".to_string()
        };
        members.push(format!("  {name}: {ty};"));
    }
    format!(
        "type Sx = string\n\
         type PropVals = {{\n{}\n}}\n\
         export function variants(\n  \
         _t: Partial<{{ [K in keyof PropVals]: Partial<Record<Extract<PropVals[K], string>, Sx>> }}>\n      \
         & {{ compound?: Array<{{ when: Partial<PropVals>; sx: Sx }}> }},\n\
         ): void {{}}\n",
        members.join("\n")
    )
}

/// Type-check the synthetic controller + render pair for an SFC `source`: emit
/// the pair (composing the inline `<controller>` block), write them alongside a
/// tsconfig whose `paths` map `@ship/sui-runtime` / `@ship/sui-dom` to the real
/// package sources under `workspace_root`, and run tsgo. A template expression
/// that reads a name the controller's typed scope does not provide
/// (`scope.dragVisible` with no `dragVisible`) is a hard type error here — the
/// template↔controller contract, gated at compile time. This turns the
/// `$scope.X is not a function` runtime crash class into a build error.
///
/// Returns tsgo diagnostics (empty = clean). `Ok(vec![])` when there's no inline
/// controller to check against (bare fragment) or the pair can't be emitted.
pub fn check_synthetic_controller(
    source: &str,
    source_dir: &Path,
    out_dir: &Path,
    node: &str,
    tsgo_js: &str,
    workspace_root: &Path,
) -> std::io::Result<Vec<TsDiag>> {
    use crate::emit::synthetic::{emit_synthetic, AppManifest, AuthoredController};
    use crate::emit::writer::js_string;
    use crate::parse::{self, preprocess};
    use crate::CompileMode;

    // SFC: strip the inline `<controller>` block. No block → nothing to check
    // here (the paired-file model is composed elsewhere); a bare fragment has no
    // scope contract to violate.
    let (stripped, inline_ctrl) = preprocess::extract_controller_block(source);
    let Some(body) = inline_ctrl else {
        return Ok(Vec::new());
    };
    let Ok(program) = parse::parse(&stripped) else {
        return Ok(Vec::new());
    };

    std::fs::create_dir_all(out_dir)?;
    // Materialize the inline controller, then compose it into the typed pair.
    std::fs::write(out_dir.join("view.controller.ts"), preprocess::wrap_controller_body(&body))?;
    let authored = AuthoredController { specifier: "./view.controller.ts".to_string() };
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
    std::fs::write(out_dir.join("view.synthetic.controller.ts"), &out.controller_ts)?;
    std::fs::write(out_dir.join("view.render.ts"), &out.render_ts)?;

    // Resolve the controller's sibling imports so the isolated check doesn't
    // TS2307 on `import type { Scope } from './X.sui'` / `import {..} from
    // './utils'`. Copy sibling `.ts`/`.d.ts` verbatim, and GENERATE
    // `<sib>.sui.d.ts` for every sibling `.sui` — the emitted d.ts exports the
    // `Scope` type and TS resolves `./X.sui` to `./X.sui.d.ts`. Best-effort: a
    // sibling that fails to compile is skipped (the tsgo run reports the real
    // error if the missing type actually matters).
    if let Ok(entries) = std::fs::read_dir(source_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            let Some(name) = p.file_name().and_then(|s| s.to_str()) else {
                continue;
            };
            if name.ends_with(".ts") {
                let _ = std::fs::copy(&p, out_dir.join(name));
            } else if name.ends_with(".sui") {
                if let Ok(sib) = std::fs::read_to_string(&p) {
                    let stem = name.strip_suffix(".sui").unwrap_or(name);
                    let opts = crate::CompileOpts {
                        setup_url: None,
                        mode: crate::CompileMode::Component,
                        source_path: Some(p.clone()),
                        component_meta: Some(crate::ComponentMeta {
                            name: stem.to_string(),
                            shadow: true,
                        }),
                        emit_dts: true,
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

    // Map the bare package specifiers to the real workspace sources so tsgo
    // checks `scope.X` against the REAL runtime types. `baseUrl` was removed in
    // tsgo v7 — absolute `paths` targets work without it.
    let runtime = workspace_root.join("packages/sui/runtime/src/index.ts");
    let dom = workspace_root.join("packages/sui/dom/src/index.ts");
    let tsconfig = format!(
        "{{\n  \"compilerOptions\": {{\n    \"noEmit\": true, \"strict\": true, \"skipLibCheck\": true,\n\
         \"moduleResolution\": \"bundler\", \"module\": \"esnext\", \"target\": \"esnext\",\n\
         \"allowImportingTsExtensions\": true,\n    \"paths\": {{\n      \"@ship/sui-runtime\": [{}],\n\
         \"@ship/sui-dom\": [{}]\n    }}\n  }},\n  \
         \"include\": [\"view.render.ts\", \"view.synthetic.controller.ts\", \"view.controller.ts\"]\n}}\n",
        js_string(&runtime.to_string_lossy()),
        js_string(&dom.to_string_lossy()),
    );
    let tsconfig_path = out_dir.join("tsconfig.json");
    std::fs::write(&tsconfig_path, tsconfig)?;

    let output = Command::new(node).arg(tsgo_js).arg("-p").arg(&tsconfig_path).output()?;
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    // This gate owns exactly ONE contract: does a template expression read a
    // name / member the controller's typed scope does not provide? Keep only
    // those diagnostics — a missing property (`scope.dragVisible`) or an unbound
    // name (`scope.countt`). Everything else tsgo emits over the generated pair
    // (implicit-any params `TS7006`, module-resolution `TS2307`, strict-null
    // noise…) is harness strictness, NOT a template↔controller violation, and
    // must not fail the build.
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The template↔controller gate: an inline-controller SFC whose template
    /// reads a name the controller's typed scope provides typechecks clean;
    /// a typo (`countt`) is a hard tsgo error. Proves the `$scope.X is not a
    /// function` class is caught at compile time.
    #[test]
    fn synthetic_controller_gate_catches_template_typo() {
        let Some((node, js)) = tsgo() else {
            eprintln!("skip: SUI_TSGO_NODE/SUI_TSGO_JS not set");
            return;
        };
        let ws = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("workspace root above crates/sui/compiler")
            .to_path_buf();
        let dir = std::env::temp_dir().join("sui-synthetic-gate-test");

        const CTRL: &str = "<controller>\n\
            import { controller, signal } from '@ship/sui-runtime'\n\
            export default controller({ setup: () => ({ count: signal(0) }) })\n\
            </controller>\n";

        // VALID: the template reads `count`, which the scope provides.
        // Self-contained controller → a nonexistent source_dir skips the
        // sibling-resolution copy/generate step.
        let no_sibs = Path::new("/nonexistent-sui-siblings");
        let ok = format!("{CTRL}<button on:click={{() => count.set(count() + 1)}}>{{count()}}</button>");
        let diags = check_synthetic_controller(&ok, no_sibs, &dir, &node, &js, &ws).unwrap();
        assert!(diags.is_empty(), "valid SFC must typecheck clean, got {diags:?}");

        // BROKEN: `countt` is not a scope member — a hard type error.
        let bad = format!("{CTRL}<button>{{countt()}}</button>");
        let diags = check_synthetic_controller(&bad, no_sibs, &dir, &node, &js, &ws).unwrap();
        assert!(
            diags.iter().any(|d| d.message.contains("countt")),
            "template typo scope.countt must be a compile error, got {diags:?}"
        );
    }

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

    #[test]
    fn variants_check_catches_undeclared_value() {
        let Some((node, js)) = tsgo() else {
            eprintln!("skip: SUI_TSGO_NODE/SUI_TSGO_JS not set");
            return;
        };
        let dir = std::env::temp_dir().join("sui-variants-check-test");

        // VALID: every variant value is a declared member of the prop union.
        let ok = "<component>{ props: { variant: ['solid','outline'], size },\n\
                  variants: { variant: { solid: 'bg:tone', outline: 'border:tone' } } }\
                  </component>\n<button>x</button>";
        assert!(
            check_component_variants(ok, &dir, &node, &js).unwrap().is_empty(),
            "valid variants should be clean"
        );

        // BROKEN: `sold` is not a declared value of `variant: 'solid'|'outline'`.
        // The hand-parse would silently emit `:host([variant='sold'])` CSS that
        // never matches; tsgo rejects it.
        let bad = "<component>{ props: { variant: ['solid','outline'] },\n\
                   variants: { variant: { solid: 'bg:tone', sold: 'border:tone' } } }\
                   </component>\n<button>x</button>";
        let diags = check_component_variants(bad, &dir, &node, &js).unwrap();
        assert!(
            !diags.is_empty(),
            "undeclared variant value 'sold' should be rejected, got {diags:?}"
        );
    }
}
