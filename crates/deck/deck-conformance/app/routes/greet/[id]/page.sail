<!-- Dynamic segment: route.params.id() in the heading. -->

<scope>
  style page = { pad(6).size({ max-w: breakpoint-tablet }).margin({ x: auto }) }
  style title = { fg(foreground).type({ size: 2xl, weight: bold }) }
</scope>

<column gap="2" style(page)>
  <h1 style(title)>Hello, {route.params.id()}</h1>
  <p><a href="/_deck/view/greet">back to greet</a></p>
</column>
