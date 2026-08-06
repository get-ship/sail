<!--
  Root `/` → redirect to `/edit`. Deep links land on leaf surfaces.
-->
<controller>
import { navigate } from '@ship/deck/router'
import { appRouteBase } from '../src/shell/shell-rail'

export default controller(() => {
  mount(() => {
    const base = appRouteBase()
    navigate(`${base}/edit`, { replace: true })
  })
  return {}
})
</controller>

<column studio-redirect align="center" justify="center">
  style({  box: grow(1).minH(0).h(full), paint: bg(background)  })

  <p>
    style({ paint: fg(muted-foreground), text: fontSize(sm) })
Opening Edit…</p>
</column>
