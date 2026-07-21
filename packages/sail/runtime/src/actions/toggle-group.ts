import type { Action } from '../action'

/**
 * `use:toggleGroup` — the L1 `<toggle-group>` element's behavior, light DOM.
 * Owns the group ARIA (role=group + aria-orientation), roving tabindex, arrow
 * nav (per orientation) + Home/End, click selection, and `value` sync. `type`
 * = `single` (one value, deselectable) or `multiple` (space-separated set).
 * Each `.sail-toggle` gets `aria-pressed`. Emits `sail-toggle-change`.
 *
 * Stamps group `variant` + `orientation` onto each cell so join radii /
 * ghost/segmented chrome live as child CVA + first:/last: (no parent→child PE).
 */
export const toggleGroup: Action<void> = (node) => {
  const root = node as HTMLElement
  const isMulti = (): boolean => root.getAttribute('type') === 'multiple'
  const valueOf = (el: Element): string => el.getAttribute('value') ?? ''
  const togglesOf = (): HTMLElement[] =>
    Array.from(root.querySelectorAll<HTMLElement>('.sail-toggle'))
  const activeSet = (): Set<string> =>
    new Set((root.getAttribute('value') ?? '').split(/\s+/).filter(Boolean))
  const orientationOf = (): 'horizontal' | 'vertical' =>
    root.getAttribute('orientation') === 'vertical' ? 'vertical' : 'horizontal'
  const variantOf = (): string => root.getAttribute('variant') || 'outline'

  if (!root.hasAttribute('role')) root.setAttribute('role', 'group')

  const stampChrome = (): void => {
    const orientation = orientationOf()
    const variant = variantOf()
    for (const t of togglesOf()) {
      t.setAttribute('orientation', orientation)
      t.setAttribute('variant', variant)
    }
  }

  const sync = (): void => {
    stampChrome()
    root.setAttribute('aria-orientation', orientationOf())
    const active = activeSet()
    const toggles = togglesOf()
    for (const t of toggles) {
      t.setAttribute('role', 'button')
      t.setAttribute('aria-pressed', active.has(valueOf(t)) ? 'true' : 'false')
    }
    const enabled = toggles.filter((t) => !t.hasAttribute('disabled'))
    const focusable = enabled.find((t) => active.has(valueOf(t))) ?? enabled[0]
    for (const t of toggles) t.setAttribute('tabindex', t === focusable ? '0' : '-1')
  }

  const commit = (v: string): void => {
    root.setAttribute('value', v)
    sync()
    root.dispatchEvent(
      new CustomEvent('sail-toggle-change', { detail: { value: v }, bubbles: true, composed: true }),
    )
  }

  const toggleFrom = (e: Event): HTMLElement | null => {
    const t = (e.target as Element | null)?.closest<HTMLElement>('.sail-toggle') ?? null
    return t && root.contains(t) ? t : null
  }

  const onClick = (e: MouseEvent): void => {
    const t = toggleFrom(e)
    if (!t || t.hasAttribute('disabled')) return
    const v = valueOf(t)
    const active = activeSet()
    if (isMulti()) {
      if (active.has(v)) active.delete(v)
      else active.add(v)
      commit([...active].join(' '))
    } else {
      commit(active.has(v) ? '' : v)
    }
    t.focus()
  }

  const onKeydown = (e: KeyboardEvent): void => {
    const t = toggleFrom(e)
    if (!t) return
    const enabled = togglesOf().filter((x) => !x.hasAttribute('disabled'))
    const i = enabled.indexOf(t)
    if (i < 0) return
    const vertical = orientationOf() === 'vertical'
    const [next, prev] = vertical ? ['ArrowDown', 'ArrowUp'] : ['ArrowRight', 'ArrowLeft']
    let j = -1
    if (e.key === next) j = (i + 1) % enabled.length
    else if (e.key === prev) j = (i - 1 + enabled.length) % enabled.length
    else if (e.key === 'Home') j = 0
    else if (e.key === 'End') j = enabled.length - 1
    if (j >= 0) {
      e.preventDefault()
      const to = enabled[j]!
      t.setAttribute('tabindex', '-1')
      to.setAttribute('tabindex', '0')
      to.focus()
    }
  }

  root.addEventListener('click', onClick)
  root.addEventListener('keydown', onKeydown)
  const mo = new MutationObserver(() => sync())
  mo.observe(root, {
    childList: true,
    subtree: true,
    attributes: true,
    attributeFilter: ['orientation', 'variant', 'value', 'type'],
  })
  sync()

  return {
    destroy: () => {
      root.removeEventListener('click', onClick)
      root.removeEventListener('keydown', onKeydown)
      mo.disconnect()
    },
  }
}
