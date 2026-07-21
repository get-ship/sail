<!--
  <separator> — L1 language element (element mode). Native <hr>.

  Host paint (via sx): display/width/height/margin/background (solid horizontal
  default) + border-0/p-0.
  Residue (author <style>): --separator-line channel writes, tone overrides that
  set the channel, dashed/orientation matrices, dashed border-top/left, vertical
  geometry (inline-block, align-self, size/margin/bg overrides).
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

<hr sx="block w-full h-px m-2 border-0 p-0 bg-(--separator-line)">
