<!--
  <error-state> — L1 language element (element mode). The "something broke"
  sibling of <empty-state>. Behavior rides use:recoverySurface.
  Bare tag lowers to <div class=sail-error-state>.

  Paint (one path): layout + host sx + structure sx + CVA + channels: +
  use:recoverySurface. Zero contracts PE.
-->

<scope>
  props: {
    variant: {
      default: 'text-center border border-border rounded-lg bg-tone-muted-bg shadow-xs',
      inline: 'text-start border border-tone-destructive-border rounded-md bg-tone-destructive-bg shadow-none py-3 px-4',
    }
    size: {
      md: 'py-8 px-4',
      sm: 'py-7',
      lg: 'py-8 px-6',
    }
    code = ''
  }
  channels: {
    default: {
      'es-icon': 'control-h-2xs',
      'es-tile': 'control-h-xl',
      'es-icon-bg': 'tone-destructive-bg',
      'es-icon-outline': 'tone-destructive-ring',
    }
    size: {
      sm: {
        gap: '2',
        'es-icon': 'icon-size-md',
        'es-tile': 'space-9',
      }
      lg: {
        gap: '4',
        'es-icon': 'icon-size-3xl',
        'es-tile': 'control-h-xl',
      }
    }
    variant: {
      inline: {
        direction: 'row',
        align: 'center',
        justify: 'start',
        'es-tile': 'es-icon',
        'es-icon-bg': 'transparent',
        'es-icon-outline': 'transparent',
      }
    }
    '[hidden]': { display: 'none' }
  }
</scope>

<column
  align="stretch"
  justify="center"
  gap="3"
  use:recoverySurface
  role="alert"
  sx="font-body text-muted-foreground text-center border border-border rounded-lg bg-tone-muted-bg shadow-xs py-8 px-4 group forced-colors:border-CanvasText"
>
  <row
    inline
    nowrap
    align="center"
    justify="center"
    class="icon"
    aria-hidden="true"
    sx="shrink-0 self-center rounded-lg bg-(--es-icon-bg) text-destructive size-(--es-tile) outline-1 outline-solid outline-(--es-icon-outline) outline-offset-ring-inset forced-colors:bg-Canvas forced-colors:text-CanvasText"
  >
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" sx="size-(--es-icon)">
      <circle cx="12" cy="12" r="9"></circle>
      <path d="M12 8v5"></path>
      <path d="M12 16h.01"></path>
    </svg>
  </row>
  <column align="stretch" gap="1" class="copy" sx="min-w-0 self-stretch grow-1">
    <div class="title" sx="font-heading font-semibold text-foreground leading-tight text-md">
      <slot name="title">Something went wrong</slot>
    </div>
    <div class="desc" sx="leading-loose text-muted-foreground text-sm max-w-panel-ch-w-md empty:hidden">
      <slot></slot>
    </div>
    <div class="code" sx="font-mono text-xs text-muted-foreground bg-muted border border-border-subtle rounded-sm py-1 px-2 mt-1 empty:hidden"></div>
    <button
      class="details-toggle"
      type="button"
      anatomy
      aria-expanded="false"
      sx="hidden group-state-has-details:inline-block mt-1 py-1 px-2 border-0 bg-transparent text-muted-foreground font-body text-xs rounded-sm cursor-pointer hover:text-foreground hover:bg-muted focus:outline-solid focus:outline-2 focus:outline-ring focus:outline-offset-2"
    >Show details</button>
    <div class="details" sx="hidden group-state-open:block mt-2 py-2 px-3 bg-tone-destructive-bg rounded-sm font-mono text-xs text-muted-foreground whitespace-pre-wrap break-words text-start max-w-panel-ch-w-lg">
      <slot name="details"></slot>
    </div>
  </column>
  <row align="center" justify="center" gap="2" class="actions" sx="self-center empty:hidden">
    <slot name="action"></slot>
  </row>
</column>
