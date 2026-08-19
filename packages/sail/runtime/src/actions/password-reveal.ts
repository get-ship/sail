import type { Action } from '../action'

/**
 * `use:passwordReveal` — peek control. Paint is Style / platform CSS
 * (Job A). Anatomy in `input.sail` is E/C. This stamp only toggles type.
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
    const revealOff =
      el.getAttribute('reveal') === 'false' ||
      el.getAttribute('reveal') === '0' ||
      el.getAttribute('reveal') === 'off'
    if (revealOff) {
      teardown()
      return
    }
    if (btn) return
    btn = document.createElement('button')
    btn.type = 'button'
    btn.tabIndex = -1
    btn.setAttribute('aria-label', 'Show password')
    btn.setAttribute('aria-pressed', 'false')
    btn.textContent = 'Show'
    btn.addEventListener('click', onToggle)
    el.insertAdjacentElement('afterend', btn)
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
