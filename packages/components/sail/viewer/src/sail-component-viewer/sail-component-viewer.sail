<!--
  <sail-component-viewer> — the shared component-viewing surface: a live stage
  holding the REAL component (slot="stage"), variant chips that drive a real
  attribute on it, and its generated prop table. Every surface that shows a
  component composes this one — docs, studio, devtools, eval.

    <sail-component-viewer name="badge" variant-attr="variant"
                          prop:variants={['solid','soft','outline']}
                          prop:props={PROP_REGISTRY.badge}
                          usage={snippet}>
      <badge slot="stage" variant="solid" tone="primary">Live</badge>
    </sail-component-viewer>

  The specimen is slotted, so the viewer never mounts a fake: the host renders
  the component it actually ships. Picking a variant writes the named attribute
  onto every slotted element and emits sail-variant-change.
-->

<scope>
  props: {
    /** Component name — the heading. */
    name = ''
    /** Short blurb under the heading. */
    blurb = ''
    /** Pack label chip (e.g. "L1 · language element"). */
    pack = ''
    /** Which attribute the variant chips drive (empty = no chips). */
    'variant-attr' = ''
    /** Variant values. */
    variants: string[]
    /** Prop rows — the generated registry's shape. (`props` is the runtime's
        own context key, so the rows land on `rows`.) */
    rows: PropRow[]
    /** A complete, compiling usage program. */
    usage = ''
    /** Hide the a11y assurance chip (fx actions and the like). */
    'no-a11y-chip' = false
  }
</scope>

<controller>
import { controller, derived, effect, signal } from '@ship/sail-runtime'
import type { Scope } from './sail-component-viewer.sail'

export interface PropRow {
  name: string
  type: string
  desc: string
}

export default controller(({ variants, rows, usage, variantAttr }: Scope) => {
    const active = signal('')
    const variantList = derived(() => variants() ?? [])
    const hasVariants = derived(() => (variantAttr() ?? '') !== '' && variantList().length > 0)
    const current = derived(() => active() || variantList()[0] || '')
    const propRows = derived(() => rows() ?? [])
    const hasProps = derived(() => propRows().length > 0)
    const hasUsage = derived(() => (usage() ?? '') !== '')
    const chips = derived(() =>
      variantList().map((v) => ({ v, variant: v === current() ? 'solid' : 'ghost' })),
    )
    mount((host) => {
      // Picking a variant writes the real attribute onto the slotted specimen —
      // the viewer drives the component, it does not simulate it.
      const apply = (): void => {
        const attr = variantAttr()
        const val = current()
        if (!attr || !val) return
        for (const el of host.querySelectorAll('[slot="stage"]')) el.setAttribute(attr, val)
      }
      const onClick = (e: Event): void => {
        const btn = (e.target as HTMLElement).closest('[data-variant]')
        if (!btn) return
        const v = btn.getAttribute('data-variant') ?? ''
        active.set(v)
        apply()
        host.dispatchEvent(
          new CustomEvent('sail-variant-change', {
            detail: { variant: v },
            bubbles: true,
            composed: true,
          }),
        )
      }
      host.addEventListener('click', onClick)
      // The specimen tracks the selection: identity props (variants) may land
      // after mount, and a pick must reach the slotted element either way.
      const stop = effect(apply)
      return () => {
        stop()
        host.removeEventListener('click', onClick)
      }
    })

    return { active, current, chips, hasVariants, propRows, hasProps, hasUsage }
  })
</controller>

<column gap="4" sx="w-full">
  <row gap="3" align="center">
    <h2 sx="font-mono tracking-tight">{name()}</h2>
    {#if pack() !== ''}
      <badge tone="primary" variant="soft">{pack()}</badge>
    {/if}
    {#if !noA11yChip()}
      <badge tone="muted" variant="outline">a11y ✓ · keyboard ✓</badge>
    {/if}
  </row>
  {#if blurb() !== ''}
    <p sx="text-muted-foreground">{blurb()}</p>
  {/if}

  <column gap="4" sx="p-5 rounded-lg border border-border bg-surface-0">
    <row gap="2" align="center" justify="between">
      <row gap="2" align="center">
        {#if hasVariants()}
          <span sx="font-mono text-xs text-muted-foreground">{variantAttr()}:</span>
          {#for c of chips() key={c.v}}
            <button size="sm" tone="primary" variant={c.variant} data-variant={c.v}>{c.v}</button>
          {/for}
        {/if}
      </row>
      <row gap="2" align="center">
        <status-dot tone="success"></status-dot>
        <span sx="font-mono text-xs text-muted-foreground">live</span>
      </row>
    </row>

    <center data-stage sx="p-6 min-h-9 grow-1">
      <slot name="stage"></slot>
    </center>
  </column>

  {#if hasUsage()}
    <column gap="2">
      <p sx="text-xs text-muted-foreground uppercase tracking-wide">Usage — a complete program</p>
      <code-block code={usage()}></code-block>
    </column>
  {/if}

  {#if hasProps()}
    <card sx="p-4">
      <column gap="3">
        <p sx="text-xs text-muted-foreground uppercase tracking-wide">Props</p>
        {#for r of propRows() key={r.name}}
          <column gap="1">
            <row gap="3" align="baseline">
              <code>{r.name}</code>
              <span sx="font-mono text-xs text-muted-foreground grow-1">{r.type}</span>
            </row>
            {#if r.desc !== ''}
              <p sx="text-sm text-muted-foreground">{r.desc}</p>
            {/if}
          </column>
        {/for}
      </column>
    </card>
  {/if}

  <slot></slot>
</column>
