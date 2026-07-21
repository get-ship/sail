<!--
  <tab name="…"> — tab button inside <tab-list>. use:tabs on parent wires a11y.

  Paint: host sx (type/chrome/pad channels/hover/selected/disabled).
  Pad channels --tab-py/--tab-px set by parent <tabs> (contracts).
  Contracts: selected underline color (tone fallback) + inset focus ring.
-->

<scope>
  props: {
    name = ''
    disabled = false
  }
</scope>

<button
  type="button"
  sx="font-body font-medium leading-none text-muted-foreground bg-transparent cursor-pointer whitespace-nowrap select-none py-(--tab-py) px-(--tab-px) border-0 border-b-2 border-b-transparent rounded-t-sm tap-highlight-none transition-colors duration-fast ease-out hover:text-foreground selected:text-foreground selected:font-semibold disabled:opacity-50 disabled:cursor-not-allowed"
>
  <slot></slot>
</button>
