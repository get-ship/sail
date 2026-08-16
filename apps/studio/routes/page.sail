<!-- Root `/` → `/edit`. Deep links land on leaf surfaces. -->
<controller>
import { appRouteBase } from '../src/compositions/shell/shell-rail'

export default controller(() => {
  mount(() => {
    const base = appRouteBase()
    navigate(`${base}/edit`, { replace: true })
  })
  return {}
})
</controller>

<column studio-redirect align="center" justify="center">
  <p>Opening Edit…</p>
</column>
