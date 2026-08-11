<!-- Legacy path redirect → /docs/reference/template -->
<scope>
  style redirectRoot = { box: grow(1).minH(0).h(full), paint: bg(background) }
  style redirectText = { paint: fg(muted-foreground), text: fontSize(sm) }
</scope>

<controller>

export default controller(() => {
  mount(() => {
    navigate('/docs/reference/template', { replace: true })
  })
  return {}
})
</controller>

<column align="center" justify="center" style(redirectRoot)>
  <p style(redirectText)>Redirecting to Reference…</p>
</column>
