// hydrate-mark.ts — record hydration markers on the server-DOM.
//
// The synthetic render's SSR path runs against the shared server-DOM
// (the SAME `bindText` / `bindFor` / `bindIf` / `bindOn` / `bindWhile*`
// runtime helpers the browser path uses). For client hydration to
// re-attach reactivity on top of SSR'd HTML, the serialized output
// must carry the marker contract the client `hydrate()` reads —
// `<!--sail:text:bN-->...<!--/sail-->`, `on:event="bN"`, `<sail-if key="bN">`,
// etc. (defined in `crates/sail/compiler/src/render/walker.rs`).
//
// This module exposes tiny "record" helpers the synthetic emit calls
// alongside each binding call. On the BROWSER DOM they're a true
// no-op — the production walker / ship-js SSR already emit markers,
// and CSR doesn't need them. On the SERVER DOM each helper stamps
// the relevant marker on the node so the next serialization includes
// it. This is the SAME marker contract the production walker emits;
// we move WHO emits them, not WHAT they look like.
//
// The functions are exported from `@ship/sail-runtime` (alongside
// `bindText` et al.) so the synthetic emit can `import { markHydrateText }`
// next to its other runtime imports and the client bundles tree-shake
// them cleanly (the typeof / `in` checks JIT to true no-ops).

/**
 * Detect whether `node` is a server-DOM TextNode (carries the
 * `__hydrateKey` field the server-DOM serialization reads). Used by
 * the marker-recording helpers to be true no-ops on the browser DOM.
 */
function isServerText(node: unknown): node is Text & { __hydrateKey: string | null } {
  return (
    typeof node === 'object' &&
    node !== null &&
    '__hydrateKey' in (node as Record<string, unknown>)
  )
}

function isServerEl(node: unknown): node is Element & {
  __hydrateAttr: Record<string, string> | null
  __hydrateBoundary: string | null
} {
  return (
    typeof node === 'object' &&
    node !== null &&
    '__hydrateAttr' in (node as Record<string, unknown>)
  )
}

/**
 * Tag a text node with its hydration key. Server-DOM serialization wraps
 * the node in `<!--sail:text:bN-->...<!--/sail-->`; browser DOM ignores.
 * `key` is the `b<N>` string the synthetic emitter's binding registry
 * allocated for this dynamic text position.
 */
export function markHydrateText(node: Text, key: string): void {
  if (isServerText(node)) {
    node.__hydrateKey = key
  }
}

/**
 * Tag an element with a marker attribute (`on:event`,
 * `style:name`, `prop:name`, `attr:name`, `bind:value`, `bind:checked`,
 * `bind:this`, `attr-dynamic`, `mutation`). Server-DOM serialization sets
 * the attribute on the element; browser DOM ignores.
 *
 * `name` is the marker's attribute name (`"on:click"`, `"prop:items"`,
 * etc.); `key` is the `b<N>` string.
 */
export function markHydrateAttr(el: Element, name: string, key: string): void {
  if (isServerEl(el)) {
    if (!el.__hydrateAttr) el.__hydrateAttr = {}
    el.__hydrateAttr[name] = key
  }
}

/**
 * Tag a control-flow boundary wrapper element. The synthetic render
 * inserts `<sail-{if,for,match,await,while-loading,while-error}>` comment
 * or element wrappers around each branch's children; this stamps the
 * key onto the wrapper so the client's `collectElementMarkers` picks
 * it up via the `key="bN"` attribute (matches the walker contract).
 */
export function markHydrateBoundary(el: Element, key: string): void {
  if (isServerEl(el)) {
    el.__hydrateBoundary = key
  } else {
    // Browser DOM path — the runtime `bindFor` etc. also set `key`
    // through `setAttribute`, so the marker is preserved either way.
    el.setAttribute('key', key)
  }
}
