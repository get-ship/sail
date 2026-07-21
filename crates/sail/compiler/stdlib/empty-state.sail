<!--
  <empty-state> — L1 language element (element mode). A centered placeholder for
  empty lists / missing content: an icon tile, a title, description copy, and an
  action row. Named slots (icon / title / action + the default description slot)
  project at compile time; bare tag lowers to <div class="sail-empty-state">.

  Paint (one path):
    layout keywords  — host column; icon row tile; copy column; actions row
    host sx=         — type + PE default surface (md pad, bordered muted card)
    CVA-in-props     — size + variant maps (first key = default = PE)
    global [tone]    — icon tile channel reads
    contracts        — size channels for icon/title; parent-keyed inline layout;
                       :empty gates; PE muted icon for default/muted; forced-colors
  No residual empty-state matrix.
-->

<scope>
  props: {
    // first key = default = PE bordered muted card
    variant: {
      default: 'text-center border border-border rounded-lg bg-tone-muted-bg shadow-xs',
      inline: 'text-start border border-dashed border-border rounded-md bg-tone-muted-border shadow-none py-3 px-4',
      placeholder: 'text-center border-0 bg-transparent rounded-none shadow-none py-6',
      connecting: 'text-center border-0 bg-transparent rounded-none shadow-none py-6',
    }
    // first key = default = PE md
    size: {
      md: 'py-16 px-4',
      sm: 'py-7 px-4',
      lg: 'py-24 px-6',
    }
    tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'default'
  }
</scope>

<column
  align="center"
  justify="center"
  gap="3"
  use:variantLive
  role="note"
  aria-live="off"
  sx="font-body text-muted-foreground text-center border border-border rounded-lg bg-tone-muted-bg shadow-xs py-16 px-4"
>
  <row
    inline
    nowrap
    align="center"
    justify="center"
    class="icon"
    aria-hidden="true"
    sx="shrink-0 rounded-lg bg-tone-bg text-tone-fg size-(--es-tile)"
  >
    <slot name="icon"></slot>
  </row>
  <column align="center" gap="1" class="copy" sx="min-w-0">
    <div class="title" sx="font-heading font-semibold text-foreground leading-tight">
      <slot name="title"></slot>
    </div>
    <div class="desc" sx="leading-loose text-muted-foreground">
      <slot></slot>
    </div>
  </column>
  <row align="center" justify="center" gap="2" class="actions" sx="mt-1">
    <slot name="action"></slot>
  </row>
</column>
