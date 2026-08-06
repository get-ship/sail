<!--
  /sail/language/styling — Style product chapter: style.* chains, match, when,
  compound, scope Style consts. Every live demo paints with Style; code samples
  match. Responsive paint uses when({ tablet|desktop|… }) locked to breakpoint
  tokens — never tablet: utility prefixes.
-->

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { code } from './snippets'

export default controller((_ctx: ControllerScope<Scope>) => {
  // Range writes shift; effect mirrors into px for style.translateX(bind(px)).
  const shift = signal('0')
  const px = signal('0px')
  effect(() => px.set(`${shift()}px`))
  return { code, shift, px }
})
</controller>

<column gap="5">
  style.maxW(breakpoint-tablet).mx(auto)

  <p>
    style.fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide).fg(accent).mb(2)
The language</p>
  <h1>Styling</h1>
  <p>
    style.fontSize(lg).fg(muted-foreground).leading(relaxed)

    Paint is the <strong>Style</strong> value language on <code>sx</code>:
    <code>style.bg(token)</code> chains, <code>match</code>, <code>when</code>,
    <code>compound</code>, and scope Style consts. Values are design tokens —
    never raw hex. The compiler and the
    <a href="/sail/reference/sx">sx reference</a> share one table.
  </p>
  <p>
    Familiar spacing and color ideas still apply
    (<code>pad(4)</code>, <code>bg(surface-1)</code>, <code>radius(lg)</code>),
    but the product form is a typed chain, not a utility string. Closed and
    token-backed is what makes the theme switch in this site's header work —
    swap the theme and everything repaints, because nothing held a raw color.
  </p>

  <h2>The chain</h2>
  <card>
    style.pad(5).bg(surface-1).borderWidth(1).border(border).radius(lg).shadow(sm)

    <p>
      style.fg(muted-foreground).fontSize(sm)
This card is styled by the code below it.</p>
  </card>
  <code-block code={code.sx}></code-block>
  <p>
    A value off the system is a compile error, and the error names what the
    scale holds:
  </p>
  <code-block code={code.sxError}></code-block>
  <p>
    Prefer a scope const when the same paint repeats:
    <code>const card = style.pad(5).bg(surface-1)…</code> then
    <code>sx=&#123;card&#125;</code> or <code>sx=&#123;[base, emphasis]&#125;</code>.
  </p>

  <h2>States with when</h2>
  <p>
    Browser and ARIA state are a map, not string prefixes. Hover these:
  </p>
  <row gap="3">
    style.pad(5).bg(surface-1).borderWidth(1).border(border).radius(lg)

    <button>
      when({ hover: style.bg(muted) })
hover → bg(muted)</button>
    <button>
      when({ hover: style.fg(primary) })
hover → fg(primary)</button>
    <button>
      when({ active: style.bg(primary).fg(primary-foreground) })
active:…</button>
  </row>
  <code-block code={code.states}></code-block>
  <p>
    Keys include <code>hover</code>, <code>focus</code> (→
    <code>:focus-visible</code>), <code>focusWithin</code>, <code>active</code>,
    <code>disabled</code>, and ARIA-backed states components already stamp:
    <code>current</code>, <code>selected</code>, <code>expanded</code>,
    <code>checked</code>, <code>pressed</code>. This site's product switcher:
  </p>
  <code-block code={code.current}></code-block>

  <h2>Responsive and preference with when</h2>
  <p>
    Breakpoints are Style product conditions locked to
    <code>breakpoint-*</code> tokens — not <code>tablet:</code> string
    prefixes. Unprefixed paint is the phone baseline; layer larger steps with
    <code>when(&#123; tablet: …, desktop: … &#125;)</code>. Resize to see this panel
    pad more from 768px up:
  </p>
  <column gap="2">
    style.pad(3).bg(surface-1).borderWidth(1).border(border).radius(lg).when({ tablet: style.pad(6) })

    <p>
      style.fontSize(sm)
pad(3) on a phone; when tablet → pad(6).</p>
  </column>
  <code-block code={code.conditions}></code-block>
  <p>
    Preference media uses the same map:
    <code>dark</code>, <code>light</code>, <code>motionReduce</code>,
    <code>motionSafe</code>, <code>forced-colors</code> — closed set.
  </p>

  <h2>Variants with match and compound</h2>
  <p>
    Named looks are data: typed props plus Style fragments composed with
    <code>match</code> and <code>compound</code> — not CVA string maps. Each
    arm is ordinary Style, same pipeline as host <code>sx</code>. Variance
    authoring for libraries may still need a directory permit; plain pages
    usually paint with consts + match only.
  </p>
  <code-block code={code.variants}></code-block>
  <p>Callers pick a look the way they set any field:</p>
  <code-block code={code.variantUse}></code-block>

  <h2>Styles can be dynamic too</h2>
  <p>
    Computed paint flows through signals. Style product reads them with
    <code>bind(name)</code> — the compiler registers the CSS custom property
    and reflects the signal onto the host. Drag:
  </p>
  <column gap="3">
    style.pad(5).bg(surface-1).borderWidth(1).border(border).radius(lg)

    <input type="range" min="0" max="200" step="1" bind:value={shift}>
      style.minW(0)
    </input>
    <card>
      style.w(9).h(9).bg(primary).radius(md).translateX(bind(px))
</card>
  </column>
  <code-block code={code.dynamic}></code-block>
  <p>
    State stays in the controller and Style reads it — the
    <a href="/sail/language/animation">animation chapter</a> drives whole
    choreographies through this channel.
  </p>

  <h2>What Style does not do</h2>
  <p>
    Two things sit outside paint on purpose. <strong>Layout</strong> — parent
    elements decide how children sit (<a href="/sail/language/layout">layout
    elements</a>). And <strong>authored CSS</strong> —
    <code>&lt;style&gt;</code> blocks and <code>style=</code> attributes are
    compile errors in application code. Utility-string
    <code>sx="p-4 bg-muted"</code> is not the product either — use
    <code>style.pad(4).bg(muted)</code>. When the vocabulary can't express
    something you need, file a gap: the system grows a setter or token, not a
    hole.
  </p>

  <p>
    style.mt(4).fontSize(sm).fg(muted-foreground)

    For the full surface: <a href="/sail/reference/sx">reference → sx</a>.
  </p>

  <row justify="between" gap="2">
    style.mt(8).padT(4).borderTWidth(1).borderT(border)

    <a variant="default" href="/sail/language/templates">← Templates</a>
    <a variant="solid" tone="primary" href="/sail/language/layout">Layout →</a>
  </row>
</column>
