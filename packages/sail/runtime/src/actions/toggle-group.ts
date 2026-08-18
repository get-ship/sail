import type { Action } from '../action'
import { invokeCallback } from '@ship/sail-runtime/internal'

const host = (name: string) => `[data-sail-host="${name}"]`

/**
 * `use:toggleGroup` — the L1 `<toggle-group>` element's behavior, light DOM.
 * Owns the group ARIA (role=group + aria-orientation), roving tabindex, arrow
 * nav (per orientation) + Home/End, click selection, and `value` sync. `type`
 * = `single` (one value, deselectable) or `multiple` (space-separated set).
 * Each toggle host gets `aria-pressed`. Emits `sail-toggle-change`.
 *
 * Orientation is structural (join/axis); stamped onto cells. Size/variant
 * Style is authored on each toggle.
 */
export const toggleGroup: Action<void> = (node) => {
  const root = node as HTMLElement
  const isMulti = (): boolean => root.getAttribute('type') === 'multiple'
  const valueOf = (el: Element): string => el.getAttribute('value') ?? ''
  const togglesOf = (): HTMLElement[] =>
    Array.from(root.querySelectorAll<HTMLElement>(host('toggle')))
  const activeSet = (): Set<string> =>
    new Set((root.getAttribute('value') ?? '').split(/\s+/).filter(Boolean))
  const orientationOf = (): 'horizontal' | 'vertical' =>
    root.getAttribute('orientation') === 'vertical' ? 'vertical' : 'horizontal'

  if (!root.hasAttribute('role')) root.setAttribute('role', 'group')

  // Skip no-op writes — MutationObserver watches the same attrs we stamp.
  const setAttr = (el: Element, name: string, value: string): void => {
    if (el.getAttribute(name) !== value) el.setAttribute(name, value)
  }

  // Structural layout only — not paint chrome (size/variant/tone).
  const stampOrientation = (): void => {
    const orientation = orientationOf()
    for (const t of togglesOf()) setAttr(t, 'orientation', orientation)
  }

  // MutationObserver watches value/type/orientation under subtree while sync
  // stamps orientation onto cells. Disconnect during apply + no-op setAttr so
  // a host Style/skin rebuild cannot re-enter sync forever (components catalog freeze
  // when demos use real <toggle> cells).
  const sync = (): void => {
    mo.disconnect()
    try {
      stampOrientation()
      setAttr(root, 'aria-orientation', orientationOf())
      const active = activeSet()
      const toggles = togglesOf()
      for (const t of toggles) {
        setAttr(t, 'role', 'button')
        setAttr(t, 'aria-pressed', active.has(valueOf(t)) ? 'true' : 'false')
      }
      const enabled = toggles.filter((t) => !t.hasAttribute('disabled'))
      const focusable = enabled.find((t) => active.has(valueOf(t))) ?? enabled[0]
      for (const t of toggles) setAttr(t, 'tabindex', t === focusable ? '0' : '-1')
    } finally {
      mo.observe(root, {
        childList: true,
        subtree: true,
        attributes: true,
        // Group-level only. Cell `orientation` is stamped by us — watching it
        // re-entered sync forever when demos switched from bare buttons to
        // real <toggle> hosts.
        attributeFilter: ['value', 'type', 'orientation'],
      })
    }
  }

  const commit = (v: string): void => {
    setAttr(root, 'value', v)
    sync()
    invokeCallback(root, 'onToggleChange', { value: v })
  }

  const toggleFrom = (e: Event): HTMLElement | null => {
    const t = (e.target as Element | null)?.closest<HTMLElement>(host('toggle')) ?? null
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
      setAttr(t, 'tabindex', '-1')
      setAttr(to, 'tabindex', '0')
      to.focus()
    }
  }

  root.addEventListener('click', onClick)
  root.addEventListener('keydown', onKeydown)
  const mo = new MutationObserver(() => sync())
  // Initial observe happens inside sync()'s finally (after first apply).
  sync()

  return {
    destroy: () => {
      root.removeEventListener('click', onClick)
      root.removeEventListener('keydown', onKeydown)
      mo.disconnect()
    },
  }
}
