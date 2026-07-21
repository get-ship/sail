<!--
  <alert> — L1 language element (element mode). Inline status banner: tone
  stripe, optional icon, title, body, actions, dismiss control.

  Behavior: use:liveRegion + use:dismissControl (capability mix-ins).

  Paint (one path):
    layout keywords  — host <row nowrap align=start gap=3> owns ico | content |
                       close (layout system; no free grid-template PE)
    host sx=         — surface + tone stripe via before: (TW-shaped pseudo);
                       PE default stripe is muted (attr-absent); CVA brands;
                       forced-colors:border/before CanvasText
    child sx=        — empty:hidden on ico/title/actions; dismiss close via
                       host group + group-dismissible:
    CVA-in-props     — tone stripe (first key = default = PE muted)
    global [tone]    — stripe/icon channel reads for branded tones
  No residual alert matrix.
-->

<scope>
  props: {
    // first key = default = PE muted stripe (attr-absent host sx matches)
    tone: {
      default: 'before:bg-muted-foreground',
      muted: 'before:bg-muted-foreground',
      info: 'before:bg-tone',
      success: 'before:bg-tone',
      warning: 'before:bg-tone',
      destructive: 'before:bg-tone',
    }
    dismissible = false
  }
</scope>

<row
  nowrap
  align="start"
  gap="3"
  use:liveRegion
  use:dismissControl
  role="status"
  aria-live="polite"
  aria-atomic="true"
  sx="group relative overflow-hidden p-3 border border-border rounded-md bg-surface-1 text-foreground font-body text-base leading-normal shadow-xs before:content-empty before:absolute before:left-0 before:top-0 before:bottom-0 before:w-1 before:bg-muted-foreground forced-colors:border-CanvasText forced-colors:before:bg-CanvasText"
>
  <row inline nowrap align="center" justify="center" class="ico" sx="shrink-0 size-control-h-xs rounded-md bg-tone-bg text-tone-fg empty:hidden">
    <slot name="icon"></slot>
  </row>
  <column gap="1" class="content" sx="min-w-0 grow-1">
    <div class="title" sx="font-semibold empty:hidden">
      <slot name="title"></slot>
    </div>
    <div class="body" sx="text-muted-foreground">
      <slot></slot>
    </div>
    <row align="center" gap="2" class="actions" sx="mt-2 empty:hidden">
      <slot name="actions"></slot>
    </row>
  </column>
  <button
    class="close"
    type="button"
    anatomy
    aria-label="Dismiss"
    sx="hidden group-dismissible:inline-block size-control-h-xs shrink-0 self-start p-0 border-0 rounded-sm bg-transparent text-muted-foreground cursor-pointer transition-colors duration-fast ease-out hover:bg-muted hover:text-foreground focus:outline-solid focus:outline-2 focus:outline-ring focus:outline-offset-2"
  >
    <svg viewBox="0 0 10 10" aria-hidden="true" sx="block size-icon-size-sm stroke-current fill-none" stroke-width="1.75" stroke-linecap="round">
      <path d="M2 2 L8 8 M8 2 L2 8"></path>
    </svg>
  </button>
</row>
