<!-- Legacy path redirect → /docs/concepts/data -->
<scope>
  style redirectRoot = { box: grow(1).minH(0).h(full), paint: bg(background) }
  style redirectText = { paint: fg(muted-foreground), text: fontSize(sm) }
</scope>

<controller>
import { navigate } from '@ship/deck/router'

export default controller(() => {
  mount(() => {
    navigate('/docs/concepts/data', { replace: true })
  })
  return {}
})
</controller>

<column align="center" justify="center" style(redirectRoot)>
  <p style(redirectText)>Redirecting to Concepts…</p>
</column>
