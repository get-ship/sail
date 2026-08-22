<!-- SSR constant-fold: {1 + 1} evaluated at render time (no controller). -->

<scope>
  style page = { pad(6).size({ max-w: breakpoint-tablet }).margin({ x: auto }) }
  style title = { fg(foreground).type({ size: 2xl, weight: bold }) }
  style lead = { fg(muted-foreground).type({ size: sm, leading: relaxed }) }
  style code = { pad({ x: 1 }).bg(surface).radius(md).type({ family: mono }) }
</scope>

<column gap="2" style(page)>
  <h1 style(title)>Hello from deck</h1>
  <p style(lead)>
    This page is server-rendered by deck with no daemon, no ship domains, and
    no design-system primitives. The compiler evaluated <code style(code)>{1 + 1}</code> at render time.
  </p>
</column>
