// @ship/sail-components/chat — unbound multi-modal chat-shape primitives.
//
// Portable (`@ship/sail-*`, tags `sail-*`): runs anywhere — no `ship` Proxy,
// no `setShipClient`, no SDK reads. Ship-coupled product packs are `@ship/*`
// (tags `ship-*`). Consumers (the `@ship/agents` ship-* wrappers, edit-mode
// side chats, the activity stream) bind these to their own data.
//
// Each <tag>.compiled.js calls customElements.define() on import, so a
// single `import '@ship/sail-components/chat'` registers every tag globally.
import './sail-message/sail-message.compiled.js'
import './sail-message-list/sail-message-list.compiled.js'
import './sail-composer/sail-composer.compiled.js'
import './sail-thinking-dots/sail-thinking-dots.compiled.js'
import './sail-tool-call/sail-tool-call.compiled.js'
import './sail-citation/sail-citation.compiled.js'
