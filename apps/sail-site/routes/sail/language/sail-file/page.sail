<!-- Legacy path redirect → /docs/concepts/sail-file -->
<scope>
  style redirectRoot = { grow(1).size({ h: full, min-h: 0 }).bg(background) }
  style redirectText = { fg(muted-foreground).type({ size: sm }) }
</scope>

<controller>

export default controller(() => {
  mount(() => {
    navigate('/docs/concepts/sail-file', { replace: true })
  })
  return {}
})
</controller>

<column align="center" justify="center" style(redirectRoot)>
  <p style(redirectText)>Redirecting to Concepts…</p>
</column>
