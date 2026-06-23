import { controller, signal } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

interface Todo { id: number; text: string }

export default controller<Scope>({
  scope: () => {
    const items = signal<Todo[]>([
      { id: 1, text: 'Read ship-views skill' },
      { id: 2, text: 'Open /demos' },
    ])
    const draft = signal('')
    let nextId = 3

    return {
      items,
      draft,
      onSubmit: (e: Event) => {
        e.preventDefault()
        const text = draft().trim()
        if (!text) return
        items.set([...items(), { id: nextId++, text }])
        draft.set('')
      },
      remove: (id: number) => {
        items.set(items().filter((t) => t.id !== id))
      },
    }
  },
})
