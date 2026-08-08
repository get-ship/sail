<!-- Static links into the dynamic greet/[id] segment. -->

<scope>
  style page = { box: pad(6).maxW(breakpoint-tablet).mx(auto) }
  style title = { paint: fg(foreground), text: fontSize(2xl).fontWeight(bold) }
  style muted = { paint: fg(muted-foreground), text: fontSize(sm) }
  style list = { box: pad(0).m(0), paint: list(none) }
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
