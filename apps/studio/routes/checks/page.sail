<!--
  /checks deep-link → open Checks panel and land on Edit.
-->
<scope>
  style redirectRoot = { grow(1).size({ h: full, min-h: 0 }).bg(background) }
  style redirectText = { fg(muted-foreground).type({ size: sm }) }
</scope>

<controller>
import { setChecksPanelOpen } from '../../src/compositions/studio-shell'
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
