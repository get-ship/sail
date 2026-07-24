<!--
  <pagination> — nav state via use:pagination (ARIA current). Size is an ordinary
  prop on each <page> (child CVA).
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
