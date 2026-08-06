<!--
  /checks deep-link → open Checks panel + land on Edit (not a rail home).
  STUDIO-V1-LAUNCH: Checks is a shell panel, not a fifth activity page.
-->
<controller>
import { navigate } from '@ship/deck/router'
import { setChecksPanelOpen } from '@ship/sail-studio-shell'
import { appRouteBase } from '../../src/shell/shell-rail'

export default controller(() => {
  mount(() => {
    setChecksPanelOpen(true)
    const base = appRouteBase()
    navigate(`${base}/edit`, { replace: true })
  })
  return {}
})
</controller>

<column studio-redirect checks-redirect align="center" justify="center">
  style({  box: grow(1).minH(0).h(full), paint: bg(background)  })

  <p>
    style({ paint: fg(muted-foreground), text: fontSize(sm) })
Opening Checks panel…</p>
</column>
