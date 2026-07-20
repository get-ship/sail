<!--
  <icon> — L1 language element. Sizes and inks one glyph: a slotted SVG, a
  `name` from the host's icon endpoint (deck serves the embedded lucide set),
  or an arbitrary `src` URL. Fetch/ARIA currently attach via `use:icon`
  (mix-in); L1 behavior belongs on an element controller long-term (Part B).
  Ink is currentColor inheritance. Brand/identity mappings are app-land
  composition, not language.
-->

<scope>
  props: {
    /** Named glyph from the icon endpoint (deck serves the embedded lucide
        set). Empty keeps slotted children. */
    name = ''
    /** Fetch an arbitrary SVG URL instead of a named glyph; wins over name. */
    src = ''
    /** Accessible name — role=img + aria-label; label-less icons are
        aria-hidden decoration. */
    label = ''
    size: 'xs' | 'sm' | 'md' | 'lg' | 'xl' = 'md'
    spin = false
  }
</scope>

<style>
  .sail-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    vertical-align: -0.125em;
    flex: 0 0 auto;
    color: currentColor;
    --sz: var(--icon-size-md);
  }
  .sail-icon[size='xs'] { --sz: var(--icon-size-2xs); }
  .sail-icon[size='sm'] { --sz: var(--icon-size-sm); }
  .sail-icon[size='lg'] { --sz: var(--icon-size-xl); }
  .sail-icon[size='xl'] { --sz: var(--icon-size-2xl); }

  /* Both glyph sources fill the icon box: the action-injected svg and a
     slotted one. */
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

<span use:icon><slot></slot></span>
