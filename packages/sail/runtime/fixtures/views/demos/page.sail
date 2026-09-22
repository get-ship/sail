<controller>
export default null
</controller>

<scope>
  style page = { pad(6).margin({ x: auto }).size({ max-w: breakpoint-tablet }) }
  style caption = { fg(muted-foreground) }
  type DemoLinkProps = { href: string; name: string; blurb: string }
</scope>

<partial demo-link props: DemoLinkProps>
  <column gap="1" style({ pad(3).border({ color: border, width: 1 }).radius(md).bg(surface) })>
    <a href={href}>{name}</a>
    <p style(caption)>{blurb}</p>
  </column>
</partial>

<column gap="4" style(page)>
  <column gap="1">
    <h1>Sail demos</h1>
    <p style(caption)>
      Small, single-feature views exercising one piece of the Sail surface each.
      Each demo lives at <code>views/demos/&lt;name&gt;/</code> as a short
      <code>page.sail</code> with an inline controller.
    </p>
  </column>

  <column gap="2">
    <demo-link
      href="/demos/signals-and-derived"
      name="signals-and-derived"
      blurb="counter, derived double, history list — signal + derived."
    />
    <demo-link
      href="/demos/conditional-render"
      name="conditional-render"
      blurb="checkbox toggles a panel — block if/else."
    />
    <demo-link
      href="/demos/iteration"
      name="iteration"
      blurb="todo list — keyed for over a signal array."
    />
    <demo-link
      href="/demos/match"
      name="match"
      blurb="alert tone selector — block match."
    />
    <demo-link
      href="/demos/lifecycle"
      name="lifecycle"
      blurb="mount tick log — effect mount cleanup."
    />
  </column>
</column>
