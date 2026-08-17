<!--
  /docs/concepts/styling — Style product: object Style, name =, when/match/
  compound, bind. Live demos use L1 tags that already show the behavior;
  teaching text stays as style&#40; / code-block.
-->

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { code } from '../../../../src/compositions/sail-language-styling-snippets'

export default controller((_ctx: ControllerScope<Scope>) => {
  const shift = signal('0')
  return { code, shift }
})
</controller>

<article>
  <column gap="5">
    <p>The language</p>
    <h1>Styling</h1>
    <p>
      Paint is the <strong>Style</strong> language:
      <code>style&#40;&#123; pad(4).bg(surface-1) &#125;&#41;</code> on the host,
      with optional <code>when</code>, <code>match</code>, and <code>compound</code>
      keys for conditionality, plus named Style
      (<code>style&#32;name = &#123; … &#125;</code>). Values are design tokens —
      never raw hex. The compiler and the
      <a href="/docs/reference/sx">Style reference</a> share one table.
    </p>
    <p>
      Familiar spacing and color ideas still apply
      (<code>pad(4)</code>, <code>bg(surface-1)</code>, <code>radius(lg)</code>),
      but the product form is a typed setter chain, not a utility string. Closed and
      token-backed is what makes the theme switch in this site’s header work —
      swap the theme and everything repaints, because nothing held a raw color.
    </p>

    <h2>The chain</h2>
    <column gap="2">
      <p>This surface is styled by the code below it.</p>
    </column>
    <code-block code={code.sx}></code-block>
    <p>
      A value off the system is a compile error, and the error names what the
      scale holds:
    </p>
    <code-block code={code.sxError}></code-block>
    <p>
      Prefer a named Style when the same paint repeats:
      <code>style&#32;card = &#123; pad(5).bg(surface-1) &#125;</code> then
      attach <code>style&#40;card&#41;</code> or compose Style pieces
      <code>style&#40;base, emphasis&#41;</code> on the host. (That is paint
      composition — not an in-file <code>&lt;partial&gt;</code>, which reuses
      template structure; see
      <a href="/docs/concepts/templates">templates</a>.)
    </p>

    <h2>States with when</h2>
    <p>
      Browser and ARIA state are a map, not string prefixes. Hover these —
      <code>&lt;button&gt;</code> already lifts on hover:
    </p>
    <row gap="3">
      <button>hover → lift</button>
      <button variant="outline" tone="primary">hover → outline</button>
      <button variant="solid" tone="primary">active: press</button>
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
      <code>style&#40;&#123; when: &#123; tablet: …, desktop: … &#125; &#125;&#41;</code>. A
      <code>&lt;card&gt;</code> already owns pad + surface:
    </p>
    <p>pad 3 on a phone; when tablet → pad 6 — written as Style in L1/L2, not in a route.</p>
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
      setter is px. Drag — the value is live; the tile paint lives in L1/L2:
    </p>
    <column gap="3">
      <slider min="0" max="200" step="1" bind:value={shift} />
      <row gap="2" align="center">
        <small>{shift()} px</small>
      </row>
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
      <code>style&#40;&#123; pad(4).bg(muted) &#125;&#41;</code>. When the vocabulary can’t express
      something you need, file a gap: the system grows a setter or token, not a
      hole.
    </p>

    <p>
      For the full surface: <a href="/docs/reference/sx">reference → Style</a>
      · <a href="/docs/reference/regions">region matrix</a>.
    </p>

    <separator />
    <row gap="2">
      <a variant="default" href="/docs/concepts/templates">← Templates</a>
      <a variant="solid" tone="primary" href="/docs/concepts/layout">Layout →</a>
    </row>
  </column>
</article>
