//! Initial-state JSON island.
//!
//! Per `docs/ship-adrs/.../sui-marker-contract.md`, every SSR'd
//! page emits exactly one `<script type="application/json"
//! id="__set_init__">` element after the rendered body. The contents
//! are a JSON object keyed by the names of the page's `<data>`
//! declarations — `{kind: "loading"}` envelopes for D-2.
//!
//! The HTML parser ends a `<script>` element at `</script` regardless
//! of the script's `type`. JSON values can contain that exact byte
//! sequence (e.g. `{"x": "</script>"}`) so we escape `</` as `<\/`
//! before splicing — JSON parsers accept the `\/` escape per spec, so
//! the inline value round-trips back to the original string when the
//! attach runtime calls `JSON.parse`.

use serde_json::Value;

const OPEN: &str = "<script type=\"application/json\" id=\"__set_init__\">";
const CLOSE: &str = "</script>";

pub fn emit(init: &Value) -> String {
    let body = serde_json::to_string(init).unwrap_or_else(|_| "{}".into());
    let safe = body.replace("</", "<\\/");
    format!("{OPEN}{safe}{CLOSE}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn empty_object_renders() {
        let html = emit(&json!({}));
        assert_eq!(
            html,
            r#"<script type="application/json" id="__set_init__">{}</script>"#
        );
    }

    #[test]
    fn keys_appear_in_order() {
        let html = emit(&json!({"agents": {"kind": "loading"}, "n": 2}));
        assert!(html.contains(r#""agents":{"kind":"loading"}"#));
        assert!(html.contains(r#""n":2"#));
    }

    #[test]
    fn closing_tag_inside_value_is_escaped() {
        let html = emit(&json!({"x": "</script>"}));
        // Raw `</script` must not appear inside the JSON body — that
        // would terminate the outer <script>.
        let body_start = html.find('>').unwrap() + 1;
        let body_end = html.rfind("</script>").unwrap();
        let body = &html[body_start..body_end];
        assert!(
            !body.contains("</"),
            "body must not contain raw `</`; got {body}"
        );
        // The escape JSON parsers accept.
        assert!(body.contains(r#"<\/script>"#));
    }
}
