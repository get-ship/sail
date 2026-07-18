// chromeOffset — publish a sticky chrome element's measured height as
// `--chrome-offset` on :root. `use:chromeOffset`
//
// The stdlib `h:fill` scale is `calc(100dvh - var(--chrome-offset, 0px))` — a
// fixed-viewport pane (the docs tutorial's editor) minus the chrome the app
// publishes above it. The offset is the top bar's RENDERED height: a
// measurement, not a token, so it is owned by an action — the sanctioned home
// for measurement-driven geometry. This is the missing publisher for the
// `--chrome-offset` name the `h:fill` scale already reads.
//
// The action measures the node it is attached to (the sticky bar) and writes
// the number to :root, where every `h:fill` pane in the document — a sibling
// of the bar, not a descendant — can read it. It keeps the value fresh across
// bar reflows (ResizeObserver) and viewport resizes.

import type { Action } from '../action'

/**
 * `use:chromeOffset` — publish the attached element's measured height as
 * `--chrome-offset` on `:root`, kept fresh as the element reflows and on
 * window resize. No parameter. Pairs with the `h:fill` size scale
 * (`calc(100dvh - var(--chrome-offset, 0px))`).
 */
export const chromeOffset: Action<void> = (node) => {
  const root =
    typeof document !== 'undefined' ? document.documentElement : null
  const publish = (): void => {
    if (!root) return
    const h = Math.round(node.getBoundingClientRect().height)
    if (h > 0) root.style.setProperty('--chrome-offset', `${h}px`)
  }

  let ro: ResizeObserver | undefined
  if (typeof requestAnimationFrame !== 'undefined') {
    requestAnimationFrame(() => {
      publish()
      if (typeof ResizeObserver !== 'undefined') {
        ro = new ResizeObserver(publish)
        ro.observe(node)
      }
    })
  } else {
    publish()
  }

  const onResize = (): void => publish()
  if (typeof window !== 'undefined') {
    window.addEventListener('resize', onResize)
  }

  return {
    destroy: () => {
      ro?.disconnect()
      if (typeof window !== 'undefined') {
        window.removeEventListener('resize', onResize)
      }
    },
  }
}
