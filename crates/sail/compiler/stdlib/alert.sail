<!--
  <alert> — L1 language element (element mode). Inline status banner: tone
  stripe, optional icon, title, body, actions, dismiss control.

  Behavior: use:liveRegion + use:dismissControl (capability mix-ins).

  Paint (one path):
    host sx=         — surface, type, padding, border, shadow
    global [tone]    — stripe/icon channel reads; PE muted default in contracts
    contracts        — named unexpressibles: grid template, ::before stripe,
                       :empty anatomy, dismissible close gate
  No residual alert matrix.
-->

<scope>
  props: {
    tone: 'default' | 'info' | 'success' | 'warning' | 'destructive' | 'muted' = 'default'
    dismissible = false
  }
</scope>

<div
  use:liveRegion
  use:dismissControl
  role="status"
  aria-live="polite"
  aria-atomic="true"
  sx="relative overflow-hidden p-3 border border-border rounded-md bg-surface-1 text-foreground font-body text-base leading-normal shadow-xs"
>
  <span class="ico" sx="shrink-0 size-control-h-xs rounded-md bg-tone-bg text-tone-fg">
    <slot name="icon"></slot>
  </span>
  <div class="content" sx="min-w-0">
    <div class="title" sx="font-semibold">
      <slot name="title"></slot>
    </div>
    <div class="body" sx="text-muted-foreground">
      <slot></slot>
    </div>
    <div class="actions" sx="mt-2">
      <slot name="actions"></slot>
    </div>
  </div>
  <button
    class="close"
    type="button"
    anatomy
    aria-label="Dismiss"
    sx="size-control-h-xs shrink-0 self-start p-0 border-0 rounded-sm bg-transparent text-muted-foreground cursor-pointer transition-colors duration-fast ease-out hover:bg-muted hover:text-foreground focus:outline-solid focus:outline-2 focus:outline-ring focus:outline-offset-2"
  >
    <svg viewBox="0 0 10 10" aria-hidden="true" sx="block size-icon-size-sm stroke-current fill-none" stroke-width="1.75" stroke-linecap="round">
      <path d="M2 2 L8 8 M8 2 L2 8"></path>
    </svg>
  </button>
</div>
