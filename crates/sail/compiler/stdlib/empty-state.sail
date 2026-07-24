<!--
  <empty-state> — L1 language element (element mode). A centered placeholder for
  empty lists / missing content: an icon tile, a title, description copy, and an
  action row. Named slots (icon / title / action + the default description slot)
  project at compile time; bare tag lowers to <div class="sail-empty-state">.

  Paint (one path):
    layout keywords  — host column (stretch + self-center kids); icon row tile;
                       copy column stretch/grow; actions row
    host sx=         — type + PE default surface; forced-colors border
    child sx=        — icon/title/desc PE tokens; empty:hidden; forced-colors on icon
    CVA-in-props     — size + variant maps (first key = default = PE)
    use:variantLive  — connecting live-region upgrade
  Variance = CVA-in-props axes + optional compound only. tone/size/variant are
  ordinary props (axes or unions), not language channels.
  Zero contracts PE.
-->

<scope>
  props: {
    variant: {
      default: 'text: center border border: border rounded: lg bg: tone-muted-bg shadow: xs',
      inline: 'text: start border border: dashed border: border rounded: md bg: tone-muted-border shadow: none py: 3 px: 4',
      placeholder: 'text: center border: 0 bg: transparent rounded: none shadow: none py: 6',
      connecting: 'text: center border: 0 bg: transparent rounded: none shadow: none py: 6',
    }
    size: {
      md: 'py: 16 px: 4',
      sm: 'py: 7 px: 4',
      lg: 'py: 24 px: 6',
    }
    tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'default'
  }
</scope>

<column
  align="stretch"
  justify="center"
  gap="3"
  use:variantLive
  role="note"
  aria-live="off"
  sx="font-family: body color: muted-foreground text: center border border: border rounded: lg bg: tone-muted-bg shadow: xs py: 16 px: 4 forced-colors:border: CanvasText"
>
  <row
    inline
    nowrap
    align="center"
    justify="center"
    class="icon"
    aria-hidden="true"
    sx="shrink: 0 self: center rounded: lg bg: tone-bg color: tone-fg size: 12 empty:hidden forced-colors:bg: Canvas forced-colors:color: CanvasText"
  >
    <slot name="icon"></slot>
  </row>
  <column align="stretch" gap="1" class="copy" sx="min-w: 0 self: stretch grow: 1">
    <div class="title" sx="font-family: heading font-weight: semibold color: foreground leading: tight font-size: md empty:hidden">
      <slot name="title"></slot>
    </div>
    <div class="desc" sx="leading: loose color: muted-foreground font-size: sm max-w: panel-ch-w-md empty:hidden">
      <slot></slot>
    </div>
  </column>
  <row align="center" justify="center" gap="2" class="actions" sx="self: center empty:hidden">
    <slot name="action"></slot>
  </row>
</column>
