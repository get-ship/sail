<!--
  <pagination> — nav state via use:pagination (ARIA current). Host is layout-only.
  Size Style match lives on each <page-item> (author there).

  variant=simple: denser host Style (no extra pad). Page-number chrome is
  authored in the slot — there are no built-in number buttons to hide.
  Slotted <page-item> does not inherit this variant (Style cannot target
  descendants); set variant on each <page-item> for smaller pad / no border.
-->

<scope>
  props: {
    value = ''
    variant: 'default' | 'simple' = 'default'

    onPageChange: (detail: { page: string }) => void
  }

  style simple = {pad(0)}
</scope>

<row inline nowrap align="center" gap="1" use:pagination style({match: { variant: { default: pad(0), simple: simple } }})>
  <slot></slot>
</row>
