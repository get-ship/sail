<!--
  <sail-thinking-dots> — three pulsing dots that signal an assistant is
  composing a reply. Pure presentational primitive, no controller.

  Accessibility: the host is a polite live region labelled "Assistant is
  thinking" so screen readers announce the pending state once without
  spamming. The dots themselves are decorative (aria-hidden).

  The root span is the inline box (the indicator sits in a line of text);
  the track row inside it owns the dot rhythm. The track carries `tone` +
  `size` as a `data-state` token list, and each dot reads it through
  `group-state-<value>:` sx. The pulse is `.loop({…})` with a 150ms
  stagger step — the dots arrive together, so the step lands as a per-dot
  phase offset (dot 1: 0ms, dot 2: 150ms, dot 3: 300ms).

  Props
  ─────────────────────────────────────────────────────────────────────
  label   (string)  Overrides the announced label.
  size    (union)   sm | md | lg — scales the dots.
  tone    (union)   Colors the dots; omit for muted-foreground rest.
  ─────────────────────────────────────────────────────────────────────
-->

<scope>
  props: {
    label = 'Assistant is thinking'
    size: 'md' | 'sm' | 'lg' = 'md'
    tone: 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted'
  }
</scope>

<span
  role="status"
  aria-live="polite"
  aria-label={label()}>
  <row
    gap={size() === 'lg' ? '2' : '1'}
    align="center"
    data-state={(tone() ?? '') + ' ' + size()} style({ paint: group() })>
    <span
aria-hidden="true" style({ box: w(2).h(2), paint: radius(full).bg(muted-foreground), animate: loop({ translate-y: 'bounce', opacity: 'bounce', stagger: '150ms' }), when: { groupStateSm: { paint: scale(75) }, groupStateLg: { paint: scale(125) }, groupStatePrimary: { paint: bg(primary) }, groupStateInfo: { paint: bg(info) }, groupStateSuccess: { paint: bg(success) }, groupStateWarning: { paint: bg(warning) }, groupStateDestructive: { paint: bg(destructive) }, groupStateAccent: { paint: bg(accent) } } })></span>
    <span
aria-hidden="true" style({ box: w(2).h(2), paint: radius(full).bg(muted-foreground), animate: loop({ translate-y: 'bounce', opacity: 'bounce', stagger: '150ms' }), when: { groupStateSm: { paint: scale(75) }, groupStateLg: { paint: scale(125) }, groupStatePrimary: { paint: bg(primary) }, groupStateInfo: { paint: bg(info) }, groupStateSuccess: { paint: bg(success) }, groupStateWarning: { paint: bg(warning) }, groupStateDestructive: { paint: bg(destructive) }, groupStateAccent: { paint: bg(accent) } } })></span>
    <span
aria-hidden="true" style({ box: w(2).h(2), paint: radius(full).bg(muted-foreground), animate: loop({ translate-y: 'bounce', opacity: 'bounce', stagger: '150ms' }), when: { groupStateSm: { paint: scale(75) }, groupStateLg: { paint: scale(125) }, groupStatePrimary: { paint: bg(primary) }, groupStateInfo: { paint: bg(info) }, groupStateSuccess: { paint: bg(success) }, groupStateWarning: { paint: bg(warning) }, groupStateDestructive: { paint: bg(destructive) }, groupStateAccent: { paint: bg(accent) } } })></span>
  </row>
</span>
