//! Clash slots and domain token validation.
//!
//! Product names live on `vocab.rs`. Concern / axis rules live on
//! `concerns.rs`. Plains accept data lives on `plains.rs`. CSS lowering
//! lives in `spinnaker-css`.

use crate::scales;

use super::ir::{PaintArg, PaintOp, StyleDiagnostic};
use crate::plains::{find_plain, PlainSpec};

/// Plains emit after vocab confirms Plain. Not a name atlas.
fn emit_plain(name: &str) -> Option<&'static PlainSpec> {
    let row = crate::vocab::find_vocab(name)?;
    if row.tag != crate::vocab::VocabTag::Plain {
        return None;
    }
    find_plain(row.name)
}

/// Unique closest product setter for typo correction (confluent only).
/// Returns `None` when no candidate is close enough or two candidates tie.
///
/// Ranking: lower Levenshtein first; then prefer names that are a prefix of the
/// typo (or vice versa) so `padd` → `pad` beats `padX` at the same distance;
/// then shorter absolute length delta. Exact ties remain non-confluent.
pub fn closest_unique_setter(typo: &str) -> Option<&'static str> {
    let typo_l = typo.to_ascii_lowercase();
    // score: (distance, prefix_penalty 0|1, len_delta)
    let mut best: Option<(usize, u8, usize, &'static str)> = None;
    let mut tied = false;
    for row in crate::vocab::vocab_rows() {
        let name = row.name;
        let name_l = name.to_ascii_lowercase();
        let cap = std::cmp::max(2, name.len() / 2 + 1);
        let d = setter_levenshtein(&typo_l, &name_l, cap + 1);
        if d > cap || d == 0 {
            continue;
        }
        let prefix_ok = typo_l.starts_with(&name_l) || name_l.starts_with(&typo_l);
        let prefix_penalty: u8 = if prefix_ok { 0 } else { 1 };
        let len_delta = typo_l.len().abs_diff(name_l.len());
        let score = (d, prefix_penalty, len_delta, name);
        match best {
            None => {
                best = Some(score);
                tied = false;
            }
            Some(b) => {
                let b_key = (b.0, b.1, b.2);
                let s_key = (score.0, score.1, score.2);
                if s_key < b_key {
                    best = Some(score);
                    tied = false;
                } else if s_key == b_key {
                    tied = true;
                }
            }
        }
    }
    if tied {
        return None;
    }
    best.map(|(_, _, _, n)| n)
}

fn setter_levenshtein(a: &str, b: &str, cap: usize) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.is_empty() {
        return b.len().min(cap);
    }
    if b.is_empty() {
        return a.len().min(cap);
    }
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut curr: Vec<usize> = vec![0; b.len() + 1];
    for i in 1..=a.len() {
        curr[0] = i;
        let mut row_min = curr[0];
        for j in 1..=b.len() {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
            if curr[j] < row_min {
                row_min = curr[j];
            }
        }
        if row_min > cap {
            return cap;
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[b.len()].min(cap)
}

/// Paint slots THIS op writes. Concern ops key on `(concern, kind)` so a
/// base stroke and a focus ring coexist; two ring calls in one scope clash.
/// Plains key on the emit-slot names the spec publishes.
pub fn op_slots(op: &PaintOp) -> Vec<&'static str> {
    if crate::concerns::find_concern(&op.setter).is_some() {
        return crate::concerns::concern_clash_keys(op).unwrap_or_default();
    }
    emit_plain(&op.setter)
        .map(|spec| spec.slots.to_vec())
        .unwrap_or_default()
}

/// Validate paint arg against setter table; fail closed.
pub fn validate_op(op: &PaintOp) -> Result<(), StyleDiagnostic> {
    if let Some(d) = crate::concerns::retired_diag(&op.setter, &op.source) {
        return Err(d);
    }
    if let Some(concern) = crate::concerns::find_concern(&op.setter) {
        return crate::concerns::validate_concern_op(concern, op);
    }
    let Some(spec) = emit_plain(&op.setter) else {
        return Err(StyleDiagnostic::UnknownSetter {
            setter: op.setter.clone(),
            source: op.source.clone(),
            suggestion: closest_unique_setter(&op.setter).map(|s| s.to_string()),
        });
    };
    // Marker setters with empty arg. `truncate` is retired — type.wrap.
    if matches!(
        spec.name,
        "group" | "scrollbarThin" | "isolate" | "srOnly" | "controlFrame" | "controlLift" | "press"
    ) {
        return Ok(());
    }
    // Composite positional setters: style.border(1, border) = width + color.
    if op.args.len() > 1 {
        return validate_composite_op(spec, op);
    }
    match op.first_arg() {
        PaintArg::Bind(_) => Ok(()),
        // Bound paths validated at parse; emit expands before lower.
        PaintArg::MemberPath { .. } => Ok(()),
        PaintArg::Slots(_) => Err(StyleDiagnostic::BadToken {
            setter: op.setter.clone(),
            value: op.first_arg().display(),
            source: op.source.clone(),
        }),
        PaintArg::Token(tok) => {
            let (neg, bare) = strip_neg(tok);
            if neg && !spec.negatable {
                return Err(StyleDiagnostic::BadToken {
                    setter: op.setter.clone(),
                    value: tok.clone(),
                    source: op.source.clone(),
                });
            }
            // SPEC-D1 / THEMES-1-ENFORCE: component surface public names are
            // emit labels only — not Style author Tokens. Host units use
            // dotted MemberPath. App Style gets a theme-owned teaching error.
            if spec.scale == Some("color")
                && crate::component_surfaces::is_component_public_name(bare)
            {
                return Err(StyleDiagnostic::ThemeOwnedToken {
                    name: tok.clone(),
                    source: op.source.clone(),
                });
            }
            if token_is_accepted(spec, bare) {
                return Ok(());
            }
            Err(StyleDiagnostic::BadToken {
                setter: op.setter.clone(),
                value: tok.clone(),
                source: op.source.clone(),
            })
        }
    }
}

/// Domain token accept: keywords, scale membership, kebab/underscore aliases
/// of scale tokens, and a few closed extras that are not CSS (ident names,
/// unitless rotate degrees). CSS success is not a language rule.
fn token_is_accepted(spec: &PlainSpec, bare: &str) -> bool {
    if spec.keywords.contains(&bare) {
        return true;
    }
    if let Some(scale) = spec.scale {
        if scales::contains(scale, bare) {
            return true;
        }
        for cand in token_candidates(bare) {
            if scales::contains(scale, &cand) {
                return true;
            }
        }
    }
    extra_plain_token(spec.name, bare)
}

/// Product extras that are not a scale step and not a listed keyword.
fn extra_plain_token(name: &str, bare: &str) -> bool {
    match name {
        "containerName" => is_container_ident(bare),
        "rotateX" | "rotateY" | "rotateZ" => {
            !bare.is_empty() && bare.chars().all(|c| c.is_ascii_digit())
        }
        _ => false,
    }
}

fn is_container_ident(t: &str) -> bool {
    !t.is_empty()
        && t.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

/// Setters that take two positional args: (width-token?, color-token).
/// One arg stays the current color/width/style-by-value behavior.
pub fn is_border_composite(name: &str) -> bool {
    matches!(
        name,
        "border"
            | "borderX"
            | "borderY"
            | "borderT"
            | "borderB"
            | "borderL"
            | "borderR"
            | "borderS"
            | "borderE"
    )
}

/// Validate a composite (two-arg) paint op positionally.
fn validate_composite_op(spec: &PlainSpec, op: &PaintOp) -> Result<(), StyleDiagnostic> {
    if op.args.len() != 2 {
        return Err(StyleDiagnostic::BadArity {
            setter: op.setter.clone(),
            got: op.args.len(),
            source: op.source.clone(),
        });
    }
    if is_border_composite(spec.name) {
        validate_border_width_arg(spec, op, &op.args[0])?;
        validate_border_color_arg(spec, op, &op.args[1])?;
        return Ok(());
    }
    if spec.name == "colorDeepen" || spec.name == "colorMix" {
        // Same arity/token rules: color token + closed percent ramp.
        return validate_color_deepen_args(spec, op);
    }
    Err(StyleDiagnostic::BadArity {
        setter: op.setter.clone(),
        got: op.args.len(),
        source: op.source.clone(),
    })
}

/// Position 0 of a composite border call: a width token on the closed
/// border-width ramp (or `control-stroke` on plain `border`, mirroring
/// `borderWidth`).
fn validate_border_width_arg(
    spec: &PlainSpec,
    op: &PaintOp,
    arg: &PaintArg,
) -> Result<(), StyleDiagnostic> {
    match arg {
        PaintArg::Bind(_) => Ok(()),
        PaintArg::MemberPath { .. } | PaintArg::Slots(_) => Err(StyleDiagnostic::BadToken {
            setter: op.setter.clone(),
            value: arg.display(),
            source: op.source.clone(),
        }),
        PaintArg::Token(tok) => {
            let (neg, bare) = strip_neg(tok);
            if !neg
                && (scales::contains("border-width", bare)
                    || (spec.name == "border" && bare == "control-stroke"))
            {
                return Ok(());
            }
            Err(StyleDiagnostic::BadToken {
                setter: op.setter.clone(),
                value: tok.clone(),
                source: op.source.clone(),
            })
        }
    }
}

/// Position 1 of a composite border call: a color token / keyword. The
/// resolved declarations must all be `*-color` (a width or style token in
/// this position is a positional error, fail closed).
fn validate_border_color_arg(
    _spec: &PlainSpec,
    op: &PaintOp,
    arg: &PaintArg,
) -> Result<(), StyleDiagnostic> {
    match arg {
        PaintArg::Bind(_) | PaintArg::MemberPath { .. } => Ok(()),
        PaintArg::Slots(_) => Err(StyleDiagnostic::BadToken {
            setter: op.setter.clone(),
            value: arg.display(),
            source: op.source.clone(),
        }),
        PaintArg::Token(tok) => {
            let (neg, bare) = strip_neg(tok);
            if !neg
                && (matches!(bare, "transparent" | "current" | "currentColor")
                    || scales::contains("color", bare))
            {
                return Ok(());
            }
            if !neg {
                for cand in token_candidates(bare) {
                    if scales::contains("color", &cand) {
                        return Ok(());
                    }
                }
            }
            Err(StyleDiagnostic::BadToken {
                setter: op.setter.clone(),
                value: tok.clone(),
                source: op.source.clone(),
            })
        }
    }
}

/// Validate `colorDeepen(token, pct)`: token on color scale, pct in 5..95 step 5.
fn validate_color_deepen_args(_spec: &PlainSpec, op: &PaintOp) -> Result<(), StyleDiagnostic> {
    // Position 0: color token or bind.
    match &op.args[0] {
        PaintArg::Bind(_) | PaintArg::MemberPath { .. } => {}
        PaintArg::Slots(_) => {
            return Err(StyleDiagnostic::BadToken {
                setter: op.setter.clone(),
                value: op.args[0].display(),
                source: op.source.clone(),
            });
        }
        PaintArg::Token(tok) => {
            let (neg, bare) = strip_neg(tok);
            if neg
                || (!scales::contains("color", bare)
                    && !matches!(bare, "transparent" | "current" | "currentColor"))
            {
                return Err(StyleDiagnostic::BadToken {
                    setter: op.setter.clone(),
                    value: tok.clone(),
                    source: op.source.clone(),
                });
            }
        }
    }
    // Position 1: closed percent ramp 5..95 step 5, or bind.
    match &op.args[1] {
        PaintArg::Bind(_) => {}
        PaintArg::MemberPath { .. } | PaintArg::Slots(_) => {
            return Err(StyleDiagnostic::BadToken {
                setter: op.setter.clone(),
                value: op.args[1].display(),
                source: op.source.clone(),
            });
        }
        PaintArg::Token(tok) => {
            let (neg, bare) = strip_neg(tok);
            if neg {
                return Err(StyleDiagnostic::BadToken {
                    setter: op.setter.clone(),
                    value: tok.clone(),
                    source: op.source.clone(),
                });
            }
            let ok = bare
                .parse::<i32>()
                .is_ok_and(|n| (5..=95).contains(&n) && n % 5 == 0);
            if !ok {
                return Err(StyleDiagnostic::BadToken {
                    setter: op.setter.clone(),
                    value: tok.clone(),
                    source: op.source.clone(),
                });
            }
        }
    }
    Ok(())
}

pub fn strip_neg(tok: &str) -> (bool, &str) {
    if let Some(rest) = tok.strip_prefix('-') {
        if !rest.is_empty() {
            return (true, rest);
        }
    }
    (false, tok)
}

pub fn token_candidates(tok: &str) -> Vec<String> {
    let mut out = vec![tok.to_string()];
    // camelCase → kebab
    let mut kebab = String::new();
    for (i, c) in tok.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                kebab.push('-');
            }
            kebab.extend(c.to_lowercase());
        } else {
            kebab.push(c);
        }
    }
    if kebab != tok {
        out.push(kebab.clone());
    }
    out.push(tok.replace('_', "-"));
    out
}

#[cfg(test)]
mod tests {
    use super::super::ir::{Style, StyleDiagnostic};
    use super::*;

    /// Product parse (channel-only).
    fn parse(src: &str) -> (Style, Vec<StyleDiagnostic>) {
        super::super::parse::parse(src)
    }

    fn only_op(src: &str) -> PaintOp {
        let (style, diags) = parse(src);
        assert!(
            diags.is_empty(),
            "{src}: {:?}",
            diags.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        match style {
            Style::Set(op) => op,
            other => panic!("{src}: expected single Set, got {other:?}"),
        }
    }

    #[test]
    fn absorbed_border_forms_are_retired() {
        for src in ["style({ borderX(2, primary) })"] {
            let (_, diags) = parse(src);
            assert!(
                diags
                    .iter()
                    .any(|d| matches!(d, StyleDiagnostic::RetiredSetter { .. })),
                "{src}: {:?}",
                diags.iter().map(|d| d.message()).collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn concern_clash_keys_are_kind_aware() {
        let op = only_op("style({ border(subtle) })");
        let slots = op_slots(&op);
        assert!(
            slots.contains(&"border:stroke:all")
                || (slots.contains(&"border:stroke:t") && slots.contains(&"border:stroke:b")),
            "{slots:?}"
        );
        let op = only_op("style({ border({ kind: ring, color: primary }) })");
        assert_eq!(op_slots(&op), vec!["border:ring"]);
        let op = only_op("style({ bg(primary) })");
        assert_eq!(op_slots(&op), vec!["bg"]);
    }

    #[test]
    fn validate_accepts_scale_and_rejects_raw() {
        let (_, ok) = parse("style({ duration(fast) })");
        assert!(
            ok.is_empty(),
            "{:?}",
            ok.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
        let (_, bad) = parse("style({ duration(16ms) })");
        assert!(
            bad.iter()
                .any(|d| matches!(d, StyleDiagnostic::BadToken { .. })),
            "{:?}",
            bad.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn outline_and_ring_setters_are_retired() {
        for src in [
            "style({ outline(solid) })",
            "style({ ring(primary) })",
            "style({ truncate() })",
            "style({ borderImage(none) })",
        ] {
            let (_, diags) = parse(src);
            assert!(
                diags.iter().any(|d| matches!(
                    d,
                    StyleDiagnostic::RetiredSetter { .. } | StyleDiagnostic::UnknownSetter { .. }
                )),
                "{src}: {:?}",
                diags.iter().map(|d| d.message()).collect::<Vec<_>>()
            );
        }
    }
}
