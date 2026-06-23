// @ship/sui-chat-elements — unbound multi-modal chat-shape primitives.
//
// Portable (`@ship/sui-*`, tags `sui-*`): runs anywhere — no `ship` Proxy,
// no `setShipClient`, no SDK reads. Ship-coupled product packs are `@ship/*`
// (tags `ship-*`). Consumers (the `@ship/agents` ship-* wrappers, edit-mode
// side chats, the activity stream) bind these to their own data.
//
// Each <tag>.compiled.js calls customElements.define() on import, so a
// single `import '@ship/sui-chat-elements'` registers every tag globally.
import './sui-message/sui-message.compiled.js'
import './sui-message-list/sui-message-list.compiled.js'
import './sui-composer/sui-composer.compiled.js'
import './sui-thinking-dots/sui-thinking-dots.compiled.js'
import './sui-tool-call/sui-tool-call.compiled.js'
import './sui-citation/sui-citation.compiled.js'
