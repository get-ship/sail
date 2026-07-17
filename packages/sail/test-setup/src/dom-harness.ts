// Lightweight component-mount helpers for SUI component tests.
//
// Most component specs follow the same dance: make a throwaway host container,
// append it to the document, write `<tag attrs>…</tag>` into it, and query the
// element + its shadow root. These helpers compress that boilerplate so packs
// stop each rolling their own near-identical `test-utils.ts`.
//
// This is the lightweight, registration-based path: the custom element must
// already be registered (typically via the component's `.compiled.js` import in
// the spec). For the compile-from-source path, use `renderComponent` instead.

/** A managed test container. Tracks every container it creates so a single
 *  `afterEach(() => reg.cleanup())` removes all of them between tests. */
export interface TestContainerRegistry {
  /** Make a fresh container, append it to `document.body`, register it for cleanup. */
  mk(): HTMLElement
  /** Remove every registered container from the DOM. */
  cleanup(): void
}

/** Create a container registry. Call `cleanup()` in `afterEach`. */
export function createContainerRegistry(): TestContainerRegistry {
  const containers: HTMLElement[] = []
  return {
    mk() {
      const c = document.createElement('div')
      document.body.appendChild(c)
      containers.push(c)
      return c
    },
    cleanup() {
      for (const c of containers.splice(0)) c.remove()
    },
  }
}

/**
 * Mount a custom element with the given attributes into a fresh registered
 * container. Returns the element, its shadow root (when present), and the host
 * container. The element must already be registered (import its `.compiled.js`
 * in the spec). Pass the registry so the host is cleaned up automatically.
 */
export function mount(
  reg: TestContainerRegistry,
  tag: string,
  attrs: Record<string, string | boolean> = {},
  innerHTML = '',
): { el: HTMLElement; shadow: ShadowRoot | null; container: HTMLElement } {
  const container = reg.mk()
  const attrsHtml = Object.entries(attrs)
    .map(([k, v]) => (v === true ? k : v === false ? '' : `${k}="${escapeAttr(String(v))}"`))
    .filter((s) => s.length > 0)
    .join(' ')
  container.innerHTML = `<${tag}${attrsHtml ? ' ' + attrsHtml : ''}>${innerHTML}</${tag}>`
  const el = container.querySelector(tag) as HTMLElement
  return { el, shadow: el.shadowRoot, container }
}

/** Wait one macrotask so attributeChangedCallback → signal-set → effect runs
 *  settle before assertions. */
export function flush(): Promise<void> {
  return new Promise((r) => setTimeout(r, 0))
}

const escapeAttr = (s: string): string =>
  s.replace(/&/g, '&amp;').replace(/"/g, '&quot;').replace(/</g, '&lt;')
