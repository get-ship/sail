// chromeOffset — publish sticky/fixed chrome height as `--chrome-offset` on :root.
//
// Pairs with `h:fill` → `calc(100dvh - var(--chrome-offset, 0px))`.
//
// When the host has `autohide` (navbar prop), the bar slides away after scroll-down
// to reclaim space, and returns on scroll-up or pointer near the top edge. Hidden
// chrome publishes offset 0 so fill panes reclaim the viewport.

import type { Action } from '../action'

const TOP_REVEAL_PX = 56
const HIDE_AFTER_Y = 72

/**
 * `use:chromeOffset` — measure host height → `--chrome-offset`.
 * Optional host attr `autohide`: hide on scroll-down, show on scroll-up / top hover.
 */
export const chromeOffset: Action<void> = (node) => {
  const el = node as HTMLElement
  const root =
    typeof document !== 'undefined' ? document.documentElement : null

  let naturalH = 0
  let hidden = false
  let lastY = 0
  let enabled = false

  const readEnabled = (): boolean => {
    if (!el.hasAttribute('autohide')) return false
    const v = el.getAttribute('autohide')
    // Boolean prop reflection: "", "true", "autohide" → on; "false" → off
    return v !== 'false' && v !== '0'
  }

  const measureNatural = (): void => {
    // offsetHeight ignores transform; getBoundingClientRect is collapsed when hidden.
    const h = Math.round(el.offsetHeight)
    if (h > 0) naturalH = h
  }

  const publish = (): void => {
    if (!root) return
    measureNatural()
    const h = hidden && enabled ? 0 : naturalH
    root.style.setProperty('--chrome-offset', `${Math.max(0, h)}px`)
  }

  const applyHidden = (next: boolean): void => {
    if (!enabled) {
      hidden = false
      el.style.removeProperty('transform')
      el.style.removeProperty('pointer-events')
      publish()
      return
    }
    if (hidden === next) {
      publish()
      return
    }
    hidden = next
    el.style.transform = next ? 'translateY(-100%)' : 'translateY(0)'
    // Keep hit-testing at the top strip while hidden so hover can reveal.
    el.style.pointerEvents = next ? 'none' : ''
    publish()
  }

  const syncMode = (): void => {
    enabled = readEnabled()
    if (!enabled) applyHidden(false)
    else publish()
  }

  const scrollYOf = (t: EventTarget | null): number => {
    if (!t || typeof window === 'undefined') return 0
    if (t === window || t === document || t === document.documentElement) {
      return window.scrollY || document.documentElement.scrollTop || 0
    }
    if (t instanceof Element) return t.scrollTop
    return 0
  }

  const onScroll = (e: Event): void => {
    if (!enabled) return
    const y = scrollYOf(e.target === document ? window : e.target)
    if (y < HIDE_AFTER_Y) {
      applyHidden(false)
    } else if (y > lastY + 2) {
      applyHidden(true)
    } else if (y < lastY - 2) {
      applyHidden(false)
    }
    lastY = y
  }

  /** Window + any overflow scroll ancestor (landing main is the real scrollport). */
  const scrollTargets: EventTarget[] = []
  const bindScrollTargets = (): void => {
    for (const t of scrollTargets) {
      t.removeEventListener('scroll', onScroll)
    }
    scrollTargets.length = 0
    if (typeof window === 'undefined') return
    scrollTargets.push(window)
    let p: Element | null = el.parentElement
    while (p) {
      const st = getComputedStyle(p)
      const oy = st.overflowY
      if (oy === 'auto' || oy === 'scroll' || oy === 'overlay') {
        scrollTargets.push(p)
      }
      p = p.parentElement
    }
    for (const t of scrollTargets) {
      t.addEventListener('scroll', onScroll, { passive: true })
    }
  }

  // Invisible hit target along the top of the viewport when chrome is tucked away.
  let edge: HTMLElement | null = null
  const ensureEdge = (): void => {
    if (typeof document === 'undefined' || edge) return
    edge = document.createElement('div')
    edge.setAttribute('data-chrome-reveal-edge', '')
    edge.setAttribute('aria-hidden', 'true')
    edge.style.cssText =
      'position:fixed;left:0;right:0;top:0;height:12px;z-index:9998;pointer-events:auto;'
    edge.addEventListener('mouseenter', () => {
      if (enabled) applyHidden(false)
    })
    document.body.appendChild(edge)
  }

  const onMouseMove = (e: MouseEvent): void => {
    if (!enabled || !hidden) return
    if (e.clientY <= TOP_REVEAL_PX) applyHidden(false)
  }

  let ro: ResizeObserver | undefined
  if (typeof requestAnimationFrame !== 'undefined') {
    requestAnimationFrame(() => {
      syncMode()
      measureNatural()
      publish()
      ensureEdge()
      if (typeof ResizeObserver !== 'undefined') {
        ro = new ResizeObserver(() => {
          measureNatural()
          publish()
        })
        ro.observe(el)
      }
    })
  } else {
    syncMode()
    publish()
  }

  // React to autohide attr flips (SPA path modes).
  const mo =
    typeof MutationObserver !== 'undefined'
      ? new MutationObserver(() => {
          syncMode()
        })
      : null
  mo?.observe(el, { attributes: true, attributeFilter: ['autohide'] })

  if (typeof window !== 'undefined') {
    bindScrollTargets()
    // Parent scrollports may mount after the navbar (layout chain).
    requestAnimationFrame(() => bindScrollTargets())
    window.addEventListener('resize', publish)
    window.addEventListener('mousemove', onMouseMove, { passive: true })
  }

  return {
    destroy: () => {
      ro?.disconnect()
      mo?.disconnect()
      for (const t of scrollTargets) {
        t.removeEventListener('scroll', onScroll)
      }
      scrollTargets.length = 0
      if (typeof window !== 'undefined') {
        window.removeEventListener('resize', publish)
        window.removeEventListener('mousemove', onMouseMove)
      }
      edge?.remove()
      edge = null
      el.style.removeProperty('transform')
      el.style.removeProperty('pointer-events')
      root?.style.removeProperty('--chrome-offset')
    },
  }
}
