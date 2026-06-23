//! SSE envelope encoders for the SUI live-data wire protocol.
//!
//! These pure formatting functions are the single source of truth for every
//! SSE frame the runtime consumes. Both `shipd` and `deck-conformance` (and any
//! future deck host) import from here — no copies.
//!
//! ## Wire formats
//!
//! ```text
//! event: patch-signals
//! data: {"name":"<name>","value":<LiveState JSON>}
//!
//! event: patch-elements
//! data: {"selector":"<css>","mode":"<mode>","html":"<html>","settle":<ms>?}
//!
//! event: patch
//! data: {"op":"set","value":<JSON>,"_expr":"<expr>"}
//!
//! : ship-keepalive
//! ```
//!
//! `patch-signals` updates a named LiveSignal in the runtime scope.
//! `patch-elements` is an out-of-band surgical DOM patch.
//! `patch` (op: set) is the legacy `/_deck/data` single-expression channel.
//! The keepalive comment prevents idle proxies from closing SSE connections.

use serde::{Deserialize, Serialize};

// ── patch-signals ─────────────────────────────────────────────────────────────

/// Format one `patch-signals` SSE frame for a named data source.
///
/// `name` is the `<data name="...">` declaration's name; `value` is the
/// `LiveState<T>` JSON value to apply (e.g. `{"kind":"ok","value":{...}}`).
pub fn format_patch_signals(name: &str, value: &serde_json::Value) -> String {
    let payload = serde_json::json!({
        "name": name,
        "value": value,
    });
    format!(
        "event: patch-signals\ndata: {}\n\n",
        serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string())
    )
}

// ── patch (op: set) ───────────────────────────────────────────────────────────

/// Format one `patch` SSE frame for the `/_deck/data` single-expression channel.
///
/// `value` is the resolved expression value; `expr` is included as `_expr` for
/// client-side diagnostics. The `op: "set"` payload replaces the full value on
/// every push (no diffing).
pub fn format_patch_set(value: &serde_json::Value, expr: &str) -> String {
    let payload = serde_json::json!({
        "op": "set",
        "value": value,
        "_expr": expr,
    });
    format!(
        "event: patch\ndata: {}\n\n",
        serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string())
    )
}

// ── patch-elements ────────────────────────────────────────────────────────────

/// Modes accepted by a `patch-elements` event. Mirrors Datastar's PatchElements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PatchElementsMode {
    /// Replace the matched element entirely (`outerHTML`).
    Replace,
    /// Append HTML as last children of the matched element.
    Append,
    /// Prepend HTML as first children.
    Prepend,
    /// Insert HTML as previous sibling.
    Before,
    /// Insert HTML as next sibling.
    After,
    /// Diff & apply minimum mutations (preserves focus/scroll/form state).
    Morph,
}

impl PatchElementsMode {
    /// Lowercase wire string used in the SSE frame payload.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Replace => "replace",
            Self::Append => "append",
            Self::Prepend => "prepend",
            Self::Before => "before",
            Self::After => "after",
            Self::Morph => "morph",
        }
    }
}

/// Validated payload for a `patch-elements` event. Constructed by the
/// agent-push endpoint after body validation; consumed by [`format_patch_elements`].
#[derive(Debug, Clone, Deserialize)]
pub struct PatchElements {
    pub selector: String,
    pub mode: PatchElementsMode,
    pub html: String,
    /// Optional settle duration in milliseconds for transition animation.
    /// `None` means apply immediately with no settle class.
    #[serde(default)]
    pub settle: Option<u32>,
}

/// Format one `patch-elements` SSE frame.
///
/// The selector is passed through verbatim — view-root scoping is the
/// runtime's job (it queries against `[data-sui-root][data-view]`, not
/// the document). The daemon does not rewrite or sanitize selectors.
///
/// HTML is also passed through verbatim. Per the spec's "trust the server"
/// boundary, the runtime does not sanitize either; the SSE channel auth
/// is the trust boundary.
pub fn format_patch_elements(patch: &PatchElements) -> String {
    let mut payload = serde_json::Map::with_capacity(4);
    payload.insert("selector".into(), serde_json::Value::String(patch.selector.clone()));
    payload.insert(
        "mode".into(),
        serde_json::Value::String(patch.mode.as_str().to_string()),
    );
    payload.insert("html".into(), serde_json::Value::String(patch.html.clone()));
    if let Some(ms) = patch.settle {
        payload.insert("settle".into(), serde_json::Value::Number(ms.into()));
    }
    format!(
        "event: patch-elements\ndata: {}\n\n",
        serde_json::to_string(&serde_json::Value::Object(payload))
            .unwrap_or_else(|_| "{}".to_string())
    )
}

// ── keepalive ─────────────────────────────────────────────────────────────────

/// Format a keepalive comment frame. Sent periodically to prevent proxies
/// from closing idle SSE connections.
pub fn format_keepalive() -> &'static str {
    ": ship-keepalive\n\n"
}

// ── multiplex (live-data subscription fan-out) ───────────────────────────────
//
// Sibling of the `format_patch_*` family above. The deck-axum multiplex
// streaming handler calls these to emit per-subscription data frames + the
// top-level heartbeat on a single SSE stream. The wire is deck-owned (not a
// Ship proto envelope) so deck-core does not depend on ship-ops-rpc.

/// Format one `mux-data` SSE frame for a single subscription's current value.
///
/// The frame is tagged by `sub_id` so the client demuxes ONE stream into N
/// logical reads keyed on the subscription id the server minted in
/// `Subscribe`. `version` is a monotonically-increasing per-stream counter
/// (incremented on every re-resolve sweep) — clients use it to ignore stale
/// frames that arrive after a newer one for the same subscription.
pub fn format_mux_data(sub_id: &str, value: &serde_json::Value, version: u32) -> String {
    let payload = serde_json::json!({
        "sub": sub_id,
        "value": value,
        "version": version,
    });
    format!(
        "event: mux-data\ndata: {}\n\n",
        serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string())
    )
}

/// Format one `mux-heartbeat` SSE frame — the top-level liveness tick on the
/// multiplex stream. Empty `sub` field; clients ignore it for data but use
/// it (and the keepalive below) to detect a dead stream.
pub fn format_mux_heartbeat() -> &'static str {
    "event: mux-heartbeat\ndata: {}\n\n"
}

/// Format one `mux-close` SSE frame — the stream is ending. `reason` is a
/// short stable code (e.g. `"window_gone"`, `"policy_denied"`,
/// `"internal_error"`); `message` is human-readable detail. Clients use
/// this to surface a typed error rather than guess from a half-open stream.
pub fn format_mux_close(reason: &str, message: &str) -> String {
    let payload = serde_json::json!({
        "reason": reason,
        "message": message,
    });
    format!(
        "event: mux-close\ndata: {}\n\n",
        serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string())
    )
}
// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ── patch-signals exact-byte pins ─────────────────────────────────────────

    #[test]
    fn patch_signals_exact_bytes_ok_value() {
        let value = json!({"kind": "ok", "value": {"branch": "main"}});
        let frame = format_patch_signals("repo", &value);
        assert_eq!(
            frame,
            "event: patch-signals\ndata: {\"name\":\"repo\",\"value\":{\"kind\":\"ok\",\"value\":{\"branch\":\"main\"}}}\n\n"
        );
    }

    #[test]
    fn patch_signals_exact_bytes_loading() {
        let value = json!({"kind": "loading"});
        let frame = format_patch_signals("x", &value);
        assert_eq!(
            frame,
            "event: patch-signals\ndata: {\"name\":\"x\",\"value\":{\"kind\":\"loading\"}}\n\n"
        );
    }

    #[test]
    fn patch_signals_exact_bytes_null() {
        let frame = format_patch_signals("x", &json!(null));
        assert_eq!(
            frame,
            "event: patch-signals\ndata: {\"name\":\"x\",\"value\":null}\n\n"
        );
    }

    #[test]
    fn patch_signals_frame_starts_with_event_line() {
        let frame = format_patch_signals("overview", &json!({"kind": "loading"}));
        assert!(frame.starts_with("event: patch-signals\n"), "got: {frame}");
    }

    #[test]
    fn patch_signals_frame_ends_with_double_newline() {
        let frame = format_patch_signals("x", &json!(null));
        assert!(frame.ends_with("\n\n"), "SSE frames must end with \\n\\n; got: {frame:?}");
    }

    #[test]
    fn patch_signals_encodes_name_and_value() {
        let value = json!({"kind": "ok", "value": {"branch": "main"}});
        let frame = format_patch_signals("overview", &value);
        assert!(frame.contains(r#""name":"overview""#), "got: {frame}");
        assert!(frame.contains(r#""branch":"main""#), "got: {frame}");
    }

    #[test]
    fn patch_signals_loading_envelope_round_trips() {
        let value = json!({"kind": "loading"});
        let frame = format_patch_signals("agents", &value);
        let data_line = frame.lines().find(|l| l.starts_with("data: ")).expect("data line");
        let parsed: serde_json::Value =
            serde_json::from_str(&data_line["data: ".len()..]).expect("valid JSON");
        assert_eq!(parsed["name"], "agents");
        assert_eq!(parsed["value"]["kind"], "loading");
    }

    #[test]
    fn patch_signals_ok_envelope_round_trips() {
        let value = json!({"kind": "ok", "value": {"commits": 3}});
        let frame = format_patch_signals("log", &value);
        let data_line = frame.lines().find(|l| l.starts_with("data: ")).expect("data line");
        let parsed: serde_json::Value =
            serde_json::from_str(&data_line["data: ".len()..]).expect("valid JSON");
        assert_eq!(parsed["value"]["kind"], "ok");
        assert_eq!(parsed["value"]["value"]["commits"], 3);
    }

    // ── patch (op: set) exact-byte pins ──────────────────────────────────────

    #[test]
    fn patch_set_exact_bytes() {
        let frame = format_patch_set(&json!({"count": 5}), "ship.foo.bar()");
        assert_eq!(
            frame,
            "event: patch\ndata: {\"op\":\"set\",\"value\":{\"count\":5},\"_expr\":\"ship.foo.bar()\"}\n\n"
        );
    }

    #[test]
    fn patch_set_frame_starts_with_event_line() {
        let frame = format_patch_set(&json!(null), "ship.x()");
        assert!(frame.starts_with("event: patch\n"), "got: {frame}");
    }

    #[test]
    fn patch_set_frame_ends_with_double_newline() {
        let frame = format_patch_set(&json!(42), "ship.x()");
        assert!(frame.ends_with("\n\n"), "SSE frames must end with \\n\\n; got: {frame:?}");
    }

    #[test]
    fn patch_set_null_value_round_trips() {
        let frame = format_patch_set(&serde_json::Value::Null, "ship.actors.list()");
        let data_line = frame.lines().find(|l| l.starts_with("data: ")).expect("data line");
        let parsed: serde_json::Value =
            serde_json::from_str(&data_line["data: ".len()..]).expect("valid JSON");
        assert_eq!(parsed["op"], "set");
        assert_eq!(parsed["value"], serde_json::Value::Null);
        assert_eq!(parsed["_expr"], "ship.actors.list()");
    }

    // ── patch-elements exact-byte pins ────────────────────────────────────────

    #[test]
    fn patch_elements_exact_bytes_no_settle() {
        let p = PatchElements {
            selector: "#x".into(),
            mode: PatchElementsMode::Replace,
            html: "<p>hi</p>".into(),
            settle: None,
        };
        let frame = format_patch_elements(&p);
        assert_eq!(
            frame,
            "event: patch-elements\ndata: {\"selector\":\"#x\",\"mode\":\"replace\",\"html\":\"<p>hi</p>\"}\n\n"
        );
    }

    #[test]
    fn patch_elements_exact_bytes_with_settle() {
        let p = PatchElements {
            selector: "#x".into(),
            mode: PatchElementsMode::Morph,
            html: "<div/>".into(),
            settle: Some(150),
        };
        let frame = format_patch_elements(&p);
        assert_eq!(
            frame,
            "event: patch-elements\ndata: {\"selector\":\"#x\",\"mode\":\"morph\",\"html\":\"<div/>\",\"settle\":150}\n\n"
        );
    }

    #[test]
    fn patch_elements_frame_has_correct_event_name() {
        let p = PatchElements {
            selector: "#x".into(),
            mode: PatchElementsMode::Replace,
            html: "<div/>".into(),
            settle: None,
        };
        let frame = format_patch_elements(&p);
        assert!(frame.starts_with("event: patch-elements\n"), "got: {frame}");
        assert!(frame.ends_with("\n\n"));
    }

    #[test]
    fn patch_elements_encodes_all_fields() {
        let p = PatchElements {
            selector: "#main-list .item[data-id='42']".into(),
            mode: PatchElementsMode::Morph,
            html: "<div class='item' data-id='42'>x</div>".into(),
            settle: Some(150),
        };
        let frame = format_patch_elements(&p);
        let data_line = frame.lines().find(|l| l.starts_with("data: ")).unwrap();
        let parsed: serde_json::Value =
            serde_json::from_str(&data_line["data: ".len()..]).unwrap();
        assert_eq!(parsed["selector"], "#main-list .item[data-id='42']");
        assert_eq!(parsed["mode"], "morph");
        assert_eq!(parsed["html"], "<div class='item' data-id='42'>x</div>");
        assert_eq!(parsed["settle"], 150);
    }

    #[test]
    fn patch_elements_omits_settle_when_none() {
        let p = PatchElements {
            selector: "#x".into(),
            mode: PatchElementsMode::Append,
            html: "<i/>".into(),
            settle: None,
        };
        let frame = format_patch_elements(&p);
        let data_line = frame.lines().find(|l| l.starts_with("data: ")).unwrap();
        let parsed: serde_json::Value =
            serde_json::from_str(&data_line["data: ".len()..]).unwrap();
        assert!(parsed.get("settle").is_none(), "settle should be absent: {parsed}");
    }

    #[test]
    fn patch_elements_modes_serialize_lowercase() {
        for (mode, expected) in [
            (PatchElementsMode::Replace, "replace"),
            (PatchElementsMode::Append, "append"),
            (PatchElementsMode::Prepend, "prepend"),
            (PatchElementsMode::Before, "before"),
            (PatchElementsMode::After, "after"),
            (PatchElementsMode::Morph, "morph"),
        ] {
            assert_eq!(mode.as_str(), expected);
        }
    }

    #[test]
    fn patch_elements_payload_deserializes_from_post_body() {
        let raw = r##"{"selector":"#x","mode":"morph","html":"<p>hi</p>","settle":200}"##;
        let p: PatchElements = serde_json::from_str(raw).expect("parse");
        assert_eq!(p.selector, "#x");
        assert!(matches!(p.mode, PatchElementsMode::Morph));
        assert_eq!(p.html, "<p>hi</p>");
        assert_eq!(p.settle, Some(200));
    }

    #[test]
    fn patch_elements_settle_optional_in_body() {
        let raw = r##"{"selector":"#x","mode":"replace","html":"<p/>"}"##;
        let p: PatchElements = serde_json::from_str(raw).expect("parse");
        assert_eq!(p.settle, None);
    }

    #[test]
    fn patch_elements_unknown_mode_rejected() {
        let raw = r##"{"selector":"#x","mode":"explode","html":"<p/>"}"##;
        let r: Result<PatchElements, _> = serde_json::from_str(raw);
        assert!(r.is_err(), "unknown mode should not deserialize");
    }

    // ── keepalive exact-byte pin ──────────────────────────────────────────────

    #[test]
    fn keepalive_exact_bytes() {
        assert_eq!(format_keepalive(), ": ship-keepalive\n\n");
    }

    #[test]
    fn keepalive_is_sse_comment() {
        let ka = format_keepalive();
        assert!(ka.starts_with(':'), "SSE comments start with colon; got: {ka:?}");
    }

    #[test]
    fn keepalive_ends_with_double_newline() {
        assert!(format_keepalive().ends_with("\n\n"));
    }
}
