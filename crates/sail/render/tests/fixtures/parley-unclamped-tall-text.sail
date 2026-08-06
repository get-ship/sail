<!--
  RENDER-0 residual: unclamped multi-line text no longer stops measure at
  viewport height (viewport-as-max_height removed). Pins: no panic + non-zero
  text-run height. Not a pixel-parity gate vs upstream capped measure.
-->
<column w="full">
  style.pad(2)

  <span>
    Line one of unclamped body copy that should wrap under a modest width.
    Line two continues the run so taffy/takumi measures multiple lines fully.
    Line three keeps going past a short viewport if someone re-enables the
    old viewport MaxHeight shortcut — this fixture must still not panic.
    Line four and five pad the paragraph so height is obviously non-trivial
    under a narrow column at desktop width.
  </span>
</column>
