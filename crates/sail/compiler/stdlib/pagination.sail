<!--
  <pagination> — L1 pager (element mode). A nav of <page> controls; use:pagination
  marks the page matching `value` with aria-current and emits `sail-page-change`
  on click. Prev/next are <page>s whose value is the target page (disabled at the
  ends). Bare tag lowers to `<row inline nowrap align="center" gap="1" role="navigation" class="sail-pagination">`.

    <pagination value="3">
      <page value="2" aria-label="Previous">‹</page>
      <page value="1">1</page> <page value="2">2</page> <page value="3">3</page>
      <page value="4" aria-label="Next">›</page>
    </pagination>
-->

<scope>
  props: {
    value = ''
    size: 'sm' | 'md' | 'lg' = 'md'
  }
</scope>

<style>
  .sail-pagination {
    --pg-h: var(--control-h-md);
    --pg-fs: var(--text-sm);
  }
  .sail-pagination[size='sm'] { --pg-h: var(--control-h-sm); --pg-fs: var(--text-xs); }
  .sail-pagination[size='lg'] { --pg-h: var(--control-h-lg); --pg-fs: var(--text-base); }
</style>
<row inline nowrap align="center" gap="1" use:pagination role="navigation">
  <slot></slot>
</row>
