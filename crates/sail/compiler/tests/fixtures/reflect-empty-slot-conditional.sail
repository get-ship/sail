<scope>
  props: {
    busy = false
  }
</scope>
<button>
  <if cond={busy()}>
    <span aria-hidden="true"></span>
  <else>
    <slot reflect-empty="data-icon-only"></slot>
  </else></if>
</button>
