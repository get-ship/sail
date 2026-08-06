//! Match exhaustiveness for two surfaces:
//!
//! 1. **Style** `match(prop, {…})` on a closed-union scope prop — WARNING
//!    when members lack an arm (partial paint is legitimate).
//! 2. **Template** `<match {disc}>` with kind arms on LiveState /
//!    MutationState closed unions — COMPILE ERROR naming the missing arm.
//!    `{else => …}` covers any remainder.
//!
//! Style remains warning-only. Template kind-unions gate the build.

use std::collections::HashSet;

use sail_lint::style::Style;

use crate::diagnostics::{DiagSeverity, Diagnostic, line_col};
use crate::ir::{Instr, MatchInstr, MatchTest, Program, PropKind};

/// Style-match rule id (warnings).
pub const RULE: &str = "sail-style-match-exhaustive";

/// Template match exhaustiveness rule id (errors).
pub const TEMPLATE_RULE: &str = "sail-match-exhaustive";

const MATCH_OPEN: &str = "match(";

/// LiveState closed kinds.
const LIVE_STATE_KINDS: &[&str] = &["loading", "error", "ok"];
/// MutationState closed kinds.
const MUTATION_STATE_KINDS: &[&str] = &["idle", "loading", "ok", "error"];

/// Collect Style match warnings + template match exhaustiveness errors.
pub fn collect(program: &Program, source: &str) -> Vec<Diagnostic> {
    let mut out = Vec::new();

    // ── Template `<match>` kind-union exhaustiveness (errors) ──────────
    walk_template_match(&program.template, source, &mut out);

    // ── Style match (warnings) ─────────────────────────────────────────
    // Closed-union props: name → members.
    let unions: Vec<(&str, &[String])> = program
        .props
        .iter()
        .filter_map(|p| match &p.kind {
            PropKind::Union(members) => Some((p.name.as_str(), members.as_slice())),
            _ => None,
        })
        .collect();
    if unions.is_empty() {
        return out;
    }

    // `<scope>` Style consts — `const tones = match(variant, {…})`.
    let env = crate::parse::scope_block::style_env_from_source(source);
    let mut seen_consts: HashSet<&str> = HashSet::new();
    for (body_start, body) in scope_bodies(source) {
        for (name, style) in &env {
            if seen_consts.contains(name.as_str()) {
                continue;
            }
            let Some(rel) = find_const_decl(body, name) else {
                continue;
            };
            seen_consts.insert(name.as_str());
            check_style(
                style,
                &unions,
                source,
                body_start + rel,
                &body[rel..],
                &mut out,
            );
        }
    }

    // Host `style={…}` / `style="…"` values.
    for (vstart, val) in crate::parse::preprocess::find_sx_attr_values(source) {
        let (style, _diags) = sail_lint::style::parse(val);
        check_style(&style, &unions, source, vstart, val, &mut out);
    }

    out
}

fn walk_template_match(instrs: &[Instr], source: &str, out: &mut Vec<Diagnostic>) {
    for i in instrs {
        match i {
            Instr::Match(m) => {
                check_template_match(m, source, out);
                for c in &m.cases {
                    walk_template_match(&c.body, source, out);
                }
                if let Some(d) = &m.default {
                    walk_template_match(d, source, out);
                }
            }
            Instr::If(iff) => {
                for b in &iff.branches {
                    walk_template_match(&b.body, source, out);
                }
                if let Some(e) = &iff.else_body {
                    walk_template_match(e, source, out);
                }
            }
            Instr::For(f) => {
                walk_template_match(&f.body, source, out);
                if let Some(e) = &f.else_body {
                    walk_template_match(e, source, out);
                }
            }
            Instr::When(w) => {
                walk_template_match(&w.body, source, out);
                if let Some(e) = &w.else_body {
                    walk_template_match(e, source, out);
                }
            }
            Instr::Element(el) => walk_template_match(&el.children, source, out),
            Instr::StaticText { .. } | Instr::DynamicText { .. } | Instr::Comment { .. } => {}
        }
    }
}

fn check_template_match(m: &MatchInstr, source: &str, out: &mut Vec<Diagnostic>) {
    // `{else => …}` covers every remainder.
    if m.default.is_some() {
        return;
    }
    let mut kind_arms: HashSet<&str> = HashSet::new();
    for c in &m.cases {
        if let MatchTest::Kind(k) = &c.test {
            kind_arms.insert(k.as_str());
        }
    }
    if kind_arms.is_empty() {
        return; // pure expression-equality match — no closed-kind law
    }

    // idle present → MutationState; else any LiveState kind → LiveState.
    // Custom kind words alone do not invent a closed union.
    let expected: &[&str] = if kind_arms.contains("idle") {
        MUTATION_STATE_KINDS
    } else if kind_arms
        .iter()
        .any(|k| matches!(*k, "loading" | "error" | "ok"))
    {
        LIVE_STATE_KINDS
    } else {
        return;
    };

    let missing: Vec<&str> = expected
        .iter()
        .copied()
        .filter(|k| !kind_arms.contains(k))
        .collect();
    if missing.is_empty() {
        return;
    }

    let quoted = missing
        .iter()
        .map(|m| format!("`{m}`"))
        .collect::<Vec<_>>()
        .join(", ");
    let (line, col) = line_col(source, m.span.start);
    out.push(Diagnostic {
        line,
        col,
        end_line: line,
        end_col: col + 1,
        severity: DiagSeverity::Error,
        message: format!(
            "<match> is not exhaustive — missing arm(s) for {quoted}; \
             add `{{{first} => …}}` (or `{{else => …}}` to cover the rest).",
            first = missing[0]
        ),
        rule: Some(TEMPLATE_RULE.to_string()),
    });
}

/// Walk `style` pre-order; every `Match` whose scrutinee resolves to a
/// closed-union prop and misses members becomes one warning. `region` is the
/// source text the style was parsed from, starting at `region_start` — the
/// k-th `match(` in walk order is the k-th in textual order, which pins the
/// diagnostic span without re-parsing for offsets.
fn check_style(
    style: &Style,
    unions: &[(&str, &[String])],
    source: &str,
    region_start: usize,
    region: &str,
    out: &mut Vec<Diagnostic>,
) {
    let mut matches = Vec::new();
    collect_matches(style, &mut matches);
    if matches.is_empty() {
        return;
    }
    let mut cursor = 0usize;
    for (scrutinee, arms) in matches {
        let match_offset = region[cursor..]
            .find(MATCH_OPEN)
            .map(|rel| cursor + rel)
            .unwrap_or(cursor);
        cursor = match_offset + MATCH_OPEN.len();

        // Only a single-segment scrutinee can name a scope prop.
        if scrutinee.contains('.') || scrutinee.is_empty() {
            continue;
        }
        let Some((_, members)) = unions.iter().find(|(name, _)| *name == scrutinee) else {
            continue;
        };
        let covered: HashSet<&str> = arms.iter().map(|(key, _)| key.as_str()).collect();
        let missing: Vec<&str> = members
            .iter()
            .map(|m| m.as_str())
            .filter(|m| !covered.contains(m))
            .collect();
        if missing.is_empty() {
            continue;
        }

        let quoted = missing
            .iter()
            .map(|m| format!("`{m}`"))
            .collect::<Vec<_>>()
            .join(", ");
        let offset = region_start + match_offset;
        let (line, col) = line_col(source, offset);
        out.push(Diagnostic {
            line,
            col,
            end_line: line,
            end_col: col + MATCH_OPEN.len() - 1,
            severity: DiagSeverity::Warning,
            message: format!(
                "Style match on `{scrutinee}` is not exhaustive — missing arm(s) for {quoted}; \
                 an unmatched variant renders with no paint from this match."
            ),
            rule: Some(RULE.to_string()),
        });
    }
}

/// Pre-order harvest of every `Match` node (scrutinee + arm keys). `Ref` is
/// NOT resolved — a referenced scope const is checked at its declaration, so
/// resolving here would double-report.
fn collect_matches<'a>(style: &'a Style, out: &mut Vec<(&'a str, &'a [(String, Style)])>) {
    match style {
        Style::Match { scrutinee, arms } => {
            out.push((scrutinee.as_str(), arms.as_slice()));
            for (_, body) in arms {
                collect_matches(body, out);
            }
        }
        Style::Chain(items) | Style::List(items) => {
            for item in items {
                collect_matches(item, out);
            }
        }
        Style::Compound { body, .. } => collect_matches(body, out),
        Style::When { entries } => {
            for (_, body) in entries {
                collect_matches(body, out);
            }
        }
        Style::Empty | Style::Set(_) | Style::Ref(_) | Style::MotionAttach { .. } => {}
    }
}

/// Every `<scope>…</scope>` body as `(body_start_offset, body_text)` — the
/// same tag walk `scope_block::style_env_from_source` uses.
fn scope_bodies(source: &str) -> Vec<(usize, &str)> {
    let mut out = Vec::new();
    let mut rest = source;
    let mut base = 0usize;
    while let Some(open_rel) = rest.find("<scope") {
        let after_open_tag = &rest[open_rel..];
        let Some(gt_rel) = after_open_tag.find('>') else {
            break;
        };
        let tag_head = &after_open_tag[..gt_rel];
        if tag_head
            .as_bytes()
            .get(6)
            .is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'-')
        {
            rest = &after_open_tag[1..];
            base += open_rel + 1;
            continue;
        }
        let body_start = base + open_rel + gt_rel + 1;
        let after_body = &rest[open_rel + gt_rel + 1..];
        let Some(close_rel) = after_body.find("</scope>") else {
            break;
        };
        out.push((body_start, &after_body[..close_rel]));
        let advance = open_rel + gt_rel + 1 + close_rel + "</scope>".len();
        rest = &rest[advance..];
        base += advance;
    }
    out
}

/// Offset of `const <name>` in a `<scope>` body (word-boundary matched), so
/// the warning can point at the declaration the author wrote.
fn find_const_decl(body: &str, name: &str) -> Option<usize> {
    let bytes = body.as_bytes();
    let is_ident = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
    let mut i = 0usize;
    while i + 5 + name.len() <= bytes.len() {
        let Some(rel) = body[i..].find("const") else {
            return None;
        };
        let kw = i + rel;
        let before_ok = kw == 0 || !is_ident(bytes[kw - 1]);
        let mut j = kw + 5;
        while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t') {
            j += 1;
        }
        if before_ok
            && body[j..].starts_with(name)
            && j + name.len() < bytes.len()
            && !is_ident(bytes[j + name.len()])
        {
            return Some(kw);
        }
        i = kw + 5;
    }
    None
}
