<!--
  <spinner> — L1 loading indicator. Host paint: vertical-align on sx.
  Size/tone channels + .ring geometry + --animate-spin stay (child anatomy / fx).
  centered remains attr CSS (display/margin block).
-->

<scope>
  props: {
    size: 'xs' | 'sm' | 'md' | 'lg' = 'md'
    tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' = 'default'
    centered = false
  }
</scope>

<style>
  .sail-spinner {
    /* Do not set --tone-fg on the bare host: skins load after contracts.css and
       would clobber `[tone=…]` channel fills (same specificity). */
    --sz: var(--icon-size-md);
    --bw: var(--control-stroke);
  }
  .sail-spinner:not([tone]),
  .sail-spinner[tone='default'] {
    --tone-fg: var(--muted-foreground);
  }
  .sail-spinner[size='xs'] { --sz: var(--icon-size-2xs); --bw: 1.5px; }
  .sail-spinner[size='sm'] { --sz: var(--icon-size-sm); --bw: 1.5px; }
  .sail-spinner[size='lg'] { --sz: var(--icon-size-xl); --bw: 2.5px; }
  .sail-spinner[centered] {
    display: block;
    margin: var(--space-8) auto;
  }
  .ring {
    display: inline-block;
    width: var(--sz);
    height: var(--sz);
    border: var(--bw) solid var(--tone-ring, var(--border-subtle));
    border-right-color: var(--tone-fg);
    border-radius: var(--shape-pill);
    animation: var(--animate-spin);
  }
</style>

<span class="spin" role="status" aria-label="Loading" sx="align-middle">
  <span class="ring" aria-hidden="true"></span>
</span>
