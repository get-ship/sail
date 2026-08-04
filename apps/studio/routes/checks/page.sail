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

<column
  studio-redirect
  checks-redirect
  grow
  align="center"
  justify="center"
  sx={style.minH(0).h(full).bg(background)}
>
  <p sx={style.fontSize(sm).fg(muted-foreground)}>Opening Checks panel…</p>
</column>
