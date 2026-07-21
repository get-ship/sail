<!--
  <tooltip> — hover/focus text bubble via CSS Anchor Positioning. use:tipDescribe
  supplies anchor name + content + Escape.

  Paint (one path):
    host sx=         — (inline-flex from span root display)
    child sx=        — bubble chrome (pad/colors/type/radius/shadow)
    contracts        — anchor-name / position-anchor / position-area placement;
                       opacity show/hide; arrow ::after; @supports fallbacks
  No residual tooltip matrix.
-->

<scope>
  props: {
    placement: 'top' | 'bottom' | 'left' | 'right' = 'top'
    content = ''
  }
</scope>

<span use:tipDescribe>
  <slot></slot>
  <span
    class="bubble"
    role="tooltip"
    sx="py-1 px-2 bg-foreground text-background font-body text-xs font-medium rounded-md shadow-md whitespace-nowrap pointer-events-none"
  ></span>
</span>
