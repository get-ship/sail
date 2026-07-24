<!--
  <chip> — L1 language element (element mode). An interactive filter/selection
  pill: the element IS the interactive surface (role=button + keyboard). A
  leading slot + the default label slot, an optional remove button (× shown
  under [removable]), and selected/disabled states. Bare tag lowers to a
  layout row + capability mix-ins.

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

  Paint (one path — same as L2+/app code):
    layout keywords  — inline row, gap, align
    host sx=         — base type/border/cursor + PE defaults (solid, size md)
                       for attr-absent (defaults are not stamped on structure)
    CVA-in-props     — variant + size maps (first key = default = PE)
    compound         — selected+muted combination
    contracts        — shared removable anatomy dismiss control
  Variance = CVA-in-props axes + optional compound only. tone/size/variant are
  ordinary props (axes or unions), not language channels.
  No residual chip matrix. No L1 controller.
-->

<scope>
  props: {
    tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'default'
    // first key = default = PE solid
    variant: {
      solid: 'bg: tone color: tone-contrast border: tone shadow: xs',
      // shadow-none: host PE solid base carries shadow-xs — non-solid must clear it
      soft: 'bg: tone-bg color: tone-fg border: tone-border shadow: none hover:bg: muted hover:border: tone',
      outline: 'bg: transparent color: tone-fg border: tone shadow: none hover:bg: tone-bg',
      ghost: 'bg: transparent color: tone-fg border: transparent shadow: none hover:bg: tone-bg',
    }
    // first key = default = PE md
    size: {
      md: 'min-h: control-h-xs px: 2_5 font-size: sm',
      sm: 'min-h: control-h-2xs px: 2 font-size: xs',
      lg: 'min-h: control-h-md px: 3 font-size: base',
    }
    selected = false
    removable = false
    disabled = false
  }
  compound: [
    { when: { selected: true, tone: 'muted' }, sx: 'bg: primary color: primary-foreground border: primary' },
  ]
</scope>

<row
  inline
  nowrap
  align="center"
  gap="2"
  use:pressable
  use:pressed
  use:dismissible
  sx="font-family: body font-weight: medium leading: none align: middle whitespace: nowrap cursor: pointer select: none tap-highlight: none rounded: full border min-h: control-h-xs px: 2_5 font-size: sm bg: tone color: tone-contrast border: tone shadow: xs transition: colors duration: fast ease: out pressed:bg: tone pressed:color: tone-contrast pressed:border: tone pressed:font-weight: semibold pressed:shadow: xs disabled:cursor: not-allowed disabled:bg: muted disabled:border: border-subtle disabled:color: muted-foreground disabled:saturate: 65 disabled:shadow: none focus:outline: solid focus:outline: 2 focus:outline: ring focus:outline-offset: 2"
>
  <span class="leading" sx="contents">
    <slot name="leading"></slot>
  </span>
  <slot></slot>
  <button
    class="remove"
    type="button"
    anatomy
    aria-label="Remove"
    tabindex="-1"
    sx="size: icon-size-md shrink: 0 p: 0 border: 0 bg: transparent color: currentColor cursor: pointer rounded: full transition: colors duration: fast ease: out"
  >
    <svg viewBox="0 0 10 10" aria-hidden="true" sx="block size: full stroke: currentColor fill: none" stroke-width="1.75" stroke-linecap="round">
      <path d="M2 2 L8 8 M8 2 L2 8"></path>
    </svg>
  </button>
</row>
