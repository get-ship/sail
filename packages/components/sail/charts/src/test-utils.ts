// Test helpers for ported primitives.
//
// Each component test repeats the same dance: create a throwaway
// container, append it to body, write `<tag attrs>` HTML, query the
// shadow root. These helpers compress the boilerplate.
//
// Imported from `test/*.test.ts` files in this package; not part of
// the runtime API.

/** A managed test container. Tracks added containers so a single
 * `afterEach(() => cleanup())` removes all of them between tests. */
export interface TestContainerRegistry {
  /** Make a fresh container, append to body, register for cleanup. */
  mk(): HTMLElement
  /** Remove every registered container from the DOM. */
  cleanup(): void
}

/** Create a registry. Call `cleanup()` in `afterEach`. */
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
 * Mount a component instance with the given attributes. Pass through a
 * registry so the host container is cleaned up automatically.
 *
 * These components render into the LIGHT DOM: the host element IS the query
 * root, so assertions run against `el` directly.
 *
 * @example
 * ```ts
 * const reg = createContainerRegistry()
 * afterEach(() => reg.cleanup())
 *
 * it('reflects tone', () => {
 *   const { el } = mount(reg, 'sail-status-dot', { tone: 'primary', pulse: true })
 *   expect(el.getAttribute('tone')).toBe('primary')
 *   expect(el.querySelector('svg')).not.toBeNull()
 * })
 * ```
 */
export function mount(
  reg: TestContainerRegistry,
  tag: string,
  attrs: Record<string, string | boolean> = {},
  innerHTML: string = '',
): { el: HTMLElement; container: HTMLElement } {
  const container = reg.mk()
  const attrsHtml = Object.entries(attrs)
    .map(([k, v]) =>
      v === true ? k : v === false ? '' : `${k}="${escapeAttr(String(v))}"`,
    )
    .filter((s) => s.length > 0)
    .join(' ')
  container.innerHTML = `<${tag}${attrsHtml ? ' ' + attrsHtml : ''}>${innerHTML}</${tag}>`
  const el = container.querySelector(tag) as HTMLElement
  return { el, container }
}

/**
 * Every color a chart paints must be a design token — `var(--…)`, `none`, or
 * `currentColor`. Returns the `fill`/`stroke` values that are none of those
 * (a hex/rgb/oklch/color-mix literal); an empty list is a token-clean chart.
 */
export function nonTokenPaints(root: Element): string[] {
  const offenders: string[] = []
  for (const node of root.querySelectorAll('[fill], [stroke]')) {
    for (const attr of ['fill', 'stroke']) {
      const v = node.getAttribute(attr)
      if (v === null || v === 'none' || v === 'currentColor') continue
      if (!v.startsWith('var(--')) offenders.push(`${node.tagName}[${attr}=${v}]`)
    }
  }
  return offenders
}

/** Wait one microtask + one macrotask so attributeChangedCallback →
 * signal-set → effect runs settle before assertions. */
export function flush(): Promise<void> {
  return new Promise((r) => setTimeout(r, 0))
}

const escapeAttr = (s: string) =>
  s.replace(/&/g, '&amp;').replace(/"/g, '&quot;').replace(/</g, '&lt;')
