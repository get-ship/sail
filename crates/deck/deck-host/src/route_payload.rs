use std::collections::HashMap;

use serde::Serialize;

#[derive(Debug, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct RoutePayload {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<String>,
    pub html: String,
    pub css: String,
    pub scope: serde_json::Value,
    #[serde(rename = "moduleUrl")]
    pub module_url: String,
    /// Route params captured from `[param]` directory segments during
    /// resolution (e.g. `/agents/42` → `{ "id": "42" }`). The runtime
    /// seeds `route.params` from this so dynamic-route controllers read
    /// the segment value without depending on client-side path matching.
    pub params: HashMap<String, String>,
    /// Client keys the view declares via top-level `<client name="…" />`
    /// directives. The host page builds the keyed `clients` map it passes
    /// to `mount()` from this list — each name bound to the host's data
    /// client. Empty for views that rely on the ambient-`ship`
    /// compatibility default.
    pub clients: Vec<String>,
}
