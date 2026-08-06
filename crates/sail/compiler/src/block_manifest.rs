//! The template-block vocabulary — one source of truth for element control
//! flow (`<if>`, `<for>`, `<match>`, `<when>`). Brace form (`{#if}`, …) is
//! retired. Arms use `{pattern as binding => subtree}` inside the block.
//!
//! The parser (`parse::template::blocks` + `control_elements`) enumerates
//! these same block kinds; this table stays in lockstep with grammar /
//! surface drift tests.

use serde::Serialize;

/// One template block: the opening tag, its sub-clauses (if any), and a
/// one-line summary of what it does.
#[derive(Debug, Clone, Serialize)]
pub struct BlockEntry {
    /// The opening directive as authored: `<if {…}>`.
    pub open: &'static str,
    /// Sub-clauses this block accepts, in the order they may appear.
    /// Empty for blocks with none.
    pub clauses: &'static [&'static str],
    /// One-line description of what the block does and any required
    /// syntax (e.g. `{#for}`'s mandatory `key=`).
    pub summary: &'static str,
}

/// The template block registry — four blocks only.
pub const BLOCKS: &[BlockEntry] = &[
    BlockEntry {
        open: "<if {…}>",
        clauses: &["{else-if {…} => …}", "{else => …}"],
        summary: "conditional branches; brace-primary predicate; free then-body; optional {else-if {cond} => …} and trailing {else => …} arms",
    },
    BlockEntry {
        open: "<for {items as item} key={…}>",
        clauses: &["{else => …}"],
        summary: "iterate a list; brace-primary `iterable as item[, index]`; `key` required for stable DOM; optional {else => …} when empty",
    },
    BlockEntry {
        open: "<match {…}>",
        clauses: &["{pattern as binding => …}", "{else => …}"],
        summary: "pattern arms over a brace-primary discriminant; kind arms (`ok`/`error`/`loading`/`idle`) bind payloads; optional {else => …}; missing LiveState/MutationState arm is a compile error",
    },
    BlockEntry {
        open: "<when {token}>",
        clauses: &["{else => …}"],
        summary: "structural viewport branch on a theme breakpoint token (mobile/tablet/desktop); optional {else => …} is the SSR/mobile-first default; matchMedia swap on the client",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn covers_every_known_clause() {
        let all_clauses: Vec<&str> = BLOCKS
            .iter()
            .flat_map(|b| b.clauses.iter().copied())
            .collect();
        assert!(all_clauses.iter().any(|c| c.contains("else-if")));
        assert!(all_clauses.iter().any(|c| c.contains("{else =>")));
        assert!(all_clauses.iter().any(|c| c.contains("pattern as binding")));
        assert_eq!(
            BLOCKS.len(),
            4,
            "one row per block kind: if, for, match, when"
        );
    }

    #[test]
    fn open_tags_are_unique() {
        let mut opens: Vec<&str> = BLOCKS.iter().map(|b| b.open).collect();
        let before = opens.len();
        opens.sort_unstable();
        opens.dedup();
        assert_eq!(opens.len(), before, "BLOCKS must have unique open tags");
    }
}
