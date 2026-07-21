<!--
  <toast> — L1 toast: auto-dismiss + close control via stacked use: behaviors
    (use:autoDismiss, use:dismissControl). tone + duration attrs; PE role/aria-live.

  Paint: layout row + type/surface/pad/border on host sx.
  Contracts: 3px tone stripe + close-button anatomy (injected control).
-->

<scope>
  props: {
    tone = 'default'
    duration = ''
  }
</scope>

<row
  align="start"
  gap="3"
  use:autoDismiss
  use:dismissControl
  role="status"
  aria-live="polite"
  sx="font-body text-sm text-foreground bg-background rounded-md shadow-md py-3 px-4 border border-border"
>
  <slot></slot>
</row>
