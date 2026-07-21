<!--
  <page> — pagination cell. Size channels --pg-h set by parent pagination (contracts).
  Current/hover/disabled/focus on host sx (primary ink for current — no tone prop).
-->

<scope>
  props: {
    value = ''
    disabled = false
  }
</scope>

<button
  type="button"
  sx="font-body font-medium text-foreground bg-transparent rounded-md cursor-pointer select-none min-w-(--pg-h) h-(--pg-h) px-2 border border-transparent transition-colors duration-fast ease-out hover:bg-muted current:bg-primary current:text-primary-foreground current:border-primary disabled:opacity-50 disabled:cursor-not-allowed focus:outline-solid focus:outline-2 focus:outline-ring"
>
  <slot></slot>
</button>
