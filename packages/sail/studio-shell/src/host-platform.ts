/**
 * Framework-owned platform seams for Studio (and app shells that embed it).
 *
 * Pure product code (apps/studio/src compositions/stores) must not name
 * `window` / `document` / `navigator` — call these instead. Studio is an app;
 * these live under `packages/sail/**` (Framework env), not as Pure freebies.
 */
import { scheduleFrame, asSailHost, type SailHost } from '@ship/sail-runtime'
import { STUDIO_CHANNEL } from './channel'

/** Single rAF helper — re-export runtime (no drift vs sail-site / draft paint). */
export { scheduleFrame }

/** postMessage on the studio channel (same-window bus). */
export function postStudioMessage(body: Record<string, unknown>): void {
  if (typeof window === 'undefined') return
  try {
    window.postMessage({ channel: STUDIO_CHANNEL, ...body }, '*')
  } catch {
    /* */
  }
}

/** Ask the host overlay (parent frame) to close Studio. */
export function closeStudioOverlay(): void {
  try {
    if (typeof window !== 'undefined' && window.parent && window.parent !== window) {
      window.parent.postMessage({ channel: STUDIO_CHANNEL, type: 'close' }, '*')
    }
  } catch {
    /* cross-origin */
  }
}

/** Open a URL in a new browsing context (external editor deep links, etc.). */
export function openExternalUrl(url: string): void {
  if (!url || typeof window === 'undefined') return
  try {
    window.open(url, '_blank')
  } catch {
    /* */
  }
}

/** Subscribe to window `message` (studio-bridge traffic). */
export function subscribeWindowMessages(handler: (data: unknown) => void): () => void {
  if (typeof window === 'undefined') return () => {}
  const onMsg = (e: MessageEvent) => {
    handler(e.data)
  }
  window.addEventListener('message', onMsg)
  return () => window.removeEventListener('message', onMsg)
}

/** Subscribe to product `sail-theme-change` custom events on window. */
export function subscribeSailThemeChange(
  handler: (detail: { theme?: string; mode?: string }) => void,
): () => void {
  if (typeof window === 'undefined') return () => {}
  const on = (e: Event) => {
    const d = (e as CustomEvent).detail as { theme?: string; mode?: string } | undefined
    handler(d ?? {})
  }
  window.addEventListener('sail-theme-change', on)
  return () => window.removeEventListener('sail-theme-change', on)
}

/**
 * Dirty-document leave guard via `beforeunload`.
 * Return a non-empty string from `handler` to prompt (browser may ignore text).
 */
export function onBeforeUnloadGuard(handler: () => string | void | undefined | null): () => void {
  if (typeof window === 'undefined') return () => {}
  const on = (e: BeforeUnloadEvent) => {
    const msg = handler()
    if (msg) {
      e.preventDefault()
      e.returnValue = msg
    }
  }
  window.addEventListener('beforeunload', on)
  return () => window.removeEventListener('beforeunload', on)
}

/** Platform confirm dialog (tests inject via setPlatformConfirm). */
export type PlatformConfirmFn = (message: string) => boolean

let confirmFn: PlatformConfirmFn = (message) => {
  if (typeof window === 'undefined' || typeof window.confirm !== 'function') return true
  return window.confirm(message)
}

export function setPlatformConfirm(fn: PlatformConfirmFn | null): void {
  confirmFn =
    fn ??
    ((message) => {
      if (typeof window === 'undefined' || typeof window.confirm !== 'function') return true
      return window.confirm(message)
    })
}

export function platformConfirm(message: string): boolean {
  return confirmFn(message)
}

/** ⌘ on Apple platforms, Ctrl+ elsewhere — for rail tooltips. */
export function hotkeyModLabel(): string {
  if (typeof navigator === 'undefined') return 'Ctrl+'
  const p =
    (navigator as { userAgentData?: { platform?: string } }).userAgentData?.platform ||
    navigator.platform ||
    ''
  return /mac|iphone|ipad|ipod/i.test(p) ? '⌘' : 'Ctrl+'
}

/**
 * Shell hotkey map shape (handlers ignore the keyboard event).
 * Compatible with runtime `hotkeys` Action entry values.
 */
export type ShellHotkeysMap = Record<
  string,
  (() => void) | { run: () => void; allowInInputs?: boolean }
>

/**
 * Attach catalog hotkeys on the document root while shell is mounted.
 * Framework owns the ambient document target (apps must not pass documentElement).
 * `hotkeysAction` is the runtime `hotkeys` Action (passed from the app so this
 * package does not hard-depend on the full actions graph).
 */
export function attachShellHotkeys(
  map: ShellHotkeysMap,
  hotkeysAction: (
    node: Element,
    map: ShellHotkeysMap,
  ) => { destroy?: () => void } | void,
): () => void {
  if (typeof document === 'undefined') return () => {}
  const handle = hotkeysAction(document.documentElement, map)
  return () => {
    handle?.destroy?.()
  }
}

/** Nested host attr paint helpers (draft / registered theme preview). */
export function hostSetAttribute(host: unknown, name: string, value: string): void {
  ;(host as HTMLElement).setAttribute(name, value)
}

export function hostRemoveAttribute(host: unknown, name: string): void {
  ;(host as HTMLElement).removeAttribute(name)
}

/** Query under a root; returns opaque SailHost (framework casts once). */
export function hostQuerySelector(root: unknown, selector: string): SailHost | null {
  const r = root as ParentNode | null | undefined
  if (!r || typeof r.querySelector !== 'function') return null
  const el = r.querySelector(selector)
  return el ? asSailHost(el) : null
}

export function observeSubtree(root: unknown, onMutate: () => void): () => void {
  if (typeof MutationObserver === 'undefined') return () => {}
  if (!(root instanceof Node)) return () => {}
  const mo = new MutationObserver(() => onMutate())
  mo.observe(root, { childList: true, subtree: true })
  return () => mo.disconnect()
}

/**
 * Programmatic clipboard write (session actions, copy brief).
 * Prefer `use:clipboard` on click hosts; this path is for non-click handlers.
 */
export async function writeClipboardText(text: string): Promise<boolean> {
  try {
    if (typeof navigator === 'undefined' || !navigator.clipboard?.writeText) return false
    await navigator.clipboard.writeText(text)
    return true
  } catch {
    /* offline / denied */
  }
  return false
}

/** Source-editor host capabilities (CodeMirror CE / textarea). */
export type SourceEditorHost = {
  setCursor?: (line: number, col?: number) => void
  focus?: () => void
  value?: string
  tagName?: string
  querySelector?: (s: string) => {
    scrollTop?: number
    scrollHeight?: number
    clientHeight?: number
    textContent?: string | null
  } | null
  setDiagnostics?: (
    d: {
      line: number
      column?: number | undefined
      endLine?: number | undefined
      endColumn?: number | undefined
      severity: 'error' | 'warning' | 'info'
      message: string
      source?: string | undefined
    }[],
  ) => void
}

function applySourceScrollTextarea(
  ta: { value?: string; scrollTop: number; scrollHeight: number; clientHeight: number },
  line: number,
  col: number,
): void {
  const lines = String(ta.value ?? '').split('\n')
  const target = Math.min(Math.max(1, line), lines.length)
  const ratio = (target - 1) / Math.max(1, lines.length)
  ta.scrollTop = ratio * Math.max(0, ta.scrollHeight - ta.clientHeight)
  void col
}

/**
 * Scroll a registered source editor host to 1-based line/col.
 * Framework-owned — Pure product code must not dig DOM on the host.
 */
export function applySourceEditorScroll(
  host: unknown,
  line: number,
  col = 1,
): boolean {
  if (!host || line < 1) return false
  const el = host as SourceEditorHost & HTMLElement
  if (typeof el.setCursor === 'function') {
    el.setCursor(line, col)
    return true
  }
  if (el instanceof HTMLTextAreaElement || el.tagName === 'TEXTAREA') {
    applySourceScrollTextarea(el as HTMLTextAreaElement, line, col)
    return true
  }
  const scroller = el.querySelector?.('.cm-scroller') as
    | (HTMLElement & { scrollTop: number; scrollHeight: number; clientHeight: number })
    | null
    | undefined
  if (!scroller) return false
  const text =
    typeof el.value === 'string'
      ? el.value
      : (el.querySelector?.('.cm-content')?.textContent ?? '')
  const n = Math.max(1, String(text).split('\n').length)
  const ratio = (Math.min(line, n) - 1) / n
  scroller.scrollTop = ratio * Math.max(0, scroller.scrollHeight - scroller.clientHeight)
  return true
}

export type SourceEditorDiagnostic = {
  line: number
  column?: number | undefined
  endLine?: number | undefined
  endColumn?: number | undefined
  severity: 'error' | 'warning' | 'info'
  message: string
  source?: string | undefined
}

export function pushSourceEditorDiagnostics(
  host: unknown,
  diags: SourceEditorDiagnostic[],
): boolean {
  const el = host as SourceEditorHost | null
  if (!el || typeof el.setDiagnostics !== 'function') return false
  el.setDiagnostics(diags)
  return true
}
