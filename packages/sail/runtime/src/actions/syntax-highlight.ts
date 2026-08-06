import type { Action } from '../action'
import { highlight } from './highlight'

/**
 * `use:syntaxHighlight` — the `<code-block>` element's behavior.
 *
 * Element-mode stdlib cannot pass `use:syntaxHighlight={code}` (static splice).
 * At use sites the compiler typically emits:
 *   runAction(inner, syntaxHighlight)
 *   bindProp/bindAttr(host, 'code', () => …)
 * so the first action paint often sees no `code` yet, and bindProp only sets a
 * JS property (no attribute mutation for MutationObserver).
 *
 * Source resolution (first non-empty wins):
 *  1. Action param when present
 *  2. Host CE / stamped root `.code` property / `code` attribute
 *  3. Action node's `code` attribute
 *
 * We install a `code` property interceptor on the host so later bindProp
 * writes re-render; also observe attribute mutations + microtask/rAF.
 */
export type SyntaxHighlightParam = string | null | undefined

function findHost(el: HTMLElement): HTMLElement | null {
  if (
    el.getAttribute('data-sail-host') === 'code-block' ||
    el.classList.contains('sail-code-block')
  ) {
    return el
  }
  return (
    (el.closest('[data-sail-host="code-block"]') as HTMLElement | null) ??
    (el.closest('.sail-code-block') as HTMLElement | null) ??
    (el.closest('code-block') as HTMLElement | null)
  )
}

function readCode(node: HTMLElement | null): string {
  if (!node) return ''
  const prop = (node as HTMLElement & { code?: unknown }).code
  if (typeof prop === 'string' && prop.length > 0) return prop
  if (typeof prop === 'function') {
    try {
      const v = (prop as () => unknown)()
      if (typeof v === 'string' && v.length > 0) return v
    } catch {
      /* ignore */
    }
  }
  const attr = node.getAttribute('code')
  if (attr != null && attr.length > 0) return attr
  return ''
}

function resolveSource(el: HTMLElement, param: SyntaxHighlightParam): string {
  if (typeof param === 'string' && param.length > 0) return param
  const host = findHost(el)
  const fromHost = readCode(host)
  if (fromHost.length > 0) return fromHost
  return readCode(el)
}

/**
 * Wrap host.code so bindProp assignments (identity property, no attribute)
 * re-trigger highlight after the action mounts.
 */
function installCodeInterceptor(host: HTMLElement, onSet: () => void): () => void {
  const key = 'code'
  const existing = Object.getOwnPropertyDescriptor(host, key)
  // Don't double-wrap if we already installed.
  if ((host as HTMLElement & { __sailCodeHook?: boolean }).__sailCodeHook) {
    return () => {}
  }

  let store: unknown =
    existing && 'value' in existing
      ? existing.value
      : (host as HTMLElement & { code?: unknown }).code

  Object.defineProperty(host, key, {
    configurable: true,
    enumerable: true,
    get() {
      return store
    },
    set(v: unknown) {
      store = v
      // Reflect string codes to the attribute so SSR/debug + MO stay in sync.
      if (typeof v === 'string' && v.length > 0) {
        host.setAttribute('code', v)
      } else if (v == null || v === '') {
        host.removeAttribute('code')
      }
      onSet()
    },
  })
  ;(host as HTMLElement & { __sailCodeHook?: boolean }).__sailCodeHook = true

  return () => {
    delete (host as HTMLElement & { __sailCodeHook?: boolean }).__sailCodeHook
    if (existing) {
      Object.defineProperty(host, key, existing)
    } else {
      delete (host as HTMLElement & { code?: unknown }).code
      if (typeof store === 'string' && store.length > 0) {
        ;(host as HTMLElement & { code?: string }).code = store
      }
    }
  }
}

export const syntaxHighlight: Action<SyntaxHighlightParam> = (node, param) => {
  const el = node as HTMLElement
  const pre = el.querySelector('pre') ?? el
  let current = param
  const host = findHost(el) ?? el
  let last = ''

  const render = (): void => {
    const src = resolveSource(el, current)
    if (src === last && pre.childNodes.length > 0) return
    last = src
    // Always write — empty source clears a previous highlight.
    pre.innerHTML = src.length > 0 ? highlight(src) : ''
  }

  const unhook = installCodeInterceptor(host, render)

  render()

  // bindProp/bindAttr are sibling effects after this action — re-read after flush.
  queueMicrotask(render)
  const raf =
    typeof requestAnimationFrame === 'function'
      ? requestAnimationFrame(() => {
          render()
          // One more frame: CE upgrade / dual bind sometimes lands later.
          requestAnimationFrame(render)
        })
      : 0

  const mo = new MutationObserver(render)
  mo.observe(el, { attributes: true, attributeFilter: ['code'] })
  if (host !== el) {
    mo.observe(host, { attributes: true, attributeFilter: ['code'] })
  }

  return {
    update: (next) => {
      current = next
      render()
    },
    destroy: () => {
      mo.disconnect()
      unhook()
      if (raf && typeof cancelAnimationFrame === 'function') {
        cancelAnimationFrame(raf)
      }
    },
  }
}
