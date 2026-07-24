<!--
  <error-state> — L1 language element (element mode). The "something broke"
  sibling of <empty-state>. Behavior rides use:recoverySurface.
  Bare tag lowers to <div class=sail-error-state>.

  Paint (one path): layout + host sx + child sx + CVA + use:recoverySurface.
  Zero contracts PE.
-->

<scope>
  props: {
    variant: {
      default: 'text: center border border: error-state-border rounded: lg bg: error-state shadow: xs',
      inline: 'text: start border border: tone-destructive-border rounded: md bg: tone-destructive-bg shadow: none py: 3 px: 4',
    }
    size: {
      md: 'py: 8 px: 4',
      sm: 'py: 7',
      lg: 'py: 8 px: 6',
    }
    code = ''
  }
</scope>

<column
  align="stretch"
  justify="center"
  gap="3"
  use:recoverySurface
  role="alert"
  sx="font-family: body color: error-state-foreground text: center border border: error-state-border rounded: lg bg: error-state shadow: xs py: 8 px: 4 group forced-colors:border: CanvasText"
>
  <row
    inline
    nowrap
    align="center"
    justify="center"
    class="icon"
    aria-hidden="true"
    sx="shrink: 0 self: center rounded: lg bg: tone-destructive-bg color: destructive size: control-h-xl outline: 1 outline: solid outline: tone-destructive-ring outline-offset: ring-inset forced-colors:bg: Canvas forced-colors:color: CanvasText"
  >
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" sx="size: control-h-2xs">
      <circle cx="12" cy="12" r="9"></circle>
      <path d="M12 8v5"></path>
      <path d="M12 16h.01"></path>
    </svg>
  </row>
  <column align="stretch" gap="1" class="copy" sx="min-w: 0 self: stretch grow: 1">
    <div class="title" sx="font-family: heading font-weight: semibold color: foreground leading: tight font-size: md">
      <slot name="title">Something went wrong</slot>
    </div>
    <div class="desc" sx="leading: loose color: muted-foreground font-size: sm max-w: panel-ch-w-md empty:hidden">
      <slot></slot>
    </div>
    <div class="code" sx="font-family: mono font-size: xs color: muted-foreground bg: muted border border: border-subtle rounded: sm py: 1 px: 2 mt: 1 empty:hidden"></div>
    <button
      class="details-toggle"
      type="button"
      anatomy
      aria-expanded="false"
      sx="hidden group-state-has-details:inline-block mt: 1 py: 1 px: 2 border: 0 bg: transparent color: muted-foreground font-family: body font-size: xs rounded: sm cursor: pointer hover:color: foreground hover:bg: muted focus:outline: solid focus:outline: 2 focus:outline: ring focus:outline-offset: 2"
    >Show details</button>
    <div class="details" sx="hidden group-state-open:block mt: 2 py: 2 px: 3 bg: tone-destructive-bg rounded: sm font-family: mono font-size: xs color: muted-foreground whitespace: pre-wrap break: words text: start max-w: panel-ch-w-lg">
      <slot name="details"></slot>
    </div>
  </column>
  <row align="center" justify="center" gap="2" class="actions" sx="self: center empty:hidden">
    <slot name="action"></slot>
  </row>
</column>
