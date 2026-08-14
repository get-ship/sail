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

<scope>
  style page = { pad(6).margin({ x: auto }).size({ max-w: breakpoint-tablet }) }
  style muted = { fg(muted-foreground).type({ size: sm }) }
  style todoRow = { pad({ x: 3, y: 2 }).border({ color: border, width: 1 }).radius(md).bg(surface-1) }
</scope>

<column gap="4" style(page)>
  <p><a href="/demos">← demos</a></p>
  <column gap="1">
    <h1>iteration</h1>
    <p style(muted)>
      Block <code>for</code> with a stable <code>key=</code>. The key tells
      the runtime which DOM nodes belong to which item across re-renders.
    </p>
  </column>

  <form on:submit={onSubmit}>
    <row gap="2" align="center">
      <input
        type="text"
        placeholder="What's next?"
        data-test="todo-input"
        bind:value={draft}
      />
      <button type="submit" data-test="todo-add" variant="solid" tone="primary">add</button>
    </row>
  </form>

  <if {items().length === 0}>
    <p data-test="empty" style(muted)>List is empty.</p>
  {else =>
    <column gap="1" data-test="todos">
      <for {items() as t} key={t.id}>
        <row align="center" justify="between" style(todoRow)>
          <span>{t.text}</span>
          <button data-test="remove" variant="ghost" size="sm" on:click={() => remove(t.id)}>×</button>
        </row>
      </for>
    </column>
  }</if>
</column>
