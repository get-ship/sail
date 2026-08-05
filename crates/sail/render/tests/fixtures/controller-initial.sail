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

<column gap="3" sx={style.pad(4).bg(surface-0)}>
  <h1 sx={style.fontSize(xl).fg(foreground)}>{title()}</h1>
  <if {items().length > 0}>
    <column gap="2">
      <for {items()} as="item" key={item.id}>
        <row gap="2" sx={style.pad(2).bg(surface-1).radius(sm)}>
          <span sx={style.fg(foreground)}>{item.label}</span>
        </row>
      </for>
    </column>
    <else>
      <p sx={style.fg(muted-foreground)}>Nothing on the board.</p>
    </else>
  </if>
</column>
