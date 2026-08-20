//! Shared identifier helpers for the JS emitter.

/// Binding for a mount-form L1 import (`tabs` → `__mod_tabs`).
pub fn l1_controller_ident(tag: &str) -> String {
    format!("__mod_{}", js_ident(tag))
}

/// Specifier the bundler compiles as tagless mount-form. Logical id
/// (`@sail/element/<tag>`), not a filesystem path — the bundler resolves
/// it via `resolve_id`. No `CARGO_MANIFEST_DIR` in the compiler.
pub fn l1_controller_specifier(tag: &str) -> String {
    format!("@sail/element/{tag}")
}

/// Coerce `name` into a valid JS identifier — replace any character
/// that wouldn't be legal in a binding with `_`.
pub fn js_ident(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for (i, c) in name.chars().enumerate() {
        let valid = if i == 0 {
            c.is_ascii_alphabetic() || c == '_' || c == '$'
        } else {
            c.is_ascii_alphanumeric() || c == '_' || c == '$'
        };
        if valid {
            out.push(c);
        } else {
            out.push('_');
        }
    }
    out
}

/// `fetch-url` → `fetchUrl`. Used when building component scope
/// identifiers from prop attribute names.
pub fn camel_of(attr: &str) -> String {
    let mut out = String::with_capacity(attr.len());
    let mut up_next = false;
    for c in attr.chars() {
        if c == '-' {
            up_next = true;
            continue;
        }
        if up_next {
            out.push(c.to_ascii_uppercase());
            up_next = false;
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camel_of_basic() {
        assert_eq!(camel_of("foo-bar"), "fooBar");
        assert_eq!(camel_of("count"), "count");
    }
}
