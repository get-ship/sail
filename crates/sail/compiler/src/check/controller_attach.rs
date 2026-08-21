//! Platform-free controller attach type — no DOM, no CSS.
//!
//! Validates `ident` / `ident()` bindings for parameterized `use:` /
//! `on:` / live `name={expr}` on an L1 element with a `<controller>`.
//! Core emits declarative `path: Vec<String>` (`:nth-of-type` steps,
//! empty = host). Web emitter owns CSS string + querySelector.

use crate::CompileError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControllerAttach {
    Use {
        action: String,
        param: String,
        /// Host-relative tree path (`:nth-of-type` steps), empty = host.
        path: Vec<String>,
    },
    On {
        event: String,
        handler: String,
        path: Vec<String>,
    },
    Bind {
        name: String,
        expr: String,
        path: Vec<String>,
    },
}

/// A controller binding: bare `ident` or `ident()` — anything richer is
/// page `$scope` again.
pub fn inst_field(source: &str) -> Result<String, CompileError> {
    let s = source.trim();
    let ident = s.strip_suffix("()").unwrap_or(s).trim();
    let mut chars = ident.chars();
    let Some(first) = chars.next() else {
        return Err(CompileError::new(
            "element mode: empty controller binding on a parameterized use:/on:/attr",
        )
        .with_category("sail-element-controller"));
    };
    let ok = (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_');
    if !ok {
        return Err(CompileError::new(format!(
            "element mode: `{s}` is not a controller binding ident — parameterized \
             use:/on:/attr resolve against the controller return, not the page"
        ))
        .with_category("sail-element-controller"));
    }
    Ok(s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inst_field_is_bare_ident() {
        assert_eq!(inst_field("keys").unwrap(), "keys");
        assert_eq!(inst_field("keys()").unwrap(), "keys()");
        assert!(inst_field("keys.foo").is_err());
        assert!(inst_field("").is_err());
    }
}
