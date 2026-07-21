<!--
  <empty-state> — L1 language element (element mode). A centered placeholder for
  empty lists / missing content: an icon tile, a title, description copy, and an
  action row. Named slots (icon / title / action + the default description slot)
  project at compile time; bare tag lowers to <div class="sail-empty-state">.

  Paint (one path):
    layout keywords  — host column (stretch + self-center kids); icon row tile;
                       copy column stretch/grow; actions row
    host sx=         — type + PE default surface; forced-colors border
    child sx=        — icon/title/desc via --es-* channels; empty:hidden;
                       forced-colors on icon
    CVA-in-props     — size + variant maps (first key = default = PE)
    channels:        — size/variant/tone --es-* writes + gap + inline flex axes;
                       .icon svg size anatomy
    use:variantLive  — connecting live-region upgrade
  Zero contracts PE.
-->

<scope>
  props: {
    variant: {
      default: 'text-center border border-border rounded-lg bg-tone-muted-bg shadow-xs',
      inline: 'text-start border border-dashed border-border rounded-md bg-tone-muted-border shadow-none py-3 px-4',
      placeholder: 'text-center border-0 bg-transparent rounded-none shadow-none py-6',
      connecting: 'text-center border-0 bg-transparent rounded-none shadow-none py-6',
    }
    size: {
      md: 'py-16 px-4',
      sm: 'py-7 px-4',
      lg: 'py-24 px-6',
    }
    tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'default'
  }
  channels: {
    default: {
      'es-icon': 'control-h-2xs',
      'es-tile': 'space-12',
      'es-title-size': 'text-md',
      'es-desc-size': 'text-sm',
      'es-desc-max': 'panel-ch-w-md',
      'es-icon-bg': 'muted',
      'es-icon-fg': 'muted-foreground',
      'es-icon-ring': 'none',
      'es-title-weight': 'font-weight-semibold',
      'es-title-color': 'foreground',
    }
    size: {
      sm: {
        gap: '2',
        'es-icon': 'icon-size-md',
        'es-tile': 'space-9',
        'es-title-size': 'text-base',
      }
      lg: {
        gap: '4',
        'es-icon': 'icon-size-3xl',
        'es-tile': 'space-16',
        'es-title-size': 'text-lg',
        'es-desc-size': 'text-base',
      }
    }
    variant: {
      inline: {
        direction: 'row',
        align: 'center',
        justify: 'start',
        'es-desc-max': 'panel-ch-w-lg',
        'es-tile': 'es-icon',
        'es-icon-bg': 'transparent',
        'es-icon-ring': 'none',
      }
      connecting: {
        'es-title-weight': 'font-weight-medium',
        'es-title-color': 'muted-foreground',
      }
      placeholder: {
        'es-title-weight': 'font-weight-medium',
        'es-title-color': 'muted-foreground',
      }
    }
    tone: {
      primary: {
        'es-icon-bg': 'tone-bg',
        'es-icon-fg': 'tone-fg',
        'es-icon-ring': 'inset 0 0 0 1px var(--tone-ring)',
      }
      info: {
        'es-icon-bg': 'tone-bg',
        'es-icon-fg': 'tone-fg',
        'es-icon-ring': 'inset 0 0 0 1px var(--tone-ring)',
      }
      success: {
        'es-icon-bg': 'tone-bg',
        'es-icon-fg': 'tone-fg',
        'es-icon-ring': 'inset 0 0 0 1px var(--tone-ring)',
      }
      warning: {
        'es-icon-bg': 'tone-bg',
        'es-icon-fg': 'tone-fg',
        'es-icon-ring': 'inset 0 0 0 1px var(--tone-ring)',
      }
      destructive: {
        'es-icon-bg': 'tone-bg',
        'es-icon-fg': 'tone-fg',
        'es-icon-ring': 'inset 0 0 0 1px var(--tone-ring)',
      }
      accent: {
        'es-icon-bg': 'tone-bg',
        'es-icon-fg': 'tone-fg',
        'es-icon-ring': 'inset 0 0 0 1px var(--tone-ring)',
      }
    }
    '.icon svg': { width: 'es-icon', height: 'es-icon' }
  }
</scope>

<column
  align="stretch"
  justify="center"
  gap="3"
  use:variantLive
  role="note"
  aria-live="off"
  sx="font-body text-muted-foreground text-center border border-border rounded-lg bg-tone-muted-bg shadow-xs py-16 px-4 forced-colors:border-CanvasText"
>
  <row
    inline
    nowrap
    align="center"
    justify="center"
    class="icon"
    aria-hidden="true"
    sx="shrink-0 self-center rounded-lg bg-(--es-icon-bg) text-(--es-icon-fg) size-(--es-tile) shadow-(--es-icon-ring) empty:hidden forced-colors:bg-Canvas forced-colors:text-CanvasText"
  >
    <slot name="icon"></slot>
  </row>
  <column align="stretch" gap="1" class="copy" sx="min-w-0 self-stretch grow-1">
    <div class="title" sx="font-heading font-(--es-title-weight) text-(--es-title-color) leading-tight text-(--es-title-size) empty:hidden">
      <slot name="title"></slot>
    </div>
    <div class="desc" sx="leading-loose text-muted-foreground text-(--es-desc-size) max-w-(--es-desc-max) empty:hidden">
      <slot></slot>
    </div>
  </column>
  <row align="center" justify="center" gap="2" class="actions" sx="self-center empty:hidden">
    <slot name="action"></slot>
  </row>
</column>
