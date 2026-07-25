//! The `<scope>` grammar — one source of truth for every construct a
//! `<scope>` block accepts: the prop declaration forms, type aliases,
//! Style consts (`const name = style.…`), and residual `compound:`.
//!
//! **Paint (Style product):** host `sx=` with chain / match / when / compound
//! and scope Style consts. Residual CVA key→sx maps type as unions without
//! paint. Shared `tone` / `size` bare defaults expand to stdlib convention
//! unions — ordinary reflected props.
//!
//! A sibling of `block_manifest` / `directive_manifest`: the closed grammar
//! is data, not a hand-listed paragraph on the docs page. The parser
//! (`parse::scope_block`) is the enforcement point; every row here carries
//! an `example` that the tests below actually compile (or, for refused
//! rows, assert fails to compile), so this table cannot drift from the
//! grammar without failing the build.

use serde::Serialize;

/// One `<scope>` grammar construct: its authored form and what it means.
#[derive(Debug, Clone, Serialize)]
pub struct ScopeEntry {
    /// Short name of the construct, e.g. "union prop".
    pub form: &'static str,
    /// The declaration as authored, e.g. `open = false`.
    pub syntax: &'static str,
    /// One-line description of the semantics.
    pub summary: &'static str,
    /// A complete component source exercising the construct. Compiled by
    /// the tests; rendered on the reference page as the worked example.
    pub example: &'static str,
}

/// The `<scope>` grammar, in documentation order: props forms first, then
/// type aliases, then CVA / compound.
pub const SCOPE_GRAMMAR: &[ScopeEntry] = &[
    ScopeEntry {
        form: "boolean prop",
        syntax: "open = false",
        summary: "a true/false literal default infers a boolean attribute, reflected onto the host",
        example: "<scope>\n  props: { open = false }\n</scope>\n<p>{open}</p>",
    },
    ScopeEntry {
        form: "number prop",
        syntax: "count = 0",
        summary: "a numeric literal default infers a number prop",
        example: "<scope>\n  props: { count = 0 }\n</scope>\n<p>{count}</p>",
    },
    ScopeEntry {
        form: "string prop",
        syntax: "label = 'Hi'",
        summary: "a quoted literal default infers a string prop",
        example: "<scope>\n  props: { label = 'Hi' }\n</scope>\n<p>{label}</p>",
    },
    ScopeEntry {
        form: "annotated prop",
        syntax: "title: string",
        summary: "an annotation may stand alone (string / number / boolean); annotation and default are each optional, but a field needs at least one",
        example: "<scope>\n  props: { title: string }\n</scope>\n<p>{title}</p>",
    },
    ScopeEntry {
        form: "union prop",
        syntax: "variant: 'solid' | 'soft' = 'solid'",
        summary: "a union of string literals, reflected as an attribute; the default is required and must be a member — there is no first-member magic",
        example: "<scope>\n  props: { variant: 'solid' | 'soft' = 'solid' }\n</scope>\n<p>{variant}</p>",
    },
    ScopeEntry {
        form: "shared tone/size prop",
        syntax: "tone = 'primary'  ·  size = 'md'",
        summary: "Bare tone/size defaults type as the shared stdlib unions (tone: default·primary·info·success·warning·destructive·accent·muted; size: xs·sm·md·lg) and reflect as ordinary props — CVA axes by convention",
        example: "<scope>\n  props: { tone = 'primary', size = 'md' }\n</scope>\n<p>{tone}</p>",
    },
    ScopeEntry {
        form: "identity prop",
        syntax: "rows: Row[] = []",
        summary: "any other TS type is an identity prop — a plain reactive JS property (set with plain name={expr}), never an attribute, no coercion; the type is carried verbatim into the .d.ts",
        example: "<scope>\n  type Row = { id: string }\n  props: { rows: Row[] = [] }\n</scope>\n<p>{rows.length}</p>",
    },
    ScopeEntry {
        form: "as-cast default",
        syntax: "items = [] as string[]",
        summary: "a literal default with an `as` cast — the cast supplies the type when no annotation was written",
        example: "<scope>\n  props: { items = [] as string[] }\n</scope>\n<p>{items.length}</p>",
    },
    ScopeEntry {
        form: "kebab-case name",
        syntax: "'line-numbers' = false",
        summary: "quote a prop name to declare a hyphenated attribute",
        example: "<scope>\n  props: { 'line-numbers' = false }\n</scope>\n<p>ok</p>",
    },
    ScopeEntry {
        form: "documented prop",
        syntax: "/** Shown when the list is empty. */ emptyText = ''",
        summary: "a JSDoc block immediately before a field becomes the prop's description — it feeds the CEM and the generated docs tables",
        example: "<scope>\n  props: {\n    /** Shown when the list is empty. */\n    emptyText = ''\n  }\n</scope>\n<p>{emptyText}</p>",
    },
    ScopeEntry {
        form: "type alias",
        syntax: "type Row = { id: string }  ·  export type …",
        summary: "TypeScript type aliases, threaded verbatim into the compiled module; `export type` makes the alias importable from the emitted .d.ts",
        example: "<scope>\n  export type Item = { id: string, label: string }\n  props: { selected: Item | null = null }\n</scope>\n<p>ok</p>",
    },
    ScopeEntry {
        form: "Style const",
        syntax: "const filled = style.bg(primary).padX(2)",
        summary: "compile-time Style value (chain / match / when / compound / list / ref); resolved when host sx= refs the name — not a runtime binding",
        example: "<scope>\n  const filled = style.bg(primary)\n  props: { open = false }\n</scope>\n<div sx={filled}>ok</div>",
    },
    ScopeEntry {
        form: "CVA-in-props axis (residual)",
        syntax: "size: { sm: 'px-2', lg: 'px-4' }",
        summary: "residual key→string map: keys become the reflected union type (first = default); values do not emit paint — use host match / Style",
        example: "<scope>\n  props: {\n    density: { cozy: 'p-2', compact: 'p-1' }\n  }\n</scope>\n<p>ok</p>",
    },
    ScopeEntry {
        form: "compound variant (residual)",
        syntax: "compound: [ { when: { variant: 'solid', size: 'lg' }, sx: '…' } ]",
        summary: "residual combination rows — accepted so unmigrated files parse, but no CSS is emitted; product form is host compound({…}, style.…)",
        example: "<scope>\n  props: {\n    variant: { solid: 'font-semibold', soft: 'font-normal' }\n    size: { sm: 'p-1', lg: 'p-4' }\n  }\n  compound: [ { when: { variant: 'solid', size: 'lg' }, sx: 'tracking-wide' } ]\n</scope>\n<p>ok</p>",
    },
];

/// One member kind `<scope>` refuses, with the redirect the error names.
#[derive(Debug, Clone, Serialize)]
pub struct ScopeRefusal {
    pub syntax: &'static str,
    pub instead: &'static str,
    /// A source that must FAIL to compile — asserted by the tests.
    pub example: &'static str,
}

/// What `<scope>` is not: nothing in the block executes. These members are
/// refused with an error naming the alternative.
pub const SCOPE_REFUSALS: &[ScopeRefusal] = &[
    ScopeRefusal {
        syntax: "import …",
        instead: "inline the type — <scope> is types and literals only",
        example: "<scope>\n  import type { X } from './x'\n  props: { open = false }\n</scope>\n<p>ok</p>",
    },
    ScopeRefusal {
        syntax: "const/let/var computation",
        instead: "Style const (`const name = style.…`), a literal prop default, or state in <controller>",
        example: "<scope>\n  const n = 1\n  props: { open = false }\n</scope>\n<p>ok</p>",
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{compile, CompileMode, CompileOpts, ComponentMeta};

    fn opts() -> CompileOpts {
        CompileOpts {
            mode: CompileMode::Component,
            component_meta: Some(ComponentMeta { name: "scope-doc-demo".into() }),
            ..Default::default()
        }
    }

    #[test]
    fn every_grammar_example_compiles() {
        for row in SCOPE_GRAMMAR {
            if let Err(e) = compile(row.example, opts()) {
                panic!(
                    "scope manifest row `{}` has a non-compiling example:\n{}\n-> {}",
                    row.form, row.example, e.message
                );
            }
        }
    }

    #[test]
    fn every_refusal_example_fails() {
        for row in SCOPE_REFUSALS {
            assert!(
                compile(row.example, opts()).is_err(),
                "scope refusal `{}` unexpectedly compiled — the grammar accepts what this table documents as refused",
                row.syntax
            );
        }
    }

    #[test]
    fn forms_are_unique() {
        let mut forms: Vec<&str> = SCOPE_GRAMMAR.iter().map(|r| r.form).collect();
        let before = forms.len();
        forms.sort_unstable();
        forms.dedup();
        assert_eq!(forms.len(), before, "SCOPE_GRAMMAR must have unique form names");
    }
}
