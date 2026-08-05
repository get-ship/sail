<controller>
import { controller, query } from '@ship/sail-runtime'
import { call } from 'ship'
export default controller((_ctx) => ({ list: query(() => call('ship.demo.todos', {})) }))
</controller>

<column gap="3" sx={style.pad(6).maxW(breakpoint-tablet).mx(auto)}>
  <h1 sx={style.fontSize(2xl).fontWeight(bold).fg(foreground)}>Todos</h1>

  <form mutation="demo.addTodo">
    <input name="text" placeholder="New todo…" />
    <button type="submit">Add</button>
  </form>

  <while-loading {list}>
  <p sx={style.fg(muted-foreground).fontSize(sm)}>Loading…</p>
  </while-loading>
  <while-error as="err" {list}>
  <p sx={style.fg(destructive).fontSize(sm)}>Error: {err.message}</p>
  </while-error>
  <if {list().kind === 'ok'}>
  <column gap="1" sx={style.list(none).pad(0).m(0)}>
    <for {list().value.items} as="item" key={item.id}>
    <li aria-pressed={item.done} sx={style.radius(md).pad(1).when({ pressed: style.underline(line-through).fg(muted-foreground) })}>{item.text}</li>
    </for>
  </column>
  </if>
</column>
