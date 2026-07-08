// register-all — the single side-effect import that registers EVERY public
// `<sui-*>` component pack the component viewer surfaces. Each pack's index
// calls `customElements.define()` on import, so importing this module once
// registers the whole catalog. The /components list AND /components/[tag]
// detail routes import this so the viewer can mount any catalogued component —
// the trust gate depends on every real component being registered, not just
// the primitives pack.
import '@ship/sui-primitives'
import '@ship/sui-charts'
import '@ship/sui-canvas'
import '@ship/sui-chat-elements'
import '@ship/sui-media'
import '@ship/sui-mermaid'
import '@ship/sui-code-editor'
import '@ship/sui-viewer'
