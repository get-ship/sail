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

<column gap="6" sx={style.minW(0)}>
  <column sx={style.maxW(breakpoint-tablet)} gap="5">
    <p role="doc-eyebrow">Reference</p>
    <h1>Style — paint on sx</h1>
    <p role="doc-lead">
      Paint is Style only: <code>sx=&#123;style.bg(primary).padX(2)&#125;</code>,
      <code>sx=&#123;[base, match(…), when(…)]&#125;</code>, or a scope const.
      Every setter argument is a design token from a closed scale — the rows
      below list those values. An off-list token is a compile error that names
      the same list.
    </p>
    <p sx={style.fontSize(sm).fg(muted-foreground)}>
      Utility-string <code>sx="…"</code> is not product. Responsive and state
      paint use <code>when(&#123; tablet: style…, hover: style… &#125;)</code>, not
      <code>tablet:</code> / <code>hover:</code> prefixes on atoms.
      {sxCount()} scale entries feed the setters.
    </p>
  </column>

  <column gap="3">
    <h3>Setter value scales</h3>
    <p sx={style.maxW(breakpoint-tablet).fontSize(sm).fg(muted-foreground)}>
      Rows are named by historical atom heads (<code>bg</code>, <code>p</code>,
      <code>rounded</code>, …). In product form you call the Style setter:
      <code>style.bg(…)</code>, <code>style.pad(…)</code>,
      <code>style.radius(…)</code>, <code>style.fg(…)</code>, and so on.
    </p>
    <grid cols="2" gap="2">
      <for {sxRows()} as="a" key={a.prop}>
        <card padding="sm" sx={style.minW(0)}>
          <column gap="1">
            <code>{a.prop}</code>
            <small sx={style.fg(muted-foreground).maxW(panel-max-w-sm)}>{a.values}</small>
          </column>
        </card>
      </for>
    </grid>
  </column>

  <column gap="3">
    <h3>Not Style — use this ({refusalCount()})</h3>
    <p sx={style.maxW(breakpoint-tablet)}>
      The refused table is the other half of the vocabulary. Every entry below
      is something a Tailwind prior will reach for and Sail refuses by name;
      each names the element or attribute to use instead.
    </p>
    <grid cols="2" gap="2">
      <for {refusals()} as="r" key={r.atom}>
        <card padding="sm" sx={style.minW(0)}>
          <column gap="1">
            <code sx={style.fg(destructive)}>{r.atom}</code>
            <small sx={style.fg(muted-foreground).maxW(panel-max-w-sm)}>→ {r.instead}</small>
          </column>
        </card>
      </for>
    </grid>
  </column>

  <column gap="3">
    <h3>when keys — interaction &amp; ARIA</h3>
    <p sx={style.maxW(breakpoint-tablet).fontSize(sm).fg(muted-foreground)}>
      Product form: <code>when(&#123; hover: style.bg(muted), current: style.fg(primary) &#125;)</code>.
      Historical prefix names below map 1:1 to those keys.
    </p>
    <grid cols="3" gap="2">
      <for {statePrefixes()} as="s" key={s.prefix}>
        <row gap="3" sx={style.padX(3).padY(1).radius(md)} align="center">
          <code sx={style.minW(9)}>{s.prefix}</code>
          <small sx={style.fg(muted-foreground).fontFamily(mono).maxW(ch-24)}>{s.selector}</small>
        </row>
      </for>
    </grid>
    <p sx={style.maxW(breakpoint-tablet)}>
      Ancestor propagation: mark the ancestor with <code>style.group()</code>,
      then <code>when(&#123; groupHover: … &#125;)</code> on the descendant.
    </p>
  </column>

  <column gap="3">
    <h3>when keys — media conditions</h3>
    <p sx={style.maxW(breakpoint-tablet).fontSize(sm).fg(muted-foreground)}>
      Breakpoints are locked to design-token <code>breakpoint-*</code> lengths.
      Use <code>when(&#123; tablet: style.fontSize(lg), desktop: style.fontSize(xl) &#125;)</code>.
    </p>
    <grid cols="3" gap="2">
      <for {conditionPrefixes()} as="c" key={c.prefix}>
        <row gap="3" sx={style.padX(3).padY(1).radius(md)} align="center">
          <code sx={style.minW(9)}>{c.prefix}</code>
          <small sx={style.fg(muted-foreground).fontFamily(mono).maxW(ch-24)}>{c.media}</small>
        </row>
      </for>
    </grid>
  </column>

  <column gap="3">
    <h3>Composition &amp; bind</h3>
    <p sx={style.maxW(breakpoint-tablet)}>
      <code>match(prop, &#123; … &#125;)</code> branches on a prop value;
      <code>compound(&#123; … &#125;, style…)</code> intersects prop arms;
      <code>bind(signalName)</code> reads a controller signal as a CSS custom
      property (timeline tracks, range-driven paint).
    </p>
    <grid cols="2" gap="2">
      <for {extraForms()} as="f" key={f.form}>
        <card padding="sm" sx={style.minW(0)}>
          <column gap="1">
            <code>{f.form}</code>
            <small sx={style.fg(muted-foreground).maxW(panel-max-w-sm)}>{f.doc}</small>
          </column>
        </card>
      </for>
    </grid>
  </column>

  <row role="navigation">
    <a variant="default" href="/sail/reference/layout">← Layout keywords</a>
    <a variant="solid" href="/sail/reference/motion">Motion →</a>
  </row>
</column>
