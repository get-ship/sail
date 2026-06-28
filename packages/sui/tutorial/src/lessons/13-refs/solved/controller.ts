import { controller, signal, effect } from '@ship/sui-runtime'
import type { Scope } from './template.sui'

export default controller<Scope>({
  scope: () => {
    const inputEl = signal<HTMLInputElement | null>(null)
    const text = signal('')
    const width = signal(0)

    // Focus on mount.
    effect(() => {
      inputEl()?.focus()
    })

    // Observe size changes.
    effect(() => {
      const el = inputEl()
      if (!el) return
      const ro = new ResizeObserver(() => width.set(Math.round(el.getBoundingClientRect().width)))
      ro.observe(el)
      return () => ro.disconnect()
    })

    return { inputEl, text, width }
  },
})
