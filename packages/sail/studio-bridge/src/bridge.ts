export const STUDIO_HOST_ID = 'sail-studio-root'
export const STUDIO_CHANNEL = 'sail-studio' as const
const FAB = 'sail-studio-fab'
const LS = 'sail-studio.fab'

export type StudioMode = 'inspect' | 'annotate' | 'screenshot'
export type StudioBridgeMessage = {
  channel: typeof STUDIO_CHANNEL
  type: string
  open?: boolean
  mode?: StudioMode
  nodeId?: string | null
  selector?: string
  rect?: R
  kind?: string
  fabVisible?: boolean
}
type R = { left: number; top: number; width: number; height: number }
export type StudioBridgeHandle = {
  close(): void
  open(): void
  setOpen(v: boolean): void
  setMode(m: StudioMode): void
  isOpen(): boolean
  mode(): StudioMode
}

export const isDevMode = () =>
  typeof document !== 'undefined' &&
  document.querySelector('meta[name="deck-mode"]')?.getAttribute('content') === 'development'

const post = (m: StudioBridgeMessage) => {
  if (typeof window !== 'undefined') window.postMessage(m, '*')
}

const fabPref = () => {
  try {
    return localStorage.getItem(LS) !== '0'
  } catch {
    return true
  }
}

/** Host-app entry only — no FAB when Studio is the focused open surface. */
const isStudioApp = () => {
  const p = typeof location !== 'undefined' ? location.pathname : ''
  if (p === '/_studio' || p.startsWith('/_studio/')) return true
  const proj =
    document.querySelector('meta[name="sail-project"]')?.getAttribute('content') ?? ''
  return (
    proj === 'apps/studio' ||
    proj.endsWith('/studio') ||
    !!document.querySelector('[data-sail-studio-shell]')
  )
}

const dropFab = () => document.getElementById(FAB)?.remove()

function fabShow(v: boolean) {
  const b = document.getElementById(FAB)
  if (b) b.style.display = v ? '' : 'none'
}

function mountFab(toggle: () => void) {
  if (isStudioApp() || !fabPref() || document.getElementById(FAB)) return
  const b = document.createElement('button')
  b.id = FAB
  b.type = 'button'
  b.setAttribute('aria-label', 'S')
  b.textContent = 'S'
  b.style.cssText =
    'all:initial;position:fixed;bottom:16px;right:16px;z-index:2147483001;width:48px;height:48px;border-radius:99px;background:#7c5cff;color:#fff;font:700 11px/48px system-ui;text-align:center;cursor:pointer'
  b.onclick = (e) => {
    e.preventDefault()
    toggle()
  }
  document.documentElement.appendChild(b)
}

export function setFabVisible(v: boolean) {
  try {
    localStorage.setItem(LS, v ? '1' : '0')
  } catch {
    /* */
  }
  if (isStudioApp()) dropFab()
  else fabShow(v)
  post({ channel: STUDIO_CHANNEL, type: 'settings', fabVisible: v })
}

function host() {
  let h = document.getElementById(STUDIO_HOST_ID)
  if (h) return h
  h = document.createElement('div')
  h.id = STUDIO_HOST_ID
  h.style.cssText =
    'all:initial;position:fixed;inset:0;z-index:2147483000;pointer-events:none;contain:strict;display:none'
  h.attachShadow({ mode: 'open' }).innerHTML =
    '<style>:host{all:initial}#f{position:fixed;inset:0;background:#111;color:#eee;font:12px system-ui;overflow:auto;contain:strict;pointer-events:auto}#b{display:none;position:fixed;top:8px;left:50%;transform:translateX(-50%);background:#7c5cff;color:#fff;padding:4px 8px;border-radius:8px}#b[data-on]{display:block}#box{display:none;position:fixed;border:1px solid #7c5cff;background:#7c5cff22}:host([data-x]){cursor:crosshair}</style><div id=b></div><div id=box></div><div id=f></div>'
  document.documentElement.appendChild(h)
  return h
}

export function attachStudioBridge(): StudioBridgeHandle {
  if (!isDevMode() || typeof document === 'undefined') {
    return {
      close() {},
      open() {},
      setOpen() {},
      setMode() {},
      isOpen: () => false,
      mode: () => 'inspect',
    }
  }
  const h = host()
  const sh = h.shadowRoot!
  const frame = sh.getElementById('f')!
  const banner = sh.getElementById('b')!
  const boxEl = sh.getElementById('box')!
  let open = false
  let mode: StudioMode = 'inspect'
  let drag: { x: number; y: number } | null = null
  const toggle = () => setOpen(!open)

  const fill = async () => {
    if (frame.dataset.ok) return
    try {
      const r = await fetch('/_studio')
      if (r.ok) {
        const doc = new DOMParser().parseFromString(await r.text(), 'text/html')
        const root = doc.querySelector('#root')
        if (root?.childNodes.length) {
          frame.innerHTML = root.innerHTML
          frame.dataset.ok = '1'
          return
        }
      }
    } catch {
      /* iframe */
    }
    frame.innerHTML = ''
    const iframe = document.createElement('iframe')
    iframe.title = 'Sail Studio'
    iframe.src = '/_studio'
    iframe.style.cssText =
      'all:initial;position:absolute;inset:0;width:100%;height:100%;border:0;background:#111;pointer-events:auto'
    frame.appendChild(iframe)
    frame.dataset.ok = '1'
  }

  const setOpen = (v: boolean) => {
    open = v
    h.style.display = v ? 'block' : 'none'
    h.style.pointerEvents = v ? 'auto' : 'none'
    if (v) {
      void fill()
      dropFab()
    } else {
      if (mode !== 'inspect') setMode('inspect')
      mountFab(toggle)
    }
  }

  const setMode = (m: StudioMode) => {
    mode = m
    if (m === 'inspect') {
      banner.removeAttribute('data-on')
      banner.textContent = ''
      h.removeAttribute('data-x')
      return
    }
    banner.setAttribute('data-on', '')
    banner.textContent = m === 'annotate' ? 'A' : 'X'
    h.setAttribute('data-x', '')
    if (open) {
      open = false
      h.style.display = 'none'
      h.style.pointerEvents = 'none'
      mountFab(toggle)
    }
  }

  mountFab(toggle)

  const onMsg = (e: MessageEvent) => {
    const d = e.data
    if (!d || d.channel !== STUDIO_CHANNEL) return
    // Studio chrome "Back to app" inside the iframe / shadow frame.
    if (d.type === 'close' || (d.type === 'settings' && d.open === false)) {
      setOpen(false)
      return
    }
    if (d.type !== 'settings') return
    if (typeof d.fabVisible !== 'boolean') return
    try {
      localStorage.setItem(LS, d.fabVisible ? '1' : '0')
    } catch {
      /* */
    }
    if (open || isStudioApp()) dropFab()
    else if (!d.fabVisible) fabShow(false)
    else {
      mountFab(toggle)
      fabShow(true)
    }
  }
  window.addEventListener('message', onMsg)

  const onKey = (e: KeyboardEvent) => {
    if (e.key === 'Escape') {
      if (mode !== 'inspect') {
        e.preventDefault()
        setMode('inspect')
      } else if (open) {
        e.preventDefault()
        setOpen(false)
      }
      return
    }
    if (!(e.metaKey || e.ctrlKey) || !e.shiftKey) return
    const k = e.key.toLowerCase()
    if (k === 's') {
      e.preventDefault()
      setOpen(!open)
    } else if (k === 'a') {
      e.preventDefault()
      setMode(mode === 'annotate' ? 'inspect' : 'annotate')
    } else if (k === 'x') {
      e.preventDefault()
      setMode(mode === 'screenshot' ? 'inspect' : 'screenshot')
    }
  }

  const wipe = () => {
    boxEl.style.display = 'none'
    drag = null
  }

  const onDown = (e: PointerEvent) => {
    if (mode === 'inspect') return
    if ((e.target as Element).closest?.(`#${STUDIO_HOST_ID},#${FAB}`)) return
    e.preventDefault()
    drag = { x: e.clientX, y: e.clientY }
    h.style.display = 'block'
    h.style.pointerEvents = 'none'
    boxEl.style.display = 'block'
    boxEl.style.left = `${e.clientX}px`
    boxEl.style.top = `${e.clientY}px`
    boxEl.style.width = boxEl.style.height = '0'
  }

  const onMove = (e: PointerEvent) => {
    if (!drag) return
    boxEl.style.left = `${Math.min(drag.x, e.clientX)}px`
    boxEl.style.top = `${Math.min(drag.y, e.clientY)}px`
    boxEl.style.width = `${Math.abs(e.clientX - drag.x)}px`
    boxEl.style.height = `${Math.abs(e.clientY - drag.y)}px`
  }

  const onUp = (e: PointerEvent) => {
    if (!drag) return
    const rect: R = {
      left: Math.min(drag.x, e.clientX),
      top: Math.min(drag.y, e.clientY),
      width: Math.abs(e.clientX - drag.x),
      height: Math.abs(e.clientY - drag.y),
    }
    wipe()
    h.style.display = open ? 'block' : 'none'
    if (rect.width < 4 && rect.height < 4) {
      const el = document.elementFromPoint(e.clientX, e.clientY)
      if (el && !el.closest(`#${STUDIO_HOST_ID}`) && el.id !== FAB) {
        const selector = el.id ? `#${CSS.escape(el.id)}` : el.tagName.toLowerCase()
        post({ channel: STUDIO_CHANNEL, type: 'select', nodeId: selector, selector })
      }
      return
    }
    post(
      mode === 'annotate'
        ? { channel: STUDIO_CHANNEL, type: 'annotate', rect }
        : { channel: STUDIO_CHANNEL, type: 'screenshot', kind: 'region', rect },
    )
    setMode('inspect')
  }

  document.addEventListener('keydown', onKey, true)
  document.addEventListener('pointerdown', onDown, true)
  document.addEventListener('pointermove', onMove, true)
  document.addEventListener('pointerup', onUp, true)
  return {
    close() {
      setOpen(false)
      wipe()
      window.removeEventListener('message', onMsg)
      document.removeEventListener('keydown', onKey, true)
      document.removeEventListener('pointerdown', onDown, true)
      document.removeEventListener('pointermove', onMove, true)
      document.removeEventListener('pointerup', onUp, true)
      dropFab()
      document.getElementById(STUDIO_HOST_ID)?.remove()
    },
    open: () => setOpen(true),
    setOpen,
    setMode,
    isOpen: () => open,
    mode: () => mode,
  }
}
