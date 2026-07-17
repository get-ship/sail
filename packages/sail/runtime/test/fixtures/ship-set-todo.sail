<controller>
import { controller, signal } from '@ship/sail-runtime'

interface TodoItem {
  id: number
  text: string
  done: boolean
}

export default controller(() => {
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
  })
</controller>

<div>
  <input data-testid="draft" bind:value={draft} />
  <button data-testid="add" on:click={() => addItem()}>add</button>
  <span data-testid="count">{items().length} item(s)</span>
  <ul data-testid="list">
    {#for item of items() key={item.id}}
      <li data-testid="item">
        <button data-testid="toggle" on:click={() => toggleItem(item.id)}>
          {item.done ? 'X' : 'O'}
        </button>
        <span>{item.text}</span>
        <button data-testid="remove" on:click={() => removeItem(item.id)}>x</button>
      </li>
    {/for}
  </ul>
</div>
