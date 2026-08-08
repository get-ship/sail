<!--
  <sail-thinking-dots> — three pulsing dots (assistant composing). Presentational.
  Polite live region; dots decorative. size/tone via data-state group paint;
  pulse is .loop with 150ms stagger phase per sibling.
-->

<scope>
  props: {
    label = 'Assistant is thinking'
    size: 'md' | 'sm' | 'lg' = 'md'
    tone: 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted'
  }

  style track = { paint: group() }
  style dot = {
    box: w(2).h(2),
    paint: radius(full).bg(muted-foreground),
    animate: loop({ translate-y: 'bounce', opacity: 'bounce', stagger: '150ms' }),
    when: {
      groupStateSm: { paint: scale(75) },
      groupStateLg: { paint: scale(125) },
      groupStatePrimary: { paint: bg(primary) },
      groupStateInfo: { paint: bg(info) },
      groupStateSuccess: { paint: bg(success) },
      groupStateWarning: { paint: bg(warning) },
      groupStateDestructive: { paint: bg(destructive) },
      groupStateAccent: { paint: bg(accent) },
    },
  }
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
