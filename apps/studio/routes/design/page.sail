<!--
  /design → /themes (STUDIO-V1-THEMES cutover). Design product surface removed.
-->
<controller>
import { navigate } from '@ship/deck/router'
import { appRouteBase } from '../../src/shell/shell-rail'

export default controller(() => {
  mount(() => {
    const base = appRouteBase()
    navigate(`${base}/themes`, { replace: true })
  })
  return {}
})
</controller>

<column studio-redirect design-redirect align="center" justify="center">
  style({  box: grow(1).minH(0).h(full), paint: bg(background)  })

  <p>
    style({ paint: fg(muted-foreground), text: fontSize(sm) })
Opening Themes…</p>
</column>
