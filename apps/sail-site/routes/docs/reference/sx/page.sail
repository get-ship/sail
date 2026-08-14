<!--
  /docs/reference/sx — Style surface (URL slug is historical).
  Product: style({ pad(4).bg(surface), when: { hover: bg(muted) } }) +
  style name = / bind / match: / compound: / when:.
  Atlas is STYLE_VOCAB (offered_rows — concern | axis | plain).
-->
<scope>
  style root = { size({ min-w: 0 }) }
  style intro = { size({ max-w: breakpoint-tablet }) }
  style mutedSm = { size({ max-w: breakpoint-tablet }).fg(muted-foreground).type({ size: sm }) }
  style mutedBody = { size({ max-w: breakpoint-tablet }) }
  style cardShell = { size({ min-w: 0 }) }
  style valueText = { size({ max-w: panel-max-w-sm }).fg(muted-foreground) }
  style prefixRow = { pad({ x: 3, y: 1 }).radius(md) }
  style prefixCode = { size({ min-w: 9 }) }
  style mediaLine = { size({ max-w: ch-24 }).fg(muted-foreground).type({ family: mono }) }
</scope>

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import {
  STYLE_VOCAB,
  CONCERNS,
  SX_STATE_PREFIXES,
  SX_CONDITION_PREFIXES,
} from '../../../../src/compositions/language-surface'

function vocabRows(tag: 'concern' | 'axis' | 'plain') {
  return STYLE_VOCAB.filter((r) => r.tag === tag).map((r) => ({
    name: r.name,
    channel: r.channel,
    slots: r.slots.length ? r.slots.join(' · ') : '—',
  }))
}

export default controller((_ctx: ControllerScope<Scope>) => ({
  concerns: () => vocabRows('concern'),
  axes: () => vocabRows('axis'),
  plains: () => vocabRows('plain'),
  styleCount: () => STYLE_VOCAB.length,
  concernCount: () => CONCERNS.length,
  statePrefixes: () => SX_STATE_PREFIXES,
  conditionPrefixes: () => SX_CONDITION_PREFIXES,
}))
</controller>

<column gap="6" style(root)>
  <column gap="5" style(intro)>
    <p token="doc-eyebrow">Reference · Style</p>
    <h1>Style — tag attach</h1>
    <p token="doc-lead">
      Product paint is a <strong>tag-position call</strong> on the open tag:
      <code>style&#40;&#123; pad(4).bg(surface), when: &#123; hover: bg(muted) &#125; &#125;&#41;</code>.
      Reuse named Style in scope with
      <code>style name = &#123; … &#125;</code>, attach as
      <code>style&#40;name&#41;</code>, and compose pieces
      <code>style&#40;base, emphasis&#41;</code>. The rows below are the one
      Style vocab table — concern · axis · plain.
    </p>
    <p style(mutedSm)>
      Utility-string <code>style="…"</code> / <code>sx="…"</code> is not product
      (dead form — refuse). Responsive and state paint use object keys:
      <code>style&#40;&#123; when: &#123; tablet: type(&#123; size: lg &#125;), hover: bg(muted) &#125; &#125;&#41;</code>.
      {styleCount()} offered rows · {concernCount()} concerns. Region legality lives on the
      <a href="/docs/reference/regions">region matrix</a>.
    </p>
  </column>

  <column gap="3">
    <h3>Concerns</h3>
    <p style(mutedSm)>
      Named composite or slot object. Flat chain is the language:
      <code>style&#40;&#123; bg(surface).pad(4) &#125;&#41;</code>.
    </p>
    <grid cols="2" gap="2">
      <for {concerns() as a} key={a.name}>
        <card padding="sm" style(cardShell)>
          <column gap="1">
            <code>{a.name}</code>
            <small style(valueText)>{a.channel} · {a.slots}</small>
          </column>
        </card>
      </for>
    </grid>
  </column>

  <column gap="3">
    <h3>Axis</h3>
    <p style(mutedSm)>
      Directional families — not slots of <code>size</code> or <code>border</code>.
    </p>
    <grid cols="2" gap="2">
      <for {axes() as a} key={a.name}>
        <card padding="sm" style(cardShell)>
          <column gap="1">
            <code>{a.name}</code>
            <small style(valueText)>{a.channel} · {a.slots}</small>
          </column>
        </card>
      </for>
    </grid>
  </column>

  <column gap="3">
    <h3>Plains</h3>
    <p style(mutedSm)>
      Verbs and singles. One name, one call.
    </p>
    <grid cols="2" gap="2">
      <for {plains() as a} key={a.name}>
        <card padding="sm" style(cardShell)>
          <column gap="1">
            <code>{a.name}</code>
            <small style(valueText)>{a.channel}</small>
          </column>
        </card>
      </for>
    </grid>
  </column>

  <column gap="3">
    <h3>when keys — interaction &amp; ARIA</h3>
    <p style(mutedSm)>
      Product form:
      <code>style&#40;&#123; when: &#123; hover: bg(muted), current: fg(primary) &#125; &#125;&#41;</code>.
      Keys below map 1:1 to those entries.
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
      Ancestor propagation: mark the ancestor with <code>style&#40;&#123; group() &#125;&#41;</code>,
      then <code>style&#40;&#123; when: &#123; groupHover: &#123; … &#125; &#125; &#125;&#41;</code> on the descendant.
    </p>
  </column>

  <column gap="3">
    <h3>when keys — media conditions</h3>
    <p style(mutedSm)>
      Breakpoints are locked to design-token <code>breakpoint-*</code> lengths.
      Use
      <code>style&#40;&#123; when: &#123; tablet: type(&#123; size: lg &#125;), desktop: type(&#123; size: xl &#125;) &#125; &#125;&#41;</code>.
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
      Prefer <code>style name = &#123; … &#125;</code> in
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
