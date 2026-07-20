<!--
  <spinner> — L1 language element (element mode). Bare tag: it lowers away
  at compile time; the skin below routes to the framework layer keyed on
  .sail-spinner; the contract feeds the element manifest.

  Faithful to the feat/next sail-spinner baseline. Per-instance dynamics
  (a custom aria-label) belong to the USE SITE — its attributes merge onto
  the root, overriding the static defaults here.
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
    display: inline-flex;
    vertical-align: middle;
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

<span class="spin" role="status" aria-label="Loading">
  <span class="ring" aria-hidden="true"></span>
</span>
