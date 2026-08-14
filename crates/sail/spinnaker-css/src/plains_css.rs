//! Plains token → CSS longhands.

use spinnaker::plains::PlainSpec;

/// Lower one token/keyword on a plain to CSS longhands.
/// Concerns / composites with mixed decls stay in `concern_css`.
pub fn lower_plain_token(
    spec: &PlainSpec,
    bare: &str,
    negative: bool,
) -> Option<Vec<(String, String)>> {
    if spec.slots.is_empty() {
        return Some(Vec::new());
    }
    let css_val = if spec.keywords.iter().any(|k| *k == bare) {
        keyword_css(spec, bare)
    } else if let Some(scale) = spec.scale {
        let v = spinnaker::scales::lower(scale, bare)?;
        if negative {
            format!("calc(-1 * {v})")
        } else {
            v
        }
    } else {
        return None;
    };
    Some(
        spec.slots
            .iter()
            .map(|p| ((*p).to_string(), css_val.clone()))
            .collect(),
    )
}

fn keyword_css(spec: &PlainSpec, bare: &str) -> String {
    match (spec.name, bare) {
        ("boxSizing", "border") => "border-box".into(),
        ("boxSizing", "content") => "content-box".into(),
        ("content", "empty") => "''".into(),
        ("tapHighlight", "none") => "transparent".into(),
        ("underline", "true") => "underline".into(),
        ("underline", "none") => "none".into(),
        ("underline", "line-through") => "line-through".into(),
        ("objectPosition", k) | ("origin", k) => k.replace('-', " "),
        _ => bare.to_string(),
    }
}
