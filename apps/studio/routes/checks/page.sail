<!--
  /checks deep-link → open Checks panel and land on Edit.
-->
<scope>
  style redirectRoot = { box: grow(1).minH(0).h(full), paint: bg(background) }
  style redirectText = { paint: fg(muted-foreground), text: fontSize(sm) }
</scope>

<controller>
import { setChecksPanelOpen } from '../../../../packages/sail/studio-shell/src/index.ts'
import { appRouteBase } from '../../src/compositions/shell/shell-rail'

export default controller(() => {
  mount(() => {
    setChecksPanelOpen(true)
    const base = appRouteBase()
    navigate(`${base}/edit`, { replace: true })
  })
  return {}
})
</controller>

<column studio-redirect checks-redirect align="center" justify="center" style(redirectRoot)>
  <p style(redirectText)>Opening Checks panel…</p>
</column>
