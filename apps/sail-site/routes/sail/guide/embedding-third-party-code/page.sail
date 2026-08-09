<!-- Legacy path redirect → /docs/guide/embedding-third-party-code -->
<scope>
  style redirectRoot = { box: grow(1).minH(0).h(full), paint: bg(background) }
  style redirectText = { paint: fg(muted-foreground), text: fontSize(sm) }
</scope>

<controller>
import { navigate } from '@ship/deck/router'

export default controller(() => {
  mount(() => {
    navigate('/docs/guide/embedding-third-party-code', { replace: true })
  })
  return {}
})
</controller>

<column align="center" justify="center" style(redirectRoot)>
  <p style(redirectText)>Redirecting to Guide…</p>
</column>
