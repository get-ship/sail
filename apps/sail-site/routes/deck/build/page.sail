<!-- Legacy path redirect → /docs/app/build -->
<scope>
  style redirectRoot = { box: flow({ grow: 1 }).size({ h: full, min-h: 0 }), paint: fill({ color: background }) }
  style redirectText = { paint: ink({ color: muted-foreground }), text: type({ size: sm }) }
</scope>

<controller>

export default controller(() => {
  mount(() => {
    navigate('/docs/app/build', { replace: true })
  })
  return {}
})
</controller>

<column align="center" justify="center" style(redirectRoot)>
  <p style(redirectText)>Redirecting to App docs…</p>
</column>
