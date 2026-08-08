<!--
  /deck/examples/greet/[name] — adapted from
  packages/deck/docs/routes/examples/greet/[name]/page.sail. A live [param]
  route: routes/examples/greet/[name]/page.sail matches /examples/greet/:name.
  `route` is ambient in every route; route.params.name() reads the segment
  as a signal, so the heading tracks the URL.
-->
<article style({ box: maxW(breakpoint-tablet).mx(auto) })>
  <p style({ box: mb(2), paint: fg(accent), text: fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide) })>Live example</p>
  <h1>Hello, {route.params.name()}.</h1>
  <p style({ paint: fg(muted-foreground), text: fontSize(lg).leading(relaxed) })>
    This page is <code>routes/examples/greet/[name]/page.sail</code>. The
    <code style(name)></code> directory is a route parameter, and
    <code>route.params.name()</code> reads it as a signal.
  </p>
  <p>
    Change the URL to change the greeting — try
    <a href="/deck/examples/greet/ada">/deck/examples/greet/ada</a> or
    <a href="/deck/examples/greet/deck">/deck/examples/greet/deck</a>.
  </p>
  <row style({ box: mt(8).padT(4), paint: borderTWidth(1).borderT(border) })>
    <a href="/deck/dynamic-routes">← Dynamic routes</a>
  </row>
</article>
