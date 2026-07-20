//! Historical `sail-attr-bool-prop` — `attr:` / `prop:` author prefixes are gone.
//! Plain `name={expr}` is the only dynamic field form; the web backend chooses
//! property vs attribute. This rule is retained as a no-op so rule id stability
//! is preserved for override configs; it never emits findings.

use std::path::Path;

use crate::report::Finding;

pub const RULE_ID: &str = "sail-attr-bool-prop";

pub fn check(_path: &Path, _source: &str) -> Vec<Finding> {
    Vec::new()
}
