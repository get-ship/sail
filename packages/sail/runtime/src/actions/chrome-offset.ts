// chromeOffset — publish sticky/fixed chrome height as `--chrome-offset` on :root.
//
// Pairs with `h:fill` → `calc(100dvh - var(--chrome-offset, 0px))`.
//
// When the host has `autohide` (navbar prop), the bar slides away after scroll-down
// to reclaim space, and returns on scroll-up or pointer near the top edge. Hidden
// chrome publishes offset 0 so fill panes reclaim the viewport.
//
// Scroll is listened in *capture* on document so nested scrollports (landing
// <main>, playground panes) still drive hide/show — window.scrollY alone is
// often 0 on viewport-locked shells.

import type { Action } from '../action'

const TOP_REVEAL_PX = 56
const HIDE_AFTER_Y = 48

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
    return v !== 'false' && v !== '0'
  }

  const measureNatural = (): void => {
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
    // While tucked, bar does not steal clicks; a fixed top edge still reveals.
    el.style.pointerEvents = next ? 'none' : ''
    publish()
  }

  const syncMode = (): void => {
    enabled = readEnabled()
    if (!enabled) applyHidden(false)
    else publish()
  }

  const scrollYFromEvent = (e: Event): number => {
    const t = e.target
    if (t === document || t === document.documentElement || t === document.body) {
      return window.scrollY || document.documentElement.scrollTop || 0
    }
    if (t instanceof Element) {
      // Prefer the element's own scroll; fall back to window.
      if (t.scrollTop > 0 || t.scrollHeight > t.clientHeight + 1) return t.scrollTop
    }
    return window.scrollY || document.documentElement.scrollTop || 0
  }

  const onScroll = (e: Event): void => {
    if (!enabled) return
    const y = scrollYFromEvent(e)
    if (y < HIDE_AFTER_Y) {
      applyHidden(false)
    } else if (y > lastY + 4) {
      applyHidden(true)
    } else if (y < lastY - 4) {
      applyHidden(false)
    }
    lastY = y
  }

  // Hit strip at top of viewport when chrome is tucked (pointer-events on bar are none).
  let edge: HTMLElement | null = null
  const ensureEdge = (): void => {
    if (typeof document === 'undefined' || edge) return
    edge = document.createElement('div')
    edge.setAttribute('data-chrome-reveal-edge', '')
    edge.setAttribute('aria-hidden', 'true')
    edge.style.cssText =
      'position:fixed;left:0;right:0;top:0;height:14px;z-index:9998;pointer-events:auto;'
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

  const mo =
    typeof MutationObserver !== 'undefined'
      ? new MutationObserver(() => {
          syncMode()
        })
      : null
  mo?.observe(el, { attributes: true, attributeFilter: ['autohide'] })

  // Capture phase: nested scrollports (landing main, workbench panes) fire here.
  if (typeof document !== 'undefined') {
    document.addEventListener('scroll', onScroll, { capture: true, passive: true })
  }
  if (typeof window !== 'undefined') {
    window.addEventListener('resize', publish)
    window.addEventListener('mousemove', onMouseMove, { passive: true })
  }

  return {
    destroy: () => {
      ro?.disconnect()
      mo?.disconnect()
      if (typeof document !== 'undefined') {
        document.removeEventListener('scroll', onScroll, { capture: true } as EventListenerOptions)
      }
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
