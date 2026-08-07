<!-- SSR-RENDER fixture: controller-driven content that used to paint empty. -->
<controller>
import { controller, signal } from '@ship/sail-runtime'

export default controller(() => {
  const title = signal('Launch checklist')
  const items = signal([
    { id: 'a', label: 'Brand tokens' },
    { id: 'b', label: 'Mini-app routes' },
  ])
  return { title, items }
})
</controller>

<column gap="3" style({ box: pad(4), paint: bg(surface-0) })>
  <h1 style({ paint: fg(foreground), text: fontSize(xl) })>{title()}</h1>
  <if {items().length > 0}>
    <column gap="2">
      <for {items() as item} key={item.id}>
        <row gap="2" style({ box: pad(2), paint: bg(surface-1).radius(sm) })>
          <span style({ paint: fg(foreground) })>{item.label}</span>
        </row>
      </for>
    </column>
    {else => <p style({ paint: fg(muted-foreground) })>Nothing on the board.</p>}
  </if>
</column>
