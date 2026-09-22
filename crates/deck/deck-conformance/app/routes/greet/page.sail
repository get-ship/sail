<!-- Static links into the dynamic greet/[id] segment. -->

<scope>
  style page = { pad(6).size({ max-w: breakpoint-tablet }).margin({ x: auto }) }
  style title = { fg(foreground).type({ size: 2xl, weight: bold }) }
  style muted = { fg(muted-foreground).type({ size: sm }) }
  style list = { pad(0).margin(0).list(none) }
</scope>

<column gap="2" style(page)>
  <h1 style(title)>Greet</h1>
  <p style(muted)>Pick a name to greet:</p>
  <ul style(list)>
    <li><a href="/_deck/view/greet/zelda">greet/zelda</a></li>
    <li><a href="/_deck/view/greet/link">greet/link</a></li>
    <li><a href="/_deck/view/greet/ganon">greet/ganon</a></li>
  </ul>
</column>
