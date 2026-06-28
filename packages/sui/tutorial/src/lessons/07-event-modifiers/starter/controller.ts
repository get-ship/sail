import { controller, signal } from '@ship/sui-runtime'
import type { Scope } from './template.sui'

interface Msg { id: number; text: string }

let nextId = 0

export default controller<Scope>({
  scope: () => {
    const messages = signal<Msg[]>([])
    const draft = signal('')

    return {
      messages, draft,
      send: (): void => {
        const text = draft().trim()
        if (!text) return
        messages.set([...messages(), { id: ++nextId, text }])
        draft.set('')
      },
      // TODO: add `dismiss` + `firstAction` handlers.
    }
  },
})
