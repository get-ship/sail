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

<hr sx="block w-full h-px m-2 border-0 p-0 bg-(--separator-line)">
