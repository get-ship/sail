<!--
  <sail-thinking-dots> — three pulsing dots that signal an assistant is
  composing a reply. Pure presentational primitive, no controller.

  Accessibility: the host is a polite live region labelled "Assistant is
  thinking" so screen readers announce the pending state once without
  spamming. The dots themselves are decorative (aria-hidden).

  The root span is the inline box (the indicator sits in a line of text);
  the track row inside it owns the dot rhythm. The track carries `tone` +
  `size` as a `data-state` token list, and each dot reads it through
  `group-state-<value>:` sx. The pulse is `motion:loop` with a 150ms
  stagger step — the dots arrive together, so the step lands as a per-dot
  phase offset (dot 1: 0ms, dot 2: 150ms, dot 3: 300ms).

  Props
  ─────────────────────────────────────────────────────────────────────
  label   (string)  Overrides the announced label.
  size    (union)   sm | md | lg — scales the dots.
  tone    (union)   Colors the dots; defaults to muted-foreground.
  ─────────────────────────────────────────────────────────────────────
-->

<scope>
  props: {
    label = 'Assistant is thinking'
    size: 'md' | 'sm' | 'lg' = 'md'
    tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'default'
  }
</scope>

<span
  role="status"
  aria-live="polite"
  aria-label={label()}
  sx={style}
>
  <row
    gap={size() === 'lg' ? '2' : '1'}
    align="center"
    data-state={tone() + ' ' + size()}
    sx={style.group()}
  >
    <span
      aria-hidden="true"
      motion:loop={{ translate-y: 'bounce', opacity: 'bounce', stagger: '150ms' }}
      sx={style.radius(full).w(2).h(2).bg(muted-foreground).when({ groupStateSm: style.w(1_25).h(1_25), groupStateLg: style.w(2_5).h(2_5), groupStatePrimary: style.bg(primary), groupStateInfo: style.bg(info), groupStateSuccess: style.bg(success), groupStateWarning: style.bg(warning), groupStateDestructive: style.bg(destructive), groupStateAccent: style.bg(accent) })}
    ></span>
    <span
      aria-hidden="true"
      motion:loop={{ translate-y: 'bounce', opacity: 'bounce', stagger: '150ms' }}
      sx={style.radius(full).w(2).h(2).bg(muted-foreground).when({ groupStateSm: style.w(1_25).h(1_25), groupStateLg: style.w(2_5).h(2_5), groupStatePrimary: style.bg(primary), groupStateInfo: style.bg(info), groupStateSuccess: style.bg(success), groupStateWarning: style.bg(warning), groupStateDestructive: style.bg(destructive), groupStateAccent: style.bg(accent) })}
    ></span>
    <span
      aria-hidden="true"
      motion:loop={{ translate-y: 'bounce', opacity: 'bounce', stagger: '150ms' }}
      sx={style.radius(full).w(2).h(2).bg(muted-foreground).when({ groupStateSm: style.w(1_25).h(1_25), groupStateLg: style.w(2_5).h(2_5), groupStatePrimary: style.bg(primary), groupStateInfo: style.bg(info), groupStateSuccess: style.bg(success), groupStateWarning: style.bg(warning), groupStateDestructive: style.bg(destructive), groupStateAccent: style.bg(accent) })}
    ></span>
  </row>
</span>
</span>
