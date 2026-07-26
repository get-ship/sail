<controller>
// Authored controller for the `board` control-flow proof — a REAL
// `controller(setup)` export (not the spike's `() => scope` factory).
//
// The synthetic controller composes this by running its `setup(ctx)` — the
// same `resolveController` the browser hydrate path runs — so the authored
// state + methods resolve identically server-side (ship-js SSR) and client-
// side (jsdom mount). The contributed scope type is recovered from the
// `ControllerExport` so the template's `items()`/`add()`/`remove(id)` reads
// type-check against it.
import { controller, signal } from '@ship/sail-runtime'

export interface BoardItem {
  id: string
  label: string
  flagged: boolean
}

export default controller(() => {
    const items = signal<BoardItem[]>([
      { id: 'a', label: 'Alpha', flagged: false },
      { id: 'b', label: 'Bravo', flagged: true },
    ])
    const add = (): void => {
      const n = items().length
      items.set([...items(), { id: `n${n}`, label: `Item ${n}`, flagged: false }])
    }
    const remove = (id: string): void => {
      items.set(items().filter((i) => i.id !== id))
    }
    return { items, add, remove }
  })
</controller>

<column gap="3" sx={style.pad(6).maxW(breakpoint-tablet).mx(auto)}>
  <h1 sx={style.fontSize(2xl).fontWeight(bold).fg(foreground)}>Board</h1>
  <if cond={items().length > 0}>
  <column gap="2" sx={style.list(none).pad(0).m(0)}>
    <for as="item" of={items()} key={item.id}>
    <row gap="2" align="center" sx={style.radius(md).pad(2).borderWidth(1).border(transparent).when({ pressed: style.bg(hover-bg).border(warning) })} aria-pressed={item.flagged}>
      <demo-chip>{item.label}</demo-chip>
      <button on:click={() => remove(item.id)}>remove</button>
    </row>
    </for>
  </column>
  <else>
  <p sx={style.fg(muted-foreground).fontSize(sm).italic(true)}>Nothing on the board.</p>
  </else></if>
  <button on:click={() => add()}>add</button>
</column>
