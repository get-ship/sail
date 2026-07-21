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
    host sx=         — base type/border/cursor/pressed/disabled/focus
    CVA-in-props     — variant + size maps → attr-keyed rules via sx atoms
    global [tone]    — channel reads only (contracts write --tone*)
    contracts        — shared removable anatomy dismiss control
  No residual chip matrix. No L1 controller.
-->

<scope>
  props: {
    tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'default'
    variant: {
      solid: 'bg-tone text-tone-contrast border-tone shadow-xs',
      soft: 'bg-tone-bg text-tone-fg border-tone-border hover:bg-muted hover:border-tone',
      outline: 'bg-transparent text-tone-fg border-tone hover:bg-tone-bg',
      ghost: 'bg-transparent text-tone-fg border-transparent hover:bg-tone-bg',
    }
    size: {
      sm: 'min-h-control-h-2xs px-2 text-xs',
      md: 'min-h-control-h-xs px-2_5 text-sm',
      lg: 'min-h-control-h-md px-3 text-base',
    }
    selected = false
    removable = false
    disabled = false
  }
  compound: [
    { when: { selected: true, tone: 'muted' }, sx: 'bg-primary text-primary-foreground border-primary' },
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
  sx="font-body font-medium leading-none align-middle whitespace-nowrap cursor-pointer select-none tap-highlight-none rounded-full border border-transparent text-tone-fg transition-colors duration-fast ease-out pressed:bg-tone pressed:text-tone-contrast pressed:border-tone pressed:font-semibold pressed:shadow-xs disabled:cursor-not-allowed disabled:bg-muted disabled:border-border-subtle disabled:text-muted-foreground disabled:saturate-65 disabled:shadow-none focus:outline-solid focus:outline-2 focus:outline-ring focus:outline-offset-2 forced-colors:border-canvastext forced-colors:text-canvastext"
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
    sx="size-icon-size-md shrink-0 p-0 border-0 bg-transparent text-current cursor-pointer rounded-full transition-colors duration-fast ease-out"
  >
    <svg viewBox="0 0 10 10" aria-hidden="true" sx="block size-full stroke-current fill-none" stroke-width="1.75" stroke-linecap="round">
      <path d="M2 2 L8 8 M8 2 L2 8"></path>
    </svg>
  </button>
</row>
