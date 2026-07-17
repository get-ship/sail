// Prompt construction for the eval rig. The model receives:
//   1) a system primer describing .sui syntax + the SDK type sketch
//   2) the component catalog (so it knows which <sui-*> tags exist)
//   3) few-shot examples (static view, stateful counter, data + controller)
//
// The output contract is strict: exactly one fenced ```sui``` block — the
// complete single-file view, inline <controller> included when needed.

import type { PrimitiveCatalog, PrimitiveDef } from './catalog'

export interface PromptInputs {
  description: string
  catalog: PrimitiveCatalog
  fixtures: { template: string; setup?: string }
}

export interface PromptParts {
  system: string
  user: string
}

const SYSTEM_PRIMER = `You author SUI views — single .sui files: HTML plus top-level directives, mustache control flow, and an optional inline <controller> block of TypeScript.

Top-level directives:
  <scope> ... </scope>                               — component files only: typed props/variants, literal values.
  <controller> ... </controller>                     — the view's TypeScript: state, handlers, lifecycle.

Template grammar:
  {expr}                                — text-position expression.
  attr={expr}                           — dynamic attribute.
  on:event={(e) => ...}                 — event listener.
  bind:value={signalName}               — two-way binding (input-like elements only).
  {#if expr} ... {:else} ... {/if}      — conditional.
  {#for item of expr key={item.id}} ... {/for} — keyed list; always write key=.
  {#match expr} {:case 'a'} ... {:default} ... {/match} — enumerated branches.

Reactive values are read by calling them: \`count()\` reads, \`count.set(n)\` writes.

State and logic live in the inline <controller> block. It default-exports controller({ setup, on, mount, unmount }); whatever setup RETURNS is the template's scope:

  <controller>
  import { controller, signal } from '@ship/sui-runtime'

  export default controller({
    setup: () => ({ count: signal(0) }),
    on:    { incr: (s) => { s.count++ } },  // handlers are the only section that mutates
    mount: (s, el) => { /* once after render */ },
  })
  </controller>

Compositions (@ship/sui-std) are plain functions that return signals — form(), selection(), filteredList(). Call them inside setup and return their signals as scope members.

Layout is elements, not CSS: <row>, <column>, <grid>, <center>, <stack> with gap/align/justify attributes. Styling is the static sx attribute with token values (e.g. sx="p-4 text-muted-foreground") — never raw px or hex.

Hard rules — violating any of these fails the eval:
  - No raw <script> tags. No eval. No new Function(). No document.write.
  - No <style> blocks and no style= attributes — spend tokens through sx and layout elements.
  - No customElements.define — that's component territory, not view territory.
  - Use <sui-*> components from the catalog rather than reinventing buttons or inputs.
  - Mutation lives only in \`on.*\` handlers and setup-returned functions. Never \`s.x = v\` in templates.
  - No network calls of any kind — no fetch, no ship.* — views are pure; data arrives through the host.
  - Output exactly one fenced \`\`\`sui\`\`\` block — the complete file, inline <controller> included when needed. No prose outside the block.`

const FEW_SHOT_STATIC = {
  description: 'A static info card listing the three Ship core values.',
  template: `<column gap="3">
  <h2>Ship core values</h2>
  <ul>
    <li>Boring infra, novel surface.</li>
    <li>Server-rendered first.</li>
    <li>Agent-writable by construction.</li>
  </ul>
</column>`,
}

const FEW_SHOT_COUNTER = {
  description: 'A counter view: shows a number and two buttons that increment and reset it.',
  template: `<controller>
import { controller, signal } from '@ship/sui-runtime'

export default controller({
  setup: () => ({ count: signal(0) }),
})
</controller>

<row gap="2" align="center">
  <p>count: <span data-test="value">{count()}</span></p>
  <sui-button data-test="inc" on:click={() => count.set(count() + 1)}>increment</sui-button>
  <sui-button data-test="reset" tone="muted" on:click={() => count.set(0)}>reset</sui-button>
</row>`,
}

const FEW_SHOT_CONTROLLER = {
  description: 'A filterable agent list: the inline controller owns the list, a query signal, and a derived filtered view; the template wires them.',
  template: `<controller>
import { controller, signal, derived } from '@ship/sui-runtime'

export default controller({
  setup: () => {
    const query = signal('')
    const all = signal([
      { id: 'a1', name: 'triage' },
      { id: 'a2', name: 'deploy' },
      { id: 'a3', name: 'review' },
    ])
    const filtered = derived(() =>
      all().filter((a) => a.name.includes(query())))
    return { query, filtered, total: derived(() => filtered().length) }
  },
})
</controller>

<column gap="2">
  <input data-test="q" placeholder="filter" bind:value={query} />
  <p>{total()} agent{total() === 1 ? '' : 's'}</p>
  <ul>
    {#for a of filtered() key={a.id}}
      <li><sui-button data-test="row">{a.name}</sui-button></li>
    {/for}
  </ul>
</column>`,
}

/** Render a primitive catalog as a compact, model-readable summary. */
export function renderCatalog(catalog: PrimitiveCatalog): string {
  return catalog.primitives.map(renderPrimitive).join('\n\n')
}

function renderPrimitive(p: PrimitiveDef): string {
  const lines: string[] = [`<${p.tag}>${p.summary ? ' — ' + p.summary : ''}`]
  if (p.props.length) {
    lines.push('  props:')
    for (const prop of p.props) {
      const t =
        prop.type === 'union' && prop.values
          ? prop.values.join('|')
          : prop.type
      const def = prop.default !== undefined ? ` default=${JSON.stringify(prop.default)}` : ''
      const bind = prop.bindable ? ' bindable' : ''
      lines.push(`    - ${prop.name}: ${t}${def}${bind}`)
    }
  }
  if (p.slots.length) {
    lines.push('  slots:')
    for (const slot of p.slots) {
      lines.push(`    - ${slot.name}${slot.summary ? ': ' + slot.summary : ''}`)
    }
  }
  if (p.events.length) {
    lines.push('  events:')
    for (const ev of p.events) {
      lines.push(`    - ${ev.name}${ev.summary ? ': ' + ev.summary : ''}`)
    }
  }
  return lines.join('\n')
}

function renderFewShot(ex: {
  description: string
  template: string
  setup?: string
}): string {
  const out = [`Example feature: ${ex.description}`, '', 'Example output:', '']
  out.push('```sui', ex.template, '```')
  if (ex.setup) out.push('', '```ts', ex.setup, '```')
  return out.join('\n')
}

/** Assemble the system + user messages for one task. */
export function buildPrompt(input: PromptInputs): PromptParts {
  const catalogBlock = renderCatalog(input.catalog)
  const fewShot = [
    renderFewShot(FEW_SHOT_STATIC),
    renderFewShot(FEW_SHOT_COUNTER),
    renderFewShot(FEW_SHOT_CONTROLLER),
  ].join(
    '\n\n---\n\n',
  )

  const fixtureBlock =
    input.fixtures.template.trim() || input.fixtures.setup?.trim()
      ? `\n\nStarter fixture (extend or replace as needed):\n\n\`\`\`sui\n${input.fixtures.template}\n\`\`\`${
          input.fixtures.setup
            ? `\n\n\`\`\`ts\n${input.fixtures.setup}\n\`\`\``
            : ''
        }`
      : ''

  const user = `Primitive catalog (${input.catalog.version}):

${catalogBlock}

---

${fewShot}

---

Feature to implement:

${input.description.trim()}${fixtureBlock}

Reply with exactly one \`\`\`sui\`\`\` block — the complete file, inline <controller> included when needed. No prose.`

  return { system: SYSTEM_PRIMER, user }
}

/** Pull the single .sui block out of a model reply. */
export function parseModelReply(raw: string): { template: string; setup?: string } {
  const sui = matchFence(raw, 'sui')
  if (!sui) {
    throw new Error('model reply did not contain a ```sui``` block')
  }
  return { template: sui }
}

function matchFence(raw: string, tag: string): string | undefined {
  // Match a fenced block whose info-string starts with `tag`. We
  // tolerate trailing junk on the info line so ```html.set view``` works.
  const re = new RegExp(
    '```' + escapeRegex(tag) + '[^\\n]*\\n([\\s\\S]*?)\\n?```',
    'm',
  )
  const m = re.exec(raw)
  return m && m[1] !== undefined ? m[1] : undefined
}

function escapeRegex(s: string): string {
  return s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
}
