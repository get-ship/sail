//! Directive-level helpers consumed by the emitter.
//!
//! Today the directive lowering is essentially an identity walk —
//! the parser produces the final IR shape and the emitter reads it
//! directly. This module exists so the emit step has a single import
//! when it needs declarative info (e.g. "is this prop bool-typed?")
//! without poking at the IR enum variants directly.

use crate::ir::{PropDecl, PropKind};

/// Stable JS string literal for a prop's runtime-builder type.
pub fn prop_builder(kind: &PropKind) -> &'static str {
    match kind {
        PropKind::String => "string",
        PropKind::Bool => "bool",
        PropKind::Number => "number",
        PropKind::Union(_) => "union",
        PropKind::Identity(_) => "property",
    }
}

/// Coerce a default-attribute string into a JS literal of the target type.
pub fn prop_default_literal(p: &PropDecl) -> Option<String> {
    let raw = p.default.as_deref()?;
    Some(match p.kind {
        PropKind::Bool => {
            if raw == "true" {
                "true".into()
            } else {
                "false".into()
            }
        }
        PropKind::Number => raw.to_string(),
        PropKind::String | PropKind::Union(_) => format!("\"{}\"", escape_string(raw)),
        // Identity props: the scope grammar only admits literal defaults, so
        // the raw source text IS a valid JS literal (`[]`, `{ a: 1 }`) —
        // emit it verbatim. Dropping it here left `prop.property()` with no
        // default, so `ctx.<prop>()` returned undefined instead of the
        // declared literal.
        PropKind::Identity(_) => raw.to_string(),
    })
}

pub fn escape_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}
