// route-boundary.ts — the DEFAULT error boundary every route outlet gets.
//
// The catch *mechanism* lives in the SUI runtime: `bindFor` / `bindIf` /
// the component factory's `mount()` / `effect()` already wrap their throw-prone
// call sites in `runProtected(node, source, fn)`, which walks up the DOM from
// `node` to the nearest element carrying a `BoundaryHandle` and routes the
// error there (see `@ship/sui-runtime`'s error-boundary.ts). The opt-in
// `<sui-error-boundary>` primitive is one consumer of that mechanism.
//
// This module is the OTHER consumer: it lets the deck router install a boundary
// handle on the route outlet so a throwing route degrades to an error card
// instead of crashing its subtree / white-screening — without the route author
// having to wrap anything. deck ⊥ sui holds: this imports the runtime's
// boundary helpers (deck → sui); the runtime never imports deck.
//
// Registration-free by design: the fallback prefers the `<sui-error-state>`
// primitive but degrades to a self-contained styled card when no primitive is
// registered — mirroring `mountRouter`'s `buildNotFound`, which never assumes
// `@ship/primitives` is present (registration is a bundler concern).

import {
  attachBoundary,
  detachBoundary,
  findNearestBoundary,
  type BoundaryHandle,
  type BoundarySource,
} from '@ship/sui-runtime'

/**
 * True when the served HTML declares the daemon is in development mode.
 * Mirrors the `isDevMode()` in deck-core's `live-reload.js`: the
 * `<meta name="ship-mode" content="development">` tag the daemon injects is
 * the single source of truth, so dev detail is shown only when the *server*
 * says it is a dev session — never inferred from the bundle.
 */
export function isDevMode(): boolean {
  if (typeof document === 'undefined') return false
  return (
    document.querySelector('meta[name="ship-mode"]')?.getAttribute('content') ===
    'development'
  )
}

export interface RouteBoundaryOptions {
  /** Re-run the current route mount — wired to the fallback's Retry action. */
  onReset: () => void
  /** Override dev detection (tests). Defaults to {@link isDevMode}. */
  dev?: boolean
}

/** A handle to a route boundary installed on an outlet; `dispose()` detaches it. */
export interface RouteBoundaryHandle {
  dispose(): void
}

/**
 * Install the DEFAULT route-level error boundary on `outlet` (the chrome's
 * `<div slot="route">`, or the root element when there is no chrome). The route
 * view still mounts into `outlet` exactly as before — this only attaches a
 * `BoundaryHandle`, so a throw in the view's render / controller `mount()` /
 * reactive effect degrades to an error card rather than a dead subtree.
 *
 * Opt-out / no-double-wrap: if a boundary is already present in `outlet`'s
 * ancestry (a layout supplied its own `<sui-error-boundary>` or attached a
 * handle), this is a no-op and returns `null` — the author's boundary wins.
 * The app-wide opt-out is `mountRouter({ errorBoundary: false })`.
 *
 * @returns the installed handle, or `null` when a boundary already covers the
 *   outlet (nothing was installed).
 */
export function installRouteBoundary(
  outlet: HTMLElement,
  opts: RouteBoundaryOptions,
): RouteBoundaryHandle | null {
  // Don't double-wrap: a chrome/layout may already establish a boundary above
  // the outlet. (Even if this check races and we install anyway, ours sits on
  // the outlet — innermost — so a throw routes here first; the outer one is
  // simply never reached. The check just avoids the redundant attach.)
  if (findNearestBoundary(outlet)) return null

  const dev = opts.dev ?? isDevMode()
  // Re-entrancy guard, mirroring <sui-error-boundary>: a single render pass can
  // throw many times (e.g. every row of a bindFor) — only the first is shown
  // until reset, so the card isn't rebuilt on each repeat throw.
  let caught = false

  const handle: BoundaryHandle = {
    catch(err: unknown, source: BoundarySource): void {
      if (caught) return
      caught = true
      const error = err instanceof Error ? err : new Error(String(err))
      // Always log the raw error so devtools + server logs see it regardless
      // of whether the on-page card is redacted.
      console.error('[deck/route] route view threw — rendered error card:', error)
      outlet.setAttribute('data-route-error', '')
      outlet.replaceChildren(
        buildErrorCard(error, source, dev, () => handle.reset()),
      )
    },
    reset(): void {
      caught = false
      outlet.removeAttribute('data-route-error')
      opts.onReset()
    },
  }

  attachBoundary(outlet, handle)
  return {
    dispose(): void {
      detachBoundary(outlet)
    },
  }
}

/**
 * Build the card shown when a route view throws. Prefers `<sui-error-state>`
 * (the SUI port) and falls back to a self-contained styled `<div>` so the card
 * never depends on a primitive being registered — the same guard pattern
 * `mountRouter`'s `buildNotFound` uses for the 404 surface.
 *
 * Dev: the message is shown (in the `code` block) with the stack in a
 * collapsible `details` slot, plus the throw source. Prod: redacted to a
 * generic line — no message, no stack, no internal detail leaks to end users.
 */
function buildErrorCard(
  error: Error,
  source: BoundarySource,
  dev: boolean,
  reset: () => void,
): HTMLElement {
  const title = 'Something went wrong'
  const body = dev
    ? `A ${source} error was caught by the route boundary.`
    : 'This page hit an unexpected error. Try again, or go back.'

  if (customElements.get('sui-error-state')) {
    const el = document.createElement('sui-error-state')
    el.setAttribute('title', title)
    el.setAttribute('body', body)
    if (dev) {
      el.setAttribute('code', error.message)
      if (error.stack) {
        const details = document.createElement('pre')
        details.setAttribute('slot', 'details')
        details.textContent = error.stack
        el.appendChild(details)
      }
    }
    el.appendChild(buildRetryButton(reset, true))
    return el
  }

  // No primitive registered — a self-contained styled card (never blank).
  return buildBareCard(title, body, dev ? error.message : '', reset)
}

/** A Retry button. `slotted` projects it into `<sui-error-state>`'s action slot. */
function buildRetryButton(reset: () => void, slotted: boolean): HTMLButtonElement {
  const btn = document.createElement('button')
  if (slotted) btn.setAttribute('slot', 'action')
  btn.type = 'button'
  btn.textContent = 'Retry'
  btn.addEventListener('click', reset)
  return btn
}

/**
 * Self-contained fallback card used when no error-state primitive is
 * registered. Inline styles reference theme tokens with hard fallbacks (the
 * same approach as the HMR overlay) so it renders correctly with or without a
 * loaded theme. `textContent` keeps the message safe from injection.
 */
function buildBareCard(
  title: string,
  body: string,
  detail: string,
  reset: () => void,
): HTMLElement {
  const box = document.createElement('div')
  box.className = 'deck-route-error'
  box.setAttribute('role', 'alert')
  box.style.cssText = [
    'display:flex',
    'flex-direction:column',
    'gap:var(--space-2,8px)',
    'align-items:flex-start',
    'padding:var(--space-5,20px)',
    'border:1px solid var(--tone-destructive-border,#7f1d1d)',
    'border-radius:var(--radius-md,8px)',
    'background:var(--tone-destructive-bg,rgba(127,29,29,0.08))',
    'color:var(--foreground,inherit)',
    'font:14px/1.5 var(--font-body,system-ui,sans-serif)',
  ].join(';')

  const h = document.createElement('div')
  h.textContent = title
  h.style.cssText = 'font-weight:600;color:var(--foreground,inherit)'

  const p = document.createElement('div')
  p.textContent = body
  p.style.cssText = 'color:var(--muted-foreground,#6b7280)'

  box.append(h, p)

  if (detail) {
    const code = document.createElement('pre')
    code.textContent = detail
    code.style.cssText = [
      'margin:0',
      'max-width:100%',
      'overflow:auto',
      'padding:var(--space-2,8px) var(--space-3,12px)',
      'border-radius:var(--radius-sm,4px)',
      'background:var(--muted,rgba(0,0,0,0.06))',
      'font:12px/1.5 var(--font-mono,ui-monospace,monospace)',
      'color:var(--muted-foreground,#6b7280)',
      'white-space:pre-wrap',
      'word-break:break-word',
    ].join(';')
    box.append(code)
  }

  box.append(buildRetryButton(reset, false))
  return box
}
