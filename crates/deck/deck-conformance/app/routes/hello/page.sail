<column gap="2">
  style({ box: pad(6).maxW(breakpoint-tablet).mx(auto) })

  <h1>
    style({ paint: fg(foreground), text: fontSize(2xl).fontWeight(bold) })
Hello from deck</h1>
  <p>
    style({ paint: fg(muted-foreground), text: fontSize(sm).leading(relaxed) })

    This page is server-rendered by deck with no daemon, no ship domains, and
    no design-system primitives. The compiler evaluated <code>
      style({ box: padX(1), paint: bg(surface-2).radius(md), text: fontFamily(mono) })
{1 + 1}</code> at render time.
  </p>
</column>
