<!--
  Ship UI landing — tags-only L3. Compose ticker / card / h1 / row /
  column. Marketing soup stays out of the route (ledger look residual).
-->

<scope>
  type MotionBarsProps = {}
  type BentoCardProps = {}
  type BentoReactivityProps = {
    ticks: number
  }
</scope>

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'

const WORDS = ['Batteries', 'Components', 'Styling', 'State', 'Animation', 'Themes']

export default controller((_ctx: ControllerScope<Scope>) => {
    const i = signal(0)
    interval(() => i.set((n: number) => (n + 1) % WORDS.length), 2600)

    const ticks = signal(0)
    interval(() => ticks.set((n: number) => n + 1), 1000)

    return {
      word: () => [WORDS[i()]],
      ticks,
    }
})
</controller>

<partial motion-bars props: MotionBarsProps>
  <landing-motion-bars></landing-motion-bars>
</partial>

<partial bento-animation props: BentoCardProps>
  <a href="/docs/concepts/animation">
    <card interactive>
      <column gap="2">
        <p>Animation</p>
        <strong>Motion as a language surface</strong>
        <p>Springs, tweens, and stagger — declared in markup, not choreographed in JS.</p>
        <motion-bars></motion-bars>
      </column>
    </card>
  </a>
</partial>

<partial bento-theming props: BentoCardProps>
  <a href="/docs/concepts/theming">
    <card interactive>
      <column gap="2">
        <p>Theming</p>
        <strong>Nine themes. One token graph.</strong>
        <p>Swap theme and every surface, ink, and control re-resolves.</p>
        <row gap="2">
          <badge tone="primary" size="sm">primary</badge>
          <badge tone="accent" size="sm">accent</badge>
          <badge tone="success" size="sm">success</badge>
          <badge tone="warning" size="sm">warning</badge>
          <badge tone="info" size="sm">info</badge>
          <badge tone="muted" size="sm">muted</badge>
        </row>
      </column>
    </card>
  </a>
</partial>

<partial bento-reactivity props: BentoReactivityProps>
  <a href="/docs/concepts/state">
    <card interactive>
      <column gap="2">
        <p>Reactivity</p>
        <strong>Signals you can inspect</strong>
        <p>Derived state and effects are first-class — no store glue layer.</p>
        <row gap="3" align="baseline">
          <strong>{ticks}</strong>
          <small>ticks · live signal</small>
        </row>
      </column>
    </card>
  </a>
</partial>

<partial bento-deck props: BentoCardProps>
  <a href="/docs/app">
    <card interactive>
      <column gap="2">
        <p>Deck</p>
        <strong>SSR, routes, and HMR</strong>
        <p>The app host is already wired — layouts, data, and hot reload included.</p>
        <column gap="2">
          <row gap="2" align="center"><span dot tone="success"></span><small>ssr · streaming</small></row>
          <row gap="2" align="center"><span dot tone="primary"></span><small>file-based routes</small></row>
          <row gap="2" align="center"><span dot tone="info"></span><small>hmr · sub-second</small></row>
        </column>
      </column>
    </card>
  </a>
</partial>

<partial bento-components props: BentoCardProps>
  <a href="/components">
    <card interactive>
      <column gap="2">
        <p>Components</p>
        <strong>48 primitives, plus packs</strong>
        <p>Forms, overlays, and feedback — themed and accessible by construction.</p>
        <row gap="2" align="center">
          <badge tone="primary" size="sm">48 L1</badge>
          <badge tone="muted" size="sm" variant="soft">charts</badge>
          <badge tone="muted" size="sm" variant="soft">chat</badge>
          <badge tone="muted" size="sm" variant="soft">editor</badge>
        </row>
      </column>
    </card>
  </a>
</partial>

<partial bento-ai props: BentoCardProps>
  <a href="/docs/ai">
    <card interactive>
      <column gap="2">
        <p>AI</p>
        <strong>Agents stay inside the language</strong>
        <p>The compiler refuses invalid UI — so generated markup cannot drift the system.</p>
        <card padding="sm">
          <p><strong>$</strong> sail check app/</p>
          <p>ok · 0 refusals · 214 modules</p>
        </card>
      </column>
    </card>
  </a>
</partial>

<column gap="6">
  <column gap="5" align="center">
    <p>SHIP UI · 0.2.0</p>
    <h1>
      <for {word() as w} key={w}>
        <b>{w}</b>
      </for>
      included. All of them.
    </h1>
    <p>
      One language with the whole interface inside it — built for agents to
      write and humans to review, with no way to drift off the design system.
    </p>
    <row gap="4" justify="center" align="center">
      <a variant="solid" tone="primary" href="/docs/guide/getting-started">Get started</a>
      <a variant="outline" href="/docs/guide/why">Why Sail</a>
    </row>
  </column>

  <ticker duration="slow" reverse>
    <p>forms + validation · live data · springs + tweens · ssr · hot reload · design tokens</p>
  </ticker>
  <ticker>
    <p>NO RAW CSS ◆ SIGNALS BUILT IN ◆ 48 COMPONENTS ◆ 9 THEMES ◆ A11Y BY CONSTRUCTION</p>
  </ticker>

  <when {tablet}>
    <grid cols="3" gap="4">
      <bento-animation></bento-animation>
      <bento-theming></bento-theming>
      <bento-reactivity ticks={ticks()}></bento-reactivity>
      <bento-deck></bento-deck>
      <bento-components></bento-components>
      <bento-ai></bento-ai>
    </grid>
  {else => <grid cols="1" gap="3">
      <bento-animation></bento-animation>
      <bento-theming></bento-theming>
      <bento-reactivity ticks={ticks()}></bento-reactivity>
      <bento-deck></bento-deck>
      <bento-components></bento-components>
      <bento-ai></bento-ai>
    </grid>}</when>

  <row justify="between" align="center" gap="3">
    <row gap="6" align="center">
      <a href="/docs/guide/introduction">Docs</a>
      <a href="https://github.com/get-ship/sail">GitHub</a>
    </row>
    <span>SHIP UI · 0.2.0</span>
  </row>

  <h1>SHIP</h1>
</column>
