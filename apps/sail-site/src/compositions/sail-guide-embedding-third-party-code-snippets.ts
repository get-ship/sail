// Snippets for /sail/guide/embedding-third-party-code. The config files are
// VERBATIM copies of the real ones — packages/components/sail/*/sail.config.json.
// Never invent one: a reader pastes these.
export const code = {
  // packages/components/sail/primitives/sail.config.json
  confined: `{"sx": true, "variants": true, "raw_containers": true, "stylesheets": []}`,

  // packages/components/sail/code-editor/sail.config.json
  editorConfig: `{"sx": true, "variants": true, "raw_containers": true, "stylesheets": [], "unconfined": true}`,

  // packages/components/sail/canvas/sail.config.json
  canvasConfig: `{"sx": true, "raw_containers": true, "stylesheets": [], "network": ["self"]}`,

  // The code in the editor's controller that needs `unconfined` — a dynamic
  // import, which confinement bans outright.
  // packages/components/sail/code-editor/src/sail-code-editor/sail-code-editor.sail:329
  editorNeed: `const [diag, comp] = await Promise.all([
  import('@ship/sail-lsp-browser/diagnostics'),
  import('@ship/sail-lsp-browser/completions'),
])
lspDiagnose = diag.diagnose`,

  // The code in the canvas pack that needs `network`.
  // packages/components/sail/canvas/src/sail-annotation/sail-annotation-layer.sail:130
  canvasNeed: `const DEFAULT_ENDPOINT = '/api/feedback'
// …
const ep = endpointProp() || DEFAULT_ENDPOINT
const resp = await fetch(ep, {
  method: 'POST',
  credentials: 'include',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify(ev),
})`,

  // crates/sail/lint/src/config.rs:36 — the complete key list.
  keys: `pub const RECOGNIZED_KEYS: [&str; 8] =
    ["preset", "sx", "variants", "raw_containers", "stylesheets", "network", "unconfined", "theme"];`,

  // crates/sail/lint/src/rules/controller_no_ambient_authority.rs — the set.
  banned: `const ALWAYS_BANNED_CALLS: &[&str] = &["eval"];
const NETWORK_GATED_CALLS: &[&str] = &["fetch"];
const NETWORK_GATED_CTORS: &[&str] = &["WebSocket", "EventSource"];`,

  namedOrigin: `{"sx": true, "network": ["https://api.example.com", "self"]}`,
}
