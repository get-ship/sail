<!--
  /docs/reference/sx — Style channels surface (URL slug is historical).
  Product: style({ box, paint, text, animate }) + style name = / bind /
  match: / compound: / when:. Setter scales, when keys, refusals.
-->
<scope>
  style root = { box: minW(0) }
  style intro = { box: maxW(breakpoint-tablet) }
  style mutedSm = { box: maxW(breakpoint-tablet), paint: fg(muted-foreground), text: fontSize(sm) }
  style mutedBody = { box: maxW(breakpoint-tablet) }
  style cardShell = { box: minW(0) }
  style valueText = { box: maxW(panel-max-w-sm), paint: fg(muted-foreground) }
  style banCode = { paint: fg(destructive) }
  style prefixRow = { box: padX(3).padY(1), paint: radius(md) }
  style prefixCode = { box: minW(9) }
  style mediaLine = { box: maxW(ch-24), paint: fg(muted-foreground), text: fontFamily(mono) }
</scope>

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import {
  SX_ATOMS,
  SX_STATE_PREFIXES,
  SX_CONDITION_PREFIXES,
  SX_REFUSALS,
} from '../../../../src/compositions/language-surface'

// Some atoms share a prop name across tables (e.g. spacing and color senses
// both named `border`) — merge value lists under one card.
const STYLE_BY_PROP = new Map<string, string[]>()
for (const a of SX_ATOMS) {
  const existing = STYLE_BY_PROP.get(a.prop)
  if (existing) {
    existing.push(...a.values)
  } else {
    STYLE_BY_PROP.set(a.prop, [...a.values])
  }
}
const STYLE_ROWS = [...STYLE_BY_PROP.entries()].map(([prop, values]) => ({
  prop,
  values: values.join(' · '),
}))

export default controller((_ctx: ControllerScope<Scope>) => ({
  styleRows: () => STYLE_ROWS,
  styleCount: () => SX_ATOMS.length,
  statePrefixes: () => SX_STATE_PREFIXES,
  conditionPrefixes: () => SX_CONDITION_PREFIXES,
  refusals: () => SX_REFUSALS,
  refusalCount: () => SX_REFUSALS.length,
}))
</controller>

<column gap="6" style(root)>
  <column gap="5" style(intro)>
    <p token="doc-eyebrow">Reference · Style channels</p>
    <h1>Style — block channel attach</h1>
    <p token="doc-lead">
      Product paint is a <strong>block statement</strong> on the host with a
      channel spine:
      <code>style&#40;&#123; box: padX(2), paint: bg(primary), text: fontSize(sm), animate: enter(&#123;…&#125;) &#125;&#41;</code>.
      Reuse named Style in scope with
      <code>style name = &#123; … &#125;</code>, attach as
      <code>style&#40;name&#41;</code>, and compose pieces
      <code>style&#40;base, emphasis&#41;</code>. Every setter argument is a
      design token from a closed scale — the rows below list those values. An
      off-list token is a compile error that names the same list.
    </p>
    <p style(mutedSm)>
      Utility-string <code>style="…"</code> / <code>sx="…"</code> is not product
      (dead form — refuse). Responsive and state paint use object keys:
      <code>style&#40;&#123; when: &#123; tablet: &#123; text: fontSize(lg) &#125;, hover: &#123; paint: bg(muted) &#125; &#125; &#125;&#41;</code>
      — not <code>tablet:</code> / <code>hover:</code> prefixes on atoms.
      {styleCount()} scale entries feed the setters. Region legality lives on the
      <a href="/docs/reference/regions">region matrix</a>.
    </p>
  </column>

  <column gap="3">
    <h3>Setter value scales</h3>
    <p style(mutedSm)>
      Rows are named by historical atom heads (<code>bg</code>, <code>p</code>,
      <code>rounded</code>, …). In product form you call the Style setter:
      <code>style&#40;&#123; paint: bg(…) &#125;&#41;</code>, <code>style&#40;&#123; box: pad(…) &#125;&#41;</code>,
      <code>style&#40;&#123; paint: radius(…) &#125;&#41;</code>, <code>style&#40;&#123; paint: fg(…) &#125;&#41;</code>, and so on.
    </p>
    <grid cols="2" gap="2">
      <for {styleRows() as a} key={a.prop}>
        <card padding="sm" style(cardShell)>
          <column gap="1">
            <code>{a.prop}</code>
            <small style(valueText)>{a.values}</small>
          </column>
        </card>
      </for>
    </grid>
  </column>

  <column gap="3">
    <h3>Not Style — use this ({refusalCount()})</h3>
    <p style(mutedBody)>
      The refused table is the other half of the vocabulary. Every entry below
      is something a Tailwind prior will reach for and Sail refuses by name;
      each names the element or attribute to use instead.
    </p>
    <grid cols="2" gap="2">
      <for {refusals() as r} key={r.atom}>
        <card padding="sm" style(cardShell)>
          <column gap="1">
            <code style(banCode)>{r.atom}</code>
            <small style(valueText)>→ {r.instead}</small>
          </column>
        </card>
      </for>
    </grid>
  </column>

  <column gap="3">
    <h3>when keys — interaction &amp; ARIA</h3>
    <p style(mutedSm)>
      Product form:
      <code>style&#40;&#123; when: &#123; hover: &#123; paint: bg(muted) &#125;, current: &#123; paint: fg(primary) &#125; &#125; &#125;&#41;</code>.
      Historical prefix names below map 1:1 to those keys.
    </p>
    <grid cols="3" gap="2">
      <for {statePrefixes() as s} key={s.prefix}>
        <row gap="3" align="center" style(prefixRow)>
          <code style(prefixCode)>{s.prefix}</code>
          <small style(mediaLine)>{s.selector}</small>
        </row>
      </for>
    </grid>
    <p style(mutedBody)>
      Ancestor propagation: mark the ancestor with <code>style&#40;&#123; paint: group() &#125;&#41;</code>,
      then <code>style&#40;&#123; when: &#123; groupHover: &#123; … &#125; &#125; &#125;&#41;</code> on the descendant.
    </p>
  </column>

  <column gap="3">
    <h3>when keys — media conditions</h3>
    <p style(mutedSm)>
      Breakpoints are locked to design-token <code>breakpoint-*</code> lengths.
      Use
      <code>style&#40;&#123; when: &#123; tablet: &#123; text: fontSize(lg) &#125;, desktop: &#123; text: fontSize(xl) &#125; &#125; &#125;&#41;</code>.
    </p>
    <grid cols="3" gap="2">
      <for {conditionPrefixes() as c} key={c.prefix}>
        <row gap="3" align="center" style(prefixRow)>
          <code style(prefixCode)>{c.prefix}</code>
          <small style(mediaLine)>{c.media}</small>
        </row>
      </for>
    </grid>
  </column>

  <column gap="3">
    <h3>name = · match · compound · bind</h3>
    <p style(mutedBody)>
      Prefer <code>style name = &#123; box: …, paint: … &#125;</code> in
      <code>&lt;scope&gt;</code> when the same paint repeats; attach with
      <code>style&#40;name&#41;</code> or compose
      <code>style&#40;base, emphasis&#41;</code>.
      <code>match:</code> branches on a prop value;
      <code>compound:</code> intersects prop arms;
      <code>bind&#40;signalName&#41;</code> reads a controller signal (timeline
      tracks, range-driven paint). Free
      <code>match&#40;prop, …&#41;</code> call forms and utility-string prefixes
      are not product — keys live inside the Style object.
    </p>
    <grid cols="2" gap="2">
      <card padding="sm" style(cardShell)>
        <column gap="1">
          <code>style name =</code>
          <small style(valueText)>Named Style in scope; attach as style&#40;name&#41; or style&#40;base, extra&#41;</small>
        </column>
      </card>
      <card padding="sm" style(cardShell)>
        <column gap="1">
          <code>match: / compound:</code>
          <small style(valueText)>Object keys on Style — branch and intersect prop arms</small>
        </column>
      </card>
      <card padding="sm" style(cardShell)>
        <column gap="1">
          <code>when:</code>
          <small style(valueText)>Interaction, ARIA, and media conditions as a map</small>
        </column>
      </card>
      <card padding="sm" style(cardShell)>
        <column gap="1">
          <code>bind&#40;name&#41;</code>
          <small style(valueText)>Controller signal → CSS custom property on a setter</small>
        </column>
      </card>
    </grid>
  </column>

  <row>
    <a variant="default" href="/docs/reference/layout">← Layout keywords</a>
    <a variant="solid" href="/docs/reference/motion">Motion →</a>
  </row>
</column>
