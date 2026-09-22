<scope>
  props: {
    busy = false
  }
</scope>
<button>
  <if {busy()}>
    <span aria-hidden="true"></span>
  {else => <slot reflect-empty="data-icon-only"></slot>}</if>
</button>
