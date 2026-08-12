import type { Action } from '../action'

/**
 * `use:passwordReveal` — product peek control for password fields.
 * Stamped by the compiler on freeform `<input type="password">` when
 * `reveal` is not false. Injects an end-aligned toggle button (padE room
 * reserved by host Style). Zero-op when type is not password.
 */
export const passwordReveal: Action<void> = (node) => {
  const el = node as HTMLInputElement
  if (el.tagName !== 'INPUT') return {}

  let btn: HTMLButtonElement | null = null
  let shown = false

  const ensure = (): void => {
    if (el.getAttribute('type') !== 'password' && el.getAttribute('type') !== 'text') {
      teardown()
      return
    }
    // Only own fields that started as password (or still are).
    const revealOff =
      el.getAttribute('reveal') === 'false' ||
      el.getAttribute('reveal') === '0' ||
      el.getAttribute('reveal') === 'off'
    if (revealOff) {
      teardown()
      return
    }
    if (btn) return
    const parent = el.parentElement
    if (!parent) return
    if (getComputedStyle(parent).position === 'static') {
      parent.style.position = 'relative'
    }
    btn = document.createElement('button')
    btn.type = 'button'
    btn.tabIndex = -1
    btn.setAttribute('aria-label', 'Show password')
    btn.setAttribute('aria-pressed', 'false')
    Object.assign(btn.style, {
      position: 'absolute',
      right: '0.5rem',
      top: '50%',
      transform: 'translateY(-50%)',
      border: 'none',
      background: 'transparent',
      padding: '0.25rem',
      margin: '0',
      cursor: 'pointer',
      lineHeight: '1',
      color: 'var(--muted-foreground, currentColor)',
      fontSize: '0.75rem',
    })
    btn.textContent = 'Show'
    btn.addEventListener('click', onToggle)
    // Position relative to the input: wrap if parent is a form row with siblings.
    if (el.nextSibling !== btn) {
      el.insertAdjacentElement('afterend', btn)
      // Prefer wrapping so absolute coords attach to the field box.
      const wrap = document.createElement('span')
      wrap.style.position = 'relative'
      wrap.style.display = 'block'
      wrap.style.width = el.style.width || '100%'
      el.parentElement?.insertBefore(wrap, el)
      wrap.appendChild(el)
      wrap.appendChild(btn)
    }
  }

  const onToggle = (e: Event): void => {
    e.preventDefault()
    shown = !shown
    el.type = shown ? 'text' : 'password'
    if (btn) {
      btn.textContent = shown ? 'Hide' : 'Show'
      btn.setAttribute('aria-pressed', shown ? 'true' : 'false')
      btn.setAttribute('aria-label', shown ? 'Hide password' : 'Show password')
    }
  }

  const teardown = (): void => {
    if (btn) {
      btn.removeEventListener('click', onToggle)
      btn.remove()
      btn = null
    }
  }

  // Defer until connected so parent exists.
  queueMicrotask(ensure)
  const obs = new MutationObserver(() => ensure())
  obs.observe(el, { attributes: true, attributeFilter: ['type', 'reveal'] })

  return {
    destroy: () => {
      obs.disconnect()
      teardown()
    },
  }
}
