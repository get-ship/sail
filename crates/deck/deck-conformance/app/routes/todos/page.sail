<!-- Query list + form mutation ship.demo.addTodo; pressed style on done items. -->

<controller>
import { controller, query } from '@ship/sail-runtime'
import { ship } from '../../src/compositions/demo-clients'

export default controller(() => ({
  list: query(() => ship.demo.todos()),
}))
</controller>

<scope>
  style page = { box: pad(6).maxW(breakpoint-tablet).mx(auto) }
  style title = { paint: fg(foreground), text: fontSize(2xl).fontWeight(bold) }
  style muted = { paint: fg(muted-foreground), text: fontSize(sm) }
  style error = { paint: fg(destructive), text: fontSize(sm) }
  style list = { box: pad(0).m(0), paint: list(none) }
  style item = {
    box: pad(1),
    paint: radius(md),
    when: {
      pressed: { paint: fg(muted-foreground), text: underline(line-through) },
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
