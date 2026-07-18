<!--
  /sail/language/layout — chapter 5, round 2 per owner review: lead with WHY
  one mechanism (many ways exist; one spelling is what tools and agents can
  reason about), render EVERY element live, explain in sentences not
  keyword-lists, no "flow not clamp" jargon, no sx-refusal opener (that
  lives on the styling page), no whole-vocabulary overclaim.
-->

<controller>
import { controller, type ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { code } from './snippets'

export default controller((_ctx: ControllerScope<Scope>) => {
  return { code }
})
</controller>

<column sx="max-w-breakpoint-tablet mx-auto" gap="5">
  <p sx="font-mono text-xs uppercase tracking-wide text-accent mb-2">The language</p>
  <h1>Layout</h1>
  <p sx="text-lg text-muted-foreground leading-relaxed">
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
  <row gap="3" align="center" justify="between" sx="p-4 bg-surface-1 border border-border rounded-md">
    <strong>Ada Lovelace</strong>
    <badge tone="success">Active</badge>
  </row>
  <code-block code={code.row}></code-block>
  <p>
    When a row runs out of width, its children move to the next line —
    rows <strong>wrap by default</strong>, like text does. Watch this one
    as the window narrows:
  </p>
  <row gap="2" sx="p-4 bg-surface-1 border border-border rounded-md">
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
  <column gap="2" align="start" sx="p-4 bg-surface-1 border border-border rounded-md">
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
    <card sx="p-4"><strong>One</strong></card>
    <card sx="p-4"><strong>Two</strong></card>
    <card sx="p-4"><strong>Three</strong></card>
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
    <card sx="p-3" data-col-span="2">wide — spans 2</card>
    <card sx="p-3">one</card>
    <card sx="p-3">two</card>
  </grid>
  <code-block code={code.spans}></code-block>

  <h2><code>&lt;stack&gt;</code></h2>
  <p>
    A stack places children on top of each other in the same cell — the
    overlay cases: a badge on an avatar, a caption on an image, a dot on an
    icon.
  </p>
  <row sx="p-4 bg-surface-1 border border-border rounded-md">
    <stack align="start" justify="end">
      <card sx="w-24 h-12 bg-muted rounded-md"></card>
      <div sx="w-4 h-4 rounded-full bg-primary"></div>
    </stack>
  </row>
  <code-block code={code.stack}></code-block>

  <h2><code>&lt;center&gt;</code></h2>
  <p>
    Centers its child, horizontally and vertically, in whatever space the
    element has:
  </p>
  <center sx="min-h-11 p-4 bg-surface-1 border border-border rounded-md">
    <badge tone="primary">centered</badge>
  </center>
  <code-block code={code.center}></code-block>

  <h2>How children size themselves</h2>
  <p>
    The parent decides how children sit; a child decides only how it fills
    the space it was given, with <code>sx</code>: <code>grow-1</code>
    claims a share of the leftover space, <code>basis-*</code> sets a
    starting size, and width/height atoms fix one. Every child of a layout
    element is also born with <code>min-width: 0</code> — the browser
    default where an item refuses to shrink below its content is the reason
    truncation "doesn't work" in most flex codebases, and it is corrected
    here at the language level, so <code>truncate</code> works the first
    time.
  </p>

  <row sx="mt-8 pt-4 border-t border-t-border" justify="between" gap="2">
    <a variant="default" href="/sail/language/styling">← Styling</a>
    <a variant="solid" tone="primary" href="/sail/language/motion">Motion →</a>
  </row>
</column>
