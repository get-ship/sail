<column gap="2" sx={style.pad(6).maxW(breakpoint-tablet).mx(auto)}>
  <h1 sx={style.fontSize(2xl).fontWeight(bold).fg(foreground)}>Hello from deck</h1>
  <p sx={style.fg(muted-foreground).fontSize(sm).leading(relaxed)}>
    This page is server-rendered by deck with no daemon, no ship domains, and
    no design-system primitives. The compiler evaluated <code sx={style.fontFamily(mono).bg(surface-2).padX(1).radius(md)}>{1 + 1}</code> at render time.
  </p>
</column>
