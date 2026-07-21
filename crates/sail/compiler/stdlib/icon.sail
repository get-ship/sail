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

<span use:icon sx="align-middle"><slot></slot></span>
