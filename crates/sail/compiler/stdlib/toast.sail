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
<row align="start" gap="3" use:autoDismiss use:dismissControl role="status" aria-live="polite" sx="font-body text-sm text-foreground bg-background rounded-md shadow-md py-3 px-4 border border-border border-s-3 border-s-tone-fg pointer-events-auto">
  <slot></slot>
  <button data-toast-close type="button" anatomy aria-label="Dismiss" sx="ms-auto shrink-0 grow-0 text-muted-foreground bg-transparent border-0 rounded-sm cursor-pointer hover:text-foreground hover:bg-muted">✕</button>
</row>
