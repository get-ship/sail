<!--
  <slider> — L1 language element (element mode). A compiles-away alias for the
  NATIVE range input: <slider value=40 tone=primary> lowers to
  <input type="range"> (skinned by the control contracts' RANGE_SKIN_CSS). The
  platform owns value/min/max/step, drag, the full keyboard suite, role=slider +
  aria-valuenow, and form submission — zero JS for all of it. The one thing the
  platform can't draw on Chrome/WebKit is the filled track; use:rangeFill closes
  that (a value→--rng-pct sync so the skin's webkit-track gradient fills). One
  implementation: native range + skin + the tiny fill-sync.

  Replaces the prior custom-div slider + use:slider controller — a competing
  implementation of what the platform already gives for free.
-->

<scope>
  props: {
    value = 0
    min = 0
    max = 100
    step = 1
    disabled = false
    tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'default'
    size: 'sm' | 'md' | 'lg' = 'md'
    name = ''
  }
</scope>

<input type="range" use:rangeFill />
