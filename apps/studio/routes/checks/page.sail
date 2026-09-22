<!--
  /checks deep-link → open Checks panel and land on Edit.
-->
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

<column studio-redirect checks-redirect align="center" justify="center">
  <p>Opening Checks panel…</p>
</column>
