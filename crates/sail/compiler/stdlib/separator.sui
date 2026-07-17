<!--
  <separator> — L1 language element (element mode). Native <hr> carries the
  implicit separator role. There is no host wrapper in L1 — the root IS the
  <hr> — so the skin keys everything on :host (rewritten to .sail-separator).

  Faithful to the feat/next sail-separator baseline minus the `decorative`
  prop (dynamic role swapping is per-instance behavior — use-site attrs
  merge onto the root, so `<separator role="none">` covers it).
-->

<scope>
  props: {
    orientation: 'horizontal' | 'vertical' = 'horizontal'
    tone: 'default' | 'muted' | 'primary' | 'destructive' = 'default'
    variant: 'solid' | 'dashed' = 'solid'
  }
</scope>

<style>
  .sail-separator {
    --tone-bg: var(--border-subtle);
    border: 0;
    margin: 0;
    padding: 0;
    flex: 0 0 auto;
  }
  .sail-separator[tone='primary']     { --tone-bg: var(--tone-primary-border); }
  .sail-separator[tone='destructive'] { --tone-bg: var(--tone-destructive-border); }

  .sail-separator:not([orientation='vertical']) {
    display: block;
    width: 100%;
    height: 1px;
    margin: var(--space-2) 0;
    background: var(--tone-bg);
  }
  .sail-separator:not([orientation='vertical'])[variant='dashed'] {
    background: transparent;
    border-top: 1px dashed var(--tone-bg);
    height: 0;
  }
  .sail-separator[orientation='vertical'] {
    display: inline-block;
    align-self: stretch;
    width: 1px;
    height: 100%;
    margin: 0 var(--space-2);
    background: var(--tone-bg);
  }
  .sail-separator[orientation='vertical'][variant='dashed'] {
    background: transparent;
    border-left: 1px dashed var(--tone-bg);
    width: 0;
  }
</style>

<hr>
