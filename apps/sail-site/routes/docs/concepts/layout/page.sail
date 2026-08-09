<!--
  /docs/concepts/layout — chapter 5, round 2 per owner review: lead with WHY
  one mechanism (many ways exist; one spelling is what tools and agents can
  reason about), render EVERY element live, explain in sentences not
  keyword-lists, no "flow not clamp" jargon, no s-refusal opener (that
  lives on the styling page), no whole-vocabulary overclaim.
-->

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { code } from '../../../../src/compositions/sail-language-layout-snippets'

export default controller((_ctx: ControllerScope<Scope>) => {
  return { code }
})
</controller>

<scope>
  style page = { box: maxW(breakpoint-tablet).mx(auto) }
  style eyebrow = { box: mb(2), paint: fg(accent), text: fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide) }
  style lead = { paint: fg(muted-foreground), text: fontSize(lg).leading(relaxed) }
  style demo = { box: pad(4), paint: bg(surface-1).borderWidth(1).border(border).radius(md) }
  style demoCenter = { box: minH(11).pad(4), paint: bg(surface-1).borderWidth(1).border(border).radius(md) }
  style cardPad = { box: pad(4) }
  style cardPadSm = { box: pad(3) }
  style stackCard = { box: w(24).h(12), paint: bg(muted).radius(md) }
  style stackDot = { box: w(4).h(4), paint: radius(full).bg(primary) }
  style growMin = { box: grow(1).minW(0) }
  style shrink0 = { box: shrink(0) }
  style footNote = { box: mt(4), paint: fg(muted-foreground), text: fontSize(sm) }
  style footNav = { box: mt(8).padT(4), paint: borderTWidth(1).borderT(border) }
</scope>

<column gap="5" style(page)>
  <p style(eyebrow)>The language</p>
  <h1>Layout</h1>
  <p style(lead)>
    The web has accumulated half a dozen ways to lay things out — floats,
    positioning, flexbox, grid, and the properties that tune them. Sail
    ships <strong>one</strong> mechanism: five layout elements —
    <code>&lt;row&gt;</code>, <code>&lt;column&gt;</code>,
    <code>&lt;grid&gt;</code>, <code>&lt;stack&gt;</code>,
    <code>&lt;center&gt;</code>. Everything those systems express —
    wrapping, two dimensions, spanning, layering, centering — is still
    expressible; what’s gone is the second and third way to spell it.
  </p>
  <p>
    One spelling is the point. When every layout in a codebase is a handful
    of elements with named axes, geometry is readable from source — by the
    compiler, which checks every attribute and redirects wrong turns, and
    by an agent, which sees the same five words in every file it opens
    instead of six eras of CSS technique.
  </p>

  <h2><code>&lt;row&gt;</code></h2>
  <p>
    A row lays children out left to right. <code>gap</code> spaces them
    (spacing-scale steps), <code>align</code> positions them on the cross
    axis, <code>justify</code> distributes them along the row:
  </p>
  <row gap="3" align="center" justify="between" style(demo)>
    <strong>Ada Lovelace</strong>
    <badge tone="success">Active</badge>
  </row>
  <code-block code={code.row}></code-block>
  <p>
    When a row runs out of width, its children move to the next line —
    rows <strong>wrap by default</strong>, like text does. Watch this one
    as the window narrows:
  </p>
  <row gap="2" style(demo)>
    <chip>signals</chip>
    <chip>derived</chip>
    <chip>effects</chip>
    <chip>stores</chip>
    <chip>compositions</chip>
    <chip>actions</chip>
    <chip>tones</chip>
    <chip>variants</chip>
  </row>
  <p>
    Some rows must stay on one line no matter what — a toolbar, a sidebar
    beside content. Those say <code>nowrap</code>, and manage width with
    their children instead:
  </p>
  <code-block code={code.nowrap}></code-block>

  <h2><code>&lt;column&gt;</code></h2>
  <p>
    A column is the same idea, top to bottom. <code>gap</code> spaces the
    children; <code>align</code> positions them horizontally — the label
    and input below are <code>align="start"</code>, so they hug the left
    edge instead of stretching:
  </p>
  <column gap="2" align="start" style(demo)>
    <label>Email</label>
    <input type="email" placeholder="ada@example.com" />
    <button>Subscribe</button>
  </column>
  <code-block code={code.column}></code-block>

  <h2><code>&lt;grid&gt;</code></h2>
  <p>
    A grid lays children out in columns and rows. <code>cols</code> fixes
    the column count — every child takes one cell, wrapping to new rows as
    needed:
  </p>
  <grid cols="3" gap="3">
    <card style(cardPad)><strong>One</strong></card>
    <card style(cardPad)><strong>Two</strong></card>
    <card style(cardPad)><strong>Three</strong></card>
  </grid>
  <code-block code={code.grid}></code-block>
  <p>
    <code>min</code> is the responsive alternative to a fixed count: give a
    minimum column width, and the grid fits as many columns of at least
    that width as the space allows — fewer on a phone, more on a desktop,
    no media query written.
  </p>
  <p>
    A child can cover more than one cell with the layout attrs
    <code>col-span</code> / <code>row-span</code> — language attrs, not
    <code>data-*</code>. Below 720px, spans collapse and every child goes
    back to one cell, so a wide card never forces a phone to scroll
    sideways:
  </p>
  <grid cols="4" gap="3">
    <card col-span="2" style(cardPadSm)>wide — spans 2</card>
    <card style(cardPadSm)>one</card>
    <card style(cardPadSm)>two</card>
  </grid>
  <code-block code={code.spans}></code-block>

  <h2><code>&lt;stack&gt;</code></h2>
  <p>
    A stack places children on top of each other in the same cell — the
    overlay cases: a badge on an avatar, a caption on an image, a dot on an
    icon.
  </p>
  <row style(demo)>
    <stack align="start" justify="end">
      <card style(stackCard)></card>
      <div style(stackDot)></div>
    </stack>
  </row>
  <code-block code={code.stack}></code-block>

  <h2><code>&lt;center&gt;</code></h2>
  <p>
    Centers its child, horizontally and vertically, in whatever space the
    element has:
  </p>
  <center style(demoCenter)>
    <badge tone="primary">centered</badge>
  </center>
  <code-block code={code.center}></code-block>

  <h2>How children size themselves</h2>
  <p>
    The parent decides how children sit; a child decides how it fills the
    space it was given on the <strong>box channel</strong> — legal on any
    element, not only layout keywords:
  </p>
  <row gap="3" align="center" style(demo)>
    <strong style(growMin)>Ada Lovelace</strong>
    <badge tone="success" style(shrink0)>Active</badge>
  </row>
  <code-block code={code.grow}></code-block>
  <p>
    Participation is one spelling: the box channel.
    <code>grow(1)</code> / <code>shrink(0)</code> are flex participation;
    <code>w(full)</code>, <code>minW(0)</code>, and <code>maxW(…)</code> take
    the same size tokens as other Style box setters. Bare attrs
    (<code>grow</code>, <code>min-w=</code>, <code>shrink</code>) are refused
    (<code>sail-participation-style</code>) — write
    <code style(growMin)></code> instead.
  </p>
  <p>
    Layout-keyword <strong>axis</strong> sugar is separate and intentional:
    <code>gap</code> / <code>align</code> / <code>justify</code> stay on
    <code>&lt;row&gt;</code> / <code>&lt;column&gt;</code> /
    <code>&lt;grid&gt;</code>. Style also owns paint (pad, color, type,
    chrome), intrinsic box size (<code>h</code>, icon
    <code>w</code>/<code>size</code>), and chrome like <code>minH</code> on
    badges — all as a block Style statement. Every child of a layout element
    is also born with <code>min-width: 0</code> — the browser default where an
    item refuses to shrink below its content is corrected at the language
    level, so <code style({ text: truncate() })></code> works the first time.
  </p>

  <h2><code>&lt;when&gt;</code> — different trees per breakpoint</h2>
  <p>
    When the layout itself changes shape — a side-by-side workbench on tablet,
    a stacked column on mobile — that is a <strong>tree</strong> decision, not
    paint. <code>&lt;when &#123;tablet&#125;&gt;</code> takes a theme breakpoint
    token (<code>mobile</code> / <code>tablet</code> / <code>desktop</code>) and
    keeps an optional <code>&lt;else&gt;</code> as the default branch:
  </p>
  <code-block code={code.when}></code-block>
  <p>
    The server cannot know the viewport, so <strong>SSR always emits the
    <code>&lt;else&gt;</code> / default branch</strong> (mobile-first). After
    hydrate, the client matchMedia-swaps to the matching branch. No UA sniffing.
    Style <code>.style&#40;&#123; when: &#123; 123: tablet &#125; &#125;&#41;</code> stays paint-level —
    density and color tweaks — and is unchanged.
  </p>

  <p style(footNote)>
    For the full surface: <a href="/docs/reference/layout">reference → Layout keywords</a>.
  </p>

  <row justify="between" gap="2" style(footNav)>
    <a variant="default" href="/docs/concepts/styling">← Styling</a>
    <a variant="solid" tone="primary" href="/docs/concepts/animation">Animation →</a>
  </row>
</column>
