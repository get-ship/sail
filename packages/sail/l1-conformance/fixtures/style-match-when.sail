<div data-t="match-solid" variant="solid" style({
  match: {
    variant: {
      solid: { paint: bg(primary).fg(primary-foreground) },
      soft:  { paint: bg(muted).fg(muted-foreground) },
    }
  },
  when: { hover: { paint: opacity(50) } }
})>solid</div>
<div data-t="match-soft" variant="soft" style({
  match: {
    variant: {
      solid: { paint: bg(primary).fg(primary-foreground) },
      soft:  { paint: bg(muted).fg(muted-foreground) },
    }
  },
  when: { hover: { paint: opacity(50) } }
})>soft</div>
<div data-t="hover-host" style({
  paint: bg(primary).opacity(100),
  when: { hover: { paint: opacity(50) } }
})>hover</div>
<div data-t="when-wrap">
  <when {tablet}>
    <div data-t="wide" style({ box: w(full) })>wide</div>
    {else => <div data-t="narrow" style({ box: w(full) })>narrow</div>}
  </when>
</div>
