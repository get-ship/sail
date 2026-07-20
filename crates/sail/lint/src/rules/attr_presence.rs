//! Historical `sail-attr-presence` — taught the `attr:x={cond}` boolean idiom.
//! `attr:` is removed from the author surface; plain `name={cond}` is the form
//! and `bindAttr` still maps true → present. Rule kept as a stable no-op id.

use std::path::Path;

use crate::report::Finding;

pub const RULE_ID: &str = "sail-attr-presence";

pub fn check(_path: &Path, _source: &str) -> Vec<Finding> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn noop() {
        assert!(check(Path::new("x.sail"), r#"<div data-active={active() ? '' : null}></div>"#).is_empty());
    }
}
