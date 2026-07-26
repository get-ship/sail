//! Style `match` exhaustiveness — a warning when a `match(prop, {…})` that
//! dispatches on a CLOSED-union scope prop (`variant: 'solid' | 'soft' | …`)
//! leaves union members without an arm.
//!
//! The Style grammar has no catch-all arm (`parse_match` accepts only word
//! keys; nothing special-cases `default` / `else` / `_`), so an uncovered
//! member is always worth naming. It stays a WARNING, never an error: a
//! partial match is a legitimate way to leave a variant unstyled.
//!
//! Scrutinees that are not closed-union props (string / number / free-form
//! TS types, dotted paths like `row.tone`) emit nothing.

use std::collections::HashSet;

use sail_lint::style::Style;

use crate::diagnostics::{line_col, DiagSeverity, Diagnostic};
use crate::ir::{Program, PropKind};

/// Lint-style rule id carried on every diagnostic this pass emits.
pub const RULE: &str = "sail-style-match-exhaustive";

const MATCH_OPEN: &str = "match(";

/// Collect exhaustiveness warnings for every Style `match` in `<scope>`
/// Style consts and host `sx=` values. Warnings only — never gates the build.
pub fn collect(program: &Program, source: &str) -> Vec<Diagnostic> {
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
        return Vec::new();
    }

    let mut out = Vec::new();

    // `<scope>` Style consts — `const tones = match(variant, {…})`. The env
    // harvest already parsed each RHS; we re-locate the declaration for the
    // span. A name is checked once even if several `<scope>` blocks repeat it.
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

    // Host `sx={…}` / `sx="…"` values.
    for (vstart, val) in crate::parse::preprocess::find_sx_attr_values(source) {
        let (style, _diags) = sail_lint::style::parse(val);
        check_style(&style, &unions, source, vstart, val, &mut out);
    }

    out
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
