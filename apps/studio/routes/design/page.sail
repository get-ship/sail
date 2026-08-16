<!--
  /design → /themes redirect.
-->
<controller>
import { appRouteBase } from '../../src/compositions/shell/shell-rail'

export default controller(() => {
  mount(() => {
    const base = appRouteBase()
    navigate(`${base}/themes`, { replace: true })
  })
  return {}
})
</controller>

<column studio-redirect design-redirect align="center" justify="center">
  <p>Opening Themes…</p>
</column>
