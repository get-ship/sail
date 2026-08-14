<!--
  /design → /themes redirect.
-->
<scope>
  style redirectRoot = { grow(1).size({ h: full, min-h: 0 }).bg(background) }
  style redirectText = { fg(muted-foreground).type({ size: sm }) }
</scope>

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

<column studio-redirect design-redirect align="center" justify="center" style(redirectRoot)>
  <p style(redirectText)>Opening Themes…</p>
</column>
