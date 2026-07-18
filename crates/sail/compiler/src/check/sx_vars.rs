//! `(--x)` resolution law — the seam sx reads use for runtime state.
//!
//! Resolution order, checked once per distinct `(--name)` read found inside a
//! `sx="…"` attribute value: global token vars → declared props →
//! setup-returned signals → refusal (with a did-you-mean). A name that never
//! appears as a `(--x)` read gets no CSS wiring at all — this pass only ever
//! looks at names actually read, never at everything a component happens to
//! own.
//!
//! A token-var match needs no component-local wiring (the token already has
//! a global value). A prop or signal match is returned so the emitter can
//! register a typed CSS `@property` and reflect the value onto the host as
//! an inline custom property (seed at mount + on every write) — see
//! `emit::styles::register_properties_install_js`.
//!
//! **Action-owned properties ride the token bucket.** `use:virtualRow`,
//! `use:eventSpan`, `use:rect`, `use:pin`, `use:zoomPan`, `use:worldTransform`,
//! `use:maxWidth`, `use:cellPad`, `use:virtualSpacer`,
//! `use:swatch`, `use:portEdge`, `use:targetCursor`, `use:spotlight`,
//! `use:gridTracks`, `use:rangeFill` each write a fixed, hardcoded custom
//! property (`--row-y`, `--evt-top`, …) that is neither a prop nor a
//! setup-returned signal of the COMPONENT that reads it via `(--name)` — the
//! action owns that name the same way a design token owns its name. They're
//! listed in `sail_lint::tokens::CONVENTION_TOKENS` (the pre-existing
//! allowlist for "legitimate `var(--x)`, not a global-value token") rather
//! than given a fourth bucket here, so they resolve exactly like a token:
//! accepted, no component-local `@property`/host-reflection wiring.

use std::collections::BTreeSet;

use crate::check::controller_parse::{ParsedController, ScopeKind};
use crate::diagnostics::{CompileError, Span};
use crate::emit::util::camel_of;
use crate::ir::{PropKind, Program};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SxVarSource {
    Prop,
    Signal,
}

/// A `(--x)` read that resolved to a component-owned prop or signal — the
/// pair the emitter wires `@property` + host reflection for.
#[derive(Debug, Clone)]
pub struct SxVarBinding {
    /// The custom-property name as written, without `--` (kebab-case: `row-y`).
    pub css_name: String,
    /// The scope member it reads (camelCase: `rowY`), i.e. `$scope.<name>`.
    pub scope_name: String,
    pub source: SxVarSource,
    /// True when the value is a plain number — eligible for `syntax:
    /// '<number>'`, which is what makes a CSS transition on the property
    /// tween natively. False falls back to `syntax: '*'` (registered, but
    /// untyped — no native interpolation).
    pub numeric: bool,
}

/// Resolve every `(--x)` read in this source's `sx="…"` attributes. `Ok`
/// carries the prop/signal matches to wire at emit time (token matches are
/// dropped — they need no component-local emission). `Err` is the first
/// refusal encountered, in source order.
pub fn resolve(
    source: &str,
    program: &Program,
    controller: Option<&ParsedController>,
) -> Result<Vec<SxVarBinding>, CompileError> {
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut out: Vec<SxVarBinding> = Vec::new();

    for (val_start, val) in crate::parse::preprocess::find_sx_attr_values(source) {
        for (rel, name) in find_var_refs(val) {
            if !seen.insert(name.to_string()) {
                continue;
            }
            if sail_lint::tokens::registry().contains(name) {
                continue;
            }

            let scope_name = camel_of(name);

            if let Some(p) = program.props.iter().find(|p| camel_of(&p.name) == scope_name) {
                out.push(SxVarBinding {
                    css_name: name.to_string(),
                    scope_name,
                    source: SxVarSource::Prop,
                    numeric: matches!(p.kind, PropKind::Number),
                });
                continue;
            }

            if let Some(c) = controller {
                if let Some(m) = c
                    .returned
                    .iter()
                    .find(|m| m.name == scope_name && m.kind == ScopeKind::Signal)
                {
                    let init = c
                        .source
                        .get(m.value_span.start..m.value_span.end)
                        .unwrap_or("");
                    out.push(SxVarBinding {
                        css_name: name.to_string(),
                        scope_name,
                        source: SxVarSource::Signal,
                        numeric: signal_is_numeric(init),
                    });
                    continue;
                }
            }

            let offset = val_start + rel;
            let span = Span::new(offset, offset + name.len() + 5); // "(--" + name + ")"
            return Err(refusal(name, program, controller, span));
        }
    }
    Ok(out)
}

/// Find every `(--name)` occurrence in an sx attribute value, returning the
/// name (without `--`) and its byte offset relative to the start of `val`.
/// Deliberately permissive about WHERE inside the value it appears — an atom
/// like `translate-y-(--row-y)` embeds it as a suffix; this only extracts the
/// reference, leaving atom/utility shape validation to `sail-sx`'s own lint.
fn find_var_refs(val: &str) -> Vec<(usize, &str)> {
    let mut out = Vec::new();
    let mut i = 0;
    let bytes = val.as_bytes();
    while let Some(rel) = val[i..].find("(--") {
        let start = i + rel;
        let name_start = start + 3;
        if let Some(close_rel) = val[name_start..].find(')') {
            let name = &val[name_start..name_start + close_rel];
            if !name.is_empty()
                && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            {
                out.push((start, name));
            }
            i = name_start + close_rel + 1;
        } else {
            break;
        }
        if i > bytes.len() {
            break;
        }
    }
    out
}

/// A `signal(<literal>)` initializer whose first argument is a bare numeric
/// literal — the common case (`signal(220)`, `signal(-1)`, `signal(0.5)`).
/// Anything else (an object, a computed expression, a typed empty call) is
/// NOT assumed numeric — under-typing only costs native transitions, not
/// correctness, whereas over-typing would emit a `<number>` syntax for a
/// value CSS can't parse, which breaks the registration outright.
fn signal_is_numeric(init_text: &str) -> bool {
    let Some(rest) = init_text.trim_start().strip_prefix("signal") else {
        return false;
    };
    let rest = rest.trim_start();
    let rest = match rest.strip_prefix('<') {
        Some(r) => match r.find('>') {
            Some(i) => &r[i + 1..],
            None => return false,
        },
        None => rest,
    };
    let Some(inner) = rest.trim_start().strip_prefix('(') else {
        return false;
    };
    let bytes = inner.trim_start().as_bytes();
    let mut i = 0;
    if i < bytes.len() && (bytes[i] == b'-' || bytes[i] == b'+') {
        i += 1;
    }
    let mut has_digit = false;
    while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b'.') {
        has_digit = has_digit || bytes[i].is_ascii_digit();
        i += 1;
    }
    has_digit && (i >= bytes.len() || matches!(bytes[i], b')' | b',' | b' ' | b'\t' | b'\n'))
}

fn refusal(
    css_name: &str,
    program: &Program,
    controller: Option<&ParsedController>,
    span: Span,
) -> CompileError {
    let mut candidates: Vec<String> = program.props.iter().map(|p| kebab_of(&p.name)).collect();
    if let Some(c) = controller {
        candidates.extend(
            c.returned
                .iter()
                .filter(|m| m.kind == ScopeKind::Signal)
                .map(|m| kebab_of(&m.name)),
        );
    }
    candidates.extend(sail_lint::tokens::registry().iter().map(|s| s.to_string()));

    let mut msg = format!(
        "`(--{css_name})` does not match a global token, a declared prop, or a setup-returned \
         signal. sx reads runtime state through `(--name)` — the name must be a design token, \
         a `<scope>` prop, or a value `setup()` returns via `signal(...)`."
    );
    if let Some(s) = closest(css_name, &candidates) {
        msg.push_str(&format!(" Did you mean `(--{s})`?"));
    }
    CompileError::new(msg).with_span(span).with_category("sail-sx-var")
}

fn kebab_of(name: &str) -> String {
    let mut out = String::with_capacity(name.len() + 4);
    for (i, c) in name.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i != 0 {
                out.push('-');
            }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

/// Closest candidate by Levenshtein distance, capped so a wild guess isn't
/// offered as a "did you mean" (a miss should say nothing, not mislead).
fn closest(name: &str, candidates: &[String]) -> Option<String> {
    let max_dist = (name.len() / 2).max(2);
    candidates
        .iter()
        .map(|c| (c, levenshtein(name, c)))
        .filter(|(_, d)| *d <= max_dist)
        .min_by_key(|(_, d)| *d)
        .map(|(c, _)| c.clone())
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0usize; b.len() + 1];
    for i in 1..=a.len() {
        cur[0] = i;
        for j in 1..=b.len() {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            cur[j] = (prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::PropDecl;

    fn prop(name: &str, kind: PropKind) -> PropDecl {
        PropDecl {
            name: name.to_string(),
            kind,
            default: None,
            reflect: false,
            bindable: false,
            description: None,
            span: Span::new(0, 0),
        }
    }

    fn program_with_props(props: Vec<PropDecl>) -> Program {
        Program {
            props,
            styles: String::new(),
            scope_types: String::new(),
            template: Vec::new(),
            source_len: 0,
            span: Span::new(0, 0),
        }
    }

    #[test]
    fn a_prop_match_returns_a_binding() {
        let program = program_with_props(vec![prop("glowRadius", PropKind::Number)]);
        let source = r#"<div sx="w-(--glow-radius)"></div>"#;
        let out = resolve(source, &program, None).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].scope_name, "glowRadius");
        assert_eq!(out[0].source, SxVarSource::Prop);
        assert!(out[0].numeric);
    }

    #[test]
    fn a_signal_match_returns_a_binding() {
        let program = program_with_props(vec![]);
        let controller = crate::check::controller_parse::parse(
            Default::default(),
            "controller((ctx) => { const hue = signal(220); return { hue }; })".to_string(),
        )
        .unwrap();
        let source = r#"<div sx="bg-(--hue)"></div>"#;
        let out = resolve(source, &program, Some(&controller)).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].scope_name, "hue");
        assert_eq!(out[0].source, SxVarSource::Signal);
        assert!(out[0].numeric);
    }

    #[test]
    fn a_token_var_is_unaffected_and_wires_nothing() {
        let program = program_with_props(vec![]);
        let source = r#"<div sx="bg-(--surface-2)"></div>"#;
        let out = resolve(source, &program, None).unwrap();
        assert!(out.is_empty(), "a global token needs no component-local wiring");
    }

    #[test]
    fn an_unmatched_name_refuses_with_did_you_mean() {
        let program = program_with_props(vec![prop("rowY", PropKind::Number)]);
        let source = r#"<div sx="top-(--row-x)"></div>"#;
        let err = resolve(source, &program, None).unwrap_err();
        assert!(err.message.contains("does not match"));
        assert!(err.message.contains("row-y"), "{}", err.message);
    }

    #[test]
    fn a_signal_not_returned_from_setup_is_not_reachable() {
        let program = program_with_props(vec![]);
        let controller = crate::check::controller_parse::parse(
            Default::default(),
            "controller((ctx) => { const hue = signal(220); return {}; })".to_string(),
        )
        .unwrap();
        let source = r#"<div sx="bg-(--hue)"></div>"#;
        let err = resolve(source, &program, Some(&controller)).unwrap_err();
        assert!(err.message.contains("does not match"));
    }

    #[test]
    fn an_unread_name_emits_no_wiring() {
        // `rowY` is a real prop, but nothing reads `(--row-y)` — this pass
        // only ever looks at names actually read.
        let program = program_with_props(vec![prop("rowY", PropKind::Number)]);
        let source = r#"<div class="static"></div>"#;
        let out = resolve(source, &program, None).unwrap();
        assert!(out.is_empty());
    }
}
