<!--
  A live [param] route — routes/examples/greet/[name]/page.sail matches
  /examples/greet/:name. `route` is ambient in every route; route.params.name()
  reads the segment as a signal, so the heading tracks the URL.
-->
<article data-doc="true">
  <p role="doc-eyebrow">Live example</p>
  <h1>Hello, {route.params.name()}.</h1>
  <p role="doc-lead">
    This page is <code>routes/examples/greet/[name]/page.sail</code>. The
    <code style(name)></code> directory is a route parameter, and
    <code>route.params.name()</code> reads it as a signal.
  </p>
  <p>
    Change the URL to change the greeting — try
    <a href="/examples/greet/ada">/examples/greet/ada</a> or
    <a href="/examples/greet/deck">/examples/greet/deck</a>.
  </p>
  <div role="navigation">
    <a href="/dynamic-routes">← Dynamic routes</a>
    <span></span>
  </div>
</article>
