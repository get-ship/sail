<column gap="2" style({ box: pad(6).maxW(breakpoint-tablet).mx(auto) })>
  <h1 style({ paint: fg(foreground), text: fontSize(2xl).fontWeight(bold) })>Hello, {route.params.id()}</h1>
  <p><a href="/_deck/view/greet">back to greet</a></p>
</column>
