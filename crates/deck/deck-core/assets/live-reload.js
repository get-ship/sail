// Deck live-reload client — the one HMR consumer for every SUI host.
//
// This is Deck control-plane code: it owns the transport (an SSE
// subscription to `/_deck/dev/hmr`), the `/_deck/` URL scheme, the
// change-target classification, and the compile-error overlay. The SUI
// runtime owns only the *mechanism* (`mount().reload(payload)`); the host
// supplies how to apply a view reload via `onViewChanged`.
//
// Both surfaces load this one module: the third-party view host
// (`view-host.html`) and the first-party shell. There is deliberately no
// second implementation — HMR is one thing, done one way.
//
// Gating: this no-ops unless `<meta name="deck-mode" content="development">`
// is present, so the daemon's mode is the single source of truth. The
// route is served in every mode, but the client only subscribes in dev.
//
// Reload policy:
//   - `views/<name>/view` for the current view → host's onViewChanged
//     (in-place `handle.reload()`, preserving serializable signal state).
//   - `views/<other>/view` → ignored (next navigation fetches it fresh).
//   - anything else (component / island / runtime bundle) → full reload:
//     a `<sui-*>` custom element cannot be redefined in a live document.

const HMR_ENDPOINT = '/_deck/dev/hmr'
const OVERLAY_ID = '__sui_live_reload_overlay'

/** True when the served HTML declares the daemon is in development mode. */
export function isDevMode() {
  if (typeof document === 'undefined') return false
  const meta = document.querySelector('meta[name="deck-mode"]')
  return meta?.getAttribute('content') === 'development'
}

/**
 * Subscribe to the bundler HMR firehose and drive reloads.
 *
 * @param {object} opts
 * @param {string} [opts.viewName] - the view this page is showing; only
 *   changes to this view trigger in-place reload.
 * @param {(e: {target:string, bytes?:number}) => void|Promise<void>} [opts.onViewChanged]
 *   - apply an in-place reload of the current view. If omitted, full reload.
 * @param {(e: {target:string}) => void|Promise<void>} [opts.onOtherChanged]
 *   - react to a non-view bundle change. Defaults to full page reload.
 * @param {(e: {message:string, file?:string, line?:number, col?:number}) => void} [opts.onError]
 *   - render a compile error. Defaults to an on-page overlay.
 * @returns {{ close(): void }}
 */
export function attachLiveReload(opts = {}) {
  if (!isDevMode() || typeof EventSource === 'undefined') {
    return { close() {} }
  }
  const viewName = opts.viewName ?? ''
  const es = new EventSource(HMR_ENDPOINT)

  es.addEventListener('module-changed', (raw) => {
    let e
    try {
      e = JSON.parse(raw.data)
    } catch {
      return
    }
    clearOverlay()
    const target = e.target || ''
    // routes/ SPA soft reload: a full app rebuild (target `app`) re-imports the
    // freshly-bundled entry into the RUNNING router — state-preserving, no full
    // document reload. Gated on the SPA router being mounted (it exposes
    // `globalThis.__deckHmr`); the named-view host path below keeps its own
    // in-place / full-reload policy.
    if (globalThis.__deckHmr && target === 'app') {
      void softReloadSpa()
      return
    }
    if (target.startsWith('views/')) {
      // When the host pins a `viewName` (single-view host like view-host.html),
      // filter to it. When omitted (a navigating host like the shell), deliver
      // every view change and let the host filter against its current route.
      const changed = target.split('/')[1]
      if (viewName && changed !== viewName) return
      if (opts.onViewChanged) void opts.onViewChanged(e)
      else window.location.reload()
      return
    }
    // Component / island / runtime bundle — must full-reload (custom
    // elements can't be redefined in place).
    if (opts.onOtherChanged) void opts.onOtherChanged(e)
    else window.location.reload()
  })

  es.addEventListener('compile-error', (raw) => {
    let e
    try {
      e = JSON.parse(raw.data)
    } catch {
      return
    }
    if (opts.onError) opts.onError(e)
    else showOverlay(e)
  })

  // The browser reconnects automatically on transport errors.
  es.addEventListener('error', () => {})

  return {
    close() {
      es.close()
      clearOverlay()
    },
  }
}

function clearOverlay() {
  if (typeof document === 'undefined') return
  document.getElementById(OVERLAY_ID)?.remove()
}

/**
 * SPA soft reload (routes/ apps). Re-fetch the index, find the rebuilt entry
 * module URL (content-hashed, so it changed), and import it. The entry's
 * HMR-aware bootstrap detects the running router (`globalThis.__deckHmr`) and
 * swaps the fresh `modules` in + re-mounts the current route preserving signal
 * state — no full document reload. Any failure falls back to a full reload so a
 * broken soft reload never leaves a stale page.
 */
async function softReloadSpa() {
  try {
    const res = await fetch(location.pathname, {
      cache: 'no-store',
      headers: { accept: 'text/html' },
    })
    const html = await res.text()
    const m = html.match(/<script[^>]+type="module"[^>]+src="([^"]+)"/i)
    if (!m) throw new Error('entry module script not found in index')
    await import(/* @vite-ignore */ new URL(m[1], location.origin).href)
  } catch (err) {
    console.warn('[deck/hmr] soft reload failed — falling back to full reload:', err)
    window.location.reload()
  }
}

function showOverlay(e) {
  if (typeof document === 'undefined') return
  clearOverlay()
  const loc = e.file ? `${e.file}${e.line ? `:${e.line}${e.col ? `:${e.col}` : ''}` : ''}` : ''

  const el = document.createElement('div')
  el.id = OVERLAY_ID
  el.setAttribute('role', 'alert')
  el.style.cssText = [
    'position:fixed', 'inset:0', 'z-index:2147483647',
    'background:var(--destructive,#7f1d1d)', 'color:var(--destructive-foreground,#fff)',
    'font:13px/1.6 var(--font-mono,ui-monospace,monospace)',
    'padding:2rem', 'overflow:auto',
  ].join(';')

  // Heading — names the failure as a compile error and points at the target,
  // so it can't be mistaken for a runtime crash.
  const heading = document.createElement('div')
  heading.style.cssText = 'font-weight:700;font-size:15px;margin-bottom:.25rem'
  heading.textContent = `Compile error — ${e.target || 'unknown target'}`
  el.appendChild(heading)

  if (loc) {
    const where = document.createElement('div')
    where.style.cssText = 'opacity:.85;margin-bottom:1rem'
    where.textContent = loc
    el.appendChild(where)
  }

  // Diagnostic body — the compiler's rendered message. `textContent` (not
  // innerHTML) keeps compiler output from being interpreted as markup.
  const body = document.createElement('pre')
  body.style.cssText = 'margin:0;white-space:pre-wrap;word-break:break-word'
  body.textContent = e.message || '(no message)'
  el.appendChild(body)

  // Recovery hint — this overlay is dev-only and self-clears, so make that
  // explicit rather than leaving the user wondering how to dismiss it.
  const hint = document.createElement('div')
  hint.style.cssText = 'opacity:.7;margin-top:1.5rem;font-size:12px'
  hint.textContent = 'Fix the file and save — this overlay clears on the next successful compile.'
  el.appendChild(hint)

  document.body.appendChild(el)
}
