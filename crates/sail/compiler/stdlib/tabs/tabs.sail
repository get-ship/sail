<!--
  <tabs> — nowrap row. Horizontal is dir(column): rail on top, panel
  under. Vertical is dir(row): rail on the start edge, panel beside.

  Nested API:
    <tabs>
      <tab title="Overview">panel…</tab>
    </tabs>

  The host controller projects triggers into the built-in <tab-list> rail so the
  rail is a real layout child (not display:contents hunting). Parts form
  still works: an authored <tab-list> is the rail; the built-in one stays idle.
-->
<scope>
  props: {
    active = ''
    orientation: 'horizontal' | 'vertical' = 'horizontal'
    variant: 'line' | 'pill' | 'enclosed' = 'line'
    onTabChange: (detail: { name: string }) => void
  }

  style host = {size({ min-w: 0 })}
  // Horizontal is a column: rail on top, panel under it.
  // Vertical is a nowrap row: rail on the start edge, panel beside it.
  // A wrapping row made vertical look like horizontal — panel dropped under.
  style horiz = {dir(column)}
  style vert = {dir(row)}
</scope>
<controller>
import { tabs } from '@ship/sail-runtime/actions'
export default controller((_ctx, host) => {
  mount((el) => {
    const result = tabs((el ?? host) as HTMLElement)
    return () => {
      result?.destroy?.()
    }
  })
  return {}
})
</controller>
<row nowrap gap="3" style(host, {match: { orientation: { horizontal: horiz, vertical: vert } }})>
  <tab-list data-tabs-rail hidden></tab-list>
  <slot></slot>
</row>
