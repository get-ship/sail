<!-- SSR constant-fold: {1 + 1} evaluated at render time (no controller). -->

<scope>
  style page = { box: pad(6).maxW(breakpoint-tablet).mx(auto) }
  style title = { paint: fg(foreground), text: fontSize(2xl).fontWeight(bold) }
  style lead = { paint: fg(muted-foreground), text: fontSize(sm).leading(relaxed) }
  style code = {
    box: padX(1),
    paint: bg(surface-2).radius(md),
    text: fontFamily(mono),
  }
</scope>

<column gap="2" style(page)>
  <h1 style(title)>Hello from deck</h1>
  <p style(lead)>
    This page is server-rendered by deck with no daemon, no ship domains, and
    no design-system primitives. The compiler evaluated <code style(code)>{1 + 1}</code> at render time.
  </p>
</column>
