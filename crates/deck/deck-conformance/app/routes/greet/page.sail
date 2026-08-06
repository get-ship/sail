<column gap="2">
  style({ box: pad(6).maxW(breakpoint-tablet).mx(auto) })

  <h1>
    style({ paint: fg(foreground), text: fontSize(2xl).fontWeight(bold) })
Greet</h1>
  <p>
    style({ paint: fg(muted-foreground), text: fontSize(sm) })
Pick a name to greet:</p>
  <ul>
    style({ box: pad(0).m(0), paint: list(none) })

    <li><a href="/_deck/view/greet/zelda">greet/zelda</a></li>
    <li><a href="/_deck/view/greet/link">greet/link</a></li>
    <li><a href="/_deck/view/greet/ganon">greet/ganon</a></li>
  </ul>
</column>
