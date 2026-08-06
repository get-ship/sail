<!--
  /sail/reference/sx — Style product surface: setters (via residual atom table
  for value lists), when keys, refusals. Prose teaches style.* / match / when /
  compound; the generated tables still list scale values the setters spend.
-->
<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import {
  SX_ATOMS,
  SX_STATE_PREFIXES,
  SX_CONDITION_PREFIXES,
  SX_REFUSALS,
  SX_EXTRA_FORMS,
} from '../../../../src/language-surface.gen'

// A few atoms register under the same `prop` name in more than one table
// (e.g. a spacing sense and a color sense both named `border`) — merge
// their value lists under one card instead of letting the later
// registration silently shadow the earlier one in a keyed {#for}.
const SX_BY_PROP = new Map<string, string[]>()
for (const a of SX_ATOMS) {
  const existing = SX_BY_PROP.get(a.prop)
  if (existing) {
    existing.push(...a.values)
  } else {
    SX_BY_PROP.set(a.prop, [...a.values])
  }
}
const SX_ROWS = [...SX_BY_PROP.entries()].map(([prop, values]) => ({
  prop,
  values: values.join(' · '),
}))

export default controller((_ctx: ControllerScope<Scope>) => ({
  sxRows: () => SX_ROWS,
  sxCount: () => SX_ATOMS.length,
  statePrefixes: () => SX_STATE_PREFIXES,
  conditionPrefixes: () => SX_CONDITION_PREFIXES,
  refusals: () => SX_REFUSALS,
  refusalCount: () => SX_REFUSALS.length,
  extraForms: () => SX_EXTRA_FORMS,
}))
</controller>

<column gap="6">
  style({ box: minW(0) })

  <column gap="5">
    style({ box: maxW(breakpoint-tablet) })

    <p token="doc-eyebrow">Reference</p>
    <h1>Style — block channel attach</h1>
    <p token="doc-lead">
      Paint is Style only, as a <strong>block statement</strong> on the host:
      <code>style({ box: padX(2), paint: bg(primary) })</code>,
      <code>style([base, match(…), when(…)])</code>, or <code>style(constRef)</code>.
      Every setter argument is a design token from a closed scale — the rows
      below list those values. An off-list token is a compile error that names
      the same list.
    </p>
    <p>
      style({ paint: fg(muted-foreground), text: fontSize(sm) })

      Utility-string <code>style="…"</code> / <code>sx="…"</code> is not product.
      Responsive and state paint use
      <code>.when(tablet, &#123; … &#125;)</code> / <code>.when(hover, &#123; … &#125;)</code>
      or map <code>when(&#123; tablet: style… &#125;)</code>, not
      <code>tablet:</code> / <code>hover:</code> prefixes on atoms.
      {sxCount()} scale entries feed the setters.
    </p>
  </column>

  <column gap="3">
    <h3>Setter value scales</h3>
    <p>
      style({ box: maxW(breakpoint-tablet), paint: fg(muted-foreground), text: fontSize(sm) })

      Rows are named by historical atom heads (<code>bg</code>, <code>p</code>,
      <code>rounded</code>, …). In product form you call the Style setter:
      <code>style({ paint: bg(…) })</code>, <code>style({ box: pad(…) })</code>,
      <code>style({ paint: radius(…) })</code>, <code>style({ paint: fg(…) })</code>, and so on.
    </p>
    <grid cols="2" gap="2">
      <for {sxRows()} as="a" key={a.prop}>
        <card padding="sm">
          style({ box: minW(0) })

          <column gap="1">
            <code>{a.prop}</code>
            <small>
              style({ box: maxW(panel-max-w-sm), paint: fg(muted-foreground) })
{a.values}</small>
          </column>
        </card>
      </for>
    </grid>
  </column>

  <column gap="3">
    <h3>Not Style — use this ({refusalCount()})</h3>
    <p>
      style({ box: maxW(breakpoint-tablet) })

      The refused table is the other half of the vocabulary. Every entry below
      is something a Tailwind prior will reach for and Sail refuses by name;
      each names the element or attribute to use instead.
    </p>
    <grid cols="2" gap="2">
      <for {refusals()} as="r" key={r.atom}>
        <card padding="sm">
          style({ box: minW(0) })

          <column gap="1">
            <code>
              style({ paint: fg(destructive) })
{r.atom}</code>
            <small>
              style({ box: maxW(panel-max-w-sm), paint: fg(muted-foreground) })
→ {r.instead}</small>
          </column>
        </card>
      </for>
    </grid>
  </column>

  <column gap="3">
    <h3>when keys — interaction &amp; ARIA</h3>
    <p>
      style({ box: maxW(breakpoint-tablet), paint: fg(muted-foreground), text: fontSize(sm) })

      Product form: <code>when(&#123; hover: style({ paint: bg(muted) }), current: style({ paint: fg(primary) }) &#125;)</code>.
      Historical prefix names below map 1:1 to those keys.
    </p>
    <grid cols="3" gap="2">
      <for {statePrefixes()} as="s" key={s.prefix}>
        <row gap="3" align="center">
          style({ box: padX(3).padY(1), paint: radius(md) })

          <code>
            style({ box: minW(9) })
{s.prefix}</code>
          <small>
            style({ box: maxW(ch-24), paint: fg(muted-foreground), text: fontFamily(mono) })
{s.selector}</small>
        </row>
      </for>
    </grid>
    <p>
      style({ box: maxW(breakpoint-tablet) })

      Ancestor propagation: mark the ancestor with <code>style({ paint: group() })</code>,
      then <code>when(&#123; groupHover: … &#125;)</code> on the descendant.
    </p>
  </column>

  <column gap="3">
    <h3>when keys — media conditions</h3>
    <p>
      style({ box: maxW(breakpoint-tablet), paint: fg(muted-foreground), text: fontSize(sm) })

      Breakpoints are locked to design-token <code>breakpoint-*</code> lengths.
      Use <code>when(&#123; tablet: style({ text: fontSize(lg) }), desktop: style({ text: fontSize(xl) }) &#125;)</code>.
    </p>
    <grid cols="3" gap="2">
      <for {conditionPrefixes()} as="c" key={c.prefix}>
        <row gap="3" align="center">
          style({ box: padX(3).padY(1), paint: radius(md) })

          <code>
            style({ box: minW(9) })
{c.prefix}</code>
          <small>
            style({ box: maxW(ch-24), paint: fg(muted-foreground), text: fontFamily(mono) })
{c.media}</small>
        </row>
      </for>
    </grid>
  </column>

  <column gap="3">
    <h3>Composition &amp; bind</h3>
    <p>
      style({ box: maxW(breakpoint-tablet) })

      <code>match(prop, &#123; … &#125;)</code> branches on a prop value;
      <code>compound(&#123; … &#125;, style…)</code> intersects prop arms;
      <code>bind(signalName)</code> reads a controller signal as a CSS custom
      property (timeline tracks, range-driven paint).
    </p>
    <grid cols="2" gap="2">
      <for {extraForms()} as="f" key={f.form}>
        <card padding="sm">
          style({ box: minW(0) })

          <column gap="1">
            <code>{f.form}</code>
            <small>
              style({ box: maxW(panel-max-w-sm), paint: fg(muted-foreground) })
{f.doc}</small>
          </column>
        </card>
      </for>
    </grid>
  </column>

  <row>
    <a variant="default" href="/sail/reference/layout">← Layout keywords</a>
    <a variant="solid" href="/sail/reference/motion">Motion →</a>
  </row>
</column>
