import { controller, signal } from '@ship/sui-runtime'
import type { Scope } from './template.sui'

interface Todo { id: number; label: string; done: boolean }

let nextId = 0

export default controller<Scope>({
  scope: () => {
    const todos = signal<Todo[]>([])
    const draft = signal('')

    return {
      todos, draft,
      setDraft: (v: string): void => draft.set(v),
      add: (): void => {
        const label = draft().trim()
        if (!label) return
        todos.set([...todos(), { id: ++nextId, label, done: false }])
        draft.set('')
      },
      toggle: (id: number): void => {
        todos.set(todos().map((t) => t.id === id ? { ...t, done: !t.done } : t))
      },
    }
  },
})
