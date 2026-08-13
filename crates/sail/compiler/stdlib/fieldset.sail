<!--
  <fieldset> — form group. Style product only.
  Native groove border replaced with token surface.
-->

<scope>
  props: {
    disabled = false
  }

  style base = {box: space({ m: 0, pad: 4 }), paint: border({ color: border, width: 1 }).radius(md).fill({ color: transparent }), when: {disabled: {paint: filter({ opacity: 50 }).cursor(not-allowed)}}}
</scope>

<fieldset style(base)>
  <slot></slot>
</fieldset>
