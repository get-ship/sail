//! Compile-time Style value language: flat concern chain + when/match/compound/animate.
//!
//! Sole paint architecture: style({ pad(4).bg(surface), when: { hover: bg(muted) } }).
//!
//! Layers: domain (`ir`, `vocab`, `intent`) → application (`parse`, `complete`,
//! `clash`, `theme_env`). HTML/CSS realization is `spinnaker-css`.
//! Foreign CSS / Tailwind / HTML habits teach through [`foreign_idioms`].

pub mod capabilities;
pub mod clash;
pub mod complete;
pub mod component_surfaces;
pub mod concerns;
pub mod foreign_idioms;
pub mod host_ctx;
pub mod intent;
pub mod ir;
pub mod parse;
pub mod plains;
pub mod scales;
pub mod setters;
pub mod style_vocab;
pub mod surface;
pub mod theme_env;
pub mod token_groups;
pub mod tokens;
pub mod type_resolve;
pub mod vocab;

pub use complete::{
    OBJECT_KEYS, StyleCompleteContext, StyleCompleteItem, StyleCompleteKind,
    collect_style_const_names, style_complete_at, style_complete_context_at,
    style_complete_for_context, when_keys,
};

pub use capabilities::{
    is_animate_trigger, is_geometry_setter, is_interaction_when_key, setter_caps,
    style_contains_geometry, style_is_textish, style_is_textish_env, style_uses_text,
};
pub use component_surfaces::{
    all_surface_packs, color_paint_vocabulary, is_component_public_name,
    is_theme_owned_author_root, is_theme_owned_token_name, surface_slots,
};
pub use concerns::{
    ConcernSlot, ConcernSpec, MARKERS, concern_doc_line, concern_kind, concerns, find_concern,
    is_concern_form, is_free_size_value, is_marker, prepare_concern_call, primary_slot,
    resolve_concern_name, retired_replacement,
};
pub use foreign_idioms::{
    FOREIGN_IDIOMS, ForeignIdiom, IdiomKind, find_foreign_idiom, teaching_line,
};
pub use host_ctx::{with_bound_group_props, with_host_surface};
pub use intent::{
    Intent, IntentNode, IntentValue, intent_dump, is_known_when_key, lower_to_intent,
};
pub use ir::{PaintArg, PaintOp, Style, StyleDiagnostic, StyleEnv};
pub use parse::{
    collect_motion_attaches, collect_motion_attaches_env, looks_like_utility_string_product, parse,
    parse_style_const_rhs, validate_style_refs,
};
pub use plains::{PlainSpec, find_plain, plain_doc_line};
pub use surface::{
    LANGUAGE_PRESETS, PresetOrigin, Privilege, REQUIRED_KEYS, REQUIRED_THEME_PRESETS, StyleAtom,
    StylePreset, StyleSurface, generate_missing_warnings, is_language_size_preset,
    privilege_enforced_for_path, privilege_for_path,
};
pub use theme_env::{
    NamedStyle, PlayPreset, StylingPolicy, TagDefault, ThemeStyleEnv, VocabTag,
    current_styling_policy, with_styling_policy,
};
pub use type_resolve::{ResolvedType, resolve_type, resolve_type_for_tag, resolve_type_role};
pub use vocab::{
    VocabCaps, VocabRow, find_vocab, offered_rows, resolve_vocab, vocab_rows, vocab_surface_line,
};

#[cfg(test)]
mod tests {
    #[test]
    fn domain_modules_do_not_name_css() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        for name in [
            "parse.rs",
            "ir.rs",
            "vocab.rs",
            "intent.rs",
            "concerns.rs",
            "setters.rs",
        ] {
            let src = std::fs::read_to_string(root.join(name)).expect(name);
            for needle in ["css::", "CssRule", "background-color", "css_prop"] {
                assert!(!src.contains(needle), "{name} must not contain {needle}");
            }
        }
    }
}
