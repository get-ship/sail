//! The SAIL component-tag prefix — the single source of truth for the
//! `sail-` custom-element prefix length. Two call sites used to encode
//! "the prefix is N chars" as a bare integer (`< 6`, `tag[5..]`); when the
//! prefix was renamed those integers went stale silently. Anything that
//! needs to strip or measure the prefix should go through
//! [`strip_sail_tag_prefix`] instead of re-deriving the length.

/// The required prefix for SAIL custom-element tags (e.g. `sail-button`).
pub const SAIL_TAG_PREFIX: &str = "sail-";

/// Strip [`SAIL_TAG_PREFIX`] from `tag`, returning `None` if the prefix is
/// absent OR nothing follows it (a bare `"sail-"` is not a valid tag).
pub fn strip_sail_tag_prefix(tag: &str) -> Option<&str> {
    let rest = tag.strip_prefix(SAIL_TAG_PREFIX)?;
    if rest.is_empty() { None } else { Some(rest) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_valid_prefix() {
        assert_eq!(strip_sail_tag_prefix("sail-button"), Some("button"));
    }

    #[test]
    fn rejects_bare_prefix() {
        assert_eq!(strip_sail_tag_prefix("sail-"), None);
    }

    #[test]
    fn rejects_missing_prefix() {
        assert_eq!(strip_sail_tag_prefix("button"), None);
        assert_eq!(strip_sail_tag_prefix("ship-button"), None);
    }
}
