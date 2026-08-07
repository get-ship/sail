<!--
  <sail-component-viewer> — the shared component-viewing surface: a live stage
  holding the REAL component (slot="stage"), variant chips that drive a real
  attribute on it, and its generated prop table. Every surface that shows a
  component composes this one — docs, studio, devtools, eval.

    <sail-component-viewer name="badge" variant-attr="variant"
                          variants={['solid','soft','outline']}
                          props={PROP_REGISTRY.badge}
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
        // Skip fake/label-only attrs (e.g. dialog "open as") — invalid names
        // crash setAttribute and take the whole tab with them.
        if (!attr || !val || !/^[A-Za-z_:][\w:.-]*$/.test(attr)) return
        // No-op when already applied — setAttribute storms re-enter effects /
        // MutationObservers and freeze the tab.
        for (const el of host.querySelectorAll('[slot="stage"]')) {
          if (el.getAttribute(attr) !== val) el.setAttribute(attr, val)
        }
      }
      const onClick = (e: Event): void => {
        const btn = (e.target as HTMLElement).closest('[data-variant]')
        if (!btn) return
        const v = btn.getAttribute('data-variant') ?? ''
        active.set(v)
        apply()
        ;(host as any).onVariantChange?.({ variant: v })
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

<column gap="4" style({ box: w(full) })>
  <row gap="3" align="center">
    <h2 style({ text: fontFamily(mono).tracking(tight) })>{name()}</h2>
    <if {pack() !== ''}>
      <badge tone="primary" variant="soft">{pack()}</badge>
    </if>
    <if {!noA11yChip()}>
      <badge tone="muted" variant="outline">a11y ✓ · keyboard ✓</badge>
    </if>
  </row>
  <if {blurb() !== ''}>
    <p style({ paint: fg(muted-foreground) })>{blurb()}</p>
  </if>

  <column gap="4" style({ box: pad(5), paint: radius(lg).borderWidth(1).border(border).bg(surface-0) })>
    <row gap="2" align="center" justify="between">
      <row gap="2" align="center">
        <if {hasVariants()}>
          <span style({ paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs) })>{variantAttr()}:</span>
          <for {chips() as c} key={c.v}>
            <button size="sm" tone="primary" variant={c.variant} data-variant={c.v}>{c.v}</button>
          </for>
        </if>
      </row>
      <row gap="2" align="center">
        <status-dot tone="success"></status-dot>
        <span style({ paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs) })>live</span>
      </row>
    </row>

    <center data-stage style({  box: grow(1).pad(6).minH(9)  })>
      <slot name="stage"></slot>
    </center>
  </column>

  <if {hasUsage()}>
    <column gap="2">
      <p style({ paint: fg(muted-foreground), text: fontSize(xs).textCase(uppercase).tracking(wide) })>Usage — a complete program</p>
      <code-block code={usage()}></code-block>
    </column>
  </if>

  <if {hasProps()}>
    <card style({ box: pad(4) })>
      <column gap="3">
        <p style({ paint: fg(muted-foreground), text: fontSize(xs).textCase(uppercase).tracking(wide) })>Props</p>
        <for {propRows() as r} key={r.name}>
          <column gap="1">
            <row gap="3" align="baseline">
              <code>{r.name}</code>
              <span style({ box: grow(1), paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs) })>{r.type}</span>
            </row>
            <if {r.desc !== ''}>
              <p style({ paint: fg(muted-foreground), text: fontSize(sm) })>{r.desc}</p>
            </if>
          </column>
        </for>
      </column>
    </card>
  </if>

  <slot></slot>
</column>
