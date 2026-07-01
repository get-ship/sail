// Code snippets for the Lifecycle page. Kept OUT of the controller so the
// compiler's textual scope scan isn't fooled by the `controller({ ... })` /
// `mount(...)` patterns in these strings. Compile-checked against the in-repo
// sui-compiler (the mount + onCleanup controller pairs with the template).
export const code = {
  mount: `import { controller, signal, onCleanup } from '@ship/sui-runtime'

export default controller({
  scope: () => ({ size: signal({ w: 0, h: 0 }) }),

  // mount() runs once, after the element is in the DOM.
  // state is the scope; host is the element itself.
  mount(state, host) {
    const ro = new ResizeObserver(([entry]) => {
      // write a signal with assignment — the proxy routes it to signal.set
      state.size = { w: entry.contentRect.width, h: entry.contentRect.height }
    })
    ro.observe(host)
    // onCleanup runs when the component unmounts
    onCleanup(() => ro.disconnect())
  },
})`,

  template: `<!-- the same size signal, read in the template -->
<p>{size().w} × {size().h}</p>`,
}
