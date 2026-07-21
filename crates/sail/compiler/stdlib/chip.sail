<!--
  <chip> — L1 language element (element mode). An interactive filter/selection
  pill: the element IS the interactive surface (role=button + keyboard). A
  leading slot + the default label slot, an optional remove button (× shown
  under [removable]), and selected/disabled states. Bare tag lowers to
  <span class="sail-chip" use:pressable use:pressed use:dismissible>.

  Behavior rides stacked use: behaviors, each a reusable mix-in the normal
  use: pipeline lowers to runAction at the use site:
    use:pressable   — role=button + tabindex, aria-disabled from [disabled],
                      Enter/Space activate
    use:pressed     — aria-pressed reflects [selected]
    use:dismissible — under [removable]: Backspace/Delete and the remove
                      button's click fire sail-remove
  PE: without JS the static structure + skin still ship; with JS the
  behaviors enhance ARIA/keyboard through the standard use: machinery — no
  element-private runtime, no discovery walk.

  Faithful to the sail-chip twin. Two shadow-isms dissolve at L1: the host/.chip
  split collapses onto the single root span; the {#if removable} remove button
  becomes an always-present button gated by CSS ([removable]). The rich 10-channel
  tone vocabulary (solid/soft/outline/ghost + hover + contrast) is owned locally.
-->

<scope>
  props: {
    tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'default'
    variant: 'solid' | 'soft' | 'outline' | 'ghost' = 'solid'
    size: 'sm' | 'md' | 'lg' = 'md'
    selected = false
    removable = false
    disabled = false
  }
</scope>

<span use:pressable use:pressed use:dismissible>
  <span class="leading"><slot name="leading"></slot></span>
  <slot></slot>
  <button class="remove" type="button" anatomy aria-label="Remove" tabindex="-1">
    <svg viewBox="0 0 10 10" aria-hidden="true"><path d="M2 2 L8 8 M8 2 L2 8"></path></svg>
  </button>
</span>
