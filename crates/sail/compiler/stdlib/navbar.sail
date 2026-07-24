<!--
  <navbar> — application nav bar. Placements top/rail/bottom; sticky; bordered.

  Paint/layout (one path — no contracts PE matrix):
    layout keywords  — host row (nowrap, align center, gap 4); brand/actions
                       inline rows; links as contents so slot kids share the
                       host flex axis (row top / column rail); center row grows
    host sx=         — bg/fg from component.navbar tokens (cssNames navbar,
                       navbar-foreground) + pad/min-h + no-underline + w-full
    CVA-in-props     — placement pad/min-h + rail min-w-(--panel-rail-w) +
                       dir-column (rail flips host flex axis)
    compound         — sticky/bordered + bottom/rail border + sticky rail
                       geometry (top-0 bottom-0 overflow-y-auto; offset overrides top)
    child sx         — center grow-1; actions ms-auto mt-auto (row push / rail push)
  Slotted link/label chrome is author sx on the projected nodes (current:/hover:).
-->

<scope>
  props: {
    placement: {
      top: 'min-h: 14 py: 2 px: 4',
      bottom: 'min-h: 14 py: 2 px: 4',
      rail: 'min-h: 0 py: 5 px: 4 min-w: (--panel-rail-w) dir-column',
    }
    sticky = false
    bordered = false
    offset = ''
  }
  compound: [
    { when: { sticky: true }, sx: 'sticky top: 0 z: (--z-sticky)' },
    { when: { bordered: true }, sx: 'border-b border: border' },
    { when: { placement: 'bottom', sticky: true }, sx: 'top: auto bottom: 0' },
    { when: { placement: 'bottom', bordered: true }, sx: 'border-b: 0 border-t border: border' },
    { when: { placement: 'rail', bordered: true }, sx: 'border-b: 0 border-e border: border' },
    { when: { placement: 'rail', sticky: true }, sx: 'sticky top: 0 bottom: 0 self: start overflow-y: auto z: dropdown' },
    { when: { placement: 'rail', sticky: true, offset: '8' }, sx: 'top: 8' },
    { when: { placement: 'rail', sticky: true, offset: '10' }, sx: 'top: 10' },
    { when: { placement: 'rail', sticky: true, offset: '12' }, sx: 'top: 12' },
    { when: { placement: 'rail', sticky: true, offset: '14' }, sx: 'top: 14' },
    { when: { placement: 'rail', sticky: true, offset: '16' }, sx: 'top: 16' },
    { when: { placement: 'rail', sticky: true, offset: '20' }, sx: 'top: 20' },
    { when: { placement: 'rail', sticky: true, offset: '24' }, sx: 'top: 24' },
  ]
</scope>

<row
  nowrap
  align="center"
  gap="4"
  role="navigation"
  sx="w: full bg: navbar color: navbar-foreground min-h: 14 py: 2 px: 4 no-underline"
>
  <row inline nowrap align="center" gap="2" class="sail-nav-brand" sx="font-weight: semibold shrink: 0">
    <slot name="brand"></slot>
  </row>
  <div class="sail-nav-links" sx="contents">
    <slot></slot>
  </div>
  <row
    nowrap
    align="center"
    justify="center"
    class="sail-nav-center"
    sx="min-w: 0 grow: 1"
  >
    <slot name="center"></slot>
  </row>
  <row
    inline
    nowrap
    align="center"
    gap="2"
    class="sail-nav-actions"
    sx="shrink: 0 ms: auto mt: auto"
  >
    <slot name="actions"></slot>
  </row>
</row>
