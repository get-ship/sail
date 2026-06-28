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
        // TODO: push a new todo with id, label = draft(), done: false
        // then reset draft() to ''
      },
      toggle: (id: number): void => {
        // TODO: map todos() to flip the `done` flag on the matching id
      },
    }
  },
})
