//! Extract `<prop>` declarations from a primitive's `.sui` source.
//!
//! Primitives declare their public API at the top of their template:
//!
//! ```text
//! <prop name="variant" values="solid|soft|outline|ghost" default="solid" reflect />
//! <prop name="disabled" type="bool" default="false" />
//! ```
//!
//! Two consumers share this scanner:
//!   1. `examples/regen_primitives.rs` — walks the primitive packages
//!      and produces the canonical `primitives.json` registry.
//!   2. `rules/prop_value.rs` — validates static attribute values in
//!      view templates against that registry.
//!
//! The scanner is intentionally lenient: it accepts attributes in any
//! order, tolerates whitespace, and treats unknown attributes as no-op.
//! It only emits a `PropMeta` for declarations it fully understood.

use std::collections::BTreeMap;

/// Single `<prop>` declaration.
///
/// `values` carries the parsed enum set when the attribute was an enum;
/// `ty` carries the declared `type=` when one was given. They are not
/// mutually exclusive — a `type="string"` with an enum `values` is
/// legal and the enum wins for validation purposes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropMeta {
    pub name: String,
    pub values: Option<Vec<String>>,
    pub ty: Option<String>,
    pub default: Option<String>,
}

/// Walk `<prop ... />` declarations from a primitive source.
///
/// Returns props in source order. Duplicate `name` declarations are
/// not deduped here — callers detect that (the regen tool errors;
/// the lint just trusts the registry it loaded).
pub fn extract_props(source: &str) -> Vec<PropMeta> {
    let mut out = Vec::new();
    let bytes = source.as_bytes();
    let needle = b"<prop";
    let mut i = 0;
    while i + needle.len() < bytes.len() {
        if &bytes[i..i + needle.len()] != needle {
            i += 1;
            continue;
        }
        // Must be followed by whitespace — guard against `<property>`
        // or similar.
        let after = i + needle.len();
        if !matches!(bytes.get(after), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            i = after;
            continue;
        }
        let Some(end_rel) = find_byte(&bytes[after..], b'>') else {
            break;
        };
        let tag_end = after + end_rel;
        let inner = &source[after..tag_end];
        if let Some(prop) = parse_prop_attributes(inner) {
            out.push(prop);
        }
        i = tag_end + 1;
    }
    out
}

/// Parse the attribute soup inside `<prop ... />` (without the angle
/// brackets) into a `PropMeta`. Returns `None` if `name=` is missing
/// — every other field is optional.
fn parse_prop_attributes(inner: &str) -> Option<PropMeta> {
    let attrs = parse_attributes(inner);
    let name = attrs.get("name")?.clone();
    let values = attrs.get("values").map(|raw| {
        raw.split('|')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
    });
    let ty = attrs.get("type").cloned();
    let default = attrs.get("default").cloned();
    Some(PropMeta { name, values, ty, default })
}

/// Tokenize HTML-ish attributes inside an opening tag's interior. The
/// scanner handles `key="value"`, `key='value'`, bare flags (`reflect`),
/// and unquoted simple values (`type=string`). It does NOT decode HTML
/// entities or unicode escapes — primitive prop declarations are
/// authored with plain ASCII values.
fn parse_attributes(inner: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let bytes = inner.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // Skip whitespace.
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        // Trailing `/` (self-closing marker) — stop.
        if bytes[i] == b'/' {
            break;
        }
        // Read key.
        let key_start = i;
        while i < bytes.len() && is_attr_name_byte(bytes[i]) {
            i += 1;
        }
        if i == key_start {
            // Unparseable character at this position — skip and continue.
            i += 1;
            continue;
        }
        let key = inner[key_start..i].to_string();
        // Bare flag (no `=`)?
        if i >= bytes.len() || bytes[i] != b'=' {
            out.insert(key, String::new());
            continue;
        }
        i += 1; // consume '='
        // Quoted or unquoted value.
        let value = if i < bytes.len() && (bytes[i] == b'"' || bytes[i] == b'\'') {
            let quote = bytes[i];
            i += 1;
            let v_start = i;
            while i < bytes.len() && bytes[i] != quote {
                i += 1;
            }
            let v = inner[v_start..i].to_string();
            if i < bytes.len() {
                i += 1; // consume closing quote
            }
            v
        } else {
            let v_start = i;
            while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'/' {
                i += 1;
            }
            inner[v_start..i].to_string()
        };
        out.insert(key, value);
    }
    out
}

fn is_attr_name_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b':'
}

fn find_byte(haystack: &[u8], needle: u8) -> Option<usize> {
    haystack.iter().position(|&b| b == needle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_enum_prop() {
        let src = r#"<prop name="variant" values="solid|soft|outline" default="solid" reflect />"#;
        let p = extract_props(src);
        assert_eq!(p.len(), 1);
        assert_eq!(p[0].name, "variant");
        assert_eq!(
            p[0].values,
            Some(vec!["solid".into(), "soft".into(), "outline".into()])
        );
        assert_eq!(p[0].default.as_deref(), Some("solid"));
    }

    #[test]
    fn extracts_typed_prop() {
        let src = r#"<prop name="disabled" type="bool" default="false" reflect />"#;
        let p = extract_props(src);
        assert_eq!(p.len(), 1);
        assert_eq!(p[0].name, "disabled");
        assert_eq!(p[0].values, None);
        assert_eq!(p[0].ty.as_deref(), Some("bool"));
    }

    #[test]
    fn handles_multiple_props_and_attrs_in_any_order() {
        let src = r#"
            <prop reflect default="md" name="size" values="sm|md|lg" />
            <prop name="loading" type="bool" />
            <prop  name="label"  type="string"  default=""  />
        "#;
        let p = extract_props(src);
        assert_eq!(p.len(), 3);
        assert_eq!(p[0].name, "size");
        assert_eq!(p[0].default.as_deref(), Some("md"));
        assert_eq!(p[1].name, "loading");
        assert_eq!(p[2].name, "label");
    }

    #[test]
    fn skips_property_lookalike() {
        // <property> should not be parsed as <prop>.
        let src = r#"<property name="x" />"#;
        assert!(extract_props(src).is_empty());
    }

    #[test]
    fn missing_name_skipped() {
        let src = r#"<prop type="string" />"#;
        assert!(extract_props(src).is_empty());
    }

    #[test]
    fn enum_values_trim_whitespace() {
        let src = r#"<prop name="tone" values=" a | b |c" />"#;
        let p = extract_props(src);
        assert_eq!(
            p[0].values,
            Some(vec!["a".into(), "b".into(), "c".into()])
        );
    }

    #[test]
    fn single_quotes_supported() {
        let src = r#"<prop name='variant' values='solid|soft' />"#;
        let p = extract_props(src);
        assert_eq!(p[0].name, "variant");
        assert_eq!(p[0].values.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn unquoted_simple_value_supported() {
        let src = r#"<prop name=disabled type=bool />"#;
        let p = extract_props(src);
        assert_eq!(p[0].name, "disabled");
        assert_eq!(p[0].ty.as_deref(), Some("bool"));
    }
}
