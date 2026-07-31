// Deck live-reload client — the one HMR consumer for every SAIL host.
//
// This is Deck control-plane code: it owns the transport (an SSE
// subscription to `/_deck/dev/hmr`), the `/_deck/` URL scheme, the
// change-target classification, and the compile-error overlay. The SAIL
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
//     a `<sail-*>` custom element cannot be redefined in a live document.

const HMR_ENDPOINT = '/_deck/dev/hmr'
const OVERLAY_ID = '__sail_live_reload_overlay'

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
  // Sail Studio bridge (FAB + hotkeys + selection ping) under sail dev.
  // Dynamic import keeps the HMR path independent if the asset is absent.
  let studioClose = () => {}
  void import('/_deck/sail-studio-bridge.js')
    .then((m) => {
      if (typeof m.attachStudioBridge === 'function') {
        const handle = m.attachStudioBridge()
        studioClose = () => handle?.close?.()
      }
    })
    .catch(() => {
      /* optional asset — non-fatal */
    })

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
      studioClose()
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
 *
 * Also refreshes content-hashed `<link rel="stylesheet">` hrefs from the new
 * index so theme/token CSS regenerated by `sail-compiler tokens` (live path:
 * save DTCG → generate → HMR) paints without a full document reload.
 */
async function softReloadSpa() {
  try {
    const res = await fetch(location.pathname, {
      cache: 'no-store',
      headers: { accept: 'text/html' },
    })
    const html = await res.text()
    swapStylesheetLinks(html)
    const m = html.match(/<script[^>]+type="module"[^>]+src="([^"]+)"/i)
    if (!m) throw new Error('entry module script not found in index')
    await import(/* @vite-ignore */ new URL(m[1], location.origin).href)
  } catch (err) {
    console.warn('[deck/hmr] soft reload failed — falling back to full reload:', err)
    window.location.reload()
  }
}

/**
 * Rolldown/esbuild content hashes use a base64url-ish alphabet — letters,
 * digits, `_`, and `-` (including trailing `-`). Real emit example:
 *   sail-base-CB9jtEF-.css
 * Hex-only / `\w`-only patterns miss those and break stem matching → duplicate
 * links on HMR. Keep charset aligned with `assets/[name]-[hash][extname]`.
 */
const DECK_HASHED_CSS_RE = /^(.+)-[A-Za-z0-9_-]{4,}\.css$/

/**
 * Content-hash stem for deck CSS assets: `tokens-a1b2c3.css` → `tokens`,
 * `/sail-site/sail-base-CB9jtEF-.css` → `sail-base`. Falls back to basename.
 */
function cssAssetStem(href) {
  try {
    const path = new URL(href, location.origin).pathname
    const base = path.split('/').pop() || path
    // Greedy stem + final `-<hash>.css`; backtracking yields multi-hyphen stems
    // (sail-base, element-skins) with hash including `-` / `_`.
    const m = base.match(DECK_HASHED_CSS_RE)
    if (m) return m[1]
    return base.replace(/\.css$/i, '') || base
  } catch {
    return String(href)
  }
}

/**
 * True when href looks like a deck-emitted app CSS asset:
 * same-origin `/<namespace>/<stem>-<hash>.css` (see deck-bundler css_hrefs).
 * External sheets (Google Fonts, CDNs, extensions) must never be managed here.
 */
function isDeckCssHref(href) {
  try {
    const u = new URL(href, location.origin)
    if (u.origin !== location.origin) return false
    if (u.search) return false // fonts often use ?family=…
    const parts = u.pathname.split('/').filter(Boolean)
    // /<app>/<file>.css only (asset namespace is one segment)
    if (parts.length !== 2) return false
    return DECK_HASHED_CSS_RE.test(parts[1])
  } catch {
    return false
  }
}

function isDeckManagedStylesheet(link, href) {
  // Markers from index inject / prior HMR pass — always manage.
  if (link.dataset) {
    if (link.dataset.deckCss === '1') return true
    // First-paint sail-base / contracts sheet is tagged in the shell.
    if (link.dataset.sailContracts != null || link.dataset.deckCss != null) return true
  }
  if (
    link.hasAttribute &&
    (link.hasAttribute('data-sail-contracts') || link.hasAttribute('data-deck-css'))
  ) {
    return true
  }
  return isDeckCssHref(href)
}

/**
 * Align document stylesheet links with the rebuilt index.html (hashed asset
 * names change when tokens.css / themes.css / app CSS content changes).
 * Match by stable stem (tokens-*, styles-*, …). Only touch deck-emitted /
 * previously managed links — never disable Google Fonts or other third-party CSS.
 * Retired deck stems are removed (not left as disabled empty-href nodes).
 */
function swapStylesheetLinks(html) {
  if (typeof document === 'undefined' || typeof DOMParser === 'undefined') return
  let doc
  try {
    doc = new DOMParser().parseFromString(html, 'text/html')
  } catch {
    return
  }
  const nextHrefs = [...doc.querySelectorAll('link[rel="stylesheet"]')]
    .map((l) => l.getAttribute('href'))
    .filter(Boolean)
  if (!nextHrefs.length) return

  const nextByStem = new Map()
  for (const href of nextHrefs) {
    nextByStem.set(cssAssetStem(href), href)
  }
  const seenStems = new Set()
  const cur = [...document.querySelectorAll('link[rel="stylesheet"]')]
  for (const link of cur) {
    const hrefAttr = link.getAttribute('href') || ''
    const href = hrefAttr || link.href
    // Leave non-deck sheets alone (fonts, analytics, runtime injects, extensions).
    if (!isDeckManagedStylesheet(link, href)) continue
    const stem = cssAssetStem(href)
    const nextHref = nextByStem.get(stem)
    if (nextHref) {
      const abs = new URL(nextHref, location.origin).href
      if (link.href !== abs) link.href = abs
      link.dataset.deckCss = '1'
      seenStems.add(stem)
    } else {
      // Deck asset no longer in rebuilt index — remove (do not disable foreign CSS).
      link.remove()
    }
  }
  for (const [stem, href] of nextByStem) {
    if (seenStems.has(stem)) continue
    const link = document.createElement('link')
    link.rel = 'stylesheet'
    link.href = new URL(href, location.origin).href
    link.dataset.deckCss = '1'
    document.head.appendChild(link)
  }
}

/**
 * Pull the diagnostic OUT of the bundler's wrapper.
 *
 * A failing route table produced an overlay that was thousands of characters of
 * generated module text — the bundler embeds the whole importing source in its
 * error — with the one line that matters lost inside it. Filtering that down is
 * not enough; the wrapper itself is noise. Extract the fields a human acts on and
 * throw the rest away.
 */
function summarize(msg) {
  const raw = String(msg).replace(/\\n/g, '\n')
  const grab = (key) => {
    const m = raw.match(new RegExp(key + '\\s*:\\s*"((?:[^"\\\\]|\\\\.)*)"'))
    return m ? m[1].replace(/\\"/g, '"').replace(/\\n/g, '\n').trim() : ''
  }

  const file = grab('resolved') || grab('stable_importer') || grab('id')
  const detail = grab('error_message') || grab('reason')

  // Not a bundler wrapper — it's already the compiler talking. Show it.
  if (!file && !detail) return raw.length > 4000 ? raw.slice(0, 4000) + '\n…' : raw

  const parts = []
  if (file) parts.push(file)
  if (detail) parts.push('', detail)
  const out = parts.join('\n')
  return out.length > 4000 ? out.slice(0, 4000) + '\n…' : out
}

function showOverlay(e) {
  if (typeof document === 'undefined') return
  clearOverlay()
  const loc = e.file ? `${e.file}${e.line ? `:${e.line}${e.col ? `:${e.col}` : ''}` : ''}` : ''

  const el = document.createElement('div')
  el.id = OVERLAY_ID
  el.setAttribute('role', 'alert')
  // FIXED COLORS, not theme tokens. This overlay renders on top of an app whose
  // stylesheet may have failed to build — and it inherits from that app: the
  // container set `color: var(--destructive-foreground)` (white) and the <pre>
  // picked up the app's own light `pre` background, giving white-on-white. A dev
  // overlay must be legible when everything else is broken, which is exactly when
  // it appears.
  el.style.cssText = [
    'position:fixed', 'inset:0', 'z-index:2147483647',
    'background:#1a0d0f', 'color:#fde8e8',
    'font:13px/1.6 ui-monospace,SFMono-Regular,Menlo,monospace',
    'padding:2rem', 'overflow:auto',
  ].join(';')

  // Heading — names the failure as a compile error and points at the target,
  // so it can't be mistaken for a runtime crash.
  const heading = document.createElement('div')
  heading.style.cssText = 'font-weight:700;font-size:15px;margin-bottom:.25rem;color:#ff6b6b'
  heading.textContent = `Compile error — ${e.target || 'unknown target'}`
  el.appendChild(heading)

  if (loc) {
    const where = document.createElement('div')
    where.style.cssText = 'margin-bottom:1rem;color:#ffb3b3'
    where.textContent = loc
    el.appendChild(where)
  }

  // Diagnostic body — the compiler's rendered message. `textContent` (not
  // innerHTML) keeps compiler output from being interpreted as markup.
  //
  // The bundler wraps our diagnostic in its own, and its wrapper embeds the FULL
  // GENERATED SOURCE of the importing module (`importer_source`, `source`). For a
  // route table that is thousands of characters of noise burying the one line
  // that matters. Cut the payloads and keep the error.
  const body = document.createElement('pre')
  body.style.cssText =
    'margin:0;white-space:pre-wrap;word-break:break-word;' +
    'background:#2a1416;color:#fde8e8;border:1px solid #5a2a2e;' +
    'border-radius:6px;padding:1rem'
  body.textContent = summarize(e.message || '(no message)')
  el.appendChild(body)

  // Recovery hint — this overlay is dev-only and self-clears, so make that
  // explicit rather than leaving the user wondering how to dismiss it.
  const hint = document.createElement('div')
  hint.style.cssText = 'margin-top:1.5rem;font-size:12px;color:#c98f8f'
  hint.textContent = 'Fix the file and save — this overlay clears on the next successful compile.'
  el.appendChild(hint)

  document.body.appendChild(el)
}
