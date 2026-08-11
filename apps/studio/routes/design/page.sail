<!--
  /design → /themes redirect.
-->
<scope>
  style redirectRoot = { box: grow(1).minH(0).h(full), paint: bg(background) }
  style redirectText = { paint: fg(muted-foreground), text: fontSize(sm) }
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
