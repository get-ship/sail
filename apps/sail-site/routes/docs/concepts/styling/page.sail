<!--
  /docs/concepts/styling — Style product: object Style, name =, when/match/
  compound, bind. Live demos paint with Style; samples match.
-->

<scope>
  style page = { box: maxW(breakpoint-tablet).mx(auto) }
  style eyebrow = { box: mb(2), paint: fg(accent), text: fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide) }
  style lead = { paint: fg(muted-foreground), text: fontSize(lg).leading(relaxed) }
  style demoCard = { box: pad(5), paint: bg(surface-1).borderWidth(1).border(border).radius(lg).shadow(sm) }
  style demoMuted = { paint: fg(muted-foreground), text: fontSize(sm) }
  style demoPanel = { box: pad(5), paint: bg(surface-1).borderWidth(1).border(border).radius(lg) }
  style responsivePanel = { box: pad(3), paint: bg(surface-1).borderWidth(1).border(border).radius(lg), when: { tablet: { box: pad(6) } } }
  style bodySm = { text: fontSize(sm) }
  style rangeTrack = { box: minW(0) }
  style shiftTile = { box: w(9).h(9), paint: bg(primary).radius(md).translateX(bind(shiftPx)) }
  style footNote = { box: mt(4), paint: fg(muted-foreground), text: fontSize(sm) }
  style footNav = { box: mt(8).padT(4), paint: borderTWidth(1).borderT(border) }
</scope>

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { code } from '../../../../src/compositions/sail-language-styling-snippets'

export default controller((_ctx: ControllerScope<Scope>) => {
  // Range bind:value is string; number derived feeds translateX(bind) as px.
  const shift = signal('0')
  const shiftPx = derived((): number => Number(shift()) || 0)
  return { code, shift, shiftPx }
})
</controller>

<column gap="5" style(page)>
  <p style(eyebrow)>The language</p>
  <h1>Styling</h1>
  <p style(lead)>
    Paint is the <strong>Style</strong> channel language:
    <code>style&#40;&#123; box, paint, text, animate &#125;&#41;</code> on the host,
    with optional <code>when</code>, <code>match</code>, and <code>compound</code>
    keys for conditionality, plus named Style
    (<code>style name = &#123; … &#125;</code>). Values are design tokens —
    never raw hex. The compiler and the
    <a href="/docs/reference/sx">Style reference</a> share one table.
  </p>
  <p>
    Familiar spacing and color ideas still apply
    (<code>pad(4)</code>, <code>bg(surface-1)</code>, <code>radius(lg)</code>),
    but the product form is a typed channel spine, not a utility string. Closed and
    token-backed is what makes the theme switch in this site’s header work —
    swap the theme and everything repaints, because nothing held a raw color.
  </p>

  <h2>The chain</h2>
  <card style(demoCard)>
    <p style(demoMuted)>This card is styled by the code below it.</p>
  </card>
  <code-block code={code.sx}></code-block>
  <p>
    A value off the system is a compile error, and the error names what the
    scale holds:
  </p>
  <code-block code={code.sxError}></code-block>
  <p>
    Prefer a named Style when the same paint repeats:
    <code>style card = &#123; box: pad(5), paint: bg(surface-1) &#125;</code> then
    attach <code>style&#40;card&#41;</code> or compose Style pieces
    <code>style&#40;base, emphasis&#41;</code> on the host. (That is paint
    composition — not an in-file <code>&lt;partial&gt;</code>, which reuses
    template structure; see
    <a href="/docs/concepts/templates">templates</a>.)
  </p>

  <h2>States with when</h2>
  <p>
    Browser and ARIA state are a map, not string prefixes. Hover these:
  </p>
  <row gap="3" style(demoPanel)>
    <button style({ when: { hover: { paint: bg(muted) } } })>hover → bg(muted)</button>
    <button style({ when: { hover: { paint: fg(primary) } } })>hover → fg(primary)</button>
    <button style({ when: { active: { paint: bg(primary).fg(primary-foreground) } } })>active:…</button>
  </row>
  <code-block code={code.states}></code-block>
  <p>
    Keys include <code>hover</code>, <code>focus</code> (→
    <code>:focus-visible</code>), <code>focusWithin</code>, <code>active</code>,
    <code>disabled</code>, and ARIA-backed states components already stamp:
    <code>current</code>, <code>selected</code>, <code>expanded</code>,
    <code>checked</code>, <code>pressed</code>. This site’s product switcher:
  </p>
  <code-block code={code.current}></code-block>

  <h2>Responsive and preference with when</h2>
  <p>
    Breakpoints are Style product conditions locked to
    <code>breakpoint-*</code> tokens — not <code>tablet:</code> string
    prefixes. Unprefixed paint is the phone baseline; layer larger steps with
    <code>style&#40;&#123; when: &#123; tablet: …, desktop: … &#125; &#125;&#41;</code>. Resize to see this panel
    pad more from 768px up:
  </p>
  <column gap="2" style(responsivePanel)>
    <p style(bodySm)>pad(3) on a phone; when tablet → pad(6).</p>
  </column>
  <code-block code={code.conditions}></code-block>
  <p>
    Preference media uses the same map:
    <code>dark</code>, <code>light</code>, <code>motionReduce</code>,
    <code>motionSafe</code>, <code>forced-colors</code> — closed set.
  </p>

  <h2>Variants with match and compound</h2>
  <p>
    Named looks are data: typed props plus Style pieces composed with
    <code>match:</code> and <code>compound:</code> keys — not CVA string maps or
    free <code>match&#40;&#41;</code> calls. Each arm is an ordinary Style value, same
    pipeline as a host <code>style&#40;&#123; … &#125;&#41;</code>. Variance
    authoring for libraries may still need a directory permit; plain pages
    usually paint with named Style + <code>match:</code> only.
  </p>
  <code-block code={code.variants}></code-block>
  <p>Callers pick a look the way they set any field:</p>
  <code-block code={code.variantUse}></code-block>

  <h2>Styles can be dynamic too</h2>
  <p>
    Computed paint flows through signals. Style product reads them with
    <code>bind&#40;name&#41;</code> — the compiler registers the CSS custom property
    and reflects the signal onto the host. A number bound into a length
    setter is px. Drag:
  </p>
  <column gap="3" style(demoPanel)>
    <slider min="0" max="200" step="1" bind:value={shift} style(rangeTrack) />
    <card style(shiftTile)></card>
  </column>
  <code-block code={code.dynamic}></code-block>
  <p>
    State stays in the controller and Style reads it — the
    <a href="/docs/concepts/animation">animation chapter</a> drives whole
    choreographies through this channel.
  </p>

  <h2>What Style does not do</h2>
  <p>
    Two things sit outside paint on purpose. <strong>Layout</strong> — parent
    elements decide how children sit (<a href="/docs/concepts/layout">layout
    elements</a>). And <strong>authored CSS</strong> —
    <code>&lt;style&gt;</code> blocks and <code>style=</code> attributes are
    compile errors in application code. Utility-string hosts
    (<code>sx="…"</code> / <code>style="…"</code>) are refused the same way —
    dead forms; product paint is
    <code>style&#40;&#123; box: pad(4), paint: bg(muted) &#125;&#41;</code>. When the vocabulary can’t express
    something you need, file a gap: the system grows a setter or token, not a
    hole.
  </p>

  <p style(footNote)>
    For the full surface: <a href="/docs/reference/sx">reference → Style</a>
    · <a href="/docs/reference/regions">region matrix</a>.
  </p>

  <row justify="between" gap="2" style(footNav)>
    <a variant="default" href="/docs/concepts/templates">← Templates</a>
    <a variant="solid" tone="primary" href="/docs/concepts/layout">Layout →</a>
  </row>
</column>
