<scope>
  style tile = { size({ w: 48, h: 20 }).pad(4).bg(primary).radius(md) }
</scope>
<div>
  <div data-testid="styled-tile" style(tile)>tile</div>
  <span data-testid="plain">plain</span>
</div>
