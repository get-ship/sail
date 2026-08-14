<!-- Query list + form mutation ship.demo.addTodo; pressed style on done items. -->

<controller>
import { controller, query } from '@ship/sail-runtime'
import { ship } from '../../src/compositions/demo-clients'

export default controller(() => ({
  list: query(() => ship.demo.todos()),
}))
</controller>

<scope>
  style page = { pad(6).size({ max-w: breakpoint-tablet }).margin({ x: auto }) }
  style title = { fg(foreground).type({ size: 2xl, weight: bold }) }
  style muted = { fg(muted-foreground).type({ size: sm }) }
  style error = { fg(destructive).type({ size: sm }) }
  style list = { pad(0).margin(0).list(none) }
  style item = {
    pad(1).radius(md),
    when: {
      pressed: fg(muted-foreground).type({ decoration: line-through }),
    },
  }
</scope>

<column gap="3" style(page)>
  <h1 style(title)>Todos</h1>

  <form mutation="ship.demo.addTodo">
    <input name="text" placeholder="New todo…" />
    <button type="submit">Add</button>
  </form>

  <match {list()}>
    {loading => <p style(muted)>Loading…</p>}
    {error as err => <p style(error)>Error: {err.message}</p>}
    {ok as value =>
      <column gap="1" style(list)>
        <for {value.items as item} key={item.id}>
          <li aria-pressed={item.done} style(item)>{item.text}</li>
        </for>
      </column>
    }
  </match>
</column>
