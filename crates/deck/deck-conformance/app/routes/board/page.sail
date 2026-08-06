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

<column gap="3">
  style({ box: pad(6).maxW(breakpoint-tablet).mx(auto) })

  <h1>
    style({ paint: fg(foreground), text: fontSize(2xl).fontWeight(bold) })
Board</h1>
  <if {items().length > 0}>
  <column gap="2">
    style({ box: pad(0).m(0), paint: list(none) })

    <for {items() as item} key={item.id}>
    <row gap="2" align="center" aria-pressed={item.flagged}>
      style({ box: pad(2), paint: radius(md).borderWidth(1).border(transparent) }).when(pressed, { paint: bg(hover-bg).border(warning) })

      <demo-chip>{item.label}</demo-chip>
      <button on:click={() => remove(item.id)}>remove</button>
    </row>
    </for>
  </column>
  {else => <p>
    style({ paint: fg(muted-foreground), text: fontSize(sm).italic(true) })
Nothing on the board.</p>}</if>
  <button on:click={() => add()}>add</button>
</column>
