<scope>
  style tile = { box: size({ w: 48, h: 20 }).space({ pad: 4 }), paint: fill({ color: primary }).radius(md) }
</scope>
<div>
  <div data-testid="styled-tile" style(tile)>tile</div>
  <span data-testid="plain">plain</span>
</div>
