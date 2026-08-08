<!-- Control-flow proof: signal list + add/remove; SSR and hydrate share this setup. -->

<controller>
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

<scope>
  style page = { box: pad(6).maxW(breakpoint-tablet).mx(auto) }
  style title = { paint: fg(foreground), text: fontSize(2xl).fontWeight(bold) }
  style list = { box: pad(0).m(0), paint: list(none) }
  style row = {
    box: pad(2),
    paint: radius(md).borderWidth(1).border(transparent),
    when: { pressed: { paint: bg(hover-bg).border(warning) } },
  }
  style empty = { paint: fg(muted-foreground), text: fontSize(sm).italic(true) }
</scope>

<column gap="3" style(page)>
  <h1 style(title)>Board</h1>
  <if {items().length > 0}>
    <column gap="2" style(list)>
      <for {items() as item} key={item.id}>
        <row gap="2" align="center" aria-pressed={item.flagged} style(row)>
          <demo-chip>{item.label}</demo-chip>
          <button on:click={() => remove(item.id)}>remove</button>
        </row>
      </for>
    </column>
    {else =>
      <p style(empty)>Nothing on the board.</p>
    }
  </if>
  <button on:click={() => add()}>add</button>
</column>
