<!--
  <navbar> — application nav bar. Placements top/rail/bottom; sticky; bordered.

  Paint/layout (one path — no contracts PE matrix):
    layout keywords  — host row (nowrap, align center, gap 4); brand/actions
                       inline rows; links as contents so slot kids share the
                       host flex axis (row top / column rail); center row grows
    host sx=         — PE top-bar base (bg/fg/pad/min-h; attr-absent defaults)
                       + no-underline (inherits onto slot anchors) + w-full
    CVA-in-props     — placement pad/min-h + rail min-w-(--panel-rail-w) +
                       dir-column (rail flips host flex axis)
    compound         — sticky/bordered + bottom/rail border + sticky rail
                       geometry (top-(--rail-offset) h-rail-fill overflow-y-auto)
    channels:        — offset space-token → --rail-offset; placement rail
                       rekeys center grow + actions ms/mt push
  Slotted link/label chrome is author sx on the projected nodes (current:/hover:).
-->

<scope>
  props: {
    placement: {
      top: 'min-h-14 py-2 px-4',
      bottom: 'min-h-14 py-2 px-4',
      rail: 'min-h-0 py-5 px-4 min-w-(--panel-rail-w) dir-column',
    }
    sticky = false
    bordered = false
    offset = ''
  }
  compound: [
    { when: { sticky: true }, sx: 'sticky top-0 z-(--z-sticky)' },
    { when: { bordered: true }, sx: 'border-b border-border' },
    { when: { placement: 'bottom', sticky: true }, sx: 'top-auto bottom-0' },
    { when: { placement: 'bottom', bordered: true }, sx: 'border-b-0 border-t border-border' },
    { when: { placement: 'rail', bordered: true }, sx: 'border-b-0 border-e border-border' },
    { when: { placement: 'rail', sticky: true }, sx: 'sticky top-(--rail-offset) self-start h-rail-fill overflow-y-auto z-dropdown' },
  ]
  channels: {
    default: {
      'nav-center-grow': '1',
      'nav-actions-ms': 'auto',
      'nav-actions-mt': '0',
    }
    placement: {
      rail: {
        'nav-center-grow': '0',
        'nav-actions-ms': '0',
        'nav-actions-mt': 'auto',
      }
    }
    offset: {
      8: { 'rail-offset': 'space-8' },
      10: { 'rail-offset': 'space-10' },
      12: { 'rail-offset': 'space-12' },
      14: { 'rail-offset': 'space-14' },
      16: { 'rail-offset': 'space-16' },
      20: { 'rail-offset': 'space-20' },
      24: { 'rail-offset': 'space-24' },
    }
  }
</scope>

<row
  nowrap
  align="center"
  gap="4"
  role="navigation"
  sx="w-full bg-background text-foreground min-h-14 py-2 px-4 no-underline"
>
  <row inline nowrap align="center" gap="2" class="sail-nav-brand" sx="font-semibold shrink-0">
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
    sx="min-w-0 grow-(--nav-center-grow)"
  >
    <slot name="center"></slot>
  </row>
  <row
    inline
    nowrap
    align="center"
    gap="2"
    class="sail-nav-actions"
    sx="shrink-0 ms-(--nav-actions-ms) mt-(--nav-actions-mt)"
  >
    <slot name="actions"></slot>
  </row>
</row>
