//! Vocabulary export for the tree-sitter grammar generator.
//!
//! The grammar package (`packages/sail/grammar`) consumes this JSON to
//! regenerate `grammar.js` and `queries/highlights.scm`. Keeping the
//! dump in the native compiler (not the wasm-only `completion_vocabulary()`)
//! means the build-time generator can run without a wasm target.

use serde::Serialize;

/// Closed vocabulary shared by the compiler and the tree-sitter grammar.
/// Anything the grammar hard-codes from the compiler's manifests must flow
/// through this dump so the generated files stay in lockstep.
#[derive(Debug, Serialize)]
pub struct GrammarVocabulary {
    pub version: u32,
    pub void_tags: &'static [&'static str],
    pub directive_prefixes: &'static [&'static str],
    pub directive_elements: &'static [&'static str],
    pub blocks: &'static [crate::block_manifest::BlockEntry],
    pub events: Vec<&'static str>,
    pub modifiers: &'static [crate::directive_manifest::ModifierEntry],
    pub bind_targets: &'static [crate::directive_manifest::BindTargetEntry],
    pub foreign_idioms: &'static [crate::directive_manifest::ForeignIdiomEntry],
    pub motion_value_forms: &'static [crate::directive_manifest::MotionValueForm],
}

/// Emit the grammar's vocabulary as a JSON value. The generator script
/// reads this value and substitutes it into the grammar template.
pub fn to_json() -> serde_json::Value {
    serde_json::to_value(GrammarVocabulary {
        version: 1,
        void_tags: crate::l1_manifest::VOID_TAGS,
        directive_prefixes: crate::directive_manifest::DIRECTIVE_PREFIXES,
        directive_elements: crate::directive_manifest::DIRECTIVE_ELEMENTS,
        blocks: crate::block_manifest::BLOCKS,
        events: crate::directive_manifest::EVENT_NAMES
            .iter()
            .map(|e| e.name)
            .collect(),
        modifiers: crate::directive_manifest::EVENT_MODIFIERS,
        bind_targets: crate::directive_manifest::BIND_TARGETS,
        foreign_idioms: crate::directive_manifest::FOREIGN_IDIOMS,
        motion_value_forms: crate::directive_manifest::MOTION_VALUE_FORMS,
    })
    .expect("grammar vocabulary is serializable")
}
