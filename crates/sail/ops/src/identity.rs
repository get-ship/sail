//! TOOLING-1 WI7 — one element identity scheme for inspect, tasks, and edit.
//!
//! Shape: component id (IR `nodeId`) + semantic locator (TEST-1) + file/span.
//! Task anchors, inspect tree nodes, and identity-scoped `sail_edit` targets
//! share this type. No second reference scheme.

use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::error::OpError;

// ── Types ───────────────────────────────────────────────────────────────────

/// Source span (byte offsets + optional line/col). Shared by anchors + inspect.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub struct SourceSpan {
    pub start: u64,
    pub end: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub col: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_line: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_col: Option<u64>,
}

impl SourceSpan {
    pub fn from_model_span(v: &Value) -> Option<Self> {
        let start = v.get("start")?.as_u64()?;
        let end = v.get("end")?.as_u64()?;
        Some(Self {
            start,
            end,
            line: v.get("line").and_then(|x| x.as_u64()),
            col: v.get("col").and_then(|x| x.as_u64()),
            end_line: v
                .get("end_line")
                .or_else(|| v.get("endLine"))
                .and_then(|x| x.as_u64()),
            end_col: v
                .get("end_col")
                .or_else(|| v.get("endCol"))
                .and_then(|x| x.as_u64()),
        })
    }

    pub fn display(&self) -> String {
        match (self.line, self.end_line) {
            (Some(a), Some(b)) if a != b => format!("{a}–{b}"),
            (Some(a), _) => format!("{a}"),
            _ => format!("{}–{}", self.start, self.end),
        }
    }
}

/// TEST-1-aligned semantic locator (role / name / tag — never data-sail-host).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct SemanticLocator {
    /// Author tag (e.g. `button`, `card`).
    pub tag: String,
    /// Explicit or implied accessible role when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// Accessible/authored name: `aria-label`, `name`, `id`, `label`, …
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl SemanticLocator {
    /// Build from a tree-node JSON (`tag` + `attrs`).
    pub fn from_node(node: &Value) -> Self {
        let tag = node
            .get("tag")
            .and_then(|t| t.as_str())
            .unwrap_or("?")
            .to_string();
        let attrs = node.get("attrs").and_then(|a| a.as_object());
        let name = attrs.and_then(|a| {
            a.get("aria-label")
                .or_else(|| a.get("label"))
                .or_else(|| a.get("name"))
                .or_else(|| a.get("id"))
                .or_else(|| a.get("data-demo-node"))
                .and_then(|v| v.as_str())
                .map(str::to_string)
        });
        let role = attrs
            .and_then(|a| a.get("role").and_then(|v| v.as_str()).map(str::to_string))
            .or_else(|| implied_role(&tag));
        Self { tag, role, name }
    }

    /// Match key for unique resolve (tag + name preferred; role+name next).
    fn match_key(&self) -> String {
        let name = self.name.as_deref().unwrap_or("");
        if !name.is_empty() {
            if let Some(role) = self.role.as_deref() {
                return format!("role:{role}|name:{name}");
            }
            return format!("tag:{}|name:{name}", self.tag);
        }
        format!("tag:{}", self.tag)
    }

    fn matches_node(&self, node: &Value) -> bool {
        let other = Self::from_node(node);
        if self.tag != other.tag && !self.tag.eq_ignore_ascii_case(&other.tag) {
            return false;
        }
        match (&self.name, &other.name) {
            (Some(a), Some(b)) if !a.is_empty() => a == b,
            (Some(a), _) if !a.is_empty() => false,
            _ => {
                // No name: require unique role when both present, else tag-only
                // (caller must enforce uniqueness).
                match (&self.role, &other.role) {
                    (Some(a), Some(b)) => a == b,
                    _ => true,
                }
            }
        }
    }
}

fn implied_role(tag: &str) -> Option<String> {
    let t = tag.to_ascii_lowercase();
    let role = match t.as_str() {
        "button" => "button",
        "a" | "link" => "link",
        "nav" | "navbar" => "navigation",
        "main" => "main",
        "header" => "banner",
        "footer" => "contentinfo",
        "aside" => "complementary",
        "form" => "form",
        "img" | "image" => "img",
        "dialog" | "alert-dialog" => "dialog",
        "tab" => "tab",
        "tabs" => "tablist",
        "checkbox" | "switch" => "checkbox",
        "radio" => "radio",
        "input" | "text-field" | "textarea" => "textbox",
        "select" => "combobox",
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "heading" => "heading",
        "ul" | "ol" | "list" => "list",
        "li" | "list-item" => "listitem",
        _ => return None,
    };
    Some(role.into())
}

/// Canonical element identity (WI7). JSON field names match task anchors.
///
/// - `nodeId` — IR component id (`file#ordinalPath#contentHash`)
/// - `locator` — TEST-1 semantic locator
/// - `file` / `span` — source location
/// - `elementPath` / `contentHash` — display + drift helpers (same scheme)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ElementIdentity {
    #[serde(rename = "nodeId")]
    pub node_id: String,
    /// Human display path (`file › tag name`) — stable secondary key.
    #[serde(rename = "elementPath")]
    pub element_path: String,
    #[serde(rename = "contentHash")]
    pub content_hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span: Option<SourceSpan>,
    /// TEST-1 semantic locator (role/name/tag).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locator: Option<SemanticLocator>,
}

impl ElementIdentity {
    /// Emit identity from an inspect/model tree node JSON.
    pub fn from_tree_node(node: &Value) -> Self {
        let node_id = node
            .get("id")
            .or_else(|| node.get("nodeId"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let content_hash = content_hash_from_node_id(&node_id);
        let file = node
            .get("file")
            .and_then(|f| f.as_str())
            .map(str::to_string)
            .or_else(|| file_from_node_id(&node_id));
        let locator = SemanticLocator::from_node(node);
        let element_path = element_path_from_parts(
            file.as_deref().unwrap_or(""),
            &locator.tag,
            locator.name.as_deref().unwrap_or(""),
        );
        let span = node.get("span").and_then(SourceSpan::from_model_span);
        Self {
            node_id,
            element_path,
            content_hash,
            file,
            span,
            locator: Some(locator),
        }
    }

    pub fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or(json!({}))
    }
}

/// Task-thread anchor — same type as [`ElementIdentity`] (one scheme).
pub type Anchor = ElementIdentity;

/// Content-hash segment of `file#path#hash`.
pub fn content_hash_from_node_id(node_id: &str) -> String {
    node_id
        .rsplit('#')
        .next()
        .filter(|s| !s.is_empty())
        .unwrap_or(node_id)
        .to_string()
}

fn file_from_node_id(node_id: &str) -> Option<String> {
    let before = node_id.split('#').next()?;
    if before.ends_with(".sail") {
        Some(before.to_string())
    } else {
        None
    }
}

/// Human element path: `basename › tag [name]`.
pub fn element_path_from_node(node: &Value) -> String {
    let file = node.get("file").and_then(|f| f.as_str()).unwrap_or("");
    let tag = node.get("tag").and_then(|t| t.as_str()).unwrap_or("?");
    let attrs = node.get("attrs").and_then(|a| a.as_object());
    let label = attrs
        .and_then(|a| {
            a.get("name")
                .or_else(|| a.get("id"))
                .or_else(|| a.get("aria-label"))
                .or_else(|| a.get("label"))
                .or_else(|| a.get("data-demo-node"))
                .and_then(|v| v.as_str())
        })
        .unwrap_or("");
    element_path_from_parts(file, tag, label)
}

fn element_path_from_parts(file: &str, tag: &str, label: &str) -> String {
    let base = file.rsplit('/').next().unwrap_or(file);
    if label.is_empty() {
        format!("{base} › {tag}")
    } else {
        format!("{base} › {tag} {label}")
    }
}

/// Parse identity / anchor JSON (accepts camelCase + snake_case keys).
pub fn parse_identity(v: &Value) -> Result<ElementIdentity, OpError> {
    let node_id = v
        .get("nodeId")
        .or_else(|| v.get("node_id"))
        .and_then(|x| x.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            OpError::InvalidArgument("identity/anchor requires non-empty `nodeId`".into())
        })?
        .to_string();
    let content_hash = v
        .get("contentHash")
        .or_else(|| v.get("content_hash"))
        .and_then(|x| x.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| content_hash_from_node_id(&node_id));
    let element_path = v
        .get("elementPath")
        .or_else(|| v.get("element_path"))
        .and_then(|x| x.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| format!("? › {node_id}"));
    let file = v
        .get("file")
        .and_then(|x| x.as_str())
        .map(str::to_string)
        .or_else(|| file_from_node_id(&node_id));
    let span = v.get("span").and_then(SourceSpan::from_model_span);
    let locator = if let Some(loc) = v.get("locator") {
        Some(SemanticLocator {
            tag: loc
                .get("tag")
                .and_then(|t| t.as_str())
                .unwrap_or("?")
                .to_string(),
            role: loc
                .get("role")
                .and_then(|r| r.as_str())
                .map(str::to_string),
            name: loc
                .get("name")
                .and_then(|n| n.as_str())
                .map(str::to_string),
        })
    } else {
        // Derive a best-effort locator from elementPath when not provided.
        // Format: `basename › tag [name…]`
        derive_locator_from_element_path(&element_path)
    };
    Ok(ElementIdentity {
        node_id,
        element_path,
        content_hash,
        file,
        span,
        locator,
    })
}

fn derive_locator_from_element_path(path: &str) -> Option<SemanticLocator> {
    // `page.sail › card hero` or `page.sail › button`
    let after = path.split('›').nth(1)?.trim();
    if after.is_empty() {
        return None;
    }
    let mut parts = after.split_whitespace();
    let tag = parts.next()?.to_string();
    let name_rest: Vec<&str> = parts.collect();
    let name = if name_rest.is_empty() {
        None
    } else {
        Some(name_rest.join(" "))
    };
    Some(SemanticLocator {
        role: implied_role(&tag),
        tag,
        name,
    })
}

// ── Resolve / drift ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolveStatus {
    Ok,
    Moved,
    Lost,
    /// No identity provided.
    None,
}

impl ResolveStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Moved => "moved",
            Self::Lost => "lost",
            Self::None => "none",
        }
    }
}

/// Result of resolving an identity against live inspect nodes.
#[derive(Debug, Clone)]
pub struct ResolvedIdentity {
    pub status: ResolveStatus,
    pub span: Option<SourceSpan>,
    pub live_node_id: Option<String>,
    pub live_hash: Option<String>,
    /// Full live identity when ok/moved.
    pub live: Option<ElementIdentity>,
}

impl ResolvedIdentity {
    pub fn to_drift_json(&self) -> Value {
        json!({
            "status": self.status.as_str(),
            "span": self.span,
            "liveNodeId": self.live_node_id,
            "liveHash": self.live_hash,
            "spanDisplay": self.span.as_ref().map(|s| s.display()),
            "live": self.live,
        })
    }
}

/// Resolve identity against live model nodes.
///
/// - exact `nodeId` → `ok`
/// - unique semantic locator / elementPath / contentHash → `moved`
/// - no unique match → `lost` (never silent rebind)
pub fn resolve_identity(identity: &ElementIdentity, nodes: &[Value]) -> ResolvedIdentity {
    let by_id: HashMap<&str, &Value> = nodes
        .iter()
        .filter_map(|n| n.get("id").and_then(|i| i.as_str()).map(|id| (id, n)))
        .collect();

    if let Some(n) = by_id.get(identity.node_id.as_str()) {
        let live = ElementIdentity::from_tree_node(n);
        return ResolvedIdentity {
            status: ResolveStatus::Ok,
            span: live.span.clone(),
            live_node_id: Some(identity.node_id.clone()),
            live_hash: Some(content_hash_from_node_id(&identity.node_id)),
            live: Some(live),
        };
    }

    // Unique semantic locator match (tag+name / role+name).
    if let Some(loc) = identity.locator.as_ref() {
        if loc.name.as_ref().is_some_and(|n| !n.is_empty()) {
            let matches: Vec<&Value> = nodes.iter().filter(|n| loc.matches_node(n)).collect();
            if matches.len() == 1 {
                return moved_from_node(matches[0]);
            }
        }
    }

    // Unique elementPath match.
    let path_matches: Vec<&Value> = nodes
        .iter()
        .filter(|n| element_path_from_node(n) == identity.element_path)
        .collect();
    if path_matches.len() == 1 {
        return moved_from_node(path_matches[0]);
    }

    // Unique content-hash match within same file (when known).
    let hash_matches: Vec<&Value> = nodes
        .iter()
        .filter(|n| {
            let id = n.get("id").and_then(|i| i.as_str()).unwrap_or("");
            if content_hash_from_node_id(id) != identity.content_hash {
                return false;
            }
            if let Some(f) = identity.file.as_deref() {
                let nf = n.get("file").and_then(|x| x.as_str()).unwrap_or("");
                return nf == f || nf.ends_with(f) || f.ends_with(nf);
            }
            true
        })
        .collect();
    if hash_matches.len() == 1 {
        return moved_from_node(hash_matches[0]);
    }

    ResolvedIdentity {
        status: ResolveStatus::Lost,
        span: None,
        live_node_id: None,
        live_hash: None,
        live: None,
    }
}

fn moved_from_node(n: &Value) -> ResolvedIdentity {
    let live = ElementIdentity::from_tree_node(n);
    let id = live.node_id.clone();
    let hash = live.content_hash.clone();
    ResolvedIdentity {
        status: ResolveStatus::Moved,
        span: live.span.clone(),
        live_node_id: Some(id),
        live_hash: Some(hash),
        live: Some(live),
    }
}

/// Rewrite structured edit ops that target `from_id` to `to_id`.
pub fn rewrite_ops_node_id(ops: &mut [Value], from_id: &str, to_id: &str) {
    if from_id == to_id || from_id.is_empty() || to_id.is_empty() {
        return;
    }
    for op in ops.iter_mut() {
        let Some(obj) = op.as_object_mut() else {
            continue;
        };
        for key in ["nodeId", "node_id", "parentNodeId", "parent_node_id", "newParentId", "new_parent_id"]
        {
            if obj.get(key).and_then(|v| v.as_str()) == Some(from_id) {
                obj.insert(key.into(), json!(to_id));
            }
        }
    }
}

/// Schema example for docs / RESULT (stable MCP/JSON contract).
pub fn schema_example() -> Value {
    json!({
        "nodeId": "apps/demo/routes/page.sail#0.1#a1b2c3d4",
        "elementPath": "page.sail › button Save",
        "contentHash": "a1b2c3d4",
        "file": "apps/demo/routes/page.sail",
        "span": {
            "start": 120,
            "end": 158,
            "line": 12,
            "col": 4,
            "end_line": 14,
            "end_col": 12
        },
        "locator": {
            "tag": "button",
            "role": "button",
            "name": "Save"
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(id: &str, file: &str, tag: &str, name: &str, start: u64, end: u64) -> Value {
        let mut attrs = serde_json::Map::new();
        if !name.is_empty() {
            attrs.insert("name".into(), json!(name));
        }
        json!({
            "id": id,
            "file": file,
            "tag": tag,
            "attrs": attrs,
            "span": {
                "start": start,
                "end": end,
                "line": start,
                "col": 1,
                "end_line": end,
                "end_col": 1
            },
            "layer": "l1-element",
            "children": [],
        })
    }

    #[test]
    fn identity_from_node_includes_locator() {
        let n = node(
            "routes/page.sail#0.1#abcd1234",
            "routes/page.sail",
            "button",
            "Save",
            10,
            20,
        );
        let id = ElementIdentity::from_tree_node(&n);
        assert_eq!(id.node_id, "routes/page.sail#0.1#abcd1234");
        assert_eq!(id.content_hash, "abcd1234");
        assert_eq!(id.file.as_deref(), Some("routes/page.sail"));
        assert_eq!(id.element_path, "page.sail › button Save");
        let loc = id.locator.unwrap();
        assert_eq!(loc.tag, "button");
        assert_eq!(loc.name.as_deref(), Some("Save"));
        assert_eq!(loc.role.as_deref(), Some("button"));
    }

    #[test]
    fn resolve_survives_in_file_span_shift() {
        let file = "routes/page.sail";
        let original = node(
            "routes/page.sail#0.1#abcd1234",
            file,
            "card",
            "hero",
            42,
            58,
        );
        let identity = ElementIdentity::from_tree_node(&original);
        assert_eq!(
            resolve_identity(&identity, &[original.clone()]).status,
            ResolveStatus::Ok
        );

        // Same tag+name+hash, new ordinal path + shifted span (in-file move).
        let moved = node(
            "routes/page.sail#0.3#abcd1234",
            file,
            "card",
            "hero",
            200,
            220,
        );
        let r = resolve_identity(&identity, &[moved.clone()]);
        assert_eq!(r.status, ResolveStatus::Moved);
        assert_eq!(
            r.live_node_id.as_deref(),
            Some("routes/page.sail#0.3#abcd1234")
        );
        assert_eq!(r.span.as_ref().unwrap().start, 200);
        let live = r.live.unwrap();
        assert_eq!(live.locator.as_ref().unwrap().name.as_deref(), Some("hero"));
        assert_eq!(live.content_hash, "abcd1234");
    }

    #[test]
    fn resolve_lost_never_rebinds_other_element() {
        let identity = ElementIdentity::from_tree_node(&node(
            "routes/page.sail#0.1#abcd1234",
            "routes/page.sail",
            "card",
            "hero",
            42,
            58,
        ));
        let other = node(
            "routes/page.sail#0.9#zzzz9999",
            "routes/page.sail",
            "avatar",
            "fallback",
            1,
            2,
        );
        let r = resolve_identity(&identity, &[other]);
        assert_eq!(r.status, ResolveStatus::Lost);
        assert!(r.live_node_id.is_none());
    }

    #[test]
    fn parse_identity_accepts_legacy_anchor_without_locator() {
        let v = json!({
            "nodeId": "routes/page.sail#0.1#9f2c",
            "elementPath": "page.sail › card hero",
            "contentHash": "9f2c",
            "file": "routes/page.sail",
            "span": { "start": 42, "end": 58, "line": 42, "end_line": 58 }
        });
        let id = parse_identity(&v).unwrap();
        assert_eq!(id.node_id, "routes/page.sail#0.1#9f2c");
        let loc = id.locator.unwrap();
        assert_eq!(loc.tag, "card");
        assert_eq!(loc.name.as_deref(), Some("hero"));
    }

    #[test]
    fn rewrite_ops_updates_node_ids() {
        let mut ops = vec![
            json!({"op": "setAttr", "nodeId": "old#0#h", "name": "x", "value": "1"}),
            json!({"op": "move", "nodeId": "other", "newParentId": "old#0#h"}),
        ];
        rewrite_ops_node_id(&mut ops, "old#0#h", "new#1#h");
        assert_eq!(ops[0]["nodeId"], "new#1#h");
        assert_eq!(ops[1]["newParentId"], "new#1#h");
        assert_eq!(ops[1]["nodeId"], "other");
    }

    #[test]
    fn schema_example_is_object_with_pillars() {
        let ex = schema_example();
        assert!(ex.get("nodeId").is_some());
        assert!(ex.get("locator").is_some());
        assert!(ex.get("file").is_some());
        assert!(ex.get("span").is_some());
    }

    #[test]
    fn match_key_prefers_role_name() {
        let loc = SemanticLocator {
            tag: "button".into(),
            role: Some("button".into()),
            name: Some("Save".into()),
        };
        assert_eq!(loc.match_key(), "role:button|name:Save");
    }
}
