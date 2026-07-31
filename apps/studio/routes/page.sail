<!--
  Root `/` → redirect to `/edit`. Deep links land on leaf surfaces.
-->
<controller>
import { navigate } from '@ship/deck/router'
import { appRouteBase } from '../src/shell/shell-page-vm'

export default controller(() => {
  mount(() => {
    const base = appRouteBase()
    navigate(`${base}/edit`, { replace: true })
  })
  return {}
})
</controller>

<column
  data-studio-redirect
  grow
  align="center"
  justify="center"
  sx={style.minH(0).h(full).bg(background)}
>
  <p sx={style.fontSize(sm).fg(muted-foreground)}>Opening Edit…</p>
</column>
