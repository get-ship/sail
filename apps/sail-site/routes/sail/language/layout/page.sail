<!--
  /sail/language/layout — chapter 5, round 2 per owner review: lead with WHY
  one mechanism (many ways exist; one spelling is what tools and agents can
  reason about), render EVERY element live, explain in sentences not
  keyword-lists, no "flow not clamp" jargon, no sx-refusal opener (that
  lives on the styling page), no whole-vocabulary overclaim.
-->

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { code } from './snippets'

export default controller((_ctx: ControllerScope<Scope>) => {
  return { code }
})
</controller>

<column sx={style.maxW(breakpoint-tablet).mx(auto)} gap="5">
  <p sx={style.fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide).fg(accent).mb(2)}>The language</p>
  <h1>Layout</h1>
  <p sx={style.fontSize(lg).fg(muted-foreground).leading(relaxed)}>
    The web has accumulated half a dozen ways to lay things out — floats,
    positioning, flexbox, grid, and the properties that tune them. Sail
    ships <strong>one</strong> mechanism: five layout elements —
    <code>&lt;row&gt;</code>, <code>&lt;column&gt;</code>,
    <code>&lt;grid&gt;</code>, <code>&lt;stack&gt;</code>,
    <code>&lt;center&gt;</code>. Everything those systems express —
    wrapping, two dimensions, spanning, layering, centering — is still
    expressible; what's gone is the second and third way to spell it.
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
  <row gap="3" align="center" justify="between" sx={style.pad(4).bg(surface-1).borderWidth(1).border(border).radius(md)}>
    <strong>Ada Lovelace</strong>
    <badge tone="success">Active</badge>
  </row>
  <code-block code={code.row}></code-block>
  <p>
    When a row runs out of width, its children move to the next line —
    rows <strong>wrap by default</strong>, like text does. Watch this one
    as the window narrows:
  </p>
  <row gap="2" sx={style.pad(4).bg(surface-1).borderWidth(1).border(border).radius(md)}>
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
  <column gap="2" align="start" sx={style.pad(4).bg(surface-1).borderWidth(1).border(border).radius(md)}>
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
    <card sx={style.pad(4)}><strong>One</strong></card>
    <card sx={style.pad(4)}><strong>Two</strong></card>
    <card sx={style.pad(4)}><strong>Three</strong></card>
  </grid>
  <code-block code={code.grid}></code-block>
  <p>
    <code>min</code> is the responsive alternative to a fixed count: give a
    minimum column width, and the grid fits as many columns of at least
    that width as the space allows — fewer on a phone, more on a desktop,
    no media query written.
  </p>
  <p>
    A child can cover more than one cell with <code>data-col-span</code> /
    <code>data-row-span</code>. Below 720px, spans collapse and every child
    goes back to one cell, so a wide card never forces a phone to scroll
    sideways:
  </p>
  <grid cols="4" gap="3">
    <card sx={style.pad(3)} data-col-span="2">wide — spans 2</card>
    <card sx={style.pad(3)}>one</card>
    <card sx={style.pad(3)}>two</card>
  </grid>
  <code-block code={code.spans}></code-block>

  <h2><code>&lt;stack&gt;</code></h2>
  <p>
    A stack places children on top of each other in the same cell — the
    overlay cases: a badge on an avatar, a caption on an image, a dot on an
    icon.
  </p>
  <row sx={style.pad(4).bg(surface-1).borderWidth(1).border(border).radius(md)}>
    <stack align="start" justify="end">
      <card sx={style.w(24).h(12).bg(muted).radius(md)}></card>
      <div sx={style.w(4).h(4).radius(full).bg(primary)}></div>
    </stack>
  </row>
  <code-block code={code.stack}></code-block>

  <h2><code>&lt;center&gt;</code></h2>
  <p>
    Centers its child, horizontally and vertically, in whatever space the
    element has:
  </p>
  <center sx={style.minH(11).pad(4).bg(surface-1).borderWidth(1).border(border).radius(md)}>
    <badge tone="primary">centered</badge>
  </center>
  <code-block code={code.center}></code-block>

  <h2>How children size themselves</h2>
  <p>
    The parent decides how children sit; a child decides how it fills the
    space it was given with <strong>layout attributes</strong> — legal on any
    element, not only layout keywords:
  </p>
  <row gap="3" align="center" sx={style.pad(4).bg(surface-1).borderWidth(1).border(border).radius(md)}>
    <strong grow min-w="0">Ada Lovelace</strong>
    <badge tone="success" shrink>Active</badge>
  </row>
  <code-block code={code.grow}></code-block>
  <p>
    Boolean <code>grow</code> / <code>shrink</code> mean flex-grow/shrink 1;
    <code>grow="0"</code> / <code>shrink="0"</code> pin the other end.
    <code>w="full"</code>, <code>min-w</code>, and <code>max-w</code> take the
    same size tokens Style does. These lower to the same atomic classes as
    <code>style.grow(1)</code> / <code>style.w(full)</code> — one cascade.
  </p>
  <p>
    Layout attrs are the product home for participation in a flex/grid parent.
    Style still owns paint (pad, color, type, chrome), intrinsic box size
    (<code>h</code>, icon <code>w</code>/<code>size</code>), and chrome like
    <code>minH</code> on badges. Every child of a layout element is also born
    with <code>min-width: 0</code> — the browser default where an item refuses
    to shrink below its content is corrected at the language level, so
    <code>style.truncate()</code> works the first time.
  </p>

  <p sx={style.mt(4).fontSize(sm).fg(muted-foreground)}>
    For the full surface: <a href="/sail/reference/layout">reference → Layout keywords</a>.
  </p>

  <row sx={style.mt(8).padT(4).borderTWidth(1).borderT(border)} justify="between" gap="2">
    <a variant="default" href="/sail/language/styling">← Styling</a>
    <a variant="solid" tone="primary" href="/sail/language/motion">Motion →</a>
  </row>
</column>
