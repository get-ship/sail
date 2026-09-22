//! Edit op surface (v1).

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Child insertion position under a parent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Position {
    /// 0-based index among element children.
    Index(usize),
    /// Named shortcuts.
    Named(String),
}

impl Position {
    pub fn resolve(&self, child_count: usize) -> Result<usize, String> {
        match self {
            Position::Index(i) => {
                if *i > child_count {
                    return Err(format!(
                        "position {i} out of range (parent has {child_count} children)"
                    ));
                }
                Ok(*i)
            }
            Position::Named(s) => match s.trim().to_ascii_lowercase().as_str() {
                "start" | "first" => Ok(0),
                "end" | "last" | "append" => Ok(child_count),
                other => Err(format!(
                    "unknown position `{other}` (use number, start, or end)"
                )),
            },
        }
    }
}

/// One structured edit op.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum EditOp {
    /// Insert a parsed fragment under a parent.
    Insert {
        #[serde(rename = "parentNodeId")]
        parent_node_id: String,
        position: Position,
        fragment: String,
    },
    /// Replace a node subtree with a fragment.
    Replace {
        #[serde(rename = "nodeId")]
        node_id: String,
        fragment: String,
    },
    /// Wrap a node in a new element.
    Wrap {
        #[serde(rename = "nodeId")]
        node_id: String,
        tag: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        attrs: Option<Value>,
    },
    /// Set a static attribute (or create it).
    #[serde(rename = "setAttr")]
    SetAttr {
        #[serde(rename = "nodeId")]
        node_id: String,
        name: String,
        #[serde(default)]
        value: Option<String>,
    },
    /// Remove an attribute.
    #[serde(rename = "removeAttr")]
    RemoveAttr {
        #[serde(rename = "nodeId")]
        node_id: String,
        name: String,
    },
    /// Set/merge a Style product setter on `sx` (closed vocab).
    #[serde(rename = "setSx")]
    SetSx {
        #[serde(rename = "nodeId")]
        node_id: String,
        setter: String,
        #[serde(default)]
        args: Vec<Value>,
    },
    /// Move a node under a new parent.
    Move {
        #[serde(rename = "nodeId")]
        node_id: String,
        #[serde(rename = "newParentId")]
        new_parent_id: String,
        position: Position,
    },
}

/// Parse a JSON array (or single object) of ops.
pub fn parse_ops(v: &Value) -> Result<Vec<EditOp>, String> {
    if v.is_array() {
        serde_json::from_value(v.clone()).map_err(|e| format!("ops parse: {e}"))
    } else if v.is_object() {
        let one: EditOp =
            serde_json::from_value(v.clone()).map_err(|e| format!("op parse: {e}"))?;
        Ok(vec![one])
    } else {
        Err("ops must be an array or single op object".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_set_sx() {
        let ops = parse_ops(&json!([{
            "op": "setSx",
            "nodeId": "f#0#abc",
            "setter": "pad",
            "args": [2]
        }]))
        .unwrap();
        assert!(matches!(ops[0], EditOp::SetSx { .. }));
    }

    #[test]
    fn position_named() {
        assert_eq!(Position::Named("end".into()).resolve(3).unwrap(), 3);
        assert_eq!(Position::Named("start".into()).resolve(3).unwrap(), 0);
    }
}
