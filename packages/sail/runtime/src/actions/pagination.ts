import type { Action } from '../action'
import { invokeCallback } from '@ship/sail-runtime/internal'

const host = (name: string) => `[data-sail-host="${name}"]`

/**
 * Internal: L1 `<pagination>` closed behavior (not a public `use:`).
 * Authors use `<pagination>`, not `use:pagination`. Marks the page host whose
 * value matches the element's `value` with `aria-current="page"`, and on click
 * of a page sets `value` + invokes `onPageChange`. Prev/next are just pages
 * whose value is the target page. Structure (which pages/ellipsis to show) is
 * authored; this owns the state contract. Size Style match is on each `<page>`.
 */
export const pagination: Action<void> = (node) => {
  const root = node as HTMLElement
  const valueOf = (el: Element): string => el.getAttribute('value') ?? ''
  const pagesOf = (): HTMLElement[] =>
    Array.from(root.querySelectorAll<HTMLElement>(host('page')))
  const current = (): string => root.getAttribute('value') ?? ''

  if (!root.hasAttribute('aria-label')) root.setAttribute('aria-label', 'Pagination')

  const sync = (): void => {
    const cur = current()
    for (const p of pagesOf()) {
      if (valueOf(p) === cur) p.setAttribute('aria-current', 'page')
      else p.removeAttribute('aria-current')
    }
  }

  const setPage = (v: string): void => {
    if (!v || v === current()) return
    root.setAttribute('value', v)
    sync()
    invokeCallback(root, 'onPageChange', { page: v })
  }

  const pageFrom = (e: Event): HTMLElement | null => {
    const p = (e.target as Element | null)?.closest<HTMLElement>(host('page')) ?? null
    return p && root.contains(p) ? p : null
  }

  const onClick = (e: MouseEvent): void => {
    const p = pageFrom(e)
    if (p && !p.hasAttribute('disabled')) setPage(valueOf(p))
  }

  root.addEventListener('click', onClick)
  const mo = new MutationObserver(() => sync())
  mo.observe(root, { childList: true, subtree: true, attributes: true, attributeFilter: ['value'] })
  sync()

  return {
    destroy: () => {
      root.removeEventListener('click', onClick)
      mo.disconnect()
    },
  }
}
