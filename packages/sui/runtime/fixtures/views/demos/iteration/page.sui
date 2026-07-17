<controller>
import { controller, signal } from '@ship/sail-runtime'

interface Todo { id: number; text: string }

export default controller(() => {
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
  })
</controller>

<div data-demo="true">
  <p><a href="/demos">← demos</a></p>
  <h1>iteration</h1>
  <p>
    Block <code>for</code> with a stable <code>key=</code>. The key tells
    the runtime which DOM nodes belong to which item across re-renders.
  </p>

  <form on:submit={onSubmit}>
    <input
      type="text"
      placeholder="What's next?"
      data-test="todo-input"
      bind:value={draft} />
    <button type="submit" data-test="todo-add">add</button>
  </form>

  {#if items().length === 0}
    <p data-muted="true" data-test="empty">List is empty.</p>
  {:else}
    <ul data-test="todos">
      {#for t of items() key={t.id}}
        <li>
          <span>{t.text}</span>
          <button data-test="remove" on:click={() => remove(t.id)}>×</button>
        </li>
      {/for}
    </ul>
  {/if}
</div>
