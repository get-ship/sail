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
    border: 0;
    margin: 0;
    padding: 0;
    flex: 0 0 auto;
  }

  .sail-separator:not([orientation='vertical']) {
    display: block;
    width: 100%;
    height: 1px;
    margin: var(--space-2) 0;
    background: var(--border-subtle);
  }
  .sail-separator[tone='primary']:not([orientation='vertical']) {
    background: var(--tone-border);
  }
  .sail-separator[tone='destructive']:not([orientation='vertical']) {
    background: var(--tone-border);
  }

  .sail-separator:not([orientation='vertical'])[variant='dashed'] {
    background: transparent;
    border-top: 1px dashed var(--border-subtle);
    height: 0;
  }
  .sail-separator[tone='primary']:not([orientation='vertical'])[variant='dashed'] {
    border-top-color: var(--tone-border);
  }
  .sail-separator[tone='destructive']:not([orientation='vertical'])[variant='dashed'] {
    border-top-color: var(--tone-border);
  }

  .sail-separator[orientation='vertical'] {
    display: inline-block;
    align-self: stretch;
    width: 1px;
    height: 100%;
    margin: 0 var(--space-2);
    background: var(--border-subtle);
  }
  .sail-separator[tone='primary'][orientation='vertical'] {
    background: var(--tone-border);
  }
  .sail-separator[tone='destructive'][orientation='vertical'] {
    background: var(--tone-border);
  }

  .sail-separator[orientation='vertical'][variant='dashed'] {
    background: transparent;
    border-left: 1px dashed var(--border-subtle);
    width: 0;
  }
  .sail-separator[tone='primary'][orientation='vertical'][variant='dashed'] {
    border-left-color: var(--tone-border);
  }
  .sail-separator[tone='destructive'][orientation='vertical'][variant='dashed'] {
    border-left-color: var(--tone-border);
  }
</style>

<hr>
