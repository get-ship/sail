// Register SSR boundary markers as inert custom elements.
//
// These tags are emitted by the Rust walker as anchors for control-flow
// blocks: <sail-if>/<sail-for>/<sail-match>/<sail-when>, and
// <sail-while-loading>/<sail-while-error> async boundaries. Registering
// them ensures consistent parsing, stable styling, and prevents
// `customElements.get(tag)` consumers from treating them as unknown
// elements. Each element is inert — the attach runtime does all the work.

if (typeof customElements !== 'undefined') {
  for (const tag of [
    'sail-if',
    'sail-for',
    'sail-match',
    'sail-when',
    'sail-while-loading',
    'sail-while-error',
  ]) {
    if (!customElements.get(tag)) {
      customElements.define(tag, class extends HTMLElement {})
    }
  }
}
