<!--
  /reference — the COMPLETE language surface, rendered from
  src/language-surface.gen.ts + src/prop-registry.gen.ts (emitted by
  `sail-compiler prop-registry` from the same tables the lint/compiler
  enforce). Nothing on this page is hand-listed — sx, motion, elements,
  actions, template blocks, and the controller ambient vocabulary all
  render from generated data.
-->
<controller>
import { controller, type ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { PROP_REGISTRY } from '../../src/prop-registry.gen'
import {
  SX_ATOMS,
  SX_STATE_PREFIXES,
  SX_CONDITION_PREFIXES,
  SX_REFUSALS,
  LAYOUT_KEYWORDS,
  MOTION,
  ELEMENTS,
  ACTIONS,
  CONTROLLER_AMBIENT,
  BLOCKS,
} from '../../src/language-surface.gen'

interface ElementRow {
  tag: string
  href: string
  propCount: number
  description: string
}

function elementRows(docs: { tag: string; description: string }[]): ElementRow[] {
  return docs.map((d) => ({
    tag: d.tag,
    href: `/components/${d.tag}`,
    propCount: PROP_REGISTRY[d.tag]?.length ?? 0,
    description: d.description,
  }))
}

const STDLIB_ELEMENTS = elementRows(ELEMENTS.stdlib)
const RESKIN_ELEMENTS = elementRows(ELEMENTS.reskins)

// A few atoms register under the same `prop` name in more than one table
// (e.g. a spacing sense and a color sense both named `border`) — merge
// their value lists under one card instead of letting the later
// registration silently shadow the earlier one in a keyed {#for}.
const SX_BY_PROP = new Map<string, string[]>()
for (const a of SX_ATOMS) {
  const existing = SX_BY_PROP.get(a.prop)
  if (existing) {
    existing.push(...a.values)
  } else {
    SX_BY_PROP.set(a.prop, [...a.values])
  }
}
const SX_ROWS = [...SX_BY_PROP.entries()].map(([prop, values]) => ({
  prop,
  values: values.join(' · '),
}))
const LAYOUT_ROWS = LAYOUT_KEYWORDS.map((k) => ({ tag: k.tag, props: k.props.join(', ') }))

const BLOCK_ROWS = BLOCKS.map((b) => ({
  open: b.open,
  clauses: b.clauses.length > 0 ? b.clauses.join(' · ') : '—',
  summary: b.summary,
}))

// The compiler-only lowering target is never authored — exclude it from the
// vocabulary a controller body is taught to write.
const CONTROLLER_ROWS = CONTROLLER_AMBIENT.filter((a) => a.name !== '__invalidate')

export default controller((_ctx: ControllerScope<Scope>) => ({
  stdlibElements: () => STDLIB_ELEMENTS,
  stdlibCount: () => STDLIB_ELEMENTS.length,
  reskinElements: () => RESKIN_ELEMENTS,
  reskinCount: () => RESKIN_ELEMENTS.length,
  sxRows: () => SX_ROWS,
  sxCount: () => SX_ATOMS.length,
  statePrefixes: () => SX_STATE_PREFIXES,
  conditionPrefixes: () => SX_CONDITION_PREFIXES,
  refusals: () => SX_REFUSALS,
  refusalCount: () => SX_REFUSALS.length,
  layoutRows: () => LAYOUT_ROWS,
  motionTriggers: () => MOTION.triggers.join(' · '),
  motionProperties: () => MOTION.properties.join(' · '),
  motionStagger: () => MOTION.staggerKey,
  motionPresets: () => MOTION.presets,
  actions: () => ACTIONS,
  actionCount: () => ACTIONS.length,
  controllerRows: () => CONTROLLER_ROWS,
  blockRows: () => BLOCK_ROWS,
}))
</controller>

<column gap="6" sx="min-w-0">
  <column sx="max-w-breakpoint-tablet" gap="5">
    <p role="doc-eyebrow">Reference</p>
    <h1>The language surface</h1>
    <p role="doc-lead">
      Everything below is generated from the compiler's own tables — the
      vocabulary the lint enforces is the vocabulary this page documents, by
      construction. Regenerate with
      <code>cargo run -p sail-compiler -- prop-registry</code>.
    </p>
  </column>

  <column gap="3">
    <h2>Elements — {stdlibCount()} stdlib + {reskinCount()} reskins</h2>
    <p>Every standard element, with its one-line description and its full prop table on its detail page.</p>
    <grid cols="3" gap="2">
      {#for e of stdlibElements() key={e.tag}}
        <card padding="sm" sx="min-w-0">
          <column gap="1">
            <a href={e.href}><code>{e.tag}</code> <small sx="text-muted-foreground">{e.propCount}</small></a>
            <small sx="text-muted-foreground max-w-panel-max-w-sm">{e.description}</small>
          </column>
        </card>
      {/for}
    </grid>

    <h3>Native reskins</h3>
    <p>Bare HTML tags the framework styles — the tag is still the API.</p>
    <grid cols="3" gap="2">
      {#for e of reskinElements() key={e.tag}}
        <card padding="sm" sx="min-w-0">
          <column gap="1">
            <a href={e.href}><code>{e.tag}</code></a>
            <small sx="text-muted-foreground max-w-panel-max-w-sm">{e.description}</small>
          </column>
        </card>
      {/for}
    </grid>
  </column>

  <column gap="3">
    <h2>Layout keywords</h2>
    <p>Structure is elements, not CSS. Each keyword takes a closed set of axis props; anything else is a compile error.</p>
    <grid cols="3" gap="2">
      {#for k of layoutRows() key={k.tag}}
        <card padding="sm" sx="min-w-0">
          <column gap="1">
            <code>&lt;{k.tag}&gt;</code>
            <small sx="text-muted-foreground max-w-panel-max-w-sm">{k.props}</small>
          </column>
        </card>
      {/for}
    </grid>
  </column>

  <column gap="3">
    <h2>sx — {sxCount()} atoms</h2>
    <p sx="max-w-breakpoint-tablet">
      <code>sx="name-value name-value …"</code>, static only. Every value is a
      token from a closed list — the rows below are the complete surface, and
      an off-list value is a compile error that names this same list.
    </p>
    <grid cols="2" gap="2">
      {#for a of sxRows() key={a.prop}}
        <card padding="sm" sx="min-w-0">
          <column gap="1">
            <code>{a.prop}:</code>
            <small sx="text-muted-foreground max-w-panel-max-w-sm">{a.values}</small>
          </column>
        </card>
      {/for}
    </grid>

    <h3>Not sx — use this ({refusalCount()})</h3>
    <p sx="max-w-breakpoint-tablet">
      The refused table is the OTHER half of the vocabulary. Every entry below
      is an atom a Tailwind prior will reach for and sx refuses by name; each
      names the element or attribute to use instead. The mission is never
      "find another way" — it is "use this."
    </p>
    <grid cols="2" gap="2">
      {#for r of refusals() key={r.atom}}
        <card padding="sm" sx="min-w-0">
          <column gap="1">
            <code sx="text-destructive">sx={r.atom}</code>
            <small sx="text-muted-foreground max-w-panel-max-w-sm">→ {r.instead}</small>
          </column>
        </card>
      {/for}
    </grid>

    <h3>State prefixes</h3>
    <grid cols="3" gap="2">
      {#for s of statePrefixes() key={s.prefix}}
        <row gap="3" sx="px-3 py-1 rounded-md" align="center">
          <code sx="min-w-9">{s.prefix}:</code>
          <small sx="text-muted-foreground font-mono max-w-ch-24">{s.selector}</small>
        </row>
      {/for}
    </grid>
    <p sx="max-w-breakpoint-tablet">
      Ancestor propagation: mark the ancestor with bare <code>group</code>,
      then <code>group-hover:</code> (any state prefix) on the descendant.
    </p>

    <h3>Condition prefixes</h3>
    <grid cols="3" gap="2">
      {#for c of conditionPrefixes() key={c.prefix}}
        <row gap="3" sx="px-3 py-1 rounded-md" align="center">
          <code sx="min-w-9">{c.prefix}:</code>
          <small sx="text-muted-foreground font-mono max-w-ch-24">{c.media}</small>
        </row>
      {/for}
    </grid>
  </column>

  <column gap="3">
    <h2>Motion — one system</h2>
    <p sx="max-w-breakpoint-tablet">
      <code>motion:&lt;trigger&gt;</code> is the only animation directive;
      <code>transition:</code> / <code>in:</code> / <code>out:</code> are retired.
    </p>
    <row gap="3" sx="px-3 py-2 bg-surface-1 rounded-md"><code sx="min-w-9">triggers</code><small sx="text-muted-foreground max-w-panel-max-w-sm">{motionTriggers()}</small></row>
    <row gap="3" sx="px-3 py-2 bg-surface-1 rounded-md"><code sx="min-w-9">properties</code><small sx="text-muted-foreground max-w-panel-max-w-sm">{motionProperties()} — plus {motionStagger()}: a duration step between siblings</small></row>

    <h3>Presets</h3>
    <grid cols="3" gap="2">
      {#for p of motionPresets() key={p.name}}
        <card padding="sm" sx="min-w-0">
          <column gap="1">
            <code>{p.name}</code>
            <small sx="text-muted-foreground max-w-panel-max-w-sm">({p.kind}) {p.doc}</small>
          </column>
        </card>
      {/for}
    </grid>
  </column>

  <column gap="3">
    <h2>Behavior — {actionCount()} actions</h2>
    <p sx="max-w-breakpoint-tablet">
      <code>use:action</code> attaches behavior bound to an element — the
      built-in <code>use:</code> registry, sorted, param + one-line summary.
    </p>
    <grid cols="3" gap="2">
      {#for a of actions() key={a.name}}
        <card padding="sm" sx="min-w-0">
          <column gap="1">
            <row gap="2" align="center">
              <code>use:{a.name}</code>
              <small sx="text-muted-foreground">{a.param}</small>
            </row>
            <small sx="text-muted-foreground max-w-panel-max-w-sm">{a.summary}</small>
          </column>
        </card>
      {/for}
    </grid>
  </column>

  <column gap="3">
    <h2>Controller — the ambient vocabulary</h2>
    <p sx="max-w-breakpoint-tablet">
      A <code>&lt;controller&gt;</code> block may reference these without
      importing them — the compiler injects the import for exactly the
      idents referenced.
    </p>
    <grid cols="3" gap="2">
      {#for c of controllerRows() key={c.name}}
        <card padding="sm" sx="min-w-0">
          <column gap="1">
            <code>{c.name}</code>
            <small sx="text-muted-foreground max-w-panel-max-w-sm">{c.doc}</small>
          </column>
        </card>
      {/for}
    </grid>
  </column>

  <column gap="3">
    <h2>Template directives</h2>
    <p sx="max-w-breakpoint-tablet">
      <code>on:</code> events; modifiers |prevent |stop |self |once |debounce:ms |key:Name.
      <code>bind:</code> two-way sugar — bind:value, bind:checked, bind:this
      (element ref). <code>prop:</code> sets a JS property (identity values
      that are not attributes). <code>attr:</code> forces attribute semantics
      for a dynamic value. <code>(--x)</code> — scope owns props, the
      controller owns state: when <code>(--x)</code> matches a declared prop
      or a <code>setup()</code>-returned signal, the compiler registers a
      typed CSS <code>@property</code> and reflects it onto the host, live.
      There is no <code>style:</code> — it was the last hatch, and it is closed.
    </p>

    <h3>Blocks</h3>
    <grid cols="2" gap="2">
      {#for b of blockRows() key={b.open}}
        <card padding="sm" sx="min-w-0">
          <column gap="1">
            <row gap="2" align="center">
              <code>{b.open}</code>
              <small sx="text-muted-foreground">{b.clauses}</small>
            </row>
            <small sx="text-muted-foreground max-w-panel-max-w-sm">{b.summary}</small>
          </column>
        </card>
      {/for}
    </grid>
  </column>

  <row role="navigation">
    <a variant="default" href="/sail/templates">← Templates</a>
    <a variant="solid" href="/components">Component catalog →</a>
  </row>
</column>
