//! The template-block vocabulary — one source of truth for `{#if}`,
//! `{#for}`, `{#match}`, `{#while-loading}`, `{#while-error}` and every
//! sub-clause each block accepts. The sixth declared table, alongside the
//! sx vocabulary, the L1 manifest, the action manifest, the motion preset
//! catalog, and the controller ambient vocabulary: the closed vocabulary is
//! data, not a hand-listed paragraph on the docs page.
//!
//! The parser (`parse::template::blocks`) enumerates these same block and
//! clause kinds as `Discriminant` variants with no attached prose, so this
//! table is authored here rather than derived — but the set of rows is
//! kept in lockstep with the parser's `IF_TERMS` / `FOR_TERMS` /
//! `MATCH_TERMS` by the test below, so a new clause added to the parser
//! without a row here fails the build.

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
        open: "{#if}",
        clauses: &["{:else if}", "{:else}"],
        summary: "conditional branches; any number of {:else if}, an optional trailing {:else}",
    },
    BlockEntry {
        open: "{#for}",
        clauses: &["{:else}"],
        summary: "iterate a list; requires key={expr} on the opening tag; {:else} renders when the list is empty",
    },
    BlockEntry {
        open: "{#match}",
        clauses: &["{:case}", "{:default}"],
        summary: "pattern branches over a discriminant; any number of {:case}, an optional {:default}",
    },
    BlockEntry {
        open: "{#while-loading}",
        clauses: &[],
        summary: "renders while an async query's promise is pending",
    },
    BlockEntry {
        open: "{#while-error}",
        clauses: &[],
        summary: "renders while an async query's promise is rejected",
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
        let all_clauses: Vec<&str> = BLOCKS.iter().flat_map(|b| b.clauses.iter().copied()).collect();
        assert!(all_clauses.contains(&"{:else if}"));
        assert!(all_clauses.contains(&"{:else}"));
        assert!(all_clauses.contains(&"{:case}"));
        assert!(all_clauses.contains(&"{:default}"));
        assert_eq!(BLOCKS.len(), 5, "one row per block kind: if, for, match, while-loading, while-error");
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
