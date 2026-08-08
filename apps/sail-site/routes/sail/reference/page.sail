<!--
  /sail/reference — language surface index by API tier.
  Tier A: compiler tables. Tier B: author-surface.json → runtime-surface.gen.
-->
<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { PROP_REGISTRY } from '../../../src/compositions/prop-registry'
import {
  SX_ATOMS,
  LAYOUT_KEYWORDS,
  MOTION,
  ELEMENTS,
  ACTIONS,
  DIRECTIVES,
  SCOPE_GRAMMAR,
  CONFIG_KEYS,
  SEMANTIC_PROPS,
  AUTHORING_BANS,
  REGION_CAPABILITY_MATRIX,
} from '../../../src/compositions/language-surface'
import { AUTHOR_RUNTIME_COUNT, RUNTIME_CATEGORIES } from '../../../src/compositions/runtime-surface'

interface SectionCard {
  href: string
  title: string
  count: string
  description: string
  tier: string
}

const SECTIONS: SectionCard[] = [
  {
    href: '/sail/reference/runtime',
    title: 'Author runtime',
    count: `${AUTHOR_RUNTIME_COUNT} exports`,
    description: `Tier B — intentional @ship/sail-runtime imports (${RUNTIME_CATEGORIES.length} groups: reactivity, controller, data, motion, …). Not host/framework dumps.`,
    tier: 'B',
  },
  {
    href: '/sail/reference/template',
    title: 'Template',
    count: `${DIRECTIVES.length} directives`,
    description: 'Tier A — every attribute-position directive: on:, bind:, use:, Style motion (.enter/.exit/…), slots, and the block vocabulary.',
    tier: 'A',
  },
  {
    href: '/sail/reference/scope',
    title: 'Scope grammar',
    count: `${SCOPE_GRAMMAR.length} constructs`,
    description: 'Tier A — everything a <scope> block can declare — props forms, type aliases, variants.',
    tier: 'A',
  },
  {
    href: '/sail/reference/sx',
    title: 'Style',
    count: `${SX_ATOMS.length} scale entries`,
    description: 'Tier A — style({ box, paint, text, animate }), style name =, when/match/compound/bind; setter scales and refusals.',
    tier: 'A',
  },
  {
    href: '/sail/reference/layout',
    title: 'Layout keywords',
    count: `${LAYOUT_KEYWORDS.length} keywords`,
    description: 'Tier A — structure is elements, not CSS — the closed set of layout tags and their axis props.',
    tier: 'A',
  },
  {
    href: '/sail/reference/motion',
    title: 'Motion',
    count: `${MOTION.presets.length} presets`,
    description: 'Tier A — Style motion triggers, properties, presets (template). Scrub-lane timeline() is tier B on Author runtime.',
    tier: 'A',
  },
  {
    href: '/sail/reference/actions',
    title: 'Actions',
    count: `${ACTIONS.length} actions`,
    description: 'Tier A — the use: registry — built-in element behaviors, sorted, with params and summaries.',
    tier: 'A',
  },
  {
    href: '/sail/reference/elements',
    title: 'Elements',
    count: `${ELEMENTS.stdlib.length} elements`,
    description: 'Tier A — every stdlib L1 element with prop tables (native reskins retired).',
    tier: 'A',
  },
  {
    href: '/sail/reference/semantics',
    title: 'Semantics',
    count: `${SEMANTIC_PROPS.length} props · ${AUTHORING_BANS.length} bans`,
    description: 'Tier A — semantic a11y props, compiler-stamped host anatomy (selector contract for @ship/sail-test), and floor authoring bans (data-*, free aria, raw style/script).',
    tier: 'A',
  },
  {
    href: '/sail/reference/config',
    title: 'Config',
    count: `${CONFIG_KEYS.length} keys`,
    description: 'Tier A — every sail.config.json key, its type, its default, and its semantics.',
    tier: 'A',
  },
  {
    href: '/sail/reference/regions',
    title: 'Region matrix',
    count: `${REGION_CAPABILITY_MATRIX.length} cells`,
    description: 'Tier A — scope / controller / template / chain capability matrix + thin-controller size policy (CHANNELS-1 WI10).',
    tier: 'A',
  },
]

export default controller((_ctx: ControllerScope<Scope>) => ({
  sections: () => SECTIONS,
  elementCount: () => ELEMENTS.stdlib.length,
  propRegistrySize: () => Object.keys(PROP_REGISTRY).length,
  authorRuntimeCount: () => AUTHOR_RUNTIME_COUNT,
}))
</controller>

<scope>
  style root = { box: minW(0) }
  style intro = { box: maxW(breakpoint-tablet) }
  style cardShell = { box: minW(0).h(full) }
  style muted = { paint: fg(muted-foreground) }
</scope>

<column gap="6" style(root)>
  <column gap="5" style(intro)>
    <p token="doc-eyebrow">Reference</p>
    <h1>The language surface</h1>
    <p token="doc-lead">
      Three tiers: <strong>A</strong> template / ambient (compiler tables),
      <strong>B</strong> author runtime (intentional
      <code>@ship/sail-runtime</code> imports — {authorRuntimeCount()} symbols),
      <strong>C</strong> emit ABI and host entry points (<code>/internal</code>,
      <code>server</code>, <code>framework</code>, …) which never appear on these
      pages. Tier A regenerates with <code>cargo run -p sail-compiler -- prop-registry</code>;
      tier B with <code>node scripts/gen-runtime-surface.mjs</code> (both via
      <code>bun run gen</code> in sail-site). Allowlist:
      <code>packages/sail/runtime/author-surface.json</code>.
    </p>
  </column>

  <grid cols="3" gap="3">
    <for {sections() as s} key={s.href}>
      <a href={s.href}>
        <card padding="md" style(cardShell)>
          <column gap="2">
            <row justify="between" align="center">
              <h3>{s.title}</h3>
              <row gap="2" align="center">
                <badge size="sm">tier {s.tier}</badge>
                <small style(muted)>{s.count}</small>
              </row>
            </row>
            <small style(muted)>{s.description}</small>
          </column>
        </card>
      </a>
    </for>
  </grid>

  <row>
    <a variant="default" href="/sail/language/templates">← Templates</a>
    <a variant="solid" href="/components">Component catalog →</a>
  </row>
</column>
