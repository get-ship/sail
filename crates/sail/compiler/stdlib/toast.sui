<!--
  <toast tone="success" duration="5000"> — L1 toast (element mode). use:toast
  auto-dismisses after `duration` ms (0 = sticky), PAUSES while hovered/focused so
  a reader is never cut off, wires [data-toast-close], and emits
  `sail-toast-dismiss` before removing itself. Bare tag lowers to
  <div class="sail-toast" role="status" aria-live="polite">.

    <toast tone="destructive" duration="0">
      Couldn't save. <button data-toast-close>Dismiss</button>
    </toast>
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
    font-family: var(--font-body);
    font-size: var(--text-sm);
    color: var(--foreground);
    background: var(--background);
    border: 1px solid var(--tone-border, var(--border));
    border-inline-start: 3px solid var(--tone-fg, var(--border));
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-md);
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

<div use:toast><slot></slot></div>
