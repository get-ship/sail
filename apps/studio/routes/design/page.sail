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

<column
  studio-redirect
  design-redirect
  grow
  align="center"
  justify="center"

>
  style.minH(0).h(full).bg(background)

  <p>
    style.fontSize(sm).fg(muted-foreground)
Opening Themes…</p>
</column>
