//! Theme is the style namespace (SPEC-SPINNAKER-3 §10).
//!
//! Style resolution **is** theme resolution. File-local style consts are an
//! open-mode sketch pad, not a module system. A name in a style position
//! resolves in this order (first hit wins):
//!
//! 1. Vocabulary row (concern / axis / plain setter) — not a style ref
//! 2. Theme-registered named style (optional; not the composition path)
//! 3. File-local `style name =` — Open policy only. Themed → studio
//! 4. Unknown vocabulary
//!
//! Does not parse v3 flat chains or lower named styles to CSS.

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::sync::OnceLock;

use serde_json::Value;

use crate::concerns::{find_concern, prepare_concern_call, resolve_concern_name};
use crate::ir::{PaintArg, PaintOp, Style, StyleDiagnostic};
use crate::tokens::CompositeToken;
use crate::vocab;

pub use crate::vocab::VocabTag;

thread_local! {
    static STYLING_POLICY: RefCell<StylingPolicy> =
        const { RefCell::new(StylingPolicy::Open) };
}

/// Current `styling` policy. Default Open until a caller (compiler, or
/// [`with_styling_policy`] in tests) installs Themed.
pub fn current_styling_policy() -> StylingPolicy {
    STYLING_POLICY.with(|cell| *cell.borrow())
}

/// Run `f` under `policy`. Nested calls restore the previous value.
pub fn with_styling_policy<T>(policy: StylingPolicy, f: impl FnOnce() -> T) -> T {
    STYLING_POLICY.with(|cell| {
        let prev = cell.replace(policy);
        let out = f();
        *cell.borrow_mut() = prev;
        out
    })
}

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

/// Tag → type-role assignment from the theme table (SPEC-SPINNAKER-3 §10).
///
/// `<h2>` spends `type_role` unless markup `style()` overrides. Not a CSS var.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagDefault {
    pub type_role: String,
    /// Optional text-scale overlay (`4xl`). Theme owns the ramp; the tag is the way.
    pub size: Option<String>,
}

/// The only resolver Style parse / complete / emit should call for theme data.
#[derive(Debug, Clone)]
pub struct ThemeStyleEnv {
    styles: BTreeMap<String, NamedStyle>,
    plays: BTreeMap<String, PlayPreset>,
    tag_defaults: BTreeMap<String, TagDefault>,
}

impl ThemeStyleEnv {
    /// Load named styles, play presets, and tag defaults from the generated
    /// token registry.
    ///
    /// `styles` / `keyframes` / `tagDefaults` may be absent — they parse as
    /// empty maps via default. Composites stay in [`crate::tokens`].
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
        let mut tag_defaults = BTreeMap::new();
        for key in ["tagDefaults", "tags"] {
            for (tag, raw) in object_entries(root, &[key]) {
                if let Some(def) = tag_default_from_value(&raw) {
                    tag_defaults.entry(tag).or_insert(def);
                }
            }
        }
        Self {
            styles,
            plays,
            tag_defaults,
        }
    }

    /// Classify a name against the one vocabulary table (concern | axis | plain).
    pub fn resolve_vocab(&self, name: &str) -> Option<VocabTag> {
        vocab::resolve_vocab(name)
    }

    /// Named composite for a concern (`border` + `subtle` → `border.subtle`).
    /// Delegates to [`resolve_concern_name`] / [`crate::tokens::find_composite`].
    pub fn resolve_composite(&self, concern: &str, name: &str) -> Option<&'static CompositeToken> {
        let spec = find_concern(concern)?;
        resolve_concern_name(spec, name)
    }

    /// Theme-table named style, if the registry has one.
    pub fn resolve_named_style(&self, name: &str) -> Option<NamedStyle> {
        self.styles.get(name).cloned()
    }

    /// Theme animation preset or play prop-set (`fade`, `colors`).
    pub fn resolve_play(&self, name: &str) -> Option<PlayPreset> {
        self.plays.get(name).cloned()
    }

    /// Type role the theme assigns to `tag` (`h2` → `heading`). `None` if unset.
    pub fn resolve_tag_default(&self, tag: &str) -> Option<TagDefault> {
        self.tag_defaults.get(tag).cloned()
    }

    /// Theme-registered named style identities (complete / diagnostics).
    pub fn named_style_names(&self) -> impl Iterator<Item = &str> {
        self.styles.keys().map(String::as_str)
    }

    /// Theme play / keyframe identities (`fade`, `colors`).
    pub fn play_names(&self) -> impl Iterator<Item = &str> {
        self.plays.keys().map(String::as_str)
    }

    /// Lower a theme named style to Style IR (concern calls + when + animate).
    pub fn named_style_to_ir(&self, style: &NamedStyle) -> Style {
        named_style_to_ir(style)
    }

    /// Expand a `Style::Ref` name: theme first, then file-local (Open only).
    ///
    /// Locals are the already-parsed Style IR (`StyleEnv`). Theme hits lower
    /// through [`Self::named_style_to_ir`]; local hits return the IR as-is.
    pub fn expand_style_ref(
        &self,
        name: &str,
        env: &HashMap<String, Style>,
        policy: StylingPolicy,
    ) -> Result<Style, StyleDiagnostic> {
        let locals = locals_from_env(env);
        let named = self.resolve_style_ref(name, &locals, policy)?;
        if self.resolve_named_style(name).is_some() {
            return Ok(self.named_style_to_ir(&named));
        }
        env.get(name)
            .cloned()
            .ok_or_else(|| StyleDiagnostic::UnknownVocabulary {
                name: name.to_string(),
                source: name.to_string(),
            })
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
    matches!(
        vocab::resolve_vocab(name),
        Some(VocabTag::Concern | VocabTag::Axis)
    )
}

/// Presence-only locals for [`ThemeStyleEnv::resolve_style_ref`]. IR lives in `env`.
fn locals_from_env(env: &HashMap<String, Style>) -> BTreeMap<String, NamedStyle> {
    env.keys()
        .map(|name| {
            (
                name.clone(),
                NamedStyle {
                    name: name.clone(),
                    r#type: "sail.style".into(),
                    value: BTreeMap::new(),
                    when: BTreeMap::new(),
                    animate: BTreeMap::new(),
                },
            )
        })
        .collect()
}

fn named_style_to_ir(style: &NamedStyle) -> Style {
    let mut parts = Vec::new();
    let mut ops = Vec::new();
    for (key, value) in &style.value {
        ops.push(json_member_to_set(key, value));
    }
    if !ops.is_empty() {
        parts.push(Style::chain(ops));
    }
    if !style.when.is_empty() {
        let entries = style
            .when
            .iter()
            .map(|(k, v)| (k.clone(), json_value_to_style(v)))
            .collect();
        parts.push(Style::When { entries });
    }
    if !style.animate.is_empty() {
        for (trigger, value) in &style.animate {
            parts.push(json_animate_to_motion(trigger, value));
        }
    }
    Style::chain(parts)
}

fn json_value_to_style(value: &Value) -> Style {
    match value {
        Value::Object(map) => {
            let ops: Vec<Style> = map.iter().map(|(k, v)| json_member_to_set(k, v)).collect();
            Style::chain(ops)
        }
        Value::String(s) => Style::Ref(s.clone()),
        _ => Style::Empty,
    }
}

fn json_member_to_set(setter: &str, value: &Value) -> Style {
    let args = json_to_paint_args(value);
    let source = format!("{setter}({})", format_paint_args(&args));
    let (setter, args) = prepare_concern_call(setter, args);
    Style::Set(PaintOp {
        setter,
        args,
        source,
    })
}

fn json_to_paint_args(value: &Value) -> Vec<PaintArg> {
    match value {
        Value::Object(map) => vec![PaintArg::Slots(
            map.iter()
                .map(|(k, v)| (k.clone(), json_to_paint_arg(v)))
                .collect(),
        )],
        other => vec![json_to_paint_arg(other)],
    }
}

fn json_to_paint_arg(value: &Value) -> PaintArg {
    match value {
        Value::String(s) => PaintArg::Token(s.clone()),
        Value::Number(n) => PaintArg::Token(n.to_string()),
        Value::Bool(b) => PaintArg::Token(b.to_string()),
        Value::Object(map) => PaintArg::Slots(
            map.iter()
                .map(|(k, v)| (k.clone(), json_to_paint_arg(v)))
                .collect(),
        ),
        Value::Null => PaintArg::Token(String::new()),
        other => PaintArg::Token(other.to_string()),
    }
}

fn format_paint_args(args: &[PaintArg]) -> String {
    args.iter()
        .map(PaintArg::display)
        .collect::<Vec<_>>()
        .join(", ")
}

fn json_animate_to_motion(trigger: &str, value: &Value) -> Style {
    let config = match value {
        Value::String(name) => format!("{{ preset: {name} }}"),
        Value::Object(map) => {
            if let Some(preset) = map
                .get("preset")
                .or_else(|| map.get("play"))
                .and_then(Value::as_str)
            {
                if let Some(duration) = map.get("duration").and_then(Value::as_str) {
                    format!("{{ preset: {preset}, duration: {duration} }}")
                } else {
                    format!("{{ preset: {preset} }}")
                }
            } else {
                value.to_string()
            }
        }
        other => other.to_string(),
    };
    Style::MotionAttach {
        trigger: trigger.to_string(),
        config,
    }
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

fn tag_default_from_value(raw: &Value) -> Option<TagDefault> {
    let type_role = match raw {
        Value::String(s) if !s.is_empty() => s.clone(),
        Value::Object(obj) => {
            if let Some(s) = field_str(Some(obj), &["type", "type_role"]) {
                s.to_string()
            } else if let Some(s) = obj.get("$value").and_then(Value::as_str) {
                s.to_string()
            } else if let Some(inner) = obj.get("$value").and_then(Value::as_object) {
                field_str(Some(inner), &["type", "type_role"])?.to_string()
            } else {
                return None;
            }
        }
        _ => return None,
    };
    if type_role.is_empty() {
        return None;
    }
    let size = match raw {
        Value::Object(obj) => obj
            .get("size")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .map(str::to_string),
        _ => None,
    };
    Some(TagDefault { type_role, size })
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
        for vocab in ["border", "pad", "bg", "type"] {
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
    fn h2_resolves_to_heading_type_role() {
        let env = ThemeStyleEnv::from_registry();
        let got = env.resolve_tag_default("h2").expect("h2 tag default");
        assert_eq!(got.type_role, "heading");
        for tag in ["h1", "h3", "h4", "h5", "h6"] {
            let got = env
                .resolve_tag_default(tag)
                .unwrap_or_else(|| panic!("{tag}"));
            assert_eq!(got.type_role, "heading", "{tag}");
        }
        assert_eq!(
            env.resolve_tag_default("p").map(|d| d.type_role),
            Some("body".into())
        );
        assert_eq!(
            env.resolve_tag_default("code").map(|d| d.type_role),
            Some("mono".into())
        );
    }

    #[test]
    fn unknown_tag_has_no_default() {
        let env = ThemeStyleEnv::from_registry();
        assert!(env.resolve_tag_default("not-a-tag-zzz").is_none());
        assert!(
            env.resolve_tag_default("a").is_none(),
            "anchor has no type default — variants own the look"
        );
        assert!(
            env.resolve_tag_default("strong").is_none(),
            "strong has no type role; weight stays on the tag"
        );
    }

    #[test]
    fn parses_tag_defaults_from_either_key() {
        let json = r#"{
            "tagDefaults": {
                "h2": { "type": "heading" },
                "p": "body"
            },
            "tags": {
                "code": { "$value": "mono" }
            }
        }"#;
        let env = ThemeStyleEnv::from_json(json);
        assert_eq!(
            env.resolve_tag_default("h2").map(|d| d.type_role),
            Some("heading".into())
        );
        assert_eq!(
            env.resolve_tag_default("p").map(|d| d.type_role),
            Some("body".into())
        );
        assert_eq!(
            env.resolve_tag_default("code").map(|d| d.type_role),
            Some("mono".into())
        );
        assert!(env.resolve_tag_default("small").is_none());
    }

    #[test]
    fn missing_tag_defaults_key_is_empty_not_a_parse_error() {
        let env = ThemeStyleEnv::from_json(r#"{"names":[],"colors":[],"composites":{}}"#);
        assert!(env.resolve_tag_default("h2").is_none());
    }

    #[test]
    fn named_style_lowers_to_ir_and_emits() {
        let json = r#"{
            "styles": {
                "card-chrome": {
                    "$type": "sail.style",
                    "$value": { "bg": "card", "border": "subtle", "radius": "lg" }
                }
            }
        }"#;
        let env = ThemeStyleEnv::from_json(json);
        let named = env.resolve_named_style("card-chrome").expect("card-chrome");
        let ir = env.named_style_to_ir(&named);
        assert!(!matches!(ir, crate::ir::Style::Empty));
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
