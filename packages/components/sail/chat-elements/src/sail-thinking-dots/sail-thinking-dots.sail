<!--
  <sail-thinking-dots> — three bouncing dots (assistant composing). Presentational.
  Polite live region; dots decorative. size/tone via data-state group paint;
  pulse is .loop with 150ms stagger phase per sibling.

  Use explicit px travel on translate-y only — not opacity bounce (0→1 pins
  rest at 0 and reads as "not animating"). -10px on an 8px dot is an obvious
  hop; loop+alternate is the bounce rhythm.
-->

<scope>
  props: {
    label = 'Assistant is thinking'
    size: 'md' | 'sm' | 'lg' = 'md'
    tone: 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted'
  }

  style track = { group() }
  style dot = { size({ w: 2, h: 2 }).radius(full).bg(muted-foreground), animate: { loop: play(slide-up) }, when: { groupStateSm: scale(75), groupStateLg: scale(125), groupStatePrimary: bg(primary), groupStateInfo: bg(info), groupStateSuccess: bg(success), groupStateWarning: bg(warning), groupStateDestructive: bg(destructive), groupStateAccent: bg(accent) } }
</scope>

<span
  role="status"
  aria-live="polite"
  aria-label={label()}
>
  <row
    gap={size() === 'lg' ? '2' : '1'}
    align="center"
    data-state={(tone() ?? '') + ' ' + size()}
    style(track)
  >
    <span aria-hidden="true" style(dot)></span>
    <span aria-hidden="true" style(dot)></span>
    <span aria-hidden="true" style(dot)></span>
  </row>
</span>
