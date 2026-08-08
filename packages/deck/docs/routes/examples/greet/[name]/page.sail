<!-- Live [param] route — route.params.name() tracks the URL segment as a signal. -->

<scope>
  style page = { box: maxW(breakpoint-tablet).mx(auto) }
  style eyebrow = { box: mb(2), paint: fg(accent), text: fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide) }
  style lead = { paint: fg(muted-foreground), text: fontSize(lg).leading(relaxed) }
  style footNav = { box: mt(8).padT(4), paint: borderTWidth(1).borderT(border) }
</scope>

<article style(page)>
  <p style(eyebrow)>Live example</p>
  <h1>Hello, {route.params.name()}.</h1>
  <p style(lead)>
    This page is <code>routes/examples/greet/[name]/page.sail</code>. The
    <code>{'[name]'}</code> directory is a route parameter, and
    <code>route.params.name()</code> reads it as a signal.
  </p>
  <p>
    Change the URL to change the greeting — try
    <a href="/examples/greet/ada">/examples/greet/ada</a> or
    <a href="/examples/greet/deck">/examples/greet/deck</a>.
  </p>
  <row justify="between" style(footNav)>
    <a href="/dynamic-routes">← Dynamic routes</a>
    <span></span>
  </row>
</article>
