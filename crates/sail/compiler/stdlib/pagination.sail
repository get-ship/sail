<!--
  <pagination> — nav of <page>s. Size channels for children live in contracts.
-->

<scope>
  props: {
    value = ''
    size: 'sm' | 'md' | 'lg' = 'md'
  }
</scope>

<row inline nowrap align="center" gap="1" use:pagination role="navigation">
  <slot></slot>
</row>
