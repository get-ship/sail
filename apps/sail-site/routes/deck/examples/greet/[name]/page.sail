<!-- Legacy path redirect → /docs/app/examples/greet/ada -->
<scope>
  style redirectRoot = { grow(1).size({ h: full, min-h: 0 }).bg(background) }
  style redirectText = { fg(muted-foreground).type({ size: sm }) }
</scope>

<controller>

export default controller(() => {
  mount(() => {
    navigate('/docs/app/examples/greet/ada', { replace: true })
  })
  return {}
})
</controller>

<column align="center" justify="center" style(redirectRoot)>
  <p style(redirectText)>Redirecting to App example…</p>
</column>
