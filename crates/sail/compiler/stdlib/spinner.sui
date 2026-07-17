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
    --tone-fg: var(--muted-foreground);
    --sz: var(--icon-size-md);
    --bw: var(--control-stroke);
  }
  .sail-spinner[tone='primary']     { --tone-fg: var(--primary);     --tone-ring: var(--tone-primary-ring); }
  .sail-spinner[tone='info']        { --tone-fg: var(--info);        --tone-ring: var(--tone-info-ring); }
  .sail-spinner[tone='success']     { --tone-fg: var(--success);     --tone-ring: var(--tone-success-ring); }
  .sail-spinner[tone='warning']     { --tone-fg: var(--warning);     --tone-ring: var(--tone-warning-ring); }
  .sail-spinner[tone='destructive'] { --tone-fg: var(--destructive); --tone-ring: var(--tone-destructive-ring); }
  .sail-spinner[tone='accent']      { --tone-fg: var(--accent);      --tone-ring: var(--tone-accent-ring); }
  .sail-spinner[size='xs'] { --sz: 10px; --bw: 1.5px; }
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
    border-radius: 50%;
    animation: sail-spin 0.7s linear infinite;
  }
  @keyframes sail-spin { to { transform: rotate(360deg); } }
  @media (prefers-reduced-motion: reduce) { .ring { animation-duration: 1.5s; } }
</style>

<span class="spin" role="status" aria-label="Loading">
  <span class="ring" aria-hidden="true"></span>
</span>
