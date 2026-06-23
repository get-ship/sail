import { controller, signal } from '@ship/sui-runtime'

interface TodoItem {
  id: number
  text: string
  done: boolean
}

export default controller({
  scope: () => {
    let nextId = 3
    const items = signal<TodoItem[]>([
      { id: 1, text: 'buy milk', done: false },
      { id: 2, text: 'write tests', done: false },
    ])
    const draft = signal('')

    const addItem = () => {
      const text = draft().trim()
      if (!text) return
      items.set([...items(), { id: nextId++, text, done: false }])
      draft.set('')
    }

    const toggleItem = (id: number) => {
      items.set(items().map((it) => (it.id === id ? { ...it, done: !it.done } : it)))
    }

    const removeItem = (id: number) => {
      items.set(items().filter((it) => it.id !== id))
    }

    return { items, draft, addItem, toggleItem, removeItem }
  },
})
