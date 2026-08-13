//! Theme is the style namespace (SPEC-SPINNAKER-3 §10).
//!
//! Style resolution **is** theme resolution. File-local style consts are an
//! open-mode sketch pad, not a module system. A name in a style position
//! resolves in this order (first hit wins):
//!
//! 1. Vocabulary row (concern / axis / plain setter) — not a style ref
//! 2. Theme named style (`card-chrome`, `navLink`, `muted`)
//! 3. File-local `style name =` — Open policy only. Themed → studio
//! 4. Unknown vocabulary
//!
//! Does not parse v3 flat chains or lower named styles to CSS.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use serde_json::Value;

use crate::concerns::{find_concern, is_marker, resolve_concern_name};
use crate::ir::StyleDiagnostic;
use crate::setters::find_setter;
use crate::tokens::CompositeToken;

/// Generated registry artifact (DTCG emit). Same file [`crate::tokens`] reads.
const TOKEN_REGISTRY_JSON: &str =
    include_str!("../../../../packages/sail/tokens/src/token-registry.json");

/// `sail.config.jsonc` styling policy. Default is Open.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StylingPolicy {
    #[default]
    Open,
    Themed,
}

/// Vocabulary row tag. Stub until the v3 table lands (table absent → classify
/// from today's concern / setter / marker registries).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VocabTag {
    Concern,
    Axis,
    Plain,
}

/// Theme-registered named style (`$type: "sail.style"`).
///
/// Holds the `$value` map plus optional `when` / `animate` maps as structured
/// JSON. Does not lower to CSS.
#[derive(Debug, Clone, PartialEq)]
pub struct NamedStyle {
    pub name: String,
    pub r#type: String,
    pub value: BTreeMap<String, Value>,
    pub when: BTreeMap<String, Value>,
    pub animate: BTreeMap<String, Value>,
}

/// Theme animation preset (`$type: "sail.keyframes"`) or play prop-set.
#[derive(Debug, Clone, PartialEq)]
pub struct PlayPreset {
    pub name: String,
    pub r#type: String,
    pub value: BTreeMap<String, Value>,
}

/// The only resolver Style parse / complete / emit should call for theme data.
#[derive(Debug, Clone)]
pub struct ThemeStyleEnv {
    styles: BTreeMap<String, NamedStyle>,
    plays: BTreeMap<String, PlayPreset>,
}

impl ThemeStyleEnv {
    /// Load named styles and play presets from the generated token registry.
    ///
    /// `styles` / `keyframes` may be absent (Slice A not landed) — they parse
    /// as empty maps via default. Composites stay in [`crate::tokens`].
    pub fn from_registry() -> Self {
        static CACHE: OnceLock<ThemeStyleEnv> = OnceLock::new();
        CACHE
            .get_or_init(|| Self::from_json(TOKEN_REGISTRY_JSON))
            .clone()
    }

    fn from_json(json: &str) -> Self {
        let root: Value = serde_json::from_str(json)
            .unwrap_or_else(|e| panic!("token-registry.json parse failed: {e}"));
        Self::from_value(&root)
    }

    fn from_value(root: &Value) -> Self {
        let styles = object_entries(root, &["styles", "style"])
            .into_iter()
            .map(|(name, raw)| (name.clone(), named_style_from_value(&name, &raw)))
            .collect();
        let mut plays = BTreeMap::new();
        for key in ["keyframes", "plays", "play"] {
            for (name, raw) in object_entries(root, &[key]) {
                plays
                    .entry(name.clone())
                    .or_insert_with(|| play_preset_from_value(&name, &raw));
            }
        }
        Self { styles, plays }
    }

    /// Classify a name against the live vocabulary tables. `None` until a
    /// later slice owns a dedicated v3 row; today's concern / setter / marker
    /// registries stand in.
    pub fn resolve_vocab(&self, name: &str) -> Option<VocabTag> {
        resolve_vocab_tag(name)
    }

    /// Named composite for a concern (`border` + `subtle` → `border.subtle`).
    /// Delegates to [`resolve_concern_name`] / [`crate::tokens::find_composite`].
    pub fn resolve_composite(&self, concern: &str, name: &str) -> Option<&'static CompositeToken> {
        let spec = find_concern(concern)?;
        resolve_concern_name(spec, name)
    }

    /// Theme-table named style (`card-chrome`, `navLink`, `muted`).
    pub fn resolve_named_style(&self, name: &str) -> Option<NamedStyle> {
        self.styles.get(name).cloned()
    }

    /// Theme animation preset or play prop-set (`fade`, `colors`).
    pub fn resolve_play(&self, name: &str) -> Option<PlayPreset> {
        self.plays.get(name).cloned()
    }

    /// Resolve a name in a style-ref position.
    ///
    /// First hit wins: vocabulary (not a style ref) → theme named style →
    /// file-local (Open only; Themed points at studio) → unknown vocabulary.
    pub fn resolve_style_ref(
        &self,
        name: &str,
        locals: &BTreeMap<String, NamedStyle>,
        policy: StylingPolicy,
    ) -> Result<NamedStyle, StyleDiagnostic> {
        if is_vocabulary(name) {
            return Err(StyleDiagnostic::NotAStyleRef {
                name: name.to_string(),
                source: name.to_string(),
            });
        }
        if let Some(style) = self.resolve_named_style(name) {
            return Ok(style);
        }
        if let Some(local) = locals.get(name) {
            return match policy {
                StylingPolicy::Open => Ok(local.clone()),
                StylingPolicy::Themed => Err(StyleDiagnostic::ThemedLocalStyle {
                    name: name.to_string(),
                    source: name.to_string(),
                }),
            };
        }
        Err(StyleDiagnostic::UnknownVocabulary {
            name: name.to_string(),
            source: name.to_string(),
        })
    }
}

fn is_vocabulary(name: &str) -> bool {
    find_concern(name).is_some() || is_marker(name) || find_setter(name).is_some()
}

fn resolve_vocab_tag(name: &str) -> Option<VocabTag> {
    if find_concern(name).is_some() {
        return Some(VocabTag::Concern);
    }
    if is_axis_name(name) {
        return Some(VocabTag::Axis);
    }
    if is_marker(name) || find_setter(name).is_some() {
        return Some(VocabTag::Plain);
    }
    None
}

/// Axis-collapsed families (SPEC-SPINNAKER-3 §1) that are not already a concern.
fn is_axis_name(name: &str) -> bool {
    matches!(name, "pad" | "margin" | "gap")
}

fn object_entries(root: &Value, keys: &[&str]) -> BTreeMap<String, Value> {
    for key in keys {
        if let Some(Value::Object(map)) = root.get(*key) {
            return map.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        }
    }
    BTreeMap::new()
}

fn named_style_from_value(name: &str, raw: &Value) -> NamedStyle {
    let obj = raw.as_object();
    let r#type = field_str(obj, &["$type", "type"])
        .unwrap_or("sail.style")
        .to_string();
    let value = field_object(obj, &["$value", "value", "members"]);
    let when = field_object(obj, &["when"]);
    let animate = field_object(obj, &["animate"]);
    NamedStyle {
        name: name.to_string(),
        r#type,
        value,
        when,
        animate,
    }
}

fn play_preset_from_value(name: &str, raw: &Value) -> PlayPreset {
    let obj = raw.as_object();
    let r#type = field_str(obj, &["$type", "type"])
        .unwrap_or("sail.keyframes")
        .to_string();
    let value = field_object(obj, &["$value", "value", "members"]);
    PlayPreset {
        name: name.to_string(),
        r#type,
        value,
    }
}

fn field_str<'a>(
    obj: Option<&'a serde_json::Map<String, Value>>,
    keys: &[&str],
) -> Option<&'a str> {
    let obj = obj?;
    for key in keys {
        if let Some(s) = obj.get(*key).and_then(Value::as_str) {
            return Some(s);
        }
    }
    None
}

fn field_object(
    obj: Option<&serde_json::Map<String, Value>>,
    keys: &[&str],
) -> BTreeMap<String, Value> {
    let Some(obj) = obj else {
        return BTreeMap::new();
    };
    for key in keys {
        if let Some(Value::Object(map)) = obj.get(*key) {
            return map.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        }
    }
    BTreeMap::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn local_foo() -> NamedStyle {
        NamedStyle {
            name: "foo".into(),
            r#type: "sail.style".into(),
            value: BTreeMap::from([("bg".into(), Value::String("muted".into()))]),
            when: BTreeMap::new(),
            animate: BTreeMap::new(),
        }
    }

    fn locals_foo() -> BTreeMap<String, NamedStyle> {
        BTreeMap::from([("foo".into(), local_foo())])
    }

    #[test]
    fn border_subtle_resolves_as_composite() {
        let env = ThemeStyleEnv::from_registry();
        let c = env
            .resolve_composite("border", "subtle")
            .expect("border.subtle");
        assert_eq!(c.name, "border.subtle");
        assert_eq!(c.r#type, "border");
        assert!(
            c.members.iter().any(|(s, _)| *s == "color"),
            "{:?}",
            c.members
        );
    }

    #[test]
    fn named_style_card_chrome_if_present() {
        let env = ThemeStyleEnv::from_registry();
        if let Some(style) = env.resolve_named_style("card-chrome") {
            assert_eq!(style.name, "card-chrome");
            assert!(
                style.r#type == "sail.style" || style.r#type.contains("style"),
                "type={}",
                style.r#type
            );
            assert!(
                !style.value.is_empty() || !style.when.is_empty(),
                "card-chrome should carry $value or when"
            );
        }
    }

    #[test]
    fn open_local_foo_resolves_to_local() {
        let env = ThemeStyleEnv::from_registry();
        let got = env
            .resolve_style_ref("foo", &locals_foo(), StylingPolicy::Open)
            .expect("open local foo");
        assert_eq!(got, local_foo());
    }

    #[test]
    fn themed_local_only_foo_errors_studio() {
        let env = ThemeStyleEnv::from_registry();
        let err = env
            .resolve_style_ref("foo", &locals_foo(), StylingPolicy::Themed)
            .expect_err("themed must not return the local");
        assert!(
            matches!(err, StyleDiagnostic::ThemedLocalStyle { ref name, .. } if name == "foo"),
            "{err:?}"
        );
        let msg = err.message();
        assert!(
            msg.contains("studio") && (msg.contains("theme") || msg.contains("themed")),
            "{msg}"
        );
    }

    #[test]
    fn unknown_name_is_unknown_vocabulary() {
        let env = ThemeStyleEnv::from_registry();
        let empty = BTreeMap::new();
        for policy in [StylingPolicy::Open, StylingPolicy::Themed] {
            let err = env
                .resolve_style_ref("notARealStyleZzz", &empty, policy)
                .expect_err("unknown name");
            assert!(
                matches!(
                    err,
                    StyleDiagnostic::UnknownVocabulary { ref name, .. } if name == "notARealStyleZzz"
                ),
                "{err:?}"
            );
        }
    }

    #[test]
    fn vocabulary_name_is_not_a_style_ref() {
        let env = ThemeStyleEnv::from_registry();
        let empty = BTreeMap::new();
        for vocab in ["border", "pad", "srOnly", "bg", "type"] {
            let err = env
                .resolve_style_ref(vocab, &empty, StylingPolicy::Open)
                .expect_err(vocab);
            assert!(
                matches!(err, StyleDiagnostic::NotAStyleRef { ref name, .. } if name == vocab),
                "{vocab}: {err:?}"
            );
        }
    }

    #[test]
    fn parses_styles_and_keyframes_when_present() {
        let json = r#"{
            "names": [],
            "colors": [],
            "styles": {
                "card-chrome": {
                    "$type": "sail.style",
                    "$value": { "bg": "card", "border": "subtle", "radius": "lg" }
                },
                "navLink": {
                    "type": "sail.style",
                    "value": { "type": "label" },
                    "when": { "hover": { "bg": "surface-2" } }
                }
            },
            "keyframes": {
                "fade": {
                    "$type": "sail.keyframes",
                    "$value": { "from": { "opacity": 0 }, "to": { "opacity": 1 } }
                }
            },
            "plays": {
                "colors": {
                    "type": "sail.play",
                    "value": { "props": ["color", "background-color"] }
                }
            }
        }"#;
        let env = ThemeStyleEnv::from_json(json);
        let chrome = env.resolve_named_style("card-chrome").expect("card-chrome");
        assert_eq!(chrome.value.get("bg"), Some(&Value::String("card".into())));
        assert_eq!(
            chrome.value.get("border"),
            Some(&Value::String("subtle".into()))
        );
        let nav = env.resolve_named_style("navLink").expect("navLink");
        assert!(nav.when.contains_key("hover"), "{:?}", nav.when);
        let fade = env.resolve_play("fade").expect("fade");
        assert_eq!(fade.r#type, "sail.keyframes");
        assert!(fade.value.contains_key("from"));
        let colors = env.resolve_play("colors").expect("colors prop-set");
        assert_eq!(colors.r#type, "sail.play");
    }

    #[test]
    fn missing_styles_key_is_empty_not_a_parse_error() {
        let env = ThemeStyleEnv::from_json(r#"{"names":[],"colors":[],"composites":{}}"#);
        assert!(env.resolve_named_style("card-chrome").is_none());
        assert!(env.resolve_play("fade").is_none());
    }

    #[test]
    fn theme_named_style_wins_over_local() {
        let json = r#"{
            "styles": {
                "foo": { "$type": "sail.style", "$value": { "bg": "card" } }
            }
        }"#;
        let env = ThemeStyleEnv::from_json(json);
        let got = env
            .resolve_style_ref("foo", &locals_foo(), StylingPolicy::Open)
            .expect("theme wins");
        assert_eq!(got.value.get("bg"), Some(&Value::String("card".into())));
    }
}
