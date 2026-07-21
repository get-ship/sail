<!--
  <navbar> — application nav bar. Placements top/rail/bottom; sticky; bordered.
  Host: PE top-bar base on sx. Multi-region rail/sticky/link chrome lives as
  named contracts recipes (true multi-node unexpressible). No residual section.
-->

<scope>
  props: {
    placement: 'top' | 'rail' | 'bottom' = 'top'
    sticky = false
    bordered = false
  }
</scope>

<nav sx="bg-background text-foreground">
  <div class="sail-nav-brand"><slot name="brand"></slot></div>
  <div class="sail-nav-links"><slot></slot></div>
  <div class="sail-nav-center"><slot name="center"></slot></div>
  <div class="sail-nav-actions"><slot name="actions"></slot></div>
</nav>
