// Deck dev-mode error overlay — the fallback surface for task #80's
// incident: a page that rendered nothing, with the only symptom being
// "[sui] effect threw:" buried in console.error. Dozens of throws, zero
// visible failure surface unless a developer already had devtools open.
//
// This listens for the `sui:error` CustomEvent the sui runtime dispatches
// (packages/sui/runtime/src/error-overlay.ts) whenever an effect or
// component-setup throw reaches no `<sui-error-boundary>`, and renders a
// fixed, dismissible panel on top of the page.
//
// Deliberately dependency-free plain DOM + inline styles: it must render
// even when the framework itself is broken (no sui components, no
// contracts/sx, nothing that could itself throw). Gating: this no-ops
// unless `<meta name="deck-mode" content="development">` is present, so
// the daemon's mode stays the single source of truth — the injecting
// `<script>` tag is dev-only already (never served in prod), and this is
// a second, defense-in-depth gate mirroring live-reload.js's.

const OVERLAY_ID = '__sui_error_overlay'

/** True when the served HTML declares the daemon is in development mode. */
function isDevMode() {
  if (typeof document === 'undefined') return false
  const meta = document.querySelector('meta[name="deck-mode"]')
  return meta?.getAttribute('content') === 'development'
}

/**
 * Subscribe to `sui:error` and render the overlay on each occurrence.
 * Repeat errors (same tag + message) bump a count on the existing panel
 * instead of stacking duplicate panels.
 *
 * @returns {{ close(): void }}
 */
export function attachErrorOverlay() {
  if (typeof document === 'undefined' || !isDevMode()) {
    return { close() {} }
  }
  const counts = new Map()
  const handler = (e) => renderError(e.detail, counts)
  document.addEventListener('sui:error', handler)
  return {
    close() {
      document.removeEventListener('sui:error', handler)
      clearOverlay()
    },
  }
}

function clearOverlay() {
  if (typeof document === 'undefined') return
  document.getElementById(OVERLAY_ID)?.remove()
}

function errorKey(detail) {
  const message = detail?.error instanceof Error ? detail.error.message : String(detail?.error)
  return `${detail?.phase ?? ''}::${detail?.tag ?? ''}::${message}`
}

function renderError(detail, counts) {
  if (typeof document === 'undefined' || !document.body) return
  const key = errorKey(detail)
  const count = (counts.get(key) ?? 0) + 1
  counts.set(key, count)

  const existing = document.getElementById(OVERLAY_ID)
  if (existing) existing.remove()

  const err = detail?.error
  const message = err instanceof Error ? err.message : String(err)
  const stack = err instanceof Error ? err.stack : undefined

  // FIXED COLORS, not theme tokens — this overlay must be legible even
  // when the app's own stylesheet failed to build or apply.
  const el = document.createElement('div')
  el.id = OVERLAY_ID
  el.setAttribute('role', 'alert')
  el.style.cssText = [
    'position:fixed', 'inset:auto 1rem 1rem auto', 'max-width:32rem',
    'max-height:70vh', 'overflow:auto', 'z-index:2147483647',
    'background:#1a0d0f', 'color:#fde8e8',
    'font:13px/1.5 ui-monospace,SFMono-Regular,Menlo,monospace',
    'padding:1rem 1.25rem', 'border:1px solid #5a2a2e', 'border-radius:8px',
    'box-shadow:0 4px 24px rgba(0,0,0,.5)',
  ].join(';')

  const closeBtn = document.createElement('button')
  closeBtn.textContent = '×'
  closeBtn.setAttribute('aria-label', 'Dismiss error overlay')
  closeBtn.style.cssText = [
    'position:absolute', 'top:.5rem', 'right:.5rem', 'width:1.5rem', 'height:1.5rem',
    'background:transparent', 'color:#ffb3b3', 'border:none', 'cursor:pointer',
    'font-size:16px', 'line-height:1',
  ].join(';')
  closeBtn.addEventListener('click', () => el.remove())
  el.appendChild(closeBtn)

  const heading = document.createElement('div')
  heading.style.cssText = 'font-weight:700;font-size:14px;margin-bottom:.25rem;color:#ff6b6b;padding-right:1.5rem'
  const phaseLabel = detail?.phase === 'setup' ? 'setup' : 'effect'
  heading.textContent = detail?.tag
    ? `[sui] ${phaseLabel} threw in <${detail.tag}>`
    : `[sui] ${phaseLabel} threw`
  el.appendChild(heading)

  if (count > 1) {
    const badge = document.createElement('div')
    badge.style.cssText = 'margin-bottom:.5rem;color:#ffb3b3;font-size:12px'
    badge.textContent = `Repeated ${count} times`
    el.appendChild(badge)
  }

  const body = document.createElement('pre')
  body.style.cssText =
    'margin:0;white-space:pre-wrap;word-break:break-word;' +
    'background:#2a1416;color:#fde8e8;border:1px solid #5a2a2e;' +
    'border-radius:6px;padding:.75rem'
  body.textContent = stack || message
  el.appendChild(body)

  document.body.appendChild(el)
}
