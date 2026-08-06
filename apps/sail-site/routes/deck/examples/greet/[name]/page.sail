<!--
  /deck/examples/greet/[name] — adapted from
  packages/deck/docs/routes/examples/greet/[name]/page.sail. A live [param]
  route: routes/examples/greet/[name]/page.sail matches /examples/greet/:name.
  `route` is ambient in every route; route.params.name() reads the segment
  as a signal, so the heading tracks the URL.
-->
<article>
  style.maxW(breakpoint-tablet).mx(auto)

  <p>
    style.fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide).fg(accent).mb(2)
Live example</p>
  <h1>Hello, {route.params.name()}.</h1>
  <p>
    style.fontSize(lg).fg(muted-foreground).leading(relaxed)

    This page is <code>routes/examples/greet/[name]/page.sail</code>. The
    <code>[name]</code> directory is a route parameter, and
    <code>route.params.name()</code> reads it as a signal.
  </p>
  <p>
    Change the URL to change the greeting — try
    <a href="/deck/examples/greet/ada">/deck/examples/greet/ada</a> or
    <a href="/deck/examples/greet/deck">/deck/examples/greet/deck</a>.
  </p>
  <row>
    style.mt(8).padT(4).borderTWidth(1).borderT(border)

    <a href="/deck/dynamic-routes">← Dynamic routes</a>
  </row>
</article>
