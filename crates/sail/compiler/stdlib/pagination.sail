<!--
  <pagination> — nav state via use:pagination (ARIA current). Size is an ordinary
  prop stamped onto each <page> (child Style match). Host is layout-only.
-->

<scope>
  props: {
    value = ''
    size: 'sm' | 'md' | 'lg' = 'md'

    onPageChange: (detail: { page: string }) => void
  }
</scope>

<row inline nowrap align="center" gap="1" use:pagination>
  <slot></slot>
</row>
