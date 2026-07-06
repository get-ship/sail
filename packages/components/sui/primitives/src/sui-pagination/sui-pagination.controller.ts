import { controller, derived, ref, signal } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-pagination.sui'

/** A numbered page button's view model (the `{#for}` row). */
interface PageView {
  page: number
  current: boolean
  label: string
}

/** Parse the `page-size-options` string ("10,25,50,100") into a deduped,
 *  positive-integer list, order preserved. Non-numeric chunks are dropped. */
function parseSizes(raw: string): number[] {
  const out: number[] = []
  for (const chunk of raw.split(',')) {
    const n = Number(chunk.trim())
    if (Number.isInteger(n) && n > 0 && !out.includes(n)) out.push(n)
  }
  return out
}

/** Total page count — never below 1 so an empty set still reports "page 1 of 1"
 *  and the edge controls collapse to a single disabled page. */
function pageCountOf(total: number, pageSize: number): number {
  if (pageSize <= 0 || total <= 0) return 1
  return Math.max(1, Math.ceil(total / pageSize))
}

export default controller({
  scope: ({ page, total, pageSize, pageSizeOptions }: ViewScope<Scope>) => {
    const navEl = ref<HTMLElement>(null)
    const sizeSelectEl = ref<HTMLElement>(null)

    // prop:* identity seam — a live array assigned to the host `pageSizeOptions`
    // property wins over the parsed `page-size-options` string (the structured
    // input path). Null until a consumer assigns it.
    const optionsIdentity = signal<number[] | null>(null)

    const pageCount = derived<number>(() => pageCountOf(total(), pageSize()))
    const current = derived<number>(() => {
      const p = Math.trunc(page())
      const max = pageCount()
      if (p < 1) return 1
      if (p > max) return max
      return p
    })

    const atStart = derived<boolean>(() => current() <= 1)
    const atEnd = derived<boolean>(() => current() >= pageCount())

    const rangeLabel = derived<string>(() => {
      const t = total()
      if (t <= 0) return '0 of 0'
      const ps = pageSize() > 0 ? pageSize() : t
      const start = (current() - 1) * ps + 1
      const end = Math.min(current() * ps, t)
      return `${start}–${end} of ${t}`
    })

    const pageNumbers = derived<PageView[]>(() => {
      const max = pageCount()
      const cur = current()
      const out: PageView[] = []
      for (let p = 1; p <= max; p++) {
        out.push({ page: p, current: p === cur, label: `Page ${p}` })
      }
      return out
    })

    const sizeOptions = derived<number[]>(() => {
      const identity = optionsIdentity()
      if (identity && identity.length > 0) return identity
      return parseSizes(pageSizeOptions())
    })

    const pageSizeStr = derived<string>(() => String(pageSize()))

    return {
      navEl,
      sizeSelectEl,
      optionsIdentity,
      pageCount,
      current,
      atStart,
      atEnd,
      rangeLabel,
      pageNumbers,
      sizeOptions,
      pageSizeStr,
    }
  },

  mount(state, host) {
    const emitPage = (next: number): void => {
      const clamped = Math.max(1, Math.min(next, state.pageCount))
      host.emit!('sui-page-change', { page: clamped, pageSize: state.pageSize })
    }

    // prop:* identity — a live `pageSizeOptions` array assigned to the host
    // routes into the scope signal so the size <sui-select> reacts and wins over
    // the declarative `page-size-options` string (the SSR/fallback path the
    // `sizeOptions` derived parses when no identity array is set).
    Object.defineProperty(host, 'pageSizeOptions', {
      configurable: true,
      get: () => state.optionsIdentity,
      set: (v: unknown) => {
        state.optionsIdentity = Array.isArray(v)
          ? v.map((n) => Number(n)).filter((n) => Number.isInteger(n) && n > 0)
          : null
      },
    })

    // ── Delegated navigation ────────────────────────────────────────────────
    // One listener on the <nav> covers the edge controls AND the numbered page
    // buttons. A disabled <button> swallows its own click, so no edge guard is
    // needed here — but a defensive bounds check keeps the contract honest if
    // a host overrides disabled CSS.
    const onClick = (e: Event): void => {
      const target = e.target as HTMLElement
      const numbered = target.closest('[data-page]')
      if (numbered) {
        const n = Number(numbered.getAttribute('data-page'))
        if (Number.isInteger(n)) emitPage(n)
        return
      }
      const edge = target.closest('[data-page-action]')
      if (!edge) return
      switch (edge.getAttribute('data-page-action')) {
        case 'first':
          if (state.current > 1) emitPage(1)
          break
        case 'prev':
          if (state.current > 1) emitPage(state.current - 1)
          break
        case 'next':
          if (state.current < state.pageCount) emitPage(state.current + 1)
          break
        case 'last':
          if (state.current < state.pageCount) emitPage(state.pageCount)
          break
      }
    }

    // ── Page-size change ────────────────────────────────────────────────────
    // The composed <sui-select> emits `sui-change { value }`. A size change
    // resets to page 1 so the viewer never lands past the new last page.
    const onSizeChange = (e: Event): void => {
      const detail = (e as CustomEvent<{ value?: string }>).detail
      const next = Number(detail?.value)
      if (!Number.isInteger(next) || next <= 0) return
      host.emit!('sui-page-change', { page: 1, pageSize: next })
    }

    const nav = state.navEl as HTMLElement | null
    nav?.addEventListener('click', onClick)
    nav?.addEventListener('sui-change', onSizeChange as EventListener)

    return () => {
      nav?.removeEventListener('click', onClick)
      nav?.removeEventListener('sui-change', onSizeChange as EventListener)
    }
  },
})
