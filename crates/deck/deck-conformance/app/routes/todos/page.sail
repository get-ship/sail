<controller>
import { controller, query } from '@ship/sail-runtime'
import { call } from 'ship'
export default controller((_ctx) => ({ list: query(() => call('ship.demo.todos', {})) }))
</controller>

<column gap="3" style({ box: pad(6).maxW(breakpoint-tablet).mx(auto) })>
  <h1 style({ paint: fg(foreground), text: fontSize(2xl).fontWeight(bold) })>Todos</h1>

  <form mutation="demo.addTodo">
    <input name="text" placeholder="New todo…" />
    <button type="submit">Add</button>
  </form>

  <match {list()}>
  {loading => <p style({ paint: fg(muted-foreground), text: fontSize(sm) })>Loading…</p>}
  {error as err => <p style({ paint: fg(destructive), text: fontSize(sm) })>Error: {err.message}</p>}
  {ok as value => <column gap="1" style({ box: pad(0).m(0), paint: list(none) })>
    <for {value.items as item} key={item.id}>
    <li aria-pressed={item.done} style({ box: pad(1), paint: radius(md) }).when(pressed, { paint: fg(muted-foreground), text: underline(line-through) })>{item.text}</li>
    </for>
  </column>}
</match>
</column>
