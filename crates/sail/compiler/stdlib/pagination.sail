<!--
  <pagination> — stamps size onto child pages via use:pagination.
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
