<!--
  <card> — L1 language element (element mode). Generic surface: header/body/footer
  composition, tone-tinted border, accent bar, status rail, interactive lift.

  Paint:
    host sx — bg/fg from component.card tokens (cssNames card, card-foreground),
              backdrop-(--card-backdrop-filter), shadow card-rest, padding on host
    structure — header/footer slots; empty:hidden collapses unfilled regions
    CVA-in-props — variant + padding; compound for interactive / fill / rails / accent-bar
-->

<scope>
  props: {
    tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'default'
    // first key = default fill from component.card.bg (--card)
    variant: {
      default: 'bg: card',
      outline: 'bg: transparent',
    }
    // first key = default = PE md
    padding: {
      md: 'py: 4 px: 5',
      none: 'p: 0',
      sm: 'py: 2 px: 3',
      lg: 'py: 5 px: 6',
    }
    interactive = false
    'accent-bar': 'off' | 'hover' | 'static' = 'off'
    'status-rail' = false
    fill = false
  }
  compound: [
    { when: { interactive: true }, sx: 'cursor: pointer tap-highlight: none hover:border: border-strong hover:shadow: hover hover:-translate-y: px active:shadow: card-rest active:translate-y: 0 active:duration: instant motion-reduce:translate-y: 0 focus:outline: solid focus:outline: 2 focus:outline: ring focus:outline-offset: 2 forced-colors:focus:outline: CanvasText' },
    { when: { fill: true }, sx: 'bg: tone-bg border: tone-border' },
    { when: { tone: 'primary' }, sx: 'border: tone-border' },
    { when: { tone: 'info' }, sx: 'border: tone-border' },
    { when: { tone: 'success' }, sx: 'border: tone-border' },
    { when: { tone: 'warning' }, sx: 'border: tone-border' },
    { when: { tone: 'destructive' }, sx: 'border: tone-border' },
    { when: { tone: 'accent' }, sx: 'border: tone-border' },
    { when: { tone: 'muted' }, sx: 'border: tone-border' },
    // Untoned/default rail = border-strong (covers attr-absent + tone=default).
    // Branded tones recolor to tone-fg. Pad-adjust: space-N − space-1 for 3px rail
    // (md PE default ps-4; sm/lg/none compounds override).
    { when: { 'status-rail': true }, sx: 'border-s: 3 border-s: border-strong ps: 4' },
    { when: { 'status-rail': true, padding: 'sm' }, sx: 'ps: 2' },
    { when: { 'status-rail': true, padding: 'md' }, sx: 'ps: 4' },
    { when: { 'status-rail': true, padding: 'lg' }, sx: 'ps: 5' },
    { when: { 'status-rail': true, padding: 'none' }, sx: 'ps: 0' },
    { when: { 'status-rail': true, tone: 'primary' }, sx: 'border-s: tone-fg' },
    { when: { 'status-rail': true, tone: 'info' }, sx: 'border-s: tone-fg' },
    { when: { 'status-rail': true, tone: 'success' }, sx: 'border-s: tone-fg' },
    { when: { 'status-rail': true, tone: 'warning' }, sx: 'border-s: tone-fg' },
    { when: { 'status-rail': true, tone: 'destructive' }, sx: 'border-s: tone-fg' },
    { when: { 'status-rail': true, tone: 'accent' }, sx: 'border-s: tone-fg' },
    { when: { 'status-rail': true, tone: 'muted' }, sx: 'border-s: tone-fg' },
    { when: { 'accent-bar': 'static' }, sx: 'before:content: empty before:absolute before:inset-x: 0 before:top: 0 before:h: half before:bg-gradient-to-r before:from-primary before:to-accent before:opacity: 100 before:rounded-t: lg before:pointer-events: none before:transition: opacity before:duration: medium before:ease: out' },
    { when: { 'accent-bar': 'hover' }, sx: 'before:content: empty before:absolute before:inset-x: 0 before:top: 0 before:h: half before:bg-gradient-to-r before:from-primary before:to-accent before:opacity: 0 hover:before:opacity: 100 before:rounded-t: lg before:pointer-events: none before:transition: opacity before:duration: medium before:ease: out' },
  ]
</scope>

<div sx="relative isolate overflow: clip rounded: lg border border: border color: card-foreground shadow: card-rest transition: colors duration: fast ease: out bg: card py: 4 px: 5 backdrop-(--card-backdrop-filter) border-image-(--corner-image)">
  <div class="header" sx="empty:hidden block -mt: 4 -mx: 5 mb: 3 px: 5 py: 4 border-b border: border-subtle font-family: heading font-weight: (--heading-weight)">
    <slot name="header"></slot>
  </div>
  <slot></slot>
  <div class="footer" sx="empty:hidden block mt: 3 -mx: 5 -mb: 4 px: 5 py: 4 border-t border: border-subtle">
    <slot name="footer"></slot>
  </div>
</div>
