<!--
  <error-state> — L1 language element (element mode). The "something broke"
  sibling of <empty-state>: an always-destructive placeholder with a fixed
  alert-circle icon tile, a title, description, an optional error code chip, an
  optional collapsible technical-details section, and a retry action.

  Named slots project at compile time (title / details / action + the default
  description slot). Behavior rides use:recoverySurface: fill the code chip from the
  `code` attribute, gate + drive the details toggle (Show/Hide, aria-expanded),
  and auto-focus the retry action. Bare tag lowers to <div class=sail-error-state>.

  Paint (one path):
    layout keywords  — host column; icon row; copy column; actions row
    host sx=         — type + PE default surface (md pad, bordered muted card)
    CVA-in-props     — size + variant maps (first key = default = PE)
    contracts        — size channels; parent-keyed inline layout; data-has-details /
                       data-details-open gates; :empty; forced-colors; icon inset ring
  No residual error-state matrix.
-->

<scope>
  props: {
    // first key = default = PE bordered muted card
    variant: {
      default: 'text-center border border-border rounded-lg bg-tone-muted-bg shadow-xs',
      inline: 'text-start border border-tone-destructive-border rounded-md bg-tone-destructive-bg shadow-none py-3 px-4',
    }
    // first key = default = PE md
    size: {
      md: 'py-8 px-4',
      sm: 'py-7',
      lg: 'py-8 px-6',
    }
    code = ''
  }
</scope>

<column
  align="center"
  justify="center"
  gap="3"
  use:recoverySurface
  role="alert"
  sx="font-body text-muted-foreground text-center border border-border rounded-lg bg-tone-muted-bg shadow-xs py-8 px-4"
>
  <row
    inline
    nowrap
    align="center"
    justify="center"
    class="icon"
    aria-hidden="true"
    sx="shrink-0 rounded-lg bg-tone-destructive-bg text-destructive size-(--es-tile)"
  >
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" sx="size-(--es-icon)">
      <circle cx="12" cy="12" r="9"></circle>
      <path d="M12 8v5"></path>
      <path d="M12 16h.01"></path>
    </svg>
  </row>
  <column align="center" gap="1" class="copy" sx="min-w-0">
    <div class="title" sx="font-heading font-semibold text-foreground leading-tight">
      <slot name="title">Something went wrong</slot>
    </div>
    <div class="desc" sx="leading-loose text-muted-foreground">
      <slot></slot>
    </div>
    <div class="code" sx="font-mono text-xs text-muted-foreground bg-muted border border-border-subtle rounded-sm py-1 px-2 mt-1"></div>
    <button
      class="details-toggle"
      type="button"
      anatomy
      aria-expanded="false"
      sx="mt-1 py-1 px-2 border-0 bg-transparent text-muted-foreground font-body text-xs rounded-sm cursor-pointer hover:text-foreground hover:bg-muted focus:outline-solid focus:outline-2 focus:outline-ring focus:outline-offset-2"
    >Show details</button>
    <div class="details" sx="mt-2 py-2 px-3 bg-tone-destructive-bg rounded-sm font-mono text-xs text-muted-foreground whitespace-pre-wrap break-words text-start">
      <slot name="details"></slot>
    </div>
  </column>
  <row align="center" justify="center" gap="2" class="actions" sx="mt-1">
    <slot name="action"></slot>
  </row>
</column>
