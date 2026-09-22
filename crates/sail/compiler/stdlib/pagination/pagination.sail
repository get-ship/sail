<!--
  <pagination> — nav state lives in this file's <controller> (page signal,
  selection, onPageChange). No DOM, no use: action; host is layout-only
  (role navigation from host anatomy, label here).

  Each slotted <page-item> takes `page` + `select` as props from this
  instance (signal cell, not a snapshot) and stamps its own aria-current.
  Size/variant Style matches live on each <page-item> (author there).

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

<controller>
export default controller((init) => {
  const page = signal(String(init.value || ''))
  function select(v) {
    if (!v || v === page()) return
    page.set(v)
    const cb = init.onPageChange
    if (typeof cb === 'function') cb({ page: v })
  }
  return { page, select }
})
</controller>

<row inline nowrap align="center" gap="1" aria-label="Pagination" style({match: { variant: { default: pad(0), simple: simple } }})>
  <slot></slot>
</row>