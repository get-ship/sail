// Register SSR boundary markers as inert custom elements.
//
// These tags are emitted by the Rust walker as anchors for control-flow
// blocks: <sui-if>/<sui-for>/<sui-match> (R5 control flow), and
// <sui-while-loading>/<sui-while-error> ({#while-loading} / {#while-error}
// async boundaries). Registering them ensures consistent parsing,
// stable styling, and prevents `customElements.get(tag)` consumers
// from treating them as unknown elements. Each element is inert —
// the attach runtime does all the work.

if (typeof customElements !== 'undefined') {
  for (const tag of [
    'sui-if',
    'sui-for',
    'sui-match',
    'sui-while-loading',
    'sui-while-error',
  ]) {
    if (!customElements.get(tag)) {
      customElements.define(tag, class extends HTMLElement {})
    }
  }
}
