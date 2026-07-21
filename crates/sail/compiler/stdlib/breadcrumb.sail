<!--
  <breadcrumb> — L1 breadcrumb trail (element mode). A nav > ol; author the
  crumbs as <li> with an <a> (mark the current page's <a> aria-current="page").
  Separators are drawn by CSS between items. Bare tag lowers to
  <nav class="sail-breadcrumb"><ol>.

    <breadcrumb>
      <li><a href="/">Home</a></li>
      <li><a href="/docs">Docs</a></li>
      <li><a aria-current="page">Tabs</a></li>
    </breadcrumb>
-->

<scope>
</scope>

<nav aria-label="Breadcrumb"><ol><slot></slot></ol></nav>
