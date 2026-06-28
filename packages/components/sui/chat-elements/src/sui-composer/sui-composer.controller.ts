import { controller, effect, ref, signal } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-composer.sui'

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const text = signal('')
    // Consumer-owned attachment payloads. The composer never fabricates
    // these — it echoes whatever the host assigned in the submit event.
    const attachments = signal<unknown[]>([])
    const fieldEl = ref<HTMLInputElement | HTMLTextAreaElement>(null)
    const sendEl = ref<HTMLElement>(null)
    return { text, attachments, fieldEl, sendEl }
  },

  mount(state, host) {
    // Expose `attachments` as a JS-property: `el.attachments = [...]`.
    const init = (host as unknown as { attachments?: unknown[] }).attachments
    if (init != null) state.attachments = init
    Object.defineProperty(host, 'attachments', {
      configurable: true,
      get: () => state.attachments,
      set: (v: unknown[]) => { state.attachments = v },
    })

    const submit = () => {
      if (state.disabled) return
      const text = state.text.trim()
      if (text.length === 0) return
      host.dispatchEvent(
        new CustomEvent('sui-composer-submit', {
          detail: { text, attachments: [...state.attachments] },
          bubbles: true,
          composed: true,
        }),
      )
      state.text = ''
    }

    const onKeydown = (e: KeyboardEvent) => {
      if (e.key !== 'Enter') return
      // Shift+Enter inserts a newline in multiline mode; plain Enter
      // always submits. Single-line mode submits on Enter regardless.
      if (state.multiline && e.shiftKey) return
      e.preventDefault()
      submit()
    }

    // `fieldEl` swaps element when `multiline` toggles the {#if}, so
    // track it via effect to rebind keydown onto the live field.
    const stopField = effect(() => {
      // HTMLElement (not the input|textarea union): addEventListener's typed
      // overload resolves the 'keydown' listener cleanly on a single element
      // type, where the union receiver collapses it to a non-assignable shape.
      const field = state.fieldEl as HTMLElement | null
      if (!field) return
      field.addEventListener('keydown', onKeydown)
      return () => field.removeEventListener('keydown', onKeydown)
    })

    const stopSend = effect(() => {
      const send = state.sendEl as HTMLElement | null
      if (!send) return
      send.addEventListener('click', submit)
      return () => send.removeEventListener('click', submit)
    })

    return () => {
      stopField()
      stopSend()
    }
  },
})
