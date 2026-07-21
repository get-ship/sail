<!--
  <separator> — L1 language element (element mode). Native <hr>.

  Host paint: display/width/height/margin/background (for solid horizontal default)
  via sx. Residue: --separator-line channel, border:0/p-0, orientation/variant
  matrices, dashed border-top/left, vertical inline-block + align-self, tone
  overrides.
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
    --separator-line: var(--border-subtle);
    border: 0;
    padding: 0;
  }
  .sail-separator[tone='primary'],
  .sail-separator[tone='destructive'] {
    --separator-line: var(--tone-border);
  }

  .sail-separator:not([orientation='vertical'])[variant='dashed'] {
    background: transparent;
    border-top: 1px dashed var(--separator-line);
    height: 0;
  }

  .sail-separator[orientation='vertical'] {
    display: inline-block;
    align-self: stretch;
    width: 1px;
    height: 100%;
    margin: 0 var(--space-2);
    background: var(--separator-line);
  }
  .sail-separator[orientation='vertical'][variant='dashed'] {
    background: transparent;
    border-left: 1px dashed var(--separator-line);
    width: 0;
  }
</style>

<hr sx="block w-full h-px m-2 bg-(--separator-line)">
