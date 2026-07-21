<!--
  <toast> — L1 toast: auto-dismiss + close control via stacked use: behaviors
    (use:autoDismiss, use:dismissControl). tone + duration attrs; PE role/aria-live.

  Host paint: typeface/color/bg/radius/shadow on sx. Flex geometry + 3px tone
  stripe border residue + close-button anatomy stay author CSS.
-->

<scope>
  props: {
    tone = 'default'
    duration = ''
  }
</scope>

<style>
  .sail-toast {
    display: flex;
    align-items: flex-start;
    gap: var(--space-3);
    padding: var(--space-3) var(--space-4);
    border: 1px solid var(--tone-border, var(--border));
    border-inline-start: 3px solid var(--tone-fg, var(--border)); /* residue — 3px tone stripe */
  }
  .sail-toast [data-toast-close] {
    margin-inline-start: auto;
    flex: 0 0 auto;
    color: var(--muted-foreground);
    background: transparent;
    border: 0;
    border-radius: var(--radius-sm);
    cursor: pointer;
  }
  .sail-toast [data-toast-close]:hover { color: var(--foreground); background: var(--muted); }
</style>

<div use:autoDismiss use:dismissControl role="status" aria-live="polite" sx="font-body text-sm text-foreground bg-background rounded-md shadow-md"><slot></slot></div>
