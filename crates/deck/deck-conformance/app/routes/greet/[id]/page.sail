<!-- Dynamic segment: route.params.id() in the heading. -->

<scope>
  style page = { box: pad(6).maxW(breakpoint-tablet).mx(auto) }
  style title = { paint: fg(foreground), text: fontSize(2xl).fontWeight(bold) }
</scope>

<column gap="2" style(page)>
  <h1 style(title)>Hello, {route.params.id()}</h1>
  <p><a href="/_deck/view/greet">back to greet</a></p>
</column>
