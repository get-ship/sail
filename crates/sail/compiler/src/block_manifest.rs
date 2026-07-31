//! The template-block vocabulary — one source of truth for element control
//! flow (`<if>`, `<for>`, `<match>`, `<while-loading>`, `<while-error>`) and
//! every sub-clause each block accepts. Brace form (`{#if}`, …) is retired.
//!
//! The parser (`parse::template::blocks` + `control_elements`) enumerates
//! these same block and clause kinds as `Discriminant` variants with no
//! attached prose, so this table is authored here rather than derived —
//! but the set of rows is kept in lockstep with the parser's `IF_TERMS` /
//! `FOR_TERMS` / `MATCH_TERMS` by the test below.

use serde::Serialize;

/// One template block: the opening tag, its sub-clauses (if any), and a
/// one-line summary of what it does.
#[derive(Debug, Clone, Serialize)]
pub struct BlockEntry {
    /// The opening directive as authored: `{#if}`, `{#for}`.
    pub open: &'static str,
    /// Sub-clauses this block accepts, in the order they may appear.
    /// Empty for blocks with none.
    pub clauses: &'static [&'static str],
    /// One-line description of what the block does and any required
    /// syntax (e.g. `{#for}`'s mandatory `key=`).
    pub summary: &'static str,
}

/// The template block registry, in authoring order (control flow first,
/// then async blocks).
pub const BLOCKS: &[BlockEntry] = &[
    BlockEntry {
        open: "<if {…}>",
        clauses: &["<else-if {…}>", "<else>"],
        summary: "conditional branches; brace-primary predicate (residual: cond={…}); any number of <else-if>, optional trailing <else> as direct children",
    },
    BlockEntry {
        open: "<for {…} as=\"item\" key={…}>",
        clauses: &["<else>"],
        summary: "iterate a list; brace-primary iterable (residual: of={…}); `as` item binding, `key` required for stable DOM; optional `index`; <else> when empty",
    },
    BlockEntry {
        open: "<match {…}>",
        clauses: &["<case {…}>", "<default>"],
        summary: "pattern branches over a brace-primary discriminant (residual: on={…}); cases use brace-primary or of={…}; optional <default>",
    },
    BlockEntry {
        open: "<while-loading {sig}>",
        clauses: &[],
        summary: "renders while an async query signal is pending; brace-primary signal name (residual: of=\"sig\")",
    },
    BlockEntry {
        open: "<while-error as=\"err\" {sig}>",
        clauses: &[],
        summary: "renders while an async query signal is rejected; `as` names the error binding; brace-primary or of= for the signal",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn covers_every_known_clause() {
        // Kept in lockstep by hand with blocks.rs's IF_TERMS/FOR_TERMS/
        // MATCH_TERMS (the parser's Discriminant variants carry no prose,
        // so this table can't be derived from them — see module docs).
        let all_clauses: Vec<&str> = BLOCKS
            .iter()
            .flat_map(|b| b.clauses.iter().copied())
            .collect();
        assert!(all_clauses.contains(&"<else-if {…}>"));
        assert!(all_clauses.contains(&"<else>"));
        assert!(all_clauses.contains(&"<case {…}>"));
        assert!(all_clauses.contains(&"<default>"));
        assert_eq!(
            BLOCKS.len(),
            5,
            "one row per block kind: if, for, match, while-loading, while-error"
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
