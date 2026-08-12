<!--
  <sail-component-viewer> — live stage (slot=stage), variant chips, prop table.
  Specimen is slotted; picking a variant writes the named attribute on stage kids.
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
    /** Prop rows — registry shape (`props` is a runtime context key → `rows`). */
    rows: PropRow[]
    /** A complete, compiling usage program. */
    usage = ''
    /** Hide the a11y assurance chip. */
    'no-a11y-chip' = false
  }
  style sectionLabel = {
    paint: fg(muted-foreground),
    text: fontSize(xs).textCase(uppercase).tracking(wide),
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
      const apply = (): void => {
        const attr = variantAttr()
        const val = current()
        // Skip label-only attrs (invalid names crash setAttribute).
        if (!attr || !val || !/^[A-Za-z_:][\w:.-]*$/.test(attr)) return
        for (const el of host.querySelectorAll('[slot="stage"]')) {
          if (hostGetAttr(el as HostBag, attr) !== val) hostSetAttr(el as HostBag, attr, val)
        }
      }
      const onClick = (e: DomEv): void => {
        const btn = (e.target as HostBag | null)?.closest?.('[data-variant]') as HostBag | null | undefined
        if (!btn) return
        const v = hostGetAttr(btn as HostBag, 'data-variant') ?? ''
        active.set(v)
        apply()
        ;(host).onVariantChange?.({ variant: v })
      }
      host.addEventListener('click', onClick)
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
        <badge size="sm" variant="soft" tone="success">live</badge>
      </row>
    </row>

    <!-- Full-width stage (not <center>) so configurators can split specimen | knobs. -->
    <div data-stage style({ box: w(full).minW(0).grow(1).pad(0).minH(64) })>
      <slot name="stage"></slot>
    </div>
  </column>

  <if {hasUsage()}>
    <column gap="2">
      <p style(sectionLabel)>Usage — a complete program</p>
      <code-block code={usage()}></code-block>
    </column>
  </if>

  <if {hasProps()}>
    <card style({ box: pad(4) })>
      <column gap="3">
        <p style(sectionLabel)>Props</p>
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
