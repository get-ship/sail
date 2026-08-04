<!--
  /design → /themes (STUDIO-V1-THEMES cutover). Design product surface removed.
-->
<controller>
import { navigate } from '@ship/deck/router'
import { appRouteBase } from '../../src/shell/shell-page-vm'

export default controller(() => {
  mount(() => {
    const base = appRouteBase()
    navigate(`${base}/themes`, { replace: true })
  })
  return {}
})
</controller>

<column
  data-studio-redirect
  data-design-redirect
  grow
  align="center"
  justify="center"
  sx={style.minH(0).h(full).bg(background)}
>
  <p sx={style.fontSize(sm).fg(muted-foreground)}>Opening Themes…</p>
</column>
