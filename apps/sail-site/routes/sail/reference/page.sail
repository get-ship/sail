<!--
  /sail/reference — index of the complete language surface, split per API.
  Each linked page renders its own section from the generated modules
  (apps/sail-site/src/language-surface.gen.ts, prop-registry.gen.ts,
  runtime-surface.gen.ts) — nothing on any reference page is hand-listed.
-->
<controller>
import { controller, type ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { PROP_REGISTRY } from '../../../src/prop-registry.gen'
import {
  SX_ATOMS,
  LAYOUT_KEYWORDS,
  MOTION,
  ELEMENTS,
  ACTIONS,
  DIRECTIVES,
  SCOPE_GRAMMAR,
  CONFIG_KEYS,
} from '../../../src/language-surface.gen'
import { RUNTIME_EXPORTS } from '../../../src/runtime-surface.gen'

interface SectionCard {
  href: string
  title: string
  count: string
  description: string
}

const SECTIONS: SectionCard[] = [
  {
    href: '/sail/reference/runtime',
    title: 'Runtime',
    count: `${RUNTIME_EXPORTS.length} exports`,
    description: '@ship/sail-runtime\'s complete public surface — reactivity, stores, compositions, data, motion, timeline, theme.',
  },
  {
    href: '/sail/reference/template',
    title: 'Template',
    count: `${DIRECTIVES.length} directives`,
    description: 'Every attribute-position directive: on:, bind:, use:, prop:, attr:, motion:, slots, and the block vocabulary.',
  },
  {
    href: '/sail/reference/scope',
    title: 'Scope grammar',
    count: `${SCOPE_GRAMMAR.length} constructs`,
    description: 'Everything a <scope> block can declare — props forms, type aliases, variants.',
  },
  {
    href: '/sail/reference/sx',
    title: 'sx',
    count: `${SX_ATOMS.length} atoms`,
    description: 'The closed styling vocabulary — atoms, refusals, state/condition prefixes, custom-property reads.',
  },
  {
    href: '/sail/reference/layout',
    title: 'Layout keywords',
    count: `${LAYOUT_KEYWORDS.length} keywords`,
    description: 'Structure is elements, not CSS — the closed set of layout tags and their axis props.',
  },
  {
    href: '/sail/reference/motion',
    title: 'Motion',
    count: `${MOTION.presets.length} presets`,
    description: 'The one animation system — triggers, properties, presets, and the config value grammar.',
  },
  {
    href: '/sail/reference/actions',
    title: 'Actions',
    count: `${ACTIONS.length} actions`,
    description: 'The use: registry — built-in element behaviors, sorted, with params and summaries.',
  },
  {
    href: '/sail/reference/elements',
    title: 'Elements',
    count: `${ELEMENTS.stdlib.length + ELEMENTS.reskins.length} elements`,
    description: 'Every stdlib element and native reskin, with prop counts linking to full prop tables.',
  },
  {
    href: '/sail/reference/config',
    title: 'Config',
    count: `${CONFIG_KEYS.length} keys`,
    description: 'Every sail.config.json key, its type, its default, and its semantics.',
  },
]

export default controller((_ctx: ControllerScope<Scope>) => ({
  sections: () => SECTIONS,
  elementCount: () => ELEMENTS.stdlib.length + ELEMENTS.reskins.length,
  propRegistrySize: () => Object.keys(PROP_REGISTRY).length,
}))
</controller>

<column gap="6" sx="min-w-0">
  <column sx="max-w-breakpoint-tablet" gap="5">
    <p role="doc-eyebrow">Reference</p>
    <h1>The language surface</h1>
    <p role="doc-lead">
      Everything below is generated from the compiler's own tables — the
      vocabulary the lint enforces is the vocabulary these pages document,
      by construction. Regenerate with <code>bun run gen</code> (or
      <code>cargo run -p sail-compiler -- prop-registry</code> for the
      compiler tables alone).
    </p>
  </column>

  <grid cols="3" gap="3">
    {#for s of sections() key={s.href}}
      <a href={s.href}>
        <card padding="md" sx="min-w-0 h-full">
          <column gap="2">
            <row justify="between" align="center">
              <h3>{s.title}</h3>
              <small sx="text-muted-foreground">{s.count}</small>
            </row>
            <small sx="text-muted-foreground">{s.description}</small>
          </column>
        </card>
      </a>
    {/for}
  </grid>

  <row role="navigation">
    <a variant="default" href="/sail/language/templates">← Templates</a>
    <a variant="solid" href="/components">Component catalog →</a>
  </row>
</column>
