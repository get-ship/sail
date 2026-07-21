<!--
  <icon> — sizes/inks one glyph. use:icon fetches named/src glyphs.
  Host paint: vertical-align on sx. Size channels, svg box, spin fx stay.
  flex center geometry stays (layout atoms refused on non-layout host).
-->

<scope>
  props: {
    name = ''
    src = ''
    label = ''
    size: 'xs' | 'sm' | 'md' | 'lg' | 'xl' = 'md'
    spin = false
  }
</scope>

<style>
  .sail-icon {
    align-items: center;
    justify-content: center;
    flex: 0 0 auto;
    color: currentColor;
    --sz: var(--icon-size-md);
  }
  .sail-icon[size='xs'] { --sz: var(--icon-size-2xs); }
  .sail-icon[size='sm'] { --sz: var(--icon-size-sm); }
  .sail-icon[size='lg'] { --sz: var(--icon-size-xl); }
  .sail-icon[size='xl'] { --sz: var(--icon-size-2xl); }

  .sail-icon svg {
    display: block;
    width: var(--sz);
    height: var(--sz);
    flex: 0 0 auto;
  }

  @media (prefers-reduced-motion: no-preference) {
    .sail-icon[spin] svg { animation: var(--animate-spin); }
  }
</style>

<span use:icon sx="align-middle"><slot></slot></span>
