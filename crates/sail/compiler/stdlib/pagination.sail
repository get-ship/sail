<!--
  <pagination> — nav state via use:pagination (ARIA current). Host is layout-only.
  Size Style match lives on each <page> (author there).
-->

<scope>
  props: {
    value = ''

    onPageChange: (detail: { page: string }) => void
  }
</scope>

<row inline nowrap align="center" gap="1" use:pagination>
  <slot></slot>
</row>
