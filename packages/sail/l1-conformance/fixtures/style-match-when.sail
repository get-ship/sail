<div data-t="match-solid" variant="solid" style({
  match: {
    variant: {
      solid: { paint: fill({ color: primary }).ink({ color: primary-foreground }) },
      soft:  { paint: fill({ color: muted }).ink({ color: muted-foreground }) },
    }
  },
  when: { hover: { paint: filter({ opacity: 50 }) } }
})>solid</div>
<div data-t="match-soft" variant="soft" style({
  match: {
    variant: {
      solid: { paint: fill({ color: primary }).ink({ color: primary-foreground }) },
      soft:  { paint: fill({ color: muted }).ink({ color: muted-foreground }) },
    }
  },
  when: { hover: { paint: filter({ opacity: 50 }) } }
})>soft</div>
<div data-t="hover-host" style({
  paint: fill({ color: primary }).filter({ opacity: 100 }),
  when: { hover: { paint: filter({ opacity: 50 }) } }
})>hover</div>
<div data-t="when-wrap">
  <when {tablet}>
    <div data-t="wide" style({ box: size({ w: full }) })>wide</div>
    {else => <div data-t="narrow" style({ box: size({ w: full }) })>narrow</div>}
  </when>
</div>
