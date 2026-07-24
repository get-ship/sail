<!--
  <toast> — L1 toast: auto-dismiss + close via use:autoDismiss/use:dismissControl.
  Paint + start stripe + close control on structure sx. Zero contracts PE.
-->
<scope>
  props: {
    tone = 'default'
    duration = ''
  }
</scope>
<row align="start" gap="3" use:autoDismiss use:dismissControl role="status" aria-live="polite" sx="font-family: body font-size: sm color: toast-foreground bg: toast rounded: md shadow: md py: 3 px: 4 border border: toast-border border-s: 3 border-s: tone-fg pointer-events: auto">
  <slot></slot>
  <button data-toast-close type="button" anatomy aria-label="Dismiss" sx="ms: auto shrink: 0 grow: 0 color: muted-foreground bg: transparent border: 0 rounded: sm cursor: pointer hover:color: foreground hover:bg: muted">✕</button>
</row>
